use std::os::unix::io::RawFd;

use derive_more::From;
use wayland_client::NewProxy;
use wayland_protocols::{
    misc::gtk_primary_selection::client::{
        gtk_primary_selection_offer::GtkPrimarySelectionOffer, *,
    },
    unstable::primary_selection::v1::client::{
        zwp_primary_selection_offer_v1::ZwpPrimarySelectionOfferV1, *,
    },
    wlr::unstable::data_control::v1::client::{
        zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, *,
    },
};

#[derive(From, Clone)]
pub enum Offer {
    DataControl(ZwlrDataControlOfferV1),
    GtkPrimary(GtkPrimarySelectionOffer),
    WpPrimary(ZwpPrimarySelectionOfferV1),
}

impl Offer {
    pub fn destroy(&self) {
        match self {
            Offer::DataControl(offer) => offer.destroy(),
            Offer::GtkPrimary(offer) => offer.destroy(),
            Offer::WpPrimary(offer) => offer.destroy(),
        }
    }

    pub fn receive(&self, mime_type: String, fd: RawFd) {
        match self {
            Offer::DataControl(offer) => offer.receive(mime_type, fd),
            Offer::GtkPrimary(offer) => offer.receive(mime_type, fd),
            Offer::WpPrimary(offer) => offer.receive(mime_type, fd),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Offer::DataControl(offer) => offer.as_ref().id(),
            Offer::GtkPrimary(offer) => offer.as_ref().id(),
            Offer::WpPrimary(offer) => offer.as_ref().id(),
        }
    }

    pub fn user_data<UD: 'static>(&self) -> Option<&UD> {
        match self {
            Offer::DataControl(offer) => offer.as_ref().user_data(),
            Offer::GtkPrimary(offer) => offer.as_ref().user_data(),
            Offer::WpPrimary(offer) => offer.as_ref().user_data(),
        }
    }
}

#[derive(From)]
pub enum NewOffer {
    DataControl(NewProxy<ZwlrDataControlOfferV1>),
    GtkPrimary(NewProxy<GtkPrimarySelectionOffer>),
    WpPrimary(NewProxy<ZwpPrimarySelectionOfferV1>),
}

impl NewOffer {
    pub fn implement<T, UD>(self, handler: T, user_data: UD) -> Offer
        where T: zwlr_data_control_offer_v1::EventHandler
                  + gtk_primary_selection_offer::EventHandler
                  + zwp_primary_selection_offer_v1::EventHandler
                  + 'static,
              UD: 'static
    {
        match self {
            NewOffer::DataControl(offer) => offer.implement(handler, user_data).into(),
            NewOffer::GtkPrimary(offer) => offer.implement(handler, user_data).into(),
            NewOffer::WpPrimary(offer) => offer.implement(handler, user_data).into(),
        }
    }
}
