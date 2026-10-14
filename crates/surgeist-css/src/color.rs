//! Standalone used-color operations over explicit source and facts.

pub(crate) mod alpha;
pub(crate) mod authored;
mod numerical;
pub use numerical::value::{CssUsedColor, CssUsedColorError, CssUsedColorTransformation};

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
mod serialize;
mod syntax_pass;
mod work;
pub use bounded::*;
pub use ordinary::*;
pub use protocol::{
    CssColorBank, CssColorOperationOwner, CssColorProtocolError, CssColorSlot, CssColorSlotKind,
};
pub use work::CssColorKernel;
mod channels;
mod device;
mod grammar;
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
pub use numerical::serialization::CssColorSerializationRequest;
