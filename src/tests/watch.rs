use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use wayland_protocols_wlr::data_control::v1::server::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1;

use crate::copy::{self, MimeSource, Options, ServeRequests, Source};
use crate::paste::*;
use crate::tests::state::*;
use crate::tests::TestServer;
use crate::watch::{ClipboardEvent, Watcher};

#[test]
fn watch_initial_changed() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([(
            "seat0".into(),
            SeatInfo {
                offer: Some(OfferInfo::Buffered {
                    data: vec![("text/plain".into(), b"hello".to_vec())],
                }),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher =
        Watcher::with_socket(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)).unwrap();
    let (event, _offer) = watcher.next_event().unwrap().unwrap();

    assert!(
        matches!(event, ClipboardEvent::Changed { mime_types } if mime_types == ["text/plain"])
    );
}

#[test]
fn watch_initial_cleared() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([("seat0".into(), SeatInfo::default())]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher =
        Watcher::with_socket(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)).unwrap();
    let (event, _offer) = watcher.next_event().unwrap().unwrap();

    assert!(matches!(event, ClipboardEvent::Cleared));
}

#[test]
fn watch_receive_contents() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([(
            "seat0".into(),
            SeatInfo {
                offer: Some(OfferInfo::Buffered {
                    data: vec![("text/plain".into(), b"world".to_vec())],
                }),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher =
        Watcher::with_socket(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)).unwrap();
    let (event, mut offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Changed { .. }));

    let mut pipe = offer.receive("text/plain").unwrap();
    let mut received_data = Vec::new();
    pipe.read_to_end(&mut received_data).unwrap();

    assert_eq!(received_data, b"world");
}

#[test]
fn watch_selection_change() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([("seat0".into(), SeatInfo::default())]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let (tx, rx) = mpsc::channel::<ClipboardEvent>();
    let socket_name2 = socket_name.clone();

    // Watch in a background thread; stop after seeing two events (initial + change).
    thread::spawn(move || {
        let mut watcher = Watcher::with_socket(
            ClipboardType::Regular,
            Seat::Unspecified,
            Some(socket_name2),
        )
        .unwrap();
        for _ in 0..2 {
            let Ok(Some((event, _offer))) = watcher.next_event() else {
                break;
            };
            let _ = tx.send(event);
        }
    });

    // First event should be Cleared (empty initial clipboard).
    let first = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(matches!(first, ClipboardEvent::Cleared));

    // Now copy something; the watcher should see a Changed event.
    let mut opts = Options::new();
    opts.serve_requests(ServeRequests::Only(0));
    copy::copy_internal(
        opts,
        vec![MimeSource {
            source: Source::Bytes(b"changed"[..].into()),
            mime_type: copy::MimeType::Specific("text/plain".into()),
        }],
        Some(socket_name),
    )
    .unwrap();

    let second = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(matches!(
        second,
        ClipboardEvent::Changed { ref mime_types } if mime_types.iter().any(|m| m == "text/plain")
    ));
}

// Sets the regular clipboard to `payload` via a real copy client that keeps serving paste
// requests, so the watcher can receive the contents back.
fn copy_text(socket_name: &std::ffi::OsString, payload: &[u8]) {
    copy::copy_internal(
        Options::new(),
        vec![MimeSource {
            source: Source::Bytes(payload.into()),
            mime_type: copy::MimeType::Specific("text/plain".into()),
        }],
        Some(socket_name.clone()),
    )
    .unwrap();
}

fn receive_text(offer: &mut crate::watch::Offer<'_>) -> Vec<u8> {
    let mut pipe = offer.receive("text/plain").unwrap();
    let mut data = Vec::new();
    pipe.read_to_end(&mut data).unwrap();
    data
}

#[test]
fn watch_multiple_changes() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([("seat0".into(), SeatInfo::default())]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher = Watcher::with_socket(
        ClipboardType::Regular,
        Seat::Unspecified,
        Some(socket_name.clone()),
    )
    .unwrap();

    // Initial state is the empty clipboard.
    let (event, offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Cleared));
    drop(offer);

    // Copy several times in a row, receiving and verifying the contents after each change.
    for payload in [&b"one"[..], b"two", b"three"] {
        copy_text(&socket_name, payload);

        let (event, mut offer) = watcher.next_event().unwrap().unwrap();
        assert!(matches!(event, ClipboardEvent::Changed { .. }));
        assert_eq!(receive_text(&mut offer), payload);
    }
}

#[test]
fn watch_continues_after_clear() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([("seat0".into(), SeatInfo::default())]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher = Watcher::with_socket(
        ClipboardType::Regular,
        Seat::Unspecified,
        Some(socket_name.clone()),
    )
    .unwrap();

    // Initial empty clipboard.
    let (event, offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Cleared));
    drop(offer);

    // A change followed by a successful receive.
    copy_text(&socket_name, b"before");
    let (event, mut offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Changed { .. }));
    assert_eq!(receive_text(&mut offer), b"before");
    drop(offer);

    // Clearing the clipboard yields a Cleared event. There's nothing to receive, so receive()
    // reports ClipboardEmpty rather than handing back stale data.
    copy::clear_internal(
        copy::ClipboardType::Regular,
        copy::Seat::All,
        Some(socket_name.clone()),
    )
    .unwrap();
    let (event, mut offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Cleared));
    assert!(matches!(
        offer.receive("text/plain"),
        Err(Error::ClipboardEmpty)
    ));
    drop(offer);

    // A clear in the middle of the stream doesn't stop the watcher: the next change is still
    // observed and readable.
    copy_text(&socket_name, b"after");
    let (event, mut offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Changed { .. }));
    assert_eq!(receive_text(&mut offer), b"after");
}

#[test]
fn watch_rapid_copies_yield_latest_payload() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([("seat0".into(), SeatInfo::default())]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher = Watcher::with_socket(
        ClipboardType::Regular,
        Seat::Unspecified,
        Some(socket_name.clone()),
    )
    .unwrap();

    // Initial empty clipboard.
    let (event, offer) = watcher.next_event().unwrap().unwrap();
    assert!(matches!(event, ClipboardEvent::Cleared));
    drop(offer);

    // Copy several times in a row without draining in between, so the changes queue up.
    let payloads = [&b"one"[..], b"two", b"three"];
    for payload in payloads {
        copy_text(&socket_name, payload);
    }

    // Each queued change is delivered as its own event, and the last one reads back the latest
    // payload.
    for (i, _) in payloads.iter().enumerate() {
        let (event, mut offer) = watcher.next_event().unwrap().unwrap();
        assert!(matches!(event, ClipboardEvent::Changed { .. }));
        if i == payloads.len() - 1 {
            assert_eq!(receive_text(&mut offer), b"three");
        }
    }
}

#[test]
fn watch_cancel() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([("seat0".into(), SeatInfo::default())]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let mut watcher =
        Watcher::with_socket(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)).unwrap();
    let cancel_handle = watcher.cancel_handle();
    let (tx, rx) = mpsc::channel::<()>();

    let handle = thread::spawn(move || -> Result<(), Error> {
        // Signal once per event; after the initial event the next call blocks until cancelled.
        while watcher.next_event()?.is_some() {
            let _ = tx.send(());
        }
        Ok(())
    });

    // Wait until the watcher has processed the initial event and re-entered the poll.
    rx.recv_timeout(Duration::from_secs(1)).unwrap();

    cancel_handle.cancel();

    handle.join().unwrap().unwrap();
}

#[test]
fn watch_offers_destroyed_on_exit() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        seats: HashMap::from([(
            "seat0".into(),
            SeatInfo {
                offer: Some(OfferInfo::Buffered {
                    data: vec![("text/plain".into(), b"hello".to_vec())],
                }),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    state.create_seats(&server);

    let socket_name = server.socket_name().to_owned();
    let destroy_count = Arc::clone(&state.offer_destroy_request_count);
    let state_mutex = Arc::new(Mutex::new(state));
    server.run_mutex(Arc::clone(&state_mutex));

    let mut watcher =
        Watcher::with_socket(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)).unwrap();
    let event = watcher.next_event().unwrap().unwrap();
    // Drop the event (releases its borrow), then the watcher, whose Drop destroys the offer.
    drop(event);
    drop(watcher);

    // Acquiring the mutex waits for the server thread to finish, which only happens after all
    // pending client requests (including the offer destroy) have been dispatched.
    drop(state_mutex.lock().unwrap());

    assert_eq!(destroy_count.load(SeqCst), 1);
}
