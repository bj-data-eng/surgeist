use super::subgrid_intrinsic::{
    IntrinsicChildRequest, IntrinsicGridChildInput, SubgridContributionFinish,
    SubgridContributionFinishStart, SubgridContributionStart, SubgridIntrinsicContributionInput,
    SubgridIntrinsicContributionReport, prepare_intrinsic_grid_child,
    prepare_intrinsic_grid_child_request, start_subgrid_intrinsic_contributions,
};
use super::*;
use crate::node_input::GridBaselineReference;

use crate::geometry::{FlowAxes, PhysicalProgression};

pub(in crate::grid) fn grid_track_sizing_error<Node, S: LayoutScalar, M>(
    node: Node,
    error: SpanningContributionError<S>,
) -> LayoutErrorOf<Node, S, M> {
    match error {
        SpanningContributionError::InvalidNumeric { value } => {
            crate::error::value_resolution_error(
                node,
                LengthResolutionStatus::InvalidNumeric { value },
            )
        }
        SpanningContributionError::Allocation => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::Allocation,
        ),
        SpanningContributionError::InvalidIncidence
        | SpanningContributionError::MissingContribution { .. }
        | SpanningContributionError::DuplicateContribution { .. } => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidGridTrackSizing),
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum GridContributionPurpose {
    Minimum,
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::grid) struct IntrinsicGridQuery<'a, S: LayoutScalar> {
    pub(in crate::grid) purpose: GridContributionPurpose,
    pub(in crate::grid) axis: Option<GridAxisKind>,
    pub(in crate::grid) stage: GridSizingStage,
    pub(in crate::grid) available: Size<AxisSizingRequestOf<S>>,
    pub(in crate::grid) columns: Option<&'a [S]>,
    pub(in crate::grid) rows: Option<&'a [S]>,
    pub(in crate::grid) observe_auto: bool,
    pub(in crate::grid) include_flexible_tracks: bool,
}

impl<S: LayoutScalar> IntrinsicGridQuery<'_, S> {
    pub(in crate::grid) const fn complete(purpose: GridContributionPurpose) -> Self {
        Self {
            purpose,
            axis: None,
            stage: GridSizingStage::COMPLETE,
            available: Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            )),
            columns: None,
            rows: None,
            observe_auto: false,
            include_flexible_tracks: false,
        }
    }

    pub(in crate::grid) fn track_needs_contribution(
        self,
        track: &TrackSizingOf<S>,
        basis: Option<S>,
    ) -> bool {
        track_accepts_intrinsic_contribution(track, basis)
            || self.include_flexible_tracks && track.max.is_flexible()
    }

    fn includes(self, axis: GridAxisKind) -> bool {
        self.axis.is_none_or(|queried| queried == axis)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct GridContributionReceipt<Node, S: LayoutScalar> {
    pub(in crate::grid) purpose: GridContributionPurpose,
    pub(in crate::grid) source: Node,
    pub(in crate::grid) axis: GridAxisKind,
    pub(in crate::grid) span: GridTrackSpan,
    pub(in crate::grid) value: S,
}

pub(in crate::grid) struct IntrinsicTrackSizes<Node, S: LayoutScalar> {
    pub(in crate::grid) purpose: GridContributionPurpose,
    pub(in crate::grid) columns: Vec<S>,
    pub(in crate::grid) rows: Vec<S>,
    pub(in crate::grid) column_base_floors: Vec<S>,
    pub(in crate::grid) row_base_floors: Vec<S>,
    pub(in crate::grid) column_receipts: Vec<GridContributionReceipt<Node, S>>,
    pub(in crate::grid) row_receipts: Vec<GridContributionReceipt<Node, S>>,
}

// Actual item contributions have already been normalized into the owning
// Grid frame, including their purpose-specific baseline shims. Track allocation
// consumes these facts without a second baseline redistribution.
pub(in crate::grid) struct IntrinsicAxisContributions<Node, S: LayoutScalar> {
    pub(in crate::grid) purpose: GridContributionPurpose,
    pub(in crate::grid) sizes: Vec<S>,
    pub(in crate::grid) base_floors: Vec<S>,
    pub(in crate::grid) receipts: Vec<GridContributionReceipt<Node, S>>,
}

pub(in crate::grid) struct GridAxisContributions<Node, S: LayoutScalar> {
    pub(in crate::grid) minimum: IntrinsicAxisContributions<Node, S>,
    pub(in crate::grid) min_content: IntrinsicAxisContributions<Node, S>,
    pub(in crate::grid) max_content: IntrinsicAxisContributions<Node, S>,
    pub(in crate::grid) base_floors: Vec<S>,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct GridTrackContributions<S: LayoutScalar> {
    pub(in crate::grid) minimum: S,
    pub(in crate::grid) min_content: S,
    pub(in crate::grid) max_content: S,
}

impl<Node, S: LayoutScalar> GridAxisContributions<Node, S> {
    pub(in crate::grid) fn new(
        mut minimum: IntrinsicAxisContributions<Node, S>,
        min_content: IntrinsicAxisContributions<Node, S>,
        max_content: IntrinsicAxisContributions<Node, S>,
    ) -> Self {
        let count = minimum.sizes.len();
        let mut base_floors = core::mem::take(&mut minimum.base_floors);
        debug_assert!(base_floors.is_empty() || base_floors.len() == count);
        base_floors.resize(count, S::ZERO);
        for floors in [&min_content.base_floors, &max_content.base_floors] {
            debug_assert!(floors.is_empty() || floors.len() == count);
            for (floor, contribution_floor) in base_floors.iter_mut().zip(floors) {
                *floor = floor.max(*contribution_floor);
            }
        }
        Self {
            minimum,
            min_content,
            max_content,
            base_floors,
        }
    }
}

impl<Node, S: LayoutScalar> IntrinsicTrackSizes<Node, S> {
    pub(in crate::grid) fn into_axis(
        self,
        axis: GridAxisKind,
    ) -> IntrinsicAxisContributions<Node, S> {
        match axis {
            GridAxisKind::Column => IntrinsicAxisContributions::column(
                self.purpose,
                self.columns,
                self.column_receipts,
                self.column_base_floors,
            ),
            GridAxisKind::Row => IntrinsicAxisContributions {
                purpose: self.purpose,
                sizes: self.rows,
                base_floors: self.row_base_floors,
                receipts: self.row_receipts,
            },
        }
    }
}

impl<Node, S: LayoutScalar> IntrinsicAxisContributions<Node, S> {
    pub(in crate::grid) fn column(
        purpose: GridContributionPurpose,
        sizes: Vec<S>,
        receipts: Vec<GridContributionReceipt<Node, S>>,
        base_floors: Vec<S>,
    ) -> Self {
        Self {
            purpose,
            sizes,
            receipts,
            base_floors,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct IntrinsicGrid<'input, 'a, Node, S: LayoutScalar = DefaultScalar> {
    pub(in crate::grid) style: &'a GridContainerProjection<'a, S>,
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) sizing_flow_axes: FlowAxes,
    pub(in crate::grid) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(in crate::grid) track_basis: LogicalSizeOf<Option<S>>,
    pub(in crate::grid) column_tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) row_tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(in crate::grid) row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(in crate::grid) lines: GridLines,
    pub(in crate::grid) named_columns: &'a NamedGridLines,
    pub(in crate::grid) named_rows: &'a NamedGridLines,
    pub(in crate::grid) area_facts: Option<&'a GridAreaNameFacts>,
    pub(in crate::grid) subgrid_report: &'a GridSubgridReport<Node>,
    pub(in crate::grid) placements: &'a GridPlacementContext<'input, Node, S>,
    pub(in crate::grid) query: IntrinsicGridQuery<'a, S>,
}

impl<'input, 'a, Node, S: LayoutScalar> IntrinsicGrid<'input, 'a, Node, S> {
    fn gutters(self, axis: GridAxisKind) -> Option<&'a OrdinaryGridAxisGuttersOf<S>> {
        match axis {
            GridAxisKind::Column => self.column_gutters,
            GridAxisKind::Row => self.row_gutters,
        }
    }

    pub(in crate::grid) fn column_contribution_margin_basis(
        self,
        span_tracks: &[TrackSizingOf<S>],
        parent_size: Size<Option<S>>,
    ) -> Option<Size<Option<S>>> {
        let contributes = span_tracks.iter().any(|track| {
            self.query
                .track_needs_contribution(track, self.track_basis.inline)
        });
        contributes.then(|| {
            // An ordinary grid item can grow its own inline area. Resolve its
            // cyclic percentage against zero, preserving any fixed term.
            if self.style.display.inner_display() == Display::Grid {
                Size::splat(Some(S::ZERO))
            } else {
                parent_size
            }
        })
    }

    // Initial-first availability is a source-defined maximum-track estimate,
    // not an allocated opposite axis. An unresolved maximum stays unbounded.
    pub(in crate::grid) fn initial_opposite_span(
        self,
        axis: GridAxisKind,
        start: usize,
        end: usize,
    ) -> Option<S> {
        if self.query.stage.through != GridSizingPhase::InitialFirst
            || axis == self.query.stage.first_axis
        {
            return None;
        }
        let inner = self
            .sizing_flow_axes
            .logical_size(self.constants.node_inner_size);
        let (tracks, basis, gap, alignment) = match axis {
            GridAxisKind::Column => (
                self.column_tracks,
                inner.inline,
                self.gap.inline,
                self.style.justify_content,
            ),
            GridAxisKind::Row => (
                self.row_tracks,
                inner.block,
                self.gap.block,
                self.style.align_content,
            ),
        };
        let sizes = tracks
            .get(start..end)?
            .iter()
            .map(|track| track.max.definite(basis))
            .collect::<Option<Vec<_>>>()?;
        let extent = sizes
            .iter()
            .copied()
            .fold(S::ZERO, |sum, value| sum + value);
        let gutters = self.gutters(axis);
        if let Some(inner) = basis
            && let Some(all_sizes) = tracks
                .iter()
                .map(|track| track.max.definite(basis))
                .collect::<Option<Vec<_>>>()
            && let Some(gutters) = gutters
        {
            let total = track_sum_with_gutters(&all_sizes, gap, Some(gutters));
            let aligned = ordinary_grid_axis_alignment(
                inner - total,
                gutters,
                alignment.unwrap_or(AlignContent::Stretch),
            );
            return Some(
                UsedGridAxisGeometryOf::from_active_boundary_gutters(
                    all_sizes,
                    gutters.collapsed().to_vec(),
                    gutters.active_boundary_after().to_vec(),
                    aligned.gutter_after,
                )
                .span_extent(start, end),
            );
        }
        Some(
            extent
                + gutters.map_or_else(
                    || gap * S::from_usize(end.saturating_sub(start + 1)),
                    |gutters| gutters.span_gap_total(start, end),
                ),
        )
    }

    fn span_extent(self, axis: GridAxisKind, sizes: &[S], start: usize, end: usize) -> S {
        track_span_sum_with_gutters(
            sizes,
            start,
            end,
            match axis {
                GridAxisKind::Column => self.gap.inline,
                GridAxisKind::Row => self.gap.block,
            },
            self.gutters(axis),
        )
    }
}

#[derive(Clone, Copy, Default)]
pub(in crate::grid) struct IntrinsicGridLowerBounds<'a, S: LayoutScalar = DefaultScalar> {
    pub(in crate::grid) columns: Option<&'a [S]>,
    pub(in crate::grid) rows: Option<&'a [S]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum AncestorBaselineRole {
    First,
    Last,
}

impl AncestorBaselineRole {
    pub(in crate::grid) const fn opposite(self) -> Self {
        match self {
            Self::First => Self::Last,
            Self::Last => Self::First,
        }
    }

    pub(in crate::grid) fn in_progression(
        self,
        subject: PhysicalProgression,
        containing: PhysicalProgression,
    ) -> Self {
        if subject == containing {
            self
        } else {
            self.opposite()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum BaselineOwnerEdge {
    Start,
    End,
}

/// One finite signed distance from the selected owner-track edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct AncestorBaselineTarget<S: LayoutScalar = DefaultScalar> {
    role: AncestorBaselineRole,
    selected_ancestor_track: usize,
    selected_owner_edge: BaselineOwnerEdge,
    finite_owner_logical_target: S,
}

impl<S: LayoutScalar> AncestorBaselineTarget<S> {
    fn from_member<Node: Copy>(member: AncestorBaselineMember<Node, S>) -> Self {
        let role = member.role();
        Self {
            role,
            selected_ancestor_track: member.selected_track,
            selected_owner_edge: match role {
                AncestorBaselineRole::First => BaselineOwnerEdge::Start,
                AncestorBaselineRole::Last => BaselineOwnerEdge::End,
            },
            finite_owner_logical_target: member.containing_logical_distance,
        }
    }
    pub(in crate::grid) const fn role(self) -> AncestorBaselineRole {
        self.role
    }
    pub(in crate::grid) const fn selected_ancestor_track(self) -> usize {
        self.selected_ancestor_track
    }
    pub(in crate::grid) const fn selected_owner_edge(self) -> BaselineOwnerEdge {
        self.selected_owner_edge
    }
    pub(in crate::grid) const fn finite_owner_logical_target(self) -> S {
        self.finite_owner_logical_target
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct TrackAncestorBaselineTargets<S: LayoutScalar = DefaultScalar> {
    pub(in crate::grid) first: Option<AncestorBaselineTarget<S>>,
    pub(in crate::grid) last: Option<AncestorBaselineTarget<S>>,
}

fn reduce_complete_target<S: LayoutScalar>(
    slot: &mut Option<AncestorBaselineTarget<S>>,
    candidate: AncestorBaselineTarget<S>,
) {
    if slot.is_none_or(|current| {
        candidate.finite_owner_logical_target > current.finite_owner_logical_target
    }) {
        *slot = Some(candidate);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FlattenedScalarContribution<Node, S: LayoutScalar = DefaultScalar> {
    pub(super) source: Node,
    axis: GridAxisKind,
    pub(super) ancestor_span: GridTrackSpan,
    pub(super) contribution: S,
}

impl<Node, S: LayoutScalar> FlattenedScalarContribution<Node, S> {
    pub(super) fn new(
        source: Node,
        axis: GridAxisKind,
        ancestor_span: GridTrackSpan,
        contribution: S,
    ) -> Self {
        Self {
            source,
            axis,
            ancestor_span,
            contribution,
        }
    }
}

/// The selected body set and alignment-context track are independent of the
/// owner-edge cohort. In particular, reversing a cohort never selects another
/// body port or another track in a spanning item's alignment context.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct AncestorBaselineMember<Node, S: LayoutScalar = DefaultScalar> {
    reference: GridBaselineReference,
    source: Node,
    axis: GridAxisKind,
    physical_axis: PhysicalAxis,
    ancestor_span: GridTrackSpan,
    selected_track: usize,
    selected_port: AncestorBaselineRole,
    subject_progression: PhysicalProgression,
    containing_progression: PhysicalProgression,
    alignment_context_reversed: bool,
    containing_logical_distance: S,
    border_logical_baseline: S,
}

pub(in crate::grid) struct FinalBaselineProjectionInput<S: LayoutScalar> {
    pub(in crate::grid) axis: GridAxisKind,
    pub(in crate::grid) span: GridTrackSpan,
    pub(in crate::grid) reversed: bool,
    pub(in crate::grid) progression: PhysicalProgression,
    pub(in crate::grid) point: S,
    pub(in crate::grid) selected_start: S,
    pub(in crate::grid) selected_end: S,
    pub(in crate::grid) extent: S,
    pub(in crate::grid) physical_port: S,
}

impl<Node: Copy, S: LayoutScalar> AncestorBaselineMember<Node, S> {
    pub(in crate::grid) const fn reference(self) -> GridBaselineReference {
        self.reference
    }
    pub(in crate::grid) fn role(self) -> AncestorBaselineRole {
        self.selected_port
            .in_progression(self.subject_progression, self.containing_progression)
    }
    pub(in crate::grid) const fn selected_track(self) -> usize {
        self.selected_track
    }
    pub(in crate::grid) const fn selected_port(self) -> AncestorBaselineRole {
        self.selected_port
    }
    pub(in crate::grid) const fn subject_progression(self) -> PhysicalProgression {
        self.subject_progression
    }
    pub(in crate::grid) const fn containing_progression(self) -> PhysicalProgression {
        self.containing_progression
    }
    pub(in crate::grid) const fn axis(self) -> GridAxisKind {
        self.axis
    }
    pub(in crate::grid) const fn span(self) -> GridTrackSpan {
        self.ancestor_span
    }
    pub(in crate::grid) const fn context_reversed(self) -> bool {
        self.alignment_context_reversed
    }
    pub(in crate::grid) const fn distance(self) -> S {
        self.containing_logical_distance
    }
    pub(in crate::grid) fn project_final(
        self,
        input: FinalBaselineProjectionInput<S>,
    ) -> Result<Self, InheritedCurrentGridBaselinePlacementError> {
        let selected_track =
            if (self.selected_port == AncestorBaselineRole::First) != input.reversed {
                input.span.start.checked_sub(1)
            } else {
                input.span.end.checked_sub(2)
            }
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
        let role = self
            .selected_port
            .in_progression(self.subject_progression, input.progression);
        let distance = match role {
            AncestorBaselineRole::First => input.point - input.selected_start,
            AncestorBaselineRole::Last => input.selected_end - input.point,
        };
        let border_port = if input.progression.is_decreasing() {
            input.extent - input.physical_port
        } else {
            input.physical_port
        };
        if !distance.is_finite() || !border_port.is_finite() {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }
        Ok(Self {
            axis: input.axis,
            ancestor_span: input.span,
            selected_track,
            containing_progression: input.progression,
            alignment_context_reversed: input.reversed,
            containing_logical_distance: distance,
            border_logical_baseline: border_port,
            ..self
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct AncestorBaselineGroup<Node, S: LayoutScalar = DefaultScalar> {
    owner: Node,
    axis: GridAxisKind,
    physical_axis: PhysicalAxis,
    reference: GridBaselineReference,
    targets: Vec<TrackAncestorBaselineTargets<S>>,
}

impl<Node: Copy, S: LayoutScalar> AncestorBaselineGroup<Node, S> {
    pub(in crate::grid) const fn reference(&self) -> GridBaselineReference {
        self.reference
    }
    pub(in crate::grid) fn reduce(
        owner: Node,
        axis: GridAxisKind,
        physical_axis: PhysicalAxis,
        reference: GridBaselineReference,
        track_count: usize,
        members: impl IntoIterator<Item = AncestorBaselineMember<Node, S>>,
    ) -> Self {
        let mut targets = vec![TrackAncestorBaselineTargets::default(); track_count];
        for member in members {
            if member.axis != axis
                || member.physical_axis != physical_axis
                || member.reference != reference
            {
                continue;
            }
            let use_start = (member.selected_port == AncestorBaselineRole::First)
                != member.alignment_context_reversed;
            let expected_track = if use_start {
                member.ancestor_span.start.checked_sub(1)
            } else {
                member.ancestor_span.end.checked_sub(2)
            };
            if expected_track != Some(member.selected_track) {
                continue;
            }
            let Some(track) = targets.get_mut(member.selected_track) else {
                continue;
            };
            let target = AncestorBaselineTarget::from_member(member);
            match member.role() {
                AncestorBaselineRole::First => reduce_complete_target(&mut track.first, target),
                AncestorBaselineRole::Last => reduce_complete_target(&mut track.last, target),
            }
        }
        Self {
            owner,
            axis,
            physical_axis,
            reference,
            targets,
        }
    }

    pub(in crate::grid) fn intrinsic_shim(
        &self,
        member: AncestorBaselineMember<Node, S>,
    ) -> BaselineShim<S> {
        let Some(target) = self.target_for(member) else {
            return BaselineShim::default();
        };
        let amount = (target - member.containing_logical_distance).max(S::ZERO);
        match member.role() {
            AncestorBaselineRole::First => BaselineShim {
                before: amount,
                after: S::ZERO,
            },
            AncestorBaselineRole::Last => BaselineShim {
                before: S::ZERO,
                after: amount,
            },
        }
    }
    pub(in crate::grid) fn target_for(&self, member: AncestorBaselineMember<Node, S>) -> Option<S> {
        if member.axis != self.axis
            || member.physical_axis != self.physical_axis
            || member.reference != self.reference
        {
            return None;
        }
        self.target_record(member.role(), member.selected_track)
            .map(AncestorBaselineTarget::finite_owner_logical_target)
    }
    pub(in crate::grid) fn placement_offset(
        &self,
        member: AncestorBaselineMember<Node, S>,
        selected_track_start: S,
        selected_track_end: S,
    ) -> Option<S> {
        Some(self.placement_offset_for_target(
            member,
            self.target_for(member)?,
            selected_track_start,
            selected_track_end,
        ))
    }
    pub(in crate::grid) fn placement_offset_for_target(
        &self,
        member: AncestorBaselineMember<Node, S>,
        target: S,
        selected_track_start: S,
        selected_track_end: S,
    ) -> S {
        let point = match member.role() {
            AncestorBaselineRole::First => selected_track_start + target,
            AncestorBaselineRole::Last => selected_track_end - target,
        };
        point - member.border_logical_baseline
    }
    pub(in crate::grid) const fn axis(&self) -> GridAxisKind {
        self.axis
    }
    pub(in crate::grid) const fn owner(&self) -> Node {
        self.owner
    }
    pub(in crate::grid) const fn physical_axis(&self) -> PhysicalAxis {
        self.physical_axis
    }
    pub(in crate::grid) fn track_count(&self) -> usize {
        self.targets.len()
    }
    pub(in crate::grid) fn has_any_target(&self) -> bool {
        self.targets
            .iter()
            .any(|track| track.first.is_some() || track.last.is_some())
    }
    pub(in crate::grid) fn target_record(
        &self,
        role: AncestorBaselineRole,
        selected_track: usize,
    ) -> Option<AncestorBaselineTarget<S>> {
        let track = self.targets.get(selected_track)?;
        match role {
            AncestorBaselineRole::First => track.first,
            AncestorBaselineRole::Last => track.last,
        }
    }
    pub(in crate::grid) fn track_groups(&self) -> Vec<TrackBaselineGroup<S>> {
        self.targets
            .iter()
            .map(|track| TrackBaselineGroup {
                first: track.first.map(|target| {
                    PhysicalBaseline::new(self.physical_axis, target.finite_owner_logical_target)
                }),
                last: track.last.map(|target| {
                    PhysicalBaseline::new(self.physical_axis, target.finite_owner_logical_target)
                }),
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
pub(super) struct RowIntrinsicContribution<Node, S: LayoutScalar = DefaultScalar> {
    pub(super) source: Node,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) contributes_to_row_size: bool,
    pub(super) contribution: S,
    pub(super) baseline_member: Option<AncestorBaselineMember<Node, S>>,
}

type IntrinsicTrackChildren<Node, S> = std::iter::Enumerate<
    std::iter::Zip<std::vec::IntoIter<Node>, std::vec::IntoIter<Option<GridArea<S>>>>,
>;

pub(in crate::grid) struct IntrinsicTrackRole<Node, S: LayoutScalar> {
    children: IntrinsicTrackChildren<Node, S>,
    sizes: IntrinsicTrackSizes<Node, S>,
    row_contributions: Vec<RowIntrinsicContribution<Node, S>>,
    subgrid_row_count: usize,
    column_baselines: AncestorBaselineGroup<Node, S>,
}

struct IntrinsicTrackItem<Node, S: LayoutScalar> {
    index: usize,
    child: Node,
    area: GridArea<S>,
    column_margin_basis: Option<Size<Option<S>>>,
    align_self: AlignItems,
    justify_self: AlignItems,
    contributes_row: bool,
    row_baseline_candidate: bool,
}

pub(in crate::grid) struct IntrinsicTrackChild<Node, S: LayoutScalar> {
    pub(in crate::grid) node: Node,
    query: Option<GridMinimumQuery<S>>,
    item: Option<IntrinsicTrackItem<Node, S>>,
    /// A subgrid item that needs its parent's tracks as context, so that its
    /// queries are contextual Grid computations.
    contextual: bool,
}

impl<Node: Copy, S: LayoutScalar> IntrinsicTrackChild<Node, S> {
    pub(in crate::grid) fn input<M>(
        &self,
        grid: IntrinsicGrid<'_, '_, Node, S>,
    ) -> LayoutResultOf<Node, ComputeInputOf<S>, S, M> {
        let index = self.item.as_ref().expect("active track item").index;
        let input = self.query.as_ref().expect("active track query").input();
        grid_item_baseline_demand(
            grid.style,
            grid.placements.item_input(index),
            grid.query.axis,
            input,
        )
        .map_err(|error| grid_baseline_error_at_site(LayoutErrorSiteOf::Node(self.node), error))
    }

    fn accept<M>(
        &mut self,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, Option<IntrinsicTrackItem<Node, S>>, S, M> {
        let query = self.query.take().expect("active track query");
        match query.accept(self.node, output)? {
            ControlFlow::Break(_) => Ok(self.item.take()),
            ControlFlow::Continue(input) => {
                self.query = Some(GridMinimumQuery::Final(input));
                Ok(None)
            }
        }
    }
}

fn traversal_input<'input, 'a, Node: Copy, S: LayoutScalar>(
    grid: IntrinsicGrid<'input, 'a, Node, S>,
    axis: GridAxisKind,
    children: &'a [Node],
    placed_areas: &'a [Option<GridArea<S>>],
    column_sizes: &'a [S],
    row_sizes: &'a [S],
) -> GridSubgridIntrinsicTraversalInput<'input, 'a, Node, S> {
    GridSubgridIntrinsicTraversalInput {
        grid,
        axis,
        containing_flow_axes: grid.constants.flow_axes,
        children,
        placed_areas,
        subgrid_report: grid.subgrid_report,
        placements: grid.placements,
        named_columns: grid.named_columns,
        named_rows: grid.named_rows,
        area_facts: grid.area_facts,
        parent_gap: Size::new(grid.gap.inline, grid.gap.block),
        column_gutters: grid.column_gutters,
        row_gutters: grid.row_gutters,
        column_sizes,
        row_sizes,
        container_size: grid.constants.node_inner_size,
        #[cfg(test)]
        intrinsic_min_track_facts: IntrinsicMinTrackFacts::Unknown,
        query: grid.query,
    }
}

/// Which axis's subgrid contributions a role's beginning is collecting.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BeginAxis {
    Column,
    Row,
}

/// The traversal of one axis's subgrid items, started and not yet finished.
struct BeginTraversal<'input, Node, S: LayoutScalar> {
    facts: Vec<bool>,
    root: RootTraversal<S>,
    /// The wrappers the traversal has entered, while it waits on a
    /// computation.
    frames: Option<Box<WrapperFrames<'input, Node, S>>>,
    /// The fold of the traversal, while it waits on a computation.
    fold: Option<Box<SubgridIntrinsicFold<'input, Node, S>>>,
    /// The gutter a probe of one of its root items waits on, once known.
    waiting_root_probe: bool,
}

/// What a role's beginning waits for next.
pub(in crate::grid) enum IntrinsicTrackBeginStep<Node, S: LayoutScalar> {
    /// The subgrid items of the current axis are to be traversed, with these
    /// intrinsic-minimum facts for the axis's tracks.
    Traverse(Vec<bool>),
    Done(Box<IntrinsicTrackRole<Node, S>>),
}

/// The beginning of an intrinsic track role: the contributions of each axis's
/// subgrid items, whose traversals the owner may drive one step at a time.
pub(in crate::grid) struct IntrinsicTrackBegin<'input, Node, S: LayoutScalar> {
    columns: Vec<S>,
    rows: Vec<S>,
    column_base_floors: Vec<S>,
    row_base_floors: Vec<S>,
    column_receipts: Vec<GridContributionReceipt<Node, S>>,
    row_receipts: Vec<GridContributionReceipt<Node, S>>,
    children: Vec<Node>,
    placed_areas: Vec<Option<GridArea<S>>>,
    column_area_sizes: Vec<S>,
    row_area_sizes: Vec<S>,
    axis: BeginAxis,
    column_contributions: Option<SubgridIntrinsicContributionReport<Node, S>>,
    traversal: Option<BeginTraversal<'input, Node, S>>,
    /// The contributions of the current axis, from its finished traversal.
    finished: Option<SubgridIntrinsicContributionReport<Node, S>>,
    /// The leaf queries of the current axis, while they are collected.
    finishing: Option<Box<SubgridContributionFinish<'input, Node, S>>>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> IntrinsicTrackBegin<'input, Node, S> {
    pub(in crate::grid) fn new<Tree, M>(
        tree: &mut Tree,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        lower_bounds: IntrinsicGridLowerBounds<'_, S>,
    ) -> Self
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let style = grid.style;
        let column_tracks = grid.column_tracks;
        let row_tracks = grid.row_tracks;
        let column_count = column_tracks.len();
        let row_count = row_tracks.len();
        let mut column_base_floors: Vec<S> = lower_bounds
            .columns
            .map(|bounds| bounds.iter().copied().take(column_count).collect())
            .unwrap_or_else(|| vec![S::ZERO; column_count]);
        column_base_floors.resize(column_count, S::ZERO);
        let mut row_base_floors: Vec<S> = lower_bounds
            .rows
            .map(|bounds| bounds.iter().copied().take(row_count).collect())
            .unwrap_or_else(|| vec![S::ZERO; row_count]);
        row_base_floors.resize(row_count, S::ZERO);
        let columns = vec![S::ZERO; column_count];
        let rows = vec![S::ZERO; row_count];
        let zero_columns: Vec<S> = grid
            .query
            .columns
            .map_or_else(|| vec![S::ZERO; column_count], <[_]>::to_vec);
        let zero_rows: Vec<S> = grid
            .query
            .rows
            .map_or_else(|| vec![S::ZERO; row_count], <[_]>::to_vec);
        let children = tree.children(node).collect::<Vec<_>>();
        let column_geometry = grid.column_gutters.map(|gutters| {
            UsedGridAxisGeometryOf::from_sizing_gutters(zero_columns.clone(), gutters)
        });
        let row_geometry = grid
            .row_gutters
            .map(|gutters| UsedGridAxisGeometryOf::from_sizing_gutters(zero_rows.clone(), gutters));
        let lane_grid_axis = style
            .display
            .establishes_grid_lanes_formatting_context()
            .then(|| grid_axis_for_grid_lanes(style));
        let placed_areas = if let Some(axis) = lane_grid_axis {
            let geometry = match axis {
                GridAxisKind::Column => column_geometry.clone().unwrap_or_else(|| {
                    UsedGridAxisGeometryOf::new(
                        zero_columns.clone(),
                        vec![false; column_count],
                        S::ZERO,
                    )
                }),
                GridAxisKind::Row => row_geometry.clone().unwrap_or_else(|| {
                    UsedGridAxisGeometryOf::new(zero_rows.clone(), vec![false; row_count], S::ZERO)
                }),
            };
            let mut areas = vec![None; children.len()];
            // The Lane owner visits every automatic candidate. This Grid role
            // retains only definite incidence, with no stacking-axis track span.
            for (index, area) in crate::grid::lanes::lane_intrinsic_wrapper_areas(
                &grid.placements.items,
                grid.lines,
                axis,
                &geometry,
                false,
            ) {
                let placement = match axis {
                    GridAxisKind::Column => grid.placements.items[index].column,
                    GridAxisKind::Row => grid.placements.items[index].row,
                };
                if has_definite_line(placement) {
                    areas[index] = Some(area);
                }
            }
            areas
        } else {
            resolve_grid_child_areas_with_geometry(
                ResolveGridChildAreasInput {
                    children: &children,
                    placements: grid.placements,
                    style,
                    columns: &zero_columns,
                    rows: &zero_rows,
                    gap: LogicalSizeOf::new(S::ZERO, S::ZERO),
                    lines: grid.lines,
                },
                column_geometry.as_ref(),
                row_geometry.as_ref(),
            )
        };
        let column_area_sizes = grid
            .query
            .columns
            .map_or_else(|| columns.clone(), <[_]>::to_vec);
        let row_area_sizes = grid.query.rows.map_or_else(|| rows.clone(), <[_]>::to_vec);
        Self {
            columns,
            rows,
            column_base_floors,
            row_base_floors,
            column_receipts: Vec::new(),
            row_receipts: Vec::new(),
            children,
            placed_areas,
            column_area_sizes,
            row_area_sizes,
            axis: BeginAxis::Column,
            column_contributions: None,
            traversal: None,
            finished: None,
            finishing: None,
        }
    }

    fn grid_axis(&self) -> GridAxisKind {
        match self.axis {
            BeginAxis::Column => GridAxisKind::Column,
            BeginAxis::Row => GridAxisKind::Row,
        }
    }

    /// The input of the current axis's contributions.
    fn contribution_input<'a>(
        &'a mut self,
        node: Node,
        grid: IntrinsicGrid<'input, 'a, Node, S>,
    ) -> SubgridIntrinsicContributionInput<'input, 'a, Node, S> {
        let (tracks, sizes, base_floors) = match self.axis {
            BeginAxis::Column => (
                grid.column_tracks,
                &mut self.columns[..],
                &mut self.column_base_floors[..],
            ),
            BeginAxis::Row => (
                grid.row_tracks,
                &mut self.rows[..],
                &mut self.row_base_floors[..],
            ),
        };
        SubgridIntrinsicContributionInput {
            grid,
            owner: node,
            constants: grid.constants,
            container_style: grid.style,
            placements: grid.placements,
            axis: match self.axis {
                BeginAxis::Column => GridAxisKind::Column,
                BeginAxis::Row => GridAxisKind::Row,
            },
            tracks,
            sizes,
            base_floors,
            available: grid.query.available,
            children: &self.children,
            placed_areas: &self.placed_areas,
            subgrid_report: grid.subgrid_report,
            column_sizes: &self.column_area_sizes,
            row_sizes: &self.row_area_sizes,
            query: grid.query,
        }
    }

    /// Collects the contributions of each axis in turn until one needs its
    /// subgrid items traversed, or both are done.
    pub(in crate::grid) fn advance<M>(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
    ) -> LayoutResultOf<Node, IntrinsicTrackBeginStep<Node, S>, S, M> {
        // A traversal that waits on a computation resumes where it stopped.
        if self.traversal.is_some() || self.finishing.is_some() {
            return Ok(IntrinsicTrackBeginStep::Traverse(Vec::new()));
        }
        loop {
            let axis = self.grid_axis();
            let report = if let Some(report) = self.finished.take() {
                report
            } else if !grid.query.includes(axis) {
                SubgridIntrinsicContributionReport::empty(
                    node,
                    axis,
                    match axis {
                        GridAxisKind::Column => grid.column_tracks.len(),
                        GridAxisKind::Row => grid.row_tracks.len(),
                    },
                    grid.constants.flow_axes,
                    grid_baseline_reference(grid.style.baseline_references, axis),
                )
            } else {
                match start_subgrid_intrinsic_contributions(&self.contribution_input(node, grid)) {
                    SubgridContributionStart::Empty(report) => report,
                    SubgridContributionStart::Traverse { facts } => {
                        let root = RootTraversal::new(&traversal_input(
                            grid,
                            axis,
                            &self.children,
                            &self.placed_areas,
                            &self.column_area_sizes,
                            &self.row_area_sizes,
                        ));
                        self.traversal = Some(BeginTraversal {
                            facts: facts.clone(),
                            root,
                            frames: None,
                            fold: None,
                            waiting_root_probe: false,
                        });
                        return Ok(IntrinsicTrackBeginStep::Traverse(facts));
                    }
                }
            };
            if let Some(role) = self.axis_done::<M>(node, grid, report)? {
                return Ok(IntrinsicTrackBeginStep::Done(Box::new(role)));
            }
        }
    }

    /// Takes the contributions of the current axis and moves to the next axis.
    /// The role once both axes are done.
    fn axis_done<M>(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        report: SubgridIntrinsicContributionReport<Node, S>,
    ) -> LayoutResultOf<Node, Option<IntrinsicTrackRole<Node, S>>, S, M> {
        match self.axis {
            BeginAxis::Column => {
                self.column_receipts.extend(report.receipts.iter().copied());
                self.column_contributions = Some(report);
                self.column_area_sizes = grid
                    .query
                    .columns
                    .map_or_else(|| self.columns.clone(), <[_]>::to_vec);
                self.row_area_sizes = grid
                    .query
                    .rows
                    .map_or_else(|| self.rows.clone(), <[_]>::to_vec);
                self.axis = BeginAxis::Row;
                Ok(None)
            }
            BeginAxis::Row => {
                self.row_receipts.extend(report.receipts.iter().copied());
                let subgrid_row_count = report.row_contributions.len();
                let row_contributions = report.row_contributions;
                let Some(columns) = self.column_contributions.take() else {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::InternalInvariant(
                            LayoutInternalInvariant::InvalidGridTrackSizing,
                        ),
                    ));
                };
                Ok(Some(IntrinsicTrackRole {
                    children: std::mem::take(&mut self.children)
                        .into_iter()
                        .zip(std::mem::take(&mut self.placed_areas))
                        .enumerate(),
                    sizes: IntrinsicTrackSizes {
                        purpose: grid.query.purpose,
                        columns: std::mem::take(&mut self.columns),
                        rows: std::mem::take(&mut self.rows),
                        column_base_floors: std::mem::take(&mut self.column_base_floors),
                        row_base_floors: std::mem::take(&mut self.row_base_floors),
                        column_receipts: std::mem::take(&mut self.column_receipts),
                        row_receipts: std::mem::take(&mut self.row_receipts),
                    },
                    row_contributions,
                    subgrid_row_count,
                    column_baselines: columns.ancestor_baseline_group,
                }))
            }
        }
    }

    /// Visits the root items of the current axis's traversal until one is a
    /// wrapper whose children remain to be traversed.
    pub(in crate::grid) fn next_wrapper<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        fold: &mut SubgridIntrinsicFold<'input, Node, S>,
    ) -> NextWrapper<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let axis = self.grid_axis();
        let Self {
            traversal,
            children,
            placed_areas,
            column_area_sizes,
            row_area_sizes,
            ..
        } = self;
        let Some(traversal) = traversal else {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidGridTrackSizing,
                ),
            ));
        };
        let input = traversal_input(
            grid,
            axis,
            children,
            placed_areas,
            column_area_sizes,
            row_area_sizes,
        );
        traversal.root.next_wrapper::<Tree, M>(tree, &input, fold)
    }

    /// Steps the traversal of the current axis's subgrid items and the
    /// collection of the contributions it leads to, until a computation they
    /// wait on or the end of both.
    pub(in crate::grid) fn advance_traversal<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
    ) -> LayoutResultOf<Node, TraversalProgress<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            if self.finishing.is_some() {
                return self.advance_finish::<Tree, M>(tree, node, grid);
            }
            let Some(traversal) = self.traversal.as_mut() else {
                return Err(invalid_track_sizing(node));
            };
            let mut frames = traversal
                .frames
                .take()
                .unwrap_or_else(|| Box::new(WrapperFrames::new()));
            let mut fold = traversal.fold.take().unwrap_or_else(|| {
                Box::new(SubgridIntrinsicFold::new(IntrinsicMinTrackFacts::Known(
                    &traversal.facts,
                )))
            });
            match self.walk::<Tree, M>(tree, node, grid, &mut frames, &mut fold) {
                Err(error) => {
                    frames.close_captures(tree);
                    return Err(error);
                }
                Ok(Walk::Request(request)) => {
                    let traversal = self.traversal.as_mut().expect("the traversal is active");
                    traversal.frames = Some(frames);
                    traversal.fold = Some(fold);
                    return Ok(TraversalProgress::Request(request));
                }
                Ok(Walk::Finished) => self.begin_finish(node, grid, fold.finish()),
            }
        }
    }

    /// Steps the walk over the wrappers of the traversal until it waits on a
    /// computation or its root items and wrappers are exhausted.
    fn walk<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        frames: &mut WrapperFrames<'input, Node, S>,
        fold: &mut SubgridIntrinsicFold<'input, Node, S>,
    ) -> LayoutResultOf<Node, Walk<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            match frames.step::<Tree, M>(tree, fold)? {
                FrameStep::Continue => {}
                FrameStep::Request(request) => return Ok(Walk::Request(request)),
                FrameStep::Idle => match self.next_wrapper::<Tree, M>(tree, node, grid, fold)? {
                    WalkNext::Wrapper(entry) => frames.enter::<Tree, M>(tree, fold, *entry)?,
                    WalkNext::Probe(request) => {
                        let traversal = self.traversal.as_mut().expect("the traversal is active");
                        traversal.waiting_root_probe = true;
                        return Ok(Walk::Request(TrackChildRequest::context(request)));
                    }
                    WalkNext::Done => return Ok(Walk::Finished),
                },
            }
        }
    }

    /// Accepts the output of the computation the traversal or the collection
    /// of its contributions waits on. The gutter a probe observed comes with
    /// it.
    pub(in crate::grid) fn accept_traversal<M>(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        output: ComputeOutputOf<S>,
        gutter: Option<Edges<S>>,
    ) -> LayoutResultOf<Node, (), S, M> {
        if self.finishing.is_some() {
            return self.accept_finish::<M>(node, grid, output);
        }
        let Some(traversal) = self.traversal.as_mut() else {
            return Err(invalid_track_sizing(node));
        };
        if std::mem::take(&mut traversal.waiting_root_probe) {
            traversal.root.accept_probe(gutter);
            return Ok(());
        }
        let Some(frames) = traversal.frames.as_mut() else {
            return Err(invalid_track_sizing(node));
        };
        frames.accept::<M>(output, gutter)?;
        Ok(())
    }

    /// Accepts the gutter a probe of a root item of the traversal observed.
    pub(in crate::grid) fn accept_root_probe(&mut self, gutter: Option<Edges<S>>) {
        if let Some(traversal) = self.traversal.as_mut() {
            traversal.root.accept_probe(gutter);
        }
    }

    /// Closes the effect captures the traversal's waiting preparations hold,
    /// innermost first.
    pub(in crate::grid) fn close_captures<Tree, M>(&mut self, tree: &mut Tree)
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if let Some(mut frames) = self
            .traversal
            .as_mut()
            .and_then(|traversal| traversal.frames.take())
        {
            frames.close_captures(tree);
        }
    }

    /// Whether the traversal, or the collection of its contributions, waits on
    /// a computation.
    pub(in crate::grid) fn is_waiting(&self) -> bool {
        self.finishing
            .as_ref()
            .is_some_and(|finishing| finishing.is_waiting())
            || self
                .traversal
                .as_ref()
                .is_some_and(|traversal| traversal.frames.is_some() || traversal.waiting_root_probe)
    }

    /// Takes the report of the current axis's traversal and begins collecting
    /// the contributions it leads to.
    pub(in crate::grid) fn begin_finish(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        traversal: Result<SubgridTraversalReport<'input, Node, S>, SubgridTraversalError>,
    ) {
        let _ = node;
        self.traversal = None;
        let start = {
            let mut input = self.contribution_input(node, grid);
            SubgridContributionFinish::begin(&mut input, traversal)
        };
        match start {
            SubgridContributionFinishStart::Empty(report) => self.finished = Some(report),
            SubgridContributionFinishStart::Collecting(finishing) => {
                self.finishing = Some(finishing);
            }
        }
    }

    /// The next leaf query of the contributions being collected; `Done` once
    /// they are collected.
    pub(in crate::grid) fn advance_finish<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
    ) -> LayoutResultOf<Node, TraversalProgress<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Some(mut finishing) = self.finishing.take() else {
            return Err(invalid_track_sizing(node));
        };
        let step = {
            let input = self.contribution_input(node, grid);
            finishing.advance::<Tree, M>(tree, &input)?
        };
        match step {
            ControlFlow::Continue(request) => {
                self.finishing = Some(finishing);
                Ok(TraversalProgress::Request(request))
            }
            ControlFlow::Break(()) => {
                let report = finishing.finish(tree, self.contribution_input(node, grid))?;
                self.finished = Some(report);
                Ok(TraversalProgress::Done)
            }
        }
    }

    pub(in crate::grid) fn accept_finish<M>(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let Some(mut finishing) = self.finishing.take() else {
            return Err(invalid_track_sizing(node));
        };
        let accepted = {
            let input = self.contribution_input(node, grid);
            finishing.accept::<M>(&input, output)
        };
        self.finishing = Some(finishing);
        accepted
    }
}

/// What the traversal of one axis's subgrid items leaves to do.
pub(in crate::grid) enum TraversalProgress<'input, Node, S: LayoutScalar> {
    /// It waits on a computation.
    Request(TrackChildRequest<'input, Node, S>),
    /// Its contributions are collected.
    Done,
}

enum Walk<'input, Node, S: LayoutScalar> {
    Request(TrackChildRequest<'input, Node, S>),
    Finished,
}

fn invalid_track_sizing<Node: Copy, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::InvalidGridTrackSizing),
    )
}

type IntrinsicTrackStep<Node, S, M> =
    LayoutResultOf<Node, Option<IntrinsicTrackChild<Node, S>>, S, M>;

impl<Node: Copy + Eq, S: LayoutScalar> IntrinsicTrackRole<Node, S> {
    /// The computation the current query of `child` waits on.
    pub(in crate::grid) fn child_request<'input, Tree, M>(
        &self,
        tree: &Tree,
        grid: IntrinsicGrid<'input, '_, Node, S>,
        child: &IntrinsicTrackChild<Node, S>,
    ) -> LayoutResultOf<Node, TrackChildRequest<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if !child.contextual {
            return Ok(TrackChildRequest::child(child.node, child.input(grid)?));
        }
        let item = child.item.as_ref().expect("active track item");
        let query_input = child.query.as_ref().expect("active track query").input();
        let args = self.child_input(grid, item);
        Ok(
            match prepare_intrinsic_grid_child_request::<Tree, M>(
                tree,
                child.node,
                args,
                query_input,
            )? {
                IntrinsicChildRequest::Child(input) => TrackChildRequest::child(child.node, input),
                IntrinsicChildRequest::Context(request) => TrackChildRequest::context(request),
            },
        )
    }

    pub(in crate::grid) fn next<'input, Tree, M>(
        &mut self,
        grid: IntrinsicGrid<'input, '_, Node, S>,
    ) -> IntrinsicTrackStep<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let style = grid.style;
        let constants = grid.constants;
        let column_tracks = grid.column_tracks;
        let row_tracks = grid.row_tracks;
        let column_count = column_tracks.len();
        let row_count = row_tracks.len();
        while let Some((index, (child, area))) = self.children.next() {
            let child_style = grid.placements.item_input(index);
            let columns = &self.sizes.columns;
            let rows = &self.sizes.rows;
            if !is_in_flow_grid_child(child_style) {
                continue;
            }

            let Some(mut area) = area else {
                continue;
            };
            if area.row >= row_count || area.column >= column_count {
                continue;
            }

            let column_start = area.column;
            let column_end = area.column_end;
            let row_start = area.row;
            let row_end = area.row_end;
            let column_span_tracks = column_tracks.get(column_start..column_end);
            let row_span_tracks = row_tracks.get(row_start..row_end);
            area.size = LogicalSizeOf::new(
                grid.span_extent(
                    GridAxisKind::Column,
                    grid.query.columns.unwrap_or(columns),
                    column_start,
                    column_end,
                ),
                grid.span_extent(
                    GridAxisKind::Row,
                    grid.query.rows.unwrap_or(rows),
                    row_start,
                    row_end,
                ),
            );
            let inherited_column_subgrid = grid
                .subgrid_report
                .items
                .get(index)
                .is_some_and(|item| item_inherits_parent_axis(*item, GridAxisKind::Column));
            let inherited_row_subgrid = grid
                .subgrid_report
                .items
                .get(index)
                .is_some_and(|item| item_inherits_parent_axis(*item, GridAxisKind::Row));
            let column_margin_basis =
                if grid.query.includes(GridAxisKind::Column) && !inherited_column_subgrid {
                    column_span_tracks.and_then(|tracks| {
                        grid.column_contribution_margin_basis(tracks, constants.node_inner_size)
                    })
                } else {
                    None
                };
            let align_self = child_style
                .align_self
                .or(style.align_items)
                .unwrap_or(AlignItems::Stretch);
            let justify_self = child_style
                .justify_self
                .or(style.justify_items)
                .unwrap_or(AlignItems::Stretch);
            let contributes_row = grid.query.includes(GridAxisKind::Row)
                && !inherited_row_subgrid
                && row_span_tracks.is_some_and(|tracks| {
                    tracks.iter().any(|track| {
                        grid.query
                            .track_needs_contribution(track, grid.track_basis.block)
                    })
                });
            let row_baseline_candidate = grid.query.includes(GridAxisKind::Row)
                && !inherited_row_subgrid
                && row_span_tracks.is_some()
                && matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline);
            if column_margin_basis.is_none() && !contributes_row && !row_baseline_candidate {
                continue;
            }

            let item = IntrinsicTrackItem {
                index,
                child,
                area,
                column_margin_basis,
                align_self,
                justify_self,
                contributes_row,
                row_baseline_candidate,
            };
            let args = self.child_input(grid, &item);
            let query = prepare_intrinsic_grid_child::<Tree, M>(child, &args)?;
            return Ok(Some(IntrinsicTrackChild {
                node: child,
                query: Some(query),
                item: Some(item),
                contextual: args.subgrid_item.is_some_and(|item| item.needs_context()),
            }));
        }
        Ok(None)
    }

    fn child_input<'input, 'a>(
        &'a self,
        grid: IntrinsicGrid<'input, 'a, Node, S>,
        item: &IntrinsicTrackItem<Node, S>,
    ) -> IntrinsicGridChildInput<'input, 'a, Node, S> {
        let IntrinsicTrackItem {
            index,
            area,
            align_self,
            justify_self,
            ..
        } = *item;
        let child_style = grid.placements.item_input(index);
        let constants = grid.constants;
        let columns = &self.sizes.columns;
        let rows = &self.sizes.rows;
        IntrinsicGridChildInput {
            child_style,
            child_container: grid.placements.nested_container_input(index),
            grid,
            area,
            columns: grid.query.columns.unwrap_or(columns),
            rows: grid.query.rows.unwrap_or(rows),
            subgrid_item: grid.subgrid_report.items.get(index).copied(),
            input: ComputeInputOf::for_child(
                if matches!(align_self, AlignItems::Baseline | AlignItems::LastBaseline)
                    || matches!(
                        justify_self,
                        AlignItems::Baseline | AlignItems::LastBaseline
                    )
                {
                    RunMode::PerformLayout
                } else {
                    RunMode::ComputeSize
                },
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(
                    constants.node_inner_size.width,
                    constants.node_inner_size.height,
                ),
                crate::output::ContainingLayoutContext::new(
                    grid.constants.flow_axes,
                    crate::output::ParentFormattingContext::Grid,
                ),
                SizingRequestOf::new(grid.query.available, constants.child_orthogonal_fallback),
            ),
        }
    }

    pub(in crate::grid) fn accept_child<M>(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'_, '_, Node, S>,
        child: &mut IntrinsicTrackChild<Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, bool, S, M> {
        let Some(item) = child.accept(output)? else {
            return Ok(false);
        };
        self.accept(node, grid, item, output)?;
        Ok(true)
    }

    fn accept<M>(
        &mut self,
        node: Node,
        grid: IntrinsicGrid<'_, '_, Node, S>,
        item: IntrinsicTrackItem<Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let IntrinsicTrackItem {
            index,
            child,
            area,
            column_margin_basis,
            align_self,
            justify_self,
            contributes_row,
            row_baseline_candidate,
        } = item;
        let child_style = grid.placements.item_input(index);
        let constants = grid.constants;
        let GridArea {
            column: column_start,
            column_end,
            row: row_start,
            row_end,
            ..
        } = area;
        let column_span_tracks = grid.column_tracks.get(column_start..column_end);
        let row_span_tracks = grid.row_tracks.get(row_start..row_end);
        let contribution_margin = |parent_size| {
            intrinsic_contribution_margin(child_style, constants.flow_axes, parent_size)
                .map_err(|status| crate::error::value_resolution_error(child, status))
        };
        let column_contribution_size =
            grid_axis_size(grid.sizing_flow_axes, output.size, GridAxisKind::Column);
        let row_contribution_size =
            grid_axis_size(grid.sizing_flow_axes, output.size, GridAxisKind::Row);

        if let Some(margin_basis) = column_margin_basis {
            let margin = contribution_margin(margin_basis)?;
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                child_style,
                constants.flow_axes,
                margin_basis,
                child_flow_axes,
            )
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
            let column_baseline_member = ancestor_baseline_member(AncestorBaselineMemberInput {
                reference: grid_baseline_reference(
                    grid.style.baseline_references,
                    GridAxisKind::Column,
                ),
                source: child,
                axis: GridAxisKind::Column,
                ancestor_span: GridTrackSpan::new(column_start + 1, column_end + 1),
                alignment_context_reversed: false,
                alignment: justify_self,
                block_auto_margins,
                synthesized_baseline_cycle: column_span_tracks.is_some_and(|tracks| {
                    grid_baseline_span_would_cycle(tracks, grid.track_basis.inline)
                }),
                output,
                margin,
                child_flow_axes,
                containing_flow_axes: constants.flow_axes,
                start_adjustment: S::ZERO,
                end_adjustment: S::ZERO,
            })
            .map_err(|error| grid_baseline_error(node, child, error))?;
            let column_baseline_shim = column_baseline_member
                .map_or(BaselineShim::default(), |member| {
                    self.column_baselines.intrinsic_shim(member)
                });
            let column_contribution = column_contribution_size
                + grid.sizing_flow_axes.logical_edges(margin).inline_sum()
                + column_baseline_shim.before
                + column_baseline_shim.after;
            self.sizes.column_receipts.push(GridContributionReceipt {
                purpose: grid.query.purpose,
                source: child,
                axis: GridAxisKind::Column,
                span: GridTrackSpan::new(column_start + 1, column_end + 1),
                value: column_contribution,
            });
            if column_end == column_start + 1 {
                self.sizes.columns[column_start] =
                    self.sizes.columns[column_start].max(column_contribution);
            }
        }

        if contributes_row || row_baseline_candidate {
            let margin = contribution_margin(constants.node_inner_size)?;
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                child_style,
                constants.flow_axes,
                constants.node_inner_size,
                child_flow_axes,
            )
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
            let baseline_member = ancestor_baseline_member(AncestorBaselineMemberInput {
                reference: grid_baseline_reference(
                    grid.style.baseline_references,
                    GridAxisKind::Row,
                ),
                source: child,
                axis: GridAxisKind::Row,
                ancestor_span: GridTrackSpan::new(row_start + 1, row_end + 1),
                alignment_context_reversed: false,
                alignment: align_self,
                block_auto_margins,
                synthesized_baseline_cycle: row_span_tracks.is_some_and(|tracks| {
                    grid_baseline_span_would_cycle(tracks, grid.track_basis.block)
                }),
                output,
                margin,
                child_flow_axes,
                containing_flow_axes: constants.flow_axes,
                start_adjustment: S::ZERO,
                end_adjustment: S::ZERO,
            })
            .map_err(|error| grid_baseline_error(node, child, error))?;
            self.row_contributions.push(RowIntrinsicContribution {
                source: child,
                start: row_start,
                end: row_end,
                contributes_to_row_size: contributes_row,
                contribution: row_contribution_size
                    + grid.sizing_flow_axes.logical_edges(margin).block_sum(),
                baseline_member,
            });
        }
        Ok(())
    }

    pub(in crate::grid) fn finish<M>(
        self,
        node: Node,
        grid: IntrinsicGrid<'_, '_, Node, S>,
    ) -> LayoutResultOf<Node, IntrinsicTrackSizes<Node, S>, S, M> {
        let Self {
            mut sizes,
            row_contributions,
            subgrid_row_count,
            ..
        } = self;
        let constants = grid.constants;
        let row_count = grid.row_tracks.len();
        let rows = &mut sizes.rows;
        let row_receipts = &mut sizes.row_receipts;
        let row_baseline_groups = ancestor_baseline_group_for_intrinsic_contributions(
            node,
            &row_contributions,
            row_count,
            constants.flow_axes.block_axis(),
            grid.style.baseline_references.rows,
        );
        let subgrid_receipt_start = row_receipts.len();
        let mut subgrid_receipt_end = subgrid_receipt_start;
        for (index, item) in row_contributions.into_iter().enumerate() {
            if !item.contributes_to_row_size {
                continue;
            }
            let shim = row_baseline_shim(item, &row_baseline_groups);
            let contribution = item.contribution + shim.before + shim.after;
            row_receipts.push(GridContributionReceipt {
                purpose: grid.query.purpose,
                source: item.source,
                axis: GridAxisKind::Row,
                span: GridTrackSpan::new(item.start + 1, item.end + 1),
                value: contribution,
            });
            if index < subgrid_row_count {
                subgrid_receipt_end += 1;
            }
            if item.end == item.start + 1 {
                rows[item.start] = rows[item.start].max(contribution);
            }
        }
        super::subgrid_intrinsic::reduce_subgrid_candidate_receipts(
            row_receipts,
            subgrid_receipt_start..subgrid_receipt_end,
        )
        .map_err(|error| grid_track_sizing_error(node, error))?;

        Ok(sizes)
    }
}

fn ancestor_baseline_group_for_intrinsic_contributions<Node: Copy, S: LayoutScalar>(
    owner: Node,
    contributions: &[RowIntrinsicContribution<Node, S>],
    row_count: usize,
    expected_axis: crate::geometry::PhysicalAxis,
    reference: GridBaselineReference,
) -> AncestorBaselineGroup<Node, S> {
    AncestorBaselineGroup::reduce(
        owner,
        GridAxisKind::Row,
        expected_axis,
        reference,
        row_count,
        contributions
            .iter()
            .filter_map(|contribution| contribution.baseline_member),
    )
}

fn row_baseline_shim<Node: Copy, S: LayoutScalar>(
    item: RowIntrinsicContribution<Node, S>,
    group: &AncestorBaselineGroup<Node, S>,
) -> BaselineShim<S> {
    let Some(member) = item.baseline_member else {
        return BaselineShim::default();
    };
    group.intrinsic_shim(member)
}

pub(in crate::grid) struct AncestorBaselineMemberInput<Node, S: LayoutScalar = DefaultScalar> {
    pub(in crate::grid) source: Node,
    pub(in crate::grid) reference: GridBaselineReference,
    pub(in crate::grid) axis: GridAxisKind,
    pub(in crate::grid) ancestor_span: GridTrackSpan,
    pub(in crate::grid) alignment_context_reversed: bool,
    pub(in crate::grid) alignment: AlignItems,
    pub(in crate::grid) block_auto_margins: bool,
    pub(in crate::grid) synthesized_baseline_cycle: bool,
    pub(in crate::grid) output: ComputeOutputOf<S>,
    pub(in crate::grid) margin: Edges<S>,
    pub(in crate::grid) child_flow_axes: FlowAxes,
    pub(in crate::grid) containing_flow_axes: FlowAxes,
    pub(in crate::grid) start_adjustment: S,
    pub(in crate::grid) end_adjustment: S,
}

pub(in crate::grid) fn ancestor_baseline_member<Node: Copy, S: LayoutScalar>(
    input: AncestorBaselineMemberInput<Node, S>,
) -> Result<Option<AncestorBaselineMember<Node, S>>, GridBaselineSelectionError> {
    let physical_axis = grid_axis_physical_axis(input.containing_flow_axes, input.axis);
    if input.block_auto_margins {
        return Ok(None);
    }
    let Some(selection) = select_grid_baseline(
        input.reference,
        input.alignment,
        input.child_flow_axes,
        physical_axis,
        input.output,
    )?
    else {
        return Ok(None);
    };
    if selection.synthesized && input.synthesized_baseline_cycle {
        return Ok(None);
    }
    let selected_port = selection.set;
    let use_start =
        (selected_port == AncestorBaselineRole::First) != input.alignment_context_reversed;
    let Some(selected_track) = (if use_start {
        input.ancestor_span.start.checked_sub(1)
    } else {
        input.ancestor_span.end.checked_sub(2)
    }) else {
        return Ok(None);
    };
    let physical_coordinate = selection.coordinate;
    let physical_extent = grid_axis_size(input.containing_flow_axes, input.output.size, input.axis);
    let subject_progression = input
        .child_flow_axes
        .physical_axis_progression(physical_axis);
    let containing_progression = input
        .containing_flow_axes
        .physical_axis_progression(physical_axis);
    let role = selected_port.in_progression(subject_progression, containing_progression);
    let border_logical_baseline = if containing_progression.is_decreasing() {
        physical_extent - physical_coordinate
    } else {
        physical_coordinate
    };
    let logical_margin = input.containing_flow_axes.logical_edges(input.margin);
    let (start_margin, end_margin) = match input.axis {
        GridAxisKind::Column => (logical_margin.inline_start, logical_margin.inline_end),
        GridAxisKind::Row => (logical_margin.block_start, logical_margin.block_end),
    };
    let containing_logical_distance = match role {
        AncestorBaselineRole::First => {
            input.start_adjustment + start_margin + border_logical_baseline
        }
        AncestorBaselineRole::Last => {
            input.end_adjustment + end_margin + physical_extent - border_logical_baseline
        }
    };
    if input.reference != GridBaselineReference::Physical
        && (!containing_logical_distance.is_finite() || !border_logical_baseline.is_finite())
    {
        return Err(GridBaselineSelectionError::Invalid(
            crate::node_input::GridBaselineError::UnrepresentableCoordinate,
        ));
    }
    Ok(Some(AncestorBaselineMember {
        reference: input.reference,
        source: input.source,
        axis: input.axis,
        physical_axis,
        ancestor_span: input.ancestor_span,
        selected_track,
        selected_port,
        subject_progression,
        containing_progression,
        alignment_context_reversed: input.alignment_context_reversed,
        containing_logical_distance,
        border_logical_baseline,
    }))
}

#[cfg(test)]
pub(in crate::grid) fn baseline_geometry_for_intrinsic_contribution<S: LayoutScalar>(
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    flow_axes: FlowAxes,
) -> BaselineGeometry<S> {
    let baselines = output.baselines();
    let first_baseline = baselines.first_or_synthesize_block_baseline(flow_axes, output.size);
    let last_baseline = baselines.last_or_synthesize_block_baseline(flow_axes, output.size);
    BaselineGeometry {
        available_span_size: S::ZERO,
        margin_box_size: flow_axes.block_axis_extent(output.size)
            + flow_axes.line_over_edge(margin)
            + flow_axes.line_under_edge(margin),
        major_baseline: PhysicalBaseline::new(
            first_baseline.axis(),
            flow_axes.line_over_edge(margin) + first_baseline.coordinate(),
        ),
        minor_baseline: PhysicalBaseline::new(
            last_baseline.axis(),
            flow_axes.line_under_edge(margin) + flow_axes.block_axis_extent(output.size)
                - last_baseline.coordinate(),
        ),
    }
}

pub(super) fn block_auto_margins_for_intrinsic_contribution<'input, S: LayoutScalar>(
    style: &GridItemProjection<'input, S>,
    containing_flow_axes: FlowAxes,
    parent_size: Size<Option<S>>,
    child_flow_axes: FlowAxes,
) -> Result<bool, LengthResolutionStatus<S>> {
    let margin = containing_flow_axes.zip_physical_edges_with_inline_extent(
        style.margin,
        parent_size,
        |length, basis| resolve_auto_optional(length, basis),
    );
    Ok(child_flow_axes.line_over_edge(margin)?.is_none()
        || child_flow_axes.line_under_edge(margin)?.is_none())
}

pub(in crate::grid) fn track_basis_dependent_space<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: S,
) -> S {
    tracks
        .iter()
        .filter(|track| track_has_percent_sizing(track))
        .map(|track| {
            track_base_size(track, Some(basis), S::ZERO)
                - track_base_size(track, Some(S::ZERO), S::ZERO)
        })
        .fold(S::ZERO, |sum, value| sum + value)
}

// Qualification is tied to the actual sizing-owner basis. A real missing
// percentage basis is Auto here; raw standalone distribution stays separate.
pub(in crate::grid) fn track_minimum_is_intrinsic<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
) -> bool {
    track.min.is_intrinsic()
        || matches!(&track.min, MinTrackSizingOf::Calculation(value) if track_calculation_is_cyclic_auto(value, basis))
}

pub(in crate::grid) fn track_accepts_intrinsic_contribution<S: LayoutScalar>(
    track: &TrackSizingOf<S>,
    basis: Option<S>,
) -> bool {
    track_minimum_is_intrinsic(track, basis)
        || track.max.is_intrinsic()
        || matches!(&track.max, MaxTrackSizingOf::Calculation(value) if track_calculation_is_cyclic_auto(value, basis))
}

pub(in crate::grid) fn intrinsic_contribution_margin<'input, S: LayoutScalar>(
    style: &GridItemProjection<'input, S>,
    containing_flow_axes: crate::geometry::FlowAxes,
    containing_physical_size: Size<Option<S>>,
) -> Result<Edges<S>, LengthResolutionStatus<S>> {
    let margin = containing_flow_axes.zip_physical_edges_with_inline_extent(
        style.margin,
        containing_physical_size,
        resolve_auto_or_zero,
    );
    Ok(Edges::new(
        margin.top?,
        margin.right?,
        margin.bottom?,
        margin.left?,
    ))
}
