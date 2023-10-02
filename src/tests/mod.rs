use std::ffi::OsStr;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex};
use std::thread;

use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use wayland_backend::server::ClientData;
use wayland_server::{Display, ListeningSocket};

mod copy;
mod paste;
mod state;
mod utils;

pub struct TestServer<S: 'static> {
    pub display: Display<S>,
    pub socket: ListeningSocket,
    pub poll_fd: OwnedFd,
}

struct ClientCounter(AtomicU8);

impl ClientData for ClientCounter {
    fn disconnected(
        &self,
        _client_id: wayland_backend::server::ClientId,
        _reason: wayland_backend::server::DisconnectReason,
    ) {
        self.0.fetch_sub(1, SeqCst);
    }
}

impl<S: Send + 'static> TestServer<S> {
    pub fn new() -> Self {
        let mut display = Display::new().unwrap();
        let socket = ListeningSocket::bind_auto("wl-clipboard-rs-test", 0..).unwrap();

        let poll_fd = epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC).unwrap();
        let poll_fd = unsafe { OwnedFd::from_raw_fd(poll_fd) };

        let mut evt = EpollEvent::new(EpollFlags::EPOLLIN, 0);
        epoll_ctl(
            poll_fd.as_raw_fd(),
            EpollOp::EpollCtlAdd,
            socket.as_raw_fd(),
            &mut evt,
        )
        .unwrap();
        let mut evt = EpollEvent::new(EpollFlags::EPOLLIN, 1);
        epoll_ctl(
            poll_fd.as_raw_fd(),
            EpollOp::EpollCtlAdd,
            display.backend().poll_fd().as_raw_fd(),
            &mut evt,
        )
        .unwrap();

        TestServer {
            display,
            socket,
            poll_fd,
        }
    }

    pub fn socket_name(&self) -> &OsStr {
        self.socket.socket_name().unwrap()
    }

    pub fn run(self, mut state: S) {
        thread::spawn(move || self.run_internal(&mut state));
    }

    pub fn run_mutex(self, state: Arc<Mutex<S>>) {
        thread::spawn(move || {
            let mut state = state.lock().unwrap();
            self.run_internal(&mut *state);
        });
    }

    fn run_internal(mut self, state: &mut S) {
        let mut waiting_for_first_client = true;
        let client_counter = Arc::new(ClientCounter(AtomicU8::new(0)));

        while client_counter.0.load(SeqCst) > 0 || waiting_for_first_client {
            // Wait for requests from the client.
            let mut events = [EpollEvent::empty(); 2];
            let nevents = epoll_wait(self.poll_fd.as_raw_fd(), &mut events, -1).unwrap();

            let ready_socket = events.iter().take(nevents).any(|event| event.data() == 0);
            let ready_clients = events.iter().take(nevents).any(|event| event.data() == 1);

            if ready_socket {
                // Try to accept a new client.
                if let Some(stream) = self.socket.accept().unwrap() {
                    waiting_for_first_client = false;
                    client_counter.0.fetch_add(1, SeqCst);
                    self.display
                        .handle()
                        .insert_client(stream, client_counter.clone())
                        .unwrap();
                }
            }

            if ready_clients {
                // Try to dispatch client messages.
                self.display.dispatch_clients(state).unwrap();
                self.display.flush_clients().unwrap();
            }
        }
    }
}

// https://github.com/Smithay/wayland-rs/blob/90a9ad1f8f1fdef72e96d3c48bdb76b53a7722ff/wayland-tests/tests/helpers/mod.rs
#[macro_export]
macro_rules! server_ignore_impl {
    ($handler:ty => [$($iface:ty),*]) => {
        $(
            impl wayland_server::Dispatch<$iface, ()> for $handler {
                fn request(
                    _: &mut Self,
                    _: &wayland_server::Client,
                    _: &$iface,
                    _: <$iface as wayland_server::Resource>::Request,
                    _: &(),
                    _: &wayland_server::DisplayHandle,
                    _: &mut wayland_server::DataInit<'_, Self>,
                ) {
                }
            }
        )*
    }
}

#[macro_export]
macro_rules! server_ignore_global_impl {
    ($handler:ty => [$($iface:ty),*]) => {
        $(
            impl wayland_server::GlobalDispatch<$iface, ()> for $handler {
                fn bind(
                    _: &mut Self,
                    _: &wayland_server::DisplayHandle,
                    _: &wayland_server::Client,
                    new_id: wayland_server::New<$iface>,
                    _: &(),
                    data_init: &mut wayland_server::DataInit<'_, Self>,
                ) {
                    data_init.init(new_id, ());
                }
            }
        )*
    }
}
