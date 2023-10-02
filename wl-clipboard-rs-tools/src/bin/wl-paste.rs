#![deny(unsafe_code)]

use std::fs::read_link;
use std::io::{stdout, Read, Write};

use anyhow::Context;
use libc::STDOUT_FILENO;
use log::trace;
use mime_guess::Mime;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use wl_clipboard_rs::paste::*;
use wl_clipboard_rs::utils::is_text;

#[derive(StructOpt)]
#[structopt(name = "wl-paste",
            about = "Paste clipboard contents on Wayland.",
            rename_all = "kebab-case",
            setting = AppSettings::ColoredHelp)]
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

    /// Request the given MIME type instead of inferring the MIME type
    ///
    /// As a special case, specifying "text" will look for a number of plain text types,
    /// prioritizing ones that are known to give UTF-8 text.
    #[structopt(
        name = "mime-type",
        long = "type",
        short = "t",
        conflicts_with = "list-types"
    )]
    mime_type: Option<String>,

    /// Enable verbose logging
    #[structopt(long, short, parse(from_occurrences))]
    verbose: usize,
}

fn infer_mime_type() -> Option<Mime> {
    if let Ok(stdout_path) = read_link(format!("/dev/fd/{}", STDOUT_FILENO)) {
        mime_guess::from_path(stdout_path).first()
    } else {
        None
    }
}

fn main() -> Result<(), anyhow::Error> {
    // Parse command-line options.
    let options = Options::from_args();
    let primary = if options.primary {
        ClipboardType::Primary
    } else {
        ClipboardType::Regular
    };
    let seat = options
        .seat
        .as_ref()
        .map(|x| Seat::Specific(x))
        .unwrap_or_default();

    stderrlog::new()
        .verbosity(options.verbose.saturating_add(1))
        .init()
        .unwrap();

    // If listing types is requested, do just that.
    if options.list_types {
        let mime_types = get_mime_types(primary, seat)?;

        for mime_type in mime_types.iter() {
            println!("{}", mime_type);
        }

        return Ok(());
    }

    // Otherwise, get the clipboard contents.

    // No MIME type specified—try inferring one from the output file extension (if any).
    let inferred = if options.mime_type.is_none() {
        infer_mime_type()
    } else {
        None
    };

    // Do some smart MIME type selection.
    let mime_type = match options.mime_type {
        Some(ref mime_type) if mime_type == "text" => MimeType::Text,
        Some(ref mime_type) => MimeType::Specific(mime_type),
        None => {
            let inferred: Option<&str> = inferred.as_ref().map(Mime::as_ref);
            trace!("Inferred MIME type: {:?}", inferred);
            match inferred {
                None | Some("application/octet-stream") => MimeType::Any,
                // If the inferred MIME type is text, make sure we'll fall back to requesting
                // other plain text types if this particular one is unavailable.
                Some(t) if is_text(t) => MimeType::TextWithPriority(t),
                Some(t) => MimeType::Specific(t),
            }
        }
    };

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
    stdout()
        .write_all(&contents)
        .context("Couldn't write contents to stdout")?;

    Ok(())
}
