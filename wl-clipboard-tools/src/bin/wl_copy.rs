#![deny(unsafe_code)]

use std::{ffi::OsString, os::unix::ffi::OsStringExt};

use exitfailure::ExitFailure;
use nix::unistd::{fork, ForkResult};
use structopt::{clap::AppSettings, StructOpt};

use wl_clipboard_rs::copy::{self, clear, ClipboardType, MimeType, Seat, ServeRequests, Source};

#[derive(StructOpt)]
#[structopt(name = "wl-copy",
            about = "Copy clipboard contents on Wayland.",
            rename_all = "kebab-case",
            setting = AppSettings::ColoredHelp)]
struct Options {
    /// Serve only a single paste request and then exit
    ///
    /// This option effectively clears the clipboard after the first paste. It can be used when
    /// copying e.g. sensitive data, like passwords. Note however that certain apps may have issues
    /// pasting when this option is used, in particular XWayland clients are known to suffer from
    /// this.
    #[structopt(long, short = "o", conflicts_with = "clear")]
    paste_once: bool,

    /// Stay in the foreground instead of forking
    #[structopt(long, short, conflicts_with = "clear")]
    foreground: bool,

    /// Clear the clipboard instead of copying
    #[structopt(long, short)]
    clear: bool,

    /// Use the "primary" clipboard
    ///
    /// Copying to the "primary" clipboard requires the compositor to support the data-control
    /// protocol of version 2 or above.
    #[structopt(long, short)]
    primary: bool,

    /// Use the regular clipboard
    ///
    /// Set this flag together with --primary to operate on both clipboards at once. Has no effect
    /// otherwise (since the regular clipboard is the default clipboard).
    #[structopt(long, short)]
    regular: bool,

    /// Trim the trailing newline character before copying
    ///
    /// This flag is only applied for text MIME types.
    #[structopt(long, short = "n", conflicts_with = "clear")]
    trim_newline: bool,

    /// Pick the seat to work with
    ///
    /// By default wl-copy operates on all seats at once.
    #[structopt(long, short)]
    seat: Option<String>,

    /// Override the inferred MIME type for the content
    #[structopt(name = "mime-type", long = "type", short = "t", conflicts_with = "clear")]
    mime_type: Option<String>,

    /// Text to copy
    ///
    /// If not specified, wl-copy will use data from the standard input.
    #[structopt(name = "text to copy", conflicts_with = "clear", parse(from_os_str))]
    text: Vec<OsString>,

    /// Enable verbose logging
    #[structopt(long, short, parse(from_occurrences))]
    verbose: usize,
}

impl From<Options> for copy::Options {
    fn from(x: Options) -> Self {
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
}

fn main() -> Result<(), ExitFailure> {
    // Parse command-line options.
    let mut options = Options::from_args();

    stderrlog::new().verbosity(options.verbose.saturating_add(1))
                    .init()
                    .unwrap();

    if options.clear {
        let clipboard = if options.primary {
            ClipboardType::Primary
        } else {
            ClipboardType::Regular
        };
        clear(clipboard, options.seat.map(Seat::Specific).unwrap_or_default())?;
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
    let prepared_copy = copy::Options::from(options).prepare_copy(source, mime_type)?;

    if foreground {
        prepared_copy.serve()?;
    } else {
        // We don't spawn any threads, so doing things after forking is safe.
        // TODO: is there any way to verify that we don't spawn any threads?
        if let ForkResult::Child = fork().unwrap() {
            drop(prepared_copy.serve());
        }
    }

    Ok(())
}
