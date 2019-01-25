use derive_more::From;
use wayland_protocols::{
    unstable::primary_selection::v1::client::zwp_primary_selection_device_v1::ZwpPrimarySelectionDeviceV1,
    wlr::unstable::data_control::v1::client::zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
};

use crate::protocol::gtk_primary_selection::client::gtk_primary_selection_device::GtkPrimarySelectionDevice;

#[derive(From)]
pub enum DataDevice {
    DataControl(ZwlrDataControlDeviceV1),
    GtkPrimary(GtkPrimarySelectionDevice),
    WpPrimary(ZwpPrimarySelectionDeviceV1),
}
