//! Getting the offered MIME types and the clipboard contents.

use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    ffi::OsString,
    io, mem,
    os::unix::io::AsRawFd,
    rc::Rc,
};

use failure::Fail;
use os_pipe::{pipe, PipeReader};
use wayland_client::{ConnectError, EventQueue};
use wayland_protocols::wlr::unstable::data_control::v1::client::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1;

use crate::{
    common::{self, initialize, CommonData},
    handlers::DataDeviceHandler,
    seat_data::SeatData,
    utils::is_text,
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
}

impl Default for ClipboardType {
    #[inline]
    fn default() -> Self {
        ClipboardType::Regular
    }
}

/// MIME types that can be requested from the clipboard.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum MimeType<'a> {
    /// Request any available MIME type.
    ///
    /// If multiple MIME types are offered, the requested MIME type is unspecified and depends on
    /// the order they are received from the Wayland compositor. However, plain text formats are
    /// prioritized, so if a plain text format is available among others then it will be requested.
    Any,
    /// Request a plain text MIME type.
    ///
    /// This will request one of the multiple common plain text MIME types. It will prioritize MIME
    /// types known to return UTF-8 text.
    Text,
    /// Request the given MIME type, and if it's not available fall back to `MimeType::Text`.
    ///
    /// Example use-case: pasting `text/html` should try `text/html` first, but if it's not
    /// available, any other plain text format will do fine too.
    TextWithPriority(&'a str),
    /// Request a specific MIME type.
    Specific(&'a str),
}

/// Seat to operate on.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Seat<'a> {
    /// Operate on one of the existing seats depending on the order returned by the compositor.
    ///
    /// This is perfectly fine when only a single seat is present, so for most configurations.
    Unspecified,
    /// Operate on a seat with the given name.
    Specific(&'a str),
}

impl Default for Seat<'_> {
    #[inline]
    fn default() -> Self {
        Seat::Unspecified
    }
}

/// Errors that can occur for pasting and listing MIME types.
///
/// You may want to ignore some of these errors (rather than show an error message), like
/// `NoSeats`, `ClipboardEmpty` or `NoMimeType` as they are essentially equivalent to an empty
/// clipboard.
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "There are no seats")]
    NoSeats,

    #[fail(display = "The clipboard of the requested seat is empty")]
    ClipboardEmpty,

    #[fail(display = "No suitable type of content copied")]
    NoMimeType,

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

    #[fail(display = "Couldn't create a pipe for content transfer")]
    PipeCreation(#[cause] io::Error),
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

fn get_offer(primary: bool,
             seat: Seat<'_>,
             socket_name: Option<OsString>)
             -> Result<(EventQueue, ZwlrDataControlOfferV1), Error> {
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
        let handler = DataDeviceHandler::new(seat.clone(), primary, supports_primary.clone());
        clipboard_manager.get_data_device(seat, |device| device.implement(handler, ()))
                         .unwrap();
    }

    // Retrieve all seat names and offers.
    queue.sync_roundtrip().map_err(Error::WaylandCommunication)?;

    // Check if the compositor supports primary selection.
    if primary && !supports_primary.get() {
        return Err(Error::PrimarySelectionUnsupported);
    }

    // Figure out which offer we're interested in.
    let offer = seats.borrow_mut()
                     .iter()
                     .map(|seat| seat.as_ref().user_data::<RefCell<SeatData>>().unwrap().borrow())
                     .find_map(|data| {
                         let SeatData { name, offer, .. } = &*data;
                         match seat {
                             Seat::Unspecified => return Some(offer.clone()),
                             Seat::Specific(desired_name) => {
                                 if let Some(name) = name {
                                     if name == desired_name {
                                         return Some(offer.clone());
                                     }
                                 }
                             }
                         }

                         None
                     });

    // Check if we found any seat.
    if offer.is_none() {
        return Err(Error::SeatNotFound);
    }

    offer.unwrap().map(|x| (queue, x)).ok_or(Error::ClipboardEmpty)
}

/// Retrieves the offered MIME types.
///
/// If `seat` is `None`, uses an unspecified seat (it depends on the order returned by the
/// compositor). This is perfectly fine when only a single seat is present, so for most
/// configurations.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # use wl_clipboard_rs::paste::Error;
/// # fn foo() -> Result<(), Error> {
/// use wl_clipboard_rs::{paste::{get_mime_types, ClipboardType, Seat}};
///
/// let mime_types = get_mime_types(ClipboardType::Regular, Seat::Unspecified)?;
/// for mime_type in mime_types {
///     println!("{}", mime_type);
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn get_mime_types(clipboard: ClipboardType, seat: Seat<'_>) -> Result<HashSet<String>, Error> {
    get_mime_types_internal(clipboard, seat, None)
}

// The internal function accepts the socket name, used for tests.
pub(crate) fn get_mime_types_internal(clipboard: ClipboardType,
                                      seat: Seat<'_>,
                                      socket_name: Option<OsString>)
                                      -> Result<HashSet<String>, Error> {
    let primary = clipboard == ClipboardType::Primary;
    let (_, offer) = get_offer(primary, seat, socket_name)?;

    let mut mime_types = offer.as_ref()
                              .user_data::<RefCell<HashSet<String>>>()
                              .unwrap()
                              .borrow_mut();

    let empty_hash_set = HashSet::new();
    Ok(mem::replace(&mut *mime_types, empty_hash_set))
}

/// Retrieves the clipboard contents.
///
/// This function returns a tuple of the reading end of a pipe containing the clipboard contents
/// and the actual MIME type of the contents.
///
/// If `seat` is `None`, uses an unspecified seat (it depends on the order returned by the
/// compositor). This is perfectly fine when only a single seat is present, so for most
/// configurations.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # extern crate failure;
/// # use failure::Error;
/// # fn foo() -> Result<(), Error> {
/// use std::io::Read;
/// use wl_clipboard_rs::{paste::{get_contents, ClipboardType, Error, MimeType, Seat}};
///
/// let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Any);
/// match result {
///     Ok((mut pipe, mime_type)) => {
///         println!("Got data of the {} MIME type", &mime_type);
///
///         let mut contents = vec![];
///         pipe.read_to_end(&mut contents)?;
///         println!("Read {} bytes of data", contents.len());
///     }
///
///     Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
///         // The clipboard is empty, nothing to worry about.
///     }
///
///     Err(err) => Err(err)?
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn get_contents(clipboard: ClipboardType,
                    seat: Seat<'_>,
                    mime_type: MimeType<'_>)
                    -> Result<(PipeReader, String), Error> {
    get_contents_internal(clipboard, seat, mime_type, None)
}

// The internal function accepts the socket name, used for tests.
pub(crate) fn get_contents_internal(clipboard: ClipboardType,
                                    seat: Seat<'_>,
                                    mime_type: MimeType<'_>,
                                    socket_name: Option<OsString>)
                                    -> Result<(PipeReader, String), Error> {
    let primary = clipboard == ClipboardType::Primary;
    let (mut queue, offer) = get_offer(primary, seat, socket_name)?;

    let mut mime_types = offer.as_ref()
                              .user_data::<RefCell<HashSet<String>>>()
                              .unwrap()
                              .borrow_mut();

    // Find the desired MIME type.
    let mime_type = match mime_type {
        MimeType::Any => mime_types.take("text/plain;charset=utf-8")
                                   .or_else(|| mime_types.take("UTF8_STRING"))
                                   .or_else(|| mime_types.iter().find(|x| is_text(x)).cloned())
                                   .or_else(|| mime_types.drain().next()),
        MimeType::Text => mime_types.take("text/plain;charset=utf-8")
                                    .or_else(|| mime_types.take("UTF8_STRING"))
                                    .or_else(|| mime_types.drain().find(|x| is_text(x))),
        MimeType::TextWithPriority(priority) => mime_types.take(priority)
                                                          .or_else(|| mime_types.take("text/plain;charset=utf-8"))
                                                          .or_else(|| mime_types.take("UTF8_STRING"))
                                                          .or_else(|| mime_types.drain().find(|x| is_text(x))),
        MimeType::Specific(mime_type) => mime_types.take(mime_type),
    };

    // Check if a suitable MIME type is copied.
    if mime_type.is_none() {
        return Err(Error::NoMimeType);
    }

    let mime_type = mime_type.unwrap();

    // Create a pipe for content transfer.
    let (read, write) = pipe().map_err(Error::PipeCreation)?;

    // Start the transfer.
    offer.receive(mime_type.clone(), write.as_raw_fd());
    drop(write);
    queue.sync_roundtrip().map_err(Error::WaylandCommunication)?;

    Ok((read, mime_type))
}
