#[cfg(test)]
mod scope_resource_tests;

use super::contracts::{CacheAccess, Compute, Round, UnroundedInlineFragmentState};
use super::entry_staging::{self, Rows};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf,
};
use crate::scroll::SettledAutoScrollbarState;
use crate::text_layout::scope_alignment::EntryBacking;
use crate::{
    CompletedLayoutBatchOf, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputEntryOf,
    InlineFragmentOutputOf, LayoutCacheClearEntry, LayoutCacheStoreEntryOf, LayoutInputViewOf,
    LayoutOutputEntryOf, LayoutScalar, LayoutTree, NodeInputOf, RunMode, Size, Traverse,
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

type EntrySources<Tree> = std::sync::Arc<
    std::sync::Mutex<
        super::entry_sources::SourceMemo<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    >,
>;
type EntryFunding<Tree> =
    Option<crate::text_layout::scope_alignment::EntryFunding<<Tree as Traverse>::Scalar>>;

type GroupBasis<Tree> =
    Option<crate::group_alignment::Basis<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>>;

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

pub(crate) struct ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    tree: &'a Tree,
    bound: Option<crate::LayoutTargetBasis>,
    block_scopes: Option<super::block_scope::Dispatch<Tree::Node, Tree::Scalar>>,
    scope_receipts: Vec<(Tree::Node, super::block_scope::Adoption)>,
    scope_ready: Vec<(Tree::Node, super::block_scope::Adoption)>,
    block_frames: Vec<super::block_scope::Frame<Tree::Node, Tree::Scalar>>,
    record_block_bodies: bool,
    block_bodies: Vec<super::block_scope::Body<Tree::Node, Tree::Scalar>>,
    fragmentation: Option<FragmentComputation<Tree::Node, Tree::Scalar>>,
    block_floats: crate::block::float_context::State<Tree::Node, Tree::Scalar>,
    entry_sources: Option<EntrySources<Tree>>,
    wrapping: WrappingComputation<Tree::Node, Tree::Scalar>,
    flow_graph: Option<&'a crate::engine::validation::ValidationGraph<Tree::Node>>,
    groups: Option<crate::group_alignment::State<Tree::Node, Tree::Scalar>>,
    text: Option<crate::text_layout::state::TextComputation<'a, Tree::Node, Tree::Scalar>>,
    outputs: OutputStaging<Tree::Node, Tree::Scalar>,
    cache: CacheStaging<Tree::Node, Tree::Scalar>,
    entry_storage: Option<EntryStorage>,
}

struct FragmentComputation<Node, S: LayoutScalar> {
    margins: crate::block::fragment_margins::State<Node, S>,
    allocations: Vec<crate::block_fragmentation::FragmentBoxAllocation<Node, S>>,
    intrinsics: Option<std::sync::Arc<crate::block::intrinsic::FragmentContext<Node, S>>>,
}

struct WrappingComputation<Node, S: LayoutScalar> {
    context: Option<std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    uncached: Vec<Node>,
    native_runs: Vec<crate::inline_wrapping::family::NativePublication<S>>,
    native_completions: Vec<crate::inline_wrapping::family::NativeFormatterCompletion<S>>,
}

impl<Node, S: LayoutScalar> WrappingComputation<Node, S> {
    fn new() -> Self {
        Self {
            context: None,
            uncached: Vec::new(),
            native_runs: Vec::new(),
            native_completions: Vec::new(),
        }
    }

    fn reset_receipts(&mut self) {
        self.native_runs.clear();
        self.native_completions.clear();
    }
}

struct OutputStaging<Node, S: LayoutScalar> {
    unrounded: Vec<LayoutOutputEntryOf<Node, S>>,
    rounded: Vec<LayoutOutputEntryOf<Node, S>>,
    unrounded_fragments: Vec<StagedInlineFragmentGroup<Node, S>>,
    rounded_fragments: Vec<StagedInlineFragmentGroup<Node, S>>,
}

impl<Node, S: LayoutScalar> OutputStaging<Node, S> {
    fn new() -> Self {
        Self {
            unrounded: Vec::new(),
            rounded: Vec::new(),
            unrounded_fragments: Vec::new(),
            rounded_fragments: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.unrounded.clear();
        self.rounded.clear();
        self.unrounded_fragments.clear();
        self.rounded_fragments.clear();
    }
}

struct CacheStaging<Node, S: LayoutScalar> {
    stores: Vec<LayoutCacheStoreEntryOf<Node, S>>,
    clears: Vec<LayoutCacheClearEntry<Node>>,
    invalidated: Vec<Node>,
}

struct EntryStorage {
    _dirty: EntryBacking,
    clears: EntryBacking,
    stores: EntryBacking,
    _fragments: EntryBacking,
    _wrapping: EntryBacking,
}

#[derive(Clone)]
struct StagedInlineFragmentGroup<Node, S: LayoutScalar> {
    node: Node,
    fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatChildTrial {
    Detached,
    InFlow,
}

/// One actual child request executed off-side against the same immutable tree
/// and canonical Text invocation. This is not a rounded/completed child batch.
pub(crate) struct FloatChildPacket<Node, S: LayoutScalar> {
    groups: Option<crate::group_alignment::State<Node, S>>,
    wrapping: Option<std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    formatted_basis: Option<crate::text_layout::formatted::EntryFormattedBasis<Node, S>>,
    boxes: Vec<crate::text_layout::boxes::BoxRunUse<Node, S>>,
    formatted: Option<crate::text_layout::formatted::FormattedState<Node, S>>,
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) output: ComputeOutputOf<S>,
    frames: Option<std::sync::Arc<[crate::text_layout::state::CanonicalFrame<Node, S>]>>,
    associations: Option<std::sync::Arc<crate::text_layout::capture::ScopeAssociations<Node>>>,
    entries: Vec<LayoutOutputEntryOf<Node, S>>,
    fragments: Vec<StagedInlineFragmentGroup<Node, S>>,
    native_inline_runs: Vec<crate::inline_wrapping::family::NativePublication<S>>,
    native_inline_completions: Vec<crate::inline_wrapping::family::NativeFormatterCompletion<S>>,
    stores: Vec<LayoutCacheStoreEntryOf<Node, S>>,
    clears: Vec<LayoutCacheClearEntry<Node>>,
    pending: Vec<crate::text_layout::state::PendingParagraph<Node>>,
    absolute_baselines: Vec<(Node, f64)>,
    scope_fragments: Vec<crate::text_layout::state::ScopeFragmentGroup<Node, S>>,
    _entry_clears: Option<EntryBacking>,
    _entry_stores: Option<EntryBacking>,
    _entry_owner: Option<crate::text_layout::scope_alignment::ScopeLease>,
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
    assert!(owner.outputs.unrounded.is_empty());
    assert!(owner.outputs.rounded.is_empty());
    assert!(owner.outputs.unrounded_fragments.is_empty());
    assert!(owner.outputs.rounded_fragments.is_empty());
    assert!(
        owner
            .text
            .as_ref()
            .is_none_or(|state| state.pending.is_empty())
    );
    assert!(owner.cache.stores.is_empty());
    assert!(owner.cache.clears.is_empty());
    result
}

// Seed actual retained receipts by executing previously observed ordinary leaf
// requests through the same production owner, without constructing cache output.
#[cfg(test)]
type GridLeafSeedResult<Node, S, M> =
    LayoutResultOf<Node, Vec<LayoutCacheStoreEntryOf<Node, S>>, S, M>;

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
    Ok(owner.cache.stores)
}

impl<'a, Tree> ComputeSession<'a, Tree>
where
    Tree: LayoutTree,
{
    pub(crate) fn install_block_scopes(
        &mut self,
        root: Tree::Node,
        dispatch: super::block_scope::Dispatch<Tree::Node, Tree::Scalar>,
    ) -> SessionResult<Tree, ()> {
        if self.bound.as_ref() != Some(&dispatch.target) {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::InvalidInput(
                    crate::LayoutInvalidInputOf::BlockFragmentationRequest,
                ),
            ));
        }
        self.block_scopes = Some(dispatch);
        Ok(())
    }

    pub(crate) fn new(tree: &'a Tree, invalidated_nodes: Vec<Tree::Node>) -> Self {
        let cache_clear_entries = invalidated_nodes
            .iter()
            .copied()
            .map(LayoutCacheClearEntry::new)
            .collect();
        Self::with_invalidations(tree, invalidated_nodes, cache_clear_entries)
    }

    fn with_invalidations(
        tree: &'a Tree,
        invalidated_nodes: Vec<Tree::Node>,
        cache_clear_entries: Vec<LayoutCacheClearEntry<Tree::Node>>,
    ) -> Self {
        Self {
            tree,
            bound: None,
            block_scopes: None,
            scope_receipts: Vec::new(),
            scope_ready: Vec::new(),
            block_frames: Vec::new(),
            record_block_bodies: false,
            block_bodies: Vec::new(),
            fragmentation: None,
            block_floats: crate::block::float_context::State::new(),
            entry_sources: None,
            wrapping: WrappingComputation::new(),
            flow_graph: None,
            groups: None,
            text: None,
            outputs: OutputStaging::new(),
            cache: CacheStaging {
                stores: Vec::new(),
                clears: cache_clear_entries,
                invalidated: invalidated_nodes,
            },
            entry_storage: None,
        }
    }

    fn entry_funding(&self, node: Tree::Node) -> SessionResult<Tree, EntryFunding<Tree>> {
        let Some(text) = &self.text else {
            return Ok(None);
        };
        text.entry_funding().map_err(|error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Text(error),
            )
        })
    }

    /// Disposable custody for an exact native child trial. A framed caller
    /// installs its authenticated resident chain before any child dispatch.
    pub(crate) fn child_staging(
        &self,
        node: Tree::Node,
        trial: FloatChildTrial,
        incoming: Option<crate::block::float_context::Incoming<Tree::Node, Tree::Scalar>>,
    ) -> SessionResult<Tree, Self> {
        if incoming
            .as_ref()
            .is_some_and(|entry| entry.node != node || trial != FloatChildTrial::InFlow)
        {
            return Err(crate::block::float_context::missing(node));
        }
        let error = |error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Text(error),
            )
        };
        let funding = if self.entry_sources.is_some() {
            self.entry_funding(node)?
        } else {
            None
        };
        let funding_ref = funding.as_ref();
        let mut scope = Rows::new();
        let mut visit = Rows::new();
        visit.push(node, funding_ref).map_err(error)?;
        while !visit.values.is_empty() {
            entry_staging::inspect(funding_ref, 1).map_err(error)?;
            let subject = visit.values.pop().expect("nonempty pending source stack");
            if !entry_staging::contains(&scope.values, &subject, funding_ref).map_err(error)? {
                scope.push(subject, funding_ref).map_err(error)?;
                for child in self.tree.children(subject) {
                    entry_staging::inspect(funding_ref, 1).map_err(error)?;
                    visit.push(child, funding_ref).map_err(error)?;
                }
            }
        }
        let mut dirty = Rows::new();
        for &subject in &self.cache.invalidated {
            entry_staging::inspect(funding_ref, 1).map_err(error)?;
            if entry_staging::contains(&scope.values, &subject, funding_ref).map_err(error)? {
                dirty.push(subject, funding_ref).map_err(error)?;
            }
        }
        let mut clears = Rows::new();
        for &subject in &dirty.values {
            entry_staging::inspect(funding_ref, 1).map_err(error)?;
            clears
                .push(LayoutCacheClearEntry::new(subject), funding_ref)
                .map_err(error)?;
        }
        let (dirty, dirty_storage) = dirty.into_parts();
        let (clears, clears_storage) = clears.into_parts();
        let mut probe = Self::with_invalidations(self.tree, dirty, clears);
        probe.entry_sources = self.entry_sources.clone();
        probe.block_floats = self
            .block_floats
            .fork_entry(incoming, funding_ref)
            .map_err(error)?;
        probe.groups = self
            .groups
            .as_ref()
            .map(|groups| {
                entry_staging::inspect(funding_ref, 1)?;
                Ok(groups.fork())
            })
            .transpose()
            .map_err(error)?;
        probe.wrapping.context = self.wrapping.context.clone();
        probe.flow_graph = self.flow_graph;
        probe.bound = self.bound.clone();
        let fragments =
            entry_staging::copy(self.fragment_allocations(), funding_ref).map_err(error)?;
        let (allocations, fragment_storage) = fragments.into_parts();
        if let Some(fragment) = &self.fragmentation {
            let intrinsics = fragment.intrinsics.clone();
            let margins = fragment.margins.fork_entry(funding_ref).map_err(error)?;
            probe.fragmentation = Some(FragmentComputation {
                margins,
                allocations,
                intrinsics,
            });
        }
        let wrapping = entry_staging::copy(&self.wrapping.uncached, funding_ref).map_err(error)?;
        let (wrapping, wrapping_storage) = wrapping.into_parts();
        probe.wrapping.uncached = wrapping;
        if let Some(text) = &self.text {
            let formatted = text
                .formatted
                .as_ref()
                .map(|state| state.fork_entry(funding_ref, trial == FloatChildTrial::InFlow))
                .transpose()
                .map_err(error)?;
            probe.text = Some(text.staging(formatted, funding_ref));
        }
        probe.entry_storage = funding_ref.map(|_| EntryStorage {
            _dirty: dirty_storage,
            clears: clears_storage,
            stores: EntryBacking::default(),
            _fragments: fragment_storage,
            _wrapping: wrapping_storage,
        });
        Ok(probe)
    }

    pub(super) fn bind_fragment_frame(
        &mut self,
        target: crate::LayoutTargetBasis,
        allocations: Vec<
            crate::block_fragmentation::FragmentBoxAllocation<Tree::Node, Tree::Scalar>,
        >,
    ) {
        self.bound = Some(target);
        self.fragmentation = Some(FragmentComputation {
            margins: crate::block::fragment_margins::State::new(),
            allocations,
            intrinsics: None,
        });
    }

    pub(super) fn bind_fragment_frame_with_intrinsics(
        &mut self,
        context: std::sync::Arc<crate::block::intrinsic::FragmentContext<Tree::Node, Tree::Scalar>>,
        allocations: Vec<
            crate::block_fragmentation::FragmentBoxAllocation<Tree::Node, Tree::Scalar>,
        >,
    ) {
        self.bound = Some(context.target.clone());
        self.fragmentation = Some(FragmentComputation {
            margins: crate::block::fragment_margins::State::new(),
            allocations,
            intrinsics: Some(context),
        });
    }

    fn fragment_allocations(
        &self,
    ) -> &[crate::block_fragmentation::FragmentBoxAllocation<Tree::Node, Tree::Scalar>] {
        self.fragmentation
            .as_ref()
            .map_or(&[], |fragment| fragment.allocations.as_slice())
    }

    fn fragment_intrinsics(
        &self,
    ) -> Option<&crate::block::intrinsic::FragmentContext<Tree::Node, Tree::Scalar>> {
        self.fragmentation
            .as_ref()
            .and_then(|fragment| fragment.intrinsics.as_deref())
    }

    pub(super) fn install_wrapping(
        &mut self,
        wrapping: std::sync::Arc<crate::inline_wrapping::WrappingContext<Tree::Node>>,
        graph: &'a crate::engine::validation::ValidationGraph<Tree::Node>,
    ) {
        self.flow_graph = Some(graph);
        self.block_floats.configure(self.tree, graph);
        if let Some(nodes) = crate::group_alignment::dependencies(self.tree, graph) {
            self.groups = Some(crate::group_alignment::State::new());
            self.uncache_flow_dependencies(graph, nodes.into_iter());
        }

        if wrapping.has_effects() {
            self.wrapping.context = Some(wrapping.clone());
            // Host geometry has no slot for this immutable selection basis.
            // Keep influencing results out of that cache, including ancestors
            // that could otherwise conceal an old selected sequence.
            self.uncache_flow_dependencies(graph, wrapping.dependencies().iter().copied());
        }
    }

    fn uncache_flow_dependencies(
        &mut self,
        graph: &crate::engine::validation::ValidationGraph<Tree::Node>,
        nodes: impl Iterator<Item = Tree::Node>,
    ) {
        let mut included = vec![false; graph.nodes.len()];
        for &node in &self.wrapping.uncached {
            included[graph.index(node).expect("retained validated dependency")] = true;
        }
        let mut pending = nodes
            .map(|node| graph.index(node).expect("validated flow dependency"))
            .collect::<Vec<_>>();
        while let Some(index) = pending.pop() {
            if !included[index] {
                included[index] = true;
                pending.extend(graph.parents[index].iter().copied());
            }
        }
        self.wrapping.uncached = graph
            .nodes
            .iter()
            .copied()
            .zip(included)
            .filter_map(|(node, included)| included.then_some(node))
            .collect();
    }

    pub(crate) fn with_flow(
        capture: &'a crate::LayoutFlowInput<'_, Tree>,
        invalidated_nodes: Vec<Tree::Node>,
    ) -> Self {
        let mut session = Self::new(capture.tree, invalidated_nodes);
        session.bound = capture.publication_basis();
        session.install_wrapping(capture.wrapping.clone(), &capture.graph);
        let semantic = capture
            .paragraphs
            .iter()
            .filter_map(|binding| {
                capture
                    .associations
                    .semantic_slot(&binding.prepared)
                    .map(|slot| slot.container)
            })
            .collect::<Vec<_>>();
        if !semantic.is_empty() {
            // Actual semantic work needs the same scope/metric projection even
            // when no wrapping preference or supplied metric activates it.
            session.wrapping.context = Some(capture.wrapping.clone());
            session.uncache_flow_dependencies(&capture.graph, semantic.into_iter());
        }
        if !capture.paragraphs.is_empty() {
            let mut text = crate::text_layout::state::TextComputation::new(
                &capture.paragraphs,
                capture.associations.clone(),
                crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
                    capture.scope_alignments.clone(),
                    capture.associations.inline_box_families().clone(),
                ),
            );
            text.graph = Some(&capture.graph);
            text.wrapping = session.wrapping.context.clone();
            session.text = Some(text);
        }
        session
    }

    pub(super) fn groups_active(&self) -> bool {
        self.groups.is_some()
    }
    pub(super) fn group_basis(&self, root: Tree::Node) -> SessionResult<Tree, GroupBasis<Tree>> {
        self.groups
            .as_ref()
            .map(|groups| groups.basis().map_err(|error| error.layout(root)))
            .transpose()
    }
    pub(super) fn next_groups(&mut self, basis: &GroupBasis<Tree>) {
        if let (Some(state), Some(basis)) = (&mut self.groups, basis) {
            state.next_pass(basis);
        }
    }
    pub(super) fn has_text(&self) -> bool {
        self.text.is_some()
    }

    pub(super) fn canonical_frames(
        &self,
        root: Tree::Node,
    ) -> SessionResult<Tree, CanonicalFrames<Tree>> {
        use crate::output::{AtomicReadbackPhase, GeometryStateOf};
        let mut pending = vec![(root, crate::Point::<f64>::ZERO, crate::Point::<f64>::ZERO)];
        let mut frames = Vec::new();
        while let Some((node, parent_visual, parent_normal)) = pending.pop() {
            let entry = self
                .outputs
                .rounded
                .iter()
                .find(|entry| entry.node() == node)
                .ok_or_else(|| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::TextPlacement,
                        LayoutErrorKindOf::InternalInvariant(
                            LayoutInternalInvariant::MissingStagedUnroundedOutput,
                        ),
                    )
                })?;
            let output = entry.output();
            let (normal, atomic_origins) = match entry.geometry_state() {
                GeometryStateOf::OrdinaryCanonical { normal } => (normal, None),
                GeometryStateOf::CanonicalAtomic {
                    normal,
                    normal_origin,
                    visual_origin,
                    readback: AtomicReadbackPhase::Finalized,
                    ..
                } => (normal, Some((normal_origin, visual_origin))),
                _ => {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::TextPlacement,
                        LayoutErrorKindOf::InternalInvariant(
                            LayoutInternalInvariant::InvalidOutputGeometryPhase,
                        ),
                    ));
                }
            };
            let origin_sum = |parent: crate::Point<f64>, location: crate::Point<Tree::Scalar>| {
                let mut result = parent;
                for (base, delta, target) in [
                    (parent.x, location.x.to_f64(), &mut result.x),
                    (parent.y, location.y.to_f64(), &mut result.y),
                ] {
                    let value = base + delta;
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
                    *target = value;
                }
                Ok(result)
            };
            let (normal_origin, origin) = match atomic_origins {
                Some(origins) => origins,
                None => (
                    origin_sum(parent_normal, normal.location)?,
                    origin_sum(parent_visual, output.location)?,
                ),
            };
            let measured_inset_basis = self
                .outputs
                .unrounded
                .iter()
                .find(|entry| entry.node() == node)
                .and_then(LayoutOutputEntryOf::measured_inset_basis)
                .ok_or_else(|| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::TextPlacement,
                        LayoutErrorKindOf::InternalInvariant(
                            LayoutInternalInvariant::InvalidOutputGeometryPhase,
                        ),
                    )
                })?;
            // Hidden outputs retain node geometry without supplying Text feedback.
            if !normal.is_hidden() {
                frames.push(crate::text_layout::state::CanonicalFrame {
                    node,
                    origin,
                    normal_origin,
                    normal,
                    measured_inset_basis,
                    output,
                });
            }
            pending.extend(
                self.tree
                    .children(node)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .map(|child| (child, origin, normal_origin)),
            );
        }
        Ok(frames)
    }

    pub(super) fn text_pass_basis(
        &self,
        root: Tree::Node,
        frames: Vec<crate::text_layout::state::CanonicalFrame<Tree::Node, Tree::Scalar>>,
    ) -> SessionResult<Tree, crate::text_layout::state::PassBasis<Tree::Node, Tree::Scalar>> {
        self.text
            .as_ref()
            .expect("Text compute entry initialized its state")
            .basis(frames)
            .map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(root),
                    LayoutOperation::TextPlacement,
                    LayoutErrorKindOf::Text(error),
                )
            })
    }

    pub(super) fn admit_text_pass_comparison(
        &self,
        root: Tree::Node,
        current: Option<&crate::text_layout::state::PassBasis<Tree::Node, Tree::Scalar>>,
        previous: Option<&crate::text_layout::state::PassBasis<Tree::Node, Tree::Scalar>>,
    ) -> SessionResult<Tree, ()> {
        let (Some(text), Some(current), Some(previous)) = (&self.text, current, previous) else {
            return Ok(());
        };
        current
            .admit_scope_comparison(previous, &text.scope_resources)
            .map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(root),
                    LayoutOperation::TextPlacement,
                    LayoutErrorKindOf::Text(error),
                )
            })
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

    pub(super) fn formatted_text_valid(&self, node: Tree::Node) -> SessionResult<Tree, bool> {
        self.text
            .as_ref()
            .map_or(Ok(true), |text| text.formatted_valid())
            .map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::TextPlacement,
                    LayoutErrorKindOf::Text(error),
                )
            })
    }

    pub(super) fn begin_text_pass(
        &mut self,
        node: Tree::Node,
        frames: Vec<crate::text_layout::state::CanonicalFrame<Tree::Node, Tree::Scalar>>,
    ) -> SessionResult<Tree, ()> {
        if let Some(text) = self.text.as_mut() {
            text.formatted_next_pass().map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::TextPlacement,
                    LayoutErrorKindOf::Text(error),
                )
            })?;
            text.frames = frames.into();
            if let Some(formatted) = &mut text.formatted {
                formatted.frames = text.frames.clone();
                formatted.wrapping = text.wrapping.clone();
            }
            text.reusable =
                text.pending
                    .iter()
                    .flat_map(|paragraph| {
                        paragraph.entries.iter().zip(&paragraph.amounts).map(
                            |(placement, amounts)| crate::text_layout::fit::ReusableLine {
                                finalized: placement.finalized_line().clone(),
                                amounts: amounts.clone(),
                            },
                        )
                    })
                    .collect::<Vec<_>>()
                    .into();
            text.pending.clear();
            text.boxes.clear();
            text.scope_fragments.clear();
            text.absolute_baselines.clear();
        }
        self.scope_receipts.clear();
        self.scope_ready.clear();
        self.wrapping.reset_receipts();
        self.block_floats.reset();
        self.outputs.reset();
        self.cache.stores.clear();
        Ok(())
    }

    pub(super) fn begin_content_scope(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, ()> {
        self.block_frames.try_reserve(1).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        self.block_frames.push(super::block_scope::Frame {
            node,
            input,
            entry: None,
        });
        Ok(())
    }

    pub(super) fn retain_block_bodies(&mut self) {
        self.record_block_bodies = true;
    }
    pub(super) fn record_block_body(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, ()> {
        if !self.record_block_bodies || !input.run_mode().is_perform_layout() {
            return Ok(());
        }
        let body = super::block_scope::Body {
            node,
            input,
            output,
        };
        if let Some(slot) = self.block_bodies.iter_mut().find(|slot| slot.node == node) {
            *slot = body;
        } else {
            self.block_bodies.try_reserve(1).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            self.block_bodies.push(body);
        }
        Ok(())
    }

    pub(super) fn block_packet(
        self,
    ) -> SessionResult<Tree, super::block_scope::Packet<Tree::Node, Tree::Scalar>> {
        let mut receipts = Vec::new();
        let mut projections = Vec::new();
        receipts
            .try_reserve(self.scope_receipts.len())
            .and_then(|_| projections.try_reserve(self.scope_receipts.len()))
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Standalone,
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        for (node, adoption) in self.scope_receipts {
            receipts.push(adoption.receipt);
            if let Some(projection) = adoption.projection {
                projections.push((node, projection));
            }
        }
        Ok(super::block_scope::Packet {
            target: self.bound.expect("scope production is bound to a target"),
            entries: self.outputs.unrounded,
            margins: self
                .fragmentation
                .expect("bound scope has margin provenance")
                .margins,
            receipts,
            projections,
            clears: self.cache.clears,
            bodies: self.block_bodies,
        })
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

    fn complete(self, root: Tree::Node) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let mut unrounded_inline_fragments: Vec<_> = self
            .outputs
            .rounded_fragments
            .iter()
            .flat_map(|final_group| {
                self.outputs
                    .unrounded_fragments
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
            .outputs
            .rounded_fragments
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
        let mut batch = CompletedLayoutBatchOf::from_entries(
            root,
            crate::output::CompletedBatchEntries {
                unrounded: self.outputs.unrounded,
                final_entries: self.outputs.rounded,
                unrounded_inline: unrounded_inline_fragments,
                final_inline: final_inline_fragments,
                stores: self.cache.stores,
                clears: self.cache.clears,
                invalidated: self.cache.invalidated,
            },
        );
        batch.target = self.bound;
        for entry in &mut batch.nodes {
            if let crate::block_fragmentation::NodePublication::One(value) = entry {
                value.role = self.tree.node_input(value.node()).block_generation;
            }
        }
        if let Some(graph) = self.flow_graph {
            for (index, &node) in graph.nodes.iter().enumerate() {
                if graph.parents[index].len() == 1 {
                    let parent = graph.nodes[graph.parents[index][0]];
                    let id = batch.nodes.iter().find_map(|entry| match entry {
                        crate::block_fragmentation::NodePublication::One(value)
                            if value.node() == parent =>
                        {
                            Some(value.id.clone())
                        }
                        _ => None,
                    });
                    if let Some(id) = id
                        && let Some(crate::block_fragmentation::NodePublication::One(value)) =
                            batch.nodes.iter_mut().find(|entry| entry.node() == node)
                    {
                        value.parent = Some(crate::FragmentCoordinateSpace::Fragment(id));
                    }
                }
            }
        }
        batch.with_native_inline_boxes(
            self.wrapping.context,
            self.wrapping.native_runs,
            self.wrapping.native_completions,
        )
    }

    pub(super) fn complete_for_root(
        mut self,
        root: Tree::Node,
    ) -> CompletedLayoutBatchOf<Tree::Node, Tree::Scalar> {
        let mut visited = Vec::new();
        let mut pending = vec![root];
        let mut ordered_unrounded = Vec::with_capacity(self.outputs.unrounded.len());
        let mut ordered_final = Vec::with_capacity(self.outputs.rounded.len());
        let mut source_children = Vec::new();
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if let Some(entry) = take_layout_entry(&mut self.outputs.unrounded, node) {
                ordered_unrounded.push(entry);
            }
            if let Some(entry) = take_layout_entry(&mut self.outputs.rounded, node) {
                ordered_final.push(entry);
            }
            let children = self.tree.children(node).collect::<Vec<_>>();
            pending.extend(children.iter().rev().copied());
            source_children.push((
                node,
                children
                    .into_iter()
                    .enumerate()
                    .map(|(index, child)| (crate::SourceIndex::new(index), child))
                    .collect::<Vec<_>>(),
            ));
        }
        assert!(
            self.outputs.unrounded.is_empty() && self.outputs.rounded.is_empty(),
            "completed output nodes remain reachable from the validated root"
        );
        self.outputs.unrounded = ordered_unrounded;
        self.outputs.rounded = ordered_final;
        let mut batch = self.complete(root);
        let published: Vec<_> = batch.nodes.iter().map(|entry| entry.node()).collect();
        for (node, mut children) in source_children {
            if let Some(crate::block_fragmentation::NodePublication::One(value)) =
                batch.nodes.iter_mut().find(|entry| entry.node() == node)
            {
                children.retain(|(_, child)| published.contains(child));
                value.children = children;
            }
        }
        batch
    }

    fn retain_native_completion(
        &mut self,
        node: Tree::Node,
        completion: crate::inline_wrapping::family::NativeFormatterCompletion<Tree::Scalar>,
    ) -> SessionResult<Tree, ()> {
        if let Some(old) = self
            .wrapping
            .native_completions
            .iter_mut()
            .find(|old| old.family == completion.family)
        {
            *old = completion;
        } else {
            self.wrapping
                .native_completions
                .try_reserve(1)
                .map_err(|_| {
                    crate::inline_wrapping::wrapping_error(
                        node,
                        crate::WrappingScopeError::Allocation,
                    )
                })?;
            self.wrapping.native_completions.push(completion);
        }
        Ok(())
    }

    fn replace_native_run(
        &mut self,
        node: Tree::Node,
        run: crate::inline_wrapping::family::NativePublication<Tree::Scalar>,
    ) -> SessionResult<Tree, ()> {
        if let Some(index) = self
            .wrapping
            .native_runs
            .iter()
            .position(|old| old.family == run.family && old.source == run.source)
        {
            self.wrapping.native_runs[index] = run;
        } else {
            self.wrapping.native_runs.try_reserve(1).map_err(|_| {
                crate::inline_wrapping::wrapping_error(node, crate::WrappingScopeError::Allocation)
            })?;
            self.wrapping.native_runs.push(run);
        }
        Ok(())
    }

    fn staged_source_index(&self, node: Tree::Node) -> crate::SourceIndex {
        self.outputs
            .unrounded
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
        if self.bound.is_some() {
            return false;
        }
        let mut visited = Vec::new();
        let mut pending = vec![root];
        let mut restored = Vec::new();
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            let Some(entry) = self.tree.unrounded_layout(node) else {
                return false;
            };
            if entry.node() != node || !entry.geometry_state().is_unrounded() {
                return false;
            }
            if let crate::output::GeometryStateOf::CanonicalAtomic {
                normal,
                normal_origin,
                visual_origin,
                ..
            } = entry.geometry_state()
            {
                let current = self.text.as_ref().and_then(|text| text.frame(node));
                if current.is_none_or(|frame| {
                    frame.normal != normal
                        || frame.normal_origin != normal_origin
                        || frame.origin != visual_origin
                        || frame.output.size != entry.output().size
                }) {
                    return false;
                }
            }
            restored.push(entry);
            let children = self.tree.children(node).collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        }
        for entry in restored {
            self.set_unrounded(entry);
        }
        true
    }

    fn publication_kind(
        &self,
        entry: &LayoutOutputEntryOf<Tree::Node, Tree::Scalar>,
    ) -> Option<crate::NonBoxKind> {
        use crate::output::GeometryStateOf as G;
        let hidden = match entry.geometry_state() {
            G::OrdinaryUnrounded { measured_normal } => measured_normal.is_hidden(),
            G::OrdinaryCanonical { normal } | G::CanonicalAtomic { normal, .. } => {
                normal.is_hidden()
            }
        };
        if hidden {
            return Some(crate::NonBoxKind::Hidden);
        }
        match self.tree.layout_input(entry.node()) {
            crate::LayoutInputViewOf::Box(_) => None,
            crate::LayoutInputViewOf::InlineText(_) => Some(crate::NonBoxKind::InlineText),
            crate::LayoutInputViewOf::TextControl(_) => Some(crate::NonBoxKind::TextControl),
            crate::LayoutInputViewOf::LineBreak(_) => Some(crate::NonBoxKind::LineBreak),
            crate::LayoutInputViewOf::InlineBoundary(_) => Some(crate::NonBoxKind::InlineBoundary),
        }
    }

    fn prepare_child_input(
        &self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeInputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        let input = if input.run_mode().is_perform_layout() {
            self.fragment_allocations()
                .iter()
                .find(|allocation| allocation.node == node)
                .map_or(input, |allocation| {
                    input.with_fragment_allocation(allocation.known, allocation.indefinite_axis)
                })
        } else {
            input
        };
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
            // Hidden traversal publishes zero output independently of indentation.
            if input.run_mode() != RunMode::PerformHiddenLayout {
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
        }
        Ok(input)
    }

    fn compute_child_uncached(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
    {
        super::block_scope::ordinary(self.compute_child_uncached_scoped(node, input))
    }

    fn compute_child_uncached_scoped(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> super::block_scope::ComputeResult<
        Tree::Node,
        ComputeOutputOf<Tree::Scalar>,
        Tree::Scalar,
        Tree::MeasureError,
    > {
        if input.run_mode().is_perform_layout()
            && let Some(index) = self.wrapping.context.as_ref().and_then(|context| {
                context
                    .families
                    .iter()
                    .position(|family| family.formatter() == node)
            })
        {
            // Recompute this formatter's accepted runs rather than appending an
            // earlier scrollbar/size trial. Intrinsic probes do not clear them.
            self.wrapping.native_runs.retain(|run| run.family != index);
            self.wrapping
                .native_completions
                .retain(|completion| completion.family != index);
        }
        let style = self.tree.node_input(node);
        if self.tree.has_leaf_measurement(node) {
            let icb = self.fragment_intrinsics().map(|context| {
                context
                    .request
                    .sizing_request()
                    .orthogonal_fallback()
                    .initial_containing_block()
                    .map(|value| {
                        value.expect("native intrinsic context has first-frame ICB authority")
                    })
            });
            let result = if let Some(memo) = &self.entry_sources {
                let funding = self.entry_funding(node)?;
                crate::measurement::compute_leaf_with_sampler(node, input, style, icb, |request| {
                    let mut memo = memo
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let error = |error| {
                        LayoutErrorOf::new(
                            LayoutErrorSiteOf::Node(node),
                            LayoutOperation::ChildLayout,
                            LayoutErrorKindOf::Text(error),
                        )
                    };
                    if let Some(result) =
                        memo.find(node, request, funding.as_ref()).map_err(error)?
                    {
                        return Ok(result);
                    }
                    memo.reserve(funding.as_ref()).map_err(error)?;
                    let result = self
                        .tree
                        .measure_leaf(node, request)
                        .ok_or_else(|| {
                            LayoutErrorOf::new(
                                LayoutErrorSiteOf::Node(node),
                                LayoutOperation::LeafMeasurement,
                                LayoutErrorKindOf::InternalInvariant(
                                    LayoutInternalInvariant::MissingLeafMeasurementProvider,
                                ),
                            )
                        })?
                        .map_err(|error| {
                            LayoutErrorOf::new(
                                LayoutErrorSiteOf::Node(node),
                                LayoutOperation::LeafMeasurement,
                                LayoutErrorKindOf::Measurement(error),
                            )
                        })?;
                    let result = crate::measurement::validate_native_sample(node, result)?;
                    memo.record(node, request, result, funding.as_ref())
                        .map_err(error)?;
                    Ok(result)
                })
            } else if let Some(icb) = icb {
                crate::measurement::compute_tree_leaf_with_intrinsic_icb(
                    self.tree,
                    node,
                    input,
                    style,
                    Some(icb),
                )
            } else {
                crate::measurement::compute_tree_leaf(self.tree, node, input, style)
            };
            if result.is_ok()
                && let Some(text) = &mut self.text
            {
                let anonymous = style.block_generation == crate::BlockGeneration::Anonymous;
                let error = |error| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::Text(error),
                    )
                };
                if text
                    .formatted_begin_block(node, input, anonymous)
                    .map_err(error)?
                {
                    // The actual host leaf operation emits no line-box API.
                    // A positive measured size is not a formatted line.
                    let flow = text.formatted_new_flow();
                    text.formatted_record_flow(&flow).map_err(error)?;
                    text.formatted_end_block(node, input, anonymous, true)
                        .map_err(error)?;
                }
            }
            if let Ok(output) = &result {
                self.record_block_body(node, input, *output)?;
            }
            return result.map_err(Into::into);
        }

        let output = match style.display.inner_display() {
            crate::Display::Block => crate::block::compute_block_scoped(self, node, input),
            crate::Display::Flex => {
                crate::flex::compute_flex(self, node, input).map_err(Into::into)
            }
            crate::Display::Grid | crate::Display::GridLanes => {
                crate::grid::compute_grid(self, node, input).map_err(Into::into)
            }
            crate::Display::None => super::compute_hidden(
                self,
                node,
                self.staged_source_index(node),
                input.containing_layout_context(),
                input.containing_auto_scrollbar_pass(),
            )
            .map_err(Into::into),
            crate::Display::InlineBlock
            | crate::Display::InlineGrid
            | crate::Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }?;
        if input.run_mode().is_perform_layout()
            && let Some(family) = self.wrapping.context.as_ref().and_then(|context| {
                context
                    .families
                    .iter()
                    .position(|family| family.formatter() == node)
            })
        {
            self.retain_native_completion(
                node,
                crate::inline_wrapping::family::NativeFormatterCompletion {
                    family,
                    input,
                    output,
                },
            )?;
        }
        self.record_block_body(node, input, output)?;
        Ok(output)
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

    fn wrapping_context(&self) -> Option<super::contracts::WrappingContextLoan<'_, Self::Node>> {
        Some((self.wrapping.context.as_ref()?, self.flow_graph?))
    }

    fn block_float_state(
        &mut self,
    ) -> Option<&mut crate::block::float_context::State<Self::Node, Self::Scalar>> {
        Some(&mut self.block_floats)
    }
    fn fragment_margin_state(
        &mut self,
    ) -> Option<&mut crate::block::fragment_margins::State<Self::Node, Self::Scalar>> {
        self.fragmentation
            .as_mut()
            .map(|fragment| &mut fragment.margins)
    }
    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.tree.has_leaf_measurement(node)
    }
    fn qualify_entry(
        &mut self,
        query: crate::block::entry::EntryQuery<Self::Node, Self::Scalar>,
    ) -> SessionResult<Tree, crate::block::entry::EntryObservation<Self::Node, Self::Scalar>> {
        use crate::block::entry::EntryPurpose;
        if query.purpose == EntryPurpose::StartOnly {
            return crate::block::entry::observe(self, query);
        }
        if !query
            .incoming
            .as_ref()
            .is_some_and(|incoming| incoming.node == query.node && incoming.request == query.input)
        {
            return Err(crate::block::float_context::missing(query.node));
        }
        if let Some(text) = &mut self.text {
            text.prepare_entry().map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(query.node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Text(error),
                )
            })?;
        }
        if self.entry_sources.is_none() {
            let header = self
                .entry_funding(query.node)?
                .as_ref()
                .map(|funding| {
                    funding.header::<(
                        usize,
                        usize,
                        std::sync::Mutex<
                            super::entry_sources::SourceMemo<Tree::Node, Tree::Scalar>,
                        >,
                    )>()
                })
                .transpose()
                .map_err(|error| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(query.node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::Text(error),
                    )
                })?;
            self.entry_sources = Some(std::sync::Arc::new(std::sync::Mutex::new(
                super::entry_sources::SourceMemo::new(header),
            )));
        }
        let mut trial =
            self.child_staging(query.node, FloatChildTrial::InFlow, query.incoming.clone())?;
        crate::block::entry::resolve_framed(&mut trial, query)
    }
    fn measure_entry_leaf(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
    ) -> SessionResult<Tree, ComputeOutputOf<Self::Scalar>> {
        if !self.tree.has_leaf_measurement(node) {
            return Err(crate::block::float_context::missing(node));
        }
        let input = self.prepare_child_input(node, input)?;
        self.compute_child_uncached(node, input.with_run_mode(RunMode::ComputeSize))
    }
    fn block_float_work(&self, node: Self::Node) -> bool {
        if let Some(graph) = self.flow_graph {
            let index = graph
                .index(node)
                .expect("actual child belongs to this immutable graph");
            self.block_floats
                .needs_work(index)
                .expect("float work proof shares the validated graph")
        } else {
            crate::block::float_context::discover_work(self, node)
        }
    }

    fn group_computation(
        &mut self,
    ) -> Option<&mut crate::group_alignment::State<Self::Node, Self::Scalar>> {
        self.groups.as_mut()
    }

    fn text_computation(
        &mut self,
    ) -> Option<&mut crate::text_layout::state::TextComputation<'input, Self::Node, Self::Scalar>>
    {
        self.text.as_mut()
    }

    fn staged_unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<
        Self::Node,
        LayoutOutputEntryOf<Self::Node, Self::Scalar>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        self.outputs
            .unrounded
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .copied()
            .filter(|entry| entry.geometry_state().is_unrounded())
            .ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::InternalInvariant(
                        LayoutInternalInvariant::MissingStagedUnroundedOutput,
                    ),
                )
            })
    }

    fn set_unrounded(&mut self, mut entry: LayoutOutputEntryOf<Self::Node, Self::Scalar>) {
        if let Some(&(_, id)) = self
            .scope_ready
            .iter()
            .find(|(node, _)| *node == entry.node())
        {
            if let Some((_, current)) = self
                .scope_receipts
                .iter_mut()
                .find(|(node, _)| *node == entry.node())
            {
                *current = id;
            } else {
                self.scope_receipts.push((entry.node(), id));
            }
        }

        let node = entry.node();
        entry.non_box = self.publication_kind(&entry);
        if let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
            entry.geometry_state()
            && measured_normal.is_hidden()
        {
            if let Some(text) = &mut self.text {
                text.discard_placement(node);
            }
            self.cache.stores.retain(|entry| entry.node() != node);
        }
        if let Some(index) = self
            .outputs
            .unrounded
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.outputs.unrounded.remove(index);
        }
        self.outputs.unrounded.push(entry);
    }

    fn publish_native_inline_run(
        &mut self,
        node: Self::Node,
        source: std::ops::Range<usize>,
        completion: Option<std::sync::Arc<crate::inline_wrapping::NativeOrderingProjection>>,
        lines: Vec<crate::inline_wrapping::family::NativeLineReceipt<Self::Scalar>>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, Tree::MeasureError> {
        let family = self.wrapping.context.as_ref().and_then(|context| {
            context
                .families
                .iter()
                .position(|family| family.formatter() == node)
        });
        let Some(family_index) = family else {
            if lines.is_empty() && completion.is_none() {
                return Ok(());
            }
            return Err(crate::inline_wrapping::wrapping_error(
                node,
                crate::WrappingScopeError::InconsistentIncidence,
            ));
        };
        let completion = completion.ok_or_else(|| {
            crate::inline_wrapping::wrapping_error(
                node,
                crate::WrappingScopeError::InconsistentIncidence,
            )
        })?;
        let actual_source = completion
            .selection(0..completion.ranks.len())
            .map_err(|error| crate::inline_wrapping::wrapping_error(node, error))?
            .raw_fences;
        let family = &self
            .wrapping
            .context
            .as_ref()
            .expect("actual requested family context")
            .families[family_index];
        if actual_source != source || !std::sync::Arc::ptr_eq(family.profile(), &completion.profile)
        {
            return Err(crate::inline_wrapping::wrapping_error(
                node,
                crate::WrappingScopeError::InconsistentIncidence,
            ));
        }
        if lines.iter().any(|line| {
            !std::sync::Arc::ptr_eq(&completion, &line.receipt)
                || !std::sync::Arc::ptr_eq(family.profile(), &line.profile)
                || line.source.start < source.start
                || line.source.end > source.end
                || line.source.start > line.source.end
        }) {
            return Err(crate::inline_wrapping::wrapping_error(
                node,
                crate::WrappingScopeError::InconsistentIncidence,
            ));
        }
        let run = crate::inline_wrapping::family::NativePublication::new(
            family_index,
            source,
            completion,
            lines,
        )
        .map_err(|error| crate::inline_wrapping::wrapping_error(node, error))?;
        self.replace_native_run(node, run)
    }

    fn set_unrounded_inline_fragment_state(
        &mut self,
        node: Self::Node,
        fragments: Option<Vec<InlineFragmentOutputOf<Self::Scalar>>>,
    ) {
        set_inline_fragment_group(&mut self.outputs.unrounded_fragments, node, fragments);
    }

    fn fragment_minimum(
        &self,
        node: Self::Node,
    ) -> Option<crate::block::fragment_minimum::Override<Self::Scalar>> {
        self.fragment_allocations()
            .iter()
            .find(|entry| entry.node == node)
            .and_then(|entry| {
                entry
                    .minimum_content
                    .map(|content| crate::block::fragment_minimum::Override {
                        axis: entry.indefinite_axis,
                        content,
                    })
            })
    }

    fn fragment_inline_content(
        &self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
        operation: crate::SizingOperation,
    ) -> LayoutResultOf<Self::Node, Option<Self::Scalar>, Self::Scalar, Tree::MeasureError> {
        let Some(context) = self.fragment_intrinsics() else {
            return Ok(None);
        };
        let Some(entry) = context.entries.iter().find(|entry| entry.node == node) else {
            return Ok(None);
        };
        let style = self.tree.node_input(node);
        let own = crate::FlowAxes::new(style.writing_mode, style.direction);
        // A used block allocation may change Normal geometry, never the
        // descendant percentage basis or canonical inline containing context.
        if entry.input.parent() != input.parent()
            || entry.input.containing_layout_context() != input.containing_layout_context()
            || entry.input.sizing_mode() != input.sizing_mode()
            || own.logical_size(entry.input.known()).inline
                != own.logical_size(input.known()).inline
        {
            return Err(crate::block::intrinsic::profile_error(node));
        }
        entry
            .values
            .iter()
            .find(|(op, _)| *op == operation)
            .map(|(_, value)| Some(*value))
            .ok_or_else(|| crate::block::intrinsic::profile_error(node))
    }

    fn compute_axis_demand(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
        axis: crate::PhysicalAxis,
    ) -> LayoutResultOf<Self::Node, Self::Scalar, Self::Scalar, Tree::MeasureError> {
        let style = self.tree.node_input(node);
        let own = crate::FlowAxes::new(style.writing_mode, style.direction);
        if !self.tree.has_leaf_measurement(node) {
            if own.inline_axis() == axis && style.display.inner_display() == crate::Display::Block {
                return crate::block::intrinsic::axis_demand(self, node, input, axis);
            }
            if self.bound.is_some() && self.child_count(node) != 0 {
                return Err(crate::block::intrinsic::profile_error(node));
            }
        }
        let output = self.compute_child(node, input)?;
        Ok(match axis {
            crate::PhysicalAxis::Horizontal => output.size.width,
            crate::PhysicalAxis::Vertical => output.size.height,
        })
    }

    fn block_scope_path(&self, node: Self::Node) -> bool {
        self.block_scopes
            .as_ref()
            .is_some_and(|dispatch| dispatch.paths.contains(&node))
    }

    fn block_scope_child(&self, node: Self::Node) -> bool {
        self.block_scopes
            .as_ref()
            .is_some_and(|dispatch| dispatch.owners.contains(&node))
    }

    fn compute_block_child(
        &mut self,
        request: crate::block::scope::ChildRequest<Self::Node, Self::Scalar>,
    ) -> super::block_scope::ComputeResult<
        Self::Node,
        ComputeOutputOf<Self::Scalar>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        match request {
            crate::block::scope::ChildRequest::Scope(entry) => {
                let node = entry.node;
                let dispatch = self
                    .block_scopes
                    .as_ref()
                    .expect("Block qualified a driver-owned scope");
                let ready = dispatch
                    .ready
                    .borrow()
                    .iter()
                    .find(|ready| ready.entry == entry && ready.view == dispatch.view)
                    .copied();
                if let Some(ready) = ready {
                    if let Some(projection) = ready.projection {
                        self.fragmentation
                            .as_mut()
                            .expect("bound Block retains margins")
                            .margins
                            .bind(node, projection)
                            .map_err(|error| {
                                LayoutErrorOf::new(
                                    LayoutErrorSiteOf::Node(node),
                                    LayoutOperation::ChildLayout,
                                    LayoutErrorKindOf::Text(error),
                                )
                            })?;
                    }
                    let adoption = super::block_scope::Adoption {
                        receipt: ready.id,
                        projection: ready.projection,
                    };
                    if let Some((_, id)) = self.scope_ready.iter_mut().find(|(key, _)| *key == node)
                    {
                        *id = adoption;
                    } else {
                        let needed = self.scope_ready.len() + 1 - self.scope_receipts.len();
                        self.scope_ready
                            .try_reserve(1)
                            .and_then(|_| self.scope_receipts.try_reserve(needed))
                            .map_err(|_| {
                                LayoutErrorOf::new(
                                    LayoutErrorSiteOf::Node(node),
                                    LayoutOperation::ChildLayout,
                                    LayoutErrorKindOf::Allocation,
                                )
                            })?;
                        self.scope_ready.push((node, adoption));
                    }
                    self.record_block_body(node, entry.input, ready.output)?;
                    return Ok(ready.output);
                }
                let request = super::block_scope::Request {
                    scope: dispatch.scope,
                    view: dispatch.view,
                    entry,
                    frames: std::mem::take(&mut self.block_frames),
                };
                let id = dispatch
                    .requests
                    .borrow_mut()
                    .insert(request)
                    .map_err(|_| {
                        LayoutErrorOf::new(
                            LayoutErrorSiteOf::Node(node),
                            LayoutOperation::ChildLayout,
                            LayoutErrorKindOf::Allocation,
                        )
                    })?;
                Err(super::block_scope::ComputeStop::NeedScope(id))
            }
            crate::block::scope::ChildRequest::Ordinary(request) => {
                if self.block_scopes.is_some()
                    && !self.tree.has_leaf_measurement(request.node)
                    && self.tree.node_input(request.node).display.inner_display()
                        == crate::Display::Block
                {
                    let input = self.prepare_child_input(request.node, request.input)?;
                    if input.run_mode() == RunMode::PerformHiddenLayout
                        || self.tree.node_input(request.node).display == crate::Display::None
                    {
                        return self.compute_child(request.node, input).map_err(Into::into);
                    }
                    let depth = self.block_frames.len();
                    self.block_frames.try_reserve(1).map_err(|_| {
                        LayoutErrorOf::new(
                            LayoutErrorSiteOf::Node(request.node),
                            LayoutOperation::ChildLayout,
                            LayoutErrorKindOf::Allocation,
                        )
                    })?;
                    self.block_frames.push(super::block_scope::Frame {
                        node: request.node,
                        input,
                        entry: request.entry,
                    });
                    let result = self.compute_child_uncached_scoped(
                        request.node,
                        input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL),
                    );
                    if !matches!(result, Err(super::block_scope::ComputeStop::NeedScope(_))) {
                        self.block_frames.truncate(depth);
                    }
                    result
                } else {
                    self.compute_child(request.node, request.input)
                        .map_err(Into::into)
                }
            }
        }
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

        let input = self.prepare_child_input(node, input)?;
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

        if let Some(memo) = &self.entry_sources {
            let funding = self.entry_funding(node)?;
            let measured = memo
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains(node, funding.as_ref())
                .map_err(|error| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::Text(error),
                    )
                })?;
            if measured {
                return self.compute_child_uncached(node, input);
            }
        }

        if input.run_mode().is_perform_layout() && self.child_count(node) != 0 {
            let input = input.with_settled_auto_scrollbars(SettledAutoScrollbarState::INITIAL);
            if self.subtree_uses_shape_provider(node) {
                let context = <Self as CacheAccess<Tree::MeasureError>>::cache_context(self);
                if let Some(output) = <Self as CacheAccess<Tree::MeasureError>>::cache_get(
                    self, node, &input, context,
                )? && self.cache_restore_semantics(node, &input)?
                    && self.restore_committed_subtree(node)
                {
                    return Ok(output);
                }
                let output = self.compute_child_uncached(node, input)?;
                <Self as CacheAccess<Tree::MeasureError>>::cache_store(
                    self, node, &input, context, output,
                )?;
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
        let mut observation = Self::new(self.tree, self.cache.invalidated.clone());
        observation.block_floats = self.block_floats.fork();
        observation.entry_sources = self.entry_sources.clone();
        observation.wrapping.context = self.wrapping.context.clone();
        observation.flow_graph = self.flow_graph;
        observation.bound = self.bound.clone();
        if let Some(fragment) = &self.fragmentation {
            let allocations = fragment.allocations.clone();
            let intrinsics = fragment.intrinsics.clone();
            let margins = fragment.margins.fork_entry(None).map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Text(error),
                )
            })?;
            observation.fragmentation = Some(FragmentComputation {
                margins,
                allocations,
                intrinsics,
            });
        }
        observation.wrapping.uncached = self.wrapping.uncached.clone();
        if let Some(text) = &self.text {
            let mut state = crate::text_layout::state::TextComputation::new(
                text.paragraphs,
                text.associations.clone(),
                text.scope_resources.clone(),
            );
            state.frames = text.frames.clone();
            state.wrapping = text.wrapping.clone();
            state.graph = text.graph;
            state.formatted = text.formatted.as_ref().map(|state| state.fork());
            observation.text = Some(state);
        }
        let result = probe.observe(&mut observation, node);
        if result.is_ok()
            && let Some(formatted) = observation
                .text
                .as_ref()
                .and_then(|text| text.formatted.as_ref())
            && let Some(text) = &mut self.text
        {
            // A consumed probe can affect the parent's sizing decision. Retain
            // only its purpose-scoped semantic dependencies; none of its
            // geometry, pending Text, fragments or cache writes are published.
            text.formatted_accept(formatted).map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Text(error),
                )
            })?;
        }
        result
    }

    fn probe_float_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<Self::Scalar>,
        trial: FloatChildTrial,
    ) -> LayoutResultOf<
        Self::Node,
        std::sync::Arc<FloatChildPacket<Self::Node, Self::Scalar>>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        let mut probe = self.child_staging(node, trial, None)?;
        let frames = probe
            .text
            .as_ref()
            .map(|state| std::sync::Arc::clone(&state.frames));
        let output = probe.compute_child(node, input)?;
        let (pending, absolute_baselines, scope_fragments, formatted, boxes) =
            probe.text.map_or_else(
                || (Vec::new(), Vec::new(), Vec::new(), None, Vec::new()),
                |text| {
                    (
                        text.pending,
                        text.absolute_baselines,
                        text.scope_fragments,
                        text.formatted,
                        text.boxes,
                    )
                },
            );
        let associations = self.text.as_ref().map(|text| text.associations.clone());
        let funding = if self.entry_sources.is_some() {
            self.entry_funding(node)?
        } else {
            None
        };
        let formatted_basis = self
            .text
            .as_ref()
            .and_then(|text| text.formatted.as_ref())
            .map(|state| state.entry_basis(funding.as_ref(), trial == FloatChildTrial::InFlow))
            .transpose()
            .map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Text(error),
                )
            })?;
        let packet_owner = funding
            .as_ref()
            .map(|funding| {
                funding.header::<(usize, usize, FloatChildPacket<Self::Node, Self::Scalar>)>()
            })
            .transpose()
            .map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Text(error),
                )
            })?;
        Ok(std::sync::Arc::new(FloatChildPacket {
            groups: probe.groups,
            boxes,
            wrapping: self.wrapping.context.clone(),
            formatted_basis,
            formatted,
            node,
            input,
            output,
            frames,
            associations,
            entries: probe.outputs.unrounded,
            fragments: probe.outputs.unrounded_fragments,
            native_inline_runs: probe.wrapping.native_runs,
            native_inline_completions: probe.wrapping.native_completions,
            stores: probe.cache.stores,
            clears: probe.cache.clears,
            pending,
            absolute_baselines,
            scope_fragments,
            _entry_clears: probe
                .entry_storage
                .as_mut()
                .map(|storage| std::mem::take(&mut storage.clears)),
            _entry_stores: probe
                .entry_storage
                .as_mut()
                .map(|storage| std::mem::take(&mut storage.stores)),
            _entry_owner: packet_owner,
        }))
    }

    fn accept_float_child(
        &mut self,
        packet: &FloatChildPacket<Self::Node, Self::Scalar>,
    ) -> LayoutResultOf<Self::Node, (), Self::Scalar, Tree::MeasureError> {
        if !match (&self.groups, &packet.groups) {
            (Some(left), Some(right)) => left.same_plan(right),
            (None, None) => true,
            _ => false,
        } {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(packet.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::TextFeedbackCycle,
            ));
        }
        if self.wrapping.context != packet.wrapping {
            return Err(crate::inline_wrapping::wrapping_error(
                packet.node,
                crate::inline_wrapping::WrappingScopeError::InconsistentIncidence,
            ));
        }
        if let Some(text) = &self.text
            && (!packet
                .frames
                .as_ref()
                .is_some_and(|frames| std::sync::Arc::ptr_eq(&text.frames, frames))
                || !packet
                    .associations
                    .as_ref()
                    .is_some_and(|basis| std::sync::Arc::ptr_eq(&text.associations, basis)))
        {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(packet.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::TextFeedbackCycle,
            ));
        }
        let funding = if self.entry_sources.is_some() {
            self.entry_funding(packet.node)?
        } else {
            None
        };
        let formatted = self.text.as_ref().and_then(|text| text.formatted.as_ref());
        let assignments = formatted.map_or(&[][..], |state| state.assignments.as_slice());
        let matches = if let Some(basis) = &packet.formatted_basis {
            basis
                .matches(
                    assignments,
                    formatted.map(|state| state.stack.as_slice()),
                    funding
                        .as_ref()
                        .map(crate::text_layout::scope_alignment::EntryFunding::as_ref),
                )
                .map_err(|error| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(packet.node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::Text(error),
                    )
                })?
        } else {
            assignments.is_empty()
        };
        if !matches {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(packet.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::TextFeedbackCycle,
            ));
        }
        if let (Some(groups), Some(accepted)) = (&mut self.groups, &packet.groups) {
            groups.accept(accepted);
        }
        for entry in &packet.entries {
            self.set_unrounded(*entry);
        }
        for group in &packet.fragments {
            self.set_unrounded_inline_fragment_state(group.node, group.fragments.clone());
        }
        for completion in &packet.native_inline_completions {
            let formatter = self
                .wrapping
                .context
                .as_ref()
                .and_then(|context| context.families.get(completion.family))
                .ok_or_else(|| {
                    crate::inline_wrapping::wrapping_error(
                        packet.node,
                        crate::WrappingScopeError::InconsistentIncidence,
                    )
                })?;
            if !completion.input.run_mode().is_perform_layout()
                || (formatter.formatter() == packet.node && completion.output != packet.output)
            {
                return Err(crate::inline_wrapping::wrapping_error(
                    packet.node,
                    crate::WrappingScopeError::InconsistentIncidence,
                ));
            }
            self.wrapping
                .native_runs
                .retain(|run| run.family != completion.family);
            self.retain_native_completion(packet.node, *completion)?;
        }
        for run in &packet.native_inline_runs {
            let accepted = run
                .try_clone()
                .map_err(|error| crate::inline_wrapping::wrapping_error(packet.node, error))?;
            self.replace_native_run(packet.node, accepted)?;
        }
        for entry in &packet.stores {
            self.cache_store(entry.node(), entry.input(), entry.context(), entry.output())?;
        }
        for entry in &packet.clears {
            self.cache_clear(entry.node())?;
        }
        if let Some(text) = &mut self.text {
            if let Some(observed) = &packet.formatted {
                text.formatted_accept(observed).map_err(|error| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(packet.node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::Text(error),
                    )
                })?;
            }
            for group in &packet.scope_fragments {
                text.record_scope_fragments(
                    group.container,
                    group.prepared.clone(),
                    group.nodes.clone(),
                    group.fragments.clone(),
                );
            }
            for uses in &packet.boxes {
                text.record_boxes(uses.clone());
            }
            for paragraph in &packet.pending {
                text.record(paragraph.clone());
            }
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

    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<
        Self::Node,
        LayoutOutputEntryOf<Self::Node, Self::Scalar>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        self.outputs
            .unrounded
            .iter()
            .rev()
            .find(|entry| entry.node() == node)
            .copied()
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

    fn set_final(&mut self, mut entry: LayoutOutputEntryOf<Self::Node, Self::Scalar>) {
        let node = entry.node();
        entry.non_box = self.publication_kind(&entry);
        if let Some(index) = self
            .outputs
            .rounded
            .iter()
            .position(|entry| entry.node() == node)
        {
            self.outputs.rounded.remove(index);
        }
        self.outputs.rounded.push(entry);
    }

    fn unrounded_inline_fragment_state(
        &self,
        node: Self::Node,
    ) -> UnroundedInlineFragmentState<'_, Self::Scalar> {
        if let Some(group) = self
            .outputs
            .unrounded_fragments
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
        set_inline_fragment_group(&mut self.outputs.unrounded_fragments, node, Some(unrounded));
        set_inline_fragment_group(
            &mut self.outputs.rounded_fragments,
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
    ) -> SessionResult<Tree, Option<ComputeOutputOf<Self::Scalar>>> {
        if self.bound.is_some()
            || self.cache.invalidated.contains(&node)
            || self.block_floats.blocks_cache(node, input)
            || self.wrapping.uncached.contains(&node)
        {
            return Ok(None);
        }
        let error = |error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Text(error),
            )
        };
        if let Some(text) = &self.text
            && !text.formatted_cache_available(node, input).map_err(error)?
        {
            return Ok(None);
        }
        let Some(output) = self.tree.cache_get(node, input, context) else {
            return Ok(None);
        };
        let matches = self
            .text
            .as_ref()
            .map_or(Ok(true), |text| {
                text.formatted_cache_matches(node, input, context, output)
            })
            .map_err(error)?;
        Ok(matches.then_some(output))
    }

    fn cache_restore_semantics(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
    ) -> SessionResult<Tree, bool> {
        if self.bound.is_some() {
            return Ok(false);
        }
        self.text
            .as_mut()
            .map_or(Ok(true), |text| text.formatted_restore_cache(node, input))
            .map_err(|error| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Text(error),
                )
            })
    }

    fn cache_store(
        &mut self,
        node: Self::Node,
        input: &ComputeInputOf<Self::Scalar>,
        context: crate::CacheKeyContext,
        output: ComputeOutputOf<Self::Scalar>,
    ) -> SessionResult<Tree, ()> {
        if let Some(formatted) = self.text.as_mut().and_then(|text| text.formatted.as_mut()) {
            formatted
                .bind_output(node, input, context, output)
                .map_err(|error| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::Text(error),
                    )
                })?;
        }
        if self.bound.is_some() {
            return Ok(());
        }
        if self.wrapping.uncached.contains(&node) {
            return self.cache_clear(node);
        }
        if self.block_floats.blocks_cache(node, input) {
            return Ok(());
        }

        let funding = if self.entry_storage.is_some() {
            self.entry_funding(node)?
        } else {
            None
        };
        let error = |error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Text(error),
            )
        };
        let mut previous = None;
        for (index, entry) in self.cache.stores.iter().enumerate() {
            entry_staging::inspect(funding.as_ref(), 1).map_err(error)?;
            if entry.node() == node && entry.input() == input && entry.context() == context {
                previous = Some(index);
                break;
            }
        }
        if let Some(index) = previous {
            entry_staging::inspect(funding.as_ref(), self.cache.stores.len() - index - 1)
                .map_err(error)?;
            self.cache.stores.remove(index);
        }
        if let Some(storage) = &mut self.entry_storage {
            storage
                .stores
                .reserve_push(&mut self.cache.stores, funding.as_ref())
                .map_err(error)?;
        }
        self.cache
            .stores
            .push(LayoutCacheStoreEntryOf::new(node, *input, context, output));
        Ok(())
    }

    fn cache_clear(&mut self, node: Self::Node) -> SessionResult<Tree, ()> {
        let funding = if self.entry_storage.is_some() {
            self.entry_funding(node)?
        } else {
            None
        };
        let error = |error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Text(error),
            )
        };
        for entry in &self.cache.clears {
            entry_staging::inspect(funding.as_ref(), 1).map_err(error)?;
            if entry.node() == node {
                return Ok(());
            }
        }
        if let Some(storage) = &mut self.entry_storage {
            storage
                .clears
                .reserve_push(&mut self.cache.clears, funding.as_ref())
                .map_err(error)?;
        }
        self.cache.clears.push(LayoutCacheClearEntry::new(node));
        Ok(())
    }
}

#[cfg(test)]
use crate::NodeOutputOf;

#[cfg(test)]
#[path = "formatted_session_tests.rs"]
mod formatted_tests;

#[cfg(test)]
#[path = "wrapping_session_tests.rs"]
mod wrapping_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn actual_float_packets_preserve_send_sync_for_native_nodes_and_both_scalars() {
        fn require_send_sync<T: Send + Sync>() {}
        require_send_sync::<super::FloatChildPacket<u32, f32>>();
        require_send_sync::<super::FloatChildPacket<u32, f64>>();
    }

    use super::*;
    use crate::{DefaultScalar, LayoutErrorSite, NodeInput, Point, ScrollRectOf, round_layout};

    #[test]
    fn rejected_float_probe_cannot_export_shared_descendant_rows_or_outputs() {
        fn lane<S: LayoutScalar>() {
            let sized = |width: f64, height: Option<f64>| NodeInputOf {
                display: crate::Display::Block,
                size: Size::new(
                    crate::PreferredSizeOf::px(S::from_f64(width)),
                    height.map_or(crate::PreferredSizeOf::AUTO, |height| {
                        crate::PreferredSizeOf::px(S::from_f64(height))
                    }),
                ),
                ..NodeInputOf::default()
            };
            let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [3])
                .children(3, [])
                .style(
                    1,
                    NodeInputOf {
                        float: crate::Float::Left,
                        ..sized(40.0, None)
                    },
                )
                .style(2, sized(40.0, None))
                .style(
                    3,
                    NodeInputOf {
                        float: crate::Float::Left,
                        ..sized(10.0, Some(30.0))
                    },
                );
            let mut session = ComputeSession::new(&tree, Vec::new());
            let input = ComputeInputOf::leaf_layout(
                Size::NONE,
                Size::new(Some(S::from_f64(100.0)), None),
                crate::ContainingLayoutContext::new(
                    crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
                    crate::ParentFormattingContext::BlockFlow,
                ),
                crate::SizingRequestOf::new(
                    Size::new(
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Definite(
                                crate::NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                            ),
                            crate::SizingOperation::Normal,
                        ),
                        crate::AxisSizingRequestOf::new(
                            crate::AvailableSpaceOf::Indefinite,
                            crate::SizingOperation::Normal,
                        ),
                    ),
                    crate::OrthogonalFallbackOf::new(
                        crate::Size::NONE,
                        crate::NearestScrollportOf::Unknown,
                        crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            )
            .unwrap();
            let rejected = session
                .probe_float_child(1, input, FloatChildTrial::Detached)
                .unwrap();
            assert_eq!(rejected.output.size.height, S::from_f64(30.0));
            drop(rejected);
            assert!(session.outputs.unrounded.is_empty());
            assert!(session.cache.stores.is_empty());
            let accepted = session
                .probe_float_child(1, input, FloatChildTrial::Detached)
                .unwrap();
            session.accept_float_child(&accepted).unwrap();
            let child = session
                .outputs
                .unrounded
                .iter()
                .find(|entry| entry.node() == 2)
                .unwrap()
                .output();
            assert_eq!(child.size.height, S::ZERO);
            let float = session
                .outputs
                .unrounded
                .iter()
                .find(|entry| entry.node() == 3)
                .unwrap()
                .output();
            assert_eq!(float.size.height, S::from_f64(30.0));
            assert_eq!(float.location, Point::ZERO);
        }
        lane::<f32>();
        lane::<f64>();
    }

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
                        crate::SizingRequestOf::new(
                            Size::splat(crate::AxisSizingRequestOf::new(
                                crate::AvailableSpaceOf::Indefinite,
                                crate::SizingOperation::Normal,
                            )),
                            crate::OrthogonalFallbackOf::new(
                                crate::Size::NONE,
                                crate::NearestScrollportOf::Unknown,
                                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                            ),
                        ),
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
            let normal = crate::output::NormalGeometryOf::box_geometry(
                Point::ZERO,
                Size::ZERO,
                ScrollRectOf::try_new(Point::ZERO, Size::ZERO).unwrap(),
            );
            Compute::set_unrounded(
                &mut session,
                LayoutOutputEntryOf::ordinary_unrounded(node, normal, NodeOutputOf::new()),
            );
            Round::set_final(
                &mut session,
                LayoutOutputEntryOf::ordinary_canonical(node, normal, NodeOutputOf::new()),
            );
        }

        let batch = session.complete_for_root(0);
        let expected_source_order = [0, 1, 2, 4, 5, 3];

        assert_eq!(
            batch
                .node_layouts()
                .map(|(node, _)| node)
                .collect::<Vec<_>>(),
            expected_source_order
        );
        assert_eq!(
            batch
                .node_layouts()
                .map(|(node, _)| node)
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
        let capture = crate::LayoutFlowInput::try_new(
            &tree,
            0,
            vec![paragraph.binding(0)],
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
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
            let mut session = ComputeSession::with_flow(&capture, Vec::new());
            let mut output = NodeOutputOf::new();
            output.location = Point::new(S::from_f64(0.25), S::from_f64(0.5));
            Compute::set_unrounded(
                &mut session,
                LayoutOutputEntryOf::ordinary_unrounded(
                    1,
                    crate::output::NormalGeometryOf::metadata(output.location, output.size),
                    output,
                ),
            );
            Compute::set_unrounded_inline_fragment_state(&mut session, 1, Some(fragments.clone()));
            round_layout(&mut session, 1).unwrap();
            assert_eq!(session.outputs.rounded[0].output(), output);
            let batch = session.complete(1);
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
    use crate::test_support::readback::{self, Phase};
    use crate::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct ObservationRetainedState<S: LayoutScalar> {
        unrounded: HashMap<u32, LayoutOutputEntryOf<u32, S>>,
        final_outputs: HashMap<u32, LayoutOutputEntryOf<u32, S>>,
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
            for entry in readback::continuous_custody(batch, Phase::Unrounded) {
                prepared.unrounded.insert(entry.node(), entry);
            }
            for entry in readback::continuous_custody(batch, Phase::Final) {
                prepared.final_outputs.insert(entry.node(), entry);
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
        targets: crate::LayoutPublicationTargets<u32>,
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
                targets: crate::LayoutPublicationTargets::new(),
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
            {
                let width = crate::NonNegativeFiniteOf::new(S::from_f64(width)).unwrap();
                // This fixture's real root is Htb/LTR. Its declared percentage
                // parent width is independent of unavailable ICB dimensions.
                crate::LayoutRootRequestOf::standalone(
                    crate::SizingRequestOf::new(
                        Size::new(
                            crate::AxisSizingRequestOf::new(
                                crate::AvailableSpaceOf::Definite(width),
                                crate::SizingOperation::Normal,
                            ),
                            crate::AxisSizingRequestOf::new(
                                crate::AvailableSpaceOf::Indefinite,
                                crate::SizingOperation::Normal,
                            ),
                        ),
                        crate::OrthogonalFallbackOf::new(
                            crate::Size::NONE,
                            crate::NearestScrollportOf::Absent,
                            crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                    crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
                    Size::new(Some(width), None),
                )
            }
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
                        .content_sizing_request()
                        .axes()
                        .width
                        .space()
                        .definite_size()
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
        fn unrounded_layout(&self, node: u32) -> Option<LayoutOutputEntryOf<u32, S>> {
            self.retained.unrounded.get(&node).copied()
        }
    }

    impl<S: LayoutScalar> LayoutBatchSink<u32, S> for GridGutterInlineTree<S> {
        type Error = core::convert::Infallible;
        type Prepared = (ObservationRetainedState<S>, Vec<InlineFragmentOutputOf<S>>);
        fn publication_targets(&self) -> &crate::LayoutPublicationTargets<u32> {
            &self.targets
        }
        fn publication_targets_mut(&mut self) -> &mut crate::LayoutPublicationTargets<u32> {
            &mut self.targets
        }
        fn prepare_layout_batch<'call>(
            &self,
            preparation: crate::LayoutBatchPreparation<'call, u32, S>,
        ) -> Result<Self::Prepared, Self::Error> {
            let batch = preparation.batch();
            let mut retained = self.retained.clone();
            if batch.replaces_whole_root() {
                for node in preparation.previous_members() {
                    retained.unrounded.remove(node);
                    retained.final_outputs.remove(node);
                    if let Some(cache) = retained.caches.get_mut(node) {
                        cache.clear_performed();
                    }
                }
            }
            Ok((
                retained.prepare_grid_test_batch(batch),
                batch
                    .unrounded_inline_fragments()
                    .iter()
                    .filter(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(4))
                    .map(|entry| entry.fragment().clone())
                    .collect(),
            ))
        }
        fn commit_layout_batch<'call>(
            &mut self,
            commit: crate::LayoutBatchCommit<'call, u32, S, Self::Prepared>,
        ) {
            (self.retained, self.fragments) = commit.into_prepared();
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
                let capture = crate::LayoutFlowInput::try_new(
                    &tree,
                    2,
                    vec![tree.text.binding(3)],
                    crate::WrappingInput {
                        native_inline_boxes: Vec::new(),
                        gaps: Vec::new(),
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                )
                .unwrap();
                let reference =
                    compute_layout_with_flow(&capture, 2, GridGutterInlineTree::<S>::request(80.0))
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
                let seed = compute_layout_with_flow(
                    &capture,
                    2,
                    GridGutterInlineTree::<S>::request(seed_width),
                )
                .unwrap();
                drop(capture);
                seed.apply_to(&mut tree).unwrap();
                tree.cache_reads.borrow_mut().clear();
                let capture = crate::LayoutFlowInput::try_new(
                    &tree,
                    2,
                    vec![tree.text.binding(3)],
                    crate::WrappingInput {
                        native_inline_boxes: Vec::new(),
                        gaps: Vec::new(),
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                )
                .unwrap();
                let mut session = ComputeSession::with_flow(&capture, capture.invalidated.clone());
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
                        crate::SizingRequestOf::new(
                            Size::new(
                                crate::AxisSizingRequestOf::new(
                                    crate::AvailableSpaceOf::Indefinite,
                                    crate::SizingOperation::MaxContent,
                                ),
                                crate::AxisSizingRequestOf::new(
                                    crate::AvailableSpaceOf::Indefinite,
                                    crate::SizingOperation::Normal,
                                ),
                            ),
                            crate::OrthogonalFallbackOf::new(
                                crate::Size::NONE,
                                crate::NearestScrollportOf::Absent,
                                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                            ),
                        ),
                    ),
                )
                .unwrap();
                assert_eq!(
                    measured.size,
                    Size::new(S::from_f64(50.0), S::from_f64(20.0))
                );
                assert!(session.outputs.unrounded.is_empty());
                assert!(session.outputs.unrounded_fragments.is_empty());
                assert!(session.text.as_ref().unwrap().pending.is_empty());
                assert!(
                    session
                        .cache
                        .stores
                        .iter()
                        .all(|entry| !entry.input().run_mode().is_perform_layout())
                );
                drop(session);
                let batch =
                    compute_layout_with_flow(&capture, 2, GridGutterInlineTree::<S>::request(80.0))
                        .unwrap();
                assert_eq!(
                    readback::continuous_custody(&batch, Phase::Final),
                    readback::continuous_custody(&reference, Phase::Final)
                );
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
                let capture = crate::LayoutFlowInput::try_new(
                    &tree,
                    2,
                    vec![tree.text.binding(3)],
                    crate::WrappingInput {
                        native_inline_boxes: Vec::new(),
                        gaps: Vec::new(),
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                )
                .unwrap();
                let mut session = ComputeSession::with_flow(&capture, capture.invalidated.clone());
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
                        crate::SizingRequestOf::new(
                            Size::new(
                                crate::AxisSizingRequestOf::new(
                                    crate::AvailableSpaceOf::Indefinite,
                                    crate::SizingOperation::MaxContent,
                                ),
                                crate::AxisSizingRequestOf::new(
                                    crate::AvailableSpaceOf::Indefinite,
                                    crate::SizingOperation::Normal,
                                ),
                            ),
                            crate::OrthogonalFallbackOf::new(
                                crate::Size::NONE,
                                crate::NearestScrollportOf::Absent,
                                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                            ),
                        ),
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
                        .content_sizing_request()
                        .axes()
                        .width
                        .space()
                        .definite_size()
                        .map(NonNegativeFiniteOf::get),
                    Some(S::from_f64(45.0))
                );
                assert!(session.outputs.unrounded.is_empty());
                assert!(session.outputs.unrounded_fragments.is_empty());
                assert!(session.text.as_ref().unwrap().pending.is_empty());
                assert!(
                    session
                        .cache
                        .stores
                        .iter()
                        .all(|entry| !entry.input().run_mode().is_perform_layout())
                );
                assert_eq!((tree.retained.clone(), tree.fragments.clone()), before);
                drop(session);
                tree.fail_at_width.set(None);
                let retry = compute_layout_with_flow_invalidated(
                    &capture,
                    2,
                    GridGutterInlineTree::<S>::request(80.0),
                    &[6],
                )
                .unwrap();
                assert_eq!(
                    readback::continuous_custody(&retry, Phase::Final),
                    readback::continuous_custody(&reference, Phase::Final)
                );
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
        let capture = crate::LayoutFlowInput::try_new(
            &tree,
            0,
            vec![text.binding(1)],
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let mut session = ComputeSession::with_flow(&capture, capture.invalidated.clone());
        // Both real Flex dimensions are authored100x20. This is a normal
        // publication/replay request, not a formal intrinsic query.
        let request = crate::LayoutRootRequestOf::standalone(
            crate::SizingRequestOf::new(
                Size::splat(crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                )),
                crate::OrthogonalFallbackOf::new(
                    crate::Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            Size::NONE,
        );
        super::super::root::compute_root(&mut session, 0, request).unwrap();
        super::super::rounding::round_layout(&mut session, 0).unwrap();
        let surgeist_text::PlacementEntry::Formatted(initial) =
            session.text.as_ref().unwrap().pending[0].entries[0].clone()
        else {
            panic!("native translated line remains formatted");
        };
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
        session.begin_text_pass(0, frames).unwrap();
        super::super::root::compute_root(&mut session, 0, request).unwrap();
        super::super::rounding::round_layout(&mut session, 0).unwrap();
        let surgeist_text::PlacementEntry::Formatted(canonical) =
            &session.text.as_ref().unwrap().pending[0].entries[0]
        else {
            panic!("native translated line remains formatted");
        };
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

#[cfg(test)]
#[path = "semantic_size_tests.rs"]
mod semantic_size_tests;
