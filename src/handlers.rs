use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fs::File,
    io,
    os::unix::io::{IntoRawFd, RawFd},
    path::PathBuf,
    rc::Rc,
};

use derive_new::new;
use failure::Fail;
use wayland_client::{
    protocol::{wl_seat::WlSeat, *},
    NewProxy,
};
use wayland_protocols::wlr::unstable::data_control::v1::client::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
    zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
    zwlr_data_control_source_v1::ZwlrDataControlSourceV1, *,
};

use crate::{
    seat_data::SeatData,
    utils::{self, copy_data},
};

pub struct WlSeatHandler;

impl wl_seat::EventHandler for WlSeatHandler {
    fn name(&mut self, seat: WlSeat, name: String) {
        let data = seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
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
    fn selection(&mut self, offer: Option<ZwlrDataControlOfferV1>) {
        // Replace the existing offer with the new one.
        let seat_data = self.seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_offer(offer);
    }
}

impl zwlr_data_control_device_v1::EventHandler for DataDeviceHandler {
    fn data_offer(&mut self,
                  _device: ZwlrDataControlDeviceV1,
                  offer: NewProxy<ZwlrDataControlOfferV1>) {
        // Make a container for the new offer's mime types.
        let mime_types = RefCell::new(HashSet::<String>::with_capacity(1));

        // Bind the new offer with a handler that fills out mime types.
        offer.implement(DataControlOfferHandler, mime_types);
    }

    fn selection(&mut self,
                 _device: ZwlrDataControlDeviceV1,
                 offer: Option<ZwlrDataControlOfferV1>) {
        if !self.primary {
            self.selection(offer);
        }
    }

    fn primary_selection(&mut self,
                         _device: ZwlrDataControlDeviceV1,
                         offer: Option<ZwlrDataControlOfferV1>) {
        self.got_primary_selection.set(true);

        if self.primary {
            self.selection(offer);
        }
    }

    fn finished(&mut self, _device: ZwlrDataControlDeviceV1) {
        // Destroy the device stored in the seat as it's no longer valid.
        let seat_data = self.seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_device(None);
    }
}

pub struct DataControlOfferHandler;

impl zwlr_data_control_offer_v1::EventHandler for DataControlOfferHandler {
    fn offer(&mut self, offer: ZwlrDataControlOfferV1, mime_type: String) {
        let mime_types = offer.as_ref().user_data::<RefCell<HashSet<_>>>().unwrap();
        mime_types.borrow_mut().insert(mime_type);
    }
}

#[derive(Fail, Debug)]
pub enum DataSourceError {
    #[fail(display = "Couldn't open the data file")]
    FileOpen(#[cause] io::Error),

    #[fail(display = "Couldn't copy the data to the target file descriptor")]
    Copy(#[cause] utils::Error),
}

#[derive(new)]
pub struct DataSourceHandler {
    data_path: Rc<RefCell<PathBuf>>,
    should_quit: Rc<Cell<bool>>,
    paste_once: bool,
}

impl zwlr_data_control_source_v1::EventHandler for DataSourceHandler {
    fn send(&mut self, source: ZwlrDataControlSourceV1, _mime_type: String, target_fd: RawFd) {
        // Check if some other source already handled a paste request and indicated that we should
        // quit.
        if self.should_quit.get() {
            source.destroy();
            return;
        }

        let file = File::open(&*self.data_path.borrow()).map_err(DataSourceError::FileOpen);
        let result = file.and_then(|data_file| {
                         let data_fd = data_file.into_raw_fd();
                         copy_data(Some(data_fd), target_fd, false).map_err(DataSourceError::Copy)
                     });

        let mut error = source.as_ref()
                              .user_data::<Rc<RefCell<Option<DataSourceError>>>>()
                              .unwrap()
                              .borrow_mut();
        if let Err(err) = result {
            *error = Some(err);
        }

        if self.paste_once || error.is_some() {
            self.should_quit.set(true);
            source.destroy();
        }
    }

    fn cancelled(&mut self, source: ZwlrDataControlSourceV1) {
        source.destroy();
    }
}
