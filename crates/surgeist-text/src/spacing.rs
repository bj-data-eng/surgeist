//! Selected spacing slots, allocations and spacing diagnostics.

pub use crate::error::{
    SpacingConstraint, SpacingConstraintKind, SpacingContext, SpacingFailureReason,
    SpacingOperation, SpacingPiece, SpacingStage,
};
pub use crate::protocol::{
    SpacingAllocation, SpacingAllocations, SpacingBoundaryGeometry, SpacingDeferredCause,
    SpacingPolicy, SpacingSlot, SpacingSlotDisposition, SpacingSlotFeedback, SpacingSlotGeometry,
    SpacingSlotOwner, SpacingSlotPiece, SpacingSlotTarget, SpacingStretchability, SpacingUnit,
};
