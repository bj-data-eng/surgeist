use super::contracts::{CacheAccess, Compute, Round, UnroundedInlineFragmentState};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf,
};
use crate::scroll::SettledAutoScrollbarState;
use crate::{
    CompletedLayoutBatchOf, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputEntryOf,
    InlineFragmentOutputOf, LayoutCacheClearEntry, LayoutCacheStoreEntryOf, LayoutInputViewOf,
    LayoutOutputEntryOf, LayoutScalar, LayoutTree, NodeInputOf, NodeOutputOf, RunMode, Size,
    Traverse,
};

#[cfg(test)]
std::thread_local! {
    static HIDDEN_COMPUTE_SESSION_REQUESTS: std::cell::RefCell<Vec<(
        SettledAutoScrollbarState,
        SettledAutoScrollbarState,
    )>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(crate) fn trace_hidden_compute_session_requests<T>(
    operation: impl FnOnce() -> T,
) -> (
    T,
    Vec<(SettledAutoScrollbarState, SettledAutoScrollbarState)>,
) {
    HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| requests.borrow_mut().clear());
    let result = operation();
    let requests = HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| requests.take());
    (result, requests)
}

pub(super) struct ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    tree: &'a Tree,
    unrounded_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    final_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    unrounded_inline_fragment_groups: Vec<StagedInlineFragmentGroup<Tree::Node, Tree::Scalar>>,
    final_inline_fragment_groups: Vec<StagedInlineFragmentGroup<Tree::Node, Tree::Scalar>>,
    warm_inline_text_nodes: Vec<Tree::Node>,
    cache_store_entries: Vec<LayoutCacheStoreEntryOf<Tree::Node, Tree::Scalar>>,
    cache_clear_entries: Vec<LayoutCacheClearEntry<Tree::Node>>,
    invalidated_nodes: Vec<Tree::Node>,
}

struct StagedInlineFragmentGroup<Node, S: LayoutScalar> {
    node: Node,
    fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
}

impl<'a, Tree> ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    pub(super) fn new(tree: &'a Tree, invalidated_nodes: Vec<Tree::Node>) -> Self {
        let cache_clear_entries = invalidated_nodes
            .iter()
            .copied()
            .map(LayoutCacheClearEntry::new)
            .collect();
        Self {
            tree,
            unrounded_entries: Vec::new(),
            final_entries: Vec::new(),
            unrounded_inline_fragment_groups: Vec::new(),
            final_inline_fragment_groups: Vec::new(),
            warm_inline_text_nodes: Vec::new(),
            cache_store_entries: Vec::new(),
            cache_clear_entries,
            invalidated_nodes,
        }
    }

    fn complete(self) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let unrounded_inline_fragments = self
            .final_inline_fragment_groups
            .iter()
            .flat_map(|final_group| {
                self.unrounded_inline_fragment_groups
                    .iter()
                    .find(|group| group.node == final_group.node)
                    .and_then(|group| group.fragments.as_deref())
                    .unwrap_or(&[])
                    .iter()
                    .copied()
                    .map(|fragment| InlineFragmentOutputEntryOf::new(final_group.node, fragment))
            })
            .collect();
        let final_inline_fragments = self
            .final_inline_fragment_groups
            .iter()
            .flat_map(|group| {
                group
                    .fragments
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .copied()
                    .map(|fragment| InlineFragmentOutputEntryOf::new(group.node, fragment))
            })
            .collect();
        CompletedLayoutBatchOf::from_entries(
            self.unrounded_entries,
            self.final_entries,
            unrounded_inline_fragments,
            final_inline_fragments,
            self.cache_store_entries,
            self.cache_clear_entries,
            self.invalidated_nodes,
        )
    }

    pub(super) fn complete_for_root(
        mut self,
        root: Tree::Node,
    ) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        let mut ordered_unrounded = Vec::with_capacity(self.unrounded_entries.len());
        let mut ordered_final = Vec::with_capacity(self.final_entries.len());
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if let Some(entry) = take_layout_entry(&mut self.unrounded_entries, node) {
                ordered_unrounded.push(entry);
            }
            if let Some(entry) = take_layout_entry(&mut self.final_entries, node) {
                ordered_final.push(entry);
            }
            let children = self.tree.children(node).collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        }
        assert!(
            self.unrounded_entries.is_empty() && self.final_entries.is_empty(),
            "completed output nodes remain reachable from the validated root"
        );
        self.unrounded_entries = ordered_unrounded;
        self.final_entries = ordered_final;
        self.complete()
    }

    fn staged_source_index(&self, node: Tree::Node) -> crate::SourceIndex {
        self.unrounded_entries
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .map(|entry| entry.output().source_index)
            .unwrap_or(crate::SourceIndex::ZERO)
    }

    fn subtree_uses_shape_provider(&self, root: Tree::Node) -> bool {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if self.tree.node_input(node).float_exclusion == crate::FloatExclusion::Shape {
                return true;
            }
            pending.extend(self.tree.children(node));
        }
        false
    }

    fn restore_committed_subtree(&mut self, root: Tree::Node) -> bool {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        let mut restored = Vec::new();
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            let Some(output) = self.tree.unrounded_layout(node) else {
                return false;
            };
            restored.push((node, output));
            let children = self.tree.children(node).collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        }
        for (node, output) in restored {
            self.set_unrounded(node, output);
        }
        true
    }

    fn compute_child_uncached(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        let style = self.tree.node_input(node);
        if self.tree.has_leaf_measurement(node) {
            return crate::measurement::compute_tree_leaf(self.tree, node, input, style);
        }

        match style.display.inner_display() {
            crate::Display::Block => crate::block::compute_block(self, node, input),
            crate::Display::Flex => crate::flex::compute_flex(self, node, input),
            crate::Display::Grid | crate::Display::GridLanes => {
                crate::grid::compute_grid(self, node, input)
            }
            crate::Display::None => super::compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            ),
            crate::Display::InlineBlock
            | crate::Display::InlineGrid
            | crate::Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }
}

impl<Tree> Traverse for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'b>
        = Tree::Children<'b>
    where
        Self: 'b;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.tree.children(node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<'input, Tree> Compute<'input, Tree::MeasureError> for ComputeSession<'input, Tree>
where
    Tree: LayoutTree,
{
    fn node_input(&self, node: Self::Node) -> &'input NodeInputOf<Self::Scalar> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> LayoutInputViewOf<'input, Self::Scalar> {
        self.tree.layout_input(node)
    }

    fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>) {
        if let Some(index) = self
            .unrounded_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.unrounded_entries.remove(index);
        }
        self.unrounded_entries
            .push(LayoutOutputEntryOf::new(node, layout));
    }

    fn set_unrounded_inline_fragment_state(
        &mut self,
        node: Self::Node,
        fragments: Option<Vec<InlineFragmentOutputOf<Self::Scalar>>>,
    ) {
        if fragments.is_some() && self.warm_inline_text_nodes.contains(&node) {
            return;
        }
        set_inline_fragment_group(&mut self.unrounded_inline_fragment_groups, node, fragments);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        if input.run_mode().is_perform_layout()
            && matches!(
                self.tree.layout_input(node),
                LayoutInputViewOf::InlineText(_)
            )
        {
            let context = <Self as CacheAccess<Tree::MeasureError>>::cache_context(self);
            if self.invalidated_nodes.is_empty()
                && let Some(output) = <Self as CacheAccess<Tree::MeasureError>>::cache_get(
                    self, node, &input, context,
                )
            {
                if !self.warm_inline_text_nodes.contains(&node) {
                    self.warm_inline_text_nodes.push(node);
                }
                return Ok(output);
            }

            let known = input.known();
            let size = Size::new(
                known.width.unwrap_or(Self::Scalar::ZERO),
                known.height.unwrap_or(Self::Scalar::ZERO),
            );
            let output = ComputeOutputOf::from_sizes(size, size);
            <Self as CacheAccess<Tree::MeasureError>>::cache_store(
                self, node, &input, context, output,
            );
            return Ok(output);
        }

        let style = self.tree.node_input(node);
        if input.run_mode() == RunMode::PerformHiddenLayout || style.display == crate::Display::None
        {
            #[cfg(test)]
            HIDDEN_COMPUTE_SESSION_REQUESTS.with(|requests| {
                requests.borrow_mut().push((
                    input.settled_auto_scrollbars(),
                    input.containing_auto_scrollbar_pass(),
                ));
            });
            return super::compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            );
        }

        if input.run_mode().is_perform_layout() && self.child_count(node) != 0 {
            let input = input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL);
            if self.subtree_uses_shape_provider(node) {
                let context = <Self as CacheAccess<Tree::MeasureError>>::cache_context(self);
                if let Some(output) = <Self as CacheAccess<Tree::MeasureError>>::cache_get(
                    self, node, &input, context,
                ) && self.restore_committed_subtree(node)
                {
                    return Ok(output);
                }
                let output = self.compute_child_uncached(node, input)?;
                <Self as CacheAccess<Tree::MeasureError>>::cache_store(
                    self, node, &input, context, output,
                );
                return Ok(output);
            }
            return self.compute_child_uncached(node, input);
        }

        super::contracts::compute_cached(self, node, input, |session, node, input| {
            session.compute_child_uncached(
                node,
                input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL),
            )
        })
    }

    fn observe_grid_overflow(
        &mut self,
        node: Self::Node,
        probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
    ) -> LayoutResultOf<
        Self::Node,
        crate::grid::GridOverflowObservation<Self::Scalar>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        let mut observation = Self::new(self.tree, self.invalidated_nodes.clone());
        probe.observe(&mut observation, node)
    }

    fn float_exclusion_interval(
        &self,
        node: Self::Node,
        query: crate::FloatExclusionQueryOf<Self::Scalar>,
    ) -> Option<Result<Option<crate::FloatExclusionIntervalOf<Self::Scalar>>, Tree::MeasureError>>
    {
        self.tree.float_exclusion_interval(node, query)
    }
}

impl<Tree> Round<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<Self::Node, NodeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        self.unrounded_entries
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .map(LayoutOutputEntryOf::output)
            .ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::RoundingFinalization,
                    LayoutErrorKindOf::InternalInvariant(
                        LayoutInternalInvariant::MissingStagedUnroundedOutput,
                    ),
                )
            })
    }

    fn set_final(&mut self, node: Self::Node, layout: NodeOutputOf<Self::Scalar>) {
        if let Some(index) = self
            .final_entries
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.final_entries.remove(index);
        }
        self.final_entries
            .push(LayoutOutputEntryOf::new(node, layout));
    }

    fn unrounded_inline_fragment_state(
        &self,
        node: Self::Node,
    ) -> UnroundedInlineFragmentState<'_, Self::Scalar> {
        if let Some(group) = self
            .unrounded_inline_fragment_groups
            .iter()
            .find(|group| group.node == node)
        {
            return match &group.fragments {
                Some(fragments) => UnroundedInlineFragmentState::Present(fragments),
                None => UnroundedInlineFragmentState::Absent,
            };
        }

        if matches!(
            self.tree.layout_input(node),
            LayoutInputViewOf::InlineText(_)
        ) {
            return self.tree.unrounded_inline_fragments(node).map_or(
                UnroundedInlineFragmentState::Missing,
                UnroundedInlineFragmentState::Present,
            );
        }

        UnroundedInlineFragmentState::Absent
    }

    fn set_final_inline_fragments(
        &mut self,
        node: Self::Node,
        unrounded: Vec<InlineFragmentOutputOf<Self::Scalar>>,
        final_fragments: Vec<InlineFragmentOutputOf<Self::Scalar>>,
    ) {
        set_inline_fragment_group(
            &mut self.unrounded_inline_fragment_groups,
            node,
            Some(unrounded),
        );
        set_inline_fragment_group(
            &mut self.final_inline_fragment_groups,
            node,
            Some(final_fragments),
        );
    }
}

fn set_inline_fragment_group<Node, S>(
    groups: &mut Vec<StagedInlineFragmentGroup<Node, S>>,
    node: Node,
    fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
) where
    Node: Copy + Eq,
    S: LayoutScalar,
{
    if let Some(group) = groups.iter_mut().find(|group| group.node == node) {
        group.fragments = fragments;
    } else {
        groups.push(StagedInlineFragmentGroup { node, fragments });
    }
}

fn take_layout_entry<Node, S>(
    entries: &mut Vec<LayoutOutputEntryOf<Node, S>>,
    node: Node,
) -> Option<LayoutOutputEntryOf<Node, S>>
where
    Node: Copy + Eq,
    S: LayoutScalar,
{
    entries
        .iter()
        .position(|entry| entry.node() == node)
        .map(|index| entries.swap_remove(index))
}

impl<Tree> CacheAccess<Tree::MeasureError> for ComputeSession<'_, Tree>
where
    Tree: LayoutTree,
{
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;

    fn cache_context(&self) -> crate::CacheKeyContext {
        self.tree.cache_context()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: crate::CacheKeyContext,
    ) -> Option<ComputeOutputOf<Self::Scalar>> {
        if self.invalidated_nodes.contains(&node) {
            return None;
        }
        self.tree.cache_get(node, input, context)
    }

    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: crate::CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    ) {
        if let Some(index) = self.cache_store_entries.iter().position(|entry| {
            entry.node() == node && entry.input() == input && entry.context() == context
        }) {
            self.cache_store_entries.remove(index);
        }
        self.cache_store_entries
            .push(LayoutCacheStoreEntryOf::new(node, *input, context, output));
    }

    fn cache_clear(&mut self, node: Self::Node) {
        if self
            .cache_clear_entries
            .iter()
            .any(|entry| entry.node() == node)
        {
            return;
        }
        self.cache_clear_entries
            .push(LayoutCacheClearEntry::new(node));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayoutInputOf;
    use crate::{
        BidiLevel, DefaultScalar, InlineBreakOpportunityOf, InlineMetricsOf, InlineSegmentId,
        InlineTextInputOf, InlineWhitespaceEdge, LayoutErrorSite, NodeInput, Point, ScrollRectOf,
        ShapedInlineSegmentOf, round_layout,
    };

    #[test]
    fn borrowed_input_projection_remains_live_across_mutable_child_compute() {
        fn assert_lane<S: LayoutScalar>() {
            let source = crate::test_support::layout_tree::PublicLayoutTreeOf::new()
                .children(1, [])
                .style(
                    1,
                    NodeInputOf::<S> {
                        display: crate::Display::Block,
                        size: Size::new(
                            crate::PreferredSizeOf::px(S::from_f64(20.0)),
                            crate::PreferredSizeOf::px(S::from_f64(10.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                );
            let mut session = ComputeSession::new(&source, Vec::new());
            let view = session.layout_input(1);
            let borrowed = view.as_box().unwrap();
            let projection = crate::node_projection::CommonBoxProjection::from_node(borrowed);
            let output = session
                .compute_child(
                    1,
                    ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        crate::SizingMode::InherentSize,
                        crate::RequestedAxis::Both,
                        Size::NONE,
                        Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(50.0))),
                        crate::ContainingLayoutContext::new(
                            crate::FlowAxes::new(
                                crate::WritingMode::HorizontalTb,
                                crate::Direction::Ltr,
                            ),
                            crate::ParentFormattingContext::BlockFlow,
                        ),
                        Size::splat(crate::AvailableOf::MAX_CONTENT),
                    ),
                )
                .unwrap();
            assert_eq!(output.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));
            assert!(core::ptr::eq(borrowed, source.node_input(1)));
            assert!(core::ptr::eq(projection.size, &source.node_input(1).size));
            assert_eq!(*projection.size, borrowed.size);
        }
        assert_lane::<f32>();
        assert_lane::<f64>();
    }

    struct BoundedDagTree {
        input: NodeInput,
        children: Vec<Vec<u32>>,
        adjacency_queries: std::cell::Cell<usize>,
        remaining_adjacency_budget: std::cell::Cell<usize>,
    }

    impl Traverse for BoundedDagTree {
        type Node = u32;
        type Scalar = DefaultScalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            let remaining = self.remaining_adjacency_budget.get();
            assert!(
                remaining > 0,
                "completion traversal exceeded one adjacency query per unique node"
            );
            self.remaining_adjacency_budget.set(remaining - 1);
            self.adjacency_queries.set(self.adjacency_queries.get() + 1);
            self.children[node as usize].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[node as usize].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[node as usize][index]
        }
    }

    impl LayoutTree for BoundedDagTree {
        type MeasureError = ();

        fn node_input(&self, _node: Self::Node) -> &NodeInput {
            &self.input
        }

        fn layout_input(&self, _node: Self::Node) -> crate::LayoutInputViewOf<'_, DefaultScalar> {
            crate::LayoutInputViewOf::Box(&self.input)
        }
    }

    #[test]
    fn fri06_c04_float_lifecycle_completion_shared_repeated_dag_is_source_ordered_and_bounded() {
        let tree = BoundedDagTree {
            input: NodeInput::default(),
            children: vec![
                vec![1, 1, 2, 2],
                vec![2, 2, 3, 3],
                vec![4, 4],
                vec![4, 4, 5, 5],
                vec![5, 5],
                Vec::new(),
            ],
            adjacency_queries: std::cell::Cell::new(0),
            remaining_adjacency_budget: std::cell::Cell::new(6),
        };
        let mut session = ComputeSession::new(&tree, Vec::new());
        for node in [2, 4, 5, 3, 1, 0] {
            Compute::set_unrounded(&mut session, node, NodeOutputOf::new());
            Round::set_final(&mut session, node, NodeOutputOf::new());
        }

        let batch = session.complete_for_root(0);
        let expected_source_order = [0, 1, 2, 4, 5, 3];

        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .map(LayoutOutputEntryOf::node)
                .collect::<Vec<_>>(),
            expected_source_order
        );
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .map(LayoutOutputEntryOf::node)
                .collect::<Vec<_>>(),
            expected_source_order
        );
        assert_eq!(tree.adjacency_queries.get(), 6);
        assert_eq!(tree.remaining_adjacency_budget.get(), 0);
    }

    #[test]
    fn compute_session_rejects_missing_staged_unrounded_output() {
        let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::new()
            .style(0, NodeInput::default());
        let session = ComputeSession::new(&tree, Vec::new());

        let error = Round::unrounded(&session, 0).unwrap_err();

        assert_eq!(error.site(), LayoutErrorSite::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        assert_eq!(
            error.kind(),
            &LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::MissingStagedUnroundedOutput,
            )
        );
    }

    fn fri06_c01_fragment<S: LayoutScalar>(segment: u64) -> InlineFragmentOutputOf<S> {
        InlineFragmentOutputOf::new(
            InlineSegmentId::new(segment),
            ScrollRectOf::try_new(
                Point::new(S::from_f64(0.25), S::from_f64(0.5)),
                Size::new(S::from_f64(4.5), S::from_f64(2.25)),
            )
            .unwrap(),
            Point::new(S::from_f64(0.25), S::from_f64(2.0)),
            0,
            segment as usize,
            None,
        )
    }

    fn fri06_c01_text_input<S: LayoutScalar>() -> InlineTextInputOf<S> {
        InlineTextInputOf::try_new(vec![
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(1),
                S::from_f64(4.5),
                InlineMetricsOf::from_ascent_descent(S::from_f64(1.5), S::from_f64(0.75)).unwrap(),
                BidiLevel::try_new(0).unwrap(),
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::prohibited(),
            )
            .unwrap(),
        ])
        .unwrap()
    }

    struct FragmentTree<S: LayoutScalar> {
        input: NodeInputOf<S>,
        layout_input: LayoutInputOf<S>,
        committed: Option<Vec<InlineFragmentOutputOf<S>>>,
        readback_calls: std::cell::Cell<usize>,
    }

    impl<S: LayoutScalar> Traverse for FragmentTree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a>
            = std::iter::Empty<u32>
        where
            Self: 'a;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            std::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            unreachable!("fragment test tree has no children")
        }
    }

    impl<S: LayoutScalar> LayoutTree for FragmentTree<S> {
        type MeasureError = ();

        fn node_input(&self, _node: Self::Node) -> &NodeInputOf<S> {
            &self.input
        }

        fn layout_input(&self, _node: Self::Node) -> crate::LayoutInputViewOf<'_, S> {
            self.layout_input.as_view()
        }

        fn unrounded_inline_fragments(
            &self,
            _node: Self::Node,
        ) -> Option<&[InlineFragmentOutputOf<S>]> {
            self.readback_calls.set(self.readback_calls.get() + 1);
            self.committed.as_deref()
        }
    }

    fn assert_fri06_c02_fragment_rounding_and_readback<S: LayoutScalar>() {
        let staged_tree = FragmentTree::<S> {
            input: NodeInputOf::non_box(),
            layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
            committed: None,
            readback_calls: std::cell::Cell::new(0),
        };
        let mut staged = ComputeSession::new(&staged_tree, Vec::new());
        Compute::set_unrounded(&mut staged, 0, NodeOutputOf::new());
        Compute::set_unrounded_inline_fragment_state(
            &mut staged,
            0,
            Some(vec![fri06_c01_fragment(1)]),
        );
        round_layout(&mut staged, 0).unwrap();
        assert_eq!(staged_tree.readback_calls.get(), 0);
        let staged_batch = staged.complete();

        assert_eq!(staged_batch.unrounded_inline_fragments().len(), 1);
        assert_eq!(staged_batch.final_inline_fragments().len(), 1);
        assert_eq!(
            staged_batch.unrounded_inline_fragments()[0]
                .fragment()
                .rect()
                .origin(),
            Point::new(S::from_f64(0.25), S::from_f64(0.5))
        );
        assert_eq!(
            staged_batch.final_inline_fragments()[0]
                .fragment()
                .rect()
                .origin(),
            Point::new(S::ZERO, S::from_f64(1.0))
        );

        let warm_tree = FragmentTree::<S> {
            input: NodeInputOf::non_box(),
            layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
            committed: Some(vec![fri06_c01_fragment(1)]),
            readback_calls: std::cell::Cell::new(0),
        };
        let mut warm = ComputeSession::new(&warm_tree, Vec::new());
        Compute::set_unrounded(&mut warm, 0, NodeOutputOf::new());
        round_layout(&mut warm, 0).unwrap();
        assert_eq!(warm_tree.readback_calls.get(), 1);
        let warm_batch = warm.complete();
        assert_eq!(
            warm_batch.unrounded_inline_fragments(),
            staged_batch.unrounded_inline_fragments()
        );
        assert_eq!(
            warm_batch.final_inline_fragments(),
            staged_batch.final_inline_fragments()
        );

        let empty_tree = FragmentTree::<S> {
            input: NodeInputOf::non_box(),
            layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
            committed: Some(Vec::new()),
            readback_calls: std::cell::Cell::new(0),
        };
        let mut empty = ComputeSession::new(&empty_tree, Vec::new());
        Compute::set_unrounded(&mut empty, 0, NodeOutputOf::new());
        round_layout(&mut empty, 0).unwrap();
        assert_eq!(empty_tree.readback_calls.get(), 1);
        let empty_batch = empty.complete();
        assert!(empty_batch.unrounded_inline_fragments().is_empty());
        assert!(empty_batch.final_inline_fragments().is_empty());
    }

    #[test]
    fn fri06_c02_cache_staged_and_committed_nonempty_and_empty_fragments_round_once_both_scalars() {
        assert_fri06_c02_fragment_rounding_and_readback::<f32>();
        assert_fri06_c02_fragment_rounding_and_readback::<f64>();
    }

    #[test]
    fn fri06_c02_cache_missing_warm_fragment_state_fails_without_publication_both_scalars() {
        fn assert_lane<S: LayoutScalar>() {
            let tree = FragmentTree::<S> {
                input: NodeInputOf::non_box(),
                layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
                committed: None,
                readback_calls: std::cell::Cell::new(0),
            };
            let mut session = ComputeSession::new(&tree, Vec::new());
            Compute::set_unrounded(&mut session, 0, NodeOutputOf::new());

            let error = round_layout(&mut session, 0).unwrap_err();
            assert_eq!(tree.readback_calls.get(), 1);

            assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
            assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
            assert_eq!(
                error.kind(),
                &LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingCachedInlineFragmentState,
                )
            );
            assert!(session.final_entries.is_empty());
            assert!(session.final_inline_fragment_groups.is_empty());
        }

        assert_lane::<f32>();
        assert_lane::<f64>();
    }

    #[test]
    fn fri06_c02_cache_hidden_text_needs_no_committed_fragment_state_both_scalars() {
        fn assert_lane<S: LayoutScalar>() {
            let tree = FragmentTree::<S> {
                input: NodeInputOf::non_box(),
                layout_input: LayoutInputOf::inline_text(fri06_c01_text_input()),
                committed: None,
                readback_calls: std::cell::Cell::new(0),
            };
            let mut session = ComputeSession::new(&tree, Vec::new());
            Compute::set_unrounded(&mut session, 0, NodeOutputOf::new());
            Compute::set_unrounded_inline_fragment_state(&mut session, 0, None);

            round_layout(&mut session, 0).unwrap();
            assert_eq!(tree.readback_calls.get(), 0);

            let batch = session.complete();
            assert!(batch.unrounded_inline_fragments().is_empty());
            assert!(batch.final_inline_fragments().is_empty());
        }

        assert_lane::<f32>();
        assert_lane::<f64>();
    }
}

#[cfg(test)]
mod grid_overflow_observation_tests {
    use super::*;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use crate::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct ObservationRetainedState<S: LayoutScalar> {
        unrounded: HashMap<u32, NodeOutputOf<S>>,
        final_outputs: HashMap<u32, NodeOutputOf<S>>,
        caches: HashMap<u32, CacheOf<S>>,
    }

    impl<S: LayoutScalar> ObservationRetainedState<S> {
        fn prepare_grid_test_batch(&self, batch: &CompletedLayoutBatchOf<u32, S>) -> Self {
            let mut prepared = self.clone();
            for node in batch.invalidated_nodes() {
                prepared.unrounded.remove(node);
                prepared.final_outputs.remove(node);
                prepared.caches.remove(node);
            }
            for entry in batch.unrounded_entries() {
                prepared.unrounded.insert(entry.node(), entry.output());
            }
            for entry in batch.final_entries() {
                prepared.final_outputs.insert(entry.node(), entry.output());
            }
            for entry in batch.cache_clear_entries() {
                prepared.caches.remove(&entry.node());
            }
            for entry in batch.cache_store_entries() {
                prepared
                    .caches
                    .entry(entry.node())
                    .or_default()
                    .store_with_context(entry.input(), entry.context(), entry.output());
            }
            prepared
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum GridGutterInlineMeasureError {
        Injected,
    }

    struct GridGutterInlineTree<S: LayoutScalar> {
        tree: PublicLayoutTreeOf<S>,
        retained: ObservationRetainedState<S>,
        fragments: Vec<InlineFragmentOutputOf<S>>,
        cache_reads: std::cell::RefCell<Vec<(u32, ComputeInputOf<S>, bool)>>,
        measurement_reads: std::cell::RefCell<Vec<LeafMeasureInputOf<S>>>,
        fail_at_width: std::cell::Cell<Option<S>>,
    }

    impl<S: LayoutScalar> GridGutterInlineTree<S> {
        fn new(display: Display) -> Self {
            let scalar = S::from_f64;
            let text = InlineTextInputOf::try_new(vec![
                ShapedInlineSegmentOf::try_new(
                    InlineSegmentId::new(1),
                    scalar(25.0),
                    InlineMetricsOf::from_ascent_descent(scalar(8.0), scalar(2.0)).unwrap(),
                    BidiLevel::try_new(0).unwrap(),
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::allowed(),
                )
                .unwrap(),
                ShapedInlineSegmentOf::try_new(
                    InlineSegmentId::new(2),
                    scalar(20.0),
                    InlineMetricsOf::from_ascent_descent(scalar(8.0), scalar(2.0)).unwrap(),
                    BidiLevel::try_new(0).unwrap(),
                    InlineWhitespaceEdge::Preserve,
                    InlineBreakOpportunityOf::prohibited(),
                )
                .unwrap(),
            ])
            .unwrap();
            let tree = PublicLayoutTreeOf::new()
                .children(2, [3, 5])
                .children(3, [4])
                .children(4, [])
                .children(5, [6])
                .children(6, [])
                .style(
                    2,
                    NodeInputOf {
                        display,
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(scalar(20.0))),
                        grid_template_columns: vec![TrackComponentOf::flex(
                            TrackFlexFactorOf::try_new(S::ONE).unwrap(),
                        )],
                        grid_template_rows: vec![
                            TrackComponentOf::px(scalar(30.0)),
                            TrackComponentOf::px(scalar(10.0)),
                        ],
                        overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto)
                            .unwrap(),
                        scrollbar_gutter: ScrollbarGutter::Stable,
                        scrollbar_width: ScrollbarWidthOf::try_new(scalar(5.0)).unwrap(),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    3,
                    NodeInputOf {
                        display: Display::Block,
                        text_align: TextAlign::LegacyRight,
                        ..NodeInputOf::default()
                    },
                )
                .input(4, LayoutInputOf::inline_text(text))
                .style(
                    5,
                    NodeInputOf {
                        display: Display::Block,
                        align_self: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    6,
                    NodeInputOf {
                        display: Display::Block,
                        position: Position::Absolute,
                        inset: Edges::new(
                            LengthAutoOf::AUTO,
                            LengthAutoOf::ZERO,
                            LengthAutoOf::AUTO,
                            LengthAutoOf::ZERO,
                        ),
                        ..NodeInputOf::default()
                    },
                );
            Self {
                tree,
                retained: ObservationRetainedState::default(),
                fragments: Vec::new(),
                cache_reads: std::cell::RefCell::new(Vec::new()),
                measurement_reads: std::cell::RefCell::new(Vec::new()),
                fail_at_width: std::cell::Cell::new(None),
            }
        }

        fn request(width: f64) -> LayoutRootRequestOf<S> {
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::Definite(S::from_f64(width)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap()
        }
    }

    impl<S: LayoutScalar> Traverse for GridGutterInlineTree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a>
            = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
        where
            Self: 'a;
        fn children(&self, node: u32) -> Self::Children<'_> {
            Traverse::children(&self.tree, node)
        }
        fn child_count(&self, node: u32) -> usize {
            self.tree.child_count(node)
        }
        fn child(&self, node: u32, index: usize) -> u32 {
            self.tree.child(node, index)
        }
    }

    impl<S: LayoutScalar> LayoutTree for GridGutterInlineTree<S> {
        type MeasureError = GridGutterInlineMeasureError;
        fn node_input(&self, node: u32) -> &NodeInputOf<S> {
            self.tree.node_input(node)
        }
        fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
            self.tree.layout_input(node)
        }
        fn has_leaf_measurement(&self, node: u32) -> bool {
            node == 6
        }
        fn measure_leaf(
            &self,
            node: u32,
            input: LeafMeasureInputOf<S>,
        ) -> Option<Result<Size<S>, Self::MeasureError>> {
            assert_eq!(node, 6);
            self.measurement_reads.borrow_mut().push(input);
            if self.fail_at_width.get().is_some()
                && self.fail_at_width.get()
                    == input
                        .available_content_size()
                        .width
                        .definite_value()
                        .map(NonNegativeFiniteOf::get)
            {
                Some(Err(GridGutterInlineMeasureError::Injected))
            } else {
                Some(Ok(Size::new(S::from_f64(10.0), S::from_f64(5.0))))
            }
        }
        fn cache_get(
            &self,
            node: u32,
            input: &ComputeInputOf<S>,
            context: CacheKeyContext,
        ) -> Option<ComputeOutputOf<S>> {
            // This host deliberately reuses text entries only. Retained container
            // measurements may be declined, ensuring the observation really runs.
            let output = if matches!(
                self.tree.layout_input(node),
                LayoutInputViewOf::InlineText(_)
            ) {
                self.retained
                    .caches
                    .get(&node)
                    .and_then(|cache| cache.get_with_context(input, context))
            } else {
                None
            };
            self.cache_reads
                .borrow_mut()
                .push((node, *input, output.is_some()));
            output
        }
        fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
            self.retained.unrounded.get(&node).copied()
        }
        fn unrounded_inline_fragments(&self, node: u32) -> Option<&[InlineFragmentOutputOf<S>]> {
            (node == 4).then_some(self.fragments.as_slice())
        }
    }

    impl<S: LayoutScalar> LayoutBatchSink<u32, S> for GridGutterInlineTree<S> {
        type Error = core::convert::Infallible;
        type Prepared = (ObservationRetainedState<S>, Vec<InlineFragmentOutputOf<S>>);
        fn prepare_layout_batch(
            &self,
            batch: &CompletedLayoutBatchOf<u32, S>,
        ) -> Result<Self::Prepared, Self::Error> {
            Ok((
                self.retained.prepare_grid_test_batch(batch),
                batch
                    .unrounded_inline_fragments()
                    .iter()
                    .filter(|entry| entry.node() == 4)
                    .map(|entry| entry.fragment())
                    .collect(),
            ))
        }
        fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
            (self.retained, self.fragments) = prepared;
        }
    }

    fn assert_isolated_observation<S: LayoutScalar>() {
        for display in [Display::Grid, Display::GridLanes] {
            let reference_tree = GridGutterInlineTree::<S>::new(display);
            let reference =
                compute_layout(&reference_tree, 2, GridGutterInlineTree::<S>::request(80.0))
                    .unwrap();
            // Actual root width80 minus the5-unit stable inline-edge gutter gives
            // text75. Shaped advance45 leaves30 units of right-alignment space.
            assert_eq!(
                reference
                    .unrounded_inline_fragments()
                    .iter()
                    .map(|entry| entry.fragment().rect().origin())
                    .collect::<Vec<_>>(),
                vec![
                    Point::new(S::from_f64(30.0), S::ZERO),
                    Point::new(S::from_f64(55.0), S::ZERO)
                ]
            );
            for (seed_width, probe_hit, final_hit) in [(80.0, false, true), (50.0, true, false)] {
                let mut tree = GridGutterInlineTree::<S>::new(display);
                compute_layout(&tree, 2, GridGutterInlineTree::<S>::request(seed_width))
                    .unwrap()
                    .apply_to(&mut tree)
                    .unwrap();
                tree.cache_reads.borrow_mut().clear();
                let mut session = ComputeSession::new(&tree, Vec::new());
                let measured = crate::grid::compute_grid(
                    &mut session,
                    2,
                    ComputeInputOf::for_child(
                        RunMode::ComputeSize,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::new(None, Some(S::from_f64(20.0))),
                        Size::NONE,
                        ContainingLayoutContext::new(
                            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                            ParentFormattingContext::NoParent,
                        ),
                        Size::splat(AvailableOf::MAX_CONTENT),
                    ),
                )
                .unwrap();
                assert_eq!(
                    measured.size,
                    Size::new(S::from_f64(50.0), S::from_f64(20.0))
                );
                // The real Grid intrinsic operation arranged descendants only in its
                // disposable sibling. No observational node/fragment/cache state or
                // warm-text permission is published into the consuming Session.
                assert!(session.unrounded_entries.is_empty());
                assert!(session.unrounded_inline_fragment_groups.is_empty());
                assert!(session.warm_inline_text_nodes.is_empty());
                assert!(
                    session
                        .cache_store_entries
                        .iter()
                        .all(|entry| !entry.input().run_mode().is_perform_layout())
                );
                super::super::root::compute_root(
                    &mut session,
                    2,
                    GridGutterInlineTree::<S>::request(80.0).available(),
                )
                .unwrap();
                super::super::rounding::round_layout(&mut session, 2).unwrap();
                let batch = session.complete_for_root(2);
                let reads = tree.cache_reads.borrow();
                for (width, hit) in [(45.0, probe_hit), (75.0, final_hit)] {
                    assert!(
                        reads.iter().any(|(node, input, actual)| *node == 4
                            && input.run_mode().is_perform_layout()
                            && input.parent().width == Some(S::from_f64(width))
                            && *actual == hit),
                        "actual {display:?} text request width{width} hit={hit}: {reads:?}"
                    );
                }
                assert_eq!(batch.final_entries(), reference.final_entries());
                assert_eq!(
                    batch.unrounded_inline_fragments(),
                    reference.unrounded_inline_fragments()
                );
                assert_eq!(
                    batch.final_inline_fragments(),
                    reference.final_inline_fragments()
                );
                assert!(
                    batch
                        .cache_store_entries()
                        .iter()
                        .all(|entry| entry.node() != 4
                            || !entry.input().run_mode().is_perform_layout()
                            || entry.input().parent().width == Some(S::from_f64(75.0)))
                );
            }
            let mut tree = GridGutterInlineTree::<S>::new(display);
            compute_layout(&tree, 2, GridGutterInlineTree::<S>::request(80.0))
                .unwrap()
                .apply_to(&mut tree)
                .unwrap();
            let before = (tree.retained.clone(), tree.fragments.clone());
            tree.cache_reads.borrow_mut().clear();
            tree.measurement_reads.borrow_mut().clear();
            tree.fail_at_width.set(Some(S::from_f64(45.0)));
            let invalidated =
                super::super::validation::validate_layout_request(&tree, 2, &[6]).unwrap();
            let mut session = ComputeSession::new(&tree, invalidated);
            let result = crate::grid::compute_grid(
                &mut session,
                2,
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::new(None, Some(S::from_f64(20.0))),
                    Size::NONE,
                    ContainingLayoutContext::new(
                        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                        ParentFormattingContext::NoParent,
                    ),
                    Size::splat(AvailableOf::MAX_CONTENT),
                ),
            );
            assert!(
                result.is_err(),
                "isolated host measurement not reached: result={result:?} reads={:?} measurements={:?}",
                tree.cache_reads.borrow(),
                tree.measurement_reads.borrow()
            );
            let error = result.unwrap_err();
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(6));
            assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
            assert_eq!(
                error.kind(),
                &LayoutErrorKindOf::Measurement(GridGutterInlineMeasureError::Injected)
            );
            // The actual out-of-flow leaf is excluded from intrinsic track
            // contributions. Its first provider request is reached in the
            // isolated child layout. Invalidation deliberately bypasses the
            // host's text-cache lookup, so it is not an after-text query oracle.
            assert_eq!(tree.measurement_reads.borrow().len(), 1);
            assert_eq!(
                tree.measurement_reads
                    .borrow()
                    .last()
                    .unwrap()
                    .available_content_size()
                    .width
                    .definite_value()
                    .map(NonNegativeFiniteOf::get),
                Some(S::from_f64(45.0))
            );
            assert!(session.unrounded_entries.is_empty());
            assert!(session.unrounded_inline_fragment_groups.is_empty());
            assert!(session.warm_inline_text_nodes.is_empty());
            assert!(
                session
                    .cache_store_entries
                    .iter()
                    .all(|entry| !entry.input().run_mode().is_perform_layout())
            );
            assert_eq!((tree.retained.clone(), tree.fragments.clone()), before);
            tree.fail_at_width.set(None);
            let retry = compute_layout_invalidated(
                &tree,
                2,
                GridGutterInlineTree::<S>::request(80.0),
                &[6],
            )
            .unwrap();
            assert_eq!(retry.final_entries(), reference.final_entries());
            assert_eq!(
                retry.final_inline_fragments(),
                reference.final_inline_fragments()
            );
        }
    }
    #[test]
    fn grid_gutter_isolated_warm_inline_observation_in_f32() {
        assert_isolated_observation::<f32>();
    }
    #[test]
    fn grid_gutter_isolated_warm_inline_observation_in_f64() {
        assert_isolated_observation::<f64>();
    }
}
