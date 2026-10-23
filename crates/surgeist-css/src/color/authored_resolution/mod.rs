//! Checked authored operands over the canonical CSS numerical resolver.

mod input;
pub(super) mod lowering;
pub(super) mod numeric;
mod path;
mod position;
mod scalar;
pub(super) mod site;
mod spelling;
#[cfg(test)]
pub(crate) use path::CssColorPathInspectionLayout;
pub use path::{CssColorComponentPath, CssColorPathStep};
pub(crate) use path::{
    CssColorPathBuffer, CssColorPathCapacityObservation, CssColorPathHandle,
    CssColorPathReservationLayout, CssColorPathReservationStatus,
};
pub(crate) use spelling::authored_hue_degrees;

mod controlled;
pub(crate) use controlled::{
    AuthoredError, CssAuthoredColorActionDescription, CssAuthoredColorOperation,
};
#[cfg(test)]
pub(crate) use controlled::{CssAuthoredColorCleanupLayout, CssAuthoredColorObservation};
pub use controlled::{CssAuthoredColorOutcome, CssAuthoredColorStandardFailure};
mod ordinary;
pub use ordinary::*;
#[cfg(test)]
mod tests;
