//! Helper functions.

use std::ffi::{CString, OsString};
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::abort;
use std::{env, io};

use libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execv, fork, ForkResult};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{
    event_created_child, ConnectError, Connection, Dispatch, DispatchError, Proxy,
};
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_device_v1::{
    self, ZwlrDataControlDeviceV1,
};
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1;
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1;

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
#[derive(thiserror::Error, Debug)]
pub enum CopyDataError {
    #[error("Couldn't set the source file descriptor flags")]
    SetSourceFdFlags(#[source] nix::Error),

    #[error("Couldn't set the target file descriptor flags")]
    SetTargetFdFlags(#[source] nix::Error),

    #[error("Couldn't fork")]
    Fork(#[source] nix::Error),

    #[error("Couldn't close the source file descriptor")]
    CloseSourceFd(#[source] nix::Error),

    #[error("Couldn't close the target file descriptor")]
    CloseTargetFd(#[source] nix::Error),

    #[error("Couldn't wait for the child process")]
    Wait(#[source] nix::Error),

    #[error(
        "Received an unexpected status when waiting for the child process: {:?}",
        _0
    )]
    WaitUnexpected(WaitStatus),

    #[error("The child process exited with a non-zero error code: {}", _0)]
    ChildError(i32),
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
        fcntl(from_fd, FcntlArg::F_SETFL(OFlag::empty()))
            .map_err(CopyDataError::SetSourceFdFlags)?;
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

struct PrimarySelectionState {
    // Any seat that we get from the compositor.
    seat: Option<WlSeat>,
    clipboard_manager: Option<ZwlrDataControlManagerV1>,
    clipboard_manager_was_v1: bool,
    got_primary_selection: bool,
}

impl Dispatch<WlRegistry, ()> for PrimarySelectionState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == WlSeat::interface().name && version >= 2 && state.seat.is_none() {
                let seat = registry.bind(name, 2, qh, ());
                state.seat = Some(seat);
            }

            if interface == ZwlrDataControlManagerV1::interface().name {
                assert_eq!(state.clipboard_manager, None);

                if version == 1 {
                    state.clipboard_manager_was_v1 = true;
                } else {
                    let manager = registry.bind(name, 2, qh, ());
                    state.clipboard_manager = Some(manager);
                }
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for PrimarySelectionState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for PrimarySelectionState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrDataControlManagerV1,
        _event: <ZwlrDataControlManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrDataControlDeviceV1, ()> for PrimarySelectionState {
    fn event(
        state: &mut Self,
        _device: &ZwlrDataControlDeviceV1,
        event: <ZwlrDataControlDeviceV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let zwlr_data_control_device_v1::Event::PrimarySelection { id: _ } = event {
            state.got_primary_selection = true;
        }
    }

    event_created_child!(PrimarySelectionState, ZwlrDataControlDeviceV1, [
        zwlr_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ZwlrDataControlOfferV1, ()),
    ]);
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for PrimarySelectionState {
    fn event(
        _state: &mut Self,
        _offer: &ZwlrDataControlOfferV1,
        _event: <ZwlrDataControlOfferV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

/// Errors that can occur when checking whether the primary selection is supported.
#[derive(thiserror::Error, Debug)]
pub enum PrimarySelectionCheckError {
    #[error("There are no seats")]
    NoSeats,

    #[error("Couldn't open the provided Wayland socket")]
    SocketOpenError(#[source] io::Error),

    #[error("Couldn't connect to the Wayland compositor")]
    WaylandConnection(#[source] ConnectError),

    #[error("Wayland compositor communication error")]
    WaylandCommunication(#[source] DispatchError),

    #[error(
        "A required Wayland protocol ({} version {}) is not supported by the compositor",
        name,
        version
    )]
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

pub(crate) fn is_primary_selection_supported_internal(
    socket_name: Option<OsString>,
) -> Result<bool, PrimarySelectionCheckError> {
    // Connect to the Wayland compositor.
    let conn = match socket_name {
        Some(name) => {
            let mut socket_path = env::var_os("XDG_RUNTIME_DIR")
                .map(Into::<PathBuf>::into)
                .ok_or(ConnectError::NoCompositor)
                .map_err(PrimarySelectionCheckError::WaylandConnection)?;
            if !socket_path.is_absolute() {
                return Err(PrimarySelectionCheckError::WaylandConnection(
                    ConnectError::NoCompositor,
                ));
            }
            socket_path.push(name);

            let stream = UnixStream::connect(socket_path)
                .map_err(PrimarySelectionCheckError::SocketOpenError)?;
            Connection::from_socket(stream)
        }
        None => Connection::connect_to_env(),
    }
    .map_err(PrimarySelectionCheckError::WaylandConnection)?;
    let display = conn.display();

    let mut queue = conn.new_event_queue();
    let qh = queue.handle();

    let mut state = PrimarySelectionState {
        seat: None,
        clipboard_manager: None,
        clipboard_manager_was_v1: false,
        got_primary_selection: false,
    };

    // Retrieve the global interfaces.
    let _registry = display.get_registry(&qh, ());
    queue
        .roundtrip(&mut state)
        .map_err(PrimarySelectionCheckError::WaylandCommunication)?;

    // If data control is present but is version 1, then return false as version 1 does not support primary clipboard.
    if state.clipboard_manager_was_v1 {
        return Ok(false);
    }

    // Verify that we got the clipboard manager.
    let Some(ref clipboard_manager) = state.clipboard_manager else {
        return Err(PrimarySelectionCheckError::MissingProtocol { name: ZwlrDataControlManagerV1::interface().name,
                                                                 version: 1 });
    };

    // Check if there are no seats.
    let Some(ref seat) = state.seat else {
        return Err(PrimarySelectionCheckError::NoSeats);
    };

    clipboard_manager.get_data_device(seat, &qh, ());

    queue
        .roundtrip(&mut state)
        .map_err(PrimarySelectionCheckError::WaylandCommunication)?;

    Ok(state.got_primary_selection)
}
