use derive_more::From;
use wayland_protocols::{
    misc::gtk_primary_selection::client::gtk_primary_selection_device::GtkPrimarySelectionDevice,
    unstable::primary_selection::v1::client::zwp_primary_selection_device_v1::ZwpPrimarySelectionDeviceV1,
    wlr::unstable::data_control::v1::client::zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
};

use crate::data_source::DataSource;

#[derive(From, Clone)]
pub enum DataDevice {
    DataControl(ZwlrDataControlDeviceV1),
    GtkPrimary(GtkPrimarySelectionDevice),
    WpPrimary(ZwpPrimarySelectionDeviceV1),
}

impl DataDevice {
    /// Sets or clears the selection.
    ///
    /// If a serial is required (see `ClipboardManager::requires_serial()`), `serial` must not be
    /// `None`.
    pub fn set_selection(&self, source: Option<&DataSource>, serial: Option<u32>) {
        match self {
            DataDevice::DataControl(device) => device.set_selection(source.map(AsRef::as_ref)),
            DataDevice::GtkPrimary(device) => {
                device.set_selection(source.map(AsRef::as_ref), serial.unwrap())
            }
            DataDevice::WpPrimary(device) => {
                device.set_selection(source.map(AsRef::as_ref), serial.unwrap())
            }
        }
    }

    pub fn destroy(&self) {
        match self {
            DataDevice::DataControl(device) => device.destroy(),
            DataDevice::GtkPrimary(device) => device.destroy(),
            DataDevice::WpPrimary(device) => device.destroy(),
        }
    }
}
