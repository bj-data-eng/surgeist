use std::collections::{HashMap, HashSet};

use super::lanes::*;
use super::tracks::*;
use super::*;
use crate::geometry::{
    LogicalAxis, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalProgression,
};
use crate::test_support::{
    self as lts,
    layout_tree::{OracleMeasurement, OracleTree, OracleTreeOf, PublicLayoutTreeOf},
};
use crate::{
    cache::{CacheKeyContext, CacheOf},
    engine::{
        compute_layout, compute_layout_invalidated, compute_root, contracts::Compute, round_layout,
    },
    error::{
        CalcSizeBehaviorBasis, LayoutError, LayoutErrorKind, LayoutErrorKindOf, LayoutErrorOf,
        LayoutErrorSite, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutInvalidInput,
        LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation, LayoutResultOf,
        LayoutUnsupportedCapability, SizingAlgorithm, SizingBehavior, SizingProperty,
    },
    flex::compute_flex,
    geometry::{Edges, FlowAxes, Point, Size},
    grid::{GridAxisKind, NamedGridError, compute_grid, compute_grid_with_report},
    measurement::{LeafMeasureInputOf, compute_leaf},
    node_input::{
        AlignContent, AlignItems, AtomicInlineParticipation, BaselineSource, BoxSizing,
        ComputedOverflow, Direction, Display, FlexDirection, FlexShrinkOf, FlexWrap, GridAutoFlow,
        GridBaselineError, GridBaselineReferences, GridFlowTolerance, GridFlowToleranceOf,
        GridLanesFlow, GridPlacement, InlineBaselineContext, InlineBreakOpportunity,
        InlineMetricsOf, ItemOrder, LayoutInputOf, LayoutInputViewOf, LineBreakInputOf, NodeInput,
        NodeInputOf, Overflow, Position, RawGridLine, RawGridPlacement, ScrollMargin,
        ScrollMarginOf, ScrollPaddingOf, ScrollPaddingValueOf, ScrollSnapAlign,
        ScrollSnapAlignValue, ScrollSnapStop, ScrollbarGutter, ScrollbarWidth, ScrollbarWidthOf,
        SizeContainment, WritingMode,
    },
    output::{
        BaselineCoordinateOf, BaselineFactOf, BaselinePairOf, Baselines, BaselinesOf,
        CompletedLayoutBatchOf, ComputeInput, ComputeInputOf, ComputeOutput, ComputeOutputOf,
        ContainingLayoutContext, LayoutCacheClearEntry, LayoutCacheStoreEntryOf,
        LayoutOutputEntryOf, LayoutRootRequestOf, NamedBaselinesOf, NodeOutput, NodeOutputOf,
        ParentFormattingContext, RequestedAxis, RunMode, SizingMode, SourceIndex,
    },
    scalar::{DefaultScalar, LayoutScalar},
    scroll::{ScrollRect, ScrollRectOf},
    sizing::{
        CalcSizeCalculation, MaxSize, MaxSizeCalcBasis, MaxSizeOf, MinSize, MinSizeCalcBasis,
        MinSizeOf, PreferredSize, PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation,
        SizingCalculationOf,
    },
    tree::{LayoutBatchSink, LayoutTree, Traverse},
    value::{
        AspectRatio, AspectRatioOf, AvailableSpaceOf, AxisSizingRequestOf, GridLine, GridSpan,
        GridTemplateAreaRow, GridTemplateAreas, Length, LengthAuto, LengthAutoOf, LengthOf,
        LengthPercentageOf, MaxTrackSizing, MaxTrackSizingOf, MinTrackSizing, MinTrackSizingOf,
        NearestScrollportOf, NonNegativeFiniteOf, OrthogonalFallbackLimitOf, OrthogonalFallbackOf,
        SizingOperation, SizingRequestOf, SubgridLineNameComponent, SubgridLineNameRepeatCount,
        SubgridTrack, TrackComponent, TrackComponentOf, TrackFlexFactor, TrackFlexFactorOf,
        TrackRepeat, TrackRepetition, TrackRepetitionError, TrackRepetitionOf, TrackSizing,
        TrackSizingOf,
    },
};
use surgeist_text::supplied_line::BidiLevel;

use lts::oracle::grid::{
    AlignmentSafety, AutoPlacer, ContributionSize, DefiniteTracks, Flow,
    GridArea as OracleGridArea, GridTrack, ItemContributionFacts, LinePlacement, Track,
    TrackAlignment, TrackSizingSlice, align_tracks_report,
};

#[macro_use]
#[path = "grid_tests/fixtures_tests.rs"]
mod fixtures;

#[path = "grid_tests/browser_controls_tests.rs"]
mod browser_controls;
#[path = "grid_tests/child_baseline_tests.rs"]
mod child_baseline;
#[path = "grid_tests/lane_collapsed_intrinsic_tests.rs"]
mod lane_collapsed_intrinsic;
#[path = "grid_tests/lane_track_sizing_ports_tests.rs"]
mod lane_track_sizing_ports;
#[path = "grid_tests/lanes_subgrid_tests.rs"]
mod lanes_subgrid;
#[path = "grid_tests/minimum_over_maximum_tests.rs"]
mod minimum_over_maximum;
#[path = "grid_tests/named_line_errors_tests.rs"]
mod named_line_errors;
#[path = "grid_tests/oracle_comparison_tests.rs"]
mod oracle_comparison;
#[path = "grid_tests/scroll_composition_tests.rs"]
mod scroll_composition;
#[path = "grid_tests/subgrid_border_basis_tests.rs"]
mod subgrid_border_basis;
#[path = "grid_tests/subgrid_clamping_contracts_tests.rs"]
mod subgrid_clamping_contracts;
#[path = "grid_tests/subgrid_share_tests.rs"]
mod subgrid_share;
#[path = "grid_tests/topology_placement_tests.rs"]
mod topology_placement;
#[path = "grid_tests/tracks_intrinsic_tests.rs"]
mod tracks_intrinsic;

#[path = "grid_tests/baseline_contribution.rs"]
mod baseline_contribution;

#[path = "grid_tests/empty_tracks_absolute.rs"]
mod empty_tracks_absolute;

#[path = "grid_tests/absolute_reference_tests.rs"]
mod absolute_reference;
