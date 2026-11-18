//! Selected spacing slots, allocations and spacing diagnostics.
//!
//! [`SpacingPolicy`] is the uniform native selected spacing operation. A selected request issues
//! [`SpacingSlot`] values with [`SpacingSlotOwner`], [`SpacingSlotPiece`], [`SpacingSlotTarget`],
//! [`SpacingSlotGeometry`], [`SpacingSlotDisposition`] and [`SpacingSlotFeedback`]; [`SpacingAllocation`],
//! [`SpacingAllocations`], [`SpacingUnit`], [`SpacingBoundaryGeometry`], [`SpacingDeferredCause`] and
//! [`SpacingStretchability`] describe allocations and units. [`SpacingFailureReason`], [`SpacingConstraint`],
//! [`SpacingConstraintKind`], [`SpacingContext`], [`SpacingOperation`], [`SpacingPiece`] and [`SpacingStage`]
//! describe a rejected spacing operation.

pub use crate::error::{
    SpacingConstraint, SpacingConstraintKind, SpacingContext, SpacingFailureReason,
    SpacingOperation, SpacingPiece, SpacingStage,
};
pub use crate::protocol::{
    SpacingAllocation, SpacingAllocations, SpacingBoundaryGeometry, SpacingDeferredCause,
    SpacingPolicy, SpacingSlot, SpacingSlotDisposition, SpacingSlotFeedback, SpacingSlotGeometry,
    SpacingSlotOwner, SpacingSlotPiece, SpacingSlotTarget, SpacingStretchability, SpacingUnit,
};
