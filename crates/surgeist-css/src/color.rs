//! Used-color operations over original source or checked authored values and facts.

pub(crate) mod authored;
#[cfg(test)]
mod authored_input_tests;
mod authored_resolution;
pub(crate) use authored_resolution::authored_hue_degrees;
pub use authored_resolution::*;
use surgeist_color::{Color, ColorError};

mod bounded;
mod component;
mod component_value;
mod decimal;
mod expression;
mod facts;
pub(crate) mod keywords;
mod literal;
mod operation;
mod ordinary;
mod protocol;
mod resolution;
mod syntax_pass;
mod work;
pub use bounded::*;
pub use ordinary::*;
pub use protocol::{
    CssColorBank, CssColorOperationOwner, CssColorProtocolError, CssColorSlot, CssColorSlotKind,
};
pub use work::CssColorKernel;
pub(crate) mod channels;
mod device;
mod grammar;
pub(crate) mod hex;
mod interpolate;
mod palette;
mod profile;
mod profile_conversion;
mod relative;
mod tokens;
pub use facts::{
    CssColorFactIdentity, CssColorFacts, CssColorPaletteEntry, CssColorProfileAvailability,
    CssColorProfileBinding, CssColorProfileConversion, CssColorProfileConversionRecord,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
};
