use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use wayland_client::{
    global_filter, protocol::wl_seat::WlSeat, Display, EventQueue, GlobalManager, NewProxy,
};

use crate::{
    handlers::{DataControlManagerHandler, WlSeatHandler},
    protocol::wlr_data_control::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    seat_data::SeatData,
};

pub struct CommonData {
    pub display: Display,
    pub queue: EventQueue,
    pub global_manager: GlobalManager,
    pub clipboard_manager: ZwlrDataControlManagerV1,
    pub seats: Rc<RefCell<Vec<WlSeat>>>,
}

pub fn initialize(primary: bool) -> CommonData {
    // Connect to the Wayland compositor.
    let (display, mut queue) = Display::connect_to_env().expect("Error connecting to a display");

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
    queue.sync_roundtrip().expect("Error doing a roundtrip");

    // Check that we have our interfaces.
    let data_control_version = if primary { 2 } else { 1 };

    // TODO: print a different error if data-control 1 was found but 2 is required.
    let impl_manager =
        |manager: NewProxy<_>| manager.implement(DataControlManagerHandler, Cell::new(false));
    let clipboard_manager =
        global_manager.instantiate_exact::<ZwlrDataControlManagerV1, _>(data_control_version,
                                                                        impl_manager)
                      .expect("zwlr_data_control_manager_v1 of the required version was not found");

    CommonData { display,
                 queue,
                 global_manager,
                 clipboard_manager,
                 seats }
}
