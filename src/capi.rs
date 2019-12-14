use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    slice,
};

use crate::{
    paste,
    utils::{is_primary_selection_supported, PrimarySelectionCheckError},
};

#[repr(C)]
pub enum wcrs_primary_selection_check_status_t {
    WcrsPrimarySelectionCheckStatusSuccess = 0,
    WcrsPrimarySelectionCheckStatusNoSeats,
    WcrsPrimarySelectionCheckStatusWaylandConnection,
    WcrsPrimarySelectionCheckStatusWaylandCommunication,
    WcrsPrimarySelectionCheckStatusMissingProtocol,
}

#[repr(C)]
pub enum wcrs_clipboard_type_t {
    WcrsClipboardTypeRegular,
    WcrsClipboardTypePrimary,
}

#[repr(C)]
pub struct wcrs_mime_types_t {
    count: i32,
    mime_types: *const *const c_char,
}

#[repr(C)]
pub enum wcrs_paste_status_t {
    WcrsPasteStatusSuccess = 0,
    WcrsPasteStatusNoSeats,
    WcrsPasteStatusClipboardEmpty,
    WcrsPasteStatusNoMimeType,
    WcrsPasteStatusWaylandConnection,
    WcrsPasteStatusWaylandCommunication,
    WcrsPasteStatusMissingProtocol,
    WcrsPasteStatusPrimarySelectionUnsupported,
    WcrsPasteStatusSeatNotFound,
    WcrsPasteStatusPipeCreation,
}

impl From<PrimarySelectionCheckError> for wcrs_primary_selection_check_status_t {
    #[inline]
    fn from(x: PrimarySelectionCheckError) -> Self {
        use PrimarySelectionCheckError::*;
        match x {
            NoSeats => Self::WcrsPrimarySelectionCheckStatusNoSeats,
            WaylandConnection { .. } => Self::WcrsPrimarySelectionCheckStatusWaylandConnection,
            WaylandCommunication { .. } => {
                Self::WcrsPrimarySelectionCheckStatusWaylandCommunication
            }
            MissingProtocol { .. } => Self::WcrsPrimarySelectionCheckStatusMissingProtocol,
        }
    }
}

impl From<wcrs_clipboard_type_t> for paste::ClipboardType {
    #[inline]
    fn from(x: wcrs_clipboard_type_t) -> Self {
        use wcrs_clipboard_type_t::*;
        match x {
            WcrsClipboardTypeRegular => Self::Regular,
            WcrsClipboardTypePrimary => Self::Primary,
        }
    }
}

impl From<paste::Error> for wcrs_paste_status_t {
    #[inline]
    fn from(x: paste::Error) -> Self {
        use paste::Error::*;
        match x {
            NoSeats => Self::WcrsPasteStatusNoSeats,
            ClipboardEmpty => Self::WcrsPasteStatusClipboardEmpty,
            NoMimeType => Self::WcrsPasteStatusNoMimeType,
            WaylandConnection { .. } => Self::WcrsPasteStatusWaylandConnection,
            WaylandCommunication { .. } => Self::WcrsPasteStatusWaylandCommunication,
            MissingProtocol { .. } => Self::WcrsPasteStatusMissingProtocol,
            PrimarySelectionUnsupported => Self::WcrsPasteStatusPrimarySelectionUnsupported,
            SeatNotFound => Self::WcrsPasteStatusSeatNotFound,
            PipeCreation(_) => Self::WcrsPasteStatusPipeCreation,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcrs_mime_types_free(mime_types: *const wcrs_mime_types_t) {
    // Reconstruct our boxed slice.
    let wcrs_mime_types_t { count, mime_types } = *mime_types;
    let mime_types = slice::from_raw_parts_mut(mime_types as *mut *mut c_char, count as usize)
                     as *mut [*mut c_char];
    let mime_types = Box::from_raw(mime_types);

    // Drop the individual strings and then the box itself.
    for &mime_type in mime_types.iter() {
        drop(CString::from_raw(mime_type));
    }
    drop(mime_types);
}

#[no_mangle]
pub unsafe extern "C" fn wcrs_is_primary_selection_supported(
    supported: *mut i32)
    -> wcrs_primary_selection_check_status_t {
    match is_primary_selection_supported() {
        Ok(value) => {
            *supported = if value { 1 } else { 0 };
            wcrs_primary_selection_check_status_t::WcrsPrimarySelectionCheckStatusSuccess
        }
        Err(err) => err.into(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcrs_get_mime_types(clipboard_type: wcrs_clipboard_type_t,
                                             seat: *const c_char,
                                             mime_types: *mut wcrs_mime_types_t)
                                             -> wcrs_paste_status_t {
    let clipboard_type = clipboard_type.into();
    let seat = if seat.is_null() {
        paste::Seat::Unspecified
    } else {
        paste::Seat::Specific(CStr::from_ptr(seat).to_str().unwrap())
    };

    match paste::get_mime_types(clipboard_type, seat) {
        Ok(types) => {
            let types: Box<_> = types.into_iter()
                                     .map(CString::new)
                                     .map(Result::unwrap)
                                     .map(CString::into_raw)
                                     .collect();

            *mime_types = wcrs_mime_types_t { count: types.len() as i32,
                                              mime_types: Box::into_raw(types) as _ };

            wcrs_paste_status_t::WcrsPasteStatusSuccess
        }
        Err(err) => err.into(),
    }
}
