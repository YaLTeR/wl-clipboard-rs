#![deny(unsafe_code)]

use std::fs::read_link;
use std::io::{stdout, Read, Write};

use anyhow::Context;
use clap::Parser;
use libc::STDOUT_FILENO;
use log::trace;
use mime_guess::Mime;
use wl_clipboard_rs::paste::*;
use wl_clipboard_rs::utils::is_text;
use wl_clipboard_rs_tools::wl_paste::Options;

fn infer_mime_type() -> Option<Mime> {
    if let Ok(stdout_path) = read_link(format!("/dev/fd/{}", STDOUT_FILENO)) {
        mime_guess::from_path(stdout_path).first()
    } else {
        None
    }
}

fn main() -> Result<(), anyhow::Error> {
    // Parse command-line options.
    let options = Options::parse();
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
        .verbosity(usize::from(options.verbose) + 1)
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
