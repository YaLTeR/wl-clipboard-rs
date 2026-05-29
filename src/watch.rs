//! Watching the clipboard for selection changes with a [`Watcher`].

use std::collections::HashMap;
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
use crate::paste::{ClipboardType, Error, Seat};

struct State {
    common: common::State,
    // Maps each in-flight offer to its advertised MIME types, populated as Offer events arrive
    // before the corresponding Selection event links the offer to a seat.
    offers: HashMap<data_control::Offer, Vec<String>>,
    got_primary_selection: bool,
    // Pending selection events to report from the watch loop; (is_primary, seat, offer).
    selection_events: Vec<(bool, WlSeat, Option<data_control::Offer>)>,
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
            let offer = id.map(data_control::Offer::from);
            state.selection_events.push((false, seat.clone(), offer));
        }
        Event::Finished => {
            // Destroy the device stored in the seat as it's no longer valid.
            let seat_data = state.common.seats.get_mut(seat).unwrap();
            seat_data.set_device(None);
        }
        Event::PrimarySelection { id } => {
            let offer = id.map(data_control::Offer::from);
            state.got_primary_selection = true;
            state.selection_events.push((true, seat.clone(), offer));
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
    /// Signal the associated [`Watcher`] to stop.
    ///
    /// Returns immediately; the watcher exits before its next blocking wait.
    pub fn cancel(&self) {
        let _ = rustix::io::write(&*self.0, &[0u8]);
    }
}

/// A clipboard selection event reported by [`Watcher::next_event`].
pub enum ClipboardEvent<'a> {
    /// The selection changed; `mime_types` lists offered types in protocol order.
    Changed {
        mime_types: Vec<String>,
        offer: Offer<'a>,
    },
    /// The selection was cleared.
    Cleared,
}

/// Watches the clipboard for selection changes.
///
/// Construct one with [`Watcher::new`], then drive it by calling [`Watcher::next_event`] in a
/// loop. The first call reports the current selection state; subsequent calls block until the
/// selection changes again.
///
/// The caller owns the loop, so you can `break` and `?`-propagate errors. To stop a watcher that
/// is blocked in [`Watcher::next_event`] on another thread, obtain a [`CancelHandle`] from
/// [`Watcher::cancel_handle`] and call [`CancelHandle::cancel`].
pub struct Watcher {
    queue: EventQueue<State>,
    state: State,
    primary: bool,
    // The single seat whose selections we report, resolved at construction.
    watched: WlSeat,
    // Cancellation pipe: the read end is polled in `wait`; the write end is handed out as a
    // `CancelHandle` and becomes readable once `CancelHandle::cancel` is called.
    cancel_read: PipeReader,
    cancel_write: Arc<PipeWriter>,
}

/// The data offer accompanying a [`ClipboardEvent`], borrowed from its [`Watcher`].
///
/// Returned alongside the event by [`Watcher::next_event`]. Holds the live Wayland offer for the
/// duration of the borrow, so [`Offer::receive`] is only callable before the next
/// [`Watcher::next_event`] — which is exactly when the offer is valid. A [`ClipboardEvent::Cleared`]
/// event carries an empty offer, for which [`Offer::receive`] returns [`Error::ClipboardEmpty`].
pub struct Offer<'a> {
    watcher: &'a mut Watcher,
    offer: data_control::Offer,
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
        let primary = clipboard == ClipboardType::Primary;
        let (mut queue, mut common) = initialize(primary, socket_name)?;

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
            got_primary_selection: false,
            selection_events: Vec::new(),
        };

        queue
            .roundtrip(&mut state)
            .map_err(Error::WaylandCommunication)?;

        if primary && !state.got_primary_selection {
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
            primary,
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

    // Discards leading events for other clipboards/seats (destroying their offers), returning
    // whether a matching event is now at the front of the queue.
    fn front_matches(&mut self) -> bool {
        loop {
            let Some((is_primary, event_seat, _)) = self.state.selection_events.first() else {
                return false;
            };
            if *is_primary == self.primary && *event_seat == self.watched {
                return true;
            }
            let (_, _, offer) = self.state.selection_events.remove(0);
            if let Some(offer) = offer {
                self.state.offers.remove(&offer);
                offer.destroy();
            }
        }
    }

    // Removes the front event, returning its kind and an [`Offer`] to receive from. Only call when
    // `front_matches` returned `true`.
    fn take_front_event<'a>(&'a mut self) -> ClipboardEvent<'a> {
        let (_, _, offer) = self.state.selection_events.remove(0);
        let mime_types = offer
            .as_ref()
            .and_then(|o| self.state.offers.remove(o))
            .unwrap_or_default();
        match offer {
            Some(offer) => ClipboardEvent::Changed {
                mime_types,
                offer: Offer {
                    watcher: self,
                    offer,
                },
            },
            None => ClipboardEvent::Cleared,
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

impl Drop for Offer<'_> {
    fn drop(&mut self) {
        self.offer.destroy();
    }
}

impl Offer<'_> {
    /// Reads the clipboard content of the given MIME type into a pipe.
    ///
    /// Returns `Err(Error::ClipboardEmpty)` on a [`ClipboardEvent::Cleared`] event.
    pub fn receive(&mut self, mime_type: &str) -> Result<PipeReader, Error> {
        let (read, write) = pipe().map_err(Error::PipeCreation)?;
        self.offer.receive(mime_type.to_string(), write.as_fd());
        drop(write);
        self.watcher
            .queue
            .roundtrip(&mut self.watcher.state)
            .map_err(Error::WaylandCommunication)?;
        Ok(read)
    }
}
