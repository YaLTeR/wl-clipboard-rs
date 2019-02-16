//! A safe Rust crate for working with the Wayland clipboard.
//!
//! This crate is intended to be used by terminal applications, clipboard managers and other
//! utilities which don't spawn Wayland surfaces (windows). If your application has a window,
//! please use the appropriate Wayland protocols for interacting with the Wayland clipboard
//! (`wl_data_device` from the core Wayland protocol, the `primary_selection` protocol for the
//! primary selection), for example via the
//! [smithay-clipboard](https://github.com/Smithay/smithay-clipboard) crate.
//!
//! The protocol used for clipboard interaction is `data-control` from
//! [wlroots](https://github.com/swaywm/wlr-protocols). When using the regular clipboard, the
//! compositor must support the first version of the protocol. When using the "primary" clipboard,
//! the compositor must support the second version of the protocol (or higher).
//!
//! For example applications using these features, see `src/bin/wl_copy.rs` and
//! `src/bin/wl_paste.rs` which implement terminal apps similar to
//! [wl-clipboard](https://github.com/bugaevc/wl-clipboard).
//!
//! The Rust implementation of the Wayland client is used by default; use the `native_lib` feature
//! to link to `libwayland-client.so` for communication instead. A `dlopen` feature is also
//! available for loading `libwayland-client.so` dynamically at runtime rather than linking to it.
//!
//! The code of the crate itself (and the code of the example utilities) is 100% safe Rust. This
//! doesn't include the dependencies.
//!
//! # Examples
//!
//! Copying to the regular clipboard:
//! ```no_run
//! # extern crate wl_clipboard_rs;
//! # use wl_clipboard_rs::copy::Error;
//! # fn foo() -> Result<(), Error> {
//! use wl_clipboard_rs::copy::{MimeType, Options, Source};
//!
//! let opts = Options::new();
//! opts.copy(Source::Bytes("Hello world!".as_bytes()), MimeType::Autodetect)?;
//! # Ok(())
//! # }
//! ```
//!
//! Pasting plain text from the regular clipboard:
//! ```no_run
//! # extern crate wl_clipboard_rs;
//! # extern crate failure;
//! # use failure::Error;
//! # fn foo() -> Result<(), Error> {
//! use std::io::Read;
//! use wl_clipboard_rs::{paste::{get_contents, Error, MimeType, Seat}, ClipboardType};
//!
//! let result = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);
//! match result {
//!     Ok((mut pipe, _)) => {
//!         let mut contents = vec![];
//!         pipe.read_to_end(&mut contents)?;
//!         println!("Pasted: {}", String::from_utf8_lossy(&contents));
//!     }
//!
//!     Err(Error::NoSeats) | Err(Error::ClipboardEmpty) | Err(Error::NoMimeType) => {
//!         // The clipboard is empty or doesn't contain text, nothing to worry about.
//!     }
//!
//!     Err(err) => Err(err)?
//! }
//! # Ok(())
//! # }
//! ```

#![doc(html_root_url = "https://docs.rs/wl-clipboard-rs/0.1.0")]
#![deny(unsafe_code)]

mod common;
mod handlers;
mod seat_data;

#[cfg(test)]
#[allow(unsafe_code)] // It's more convenient for testing some stuff.
mod tests;

pub mod copy;
pub mod paste;
pub mod utils;

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
