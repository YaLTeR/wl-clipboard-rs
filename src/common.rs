use std::{cell::RefCell, ffi::OsString, io, rc::Rc};

use failure::Fail;
use wayland_client::{
    global_filter, protocol::wl_seat::WlSeat, ConnectError, Display, EventQueue, GlobalManager,
    Interface, NewProxy,
};
use wayland_protocols::wlr::unstable::data_control::v1::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1;

use crate::{handlers::WlSeatHandler, seat_data::SeatData};

pub struct CommonData {
    pub queue: EventQueue,
    pub clipboard_manager: ZwlrDataControlManagerV1,
    pub seats: Rc<RefCell<Vec<WlSeat>>>,
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Couldn't connect to the Wayland compositor")]
    WaylandConnection(#[cause] ConnectError),

    #[fail(display = "Wayland compositor communication error")]
    WaylandCommunication(#[cause] io::Error),

    #[fail(display = "A required Wayland protocol ({} version {}) is not supported by the compositor",
           name, version)]
    MissingProtocol { name: &'static str, version: u32 },
}

pub fn initialize(primary: bool, socket_name: Option<OsString>) -> Result<CommonData, Error> {
    // Connect to the Wayland compositor.
    let (display, mut queue) = match socket_name {
                                   Some(name) => Display::connect_to_name(name),
                                   None => Display::connect_to_env(),
                               }.map_err(Error::WaylandConnection)?;

    let seats = Rc::new(RefCell::new(Vec::<WlSeat>::new()));

    let seats_2 = seats.clone();
    let global_manager =
        GlobalManager::new_with_cb(&display,
                                   global_filter!([WlSeat, 6, move |seat: NewProxy<WlSeat>| {
                                                      let seat_data =
                                                          RefCell::new(SeatData::default());
                                                      let seat =
                                                          seat.implement(WlSeatHandler, seat_data);
                                                      seats_2.borrow_mut().push(seat.clone());
                                                      seat
                                                  }]));

    // Retrieve the global interfaces.
    queue.sync_roundtrip()
         .map_err(Error::WaylandCommunication)?;

    // Check that we have our interfaces.
    let data_control_version = if primary { 2 } else { 1 };

    let impl_manager = |manager: NewProxy<_>| manager.implement_dummy();
    let clipboard_manager =
        global_manager.instantiate_exact::<ZwlrDataControlManagerV1, _>(data_control_version,
                                                                        impl_manager)
                      .map_err(|_| Error::MissingProtocol { name:
                                                                ZwlrDataControlManagerV1::NAME,
                                                            version: data_control_version })?;

    Ok(CommonData { queue,
                    clipboard_manager,
                    seats })
}
