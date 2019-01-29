use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use log::info;
use wayland_client::{
    global_filter, protocol::wl_seat::WlSeat, Display, EventQueue, GlobalManager, NewProxy,
};
use wayland_protocols::{
    unstable::primary_selection::v1::client::zwp_primary_selection_device_manager_v1::ZwpPrimarySelectionDeviceManagerV1,
    wlr::unstable::data_control::v1::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
};

use crate::{
    clipboard_manager::ClipboardManager, handlers::WlSeatHandler,
    protocol::gtk_primary_selection::client::gtk_primary_selection_device_manager::GtkPrimarySelectionDeviceManager,
    seat_data::SeatData, utils::GlobalManagerExt,
};

pub struct CommonData {
    pub display: Display,
    pub queue: EventQueue,
    pub global_manager: GlobalManager,
    pub clipboard_manager: ClipboardManager,
    pub seats: Arc<Mutex<Vec<WlSeat>>>,
}

pub fn initialize(primary: bool) -> CommonData {
    // Connect to the Wayland compositor.
    let (display, mut queue) = Display::connect_to_env().expect("Error connecting to a display");

    let seats = Arc::new(Mutex::new(Vec::<WlSeat>::new()));

    let seats_2 = seats.clone();
    let global_manager = GlobalManager::new_with_cb(&display,
                                                    global_filter!([WlSeat,
                                                   WlSeat::VERSION,
                                                   move |seat: NewProxy<WlSeat>| {
                                                       let seat_data =
                                                           RefCell::new(SeatData::default());
                                                       let seat =
                                                           seat.implement(WlSeatHandler, seat_data);
                                                       seats_2.lock().unwrap().push(seat.clone());
                                                       seat
                                                   }]));

    // Retrieve the global interfaces.
    queue.sync_roundtrip().expect("Error doing a roundtrip");

    // Check that we have our interfaces.
    let clipboard_manager: ClipboardManager = if primary {
        if let Some(manager) =
            global_manager.instantiate_supported::<GtkPrimarySelectionDeviceManager, _>(NewProxy::implement_dummy)
                          .ok()
                          .map(Into::into)
        {
            Some(manager)
        } else {
            global_manager.instantiate_supported::<ZwpPrimarySelectionDeviceManagerV1, _>(NewProxy::implement_dummy)
                          .ok()
                          .map(Into::into)
        }.expect("Neither gtk_primary_selection_device_manager \
                  nor zwp_primary_selection_device_manager_v1 was found")
    } else {
        global_manager.instantiate_supported::<ZwlrDataControlManagerV1, _>(NewProxy::implement_dummy)
                      .expect("zwlr_data_control_manager_v1 was not found")
                      .into()
    };

    info!("Using the {} interface.", clipboard_manager.name());

    CommonData { display,
                 queue,
                 global_manager,
                 clipboard_manager,
                 seats }
}
