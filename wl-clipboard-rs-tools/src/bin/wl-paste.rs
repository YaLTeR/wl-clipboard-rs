#![deny(unsafe_code)]

use std::fs::read_link;
use std::io::{stdout, Read, Write};
use std::process::{Command, Stdio};

use anyhow::Context;
use clap::Parser;
use libc::STDOUT_FILENO;
use log::trace;
use mime_guess::Mime;
use wl_clipboard_rs::paste::*;
use wl_clipboard_rs::utils::is_text;
use wl_clipboard_rs::watch::{ClipboardEvent, Watcher};
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
        let mime_types = get_mime_types_ordered(primary, seat)?;

        for mime_type in mime_types.iter() {
            println!("{}", mime_type);
        }

        return Ok(());
    }

    // No MIME type specified—try inferring one from the output file extension (if any).
    let inferred = if options.mime_type.is_none() {
        infer_mime_type()
    } else {
        None
    };

    // Build the MimeType selector (shared by both watch and single-paste paths).
    let mime_type_selector = match options.mime_type {
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

    if options.watch {
        return watch_mode(primary, seat, mime_type_selector, &options.watch_command);
    }

    let (mut read, mime_type) = get_contents(primary, seat, mime_type_selector)?;

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

const CLIPBOARD_STATE_DATA: &str = "data";
const CLIPBOARD_STATE_SENSITIVE: &str = "sensitive";
const CLIPBOARD_STATE_NIL: &str = "nil";
const MIME_TYPE_PASSWORD_MANAGER_HINT: &str = "x-kde-passwordManagerHint";

fn watch_mode(
    clipboard: ClipboardType,
    seat: Seat<'_>,
    mime_type_selector: MimeType<'_>,
    cmd: &[String],
) -> Result<(), anyhow::Error> {
    let mut watcher = Watcher::new(clipboard, seat)?;
    while let Some((event, mut offer)) = watcher.next_event()? {
        let mime_types = match event {
            ClipboardEvent::Cleared => None,
            ClipboardEvent::Changed { mime_types } => Some(mime_types),
        };

        let Some(mime_types) = mime_types else {
            run_watch_cmd(cmd, Stdio::null(), CLIPBOARD_STATE_NIL);
            continue;
        };

        let clipboard_state = if mime_types
            .iter()
            .any(|mt| mt == MIME_TYPE_PASSWORD_MANAGER_HINT)
        {
            CLIPBOARD_STATE_SENSITIVE
        } else {
            CLIPBOARD_STATE_DATA
        };

        let Some(selected) = select_mime_type(mime_types, mime_type_selector) else {
            continue;
        };

        match offer.receive(&selected) {
            Ok(pipe) => run_watch_cmd(cmd, Stdio::from(pipe), clipboard_state),
            Err(e) => eprintln!("wl-paste: failed to receive clipboard contents: {e}"),
        }
    }
    Ok(())
}

fn run_watch_cmd(cmd: &[String], stdin: Stdio, clipboard_state: &str) {
    match Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(stdin)
        .env("CLIPBOARD_STATE", clipboard_state)
        .spawn()
    {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(e) => eprintln!("wl-paste: failed to spawn {}: {e}", cmd[0]),
    }
}
