use crate::utils::{is_primary_selection_supported, PrimarySelectionCheckError};

#[repr(C)]
pub enum WcrsPrimarySelectionCheckStatus {
    Success = 0,
    NoSeats,
    WaylandConnection,
    WaylandCommunication,
    MissingProtocol,
}

impl From<PrimarySelectionCheckError> for WcrsPrimarySelectionCheckStatus {
    #[inline]
    fn from(x: PrimarySelectionCheckError) -> Self {
        match x {
            PrimarySelectionCheckError::NoSeats => Self::NoSeats,
            PrimarySelectionCheckError::WaylandConnection { .. } => Self::WaylandConnection,
            PrimarySelectionCheckError::WaylandCommunication { .. } => Self::WaylandCommunication,
            PrimarySelectionCheckError::MissingProtocol { .. } => Self::MissingProtocol,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcrs_is_primary_selection_supported(supported: *mut i8)
                                                             -> WcrsPrimarySelectionCheckStatus {
    match is_primary_selection_supported() {
        Ok(value) => {
            *supported = if value { 1 } else { 0 };
            WcrsPrimarySelectionCheckStatus::Success
        }
        Err(err) => err.into(),
    }
}
