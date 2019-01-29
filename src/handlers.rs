use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    fs::File,
    os::unix::io::{IntoRawFd, RawFd},
    path::PathBuf,
};

use derive_new::new;
use wayland_client::{
    protocol::{wl_seat::WlSeat, *},
    NewProxy,
};
use wayland_protocols::{
    unstable::primary_selection::v1::client::{
        zwp_primary_selection_device_v1::ZwpPrimarySelectionDeviceV1,
        zwp_primary_selection_offer_v1::ZwpPrimarySelectionOfferV1,
        zwp_primary_selection_source_v1::ZwpPrimarySelectionSourceV1, *,
    },
    wlr::unstable::{
        data_control::v1::client::{
            zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
            zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
            zwlr_data_control_source_v1::ZwlrDataControlSourceV1, *,
        },
        layer_shell::v1::client::{zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, *},
    },
};

use crate::{
    data_source::DataSource,
    offer::{NewOffer, Offer},
    protocol::gtk_primary_selection::client::{
        gtk_primary_selection_device::GtkPrimarySelectionDevice,
        gtk_primary_selection_offer::GtkPrimarySelectionOffer,
        gtk_primary_selection_source::GtkPrimarySelectionSource, *,
    },
    seat_data::SeatData,
    utils::copy_data,
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

    #[new(value = "None")]
    incoming_offer: Option<Offer>,
}

impl DataDeviceHandler {
    fn data_offer(&mut self, offer: NewOffer) {
        // We've got a new data offer. First, destroy the previous one, if any.
        if let Some(offer) = self.incoming_offer.take() {
            offer.destroy();
        }

        // Now make a container for the new offer's mime types.
        let mime_types = RefCell::new(HashSet::<String>::with_capacity(1));

        // Finally, bind the new offer with a handler that fills out mime types.
        self.incoming_offer = Some(offer.implement(DataControlOfferHandler, mime_types));
    }

    fn selection(&mut self, offer: Option<Offer>) {
        match offer {
            Some(offer) => {
                // Sanity checks.
                debug_assert!(self.incoming_offer.is_some());
                let incoming_offer = self.incoming_offer.take().unwrap();
                debug_assert_eq!(offer.id(), incoming_offer.id());

                // Replace the existing offer with the new one.
                let seat_data = self.seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
                seat_data.borrow_mut().set_offer(Some(offer));
            }
            None => {
                // Destroy the incoming offer, if any.
                if let Some(offer) = self.incoming_offer.take() {
                    offer.destroy();
                }

                // Destroy the offer stored in the seat as it's no longer valid.
                let seat_data = self.seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
                seat_data.borrow_mut().set_offer(None);
            }
        }
    }

    fn finished(&mut self) {
        // Destroy the device stored in the seat as it's no longer valid.
        let seat_data = self.seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
        seat_data.borrow_mut().set_device(None);
    }
}

impl zwlr_data_control_device_v1::EventHandler for DataDeviceHandler {
    fn data_offer(&mut self,
                  _device: ZwlrDataControlDeviceV1,
                  offer: NewProxy<ZwlrDataControlOfferV1>) {
        self.data_offer(offer.into())
    }

    fn selection(&mut self,
                 _device: ZwlrDataControlDeviceV1,
                 offer: Option<ZwlrDataControlOfferV1>) {
        self.selection(offer.map(Into::into))
    }

    fn finished(&mut self, _device: ZwlrDataControlDeviceV1) {
        self.finished()
    }
}

impl gtk_primary_selection_device::EventHandler for DataDeviceHandler {
    fn data_offer(&mut self,
                  _device: GtkPrimarySelectionDevice,
                  offer: NewProxy<GtkPrimarySelectionOffer>) {
        self.data_offer(offer.into())
    }

    fn selection(&mut self,
                 _device: GtkPrimarySelectionDevice,
                 offer: Option<GtkPrimarySelectionOffer>) {
        self.selection(offer.map(Into::into))
    }
}

impl zwp_primary_selection_device_v1::EventHandler for DataDeviceHandler {
    fn data_offer(&mut self,
                  _device: ZwpPrimarySelectionDeviceV1,
                  offer: NewProxy<ZwpPrimarySelectionOfferV1>) {
        self.data_offer(offer.into())
    }

    fn selection(&mut self,
                 _device: ZwpPrimarySelectionDeviceV1,
                 offer: Option<ZwpPrimarySelectionOfferV1>) {
        self.selection(offer.map(Into::into))
    }
}

pub struct DataControlOfferHandler;

impl DataControlOfferHandler {
    fn offer(&mut self, offer: Offer, mime_type: String) {
        let mime_types = offer.user_data::<RefCell<HashSet<_>>>().unwrap();
        mime_types.borrow_mut().insert(mime_type);
    }
}

impl zwlr_data_control_offer_v1::EventHandler for DataControlOfferHandler {
    fn offer(&mut self, offer: ZwlrDataControlOfferV1, mime_type: String) {
        self.offer(offer.into(), mime_type)
    }
}

impl gtk_primary_selection_offer::EventHandler for DataControlOfferHandler {
    fn offer(&mut self, offer: GtkPrimarySelectionOffer, mime_type: String) {
        self.offer(offer.into(), mime_type)
    }
}

impl zwp_primary_selection_offer_v1::EventHandler for DataControlOfferHandler {
    fn offer(&mut self, offer: ZwpPrimarySelectionOfferV1, mime_type: String) {
        self.offer(offer.into(), mime_type)
    }
}

#[derive(new)]
pub struct DataSourceHandler {
    paste_once: bool,

    #[new(value = "false")]
    destroyed: bool,
}

impl DataSourceHandler {
    fn send(&mut self, source: DataSource, target_fd: RawFd) {
        if self.destroyed {
            unreachable!();
        }

        let (should_quit, data_path) = source.user_data::<(Cell<bool>, RefCell<PathBuf>)>()
                                             .unwrap();

        let data_file = File::open(&*data_path.borrow()).expect("Error opening the data file");
        let data_fd = data_file.into_raw_fd();

        copy_data(Some(data_fd), target_fd, false);

        if self.paste_once {
            should_quit.set(true);

            // TODO: call destroy? Technically there could still be pending send() requests in this
            // batch, I'm afraid this can mess with them. And if we're quitting anyway does it even
            // matter that much?
        }
    }

    fn cancelled(&mut self, source: DataSource) {
        if self.destroyed {
            unreachable!();
        }

        let (should_quit, _) = source.user_data::<(Cell<bool>, RefCell<PathBuf>)>()
                                     .unwrap();
        source.destroy();

        self.destroyed = true;
        should_quit.set(true);
    }
}

impl zwlr_data_control_source_v1::EventHandler for DataSourceHandler {
    fn send(&mut self, source: ZwlrDataControlSourceV1, _mime_type: String, fd: RawFd) {
        self.send(source.into(), fd)
    }

    fn cancelled(&mut self, source: ZwlrDataControlSourceV1) {
        self.cancelled(source.into())
    }
}

impl gtk_primary_selection_source::EventHandler for DataSourceHandler {
    fn send(&mut self, source: GtkPrimarySelectionSource, _mime_type: String, fd: RawFd) {
        self.send(source.into(), fd)
    }

    fn cancelled(&mut self, source: GtkPrimarySelectionSource) {
        self.cancelled(source.into())
    }
}

impl zwp_primary_selection_source_v1::EventHandler for DataSourceHandler {
    fn send(&mut self, source: ZwpPrimarySelectionSourceV1, _mime_type: String, fd: RawFd) {
        self.send(source.into(), fd)
    }

    fn cancelled(&mut self, source: ZwpPrimarySelectionSourceV1) {
        self.cancelled(source.into())
    }
}

pub struct LayerSurfaceHandler;

impl zwlr_layer_surface_v1::EventHandler for LayerSurfaceHandler {
    fn configure(&mut self, surface: ZwlrLayerSurfaceV1, serial: u32, _width: u32, _height: u32) {
        surface.ack_configure(serial)
    }
}
