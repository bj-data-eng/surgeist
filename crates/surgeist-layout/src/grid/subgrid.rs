use std::collections::BTreeMap;

use super::*;
use crate::geometry::{FlowAxes, LogicalSizeOf};
use crate::geometry::{PhysicalAxis, PhysicalProgression};
use crate::{MaxSizeOf, NodeInputOf};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct GridSubgridReport<Node> {
    pub(super) items: Vec<SubgridItemReport<Node>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridItemReport<Node> {
    pub(super) node: Node,
    pub(super) column: SubgridAxisReport,
    pub(super) row: SubgridAxisReport,
}

impl<Node> SubgridItemReport<Node> {
    pub(super) fn needs_context(&self) -> bool {
        self.column.can_adopt() || self.row.can_adopt()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubgridAxisReport {
    pub(super) mapping: GridAxisMappingReport,
    pub(super) eligibility: SubgridEligibility,
    pub(super) inherit_parent_names: bool,
}

impl SubgridAxisReport {
    pub(super) fn can_inherit(self) -> bool {
        self.eligibility == SubgridEligibility::SharedTracks
    }

    pub(super) fn can_adopt(self) -> bool {
        self.eligibility.can_adopt()
    }

    #[cfg(test)]
    pub(super) fn requested(self) -> bool {
        self.eligibility != SubgridEligibility::Excluded(SubgridIneligibleReason::NotRequested)
    }

    pub(super) fn parent_name_context(
        self,
        lines: &NamedGridLines,
        area_facts: Option<&GridAreaNameFacts>,
    ) -> (NamedGridLines, Option<GridAreaNameFacts>) {
        if self.inherit_parent_names {
            (
                lines.clone(),
                area_facts
                    .filter(|facts| facts.is_valid_for_axis(self.mapping.parent_axis))
                    .cloned(),
            )
        } else {
            (lines.unnamed_used_frame(), None)
        }
    }
}

pub(super) fn inherited_subgrid_physical_axis(
    report: SubgridAxisReport,
    parent_flow_axes: crate::geometry::FlowAxes,
    child_flow_axes: crate::geometry::FlowAxes,
) -> Option<PhysicalAxis> {
    if !report.can_inherit() {
        return None;
    }
    let mapping = report.mapping;
    let physical_axis = physical_axis_for_grid_axis(parent_flow_axes, mapping.parent_axis);
    if physical_axis != physical_axis_for_grid_axis(child_flow_axes, mapping.child_axis) {
        return None;
    }
    Some(physical_axis)
}

pub(super) fn subgrid_parent_propagation_axes(
    item: SubgridItemReport<impl Copy>,
    parent_flow_axes: crate::geometry::FlowAxes,
    child_flow_axes: crate::geometry::FlowAxes,
) -> (bool, bool) {
    let column_axis =
        inherited_subgrid_physical_axis(item.column, parent_flow_axes, child_flow_axes);
    let row_axis = inherited_subgrid_physical_axis(item.row, parent_flow_axes, child_flow_axes);
    if column_axis.is_none() && row_axis.is_none() {
        return (true, true);
    }

    (
        column_axis == Some(PhysicalAxis::Horizontal) || row_axis == Some(PhysicalAxis::Horizontal),
        column_axis == Some(PhysicalAxis::Vertical) || row_axis == Some(PhysicalAxis::Vertical),
    )
}

fn physical_axis_for_grid_axis(
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
) -> PhysicalAxis {
    match axis {
        GridAxisKind::Column => flow_axes.inline_axis(),
        GridAxisKind::Row => flow_axes.block_axis(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridEligibility {
    SharedTracks,
    Stacking,
    Excluded(SubgridIneligibleReason),
}

impl SubgridEligibility {
    pub(super) fn can_adopt(self) -> bool {
        matches!(self, Self::SharedTracks | Self::Stacking)
    }
}

#[derive(Clone, Copy)]
pub(super) struct SubgridEligibilityInput<'input, 'a, S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) parent_style: &'a GridContainerProjection<'a, S>,
    pub(super) has_parent_grid: bool,
    pub(super) child_style: &'a GridItemProjection<'input, S>,
    pub(super) child_requested: bool,
    pub(super) child_container: Option<&'a GridContainerInput<'input, S>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridIneligibleReason {
    NotRequested,
    NoParentGrid,
    UnsupportedDisplay,
    NoChildGridContainer,
    ExcludedFromNormalLayout,
    ChildStackingAxis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct GridTrackSpan {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl GridTrackSpan {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(super) fn checked_len(self) -> Option<usize> {
        self.end
            .checked_sub(self.start)
            .filter(|length| *length > 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum IntrinsicMinTrackFacts<'a> {
    Known(&'a [bool]),
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "retained missing-facts state for subgrid traversal error parity"
        )
    )]
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridTraversalError {
    MissingIntrinsicMinTrackFacts,
    SpanOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridTraversalAxis {
    Inherited,
    Standalone,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridAxisEdges<S: LayoutScalar = Scalar> {
    pub(super) start: S,
    pub(super) end: S,
}

impl<S: LayoutScalar> Default for SubgridAxisEdges<S> {
    fn default() -> Self {
        Self {
            start: S::ZERO,
            end: S::ZERO,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(super) struct SubgridTraversalInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(super) ancestor_track_intrinsic_min_eligibility: IntrinsicMinTrackFacts<'a>,
    pub(super) root_children: Vec<SubgridTraversalChild<'input, Node, S>>,
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(super) enum SubgridTraversalChild<'input, Node, S: LayoutScalar = Scalar> {
    Subgrid(SubgridTraversalNode<'input, Node, S>),
    Leaf(SubgridTraversalLeaf<'input, Node, S>),
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(super) struct SubgridTraversalNode<'input, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) style: GridItemProjection<'input, S>,
    pub(super) axis: SubgridTraversalAxis,
    pub(super) reversed: bool,
    pub(super) span_in_parent: GridTrackSpan,

    pub(super) align_self: AlignItems,
    pub(super) min_frame: Option<SubgridLeafMinimumFrame<S>>,
    pub(super) standalone_parent_context: Option<Box<IntrinsicSubgridSource<'input, Node, S>>>,
    pub(super) margins: SubgridAxisEdges<S>,
    pub(super) border: SubgridAxisEdges<S>,
    pub(super) padding: SubgridAxisEdges<S>,
    pub(super) scrollbar_gutter: SubgridAxisEdges<S>,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: S,
    pub(super) children: Vec<SubgridTraversalChild<'input, Node, S>>,
}

/// Resolved edge geometry for one live inherited wrapper.
#[derive(Clone, Copy, Debug)]
struct SubgridWrapperFrame<S: LayoutScalar> {
    reversed: bool,
    span_in_parent: GridTrackSpan,
    margins: SubgridAxisEdges<S>,
    border: SubgridAxisEdges<S>,
    padding: SubgridAxisEdges<S>,
    scrollbar_gutter: SubgridAxisEdges<S>,
    parent_gap: S,
    subgrid_gap: S,
}

#[cfg(test)]
impl<Node, S: LayoutScalar> SubgridTraversalNode<'_, Node, S> {
    fn wrapper_frame(&self) -> SubgridWrapperFrame<S> {
        SubgridWrapperFrame {
            reversed: self.reversed,
            span_in_parent: self.span_in_parent,
            margins: self.margins,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: self.scrollbar_gutter,
            parent_gap: self.parent_gap,
            subgrid_gap: self.subgrid_gap,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTraversalLeaf<'input, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) style: GridItemProjection<'input, S>,
    pub(super) span_in_parent: GridTrackSpan,

    pub(super) align_self: AlignItems,
    pub(super) min_frame: Option<SubgridLeafMinimumFrame<S>>,
}

#[derive(Clone, Debug)]
pub(super) struct SubgridLeafContribution<'input, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) style: GridItemProjection<'input, S>,
    pub(super) ancestor_span: GridTrackSpan,
    pub(super) alignment_context_reversed: bool,

    pub(super) align_self: AlignItems,
    pub(super) min_frame: Option<SubgridLeafMinimumFrame<S>>,
    pub(super) standalone_parent_context: Option<Box<IntrinsicSubgridSource<'input, Node, S>>>,
    adjustment_frame: Option<usize>,
}

/// A leaf's own immediate-parent query authority, retained before traversal
/// translates its contribution span into an ancestor's coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridLeafMinimumFrame<S: LayoutScalar> {
    pub(super) flow_axes: FlowAxes,
    pub(super) axis: GridAxisKind,
    pub(super) algorithm: SizingAlgorithm,
    pub(super) span: GridMinimumSpanFacts<S>,
    pub(super) opposite_fixed_max_span: Option<S>,
    prepared: ComputeInputOf<S>,
}

impl<S: LayoutScalar> SubgridLeafMinimumFrame<S> {
    pub(super) const fn prepared(self) -> ComputeInputOf<S> {
        self.prepared
    }
}

/// Missing functions mean a geometry-only traversal, never a proof that a
/// minimum contribution has no fixed maximum cap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridMinimumParentFunctions<'a, S: LayoutScalar> {
    pub(super) columns: Option<GridMinimumTrackFunctions<'a, S>>,
    pub(super) rows: Option<GridMinimumTrackFunctions<'a, S>>,
}

impl<'a, S: LayoutScalar> GridMinimumParentFunctions<'a, S> {
    pub(super) fn requalified(self, basis: LogicalSizeOf<Option<S>>) -> Self {
        Self {
            columns: self
                .columns
                .map(|functions| functions.requalified(basis.inline)),
            rows: self
                .rows
                .map(|functions| functions.requalified(basis.block)),
        }
    }

    pub(super) fn for_axis(self, axis: GridAxisKind) -> Option<GridMinimumTrackFunctions<'a, S>> {
        match axis {
            GridAxisKind::Column => self.columns,
            GridAxisKind::Row => self.rows,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SubgridParentTrackSource<'a, S: LayoutScalar> {
    pub(super) baseline_references: crate::GridBaselineReferences,
    pub(super) flow_axes: FlowAxes,
    pub(super) gap: Size<S>,
    pub(super) columns: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) rows: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(super) algorithm: Option<SizingAlgorithm>,
    pub(super) lane_grid_axis: Option<GridAxisKind>,
    /// Only actual stage allocations; geometry may instead be provisional.
    pub(super) allocated: LogicalSizeOf<Option<&'a [S]>>,
}

#[derive(Clone, Copy)]
pub(super) struct SubgridInheritedAxisSource<'a, S: LayoutScalar> {
    pub(super) baseline_reference: crate::GridBaselineReference,
    pub(super) geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) allocated: Option<&'a [S]>,
    pub(super) minimum_functions: Option<GridMinimumTrackFunctions<'a, S>>,
}

impl<'a, S: LayoutScalar> SubgridParentTrackSource<'a, S> {
    fn functions_in_span(
        self,
        axis: GridAxisKind,
        area: GridArea<S>,
        reversed: bool,
    ) -> Option<GridMinimumTrackFunctions<'a, S>> {
        let (start, end) = match axis {
            GridAxisKind::Column => (area.column, area.column_end),
            GridAxisKind::Row => (area.row, area.row_end),
        };
        self.minimum_functions
            .for_axis(axis)?
            .mapped_span(start, end, reversed)
    }

    pub(super) fn frame<Node: Copy, M>(
        self,
        node: Node,
        axis: GridAxisKind,
        area: GridArea<S>,
        prepared: ComputeInputOf<S>,
    ) -> LayoutResultOf<Node, Option<SubgridLeafMinimumFrame<S>>, S, M> {
        let Some(algorithm) = self.algorithm else {
            return Ok(None);
        };
        let opposite = match axis {
            GridAxisKind::Column => GridAxisKind::Row,
            GridAxisKind::Row => GridAxisKind::Column,
        };
        let lane_track_axis = self.lane_grid_axis == Some(axis);
        if self.minimum_functions.for_axis(axis).is_none()
            || (!lane_track_axis && self.minimum_functions.for_axis(opposite).is_none())
        {
            return Ok(None);
        }
        let own = self
            .functions_in_span(axis, area, false)
            .ok_or_else(|| minimum_source_error(node))?;
        let gutter = |which| -> LayoutResultOf<Node, S, S, M> {
            let (start, end, geometry, gap) = match which {
                GridAxisKind::Column => {
                    (area.column, area.column_end, self.columns, self.gap.width)
                }
                GridAxisKind::Row => (area.row, area.row_end, self.rows, self.gap.height),
            };
            match geometry {
                Some(geometry) => {
                    let values = geometry
                        .gutter_after()
                        .get(start..end.saturating_sub(1))
                        .ok_or_else(|| minimum_source_error(node))?;
                    Ok(values
                        .iter()
                        .copied()
                        .fold(S::ZERO, |sum, value| sum + value))
                }
                None => Ok(gap * S::from_usize(end.saturating_sub(start + 1))),
            }
        };
        let opposite_fixed_max_span = if lane_track_axis {
            None
        } else {
            self.functions_in_span(opposite, area, false)
                .ok_or_else(|| minimum_source_error(node))?
                .span_facts(gutter(opposite)?)
                .map_err(|error| sizing_resolution_error(node, error))?
                .fixed_max_span()
        };
        Ok(Some(SubgridLeafMinimumFrame {
            flow_axes: self.flow_axes,
            axis,
            algorithm,
            span: own
                .span_facts(gutter(axis)?)
                .map_err(|error| sizing_resolution_error(node, error))?,
            opposite_fixed_max_span,
            prepared,
        }))
    }
}

pub(super) fn minimum_source_error<Node, S: LayoutScalar, M>(
    node: Node,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance),
    )
}

pub(super) struct GridMinimumWrapperInput<'input, 'a, Node, S: LayoutScalar> {
    pub(super) axis: GridAxisKind,
    pub(super) container: &'a GridContainerInput<'input, S>,
    pub(super) parent_context: &'a GridParentContext<S, Node>,
    pub(super) prepared: ComputeInputOf<S>,
    pub(super) containing_flow_axes: FlowAxes,
    /// Traversal geometry only; never promoted into property/known authority.
    /// A wrapper-direct caller without an allocated opposite axis supplies None.
    pub(super) content_box_estimate: Option<Size<S>>,
    pub(super) traversal: GridMinimumWrapperTraversal,
    /// Actual resolved gap for a geometry-only traversal, which has no child
    /// preparation/property basis. Prepared traversal uses its canonical gap.
    pub(super) query: IntrinsicGridQuery<'a, S>,
}

#[derive(Clone, Copy)]
pub(super) enum GridMinimumWrapperTraversal {
    /// The shared traversal visits each feasible automatic track range.
    Shared { lane_track: bool },
    /// An additional outer wrapper placement admits only its automatic items.
    SharedLaneCandidate,
    /// The Lane collector projects its own candidates from one prepared view.
    LaneCandidates,
}

impl GridMinimumWrapperTraversal {
    fn uses_lane_track_sizing(self) -> bool {
        match self {
            Self::Shared { lane_track } => lane_track,
            Self::SharedLaneCandidate | Self::LaneCandidates => true,
        }
    }
}

/// All references remain borrowed from the one live wrapper preparation. A
/// visitor may query or recurse here; only scalar leaf frames escape traversal.
pub(super) struct GridMinimumWrapperChild<'input, 'a, Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) source_index: crate::SourceIndex,
    pub(super) execution: IntrinsicGridChildInput<'input, 'a, Node, S>,
    pub(super) prepared: ComputeInputOf<S>,
    pub(super) minimum_source: SubgridParentTrackSource<'a, S>,
    pub(super) child_axis: GridAxisKind,
    traversal: GridMinimumWrapperTraversal,
    pub(super) area_size: Size<S>,
    pub(super) align_self: AlignItems,
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SubgridAncestorBaselineAdjustments<S: LayoutScalar = Scalar> {
    pub(super) start: S,
    pub(super) end: S,
}

#[derive(Clone, Copy, Debug)]
struct SubgridEdgeAdjustment<S: LayoutScalar> {
    index: usize,
    extent: S,
    start_role: bool,
    intrinsic: bool,
}

#[derive(Clone, Debug)]
struct SubgridAdjustmentFrame<S: LayoutScalar> {
    parent: Option<usize>,
    edges: [SubgridEdgeAdjustment<S>; 2],
    start: usize,
    end: usize,
    half_gap_difference: S,
    start_adjustments: SubgridTrackAdjustments<S>,
    end_adjustments: SubgridTrackAdjustments<S>,
    interior_adjustments: Option<SubgridTrackAdjustments<S>>,
}

#[derive(Clone, Copy, Debug)]
struct SubgridTrackAdjustments<S: LayoutScalar> {
    edge: S,
    gap: S,
    baseline: SubgridAncestorBaselineAdjustments<S>,
}

#[derive(Clone, Debug)]
pub(super) struct SubgridAdjustmentFrames<S: LayoutScalar> {
    track_count: usize,
    frames: Vec<SubgridAdjustmentFrame<S>>,
}

impl<S: LayoutScalar> SubgridTrackAdjustments<S> {
    const ZERO: Self = Self {
        edge: S::ZERO,
        gap: S::ZERO,
        baseline: SubgridAncestorBaselineAdjustments {
            start: S::ZERO,
            end: S::ZERO,
        },
    };
}

impl<S: LayoutScalar> SubgridAdjustmentFrame<S> {
    fn apply_to(
        &self,
        track: usize,
        mut adjustments: SubgridTrackAdjustments<S>,
    ) -> SubgridTrackAdjustments<S> {
        for edge in self.edges {
            if edge.index == track {
                if edge.intrinsic {
                    adjustments.edge = adjustments.edge + edge.extent;
                }
                if edge.start_role {
                    adjustments.baseline.start = adjustments.baseline.start + edge.extent;
                } else {
                    adjustments.baseline.end = adjustments.baseline.end + edge.extent;
                }
            }
        }
        let half_gap = self.half_gap_difference;
        if self.start < track && track <= self.end {
            adjustments.gap = adjustments.gap + half_gap;
            adjustments.baseline.start = adjustments.baseline.start + half_gap;
        }
        if self.start <= track && track < self.end {
            adjustments.gap = adjustments.gap + half_gap;
            adjustments.baseline.end = adjustments.baseline.end + half_gap;
        }
        adjustments
    }
}

impl<S: LayoutScalar> SubgridAdjustmentFrames<S> {
    fn track_adjustments(
        &self,
        mut frame: Option<usize>,
        track: usize,
        path: &mut Vec<usize>,
    ) -> SubgridTrackAdjustments<S> {
        path.clear();
        let mut adjustments = SubgridTrackAdjustments::ZERO;
        while let Some(index) = frame {
            let current = &self.frames[index];
            if track == current.start {
                adjustments = current.start_adjustments;
                break;
            }
            if track == current.end {
                adjustments = current.end_adjustments;
                break;
            }
            if current.start < track && track < current.end {
                if let Some(interior) = current.interior_adjustments {
                    adjustments = interior;
                    break;
                }
                path.push(index);
            }
            frame = current.parent;
        }
        for &index in path.iter().rev() {
            adjustments = self.frames[index].apply_to(track, adjustments);
        }
        adjustments
    }

    pub(super) fn for_leaf<Node>(
        &self,
        leaf: &SubgridLeafContribution<'_, Node, S>,
        path: &mut Vec<usize>,
    ) -> Option<(S, SubgridAncestorBaselineAdjustments<S>)> {
        let start = leaf.ancestor_span.start.checked_sub(1)?;
        let end = leaf.ancestor_span.end.checked_sub(1)?;
        if start >= end || end > self.track_count {
            return None;
        }

        let frame = leaf.adjustment_frame;
        let first = self.track_adjustments(frame, start, path);
        if end == start + 1 {
            return Some((S::ZERO + first.edge + first.gap, first.baseline));
        }

        // Preserve the ordered sum of all edge values followed by all gaps.
        let mut edge_sum = S::ZERO + first.edge;
        let mut last = first;
        for track in start + 1..end {
            last = self.track_adjustments(frame, track, path);
            edge_sum = edge_sum + last.edge;
        }
        let scalar = (start..end).fold(edge_sum, |sum, track| {
            sum + self.track_adjustments(frame, track, path).gap
        });
        Some((
            scalar,
            SubgridAncestorBaselineAdjustments {
                start: first.baseline.start,
                end: last.baseline.end,
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub(super) struct SubgridTraversalReport<'input, Node, S: LayoutScalar = Scalar> {
    pub(super) edge_lower_bounds: Vec<S>,
    pub(super) leaves: Vec<SubgridLeafContribution<'input, Node, S>>,
    pub(super) adjustments: SubgridAdjustmentFrames<S>,
}

#[cfg(test)]
impl<Node, S: LayoutScalar> SubgridTraversalReport<'_, Node, S> {
    fn track_adjustments_for_leaf(
        &self,
        index: usize,
        select: impl Fn(SubgridTrackAdjustments<S>) -> S,
    ) -> Vec<S> {
        let mut path = Vec::new();
        let frame = self.leaves[index].adjustment_frame;
        (0..self.adjustments.track_count)
            .map(|track| select(self.adjustments.track_adjustments(frame, track, &mut path)))
            .collect()
    }

    pub(super) fn edge_adjustments_for_leaf(&self, index: usize) -> Vec<S> {
        self.track_adjustments_for_leaf(index, |adjustments| adjustments.edge)
    }

    pub(super) fn gap_adjustments_for_leaf(&self, index: usize) -> Vec<S> {
        self.track_adjustments_for_leaf(index, |adjustments| adjustments.gap)
    }
}

struct SubgridQualifiedLeaf<S: LayoutScalar> {
    leaf_index: usize,
    adjustments: (S, SubgridAncestorBaselineAdjustments<S>),
}

struct SubgridCandidateLeafIndex<S: LayoutScalar> {
    by_span: BTreeMap<(usize, usize), Vec<SubgridQualifiedLeaf<S>>>,
}

impl<S: LayoutScalar> SubgridCandidateLeafIndex<S> {
    fn new() -> Self {
        Self {
            by_span: BTreeMap::new(),
        }
    }

    fn insert(
        &mut self,
        leaf_index: usize,
        span: GridTrackSpan,
        adjustments: (S, SubgridAncestorBaselineAdjustments<S>),
    ) {
        self.by_span
            .entry((span.start, span.end))
            .or_default()
            .push(SubgridQualifiedLeaf {
                leaf_index,
                adjustments,
            });
    }
}

/// Preparation streams through live wrapper frames. Hypothetical plain leaves
/// reuse earlier retained leaves only after complete contribution qualification.
struct SubgridIntrinsicFold<'input, 'a, Node, S: LayoutScalar> {
    intrinsic_min: &'a [bool],
    report: SubgridTraversalReport<'input, Node, S>,
    adjustment_path: Vec<usize>,
    candidate_index: Option<SubgridCandidateLeafIndex<S>>,
    error: Option<SubgridTraversalError>,
}

impl<'input, 'a, Node: Copy + Eq, S: LayoutScalar> SubgridIntrinsicFold<'input, 'a, Node, S> {
    fn new(facts: IntrinsicMinTrackFacts<'a>) -> Self {
        let (intrinsic_min, error) = match facts {
            IntrinsicMinTrackFacts::Known(facts) => (facts, None),
            IntrinsicMinTrackFacts::Unknown => (
                &[][..],
                Some(SubgridTraversalError::MissingIntrinsicMinTrackFacts),
            ),
        };
        Self {
            intrinsic_min,
            report: SubgridTraversalReport {
                edge_lower_bounds: vec![S::ZERO; intrinsic_min.len()],
                leaves: Vec::new(),
                adjustments: SubgridAdjustmentFrames {
                    track_count: intrinsic_min.len(),
                    frames: Vec::new(),
                },
            },
            adjustment_path: Vec::new(),
            candidate_index: None,
            error,
        }
    }

    fn enter(
        &mut self,
        subgrid: &SubgridWrapperFrame<S>,
        context: SubgridTraversalContext,
        empty: bool,
    ) -> SubgridTraversalContext {
        if self.error.is_some() {
            return context;
        }
        match apply_subgrid_edge_placeholders(
            self.intrinsic_min,
            &mut self.report.edge_lower_bounds,
            &mut self.report.adjustments,
            &mut self.adjustment_path,
            subgrid,
            context,
            empty,
        ) {
            Ok(context) => context,
            Err(error) => {
                // Preparation errors have precedence. Continue visiting all
                // candidates, but freeze structural traversal at its first error.
                self.error = Some(error);
                context
            }
        }
    }

    fn activate_candidate_index(&mut self) {
        let mut index = SubgridCandidateLeafIndex::new();
        for (leaf_index, leaf) in self.report.leaves.iter().enumerate() {
            if leaf.standalone_parent_context.is_some() {
                continue;
            }
            let Some(adjustments) = self
                .report
                .adjustments
                .for_leaf(leaf, &mut self.adjustment_path)
            else {
                continue;
            };
            index.insert(leaf_index, leaf.ancestor_span, adjustments);
        }
        self.candidate_index = Some(index);
    }

    fn leaf(
        &mut self,
        leaf: SubgridTraversalLeaf<'input, Node, S>,
        context: SubgridTraversalContext,
        standalone_parent_context: Option<Box<IntrinsicSubgridSource<'input, Node, S>>>,
    ) {
        if self.error.is_some() {
            return;
        }
        let ancestor_span = leaf
            .span_in_parent
            .checked_len()
            .ok_or(SubgridTraversalError::SpanOutOfRange)
            .and_then(|_| translate_span_to_ancestor(context.transform, leaf.span_in_parent));
        let ancestor_span = match ancestor_span {
            Ok(span) => span,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        let contribution = SubgridLeafContribution {
            node: leaf.node,
            style: leaf.style,
            ancestor_span,
            alignment_context_reversed: context.transform.line_direction < 0,
            align_self: leaf.align_self,
            min_frame: leaf.min_frame,
            standalone_parent_context,
            adjustment_frame: context.adjustment_frame,
        };
        let plain = contribution.standalone_parent_context.is_none();
        if context.candidate_path && plain && self.candidate_index.is_none() {
            self.activate_candidate_index();
        }
        // Ordinary traversal needs neither adjustment qualification nor an index.
        let qualified = if self.candidate_index.is_some() && plain {
            self.report
                .adjustments
                .for_leaf(&contribution, &mut self.adjustment_path)
        } else {
            None
        };
        // Equality is deliberately independent of arena identity. Qualify the
        // complete ordered scalar and both baseline edge roles before reuse.
        // A standalone source has additional parent authority and stays unique.
        let span_key = (ancestor_span.start, ancestor_span.end);
        if context.candidate_path
            && qualified.is_some()
            && self.candidate_index.as_ref().is_some_and(|index| {
                index.by_span.get(&span_key).is_some_and(|entries| {
                    entries.iter().any(|entry| {
                        let retained = &self.report.leaves[entry.leaf_index];
                        retained.node == contribution.node
                            && retained.style == contribution.style
                            && retained.alignment_context_reversed
                                == contribution.alignment_context_reversed
                            && retained.align_self == contribution.align_self
                            && retained.min_frame == contribution.min_frame
                            && Some(entry.adjustments) == qualified
                    })
                })
            })
        {
            return;
        }
        if let (Some(index), Some(adjustments)) = (&mut self.candidate_index, qualified) {
            // Actual occurrences are retained individually and seed the bucket
            // used by hypothetical occurrences.
            index.insert(self.report.leaves.len(), ancestor_span, adjustments);
        }
        self.report.leaves.push(contribution);
    }

    fn finish_wrapper(
        &mut self,
        context: SubgridTraversalContext,
        span_len: usize,
        empty: bool,
        frame_checkpoint: usize,
        leaf_checkpoint: usize,
    ) {
        if self.error.is_none() && empty && context.admit_wrapper_floor {
            // The streamed visitor discovers emptiness only after preparation.
            let span =
                translate_span_to_ancestor(context.transform, GridTrackSpan::new(1, span_len + 1))
                    .expect("enter validated the wrapper span");
            apply_empty_subgrid_floor(
                self.intrinsic_min,
                &mut self.report.edge_lower_bounds,
                &self.report.adjustments,
                &mut self.adjustment_path,
                context.adjustment_frame,
                span.start - 1,
                span.end - 2,
            );
        }
        // No earlier leaf can reference this suffix. Duplicate-only and empty
        // subtrees release their frames before preparing the next candidate.
        if self.report.leaves.len() == leaf_checkpoint {
            self.report.adjustments.frames.truncate(frame_checkpoint);
        }
    }

    fn finish(self) -> Result<SubgridTraversalReport<'input, Node, S>, SubgridTraversalError> {
        self.error.map_or(Ok(self.report), Err)
    }
}

#[cfg(test)]
pub(super) fn traverse_subgrid_intrinsic<'input, Node: Copy + Eq, S: LayoutScalar>(
    input: SubgridTraversalInput<'input, '_, Node, S>,
) -> Result<SubgridTraversalReport<'input, Node, S>, SubgridTraversalError> {
    let mut fold = SubgridIntrinsicFold::new(input.ancestor_track_intrinsic_min_eligibility);
    let mut stack = input
        .root_children
        .into_iter()
        .rev()
        .map(|child| (child, SubgridTraversalContext::ROOT))
        .collect::<Vec<_>>();
    while let Some((child, context)) = stack.pop() {
        match child {
            SubgridTraversalChild::Leaf(leaf) => fold.leaf(leaf, context, None),
            SubgridTraversalChild::Subgrid(mut subgrid) => {
                if subgrid.axis == SubgridTraversalAxis::Standalone {
                    fold.leaf(
                        SubgridTraversalLeaf {
                            node: subgrid.node,
                            style: subgrid.style,
                            span_in_parent: subgrid.span_in_parent,
                            align_self: subgrid.align_self,
                            min_frame: subgrid.min_frame,
                        },
                        context,
                        subgrid.standalone_parent_context,
                    );
                } else {
                    let context = fold.enter(
                        &subgrid.wrapper_frame(),
                        context,
                        subgrid.children.is_empty(),
                    );
                    for child in subgrid.children.drain(..).rev() {
                        stack.push((child, context));
                    }
                }
            }
        }
    }
    fold.finish()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SubgridLineTransform {
    line_offset: isize,
    line_direction: isize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SubgridTraversalContext {
    transform: SubgridLineTransform,
    adjustment_frame: Option<usize>,
    admit_wrapper_floor: bool,
    candidate_path: bool,
}

impl SubgridTraversalContext {
    const ROOT: Self = Self {
        transform: SubgridLineTransform {
            line_offset: 0,
            line_direction: 1,
        },
        adjustment_frame: None,
        admit_wrapper_floor: true,
        candidate_path: false,
    };
}

fn apply_subgrid_edge_placeholders<S: LayoutScalar>(
    intrinsic_min: &[bool],
    edge_lower_bounds: &mut [S],
    adjustments: &mut SubgridAdjustmentFrames<S>,
    adjustment_path: &mut Vec<usize>,
    subgrid: &SubgridWrapperFrame<S>,
    mut context: SubgridTraversalContext,
    empty: bool,
) -> Result<SubgridTraversalContext, SubgridTraversalError> {
    let span_len = subgrid
        .span_in_parent
        .checked_len()
        .ok_or(SubgridTraversalError::SpanOutOfRange)?;
    let ancestor_span = translate_span_to_ancestor(context.transform, subgrid.span_in_parent)?;
    let start_index = ancestor_span.start - 1;
    let end_index = ancestor_span.end - 2;
    if end_index >= intrinsic_min.len() || end_index >= edge_lower_bounds.len() {
        return Err(SubgridTraversalError::MissingIntrinsicMinTrackFacts);
    }

    let child_line_transform =
        child_line_transform(context.transform, subgrid.span_in_parent, subgrid.reversed);
    let (local_start_index, local_end_index) = edge_track_indices(child_line_transform, span_len)?;
    if local_start_index >= intrinsic_min.len() || local_end_index >= intrinsic_min.len() {
        return Err(SubgridTraversalError::MissingIntrinsicMinTrackFacts);
    }

    let local_start_edge = subgrid.margins.start
        + subgrid.border.start
        + subgrid.padding.start
        + subgrid.scrollbar_gutter.start;
    let local_end_edge = subgrid.margins.end
        + subgrid.border.end
        + subgrid.padding.end
        + subgrid.scrollbar_gutter.end;

    let forward = child_line_transform.line_direction > 0;
    let edges = [
        SubgridEdgeAdjustment {
            index: local_start_index,
            extent: local_start_edge,
            start_role: forward,
            intrinsic: intrinsic_min[local_start_index],
        },
        SubgridEdgeAdjustment {
            index: local_end_index,
            extent: local_end_edge,
            start_role: !forward,
            intrinsic: intrinsic_min[local_end_index],
        },
    ];
    let half_gap_difference = (subgrid.subgrid_gap - subgrid.parent_gap) / S::from_f64(2.0);
    let identity = edges.iter().all(|edge| edge.extent == S::ZERO)
        && (start_index == end_index || half_gap_difference == S::ZERO);
    let parent = context.adjustment_frame;
    let frame = if identity {
        parent
    } else {
        let start = adjustments.track_adjustments(parent, start_index, adjustment_path);
        let end = if start_index == end_index {
            start
        } else {
            adjustments.track_adjustments(parent, end_index, adjustment_path)
        };
        let mut start_edge = start.edge;
        let mut end_edge = end.edge;
        for edge in edges {
            if edge.intrinsic {
                let current = if edge.index == start_index {
                    &mut start_edge
                } else {
                    &mut end_edge
                };
                *current = *current + edge.extent;
                if context.admit_wrapper_floor {
                    edge_lower_bounds[edge.index] = edge_lower_bounds[edge.index].max(*current);
                }
            }
        }

        let mut frame = SubgridAdjustmentFrame {
            parent,
            edges,
            start: start_index,
            end: end_index,
            half_gap_difference,
            start_adjustments: start,
            end_adjustments: end,
            interior_adjustments: None,
        };
        frame.start_adjustments = frame.apply_to(start_index, start);
        frame.end_adjustments = if start_index == end_index {
            frame.start_adjustments
        } else {
            frame.apply_to(end_index, end)
        };
        if start_index + 1 < end_index {
            // A contained interval's interior cannot cross any ancestor edge.
            let inherited_interior = if let Some(parent) = parent {
                let parent = &adjustments.frames[parent];
                (parent.start <= start_index && end_index <= parent.end)
                    .then_some(parent.interior_adjustments)
                    .flatten()
            } else {
                Some(SubgridTrackAdjustments::ZERO)
            };
            frame.interior_adjustments =
                inherited_interior.map(|inherited| frame.apply_to(start_index + 1, inherited));
        }
        let index = adjustments.frames.len();
        adjustments.frames.push(frame);
        Some(index)
    };
    if empty && context.admit_wrapper_floor {
        apply_empty_subgrid_floor(
            intrinsic_min,
            edge_lower_bounds,
            adjustments,
            adjustment_path,
            frame,
            start_index,
            end_index,
        );
    }
    context.transform = child_line_transform;
    context.adjustment_frame = frame;
    Ok(context)
}

fn apply_empty_subgrid_floor<S: LayoutScalar>(
    intrinsic_min: &[bool],
    edge_lower_bounds: &mut [S],
    adjustments: &SubgridAdjustmentFrames<S>,
    adjustment_path: &mut Vec<usize>,
    frame: Option<usize>,
    start_index: usize,
    end_index: usize,
) {
    for edge_index in start_index..=end_index {
        if intrinsic_min[edge_index] {
            let track = adjustments.track_adjustments(frame, edge_index, adjustment_path);
            let lower_bound = track.edge + track.gap.max(S::ZERO);
            edge_lower_bounds[edge_index] = edge_lower_bounds[edge_index].max(lower_bound);
        }
    }
}

fn child_line_transform(
    context: SubgridLineTransform,
    span_in_parent: GridTrackSpan,
    reversed: bool,
) -> SubgridLineTransform {
    let local_offset = if reversed {
        span_in_parent.end as isize + 1
    } else {
        span_in_parent.start as isize - 1
    };
    let local_direction = if reversed { -1 } else { 1 };
    SubgridLineTransform {
        line_offset: context.line_offset + context.line_direction * local_offset,
        line_direction: context.line_direction * local_direction,
    }
}

fn translate_span_to_ancestor(
    context: SubgridLineTransform,
    local_span: GridTrackSpan,
) -> Result<GridTrackSpan, SubgridTraversalError> {
    let start_line = map_line_to_ancestor(context, local_span.start);
    let end_line = map_line_to_ancestor(context, local_span.end);
    let start = start_line.min(end_line);
    let end = start_line.max(end_line);
    if start <= 0 || end <= start {
        return Err(SubgridTraversalError::SpanOutOfRange);
    }
    Ok(GridTrackSpan::new(start as usize, end as usize))
}

fn edge_track_indices(
    context: SubgridLineTransform,
    span_len: usize,
) -> Result<(usize, usize), SubgridTraversalError> {
    let local_start_line = map_line_to_ancestor(context, 1);
    let local_end_line = map_line_to_ancestor(context, span_len + 1);
    let (start_edge_index, end_edge_index) = if context.line_direction > 0 {
        (local_start_line - 1, local_end_line - 2)
    } else {
        (local_start_line - 2, local_end_line - 1)
    };
    if start_edge_index < 0 || end_edge_index < 0 {
        return Err(SubgridTraversalError::SpanOutOfRange);
    }
    Ok((start_edge_index as usize, end_edge_index as usize))
}

fn map_line_to_ancestor(context: SubgridLineTransform, line: usize) -> isize {
    context.line_offset + context.line_direction * line as isize
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ResolvedSubgridGap<S: LayoutScalar = Scalar> {
    Normal,
    Length(S),
}

impl<S: LayoutScalar> ResolvedSubgridGap<S> {
    const fn resolve(self, parent_gap: S) -> S {
        match self {
            Self::Normal => parent_gap,
            Self::Length(length) => length,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SubgridTrackInheritanceInput<'a, S: LayoutScalar = Scalar> {
    pub(super) parent_tracks: &'a [S],
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: ResolvedSubgridGap<S>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SubgridTrackInheritanceReport<S: LayoutScalar = Scalar> {
    pub(super) parent_span: GridTrackSpan,
    pub(super) copied_parent_tracks: Vec<S>,
    pub(super) reversed: bool,
    pub(super) after_reversal: Vec<S>,
    pub(super) start_mbp_removed: Vec<S>,
    pub(super) end_mbp_removed: Vec<S>,
    pub(super) parent_gap: S,
    pub(super) subgrid_gap: ResolvedSubgridGap<S>,
    pub(super) resolved_subgrid_gap: S,
    pub(super) gap_difference: S,
    pub(super) collapsed: Vec<bool>,
    pub(super) parent_boundary_gutters: Vec<S>,
    pub(super) final_active_boundary_after: Vec<bool>,
    pub(super) final_boundary_gutters: Vec<S>,
    pub(super) final_tracks: Vec<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OwnerTrackPlacementEntry<S: LayoutScalar = Scalar> {
    owner_track: usize,
    first_frame_translation: S,
    last_frame_translation: S,
}

impl<S: LayoutScalar> OwnerTrackPlacementEntry<S> {
    fn translation(self, role: AncestorBaselineRole) -> S {
        match role {
            AncestorBaselineRole::First => self.first_frame_translation,
            AncestorBaselineRole::Last => self.last_frame_translation,
        }
    }
}

/// A checked translation of the same selected owner point through actual used
/// track frames. Each scalar includes the geometry's gutters exactly once.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CheckedOwnerToCurrentPlacementMap<Node, S: LayoutScalar = Scalar> {
    owner: Node,
    current_grid: Node,
    current_axis: GridAxisKind,
    owner_axis: GridAxisKind,
    physical_axis: PhysicalAxis,
    owner_progression: PhysicalProgression,
    current_progression: PhysicalProgression,
    boundary_count: usize,
    tracks: Vec<OwnerTrackPlacementEntry<S>>,
}

#[derive(Clone, Copy)]
pub(super) struct OwnerToCurrentPlacementBoundaryInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) parent_grid: Node,
    pub(super) current_grid: Node,
    pub(super) parent_axis: GridAxisKind,
    pub(super) current_axis: GridAxisKind,
    pub(super) physical_axis: PhysicalAxis,
    pub(super) parent_progression: PhysicalProgression,
    pub(super) current_progression: PhysicalProgression,
    pub(super) parent_span: GridTrackSpan,
    pub(super) reversed: bool,
    pub(super) parent_first_frame_origins: &'a [S],
    pub(super) parent_last_frame_origins: &'a [S],
    pub(super) current_first_frame_origins: &'a [S],
    pub(super) current_last_frame_origins: &'a [S],
    pub(super) inherited: bool,
}

impl<Node: Copy + PartialEq, S: LayoutScalar> CheckedOwnerToCurrentPlacementMap<Node, S> {
    pub(super) fn identity(
        owner: Node,
        axis: GridAxisKind,
        physical_axis: PhysicalAxis,
        progression: PhysicalProgression,
        track_count: usize,
    ) -> Self {
        Self {
            owner,
            current_grid: owner,
            current_axis: axis,
            owner_axis: axis,
            physical_axis,
            owner_progression: progression,
            current_progression: progression,
            boundary_count: 0,
            tracks: (0..track_count)
                .map(|owner_track| OwnerTrackPlacementEntry {
                    owner_track,
                    first_frame_translation: S::ZERO,
                    last_frame_translation: S::ZERO,
                })
                .collect(),
        }
    }

    pub(super) fn compose(
        &self,
        input: OwnerToCurrentPlacementBoundaryInput<'_, Node, S>,
    ) -> Result<Self, InheritedCurrentGridBaselinePlacementError> {
        if input.parent_grid != self.current_grid {
            return Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch);
        }
        let track_count = input
            .parent_span
            .checked_len()
            .filter(|_| input.parent_span.end <= self.tracks.len())
            .ok_or(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange)?;
        if input.parent_first_frame_origins.len() != self.tracks.len()
            || input.parent_last_frame_origins.len() != self.tracks.len()
            || input.current_first_frame_origins.len() != track_count
            || input.current_last_frame_origins.len() != track_count
        {
            return Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange);
        }
        if !input.inherited
            || input.parent_axis != self.current_axis
            || input.physical_axis != self.physical_axis
            || input.parent_progression != self.current_progression
            || input.reversed != (input.parent_progression != input.current_progression)
        {
            return Err(InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping);
        }
        if input
            .parent_first_frame_origins
            .iter()
            .chain(input.parent_last_frame_origins)
            .chain(input.current_first_frame_origins)
            .chain(input.current_last_frame_origins)
            .any(|origin| !origin.is_finite())
        {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }
        let mut tracks = Vec::with_capacity(track_count);
        for local in 0..track_count {
            let parent = if input.reversed {
                input.parent_span.end - 1 - local
            } else {
                input.parent_span.start + local
            };
            let parent_entry = self
                .tracks
                .get(parent)
                .copied()
                .ok_or(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange)?;
            // A reversed current First is the parent's Last edge, and vice
            // versa. Carry the corresponding accumulated translation too.
            let first_parent_role = if input.reversed {
                AncestorBaselineRole::Last
            } else {
                AncestorBaselineRole::First
            };
            let last_parent_role = first_parent_role.opposite();
            let first_frame_translation = parent_entry.translation(first_parent_role)
                + input.current_first_frame_origins[local]
                - input.parent_first_frame_origins[parent];
            let last_frame_translation = parent_entry.translation(last_parent_role)
                + input.current_last_frame_origins[local]
                - input.parent_last_frame_origins[parent];
            if !first_frame_translation.is_finite() || !last_frame_translation.is_finite() {
                return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
            }
            tracks.push(OwnerTrackPlacementEntry {
                owner_track: parent_entry.owner_track,
                first_frame_translation,
                last_frame_translation,
            });
        }
        Ok(Self {
            owner: self.owner,
            current_grid: input.current_grid,
            current_axis: input.current_axis,
            owner_axis: self.owner_axis,
            physical_axis: self.physical_axis,
            owner_progression: self.owner_progression,
            current_progression: input.current_progression,
            boundary_count: self.boundary_count + 1,
            tracks,
        })
    }
    pub(super) const fn owner(&self) -> Node {
        self.owner
    }
    pub(super) const fn current_grid(&self) -> Node {
        self.current_grid
    }
    pub(super) const fn owner_axis(&self) -> GridAxisKind {
        self.owner_axis
    }
    pub(super) const fn current_axis(&self) -> GridAxisKind {
        self.current_axis
    }
    pub(super) const fn physical_axis(&self) -> PhysicalAxis {
        self.physical_axis
    }
    pub(super) const fn current_progression(&self) -> PhysicalProgression {
        self.current_progression
    }
    pub(super) const fn boundary_count(&self) -> usize {
        self.boundary_count
    }
    pub(super) fn track_count(&self) -> usize {
        self.tracks.len()
    }
    pub(super) fn owner_track_for_local(&self, local: usize) -> Option<usize> {
        self.tracks.get(local).map(|entry| entry.owner_track)
    }
    fn translation_for(&self, local: usize, role: AncestorBaselineRole) -> Option<S> {
        self.tracks.get(local).map(|entry| entry.translation(role))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CurrentGridDirectWitness<Node> {
    current_grid: Node,
    item: Node,
    axis: GridAxisKind,
    local_span: GridTrackSpan,
    selected_port: AncestorBaselineRole,
    subject_progression: PhysicalProgression,
    containing_progression: PhysicalProgression,
}

impl<Node: Copy> CurrentGridDirectWitness<Node> {
    pub(super) const fn new(
        current_grid: Node,
        item: Node,
        axis: GridAxisKind,
        local_span: GridTrackSpan,
        selected_port: AncestorBaselineRole,
        subject_progression: PhysicalProgression,
        containing_progression: PhysicalProgression,
    ) -> Self {
        Self {
            current_grid,
            item,
            axis,
            local_span,
            selected_port,
            subject_progression,
            containing_progression,
        }
    }

    #[cfg(test)]
    pub(super) fn set_local_span_for_test(&mut self, local_span: GridTrackSpan) {
        self.local_span = local_span;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct InheritedCurrentGridBaselinePlacement<Node, S: LayoutScalar = Scalar> {
    axis: GridAxisKind,
    physical_axis: PhysicalAxis,
    role: AncestorBaselineRole,
    selected_owner_track: usize,
    selected_local_track: usize,
    immutable_owner_target: S,
    frame_translation: S,
    translated_target: S,
    mapping_witness: CheckedOwnerToCurrentPlacementMap<Node, S>,
    current_grid_direct_witness: CurrentGridDirectWitness<Node>,
}

#[derive(Clone, Debug)]
pub(super) struct InheritedCurrentGridBaselinePlacementInput<Node, S: LayoutScalar = Scalar> {
    pub(super) axis: GridAxisKind,
    pub(super) reference: crate::GridBaselineReference,
    pub(super) physical_axis: PhysicalAxis,
    pub(super) mapping: CheckedOwnerToCurrentPlacementMap<Node, S>,
    pub(super) direct_witness: CurrentGridDirectWitness<Node>,
    pub(super) current_grid: Node,
    pub(super) item: Node,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InheritedCurrentGridBaselinePlacementError {
    AxisMismatch,
    PhysicalAxisMismatch,
    SpanOutOfRange,
    SelectedTrackOutOfRange,
    RoleTargetMismatch,
    OwnershipMismatch,
    UnusableInheritedMapping,
    NonFinite,
    MissingExecutedSource,
    ExecutedSourceMismatch,
    ReferenceMismatch,
    Selection(GridBaselineSelectionError),
}

impl<Node: Copy + PartialEq, S: LayoutScalar> InheritedCurrentGridBaselinePlacement<Node, S> {
    pub(super) fn try_derive(
        group: &AncestorBaselineGroup<Node, S>,
        input: InheritedCurrentGridBaselinePlacementInput<Node, S>,
    ) -> Result<Self, InheritedCurrentGridBaselinePlacementError> {
        if group.reference() != input.reference {
            return Err(InheritedCurrentGridBaselinePlacementError::ReferenceMismatch);
        }
        let owner_axis_matches = group.axis() == input.mapping.owner_axis();
        let current_axis_matches = input.mapping.current_axis() == input.axis;
        if owner_axis_matches != current_axis_matches {
            return Err(InheritedCurrentGridBaselinePlacementError::AxisMismatch);
        }
        if group.physical_axis() != input.physical_axis {
            return Err(InheritedCurrentGridBaselinePlacementError::PhysicalAxisMismatch);
        }

        let mapping = &input.mapping;
        let track_count = mapping.track_count();
        if track_count == 0
            || mapping
                .tracks
                .iter()
                .any(|entry| entry.owner_track >= group.track_count())
        {
            return Err(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange);
        }
        let selected_local_track = match input.direct_witness.selected_port {
            AncestorBaselineRole::First => input.direct_witness.local_span.start,
            AncestorBaselineRole::Last => input
                .direct_witness
                .local_span
                .end
                .checked_sub(1)
                .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?,
        };
        if input.direct_witness.local_span.checked_len().is_none()
            || input.direct_witness.local_span.end > track_count
            || selected_local_track >= track_count
        {
            return Err(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange);
        }
        let selected_owner_track = mapping
            .owner_track_for_local(selected_local_track)
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
        let role = input.direct_witness.selected_port.in_progression(
            input.direct_witness.subject_progression,
            mapping.current_progression(),
        );
        let owner_role = input.direct_witness.selected_port.in_progression(
            input.direct_witness.subject_progression,
            mapping.owner_progression,
        );
        let target = group
            .target_record(owner_role, selected_owner_track)
            .filter(|target| {
                target.role() == owner_role
                    && target.selected_ancestor_track() == selected_owner_track
                    && target.selected_owner_edge()
                        == match owner_role {
                            AncestorBaselineRole::First => BaselineOwnerEdge::Start,
                            AncestorBaselineRole::Last => BaselineOwnerEdge::End,
                        }
            })
            .ok_or(InheritedCurrentGridBaselinePlacementError::RoleTargetMismatch)?;

        if input.direct_witness.current_grid != input.current_grid
            || input.direct_witness.item != input.item
            || mapping.owner() != group.owner()
            || mapping.current_grid() != input.current_grid
        {
            return Err(InheritedCurrentGridBaselinePlacementError::OwnershipMismatch);
        }
        if mapping.current_axis() != input.axis
            || mapping.owner_axis() != group.axis()
            || mapping.physical_axis() != input.physical_axis
            || input.direct_witness.axis != input.axis
            || input.direct_witness.containing_progression != mapping.current_progression()
            || (mapping.boundary_count() == 0 && mapping.owner() != mapping.current_grid())
            || (mapping.boundary_count() > 0 && mapping.owner() == mapping.current_grid())
        {
            return Err(InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping);
        }

        let immutable_owner_target = target.finite_owner_logical_target();
        let frame_translation = mapping
            .translation_for(selected_local_track, role)
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
        if !immutable_owner_target.is_finite() || !frame_translation.is_finite() {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }
        let translated_target = immutable_owner_target + frame_translation;
        if !translated_target.is_finite() {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }

        Ok(Self {
            axis: input.axis,
            physical_axis: input.physical_axis,
            role,
            selected_owner_track,
            selected_local_track,
            immutable_owner_target,
            frame_translation,
            translated_target,
            mapping_witness: input.mapping,
            current_grid_direct_witness: input.direct_witness,
        })
    }

    #[cfg(test)]
    pub(super) const fn selected_local_track(&self) -> usize {
        self.selected_local_track
    }

    #[cfg(test)]
    pub(super) const fn immutable_owner_target(&self) -> S {
        self.immutable_owner_target
    }

    #[cfg(test)]
    pub(super) const fn frame_translation(&self) -> S {
        self.frame_translation
    }

    pub(super) const fn translated_target(&self) -> S {
        self.translated_target
    }
}

#[cfg(test)]
type InheritedPlacementPreparationInput<'a, Node, S> = (
    &'a AncestorBaselineGroup<Node, S>,
    InheritedCurrentGridBaselinePlacementInput<Node, S>,
);

#[cfg(test)]
pub(super) fn prepare_inherited_current_grid_baseline_placements<
    Node: Copy + PartialEq,
    S: LayoutScalar,
    const N: usize,
>(
    inputs: [InheritedPlacementPreparationInput<'_, Node, S>; N],
) -> Result<
    Vec<InheritedCurrentGridBaselinePlacement<Node, S>>,
    InheritedCurrentGridBaselinePlacementError,
> {
    inputs
        .into_iter()
        .map(|(group, input)| InheritedCurrentGridBaselinePlacement::try_derive(group, input))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubgridTrackInheritanceError {
    EmptyTrackList,
    SpanOutOfRange,
}

#[cfg(test)]
pub(super) fn inherit_subgrid_tracks<S: LayoutScalar>(
    input: SubgridTrackInheritanceInput<'_, S>,
) -> Result<SubgridTrackInheritanceReport<S>, SubgridTrackInheritanceError> {
    inherit_subgrid_tracks_with_geometry(input, None)
}

pub(super) fn inherit_subgrid_tracks_with_geometry<S: LayoutScalar>(
    input: SubgridTrackInheritanceInput<'_, S>,
    parent_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<SubgridTrackInheritanceReport<S>, SubgridTrackInheritanceError> {
    let span_len = input
        .parent_span
        .checked_len()
        .ok_or(SubgridTrackInheritanceError::SpanOutOfRange)?;
    if input.parent_tracks.is_empty() {
        return Err(SubgridTrackInheritanceError::EmptyTrackList);
    }
    if input.parent_span.start == 0
        || input.parent_span.end > input.parent_tracks.len() + 1
        || span_len == 0
        || parent_geometry
            .is_some_and(|geometry| geometry.sizes().len() != input.parent_tracks.len())
    {
        return Err(SubgridTrackInheritanceError::SpanOutOfRange);
    }

    let start_index = input.parent_span.start - 1;
    let end_index = input.parent_span.end - 1;
    let copied_parent_tracks = input.parent_tracks[start_index..end_index].to_vec();
    let mut after_reversal = copied_parent_tracks.clone();
    let mut parent_gutters = parent_geometry.map_or_else(
        || OrdinaryGridAxisGuttersOf::new(copied_parent_tracks.len(), &[], input.parent_gap),
        |geometry| {
            OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
                copied_parent_tracks.len(),
                &geometry.collapsed()[start_index..end_index],
                &geometry.active_boundary_after()[start_index..end_index.saturating_sub(1)],
                &geometry.gutter_after()[start_index..end_index.saturating_sub(1)],
            )
        },
    );
    if input.reversed {
        after_reversal.reverse();
        parent_gutters = parent_gutters.reversed();
    }
    let collapsed = parent_gutters.collapsed().to_vec();
    let parent_boundary_gutters = parent_gutters.gutter_after().to_vec();

    let mut start_mbp_removed = after_reversal.clone();
    consume_track_space(
        &mut start_mbp_removed,
        input.start_mbp,
        TrackSpaceEdge::Start,
    );

    let mut end_mbp_removed = start_mbp_removed.clone();
    consume_track_space(&mut end_mbp_removed, input.end_mbp, TrackSpaceEdge::End);

    let resolved_subgrid_gap = input.subgrid_gap.resolve(input.parent_gap);
    let gap_difference = (resolved_subgrid_gap - input.parent_gap) / S::from_f64(2.0);
    let final_active_boundary_after = parent_gutters.active_boundary_after().to_vec();
    let final_boundary_gutters = final_active_boundary_after
        .iter()
        .map(|active| {
            if *active {
                resolved_subgrid_gap
            } else {
                S::ZERO
            }
        })
        .collect::<Vec<_>>();
    let mut final_tracks = end_mbp_removed.clone();
    for (edge, (parent_gutter, final_gutter)) in parent_boundary_gutters
        .iter()
        .copied()
        .zip(final_boundary_gutters.iter().copied())
        .enumerate()
    {
        let boundary_difference = (final_gutter - parent_gutter) / S::from_f64(2.0);
        final_tracks[edge] = (final_tracks[edge] - boundary_difference).max(S::ZERO);
        final_tracks[edge + 1] = (final_tracks[edge + 1] - boundary_difference).max(S::ZERO);
    }

    Ok(SubgridTrackInheritanceReport {
        parent_span: input.parent_span,
        copied_parent_tracks,
        reversed: input.reversed,
        after_reversal,
        start_mbp_removed,
        end_mbp_removed,
        parent_gap: input.parent_gap,
        subgrid_gap: input.subgrid_gap,
        resolved_subgrid_gap,
        gap_difference,
        collapsed,
        parent_boundary_gutters,
        final_active_boundary_after,
        final_boundary_gutters,
        final_tracks,
    })
}

#[derive(Clone, Copy)]
enum TrackSpaceEdge {
    Start,
    End,
}

fn consume_track_space<S: LayoutScalar>(tracks: &mut [S], mut amount: S, edge: TrackSpaceEdge) {
    match edge {
        TrackSpaceEdge::Start => {
            for track in tracks.iter_mut() {
                if amount <= S::ZERO {
                    break;
                }
                let removed = (*track).min(amount);
                *track = *track - removed;
                amount = amount - removed;
            }
        }
        TrackSpaceEdge::End => {
            for track in tracks.iter_mut().rev() {
                if amount <= S::ZERO {
                    break;
                }
                let removed = (*track).min(amount);
                *track = *track - removed;
                amount = amount - removed;
            }
        }
    }
}

pub(super) fn collect_subgrid_report<'input, Node: Copy, S: LayoutScalar>(
    parent_style: &GridContainerProjection<'_, S>,
    placements: &GridPlacementContext<'input, Node, S>,
) -> GridSubgridReport<Node> {
    let parent_is_lanes = parent_style
        .display
        .establishes_grid_lanes_formatting_context();
    let items = placements
        .children
        .iter()
        .copied()
        .enumerate()
        .map(|(index, child)| {
            let child_input = placements.child_input(index);
            let placement = placements.items[index];
            let axis_report = |axis| {
                let mut report = subgrid_axis_report(parent_style, child_input, axis);
                let placement = match report.mapping.parent_axis {
                    GridAxisKind::Column => placement.column,
                    GridAxisKind::Row => placement.row,
                };
                report.inherit_parent_names = !parent_is_lanes || has_definite_line(placement);
                report
            };
            SubgridItemReport {
                node: child,
                column: axis_report(GridAxisKind::Column),
                row: axis_report(GridAxisKind::Row),
            }
        })
        .collect();

    GridSubgridReport { items }
}

pub(super) struct GridSubgridIntrinsicTraversalInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(super) grid: IntrinsicGrid<'input, 'a, Node, S>,
    pub(super) axis: GridAxisKind,
    pub(super) containing_flow_axes: crate::geometry::FlowAxes,
    pub(super) children: &'a [Node],
    pub(super) placed_areas: &'a [Option<GridArea<S>>],
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<'input, Node, S>,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) parent_gap: Size<S>,
    pub(super) column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) column_sizes: &'a [S],
    pub(super) row_sizes: &'a [S],
    pub(super) container_size: Size<Option<S>>,
    pub(super) intrinsic_min_track_facts: IntrinsicMinTrackFacts<'a>,
    pub(super) query: IntrinsicGridQuery<'a, S>,
}

#[expect(
    clippy::type_complexity,
    reason = "nested traversal preserves both session errors and existing traversal outcomes"
)]
pub(super) fn collect_grid_subgrid_intrinsic_traversal<'input, Tree, M>(
    tree: &mut Tree,
    input: GridSubgridIntrinsicTraversalInput<'input, '_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Result<
        SubgridTraversalReport<'input, <Tree as Traverse>::Node, Tree::Scalar>,
        SubgridTraversalError,
    >,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<'input, M>,
{
    let column_geometry = input.column_gutters.map(|gutters| {
        UsedGridAxisGeometryOf::from_sizing_gutters(input.column_sizes.to_vec(), gutters)
    });
    let row_geometry = input.row_gutters.map(|gutters| {
        UsedGridAxisGeometryOf::from_sizing_gutters(input.row_sizes.to_vec(), gutters)
    });
    let mut fold = SubgridIntrinsicFold::new(input.intrinsic_min_track_facts);
    for (index, ((child, area), item_report)) in input
        .children
        .iter()
        .copied()
        .zip(input.placed_areas.iter().copied())
        .zip(input.subgrid_report.items.iter().copied())
        .enumerate()
    {
        let Some(area) = area else {
            continue;
        };
        let child_input = input.placements.child_input(index);
        let child_style = child_input.item();
        if !is_in_flow_grid_child(child_style) {
            continue;
        }
        let Some(child_axis) = inherited_subgrid_axis_for_parent_axis(item_report, input.axis)
        else {
            continue;
        };
        let minimum_source = SubgridParentTrackSource {
            baseline_references: input.grid.style.baseline_references,
            flow_axes: input.containing_flow_axes,
            gap: input.parent_gap,
            columns: column_geometry.as_ref(),
            rows: row_geometry.as_ref(),
            minimum_functions: input.grid.minimum_functions,
            algorithm: Some(sizing_algorithm_for_grid_display(input.grid.style.display)),
            lane_grid_axis: input
                .grid
                .style
                .display
                .establishes_grid_lanes_formatting_context()
                .then(|| grid_axis_for_grid_lanes(input.grid.style)),
            allocated: LogicalSizeOf::new(input.query.columns, input.query.rows),
        };
        let (placement, track_count, gutters) = match input.axis {
            GridAxisKind::Column => (
                input.placements.items[index].column,
                input.column_sizes.len(),
                input.column_gutters,
            ),
            GridAxisKind::Row => (
                input.placements.items[index].row,
                input.row_sizes.len(),
                input.row_gutters,
            ),
        };
        let expand_lane_wrapper = minimum_source.lane_grid_axis.is_none()
            && automatic_lane_subgrid(child_input, placement, child_axis);
        let collapsed = gutters
            .map(OrdinaryGridAxisGuttersOf::collapsed)
            .unwrap_or(&[]);
        for (area, traversal) in subgrid_root_intrinsic_areas(
            area,
            input.axis,
            track_count,
            collapsed,
            expand_lane_wrapper,
            false,
        ) {
            let area_size = intrinsic_traversal_area_size(
                area,
                input.column_sizes,
                input.row_sizes,
                input.container_size,
                minimum_source,
            )
            .map_err(|_| minimum_source_error(child))?;
            let grid = input.grid;
            let prepared = intrinsic_stage_child_input::<Tree, M>(
                child,
                &IntrinsicGridChildInput {
                    child_style,
                    child_container: child_input.nested_container(),
                    grid: IntrinsicGrid {
                        query: input.query,
                        ..grid
                    },
                    area: GridArea {
                        size: LogicalSizeOf::new(area_size.width, area_size.height),
                        ..area
                    },
                    columns: input.column_sizes,
                    rows: input.row_sizes,
                    subgrid_item: Some(item_report),
                    input: ComputeInputOf::for_child(
                        RunMode::ComputeSize,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        grid.constants.node_inner_size,
                        crate::ContainingLayoutContext::new(
                            grid.constants.flow_axes,
                            crate::ParentFormattingContext::Grid,
                        ),
                        SizingRequestOf::new(
                            input.query.available,
                            grid.constants.child_orthogonal_fallback,
                        ),
                    ),
                },
            )?;

            subgrid_traversal_child(
                tree,
                child,
                child_style,
                child_input.nested_container(),
                area,
                area_size,
                item_report,
                child_axis,
                input.containing_flow_axes,
                input.parent_gap,
                column_geometry.as_ref(),
                row_geometry.as_ref(),
                input.named_columns,
                input.named_rows,
                input.area_facts,
                match input.axis {
                    GridAxisKind::Column => child_style.justify_self,
                    GridAxisKind::Row => child_style.align_self,
                }
                .unwrap_or(AlignItems::Stretch),
                input.query,
                minimum_source,
                prepared,
                traversal,
                &mut fold,
                SubgridTraversalContext::ROOT,
            )?;
        }
    }

    Ok(fold.finish())
}

fn automatic_lane_subgrid<S: LayoutScalar>(
    child: &GridChildInput<'_, S>,
    placement: GridPlacement,
    axis: GridAxisKind,
) -> bool {
    !has_definite_line(placement)
        && child.nested_container().is_some_and(|container| {
            let style = container.projection();
            style.display.establishes_grid_lanes_formatting_context()
                && grid_axis_for_grid_lanes(&style) == axis
        })
}

fn subgrid_root_intrinsic_areas<S: LayoutScalar>(
    area: GridArea<S>,
    axis: GridAxisKind,
    track_count: usize,
    collapsed: &[bool],
    expand_lane_wrapper: bool,
    lane_track: bool,
) -> impl Iterator<Item = (GridArea<S>, GridMinimumWrapperTraversal)> + '_ {
    let (placed_start, placed_end) = match axis {
        GridAxisKind::Column => (area.column, area.column_end),
        GridAxisKind::Row => (area.row, area.row_end),
    };
    let span = placed_end.saturating_sub(placed_start);
    let last_candidate = track_count
        .checked_sub(span)
        .filter(|_| expand_lane_wrapper && span > 0);
    let candidates = last_candidate
        .into_iter()
        .flat_map(|last| 0..=last)
        .filter(move |&start| {
            start != placed_start && lanes::lane_candidate_is_active(collapsed, start, span)
        })
        .map(move |start| {
            let mut candidate = area;
            match axis {
                GridAxisKind::Column => {
                    (candidate.column, candidate.column_end) = (start, start + span)
                }
                GridAxisKind::Row => (candidate.row, candidate.row_end) = (start, start + span),
            }
            (candidate, GridMinimumWrapperTraversal::SharedLaneCandidate)
        });
    std::iter::once((area, GridMinimumWrapperTraversal::Shared { lane_track })).chain(candidates)
}

#[expect(
    clippy::too_many_arguments,
    reason = "subgrid traversal child creation carries explicit grid layout phase inputs"
)]
fn subgrid_traversal_child<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &GridItemProjection<'input, Tree::Scalar>,
    nested_container: Option<&GridContainerInput<'input, Tree::Scalar>>,
    area: GridArea<Tree::Scalar>,
    area_size: Size<Tree::Scalar>,
    item_report: SubgridItemReport<<Tree as Traverse>::Node>,
    queried_axis: GridAxisKind,
    containing_flow_axes: crate::geometry::FlowAxes,
    parent_semantic_gap: Size<Tree::Scalar>,
    parent_column_geometry: Option<&UsedGridAxisGeometryOf<Tree::Scalar>>,
    parent_row_geometry: Option<&UsedGridAxisGeometryOf<Tree::Scalar>>,
    parent_named_columns: &NamedGridLines,
    parent_named_rows: &NamedGridLines,
    parent_area_facts: Option<&GridAreaNameFacts>,
    align_self: AlignItems,
    query: IntrinsicGridQuery<'_, Tree::Scalar>,
    minimum_source: SubgridParentTrackSource<'_, Tree::Scalar>,
    prepared: ComputeInputOf<Tree::Scalar>,
    traversal: GridMinimumWrapperTraversal,
    fold: &mut SubgridIntrinsicFold<'input, '_, Tree::Node, Tree::Scalar>,
    mut context: SubgridTraversalContext,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let axis_report = match queried_axis {
        GridAxisKind::Column => item_report.column,
        GridAxisKind::Row => item_report.row,
    };
    let min_frame = minimum_source.frame(node, axis_report.mapping.parent_axis, area, prepared)?;
    let Some(nested_container) = nested_container else {
        fold.leaf(
            SubgridTraversalLeaf {
                node,
                style: style.clone(),
                span_in_parent: area_span(area, axis_report.mapping.parent_axis),

                align_self,
                min_frame,
            },
            context,
            None,
        );
        return Ok(());
    };
    let nested_container_style = nested_container.projection();

    let axis = if axis_report.can_inherit() {
        SubgridTraversalAxis::Inherited
    } else {
        SubgridTraversalAxis::Standalone
    };
    let (reversed, parent_axis) = match axis {
        SubgridTraversalAxis::Inherited => (
            axis_report.mapping.reversed,
            axis_report.mapping.parent_axis,
        ),
        SubgridTraversalAxis::Standalone => (false, axis_report.mapping.parent_axis),
    };
    // The track owner remains in force across ordinary Grid wrappers that
    // inherit this axis. The immediate parent axis still controls geometry.
    let lane_track =
        traversal.uses_lane_track_sizing() || minimum_source.lane_grid_axis == Some(parent_axis);
    let candidate = matches!(traversal, GridMinimumWrapperTraversal::SharedLaneCandidate);
    let traversal = if candidate {
        traversal
    } else {
        GridMinimumWrapperTraversal::Shared { lane_track }
    };
    let span_in_parent = area_span(area, parent_axis);
    let parent_axis_gap = axis_size(parent_semantic_gap, parent_axis);
    let parent_layout_gap = minimum_source.gap;
    let physical_area_size = grid_area_physical_size(
        containing_flow_axes,
        LogicalSizeOf::new(area_size.width, area_size.height),
    );
    let property_basis = prepared.parent();
    let property_flow = prepared.containing_flow_axes();
    let resolved_margin = property_flow
        .zip_physical_edges_with_inline_extent(style.margin, property_basis, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, node)?
        .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let resolved_border = property_flow
        .zip_physical_edges_with_inline_extent(style.border, property_basis, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let resolved_padding = property_flow
        .zip_physical_edges_with_inline_extent(style.padding, property_basis, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, node)?;
    let current_flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
    // Intrinsic track bases can be provisional zero. Only the actual known or
    // authored child size may clamp its reservation; track estimates are not
    // qualified border-box dimensions.
    // This is the same standalone-axis intrinsic probe as the descendant
    // traversal below. Preserve its local minimum overlay while qualifying the
    // gutter; the original authored minimum is resolved by the real phase.
    let inherited = LogicalSizeOf::new(
        item_report.column.can_inherit(),
        item_report.row.can_inherit(),
    );
    let intrinsic_minimum = standalone_intrinsic_minimum_phases(&nested_container_style, inherited);
    let mut gutter_container = *nested_container;
    gutter_container.suppress_intrinsic_minimum(intrinsic_minimum.map(|minimum| minimum.is_some()));
    let gutter_style = gutter_container.projection();
    let gutter_input = prepared.with_run_mode(RunMode::ComputeSize);
    let traversal_constants = if lane_track {
        lane_subgrid_wrapper_constants::<Tree, M>(
            tree,
            node,
            &gutter_style,
            gutter_input,
            inherited,
        )?
    } else {
        Constants::with_intrinsic_minimum::<Tree, M>(
            tree,
            node,
            &gutter_style,
            gutter_input,
            Size::NONE,
        )?
    };
    let mut resolved_scrollbar_gutter = traversal_constants.scrollbar_gutter;
    if query.observe_auto
        && style.scrollbar_width.get() > Tree::Scalar::ZERO
        && (style.overflow.x() == Overflow::Auto || style.overflow.y() == Overflow::Auto)
    {
        let parent_columns = parent_column_geometry.map_or(&[][..], |geometry| geometry.sizes());
        let parent_rows = parent_row_geometry.map_or(&[][..], |geometry| geometry.sizes());
        let probe_source = SubgridChildParentContextInput {
            parent_baseline_references: minimum_source.baseline_references,
            item: item_report,
            child_style: style,
            child_container: Some(gutter_container),
            area,
            content_box_size: (physical_area_size
                - resolved_margin.sum_axes()
                - resolved_border.sum_axes()
                - resolved_padding.sum_axes())
            .max(Size::ZERO),
            columns: parent_columns,
            rows: parent_rows,
            minimum_functions: minimum_source.minimum_functions,
            gap: LogicalSizeOf::new(parent_layout_gap.width, parent_layout_gap.height),
            parent_named_columns,
            parent_named_rows,
            parent_area_facts,
            margin: resolved_margin.map(Some),
            border: resolved_border,
            padding: resolved_padding,
            scrollbar_gutter: Edges::ZERO,
        };
        let probe_input = prepared.with_run_mode(RunMode::PerformLayout);
        let observation = tree.observe_grid_overflow(
            node,
            GridOverflowProbe {
                layout: GridOverflowLayout::Intrinsic(GridIntrinsicProbe {
                    input: probe_input,
                    stage: query.stage,
                    kind: GridIntrinsicProbeKind::ContentContribution(queried_axis),
                    intrinsic_minimum: Size::NONE,
                    source: probe_source,
                    parent_column_geometry,
                    parent_row_geometry,
                    parent_flow: containing_flow_axes,
                }),
            },
        )?;
        resolved_scrollbar_gutter = match observation {
            GridOverflowObservation::Intrinsic(observation) => observation
                .effective_gutter
                .unwrap_or(traversal_constants.scrollbar_gutter),
            GridOverflowObservation::Children(_) => {
                unreachable!("an intrinsic probe returns its canonical gutter")
            }
        };
    }
    let [margins, border, padding, scrollbar_gutter] = [
        resolved_margin,
        resolved_border,
        resolved_padding,
        resolved_scrollbar_gutter,
    ]
    .map(|edges| traversal_axis_edges(queried_axis, current_flow_axes, edges));
    let physical_content_box_size = (physical_area_size
        - resolved_margin.sum_axes()
        - resolved_border.sum_axes()
        - resolved_padding.sum_axes()
        - resolved_scrollbar_gutter.sum_axes())
    .max(Size::ZERO);
    let logical_content_box_size = current_flow_axes.logical_size(physical_content_box_size);
    let content_box_size = Size::new(
        logical_content_box_size.inline,
        logical_content_box_size.block,
    );
    if axis == SubgridTraversalAxis::Standalone {
        let parent_columns = parent_column_geometry.map_or(&[][..], |geometry| geometry.sizes());
        let parent_rows = parent_row_geometry.map_or(&[][..], |geometry| geometry.sizes());
        let standalone_parent_context = IntrinsicSubgridSource::new(
            SubgridChildParentContextInput {
                parent_baseline_references: minimum_source.baseline_references,
                item: item_report,
                child_style: style,
                child_container: Some(*nested_container),
                area,
                content_box_size: physical_content_box_size,
                columns: parent_columns,
                rows: parent_rows,
                minimum_functions: minimum_source.minimum_functions,
                gap: LogicalSizeOf::new(parent_layout_gap.width, parent_layout_gap.height),
                parent_named_columns,
                parent_named_rows,
                parent_area_facts,
                margin: resolved_margin.map(Some),
                border: resolved_border,
                padding: resolved_padding,
                scrollbar_gutter: resolved_scrollbar_gutter,
            },
            parent_column_geometry,
            parent_row_geometry,
            containing_flow_axes,
            query.stage,
        )
        .map_err(|error| subgrid_child_context_error(node, error))?;
        fold.leaf(
            SubgridTraversalLeaf {
                node,
                style: style.clone(),
                span_in_parent,
                align_self,
                min_frame,
            },
            context,
            Some(Box::new(standalone_parent_context)),
        );
        return Ok(());
    }

    let subgrid_gap = Size::new(
        resolved_subgrid_axis_gap(
            &nested_container_style,
            GridAxisKind::Column,
            item_report.column,
            parent_semantic_gap,
            traversal_constants.node_inner_size,
        )
        .map_err(|status| crate::error::value_resolution_error(node, status))?,
        resolved_subgrid_axis_gap(
            &nested_container_style,
            GridAxisKind::Row,
            item_report.row,
            parent_semantic_gap,
            traversal_constants.node_inner_size,
        )
        .map_err(|status| crate::error::value_resolution_error(node, status))?,
    );
    let subgrid_axis_gap = axis_size(subgrid_gap, queried_axis);
    let wrapper_source = SubgridChildParentContextInput {
        parent_baseline_references: minimum_source.baseline_references,
        item: item_report,
        child_style: style,
        child_container: Some(*nested_container),
        area,
        content_box_size: physical_content_box_size,
        columns: minimum_source
            .allocated
            .inline
            .unwrap_or_else(|| parent_column_geometry.map_or(&[], UsedGridAxisGeometryOf::sizes)),
        rows: minimum_source
            .allocated
            .block
            .unwrap_or_else(|| parent_row_geometry.map_or(&[], UsedGridAxisGeometryOf::sizes)),
        minimum_functions: minimum_source.minimum_functions,
        gap: LogicalSizeOf::new(minimum_source.gap.width, minimum_source.gap.height),
        parent_named_columns,
        parent_named_rows,
        parent_area_facts,
        margin: resolved_margin.map(Some),
        border: resolved_border,
        padding: resolved_padding,
        scrollbar_gutter: resolved_scrollbar_gutter,
    };
    let subgrid = SubgridWrapperFrame {
        reversed,
        span_in_parent,
        margins,
        border,
        padding,
        scrollbar_gutter,
        parent_gap: parent_axis_gap,
        subgrid_gap: subgrid_axis_gap,
    };
    context.admit_wrapper_floor = !candidate;
    context.candidate_path |= candidate;
    let frame_checkpoint = fold.report.adjustments.frames.len();
    let leaf_checkpoint = fold.report.leaves.len();
    let context = fold.enter(&subgrid, context, false);
    let child_count = subgrid_traversal_children(
        tree,
        node,
        nested_container,
        content_box_size,
        queried_axis,
        containing_flow_axes,
        subgrid_gap,
        query,
        minimum_source,
        prepared,
        traversal_constants.node_inner_size,
        wrapper_source,
        traversal,
        fold,
        context,
    )?;
    fold.finish_wrapper(
        context,
        span_in_parent.checked_len().unwrap_or(0),
        child_count == 0,
        frame_checkpoint,
        leaf_checkpoint,
    );
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "subgrid traversal recursion preserves retained report and oracle parity inputs"
)]
fn subgrid_traversal_children<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    nested_container: &GridContainerInput<'input, Tree::Scalar>,
    content_box_size: Size<Tree::Scalar>,
    queried_axis: GridAxisKind,
    containing_flow_axes: crate::geometry::FlowAxes,
    gap: Size<Tree::Scalar>,
    query: IntrinsicGridQuery<'_, Tree::Scalar>,
    minimum_source: SubgridParentTrackSource<'_, Tree::Scalar>,
    prepared: ComputeInputOf<Tree::Scalar>,
    gap_basis: Size<Option<Tree::Scalar>>,
    wrapper_source: SubgridChildParentContextInput<'input, '_, Tree::Node, Tree::Scalar>,
    traversal: GridMinimumWrapperTraversal,
    fold: &mut SubgridIntrinsicFold<'input, '_, Tree::Node, Tree::Scalar>,
    context: SubgridTraversalContext,
) -> LayoutResultOf<Tree::Node, usize, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let parent_source = SubgridParentTrackSource {
        gap,
        ..minimum_source
    };
    let item_report = wrapper_source.item;
    let allocated = |report: SubgridAxisReport| {
        report.can_inherit()
            && match report.mapping.parent_axis {
                GridAxisKind::Column => minimum_source.allocated.inline.is_some(),
                GridAxisKind::Row => minimum_source.allocated.block.is_some(),
            }
    };
    let axis_context = |axis, report| {
        if allocated(report) {
            // A parent's allocated span becomes this wrapper's own allocation
            // only after the same MBP/gap adjustment used by final placement.
            subgrid_child_axis_parent_context_with_geometry(
                wrapper_source,
                axis,
                gap_basis,
                minimum_source.columns,
                minimum_source.rows,
            )
        } else {
            // Unallocated axes retain provisional geometry and no allocation
            // authority; in particular a zero placeholder is not a used zero.
            intrinsic_subgrid_axis_parent_context(
                report,
                wrapper_source.area,
                gap,
                parent_source,
                wrapper_source.parent_named_columns,
                wrapper_source.parent_named_rows,
                wrapper_source.parent_area_facts,
            )
        }
        .map_err(|error| subgrid_child_context_error(node, error))
    };
    let parent_context = GridParentContext {
        columns: axis_context(GridAxisKind::Column, item_report.column)?,
        rows: axis_context(GridAxisKind::Row, item_report.row)?,
    };
    let query = IntrinsicGridQuery {
        columns: allocated(item_report.column).then(|| {
            parent_context
                .columns
                .as_ref()
                .expect("inherited column context")
                .tracks
                .as_slice()
        }),
        rows: allocated(item_report.row).then(|| {
            parent_context
                .rows
                .as_ref()
                .expect("inherited row context")
                .tracks
                .as_slice()
        }),
        ..query
    };
    let mut child_count = 0;
    visit_grid_minimum_wrapper_children::<_, _, ()>(
        tree,
        node,
        GridMinimumWrapperInput {
            axis: queried_axis,
            container: nested_container,
            parent_context: &parent_context,
            prepared,
            containing_flow_axes,
            content_box_estimate: Some(content_box_size),
            traversal,
            query,
        },
        |tree, child| {
            let execution = child.execution;
            subgrid_traversal_child(
                tree,
                child.node,
                execution.child_style,
                execution.child_container,
                execution.area,
                child.area_size,
                execution
                    .subgrid_item
                    .expect("wrapper placement keeps its actual item report"),
                child.child_axis,
                child.minimum_source.flow_axes,
                gap,
                Some(child.column_geometry),
                Some(child.row_geometry),
                execution.grid.named_columns,
                execution.grid.named_rows,
                execution.grid.area_facts,
                child.align_self,
                execution.grid.query,
                child.minimum_source,
                child.prepared,
                child.traversal,
                fold,
                context,
            )?;
            child_count += 1;
            Ok(None)
        },
    )?;
    Ok(child_count)
}

pub(super) fn grid_minimum_wrapper_contained<S: LayoutScalar>(
    input: &NodeInputOf<S>,
    axis: GridAxisKind,
) -> bool {
    let containment = size_containment(input);
    containment == crate::SizeContainment::BothAxes
        || (containment == crate::SizeContainment::InlineAxis && axis == GridAxisKind::Column)
}

pub(super) fn lane_subgrid_wrapper_constants<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    style: &GridContainerProjection<'_, Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
    inherited: LogicalSizeOf<bool>,
) -> LayoutResultOf<Tree::Node, Constants<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if !inherited.inline && !inherited.block {
        return Constants::with_intrinsic_minimum(tree, node, style, input, Size::NONE);
    }

    // The inherited axis uses its parent's sizing authority. An authored size
    // cannot supply a definite percentage basis or clamp a scrollbar gutter
    // there. A qualified known extent remains in the input.
    let flow = FlowAxes::new(style.writing_mode, style.direction);
    let axes = flow.physical_size(inherited);
    let size = style.size.clone().zip_map(axes, |size, inherited| {
        if inherited {
            PreferredSizeOf::AUTO
        } else {
            size
        }
    });
    let min_size =
        style.min_size.clone().zip_map(
            axes,
            |size, inherited| {
                if inherited { MinSizeOf::AUTO } else { size }
            },
        );
    let max_size =
        style.max_size.clone().zip_map(
            axes,
            |size, inherited| {
                if inherited { MaxSizeOf::NONE } else { size }
            },
        );
    let mut projected = *style;
    projected.size = &size;
    projected.min_size = &min_size;
    projected.max_size = &max_size;
    // Do not transfer a standalone authored constraint to the inherited axis.
    projected.aspect_ratio = &None;
    projected.common.size = &size;
    projected.common.min_size = &min_size;
    projected.common.max_size = &max_size;
    projected.common.aspect_ratio = &None;
    Constants::with_intrinsic_minimum(tree, node, &projected, input, Size::NONE)
}

pub(super) fn visit_grid_minimum_wrapper_children<'input, Tree, M, T>(
    tree: &mut Tree,
    node: Tree::Node,
    request: GridMinimumWrapperInput<'input, '_, Tree::Node, Tree::Scalar>,
    mut visit: impl FnMut(
        &mut Tree,
        GridMinimumWrapperChild<'input, '_, Tree::Node, Tree::Scalar>,
    ) -> LayoutResultOf<Tree::Node, Option<T>, Tree::Scalar, M>,
) -> LayoutResultOf<Tree::Node, Vec<T>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let containment = size_containment(tree.node_input(node));
    if grid_minimum_wrapper_contained(tree.node_input(node), request.axis) {
        return Ok(Vec::new());
    }
    let GridMinimumWrapperInput {
        axis: queried_axis,
        container: nested_container,
        parent_context,
        prepared,
        containing_flow_axes,
        content_box_estimate,
        traversal,
        query,
    } = request;
    let sizing_input = prepared.with_run_mode(RunMode::ComputeSize);
    let mut container_style = nested_container.projection();
    container_style.baseline_references =
        effective_grid_baseline_references(container_style.baseline_references, parent_context);
    let inherited = LogicalSizeOf::new(
        parent_context.columns.is_some(),
        parent_context.rows.is_some(),
    );
    let intrinsic_minimum = standalone_intrinsic_minimum_phases(&container_style, inherited);
    let intrinsic_axes = intrinsic_minimum.map(|minimum| minimum.is_some());
    let mut traversal_input = *nested_container;
    traversal_input.suppress_intrinsic_minimum(intrinsic_axes);
    let mut traversal_style = traversal_input.projection();
    traversal_style.baseline_references = container_style.baseline_references;
    let lane_candidates = matches!(traversal, GridMinimumWrapperTraversal::LaneCandidates);
    let constants = if traversal.uses_lane_track_sizing() {
        lane_subgrid_wrapper_constants::<Tree, M>(
            tree,
            node,
            &traversal_style,
            sizing_input,
            inherited,
        )?
    } else {
        Constants::with_intrinsic_minimum::<Tree, M>(
            tree,
            node,
            &traversal_style,
            sizing_input,
            Size::NONE,
        )?
    };
    let stage = query
        .stage
        .mapped(containing_flow_axes, constants.flow_axes);
    let contained_inline = if containment == crate::SizeContainment::InlineAxis
        && parent_context.columns.is_none()
        && stage.has_resolved_axis(GridAxisKind::Column)
    {
        let empty = compute_empty_grid(
            node,
            sizing_input,
            |_| Ok(parent_context.clone()),
            GridInvocation {
                container: (*nested_container).with_inherited_axes(inherited),
                boundary: GridMeasurementBoundary::StandaloneIntrinsicMinimumProbe {
                    axes: intrinsic_axes,
                },
            },
            stage,
        )?;
        constants.contained_inline_basis(
            contained_outer_size(containment, constants.flow_axes, empty.output.size),
            stage,
        )
    } else {
        None
    };
    let initialized = initialize_grid_tracks::<Tree, M>(
        tree,
        node,
        &container_style,
        &constants,
        parent_context,
        sizing_input,
        contained_inline,
    )?;
    let constants = constants.for_inline_track_basis(contained_inline);
    let gap = Size::new(
        initialized.context.gap.inline,
        initialized.context.gap.block,
    );
    let content_box_size = content_box_estimate.unwrap_or_else(|| {
        let size = constants.flow_axes.logical_size(constants.node_inner_size);
        Size::new(
            size.inline.unwrap_or(Tree::Scalar::ZERO),
            size.block.unwrap_or(Tree::Scalar::ZERO),
        )
    });
    let column_count = initialized.column_tracks.len();
    let row_count = initialized.row_tracks.len();
    if column_count == 0 || row_count == 0 {
        return Ok(Vec::new());
    }

    let opposite = match queried_axis {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    };
    let opposite_inherited = match opposite {
        GridAxisKind::Column => parent_context.columns.is_some(),
        GridAxisKind::Row => parent_context.rows.is_some(),
    };
    // Reuse only the actual incoming prefix. An unresolved initial-first
    // opposite remains unbounded; neither estimates nor zero loans become an
    // allocation. A later prefix obtains its own standalone opposite through
    // the same canonical stage owner used by ordinary Grid.
    let resolved = if !opposite_inherited && stage.has_resolved_axis(opposite) {
        Some(intrinsic_stage::resolve_tracks(
            tree,
            node,
            GridTrackResolutionInput {
                style: &container_style,
                constants: &constants,
                column_tracks: &initialized.column_tracks,
                row_tracks: &initialized.row_tracks,
                minimum_functions: minimum_parent_functions(
                    &initialized.column_tracks,
                    &initialized.row_tracks,
                    initialized.context.percent_basis,
                    parent_context,
                ),
                context: std::borrow::Cow::Borrowed(&initialized.context),
                subgrid_report: &initialized.subgrid_report,
                sizing_flow_axes: constants.flow_axes,
                available: initialized.track_available,
                placements: &initialized.placements,
                stage,
            },
            intrinsic_stage::GridTrackRun::Intrinsic,
        )?)
    } else {
        None
    };

    let zero_columns = vec![Tree::Scalar::ZERO; column_count];
    let zero_rows = vec![Tree::Scalar::ZERO; row_count];
    let traversal_columns = if parent_context.columns.is_none()
        && let Some(resolved) = &resolved
        && stage.has_resolved_axis(GridAxisKind::Column)
    {
        resolved.columns.clone()
    } else {
        traversal_child_area_tracks(
            parent_context.columns.as_ref(),
            &initialized.column_tracks,
            content_box_size.width,
            gap.width,
            container_style
                .justify_content
                .unwrap_or(AlignContent::Stretch),
        )
        .map_err(|error| grid_track_sizing_error(node, error))?
    };
    let traversal_rows = parent_context.rows.as_ref().map_or_else(
        || {
            resolved
                .as_ref()
                .filter(|_| stage.has_resolved_axis(GridAxisKind::Row))
                .map_or_else(|| zero_rows.clone(), |resolved| resolved.rows.clone())
        },
        |axis| axis.tracks.clone(),
    );
    let traversal_column_geometry = UsedGridAxisGeometryOf::from_sizing_gutters(
        traversal_columns.clone(),
        &initialized.context.column_gutters,
    );
    let traversal_row_geometry = UsedGridAxisGeometryOf::from_sizing_gutters(
        traversal_rows.clone(),
        &initialized.context.row_gutters,
    );
    let minimum_functions = minimum_parent_functions(
        &initialized.column_tracks,
        &initialized.row_tracks,
        initialized.context.percent_basis,
        parent_context,
    );
    let children = tree.children(node).collect::<Vec<_>>();
    let lane_grid_axis = container_style
        .display
        .establishes_grid_lanes_formatting_context()
        .then(|| grid_axis_for_grid_lanes(&container_style));
    let placed_areas = if lane_grid_axis.is_none() {
        resolve_grid_child_areas(ResolveGridChildAreasInput {
            children: &children,
            placements: &initialized.placements,
            style: &container_style,
            columns: &zero_columns,
            rows: &zero_rows,
            gap: LogicalSizeOf::new(gap.width, gap.height),
            lines: initialized.context.lines,
        })
    } else {
        Vec::new()
    };
    let lane_track = traversal.uses_lane_track_sizing();
    let child_traversal = GridMinimumWrapperTraversal::Shared { lane_track };
    let lane_areas = lane_grid_axis.into_iter().flat_map(|axis| {
        let geometry = match axis {
            GridAxisKind::Column => &traversal_column_geometry,
            GridAxisKind::Row => &traversal_row_geometry,
        };
        lanes::lane_intrinsic_wrapper_areas(
            &initialized.placements.items,
            initialized.context.lines,
            axis,
            geometry,
            !lane_candidates,
        )
        .map(move |(index, area)| (index, area, child_traversal))
    });
    let (track_count, collapsed) = match queried_axis {
        GridAxisKind::Column => (column_count, initialized.context.column_gutters.collapsed()),
        GridAxisKind::Row => (row_count, initialized.context.row_gutters.collapsed()),
    };
    let expand_lane_wrappers = lane_grid_axis.is_none() && !lane_candidates;
    let areas = placed_areas
        .into_iter()
        .enumerate()
        .filter_map(|(index, area)| area.map(|area| (index, area)))
        .flat_map(|(index, area)| {
            let item = initialized.placements.items[index];
            let placement = match queried_axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            let inherited_axis = inherited_subgrid_axis_for_parent_axis(
                initialized.subgrid_report.items[index],
                queried_axis,
            );
            let expand = expand_lane_wrappers
                && inherited_axis.is_some_and(|axis| {
                    automatic_lane_subgrid(
                        initialized.placements.child_input(index),
                        placement,
                        axis,
                    )
                });
            subgrid_root_intrinsic_areas(
                area,
                queried_axis,
                track_count,
                collapsed,
                expand,
                lane_track,
            )
            .map(move |(area, traversal)| (index, area, traversal))
        })
        .chain(lane_areas);
    let mut traversal_children = Vec::new();
    for (index, child_area, child_traversal) in areas {
        if matches!(traversal, GridMinimumWrapperTraversal::SharedLaneCandidate) {
            let placement = initialized.placements.items[index];
            let placement = match queried_axis {
                GridAxisKind::Column => placement.column,
                GridAxisKind::Row => placement.row,
            };
            if has_definite_line(placement) {
                continue;
            }
        }
        let child = children[index];
        let child_report = initialized.subgrid_report.items[index];
        let child_input = initialized.placements.child_input(index);
        let child_style = child_input.item();
        if !is_in_flow_grid_child(child_style) {
            continue;
        }
        let child_axis = traversal_child_axis_for_parent_axis(child_report, queried_axis);
        let child_query = IntrinsicGridQuery {
            axis: Some(queried_axis),
            stage,
            columns: if query.columns.is_some()
                || (parent_context.columns.is_none()
                    && resolved.is_some()
                    && stage.has_resolved_axis(GridAxisKind::Column))
            {
                Some(traversal_columns.as_slice())
            } else {
                None
            },
            rows: if query.rows.is_some()
                || (parent_context.rows.is_none()
                    && resolved.is_some()
                    && stage.has_resolved_axis(GridAxisKind::Row))
            {
                Some(traversal_rows.as_slice())
            } else {
                None
            },
            available: prepared.sizing_request().axes(),
            ..query
        };
        let minimum_source = SubgridParentTrackSource {
            baseline_references: container_style.baseline_references,
            flow_axes: constants.flow_axes,
            gap,
            columns: Some(&traversal_column_geometry),
            rows: Some(&traversal_row_geometry),
            minimum_functions,
            algorithm: Some(sizing_algorithm_for_grid_display(container_style.display)),
            lane_grid_axis,
            allocated: LogicalSizeOf::new(child_query.columns, child_query.rows),
        };
        let child_area_size = intrinsic_traversal_area_size(
            child_area,
            &traversal_columns,
            &traversal_rows,
            Size::new(Some(content_box_size.width), Some(content_box_size.height)),
            minimum_source,
        )
        .map_err(|_| minimum_source_error(child))?;
        let child_grid = IntrinsicGrid {
            style: &container_style,
            constants: &constants,
            sizing_flow_axes: constants.flow_axes,
            column_tracks: &initialized.column_tracks,
            row_tracks: &initialized.row_tracks,
            minimum_functions,
            track_basis: initialized.context.percent_basis,
            gap: LogicalSizeOf::new(gap.width, gap.height),
            column_gutters: Some(&initialized.context.column_gutters),
            row_gutters: Some(&initialized.context.row_gutters),
            lines: initialized.context.lines,
            named_columns: &initialized.context.named_columns,
            named_rows: &initialized.context.named_rows,
            area_facts: initialized.context.area_facts.as_ref(),
            subgrid_report: &initialized.subgrid_report,
            placements: &initialized.placements,
            query: child_query,
        };
        let execution = IntrinsicGridChildInput {
            child_style,
            child_container: child_input.nested_container(),
            grid: child_grid,
            area: GridArea {
                size: LogicalSizeOf::new(child_area_size.width, child_area_size.height),
                ..child_area
            },
            columns: &traversal_columns,
            rows: &traversal_rows,
            subgrid_item: Some(child_report),
            input: ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                constants.node_inner_size,
                crate::ContainingLayoutContext::new(
                    constants.flow_axes,
                    crate::ParentFormattingContext::Grid,
                ),
                SizingRequestOf::new(child_query.available, constants.child_orthogonal_fallback),
            ),
        };
        let child_prepared = intrinsic_stage_child_input::<Tree, M>(child, &execution)?;
        if let Some(child) = visit(
            tree,
            GridMinimumWrapperChild {
                node: child,
                source_index: crate::SourceIndex::new(index),
                execution,
                prepared: child_prepared,
                minimum_source,
                child_axis,
                traversal: child_traversal,
                area_size: child_area_size,
                align_self: match queried_axis {
                    GridAxisKind::Column => {
                        child_style.justify_self.or(container_style.justify_items)
                    }
                    GridAxisKind::Row => child_style.align_self.or(container_style.align_items),
                }
                .unwrap_or(AlignItems::Stretch),
                column_geometry: &traversal_column_geometry,
                row_geometry: &traversal_row_geometry,
            },
        )? {
            traversal_children.try_reserve(1).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            traversal_children.push(child);
        }
    }

    Ok(traversal_children)
}

pub(super) fn inherited_subgrid_axis_for_parent_axis<Node>(
    item_report: SubgridItemReport<Node>,
    parent_axis: GridAxisKind,
) -> Option<GridAxisKind>
where
    Node: Copy,
{
    [GridAxisKind::Column, GridAxisKind::Row]
        .into_iter()
        .find(|axis| {
            let report = match axis {
                GridAxisKind::Column => item_report.column,
                GridAxisKind::Row => item_report.row,
            };
            report.can_inherit() && report.mapping.parent_axis == parent_axis
        })
}

fn traversal_child_axis_for_parent_axis<Node>(
    item_report: SubgridItemReport<Node>,
    parent_axis: GridAxisKind,
) -> GridAxisKind
where
    Node: Copy,
{
    // Mapping is independent of whether the child inherits that axis. An
    // orthogonal standalone dimension still contributes on its physical axis.
    if item_report.column.mapping.parent_axis == parent_axis {
        GridAxisKind::Column
    } else {
        GridAxisKind::Row
    }
}

fn traversal_child_area_tracks<Node, S: LayoutScalar>(
    inherited: Option<&InheritedGridAxis<S, Node>>,
    tracks: &[TrackSizingOf<S>],
    content_width: S,
    gap: S,
    alignment: AlignContent,
) -> Result<Vec<S>, SpanningContributionError<S>> {
    if let Some(axis) = inherited {
        return Ok(axis.tracks.clone());
    }

    // This traversal prepares an opposite-axis estimate before child facts
    // exist; the retained query stage carries its unresolved provenance.
    estimate_definite_axis_tracks(DefiniteAxisTrackEstimateInput {
        tracks,
        basis: Some(content_width),
        available: content_width,
        stretch_size: Some(content_width),
        gap,
        alignment,
        gutters: None,
    })
}

pub(super) fn intrinsic_subgrid_axis_parent_context<Node, S: LayoutScalar>(
    report: SubgridAxisReport,
    area: GridArea<S>,
    gap: Size<S>,
    parent_source: SubgridParentTrackSource<'_, S>,
    named_columns: &NamedGridLines,
    named_rows: &NamedGridLines,
    area_facts: Option<&GridAreaNameFacts>,
) -> Result<Option<InheritedGridAxis<S, Node>>, SubgridChildContextError<S>> {
    if !report.can_inherit() {
        return Ok(None);
    }
    let mapping = report.mapping;
    let (parent_start, parent_end) = match mapping.parent_axis {
        GridAxisKind::Column => (area.column, area.column_end),
        GridAxisKind::Row => (area.row, area.row_end),
    };
    let (parent_gap, named_lines) = match mapping.parent_axis {
        GridAxisKind::Column => (gap.width, named_columns),
        GridAxisKind::Row => (gap.height, named_rows),
    };
    let source_geometry = match mapping.parent_axis {
        GridAxisKind::Column => parent_source.columns,
        GridAxisKind::Row => parent_source.rows,
    };
    intrinsic_subgrid_axis_parent_context_for_span(
        report,
        GridTrackSpan::new(parent_start + 1, parent_end + 1),
        parent_gap,
        SubgridInheritedAxisSource {
            baseline_reference: grid_baseline_reference(
                parent_source.baseline_references,
                mapping.parent_axis,
            ),
            geometry: source_geometry,
            allocated: match mapping.parent_axis {
                GridAxisKind::Column => parent_source.allocated.inline,
                GridAxisKind::Row => parent_source.allocated.block,
            },
            minimum_functions: parent_source
                .minimum_functions
                .for_axis(mapping.parent_axis),
        },
        named_lines,
        area_facts,
    )
}

/// The same real inherited-axis constructor for callers with one authenticated
/// grid-axis span and no enclosing two-dimensional GridArea (e.g. Lane).
pub(super) fn intrinsic_subgrid_axis_parent_context_for_span<Node, S: LayoutScalar>(
    report: SubgridAxisReport,
    span: GridTrackSpan,
    parent_gap: S,
    source: SubgridInheritedAxisSource<'_, S>,
    named_lines: &NamedGridLines,
    area_facts: Option<&GridAreaNameFacts>,
) -> Result<Option<InheritedGridAxis<S, Node>>, SubgridChildContextError<S>> {
    if !report.can_inherit() {
        return Ok(None);
    }
    let mapping = report.mapping;
    let parent_start =
        span.start
            .checked_sub(1)
            .ok_or(SubgridChildContextError::TrackInheritance(
                SubgridTrackInheritanceError::SpanOutOfRange,
            ))?;
    let parent_end = span
        .end
        .checked_sub(1)
        .filter(|end| *end > parent_start)
        .ok_or(SubgridChildContextError::TrackInheritance(
            SubgridTrackInheritanceError::SpanOutOfRange,
        ))?;
    let track_count = parent_end - parent_start;
    let mut tracks = match source.allocated {
        Some(tracks) => tracks
            .get(parent_start..parent_end)
            .ok_or(SubgridChildContextError::TrackInheritance(
                SubgridTrackInheritanceError::SpanOutOfRange,
            ))?
            .to_vec(),
        None => vec![S::ZERO; track_count],
    };
    if mapping.reversed {
        tracks.reverse();
    }
    if source
        .geometry
        .is_some_and(|geometry| parent_end > geometry.sizes().len())
    {
        return Err(SubgridChildContextError::TrackInheritance(
            SubgridTrackInheritanceError::SpanOutOfRange,
        ));
    }
    let geometry = source
        .geometry
        .map(|geometry| {
            let sizes = tracks.clone();
            let collapsed = geometry.collapsed()[parent_start..parent_end].to_vec();
            let active_boundary_after = geometry.active_boundary_after()
                [parent_start..parent_end.saturating_sub(1)]
                .to_vec();
            let gutter_after =
                geometry.gutter_after()[parent_start..parent_end.saturating_sub(1)].to_vec();
            let gutters = OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
                track_count,
                &collapsed,
                &active_boundary_after,
                &gutter_after,
            );
            let gutters = if mapping.reversed {
                gutters.reversed()
            } else {
                gutters
            };
            UsedGridAxisGeometryOf::from_sizing_gutters(sizes, &gutters)
        })
        .unwrap_or_else(|| {
            UsedGridAxisGeometryOf::new(tracks.clone(), vec![false; track_count], parent_gap)
        });

    let (named_lines, area_facts) = report.parent_name_context(named_lines, area_facts);
    Ok(Some(InheritedGridAxis {
        offset: S::ZERO,
        gap: parent_gap,
        tracks,
        minimum_functions: source
            .minimum_functions
            .map(|functions| {
                functions
                    .mapped_span(parent_start, parent_end, mapping.reversed)
                    .ok_or(SubgridChildContextError::TrackInheritance(
                        SubgridTrackInheritanceError::SpanOutOfRange,
                    ))?
                    .retain()
                    .map_err(SubgridChildContextError::MinimumFunctions)
            })
            .transpose()?,
        geometry,
        named_lines,
        area_facts,
        template_area_expanded: false,
        owner_baseline_targets: None,
        baseline_reference: source.baseline_reference,
        parent_start,
        parent_end,
        reversed: mapping.reversed,
    }))
}

fn area_span<S: LayoutScalar>(area: GridArea<S>, axis: GridAxisKind) -> GridTrackSpan {
    match axis {
        GridAxisKind::Column => GridTrackSpan::new(area.column + 1, area.column_end + 1),
        GridAxisKind::Row => GridTrackSpan::new(area.row + 1, area.row_end + 1),
    }
}

fn axis_size<S: LayoutScalar>(size: Size<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => size.width,
        GridAxisKind::Row => size.height,
    }
}

fn resolved_subgrid_axis_gap<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    axis: GridAxisKind,
    report: SubgridAxisReport,
    parent_gap: Size<S>,
    content_basis: Size<Option<S>>,
) -> Result<S, LengthResolutionStatus<S>> {
    let parent_axis = if report.can_inherit() {
        report.mapping.parent_axis
    } else {
        axis
    };
    Ok(child_subgrid_gap(style, axis, content_basis)?.resolve(axis_size(parent_gap, parent_axis)))
}

fn traversal_axis_edges<S: LayoutScalar>(
    axis: GridAxisKind,
    flow_axes: crate::geometry::FlowAxes,
    edges: Edges<S>,
) -> SubgridAxisEdges<S> {
    let edges = flow_axes.logical_edges(edges);
    match axis {
        GridAxisKind::Column => SubgridAxisEdges {
            start: edges.inline_start,
            end: edges.inline_end,
        },
        GridAxisKind::Row => SubgridAxisEdges {
            start: edges.block_start,
            end: edges.block_end,
        },
    }
}

pub(super) fn intrinsic_traversal_area_size<S: LayoutScalar>(
    area: GridArea<S>,
    columns: &[S],
    rows: &[S],
    container_size: Size<Option<S>>,
    source: SubgridParentTrackSource<'_, S>,
) -> Result<Size<S>, SubgridTraversalError> {
    Ok(Size::new(
        intrinsic_traversal_axis_area_size(
            area.column,
            area.column_end,
            columns,
            source.gap.width,
            source.columns.map(UsedGridAxisGeometryOf::gutter_after),
            container_size.width,
            source.allocated.inline,
        )?,
        intrinsic_traversal_axis_area_size(
            area.row,
            area.row_end,
            rows,
            source.gap.height,
            source.rows.map(UsedGridAxisGeometryOf::gutter_after),
            container_size.height,
            source.allocated.block,
        )?,
    ))
}

fn intrinsic_traversal_axis_area_size<S: LayoutScalar>(
    start: usize,
    end: usize,
    tracks: &[S],
    gap: S,
    gutters: Option<&[S]>,
    definite_container_size: Option<S>,
    allocated: Option<&[S]>,
) -> Result<S, SubgridTraversalError> {
    if let Some(allocated) = allocated {
        if start >= end || end > allocated.len() {
            return Err(SubgridTraversalError::SpanOutOfRange);
        }
        // An actual used span can overflow its container, including a span
        // covering every track. A real zero allocation remains authoritative.
        return Ok(track_span_sum_with_boundary_gaps(
            allocated, start, end, gap, gutters,
        ));
    }
    if tracks.is_empty() {
        return Ok(definite_container_size.unwrap_or(S::ZERO));
    }
    if start >= tracks.len() {
        return Ok(S::ZERO);
    }
    Ok(if start == 0 && end == tracks.len() {
        definite_container_size
            .unwrap_or_else(|| track_span_sum_with_boundary_gaps(tracks, start, end, gap, gutters))
    } else {
        track_span_sum_with_boundary_gaps(tracks, start, end, gap, gutters)
    })
}

pub(super) fn subgrid_eligibility<'input, S: LayoutScalar>(
    input: SubgridEligibilityInput<'input, '_, S>,
) -> SubgridEligibility {
    let reason = if !input.child_requested {
        Some(SubgridIneligibleReason::NotRequested)
    } else if !input.has_parent_grid {
        Some(SubgridIneligibleReason::NoParentGrid)
    } else if excluded_from_normal_layout(input.child_style) {
        Some(SubgridIneligibleReason::ExcludedFromNormalLayout)
    } else if !subgrid_container_display_supported(input.child_style.display) {
        Some(SubgridIneligibleReason::UnsupportedDisplay)
    } else if input.child_container.is_none() {
        Some(SubgridIneligibleReason::NoChildGridContainer)
    } else {
        None
    };
    if let Some(reason) = reason {
        return SubgridEligibility::Excluded(reason);
    }
    if parent_is_lanes_in_resolved_axis(input.parent_style, input.child_style, input.axis) {
        SubgridEligibility::Stacking
    } else if input.child_container.is_some_and(|container| {
        let style = container.projection();
        style.display.establishes_grid_lanes_formatting_context()
            && style.grid_lanes_flow.grid_axis() != input.axis
    }) {
        SubgridEligibility::Excluded(SubgridIneligibleReason::ChildStackingAxis)
    } else {
        SubgridEligibility::SharedTracks
    }
}

pub(super) fn subgrid_axis_report<'input, S: LayoutScalar>(
    parent_style: &GridContainerProjection<'_, S>,
    child_input: &GridChildInput<'input, S>,
    axis: GridAxisKind,
) -> SubgridAxisReport {
    let child_style = child_input.item();
    SubgridAxisReport {
        inherit_parent_names: true,
        mapping: map_grid_axis(GridAxisMappingInput {
            queried_axis: axis,
            parent_style,
            child_style,
        }),
        eligibility: subgrid_eligibility(SubgridEligibilityInput {
            axis,
            parent_style,
            has_parent_grid: true,
            child_style,
            child_requested: child_input.subgrid_requested(axis),
            child_container: child_input.nested_container(),
        }),
    }
}

const fn subgrid_container_display_supported(display: Display) -> bool {
    display.establishes_grid_formatting_context()
}

const fn excluded_from_normal_layout<'input, S: LayoutScalar>(
    style: &GridItemProjection<'input, S>,
) -> bool {
    matches!(style.display, Display::None) || matches!(style.position, Position::Absolute)
}

fn parent_is_lanes_in_resolved_axis<'input, S: LayoutScalar>(
    parent_style: &GridContainerProjection<'_, S>,
    child_style: &GridItemProjection<'input, S>,
    child_axis: GridAxisKind,
) -> bool {
    if !parent_style
        .display
        .establishes_grid_lanes_formatting_context()
    {
        return false;
    }
    lane_axis_for_grid_lanes(parent_style)
        == map_grid_axis(GridAxisMappingInput {
            queried_axis: child_axis,
            parent_style,
            child_style,
        })
        .parent_axis
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;
    use crate::{
        Compute, ComputeInput, ComputeOutput, DefaultScalar, LayoutErrorKind, LayoutErrorSite,
        LayoutInvalidInput, LayoutOperation, LayoutResultOf, LengthPercentageOf, NodeInput,
        NodeInputOf, PreferredSize, Size, SubgridTrack, TrackComponent, Traverse,
    };

    #[test]
    fn lane_leaf_frame_has_no_stacking_track_maximum() {
        let flow = FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr);
        let prepared = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(flow, crate::ParentFormattingContext::Grid),
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MinContent,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        );
        let columns = [TrackSizingOf::<f64>::px(50.0)];
        let rows = [TrackSizingOf::<f64>::px(10.0)];
        let area = GridArea {
            column: 0,
            row: 0,
            column_end: 1,
            row_end: 1,
            size: LogicalSizeOf::new(50.0, 10.0),
        };
        let frame = |algorithm, lane_grid_axis| {
            SubgridParentTrackSource {
                baseline_references: crate::GridBaselineReferences::default(),
                flow_axes: flow,
                gap: Size::ZERO,
                columns: None,
                rows: None,
                minimum_functions: GridMinimumParentFunctions {
                    columns: Some(GridMinimumTrackFunctions::new(&columns, None)),
                    rows: Some(GridMinimumTrackFunctions::new(&rows, None)),
                },
                algorithm: Some(algorithm),
                lane_grid_axis,
                allocated: LogicalSizeOf::new(None, None),
            }
            .frame::<_, Infallible>(0, GridAxisKind::Column, area, prepared)
            .unwrap()
            .unwrap()
        };
        assert_eq!(
            frame(SizingAlgorithm::Grid, None).opposite_fixed_max_span,
            Some(10.0)
        );
        assert_eq!(
            frame(SizingAlgorithm::GridLanes, Some(GridAxisKind::Column)).opposite_fixed_max_span,
            None
        );
    }

    struct TraversalTree {
        input: NodeInput,
    }

    impl Traverse for TraversalTree {
        type Node = u32;
        type Scalar = DefaultScalar;
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
            unreachable!("traversal test tree has no children")
        }
    }

    struct TraversalTreeCompute<'input> {
        input: &'input NodeInput,
    }

    impl TraversalTree {
        fn borrow_compute(&mut self) -> TraversalTreeCompute<'_> {
            TraversalTreeCompute { input: &self.input }
        }
    }

    impl<'input> Traverse for TraversalTreeCompute<'input> {
        type Node = u32;
        type Scalar = DefaultScalar;
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
            unreachable!("traversal test tree has no children")
        }
    }

    impl<'input> Compute<'input, Infallible> for TraversalTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, _node: Self::Node) -> &'input NodeInput {
            self.input
        }

        fn layout_input(&self, _node: Self::Node) -> crate::LayoutInputView<'input> {
            crate::LayoutInputViewOf::Box(self.input)
        }

        fn staged_unrounded(
            &self,
            node: Self::Node,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>,
            Self::Scalar,
            Infallible,
        > {
            Err(crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(node),
                crate::LayoutOperation::ChildLayout,
                crate::LayoutErrorKindOf::InternalInvariant(
                    crate::LayoutInternalInvariant::MissingStagedUnroundedOutput,
                ),
            ))
        }

        fn set_unrounded(&mut self, _entry: crate::LayoutOutputEntry<Self::Node>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInput,
        ) -> LayoutResultOf<Self::Node, ComputeOutput, Self::Scalar, Infallible> {
            unreachable!("subgrid traversal setup must not compute children")
        }
    }

    fn assert_resolved_subgrid_axis_gap_uses_node_logical_axes<S: LayoutScalar>() {
        for (writing_mode, direction) in [
            (crate::WritingMode::VerticalRl, crate::Direction::Ltr),
            (crate::WritingMode::SidewaysLr, crate::Direction::Rtl),
        ] {
            let style = NodeInputOf::<S> {
                display: Display::Grid,
                writing_mode,
                direction,
                gap: Size::new(
                    LengthOf::px(S::from_f64(7.0)),
                    LengthOf::px(S::from_f64(11.0)),
                ),
                ..NodeInputOf::default()
            };
            let parent_style = NodeInputOf::<S> {
                display: Display::Grid,
                ..NodeInputOf::default()
            };
            let child_input = GridChildInput::from_node(&style);
            let container_projection = child_input
                .nested_container()
                .expect("grid test child must retain container input")
                .projection();
            let parent_projection = GridContainerProjection::from_node(&parent_style);

            for (axis, expected) in [
                (GridAxisKind::Column, S::from_f64(11.0)),
                (GridAxisKind::Row, S::from_f64(7.0)),
            ] {
                let actual = resolved_subgrid_axis_gap(
                    &container_projection,
                    axis,
                    subgrid_axis_report(&parent_projection, &child_input, axis),
                    Size::ZERO,
                    Size::splat(Some(S::from_f64(100.0))),
                )
                .expect("fixed gap should resolve");

                assert_eq!(
                    actual, expected,
                    "{writing_mode:?} {direction:?} {axis:?} must select the node's logical gap"
                );
            }
        }
    }

    #[test]
    fn resolved_subgrid_axis_gap_uses_node_logical_axes_f32() {
        assert_resolved_subgrid_axis_gap_uses_node_logical_axes::<f32>();
    }

    #[test]
    fn resolved_subgrid_axis_gap_uses_node_logical_axes_f64() {
        assert_resolved_subgrid_axis_gap_uses_node_logical_axes::<f64>();
    }

    #[test]
    fn subgrid_traversal_propagates_track_initialization_failure() {
        let overflowing =
            LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
        let parent_style = NodeInput {
            display: Display::Grid,
            ..NodeInput::default()
        };
        let style = NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::Subgrid(SubgridTrack::new(vec![]))],
            grid_template_rows: vec![TrackComponent::from(overflowing)],
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(f32::MAX)),
            ..NodeInput::default()
        };
        let mut tree = TraversalTree {
            input: style.clone(),
        };
        let child_input = GridChildInput::from_node(&style);
        let parent_projection = GridContainerProjection::from_node(&parent_style);
        let item_report = SubgridItemReport {
            node: 0,
            column: subgrid_axis_report(&parent_projection, &child_input, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_projection, &child_input, GridAxisKind::Row),
        };
        let named_columns = NamedGridLines::new(GridAxisKind::Column, 1);
        let named_rows = NamedGridLines::new(GridAxisKind::Row, 1);

        let container = child_input
            .nested_container()
            .expect("subgrid traversal test must retain container input");
        let area = GridArea {
            column: 0,
            row: 0,
            column_end: 1,
            row_end: 1,
            size: LogicalSizeOf::new(20.0, 20.0),
        };
        let parent_axes = FlowAxes::new(parent_style.writing_mode, parent_style.direction);
        let prepared = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(20.0), Some(20.0)),
            crate::ContainingLayoutContext::new(parent_axes, crate::ParentFormattingContext::Grid),
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MaxContent,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        );
        for facts in [
            IntrinsicMinTrackFacts::Known(&[true]),
            IntrinsicMinTrackFacts::Unknown,
        ] {
            let error = subgrid_traversal_children::<_, Infallible>(
                &mut tree.borrow_compute(),
                0,
                container,
                Size::new(20.0, 20.0),
                GridAxisKind::Column,
                parent_axes,
                Size::ZERO,
                IntrinsicGridQuery::complete(GridContributionPurpose::MaxContent),
                SubgridParentTrackSource {
                    baseline_references: crate::GridBaselineReferences::default(),
                    flow_axes: parent_axes,
                    gap: Size::ZERO,
                    columns: None,
                    rows: None,
                    minimum_functions: GridMinimumParentFunctions {
                        columns: None,
                        rows: None,
                    },
                    algorithm: Some(sizing_algorithm_for_grid_display(parent_projection.display)),
                    lane_grid_axis: None,
                    allocated: LogicalSizeOf::new(None, None),
                },
                prepared,
                Size::new(None, Some(f32::MAX)),
                SubgridChildParentContextInput {
                    parent_baseline_references: crate::GridBaselineReferences::default(),
                    item: item_report,
                    child_style: child_input.item(),
                    child_container: Some(*container),
                    area,
                    content_box_size: Size::new(20.0, 20.0),
                    columns: &[],
                    rows: &[],
                    minimum_functions: GridMinimumParentFunctions {
                        columns: None,
                        rows: None,
                    },
                    gap: LogicalSizeOf::new(0.0, 0.0),
                    parent_named_columns: &named_columns,
                    parent_named_rows: &named_rows,
                    parent_area_facts: None,
                    margin: Edges::all(Some(0.0)),
                    border: Edges::ZERO,
                    padding: Edges::ZERO,
                    scrollbar_gutter: Edges::ZERO,
                },
                GridMinimumWrapperTraversal::Shared { lane_track: false },
                &mut SubgridIntrinsicFold::new(facts),
                SubgridTraversalContext::ROOT,
            )
            .unwrap_err();

            assert_eq!(error.site(), LayoutErrorSite::Node(0));
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            assert!(matches!(
                error.kind(),
                LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
            ));
        }
    }
}
