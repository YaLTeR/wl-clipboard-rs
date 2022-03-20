//! Helper functions.

use std::{
    cell::{Cell, RefCell},
    ffi::{CString, OsString},
    io,
    os::unix::io::RawFd,
    process::abort,
    rc::Rc,
};

use libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::{
    fcntl::{fcntl, FcntlArg, OFlag},
    sys::wait::{waitpid, WaitStatus},
    unistd::{close, dup2, execv, fork, ForkResult},
};
use wayland_client::{
    global_filter, protocol::wl_seat::WlSeat, ConnectError, Display, GlobalError, GlobalManager, Interface, Main,
};
use wayland_protocols::wlr::unstable::data_control::v1::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1;

use crate::{
    handlers::{data_device_handler, seat_handler, DataDeviceHandler},
    seat_data::SeatData,
};

/// Checks if the given MIME type represents plain text.
///
/// # Examples
///
/// ```
/// use wl_clipboard_rs::utils::is_text;
///
/// assert!(is_text("text/plain"));
/// assert!(!is_text("application/octet-stream"));
/// ```
pub fn is_text(mime_type: &str) -> bool {
    match mime_type {
        "TEXT" | "STRING" | "UTF8_STRING" => true,
        x if x.starts_with("text/") => true,
        _ => false,
    }
}

/// Errors that can occur in `copy_data()`.
#[derive(derive_more::Error, derive_more::Display, Debug)]
pub enum CopyDataError {
    #[display(fmt = "Couldn't set the source file descriptor flags")]
    SetSourceFdFlags(#[error(source)] nix::Error),

    #[display(fmt = "Couldn't set the target file descriptor flags")]
    SetTargetFdFlags(#[error(source)] nix::Error),

    #[display(fmt = "Couldn't fork")]
    Fork(#[error(source)] nix::Error),

    #[display(fmt = "Couldn't close the source file descriptor")]
    CloseSourceFd(#[error(source)] nix::Error),

    #[display(fmt = "Couldn't close the target file descriptor")]
    CloseTargetFd(#[error(source)] nix::Error),

    #[display(fmt = "Couldn't wait for the child process")]
    Wait(#[error(source)] nix::Error),

    #[display(fmt = "Received an unexpected status when waiting for the child process: {:?}", _0)]
    WaitUnexpected(#[error(ignore)] WaitStatus),

    #[display(fmt = "The child process exited with a non-zero error code: {}", _0)]
    ChildError(#[error(ignore)] i32),
}

/// Copies data from one file to another.
///
/// This function assumes ownership of the passed file descriptors. That is, it closes them by
/// itself. Use `into_raw_fd()`.
///
/// If `from_fd` is `None`, the standard input is used as the data source.
///
/// If `wait` is `true`, this function returns after all data has been copied, otherwise it may
/// return before all data has been copied.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use std::{fs::File, os::unix::io::IntoRawFd};
/// use wl_clipboard_rs::utils::copy_data;
///
/// let file = File::create("stdin-contents")?;
///
/// // Copy the standard input into the file.
/// copy_data(None, file.into_raw_fd(), true)?;
/// # Ok(())
/// # }
/// ```
#[allow(unsafe_code)]
pub fn copy_data(from_fd: Option<RawFd>, to_fd: RawFd, wait: bool) -> Result<(), CopyDataError> {
    // We use the cat utility for data copying. It's easier (no need to implement any complex
    // buffering logic), surprisingly safer (a Rust implementation would likely require usage of
    // `from_raw_fd()` which is unsafe) and ideally faster (cat's been around for a while and is
    // probably pretty optimized).

    // Clear O_NONBLOCK because cat doesn't know how to deal with it.
    if let Some(from_fd) = from_fd {
        fcntl(from_fd, FcntlArg::F_SETFL(OFlag::empty())).map_err(CopyDataError::SetSourceFdFlags)?;
    }
    fcntl(to_fd, FcntlArg::F_SETFL(OFlag::empty())).map_err(CopyDataError::SetTargetFdFlags)?;

    // Don't allocate memory in the child process, it's not async-signal-safe.
    let bin_env = CString::new("/usr/bin/env").unwrap();
    let env = CString::new("env").unwrap();
    let cat = CString::new("cat").unwrap();

    // Fork and exec cat.
    // SAFETY: Within the child, we are only using the following system calls: dup2, close, execv
    // As required by the safety of `fork`, these are all [async-signal-safe](https://man7.org/linux/man-pages/man7/signal-safety.7.html).
    let fork_result = unsafe { fork() }.map_err(CopyDataError::Fork)?;
    match fork_result {
        ForkResult::Child => {
            if let Some(fd) = from_fd {
                // Redirect the "from" fd to stdin.
                if dup2(fd, STDIN_FILENO).is_err() {
                    abort();
                }
            }

            // Redirect stdout to the "to" fd.
            if dup2(to_fd, STDOUT_FILENO).is_err() {
                abort();
            }

            // Close the original fds.
            if let Some(fd) = from_fd {
                if close(fd).is_err() {
                    abort();
                }
            }

            if close(to_fd).is_err() {
                abort();
            }

            // Exec cat.
            if execv(&bin_env, &[&env, &cat]).is_err() {
                abort();
            }
        }
        ForkResult::Parent { child } => {
            // Close the fds in the parent process.
            if let Some(fd) = from_fd {
                close(fd).map_err(CopyDataError::CloseSourceFd)?;
            }

            close(to_fd).map_err(CopyDataError::CloseTargetFd)?;

            if wait {
                // Wait for the child process to exit.
                match waitpid(child, None).map_err(CopyDataError::Wait)? {
                    WaitStatus::Exited(_, status) => {
                        if status != 0 {
                            return Err(CopyDataError::ChildError(status));
                        }
                    }
                    x => return Err(CopyDataError::WaitUnexpected(x)),
                }
            }
        }
    }

    Ok(())
}

/// Errors that can occur when checking whether the primary selection is supported.
#[derive(derive_more::Error, derive_more::Display, Debug)]
pub enum PrimarySelectionCheckError {
    #[display(fmt = "There are no seats")]
    NoSeats,

    #[display(fmt = "Couldn't connect to the Wayland compositor")]
    WaylandConnection(#[error(source)] ConnectError),

    #[display(fmt = "Wayland compositor communication error")]
    WaylandCommunication(#[error(source)] io::Error),

    #[display(fmt = "A required Wayland protocol ({} version {}) is not supported by the compositor",
              name,
              version)]
    MissingProtocol { name: &'static str, version: u32 },
}

/// Checks if the compositor supports the primary selection.
///
/// # Examples
///
/// ```no_run
/// # extern crate wl_clipboard_rs;
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use wl_clipboard_rs::utils::{is_primary_selection_supported, PrimarySelectionCheckError};
///
/// match is_primary_selection_supported() {
///     Ok(supported) => {
///         // We have our definitive result. False means that either data-control version 1
///         // is present (which does not support the primary selection), or that data-control
///         // version 2 is present and it did not signal the primary selection support.
///     },
///     Err(PrimarySelectionCheckError::NoSeats) => {
///         // Impossible to give a definitive result. Primary selection may or may not be
///         // supported.
///
///         // The required protocol (data-control version 2) is there, but there are no seats.
///         // Unfortunately, at least one seat is needed to check for the primary clipboard
///         // support.
///     },
///     Err(PrimarySelectionCheckError::MissingProtocol { .. }) => {
///         // The data-control protocol (required for wl-clipboard-rs operation) is not
///         // supported by the compositor.
///     },
///     Err(_) => {
///         // Some communication error occurred.
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn is_primary_selection_supported() -> Result<bool, PrimarySelectionCheckError> {
    is_primary_selection_supported_internal(None)
}

pub(crate) fn is_primary_selection_supported_internal(socket_name: Option<OsString>)
                                                      -> Result<bool, PrimarySelectionCheckError> {
    // Connect to the Wayland compositor.
    let display = match socket_name {
                      Some(name) => Display::connect_to_name(name),
                      None => Display::connect_to_env(),
                  }.map_err(PrimarySelectionCheckError::WaylandConnection)?;
    let mut queue = display.create_event_queue();
    let display = display.attach(queue.token());

    let seats = Rc::new(RefCell::new(Vec::<Main<WlSeat>>::new()));

    let seats_2 = seats.clone();
    let global_manager =
        GlobalManager::new_with_cb(&display,
                                   global_filter!([WlSeat, 2, move |seat: Main<WlSeat>, _: DispatchData| {
                                                      let seat_data = RefCell::new(SeatData::default());
                                                      seat.as_ref().user_data().set(move || seat_data);
                                                      seat.quick_assign(seat_handler);
                                                      seats_2.borrow_mut().push(seat);
                                                  }]));

    // Retrieve the global interfaces.
    queue.sync_roundtrip(&mut (), |_, _, _| {})
         .map_err(PrimarySelectionCheckError::WaylandCommunication)?;

    // Try instantiating data control version 2. If data control is missing altogether, return a
    // missing protocol error, but if it's present with version 1 then return false as version 1
    // does not support primary clipboard.
    let clipboard_manager = match global_manager.instantiate_exact::<ZwlrDataControlManagerV1>(2) {
        Ok(manager) => manager,
        Err(GlobalError::Missing) => {
            return Err(PrimarySelectionCheckError::MissingProtocol { name: ZwlrDataControlManagerV1::NAME,
                                                                     version: 1 })
        }
        Err(GlobalError::VersionTooLow(_)) => return Ok(false),
    };

    // Check if there are no seats.
    if seats.borrow_mut().is_empty() {
        return Err(PrimarySelectionCheckError::NoSeats);
    }

    let supports_primary = Rc::new(Cell::new(false));

    // Go through the seats and get their data devices. They will listen for the primary_selection
    // event and set supports_primary on receiving one.
    for seat in &*seats.borrow_mut() {
        let mut handler = DataDeviceHandler::new(seat.detach(), true, supports_primary.clone());
        let device = clipboard_manager.get_data_device(seat);
        device.quick_assign(move |data_device, event, dispatch_data| {
                  data_device_handler(&mut handler, data_device, event, dispatch_data)
              });
    }

    queue.sync_roundtrip(&mut (), |_, _, _| {})
         .map_err(PrimarySelectionCheckError::WaylandCommunication)?;

    Ok(supports_primary.get())
}
