use derive_more::From;
use wayland_protocols::{
    unstable::primary_selection::v1::client::zwp_primary_selection_source_v1::ZwpPrimarySelectionSourceV1,
    wlr::unstable::data_control::v1::client::zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
};

use crate::protocol::gtk_primary_selection::client::gtk_primary_selection_source::GtkPrimarySelectionSource;

#[derive(From, Clone)]
pub enum DataSource {
    DataControl(ZwlrDataControlSourceV1),
    GtkPrimary(GtkPrimarySelectionSource),
    WpPrimary(ZwpPrimarySelectionSourceV1),
}

impl DataSource {
    pub fn offer(&self, mime_type: String) {
        match self {
            DataSource::DataControl(source) => source.offer(mime_type),
            DataSource::GtkPrimary(source) => source.offer(mime_type),
            DataSource::WpPrimary(source) => source.offer(mime_type),
        }
    }

    pub fn destroy(&self) {
        match self {
            DataSource::DataControl(source) => source.destroy(),
            DataSource::GtkPrimary(source) => source.destroy(),
            DataSource::WpPrimary(source) => source.destroy(),
        }
    }

    pub fn user_data<UD: 'static>(&self) -> Option<&UD> {
        match self {
            DataSource::DataControl(source) => source.as_ref().user_data(),
            DataSource::GtkPrimary(source) => source.as_ref().user_data(),
            DataSource::WpPrimary(source) => source.as_ref().user_data(),
        }
    }
}

impl AsRef<ZwlrDataControlSourceV1> for DataSource {
    fn as_ref(&self) -> &ZwlrDataControlSourceV1 {
        match self {
            DataSource::DataControl(source) => source,
            _ => panic!("Trying to get a reference to the wrong data source type"),
        }
    }
}

impl AsRef<GtkPrimarySelectionSource> for DataSource {
    fn as_ref(&self) -> &GtkPrimarySelectionSource {
        match self {
            DataSource::GtkPrimary(source) => source,
            _ => panic!("Trying to get a reference to the wrong data source type"),
        }
    }
}

impl AsRef<ZwpPrimarySelectionSourceV1> for DataSource {
    fn as_ref(&self) -> &ZwpPrimarySelectionSourceV1 {
        match self {
            DataSource::WpPrimary(source) => source,
            _ => panic!("Trying to get a reference to the wrong data source type"),
        }
    }
}
