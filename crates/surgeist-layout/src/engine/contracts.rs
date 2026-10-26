use crate::error::LayoutResultOf;
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputOf, LayoutInputViewOf,
    LayoutScalar, NodeInputOf, NodeOutputOf, RunMode, Traverse,
};

pub(crate) type WrappingContextLoan<'a, Node> = (
    &'a std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>,
    &'a crate::engine::validation::ValidationGraph<Node>,
);

// Input loans refer to immutable request storage, independently of temporary
// borrows of the mutable output/cache computation facade.
pub(crate) trait Compute<'input, M = core::convert::Infallible>:
    Traverse<Node: 'input>
{
    fn node_input(&self, node: Self::Node) -> &'input NodeInputOf<Self::Scalar>;
    fn layout_input(&self, node: Self::Node) -> LayoutInputViewOf<'input, Self::Scalar>;
    /// Immutable normalized scope authority, separate from optional Text progress.
    fn wrapping_context(&self) -> Option<WrappingContextLoan<'_, Self::Node>> {
        None
    }
    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);
    // Absence is the actual box-only profile, not a successful no-op finalizer.
    fn text_computation(
        &mut self,
    ) -> Option<&mut crate::text_layout::state::TextComputation<'input, Self::Node, Self::Scalar>>
    {
        None
    }

    fn set_unrounded_inline_fragment_state(
        &mut self,
        _node: Self::Node,
        _fragments: Option<Vec<InlineFragmentOutputOf<Self::Scalar>>>,
    ) {
    }
    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, M>;

    // Return only actual Grid overflow observations. Implementations isolate
    // every probe output/fragment/cache write from their publishing computation.
    fn observe_grid_overflow(
        &mut self,
        node: Self::Node,
        probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
    ) -> LayoutResultOf<
        Self::Node,
        crate::grid::GridOverflowObservation<Self::Scalar>,
        Self::Scalar,
        M,
    >;

    /// Prepared-text floats require the actual session's disposable staging
    /// owner. Box-only executors have no captured Text publication to merge.
    fn probe_float_child(
        &mut self,
        node: Self::Node,
        _input: ComputeInputOf<Self::Scalar>,
    ) -> super::session::FloatProbeResult<Self, M> {
        Err(crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::ChildLayout,
            crate::LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::TextCapture),
        ))
    }
    fn accept_float_child(
        &mut self,
        packet: &super::session::FloatChildPacket<Self::Node, Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, M> {
        Err(crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(packet.node),
            crate::LayoutOperation::ChildLayout,
            crate::LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::TextCapture),
        ))
    }

    fn float_exclusion_interval(
        &self,
        _node: Self::Node,
        _query: crate::FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<Result<Option<crate::FloatExclusionIntervalOf<Self::Scalar>>, M>> {
        None
    }
}

pub(crate) trait Round<M = core::convert::Infallible>: Traverse {
    fn is_prepared_text(&self, _node: Self::Node) -> bool {
        false
    }
    fn canonical_text_atomic_origin(&self, _node: Self::Node) -> Option<crate::Point<f64>> {
        None
    }

    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<Self::Node, NodeOutputOf<Self::Scalar>, Self::Scalar, M>;
    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>);

    fn unrounded_inline_fragment_state(
        &self,
        _node: Self::Node,
    ) -> UnroundedInlineFragmentState<'_, Self::Scalar> {
        UnroundedInlineFragmentState::Absent
    }

    fn set_final_inline_fragments(
        &mut self,
        _node: Self::Node,
        _unrounded: Vec<InlineFragmentOutputOf<Self::Scalar>>,
        _final_fragments: Vec<InlineFragmentOutputOf<Self::Scalar>>,
    ) {
    }
}

pub(crate) enum UnroundedInlineFragmentState<'a, S: LayoutScalar> {
    Absent,
    Present(&'a [InlineFragmentOutputOf<S>]),
}

pub(crate) trait CacheAccess<M = core::convert::Infallible> {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;

    fn cache_context(&self) -> CacheKeyContext;
    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>>;
    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    );
    /// Restore invocation-local semantic receipts accompanying an admitted
    /// geometry hit. Box-only executors have no such additional state.
    fn cache_restore_semantics(
        &mut self,
        _node: Self::Node,
        _input: &ComputeInputOf<Self::Scalar>,
    ) -> bool {
        true
    }
    fn cache_clear(&mut self, node: Self::Node);
}

pub(crate) fn compute_cached<Tree, ComputeFn, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    compute: ComputeFn,
) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: CacheAccess<M> + ?Sized,
    ComputeFn: FnOnce(
        &mut Tree,
        Tree::Node,
        ComputeInputOf<Tree::Scalar>,
    )
        -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>,
{
    if input.run_mode() == RunMode::PerformHiddenLayout {
        return compute(tree, node, input);
    }

    let context = tree.cache_context();
    if let Some(output) = tree.cache_get(node, &input, context)
        && tree.cache_restore_semantics(node, &input)
    {
        return Ok(output);
    }

    let output = compute(tree, node, input)?;
    tree.cache_store(node, &input, context, output);
    Ok(output)
}
