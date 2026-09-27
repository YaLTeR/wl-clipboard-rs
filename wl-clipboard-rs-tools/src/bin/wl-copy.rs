use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::os::unix::ffi::OsStringExt;

use anyhow::Context;
use clap::Parser;
use libc::fork;
use rustix::stdio::{dup2_stdin, dup2_stdout};
use wl_clipboard_rs::copy::{
    self, clear, ClipboardType, MimeSource, MimeType, Seat, ServeRequests, Source,
};
use wl_clipboard_rs_tools::wl_copy::Options;

fn from_options(x: Options) -> wl_clipboard_rs::copy::Options {
    let mut opts = copy::Options::new();
    opts.serve_requests(if x.paste_once {
        ServeRequests::Only(1)
    } else {
        ServeRequests::Unlimited
    })
    .foreground(true) // We fork manually to support background mode.
    .clipboard(if x.primary {
        if x.regular {
            ClipboardType::Both
        } else {
            ClipboardType::Primary
        }
    } else {
        ClipboardType::Regular
    })
    .trim_newline(x.trim_newline)
    .sensitive(x.sensitive)
    .seat(x.seat.map(Seat::Specific).unwrap_or_default());
    opts
}

/// The `--offer` pairs as (MIME type, file), in the order given.
///
/// Fails for a MIME type that is not UTF-8 and for a second offer of the standard input.
fn offer_pairs(options: &Options) -> Result<Vec<(String, OsString)>, String> {
    let mut stdin_taken = false;
    let (pairs, _) = options.offer.as_chunks::<2>();
    pairs
        .iter()
        .map(|[mime_type, file]| {
            let mime_type = mime_type
                .to_str()
                .ok_or_else(|| format!("MIME type is not UTF-8: {mime_type:?}"))?
                .to_owned();
            if file == "-" {
                if stdin_taken {
                    return Err("only one --offer can read the standard input".to_owned());
                }
                stdin_taken = true;
            }
            Ok((mime_type, file.clone()))
        })
        .collect()
}

/// Reads each offered file before forking, so a file removed right after wl-copy returns is
/// still served.
fn offer_sources(offers: Vec<(String, OsString)>) -> Result<Vec<MimeSource>, anyhow::Error> {
    offers
        .into_iter()
        .map(|(mime_type, file)| {
            let source = if file == "-" {
                Source::StdIn
            } else {
                let data = fs::read(&file)
                    .with_context(|| format!("couldn't read {}", file.to_string_lossy()))?;
                Source::Bytes(data.into())
            };
            Ok(MimeSource {
                source,
                mime_type: MimeType::Specific(mime_type),
            })
        })
        .collect()
}

fn main() -> Result<(), anyhow::Error> {
    // Parse command-line options.
    let mut options = Options::parse();

    stderrlog::new()
        .verbosity(usize::from(options.verbose) + 1)
        .init()
        .unwrap();

    if options.clear {
        let clipboard = if options.primary {
            ClipboardType::Primary
        } else {
            ClipboardType::Regular
        };
        clear(
            clipboard,
            options.seat.map(Seat::Specific).unwrap_or_default(),
        )?;
        return Ok(());
    }

    let offers = offer_pairs(&options).map_err(anyhow::Error::msg)?;

    // Join arguments into a string to copy, or use stdin if no arguments.
    let source = match options.text.drain(..).reduce(|mut text, arg| {
        text.push(" ");
        text.push(arg);
        text
    }) {
        None => Source::StdIn,
        Some(text) => Source::Bytes(text.into_vec().into()),
    };

    let mime_type = if let Some(mime_type) = options.mime_type.take() {
        MimeType::Specific(mime_type)
    } else {
        MimeType::Autodetect
    };

    let foreground = options.foreground;
    let prepared_copy = if offers.is_empty() {
        from_options(options).prepare_copy(source, mime_type)?
    } else {
        from_options(options).prepare_copy_multi(offer_sources(offers)?)?
    };

    if foreground {
        prepared_copy.serve()?;
    } else {
        // SAFETY: We don't spawn any threads, so doing things after forking is safe.
        // TODO: is there any way to verify that we don't spawn any threads?
        match unsafe { fork() } {
            -1 => panic!("error forking: {:?}", std::io::Error::last_os_error()),
            0 => {
                // Replace STDIN and STDOUT with /dev/null. We won't be using them, and keeping
                // them as is hangs a potential pipeline (i.e. wl-copy hello | cat). Also, simply
                // closing the file descriptors is a bad idea because then they get reused by
                // subsequent temp file opens, which breaks the dup2/close logic during data
                // copying.
                if let Ok(dev_null) = OpenOptions::new().read(true).write(true).open("/dev/null") {
                    let _ = dup2_stdin(&dev_null);
                    let _ = dup2_stdout(&dev_null);
                }

                drop(prepared_copy.serve());
            }
            _ => (),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, clap::Error> {
        Options::try_parse_from(std::iter::once("wl-copy").chain(args.iter().copied()))
    }

    #[test]
    fn offers_keep_their_order() {
        let options = parse(&[
            "--offer",
            "text/uri-list",
            "uris",
            "--offer",
            "x-special/gnome-copied-files",
            "-",
        ])
        .unwrap();
        assert_eq!(
            offer_pairs(&options).unwrap(),
            vec![
                ("text/uri-list".to_owned(), OsString::from("uris")),
                (
                    "x-special/gnome-copied-files".to_owned(),
                    OsString::from("-")
                ),
            ]
        );
    }

    #[test]
    fn a_type_may_hold_an_equals_sign() {
        let options = parse(&["--offer", "text/plain;charset=utf-8", "a=b"]).unwrap();
        assert_eq!(
            offer_pairs(&options).unwrap(),
            vec![("text/plain;charset=utf-8".to_owned(), OsString::from("a=b"))]
        );
    }

    #[test]
    fn stdin_is_offered_once() {
        let options = parse(&["--offer", "text/plain", "-", "--offer", "text/html", "-"]).unwrap();
        assert!(offer_pairs(&options).is_err());
    }

    #[test]
    fn an_offer_needs_a_file() {
        assert!(parse(&["--offer", "text/plain"]).is_err());
    }

    #[test]
    fn offers_conflict_with_other_content() {
        assert!(parse(&["--offer", "text/plain", "f", "hello"]).is_err());
        assert!(parse(&["--offer", "text/plain", "f", "--type", "text/html"]).is_err());
        assert!(parse(&["--offer", "text/plain", "f", "--clear"]).is_err());
    }

    #[test]
    fn offers_combine_with_copy_options() {
        let options = parse(&[
            "--offer",
            "text/plain",
            "f",
            "--sensitive",
            "--paste-once",
            "--primary",
        ])
        .unwrap();
        assert!(options.sensitive && options.paste_once && options.primary);
    }

    #[test]
    fn a_missing_file_is_an_error() {
        let offers = vec![(
            "text/plain".to_owned(),
            OsString::from("/nonexistent/wl-copy"),
        )];
        assert!(offer_sources(offers).is_err());
    }
}
