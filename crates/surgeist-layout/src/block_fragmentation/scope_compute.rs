//! Native scope preparation over scheduler-owned executions.
#[path = "scope_entry.rs"]
mod entry;
#[path = "scope_margins.rs"]
mod margins;
#[path = "scope_output.rs"]
mod output;
#[path = "scope_scene.rs"]
mod scene;
#[path = "scope_window.rs"]
mod window;
#[path = "scope_work.rs"]
mod work;
use super::super::scope_view::{ScopeView, TrimContextId, ViewKey, Views};
use super::super::scopes::{ScopeId, Scopes, Unit};
use super::super::source::{Inventory, SourceKey, unsupported};
use super::{IntrinsicContext, MinimumSnapshot, NativeProvider, NativeResult};
use crate::engine::block_scope::{
    ComputeResult, ComputeStop, Dispatch, Packet, ProductionId, Ready, ReceiptId, Requests,
    SourceViewId,
};
use crate::fragmentation::{
    CompletedExecution, Execution, ScopeEndpointOf, ScopeProjectionOf, ScopeTicket,
};
use crate::*;
use entry::{Production, ProductionInput, QualificationUse, QualifiedBody};
pub(super) use output::compute;
use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

type ScopeResult<Tree, T> = ComputeResult<
    <Tree as Traverse>::Node,
    T,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

#[derive(Clone)]
struct Selection<S: LayoutScalar> {
    range: Range<usize>,
    partial: Option<usize>,
    resumed: Option<usize>,
    provenance: FragmentationStart,
    begin: Option<FragmentationGraphicalProgressOf<S>>,
    cut: FragmentationCutOf<S>,
}
struct ContentFrame<S: LayoutScalar> {
    context: crate::block::content::Context<S>,
    facts: crate::block::content::Facts<S>,
}
struct Produced<Node, S: LayoutScalar> {
    packet: Packet<Node, S>,
    view: SourceViewId,
    input: ProductionInput<S>,
    content: Option<ContentFrame<S>>,
    reference: S,
}
impl<Node: Copy + Eq, S: LayoutScalar> Produced<Node, S> {
    fn output(&self, node: Node) -> Option<NodeOutputOf<S>> {
        self.packet
            .entries
            .iter()
            .find(|entry| entry.node() == node)
            .map(LayoutOutputEntryOf::output)
    }
}
struct AcceptedFrame<Node, S: LayoutScalar> {
    frame: usize,
    selection: Selection<S>,
    allocations: Vec<super::super::FragmentBoxAllocation<Node, S>>,
    summary: Option<crate::block::content::AcceptedContent<S>>,
    receipts: Vec<ReceiptId>,
    projections: Vec<(Node, ProductionId)>,
    production: ProductionId,
}
struct ActiveFrame<Node, S: LayoutScalar> {
    frame: usize,
    selection: Selection<S>,
    accounted: Vec<(Node, S)>,
}
struct Progress<Node, S: LayoutScalar> {
    minima: Vec<(Node, S)>,
    active: Option<ActiveFrame<Node, S>>,
    completed: Vec<AcceptedFrame<Node, S>>,
    tails: Vec<Option<S>>,
    opened: bool,
}
#[derive(Clone, Copy)]
struct Placement<Node, S: LayoutScalar> {
    query: FragmentationQueryOf<S>,
    entry: crate::block::scope::Placement<Node, S>,
    origin: crate::geometry::LogicalPointOf<S>,
    outside: S,
}
struct Work<Node, S: LayoutScalar> {
    id: ReceiptId,
    scope: ScopeId,
    execution: Execution<SourceKey<Node>, S>,
    placement: Option<Placement<Node, S>>,
    progress: Rc<RefCell<Progress<Node, S>>>,
    ready: Rc<RefCell<Vec<Ready<Node, S>>>>,
    bindings: Rc<RefCell<Vec<Binding>>>,
    discovery: DiscoveryState<Node, S>,
    rebind_generation: u64,
    query: Rc<Cell<Option<FragmentationQueryOf<S>>>>,
    trial_selection: Rc<RefCell<Option<Selection<S>>>>,
    context: Option<crate::block::content::Context<S>>,
    preparations: Vec<work::Preparation<Node, S>>,
    qualifications: QualificationUses<Node, S>,
}
struct CachedPrincipal<S: LayoutScalar> {
    key: ProjectionKey,
    input: ComputeInputOf<S>,
    inline_outer: S,
    output: ComputeOutputOf<S>,
    production: ProductionId,
    inherited_trims: TrimContextId,
}
struct Completed<Node, S: LayoutScalar> {
    scope: ScopeId,
    report: Arc<CompletedExecution<SourceKey<Node>, S>>,
    placement: Option<Placement<Node, S>>,
    frames: Vec<AcceptedFrame<Node, S>>,
    ready: Rc<RefCell<Vec<Ready<Node, S>>>>,
    timeline: Option<crate::block::content::PrincipalTimeline<S>>,
    principals: RefCell<Vec<CachedPrincipal<S>>>,
    bindings: Rc<RefCell<Vec<Binding>>>,
    overflow: RefCell<Vec<(AcceptedContentWindowId, ProductionId)>>,
}

impl<Node, S: LayoutScalar> Completed<Node, S> {
    fn window_id(&self, frame: usize) -> Option<AcceptedContentWindowId> {
        self.frames
            .iter()
            .position(|accepted| accepted.frame == frame)
            .map(AcceptedContentWindowId)
    }
    fn span_end(&self, span: &crate::block::content::PrincipalSpan<S>) -> S {
        let initial = self
            .placement
            .as_ref()
            .filter(|placement| placement.query.fragmentainer_index == span.frame);
        let origin = initial.map_or(S::ZERO, |placement| placement.origin.block);
        let occupied = initial.map_or(S::ZERO, |placement| placement.outside);
        occupied.max(origin + span.opening + span.content)
    }
}

#[derive(Clone, Copy)]
struct Binding {
    owner: usize,
    receipt: ReceiptId,
    frame: usize,
    initial_deferred: bool,
    projection: Option<ProjectionKey>,
}
/// Index into one completed child receipt's immutable accepted-frame arena.
/// Geometry roles do not decide whether this content was accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AcceptedContentWindowId(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProjectionKey {
    frame: usize,
    window: Option<AcceptedContentWindowId>,
    body: bool,
    closing: bool,
}

type DiscoveryState<Node, S> = Rc<RefCell<Option<window::Discovery<Node, S>>>>;
type QualificationArena<Node, S> = Rc<RefCell<Vec<QualifiedBody<Node, S>>>>;
type QualificationUses<Node, S> = Rc<RefCell<Vec<QualificationUse<Node, S>>>>;
type ProductionArena<Node, S> = Rc<RefCell<Vec<Production<Node, S>>>>;
type ScopeRequests<Node, S> = Rc<RefCell<Requests<crate::engine::block_scope::Request<Node, S>>>>;
type CompletedReceipt<Tree> = Arc<Completed<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>>;
type NormalBlockPositions<Tree> = Vec<(<Tree as Traverse>::Node, <Tree as Traverse>::Scalar)>;

type Records<Node, S> = Rc<RefCell<Vec<Option<Arc<Completed<Node, S>>>>>>;

struct Provider<'a, 'capture, 'tree, Tree: LayoutTree> {
    environment: &'a NativeProvider<'capture, 'tree, Tree>,
    scopes: &'a Scopes<Tree::Node>,
    scope: ScopeId,
    placement: Option<Placement<Tree::Node, Tree::Scalar>>,
    progress: Rc<RefCell<Progress<Tree::Node, Tree::Scalar>>>,
    dispatch: Dispatch<Tree::Node, Tree::Scalar>,
    query: Rc<Cell<Option<FragmentationQueryOf<Tree::Scalar>>>>,
    bindings: Rc<RefCell<Vec<Binding>>>,
    records: Records<Tree::Node, Tree::Scalar>,
    discovery: DiscoveryState<Tree::Node, Tree::Scalar>,
    rebind_generation: u64,
    trial_selection: Rc<RefCell<Option<Selection<Tree::Scalar>>>>,
    views: Rc<RefCell<Views>>,
    qualifications: QualificationArena<Tree::Node, Tree::Scalar>,
    qualification_uses: QualificationUses<Tree::Node, Tree::Scalar>,
    productions: ProductionArena<Tree::Node, Tree::Scalar>,
}
impl<Tree: LayoutTree> Provider<'_, '_, '_, Tree> {
    fn root(&self) -> Tree::Node {
        self.scopes.source.nodes[self.scopes.scopes[self.scope.0].owner].node
    }
    fn failure(
        &self,
        failure: BlockFragmentationFailure,
    ) -> LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        self.environment.failure(failure)
    }
    fn context(&self, frame: usize) -> NativeResult<Tree, IntrinsicContext<Tree>> {
        let frame = self.environment.frames.get(frame).copied().ok_or_else(|| {
            self.failure(BlockFragmentationFailure::MissingFragmentainer { index: frame })
        })?;
        self.environment.intrinsics(frame)
    }
    fn content_input(&self, frame: usize) -> NativeResult<Tree, ComputeInputOf<Tree::Scalar>> {
        let placement = self
            .placement
            .as_ref()
            .expect("content scope has an exact placement");
        if frame == placement.query.fragmentainer_index {
            return Ok(placement.entry.pending.input);
        }
        self.context(frame)?
            .entries
            .iter()
            .find(|entry| entry.node == self.root())
            .map(|entry| entry.input.with_run_mode(RunMode::PerformLayout))
            .ok_or_else(|| unsupported(self.root()))
    }
    fn view_id(&self, key: &ViewKey) -> ScopeResult<Tree, SourceViewId> {
        self.views.borrow_mut().issue(key.clone()).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(self.root()),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
            .into()
        })
    }
    fn trial(
        &self,
        selection: &Selection<Tree::Scalar>,
        frame: usize,
        allocations: Vec<super::super::FragmentBoxAllocation<Tree::Node, Tree::Scalar>>,
        reference: Tree::Scalar,
    ) -> ScopeResult<Tree, Produced<Tree::Node, Tree::Scalar>> {
        let produced = self.trial_with_trims(
            selection,
            frame,
            allocations.clone(),
            reference,
            TrimContextId::EMPTY,
        )?;
        let views = self.views.borrow();
        let view = views
            .get(produced.view)
            .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
        let trimmed = margins::trimmed::<_, _, Tree::MeasureError>(
            self.scopes,
            view,
            selection.provenance,
            &produced.packet,
            &self.productions.borrow(),
            &views,
        )?;
        drop(views);
        if trimmed.is_empty() {
            return Ok(produced);
        }
        let trimmed = self
            .views
            .borrow_mut()
            .extend_trims(TrimContextId::EMPTY, trimmed)
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(self.root()),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        self.trial_with_trims(selection, frame, allocations, reference, trimmed)
    }
    fn trial_with_trims(
        &self,
        selection: &Selection<Tree::Scalar>,
        frame: usize,
        allocations: Vec<super::super::FragmentBoxAllocation<Tree::Node, Tree::Scalar>>,
        reference: Tree::Scalar,
        trimmed: TrimContextId,
    ) -> ScopeResult<Tree, Produced<Tree::Node, Tree::Scalar>> {
        *self.trial_selection.borrow_mut() = Some(selection.clone());
        for binding in self.bindings.borrow_mut().iter_mut() {
            if binding.initial_deferred || binding.owner >= selection.range.end {
                continue;
            }
            let record = self
                .records
                .borrow()
                .get(binding.receipt.0)
                .and_then(Option::as_ref)
                .cloned()
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let timeline = record
                .timeline
                .as_ref()
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let key = ProjectionKey {
                frame,
                window: record.window_id(frame),
                body: selection.range.start <= binding.owner,
                closing: selection.range.end > binding.owner + 1
                    && selection.partial != Some(binding.owner)
                    && timeline
                        .spans
                        .last()
                        .is_some_and(|span| span.frame <= frame),
            };
            if binding.projection != Some(key) {
                // Another exact parent view may still need this immutable child
                // receipt. A geometry-role change does not invalidate that view;
                // a placement rebind discards every preparation instead.
                let node = self.scopes.source.nodes
                    [self.scopes.scopes[self.scope.0].units[binding.owner].node()]
                .node;
                self.dispatch
                    .ready
                    .borrow_mut()
                    .retain(|ready| ready.entry.node != node || ready.id == binding.receipt);
                binding.projection = Some(key);
            }
        }
        let bounds = *self.environment.frames.get(frame).ok_or_else(|| {
            self.failure(BlockFragmentationFailure::MissingFragmentainer { index: frame })
        })?;
        let opening = {
            let progress = self.progress.borrow();
            !progress.opened
                || !progress.completed.iter().any(|accepted| {
                    accepted.frame < frame
                        && accepted
                            .summary
                            .is_some_and(|summary| summary.participated || summary.complete)
                })
        };
        let view_key = ViewKey {
            scope: self.scope,
            frame,
            range: selection.range.clone(),
            partial: selection.partial,
            resumed: selection.resumed,
            opening,
            closing: None,
            trimmed,
        };
        let mut dispatch = self.dispatch.clone();
        dispatch.view = self.view_id(&view_key)?;
        let performed_view = dispatch.view;
        let view = ScopeView::new(
            self.environment.capture.tree,
            self.scopes,
            &view_key,
            self.environment.axes,
            &margins::owners(
                self.scopes,
                &self.views.borrow().trim_edges(view_key.trimmed),
            ),
        );
        let intrinsics = self.context(frame)?;
        let root = self.root();
        if self.scopes.scopes[self.scope.0].contents {
            let input = self.content_input(frame)?;
            let original = self.environment.capture.tree.node_input(root);
            let output = crate::engine::compute_block_content_frame(
                &view,
                crate::engine::BlockContentFrameRequest {
                    root,
                    input,
                    original,
                    reference,
                    opening,
                    complete: selection.range.end == self.scopes.scopes[self.scope.0].units.len()
                        && selection.partial.is_none(),
                    principal_content: None,
                    accepted: None,
                    carrier: None,
                    binding: crate::engine::BlockFrameBinding {
                        allocations,
                        intrinsics,
                        dispatch,
                    },
                },
            )?;
            Ok(Produced {
                packet: output.packet,
                view: performed_view,
                input: ProductionInput::Content(input),
                content: Some(ContentFrame {
                    context: output.context,
                    facts: output.facts,
                }),
                reference,
            })
        } else {
            let packet = crate::engine::compute_block_scope_root_frame(
                &view,
                crate::engine::BlockScopeRootFrameRequest {
                    root,
                    request: self.environment.request(bounds),
                    frame: bounds,
                    axes: self.environment.axes,
                    binding: crate::engine::BlockFrameBinding {
                        allocations,
                        intrinsics,
                        dispatch,
                    },
                },
            )?;
            Ok(Produced {
                packet,
                view: performed_view,
                input: ProductionInput::Root(self.environment.request(bounds)),
                content: None,
                reference,
            })
        }
    }
}

impl<Tree: LayoutTree> Provider<'_, '_, '_, Tree> {
    fn minimum_snapshot(&self, frame: usize) -> ScopeResult<Tree, MinimumSnapshot<Tree>> {
        let context = self.context(frame)?;
        let progress = self.progress.borrow();
        progress
            .minima
            .iter()
            .map(|&(node, spent)| {
                let minimum = context
                    .entries
                    .iter()
                    .find(|entry| entry.node == node)
                    .and_then(|entry| entry.minimum)
                    .ok_or_else(|| unsupported(node))?;
                let accounted = progress
                    .active
                    .as_ref()
                    .filter(|active| active.frame == frame)
                    .and_then(|active| active.accounted.iter().find(|(key, _)| *key == node))
                    .map_or(Tree::Scalar::ZERO, |(_, value)| *value);
                Ok((
                    node,
                    (minimum.content - (spent - accounted).max(Tree::Scalar::ZERO))
                        .max(Tree::Scalar::ZERO),
                ))
            })
            .collect()
    }
    fn allocations(
        &self,
        minima: &[(Tree::Node, Tree::Scalar)],
    ) -> Vec<super::super::FragmentBoxAllocation<Tree::Node, Tree::Scalar>> {
        minima
            .iter()
            .filter(|(node, _)| *node != self.root() || !self.scopes.scopes[self.scope.0].contents)
            .map(|&(node, _)| super::super::FragmentBoxAllocation {
                node,
                known: Size::NONE,
                indefinite_axis: self.environment.axes.block_axis(),
                minimum_content: Some(Tree::Scalar::ZERO),
            })
            .collect()
    }
    fn content_extent(
        &self,
        produced: &Produced<Tree::Node, Tree::Scalar>,
        node: Tree::Node,
    ) -> ScopeResult<Tree, Option<Tree::Scalar>> {
        if node == self.root()
            && let Some(content) = &produced.content
        {
            return Ok(Some(content.facts.block_extent));
        }
        produced
            .output(node)
            .map(|output| {
                output
                    .scroll_geometry
                    .map(|geometry| {
                        self.environment
                            .axes
                            .logical_size(geometry.content_box().size())
                            .block
                    })
                    .ok_or_else(|| {
                        ComputeStop::Layout(
                            self.failure(BlockFragmentationFailure::InvalidGeometry),
                        )
                    })
            })
            .transpose()
    }
    fn measure(
        &self,
        selection: &Selection<Tree::Scalar>,
        frame: usize,
        minima: &[(Tree::Node, Tree::Scalar)],
    ) -> ScopeResult<Tree, Produced<Tree::Node, Tree::Scalar>> {
        let mut allocations = self.allocations(minima);
        let mut output = self.trial(selection, frame, allocations.clone(), Tree::Scalar::ZERO)?;
        for (index, source) in self.scopes.source.nodes.iter().enumerate().rev() {
            let Some(local) = self.scopes.range(self.scope, index) else {
                continue;
            };
            let Some(&(_, minimum)) = minima.iter().find(|(node, _)| *node == source.node) else {
                continue;
            };
            if source.node == self.root() && output.content.is_some()
                || local.end > selection.range.end
                || selection.partial.is_some_and(|unit| local.end == unit + 1)
            {
                continue;
            }
            let Some(packet) = output.output(source.node) else {
                continue;
            };
            let content = self
                .content_extent(&output, source.node)?
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidGeometry))?;
            if minimum <= content {
                continue;
            }
            let outer = self.environment.axes.logical_size(packet.size).block;
            allocations
                .iter_mut()
                .find(|allocation| allocation.node == source.node)
                .expect("minimum source is allocated")
                .known = self
                .environment
                .axes
                .physical_size(crate::geometry::LogicalSizeOf::new(
                    None,
                    Some(outer - content + minimum),
                ));
            output = self.trial(selection, frame, allocations.clone(), Tree::Scalar::ZERO)?;
        }
        Ok(output)
    }
    fn extent(
        &self,
        produced: &Produced<Tree::Node, Tree::Scalar>,
        selection: &Selection<Tree::Scalar>,
        frame: usize,
        minima: &[(Tree::Node, Tree::Scalar)],
    ) -> ScopeResult<Tree, Tree::Scalar> {
        let axes = self.environment.axes;
        let root = self.root();
        let value = if let Some(content) = &produced.content {
            let scope = &self.scopes.scopes[self.scope.0];
            let closed = selection.range.end == scope.units.len() && selection.partial.is_none();
            let minimum = if closed {
                minima
                    .iter()
                    .find(|(node, _)| *node == root)
                    .map_or(Tree::Scalar::ZERO, |(_, value)| *value)
            } else {
                Tree::Scalar::ZERO
            };
            let opening = if content.facts.participated || closed {
                content.context.opening()
            } else {
                Tree::Scalar::ZERO
            };
            let origin = self
                .placement
                .as_ref()
                .filter(|placement| placement.query.fragmentainer_index == frame)
                .map_or(Tree::Scalar::ZERO, |placement| placement.origin.block);
            origin + (opening + content.facts.block_extent.max(minimum)).max(content.facts.occupied)
        } else {
            let mut positions = Vec::new();
            let mut peak = Tree::Scalar::ZERO;
            for source in &self.scopes.source.nodes {
                let Some(entry) = produced
                    .packet
                    .entries
                    .iter()
                    .find(|entry| entry.node() == source.node)
                else {
                    continue;
                };
                let normal = match entry.geometry_state() {
                    crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } => {
                        measured_normal
                    }
                    crate::output::GeometryStateOf::OrdinaryCanonical { normal }
                    | crate::output::GeometryStateOf::CanonicalAtomic { normal, .. } => normal,
                };
                let parent_origin = source
                    .parent
                    .and_then(|parent| {
                        positions
                            .iter()
                            .find(|(node, _)| *node == self.scopes.source.nodes[parent].node)
                    })
                    .map_or(Point::ZERO, |(_, origin)| *origin);
                let origin = Point::new(
                    parent_origin.x + normal.location.x,
                    parent_origin.y + normal.location.y,
                );
                let block = axes
                    .logical_point(origin, normal.size, self.environment.frames[frame].size)
                    .block;
                peak = peak.max(block + axes.logical_size(normal.size).block);
                positions.push((source.node, origin));
            }
            peak
        };
        if !value.is_finite() {
            return Err(self
                .failure(BlockFragmentationFailure::InvalidGeometry)
                .into());
        }
        Ok(value.max(Tree::Scalar::ZERO))
    }
}

impl<Tree: LayoutTree> Provider<'_, '_, '_, Tree> {
    fn positions(
        &self,
        produced: &Produced<Tree::Node, Tree::Scalar>,
        frame: usize,
    ) -> ScopeResult<Tree, NormalBlockPositions<Tree>> {
        let mut origins: Vec<(Tree::Node, Point<Tree::Scalar>)> = Vec::new();
        let mut result = Vec::new();
        let axes = self.environment.axes;
        let reference =
            produced
                .content
                .as_ref()
                .map_or(self.environment.frames[frame].size, |content| {
                    content
                        .context
                        .axes()
                        .physical_size(crate::geometry::LogicalSizeOf::new(
                            content.context.inline_outer,
                            produced.reference,
                        ))
                });
        let base = if produced.content.is_some() {
            self.placement
                .as_ref()
                .filter(|placement| placement.query.fragmentainer_index == frame)
                .map_or(Tree::Scalar::ZERO, |placement| placement.origin.block)
        } else {
            Tree::Scalar::ZERO
        };
        origins
            .try_reserve(produced.packet.entries.len())
            .and_then(|_| result.try_reserve(produced.packet.entries.len()))
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(self.root()),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        for source in &self.scopes.source.nodes {
            let Some(entry) = produced
                .packet
                .entries
                .iter()
                .find(|entry| entry.node() == source.node)
            else {
                continue;
            };
            if entry.non_box.is_some() {
                continue;
            }
            let normal = super::NativeProvider::<Tree>::normal(*entry);
            let parent = source
                .parent
                .and_then(|parent| {
                    origins
                        .iter()
                        .find(|(node, _)| *node == self.scopes.source.nodes[parent].node)
                })
                .map_or(Point::ZERO, |(_, origin)| *origin);
            let origin = Point::new(parent.x + normal.location.x, parent.y + normal.location.y);
            let block = base + axes.logical_point(origin, normal.size, reference).block;
            if !block.is_finite() {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidGeometry)
                    .into());
            }
            origins.push((source.node, origin));
            result.push((source.node, block));
        }
        Ok(result)
    }
}

impl<Node: Copy + Eq, S: LayoutScalar> Work<Node, S> {
    fn new<M>(
        id: ReceiptId,
        scope: ScopeId,
        scopes: &Scopes<Node>,
        context: FragmentationContext,
        placement: Option<Placement<Node, S>>,
        minimum_nodes: impl IntoIterator<Item = Node>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let root = scopes.source.nodes[scopes.scopes[scope.0].owner].node;
        let input = Arc::new(FragmentationInputOf {
            contexts: vec![context],
            flows: vec![FragmentationFlowOf {
                context: 0,
                start_fragmentainer: placement
                    .as_ref()
                    .map_or(0, |placement| placement.query.fragmentainer_index),
                initial_block_offset: placement
                    .as_ref()
                    .map_or(S::ZERO, |placement| placement.outside),
                occurrences: scopes.occurrences(scope),
            }],
        });
        let execution = Execution::new(input).map_err(|_| unsupported(root))?;
        Ok(Self {
            id,
            scope,
            execution,
            placement,
            progress: Rc::new(RefCell::new(Progress {
                minima: minimum_nodes
                    .into_iter()
                    .map(|node| (node, S::ZERO))
                    .collect(),
                active: None,
                completed: Vec::new(),
                tails: vec![None; scopes.scopes[scope.0].units.len()],
                opened: false,
            })),
            ready: Rc::new(RefCell::new(Vec::new())),
            bindings: Rc::new(RefCell::new(Vec::new())),
            discovery: Rc::new(RefCell::new(None)),
            rebind_generation: 0,
            query: Rc::new(Cell::new(None)),
            trial_selection: Rc::new(RefCell::new(None)),
            context: None,
            preparations: Vec::new(),
            qualifications: Rc::new(RefCell::new(Vec::new())),
        })
    }
}

impl<'a, 'capture, 'tree, Tree: LayoutTree> Provider<'a, 'capture, 'tree, Tree> {
    fn observer(&self) -> Self {
        Self {
            environment: self.environment,
            scopes: self.scopes,
            scope: self.scope,
            placement: self.placement,
            progress: self.progress.clone(),
            dispatch: self.dispatch.clone(),
            query: self.query.clone(),
            bindings: self.bindings.clone(),
            records: self.records.clone(),
            discovery: self.discovery.clone(),
            rebind_generation: self.rebind_generation,
            trial_selection: self.trial_selection.clone(),
            views: self.views.clone(),
            qualifications: self.qualifications.clone(),
            qualification_uses: self.qualification_uses.clone(),
            productions: self.productions.clone(),
        }
    }
}

/// All work owns its execution and unpublished mode state. A suspension moves
/// its request into this driver; no producer borrow crosses a child execution.
struct Driver<'a, 'capture, 'tree, Tree: LayoutTree> {
    environment: &'a NativeProvider<'capture, 'tree, Tree>,
    scopes: &'a Scopes<Tree::Node>,
    requests: ScopeRequests<Tree::Node, Tree::Scalar>,
    records: Records<Tree::Node, Tree::Scalar>,
    owners: Arc<[Tree::Node]>,
    paths: Arc<[Tree::Node]>,
    stack: Vec<Work<Tree::Node, Tree::Scalar>>,
    views: Rc<RefCell<Views>>,
    qualifications: QualificationArena<Tree::Node, Tree::Scalar>,
    productions: ProductionArena<Tree::Node, Tree::Scalar>,
}

impl<'a, 'capture, 'tree, Tree: LayoutTree> Driver<'a, 'capture, 'tree, Tree> {
    fn new(
        environment: &'a NativeProvider<'capture, 'tree, Tree>,
        scopes: &'a Scopes<Tree::Node>,
    ) -> NativeResult<Tree, Self> {
        scopes
            .validate()
            .map_err(|_| unsupported(environment.capture.root))?;
        let first = environment.frames.first().copied().ok_or_else(|| {
            environment.failure(BlockFragmentationFailure::MissingFragmentainer { index: 0 })
        })?;
        let intrinsics = environment.intrinsics(first)?;
        let mut owners = Vec::new();
        let mut paths = Vec::new();
        for (index, source) in scopes.source.nodes.iter().enumerate() {
            if scopes.owned[index].is_none() {
                continue;
            }
            owners.push(source.node);
            let mut parent = source.parent;
            while let Some(index) = parent {
                let source = &scopes.source.nodes[index];
                if !paths.contains(&source.node) {
                    paths.push(source.node);
                }
                parent = source.parent;
            }
        }
        let context = FragmentationContext {
            kind: environment.kind,
            parent: None,
            flow_axes: environment.axes,
        };
        let minimum_nodes = minimum_nodes(environment, scopes, ScopeId(0), &intrinsics)?;
        let work = Work::new(
            ReceiptId(0),
            ScopeId(0),
            scopes,
            context,
            None,
            minimum_nodes,
        )?;
        Ok(Self {
            environment,
            scopes,
            requests: Rc::new(RefCell::new(Requests::new())),
            records: Rc::new(RefCell::new(vec![None])),
            owners: Arc::from(owners),
            paths: Arc::from(paths),
            stack: vec![work],
            views: Rc::new(RefCell::new(Views::new())),
            qualifications: Rc::new(RefCell::new(Vec::new())),
            productions: Rc::new(RefCell::new(Vec::new())),
        })
    }
    fn dispatch(
        &self,
        work: &Work<Tree::Node, Tree::Scalar>,
    ) -> Dispatch<Tree::Node, Tree::Scalar> {
        Dispatch {
            scope: work.id,
            view: SourceViewId(0),
            target: self.environment.target.clone(),
            owners: self.owners.clone(),
            paths: self.paths.clone(),
            ready: work.ready.clone(),
            requests: self.requests.clone(),
        }
    }
    fn provider(
        &self,
        work: &Work<Tree::Node, Tree::Scalar>,
    ) -> Provider<'a, 'capture, 'tree, Tree> {
        Provider {
            environment: self.environment,
            scopes: self.scopes,
            scope: work.scope,
            placement: work.placement,
            progress: work.progress.clone(),
            dispatch: self.dispatch(work),
            query: work.query.clone(),
            bindings: work.bindings.clone(),
            records: self.records.clone(),
            discovery: work.discovery.clone(),
            rebind_generation: work.rebind_generation,
            trial_selection: work.trial_selection.clone(),
            views: self.views.clone(),
            qualifications: self.qualifications.clone(),
            qualification_uses: work.qualifications.clone(),
            productions: self.productions.clone(),
        }
    }
    fn map_error(
        &self,
        error: FragmentationError<ComputeStop<Tree::Node, Tree::Scalar, Tree::MeasureError>>,
    ) -> ComputeStop<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        match error {
            FragmentationError::Provider(stop) => stop,
            FragmentationError::MissingFragmentainer { index, .. } => self
                .environment
                .failure(BlockFragmentationFailure::MissingFragmentainer { index })
                .into(),
            FragmentationError::NoProgress { occurrence, .. } => self
                .environment
                .failure(BlockFragmentationFailure::NoProgress { occurrence })
                .into(),
            FragmentationError::Allocation => LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(self.environment.capture.root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
            .into(),
            _ => self
                .environment
                .failure(BlockFragmentationFailure::InvalidGeometry)
                .into(),
        }
    }
}

impl<Tree: LayoutTree> Driver<'_, '_, '_, Tree> {
    fn principal(
        &self,
        completed: &Completed<Tree::Node, Tree::Scalar>,
        receipt: ReceiptId,
        pending: crate::block::scope::Pending<Tree::Node, Tree::Scalar>,
        key: ProjectionKey,
        parent_view: SourceViewId,
    ) -> ScopeResult<Tree, Ready<Tree::Node, Tree::Scalar>> {
        let inherited_trims = self
            .views
            .borrow()
            .get(parent_view)
            .map(|view| view.trimmed)
            .ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
        if let Some(cached) = completed.principals.borrow().iter().find(|cached| {
            cached.key == key
                && cached.inherited_trims == inherited_trims
                && cached.input == pending.input
                && cached.inline_outer == pending.inline_outer
        }) {
            // Parent placement is separate from this unchanged producer input.
            // The exact replay request receives an independently issued binding.
            return Ok(Ready {
                id: receipt,
                view: parent_view,
                projection: Some(cached.production),
                entry: pending,
                output: cached.output,
            });
        }
        let timeline = completed.timeline.as_ref().ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::InvalidSource)
        })?;
        let span = timeline.spans.iter().find(|span| span.frame == key.frame);
        let source_scope = &self.scopes.scopes[completed.scope.0];
        let root = self.scopes.source.nodes[source_scope.owner].node;
        if root != pending.node {
            return Err(self
                .environment
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let accepted = key.window.and_then(|id| completed.frames.get(id.0));
        if accepted.is_some_and(|accepted| accepted.frame != key.frame) {
            return Err(self
                .environment
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let selection = if key.window.is_some() {
            accepted
                .map(|frame| frame.selection.clone())
                .ok_or_else(|| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?
        } else {
            Selection {
                range: source_scope.units.len()..source_scope.units.len(),
                partial: None,
                resumed: None,
                provenance: FragmentationStart::InFrame,
                begin: None,
                cut: FragmentationCutOf::Source(FragmentationPosition {
                    occurrence: source_scope.units.len(),
                    offset: 0,
                }),
            }
        };
        let opening = key.body
            && timeline
                .spans
                .first()
                .is_some_and(|span| span.frame == key.frame);
        let body = if key.body {
            span.map_or(Tree::Scalar::ZERO, |span| span.content)
        } else {
            Tree::Scalar::ZERO
        };
        let mut trimmed = inherited_trims;
        let (produced, view_id) = loop {
            let view_key = ViewKey {
                scope: completed.scope,
                frame: key.frame,
                range: selection.range.clone(),
                partial: selection.partial,
                resumed: selection.resumed,
                opening,
                closing: Some(key.closing),
                trimmed,
            };
            let view = ScopeView::new(
                self.environment.capture.tree,
                self.scopes,
                &view_key,
                self.environment.axes,
                &margins::owners(
                    self.scopes,
                    &self.views.borrow().trim_edges(view_key.trimmed),
                ),
            );
            let frame = *self.environment.frames.get(key.frame).ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::MissingFragmentainer { index: key.frame })
            })?;
            let view_id = self.views.borrow_mut().issue(view_key).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(root),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            let dispatch = Dispatch {
                scope: receipt,
                view: view_id,
                target: self.environment.target.clone(),
                owners: self.owners.clone(),
                paths: self.paths.clone(),
                ready: completed.ready.clone(),
                requests: self.requests.clone(),
            };
            let intrinsics = self.environment.intrinsics(frame)?;
            let (_, context) = crate::engine::qualify_block_scope_entry(
                self.environment.capture.tree,
                crate::engine::BlockScopeEntryRequest {
                    pending,
                    binding: crate::engine::BlockFrameBinding {
                        allocations: Vec::new(),
                        intrinsics: intrinsics.clone(),
                        dispatch: dispatch.clone(),
                    },
                },
            )?;
            let reference =
                body + if opening {
                    context.opening()
                } else {
                    Tree::Scalar::ZERO
                } + if key.closing {
                    context.original_closing()
                } else {
                    Tree::Scalar::ZERO
                };
            let allocation = accepted
                .map(|frame| frame.allocations.clone())
                .unwrap_or_default();
            let produced = crate::engine::compute_block_content_frame(
                &view,
                crate::engine::BlockContentFrameRequest {
                    root,
                    input: pending.input,
                    original: self.environment.capture.tree.node_input(root),
                    reference,
                    opening,
                    complete: selection.range.end == source_scope.units.len()
                        && selection.partial.is_none(),
                    principal_content: Some(body),
                    accepted: None,
                    carrier: None,
                    binding: crate::engine::BlockFrameBinding {
                        allocations: allocation,
                        intrinsics,
                        dispatch,
                    },
                },
            )?;
            let views = self.views.borrow();
            let performed = views.get(view_id).ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
            let own_trims = margins::trimmed::<_, _, Tree::MeasureError>(
                self.scopes,
                performed,
                selection.provenance,
                &produced.packet,
                &self.productions.borrow(),
                &views,
            )?;
            drop(views);
            let next = self
                .views
                .borrow_mut()
                .extend_trims(trimmed, own_trims)
                .map_err(|_| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(root),
                        LayoutOperation::RootLayout,
                        LayoutErrorKindOf::Allocation,
                    )
                })?;
            if next != trimmed {
                trimmed = next;
                continue;
            }
            break (produced, view_id);
        };
        let output = produced
            .principal
            .expect("a principal request runs the Block materializer");
        let production = entry::retain_production::<_, _, Tree::MeasureError>(
            &self.productions,
            entry::Production {
                receipt,
                scope: completed.scope,
                view: view_id,
                projection: Some((key, pending)),
                input: ProductionInput::Content(pending.input),
                carrier: None,
                packet: produced.packet,
            },
            root,
        )?;
        let mut cache = completed.principals.borrow_mut();
        cache.try_reserve(1).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        cache.push(CachedPrincipal {
            key,
            input: pending.input,
            inline_outer: pending.inline_outer,
            output,
            production,
            inherited_trims,
        });
        Ok(Ready {
            id: receipt,
            view: parent_view,
            projection: Some(production),
            entry: pending,
            output,
        })
    }
}

fn minimum_nodes<Tree: LayoutTree>(
    environment: &NativeProvider<'_, '_, Tree>,
    scopes: &Scopes<Tree::Node>,
    scope: ScopeId,
    context: &crate::block::intrinsic::FragmentContext<Tree::Node, Tree::Scalar>,
) -> NativeResult<Tree, Vec<Tree::Node>> {
    let mut nodes = Vec::new();
    for entry in &context.entries {
        if entry.minimum.is_none() {
            continue;
        }
        let index = scopes
            .source
            .nodes
            .iter()
            .position(|source| source.node == entry.node)
            .expect("intrinsic source belongs to captured topology");
        if scopes.source.nodes[index].atomic_owner.is_some() || scopes.range(scope, index).is_none()
        {
            continue;
        }
        let profile = match scopes.owned[index] {
            Some(owner) if owner == scope => super::super::source::ContinuationProfile::Maximum,
            Some(_) => continue,
            None => super::super::source::ContinuationProfile::Minimum,
        };
        if super::super::source::unsupported_continuation(
            environment.capture.tree.node_input(entry.node),
            environment.axes,
            profile,
        )
        .map_err(|error| crate::error::sizing_resolution_error(entry.node, error))?
        {
            continue;
        }
        nodes.try_reserve(1).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(entry.node),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        nodes.push(entry.node);
    }
    Ok(nodes)
}

#[cfg(test)]
#[path = "scope_zero_progress_tests.rs"]
mod zero_progress_tests;

#[cfg(test)]
#[path = "scope_deferred_progress_tests.rs"]
mod deferred_progress_tests;
