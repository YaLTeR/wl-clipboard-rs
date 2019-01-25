use std::{cell::RefCell, collections::HashSet, rc::Rc};

use derive_new::new;
use wayland_client::{
    protocol::{wl_compositor::WlCompositor, wl_registry::WlRegistry, wl_seat::WlSeat, *},
    Interface, NewProxy,
};
use wayland_protocols::wlr::unstable::{
    data_control::v1::client::{
        zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
        zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
        zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, *,
    },
    layer_shell::v1::client::{
        zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, *,
    },
};

use crate::offer::{NewOffer, Offer};
use crate::protocol::gtk_primary_selection::client::{
    gtk_primary_selection_device::GtkPrimarySelectionDevice,
    gtk_primary_selection_device_manager::GtkPrimarySelectionDeviceManager,
    gtk_primary_selection_offer::GtkPrimarySelectionOffer, *,
};
use crate::seat_data::SeatData;

#[derive(new)]
pub struct WlRegistryHandler {
    data_control_manager: Rc<RefCell<Option<ZwlrDataControlManagerV1>>>,
    gtk_manager: Rc<RefCell<Option<GtkPrimarySelectionDeviceManager>>>,
    layer_shell: Rc<RefCell<Option<ZwlrLayerShellV1>>>,
    compositor: Rc<RefCell<Option<WlCompositor>>>,
    seats: Rc<RefCell<Vec<WlSeat>>>,
}

impl wl_registry::EventHandler for WlRegistryHandler {
    fn global(&mut self, registry: WlRegistry, name: u32, interface: String, version: u32) {
        match interface.as_ref() {
            ZwlrDataControlManagerV1::NAME if version >= ZwlrDataControlManagerV1::VERSION => {
                let data_control_manager = registry.bind(ZwlrDataControlManagerV1::VERSION,
                                                         name,
                                                         NewProxy::implement_dummy)
                                                   .unwrap();
                *self.data_control_manager.borrow_mut() = Some(data_control_manager);
            }
            GtkPrimarySelectionDeviceManager::NAME
                if version >= GtkPrimarySelectionDeviceManager::VERSION =>
            {
                let gtk_manager = registry.bind(GtkPrimarySelectionDeviceManager::VERSION,
                                                name,
                                                NewProxy::implement_dummy)
                                          .unwrap();
                *self.gtk_manager.borrow_mut() = Some(gtk_manager);
            }
            ZwlrLayerShellV1::NAME if version >= ZwlrLayerShellV1::VERSION => {
                let layer_shell =
                    registry.bind(ZwlrLayerShellV1::VERSION, name, NewProxy::implement_dummy)
                            .unwrap();
                *self.layer_shell.borrow_mut() = Some(layer_shell);
            }
            WlCompositor::NAME if version >= WlCompositor::VERSION => {
                let compositor =
                    registry.bind(WlCompositor::VERSION, name, NewProxy::implement_dummy)
                            .unwrap();
                *self.compositor.borrow_mut() = Some(compositor);
            }
            WlSeat::NAME if version >= WlSeat::VERSION => {
                let seat_data = RefCell::new(SeatData::default());
                let seat = registry.bind(WlSeat::VERSION, name, |seat| {
                                       seat.implement(WlSeatHandler, seat_data)
                                   })
                                   .unwrap();

                self.seats.borrow_mut().push(seat);
            }
            _ => {}
        }
    }
}

pub struct WlSeatHandler;

impl wl_seat::EventHandler for WlSeatHandler {
    fn name(&mut self, seat: WlSeat, name: String) {
        let data = seat.as_ref().user_data::<RefCell<SeatData>>().unwrap();
        data.borrow_mut().set_name(name);
    }
}

pub struct DataDeviceHandler {
    seat: WlSeat,
    incoming_offer: Option<Offer>,
}

impl DataDeviceHandler {
    pub fn new(seat: WlSeat) -> Self {
        Self { seat,
               incoming_offer: None }
    }

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
}

impl zwlr_data_control_device_v1::EventHandler for DataDeviceHandler {
    fn data_offer(&mut self,
                  _device: ZwlrDataControlDeviceV1,
                  offer: NewProxy<ZwlrDataControlOfferV1>) {
        self.data_offer(offer.into())
    }

    fn selection(&mut self,
                 __device: ZwlrDataControlDeviceV1,
                 offer: Option<ZwlrDataControlOfferV1>) {
        self.selection(offer.map(Into::into))
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

pub struct LayerSurfaceHandler;

impl zwlr_layer_surface_v1::EventHandler for LayerSurfaceHandler {
    fn configure(&mut self, surface: ZwlrLayerSurfaceV1, serial: u32, _width: u32, _height: u32) {
        surface.ack_configure(serial)
    }
}
