//! Watching the clipboard for selection changes with a [`Watcher`].

use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::io;
use std::os::fd::AsFd;
use std::sync::Arc;

use os_pipe::{pipe, PipeReader, PipeWriter};
use rustix::event::{PollFd, PollFlags};
use wayland_backend::client::WaylandError;
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{delegate_dispatch, event_created_child, Dispatch, EventQueue};

use crate::common::{self, initialize};
use crate::data_control::{self, impl_dispatch_device, impl_dispatch_manager, impl_dispatch_offer};
use crate::paste::{self, Error, Seat};

struct State {
    common: common::State,
    // Maps each newly introduced offer to its advertised MIME types, populated as Offer events
    // arrive. The Selection event that follows moves the entry into its `SelectionEvent`.
    offers: HashMap<data_control::Offer, Vec<String>>,
    selection_events: VecDeque<SelectionEvent>,
}

impl State {
    fn got_primary_selection(&self) -> bool {
        self.selection_events
            .iter()
            .any(|event| event.clipboard == paste::ClipboardType::Primary)
    }

    fn push_selection_event(
        &mut self,
        clipboard: paste::ClipboardType,
        seat: &WlSeat,
        offer: Option<data_control::Offer>,
    ) {
        let offer = offer.map(|offer| {
            // Each offer belongs to exactly one selection event, so take its MIME types here.
            let mime_types = self.offers.remove(&offer).unwrap_or_default();
            SelectionOffer {
                offer: OwnedOffer(offer),
                mime_types,
            }
        });
        self.selection_events.push_back(SelectionEvent {
            clipboard,
            seat: seat.clone(),
            offer,
        });
    }
}

/// Pending selection event to report from the watch loop.
struct SelectionEvent {
    clipboard: paste::ClipboardType,
    seat: WlSeat,
    offer: Option<SelectionOffer>,
}

struct SelectionOffer {
    offer: OwnedOffer,
    mime_types: Vec<String>,
}

// Not `Clone`: dropping it destroys the underlying Wayland object.
struct OwnedOffer(data_control::Offer);

impl Drop for OwnedOffer {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

delegate_dispatch!(State: [WlSeat: ()] => common::State);

impl AsMut<common::State> for State {
    fn as_mut(&mut self) -> &mut common::State {
        &mut self.common
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &wayland_client::Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl_dispatch_manager!(State);

impl_dispatch_device!(State, WlSeat, |state: &mut Self, event, seat: &WlSeat| {
    match event {
        Event::DataOffer { id } => {
            let offer = data_control::Offer::from(id);
            state.offers.insert(offer, Vec::new());
        }
        Event::Selection { id } => {
            state.push_selection_event(
                paste::ClipboardType::Regular,
                seat,
                id.map(data_control::Offer::from),
            );
        }
        Event::Finished => {
            // Destroy the device stored in the seat as it's no longer valid.
            let seat_data = state.common.seats.get_mut(seat).unwrap();
            seat_data.set_device(None);
        }
        Event::PrimarySelection { id } => {
            state.push_selection_event(
                paste::ClipboardType::Primary,
                seat,
                id.map(data_control::Offer::from),
            );
        }
        _ => (),
    }
});

impl_dispatch_offer!(State, |state: &mut Self,
                             offer: data_control::Offer,
                             event| {
    if let Event::Offer { mime_type } = event {
        state.offers.get_mut(&offer).unwrap().push(mime_type);
    }
});

/// Handle used to stop a running [`Watcher`] from another thread.
///
/// Obtain one with [`Watcher::cancel_handle`]. Clone-able and safe to send across threads.
#[derive(Clone)]
pub struct CancelHandle(Arc<PipeWriter>);

impl CancelHandle {
    /// Signals the associated [`Watcher`] to stop.
    ///
    /// Returns immediately; the watcher exits before its next blocking wait.
    pub fn cancel(&self) {
        let _ = rustix::io::write(&*self.0, &[0u8]);
    }
}

/// The clipboard to watch.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord, Default)]
pub enum ClipboardType {
    /// The regular clipboard.
    #[default]
    Regular,
    /// The "primary" clipboard.
    ///
    /// Watching the "primary" clipboard requires the compositor to support ext-data-control,
    /// or wlr-data-control version 2 or above.
    Primary,
    /// Watch both clipboards at once.
    ///
    /// Each [`ClipboardEvent`] reports which clipboard it came from. This option requires the
    /// "primary" clipboard to be supported.
    Both,
}

impl ClipboardType {
    fn includes(self, clipboard: paste::ClipboardType) -> bool {
        match self {
            ClipboardType::Regular => clipboard == paste::ClipboardType::Regular,
            ClipboardType::Primary => clipboard == paste::ClipboardType::Primary,
            ClipboardType::Both => true,
        }
    }
}

impl From<paste::ClipboardType> for ClipboardType {
    fn from(clipboard: paste::ClipboardType) -> Self {
        match clipboard {
            paste::ClipboardType::Regular => ClipboardType::Regular,
            paste::ClipboardType::Primary => ClipboardType::Primary,
        }
    }
}

/// A clipboard selection event reported by [`Watcher::next_event`].
pub enum ClipboardEvent<'a> {
    /// The selection has changed.
    Changed {
        /// The clipboard whose selection changed.
        clipboard: paste::ClipboardType,
        /// The offered MIME types in protocol order.
        mime_types: Vec<String>,
        /// The offer used to receive the data.
        offer: Offer<'a>,
    },
    /// The selection was cleared.
    Cleared {
        /// The clipboard that was cleared.
        clipboard: paste::ClipboardType,
    },
}

/// Watches the clipboard for selection changes.
///
/// Construct one with [`Watcher::new`], then drive it by calling [`Watcher::next_event`] in a
/// loop. The first call reports the current selection state; subsequent calls block until the
/// selection changes again.
///
/// Seats are resolved when the watcher starts, so passing [`Seat::Unspecified`] will use the
/// first seat found on construction rather than re-enumerating seats as offers come in.
pub struct Watcher {
    queue: EventQueue<State>,
    state: State,
    clipboard: ClipboardType,
    // The single seat whose selections we report, resolved at construction.
    watched: WlSeat,
    // Cancellation pipe: the read end is polled in `wait`; the write end is handed out as a
    // `CancelHandle` and becomes readable once `CancelHandle::cancel` is called.
    cancel_read: PipeReader,
    cancel_write: Arc<PipeWriter>,
}

impl Watcher {
    /// Starts watching the clipboard.
    ///
    /// Returns an error immediately if there are no seats, the requested seat or protocol is
    /// missing, or primary selection was requested but is unsupported.
    pub fn new(clipboard: ClipboardType, seat: Seat<'_>) -> Result<Self, Error> {
        Self::with_socket(clipboard, seat, None)
    }

    // The internal constructor accepts the socket name, used for tests.
    pub(crate) fn with_socket(
        clipboard: ClipboardType,
        seat: Seat<'_>,
        socket_name: Option<OsString>,
    ) -> Result<Self, Error> {
        let (mut queue, mut common) = initialize(clipboard != ClipboardType::Regular, socket_name)?;

        if common.seats.is_empty() {
            return Err(Error::NoSeats);
        }

        for (seat, data) in &mut common.seats {
            let device =
                common
                    .clipboard_manager
                    .get_data_device(seat, &queue.handle(), seat.clone());
            data.set_device(Some(device));
        }

        let mut state = State {
            common,
            offers: HashMap::new(),
            selection_events: VecDeque::new(),
        };

        queue
            .roundtrip(&mut state)
            .map_err(Error::WaylandCommunication)?;

        if clipboard != ClipboardType::Regular && !state.got_primary_selection() {
            return Err(Error::PrimarySelectionUnsupported);
        }

        let seats = &state.common.seats;
        let watched = match seat {
            Seat::Unspecified => seats.keys().next().cloned(),
            Seat::Specific(name) => seats
                .iter()
                .find(|(_, data)| data.name.as_deref() == Some(name))
                .map(|(seat, _)| seat.clone()),
        };
        let Some(watched) = watched else {
            return Err(Error::SeatNotFound);
        };

        let (cancel_read, cancel_write) = pipe().map_err(Error::PipeCreation)?;

        Ok(Watcher {
            queue,
            state,
            clipboard,
            watched,
            cancel_read,
            cancel_write: Arc::new(cancel_write),
        })
    }

    /// Returns a handle that stops this watcher when [`CancelHandle::cancel`] is called.
    ///
    /// The handle can be cloned and sent to another thread to interrupt a [`Watcher::next_event`]
    /// call that is blocked waiting for the next selection change.
    pub fn cancel_handle(&self) -> CancelHandle {
        CancelHandle(Arc::clone(&self.cancel_write))
    }

    /// Blocks until the next selection event for the watched clipboard and seat.
    ///
    /// On success yields the [`ClipboardEvent`] and an [`Offer`] to read its contents from.
    /// Returns `Ok(None)` if cancelled via [`CancelHandle::cancel`], or `Err` on a Wayland
    /// communication failure.
    pub fn next_event<'a>(&'a mut self) -> Result<Option<ClipboardEvent<'a>>, Error> {
        while !self.front_matches() {
            if self.wait()? {
                return Ok(None);
            }
        }
        Ok(Some(self.take_front_event()))
    }

    // Discards leading events for other clipboards/seats, returning whether a matching event is now
    // at the front of the queue.
    fn front_matches(&mut self) -> bool {
        while let Some(event) = self.state.selection_events.front() {
            if self.clipboard.includes(event.clipboard) && event.seat == self.watched {
                return true;
            }
            self.state.selection_events.pop_front();
        }
        false
    }

    // Removes the front event, returning its kind and an [`Offer`] to receive from. Only call when
    // `front_matches` returned `true`.
    fn take_front_event<'a>(&'a mut self) -> ClipboardEvent<'a> {
        let SelectionEvent {
            clipboard, offer, ..
        } = self.state.selection_events.pop_front().unwrap();
        match offer {
            Some(SelectionOffer { offer, mime_types }) => ClipboardEvent::Changed {
                clipboard,
                mime_types,
                offer: Offer {
                    watcher: self,
                    offer,
                },
            },
            None => ClipboardEvent::Cleared { clipboard },
        }
    }

    // Blocks until more Wayland events arrive (or cancellation). Returns `Ok(true)` if cancelled.
    fn wait(&mut self) -> Result<bool, Error> {
        self.queue
            .flush()
            .map_err(|e| Error::WaylandCommunication(e.into()))?;

        if let Some(guard) = self.queue.prepare_read() {
            let wayland_fd = guard.connection_fd();
            let mut poll_fds = [
                PollFd::new(&wayland_fd, PollFlags::IN | PollFlags::ERR),
                PollFd::new(&self.cancel_read, PollFlags::IN),
            ];
            loop {
                match rustix::event::poll(&mut poll_fds, None) {
                    Ok(_) => break,
                    Err(rustix::io::Errno::INTR) => continue,
                    Err(e) => {
                        return Err(Error::WaylandCommunication(
                            WaylandError::Io(e.into()).into(),
                        ))
                    }
                }
            }
            // Got data on the cancel pipe, bail with `true`.
            if poll_fds[1].revents().contains(PollFlags::IN) {
                return Ok(true);
            }
            match guard.read() {
                Ok(_) => {}
                Err(WaylandError::Io(e)) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(Error::WaylandCommunication(e.into())),
            }
        }

        self.queue
            .dispatch_pending(&mut self.state)
            .map_err(Error::WaylandCommunication)?;
        Ok(false)
    }
}

/// The data offer accompanying a [`ClipboardEvent`], borrowed from its [`Watcher`].
pub struct Offer<'a> {
    watcher: &'a mut Watcher,
    offer: OwnedOffer,
}

impl Offer<'_> {
    /// Reads the clipboard content of the given MIME type into a pipe.
    ///
    /// Returns `Err(Error::ClipboardEmpty)` on a [`ClipboardEvent::Cleared`] event.
    pub fn receive(&mut self, mime_type: &str) -> Result<PipeReader, Error> {
        let (read, write) = pipe().map_err(Error::PipeCreation)?;
        self.offer.0.receive(mime_type.to_string(), write.as_fd());
        drop(write);
        self.watcher
            .queue
            .flush()
            .map_err(|e| Error::WaylandCommunication(e.into()))?;
        Ok(read)
    }
}
