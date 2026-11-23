//! Break controls, fragmentation kinds, page sides and progression, and the native Block fragmentation request and
//! results.
//!
//! The break vocabulary of a node is [`FragmentationBreak`] and [`FragmentationBreakInside`]; a boundary crosses
//! [`FragmentationBreakKinds`] of a [`FragmentationKind`] (including a page with its [`PageProgression`] and
//! [`FragmentationPageSide`]). A host requests native Block fragmentation with a [`LayoutBoundRequestOf`]
//! (`LayoutBoundRequestOf::block_fragmentation`, which reports a [`FragmentationInputError`]) over external
//! [`BlockFragmentainerOf`] rectangles. The completed result is read back as [`NodeLayoutRefOf`]: a single box,
//! fragments ([`FragmentOutputOf`], [`BoxFragmentGeometryOf`], [`OverflowFragmentGeometryOf`]) with their
//! [`FragmentCoordinateSpace`], or a non-box participant ([`NonBoxGeometryOf`]). [`BlockFragmentationFailure`] and
//! [`LayoutHandleError`] report refusals.

pub use crate::block_fragmentation::{
    BlockFragmentainerOf, BlockFragmentationFailure, BoxFragmentGeometryOf, BoxGeometryPairOf,
    FragmentCoordinateSpace, FragmentOutputOf, FragmentPayloadOf, LayoutBoundRequestOf,
    LayoutFragmentId, LayoutFragmentainerId, LayoutFragmentainerOutputOf, LayoutHandleError,
    NodeLayoutRefOf, NonBoxGeometryOf, NonBoxKind, NonBoxOutputRefOf, OverflowFragmentGeometryOf,
    OverflowGeometryPairOf, SingleBoxOutputRefOf,
};
pub use crate::fragmentation::input::{
    FragmentationBreak, FragmentationBreakInside, FragmentationBreakKinds, FragmentationKind,
};
pub use crate::fragmentation::output::{FragmentationGraphicalCoverageOf, FragmentationInputError};
pub use crate::fragmentation::page::{
    FragmentationLogicalPageSide, FragmentationPageSide, FragmentationPhysicalPageSide,
    PageProgression,
};

pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod page;
mod policy;
pub(crate) mod scheduler;
mod validation;
use input::FragmentationBreakControl;
pub(crate) use scheduler::{
    AcceptedStep, CompletedExecution, Execution, ScopeEndpointOf, ScopeHeadProjectionOf,
    ScopeProjectionOf, ScopeTicket,
};

/// Tests contextual applicability through the same force-policy owner used by
/// scheduling, without exposing a selected Force or selecting a boundary.
pub(crate) fn controls_require_break(
    before: &[FragmentationBreakControl],
    after: &[FragmentationBreakControl],
    available: FragmentationBreakKinds,
) -> bool {
    policy::force_controls(before, after, available).is_some()
}
