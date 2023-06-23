use wayland_protocols_wlr::data_control::v1::server::zwlr_data_control_device_v1::ZwlrDataControlDeviceV1;
use wayland_protocols_wlr::data_control::v1::server::zwlr_data_control_manager_v1::{
    self, ZwlrDataControlManagerV1,
};
use wayland_server::protocol::wl_seat::WlSeat;
use wayland_server::Dispatch;

use crate::tests::TestServer;
use crate::utils::*;
use crate::{server_ignore_global_impl, server_ignore_impl};

struct State {
    advertise_primary_selection: bool,
}

server_ignore_global_impl!(State => [WlSeat, ZwlrDataControlManagerV1]);
server_ignore_impl!(State => [WlSeat, ZwlrDataControlDeviceV1]);

impl Dispatch<ZwlrDataControlManagerV1, ()> for State {
    fn request(
        state: &mut Self,
        _client: &wayland_server::Client,
        _resource: &ZwlrDataControlManagerV1,
        request: <ZwlrDataControlManagerV1 as wayland_server::Resource>::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut wayland_server::DataInit<'_, Self>,
    ) {
        if let zwlr_data_control_manager_v1::Request::GetDataDevice { id, .. } = request {
            let data_device = data_init.init(id, ());

            if state.advertise_primary_selection {
                data_device.primary_selection(None);
            }
        }
    }
}

#[test]
fn is_primary_selection_supported_test() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, WlSeat, ()>(6, ());
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        advertise_primary_selection: true,
    };

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let result = is_primary_selection_supported_internal(Some(socket_name)).unwrap();
    assert!(result);
}

#[test]
fn is_primary_selection_supported_primary_selection_unsupported() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, WlSeat, ()>(6, ());
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        advertise_primary_selection: false,
    };

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let result = is_primary_selection_supported_internal(Some(socket_name)).unwrap();
    assert!(!result);
}

#[test]
fn is_primary_selection_supported_data_control_v1() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, WlSeat, ()>(6, ());
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(1, ());

    let state = State {
        advertise_primary_selection: false,
    };

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let result = is_primary_selection_supported_internal(Some(socket_name)).unwrap();
    assert!(!result);
}

#[test]
fn is_primary_selection_supported_no_seats() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        advertise_primary_selection: true,
    };

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let result = is_primary_selection_supported_internal(Some(socket_name));
    assert!(matches!(result, Err(PrimarySelectionCheckError::NoSeats)));
}

#[test]
fn supports_v2_seats() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, WlSeat, ()>(2, ());
    server
        .display
        .handle()
        .create_global::<State, ZwlrDataControlManagerV1, ()>(2, ());

    let state = State {
        advertise_primary_selection: true,
    };

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let result = is_primary_selection_supported_internal(Some(socket_name)).unwrap();
    assert!(result);
}

#[test]
fn is_primary_selection_supported_no_data_control() {
    let server = TestServer::new();
    server
        .display
        .handle()
        .create_global::<State, WlSeat, ()>(6, ());

    let state = State {
        advertise_primary_selection: false,
    };

    let socket_name = server.socket_name().to_owned();
    server.run(state);

    let result = is_primary_selection_supported_internal(Some(socket_name));
    assert!(matches!(
        result,
        Err(PrimarySelectionCheckError::MissingProtocol {
            name: "zwlr_data_control_manager_v1",
            version: 1
        })
    ));
}
