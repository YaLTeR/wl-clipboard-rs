//! Copying and clearing clipboard contents.

use std::{
    cell::{Cell, RefCell},
    collections::{hash_map::Entry, HashMap, HashSet},
    ffi::OsString,
    fs::{remove_dir, remove_file, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    iter,
    os::unix::io::IntoRawFd,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::sync_channel,
    thread,
};

use failure::Fail;
use log::info;
use wayland_client::{ConnectError, EventQueue, Proxy};
use wayland_protocols::wlr::unstable::data_control::v1::client::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
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
#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Source {
    /// Copy contents of the standard input.
    StdIn,
    /// Copy the given bytes.
    Bytes(Box<[u8]>),
}

/// Source for copying, with a MIME type.
///
/// Used for [`copy_multi`].
///
/// [`copy_multi`]: fn.copy_multi.html
#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub struct MimeSource {
    pub source: Source,
    pub mime_type: MimeType,
}

/// Seat to operate on.
#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Seat {
    /// Operate on all existing seats at once.
    All,
    /// Operate on a seat with the given name.
    Specific(String),
}

impl Default for Seat {
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
#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, PartialOrd, Ord)]
pub struct Options {
    /// The clipboard to work with.
    clipboard: ClipboardType,

    /// The seat to work with.
    seat: Seat,

    /// Trim the trailing newline character before copying.
    ///
    /// This flag is only applied for text MIME types.
    trim_newline: bool,

    /// Do not spawn a separate thread for serving copy requests.
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

/// A copy operation ready to start serving requests.
pub struct PreparedCopy {
    should_quit: Rc<Cell<bool>>,
    queue: EventQueue,
    sources: Vec<Proxy<ZwlrDataControlSourceV1>>,
    data_paths: HashMap<String, Rc<RefCell<PathBuf>>>,
    error: Rc<RefCell<Option<DataSourceError>>>,
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

impl Options {
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
    pub fn seat(&mut self, seat: Seat) -> &mut Self {
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

    /// Sets the flag for not spawning a separate thread for serving copy requests.
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
    /// opts.copy(Source::Bytes([1, 2, 3][..].into()), MimeType::Autodetect)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn copy(self, source: Source, mime_type: MimeType) -> Result<(), Error> {
        copy(self, source, mime_type)
    }

    /// Invokes the copy_multi operation. See `copy_multi()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate wl_clipboard_rs;
    /// # use wl_clipboard_rs::copy::Error;
    /// # fn foo() -> Result<(), Error> {
    /// use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, Source};
    ///
    /// let opts = Options::new();
    /// opts.copy_multi(vec![MimeSource { source: Source::Bytes([1, 2, 3][..].into()),
    ///                                   mime_type: MimeType::Autodetect },
    ///                      MimeSource { source: Source::Bytes([7, 8, 9][..].into()),
    ///                                   mime_type: MimeType::Text }])?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn copy_multi(self, sources: Vec<MimeSource>) -> Result<(), Error> {
        copy_multi(self, sources)
    }

    /// Invokes the prepare_copy operation. See `prepare_copy()`.
    ///
    /// # Panics
    ///
    /// Panics if `foreground` is `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate wl_clipboard_rs;
    /// # use wl_clipboard_rs::copy::Error;
    /// # fn foo() -> Result<(), Error> {
    /// use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, Source};
    ///
    /// let mut opts = Options::new();
    /// opts.foreground(true);
    /// let prepared_copy = opts.prepare_copy(Source::Bytes([1, 2, 3][..].into()),
    ///                                       MimeType::Autodetect)?;
    /// prepared_copy.serve()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn prepare_copy(self, source: Source, mime_type: MimeType) -> Result<PreparedCopy, Error> {
        prepare_copy(self, source, mime_type)
    }

    /// Invokes the prepare_copy_multi operation. See `prepare_copy_multi()`.
    ///
    /// # Panics
    ///
    /// Panics if `foreground` is `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate wl_clipboard_rs;
    /// # use wl_clipboard_rs::copy::Error;
    /// # fn foo() -> Result<(), Error> {
    /// use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, Source};
    ///
    /// let mut opts = Options::new();
    /// opts.foreground(true);
    /// let prepared_copy =
    ///     opts.prepare_copy_multi(vec![MimeSource { source: Source::Bytes([1, 2, 3][..].into()),
    ///                                               mime_type: MimeType::Autodetect },
    ///                                  MimeSource { source: Source::Bytes([7, 8, 9][..].into()),
    ///                                               mime_type: MimeType::Text }])?;
    /// prepared_copy.serve()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn prepare_copy_multi(self, sources: Vec<MimeSource>) -> Result<PreparedCopy, Error> {
        prepare_copy_multi(self, sources)
    }
}

impl PreparedCopy {
    /// Starts serving copy requests.
    ///
    /// This function **blocks** until all requests are served or the clipboard is taken over by
    /// some other application.
    pub fn serve(mut self) -> Result<(), Error> {
        // Loop until we're done.
        while !self.should_quit.get() {
            self.queue.dispatch().map_err(Error::WaylandCommunication)?;

            // Check if all sources have been destroyed.
            let all_destroyed = self.sources.iter().all(|x| !x.is_alive());
            if all_destroyed {
                self.should_quit.set(true);
            }
        }

        // Clean up the temp file and directory.
        //
        // We want to try cleaning up all files and folders, so if any errors occur in process,
        // collect them into a vector without interruption, and then return the first one.
        let mut results = Vec::new();
        let mut dropped = HashSet::new();
        for data_path in self.data_paths.values() {
            let buf = data_path.as_ptr();
            // data_paths can contain duplicate items, we want to free each only once.
            if dropped.contains(&buf) {
                continue;
            };
            dropped.insert(buf);

            let mut data_path = data_path.borrow_mut();
            match remove_file(&*data_path).map_err(Error::TempFileRemove) {
                Ok(()) => {
                    data_path.pop();
                    results.push(remove_dir(&*data_path).map_err(Error::TempDirRemove));
                }
                result @ Err(_) => results.push(result),
            }
        }

        // Return the error, if any.
        let result: Result<_, _> = results.into_iter().collect();
        result?;

        // Check if an error occurred during data transfer.
        if let Some(err) = self.error.borrow_mut().take() {
            return Err(Error::Paste(err));
        }

        Ok(())
    }
}

fn make_source(source: Source,
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
    seat: Seat,
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
pub fn clear(clipboard: ClipboardType, seat: Seat) -> Result<(), Error> {
    clear_internal(clipboard, seat, None)
}

pub(crate) fn clear_internal(clipboard: ClipboardType,
                             seat: Seat,
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

/// Prepares a data copy to the clipboard.
///
/// The data is copied from `source` and offered in the `mime_type` MIME type. See `Options` for
/// customizing the behavior of this operation.
///
/// This function can be used instead of `copy()` when it's desirable to separately prepare the
/// copy operation, handle any errors that this may produce, and then start the serving loop,
/// potentially past a fork (which is how `wl-copy` uses it). It is meant to be used in the
/// foreground mode and does not spawn any threads.
///
/// # Panics
///
/// Panics if `foreground` is `false`.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::copy::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, Source};
///
/// let mut opts = Options::new();
/// opts.foreground(true);
/// let prepared_copy = opts.prepare_copy(Source::Bytes([1, 2, 3][..].into()),
///                                       MimeType::Autodetect)?;
/// prepared_copy.serve()?;
///
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn prepare_copy(options: Options,
                    source: Source,
                    mime_type: MimeType)
                    -> Result<PreparedCopy, Error> {
    assert_eq!(options.foreground, true);

    let sources = vec![MimeSource { source: source,
                                    mime_type: mime_type }];

    prepare_copy_internal(options, sources, None)
}

/// Prepares a data copy to the clipboard, offering multiple data sources.
///
/// The data from each source in `sources` is copied and offered in the corresponding MIME type.
/// See `Options` for customizing the behavior of this operation.
///
/// If multiple sources specify the same MIME type, the first one is offered. If one of the MIME
/// types is text, all automatically added plain text offers will fall back to the first source
/// with a text MIME type.
///
/// This function can be used instead of `copy()` when it's desirable to separately prepare the
/// copy operation, handle any errors that this may produce, and then start the serving loop,
/// potentially past a fork (which is how `wl-copy` uses it). It is meant to be used in the
/// foreground mode and does not spawn any threads.
///
/// # Panics
///
/// Panics if `foreground` is `false`.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::copy::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, Source};
///
/// let mut opts = Options::new();
/// opts.foreground(true);
/// let prepared_copy =
///     opts.prepare_copy_multi(vec![MimeSource { source: Source::Bytes([1, 2, 3][..].into()),
///                                               mime_type: MimeType::Autodetect },
///                                  MimeSource { source: Source::Bytes([7, 8, 9][..].into()),
///                                               mime_type: MimeType::Text }])?;
/// prepared_copy.serve()?;
///
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn prepare_copy_multi(options: Options,
                          sources: Vec<MimeSource>)
                          -> Result<PreparedCopy, Error> {
    assert_eq!(options.foreground, true);

    prepare_copy_internal(options, sources, None)
}

fn prepare_copy_internal(options: Options,
                         sources: Vec<MimeSource>,
                         socket_name: Option<OsString>)
                         -> Result<PreparedCopy, Error> {
    let Options { clipboard,
                  seat,
                  trim_newline,
                  serve_requests,
                  .. } = options;

    let primary = clipboard != ClipboardType::Regular;
    let (queue, clipboard_manager, devices) = get_devices(primary, seat, socket_name)?;

    // Collect the source data to copy.
    let data_paths = {
        let mut data_paths = HashMap::new();
        let mut text_data_path = None;
        for MimeSource { source, mime_type } in sources.into_iter() {
            let (mime_type, mut data_path) =
                make_source(source, mime_type, trim_newline).map_err(Error::TempCopy)?;

            let mime_type_is_text = is_text(&mime_type);

            match data_paths.entry(mime_type) {
                Entry::Occupied(_) => {
                    // This MIME type has already been specified, so ignore it.
                    remove_file(&*data_path).map_err(Error::TempFileRemove)?;
                    data_path.pop();
                    remove_dir(&*data_path).map_err(Error::TempDirRemove)?;
                }
                Entry::Vacant(entry) => {
                    let data_path = Rc::new(RefCell::new(data_path));

                    if text_data_path.is_none() && mime_type_is_text {
                        text_data_path = Some(data_path.clone());
                    }

                    entry.insert(data_path);
                }
            }
        }

        // If the MIME type is text, offer it in some other common formats.
        if let Some(text_data_path) = text_data_path {
            let text_mimes = ["text/plain;charset=utf-8",
                              "text/plain",
                              "STRING",
                              "UTF8_STRING",
                              "TEXT"];
            for &mime_type in &text_mimes {
                // We don't want to overwrite an explicit mime type, because it might be bound to a
                // different data_path
                if !data_paths.contains_key(mime_type) {
                    data_paths.insert(mime_type.to_string(), text_data_path.clone());
                }
            }
        }
        data_paths
    };

    // This bool can be set to true when serving a request: either if an error occurs, or if the
    // number of requests to serve was limited and the last request was served.
    let should_quit = Rc::new(Cell::new(false));
    // An error that occurred while serving a request, if any.
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
    let sources = devices_iter.map(|(device, primary)| {
                                  let handler = DataSourceHandler::new(data_paths.clone(),
                                                                       should_quit.clone(),
                                                                       serve_requests.clone());
                                  let data_source = clipboard_manager.create_data_source(|source| {
                                                        source.implement(handler, error.clone())
                                                    })
                                                    .unwrap();

                                  for mime_type in data_paths.keys() {
                                      data_source.offer(mime_type.clone());
                                  }

                                  if primary {
                                      device.set_primary_selection(Some(&data_source));
                                  } else {
                                      device.set_selection(Some(&data_source));
                                  }

                                  // If we need to serve 0 requests, kill the data source right away.
                                  if let ServeRequests::Only(0) = serve_requests.get() {
                                      data_source.destroy();
                                  }

                                  data_source.into()
                              })
                              .collect::<Vec<Proxy<_>>>();

    Ok(PreparedCopy { should_quit,
                      queue,
                      sources,
                      data_paths,
                      error })
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
/// copy(opts, Source::Bytes([1, 2, 3][..].into()), MimeType::Autodetect)?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn copy(options: Options, source: Source, mime_type: MimeType) -> Result<(), Error> {
    let sources = vec![MimeSource { source: source,
                                    mime_type: mime_type }];
    copy_internal(options, sources, None)
}

/// Copies data to the clipboard, offering multiple data sources.
///
/// The data from each source in `sources` is copied and offered in the corresponding MIME type.
/// See `Options` for customizing the behavior of this operation.
///
/// If multiple sources specify the same MIME type, the first one is offered. If one of the MIME
/// types is text, all automatically added plain text offers will fall back to the first source
/// with a text MIME type.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::copy::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::copy::{MimeSource, MimeType, Options, Source};
///
/// let opts = Options::new();
/// opts.copy_multi(vec![MimeSource { source: Source::Bytes([1, 2, 3][..].into()),
///                                   mime_type: MimeType::Autodetect },
///                      MimeSource { source: Source::Bytes([7, 8, 9][..].into()),
///                                   mime_type: MimeType::Text }])?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn copy_multi(options: Options, sources: Vec<MimeSource>) -> Result<(), Error> {
    copy_internal(options, sources, None)
}

pub(crate) fn copy_internal(options: Options,
                            sources: Vec<MimeSource>,
                            socket_name: Option<OsString>)
                            -> Result<(), Error> {
    if options.foreground {
        prepare_copy_internal(options, sources, socket_name)?.serve()
    } else {
        // The copy must be prepared on the thread because PreparedCopy isn't Send.
        // To receive errors from prepare_copy, use a channel.
        let (tx, rx) = sync_channel(1);

        thread::spawn(move || match prepare_copy_internal(options, sources, socket_name) {
                          Ok(prepared_copy) => {
                              // prepare_copy completed successfully, report that.
                              drop(tx.send(None));

                              // There's nobody listening for errors at this point, just drop it.
                              drop(prepared_copy.serve());
                          }
                          Err(err) => drop(tx.send(Some(err))),
                      });

        if let Some(err) = rx.recv().unwrap() {
            return Err(err);
        }

        Ok(())
    }
}
