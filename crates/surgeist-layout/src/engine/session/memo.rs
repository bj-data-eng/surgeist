use std::{
    cell::Cell,
    collections::{HashSet, TryReserveError},
    sync::{Arc, Mutex},
};

use super::{CacheAccess, Compute, ComputeSession, SessionResult};
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, LayoutCacheStoreEntryOf, LayoutErrorKindOf,
    LayoutErrorOf, LayoutErrorSiteOf, LayoutOperation, LayoutOutputEntryOf, LayoutScalar,
    LayoutTree, LeafMeasureInputOf, Size,
};

#[derive(Clone, Copy)]
pub(super) struct RequestKey<S: LayoutScalar> {
    index: usize,
    input: ComputeInputOf<S>,
    context: CacheKeyContext,
}

#[derive(Clone, Copy)]
pub(super) enum Effect<Node, S: LayoutScalar> {
    Store(LayoutCacheStoreEntryOf<Node, S>),
    Geometry(LayoutOutputEntryOf<Node, S>),
    Clear { node: Node, index: usize },
    ClearInline(Node),
    Body(crate::engine::block_scope::Body<Node, S>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Profile {
    Ordinary,
    Scoped,
}

#[derive(Clone, Copy)]
enum Write<Node, S: LayoutScalar> {
    Effect(Effect<Node, S>),
    Completed(usize),
}

struct Completion<S: LayoutScalar> {
    key: RequestKey<S>,
    output: ComputeOutputOf<S>,
    effects: usize,
}

struct Completions<Node, S: LayoutScalar> {
    nodes: Vec<Vec<Completion<S>>>,
    effects: Vec<Vec<Write<Node, S>>>,
}

struct Sample<S: LayoutScalar> {
    input: LeafMeasureInputOf<S>,
    output: Size<S>,
}

struct Table<Node, S: LayoutScalar> {
    profile: Profile,
    eligible: Vec<bool>,
    completions: Mutex<Completions<Node, S>>,
    samples: Mutex<Vec<Vec<Sample<S>>>>,
}

/// Ordinary completed effects can be shared with disposable trials of the same call.
/// Scoped effects retain the original session's margin and body custody.
/// In-flight writes belong to the individual session that executes them.
pub(super) struct Memo<Node, S: LayoutScalar> {
    table: Arc<Table<Node, S>>,
    effects: Vec<Write<Node, S>>,
    active: Vec<Active>,
    failed: Option<Node>,
}

struct Active {
    start: usize,
    replayable: Cell<bool>,
}

pub(super) struct Capture<S: LayoutScalar> {
    key: RequestKey<S>,
}

impl<Node: Copy + Eq, S: LayoutScalar> Memo<Node, S> {
    fn new(eligible: Vec<bool>, profile: Profile) -> Self {
        let completions = (0..eligible.len()).map(|_| Vec::new()).collect();
        let samples = (0..eligible.len()).map(|_| Vec::new()).collect();
        Self {
            table: Arc::new(Table {
                profile,
                eligible,
                completions: Mutex::new(Completions {
                    nodes: completions,
                    effects: Vec::new(),
                }),
                samples: Mutex::new(samples),
            }),
            effects: Vec::new(),
            active: Vec::new(),
            failed: None,
        }
    }

    pub(super) fn fork(&self) -> Option<Self> {
        if self.is_scoped() {
            return None;
        }
        Some(Self {
            table: self.table.clone(),
            effects: Vec::new(),
            active: Vec::new(),
            failed: None,
        })
    }

    pub(super) fn is_scoped(&self) -> bool {
        self.table.profile == Profile::Scoped
    }

    pub(super) fn shares_samples(&self) -> bool {
        self.table.profile == Profile::Ordinary
    }

    fn begin(&mut self, key: RequestKey<S>) -> Result<Capture<S>, TryReserveError> {
        self.active.try_reserve(1)?;
        self.active.push(Active {
            start: self.effects.len(),
            replayable: Cell::new(true),
        });
        Ok(Capture { key })
    }

    pub(super) fn record(&mut self, node: Node, effect: Effect<Node, S>) {
        self.write(node, Write::Effect(effect));
    }

    fn write(&mut self, node: Node, write: Write<Node, S>) {
        if self.active.is_empty() || self.failed.is_some() {
            return;
        }
        if self.effects.try_reserve(1).is_err() {
            self.failed = Some(node);
            return;
        }
        self.effects.push(write);
    }

    /// Relative placement reads the geometry staged by the preceding placement
    /// pass. A capture that reads outside its own writes cannot be replayed.
    pub(super) fn observe_geometry(&self, node: Node) {
        if self.active.is_empty() {
            return;
        }
        let table = self
            .table
            .completions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for active in self.active.iter().rev() {
            if self.effects[active.start..].iter().any(|write| {
                match write {
                Write::Effect(Effect::Geometry(entry)) => entry.node() == node,
                Write::Completed(index) => table.effects[*index].iter().any(|write| {
                    matches!(write, Write::Effect(Effect::Geometry(entry)) if entry.node() == node)
                }),
                Write::Effect(_) => false,
            }
            }) {
                break;
            }
            active.replayable.set(false);
        }
    }

    pub(super) fn barrier(&self) {
        for active in &self.active {
            active.replayable.set(false);
        }
    }

    fn find(&self, key: RequestKey<S>) -> Option<(ComputeOutputOf<S>, usize)> {
        let table = self
            .table
            .completions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = table.nodes[key.index].iter().find(|entry| {
            entry.key.input == key.input
                && exact_optional_size(entry.key.input.known(), key.input.known())
                && exact_optional_size(entry.key.input.parent(), key.input.parent())
                && entry.key.context == key.context
        })?;
        Some((entry.output, entry.effects))
    }

    fn replay(&self, index: usize) -> Result<Vec<Effect<Node, S>>, TryReserveError> {
        let table = self
            .table
            .completions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut visited = HashSet::new();
        let mut cleared = HashSet::new();
        let mut first_writes = Vec::new();
        let mut forward = Vec::new();
        forward.try_reserve(1)?;
        forward.push(table.effects[index].iter());
        // Cache clears are deduplicated in first-encounter order.
        while let Some(next) = forward.last_mut() {
            match next.next() {
                Some(Write::Effect(Effect::Clear { node, index })) => {
                    cleared.try_reserve(1)?;
                    if cleared.insert(*index) {
                        first_writes.try_reserve(1)?;
                        first_writes.push(Effect::Clear {
                            node: *node,
                            index: *index,
                        });
                    }
                }
                Some(Write::Completed(child)) => {
                    visited.try_reserve(1)?;
                    if visited.insert(*child) {
                        forward.try_reserve(1)?;
                        forward.push(table.effects[*child].iter());
                    }
                }
                Some(Write::Effect(_)) => {}
                None => {
                    forward.pop();
                }
            }
        }
        visited.clear();
        let mut pending = Vec::new();
        let mut effects = Vec::new();
        pending.try_reserve(1)?;
        pending.push(table.effects[index].iter().rev());
        // A later replay of the same completed write sequence overrides every
        // write in its earlier replay. Walk backward, visiting each sequence
        // once, then restore the surviving sequence's forward order.
        while let Some(next) = pending.last_mut() {
            match next.next() {
                Some(Write::Effect(Effect::Clear { .. })) => {}
                Some(Write::Effect(effect)) => {
                    effects.try_reserve(1)?;
                    effects.push(*effect);
                }
                Some(Write::Completed(child)) => {
                    visited.try_reserve(1)?;
                    if visited.insert(*child) {
                        pending.try_reserve(1)?;
                        pending.push(table.effects[*child].iter().rev());
                    }
                }
                None => {
                    pending.pop();
                }
            }
        }
        effects.reverse();
        first_writes.try_reserve(effects.len())?;
        first_writes.extend(effects);
        Ok(first_writes)
    }

    fn complete(
        &mut self,
        node: Node,
        capture: Capture<S>,
        output: ComputeOutputOf<S>,
    ) -> Result<(), TryReserveError> {
        let active = self
            .active
            .pop()
            .expect("started capture owns an effect range");
        if active.replayable.get() {
            let suffix = &self.effects[active.start..];
            let mut writes = Vec::new();
            writes.try_reserve(suffix.len())?;
            writes.extend_from_slice(suffix);
            let index = {
                let mut table = self
                    .table
                    .completions
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                table.effects.try_reserve(1)?;
                reserve_row(&mut table.nodes[capture.key.index])?;
                let index = table.effects.len();
                table.effects.push(writes);
                table.nodes[capture.key.index].push(Completion {
                    key: capture.key,
                    output,
                    effects: index,
                });
                index
            };
            self.effects.truncate(active.start);
            self.write(node, Write::Completed(index));
        }
        if self.active.is_empty() {
            self.effects.clear();
        }
        Ok(())
    }
}

impl<'a, Tree: LayoutTree> ComputeSession<'a, Tree> {
    pub(crate) fn enable_request_memo(&mut self) {
        // These profiles own receipts or context beyond ordinary cache stores
        // and unrounded geometry. A continuous publication target contributes
        // no child state. Its host cache bypass is enforced by the cache owner.
        if self.text.is_some()
            || self.groups.is_some()
            || self.wrapping.context.is_some()
            || self.entry_sources.is_some()
        {
            return;
        }
        let Some(graph) = self.flow_graph else { return };
        let profile = if self.fragmentation.is_some() {
            if self.block_scopes().is_none()
                || self.block_bodies.is_none()
                || self.bound.is_none()
                || graph.parents.iter().any(|parents| parents.len() > 1)
            {
                return;
            }
            Profile::Scoped
        } else {
            Profile::Ordinary
        };
        // The public invalidation closure includes every affected ancestor.
        // Trial staging narrows it to its own subtree, preserving the same
        // immutable closure on each node's reachable descendants.
        let mut eligible = vec![false; graph.nodes.len()];
        for &index in &graph.finish {
            let node = graph.nodes[index];
            let style = match self.tree.layout_input(node) {
                crate::LayoutInputViewOf::Box(style) => style,
                // With the context exclusions above, numeric leaf breaks and
                // Contribution boundaries retain only staged source/edge geometry.
                // BoxEdge requires a wrapping context and remains excluded, as do
                // Text formatting receipts and scope-owned control effects.
                crate::LayoutInputViewOf::LineBreak(_)
                | crate::LayoutInputViewOf::InlineBoundary(_) => {
                    eligible[index] =
                        profile == Profile::Ordinary && graph.children[index].is_empty();
                    continue;
                }
                crate::LayoutInputViewOf::InlineText(_)
                | crate::LayoutInputViewOf::TextControl(_) => continue,
            };
            // Hidden leaves have only ordinary staged geometry and clear effects.
            // Other hidden subtrees, shape providers and inline participants
            // retain their own placement paths.
            eligible[index] = (style.display != crate::Display::None
                || graph.children[index].is_empty())
                && style.float_exclusion != crate::FloatExclusion::Shape
                && graph.children[index].iter().all(|&child| eligible[child]);
            if profile == Profile::Scoped {
                let scopes = self
                    .block_scopes()
                    .expect("scoped memo has a fixed dispatch");
                let terminal = graph.children[index].is_empty();
                let terminal_parent = graph.parents[index].first().is_some_and(|&parent| {
                    matches!(
                        self.tree
                            .node_input(graph.nodes[parent])
                            .display
                            .inner_display(),
                        crate::Display::Flex | crate::Display::Grid | crate::Display::GridLanes
                    )
                });
                eligible[index] &= !scopes.dispatch.owners.contains(&node)
                    && !scopes.dispatch.paths.contains(&node)
                    && style.float.is_none()
                    && self.block_floats.needs_work(index) != Some(true)
                    && match style.display.inner_display() {
                        crate::Display::Flex | crate::Display::Grid | crate::Display::GridLanes => {
                            true
                        }
                        crate::Display::Block => terminal && terminal_parent,
                        crate::Display::None => terminal,
                        _ => false,
                    };
            }
        }
        self.memo = Some(Memo::new(eligible, profile));
        if self.bound.is_none()
            && (0..graph.nodes.len()).any(|index| self.block_floats.needs_work(index) == Some(true))
        {
            self.cache.indexed_stores = Some(super::cache_stores::CacheStores::new());
        }
    }

    pub(crate) fn release_request_memo(&mut self) {
        self.memo = None;
    }

    pub(super) fn invalidate_scoped_request_memo(&mut self) {
        if self.memo.as_ref().is_some_and(Memo::is_scoped) {
            self.memo = None;
        }
    }

    pub(super) fn request_key(
        &self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        context: CacheKeyContext,
    ) -> Option<RequestKey<Tree::Scalar>> {
        let memo = self.memo.as_ref()?;
        let graph = self.flow_graph?;
        let index = graph.index(node)?;
        // A terminal Block under Flex/Grid records an empty margin flow. Its
        // retained row has invariant empty groups across exact request replay.
        // Nonempty Blocks and scope entries have different margin custody.
        if memo.is_scoped()
            && self.tree.node_input(node).display.inner_display() == crate::Display::Block
            && !matches!(
                input.parent_formatting_context(),
                crate::ParentFormattingContext::Flex | crate::ParentFormattingContext::Grid
            )
        {
            memo.barrier();
            return None;
        }
        // Inherited float frames and framed entry observations contribute
        // mutable, purpose-scoped state beyond the ordinary request key.
        // The validated graph root is laid out once and can publish the whole
        // tree's geometry. Descendants can reuse exact requests that inherit
        // the root layout mode.
        (memo.table.eligible[index]
            && self.entry_sources.is_none()
            && !self.block_floats.blocks_cache(node, &input)
            && (index != 0 || input.run_mode() != crate::RunMode::PerformRootLayout))
            .then_some(RequestKey {
                index,
                input,
                context,
            })
    }

    pub(super) fn memo_sample(
        &self,
        node: Tree::Node,
        input: LeafMeasureInputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, Size<Tree::Scalar>> {
        let memo = self
            .memo
            .as_ref()
            .expect("sample belongs to a memo-enabled call");
        let Some(index) = self.flow_graph.and_then(|graph| graph.index(node)) else {
            return crate::measurement::sample_tree_leaf(self.tree, node, input);
        };
        {
            let samples = memo
                .table
                .samples
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(sample) = samples[index].iter().find(|sample| {
                sample.input == input
                    && exact_optional_size(
                        sample.input.known_content_size(),
                        input.known_content_size(),
                    )
            }) {
                return Ok(sample.output);
            }
        }
        // Several full compute inputs can lower to the same exact provider
        // input. Only the native sample is shared across those distinct calls.
        let output = crate::measurement::sample_tree_leaf(self.tree, node, input)?;
        let output = crate::measurement::validate_native_sample(node, output)?;
        let mut samples = memo
            .table
            .samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let row = &mut samples[index];
        reserve_row(row).map_err(|_| memo_allocation(node))?;
        row.push(Sample { input, output });
        Ok(output)
    }

    pub(super) fn memo_hit(
        &mut self,
        node: Tree::Node,
        key: RequestKey<Tree::Scalar>,
    ) -> SessionResult<Tree, Option<ComputeOutputOf<Tree::Scalar>>> {
        let memo = self.memo.as_ref().expect("admitted request has a table");
        let Some((output, index)) = memo.find(key) else {
            return Ok(None);
        };
        let effects = memo.replay(index).map_err(|_| memo_allocation(node))?;
        // The staging owner still applies every surviving write. The
        // enclosing capture retains one reference to the completed sequence.
        let mut memo = self.memo.take().expect("admitted request has a table");
        let result = self.replay_effects(effects);
        memo.write(node, Write::Completed(index));
        self.memo = Some(memo);
        result.map(|()| Some(output))
    }

    fn replay_effects(
        &mut self,
        effects: Vec<Effect<Tree::Node, Tree::Scalar>>,
    ) -> SessionResult<Tree, ()> {
        for effect in effects {
            match effect {
                Effect::Store(entry) => {
                    self.cache_store(entry.node(), entry.input(), entry.context(), entry.output())?;
                }
                Effect::Geometry(entry) => self.set_unrounded(entry),
                Effect::Clear { node, .. } => self.cache_clear(node)?,
                Effect::ClearInline(node) => self.set_unrounded_inline_fragment_state(node, None),
                Effect::Body(body) => self.record_block_body(body.node, body.input, body.output)?,
            }
        }
        Ok(())
    }

    pub(super) fn begin_memo(
        &mut self,
        node: Tree::Node,
        key: Option<RequestKey<Tree::Scalar>>,
    ) -> SessionResult<Tree, Option<Capture<Tree::Scalar>>> {
        let Some(key) = key else { return Ok(None) };
        self.memo
            .as_mut()
            .expect("admitted request has a table")
            .begin(key)
            .map(Some)
            .map_err(|_| memo_allocation(node))
    }

    pub(super) fn complete_memo(
        &mut self,
        node: Tree::Node,
        capture: Option<Capture<Tree::Scalar>>,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, ()> {
        let Some(capture) = capture else {
            return Ok(());
        };
        let memo = self
            .memo
            .as_mut()
            .expect("started request retains its table");
        if let Some(node) = memo.failed {
            return Err(memo_allocation(node));
        }
        memo.complete(node, capture, output)
            .map_err(|_| memo_allocation(node))
    }
}

// The scalar API exposes the sign of a raw zero. Typed sizing offers
// already normalize zero, so preserve the representation of raw dimensions.
fn exact_optional_size<S: LayoutScalar>(left: Size<Option<S>>, right: Size<Option<S>>) -> bool {
    left.zip_map(right, |left, right| {
        left.map(|value| value.to_f64().to_bits()) == right.map(|value| value.to_f64().to_bits())
    }) == Size::splat(true)
}

fn reserve_row<T>(row: &mut Vec<T>) -> Result<(), TryReserveError> {
    if row.capacity() == 0 {
        row.try_reserve_exact(1)
    } else {
        row.try_reserve(1)
    }
}

fn memo_allocation<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::Allocation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use crate::{
        AvailableSpaceOf, AxisSizingRequestOf, ContainingLayoutContext, Direction, Display,
        FlowAxes, LayoutInputViewOf, NearestScrollportOf, NodeInputOf, OrthogonalFallbackLimitOf,
        OrthogonalFallbackOf, ParentFormattingContext, PreferredSizeOf, SizingOperation,
        SizingRequestOf, Traverse, WrappingInput, WritingMode,
    };

    struct ColdBlockTree<S: LayoutScalar> {
        tree: PublicLayoutTreeOf<S>,
        cache_reads: Cell<usize>,
    }

    impl<S: LayoutScalar> Traverse for ColdBlockTree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a> = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>;

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

    impl<S: LayoutScalar> LayoutTree for ColdBlockTree<S> {
        type MeasureError = core::convert::Infallible;

        fn node_input(&self, node: u32) -> &NodeInputOf<S> {
            self.tree.node_input(node)
        }

        fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
            self.tree.layout_input(node)
        }

        fn cache_get(
            &self,
            _node: u32,
            _input: &ComputeInputOf<S>,
            _context: CacheKeyContext,
        ) -> Option<ComputeOutputOf<S>> {
            self.cache_reads.set(self.cache_reads.get() + 1);
            None
        }
    }

    fn assert_inherited_root_request_replays_staged_work<S: LayoutScalar>() {
        const ROOT: u32 = 9;
        const DESCENDANT: u32 = 0;
        const LEAF: u32 = 7;
        let block = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let tree = ColdBlockTree {
            tree: PublicLayoutTreeOf::new()
                .children(ROOT, [DESCENDANT])
                .children(DESCENDANT, [LEAF])
                .style(ROOT, block.clone())
                .style(DESCENDANT, block.clone())
                .style(
                    LEAF,
                    NodeInputOf {
                        size: Size::new(
                            PreferredSizeOf::AUTO,
                            PreferredSizeOf::px(S::from_f64(7.0)),
                        ),
                        ..block
                    },
                ),
            cache_reads: Cell::new(0),
        };
        let (graph, invalidated) =
            crate::engine::validation::validate_layout_request_with_text_graph(
                &tree,
                ROOT,
                &[],
                &[],
            )
            .unwrap();
        let wrapping = crate::inline_wrapping::WrappingContext::new(
            &tree,
            &graph,
            &WrappingInput::default(),
            &[],
            |_| false,
        )
        .unwrap();
        let mut session = ComputeSession::new(&tree, invalidated);
        session.install_wrapping(Arc::new(wrapping), &graph);
        session.enable_request_memo();

        let input = |width, parent| {
            ComputeInputOf::root_layout(
                Size::new(Some(S::from_f64(width)), None),
                Size::NONE,
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    parent,
                ),
                SizingRequestOf::new(
                    Size::splat(AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::Normal,
                    )),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Absent,
                        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            )
        };

        let root_input = input(100.0, ParentFormattingContext::NoParent);
        session.compute_child(ROOT, root_input).unwrap();
        let root_key = RequestKey {
            index: graph.index(ROOT).unwrap(),
            input: root_input,
            context: CacheKeyContext::new(),
        };
        assert_eq!(root_key.index, 0);
        assert!(session.memo_hit(ROOT, root_key).unwrap().is_none());
        session.outputs.unrounded.clear();
        session.cache.stores.clear();

        let first = input(60.0, ParentFormattingContext::BlockFlow);
        let output = session.compute_child(DESCENDANT, first).unwrap();
        let geometry = session.staged_unrounded(LEAF).unwrap();
        assert_eq!(
            geometry.output().size,
            Size::new(S::from_f64(60.0), S::from_f64(7.0))
        );
        let stores = std::mem::take(&mut session.cache.stores);
        assert!(stores.iter().any(|entry| entry.node() == LEAF));

        session.outputs.unrounded.clear();
        tree.cache_reads.set(0);
        assert_eq!(session.compute_child(DESCENDANT, first).unwrap(), output);
        assert_eq!(
            tree.cache_reads.get(),
            0,
            "repeat must skip the leaf's host cache"
        );
        assert_eq!(session.staged_unrounded(LEAF).unwrap(), geometry);
        assert_eq!(session.cache.stores, stores);

        let changed = input(80.0, ParentFormattingContext::BlockFlow);
        assert_eq!(
            session
                .compute_child(DESCENDANT, changed)
                .unwrap()
                .size
                .width,
            S::from_f64(80.0)
        );
        assert_eq!(
            session.staged_unrounded(LEAF).unwrap().output().size.width,
            S::from_f64(80.0)
        );
        tree.cache_reads.set(0);
        assert_eq!(session.compute_child(DESCENDANT, first).unwrap(), output);
        assert_eq!(tree.cache_reads.get(), 0);
        assert_eq!(session.staged_unrounded(LEAF).unwrap(), geometry);
    }

    #[test]
    fn inherited_root_request_replays_staged_work_for_f32() {
        assert_inherited_root_request_replays_staged_work::<f32>();
    }

    #[test]
    fn inherited_root_request_replays_staged_work_for_f64() {
        assert_inherited_root_request_replays_staged_work::<f64>();
    }

    #[test]
    fn replay_keeps_first_clear_order_across_repeated_child_completions() {
        let memo = Memo::<u32, f32>::new(Vec::new(), Profile::Ordinary);
        {
            let mut table = memo.table.completions.lock().unwrap();
            table.effects.push(vec![
                Write::Effect(Effect::Clear { node: 4, index: 0 }),
                Write::Effect(Effect::ClearInline(4)),
                Write::Effect(Effect::Clear { node: 2, index: 1 }),
            ]);
            table.effects.push(vec![
                Write::Completed(0),
                Write::Effect(Effect::Clear { node: 7, index: 2 }),
                Write::Effect(Effect::ClearInline(7)),
                Write::Completed(0),
                Write::Effect(Effect::Clear { node: 4, index: 0 }),
            ]);
        }

        let replay = memo.replay(1).unwrap();
        assert!(matches!(
            replay.as_slice(),
            [
                Effect::Clear { node: 4, index: 0 },
                Effect::Clear { node: 2, index: 1 },
                Effect::Clear { node: 7, index: 2 },
                Effect::ClearInline(7),
                Effect::ClearInline(4),
            ]
        ));
    }
}

#[cfg(test)]
#[path = "scoped_memo_tests.rs"]
mod scoped_tests;

#[cfg(test)]
#[path = "numeric_control_memo_tests.rs"]
mod numeric_control_tests;
