use std::{ffi::OsString, time::Duration};

use wayland_server as ways;

mod copy;
mod paste;

// Taken from wayland-rs: https://github.com/Smithay/wayland-rs
pub(crate) struct TestServer {
    pub display: self::ways::Display,
    pub event_loop: self::ways::calloop::EventLoop<()>,
    pub socket_name: OsString,
}

impl TestServer {
    pub fn new() -> TestServer {
        let event_loop = self::ways::calloop::EventLoop::<()>::new().unwrap();
        let mut display = self::ways::Display::new(event_loop.handle());
        let socket_name = display.add_socket_auto()
                                 .expect("Failed to create a server socket.");

        TestServer { display: display,
                     event_loop: event_loop,
                     socket_name: socket_name }
    }

    pub fn answer(&mut self) {
        self.event_loop
            .dispatch(Some(Duration::from_millis(10)), &mut ())
            .unwrap();
        self.display.flush_clients();
    }
}
