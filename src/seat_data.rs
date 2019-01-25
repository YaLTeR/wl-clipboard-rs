use crate::offer::Offer;

#[derive(Default)]
pub struct SeatData {
    /// The name of this seat, if any.
    pub name: Option<String>,

    /// The data offer of this seat, if any.
    ///
    /// Contains mime types in its user data.
    pub offer: Option<Offer>,
}

impl SeatData {
    /// Sets this seat's name.
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name)
    }

    /// Sets this seat's data offer.
    ///
    /// Destroys the old one, if any.
    pub fn set_offer(&mut self, new_offer: Option<Offer>) {
        let old_offer = self.offer.take();
        self.offer = new_offer;

        if let Some(offer) = old_offer {
            offer.destroy();
        }
    }
}
