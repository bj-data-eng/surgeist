//! Checked ICC conversion over caller-supplied profile bytes.
//!
//! The profile view borrows immutable bytes. Conversion qualifies the selected
//! transform and accounts for its temporary state through Render Ledger.

#[doc(inline)]
pub use crate::color_profile::native::{
    IccDeviceColor, IccDeviceSpace, IccPcsColor, IccPcsSpace, IccProfile, IccProfileClass,
    IccProfileError, IccRenderingIntent,
};
