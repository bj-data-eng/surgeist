//! The caller-serviced colour parse protocol.
//!
//! [`CssColorParse`] is started with an exclusive [`CssColorOperationOwner`] and then yields one
//! finite action at a time: read an input unit, read a fact, write a record into a caller-owned
//! bank, or run a fixed-register kernel step. The caller services each action and the parse ends
//! in a [`CssColorParseOutcome`]. A used color is a [`CssUsedColorCheckpoint`], which a
//! [`CssColorTransform`] can transform numerically. [`parse_used_color`](crate::color::parse_used_color)
//! drives this protocol with ordinary limits for callers that do not need to service it.

pub(crate) mod bounded;
pub(crate) mod protocol;
pub(crate) mod work;

#[doc(inline)]
pub use bounded::{
    CssColorActionLayout, CssColorCompute, CssColorFactRead, CssColorFailureCheckpoint,
    CssColorInitializationLayout, CssColorParse, CssColorParseAction, CssColorParseOutcome,
    CssColorParseStep, CssColorReadFact, CssColorReadInput, CssColorReadSlot, CssColorSlotReceipt,
    CssColorTransform, CssColorWriteSlot, CssUsedColorCheckpoint,
};
#[doc(inline)]
pub use protocol::{CssColorBank, CssColorOperationOwner, CssColorSlot};
#[doc(inline)]
pub use work::CssColorKernel;
