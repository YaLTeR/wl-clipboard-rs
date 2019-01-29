use derive_more::From;
use wayland_client::{protocol::wl_seat::WlSeat, Interface};
use wayland_protocols::{
    misc::gtk_primary_selection::client::{
        gtk_primary_selection_device_manager::GtkPrimarySelectionDeviceManager, *,
    },
    unstable::primary_selection::v1::client::{
        zwp_primary_selection_device_manager_v1::ZwpPrimarySelectionDeviceManagerV1, *,
    },
    wlr::unstable::data_control::v1::client::{
        zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, *,
    },
};

use crate::{data_device::DataDevice, data_source::DataSource};

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

    pub fn create_source<T, UD>(&self, handler: T, user_data: UD) -> Result<DataSource, ()>
        where T: zwlr_data_control_source_v1::EventHandler
                  + gtk_primary_selection_source::EventHandler
                  + zwp_primary_selection_source_v1::EventHandler
                  + 'static,
              UD: 'static
    {
        match self {
            ClipboardManager::DataControl(manager) => {
                manager.create_data_source(move |source| source.implement(handler, user_data))
                       .map(Into::into)
            }
            ClipboardManager::GtkPrimary(manager) => {
                manager.create_source(move |source| source.implement(handler, user_data))
                       .map(Into::into)
            }
            ClipboardManager::WpPrimary(manager) => {
                manager.create_source(move |source| source.implement(handler, user_data))
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

    pub fn requires_serial(&self) -> bool {
        // Happens to coincide.
        self.requires_keyboard_focus()
    }

    pub fn name(&self) -> &'static str {
        match self {
            ClipboardManager::DataControl(_) => ZwlrDataControlManagerV1::NAME,
            ClipboardManager::GtkPrimary(_) => GtkPrimarySelectionDeviceManager::NAME,
            ClipboardManager::WpPrimary(_) => ZwpPrimarySelectionDeviceManagerV1::NAME,
        }
    }
}
