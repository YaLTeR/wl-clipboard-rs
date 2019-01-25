use std::{
    cell::RefCell,
    collections::HashSet,
    io::{stdout, Read, Write},
    os::unix::io::AsRawFd,
    process,
    rc::Rc,
};

use os_pipe::pipe;
use structopt::{clap::AppSettings, StructOpt};
use wayland_client::{
    protocol::{wl_compositor::WlCompositor, wl_seat::WlSeat},
    Display, NewProxy,
};
use wayland_protocols::wlr::unstable::{
    data_control::v1::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    layer_shell::v1::client::zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
};

mod protocol;
use protocol::gtk_primary_selection::client::gtk_primary_selection_device_manager::GtkPrimarySelectionDeviceManager;

mod clipboard_manager;
use clipboard_manager::ClipboardManager;

mod data_device;
mod offer;

mod seat_data;
use seat_data::SeatData;

mod handlers;
use handlers::{DataDeviceHandler, LayerSurfaceHandler, WlRegistryHandler};

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

fn is_text(mime_type: &str) -> bool {
    match mime_type {
        "TEXT" | "STRING" | "UTF8_STRING" => true,
        x if x.starts_with("text/") => true,
        _ => false,
    }
}

fn main() {
    // Parse command-line options.
    let options = Options::from_args();

    // Connect to the Wayland compositor.
    let (display, mut queue) = Display::connect_to_env().expect("Error connecting to a display");

    let data_control_manager = Rc::new(RefCell::new(None::<ZwlrDataControlManagerV1>));
    let gtk_manager = Rc::new(RefCell::new(None::<GtkPrimarySelectionDeviceManager>));
    let layer_shell = Rc::new(RefCell::new(None::<ZwlrLayerShellV1>));
    let compositor = Rc::new(RefCell::new(None::<WlCompositor>));
    let seats = Rc::new(RefCell::new(Vec::<WlSeat>::new()));

    display.get_registry(|registry| {
               registry.implement(WlRegistryHandler::new(data_control_manager.clone(),
                                                         gtk_manager.clone(),
                                                         layer_shell.clone(),
                                                         compositor.clone(),
                                                         seats.clone()),
                                  ())
           })
           .unwrap();

    // Retrieve the global interfaces.
    queue.sync_roundtrip().expect("Error doing a roundtrip");

    // Check that we have our interfaces.
    let manager: ClipboardManager = if options.primary {
        gtk_manager.borrow_mut()
                   .take()
                   .expect("gtk_primary_selection_device_manager was not found")
                   .into()
    } else {
        data_control_manager.borrow_mut()
                            .take()
                            .expect("zwlr_data_control_manager_v1 was not found")
                            .into()
    };

    // If there are no seats, print an error message and exit.
    if seats.borrow().is_empty() {
        eprintln!("There are no seats; nowhere to paste from.");
        process::exit(1);
    }

    // If using a protocol that requires keyboard focus, make a surface.
    if manager.requires_keyboard_focus() {
        let compositor = compositor.borrow_mut()
                                   .take()
                                   .expect("wl_compositor was not found");
        let surface = compositor.create_surface(NewProxy::implement_dummy)
                                .unwrap();

        let layer_shell = layer_shell.borrow_mut()
                                     .take()
                                     .expect("zwlr_layer_shell_v1 was not found");
        let layer_surface =
            layer_shell.get_layer_surface(&surface,
                                          None,
                                          Layer::Overlay,
                                          "wl-clipboard-rs".to_string(),
                                          |surface| surface.implement(LayerSurfaceHandler, ()))
                       .unwrap();

        layer_surface.set_keyboard_interactivity(1);
        surface.commit();

        queue.sync_roundtrip().expect("Error doing a roundtrip");
    }

    // Go through the seats and get their data devices.
    for seat in &*seats.borrow() {
        manager.get_device(seat, DataDeviceHandler::new(seat.clone()))
               .unwrap();
    }

    // Retrieve all seat names and offers.
    queue.sync_roundtrip().expect("Error doing a roundtrip");

    // Figure out which offer we're interested in.
    let offer = seats.borrow()
                     .iter()
                     .map(|seat| {
                         seat.as_ref()
                             .user_data::<RefCell<SeatData>>()
                             .unwrap()
                             .borrow()
                     })
                     .find_map(|data| {
                         let SeatData { name, offer } = &*data;
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
    let mut mime_types = offer.user_data::<RefCell<HashSet<String>>>()
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
