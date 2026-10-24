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

type SessionResult<Tree, T> = LayoutResultOf<
    <Tree as Traverse>::Node,
    T,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

type CanonicalFrames<Tree> = Vec<
    crate::text_layout::state::CanonicalFrame<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
>;

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
    text: Option<crate::text_layout::state::TextComputation<'a, Tree::Node, Tree::Scalar>>,
    unrounded_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    final_entries: Vec<LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    unrounded_inline_fragment_groups: Vec<StagedInlineFragmentGroup<Tree::Node, Tree::Scalar>>,
    final_inline_fragment_groups: Vec<StagedInlineFragmentGroup<Tree::Node, Tree::Scalar>>,
    cache_store_entries: Vec<LayoutCacheStoreEntryOf<Tree::Node, Tree::Scalar>>,
    cache_clear_entries: Vec<LayoutCacheClearEntry<Tree::Node>>,
    invalidated_nodes: Vec<Tree::Node>,
}

#[derive(Clone)]
struct StagedInlineFragmentGroup<Node, S: LayoutScalar> {
    node: Node,
    fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
}

/// One actual child request executed off-side against the same immutable tree
/// and canonical Text invocation. This is not a rounded/completed child batch.
pub(crate) struct FloatChildPacket<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) output: ComputeOutputOf<S>,
    frames: std::sync::Arc<[crate::text_layout::state::CanonicalFrame<Node, S>]>,
    entries: Vec<LayoutOutputEntryOf<Node, S>>,
    fragments: Vec<StagedInlineFragmentGroup<Node, S>>,
    stores: Vec<LayoutCacheStoreEntryOf<Node, S>>,
    clears: Vec<LayoutCacheClearEntry<Node>>,
    pending: Vec<crate::text_layout::state::PendingParagraph<Node>>,
    atomic_origins: Vec<(Node, crate::Point<f64>)>,
    absolute_baselines: Vec<(Node, f64)>,
    scope_fragments: Vec<crate::text_layout::state::ScopeFragmentGroup<Node, S>>,
}

pub(crate) type FloatProbeResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    std::sync::Arc<FloatChildPacket<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>>,
    <Tree as Traverse>::Scalar,
    M,
>;

// The Grid owner's bounded-stage tests use the real staging owner and real
// host adapter; only the disposable owner's typed observation can escape.
#[cfg(test)]
pub(crate) fn observe_grid_probe<'input, Tree: LayoutTree>(
    tree: &'input Tree,
    node: Tree::Node,
    probe: crate::grid::GridOverflowProbe<'input, '_, Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<
    Tree::Node,
    crate::grid::GridOverflowObservation<Tree::Scalar>,
    Tree::Scalar,
    Tree::MeasureError,
> {
    let mut owner = ComputeSession::new(tree, Vec::new());
    let result = owner.observe_grid_overflow(node, probe);
    assert!(owner.unrounded_entries.is_empty());
    assert!(owner.final_entries.is_empty());
    assert!(owner.unrounded_inline_fragment_groups.is_empty());
    assert!(owner.final_inline_fragment_groups.is_empty());
    assert!(
        owner
            .text
            .as_ref()
            .is_none_or(|state| state.pending.is_empty())
    );
    assert!(owner.cache_store_entries.is_empty());
    assert!(owner.cache_clear_entries.is_empty());
    result
}

// Seed actual retained receipts by executing previously observed ordinary leaf
// requests through the same production owner, without constructing cache output.
#[cfg(test)]
type GridLeafSeedResult<Node, S, M> = LayoutResultOf<Node, CompletedLayoutBatchOf<Node, S>, S, M>;

#[cfg(test)]
pub(crate) fn seed_grid_leaf_requests<Tree: LayoutTree>(
    tree: &Tree,
    requests: &[(Tree::Node, ComputeInputOf<Tree::Scalar>)],
) -> GridLeafSeedResult<Tree::Node, Tree::Scalar, Tree::MeasureError> {
    let mut owner = ComputeSession::new(tree, Vec::new());
    for &(node, input) in requests {
        assert!(tree.has_leaf_measurement(node));
        owner.compute_child(node, input)?;
    }
    Ok(owner.complete())
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
            text: None,
            unrounded_entries: Vec::new(),
            final_entries: Vec::new(),
            unrounded_inline_fragment_groups: Vec::new(),
            final_inline_fragment_groups: Vec::new(),
            cache_store_entries: Vec::new(),
            cache_clear_entries,
            invalidated_nodes,
        }
    }

    pub(super) fn with_text(
        capture: &'a crate::TextLayoutInput<'_, Tree>,
        invalidated_nodes: Vec<Tree::Node>,
    ) -> Self {
        let mut session = Self::new(capture.tree, invalidated_nodes);
        session.text = Some(crate::text_layout::state::TextComputation::new(
            &capture.paragraphs,
        ));
        session
    }

    pub(super) fn canonical_frames(
        &self,
        root: Tree::Node,
    ) -> SessionResult<Tree, CanonicalFrames<Tree>> {
        let mut pending = vec![(root, crate::Point::<f64>::ZERO)];
        let mut frames = Vec::new();
        while let Some((node, parent_origin)) = pending.pop() {
            let output = self
                .final_entries
                .iter()
                .find(|entry| entry.node() == node)
                .map(LayoutOutputEntryOf::output)
                .ok_or_else(|| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::TextPlacement,
                        LayoutErrorKindOf::InternalInvariant(
                            LayoutInternalInvariant::MissingStagedUnroundedOutput,
                        ),
                    )
                })?;
            let origin = crate::Point::new(
                parent_origin.x + output.location.x.to_f64(),
                parent_origin.y + output.location.y.to_f64(),
            );
            for (base, delta, value) in [
                (parent_origin.x, output.location.x.to_f64(), origin.x),
                (parent_origin.y, output.location.y.to_f64(), origin.y),
            ] {
                let reason = if !value.is_finite() {
                    Some(crate::LayoutTextNumericError::NonFinite)
                } else if delta != 0.0 && value == base {
                    Some(crate::LayoutTextNumericError::NonZeroLost)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::TextPlacement,
                        LayoutErrorKindOf::TextNumeric {
                            value: if value.is_finite() { delta } else { value },
                            reason,
                        },
                    ));
                }
            }
            frames.push(crate::text_layout::state::CanonicalFrame {
                node,
                origin,
                output,
            });
            pending.extend(
                self.tree
                    .children(node)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .map(|child| (child, origin)),
            );
        }
        Ok(frames)
    }

    pub(super) fn text_pass_basis(
        &self,
        frames: Vec<crate::text_layout::state::CanonicalFrame<Tree::Node, Tree::Scalar>>,
    ) -> crate::text_layout::state::PassBasis<Tree::Node, Tree::Scalar> {
        self.text
            .as_ref()
            .expect("Text compute entry initialized its state")
            .basis(frames)
    }

    pub(super) fn text_basis(
        &self,
    ) -> &[crate::text_layout::state::CanonicalFrame<Tree::Node, Tree::Scalar>] {
        &self
            .text
            .as_ref()
            .expect("Text compute entry initialized its state")
            .frames
    }

    pub(super) fn begin_text_pass(
        &mut self,
        frames: Vec<crate::text_layout::state::CanonicalFrame<Tree::Node, Tree::Scalar>>,
    ) {
        let text = self
            .text
            .as_mut()
            .expect("Text compute entry initialized its state");
        text.frames = frames.into();
        text.reusable = text
            .pending
            .iter()
            .flat_map(|paragraph| {
                paragraph
                    .lines
                    .iter()
                    .zip(&paragraph.amounts)
                    .map(
                        |(placement, amounts)| crate::text_layout::fit::ReusableLine {
                            finalized: placement.line.clone(),
                            amounts: amounts.clone(),
                        },
                    )
            })
            .collect::<Vec<_>>()
            .into();
        text.pending.clear();
        text.scope_fragments.clear();
        text.atomic_origins.clear();
        text.absolute_baselines.clear();
        self.unrounded_entries.clear();
        self.final_entries.clear();
        self.unrounded_inline_fragment_groups.clear();
        self.final_inline_fragment_groups.clear();
        self.cache_store_entries.clear();
    }

    pub(super) fn complete_with_text(
        mut self,
        root: Tree::Node,
    ) -> SessionResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
        let placed = self
            .text
            .as_mut()
            .expect("Text compute entry initialized its state")
            .finish()?;
        Ok(self.complete_for_root(root).with_placed_text(placed))
    }

    fn complete(self) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let mut unrounded_inline_fragments: Vec<_> = self
            .final_inline_fragment_groups
            .iter()
            .flat_map(|final_group| {
                self.unrounded_inline_fragment_groups
                    .iter()
                    .find(|group| group.node == final_group.node)
                    .and_then(|group| group.fragments.as_deref())
                    .unwrap_or(&[])
                    .iter()
                    .cloned()
                    .map(|fragment| {
                        InlineFragmentOutputEntryOf::new(
                            crate::InlineFragmentOwner::Node(final_group.node),
                            fragment,
                        )
                    })
            })
            .collect();
        let mut final_inline_fragments: Vec<_> = self
            .final_inline_fragment_groups
            .iter()
            .flat_map(|group| {
                group
                    .fragments
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .cloned()
                    .map(|fragment| {
                        InlineFragmentOutputEntryOf::new(
                            crate::InlineFragmentOwner::Node(group.node),
                            fragment,
                        )
                    })
            })
            .collect();
        // Text has already placed these fragments on the canonical line frame.
        // A scope owns no synthetic node to round or receive duplicate geometry.
        if let Some(text) = &self.text {
            for group in &text.scope_fragments {
                unrounded_inline_fragments.extend(group.fragments.iter().cloned());
                final_inline_fragments.extend(group.fragments.iter().cloned());
            }
        }
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

    fn text_computation(
        &mut self,
    ) -> Option<&mut crate::text_layout::state::TextComputation<'input, Self::Node, Self::Scalar>>
    {
        self.text.as_mut()
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
        set_inline_fragment_group(&mut self.unrounded_inline_fragment_groups, node, fragments);
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, ComputeOutputOf<Self::Scalar>, Self::Scalar, Tree::MeasureError>
    {
        if self.tree.layout_input(node).as_inline_text().is_some()
            || self.tree.layout_input(node).as_text_control().is_some()
        {
            if self.text.is_none() {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::TextCapture),
                ));
            }
            let known = input.known();
            let size = Size::new(
                known.width.unwrap_or(Self::Scalar::ZERO),
                known.height.unwrap_or(Self::Scalar::ZERO),
            );
            return Ok(ComputeOutputOf::from_sizes(size, size));
        }

        let style = self.tree.node_input(node);
        if style.block_generation == crate::BlockGeneration::Anonymous {
            if style.display != crate::Display::Block
                || style.position == crate::Position::Absolute
                || !style.float.is_none()
                || matches!(
                    input.parent_formatting_context(),
                    crate::ParentFormattingContext::Flex | crate::ParentFormattingContext::Grid
                )
            {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::InvalidInput(
                        crate::LayoutInvalidInputOf::AnonymousBlockRole,
                    ),
                ));
            }
            let context = input.block_indent_context().ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::MissingContext(
                        crate::LayoutMissingContext::AnonymousGeneratingBlock,
                    ),
                )
            })?;
            if context.inherited != style.text_indent {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::InvalidInput(
                        crate::LayoutInvalidInputOf::AnonymousIndentMismatch,
                    ),
                ));
            }
        }
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
        if let Some(text) = &self.text {
            let mut state = crate::text_layout::state::TextComputation::new(text.paragraphs);
            state.frames = text.frames.clone();
            observation.text = Some(state);
        }
        probe.observe(&mut observation, node)
    }

    fn probe_float_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> LayoutResultOf<
        Self::Node,
        std::sync::Arc<FloatChildPacket<Self::Node, Self::Scalar>>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        // The request is the block owner's complete child request. Reuse the
        // validated topology and the same dependency/phase-bearing input; do
        // not enter a fresh public root computation or reset a Grid probe.
        let mut scope = Vec::new();
        let mut visit = vec![node];
        while let Some(subject) = visit.pop() {
            if !scope.contains(&subject) {
                scope.push(subject);
                visit.extend(self.tree.children(subject));
            }
        }
        let dirty = self
            .invalidated_nodes
            .iter()
            .copied()
            .filter(|node| scope.contains(node))
            .collect();
        let mut probe = Self::new(self.tree, dirty);
        let frames = if let Some(text) = &self.text {
            let mut state = crate::text_layout::state::TextComputation::new(text.paragraphs);
            state.frames = std::sync::Arc::clone(&text.frames);
            state.reusable = std::sync::Arc::clone(&text.reusable);
            let frames = std::sync::Arc::clone(&state.frames);
            probe.text = Some(state);
            frames
        } else {
            std::sync::Arc::from([])
        };
        let output = probe.compute_child(node, input)?;
        let (pending, atomic_origins, absolute_baselines, scope_fragments) =
            probe.text.map_or_else(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |text| {
                    (
                        text.pending,
                        text.atomic_origins,
                        text.absolute_baselines,
                        text.scope_fragments,
                    )
                },
            );
        Ok(std::sync::Arc::new(FloatChildPacket {
            node,
            input,
            output,
            frames,
            entries: probe.unrounded_entries,
            fragments: probe.unrounded_inline_fragment_groups,
            stores: probe.cache_store_entries,
            clears: probe.cache_clear_entries,
            pending,
            atomic_origins,
            absolute_baselines,
            scope_fragments,
        }))
    }

    fn accept_float_child(
        &mut self,
        packet: &FloatChildPacket<Self::Node, Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, Tree::MeasureError> {
        if let Some(text) = &self.text
            && !std::sync::Arc::ptr_eq(&text.frames, &packet.frames)
        {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(packet.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::TextFeedbackCycle,
            ));
        }
        for entry in &packet.entries {
            self.set_unrounded(entry.node(), entry.output());
        }
        for group in &packet.fragments {
            self.set_unrounded_inline_fragment_state(group.node, group.fragments.clone());
        }
        for entry in &packet.stores {
            self.cache_store(entry.node(), entry.input(), entry.context(), entry.output());
        }
        for entry in &packet.clears {
            self.cache_clear(entry.node());
        }
        if let Some(text) = &mut self.text {
            for group in &packet.scope_fragments {
                text.record_scope_fragments(
                    group.container,
                    group.nodes.clone(),
                    group.fragments.clone(),
                );
            }
            for paragraph in &packet.pending {
                text.record(paragraph.clone());
            }
            text.record_atomic_origins(packet.atomic_origins.clone());
            for &(node, baseline) in &packet.absolute_baselines {
                text.record_absolute_baseline(node, baseline);
            }
        }
        Ok(())
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
    fn is_prepared_text(&self, node: Self::Node) -> bool {
        self.text.as_ref().is_some_and(|state| {
            state.paragraphs.iter().any(|paragraph| {
                paragraph.origins.iter().any(|row| row.node == node)
                    || paragraph.controls.iter().any(|row| row.node == node)
            })
        })
    }

    fn canonical_text_atomic_origin(&self, node: Self::Node) -> Option<crate::Point<f64>> {
        self.text
            .as_ref()?
            .atomic_origins
            .iter()
            .find_map(|(subject, origin)| (*subject == node).then_some(*origin))
    }

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
    use crate::{DefaultScalar, LayoutErrorSite, NodeInput, Point, ScrollRectOf, round_layout};

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
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::MissingStagedUnroundedOutput,
            ),
        );
    }

    fn assert_prepared_fragment_staging<S: LayoutScalar>() {
        use crate::test_support::text::{PreparedParagraph, ahem_style};
        let paragraph =
            PreparedParagraph::plain(1, surgeist_text::Source::new("X"), ahem_style(10.0, 10.0));
        let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: crate::Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .input(1, paragraph.text_input(1));
        let capture =
            crate::TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
        for fragments in [
            Vec::new(),
            vec![InlineFragmentOutputOf::new(
                crate::test_support::text_fragment::fragment_for(&paragraph.prepared),
                Some(
                    ScrollRectOf::try_new(
                        Point::new(S::from_f64(0.25), S::from_f64(0.5)),
                        Size::new(S::from_f64(4.5), S::from_f64(2.25)),
                    )
                    .unwrap(),
                ),
                Point::new(S::from_f64(0.25), S::from_f64(2.0)),
                0,
            )],
        ] {
            let mut session = ComputeSession::with_text(&capture, Vec::new());
            let mut output = NodeOutputOf::new();
            output.location = Point::new(S::from_f64(0.25), S::from_f64(0.5));
            Compute::set_unrounded(&mut session, 1, output);
            Compute::set_unrounded_inline_fragment_state(&mut session, 1, Some(fragments.clone()));
            round_layout(&mut session, 1).unwrap();
            assert_eq!(session.final_entries[0].output(), output);
            let batch = session.complete();
            assert_eq!(
                batch
                    .unrounded_inline_fragments()
                    .iter()
                    .map(|row| row.fragment().clone())
                    .collect::<Vec<_>>(),
                fragments
            );
            assert_eq!(
                batch.final_inline_fragments(),
                batch.unrounded_inline_fragments()
            );
        }
    }

    #[test]
    fn prepared_fragments_keep_common_anchor_geometry_without_independent_rounding() {
        assert_prepared_fragment_staging::<f32>();
        assert_prepared_fragment_staging::<f64>();
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
        text: crate::test_support::text::PreparedParagraph,
        cache_reads: std::cell::RefCell<Vec<(u32, ComputeInputOf<S>, bool)>>,
        measurement_reads: std::cell::RefCell<Vec<LeafMeasureInputOf<S>>>,
        fail_at_width: std::cell::Cell<Option<S>>,
    }

    impl<S: LayoutScalar> GridGutterInlineTree<S> {
        fn new(display: Display) -> Self {
            let scalar = S::from_f64;
            let text = crate::test_support::text::PreparedParagraph::plain(
                4,
                surgeist_text::Source::new("AAAA AAAA"),
                crate::test_support::text::ahem_style(5.0, 10.0),
            );
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
                .input(4, text.text_input(4))
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
                text,
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
                    .filter(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(4))
                    .map(|entry| entry.fragment().clone())
                    .collect(),
            ))
        }
        fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
            (self.retained, self.fragments) = prepared;
        }
    }

    fn assert_isolated_observation<S: LayoutScalar>() {
        let fragments = |batch: &CompletedLayoutBatchOf<u32, S>| {
            batch
                .final_inline_fragments()
                .iter()
                .map(|entry| {
                    (
                        entry.owner().clone(),
                        entry.fragment().rect(),
                        entry.fragment().baseline(),
                        entry
                            .fragment()
                            .finalized_fragment()
                            .view()
                            .aggregate_range(),
                    )
                })
                .collect::<Vec<_>>()
        };
        for display in [Display::Grid, Display::GridLanes] {
            for seed_width in [80.0, 50.0] {
                let mut tree = GridGutterInlineTree::<S>::new(display);
                let capture =
                    crate::TextLayoutInput::try_new(&tree, 2, vec![tree.text.binding(3)]).unwrap();
                let reference =
                    compute_layout_with_text(&capture, 2, GridGutterInlineTree::<S>::request(80.0))
                        .unwrap();
                // Ahem at 5px has nine one-em advances. The actual 75-unit content
                // band right-aligns this 45-unit line at 30, independent of cache.
                // Its 4-unit ascender plus 2.5 units of half-leading puts the
                // natural baseline at 6.5. The common canonical baseline is 7,
                // so the retained line-local geometry starts at physical 0.5.
                assert_eq!(
                    fragments(&reference)[0].1.unwrap().origin(),
                    Point::new(S::from_f64(30.0), S::from_f64(0.5))
                );
                assert_eq!(fragments(&reference)[0].2.y, S::from_f64(7.0));
                let seed = compute_layout_with_text(
                    &capture,
                    2,
                    GridGutterInlineTree::<S>::request(seed_width),
                )
                .unwrap();
                drop(capture);
                seed.apply_to(&mut tree).unwrap();
                tree.cache_reads.borrow_mut().clear();
                let capture =
                    crate::TextLayoutInput::try_new(&tree, 2, vec![tree.text.binding(3)]).unwrap();
                let mut session = ComputeSession::with_text(&capture, capture.invalidated.clone());
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
                assert!(session.unrounded_entries.is_empty());
                assert!(session.unrounded_inline_fragment_groups.is_empty());
                assert!(session.text.as_ref().unwrap().pending.is_empty());
                assert!(
                    session
                        .cache_store_entries
                        .iter()
                        .all(|entry| !entry.input().run_mode().is_perform_layout())
                );
                drop(session);
                let batch =
                    compute_layout_with_text(&capture, 2, GridGutterInlineTree::<S>::request(80.0))
                        .unwrap();
                assert_eq!(batch.final_entries(), reference.final_entries());
                assert_eq!(fragments(&batch), fragments(&reference));
                assert!(
                    tree.cache_reads
                        .borrow()
                        .iter()
                        .all(|(node, _, _)| *node != 4)
                );
                assert!(
                    batch
                        .cache_store_entries()
                        .iter()
                        .all(|entry| entry.node() != 4)
                );
                drop(capture);
                let before = (tree.retained.clone(), tree.fragments.clone());
                tree.measurement_reads.borrow_mut().clear();
                tree.fail_at_width.set(Some(S::from_f64(45.0)));
                let capture =
                    crate::TextLayoutInput::try_new(&tree, 2, vec![tree.text.binding(3)]).unwrap();
                let mut session = ComputeSession::with_text(&capture, capture.invalidated.clone());
                let error = crate::grid::compute_grid(
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
                .unwrap_err();
                assert_eq!(error.site(), LayoutErrorSiteOf::Node(6));
                assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
                crate::test_support::assert_error_kind(
                    error.kind(),
                    &LayoutErrorKindOf::Measurement(GridGutterInlineMeasureError::Injected),
                );
                assert_eq!(tree.measurement_reads.borrow().len(), 1);
                assert_eq!(
                    tree.measurement_reads.borrow()[0]
                        .available_content_size()
                        .width
                        .definite_value()
                        .map(NonNegativeFiniteOf::get),
                    Some(S::from_f64(45.0))
                );
                assert!(session.unrounded_entries.is_empty());
                assert!(session.unrounded_inline_fragment_groups.is_empty());
                assert!(session.text.as_ref().unwrap().pending.is_empty());
                assert!(
                    session
                        .cache_store_entries
                        .iter()
                        .all(|entry| !entry.input().run_mode().is_perform_layout())
                );
                assert_eq!((tree.retained.clone(), tree.fragments.clone()), before);
                drop(session);
                tree.fail_at_width.set(None);
                let retry = compute_layout_with_text_invalidated(
                    &capture,
                    2,
                    GridGutterInlineTree::<S>::request(80.0),
                    &[6],
                )
                .unwrap();
                assert_eq!(retry.final_entries(), reference.final_entries());
                assert_eq!(fragments(&retry), fragments(&reference));
            }
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

    fn assert_rigid_translation_reuses_finalization<S: LayoutScalar>() {
        use crate::test_support::text::{PreparedParagraph, ahem_style};
        let text =
            PreparedParagraph::plain(2, surgeist_text::Source::new("X"), ahem_style(10.0, 10.0));
        let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [2])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: crate::Display::Flex,
                    size: Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(100.0)),
                        crate::PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    justify_content: Some(crate::AlignContent::Center),
                    align_items: Some(crate::AlignItems::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: crate::Display::Block,
                    size: Size::new(
                        crate::PreferredSizeOf::px(S::from_f64(50.0)),
                        crate::PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .input(2, text.text_input(2));
        let capture = crate::TextLayoutInput::try_new(&tree, 0, vec![text.binding(1)]).unwrap();
        let mut session = ComputeSession::with_text(&capture, capture.invalidated.clone());
        let available = Size::splat(AvailableOf::MAX_CONTENT);
        super::super::root::compute_root(&mut session, 0, available).unwrap();
        super::super::rounding::round_layout(&mut session, 0).unwrap();
        let initial = session.text.as_ref().unwrap().pending[0].lines[0].clone();
        let frames = session.canonical_frames(0).unwrap();
        assert_eq!(
            frames
                .iter()
                .find(|frame| frame.node == 1)
                .unwrap()
                .origin
                .x,
            25.0
        );
        session.begin_text_pass(frames);
        super::super::root::compute_root(&mut session, 0, available).unwrap();
        super::super::rounding::round_layout(&mut session, 0).unwrap();
        let canonical = &session.text.as_ref().unwrap().pending[0].lines[0];
        assert_eq!(initial.line, canonical.line);
        assert_eq!(initial.anchor.inline_origin, 0.0);
        assert_eq!(canonical.anchor.inline_origin, 25.0);
        assert_eq!(canonical.anchor.baseline, 8.0);
    }

    #[test]
    fn actual_flex_placement_translation_reuses_exact_finalized_text_line() {
        assert_rigid_translation_reuses_finalization::<f32>();
        assert_rigid_translation_reuses_finalization::<f64>();
    }
}
