use super::source::{Inventory, WindowTree, unsupported};
use super::{BoundMode, FragmentBoxAllocation, NodePublication, ResultIdentity, SingleBox};
use crate::geometry::LogicalSizeOf;
use crate::output::{GeometryStateOf, NormalGeometryOf};
use crate::*;
use std::ops::Range;
use std::sync::Arc;

type NativeResult<Tree, T> = LayoutResultOf<
    <Tree as Traverse>::Node,
    T,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

type IntrinsicContext<Tree> = Arc<
    crate::block::intrinsic::FragmentContext<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
>;

struct NativeProvider<'a, 'tree, Tree: LayoutTree> {
    capture: &'a LayoutFlowInput<'tree, Tree>,
    inventory: &'a Inventory<Tree::Node>,
    frames: &'a [BlockFragmentainerOf<Tree::Scalar>],
    axes: FlowAxes,
    kind: FragmentationKind,
    target: LayoutTargetBasis,
    template: LayoutRootRequestOf<Tree::Scalar>,
    intrinsics: std::cell::RefCell<Vec<IntrinsicContext<Tree>>>,
}
impl<Tree: LayoutTree> NativeProvider<'_, '_, Tree> {
    fn failure(
        &self,
        failure: BlockFragmentationFailure,
    ) -> LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(self.capture.root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::BlockFragmentation(failure),
        )
    }
    fn request(
        &self,
        frame: BlockFragmentainerOf<Tree::Scalar>,
    ) -> LayoutRootRequestOf<Tree::Scalar> {
        let inline = self.axes.logical_size(frame.size).inline;
        let axes = self.axes.physical_size(LogicalSizeOf::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    NonNegativeFiniteOf::new(inline)
                        .expect("bound fragmentainer geometry is validated"),
                ),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ));
        let LayoutRootContextOf::Standalone {
            percentage_parent, ..
        } = self.template.context()
        else {
            unreachable!("native admission qualified the standalone root context")
        };
        let original = self.axes.logical_size(percentage_parent);
        let parent = self.axes.physical_size(LogicalSizeOf::new(
            Some(NonNegativeFiniteOf::new(inline).expect("bound inline constraint is validated")),
            original.block,
        ));
        LayoutRootRequestOf::standalone(
            SizingRequestOf::new(axes, self.template.sizing_request().orthogonal_fallback()),
            self.axes,
            parent,
        )
        .with_rounding_mode(self.template.rounding_mode())
    }
    fn intrinsics(
        &self,
        frame: BlockFragmentainerOf<Tree::Scalar>,
    ) -> NativeResult<Tree, IntrinsicContext<Tree>> {
        let mut request = self.request(frame);
        let fallback = request.sizing_request().orthogonal_fallback();
        let first = self.frames.first().ok_or_else(|| {
            self.failure(BlockFragmentationFailure::MissingFragmentainer { index: 0 })
        })?;
        let icb = first.size.map(|value| {
            Some(NonNegativeFiniteOf::new(value).expect("bound first fragmentainer is validated"))
        });
        request = LayoutRootRequestOf::standalone(
            request
                .sizing_request()
                .with_orthogonal_fallback(OrthogonalFallbackOf::new(
                    icb,
                    fallback.nearest_scrollport(),
                    fallback.containing_limits(),
                )),
            self.axes,
            match request.context() {
                LayoutRootContextOf::Standalone {
                    percentage_parent, ..
                } => percentage_parent,
                _ => unreachable!("native request is standalone"),
            },
        )
        .with_rounding_mode(request.rounding_mode());
        if let Some(context) = self
            .intrinsics
            .borrow()
            .iter()
            .find(|context| context.request == request)
        {
            return Ok(context.clone());
        }
        let context = Arc::new(crate::engine::prepare_block_fragment_intrinsics(
            self.capture.tree,
            self.capture.root,
            request,
            self.target.clone(),
            self.axes,
        )?);
        self.intrinsics.borrow_mut().push(context.clone());
        Ok(context)
    }

    fn trial(
        &self,
        range: Range<usize>,
        frame: usize,
        allocations: Vec<FragmentBoxAllocation<Tree::Node, Tree::Scalar>>,
        start_provenance: FragmentationStart,
    ) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
        let bounds = *self.frames.get(frame).ok_or_else(|| {
            self.failure(BlockFragmentationFailure::MissingFragmentainer { index: frame })
        })?;
        let forced = matches!(
            start_provenance,
            FragmentationStart::AfterBreak(FragmentationBreakReason::Forced { .. })
        );
        let window = WindowTree::new(
            self.capture.tree,
            self.inventory,
            range.clone(),
            self.axes,
            &[],
        );
        let produced = crate::engine::compute_block_fragment_frame(
            &window,
            self.capture.root,
            self.request(bounds),
            allocations.clone(),
            bounds,
            self.axes,
            self.intrinsics(bounds)?,
        )?;
        let mut trimmed = Vec::new();
        for (index, entry) in self.inventory.nodes.iter().enumerate() {
            if entry.atomic_owner.is_some() || entry.first >= range.end || entry.end <= range.start
            {
                continue;
            }
            if range.end < self.inventory.leaves.len() && (index == 0 || entry.end > range.end) {
                produced.margins.boundary(entry.node, false, &mut trimmed);
            }
            if range.start > 0 && !forced && (index == 0 || entry.first < range.start) {
                produced.margins.boundary(entry.node, true, &mut trimmed);
            }
        }
        trimmed.retain(|owner| {
            produced
                .batch
                .node_layout(owner.node)
                .and_then(NodeLayoutRefOf::single_box)
                .is_some_and(|entry| {
                    entry.unrounded_output().margin.at_physical_side(owner.side)
                        != Tree::Scalar::ZERO
                })
        });
        if trimmed.is_empty() {
            return Ok(produced.batch);
        }
        let window = WindowTree::new(
            self.capture.tree,
            self.inventory,
            range,
            self.axes,
            &trimmed,
        );
        crate::engine::compute_block_fragment_frame(
            &window,
            self.capture.root,
            self.request(bounds),
            allocations,
            bounds,
            self.axes,
            self.intrinsics(bounds)?,
        )
        .map(|produced| produced.batch)
    }

    fn root_packet(
        &self,
        batch: &CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>,
    ) -> NativeResult<Tree, NodeOutputOf<Tree::Scalar>> {
        batch
            .node_layout(self.capture.root)
            .and_then(NodeLayoutRefOf::single_box)
            .map(|value| value.unrounded_output())
            .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))
    }
    fn normal(
        entry: LayoutOutputEntryOf<Tree::Node, Tree::Scalar>,
    ) -> NormalGeometryOf<Tree::Scalar> {
        match entry.geometry_state() {
            GeometryStateOf::OrdinaryUnrounded { measured_normal } => measured_normal,
            GeometryStateOf::OrdinaryCanonical { normal }
            | GeometryStateOf::CanonicalAtomic { normal, .. } => normal,
        }
    }
    fn positions(
        &self,
        batch: &CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>,
    ) -> Vec<(Tree::Node, Point<Tree::Scalar>)> {
        let mut positions: Vec<(Tree::Node, Point<Tree::Scalar>)> = Vec::new();
        for entry in &self.inventory.nodes {
            if let Some(NodeLayoutRefOf::SingleBox(value)) = batch.node_layout(entry.node) {
                let normal = Self::normal(value.unrounded_entry());
                let parent = entry
                    .parent
                    .and_then(|index| {
                        positions
                            .iter()
                            .find(|(node, _)| *node == self.inventory.nodes[index].node)
                            .map(|(_, origin)| *origin)
                    })
                    .unwrap_or(Point::ZERO);
                positions.push((
                    entry.node,
                    Point::new(parent.x + normal.location.x, parent.y + normal.location.y),
                ));
            }
        }
        positions
    }
    fn extent(
        &self,
        batch: &CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>,
        frame: usize,
    ) -> NativeResult<Tree, Tree::Scalar> {
        let root = self.root_packet(batch)?;
        let mut extent = self.axes.logical_size(root.size).block;
        for (node, origin) in self.positions(batch) {
            let Some(NodeLayoutRefOf::SingleBox(value)) = batch.node_layout(node) else {
                continue;
            };
            let normal = Self::normal(value.unrounded_entry());
            let start = self
                .axes
                .logical_point(origin, normal.size, self.frames[frame].size)
                .block;
            extent = extent.max(start + self.axes.logical_size(normal.size).block);
        }
        if !extent.is_finite() {
            return Err(self.failure(BlockFragmentationFailure::InvalidGeometry));
        }
        Ok(extent.max(Tree::Scalar::ZERO))
    }
    fn accepted(
        &self,
        range: Range<usize>,
        frame: usize,
        start_provenance: FragmentationStart,
    ) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
        let natural = self.trial(range.clone(), frame, Vec::new(), start_provenance)?;
        let capacity = self
            .axes
            .logical_size(self.frames[frame].size)
            .block
            .max(Tree::Scalar::ONE);
        let mut allocations = Vec::new();
        for (node, origin) in self.positions(&natural) {
            let source = self
                .inventory
                .nodes
                .iter()
                .find(|entry| entry.node == node)
                .expect("accepted source participant belongs to the captured inventory");
            if source.atomic_owner.is_some() {
                continue;
            }
            let Some(NodeLayoutRefOf::SingleBox(value)) = natural.node_layout(node) else {
                continue;
            };
            let normal = Self::normal(value.unrounded_entry());
            let start = self
                .axes
                .logical_point(origin, normal.size, self.frames[frame].size)
                .block;
            let remaining = (capacity - start).max(Tree::Scalar::ZERO);
            let natural_extent = self.axes.logical_size(normal.size).block;
            if super::source::block_constrained(self.capture.tree.node_input(node), self.axes)
                && natural_extent > remaining
            {
                return Err(unsupported(node));
            }
            if source.end <= range.end {
                continue;
            }
            let used = natural_extent.max(remaining);
            let known = self
                .axes
                .physical_size(LogicalSizeOf::new(None, Some(used)));
            allocations.push(FragmentBoxAllocation {
                node,
                known,
                indefinite_axis: self.axes.block_axis(),
            });
        }
        if allocations.is_empty() {
            Ok(natural)
        } else {
            self.trial(range, frame, allocations, start_provenance)
        }
    }
}
impl<Tree: LayoutTree> FragmentationProvider<Tree::Scalar> for NativeProvider<'_, '_, Tree> {
    type Error = LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError>;
    fn fragmentainer(
        &mut self,
        _context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<Tree::Scalar>>, Self::Error> {
        Ok(self.frames.get(index).map(|frame| {
            let size = self.axes.logical_size(frame.size);
            FragmentainerOf {
                inline_size: size.inline,
                block_size: size.block,
                breaks: frame.breaks,
                parent_fragmentainer: None,
                page_side: frame.page_side,
            }
        }))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<Tree::Scalar>,
    ) -> Result<FragmentationReplyOf<Tree::Scalar>, Self::Error> {
        if query.position.offset != 0 || query.graphical_progress.is_some() {
            return Err(self.failure(BlockFragmentationFailure::InvalidSource));
        }
        let start = query.position.occurrence;
        let flow_end = self.inventory.leaves.len();
        let (before, after, ancestors) = self.inventory.controls(self.capture.tree, start);
        let has_controls = |before: &[FragmentationBreakControl],
                            after: &[FragmentationBreakControl]| {
            before
                .iter()
                .chain(after)
                .any(|control| control.value != FragmentationBreak::Auto)
        };
        if query.scan_after.is_none() && has_controls(&before, &after) {
            return Ok(FragmentationReplyOf::ControlAtStart(
                FragmentationBoundaryControls {
                    kind: FragmentationControlKind::Sibling,
                    before,
                    after,
                },
            ));
        }
        // Discover raw controls before trial-producing any source beyond them.
        // Scheduler policy decides applicability; a discovery retry remeasures
        // this same source origin through a later endpoint.
        let mut end = flow_end;
        for boundary in start + 1..flow_end {
            let position = FragmentationPosition {
                occurrence: boundary,
                offset: 0,
            };
            if query.scan_after.is_some_and(|scanned| position <= scanned) {
                continue;
            }
            let (before, after, _) = self.inventory.controls(self.capture.tree, boundary);
            if has_controls(&before, &after) {
                end = boundary;
                break;
            }
        }
        let mut opportunities = vec![FragmentationOpportunityOf {
            position: query.position,
            block_offset: Tree::Scalar::ZERO,
            kind: FragmentationOpportunityKindOf::Sibling,
            before,
            after,
            ancestors,
        }];
        let mut extent = Tree::Scalar::ZERO;
        let mut head = Tree::Scalar::ZERO;
        for boundary in start + 1..=end {
            let trial = self.trial(
                start..boundary,
                query.fragmentainer_index,
                Vec::new(),
                query.start_provenance,
            )?;
            let body = self.extent(&trial, query.fragmentainer_index)?;
            extent = extent.max(body);
            if boundary == start + 1 {
                head = body;
            }
            if boundary < flow_end {
                let (before, after, ancestors) =
                    self.inventory.controls(self.capture.tree, boundary);
                opportunities.push(FragmentationOpportunityOf {
                    position: FragmentationPosition {
                        occurrence: boundary,
                        offset: 0,
                    },
                    block_offset: body,
                    kind: FragmentationOpportunityKindOf::Sibling,
                    before,
                    after,
                    ancestors,
                });
            }
        }
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: if end == flow_end {
                FragmentationWindowEnd::FlowEnd
            } else {
                FragmentationWindowEnd::PotentialControl(FragmentationPosition {
                    occurrence: end,
                    offset: 0,
                })
            },
            block_size: extent,
            head: FragmentationMeasuredHeadOf {
                end: FragmentationPosition {
                    occurrence: start + 1,
                    offset: 0,
                },
                block_size: head,
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities,
            cloned_block_start: Tree::Scalar::ZERO,
            cloned_block_end: Tree::Scalar::ZERO,
            emergency_slice: None,
        }))
    }
}

pub(crate) fn compute<Tree: LayoutTree>(
    capture: &LayoutFlowInput<'_, Tree>,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
    let bound = capture
        .bound
        .as_ref()
        .expect("native dispatch requires its bound request");
    let BoundMode::Block { kind, axes, frames } = bound.mode() else {
        return Err(unsupported(root));
    };
    if !capture.paragraphs.is_empty()
        || !matches!(
            bound.root_request().context(),
            LayoutRootContextOf::Standalone { .. }
        )
        || bound
            .root_request()
            .sizing_request()
            .axes()
            .width
            .operation()
            != SizingOperation::Normal
        || bound
            .root_request()
            .sizing_request()
            .axes()
            .height
            .operation()
            != SizingOperation::Normal
    {
        return Err(unsupported(root));
    }
    let inventory = Inventory::new(capture.tree, &capture.graph, *axes)?;
    let input = FragmentationInputOf {
        contexts: vec![FragmentationContext {
            kind: *kind,
            parent: None,
            flow_axes: *axes,
        }],
        flows: vec![FragmentationFlowOf {
            context: 0,
            start_fragmentainer: 0,
            initial_block_offset: Tree::Scalar::ZERO,
            occurrences: inventory
                .leaves
                .iter()
                .map(|&index| FragmentationOccurrence {
                    key: inventory.nodes[index].node,
                    source_extent: 1,
                    monolithic: true,
                })
                .collect(),
        }],
    };
    let mut provider = NativeProvider {
        capture,
        inventory: &inventory,
        frames,
        axes: *axes,
        kind: *kind,
        target: bound.target_basis(),
        template: bound.root_request(),
        intrinsics: std::cell::RefCell::new(Vec::new()),
    };
    let report = resolve_fragmentation(&input, &mut provider).map_err(|error| match error {
        FragmentationError::Provider(error) => error,
        FragmentationError::MissingFragmentainer { index, .. } => {
            provider.failure(BlockFragmentationFailure::MissingFragmentainer { index })
        }
        FragmentationError::NoProgress { occurrence, .. } => {
            provider.failure(BlockFragmentationFailure::NoProgress { occurrence })
        }
        FragmentationError::Allocation => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::Allocation,
        ),
        _ => provider.failure(BlockFragmentationFailure::InvalidGeometry),
    })?;
    let mut groups: Vec<(usize, Range<usize>, FragmentationStart)> = Vec::new();
    for piece in report.pieces {
        let range = piece.start.occurrence..piece.end.occurrence;
        if let Some((frame, prior, _)) = groups.last_mut()
            && *frame == piece.fragmentainer
            && prior.end == range.start
        {
            prior.end = range.end;
        } else {
            groups.push((piece.fragmentainer, range, piece.start_provenance));
        }
    }
    for (_, range, _) in &groups {
        inventory.validate_continuation(capture.tree, range, *axes)?;
    }
    let mut accepted = Vec::new();
    for (frame, range, start_provenance) in groups {
        let batch = provider.accepted(range.clone(), frame, start_provenance)?;
        accepted.push(AcceptedFrame {
            frame,
            range,
            batch,
        });
    }
    let invalidated = capture.graph.invalidation(changed_nodes)?;
    assemble(&provider, accepted, invalidated)
}

struct AcceptedFrame<Node, S: LayoutScalar> {
    frame: usize,
    range: Range<usize>,
    batch: CompletedLayoutBatchOf<Node, S>,
}

struct Appearance<Node, S: LayoutScalar> {
    node: Node,
    frame: usize,
    range: Range<usize>,
    id: LayoutBoxId,
    unrounded: LayoutOutputEntryOf<Node, S>,
    final_entry: LayoutOutputEntryOf<Node, S>,
}
fn assemble<Tree: LayoutTree>(
    provider: &NativeProvider<'_, '_, Tree>,
    accepted: Vec<AcceptedFrame<Tree::Node, Tree::Scalar>>,
    invalidated: Vec<Tree::Node>,
) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
    let capture = provider.capture;
    let inventory = provider.inventory;
    let frames = provider.frames;
    let axes = provider.axes;
    let kind = provider.kind;
    let result = Arc::new(ResultIdentity);
    let mut appearances = Vec::new();
    let mut metadata = Vec::new();
    for AcceptedFrame {
        frame,
        range,
        batch,
    } in accepted
    {
        for entry in batch.nodes {
            match entry {
                NodePublication::Single(value) => {
                    let source = inventory
                        .nodes
                        .iter()
                        .find(|entry| entry.node == value.final_entry.node())
                        .expect("accepted box has captured source custody");
                    let mut unrounded = value.unrounded;
                    let mut final_entry = value.final_entry;
                    unrounded = unrounded.with_source_index(source.source_index);
                    final_entry = final_entry.with_source_index(source.source_index);
                    appearances.push(Appearance {
                        node: source.node,
                        frame,
                        range: range.clone(),
                        id: LayoutBoxId {
                            result: result.clone(),
                            index: appearances.len(),
                        },
                        unrounded,
                        final_entry,
                    });
                }
                NodePublication::NonBox {
                    kind,
                    mut unrounded,
                    mut final_entry,
                } => {
                    let source = inventory
                        .nodes
                        .iter()
                        .find(|source| source.node == unrounded.node())
                        .expect("accepted metadata has captured source custody");
                    unrounded = unrounded.with_source_index(source.source_index);
                    final_entry = final_entry.with_source_index(source.source_index);
                    metadata.push(NodePublication::NonBox {
                        kind,
                        unrounded,
                        final_entry,
                    });
                }
                NodePublication::Fragments { .. } => return Err(unsupported(capture.root)),
            }
        }
    }
    let mut nodes = Vec::new();
    for source in &inventory.nodes {
        let instances: Vec<_> = appearances
            .iter()
            .filter(|entry| entry.node == source.node)
            .collect();
        if instances.is_empty() {
            continue;
        }
        let parent = |entry: &Appearance<Tree::Node, Tree::Scalar>| {
            source
                .parent
                .and_then(|parent| {
                    appearances.iter().find(|candidate| {
                        candidate.frame == entry.frame
                            && candidate.node == inventory.nodes[parent].node
                    })
                })
                .map_or_else(
                    || {
                        BoxCoordinateSpace::Fragmentainer(LayoutFragmentainerId {
                            result: result.clone(),
                            index: entry.frame,
                        })
                    },
                    |parent| BoxCoordinateSpace::Box(parent.id.clone()),
                )
        };
        if instances.len() == 1 {
            let entry = instances[0];
            nodes.push(NodePublication::Single(SingleBox {
                id: entry.id.clone(),
                role: capture.tree.node_input(source.node).block_generation,
                parent: Some(parent(entry)),
                unrounded: entry.unrounded,
                final_entry: entry.final_entry,
            }));
        } else {
            let fragments = instances
                .iter()
                .enumerate()
                .map(|(order, entry)| BoxFragmentOutputOf {
                    id: entry.id.clone(),
                    node: source.node,
                    role: capture.tree.node_input(source.node).block_generation,
                    frame: LayoutFragmentainerId {
                        result: result.clone(),
                        index: entry.frame,
                    },
                    parent: parent(entry),
                    order,
                    previous: order
                        .checked_sub(1)
                        .map(|index| instances[index].id.clone()),
                    next: instances.get(order + 1).map(|next| next.id.clone()),
                    children: source
                        .children
                        .iter()
                        .filter(|&&child| {
                            inventory.nodes[child].first < entry.range.end
                                && inventory.nodes[child].end > entry.range.start
                        })
                        .map(|&child| {
                            (
                                inventory.nodes[child].source_index,
                                inventory.nodes[child].node,
                            )
                        })
                        .collect(),
                    graphical: None,
                    unrounded: BoxFragmentGeometryOf::from_packet(entry.unrounded.output()),
                    final_geometry: BoxFragmentGeometryOf::from_packet(entry.final_entry.output()),
                })
                .collect();
            nodes.push(NodePublication::Fragments {
                node: source.node,
                fragments,
            });
        }
    }
    nodes.extend(metadata);
    let mut batch = CompletedLayoutBatchOf::from_entries(
        capture.root,
        crate::output::CompletedBatchEntries {
            unrounded: Vec::new(),
            final_entries: Vec::new(),
            unrounded_inline: Vec::new(),
            final_inline: Vec::new(),
            stores: Vec::new(),
            clears: invalidated
                .iter()
                .copied()
                .map(crate::output::LayoutCacheClearEntry::new)
                .collect(),
            invalidated,
        },
    );
    batch.result = result.clone();
    batch.nodes = nodes;
    batch.target = Some(provider.target.clone());
    batch.fragmentainers = frames
        .iter()
        .enumerate()
        .map(|(index, &frame)| LayoutFragmentainerOutputOf {
            id: LayoutFragmentainerId {
                result: result.clone(),
                index,
            },
            kind,
            axes,
            frame,
        })
        .collect();
    Ok(batch)
}
