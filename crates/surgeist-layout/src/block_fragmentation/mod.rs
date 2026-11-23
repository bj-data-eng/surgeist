//! Source-bound Block fragmentation and atomic box publication.
mod compute;
mod scope_assembly;
mod scope_view;
mod scopes;
mod source;
pub(crate) use compute::compute;
pub(crate) mod input;
mod output;
pub(crate) mod publication;
pub use input::{BlockFragmentainerOf, BlockFragmentationFailure, LayoutBoundRequestOf};
pub(crate) use input::{BoundMode, FragmentBoxAllocation};
pub use output::{
    BoxFragmentGeometryOf, BoxGeometryPairOf, FragmentCoordinateSpace, FragmentOutputOf,
    FragmentPayloadOf, LayoutFragmentId, LayoutFragmentainerId, LayoutFragmentainerOutputOf,
    LayoutHandleError, NodeLayoutRefOf, NonBoxGeometryOf, NonBoxKind, NonBoxOutputRefOf,
    OverflowFragmentGeometryOf, OverflowGeometryPairOf, SingleBoxOutputRefOf,
};
pub(crate) use output::{NodePublication, ResultIdentity};
pub(crate) use publication::check_target;
#[cfg(test)]
mod result_tests;
