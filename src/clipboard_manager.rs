use derive_more::From;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_protocols::{
    unstable::primary_selection::v1::client::{
        zwp_primary_selection_device_manager_v1::ZwpPrimarySelectionDeviceManagerV1, *,
    },
    wlr::unstable::data_control::v1::client::{
        zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, *,
    },
};

use crate::data_device::DataDevice;
use crate::protocol::gtk_primary_selection::client::{
    gtk_primary_selection_device_manager::GtkPrimarySelectionDeviceManager, *,
};

#[derive(From)]
pub enum ClipboardManager {
    DataControl(ZwlrDataControlManagerV1),
    GtkPrimary(GtkPrimarySelectionDeviceManager),
    WpPrimary(ZwpPrimarySelectionDeviceManagerV1),
}

impl ClipboardManager {
    pub fn get_device<T>(&self, seat: &WlSeat, handler: T) -> Result<DataDevice, ()>
        where T: zwlr_data_control_device_v1::EventHandler
                  + gtk_primary_selection_device::EventHandler
                  + zwp_primary_selection_device_v1::EventHandler
                  + 'static
    {
        match self {
            ClipboardManager::DataControl(manager) => {
                manager.get_data_device(seat, move |device| device.implement(handler, ()))
                       .map(Into::into)
            }
            ClipboardManager::GtkPrimary(manager) => {
                manager.get_device(seat, move |device| device.implement(handler, ()))
                       .map(Into::into)
            }
            ClipboardManager::WpPrimary(manager) => {
                manager.get_device(seat, move |device| device.implement(handler, ()))
                       .map(Into::into)
            }
        }
    }

    pub fn requires_keyboard_focus(&self) -> bool {
        match self {
            ClipboardManager::DataControl(_) => false,
            ClipboardManager::GtkPrimary(_) => true,
            ClipboardManager::WpPrimary(_) => true,
        }
    }
}
