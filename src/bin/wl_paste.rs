use std::io::{stdout, Read, Write};

use exitfailure::ExitFailure;
use failure::ResultExt;
use structopt::{clap::AppSettings, StructOpt};
use wl_clipboard_rs::{paste::*, utils::is_text, ClipboardType};

#[derive(StructOpt)]
#[structopt(name = "wl-paste",
            about = "Paste clipboard contents on Wayland.",
            rename_all = "kebab-case",
            raw(setting = "AppSettings::ColoredHelp"))]
struct Options {
    /// List the offered MIME types instead of pasting
    #[structopt(long, short)]
    list_types: bool,

    /// Use the "primary" clipboard
    ///
    /// Pasting to the "primary" clipboard requires the compositor to support the data-control
    /// protocol of version 2 or above.
    #[structopt(long, short)]
    primary: bool,

    /// Do not append a newline character
    ///
    /// By default the newline character is appended automatically when pasting text MIME types.
    #[structopt(long, short, conflicts_with = "list-types")]
    no_newline: bool,

    /// Pick the seat to work with
    ///
    /// By default the seat used is unspecified (it depends on the order returned by the
    /// compositor). This is perfectly fine when only a single seat is present, so for most
    /// configurations.
    #[structopt(long, short)]
    seat: Option<String>,

    /// Override the inferred MIME type for the content
    #[structopt(name = "mime-type",
                long = "type",
                short = "t",
                conflicts_with = "list-types")]
    mime_type: Option<String>,
}

fn main() -> Result<(), ExitFailure> {
    // Parse command-line options.
    let options = Options::from_args();
    let primary = if options.primary {
        ClipboardType::Primary
    } else {
        ClipboardType::Regular
    };
    let seat = options.seat.map(Seat::Specific).unwrap_or_default();

    env_logger::init();

    // If listing types is requested, do just that.
    if options.list_types {
        let mime_types = get_mime_types(primary, seat)?;

        for mime_type in mime_types.iter() {
            println!("{}", mime_type);
        }

        return Ok(());
    }

    // Otherwise, get the clipboard contents.
    let mime_type = options.mime_type
                           .map(MimeType::Specific)
                           .unwrap_or(MimeType::Any);
    let (mut read, mime_type) = get_contents(primary, seat, mime_type)?;

    // Read the contents.
    let mut contents = vec![];
    read.read_to_end(&mut contents)
        .context("Couldn't read clipboard contents")?;

    // Append a newline if needed.
    let last_character_is_newline = contents.last().map(|&c| c == b'\n').unwrap_or(false);
    if !options.no_newline && is_text(&mime_type) && !last_character_is_newline {
        contents.push(b'\n');
    }

    // Write everything to stdout.
    stdout().write_all(&contents)
            .context("Couldn't write contents to stdout")?;

    Ok(())
}
