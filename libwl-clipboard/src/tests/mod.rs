use std::{ffi::OsString, time::Duration};

use wayland_server as ways;

mod copy;
mod paste;
mod utils;

// Taken from wayland-rs: https://github.com/Smithay/wayland-rs
pub(crate) struct TestServer {
    pub display: self::ways::Display,
    pub socket_name: OsString,
}

impl TestServer {
    pub fn new() -> TestServer {
        let mut display = self::ways::Display::new();
        let socket_name = display.add_socket_auto().expect("Failed to create a server socket.");

        TestServer { display, socket_name }
    }

    pub fn answer(&mut self) {
        self.display.dispatch(Duration::from_millis(10), &mut ()).unwrap();
        self.display.flush_clients(&mut ());
    }
}
