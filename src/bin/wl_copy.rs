use std::{ffi::OsString, os::unix::ffi::OsStrExt};

use exitfailure::ExitFailure;
use structopt::{clap::AppSettings, StructOpt};

use wl_clipboard_rs::{
    copy::{self, clear, MimeType, Seat, Source},
    ClipboardType,
};

#[derive(StructOpt)]
#[structopt(name = "wl-copy",
            about = "Copy clipboard contents on Wayland.",
            rename_all = "kebab-case",
            raw(setting = "AppSettings::ColoredHelp"))]
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
    #[structopt(name = "mime-type",
                long = "type",
                short = "t",
                conflicts_with = "clear")]
    mime_type: Option<String>,

    /// Text to copy
    ///
    /// If not specified, wl-copy will use data from the standard input.
    #[structopt(name = "text to copy", conflicts_with = "clear", parse(from_os_str))]
    text: Vec<OsString>,
}

impl<'a> From<&'a Options> for copy::Options<'a> {
    fn from(x: &'a Options) -> Self {
        let mut opts = copy::Options::new();
        opts.paste_once(x.paste_once)
            .foreground(x.foreground)
            .clipboard(if x.primary {
                           ClipboardType::Primary
                       } else {
                           ClipboardType::Regular
                       })
            .trim_newline(x.trim_newline)
            .seat(x.seat
                   .as_ref()
                   .map(|x| Seat::Specific(x))
                   .unwrap_or_default());
        opts
    }
}

fn main() -> Result<(), ExitFailure> {
    // Parse command-line options.
    let mut options = Options::from_args();

    env_logger::init();

    if options.clear {
        let clipboard = if options.primary {
            ClipboardType::Primary
        } else {
            ClipboardType::Regular
        };
        clear(clipboard,
              options.seat
                     .as_ref()
                     .map(|x| Seat::Specific(x))
                     .unwrap_or_default())?;
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

    let source = if source_data.is_none() {
        Source::StdIn
    } else {
        Source::Bytes(source_data.as_ref().unwrap().as_bytes())
    };

    let mime_type = if let Some(mime_type) = options.mime_type.take() {
        MimeType::Specific(mime_type)
    } else {
        MimeType::Autodetect
    };

    copy::Options::from(&options).copy(source, mime_type)?;

    Ok(())
}
