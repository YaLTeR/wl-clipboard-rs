//! Helper functions.

use std::{ffi::CString, os::unix::io::RawFd, process::abort};

use failure::Fail;
use libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::{
    fcntl::{fcntl, FcntlArg, OFlag},
    sys::wait::{waitpid, WaitStatus},
    unistd::{close, dup2, execvp, fork, ForkResult},
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
#[derive(Fail, Debug)]
pub enum CopyDataError {
    #[fail(display = "Couldn't set the source file descriptor flags")]
    SetSourceFdFlags(#[cause] nix::Error),

    #[fail(display = "Couldn't set the target file descriptor flags")]
    SetTargetFdFlags(#[cause] nix::Error),

    #[fail(display = "Couldn't fork")]
    Fork(#[cause] nix::Error),

    #[fail(display = "Couldn't close the source file descriptor")]
    CloseSourceFd(#[cause] nix::Error),

    #[fail(display = "Couldn't close the target file descriptor")]
    CloseTargetFd(#[cause] nix::Error),

    #[fail(display = "Couldn't wait for the child process")]
    Wait(#[cause] nix::Error),

    #[fail(display = "Received an unexpected status when waiting for the child process: {:?}",
           _0)]
    WaitUnexpected(WaitStatus),

    #[fail(display = "The child process exited with a non-zero error code: {}",
           _0)]
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
/// # extern crate failure;
/// # use failure::Error;
/// # fn foo() -> Result<(), Error> {
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
pub fn copy_data(from_fd: Option<RawFd>, to_fd: RawFd, wait: bool) -> Result<(), CopyDataError> {
    // We use the cat utility for data copying. It's easier (no need to implement any comlpex
    // buffering logic), surprisingly safer (a Rust implementation would likely require usage of
    // `from_raw_fd()` which is unsafe) and ideally faster (cat's been around for a while and is
    // probably pretty optimized).

    // Clear O_NONBLOCK because cat doesn't know how to deal with it.
    if let Some(from_fd) = from_fd {
        fcntl(from_fd, FcntlArg::F_SETFL(OFlag::empty())).map_err(CopyDataError::SetSourceFdFlags)?;
    }
    fcntl(to_fd, FcntlArg::F_SETFL(OFlag::empty())).map_err(CopyDataError::SetTargetFdFlags)?;

    // Don't allocate memory in the child process, it's not async-signal-safe.
    let cat = CString::new("cat").unwrap();
    let also_cat = cat.clone();

    // Fork and exec cat.
    let fork_result = fork().map_err(CopyDataError::Fork)?;
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
            if execvp(&cat, &[also_cat]).is_err() {
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
