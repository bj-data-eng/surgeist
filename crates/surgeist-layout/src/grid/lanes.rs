use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{LogicalAxis, LogicalPointOf, LogicalSizeOf, PhysicalAxis};
use crate::grid::intrinsic_stage::{IntrinsicStageFrame, intrinsic_role_grid};
use crate::sizing::resolve::{
    ResolvedPreferredSize, SizingResolutionError, resolve_maximum_optional,
    resolve_minimum_optional, resolve_preferred_sizing,
};
use crate::{
    GridFlowToleranceOf, LayoutErrorSiteOf, LengthResolutionOf, LengthResolutionStatus,
    MinTrackSizingOf, PercentageBasisOf,
};
use std::collections::TryReserveError;
use std::num::NonZeroUsize;
use std::ops::Range;

mod packed;
use packed::{PackedLaneItem, PackedLanePlacement};

/// Inputs for sparse placement over a finite number of equal-size tracks.
///
/// Dense placement requires track-size compatibility and is owned by
/// container layout, which consumes `NodeInputOf::grid_lanes_flow`.
#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementInputOf<Item, S: LayoutScalar = DefaultScalar> {
    pub grid_axis_tracks: usize,
    pub grid_axis: GridAxisKind,
    pub reversed: bool,
    pub lane_gap: S,
    pub tolerance: GridFlowToleranceOf<S>,
    pub tolerance_basis: S,
    pub items: Vec<LaneItemOf<Item, S>>,
}

pub type LanePlacementInput<Item> = LanePlacementInputOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOf<Item, S: LayoutScalar = DefaultScalar> {
    pub item: Item,
    pub grid_axis_span: usize,
    pub definite_grid_axis_start: Option<usize>,
    pub lane_axis_margin_box: S,
}

pub type LaneItem<Item> = LaneItemOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneItemOffsetOf<Item, S: LayoutScalar = DefaultScalar> {
    pub item: Item,
    pub grid_axis_start: usize,
    pub grid_axis_span: usize,
    pub offset: S,
    pub lane_axis_margin_box: S,
}

pub type LaneItemOffset<Item> = LaneItemOffsetOf<Item, DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LanePlacementReportOf<Item, S: LayoutScalar = DefaultScalar> {
    pub lane_axis: GridAxisKind,
    pub grid_axis: GridAxisKind,
    pub item_offsets: Vec<LaneItemOffsetOf<Item, S>>,
    pub content_size: S,
}

pub type LanePlacementReport<Item> = LanePlacementReportOf<Item, DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridLanesItemContainingBlockOf<S: LayoutScalar = DefaultScalar> {
    grid_axis: GridAxisKind,
    logical_size: LogicalSizeOf<Option<S>>,
    physical_size: Size<Option<S>>,
}

impl<S: LayoutScalar> GridLanesItemContainingBlockOf<S> {
    pub(super) fn new(
        flow_axes: crate::geometry::FlowAxes,
        grid_axis: GridAxisKind,
        grid_axis_size: S,
        container_content_box_size: LogicalSizeOf<Option<S>>,
    ) -> Self {
        let logical_size = match grid_axis.logical_axis() {
            LogicalAxis::Inline => {
                LogicalSizeOf::new(Some(grid_axis_size), container_content_box_size.block)
            }
            LogicalAxis::Block => {
                LogicalSizeOf::new(container_content_box_size.inline, Some(grid_axis_size))
            }
        };
        Self {
            grid_axis,
            logical_size,
            physical_size: flow_axes.physical_size(logical_size),
        }
    }

    const fn grid_axis(self) -> GridAxisKind {
        self.grid_axis
    }

    const fn logical_size(self) -> LogicalSizeOf<Option<S>> {
        self.logical_size
    }

    const fn physical_size(self) -> Size<Option<S>> {
        self.physical_size
    }

    fn definite_logical_size(self) -> LogicalSizeOf<S> {
        LogicalSizeOf::new(
            self.logical_size
                .inline
                .expect("final grid-lanes inline containing extent is definite"),
            self.logical_size
                .block
                .expect("final grid-lanes block containing extent is definite"),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanePlacementError {
    EmptyTrackList,
    InvalidGridAxisStart {
        start: usize,
    },
    InvalidGridAxisSpan {
        span: usize,
    },
    GridAxisSpanOutOfRange {
        start: usize,
        span: usize,
        tracks: usize,
    },
    ContentSizedTrackOutOfRange {
        track_index: usize,
        tracks: usize,
    },
    InvalidDefiniteLaneSpan {
        span: LaneTrackSpan,
    },
    DefiniteLaneSpanOutOfRange {
        span: LaneTrackSpan,
        tracks: usize,
    },
    InvalidGridFlowToleranceBasis,
    InvalidGridFlowToleranceResolution,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneContributionFactsOf<S: LayoutScalar = DefaultScalar> {
    pub min_content: S,
    pub max_content: S,
    pub min_size: S,
    pub automatic_minimum_applies: bool,
}

pub type LaneContributionFacts = LaneContributionFactsOf<DefaultScalar>;

impl<S: LayoutScalar> LaneContributionFactsOf<S> {
    fn contributions(self) -> LaneContributionsOf<S> {
        let minimum = if self.automatic_minimum_applies {
            self.min_content
        } else {
            self.min_size
        };
        LaneContributionsOf {
            minimum,
            min_content: self.min_content,
            max_content: self.max_content,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LaneContributionsOf<S: LayoutScalar = DefaultScalar> {
    pub(super) minimum: S,
    pub(super) min_content: S,
    pub(super) max_content: S,
}

impl<S: LayoutScalar> From<LaneContributionFactsOf<S>> for LaneContributionsOf<S> {
    fn from(facts: LaneContributionFactsOf<S>) -> Self {
        facts.contributions()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneTrackSpan {
    pub start: usize,
    pub end: usize,
}

impl LaneTrackSpan {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn len(self) -> Option<usize> {
        if self.start == 0 || self.end <= self.start {
            None
        } else {
            Some(self.end - self.start)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneTrackSpanLength(NonZeroUsize);

impl LaneTrackSpanLength {
    #[must_use]
    pub const fn new(span: usize) -> Option<Self> {
        match NonZeroUsize::new(span) {
            Some(span) => Some(Self(span)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingInputOf<S: LayoutScalar = DefaultScalar> {
    pub axis: GridAxisKind,
    pub available: Option<S>,
    pub gap: S,
    pub tracks: Vec<TrackSizingOf<S>>,
    pub content_sized_tracks: Vec<usize>,
    pub items: Vec<LaneIntrinsicItemOf<S>>,
}

pub type LaneIntrinsicSizingInput = LaneIntrinsicSizingInputOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicItemOf<S: LayoutScalar = DefaultScalar> {
    id: &'static str,
    kind: LaneIntrinsicItemKind,
    contribution: LaneContributionFactsOf<S>,
}

pub type LaneIntrinsicItem = LaneIntrinsicItemOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneIntrinsicItemKind {
    Definite { span: LaneTrackSpan },
    Indefinite { span: LaneTrackSpanLength },
}

impl<S: LayoutScalar> LaneIntrinsicItemOf<S> {
    pub fn definite(
        id: &'static str,
        span: LaneTrackSpan,
        contribution: LaneContributionFactsOf<S>,
    ) -> Result<Self, LanePlacementError> {
        if span.len().is_none() {
            return Err(LanePlacementError::InvalidDefiniteLaneSpan { span });
        }
        Ok(Self {
            id,
            kind: LaneIntrinsicItemKind::Definite { span },
            contribution,
        })
    }

    #[must_use]
    pub const fn indefinite(
        id: &'static str,
        span: LaneTrackSpanLength,
        contribution: LaneContributionFactsOf<S>,
    ) -> Self {
        Self {
            id,
            kind: LaneIntrinsicItemKind::Indefinite { span },
            contribution,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.id
    }

    #[must_use]
    pub const fn kind(&self) -> LaneIntrinsicItemKind {
        self.kind
    }

    #[must_use]
    pub const fn contribution(&self) -> LaneContributionFactsOf<S> {
        self.contribution
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DefiniteLaneIntrinsicItemOf<S: LayoutScalar = DefaultScalar> {
    pub id: &'static str,
    pub span: LaneTrackSpan,
    pub contribution: LaneContributionFactsOf<S>,
}

pub type DefiniteLaneIntrinsicItem = DefiniteLaneIntrinsicItemOf<DefaultScalar>;

#[derive(Clone, Debug, PartialEq)]
pub struct IndefiniteLaneContributionGroupOf<S: LayoutScalar = DefaultScalar> {
    pub span: usize,
    pub max_min_content: S,
    pub max_max_content: S,
    pub max_min_size: S,
    pub item_ids: Vec<&'static str>,
}

pub type IndefiniteLaneContributionGroup = IndefiniteLaneContributionGroupOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct LaneIntrinsicEdgeFactsOf<S: LayoutScalar = Scalar> {
    pub(super) start_mbp: S,
    pub(super) end_mbp: S,
    pub(super) start_half_gap: S,
    pub(super) end_half_gap: S,
}

#[derive(Clone, Debug, PartialEq)]
struct LaneIntrinsicEquivalenceKeyOf<S: LayoutScalar = Scalar> {
    span: usize,
    candidate_starts: Vec<usize>,
    edges: LaneIntrinsicEdgeFactsOf<S>,
    contribution_kind: IntrinsicSpanContribution,
}

#[derive(Clone, Debug)]
pub(super) struct ProjectedLaneIntrinsicItemOf<S: LayoutScalar = Scalar> {
    pub(super) id: &'static str,
    pub(super) kind: LaneIntrinsicItemKind,
    pub(super) candidate_starts: Option<Vec<usize>>,
    pub(super) contribution: LaneContributionsOf<S>,
    pub(super) edges: LaneIntrinsicEdgeFactsOf<S>,
    pub(super) contribution_kind: IntrinsicSpanContribution,
}

#[derive(Clone, Debug)]
struct ProjectedLaneIntrinsicGroupOf<S: LayoutScalar = Scalar> {
    key: LaneIntrinsicEquivalenceKeyOf<S>,
    max_min_content: S,
    max_max_content: S,
    max_minimum: S,
    item_ids: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneIntrinsicSizingReportOf<S: LayoutScalar = DefaultScalar> {
    pub definite_items: Vec<DefiniteLaneIntrinsicItemOf<S>>,
    pub indefinite_groups: Vec<IndefiniteLaneContributionGroupOf<S>>,
    pub converted_indefinite_items: Vec<DefiniteLaneIntrinsicItemOf<S>>,
    pub final_track_sizes: Vec<S>,
}

pub type LaneIntrinsicSizingReport = LaneIntrinsicSizingReportOf<DefaultScalar>;

pub(super) fn lane_axis_for_grid_lanes<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
) -> GridAxisKind {
    style.grid_lanes_flow.stacking_axis()
}

pub(super) fn grid_axis_for_grid_lanes<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
) -> GridAxisKind {
    style.grid_lanes_flow.grid_axis()
}

pub(super) fn column_flow_for_grid_lanes<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
) -> bool {
    grid_axis_for_grid_lanes(style) == GridAxisKind::Row
}

pub(super) fn apply_grid_lanes_auto_fit_policy<'input, Node, S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    topology: &mut ExpandedGridTopology<S>,
    placements: &GridPlacementContext<'input, Node, S>,
    track_count: usize,
    explicit_start: usize,
) -> Result<(), GridPlacementDemandError> {
    let axis = grid_axis_for_grid_lanes(style);
    let lines = GridAxisLines {
        explicit_start,
        explicit_count: match axis {
            GridAxisKind::Column => topology.explicit_columns,
            GridAxisKind::Row => topology.explicit_rows,
        },
    };
    let lanes_placements = placements
        .items
        .iter()
        .filter(|placement| placement.in_flow)
        .map(|placement| {
            let placement = match axis {
                GridAxisKind::Column => placement.column,
                GridAxisKind::Row => placement.row,
            };
            let (definite_start, span) = lane_grid_axis_facts(placement, track_count, lines);
            super::topology::LanesAutoFitPlacement {
                definite_start: definite_start.map(|line| line - 1),
                span,
            }
        })
        .collect::<Vec<_>>();
    topology.apply_lanes_auto_fit_policy(axis, track_count, explicit_start, &lanes_placements)
}

/// Places items sparsely over a finite set of tracks.
pub fn place_lanes<Item, S: LayoutScalar>(
    input: LanePlacementInputOf<Item, S>,
) -> Result<LanePlacementReportOf<Item, S>, LanePlacementError> {
    if input.grid_axis_tracks == 0 {
        return Err(LanePlacementError::EmptyTrackList);
    }

    let mut running = vec![S::ZERO; input.grid_axis_tracks];
    let mut item_offsets = Vec::new();
    let mut cursor = 0usize;
    let tolerance = resolve_tolerance(input.tolerance, input.tolerance_basis)?;
    let mut content_size = S::ZERO;
    let collapsed = vec![false; input.grid_axis_tracks];

    for item in input.items {
        let (start_zero, span) = match item.definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 {
                    return Err(LanePlacementError::InvalidGridAxisStart { start: start_line });
                }
                if item.grid_axis_span == 0 {
                    return Err(LanePlacementError::InvalidGridAxisSpan {
                        span: item.grid_axis_span,
                    });
                }
                let start_zero = start_line - 1;
                if start_zero + item.grid_axis_span > input.grid_axis_tracks {
                    return Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: start_line,
                        span: item.grid_axis_span,
                        tracks: input.grid_axis_tracks,
                    });
                }
                (start_zero, item.grid_axis_span)
            }
            None => {
                let span = item.grid_axis_span.clamp(1, input.grid_axis_tracks);
                let start_zero = if matches!(input.tolerance, GridFlowToleranceOf::Infinite) {
                    infinite_candidate_start(cursor, span, &collapsed, input.reversed)
                } else {
                    finite_candidate_start(
                        &running,
                        cursor,
                        span,
                        tolerance,
                        &collapsed,
                        input.reversed,
                    )
                };
                let Some(start_zero) = start_zero else {
                    return Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: 1,
                        span,
                        tracks: input.grid_axis_tracks,
                    });
                };
                (start_zero, span)
            }
        };

        if item.definite_grid_axis_start.is_none() {
            cursor = lane_cursor_after(start_zero, span, input.grid_axis_tracks, input.reversed);
        }
        let previous = max_running_position(&running, start_zero, span);
        let new_position = previous + item.lane_axis_margin_box.max(S::ZERO) + input.lane_gap;
        content_size = content_size.max(previous + item.lane_axis_margin_box);
        for position in &mut running[start_zero..start_zero + span] {
            *position = new_position;
        }

        item_offsets.push(LaneItemOffsetOf {
            item: item.item,
            grid_axis_start: start_zero + 1,
            grid_axis_span: span,
            offset: previous,
            lane_axis_margin_box: item.lane_axis_margin_box,
        });
    }

    Ok(LanePlacementReportOf {
        lane_axis: match input.grid_axis {
            GridAxisKind::Column => GridAxisKind::Row,
            GridAxisKind::Row => GridAxisKind::Column,
        },
        grid_axis: input.grid_axis,
        item_offsets,
        content_size,
    })
}

pub fn lane_intrinsic_sizing<S: LayoutScalar>(
    input: LaneIntrinsicSizingInputOf<S>,
) -> LayoutResultOf<(), Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError>, S> {
    lane_intrinsic_sizing_with(input, LayoutErrorSiteOf::Standalone)
}

pub(super) fn lane_intrinsic_sizing_with<Node, S, M>(
    input: LaneIntrinsicSizingInputOf<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let projected_items = input
        .items
        .iter()
        .map(|item| ProjectedLaneIntrinsicItemOf {
            id: item.id(),
            kind: item.kind(),
            candidate_starts: None,
            contribution: item.contribution().into(),
            edges: LaneIntrinsicEdgeFactsOf::default(),
            contribution_kind: IntrinsicSpanContribution::MinContent {
                prioritize_min_tracks: false,
            },
        })
        .collect::<Vec<_>>();
    lane_intrinsic_sizing_projected_with(&input, &projected_items, None, site)
}

struct LaneProjectedSpanOf<S: LayoutScalar> {
    span: LaneTrackSpan,
    contributions: LaneContributionsOf<S>,
    contribution_kind: IntrinsicSpanContribution,
}

struct LaneProjectedContributionsOf<S: LayoutScalar> {
    definite_items: Vec<DefiniteLaneIntrinsicItemOf<S>>,
    indefinite_groups: Vec<IndefiniteLaneContributionGroupOf<S>>,
    converted_indefinite_items: Vec<DefiniteLaneIntrinsicItemOf<S>>,
    virtual_items: Vec<LaneProjectedSpanOf<S>>,
}

fn project_lane_intrinsic_contributions<S: LayoutScalar>(
    input: &LaneIntrinsicSizingInputOf<S>,
    projected_items: &[ProjectedLaneIntrinsicItemOf<S>],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
) -> Result<LaneProjectedContributionsOf<S>, LanePlacementError> {
    if input.content_sized_tracks.is_empty() || input.tracks.is_empty() {
        return Err(LanePlacementError::EmptyTrackList);
    }
    if let Some(track_index) = input
        .content_sized_tracks
        .iter()
        .copied()
        .find(|track_index| *track_index >= input.tracks.len())
    {
        return Err(LanePlacementError::ContentSizedTrackOutOfRange {
            track_index,
            tracks: input.tracks.len(),
        });
    }

    // The report retains authored public facts. Generated numerical spans use
    // contribution values separately from the public automatic-minimum flag.
    let definite_items = input
        .items
        .iter()
        .filter_map(|item| match item.kind() {
            LaneIntrinsicItemKind::Definite { span } => Some(DefiniteLaneIntrinsicItemOf {
                id: item.id(),
                span,
                contribution: item.contribution(),
            }),
            LaneIntrinsicItemKind::Indefinite { .. } => None,
        })
        .collect::<Vec<_>>();
    let mut definite_virtual_items = Vec::new();
    let mut projected_groups: Vec<ProjectedLaneIntrinsicGroupOf<S>> = Vec::new();
    let collapsed = gutters
        .map(OrdinaryGridAxisGuttersOf::collapsed)
        .unwrap_or(&[]);

    for item in projected_items {
        match item.kind {
            LaneIntrinsicItemKind::Definite { span } => {
                if span.len().is_none() || span.end > input.tracks.len() + 1 {
                    return Err(LanePlacementError::DefiniteLaneSpanOutOfRange {
                        span,
                        tracks: input.tracks.len(),
                    });
                }
                definite_virtual_items.push(LaneProjectedSpanOf {
                    span,
                    contributions: item.contribution,
                    contribution_kind: item.contribution_kind,
                });
            }
            LaneIntrinsicItemKind::Indefinite { span } => {
                let span = span.get().min(input.tracks.len());
                let contributions = item.contribution;
                let active_starts = active_candidate_starts(input.tracks.len(), span, collapsed);
                let mut candidate_starts =
                    item.candidate_starts
                        .as_ref()
                        .map_or(active_starts.clone(), |candidates| {
                            candidates
                                .iter()
                                .copied()
                                .filter(|candidate| active_starts.contains(candidate))
                                .collect()
                        });
                candidate_starts.sort_unstable();
                candidate_starts.dedup();
                let key = LaneIntrinsicEquivalenceKeyOf {
                    span,
                    candidate_starts,
                    edges: item.edges,
                    contribution_kind: item.contribution_kind,
                };
                if let Some(group) = projected_groups.iter_mut().find(|group| group.key == key) {
                    group.max_min_content = group.max_min_content.max(contributions.min_content);
                    group.max_max_content = group.max_max_content.max(contributions.max_content);
                    group.max_minimum = group.max_minimum.max(contributions.minimum);
                    group.item_ids.push(item.id);
                } else {
                    projected_groups.push(ProjectedLaneIntrinsicGroupOf {
                        key,
                        max_min_content: contributions.min_content,
                        max_max_content: contributions.max_content,
                        max_minimum: contributions.minimum,
                        item_ids: vec![item.id],
                    });
                }
            }
        }
    }

    let indefinite_groups = projected_groups
        .iter()
        .map(|group| IndefiniteLaneContributionGroupOf {
            span: group.key.span,
            max_min_content: group.max_min_content,
            max_max_content: group.max_max_content,
            max_min_size: group.max_minimum,
            item_ids: group.item_ids.clone(),
        })
        .collect::<Vec<_>>();
    let mut converted_indefinite_items = Vec::new();
    let mut virtual_items = definite_virtual_items;
    for group in &projected_groups {
        for start_index in &group.key.candidate_starts {
            let span = LaneTrackSpan::new(*start_index + 1, *start_index + 1 + group.key.span);
            converted_indefinite_items.push(DefiniteLaneIntrinsicItemOf {
                id: "indefinite-group",
                span,
                contribution: LaneContributionFactsOf {
                    min_content: group.max_min_content,
                    max_content: group.max_max_content,
                    min_size: group.max_minimum,
                    automatic_minimum_applies: false,
                },
            });
            virtual_items.push(LaneProjectedSpanOf {
                span,
                contributions: LaneContributionsOf {
                    minimum: group.max_minimum,
                    min_content: group.max_min_content,
                    max_content: group.max_max_content,
                },
                contribution_kind: group.key.contribution_kind,
            });
        }
    }

    Ok(LaneProjectedContributionsOf {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        virtual_items,
    })
}

pub(super) fn lane_intrinsic_sizing_projected_with<Node, S, M>(
    input: &LaneIntrinsicSizingInputOf<S>,
    projected_items: &[ProjectedLaneIntrinsicItemOf<S>],
    gutters: Option<&OrdinaryGridAxisGuttersOf<S>>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, Result<LaneIntrinsicSizingReportOf<S>, LanePlacementError>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let LaneProjectedContributionsOf {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        virtual_items,
    } = match project_lane_intrinsic_contributions(input, projected_items, gutters) {
        Ok(projection) => projection,
        Err(error) => return Ok(Err(error)),
    };
    let mut minimum_sizes = vec![S::ZERO; input.tracks.len()];
    let mut min_content_sizes = vec![S::ZERO; input.tracks.len()];
    let mut max_content_sizes = vec![S::ZERO; input.tracks.len()];
    for LaneProjectedSpanOf {
        span,
        contributions,
        contribution_kind,
    } in virtual_items
    {
        if !span_overlaps_content_tracks(span, &input.content_sized_tracks) {
            continue;
        }
        let start = span.start - 1;
        let end = span.end - 1;
        for (sizes, kind, contribution) in [
            (&mut minimum_sizes, contribution_kind, contributions.minimum),
            (
                &mut min_content_sizes,
                IntrinsicSpanContribution::MinContent {
                    prioritize_min_tracks: matches!(
                        contribution_kind,
                        IntrinsicSpanContribution::MinContent {
                            prioritize_min_tracks: true
                        }
                    ),
                },
                contributions.min_content,
            ),
            (
                &mut max_content_sizes,
                IntrinsicSpanContribution::MaxContent,
                contributions.max_content,
            ),
        ] {
            apply_ordinary_intrinsic_contribution(
                sizes,
                OrdinaryIntrinsicContributionInput {
                    tracks: &input.tracks,
                    start,
                    end,
                    kind,
                    percent_basis: input.available,
                    contribution,
                    gap: input.gap,
                    gutters,
                },
            );
        }
    }

    let initialized_track_sizes = input
        .tracks
        .iter()
        .map(|track| initialized_track_base(track.clone(), input.available, site))
        .collect::<LayoutResultOf<Node, Vec<_>, S, M>>()?;
    let final_track_sizes = input
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let contribution = match track.min {
                MinTrackSizingOf::MinContent => min_content_sizes[index],
                MinTrackSizingOf::MaxContent => max_content_sizes[index],
                MinTrackSizingOf::Auto => minimum_sizes[index],
                MinTrackSizingOf::Calculation(_) => S::ZERO,
            };
            initialized_track_sizes[index].max(contribution)
        })
        .collect();

    Ok(Ok(LaneIntrinsicSizingReportOf {
        definite_items,
        indefinite_groups,
        converted_indefinite_items,
        final_track_sizes,
    }))
}

fn candidate_starts(track_count: usize, span: usize) -> impl Iterator<Item = usize> {
    let span = span.max(1).min(track_count);
    0..=track_count - span
}

fn active_candidate_starts(track_count: usize, span: usize, collapsed: &[bool]) -> Vec<usize> {
    candidate_starts(track_count, span)
        .filter(|start| lane_candidate_is_active(collapsed, *start, span))
        .collect()
}

pub(super) fn lane_candidate_is_active(collapsed: &[bool], start: usize, span: usize) -> bool {
    collapsed
        .get(start..start + span)
        .is_none_or(|candidate| candidate.iter().all(|collapsed| !collapsed))
}

pub(super) fn lane_intrinsic_wrapper_areas<'a, S: LayoutScalar>(
    placements: &'a [ResolvedGridItemPlacement],
    lines: GridLines,
    axis: GridAxisKind,
    geometry: &'a UsedGridAxisGeometryOf<S>,
    all_automatic_positions: bool,
) -> impl Iterator<Item = (usize, GridArea<S>)> + 'a {
    let track_count = geometry.sizes().len();
    let lines = grid_axis_lines(lines, axis);
    placements
        .iter()
        .enumerate()
        .filter(|(_, item)| item.in_flow)
        .flat_map(move |(index, item)| {
            let placement = match axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            let (line, span) = lane_grid_axis_facts(placement, track_count, lines);
            let definite = line.map(|line| line - 1);
            let span = span.min(track_count);
            let first = definite.unwrap_or(0);
            let last = definite.unwrap_or(track_count - span);
            (first..=last)
                .filter(move |start| {
                    definite.is_some()
                        || lane_candidate_is_active(geometry.collapsed(), *start, span)
                })
                .take(if all_automatic_positions {
                    usize::MAX
                } else {
                    1
                })
                .map(move |start| {
                    let area = grid_lanes_item_area(
                        axis,
                        start,
                        start + span,
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    );
                    (index, area)
                })
        })
}

fn initialized_track_base<Node, S, M>(
    track: TrackSizingOf<S>,
    available: Option<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, S, S, M>
where
    S: LayoutScalar,
{
    match track.min {
        MinTrackSizingOf::Calculation(calculation) => {
            let resolution = super::tracks::resolve_track_calculation(&calculation, available);
            resolution_or_zero(resolution, site)
        }
        MinTrackSizingOf::Auto | MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => {
            Ok(S::ZERO)
        }
    }
}

fn resolution_or_zero<Node, S, M>(
    resolution: LengthResolutionOf<S>,
    site: LayoutErrorSiteOf<Node>,
) -> LayoutResultOf<Node, S, S, M>
where
    S: LayoutScalar,
{
    match resolution.status() {
        LengthResolutionStatus::Resolved => Ok(resolution
            .value
            .expect("resolved length resolution must carry a value")),
        LengthResolutionStatus::InvalidNumeric { .. } => Err(
            crate::error::value_resolution_error_at_site(site, resolution.status()),
        ),
        LengthResolutionStatus::MissingBasis | LengthResolutionStatus::NonNumeric => Ok(S::ZERO),
    }
}

fn span_overlaps_content_tracks(span: LaneTrackSpan, content_sized_tracks: &[usize]) -> bool {
    let start = span.start - 1;
    let end = span.end - 1;
    content_sized_tracks
        .iter()
        .any(|track_index| (start..end).contains(track_index))
}

#[expect(
    clippy::too_many_arguments,
    reason = "lane placement resolution keeps explicit grid layout phase inputs separate"
)]
#[expect(
    clippy::type_complexity,
    reason = "lane placement preserves the session's node, scalar, and provider error types"
)]
pub(super) fn resolve_grid_lanes_placement_with_resolved_tracks<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    style: &GridContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    context: GridContainerContext<Tree::Scalar>,
    columns: &[Tree::Scalar],
    rows: &[Tree::Scalar],
    placements: &GridPlacementContext<'input, <Tree as Traverse>::Node, Tree::Scalar>,
    grid_axis_gap: Tree::Scalar,
    container_content_box_size: LogicalSizeOf<Option<Tree::Scalar>>,
    stacking_operation: SizingOperation,
    minimum_functions: GridMinimumParentFunctions<'_, Tree::Scalar>,
    subgrid_report: &GridSubgridReport<Tree::Node>,
    sizing_stage: GridSizingStage,
) -> LayoutResultOf<
    <Tree as Traverse>::Node,
    Result<PackedLanePlacement<Tree::Scalar>, LanePlacementError>,
    Tree::Scalar,
    M,
>
where
    Tree: Compute<'input, M>,
{
    let grid_axis = grid_axis_for_grid_lanes(style);
    let lane_axis = lane_axis_for_grid_lanes(style);
    let grid_axis_tracks = match grid_axis {
        GridAxisKind::Column => columns,
        GridAxisKind::Row => rows,
    };
    let grid_axis_gutters = match grid_axis {
        GridAxisKind::Column => &context.column_gutters,
        GridAxisKind::Row => &context.row_gutters,
    };
    if grid_axis_tracks.is_empty() {
        return Ok(Err(LanePlacementError::EmptyTrackList));
    }

    let tolerance = match resolve_tolerance(
        style.grid_flow_tolerance,
        match grid_axis {
            GridAxisKind::Column => context.percent_basis.inline.unwrap_or(Tree::Scalar::ZERO),
            GridAxisKind::Row => context.percent_basis.block.unwrap_or(Tree::Scalar::ZERO),
        },
    ) {
        Ok(tolerance) => tolerance,
        Err(error) => return Ok(Err(error)),
    };
    let lane_gap = match lane_axis {
        GridAxisKind::Column => context.gap.inline,
        GridAxisKind::Row => context.gap.block,
    };
    let mut running = vec![Tree::Scalar::ZERO; grid_axis_tracks.len()];
    let collapsed = grid_axis_gutters.collapsed();
    let mut skipped = style
        .grid_lanes_flow
        .is_dense()
        .then(|| vec![Vec::new(); grid_axis_tracks.len()]);
    let mut item_offsets = Vec::new();
    let mut cursor = 0usize;
    let mut inherited_geometry = None;

    let children = tree.children(node).collect::<Vec<_>>();
    let _ = placements.checked_child_placements(&children);
    for source_index in &placements.order_modified_indexes {
        let source_slot = source_index.get();
        let child = children[source_slot];
        let placement = placements.items[source_slot];
        let child_style = placements.item_input(source_slot);
        if !is_in_flow_grid_child(child_style) {
            continue;
        }
        let placement = match grid_axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let (definite_grid_axis_start, grid_axis_span) = lane_grid_axis_facts(
            placement,
            grid_axis_tracks.len(),
            grid_axis_lines(context.lines, grid_axis),
        );
        let (start, span) = match definite_grid_axis_start {
            Some(start_line) => {
                if start_line == 0 {
                    return Ok(Err(LanePlacementError::InvalidGridAxisStart {
                        start: start_line,
                    }));
                }
                if grid_axis_span == 0 {
                    return Ok(Err(LanePlacementError::InvalidGridAxisSpan {
                        span: grid_axis_span,
                    }));
                }
                let start = start_line - 1;
                if start + grid_axis_span > grid_axis_tracks.len() {
                    return Ok(Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: start_line,
                        span: grid_axis_span,
                        tracks: grid_axis_tracks.len(),
                    }));
                }
                (start, grid_axis_span)
            }
            None => {
                let span = grid_axis_span.clamp(1, grid_axis_tracks.len());
                let start = if matches!(style.grid_flow_tolerance, GridFlowToleranceOf::Infinite) {
                    infinite_candidate_start(
                        cursor,
                        span,
                        collapsed,
                        style.grid_lanes_flow.reversed(),
                    )
                } else {
                    finite_candidate_start(
                        &running,
                        cursor,
                        span,
                        tolerance,
                        collapsed,
                        style.grid_lanes_flow.reversed(),
                    )
                };
                let Some(start) = start else {
                    return Ok(Err(LanePlacementError::GridAxisSpanOutOfRange {
                        start: 1,
                        span,
                        tracks: grid_axis_tracks.len(),
                    }));
                };
                (start, span)
            }
        };
        let grid_area_size = |start| {
            track_span_sum_with_gutters(
                grid_axis_tracks,
                start,
                start + span,
                grid_axis_gap,
                Some(grid_axis_gutters),
            )
        };
        let grid_axis_size = grid_area_size(start);
        let containing_block = GridLanesItemContainingBlockOf::new(
            constants.flow_axes,
            grid_axis,
            grid_axis_size,
            container_content_box_size,
        );
        let item_report = subgrid_report.items[source_slot];
        let has_subgrid = item_report.needs_context();
        let inherits_grid_axis =
            inherited_subgrid_axis_for_parent_axis(item_report, grid_axis).is_some();
        if has_subgrid && inherited_geometry.is_none() {
            inherited_geometry = Some((
                UsedGridAxisGeometryOf::from_sizing_gutters(
                    columns.to_vec(),
                    &context.column_gutters,
                ),
                UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), &context.row_gutters),
            ));
        }
        let lane_axis_margin_box = measure_lane_axis_margin_box_with_grid_axis(
            tree,
            child,
            LaneAxisMarginBoxMeasureInput {
                child_style,
                container_style: style,
                constants,
                lane_axis,
                containing_block,
                stacking_operation,
                inherits_grid_axis,
            },
            |tree, input| {
                // Host-measured leaves retain the caller's measurement boundary.
                if !has_subgrid || tree.has_leaf_measurement(child) {
                    return tree.compute_child(child, input);
                }
                let (padding, border) = resolve_containing_padding_border(
                    input.containing_flow_axes(),
                    input.parent(),
                    child_style.padding,
                    child_style.border,
                    resolve_length_or_zero,
                    |edges| edges.transpose_with_node(tree, child),
                )?;
                let margin = input
                    .containing_flow_axes()
                    .zip_physical_edges_with_inline_extent(
                        child_style.margin,
                        input.parent(),
                        resolve_auto_optional,
                    )
                    .transpose_with_node(tree, child)?;
                // Only the inherited track range is allocated. The stacking
                // estimate stays separate from the prepared percentage basis.
                let area_size = input.parent().unwrap_or(Size::ZERO);
                let content_box_size = (area_size
                    - margin
                        .map(|value| value.unwrap_or(Tree::Scalar::ZERO))
                        .sum_axes()
                    - padding.sum_axes()
                    - border.sum_axes())
                .max(Size::ZERO);
                let source = SubgridChildParentContextInput {
                    parent_baseline_references: style.baseline_references,
                    minimum_functions,
                    item: item_report,
                    child_style,
                    child_container: placements.nested_container_input(source_slot).copied(),
                    area: grid_lanes_item_area(
                        grid_axis,
                        start,
                        start + span,
                        constants.flow_axes.logical_size(area_size),
                    ),
                    content_box_size,
                    columns,
                    rows,
                    gap: context.gap,
                    parent_named_columns: &context.named_columns,
                    parent_named_rows: &context.named_rows,
                    parent_area_facts: context.area_facts.as_ref(),
                    margin,
                    border,
                    padding,
                    scrollbar_gutter: Edges::ZERO,
                };
                let (column_geometry, row_geometry) = inherited_geometry
                    .as_ref()
                    .expect("inherited stacking probes retain their parent geometry");
                match tree.observe_grid_overflow(
                    child,
                    GridOverflowProbe {
                        layout: GridOverflowLayout::Intrinsic(GridIntrinsicProbe {
                            input,
                            stage: sizing_stage,
                            kind: GridIntrinsicProbeKind::AllocatedPrefix,
                            intrinsic_minimum: Size::NONE,
                            source,
                            parent_column_geometry: Some(column_geometry),
                            parent_row_geometry: Some(row_geometry),
                            parent_flow: constants.flow_axes,
                        }),
                    },
                )? {
                    GridOverflowObservation::Intrinsic(observation) => Ok(observation.output),
                    GridOverflowObservation::Children(_) => {
                        unreachable!("a stacking intrinsic probe returns its prepared output")
                    }
                }
            },
        )?;
        let normal_offset = max_running_position(&running, start, span);
        let occupied_end =
            |offset| offset + lane_axis_margin_box.max(Tree::Scalar::ZERO) + lane_gap;
        let backfill = skipped
            .as_ref()
            .filter(|skipped| skipped.iter().any(|lane| !lane.is_empty()))
            .and_then(|skipped| {
                let track_size = |start| {
                    track_span_sum_with_gutters(
                        grid_axis_tracks,
                        start,
                        start + span,
                        Tree::Scalar::ZERO,
                        None,
                    )
                };
                let normal_track_size = track_size(start);
                let starts = if definite_grid_axis_start.is_some() {
                    start..start + 1
                } else {
                    0..grid_axis_tracks.len() - span + 1
                };
                let compatible_starts = lane_candidate_order(
                    starts,
                    style.grid_lanes_flow.reversed(),
                )
                .filter(|&candidate| {
                    (candidate == start || lanes_candidate_is_retained(collapsed, candidate, span))
                        && track_size(candidate) == normal_track_size
                        && grid_area_size(candidate) == grid_axis_size
                });
                find_lane_backfill(
                    skipped,
                    &running,
                    (start, normal_offset),
                    compatible_starts,
                    span,
                    occupied_end,
                    tolerance,
                )
            });
        if backfill.is_none() && definite_grid_axis_start.is_none() {
            cursor = lane_cursor_after(
                start,
                span,
                grid_axis_tracks.len(),
                style.grid_lanes_flow.reversed(),
            );
        }
        let (start, offset) = backfill.unwrap_or((start, normal_offset));
        let end = start + span;
        let new_position = occupied_end(offset);
        for (track, position) in running.iter_mut().enumerate().take(end).skip(start) {
            let old_position = *position;
            let occupies_tail = offset >= old_position;
            if let Some(skipped) = &mut skipped {
                if occupies_tail {
                    if old_position < offset {
                        skipped[track].push(old_position..offset);
                    }
                } else {
                    consume_lane_opening(&mut skipped[track], offset..new_position);
                }
            }
            if occupies_tail {
                *position = new_position;
            }
        }
        item_offsets.push(PackedLaneItem {
            source_index: source_slot,
            alignment_extent: Tree::Scalar::ZERO,
            grid_axis_start: start + 1,
            grid_axis_span: span,
            offset,
            lane_axis_margin_box,
        });
    }

    Ok(Ok(PackedLanePlacement::new(
        grid_axis,
        lane_axis,
        item_offsets,
        grid_axis_tracks.len(),
        lane_gap,
    )))
}

fn find_lane_backfill<S: LayoutScalar>(
    skipped: &[Vec<Range<S>>],
    running: &[S],
    normal: (usize, S),
    starts: impl Iterator<Item = usize>,
    span: usize,
    occupied_end: impl Fn(S) -> S,
    tolerance: S,
) -> Option<(usize, S)> {
    let mut openings = vec![0; span];
    // Starts follow primary traversal order and include the normal start.
    // Its position determines which equal-offset openings precede it.
    let mut precedes_normal = true;
    let candidates = starts
        .filter_map(|start| {
            // Advance even when the normal candidate has no opening.
            if start == normal.0 {
                precedes_normal = false;
            }
            let span = start..start + span;
            openings.fill(0);
            let offset = first_lane_opening(
                &skipped[span.clone()],
                &running[span],
                &mut openings,
                &occupied_end,
            )?;
            let earlier = offset < normal.1 || (offset == normal.1 && precedes_normal);
            earlier.then_some((start, offset))
        })
        .collect::<Vec<_>>();
    let highest = candidates
        .iter()
        .map(|&(_, offset)| offset)
        .reduce(S::min)?;
    candidates
        .into_iter()
        .find(|&(_, offset)| offset <= highest + tolerance.max(S::ZERO))
}

fn first_lane_opening<S: LayoutScalar>(
    skipped: &[Vec<Range<S>>],
    running: &[S],
    openings: &mut [usize],
    occupied_end: &impl Fn(S) -> S,
) -> Option<S> {
    loop {
        let mut start = S::ZERO;
        let mut earliest_end = None;
        for (track, &opening) in openings.iter().enumerate() {
            if let Some(opening) = skipped[track].get(opening) {
                start = start.max(opening.start);
                if earliest_end.is_none_or(|(_, end)| opening.end < end) {
                    earliest_end = Some((track, opening.end));
                }
            } else {
                start = start.max(running[track]);
            }
        }
        // At least one track must contribute a finite opening. If the current
        // intersection is too short, only the earliest end limits it.
        let (track, end) = earliest_end?;
        if start <= end && occupied_end(start) <= end {
            return Some(start);
        }
        openings[track] += 1;
    }
}

fn consume_lane_opening<S: LayoutScalar>(skipped: &mut Vec<Range<S>>, occupied: Range<S>) {
    if occupied.start == occupied.end {
        return;
    }
    let index = skipped
        .iter()
        .position(|opening| opening.start <= occupied.start && occupied.end <= opening.end)
        .expect("backfill fits in the selected opening");
    let opening = &skipped[index];
    let remaining = [opening.start..occupied.start, occupied.end..opening.end];
    skipped.splice(
        index..=index,
        remaining
            .into_iter()
            .filter(|range| range.start < range.end),
    );
}

pub(super) struct GridLanesLayoutInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(super) scroll_box: CanonicalScrollBoxOf<S>,
    pub(super) style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(super) container_content_box_size: LogicalSizeOf<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) track_basis: LogicalSizeOf<Option<S>>,
    pub(super) parent_context: &'a GridParentContext<S, Node>,
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) context: GridContainerContext<S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<'input, Node, S>,
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    pub(super) sizing_stage: GridSizingStage,
    pub(super) stacking_operation: SizingOperation,
    pub(super) execution_input: ComputeInputOf<S>,
}

#[derive(Clone, Copy)]
pub(super) struct LaneIntrinsicTrackSizeInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(super) grid: IntrinsicGrid<'input, 'a, Node, S>,
    pub(super) frame: IntrinsicStageFrame<'a, S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [TrackSizingOf<S>],
    pub(super) gap: S,
    pub(super) gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) lines: GridLines,
    pub(super) placements: &'a GridPlacementContext<'input, Node, S>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct LanePurposeContributionOf<Node, S: LayoutScalar> {
    source: Node,
    span: GridTrackSpan,
    value: S,
}

#[derive(PartialEq)]
pub(super) struct LanePurposeContributions<Node, S: LayoutScalar> {
    pub(super) rows: Vec<LanePurposeContributionOf<Node, S>>,
    base_floors: Vec<S>,
}

impl<Node, S: LayoutScalar> LanePurposeContributions<Node, S> {
    pub(super) fn empty() -> Self {
        Self {
            rows: Vec::new(),
            base_floors: Vec::new(),
        }
    }
}

pub(super) type LanePurposeResultOf<Node, S, M> =
    LayoutResultOf<Node, Result<LanePurposeContributions<Node, S>, LanePlacementError>, S, M>;

pub(super) struct LaneIntrinsicContributions<Node, S: LayoutScalar> {
    pub(super) rows: Vec<GridIntrinsicContributionRow<S>>,
    pub(super) min_content: LanePurposeContributions<Node, S>,
    pub(super) base_floors: Vec<S>,
}

type LaneIntrinsicResult<Node, S, M> =
    LayoutResultOf<Node, Result<LaneIntrinsicContributions<Node, S>, LanePlacementError>, S, M>;

pub(super) fn lane_intrinsic_contributions<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: LaneIntrinsicTrackSizeInput<'input, '_, Tree::Node, Tree::Scalar>,
    observed_min_content: Option<LanePurposeContributions<Tree::Node, Tree::Scalar>>,
    retain_min_content: bool,
) -> LaneIntrinsicResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut collect = |purpose| collect_lane_intrinsic_purpose(tree, node, input, purpose);
    let mut minimum = match collect(GridContributionPurpose::Minimum)? {
        Ok(rows) => rows,
        Err(error) => return Ok(Err(error)),
    };
    let min_content = match observed_min_content {
        Some(rows) => rows,
        None => match collect(GridContributionPurpose::MinContent)? {
            Ok(rows) => rows,
            Err(error) => return Ok(Err(error)),
        },
    };
    let max_content = match collect(GridContributionPurpose::MaxContent)? {
        Ok(rows) => rows,
        Err(error) => return Ok(Err(error)),
    };
    let mut base_floors = core::mem::take(&mut minimum.base_floors);
    for floors in [&min_content.base_floors, &max_content.base_floors] {
        if base_floors.len() < floors.len() {
            base_floors
                .try_reserve_exact(floors.len() - base_floors.len())
                .map_err(|_| {
                    grid_track_sizing_error(node, SpanningContributionError::Allocation)
                })?;
            base_floors.resize(floors.len(), Tree::Scalar::ZERO);
        }
        for (floor, &value) in base_floors.iter_mut().zip(floors) {
            *floor = floor.max(value);
        }
    }
    let (min_rows, min_content) = if retain_min_content {
        // Reduction combines equal spans. Preserve the individual facts
        // only when this collection supplies a later comparison.
        let mut min_rows = Vec::new();
        min_rows
            .try_reserve_exact(min_content.rows.len())
            .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
        min_rows.extend_from_slice(&min_content.rows);
        (min_rows, min_content)
    } else {
        (min_content.rows, LanePurposeContributions::empty())
    };
    let rows = complete_lane_contribution_rows(minimum.rows, min_rows, max_content.rows)
        .map_err(|error| grid_track_sizing_error(node, error))?;
    Ok(Ok(LaneIntrinsicContributions {
        rows,
        min_content,
        base_floors,
    }))
}

fn normalize_lane_purpose_rows<Node: Copy, S: LayoutScalar>(
    mut rows: Vec<LanePurposeContributionOf<Node, S>>,
) -> Result<Vec<LanePurposeContributionOf<Node, S>>, SpanningContributionError<S>> {
    rows.sort_unstable_by_key(|row| (row.span.start, row.span.end));
    let mut written = 0;
    for read in 0..rows.len() {
        let row = rows[read];
        if row.span.start == 0 || row.span.end <= row.span.start {
            return Err(SpanningContributionError::InvalidIncidence);
        }
        if !row.value.is_finite() {
            return Err(SpanningContributionError::InvalidNumeric { value: row.value });
        }
        if written > 0 && rows[written - 1].span == row.span {
            rows[written - 1].value = rows[written - 1].value.max(row.value);
        } else {
            rows[written] = row;
            written += 1;
        }
    }
    rows.truncate(written);
    Ok(rows)
}

// Equal spans describe numerical inequalities, not a measured node identity.
// Max preserves every same-span constraint. Presence comes from actual rows,
// including real zero values, and each purpose must own the same incidence.
fn complete_lane_contribution_rows<Node: Copy, S: LayoutScalar>(
    minimum: Vec<LanePurposeContributionOf<Node, S>>,
    min_content: Vec<LanePurposeContributionOf<Node, S>>,
    max_content: Vec<LanePurposeContributionOf<Node, S>>,
) -> Result<Vec<GridIntrinsicContributionRow<S>>, SpanningContributionError<S>> {
    let minimum = normalize_lane_purpose_rows(minimum)?;
    let min_content = normalize_lane_purpose_rows(min_content)?;
    let max_content = normalize_lane_purpose_rows(max_content)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(minimum.len())
        .map_err(|_| SpanningContributionError::Allocation)?;
    let mut minimum_index = 0;
    let mut min_content_index = 0;
    let mut max_content_index = 0;
    loop {
        let span = [
            minimum.get(minimum_index),
            min_content.get(min_content_index),
            max_content.get(max_content_index),
        ]
        .into_iter()
        .flatten()
        .map(|row| row.span)
        .min_by_key(|span| (span.start, span.end));
        let Some(span) = span else {
            break;
        };
        let take = |rows: &[LanePurposeContributionOf<Node, S>],
                    index: &mut usize,
                    purpose: GridContributionPurpose|
         -> Result<S, SpanningContributionError<S>> {
            let row = rows
                .get(*index)
                .filter(|row| row.span == span)
                .ok_or(SpanningContributionError::MissingContribution { purpose })?;
            *index += 1;
            Ok(row.value)
        };
        result.push(GridIntrinsicContributionRow {
            span,
            contributions: GridTrackContributions {
                minimum: take(
                    &minimum,
                    &mut minimum_index,
                    GridContributionPurpose::Minimum,
                )?,
                min_content: take(
                    &min_content,
                    &mut min_content_index,
                    GridContributionPurpose::MinContent,
                )?,
                max_content: take(
                    &max_content,
                    &mut max_content_index,
                    GridContributionPurpose::MaxContent,
                )?,
            },
        });
    }
    Ok(result)
}

pub(super) fn collect_lane_intrinsic_purpose<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: LaneIntrinsicTrackSizeInput<'input, '_, Tree::Node, Tree::Scalar>,
    purpose: GridContributionPurpose,
) -> LanePurposeResultOf<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let input = LaneIntrinsicTrackSizeInput {
        grid: intrinsic_role_grid(input.grid, input.frame, purpose)
            .map_err(|error| grid_track_sizing_error(node, error))?,
        ..input
    };
    let LaneIntrinsicTrackSizeInput {
        grid,
        frame: _,
        container_style,
        constants,
        axis,
        tracks,
        gap,
        gutters,
        lines,
        placements,
        subgrid_report,
    } = input;
    let basis = match axis {
        GridAxisKind::Column => grid.track_basis.inline,
        GridAxisKind::Row => grid.track_basis.block,
    };
    let mut contributions = LanePurposeContributions::empty();
    if tracks.is_empty()
        || !tracks
            .iter()
            .any(|track| grid.query.track_needs_contribution(track, basis))
    {
        return Ok(Ok(contributions));
    }

    let children = tree.children(node).collect::<Vec<_>>();
    let _ = placements.checked_child_placements(&children);
    for source_index in &placements.order_modified_indexes {
        let source_slot = source_index.get();
        let child = children[source_slot];
        let placement = placements.items[source_slot];
        let child_style = placements.item_input(source_slot);
        if !is_in_flow_grid_child(child_style) {
            continue;
        }

        let placement = match axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let (definite_grid_axis_start, grid_axis_span) =
            lane_grid_axis_facts(placement, tracks.len(), grid_axis_lines(lines, axis));
        let item_report = subgrid_report
            .items
            .get(source_slot)
            .copied()
            .expect("grid-lanes subgrid report must preserve one item per child");
        if let Some(child_axis) = inherited_subgrid_axis_for_parent_axis(item_report, axis) {
            let axis_report = match child_axis {
                GridAxisKind::Column => item_report.column,
                GridAxisKind::Row => item_report.row,
            };
            let reversed = axis_report.mapping.reversed;
            let wrapper_span = grid_axis_span.max(1).min(tracks.len());
            let collapsed = gutters
                .map(OrdinaryGridAxisGuttersOf::collapsed)
                .unwrap_or(&[]);
            let wrapper_starts = definite_grid_axis_start.map_or_else(
                || active_candidate_starts(tracks.len(), wrapper_span, collapsed),
                |start| vec![start - 1],
            );
            let wrapper_edges = lane_child_edge_facts(
                tree,
                child,
                child_style,
                constants.flow_axes,
                constants.node_inner_size,
                child_axis,
            )?;
            let functions = grid
                .minimum_functions
                .for_axis(axis)
                .ok_or_else(|| minimum_source_error(child))?;
            let geometry = gutters.map_or_else(
                || {
                    UsedGridAxisGeometryOf::new(
                        vec![Tree::Scalar::ZERO; tracks.len()],
                        vec![false; tracks.len()],
                        gap,
                    )
                },
                |gutters| {
                    UsedGridAxisGeometryOf::from_sizing_gutters(
                        vec![Tree::Scalar::ZERO; tracks.len()],
                        gutters,
                    )
                },
            );
            let named_lines = match axis {
                GridAxisKind::Column => grid.named_columns,
                GridAxisKind::Row => grid.named_rows,
            };
            let prepared = prepare_grid_lanes_intrinsic_child_input(
                child_style,
                constants.flow_axes,
                axis,
                grid.query,
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    constants.node_inner_size,
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    SizingRequestOf::new(grid.query.available, constants.child_orthogonal_fallback),
                ),
            )
            .map_err(|error| sizing_resolution_error(child, error))?;
            if contributions.base_floors.is_empty() {
                contributions
                    .base_floors
                    .try_reserve_exact(tracks.len())
                    .map_err(|_| {
                        grid_track_sizing_error(node, SpanningContributionError::Allocation)
                    })?;
                contributions
                    .base_floors
                    .resize(tracks.len(), Tree::Scalar::ZERO);
            }
            let mut memo = NestedLaneTraversalMemoOf::new();
            for start in wrapper_starts {
                let inherited = intrinsic_subgrid_axis_parent_context_for_span(
                    axis_report,
                    GridTrackSpan::new(start + 1, start + wrapper_span + 1),
                    gap,
                    SubgridInheritedAxisSource {
                        baseline_reference: grid_baseline_reference(
                            grid.style.baseline_references,
                            axis,
                        ),
                        geometry: Some(&geometry),
                        allocated: match axis {
                            GridAxisKind::Column => grid.query.columns,
                            GridAxisKind::Row => grid.query.rows,
                        },
                        minimum_functions: Some(functions),
                    },
                    named_lines,
                    grid.area_facts,
                )
                .map_err(|error| subgrid_child_context_error(child, error))?;
                let mut parent_context = GridParentContext::none();
                match child_axis {
                    GridAxisKind::Column => parent_context.columns = inherited,
                    GridAxisKind::Row => parent_context.rows = inherited,
                }
                collect_nested_lane_intrinsic_items(
                    tree,
                    child,
                    GridMinimumWrapperInput {
                        axis: child_axis,
                        container: placements
                            .child_input(source_slot)
                            .nested_container()
                            .expect("an inherited Lane wrapper retains its actual container input"),
                        parent_context: &parent_context,
                        prepared,
                        containing_flow_axes: constants.flow_axes,
                        content_box_estimate: None,
                        traversal: GridMinimumWrapperTraversal::LaneCandidates,
                        query: grid.query,
                    },
                    NestedLaneIntrinsicProjectionOf {
                        root_track_count: tracks.len(),
                        axis: child_axis,
                        wrapper_span,
                        wrapper_starts: vec![start],
                        hypothetical_descendants: definite_grid_axis_start.is_none(),
                        envelope_empty_leaf: false,
                        reversed,
                        parent_gap: gap,
                        interior_half_gap: Tree::Scalar::ZERO,
                        accumulated_edges: LaneIntrinsicEdgeFactsOf::default(),
                        wrapper_edges,
                    },
                    &mut memo,
                    purpose,
                    &mut contributions,
                )?;
            }
            memo.emit_empty_leaf_floors(&mut contributions.base_floors);
            continue;
        }
        let span = if definite_grid_axis_start.is_some() {
            grid_axis_span
        } else {
            grid_axis_span.max(1).min(tracks.len())
        };
        let collapsed = gutters
            .map(OrdinaryGridAxisGuttersOf::collapsed)
            .unwrap_or(&[]);
        let starts = definite_grid_axis_start.map_or_else(
            || active_candidate_starts(tracks.len(), span, collapsed),
            |start| vec![start - 1],
        );
        let prepared = prepare_grid_lanes_intrinsic_child_input(
            child_style,
            constants.flow_axes,
            axis,
            grid.query,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                constants.node_inner_size,
                crate::ContainingLayoutContext::new(
                    constants.flow_axes,
                    crate::ParentFormattingContext::Grid,
                ),
                SizingRequestOf::new(grid.query.available, constants.child_orthogonal_fallback),
            ),
        )
        .map_err(|error| sizing_resolution_error(child, error))?;
        let functions = grid
            .minimum_functions
            .for_axis(axis)
            .ok_or_else(|| minimum_source_error(child))?;
        for start in starts {
            let end = start + span;
            if tracks.get(start..end).is_none() {
                return Ok(Err(LanePlacementError::DefiniteLaneSpanOutOfRange {
                    span: LaneTrackSpan::new(start + 1, end + 1),
                    tracks: tracks.len(),
                }));
            }
            let span_gutter = gutters.map_or_else(
                || gap * Tree::Scalar::from_usize(span.saturating_sub(1)),
                |gutters| gutters.span_gap_total(start, end),
            );
            let minimum_span = functions
                .mapped_span(start, end, false)
                .ok_or_else(|| minimum_source_error(child))?
                .span_facts(span_gutter)
                .map_err(|error| sizing_resolution_error(child, error))?;
            let value = lane_child_contribution(
                tree,
                child,
                GridMinimumContext {
                    item: child_style,
                    flow_axes: constants.flow_axes,
                    axis,
                    prepared,
                    margin_basis: prepared.parent(),
                    span: minimum_span,
                    // This is the real Lane stacking axis, not a Grid span.
                    opposite_fixed_max_span: None,
                    subgrid_requests: lane_child_subgrid_requests(
                        placements.child_input(source_slot).nested_container(),
                    ),
                    algorithm: sizing_algorithm_for_grid_display(container_style.display),
                },
                purpose,
                |tree, input| {
                    if let Some(container) = placements
                        .nested_container_input(source_slot)
                        .filter(|container| container.requires_context())
                    {
                        let child_flow =
                            crate::FlowAxes::new(child_style.writing_mode, child_style.direction);
                        Ok(compute_grid_with_context_source(
                            tree,
                            child,
                            input,
                            Size::NONE,
                            *container,
                            grid.query.stage.mapped(constants.flow_axes, child_flow),
                            |_| Ok(GridParentContext::none()),
                        )?
                        .output)
                    } else {
                        tree.compute_child(child, input)
                    }
                },
            )?;
            contributions.rows.try_reserve(1).map_err(|_| {
                grid_track_sizing_error(child, SpanningContributionError::Allocation)
            })?;
            contributions.rows.push(LanePurposeContributionOf {
                source: child,
                span: GridTrackSpan::new(start + 1, end + 1),
                value,
            });
        }
    }

    for (floor, track) in contributions.base_floors.iter_mut().zip(tracks) {
        if !track_minimum_is_intrinsic(track, basis) {
            *floor = Tree::Scalar::ZERO;
        }
    }
    Ok(Ok(contributions))
}

pub(in crate::grid) fn lane_child_subgrid_requests<S: LayoutScalar>(
    container: Option<&GridContainerInput<'_, S>>,
) -> Size<bool> {
    container.map_or(Size::new(false, false), |container| {
        let projection = container.projection();
        let requests = |tracks: &[TrackComponentOf<S>]| {
            tracks
                .iter()
                .any(|track| matches!(track, TrackComponentOf::Subgrid(_)))
        };
        Size::new(
            requests(projection.grid_template_columns),
            requests(projection.grid_template_rows),
        )
    })
}

#[derive(Clone, PartialEq)]
pub(super) struct NestedLaneIntrinsicProjectionOf<S: LayoutScalar = Scalar> {
    pub(super) root_track_count: usize,
    pub(super) axis: GridAxisKind,
    pub(super) wrapper_span: usize,
    pub(super) wrapper_starts: Vec<usize>,
    pub(super) hypothetical_descendants: bool,
    pub(super) envelope_empty_leaf: bool,
    pub(super) reversed: bool,
    pub(super) parent_gap: S,
    pub(super) interior_half_gap: S,
    pub(super) accumulated_edges: LaneIntrinsicEdgeFactsOf<S>,
    pub(super) wrapper_edges: LaneIntrinsicEdgeFactsOf<S>,
}

// A source slot has the same authored container input for the whole
// contribution pass. Retain a separate memo for each descendant slot.
struct NestedLaneTraversalMemoOf<S: LayoutScalar, Node> {
    states: Vec<NestedLaneTraversalStateOf<S, Node>>,
    children: Vec<(crate::SourceIndex, Self)>,
    empty_leaf: Option<LaneEmptyLeafFloorOf<S>>,
}

struct LaneEmptyLeafFloorOf<S: LayoutScalar> {
    starts: Vec<usize>,
    value: S,
}

struct NestedLaneTraversalStateOf<S: LayoutScalar, Node> {
    parent_context: GridParentContext<S, Node>,
    prepared: ComputeInputOf<S>,
    containing_flow_axes: crate::geometry::FlowAxes,
    content_box_estimate: Option<Size<S>>,
    query: IntrinsicGridQuery<'static, S>,
    columns: Option<Vec<S>>,
    rows: Option<Vec<S>>,
    projection: NestedLaneIntrinsicProjectionOf<S>,
}

impl<S: LayoutScalar, Node: Copy + Eq> NestedLaneTraversalMemoOf<S, Node> {
    fn new() -> Self {
        Self {
            states: Vec::new(),
            children: Vec::new(),
            empty_leaf: None,
        }
    }

    fn record_empty_leaf(&mut self, starts: &[usize], value: S) -> Result<(), TryReserveError> {
        let leaf = self.empty_leaf.get_or_insert_with(|| LaneEmptyLeafFloorOf {
            starts: Vec::new(),
            value: S::ZERO,
        });
        leaf.starts.try_reserve(starts.len())?;
        leaf.starts.extend_from_slice(starts);
        leaf.value = leaf.value.max(value);
        Ok(())
    }

    fn emit_empty_leaf_floors(self, floors: &mut [S]) {
        if let Some(leaf) = self.empty_leaf
            && leaf.value > S::ZERO
        {
            // Each empty span-one descendant applies its largest qualified
            // edge floor to every track it can reach in the automatic chain.
            for start in leaf.starts {
                floors[start] = floors[start].max(leaf.value);
            }
        }
        for (_, child) in self.children {
            child.emit_empty_leaf_floors(floors);
        }
    }

    fn child(&mut self, source_index: crate::SourceIndex) -> Result<&mut Self, TryReserveError> {
        let index = match self
            .children
            .binary_search_by_key(&source_index, |child| child.0)
        {
            Ok(index) => index,
            Err(index) => {
                self.children.try_reserve(1)?;
                self.children.insert(index, (source_index, Self::new()));
                index
            }
        };
        Ok(&mut self.children[index].1)
    }

    fn record(
        &mut self,
        request: &GridMinimumWrapperInput<'_, '_, Node, S>,
        projection: &NestedLaneIntrinsicProjectionOf<S>,
    ) -> Result<bool, TryReserveError> {
        if self.states.iter().any(|state| {
            let query = IntrinsicGridQuery {
                columns: state.columns.as_deref(),
                rows: state.rows.as_deref(),
                ..state.query
            };
            state.projection == *projection
                && state.parent_context == *request.parent_context
                && state.prepared == request.prepared
                && state.containing_flow_axes == request.containing_flow_axes
                && state.content_box_estimate == request.content_box_estimate
                && query == request.query
        }) {
            return Ok(false);
        }
        self.states.try_reserve(1)?;
        let query = request.query;
        self.states.push(NestedLaneTraversalStateOf {
            parent_context: request.parent_context.clone(),
            prepared: request.prepared,
            containing_flow_axes: request.containing_flow_axes,
            content_box_estimate: request.content_box_estimate,
            query: IntrinsicGridQuery {
                purpose: query.purpose,
                axis: query.axis,
                stage: query.stage,
                available: query.available,
                columns: None,
                rows: None,
                observe_auto: query.observe_auto,
                include_flexible_tracks: query.include_flexible_tracks,
            },
            columns: query.columns.map(<[S]>::to_vec),
            rows: query.rows.map(<[S]>::to_vec),
            projection: projection.clone(),
        });
        Ok(true)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NestedLaneCandidateGroupOf<S: LayoutScalar = Scalar> {
    pub(super) starts: Vec<usize>,
    pub(super) edges: LaneIntrinsicEdgeFactsOf<S>,
}

fn add_lane_intrinsic_edges<S: LayoutScalar>(
    left: LaneIntrinsicEdgeFactsOf<S>,
    right: LaneIntrinsicEdgeFactsOf<S>,
) -> LaneIntrinsicEdgeFactsOf<S> {
    LaneIntrinsicEdgeFactsOf {
        start_mbp: left.start_mbp + right.start_mbp,
        end_mbp: left.end_mbp + right.end_mbp,
        start_half_gap: left.start_half_gap + right.start_half_gap,
        end_half_gap: left.end_half_gap + right.end_half_gap,
    }
}

fn translated_nested_lane_start(
    wrapper_start: usize,
    wrapper_span: usize,
    local_start: usize,
    child_span: usize,
    reversed: bool,
) -> usize {
    if reversed {
        wrapper_start + wrapper_span - local_start - child_span
    } else {
        wrapper_start + local_start
    }
}

pub(super) fn nested_lane_candidate_groups<S: LayoutScalar>(
    projection: &NestedLaneIntrinsicProjectionOf<S>,
    child_span: usize,
    local_starts: impl IntoIterator<Item = usize>,
    explicit_local_start: Option<usize>,
) -> Vec<NestedLaneCandidateGroupOf<S>> {
    let mut groups: Vec<NestedLaneCandidateGroupOf<S>> = Vec::new();
    for local_start in local_starts {
        // Explicit edge contact qualifies the contribution at each candidate.
        let edge_start = explicit_local_start.unwrap_or(local_start);
        let touches_local_start = edge_start == 0;
        let touches_local_end = edge_start + child_span == projection.wrapper_span;
        let (touches_root_start, touches_root_end) = if projection.reversed {
            (touches_local_end, touches_local_start)
        } else {
            (touches_local_start, touches_local_end)
        };
        let mut inherited_edges = projection.accumulated_edges;
        if !touches_root_start {
            inherited_edges.start_mbp = S::ZERO;
            inherited_edges.start_half_gap = projection.interior_half_gap;
        }
        if !touches_root_end {
            inherited_edges.end_mbp = S::ZERO;
            inherited_edges.end_half_gap = projection.interior_half_gap;
        }
        for wrapper_start in &projection.wrapper_starts {
            let mut boundary_edges = LaneIntrinsicEdgeFactsOf::default();
            if touches_local_start {
                if projection.reversed {
                    boundary_edges.end_mbp = projection.wrapper_edges.start_mbp;
                } else {
                    boundary_edges.start_mbp = projection.wrapper_edges.start_mbp;
                }
            }
            if touches_local_end {
                if projection.reversed {
                    boundary_edges.start_mbp = projection.wrapper_edges.end_mbp;
                } else {
                    boundary_edges.end_mbp = projection.wrapper_edges.end_mbp;
                }
            }
            let edges = add_lane_intrinsic_edges(inherited_edges, boundary_edges);
            let start = translated_nested_lane_start(
                *wrapper_start,
                projection.wrapper_span,
                local_start,
                child_span,
                projection.reversed,
            );
            if start + child_span > projection.root_track_count {
                continue;
            }
            if let Some(group) = groups.iter_mut().find(|group| group.edges == edges) {
                group.starts.push(start);
            } else {
                groups.push(NestedLaneCandidateGroupOf {
                    starts: vec![start],
                    edges,
                });
            }
        }
    }
    for group in &mut groups {
        group.starts.sort_unstable();
        group.starts.dedup();
    }
    groups
}

fn lane_wrapper_edges_with_gutter<S: LayoutScalar>(
    projection: &NestedLaneIntrinsicProjectionOf<S>,
    constants: &Constants<S>,
) -> LaneIntrinsicEdgeFactsOf<S> {
    let gutter = constants
        .flow_axes
        .logical_edges(constants.scrollbar_gutter);
    let (start, end) = match projection.axis {
        GridAxisKind::Column => (gutter.inline_start, gutter.inline_end),
        GridAxisKind::Row => (gutter.block_start, gutter.block_end),
    };
    let mut edges = projection.wrapper_edges;
    edges.start_mbp = edges.start_mbp + start;
    edges.end_mbp = edges.end_mbp + end;
    edges
}

fn collect_empty_lane_subgrid_edges<'input, Tree, M>(
    tree: &Tree,
    wrapper: Tree::Node,
    request: &GridMinimumWrapperInput<'input, '_, Tree::Node, Tree::Scalar>,
    projection: &NestedLaneIntrinsicProjectionOf<Tree::Scalar>,
    memo: &mut NestedLaneTraversalMemoOf<Tree::Scalar, Tree::Node>,
    floors: &mut [Tree::Scalar],
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut container = *request.container;
    let inherited = LogicalSizeOf::new(
        request.parent_context.columns.is_some(),
        request.parent_context.rows.is_some(),
    );
    let intrinsic_minimum = standalone_intrinsic_minimum_phases(&container.projection(), inherited);
    container.suppress_intrinsic_minimum(intrinsic_minimum.map(|minimum| minimum.is_some()));
    let style = container.projection();
    let constants = lane_subgrid_wrapper_constants(
        tree,
        wrapper,
        &style,
        request.prepared.with_run_mode(RunMode::ComputeSize),
        inherited,
    )?;
    let edges = lane_wrapper_edges_with_gutter(projection, &constants);
    let (start_mbp, end_mbp) = (edges.start_mbp, edges.end_mbp);
    let (start_mbp, end_mbp) = if projection.reversed {
        (end_mbp, start_mbp)
    } else {
        (start_mbp, end_mbp)
    };
    let gap = match child::child_subgrid_gap(&style, projection.axis, constants.node_inner_size)
        .map_err(|status| crate::error::value_resolution_error(wrapper, status))?
    {
        ResolvedSubgridGap::Normal => projection.parent_gap,
        ResolvedSubgridGap::Length(value) => value,
    };
    let half_gap =
        projection.interior_half_gap + (gap - projection.parent_gap) / Tree::Scalar::from_f64(2.0);
    let zero = Tree::Scalar::ZERO;
    for offset in 0..projection.wrapper_span {
        let (start_edge, start_gap) = if offset == 0 {
            (
                start_mbp + projection.accumulated_edges.start_mbp,
                projection.accumulated_edges.start_half_gap,
            )
        } else {
            (zero, half_gap)
        };
        let (end_edge, end_gap) = if offset + 1 == projection.wrapper_span {
            (
                end_mbp + projection.accumulated_edges.end_mbp,
                projection.accumulated_edges.end_half_gap,
            )
        } else {
            (zero, half_gap)
        };
        let gap = start_gap + end_gap;
        let value = if gap.is_finite() {
            start_edge + end_edge + gap.max(zero)
        } else {
            gap
        };
        if !value.is_finite() {
            return Err(grid_track_sizing_error(
                wrapper,
                SpanningContributionError::InvalidNumeric { value },
            ));
        }
        if projection.envelope_empty_leaf && projection.wrapper_span == 1 {
            return memo
                .record_empty_leaf(&projection.wrapper_starts, value)
                .map_err(|_| {
                    grid_track_sizing_error(wrapper, SpanningContributionError::Allocation)
                });
        }
        for &start in &projection.wrapper_starts {
            let floor = &mut floors[start + offset];
            *floor = floor.max(value);
        }
    }
    Ok(())
}

fn collect_nested_lane_intrinsic_items<'input, Tree, M>(
    tree: &mut Tree,
    wrapper: Tree::Node,
    request: GridMinimumWrapperInput<'input, '_, Tree::Node, Tree::Scalar>,
    projection: NestedLaneIntrinsicProjectionOf<Tree::Scalar>,
    memo: &mut NestedLaneTraversalMemoOf<Tree::Scalar, Tree::Node>,
    purpose: GridContributionPurpose,
    contributions: &mut LanePurposeContributions<Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if projection.wrapper_starts.is_empty() || projection.wrapper_span == 0 {
        return Ok(());
    }
    if projection.hypothetical_descendants
        && !memo
            .record(&request, &projection)
            .map_err(|_| grid_track_sizing_error(wrapper, SpanningContributionError::Allocation))?
    {
        return Ok(());
    }
    if grid_minimum_wrapper_contained(tree.node_input(wrapper), projection.axis)
        || tree.children(wrapper).all(|child| {
            !is_in_flow_grid_child(GridChildInput::from_node(tree.node_input(child)).item())
        })
    {
        return collect_empty_lane_subgrid_edges(
            tree,
            wrapper,
            &request,
            &projection,
            memo,
            &mut contributions.base_floors,
        );
    }
    let _: Vec<()> = visit_grid_minimum_wrapper_children(tree, wrapper, request, |tree, view| {
        let child = view.node;
        let child_style = view.execution.child_style;
        // Inherited track geometry keeps the ancestor gutters. Authored gap
        // differences remain edge adjustments, resolved in this wrapper's
        // actual property frame rather than in the outer Lane container.
        let gap = child::child_subgrid_gap(
            view.execution.grid.style,
            projection.axis,
            view.execution.grid.constants.node_inner_size,
        )
        .map_err(|status| crate::error::value_resolution_error(wrapper, status))?;
        let wrapper_gap = match gap {
            ResolvedSubgridGap::Normal => projection.parent_gap,
            ResolvedSubgridGap::Length(value) => value,
        };
        let mut local_projection = projection.clone();
        local_projection.wrapper_edges =
            lane_wrapper_edges_with_gutter(&projection, view.execution.grid.constants);
        let half_gap_difference =
            (wrapper_gap - projection.parent_gap) / Tree::Scalar::from_f64(2.0);
        local_projection.interior_half_gap = projection.interior_half_gap + half_gap_difference;
        let placement = view.execution.grid.placements.items[view.source_index.get()];
        let placement = match projection.axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let explicit_range =
            signed_definite_axis_range(placement, projection.wrapper_span, projection.axis)
                .map_err(|error| grid_placement_demand_error(child, error))?
                .map(|(start, end)| {
                    clamp_subgrid_axis_range(
                        start.max(0) as usize,
                        end.max(0) as usize,
                        projection.wrapper_span,
                    )
                });
        let child_span = explicit_range.map_or_else(
            || {
                placement
                    .span()
                    .map_or(1, crate::GridSpan::get)
                    .min(projection.wrapper_span)
            },
            |(start, end)| end - start,
        );
        let explicit_local_start = explicit_range.map(|(start, _)| start);
        let local_starts = if let Some(start) = explicit_local_start
            && !projection.hypothetical_descendants
        {
            vec![start]
        } else {
            candidate_starts(projection.wrapper_span, child_span).collect()
        };
        for start in local_starts {
            let end = start + child_span;
            let groups = nested_lane_candidate_groups(
                &local_projection,
                child_span,
                std::iter::once(start),
                explicit_local_start,
            );
            if groups.is_empty() {
                continue;
            }
            let mut execution = view.execution;
            match projection.axis {
                GridAxisKind::Column => {
                    execution.area.column = start;
                    execution.area.column_end = end;
                    execution.area.size.inline = view.column_geometry.span_extent(start, end);
                }
                GridAxisKind::Row => {
                    execution.area.row = start;
                    execution.area.row_end = end;
                    execution.area.size.block = view.row_geometry.span_extent(start, end);
                }
            }
            let prepared = intrinsic_stage_child_input::<Tree, M>(child, &execution)?;
            let item_report = execution
                .subgrid_item
                .expect("canonical wrapper preparation retains each actual item report");
            if let Some(child_axis) =
                inherited_subgrid_axis_for_parent_axis(item_report, projection.axis)
            {
                let axis_report = match child_axis {
                    GridAxisKind::Column => item_report.column,
                    GridAxisKind::Row => item_report.row,
                };
                let mut parent_context = GridParentContext {
                    columns: intrinsic_subgrid_axis_parent_context(
                        item_report.column,
                        execution.area,
                        view.minimum_source.gap,
                        view.minimum_source,
                        execution.grid.named_columns,
                        execution.grid.named_rows,
                        execution.grid.area_facts,
                    )
                    .map_err(|error| subgrid_child_context_error(child, error))?,
                    rows: intrinsic_subgrid_axis_parent_context(
                        item_report.row,
                        execution.area,
                        view.minimum_source.gap,
                        view.minimum_source,
                        execution.grid.named_columns,
                        execution.grid.named_rows,
                        execution.grid.area_facts,
                    )
                    .map_err(|error| subgrid_child_context_error(child, error))?,
                };
                if projection.hypothetical_descendants {
                    let inherited = match child_axis {
                        GridAxisKind::Column => parent_context.columns.as_mut(),
                        GridAxisKind::Row => parent_context.rows.as_mut(),
                    };
                    let placed = view.execution.area;
                    let named_range = explicit_range.unwrap_or(match projection.axis {
                        GridAxisKind::Column => (placed.column, placed.column_end),
                        GridAxisKind::Row => (placed.row, placed.row_end),
                    });
                    // A clipped automatic placement can be shorter than the
                    // candidate geometry. Its local names still use the candidate span.
                    if let Some(inherited) = inherited
                        && named_range.1 - named_range.0 == child_span
                    {
                        (inherited.parent_start, inherited.parent_end) = named_range;
                    }
                }
                let wrapper_edges = lane_child_edge_facts(
                    tree,
                    child,
                    child_style,
                    prepared.containing_flow_axes(),
                    prepared.parent(),
                    child_axis,
                )?;
                let child_memo = if projection.hypothetical_descendants {
                    memo.child(view.source_index).map_err(|_| {
                        grid_track_sizing_error(child, SpanningContributionError::Allocation)
                    })?
                } else {
                    &mut *memo
                };
                for group in groups {
                    collect_nested_lane_intrinsic_items(
                        tree,
                        child,
                        GridMinimumWrapperInput {
                            axis: child_axis,
                            container: execution
                                .child_container
                                .expect("an inherited child retains its real Grid container"),
                            parent_context: &parent_context,
                            prepared,
                            containing_flow_axes: view.minimum_source.flow_axes,
                            content_box_estimate: None,
                            traversal: GridMinimumWrapperTraversal::LaneCandidates,
                            query: execution.grid.query,
                        },
                        NestedLaneIntrinsicProjectionOf {
                            root_track_count: projection.root_track_count,
                            axis: child_axis,
                            wrapper_span: child_span,
                            wrapper_starts: group.starts,
                            hypothetical_descendants: projection.hypothetical_descendants,
                            envelope_empty_leaf: projection.hypothetical_descendants,
                            reversed: projection.reversed ^ axis_report.mapping.reversed,
                            parent_gap: wrapper_gap,
                            interior_half_gap: local_projection.interior_half_gap,
                            accumulated_edges: group.edges,
                            wrapper_edges,
                        },
                        child_memo,
                        purpose,
                        contributions,
                    )?;
                }
                continue;
            }
            let frame = view
                .minimum_source
                .frame(child, projection.axis, execution.area, prepared)?
                .ok_or_else(|| minimum_source_error(child))?;
            let prepared = frame.prepared();
            let value = lane_child_contribution(
                tree,
                child,
                GridMinimumContext {
                    item: child_style,
                    flow_axes: frame.flow_axes,
                    axis: frame.axis,
                    algorithm: frame.algorithm,
                    prepared,
                    margin_basis: prepared.parent(),
                    span: frame.span,
                    opposite_fixed_max_span: frame.opposite_fixed_max_span,
                    subgrid_requests: lane_child_subgrid_requests(execution.child_container),
                },
                purpose,
                |tree, input| compute_prepared_intrinsic_grid_child(tree, child, execution, input),
            )?;
            for group in groups {
                let adjustment = group.edges.start_mbp
                    + group.edges.end_mbp
                    + group.edges.start_half_gap
                    + group.edges.end_half_gap;
                let adjusted = value + adjustment;
                if !adjusted.is_finite() {
                    return Err(grid_track_sizing_error(
                        child,
                        SpanningContributionError::InvalidNumeric { value: adjusted },
                    ));
                }
                for start in group.starts {
                    contributions.rows.try_reserve(1).map_err(|_| {
                        grid_track_sizing_error(child, SpanningContributionError::Allocation)
                    })?;
                    contributions.rows.push(LanePurposeContributionOf {
                        source: child,
                        span: GridTrackSpan::new(start + 1, start + child_span + 1),
                        value: adjusted.max(Tree::Scalar::ZERO),
                    });
                }
            }
        }
        Ok(None)
    })?;
    Ok(())
}

fn lane_child_edge_facts<'input, Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    child_style: &GridItemProjection<'input, Tree::Scalar>,
    containing_flow_axes: crate::geometry::FlowAxes,
    containing_size: Size<Option<Tree::Scalar>>,
    axis: GridAxisKind,
) -> LayoutResultOf<<Tree as Traverse>::Node, LaneIntrinsicEdgeFactsOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let margin = intrinsic_contribution_margin(child_style, containing_flow_axes, containing_size)
        .map_err(|status| crate::error::value_resolution_error(child, status))?;
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.padding,
            containing_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.border,
            containing_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let child_flow_axes =
        crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction);
    let logical_mbp = child_flow_axes.logical_edges(margin + padding + border);
    let (start_mbp, end_mbp) = match axis.logical_axis() {
        LogicalAxis::Inline => (logical_mbp.inline_start, logical_mbp.inline_end),
        LogicalAxis::Block => (logical_mbp.block_start, logical_mbp.block_end),
    };
    Ok(LaneIntrinsicEdgeFactsOf {
        start_mbp,
        end_mbp,
        start_half_gap: Tree::Scalar::ZERO,
        end_half_gap: Tree::Scalar::ZERO,
    })
}

/// Prepare a child of the real Lane grid axis without a fictitious stacking
/// track span. Preferred properties are authored facts in the incoming parent
/// frame; existing known dimensions remain query constraints.
pub(in crate::grid) fn prepare_grid_lanes_intrinsic_child_input<S: LayoutScalar>(
    item: &GridItemProjection<'_, S>,
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
    query: IntrinsicGridQuery<'_, S>,
    incoming: ComputeInputOf<S>,
) -> Result<ComputeInputOf<S>, SizingResolutionError<S>> {
    let preferred = grid_lanes_child_sizing_preflight(item, incoming.parent())?;
    let requested = flow_axes.logical_size(incoming.sizing_request().axes());
    let queried_available = match axis {
        GridAxisKind::Column => requested.inline,
        GridAxisKind::Row => requested.block,
    };
    let mut available = flow_axes.logical_size(lane_child_intrinsic_available(
        flow_axes,
        axis,
        preferred,
        queried_available,
    ));
    let logical_preferred = flow_axes.logical_size(preferred);
    let (opposite, source, opposite_preferred) = match axis {
        GridAxisKind::Column => (GridAxisKind::Row, query.rows, logical_preferred.block),
        GridAxisKind::Row => (
            GridAxisKind::Column,
            query.columns,
            logical_preferred.inline,
        ),
    };
    if matches!(opposite_preferred, ResolvedPreferredSize::Auto)
        && query.axis == Some(axis)
        && query.stage.has_resolved_axis(opposite)
        && source.is_some()
    {
        // Keep the issued opposite constraint instead of the Auto fallback.
        // Explicit preferred sizing retains its own MIN/MAX/definite operation.
        // This does not create a known dimension, percentage basis, or
        // inherited stacking-track span.
        match opposite {
            GridAxisKind::Column => available.inline = requested.inline,
            GridAxisKind::Row => available.block = requested.block,
        }
    }
    let available = flow_axes.physical_size(available);
    let mut prepared = ComputeInputOf::for_child(
        incoming.run_mode(),
        incoming.sizing_mode(),
        incoming.requested_axis(),
        incoming.known(),
        incoming.parent(),
        incoming.containing_layout_context(),
        incoming.sizing_request().with_axes(available),
    )
    .with_baseline_demand_from(incoming)
    .with_settled_auto_scrollbars(incoming.settled_auto_scrollbars())
    .with_containing_auto_scrollbar_pass(incoming.containing_auto_scrollbar_pass());
    if let Some(indent) = incoming.block_indent_context() {
        prepared = prepared.with_block_indent_context(indent);
    }
    Ok(prepared)
}

// Change only the queried purpose. The actual opposite-axis preparation,
// percentage frame, baseline and scrollbar inputs stay with the same query.
fn lane_intrinsic_query_for_role<S: LayoutScalar>(
    prepared: ComputeInputOf<S>,
    flow_axes: crate::geometry::FlowAxes,
    axis: GridAxisKind,
    role: SizingOperation,
) -> ComputeInputOf<S> {
    let mut available = flow_axes.logical_size(prepared.sizing_request().axes());
    match axis {
        GridAxisKind::Column => available.inline = available.inline.with_operation(role),
        GridAxisKind::Row => available.block = available.block.with_operation(role),
    }
    let mut input = ComputeInputOf::for_child(
        prepared.run_mode(),
        prepared.sizing_mode(),
        prepared.requested_axis(),
        prepared.known(),
        prepared.parent(),
        prepared.containing_layout_context(),
        prepared
            .sizing_request()
            .with_axes(flow_axes.physical_size(available)),
    )
    .with_baseline_demand_from(prepared)
    .with_settled_auto_scrollbars(prepared.settled_auto_scrollbars())
    .with_containing_auto_scrollbar_pass(prepared.containing_auto_scrollbar_pass());
    if let Some(indent) = prepared.block_indent_context() {
        input = input.with_block_indent_context(indent);
    }
    input
}

fn lane_child_contribution<'input, Tree, M>(
    tree: &mut Tree,
    child: Tree::Node,
    mut minimum: GridMinimumContext<'_, 'input, Tree::Scalar>,
    purpose: GridContributionPurpose,
    mut query: impl FnMut(
        &mut Tree,
        ComputeInputOf<Tree::Scalar>,
    )
        -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>,
) -> LayoutResultOf<Tree::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let item = minimum.item;
    let flow_axes = minimum.flow_axes;
    let axis = minimum.axis;
    let role = match purpose {
        GridContributionPurpose::Minimum | GridContributionPurpose::MinContent => {
            SizingOperation::MinContent
        }
        GridContributionPurpose::MaxContent => SizingOperation::MaxContent,
    };
    minimum.prepared = lane_intrinsic_query_for_role(minimum.prepared, flow_axes, axis, role);
    let prepared = minimum.prepared;
    let output = match purpose {
        GridContributionPurpose::Minimum => {
            compute_grid_minimum_contribution(child, minimum, |input| query(tree, input))?
        }
        GridContributionPurpose::MinContent | GridContributionPurpose::MaxContent => {
            query(tree, prepared)?
        }
    };
    let margin =
        intrinsic_contribution_margin(item, prepared.containing_flow_axes(), prepared.parent())
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
    // Border-box queries already contain actual padding/border. Baseline
    // constraints retain their real ordinary members in the owning axis.
    Ok(lane_axis_size(flow_axes.logical_size(output.size), axis)
        + lane_axis_margin_sum(flow_axes.logical_edges(margin), axis))
}

fn lane_axis_size<S: LayoutScalar>(size: LogicalSizeOf<S>, axis: GridAxisKind) -> S {
    match axis.logical_axis() {
        LogicalAxis::Inline => size.inline,
        LogicalAxis::Block => size.block,
    }
}

fn lane_axis_margin_sum<S: LayoutScalar>(
    margin: crate::geometry::LogicalEdgesOf<S>,
    axis: GridAxisKind,
) -> S {
    match axis.logical_axis() {
        LogicalAxis::Inline => margin.inline_sum(),
        LogicalAxis::Block => margin.block_sum(),
    }
}

pub(super) fn layout_grid_lanes_children<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: GridLanesLayoutInput<'input, '_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> GridChildrenLayoutResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let GridLanesLayoutInput {
        scroll_box,
        style,
        constants,
        minimum_functions,
        container_content_box_size,
        columns,
        rows,
        row_tracks,
        track_basis,
        parent_context,
        gap,
        mut context,
        subgrid_report,
        placements,
        containing_auto_scrollbar_pass,
        sizing_stage,
        stacking_operation,
        execution_input,
    } = input;

    if columns.is_empty() || rows.is_empty() {
        for (source_index, child) in tree
            .children(node)
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
        {
            stage_hidden_grid_child(
                tree,
                child,
                source_index,
                ComputeInputOf::hidden_in_containing_pass(
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    containing_auto_scrollbar_pass,
                ),
            )?;
        }
        return Ok(GridChildrenLayout {
            executed_baselines: execution_input.run_mode().is_perform_layout().then(|| {
                ExecutedBaselineInventory::empty(
                    node,
                    ExecutedBaselineFrame {
                        style,
                        size: constants.node_outer_size.unwrap_or(
                            constants
                                .flow_axes
                                .physical_size(container_content_box_size)
                                + constants.content_box_inset.sum_axes(),
                        ),
                        execution_input,
                        stage: sizing_stage,
                        columns: (grid_axis_for_grid_lanes(style) == GridAxisKind::Column
                            && sizing_stage.has_resolved_axis(GridAxisKind::Column))
                        .then_some(ExecutedBaselineAxisFrame {
                            starts: &[],
                            sizes: columns,
                        }),
                        rows: (grid_axis_for_grid_lanes(style) == GridAxisKind::Row
                            && sizing_stage.has_resolved_axis(GridAxisKind::Row))
                        .then_some(ExecutedBaselineAxisFrame {
                            starts: &[],
                            sizes: rows,
                        }),
                        sizing: ExecutedBaselineSizing {
                            row_tracks,
                            track_basis,
                        },
                    },
                )
            }),
            visible_content_size: Size::ZERO,
            contributions: empty_grid_contributions(),
            baselines: BaselinesOf::NONE,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        });
    }

    let flow_axes = constants.flow_axes;
    let track_content_size = LogicalSizeOf::new(
        track_sum_with_gutters(columns, gap.inline, Some(&context.column_gutters)),
        track_sum_with_gutters(rows, gap.block, Some(&context.row_gutters)),
    );
    let content_box_size = flow_axes
        .logical_size(constants.node_inner_size)
        .unwrap_or(container_content_box_size);
    let alignment_free_space = content_box_size - track_content_size;
    let column_has_collapse = context
        .column_gutters
        .collapsed()
        .iter()
        .any(|value| *value);
    let row_has_collapse = context.row_gutters.collapsed().iter().any(|value| *value);
    let legacy_column_alignment = grid_alignment(
        alignment_free_space.inline,
        columns.len(),
        gap.inline,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let legacy_row_alignment = grid_alignment(
        alignment_free_space.block,
        rows.len(),
        gap.block,
        style.align_content.unwrap_or(AlignContent::Stretch),
    );
    let column_alignment = if column_has_collapse {
        ordinary_grid_axis_alignment(
            alignment_free_space.inline,
            &context.column_gutters,
            style.justify_content.unwrap_or(AlignContent::Stretch),
        )
    } else {
        OrdinaryGridAxisAlignment {
            start: legacy_column_alignment.start,
            gutter_after: vec![legacy_column_alignment.gap; columns.len().saturating_sub(1)],
        }
    };
    let row_alignment = if row_has_collapse {
        ordinary_grid_axis_alignment(
            alignment_free_space.block,
            &context.row_gutters,
            style.align_content.unwrap_or(AlignContent::Stretch),
        )
    } else {
        OrdinaryGridAxisAlignment {
            start: legacy_row_alignment.start,
            gutter_after: vec![legacy_row_alignment.gap; rows.len().saturating_sub(1)],
        }
    };
    context.column_gutters = OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
        columns.len(),
        context.column_gutters.collapsed(),
        context.column_gutters.active_boundary_after(),
        &column_alignment.gutter_after,
    );
    context.row_gutters = OrdinaryGridAxisGuttersOf::from_active_boundary_gutters(
        rows.len(),
        context.row_gutters.collapsed(),
        context.row_gutters.active_boundary_after(),
        &row_alignment.gutter_after,
    );
    context.gap = gap;
    let grid_axis_gap = match grid_axis_for_grid_lanes(style) {
        GridAxisKind::Column => gap.inline,
        GridAxisKind::Row => gap.block,
    };
    let Ok(lane_report) = resolve_grid_lanes_placement_with_resolved_tracks(
        tree,
        node,
        style,
        constants,
        context.clone(),
        columns,
        rows,
        placements,
        grid_axis_gap,
        content_box_size.map(Some),
        stacking_operation,
        minimum_functions,
        subgrid_report,
        sizing_stage,
    )?
    else {
        return Ok(GridChildrenLayout {
            executed_baselines: None,
            visible_content_size: Size::ZERO,
            contributions: empty_grid_contributions(),
            baselines: BaselinesOf::NONE,
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
        });
    };
    let logical_content_box_inset = flow_axes.logical_edges(constants.content_box_inset);
    let column_geometry = parent_context
        .columns
        .as_ref()
        .map(|axis| axis.geometry.clone())
        .unwrap_or_else(|| {
            UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), &context.column_gutters)
        })
        .translated(
            logical_content_box_inset.inline_start
                + parent_context
                    .columns
                    .as_ref()
                    .map_or(column_alignment.start, |axis| axis.offset),
        );
    let row_geometry = parent_context
        .rows
        .as_ref()
        .map(|axis| axis.geometry.clone())
        .unwrap_or_else(|| {
            UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), &context.row_gutters)
        })
        .translated(
            logical_content_box_inset.block_start
                + parent_context
                    .rows
                    .as_ref()
                    .map_or(row_alignment.start, |axis| axis.offset),
        );
    let column_offsets = if column_has_collapse || parent_context.columns.is_some() {
        column_geometry.line_offsets().to_vec()
    } else {
        grid_axis_logical_offsets(
            columns,
            None,
            logical_content_box_inset.inline_start,
            legacy_column_alignment,
        )
    };
    let row_offsets = if row_has_collapse || parent_context.rows.is_some() {
        row_geometry.line_offsets().to_vec()
    } else {
        grid_axis_logical_offsets(
            rows,
            None,
            logical_content_box_inset.block_start,
            legacy_row_alignment,
        )
    };
    let containing_size = constants.node_outer_size.unwrap_or(
        flow_axes.physical_size(container_content_box_size)
            + constants.content_box_inset.sum_axes(),
    );
    let lane_axis = lane_report.lane_axis;
    // Distributed values use their fallback for the single stacking subject.
    let stacking_alignment = |alignment| match alignment {
        AlignContent::SpaceAround | AlignContent::SpaceEvenly => AlignContent::SafeCenter,
        AlignContent::SpaceBetween => AlignContent::Start,
        alignment => alignment,
    };
    let lane_axis_alignment_start = match lane_axis.logical_axis() {
        LogicalAxis::Inline => {
            grid_alignment(
                content_box_size.inline - lane_report.range_extent(),
                1,
                gap.inline,
                stacking_alignment(style.justify_content.unwrap_or(AlignContent::Stretch)),
            )
            .start
                + logical_content_box_inset.inline_start
        }
        LogicalAxis::Block => {
            grid_alignment(
                content_box_size.block - lane_report.range_extent(),
                1,
                gap.block,
                stacking_alignment(style.align_content.unwrap_or(AlignContent::Stretch)),
            )
            .start
                + logical_content_box_inset.block_start
        }
    };
    let lane_axis_alignment_start = lane_axis_alignment_start - lane_report.range_start;
    let children = tree.children(node).collect::<Vec<_>>();
    let mut child_contributions = Vec::new();
    let PendingGridItems {
        in_flow: mut pending_items,
        absolute: pending_absolute,
    } = layout_grid_child_items(
        tree,
        node,
        GridChildItemsInput {
            children: &children,
            item_count: children.len(),
            style,
            constants,
            columns,
            rows,
            row_tracks,
            minimum_functions,
            track_basis,
            gap,
            named_columns: &context.named_columns,
            named_rows: &context.named_rows,
            area_facts: context.area_facts.as_ref(),
            subgrid_report,
            placements,
            containing_auto_scrollbar_pass,
            sizing_stage,
            geometry: FinalGridChildGeometry::Lanes {
                grid_axis: grid_axis_for_grid_lanes(style),
            },
        },
        |source_index, child_style| {
            if !is_in_flow_grid_child(child_style) {
                return GridItemDisposition::Skip;
            }
            let Some(packed_item) = lane_report.item(source_index) else {
                return GridItemDisposition::Skip;
            };
            let item_offset = packed_item;

            let start = item_offset.grid_axis_start - 1;
            let end = start + item_offset.grid_axis_span;
            let grid_axis_size = match lane_report.grid_axis {
                GridAxisKind::Column => track_span_sum_with_gutters(
                    columns,
                    start,
                    end.min(columns.len()),
                    gap.inline,
                    Some(&context.column_gutters),
                ),
                GridAxisKind::Row => track_span_sum_with_gutters(
                    rows,
                    start,
                    end.min(rows.len()),
                    gap.block,
                    Some(&context.row_gutters),
                ),
            };
            let containing_block = GridLanesItemContainingBlockOf::new(
                flow_axes,
                lane_report.grid_axis,
                grid_axis_size,
                content_box_size.map(Some),
            );
            let containing_logical_size = containing_block.definite_logical_size();
            let alignment_size = match lane_axis.logical_axis() {
                LogicalAxis::Inline => {
                    LogicalSizeOf::new(packed_item.alignment_extent, containing_logical_size.block)
                }
                LogicalAxis::Block => {
                    LogicalSizeOf::new(containing_logical_size.inline, packed_item.alignment_extent)
                }
            };
            let area = grid_lanes_item_area(lane_report.grid_axis, start, end, alignment_size);

            let physical_area_size = containing_block
                .physical_size()
                .map(|extent| extent.expect("final grid-lanes containing extent is definite"));
            GridItemDisposition::Placed {
                area,
                space: GridItemLayoutSpace {
                    alignment_size: flow_axes.physical_size(alignment_size),
                    containing_size: physical_area_size,
                    natural_stacking_margin_box: Some(item_offset.lane_axis_margin_box),
                },
            }
        },
    )?;

    let grid_axis = lane_report.grid_axis;
    let executed_frame = ExecutedBaselineFrame {
        style,
        size: containing_size,
        execution_input,
        stage: sizing_stage,
        columns: (grid_axis == GridAxisKind::Column
            && sizing_stage.has_resolved_axis(GridAxisKind::Column))
        .then_some(ExecutedBaselineAxisFrame {
            starts: &column_offsets,
            sizes: columns,
        }),
        rows: (grid_axis == GridAxisKind::Row && sizing_stage.has_resolved_axis(GridAxisKind::Row))
            .then_some(ExecutedBaselineAxisFrame {
                starts: &row_offsets,
                sizes: rows,
            }),
        sizing: ExecutedBaselineSizing {
            row_tracks,
            track_basis,
        },
    };
    let executed_inventory =
        capture_executed_baselines(node, executed_frame, &mut pending_items, subgrid_report)?;
    let ancestor_baseline_groups =
        final_ancestor_baseline_groups(&executed_inventory).with_parent_context(parent_context);
    let current_grid_baseline_groups = ancestor_baseline_groups.placement_groups();
    refresh_subgrid_items_with_baselines(
        tree,
        SubgridBaselineRefreshInput {
            node,
            container_style: style,
            columns,
            rows,
            minimum_functions,
            column_geometry: &column_geometry,
            row_geometry: &row_geometry,
            row_tracks,
            track_basis,
            gap,
            named_columns: context.named_columns.clone(),
            named_rows: context.named_rows.clone(),
            area_facts: context.area_facts.clone(),
            template_area_expanded_axes: TemplateAreaExpandedAxes::new(
                parent_context.columns.as_ref().map_or_else(
                    || {
                        context
                            .topology
                            .template_area_expanded_axes()
                            .for_axis(GridAxisKind::Column)
                    },
                    |axis| axis.template_area_expanded,
                ),
                parent_context.rows.as_ref().map_or_else(
                    || {
                        context
                            .topology
                            .template_area_expanded_axes()
                            .for_axis(GridAxisKind::Row)
                    },
                    |axis| axis.template_area_expanded,
                ),
            ),
            subgrid_report,
            ancestor_baseline_groups: &ancestor_baseline_groups,
            executed_frame,
            orthogonal_fallback: constants.child_orthogonal_fallback,
            original_executed: &executed_inventory,
            containing_auto_scrollbar_pass,
            sizing_stage,
        },
        &mut pending_items,
        |item| {
            let packed = lane_report
                .item(item.source_index)
                .expect("refreshed lane item retains its packed cell");
            let mut containing = item.area.size;
            match lane_axis.logical_axis() {
                LogicalAxis::Inline => containing.inline = content_box_size.inline,
                LogicalAxis::Block => containing.block = content_box_size.block,
            }
            GridItemLayoutSpace {
                alignment_size: flow_axes.physical_size(item.area.size),
                containing_size: flow_axes.physical_size(containing),
                natural_stacking_margin_box: Some(packed.lane_axis_margin_box),
            }
        },
    )?;

    for item in &mut pending_items {
        let baseline_offset = baseline_aligned_axis_offset(BaselineAlignedAxisInput {
            item,
            child_style: &item.style,
            container_style: style,
            group: ancestor_baseline_groups.for_axis(grid_axis),
            axis: grid_axis,
            geometry: match grid_axis {
                GridAxisKind::Column => &column_geometry,
                GridAxisKind::Row => &row_geometry,
            },
            row_tracks,
            track_basis,
            subgrid_item: subgrid_report.items.get(item.source_index).copied(),
            container_flow_axes: flow_axes,
            inherited_owner_targets: ancestor_baseline_groups.inherited_targets_for_axis(grid_axis),
            current_grid: node,
        })
        .map_err(|error| {
            subgrid_child_context_container_error(
                node,
                item.node,
                SubgridChildContextError::BaselineInheritance(error),
            )
        })?;
        let inline_offset = if grid_axis == GridAxisKind::Column {
            baseline_offset.unwrap_or(item.horizontal_axis.offset)
        } else {
            item.horizontal_axis.offset
        };
        let block_offset = if grid_axis == GridAxisKind::Row {
            baseline_offset.unwrap_or(item.vertical_axis.offset)
        } else {
            item.vertical_axis.offset
        };
        let lane_offset = lane_report
            .item(item.source_index)
            .expect("pending lane item retains its packed cell")
            .offset;
        let selected_inline_lane_offset = lane_axis_alignment_start;
        let selected_block_track_offset =
            grid_area_track_offset(&row_offsets, item.area.row, item.area.row_end);
        let normal_logical_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => LogicalPointOf::new(
                selected_inline_lane_offset + lane_offset + inline_offset,
                selected_block_track_offset + block_offset,
            ),
            LogicalAxis::Block => LogicalPointOf::new(
                grid_area_inline_offset(&column_offsets, item.area) + inline_offset,
                lane_axis_alignment_start + lane_offset + block_offset,
            ),
        };
        let normal_location = flow_axes.physical_point(
            normal_logical_location,
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let logical_location = LogicalPointOf::new(
            normal_logical_location.inline + item.logical_relative_offset.inline,
            normal_logical_location.block + item.logical_relative_offset.block,
        );
        let location = flow_axes.physical_point(
            logical_location,
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let baseline_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => flow_axes.physical_point(
                LogicalPointOf::new(
                    lane_axis_alignment_start + lane_offset + inline_offset,
                    grid_area_track_offset(&row_offsets, item.area.row, item.area.row_end)
                        + block_offset,
                ),
                flow_axes.logical_size(item.output.size),
                containing_size,
            ),
            LogicalAxis::Block => normal_location,
        };
        let area_origin = LogicalPointOf::new(
            grid_area_inline_offset(&column_offsets, item.area),
            grid_area_track_offset(&row_offsets, item.area.row, item.area.row_end),
        );
        item.block_offset = normal_logical_location.block - area_origin.block;
        item.location = baseline_location;
        let scroll_geometry = item
            .output
            .scroll_geometry
            .expect("pending grid-lanes item retains canonical geometry");
        let subgrid_item = subgrid_report
            .items
            .get(item.source_index)
            .copied()
            .expect("grid-lanes subgrid report preserves source identity");
        let (horizontal, vertical) =
            subgrid_parent_propagation_axes(subgrid_item, flow_axes, item.child_flow_axes);
        child_contributions.push(GridChildContribution {
            source_index: crate::SourceIndex::new(item.source_index),
            location,
            margin: item.margin,
            geometry: scroll_geometry,
            descendants: scroll_geometry
                .propagatable_descendant_intervals()
                .retain_physical_axes(horizontal, vertical),
            in_flow: true,
        });
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            item.node,
            crate::output::NormalGeometryOf::box_geometry(
                normal_location,
                item.output.size,
                scroll_geometry.content_box(),
            ),
            NodeOutputOf {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: item.output.size,
                named_baselines: item.output.named_baselines,
                content_size: item.output.content_size,
                scroll_geometry: Some(scroll_geometry),
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        ));
    }
    let mut baselines = logical_grid_container_baselines(
        &pending_items,
        &current_grid_baseline_groups,
        &row_offsets,
        rows,
        flow_axes,
        containing_size,
    );
    lane_report.replace_stacking_baselines(&pending_items, flow_axes, &mut baselines.baselines);
    if !pending_absolute.is_empty() {
        let in_flow_contributions = grid_scroll_contributions(
            child_contributions.clone(),
            None,
            flow_axes,
            constants.padding,
        )
        .map_err(|error| layout_child_geometry_error(node, node, error))?;
        let auto_area = grid_absolute_auto_area(in_flow_contributions, scroll_box)
            .map_err(|error| layout_child_geometry_error(node, node, error))?;
        for (source_index, child, column, row) in pending_absolute {
            let child_style = placements.item_input(source_index);
            child_contributions.push(layout_absolute_grid_child(
                tree,
                child,
                source_index,
                child_style,
                AbsoluteGridContext::for_lanes(
                    AbsoluteGridContextInput {
                        container_style: style,
                        constants,
                        containing_size,
                        auto_area,
                        column,
                        row,
                        column_geometry: &column_geometry,
                        row_geometry: &row_geometry,
                        lines: context.lines,
                    },
                    lane_axis_alignment_start + lane_report.range_start,
                    lane_axis_alignment_start + lane_report.range_end,
                )
                .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass),
            )?);
        }
    }
    let mut contributions =
        grid_scroll_contributions(child_contributions, None, flow_axes, constants.padding)
            .map_err(|error| layout_child_geometry_error(node, node, error))?;
    let inline_start = column_offsets
        .iter()
        .copied()
        .reduce(Tree::Scalar::min)
        .unwrap_or(logical_content_box_inset.inline_start);
    let inline_end = column_offsets
        .iter()
        .copied()
        .zip(columns.iter().copied())
        .map(|(offset, size)| offset + size)
        .reduce(Tree::Scalar::max)
        .unwrap_or(inline_start);
    let block_start = row_offsets
        .iter()
        .copied()
        .reduce(Tree::Scalar::min)
        .unwrap_or(logical_content_box_inset.block_start);
    let block_end = row_offsets
        .iter()
        .copied()
        .zip(rows.iter().copied())
        .map(|(offset, size)| offset + size)
        .reduce(Tree::Scalar::max)
        .unwrap_or(block_start);
    let mut logical_subject_size =
        LogicalSizeOf::new(inline_end - inline_start, block_end - block_start);
    let mut logical_subject_origin = LogicalPointOf::new(inline_start, block_start);
    let stacking_start = lane_axis_alignment_start + lane_report.range_start;
    let stacking_end = lane_axis_alignment_start + lane_report.range_end;
    let stacking_origin = stacking_start.min(stacking_end);
    let stacking_extent = stacking_start.max(stacking_end) - stacking_origin;
    match lane_axis.logical_axis() {
        LogicalAxis::Inline => {
            logical_subject_origin.inline = stacking_origin;
            logical_subject_size.inline = stacking_extent;
        }
        LogicalAxis::Block => {
            logical_subject_origin.block = stacking_origin;
            logical_subject_size.block = stacking_extent;
        }
    }
    let subject_size = flow_axes.physical_size(logical_subject_size);
    let subject_origin = flow_axes.physical_point(
        logical_subject_origin,
        logical_subject_size,
        containing_size,
    );
    let track_subject = crate::ScrollRectOf::try_new(subject_origin, subject_size)
        .map_err(|error| layout_child_geometry_error(node, node, error))?;
    if style.justify_content.is_some() {
        contributions.set_active_alignment_subject(flow_axes.inline_axis(), track_subject);
    }
    if style.align_content.is_some() {
        contributions.set_active_alignment_subject(flow_axes.block_axis(), track_subject);
    }
    let visible_content_size = contributions
        .content_size_from_anchor(Point::ZERO)
        .map_err(|error| layout_child_geometry_error(node, node, error))?;

    Ok(GridChildrenLayout {
        visible_content_size,
        contributions,
        baselines: baselines.baselines,
        baseline_groups: current_grid_baseline_groups,
        executed_baselines: execution_input
            .run_mode()
            .is_perform_layout()
            .then_some(executed_inventory),
    })
}

pub(super) fn normalize_grid_lanes_stacking_axis_sizing<'input, S: LayoutScalar>(
    child_style: &GridItemProjection<'input, S>,
    container_style: &GridContainerProjection<'_, S>,
    lane_axis: GridAxisKind,
    flow_axes: crate::geometry::FlowAxes,
    lane_axis_margin_box: S,
    item: &mut GridItemSizing<S>,
) {
    let logical_style_size =
        flow_axes.logical_size(Size::new(&child_style.size.width, &child_style.size.height));
    let mut logical_known = flow_axes.logical_size(item.known);
    let stacking_axis = match lane_axis.logical_axis() {
        LogicalAxis::Inline => flow_axes.inline_axis(),
        LogicalAxis::Block => flow_axes.block_axis(),
    };
    // A packed natural ratio size is a result of the child's intrinsic inline
    // layout. Let the child settle it again with the same preferred sizing.
    let child_owns_ratio_cross = child_style.aspect_ratio.is_some()
        && child_style.flow_axes.block_axis() == stacking_axis
        && child_style
            .flow_axes
            .logical_size(child_style.flex_preferred_axes)
            .inline;

    let logical_margin = flow_axes.logical_edges(
        item.unresolved_margin
            .map(|margin| margin.unwrap_or(S::ZERO)),
    );
    // Normalizing a natural lane size also replaces any stretch allocation
    // made before the stacking alignment was selected.
    let natural_stacking_size =
        |margin| (!child_owns_ratio_cross).then(|| (lane_axis_margin_box - margin).max(S::ZERO));
    match lane_axis.logical_axis() {
        LogicalAxis::Inline => {
            let alignment = resolve_grid_item_normal_alignment(
                child_style.justify_self,
                container_style.justify_items,
                child_style.item_is_replaced,
                logical_style_size.inline.is_auto(),
                AlignItems::Start,
            );
            let alignment = stacking_self_alignment(
                alignment,
                flow_axes,
                crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction),
                lane_axis,
            );
            let margins = flow_axes.logical_edges(item.unresolved_margin);
            if (alignment != AlignItems::Stretch
                || margins.inline_start.is_none()
                || margins.inline_end.is_none())
                && logical_style_size.inline.is_auto()
            {
                logical_known.inline = natural_stacking_size(logical_margin.inline_sum());
            }
            item.justify_self = alignment;
        }
        LogicalAxis::Block => {
            let alignment = resolve_grid_item_normal_alignment(
                child_style.align_self,
                container_style.align_items,
                child_style.item_is_replaced,
                logical_style_size.block.is_auto(),
                AlignItems::Start,
            );
            let alignment = stacking_self_alignment(
                alignment,
                flow_axes,
                crate::geometry::FlowAxes::new(child_style.writing_mode, child_style.direction),
                lane_axis,
            );
            let margins = flow_axes.logical_edges(item.unresolved_margin);
            if (alignment != AlignItems::Stretch
                || margins.block_start.is_none()
                || margins.block_end.is_none())
                && logical_style_size.block.is_auto()
            {
                logical_known.block = natural_stacking_size(logical_margin.block_sum());
            }
            item.align_self = alignment;
        }
    }
    item.known = flow_axes.physical_size(logical_known);
}

#[derive(Clone, Copy)]
pub(super) struct LaneAxisMarginBoxMeasureInput<'input, 'a, S: LayoutScalar = Scalar> {
    pub(super) child_style: &'a GridItemProjection<'input, S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) lane_axis: GridAxisKind,
    pub(super) stacking_operation: SizingOperation,
    pub(super) containing_block: GridLanesItemContainingBlockOf<S>,
    pub(super) inherits_grid_axis: bool,
}

pub(super) fn measure_lane_axis_margin_box_with_grid_axis<'input, Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    input: LaneAxisMarginBoxMeasureInput<'input, '_, Tree::Scalar>,
    measure: impl FnOnce(
        &mut Tree,
        ComputeInputOf<Tree::Scalar>,
    )
        -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let LaneAxisMarginBoxMeasureInput {
        child_style,
        container_style,
        constants,
        lane_axis,
        stacking_operation,
        containing_block,
        inherits_grid_axis,
    } = input;
    let flow_axes = constants.flow_axes;
    let containing_physical_size = containing_block.physical_size();
    let preferred_size = grid_lanes_child_sizing_preflight(child_style, containing_physical_size)
        .map_err(|error| sizing_resolution_error(child, error))?;
    let logical_preferred_size = flow_axes.logical_size(preferred_size);
    let flex_owns_preferred =
        child_style.flex_preferred_axes.width || child_style.flex_preferred_axes.height;
    let logical_parent = containing_block.logical_size();
    let packing_sizing = if flex_owns_preferred && !inherits_grid_axis {
        let track_extent = match containing_block.grid_axis().logical_axis() {
            LogicalAxis::Inline => LogicalSizeOf::new(logical_parent.inline, None),
            LogicalAxis::Block => LogicalSizeOf::new(None, logical_parent.block),
        };
        Some(
            grid_item_sizing_with_optional_alignment(
                child_style,
                container_style,
                Size::splat(false),
                flow_axes.physical_size(track_extent),
                containing_physical_size,
                flow_axes,
            )
            .map_err(|error| sizing_resolution_error(child, error))?,
        )
    } else {
        None
    };
    let (margin, known, parent, available) = {
        let unresolved_margin = if let Some(sizing) = packing_sizing {
            sizing.unresolved_margin
        } else {
            flow_axes
                .zip_physical_edges_with_inline_extent(
                    child_style.margin,
                    containing_physical_size,
                    resolve_auto_optional,
                )
                .transpose_with_node(tree, child)?
        };
        let margin = unresolved_margin.map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
        let logical_margin = flow_axes.logical_edges(margin);
        let logical_style_size =
            flow_axes.logical_size(Size::new(&child_style.size.width, &child_style.size.height));
        let mut logical_known = LogicalSizeOf::new(None, None);
        let mut logical_available = LogicalSizeOf::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        );
        match lane_axis.logical_axis() {
            LogicalAxis::Inline => {
                logical_available.inline = lane_axis_intrinsic_available(
                    logical_preferred_size.inline,
                    stacking_operation,
                );
            }
            LogicalAxis::Block => {
                logical_available.block =
                    lane_axis_intrinsic_available(logical_preferred_size.block, stacking_operation);
            }
        }
        match containing_block.grid_axis().logical_axis() {
            LogicalAxis::Inline => {
                let grid_axis_size = logical_parent
                    .inline
                    .expect("grid-lanes grid-axis containing extent is definite");
                let available_inline = crate::layout_math::remaining_available_extent(
                    grid_axis_size,
                    logical_margin.inline_sum(),
                )
                .map_err(|status| crate::error::value_resolution_error(child, status))?;
                let justify_self = resolve_grid_item_normal_alignment(
                    child_style.justify_self,
                    container_style.justify_items,
                    child_style.item_is_replaced,
                    logical_style_size.inline.is_auto(),
                    AlignItems::Stretch,
                );
                logical_known.inline = if inherits_grid_axis {
                    Some(available_inline)
                } else if let Some(sizing) = packing_sizing {
                    flow_axes.logical_size(sizing.known).inline
                } else {
                    definite_preferred_size(logical_preferred_size.inline).or_else(|| {
                        (justify_self == AlignItems::Stretch).then_some(available_inline)
                    })
                };
                logical_available.inline = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(available_inline).map_err(
                        |_| {
                            crate::error::value_resolution_error(
                                child,
                                LengthResolutionStatus::InvalidNumeric {
                                    value: available_inline,
                                },
                            )
                        },
                    )?,
                    SizingOperation::Normal,
                );
            }
            LogicalAxis::Block => {
                let grid_axis_size = logical_parent
                    .block
                    .expect("grid-lanes grid-axis containing extent is definite");
                let available_block = crate::layout_math::remaining_available_extent(
                    grid_axis_size,
                    logical_margin.block_sum(),
                )
                .map_err(|status| crate::error::value_resolution_error(child, status))?;
                let align_self = resolve_grid_item_normal_alignment(
                    child_style.align_self,
                    container_style.align_items,
                    child_style.item_is_replaced,
                    logical_style_size.block.is_auto(),
                    AlignItems::Stretch,
                );
                logical_known.block = if inherits_grid_axis {
                    Some(available_block)
                } else if let Some(sizing) = packing_sizing {
                    flow_axes.logical_size(sizing.known).block
                } else {
                    definite_preferred_size(logical_preferred_size.block).or_else(|| {
                        (align_self == AlignItems::Stretch && child_style.aspect_ratio.is_none())
                            .then_some(available_block)
                    })
                };
                logical_available.block = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(available_block).map_err(|_| {
                        crate::error::value_resolution_error(
                            child,
                            LengthResolutionStatus::InvalidNumeric {
                                value: available_block,
                            },
                        )
                    })?,
                    SizingOperation::Normal,
                );
            }
        }
        (
            logical_margin,
            flow_axes.physical_size(logical_known),
            flow_axes.physical_size(logical_parent),
            flow_axes.physical_size(logical_available),
        )
    };
    let output = measure(
        tree,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            known,
            parent,
            crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::Grid),
            SizingRequestOf::new(available, constants.child_orthogonal_fallback),
        ),
    )?;
    Ok(
        lane_axis_size(flow_axes.logical_size(output.size), lane_axis)
            + lane_axis_margin_sum(margin, lane_axis),
    )
}

fn lane_axis_intrinsic_available<S: LayoutScalar>(
    child: ResolvedPreferredSize<S>,
    stacking_operation: SizingOperation,
) -> AxisSizingRequestOf<S> {
    if matches!(child, ResolvedPreferredSize::Auto)
        && stacking_operation == SizingOperation::MinContent
    {
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent)
    } else {
        intrinsic_available_for_dimension(child)
    }
}

fn lane_child_intrinsic_available<S: LayoutScalar>(
    flow_axes: crate::geometry::FlowAxes,
    grid_axis: GridAxisKind,
    preferred_size: Size<ResolvedPreferredSize<S>>,
    grid_axis_available: AxisSizingRequestOf<S>,
) -> Size<AxisSizingRequestOf<S>> {
    let logical_size = flow_axes.logical_size(preferred_size);
    let logical_available = match grid_axis.logical_axis() {
        LogicalAxis::Inline => LogicalSizeOf::new(
            grid_axis_available,
            intrinsic_available_for_dimension(logical_size.block),
        ),
        LogicalAxis::Block => LogicalSizeOf::new(
            intrinsic_available_for_dimension(logical_size.inline),
            grid_axis_available,
        ),
    };
    flow_axes.physical_size(logical_available)
}

fn intrinsic_available_for_dimension<S: LayoutScalar>(
    dimension: ResolvedPreferredSize<S>,
) -> AxisSizingRequestOf<S> {
    match dimension {
        ResolvedPreferredSize::MinContent => {
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent)
        }
        ResolvedPreferredSize::MaxContent
        | ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::Definite(_) => {
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent)
        }
    }
}

fn definite_preferred_size<S: LayoutScalar>(dimension: ResolvedPreferredSize<S>) -> Option<S> {
    match dimension {
        ResolvedPreferredSize::Definite(value) => Some(value),
        ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::MinContent
        | ResolvedPreferredSize::MaxContent => None,
    }
}

fn grid_lanes_child_sizing_preflight<'input, S: LayoutScalar>(
    child_style: &GridItemProjection<'input, S>,
    parent: Size<Option<S>>,
) -> Result<Size<ResolvedPreferredSize<S>>, SizingResolutionError<S>> {
    let preferred_size = Size::new(
        resolve_preferred_sizing(
            &child_style.size.width,
            SizingAlgorithm::GridLanes,
            PhysicalAxis::Horizontal,
            parent.width,
            true,
        )?,
        resolve_preferred_sizing(
            &child_style.size.height,
            SizingAlgorithm::GridLanes,
            PhysicalAxis::Vertical,
            parent.height,
            true,
        )?,
    );
    resolve_minimum_optional(
        &child_style.min_size.width,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Horizontal,
        parent.width,
        true,
    )?;
    resolve_minimum_optional(
        &child_style.min_size.height,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Vertical,
        parent.height,
        true,
    )?;
    resolve_maximum_optional(
        &child_style.max_size.width,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Horizontal,
        parent.width,
        true,
    )?;
    resolve_maximum_optional(
        &child_style.max_size.height,
        SizingAlgorithm::GridLanes,
        PhysicalAxis::Vertical,
        parent.height,
        true,
    )?;
    Ok(preferred_size)
}

fn grid_lanes_item_area<S: LayoutScalar>(
    grid_axis: GridAxisKind,
    start: usize,
    end: usize,
    size: LogicalSizeOf<S>,
) -> GridArea<S> {
    match grid_axis {
        GridAxisKind::Column => GridArea {
            column: start,
            row: 0,
            column_end: end,
            row_end: 1,
            size,
        },
        GridAxisKind::Row => GridArea {
            column: 0,
            row: start,
            column_end: 1,
            row_end: end,
            size,
        },
    }
}

fn lane_grid_axis_facts(
    placement: GridPlacement,
    track_count: usize,
    lines: GridAxisLines,
) -> (Option<usize>, usize) {
    if placement.is_auto() {
        return (None, 1);
    }

    if let Some((start, span)) = definite_axis_start_and_span(
        placement,
        track_count,
        lines.explicit_start,
        lines.explicit_count,
    ) {
        return (Some(start + 1), span);
    }

    (None, placement.span().map(|span| span.get()).unwrap_or(1))
}

#[derive(Clone, Copy)]
struct GridAxisLines {
    explicit_start: usize,
    explicit_count: usize,
}

fn grid_axis_lines(lines: GridLines, axis: GridAxisKind) -> GridAxisLines {
    match axis {
        GridAxisKind::Column => GridAxisLines {
            explicit_start: lines.column_explicit_start,
            explicit_count: lines.column_explicit_count,
        },
        GridAxisKind::Row => GridAxisLines {
            explicit_start: lines.row_explicit_start,
            explicit_count: lines.row_explicit_count,
        },
    }
}

fn resolve_tolerance<S: LayoutScalar>(
    tolerance: GridFlowToleranceOf<S>,
    basis: S,
) -> Result<S, LanePlacementError> {
    let Ok(basis) = PercentageBasisOf::definite(basis) else {
        return Err(LanePlacementError::InvalidGridFlowToleranceBasis);
    };
    let basis_value = basis
        .definite_value()
        .expect("validated definite basis")
        .get();

    match tolerance {
        GridFlowToleranceOf::Normal { font_size } => finite_tolerance(font_size),
        GridFlowToleranceOf::Length(length) => {
            resolve_length_tolerance(length.resolve_against(basis))
        }
        GridFlowToleranceOf::Percent(factor) => finite_tolerance(factor * basis_value),
        GridFlowToleranceOf::Infinite => Ok(S::INFINITY),
    }
}

fn resolve_length_tolerance<S: LayoutScalar>(
    resolution: LengthResolutionOf<S>,
) -> Result<S, LanePlacementError> {
    match resolution.status() {
        LengthResolutionStatus::Resolved => resolution
            .value
            .ok_or(LanePlacementError::InvalidGridFlowToleranceResolution)
            .and_then(finite_tolerance),
        LengthResolutionStatus::MissingBasis
        | LengthResolutionStatus::InvalidNumeric { .. }
        | LengthResolutionStatus::NonNumeric => {
            Err(LanePlacementError::InvalidGridFlowToleranceResolution)
        }
    }
}

fn finite_tolerance<S: LayoutScalar>(value: S) -> Result<S, LanePlacementError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LanePlacementError::InvalidGridFlowToleranceResolution)
    }
}

fn lane_cursor_after(start: usize, span: usize, tracks: usize, reversed: bool) -> usize {
    if reversed {
        (tracks - start) % tracks
    } else {
        (start + span) % tracks
    }
}

fn lane_candidate_order(starts: Range<usize>, reversed: bool) -> impl Iterator<Item = usize> {
    let end = starts.end;
    let start = starts.start;
    starts.map(move |index| {
        if reversed {
            end - 1 - (index - start)
        } else {
            index
        }
    })
}

fn candidate_start_at(cursor: usize, offset: usize, count: usize, reversed: bool) -> usize {
    let cursor = if cursor >= count { 0 } else { cursor };
    let index = (cursor + offset) % count;
    if reversed { count - 1 - index } else { index }
}

fn infinite_candidate_start(
    cursor: usize,
    span: usize,
    collapsed: &[bool],
    reversed: bool,
) -> Option<usize> {
    let max_start = collapsed.len().checked_add(1)?.checked_sub(span)?;
    (0..max_start)
        .map(|offset| candidate_start_at(cursor, offset, max_start, reversed))
        .find(|start| lanes_candidate_is_retained(collapsed, *start, span))
}

fn lanes_candidate_is_retained(collapsed: &[bool], start: usize, span: usize) -> bool {
    collapsed
        .get(start..start.saturating_add(span))
        .is_some_and(|candidate| candidate.iter().all(|track| !*track))
}

fn finite_candidate_start<S: LayoutScalar>(
    running: &[S],
    cursor: usize,
    span: usize,
    tolerance: S,
    collapsed: &[bool],
    reversed: bool,
) -> Option<usize> {
    let track_count = running.len();
    let max_start = track_count.checked_add(1)?.checked_sub(span)?;
    let absolute_shortest = (0..max_start)
        .filter(|start| lanes_candidate_is_retained(collapsed, *start, span))
        .map(|start| max_running_position(running, start, span))
        .fold(S::INFINITY, S::min);
    let tie_limit = absolute_shortest + tolerance.max(S::ZERO);

    for offset in 0..max_start {
        let start = candidate_start_at(cursor, offset, max_start, reversed);
        if lanes_candidate_is_retained(collapsed, start, span)
            && max_running_position(running, start, span) <= tie_limit
        {
            return Some(start);
        }
    }
    None
}

fn max_running_position<S: LayoutScalar>(running: &[S], start: usize, span: usize) -> S {
    running[start..start + span]
        .iter()
        .copied()
        .fold(S::ZERO, S::max)
}

#[cfg(test)]
mod resolved_flow_tests {
    use super::*;
    use crate::GridLanesFlow;

    fn assert_reverse_candidates<S: LayoutScalar>() {
        for tolerance in [
            GridFlowToleranceOf::Length(LengthOf::ZERO),
            GridFlowToleranceOf::Infinite,
        ] {
            let report = place_lanes(LanePlacementInputOf {
                grid_axis_tracks: 3,
                grid_axis: GridAxisKind::Column,
                reversed: true,
                lane_gap: S::ZERO,
                tolerance,
                tolerance_basis: S::ZERO,
                items: [10.0, 20.0, 30.0, 5.0]
                    .into_iter()
                    .enumerate()
                    .map(|(item, height)| LaneItemOf {
                        item,
                        grid_axis_span: 1,
                        definite_grid_axis_start: None,
                        lane_axis_margin_box: S::from_f64(height),
                    })
                    .collect(),
            })
            .unwrap();
            assert_eq!(
                report
                    .item_offsets
                    .iter()
                    .map(|item| item.grid_axis_start)
                    .collect::<Vec<_>>(),
                [3, 2, 1, 3]
            );
            assert_eq!(
                report
                    .item_offsets
                    .iter()
                    .map(|item| item.offset)
                    .collect::<Vec<_>>(),
                [0.0, 0.0, 0.0, 10.0].map(S::from_f64)
            );
        }
        let report = place_lanes(LanePlacementInputOf {
            grid_axis_tracks: 4,
            grid_axis: GridAxisKind::Row,
            reversed: true,
            lane_gap: S::ZERO,
            tolerance: GridFlowToleranceOf::Infinite,
            tolerance_basis: S::ZERO,
            items: [(None, 2), (Some(1), 1), (None, 1), (None, 2)]
                .into_iter()
                .enumerate()
                .map(|(item, (line, span))| LaneItemOf {
                    item,
                    grid_axis_span: span,
                    definite_grid_axis_start: line,
                    lane_axis_margin_box: S::from_f64(10.0),
                })
                .collect(),
        })
        .unwrap();
        assert_eq!(
            report
                .item_offsets
                .iter()
                .map(|item| item.grid_axis_start)
                .collect::<Vec<_>>(),
            [3, 1, 2, 3]
        );
        assert_eq!(
            report
                .item_offsets
                .iter()
                .map(|item| item.offset)
                .collect::<Vec<_>>(),
            [0.0, 0.0, 0.0, 10.0].map(S::from_f64)
        );
        assert_eq!(report.grid_axis, GridAxisKind::Row);
        assert_eq!(report.lane_axis, GridAxisKind::Column);
    }

    #[test]
    fn resolved_flow_reverse_traverses_tracks_and_preserves_explicit_lines() {
        assert_reverse_candidates::<f32>();
        assert_reverse_candidates::<f64>();
    }

    #[test]
    fn resolved_flow_reverse_skips_collapsed_tracks_and_spans() {
        let collapsed = [false, true, false, false];
        assert_eq!(infinite_candidate_start(0, 1, &collapsed, true), Some(3));
        assert_eq!(infinite_candidate_start(1, 1, &collapsed, true), Some(2));
        assert_eq!(infinite_candidate_start(2, 1, &collapsed, true), Some(0));
        assert_eq!(infinite_candidate_start(0, 2, &collapsed, true), Some(2));
        assert_eq!(
            finite_candidate_start(&[0.0f32, 0.0, 5.0, 10.0], 0, 1, 0.0, &collapsed, true),
            Some(0)
        );
        assert_eq!(
            finite_candidate_start(&[0.0f64, 0.0, 5.0, 10.0], 0, 2, 0.0, &collapsed, true),
            Some(2)
        );
    }

    #[test]
    fn resolved_flow_default_is_columns_forward_sparse() {
        let flow = GridLanesFlow::default();
        assert_eq!(flow.grid_axis(), GridAxisKind::Column);
        assert_eq!(flow.stacking_axis(), GridAxisKind::Row);
        assert!(!flow.reversed());
        assert!(!flow.is_dense());
    }
}
