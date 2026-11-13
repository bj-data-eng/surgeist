use crate::measurement::LeafMeasureInputOf;
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, FloatExclusionIntervalOf,
    FloatExclusionQueryOf, LayoutInputViewOf, LayoutScalar, NodeInputOf, Size,
};

type FloatExclusionProviderResultOf<S, M> = Result<Option<FloatExclusionIntervalOf<S>>, M>;

pub trait Traverse {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;
    type Children<'a>: Iterator<Item = Self::Node>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_>;
    fn child_count(&self, node: Self::Node) -> usize;
    fn child(&self, node: Self::Node, index: usize) -> Self::Node;
}

pub trait LayoutTree: Traverse {
    type MeasureError;

    /// Returns this node's canonical box facts. For a box participant this must
    /// reference the same stored input as [`Self::layout_input`]. Non-box nodes
    /// supply canonical non-box facts, checked by the layout front door.
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar>;
    /// Borrows the normalized participant from immutable request storage.
    /// Read access must not clone input backing or allocate a carrier.
    fn layout_input(&self, node: Self::Node) -> LayoutInputViewOf<'_, Self::Scalar>;

    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        false
    }

    /// Measures the immutable leaf for the supplied content-space request.
    /// Within one layout call, the same node and request may reuse a
    /// successful measurement.
    fn measure_leaf(
        &self,
        _node: Self::Node,
        _input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        None
    }

    fn cache_context(&self) -> CacheKeyContext {
        CacheKeyContext::new()
    }

    fn cache_get(
        &self,
        _node: Self::Node,
        _input: &ComputeInputOf<Self::Scalar>,
        _context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        None
    }

    /// Returns the original committed unrounded entry of a warm cached subtree.
    ///
    /// Retain the opaque entry from typed SingleBox or NonBox unrounded custody, including its
    /// actual normal/visual geometry and readback phase. A final entry or a
    /// visual output alone is not unrounded custody. Missing, wrong-node,
    /// wrong-phase or mismatched canonical-frame custody prevents reuse; the
    /// existing compute-input and cache-context checks still apply.
    fn unrounded_layout(
        &self,
        _node: Self::Node,
    ) -> Option<crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>> {
        None
    }

    /// Queries one shape-backed float exclusion interval for a physical band.
    ///
    /// Outer `None` means the requested provider is absent. `Ok(None)` means
    /// the shape has no exclusion intersection with this band. Layout invokes
    /// this provider for inline wrapping around a shape float overlapping a
    /// finite candidate band. Float placement uses physical margin boxes.
    fn float_exclusion_interval(
        &self,
        _node: Self::Node,
        _query: FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<FloatExclusionProviderResultOf<Self::Scalar, Self::MeasureError>> {
        None
    }
}

/// Atomically prepares and commits every state class in a completed layout batch.
///
/// Preparation is immutable and may fail. Commit receives an owned prepared
/// replacement under exclusive access and is infallible. Keep the registry with
/// its coupled visible/cache state and ownership history. No other access path
/// may change target, membership or visible publication between the final target
/// recheck and commit. The call-scoped carriers cannot authorize later replay.
pub trait LayoutBatchSink<Node, S: LayoutScalar> {
    type Error;
    type Prepared;

    fn publication_targets(&self) -> &crate::LayoutPublicationTargets<Node>;
    fn publication_targets_mut(&mut self) -> &mut crate::LayoutPublicationTargets<Node>;

    fn prepare_layout_batch<'call>(
        &self,
        preparation: crate::LayoutBatchPreparation<'call, Node, S>,
    ) -> Result<Self::Prepared, Self::Error>;

    fn commit_layout_batch<'call>(
        &mut self,
        commit: crate::LayoutBatchCommit<'call, Node, S, Self::Prepared>,
    );
}
