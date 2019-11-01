//! Copying and clearing clipboard contents.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ffi::OsString,
    fs::{remove_dir, remove_file, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    iter,
    os::unix::io::IntoRawFd,
    path::PathBuf,
    process,
    rc::Rc,
};

use failure::Fail;
use log::info;
use nix::unistd::{fork, ForkResult};
use wayland_client::{ConnectError, EventQueue, Proxy};
use wayland_protocols::wlr::unstable::data_control::v1::client::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
};

use crate::{
    common::{self, initialize, CommonData},
    handlers::{DataDeviceHandler, DataSourceError, DataSourceHandler},
    seat_data::SeatData,
    utils::{self, copy_data, is_text},
};

/// The clipboard to operate on.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum ClipboardType {
    /// The regular clipboard.
    Regular,
    /// The "primary" clipboard.
    ///
    /// Working with the "primary" clipboard requires the compositor to support the data-control
    /// protocol of version 2 or above.
    Primary,
    /// Operate on both clipboards at once.
    ///
    /// Useful for atomically setting both clipboards at once. This option requires the "primary"
    /// clipboard to be supported.
    Both,
}

impl Default for ClipboardType {
    #[inline]
    fn default() -> Self {
        ClipboardType::Regular
    }
}

/// MIME type to offer the copied data under.
#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum MimeType {
    /// Detect the MIME type automatically from the data.
    Autodetect,
    /// Offer a number of common plain text MIME types.
    Text,
    /// Offer a specific MIME type.
    Specific(String),
}

/// Source for copying.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Source<'a> {
    /// Copy contents of the standard input.
    StdIn,
    /// Copy the given bytes.
    Bytes(&'a [u8]),
}

/// Source for copying, with its MIME type included
#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub struct MimeSource<'a> {
    pub source: Source<'a>,
    pub mime: MimeType,
}

/// Seat to operate on.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Seat<'a> {
    /// Operate on all existing seats at once.
    All,
    /// Operate on a seat with the given name.
    Specific(&'a str),
}

impl Default for Seat<'_> {
    #[inline]
    fn default() -> Self {
        Seat::All
    }
}

/// Number of paste requests to serve.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum ServeRequests {
    /// Serve requests indefinitely.
    Unlimited,
    /// Serve only the given number of requests.
    Only(usize),
}

impl Default for ServeRequests {
    #[inline]
    fn default() -> Self {
        ServeRequests::Unlimited
    }
}

/// Options and flags that are used to customize the copying.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, PartialOrd, Ord)]
pub struct Options<'a> {
    /// The clipboard to work with.
    clipboard: ClipboardType,

    /// The seat to work with.
    seat: Seat<'a>,

    /// Trim the trailing newline character before copying.
    ///
    /// This flag is only applied for text MIME types.
    trim_newline: bool,

    /// Do not fork.
    ///
    /// Setting this flag will result in the call to `copy()` **blocking** until all data sources
    /// it creates are destroyed, e.g. until someone else copies something into the clipboard.
    foreground: bool,

    /// Number of paste requests to serve.
    ///
    /// Limiting the number of paste requests to one effectively clears the clipboard after the
    /// first paste. It can be used when copying e.g. sensitive data, like passwords. Note however
    /// that certain apps may have issues pasting when this option is used, in particular XWayland
    /// clients are known to suffer from this.
    serve_requests: ServeRequests,
}

/// Errors that can occur for copying the source data to a temporary file.
#[derive(Fail, Debug)]
pub enum SourceCreationError {
    #[fail(display = "Couldn't create a temporary directory")]
    TempDirCreate(#[cause] io::Error),

    #[fail(display = "Couldn't create a temporary file")]
    TempFileCreate(#[cause] io::Error),

    #[fail(display = "Couldn't copy data to the temporary file")]
    DataCopy(#[cause] utils::CopyDataError),

    #[fail(display = "Couldn't write to the temporary file")]
    TempFileWrite(#[cause] io::Error),

    #[fail(display = "Couldn't open the temporary file for newline trimming")]
    TempFileOpen(#[cause] io::Error),

    #[fail(display = "Couldn't get the temporary file metadata for newline trimming")]
    TempFileMetadata(#[cause] io::Error),

    #[fail(display = "Couldn't seek the temporary file for newline trimming")]
    TempFileSeek(#[cause] io::Error),

    #[fail(display = "Couldn't read the last byte of the temporary file for newline trimming")]
    TempFileRead(#[cause] io::Error),

    #[fail(display = "Couldn't truncate the temporary file for newline trimming")]
    TempFileTruncate(#[cause] io::Error),
}

/// Errors that can occur for copying and clearing the clipboard.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "There are no seats")]
    NoSeats,

    #[fail(display = "Couldn't connect to the Wayland compositor")]
    WaylandConnection(#[cause] ConnectError),

    #[fail(display = "Wayland compositor communication error")]
    WaylandCommunication(#[cause] io::Error),

    #[fail(display = "A required Wayland protocol ({} version {}) is not supported by the compositor",
           name, version)]
    MissingProtocol { name: &'static str, version: u32 },

    #[fail(display = "The compositor does not support primary selection")]
    PrimarySelectionUnsupported,

    #[fail(display = "The requested seat was not found")]
    SeatNotFound,

    #[fail(display = "Couldn't fork")]
    Fork(#[cause] nix::Error),

    #[fail(display = "Error copying the source into a temporary file")]
    TempCopy(#[cause] SourceCreationError),

    #[fail(display = "Couldn't remove the temporary file")]
    TempFileRemove(#[cause] io::Error),

    #[fail(display = "Couldn't remove the temporary directory")]
    TempDirRemove(#[cause] io::Error),

    #[fail(display = "Error satisfying a paste request")]
    Paste(#[cause] DataSourceError),
}

impl From<common::Error> for Error {
    fn from(x: common::Error) -> Self {
        use common::Error::*;

        match x {
            WaylandConnection(err) => Error::WaylandConnection(err),
            WaylandCommunication(err) => Error::WaylandCommunication(err),
            MissingProtocol { name, version } => Error::MissingProtocol { name, version },
        }
    }
}

impl<'a> Options<'a> {
    /// Creates a blank new set of options ready for configuration.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the clipboard to work with.
    #[inline]
    pub fn clipboard(&mut self, clipboard: ClipboardType) -> &mut Self {
        self.clipboard = clipboard;
        self
    }

    /// Sets the seat to use for copying.
    #[inline]
    pub fn seat(&mut self, seat: Seat<'a>) -> &mut Self {
        self.seat = seat;
        self
    }

    /// Sets the flag for trimming the trailing newline.
    ///
    /// This flag is only applied for text MIME types.
    #[inline]
    pub fn trim_newline(&mut self, trim_newline: bool) -> &mut Self {
        self.trim_newline = trim_newline;
        self
    }

    /// Sets the flag for not forking.
    ///
    /// Setting this flag will result in the call to `copy()` **blocking** until all data sources
    /// it creates are destroyed, e.g. until someone else copies something into the clipboard.
    #[inline]
    pub fn foreground(&mut self, foreground: bool) -> &mut Self {
        self.foreground = foreground;
        self
    }

    /// Sets the number of requests to serve.
    ///
    /// Limiting the number of requests to one effectively clears the clipboard after the first
    /// paste. It can be used when copying e.g. sensitive data, like passwords. Note however that
    /// certain apps may have issues pasting when this option is used, in particular XWayland
    /// clients are known to suffer from this.
    #[inline]
    pub fn serve_requests(&mut self, serve_requests: ServeRequests) -> &mut Self {
        self.serve_requests = serve_requests;
        self
    }

    /// Invokes the copy operation. See `copy()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate wl_clipboard_rs;
    /// # use wl_clipboard_rs::copy::Error;
    /// # fn foo() -> Result<(), Error> {
    /// use wl_clipboard_rs::copy::{MimeType, Options, Source};
    ///
    /// let opts = Options::new();
    /// opts.copy(Source::Bytes(&[1, 2, 3]), MimeType::Autodetect)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn copy(self, source: Source<'_>, mime_type: MimeType) -> Result<(), Error> {
        copy(self, source, mime_type)
    }

    /// Invokes the copy_multi operation. See `copy_multi()`.
    #[inline]
    pub fn copy_multi(self, sources: Vec<MimeSource>) -> Result<(), Error> {
        copy_multi(self, sources)
    }
}

fn make_source(source: Source<'_>,
               mime_type: MimeType,
               trim_newline: bool)
               -> Result<(String, PathBuf), SourceCreationError> {
    let temp_dir = tempfile::tempdir().map_err(SourceCreationError::TempDirCreate)?;
    let mut temp_filename = temp_dir.into_path();
    temp_filename.push("stdin");
    info!("Temp filename: {}", temp_filename.to_string_lossy());
    let mut temp_file = File::create(&temp_filename).map_err(SourceCreationError::TempFileCreate)?;

    if let Source::Bytes(data) = source {
        temp_file.write_all(&data)
                 .map_err(SourceCreationError::TempFileWrite)?;
    } else {
        // Copy the standard input into the target file.
        copy_data(None, temp_file.into_raw_fd(), true).map_err(SourceCreationError::DataCopy)?;
    }

    let mime_type = match mime_type {
        MimeType::Autodetect => tree_magic::from_filepath(&temp_filename),
        MimeType::Text => "text/plain".to_string(),
        MimeType::Specific(mime_type) => mime_type,
    };

    info!("Base MIME type: {}", mime_type);

    // Trim the trailing newline if needed.
    if trim_newline && is_text(&mime_type) {
        let mut temp_file = OpenOptions::new().read(true)
                                              .write(true)
                                              .open(&temp_filename)
                                              .map_err(SourceCreationError::TempFileOpen)?;
        let metadata = temp_file.metadata()
                                .map_err(SourceCreationError::TempFileMetadata)?;
        let length = metadata.len();
        if length > 0 {
            temp_file.seek(SeekFrom::End(-1))
                     .map_err(SourceCreationError::TempFileSeek)?;

            let mut buf = [0];
            temp_file.read_exact(&mut buf)
                     .map_err(SourceCreationError::TempFileRead)?;
            if buf[0] == b'\n' {
                temp_file.set_len(length - 1)
                         .map_err(SourceCreationError::TempFileTruncate)?;
            }
        }
    }

    Ok((mime_type, temp_filename))
}

fn get_devices(
    primary: bool,
    seat: Seat<'_>,
    socket_name: Option<OsString>)
    -> Result<(EventQueue, ZwlrDataControlManagerV1, Vec<ZwlrDataControlDeviceV1>), Error> {
    let CommonData { mut queue,
                     clipboard_manager,
                     seats, } = initialize(primary, socket_name)?;

    // Check if there are no seats.
    if seats.borrow_mut().is_empty() {
        return Err(Error::NoSeats);
    }

    let supports_primary = Rc::new(Cell::new(false));

    // Go through the seats and get their data devices.
    for seat in &*seats.borrow_mut() {
        // TODO: fast path here if all seats.
        let handler = DataDeviceHandler::new(seat.clone(), primary, supports_primary.clone());
        let device =
            clipboard_manager.get_data_device(seat, |device| device.implement(handler, ()))
                             .unwrap();

        let seat_data = seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_device(Some(device));
    }

    // Retrieve all seat names.
    queue.sync_roundtrip()
         .map_err(Error::WaylandCommunication)?;

    // Check if the compositor supports primary selection.
    if primary && !supports_primary.get() {
        return Err(Error::PrimarySelectionUnsupported);
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

                           match seat {
                               Seat::All => {
                                   // If no seat was specified, handle all of them.
                                   return Some(device);
                               }
                               Seat::Specific(ref desired_name) => {
                                   if let Some(name) = name {
                                       if name == desired_name {
                                           return Some(device);
                                       }
                                   }
                               }
                           }

                           None
                       })
                       .collect::<Vec<_>>();

    // If we didn't find the seat, print an error message and exit.
    if devices.is_empty() {
        return Err(Error::SeatNotFound);
    }

    Ok((queue, clipboard_manager, devices))
}

/// Clears the clipboard for the given seat.
///
/// If `seat` is `None`, clears clipboards of all existing seats.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::copy::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::{copy::{clear, ClipboardType, Seat}};
///
/// clear(ClipboardType::Regular, Seat::All)?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn clear(clipboard: ClipboardType, seat: Seat<'_>) -> Result<(), Error> {
    clear_internal(clipboard, seat, None)
}

pub(crate) fn clear_internal(clipboard: ClipboardType,
                             seat: Seat<'_>,
                             socket_name: Option<OsString>)
                             -> Result<(), Error> {
    let primary = clipboard != ClipboardType::Regular;
    let (mut queue, _, devices) = get_devices(primary, seat, socket_name)?;

    for device in devices {
        if clipboard == ClipboardType::Primary || clipboard == ClipboardType::Both {
            device.set_primary_selection(None);
        }
        if clipboard == ClipboardType::Regular || clipboard == ClipboardType::Both {
            device.set_selection(None);
        }
    }

    // We're clearing the clipboard so just do one roundtrip and quit.
    queue.sync_roundtrip()
         .map_err(Error::WaylandCommunication)?;

    Ok(())
}

fn copy_past_fork(clipboard: ClipboardType,
                  serve_requests: ServeRequests,
                  mut queue: EventQueue,
                  clipboard_manager: ZwlrDataControlManagerV1,
                  devices: Vec<ZwlrDataControlDeviceV1>,
                  data_paths: HashMap<String, Rc<RefCell<PathBuf>>>,
                  create_multi_text: bool)
                  -> Result<(), Error> {
    let should_quit = Rc::new(Cell::new(false));
    let error = Rc::new(RefCell::new(None::<DataSourceError>));
    let serve_requests = Rc::new(Cell::new(serve_requests));

    // Create an iterator over (device, primary) for source creation later.
    //
    // This is needed because for ClipboardType::Both each device needs to appear twice because
    // separate data sources need to be made for the regular and the primary clipboards (data
    // sources cannot be reused).
    let devices_iter = devices.iter().flat_map(|device| {
                                         let first = match clipboard {
                                             ClipboardType::Regular => iter::once((device, false)),
                                             ClipboardType::Primary => iter::once((device, true)),
                                             ClipboardType::Both => iter::once((device, false)),
                                         };

                                         let second = if clipboard == ClipboardType::Both {
                                             iter::once(Some((device, true)))
                                         } else {
                                             iter::once(None)
                                         };

                                         first.chain(second.filter_map(|x| x))
                                     });

    // Create the data sources and set them as selections.
    let mut sources: Vec<Proxy<_>> = Vec::new();
    for (device, primary) in devices_iter {
        let handler = DataSourceHandler::new(data_paths.clone(),
                                            should_quit.clone(),
                                            serve_requests.clone());
        let data_source = clipboard_manager.create_data_source(|source| {
                            source.implement(handler, error.clone())
                        })
                        .unwrap();

        for (mime_type, _) in &data_paths {
            dbg!(&mime_type);
            // If the MIME type is text, offer it in some other common formats.
            if create_multi_text && is_text(&mime_type) {
                data_source.offer("text/plain;charset=utf-8".to_string());
                data_source.offer("text/plain".to_string());
                data_source.offer("STRING".to_string());
                data_source.offer("UTF8_STRING".to_string());
                data_source.offer("TEXT".to_string());
            }

            data_source.offer(mime_type.clone());
        };

        if primary {
            device.set_primary_selection(Some(&data_source));
        } else {
            device.set_selection(Some(&data_source));
        }

        // If we need to serve 0 requests, kill the data source right away.
        if let ServeRequests::Only(0) = serve_requests.get() {
            data_source.destroy();
        }

        sources.push(data_source.into());
    };
    let sources = sources;

    // Loop until we're done.
    while !should_quit.get() {
        queue.dispatch().map_err(Error::WaylandCommunication)?;

        // Check if all sources have been destroyed.
        let all_destroyed = sources.iter().all(|x| !x.is_alive());
        if all_destroyed {
            should_quit.set(true);
        }
    }

    // Clean up the temp file and directory.
    for (_, data_path) in data_paths.iter() {
        let mut data_path = data_path.borrow_mut();
        remove_file(&*data_path).map_err(Error::TempFileRemove)?;
        data_path.pop();
        remove_dir(&*data_path).map_err(Error::TempDirRemove)?;
    };

    // Check if an error occurred during data transfer.
    if let Some(err) = error.borrow_mut().take() {
        return Err(Error::Paste(err));
    }

    Ok(())
}

/// Copies data to the clipboard.
///
/// The data is copied from `source` and offered in the `mime_type` MIME type. See `Options` for
/// customizing the behavior of this operation.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::copy::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::copy::{copy, MimeType, Options, Source};
///
/// let opts = Options::new();
/// copy(opts, Source::Bytes(&[1, 2, 3]), MimeType::Autodetect)?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn copy(options: Options<'_>, source: Source<'_>, mime_type: MimeType) -> Result<(), Error> {
    let sources = vec![MimeSource { source: source, mime: mime_type}];
    copy_internal(options, sources, None, true)
}

/// Copies multiple data to the clipboard.
#[inline]
pub fn copy_multi(options: Options<'_>, sources: Vec<MimeSource>) -> Result<(), Error> {
    copy_internal(options, sources, None, false)
}

pub(crate) fn copy_internal(options: Options<'_>,
                            sources: Vec<MimeSource>,
                            socket_name: Option<OsString>,
                            create_multi_text: bool)
                            -> Result<(), Error> {
    let Options { clipboard,
                  seat,
                  trim_newline,
                  foreground,
                  serve_requests, } = options;

    let primary = clipboard != ClipboardType::Regular;
    let (queue, clipboard_manager, devices) = get_devices(primary, seat, socket_name)?;

    // Collect the source data to copy.
    let mut data_paths = HashMap::new();
    for mimesource in sources {
        let (mime_type, data_path) =
            make_source(mimesource.source, mimesource.mime, trim_newline).map_err(Error::TempCopy)?;
        data_paths.insert(mime_type, Rc::new(RefCell::new(data_path)));
    };

    if !foreground {
        // Fork an exit the parent.
        if let ForkResult::Parent { .. } = fork().map_err(Error::Fork)? {
            return Ok(());
        }
    }
    // If we forked, we must not return back past this point, just exit the process.

    let result = copy_past_fork(clipboard,
                                serve_requests,
                                queue,
                                clipboard_manager,
                                devices,
                                data_paths,
                                create_multi_text);

    if foreground {
        return result;
    }

    if let Err(err) = result {
        // TODO: print causes.
        eprintln!("Error: {}", err);
        process::exit(1);
    } else {
        process::exit(0);
    }
}
