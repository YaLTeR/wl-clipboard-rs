use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    io::{stdout, Read, Write},
    os::unix::io::AsRawFd,
    process,
};

use os_pipe::pipe;
use structopt::{clap::AppSettings, StructOpt};

mod common;
use common::{initialize, CommonData};

mod handlers;
use handlers::DataDeviceHandler;

mod protocol;

mod seat_data;
use seat_data::SeatData;

mod utils;
use utils::is_text;

#[derive(StructOpt)]
#[structopt(name = "wl-paste",
            about = "Paste clipboard contents on Wayland.",
            rename_all = "kebab-case",
            raw(setting = "AppSettings::ColoredHelp"))]
struct Options {
    /// List the offered types instead of pasting
    #[structopt(long, short)]
    list_types: bool,

    /// Use the "primary" clipboard
    #[structopt(long, short)]
    primary: bool,

    /// Do not append a newline character
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

fn main() {
    // Parse command-line options.
    let options = Options::from_args();

    env_logger::init();

    let CommonData { mut queue,
                     clipboard_manager,
                     seats,
                     .. } = initialize(options.primary);

    // If there are no seats, print an error message and exit.
    if seats.borrow_mut().is_empty() {
        eprintln!("There are no seats; nowhere to paste from.");
        process::exit(1);
    }

    // Go through the seats and get their data devices.
    for seat in &*seats.borrow_mut() {
        clipboard_manager.get_data_device(seat, |device| {
                             device.implement(DataDeviceHandler::new(seat.clone(), options.primary),
                                              ())
                         })
                         .unwrap();
    }

    // Retrieve all seat names and offers.
    queue.sync_roundtrip().expect("Error doing a roundtrip");

    // Check if the compositor supports primary selection.
    if options.primary {
        let supports_primary = clipboard_manager.as_ref()
                                                .user_data::<Cell<bool>>()
                                                .unwrap()
                                                .get();
        if !supports_primary {
            eprintln!("The compositor does not support primary selection.");
            process::exit(1);
        }
    }

    // Figure out which offer we're interested in.
    let offer = seats.borrow_mut()
                     .iter()
                     .map(|seat| {
                         seat.as_ref()
                             .user_data::<RefCell<SeatData>>()
                             .unwrap()
                             .borrow()
                     })
                     .find_map(|data| {
                         let SeatData { name, offer, .. } = &*data;
                         if options.seat.is_none() {
                             return Some(offer.clone());
                         }

                         let desired_name = options.seat.as_ref().unwrap();
                         if let Some(name) = name {
                             if name == desired_name {
                                 return Some(offer.clone());
                             }
                         }

                         None
                     });

    // If we didn't find the seat, print an error message and exit.
    if offer.is_none() {
        eprintln!("Cannot find the requested seat.");
        process::exit(1);
    }

    let offer = offer.unwrap();

    // If there is no offer for the seat, exit with code 1.
    if offer.is_none() {
        eprintln!("The clipboard of the requested seat is empty.");
        process::exit(1);
    }

    let offer = offer.unwrap();
    let mut mime_types = offer.as_ref()
                              .user_data::<RefCell<HashSet<String>>>()
                              .unwrap()
                              .borrow_mut();

    // If requested, print out the types and exit.
    if options.list_types {
        for mime_type in mime_types.iter() {
            println!("{}", mime_type);
        }

        return;
    }

    // Find the desired MIME type.
    let mime_type = match options.mime_type {
        Some(mime_type) => mime_types.take(&mime_type),
        None => mime_types.drain().next(),
    };

    // If no suitable MIME type is copied, print an error message and exit.
    if mime_type.is_none() {
        eprintln!("No suitable type of content copied.");
        process::exit(1);
    }

    let mime_type = mime_type.unwrap();
    let mime_type_is_text = is_text(&mime_type);

    // Create a pipe for content transfer.
    let (mut read, write) = pipe().expect("Error creating a pipe");

    // Start the transfer.
    offer.receive(mime_type, write.as_raw_fd());
    drop(write);
    queue.sync_roundtrip().expect("Error doing a roundtrip");

    // Read the contents.
    let mut contents = vec![];
    read.read_to_end(&mut contents)
        .expect("Error reading clipboard contents");

    // Append a newline if needed.
    let last_character_is_newline = contents.last().map(|&c| c == b'\n').unwrap_or(false);
    if !options.no_newline && mime_type_is_text && !last_character_is_newline {
        contents.push(b'\n');
    }

    // Write everything to stdout.
    stdout().write_all(&contents)
            .expect("Error writing contents to stdout");
}
