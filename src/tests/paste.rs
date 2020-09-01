use std::{
    collections::HashSet,
    ffi::OsString,
    io::{Read, Write},
    mem,
    os::unix::io::FromRawFd,
    thread,
    time::Duration,
};

use os_pipe::PipeWriter;
use wayland_protocols::wlr::unstable::data_control::v1::server::{
    zwlr_data_control_manager_v1::{Request as ServerManagerRequest, ZwlrDataControlManagerV1 as ServerManager},
    zwlr_data_control_offer_v1::{Request as ServerOfferRequest, ZwlrDataControlOfferV1 as ServerOffer},
};
use wayland_server::{protocol::wl_seat::WlSeat as ServerSeat, Filter, Main};

use crate::{paste::*, tests::TestServer};

#[test]
fn get_mime_types_test() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, Filter::new(|_: (_, _), _, _| {}));
    server.display
          .create_global::<ServerManager, _>(
                                             1,
                                             Filter::new(move |(manager, _): (Main<ServerManager>, _), _, _| {
                                                 manager.quick_assign(move |_, request, _| match request {
                                                            ServerManagerRequest::GetDataDevice { id: device, .. } => {
                                                                let offer =
                                 device.as_ref()
                                       .client()
                                       .unwrap()
                                       .create_resource::<ServerOffer>(device.as_ref().version())
                                       .unwrap();
                                                                device.data_offer(&offer);
                                                                offer.offer("first".to_string());
                                                                offer.offer("second".to_string());
                                                                offer.offer("third".to_string());
                                                                device.selection(Some(&offer));
                                                            }
                                                            _ => unreachable!(),
                                                        });
                                             }),
    );

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child =
        thread::spawn(move || get_mime_types_internal(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let mime_types = child.join().unwrap().unwrap();

    let mut expected = HashSet::new();
    expected.insert("first".to_string());
    expected.insert("second".to_string());
    expected.insert("third".to_string());
    assert_eq!(mime_types, expected);
}

#[test]
fn get_mime_types_no_data_control() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, Filter::new(|_: (_, _), _, _| {}));

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child =
        thread::spawn(move || get_mime_types_internal(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let Error::MissingProtocol { name, version } = error {
        assert_eq!(name, "zwlr_data_control_manager_v1");
        assert_eq!(version, 1);
    } else {
        panic!("Invalid error: {:?}", error);
    }
}

#[test]
fn get_mime_types_no_data_control_2() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, Filter::new(|_: (_, _), _, _| {}));

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child =
        thread::spawn(move || get_mime_types_internal(ClipboardType::Primary, Seat::Unspecified, Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let Error::MissingProtocol { name, version } = error {
        assert_eq!(name, "zwlr_data_control_manager_v1");
        assert_eq!(version, 2);
    } else {
        panic!("Invalid error: {:?}", error);
    }
}

#[test]
fn get_mime_types_no_seats() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerManager, _>(1, Filter::new(|_: (_, _), _, _| {}));

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child =
        thread::spawn(move || get_mime_types_internal(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let Error::NoSeats = error {
        // Pass
    } else {
        panic!("Invalid error: {:?}", error);
    }
}

#[test]
fn get_mime_types_empty_clipboard() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, Filter::new(|_: (_, _), _, _| {}));
    server.display.create_global::<ServerManager, _>(1,
                                                     Filter::new(|(manager, _): (Main<ServerManager>, _), _, _| {
                                                         manager.quick_assign(|_, request, _| match request {
                                                                    ServerManagerRequest::GetDataDevice { id:
                                                                                                              device,
                                                                                                          .. } => {
                                                                        device.selection(None);
                                                                    }
                                                                    _ => unreachable!(),
                                                                });
                                                     }));

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child =
        thread::spawn(move || get_mime_types_internal(ClipboardType::Regular, Seat::Unspecified, Some(socket_name)));

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let Error::ClipboardEmpty = error {
        // Pass
    } else {
        panic!("Invalid error: {:?}", error);
    }
}

#[test]
fn get_contents_test() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, Filter::new(|_: (_, _), _, _| {}));
    server.display.create_global::<ServerManager, _>(
                                                     1,
                                                     Filter::new(|(manager, _): (Main<ServerManager>, _), _, _| {
                                                         manager.quick_assign(move |_, request, _| match request {
                                                             ServerManagerRequest::GetDataDevice { id: device, .. } => {
                                                                 let offer =
                                 device.as_ref()
                                       .client()
                                       .unwrap()
                                       .create_resource::<ServerOffer>(device.as_ref().version())
                                       .unwrap();
                                                                 offer.quick_assign(|_, request, _| {
                                                                          if let ServerOfferRequest::Receive { fd,
                                                                                                               .. } =
                                                                              request
                                                                          {
                                                                              let mut write = unsafe {
                                                                                  PipeWriter::from_raw_fd(fd)
                                                                              };
                                                                              let _ = write.write_all(&[1, 3, 3, 7]);
                                                                          }
                                                                      });
                                                                 device.data_offer(&offer);
                                                                 offer.offer("application/octet-stream".to_string());
                                                                 device.selection(Some(&offer));
                                                             }
                                                             _ => unreachable!(),
                                                         });
                                                     }),
    );

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || {
        get_contents_internal(ClipboardType::Regular,
                              Seat::Unspecified,
                              MimeType::Any,
                              Some(socket_name))
    });

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let (mut read, mime_type) = child.join().unwrap().unwrap();
    assert_eq!(mime_type, "application/octet-stream");

    let mut contents = vec![];
    read.read_to_end(&mut contents).unwrap();
    assert_eq!(contents, [1, 3, 3, 7]);
}

#[test]
fn get_contents_wrong_mime_type() {
    let mut server = TestServer::new();
    server.display
          .create_global::<ServerSeat, _>(6, Filter::new(|_: (_, _), _, _| {}));
    server.display.create_global::<ServerManager, _>(
                                                     1,
                                                     Filter::new(|(manager, _): (Main<ServerManager>, _), _, _| {
                                                         manager.quick_assign(move |_, request, _| match request {
                                                             ServerManagerRequest::GetDataDevice { id: device, .. } => {
                                                                 let offer =
                                 device.as_ref()
                                       .client()
                                       .unwrap()
                                       .create_resource::<ServerOffer>(device.as_ref().version())
                                       .unwrap();
                                                                 offer.quick_assign(|_, request, _| {
                                                                          if let ServerOfferRequest::Receive { fd,
                                                                                                               .. } =
                                                                              request
                                                                          {
                                                                              let mut write = unsafe {
                                                                                  PipeWriter::from_raw_fd(fd)
                                                                              };
                                                                              let _ = write.write_all(&[1, 3, 3, 7]);
                                                                          }
                                                                      });
                                                                 device.data_offer(&offer);
                                                                 offer.offer("application/octet-stream".to_string());
                                                                 device.selection(Some(&offer));
                                                             }
                                                             _ => unreachable!(),
                                                         });
                                                     }),
    );

    let socket_name = mem::replace(&mut server.socket_name, OsString::new());
    let child = thread::spawn(move || {
        get_contents_internal(ClipboardType::Regular,
                              Seat::Unspecified,
                              MimeType::Specific("wrong"),
                              Some(socket_name))
    });

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    thread::sleep(Duration::from_millis(100));
    server.answer();

    let error = child.join().unwrap().unwrap_err();
    if let Error::NoMimeType = error {
        // Pass
    } else {
        panic!("Invalid error: {:?}", error);
    }
}
