use crate::error::LayoutResultOf;
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputOf, LayoutInputViewOf,
    LayoutOutputEntryOf, LayoutScalar, NodeInputOf, Traverse,
};

pub(crate) type WrappingContextLoan<'a, Node> = (
    &'a std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>,
    &'a crate::engine::validation::ValidationGraph<Node>,
);

pub(crate) type UnroundedEntryResult<Node, S, M> =
    LayoutResultOf<Node, LayoutOutputEntryOf<Node, S>, S, M>;

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
    fn group_computation(
        &mut self,
    ) -> Option<&mut crate::group_alignment::State<Self::Node, Self::Scalar>> {
        None
    }
    fn block_float_state(
        &mut self,
    ) -> Option<&mut crate::block::float_context::State<Self::Node, Self::Scalar>> {
        None
    }
    fn fragment_margin_state(
        &mut self,
    ) -> Option<&mut crate::block::fragment_margins::State<Self::Node, Self::Scalar>> {
        None
    }
    fn fragment_minimum(
        &self,
        _node: Self::Node,
    ) -> Option<crate::block::fragment_minimum::Override<Self::Scalar>> {
        None
    }
    fn has_leaf_measurement(&self, _node: Self::Node) -> bool {
        false
    }
    fn qualify_entry(
        &mut self,
        query: crate::block::entry::EntryQuery<Self::Node, Self::Scalar>,
    ) -> crate::block::entry::EntryResult<Self::Node, Self::Scalar, M>
    where
        Self: Sized,
    {
        crate::block::entry::observe(self, query)
    }
    /// A framed entry demand samples the actual leaf producer, without host
    /// geometry-cache authority. Executors with such a cache override this.
    fn measure_entry_leaf(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, M> {
        if !self.has_leaf_measurement(node) {
            return Err(crate::block::float_context::missing(node));
        }
        self.compute_child(node, input.with_run_mode(crate::RunMode::ComputeSize))
    }
    /// Real used-box sizing of an independent source float in a disposable
    /// entry traversal. The packet producer retains its detached body policy;
    /// no complete packet or descendant publication escapes entry qualification.
    fn measure_entry_float(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, M> {
        self.probe_float_child(node, input, super::session::FloatChildTrial::Detached)
            .map(|packet| packet.output)
    }
    fn block_float_work(&self, node: Self::Node) -> bool {
        crate::block::float_context::discover_work(self, node)
    }
    fn begin_child_float_context(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
        frame: crate::block::float_context::ChildFrame<Self::Node, Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, M> {
        self.block_float_state()
            .ok_or_else(|| crate::block::float_context::missing(node))?
            .push(node, input, frame.exclusions, frame.cut)
            .map_err(|error| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(node),
                    crate::LayoutOperation::ChildLayout,
                    crate::LayoutErrorKindOf::Text(error),
                )
            })
    }

    fn end_child_float_context(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> Option<crate::block::float_context::Completed<Self::Node, Self::Scalar>> {
        self.block_float_state()
            .and_then(|state| state.pop(node, input))
    }
    /// Actual accepted current-trial custody; never a host/cache reconstruction.
    fn staged_unrounded(
        &self,
        node: Self::Node,
    ) -> UnroundedEntryResult<Self::Node, Self::Scalar, M>;
    fn set_unrounded(&mut self, entry: LayoutOutputEntryOf<Self::Node, Self::Scalar>);
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
    /// Publish only an accepted native run, never candidate/intrinsic geometry.
    fn publish_native_inline_run(
        &mut self,
        node: Self::Node,
        _source: std::ops::Range<usize>,
        completion: Option<std::sync::Arc<crate::inline_wrapping::NativeOrderingProjection>>,
        lines: Vec<crate::inline_wrapping::family::NativeLineReceipt<Self::Scalar>>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, M> {
        if lines.is_empty() && completion.is_none() {
            return Ok(());
        }
        Err(crate::inline_wrapping::wrapping_error(
            node,
            crate::WrappingScopeError::UnsupportedIncidence,
        ))
    }
    /// A demanded physical scalar; a Block inline component never requests
    /// undemanded Normal block progression from a descendant.
    fn compute_axis_demand(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
        axis: crate::PhysicalAxis,
    ) -> LayoutResultOf<Self::Node, Self::Scalar, Self::Scalar, M>
    where
        Self: Sized,
    {
        if !self.has_leaf_measurement(node)
            && self.node_input(node).display.inner_display() == crate::Display::Block
        {
            return crate::block::intrinsic::axis_demand(self, node, input, axis);
        }
        let output = self.compute_child(node, input)?;
        Ok(match axis {
            crate::PhysicalAxis::Horizontal => output.size.width,
            crate::PhysicalAxis::Vertical => output.size.height,
        })
    }

    /// An installed Block scope driver requires prepared entry geometry for
    /// owned scopes and their ancestor paths.
    fn block_scope_entry_required(&self, _node: Self::Node) -> bool {
        false
    }

    fn compute_block_child(
        &mut self,
        request: crate::block::scope::ChildRequest<Self::Node, Self::Scalar>,
    ) -> super::block_scope::ComputeResult<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, M>
    where
        Self: Sized,
    {
        self.compute_child(request.node, request.input)
            .map_err(Into::into)
    }

    fn fragment_inline_content(
        &self,
        _node: Self::Node,
        _input: ComputeInputOf<Self::Scalar>,
        _operation: crate::SizingOperation,
    ) -> LayoutResultOf<Self::Node, Option<Self::Scalar>, Self::Scalar, M> {
        Ok(None)
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

    /// Float-dependent layout trials use the actual session's disposable
    /// staging owner. Accepting the trial publishes its complete child result.
    fn probe_float_child(
        &mut self,
        node: Self::Node,
        _input: ComputeInputOf<Self::Scalar>,
        _trial: super::session::FloatChildTrial,
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

    fn unrounded(&self, node: Self::Node) -> UnroundedEntryResult<Self::Node, Self::Scalar, M>;
    fn set_final(&mut self, entry: LayoutOutputEntryOf<Self::Node, Self::Scalar>);
    fn record_fragment_anchor(
        &mut self,
        _node: Self::Node,
        _anchor: super::rounding::FragmentAnchor<Self::Scalar>,
    ) {
    }

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

pub(crate) type CacheLookupResult<Node, S, M = core::convert::Infallible> =
    LayoutResultOf<Node, Option<ComputeOutputOf<S>>, S, M>;

pub(crate) trait CacheAccess<M = core::convert::Infallible> {
    type Node: Copy + Eq;
    type Scalar: LayoutScalar;

    fn cache_context(&self) -> CacheKeyContext;
    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
    ) -> CacheLookupResult<Self::Node, Self::Scalar, M>;
    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, M>;
    /// Restore invocation-local semantic receipts accompanying an admitted
    /// geometry hit. Box-only executors have no such additional state.
    fn cache_restore_semantics(
        &mut self,
        _node: Self::Node,
        _input: &ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, bool, Self::Scalar, M> {
        Ok(true)
    }
    fn cache_clear(&mut self, node: Self::Node) -> LayoutResultOf<Self::Node, (), Self::Scalar, M>;
}

pub(crate) struct ChildCacheRequest<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    context: CacheKeyContext,
}

impl<Node: Copy + Eq, S: LayoutScalar> ChildCacheRequest<Node, S> {
    pub(crate) fn new(node: Node, input: ComputeInputOf<S>, context: CacheKeyContext) -> Self {
        Self {
            node,
            input,
            context,
        }
    }

    pub(crate) fn restore_hit<Tree, M>(&self, tree: &mut Tree) -> CacheLookupResult<Node, S, M>
    where
        Tree: CacheAccess<M, Node = Node, Scalar = S>,
    {
        let output = tree.cache_get(self.node, &self.input, self.context)?;
        if output.is_some() && tree.cache_restore_semantics(self.node, &self.input)? {
            return Ok(output);
        }
        Ok(None)
    }

    pub(crate) fn store<Tree, M>(
        self,
        tree: &mut Tree,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: CacheAccess<M, Node = Node, Scalar = S>,
    {
        tree.cache_store(self.node, &self.input, self.context, output)
    }
}
