use std::fs::OpenOptions;
use std::os::unix::ffi::OsStringExt;

use clap::Parser;
use libc::fork;
use rustix::stdio::{dup2_stdin, dup2_stdout};
use wl_clipboard_rs::copy::{self, clear, ClipboardType, MimeType, Seat, ServeRequests, Source};
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
    .seat(x.seat.map(Seat::Specific).unwrap_or_default());
    opts
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

    // Is there a way to do this without checking twice?
    let source_data = if options.text.is_empty() {
        None
    } else {
        // Copy the arguments into the target file.
        let mut iter = options.text.drain(..);
        let mut data = iter.next().unwrap();

        for arg in iter {
            data.push(" ");
            data.push(arg);
        }

        Some(data)
    };

    let source = if let Some(source_data) = source_data {
        Source::Bytes(source_data.into_vec().into())
    } else {
        Source::StdIn
    };

    let mime_type = if let Some(mime_type) = options.mime_type.take() {
        MimeType::Specific(mime_type)
    } else {
        MimeType::Autodetect
    };

    let foreground = options.foreground;
    let prepared_copy = from_options(options).prepare_copy(source, mime_type)?;

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
