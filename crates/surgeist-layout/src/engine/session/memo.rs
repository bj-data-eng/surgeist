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
    eligible: Vec<bool>,
    completions: Mutex<Completions<Node, S>>,
    samples: Mutex<Vec<Vec<Sample<S>>>>,
}

/// Completed effects can be shared with disposable trials of the same call.
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
    fn new(eligible: Vec<bool>) -> Self {
        let completions = (0..eligible.len()).map(|_| Vec::new()).collect();
        let samples = (0..eligible.len()).map(|_| Vec::new()).collect();
        Self {
            table: Arc::new(Table {
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

    pub(super) fn fork(&self) -> Self {
        Self {
            table: self.table.clone(),
            effects: Vec::new(),
            active: Vec::new(),
            failed: None,
        }
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
                Write::Effect(Effect::Store(_)) => false,
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
        let mut pending = Vec::new();
        let mut effects = Vec::new();
        pending.try_reserve(1)?;
        pending.push(table.effects[index].iter().rev());
        // A later replay of the same completed write sequence overrides every
        // write in its earlier replay. Walk backward, visiting each sequence
        // once, then restore the surviving sequence's forward order.
        while let Some(next) = pending.last_mut() {
            match next.next() {
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
        Ok(effects)
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
        // and unrounded geometry. Fragmentation also owns its cache bypass.
        if self.bound.is_some()
            || self.fragmentation.is_some()
            || self.text.is_some()
            || self.groups.is_some()
            || self.wrapping.context.is_some()
        {
            return;
        }
        let Some(graph) = self.flow_graph else { return };
        // The public invalidation closure includes every affected ancestor.
        // Trial staging narrows it to its own subtree, preserving the same
        // immutable closure on each node's reachable descendants.
        let mut eligible = vec![false; graph.nodes.len()];
        for &index in &graph.finish {
            let node = graph.nodes[index];
            let style = self.tree.node_input(node);
            // Hidden placement reads an earlier source index and clears cache
            // custody. Shape measurements have an additional provider, while
            // non-box nodes can publish inline state.
            eligible[index] = matches!(
                self.tree.layout_input(node),
                crate::LayoutInputViewOf::Box(_)
            ) && style.display != crate::Display::None
                && style.float_exclusion != crate::FloatExclusion::Shape
                && graph.children[index].iter().all(|&child| eligible[child]);
        }
        self.memo = Some(Memo::new(eligible));
    }

    pub(crate) fn release_request_memo(&mut self) {
        self.memo = None;
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
        // Inherited float frames and framed entry observations contribute
        // mutable, purpose-scoped state beyond the ordinary request key.
        // The root is executed once and can publish a full document's worth
        // of geometry. Its descendants still admit their own exact requests.
        (memo.table.eligible[index]
            && self.entry_sources.is_none()
            && !self.block_floats.blocks_cache(node, &input)
            && input.run_mode() != crate::RunMode::PerformRootLayout)
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
