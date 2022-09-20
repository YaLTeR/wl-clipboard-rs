use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    fs::File,
    io,
    os::unix::io::{IntoRawFd, RawFd},
    path::PathBuf,
    rc::Rc,
};

use derive_new::new;
use nix::unistd::close;
use wayland_client::{
    protocol::{wl_seat::WlSeat, *},
    DispatchData, Main,
};
use wayland_protocols::wlr::unstable::data_control::v1::client::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1, zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
    zwlr_data_control_source_v1::ZwlrDataControlSourceV1, *,
};

use crate::{
    copy::ServeRequests,
    seat_data::SeatData,
    utils::{self, copy_data},
};

pub fn seat_handler(seat: Main<WlSeat>, event: wl_seat::Event, _: DispatchData) {
    if let wl_seat::Event::Name { name } = event {
        let data = seat.as_ref().user_data().get::<RefCell<SeatData>>().unwrap();
        data.borrow_mut().set_name(name);
    }
}

#[derive(new)]
pub struct DataDeviceHandler {
    seat: WlSeat,
    primary: bool,
    got_primary_selection: Rc<Cell<bool>>,
}

impl DataDeviceHandler {
    fn set_offer(&mut self, offer: Option<ZwlrDataControlOfferV1>) {
        // Replace the existing offer with the new one.
        let seat_data = self.seat.as_ref().user_data().get::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_offer(offer);
    }

    fn data_offer(&mut self, offer: Main<ZwlrDataControlOfferV1>) {
        // Make a container for the new offer's mime types.
        let mime_types = RefCell::new(HashSet::<String>::with_capacity(1));

        // Bind the new offer with a handler that fills out mime types.
        offer.as_ref().user_data().set(move || mime_types);
        offer.quick_assign(data_offer_handler);
    }

    fn selection(&mut self, offer: Option<ZwlrDataControlOfferV1>) {
        if !self.primary {
            self.set_offer(offer);
        }
    }

    fn primary_selection(&mut self, offer: Option<ZwlrDataControlOfferV1>) {
        self.got_primary_selection.set(true);

        if self.primary {
            self.set_offer(offer);
        }
    }

    fn finished(&mut self) {
        // Destroy the device stored in the seat as it's no longer valid.
        let seat_data = self.seat.as_ref().user_data().get::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_device(None);
    }
}

pub fn data_device_handler(handler: &mut DataDeviceHandler,
                           _: Main<ZwlrDataControlDeviceV1>,
                           event: zwlr_data_control_device_v1::Event,
                           _: DispatchData) {
    use zwlr_data_control_device_v1::Event::*;
    match event {
        DataOffer { id } => handler.data_offer(id),
        Selection { id } => handler.selection(id),
        Finished => handler.finished(),
        PrimarySelection { id } => handler.primary_selection(id),
        _ => (),
    }
}

fn data_offer_handler(offer: Main<ZwlrDataControlOfferV1>, event: zwlr_data_control_offer_v1::Event, _: DispatchData) {
    if let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event {
        let mime_types = offer.as_ref().user_data().get::<RefCell<HashSet<_>>>().unwrap();
        mime_types.borrow_mut().insert(mime_type);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DataSourceError {
    #[error("Couldn't open the data file")]
    FileOpen(#[source] io::Error),

    #[error("Couldn't copy the data to the target file descriptor")]
    Copy(#[source] utils::CopyDataError),
}

#[derive(new)]
pub struct DataSourceHandler {
    data_paths: HashMap<String, Rc<RefCell<PathBuf>>>,
    should_quit: Rc<Cell<bool>>,
    serve_requests: Rc<Cell<ServeRequests>>,
}

impl DataSourceHandler {
    fn send(&mut self, source: Main<ZwlrDataControlSourceV1>, mime_type: String, target_fd: RawFd) {
        // Check if some other source already handled a paste request and indicated that we should
        // quit.
        if self.should_quit.get() {
            source.destroy();
            return;
        }

        // I'm not sure if it's the compositor's responsibility to check that the mime type is
        // valid. Let's check here just in case.
        if !&self.data_paths.contains_key(&mime_type) {
            let _ = close(target_fd);
            return;
        }

        let data_path = &self.data_paths[&mime_type];

        let file = File::open(&*data_path.borrow()).map_err(DataSourceError::FileOpen);
        let result = file.and_then(|data_file| {
                             let data_fd = data_file.into_raw_fd();
                             copy_data(Some(data_fd), target_fd, true).map_err(DataSourceError::Copy)
                         });

        let mut error = source.as_ref()
                              .user_data()
                              .get::<Rc<RefCell<Option<DataSourceError>>>>()
                              .unwrap()
                              .borrow_mut();
        if let Err(err) = result {
            *error = Some(err);
        }

        let done = if let ServeRequests::Only(left) = self.serve_requests.get() {
            let left = left.checked_sub(1).unwrap();
            self.serve_requests.set(ServeRequests::Only(left));
            left == 0
        } else {
            false
        };

        if done || error.is_some() {
            self.should_quit.set(true);
            source.destroy();
        }
    }

    fn cancelled(&mut self, source: Main<ZwlrDataControlSourceV1>) {
        source.destroy();
    }
}

pub fn data_source_handler(handler: &mut DataSourceHandler,
                           source: Main<ZwlrDataControlSourceV1>,
                           event: zwlr_data_control_source_v1::Event,
                           _: DispatchData) {
    use zwlr_data_control_source_v1::Event::*;
    match event {
        Send { mime_type, fd } => handler.send(source, mime_type, fd),
        Cancelled => handler.cancelled(source),
        _ => (),
    }
}
