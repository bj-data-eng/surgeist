use super::source::{Inventory, SourceKey, SourceUnit, WindowTree, unsupported};
use super::{BoundMode, FragmentBoxAllocation, NodePublication, ResultIdentity, SingleBox};
use crate::geometry::LogicalSizeOf;
use crate::output::{GeometryStateOf, NormalGeometryOf};
use crate::*;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

type NativeResult<Tree, T> = LayoutResultOf<
    <Tree as Traverse>::Node,
    T,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

type MinimumSnapshot<Tree> = Vec<(<Tree as Traverse>::Node, <Tree as Traverse>::Scalar)>;

type IntrinsicContext<Tree> = Arc<
    crate::block::intrinsic::FragmentContext<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
>;

#[derive(Clone)]
struct Selection {
    range: Range<usize>,
    partial: Option<usize>,
    provenance: FragmentationStart,
}
struct Minimum<Node, S: LayoutScalar> {
    node: Node,
    original: crate::block::fragment_minimum::Receipt<S>,
    spent: S,
    continued: bool,
}
struct ActiveFrame<Node, S: LayoutScalar> {
    index: usize,
    selection: Selection,
    accounted: Vec<(Node, S)>,
}
struct Progress<Node, S: LayoutScalar> {
    minima: Vec<Minimum<Node, S>>,
    active: Option<ActiveFrame<Node, S>>,
    completed: Vec<AcceptedFrame<Node, S>>,
    tail_extents: Vec<Option<S>>,
}
impl<Node, S: LayoutScalar> Default for Progress<Node, S> {
    fn default() -> Self {
        Self {
            minima: Vec::new(),
            active: None,
            completed: Vec::new(),
            tail_extents: Vec::new(),
        }
    }
}

struct NativeProvider<'a, 'tree, Tree: LayoutTree> {
    capture: &'a LayoutFlowInput<'tree, Tree>,
    inventory: &'a Inventory<Tree::Node>,
    frames: &'a [BlockFragmentainerOf<Tree::Scalar>],
    axes: FlowAxes,
    kind: FragmentationKind,
    target: LayoutTargetBasis,
    template: LayoutRootRequestOf<Tree::Scalar>,
    intrinsics: Rc<RefCell<Vec<IntrinsicContext<Tree>>>>,
    progress: Rc<RefCell<Progress<Tree::Node, Tree::Scalar>>>,
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
        partial: Option<usize>,
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
            partial,
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
            if (range.end < self.inventory.units.len() || partial.is_some())
                && (index == 0
                    || entry.end > range.end
                    || partial.is_some_and(|unit| entry.end == unit + 1))
            {
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
            partial,
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
    fn observer(&self) -> Self {
        Self {
            capture: self.capture,
            inventory: self.inventory,
            frames: self.frames,
            axes: self.axes,
            kind: self.kind,
            target: self.target.clone(),
            template: self.template,
            intrinsics: self.intrinsics.clone(),
            progress: self.progress.clone(),
        }
    }
    fn content_extent(
        &self,
        packet: NodeOutputOf<Tree::Scalar>,
    ) -> NativeResult<Tree, Tree::Scalar> {
        packet
            .scroll_geometry
            .map(|geometry| self.axes.logical_size(geometry.content_box().size()).block)
            .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidGeometry))
    }
    fn minimum_snapshot(&self, frame: usize) -> NativeResult<Tree, MinimumSnapshot<Tree>> {
        let bounds = self.frames[frame];
        let context = self.intrinsics(bounds)?;
        let progress = self.progress.borrow();
        progress
            .minima
            .iter()
            .map(|minimum| {
                let current = context
                    .entries
                    .iter()
                    .find(|entry| entry.node == minimum.node)
                    .and_then(|entry| entry.minimum)
                    .ok_or_else(|| unsupported(minimum.node))?;
                if minimum.continued
                    && minimum.original.varying_border_box
                    && current.whole_insets != minimum.original.whole_insets
                    && current.content != minimum.original.content
                {
                    return Err(unsupported(minimum.node));
                }
                let accounted = progress
                    .active
                    .as_ref()
                    .filter(|active| active.index == frame)
                    .and_then(|active| {
                        active
                            .accounted
                            .iter()
                            .find(|(node, _)| *node == minimum.node)
                    })
                    .map_or(Tree::Scalar::ZERO, |(_, value)| *value);
                let earlier = (minimum.spent - accounted).max(Tree::Scalar::ZERO);
                Ok((
                    minimum.node,
                    (current.content - earlier).max(Tree::Scalar::ZERO),
                ))
            })
            .collect()
    }
    fn allocations(
        &self,
        remaining: &[(Tree::Node, Tree::Scalar)],
    ) -> Vec<FragmentBoxAllocation<Tree::Node, Tree::Scalar>> {
        remaining
            .iter()
            .map(|&(node, _)| FragmentBoxAllocation {
                node,
                known: Size::NONE,
                indefinite_axis: self.axes.block_axis(),
                minimum_content: Some(Tree::Scalar::ZERO),
            })
            .collect()
    }
    fn measure(
        &self,
        selection: &Selection,
        frame: usize,
        remaining: &[(Tree::Node, Tree::Scalar)],
    ) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
        let mut allocations = self.allocations(remaining);
        let mut packet = self.trial(
            selection.range.clone(),
            selection.partial,
            frame,
            allocations.clone(),
            selection.provenance,
        )?;
        // Descendant constraints settle before an ancestor's natural height is
        // inspected. Every conversion uses the actual Block content box.
        for source in self.inventory.nodes.iter().rev() {
            let Some(&(_, minimum)) = remaining.iter().find(|(node, _)| *node == source.node)
            else {
                continue;
            };
            if source.end > selection.range.end
                || selection.partial.is_some_and(|unit| source.end == unit + 1)
            {
                continue;
            }
            let Some(value) = packet
                .node_layout(source.node)
                .and_then(NodeLayoutRefOf::single_box)
            else {
                continue;
            };
            let output = value.unrounded_output();
            let content = self.content_extent(output)?;
            if minimum <= content {
                continue;
            }
            let outer = self.axes.logical_size(output.size).block;
            let known = self
                .axes
                .physical_size(LogicalSizeOf::new(None, Some(outer - content + minimum)));
            allocations
                .iter_mut()
                .find(|allocation| allocation.node == source.node)
                .expect("minimum allocation has source custody")
                .known = known;
            packet = self.trial(
                selection.range.clone(),
                selection.partial,
                frame,
                allocations.clone(),
                selection.provenance,
            )?;
        }
        Ok(packet)
    }
    fn commit(
        &self,
        event: crate::fragmentation::AcceptedStep<'_, SourceKey<Tree::Node>, Tree::Scalar>,
    ) -> NativeResult<Tree, ()> {
        if event.flow != 0 {
            return Err(self.failure(BlockFragmentationFailure::InvalidSource));
        }
        if event.graphical_progress.is_some()
            && !matches!(
                self.inventory.units.get(event.position.occurrence),
                Some(SourceUnit::End(_))
            )
        {
            return Err(self.failure(BlockFragmentationFailure::InvalidSource));
        }
        let old = self.progress.borrow();
        let previous = old
            .active
            .as_ref()
            .filter(|active| active.index == event.from_frame);
        let mut selection = previous.map(|active| active.selection.clone());
        for piece in event.pieces {
            let (end, partial) = match piece.kind {
                FragmentationPieceKindOf::GraphicalSlice { anchor, coverage }
                    if coverage.end < coverage.extent =>
                {
                    (anchor.occurrence + 1, Some(anchor.occurrence))
                }
                _ => (piece.end.occurrence, None),
            };
            match &mut selection {
                Some(selected) => {
                    selected.range.end = end;
                    selected.partial = partial;
                }
                None => {
                    selection = Some(Selection {
                        range: piece.start.occurrence..end,
                        partial,
                        provenance: piece.start_provenance,
                    })
                }
            }
        }
        drop(old);
        let Some(selection) = selection else {
            return Ok(());
        };
        let remaining = self.minimum_snapshot(event.from_frame)?;
        let mut packet = self.measure(&selection, event.from_frame, &remaining)?;
        let leaving = !event.breaks.is_empty();
        if leaving != (event.from_frame != event.to_frame) {
            return Err(self.failure(BlockFragmentationFailure::InvalidSource));
        }
        for (node, origin) in self.positions(&packet) {
            let source = self
                .inventory
                .nodes
                .iter()
                .find(|source| source.node == node)
                .expect("actual box has source custody");
            if source.atomic_owner.is_some() {
                continue;
            }
            let style = self.capture.tree.node_input(node);
            let continuing = source.first < selection.range.start
                || source.end > selection.range.end
                || selection.partial.is_some_and(|unit| source.end == unit + 1);
            if continuing
                && super::source::unsupported_continuation(style, self.axes)
                    .map_err(|error| crate::error::sizing_resolution_error(node, error))?
            {
                return Err(unsupported(node));
            }
            // Only admitted minimum continuations may replace a whole sizing
            // floor. Other constrained boxes retain ordinary whole-fit sizing.
            if !super::source::has_size_constraints(style, self.axes)
                || remaining.iter().any(|(key, _)| *key == node)
            {
                continue;
            }
            let output = packet
                .node_layout(node)
                .and_then(NodeLayoutRefOf::single_box)
                .expect("actual normal position")
                .unrounded_output();
            let start = self
                .axes
                .logical_point(origin, output.size, self.frames[event.from_frame].size)
                .block;
            let remaining = (self
                .axes
                .logical_size(self.frames[event.from_frame].size)
                .block
                .max(Tree::Scalar::ONE)
                - start)
                .max(Tree::Scalar::ZERO);
            if self.axes.logical_size(output.size).block > remaining {
                return Err(unsupported(node));
            }
        }
        if leaving {
            let mut allocations = self.allocations(&remaining);
            for (node, origin) in self.positions(&packet) {
                let source = self
                    .inventory
                    .nodes
                    .iter()
                    .find(|source| source.node == node)
                    .expect("produced box belongs to source");
                if source.atomic_owner.is_some() {
                    continue;
                }
                let continuing = source.end > selection.range.end
                    || selection.partial.is_some_and(|unit| source.end == unit + 1);
                if !continuing {
                    continue;
                }
                let output = packet
                    .node_layout(node)
                    .and_then(NodeLayoutRefOf::single_box)
                    .expect("position has actual box")
                    .unrounded_output();
                let start = self
                    .axes
                    .logical_point(origin, output.size, self.frames[event.from_frame].size)
                    .block;
                let available = (self
                    .axes
                    .logical_size(self.frames[event.from_frame].size)
                    .block
                    .max(Tree::Scalar::ONE)
                    - start)
                    .max(Tree::Scalar::ZERO);
                let used = self.axes.logical_size(output.size).block.max(available);
                let known = self
                    .axes
                    .physical_size(LogicalSizeOf::new(None, Some(used)));
                if let Some(allocation) = allocations
                    .iter_mut()
                    .find(|allocation| allocation.node == node)
                {
                    allocation.known = known;
                } else {
                    allocations.push(FragmentBoxAllocation {
                        node,
                        known,
                        indefinite_axis: self.axes.block_axis(),
                        minimum_content: None,
                    });
                }
            }
            // Preserve settled terminal children while allocating broken ancestors.
            for source in self.inventory.nodes.iter().rev() {
                if source.end <= selection.range.end
                    && !selection.partial.is_some_and(|unit| source.end == unit + 1)
                    && let Some(allocation) = allocations
                        .iter_mut()
                        .find(|allocation| allocation.node == source.node)
                    && allocation.known == Size::NONE
                    && let Some(value) = packet
                        .node_layout(source.node)
                        .and_then(NodeLayoutRefOf::single_box)
                {
                    allocation.known = self.axes.physical_size(LogicalSizeOf::new(
                        None,
                        Some(self.axes.logical_size(value.unrounded_output().size).block),
                    ));
                }
            }
            packet = self.trial(
                selection.range.clone(),
                selection.partial,
                event.from_frame,
                allocations,
                selection.provenance,
            )?;
        }
        let accounted = remaining
            .iter()
            .filter_map(|&(node, _)| {
                packet
                    .node_layout(node)
                    .and_then(NodeLayoutRefOf::single_box)
                    .map(|value| {
                        self.content_extent(value.unrounded_output())
                            .map(|content| (node, content))
                    })
            })
            .collect::<NativeResult<Tree, Vec<_>>>()?;
        let mut progress = self.progress.borrow_mut();
        // Account deltas against this frame's prior accepted production, never
        // against speculative prefix measurements.
        let prior = progress
            .active
            .as_ref()
            .map(|active| active.accounted.clone())
            .unwrap_or_default();
        for minimum in &mut progress.minima {
            let old = prior
                .iter()
                .find(|(node, _)| *node == minimum.node)
                .map_or(Tree::Scalar::ZERO, |(_, value)| *value);
            let current = accounted
                .iter()
                .find(|(node, _)| *node == minimum.node)
                .map_or(old, |(_, value)| *value);
            minimum.spent = (minimum.spent - old).max(Tree::Scalar::ZERO) + current;
            if !minimum.spent.is_finite() {
                return Err(self.failure(BlockFragmentationFailure::InvalidGeometry));
            }
            if leaving && accounted.iter().any(|(node, _)| *node == minimum.node) {
                minimum.continued = self
                    .inventory
                    .nodes
                    .iter()
                    .find(|entry| entry.node == minimum.node)
                    .is_some_and(|entry| {
                        entry.end > selection.range.end
                            || selection.partial.is_some_and(|unit| entry.end == unit + 1)
                    });
            }
        }
        for piece in event.pieces {
            if let FragmentationPieceKindOf::GraphicalSlice { anchor, coverage } = piece.kind {
                progress.tail_extents[anchor.occurrence] = Some(coverage.extent);
            }
        }
        if leaving || event.flow_complete {
            progress.completed.push(AcceptedFrame {
                frame: event.from_frame,
                range: selection.range,
                batch: packet,
            });
            progress.active = None;
        } else {
            progress.active = Some(ActiveFrame {
                index: event.from_frame,
                selection,
                accounted,
            });
        }
        Ok(())
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
        if query.position.offset != 0 {
            return Err(self.failure(BlockFragmentationFailure::InvalidSource));
        }
        let start = query.position.occurrence;
        let flow_end = self.inventory.units.len();
        if query.graphical_progress.is_some()
            && !matches!(self.inventory.units.get(start), Some(SourceUnit::End(_)))
        {
            return Err(self.failure(BlockFragmentationFailure::InvalidSource));
        }
        let remaining = self.minimum_snapshot(query.fragmentainer_index)?;
        let (frame_start, provenance) = self
            .progress
            .borrow()
            .active
            .as_ref()
            .filter(|active| active.index == query.fragmentainer_index)
            .map_or((start, query.start_provenance), |active| {
                (active.selection.range.start, active.selection.provenance)
            });
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
            cut: FragmentationCutOf::Source(query.position),
            block_offset: Tree::Scalar::ZERO,
            kind: FragmentationOpportunityKindOf::Sibling,
            before,
            after,
            ancestors,
        }];
        let mut extent = Tree::Scalar::ZERO;
        let mut head = Tree::Scalar::ZERO;
        for boundary in start + 1..=end {
            let selection = Selection {
                range: frame_start..boundary,
                partial: None,
                provenance,
            };
            let trial = self.measure(&selection, query.fragmentainer_index, &remaining)?;
            let body = (self.extent(&trial, query.fragmentainer_index)? - query.used_block_offset)
                .max(Tree::Scalar::ZERO);
            extent = extent.max(body);
            if boundary == start + 1 {
                head = body;
            }
            if boundary < flow_end {
                let (before, after, ancestors) =
                    self.inventory.controls(self.capture.tree, boundary);
                let kind = match self.inventory.units[boundary] {
                    SourceUnit::End(node) => {
                        let minimum = remaining
                            .iter()
                            .find(|(key, _)| *key == self.inventory.nodes[node].node)
                            .map_or(Tree::Scalar::ZERO, |(_, value)| *value);
                        let content = trial
                            .node_layout(self.inventory.nodes[node].node)
                            .and_then(NodeLayoutRefOf::single_box)
                            .map(|value| self.content_extent(value.unrounded_output()))
                            .transpose()?
                            .unwrap_or(Tree::Scalar::ZERO);
                        FragmentationOpportunityKindOf::ContentEdge {
                            gap: (minimum - content).max(Tree::Scalar::ZERO),
                        }
                    }
                    SourceUnit::Atomic(_) => FragmentationOpportunityKindOf::Sibling,
                };
                opportunities.push(FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(FragmentationPosition {
                        occurrence: boundary,
                        offset: 0,
                    }),
                    block_offset: body,
                    kind,
                    before,
                    after,
                    ancestors,
                });
            }
        }
        // Class C cuts are geometric interior points of the actual box-end
        // obligation. Source units remain exact and independent of CSS pixels.
        for unit in start..end {
            let SourceUnit::End(node) = self.inventory.units[unit] else {
                continue;
            };
            let key = self.inventory.nodes[node].node;
            let Some(&(_, minimum)) = remaining.iter().find(|(key_node, _)| *key_node == key)
            else {
                continue;
            };
            let prefix = Selection {
                range: frame_start..unit,
                partial: None,
                provenance,
            };
            let packet = self.measure(&prefix, query.fragmentainer_index, &remaining)?;
            let content = packet
                .node_layout(key)
                .and_then(NodeLayoutRefOf::single_box)
                .map(|value| self.content_extent(value.unrounded_output()))
                .transpose()?
                .unwrap_or(Tree::Scalar::ZERO);
            let gap = (minimum - content).max(Tree::Scalar::ZERO);
            let tail_start = self.extent(&packet, query.fragmentainer_index)?;
            let taken = query.used_block_offset + query.available_block_size - tail_start;
            if taken <= Tree::Scalar::ZERO || taken >= gap {
                continue;
            }
            let old = if unit == start {
                query
                    .graphical_progress
                    .map_or(Tree::Scalar::ZERO, |old| old.consumed)
            } else {
                Tree::Scalar::ZERO
            };
            let total = self.progress.borrow().tail_extents[unit].unwrap_or(old + gap);
            let (_, _, ancestors) = self.inventory.controls(self.capture.tree, unit);
            opportunities.push(FragmentationOpportunityOf {
                cut: FragmentationCutOf::Graphical {
                    anchor: FragmentationPosition {
                        occurrence: unit,
                        offset: 0,
                    },
                    progress: FragmentationGraphicalProgressOf {
                        consumed: old + taken,
                        extent: total,
                    },
                },
                block_offset: query.available_block_size,
                kind: FragmentationOpportunityKindOf::ContentEdge { gap },
                before: Vec::new(),
                after: Vec::new(),
                ancestors,
            });
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
    let mut inventory = Inventory::new(capture.tree, &capture.graph, *axes)?;
    let progress = Rc::new(RefCell::new(Progress::default()));
    let intrinsics = Rc::new(RefCell::new(Vec::new()));
    let first_context = {
        let initial = NativeProvider {
            capture,
            inventory: &inventory,
            frames,
            axes: *axes,
            kind: *kind,
            target: bound.target_basis(),
            template: bound.root_request(),
            intrinsics: intrinsics.clone(),
            progress: progress.clone(),
        };
        let first = frames.first().ok_or_else(|| {
            initial.failure(BlockFragmentationFailure::MissingFragmentainer { index: 0 })
        })?;
        initial.intrinsics(*first)?
    };
    let mut minima = Vec::new();
    for entry in &first_context.entries {
        let Some(minimum) = entry.minimum else {
            continue;
        };
        if super::source::unsupported_continuation(capture.tree.node_input(entry.node), *axes)
            .map_err(|error| crate::error::sizing_resolution_error(entry.node, error))?
        {
            continue;
        }
        minima.push(Minimum {
            node: entry.node,
            original: minimum,
            spent: Tree::Scalar::ZERO,
            continued: false,
        });
    }
    inventory.install_ends(
        &minima
            .iter()
            .map(|minimum| minimum.node)
            .collect::<Vec<_>>(),
    );
    {
        let mut state = progress.borrow_mut();
        state.minima = minima;
        state.tail_extents = vec![None; inventory.units.len()];
    }
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
                .units
                .iter()
                .map(|unit| {
                    let key = inventory.nodes[unit.node()].node;
                    FragmentationOccurrence {
                        key: match unit {
                            SourceUnit::Atomic(_) => SourceKey::Atomic(key),
                            SourceUnit::End(_) => SourceKey::BoxEnd(key),
                        },
                        source_extent: usize::from(matches!(unit, SourceUnit::Atomic(_))),
                        monolithic: matches!(unit, SourceUnit::Atomic(_)),
                    }
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
        intrinsics,
        progress,
    };
    let observer = provider.observer();
    let _report = crate::fragmentation::resolve_with_feedback(&input, &mut provider, |event| {
        observer.commit(event)
    })
    .map_err(|error| match error {
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
    let accepted = std::mem::take(&mut provider.progress.borrow_mut().completed);
    for frame in &accepted {
        inventory.validate_continuation(capture.tree, &frame.range, *axes)?;
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
