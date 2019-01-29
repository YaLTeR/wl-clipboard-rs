use std::{ffi::CString, os::unix::io::RawFd, process::abort};

use libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::{
    sys::wait::{waitpid, WaitStatus},
    unistd::{close, dup2, execvp, fork, ForkResult},
};
use wayland_client::{GlobalError, GlobalManager, Interface, NewProxy, Proxy};

/// Returns `true` if `mime_type` represents text.
pub fn is_text(mime_type: &str) -> bool {
    match mime_type {
        "TEXT" | "STRING" | "UTF8_STRING" => true,
        x if x.starts_with("text/") => true,
        _ => false,
    }
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
pub fn copy_data(from_fd: Option<RawFd>, to_fd: RawFd, wait: bool) {
    // We use the cat utility for data copying. It's easier (no need to implement any comlpex
    // buffering logic), surprisingly safer (a Rust implementation would likely require usage of
    // `from_raw_fd()` which is unsafe) and ideally faster (cat's been around for a while and is
    // probably pretty optimized).

    // Don't allocate memory in the child process, it's not async-signal-safe.
    let cat = CString::new("cat").unwrap();
    let also_cat = cat.clone();

    let fork_result = fork().expect("Error forking");
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
                close(fd).expect("Error closing the data file descriptor");
            }

            close(to_fd).expect("Error closing the target file descriptor");

            if wait {
                // Wait for the child process to exit.
                match waitpid(child, None).expect("Error in waitpid()") {
                    WaitStatus::Exited(_, status) => {
                        if status != 0 {
                            panic!("The child process didn't exit successfully");
                        }
                    }
                    _ => panic!("waitpid() returned an unexpected status"),
                }
            }
        }
    }
}

pub trait GlobalManagerExt {
    /// Instantiates the supported version of the interface.
    fn instantiate_supported<I, F>(&self, implementor: F) -> Result<I, GlobalError>
        where I: Interface + From<Proxy<I>>,
              F: FnOnce(NewProxy<I>) -> I;
}

impl GlobalManagerExt for GlobalManager {
    fn instantiate_supported<I, F>(&self, implementor: F) -> Result<I, GlobalError>
        where I: Interface + From<Proxy<I>>,
              F: FnOnce(NewProxy<I>) -> I
    {
        self.instantiate_exact(I::VERSION, implementor)
    }
}
