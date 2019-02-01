use std::{
    cell::{Cell, RefCell},
    ffi::OsString,
    fs::{remove_dir, remove_file, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::unix::{ffi::OsStringExt, io::IntoRawFd},
    path::PathBuf,
    process,
    rc::Rc,
};

use log::info;
use nix::unistd::{fork, ForkResult};
use structopt::{clap::AppSettings, StructOpt};

mod common;
use common::{initialize, CommonData};

mod handlers;
use handlers::{DataDeviceHandler, DataSourceHandler};

mod protocol;
use protocol::wlr_data_control::client::zwlr_data_control_source_v1::ZwlrDataControlSourceV1;

mod seat_data;
use seat_data::SeatData;

mod utils;
use utils::{copy_data, is_text};

#[derive(StructOpt)]
#[structopt(name = "wl-copy",
            about = "Copy clipboard contents on Wayland.",
            rename_all = "kebab-case",
            raw(setting = "AppSettings::ColoredHelp"))]
struct Options {
    /// Serve only a single paste request and then exit
    #[structopt(long, short = "o", conflicts_with = "clear")]
    paste_once: bool,

    /// Stay in the foreground instead of forking
    #[structopt(long, short, conflicts_with = "clear")]
    foreground: bool,

    /// Clear the clipboard instead of copying
    #[structopt(long, short)]
    clear: bool,

    /// Use the "primary" clipboard
    #[structopt(long, short)]
    primary: bool,

    /// Trim a trailing newline character before copying
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

fn make_source(options: &mut Options) -> (String, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Error creating a temp directory");
    let mut temp_filename = temp_dir.into_path();
    temp_filename.push("stdin");
    info!("Temp filename: {}", temp_filename.to_string_lossy());
    let mut temp_file = File::create(&temp_filename).expect("Error opening a temp file");

    if options.text.is_empty() {
        // Copy the standard input into the target file.
        copy_data(None, temp_file.into_raw_fd(), true);
    } else {
        // Copy the arguments into the target file.
        let mut iter = options.text.drain(..);
        let mut data = iter.next().unwrap();

        for arg in iter {
            data.push(" ");
            data.push(arg);
        }

        let data = data.into_vec();

        temp_file.write_all(&data)
                 .expect("Error writing to the temp file");
    }

    let mime_type = options.mime_type
                           .take()
                           .unwrap_or_else(|| "application/octet-stream".to_string());

    // Trim the trailing newline if needed.
    if options.trim_newline && is_text(&mime_type) {
        let mut temp_file = OpenOptions::new().read(true)
                                              .write(true)
                                              .open(&temp_filename)
                                              .expect("Error opening the temp file");
        let metadata = temp_file.metadata()
                                .expect("Error getting the temp file metadata");
        let length = metadata.len();
        if length > 0 {
            temp_file.seek(SeekFrom::End(-1))
                     .expect("Error seeking the temp file");

            let mut buf = [0];
            temp_file.read_exact(&mut buf)
                     .expect("Error reading the last byte of the temp file");
            if buf[0] == b'\n' {
                temp_file.set_len(length - 1)
                         .expect("Error truncating the temp file");
            }
        }
    }

    (mime_type, temp_filename)
}

fn main() {
    // Parse command-line options.
    let mut options = Options::from_args();

    env_logger::init();

    let CommonData { display,
                     mut queue,
                     clipboard_manager,
                     seats,
                     .. } = initialize(options.primary);

    // If there are no seats, print an error message and exit.
    if seats.borrow_mut().is_empty() {
        eprintln!("There are no seats; nowhere to copy to.");
        process::exit(1);
    }

    let data_source = if !options.clear {
        // Collect the source data to copy.
        let (mime_type, data_path) = make_source(&mut options);
        Some((mime_type, Rc::new(RefCell::new(data_path))))
    } else {
        None
    };

    let should_quit = Rc::new(Cell::new(false));

    // Go through the seats and get their data devices.
    for seat in &*seats.borrow_mut() {
        // TODO: fast path here if all seats.
        // TODO: if not all seats, we don't need to create the data sources yet.
        let data_source = if let Some((mime_type, data_path)) = data_source.as_ref() {
            let data_source = clipboard_manager.create_data_source(|source| {
                                  source.implement(DataSourceHandler::new(data_path.clone(),
                                                                          should_quit.clone(),
                                                                          options.paste_once),
                                                   ())
                              })
                              .unwrap();

            // If the MIME type is text, offer it in some other common formats.
            if is_text(&mime_type) {
                data_source.offer("text/plain;charset=utf-8".to_string());
                data_source.offer("text/plain".to_string());
                data_source.offer("STRING".to_string());
                data_source.offer("UTF8_STRING".to_string());
                data_source.offer("TEXT".to_string());
            }

            data_source.offer(mime_type.clone());
            Some(data_source)
        } else {
            None
        };

        let device = clipboard_manager.get_data_device(seat, |device| {
                                          device.implement(DataDeviceHandler::new(seat.clone(),
                                                                                  options.primary),
                                                           data_source)
                                      })
                                      .unwrap();

        let seat_data = seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_device(Some(device));
    }

    // Retrieve all seat names.
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

    // Figure out which devices we're interested in.
    let devices = seats.borrow_mut()
                       .iter()
                       .map(|seat| {
                           seat.as_ref()
                               .user_data::<RefCell<SeatData>>()
                               .unwrap()
                               .borrow()
                       })
                       .filter_map(|data| {
                           let SeatData { name, device, .. } = &*data;

                           if device.is_none() {
                               // Can't handle seats without devices.
                               return None;
                           }

                           let device = device.as_ref().cloned().unwrap();

                           if options.seat.is_none() {
                               // If no seat was specified, handle all of them.
                               return Some(device);
                           }

                           let desired_name = options.seat.as_ref().unwrap();
                           if let Some(name) = name {
                               if name == desired_name {
                                   return Some(device);
                               }
                           }

                           None
                       })
                       .collect::<Vec<_>>();

    // If we didn't find the seat, print an error message and exit.
    if devices.is_empty() {
        eprintln!("Cannot find the requested seat.");
        process::exit(1);
    }

    if options.clear {
        for device in devices {
            if options.primary {
                device.set_primary_selection(None);
            } else {
                device.set_selection(None);
            }
        }

        // We're clearing the clipboard so just do one roundtrip and quit.
        queue.sync_roundtrip().expect("Error doing a roundtrip");
    } else {
        if !options.foreground {
            // Fork an exit the parent.
            if let ForkResult::Parent { .. } = fork().expect("Error forking") {
                return;
            }
        }

        for device in &devices {
            let source = device.as_ref()
                               .user_data::<Option<ZwlrDataControlSourceV1>>()
                               .unwrap()
                               .as_ref()
                               .unwrap();

            if options.primary {
                device.set_primary_selection(Some(&source));
            } else {
                device.set_selection(Some(&source));
            }
        }

        // Loop until we're done.
        while !should_quit.get() {
            display.flush().expect("Error flushing display");
            queue.dispatch().expect("Error dispatching queue");

            // Check if all sources have been destroyed.
            let all_destroyed = devices.iter()
                                       .map(|device| {
                                           device.as_ref()
                                                 .user_data::<Option<ZwlrDataControlSourceV1>>()
                                                 .unwrap()
                                                 .as_ref()
                                                 .unwrap()
                                                 .as_ref()
                                                 .is_alive()
                                       })
                                       .all(|x| !x);
            if all_destroyed {
                should_quit.set(true);
            }
        }

        // Clean up the temp file and directory.
        let (_, data_path) = data_source.unwrap();
        let mut data_path = data_path.borrow_mut();
        remove_file(&*data_path).expect("Error removing the temp file");
        data_path.pop();
        remove_dir(&*data_path).expect("Error removing the temp directory");
    }
}
