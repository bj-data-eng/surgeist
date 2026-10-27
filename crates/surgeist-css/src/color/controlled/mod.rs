//! The caller-serviced colour parse protocol.

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
