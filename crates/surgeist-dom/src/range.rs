//! Ranges, their boundaries and operands, and the designMode range transition.
//!
//! `DomRangeId` identifies a live Range of the owner. `DomRange` and `DomRangeBoundary`
//! are views captured in a snapshot; later mutation or retirement cannot change them.
//! `DomRangeOperand` names a Range in a batch and `CreatedRange` is the batch-local
//! creation ticket. `DomActiveRangeState`, `DomSourceDesignModeRequest`,
//! `DomSourceDesignModeStep` and `DomDesignModeRangeDisposition` belong to the designMode
//! setter and its Source continuation.
//!
//! Retained capacity is selected with `DomLimits::with_ranges` and defaults to zero.
//! `docs/ranges-and-editing.md` owns the contract.

pub use crate::internal::range::{
    CreatedRange, DomActiveRangeState, DomRange, DomRangeBoundary, DomRangeId, DomRangeOperand,
};
pub use crate::internal::source::{
    DomDesignModeRangeDisposition, DomSourceDesignModeRequest, DomSourceDesignModeStep,
};
pub use crate::internal::store::DomOwnedRange;
