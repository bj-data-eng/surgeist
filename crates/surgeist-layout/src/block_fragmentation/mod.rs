//! Source-bound Block fragmentation and atomic box publication.
mod compute;
mod source;
pub(crate) use compute::compute;
mod input;
mod output;
mod publication;
pub use input::{
    BlockFragmentainerOf, BlockFragmentationFailure, LayoutBoundRequestOf,
    LayoutPublicationTargets, LayoutSourceBasis, LayoutTargetBasis,
};
pub(crate) use input::{BoundMode, FragmentBoxAllocation};
pub use output::{
    BoxCoordinateSpace, BoxFragmentGeometryOf, BoxFragmentOutputOf, BoxInstanceRefOf, LayoutBoxId,
    LayoutFragmentainerId, LayoutFragmentainerOutputOf, LayoutHandleError, NodeLayoutRefOf,
    NonBoxGeometryOf, NonBoxKind, NonBoxOutputRefOf, SingleBoxOutputRefOf,
};
pub(crate) use output::{NodePublication, ResultIdentity, SingleBox};
pub(crate) use publication::check_target;
pub use publication::{LayoutBatchApplyError, LayoutBatchCommit, LayoutBatchPreparation};
