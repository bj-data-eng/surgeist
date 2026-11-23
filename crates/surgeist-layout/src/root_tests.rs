use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf};
use crate::test_support::layout_tree::{OracleMeasurementOf, OracleTreeOf, PublicLayoutTreeOf};
use crate::{
    block::compute_block,
    cache::{Cache, CacheKeyContext, CacheOf},
    engine::{
        compute_hidden, compute_layout, compute_layout_with_flow,
        compute_layout_with_flow_invalidated, compute_root,
        contracts::{CacheAccess, Compute},
        round_layout,
    },
    error::{
        LayoutErrorKind, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSite, LayoutErrorSiteOf,
        LayoutInternalInvariant, LayoutInvalidInput, LayoutInvalidInputOf, LayoutOperation,
        LayoutResultOf, LayoutUnsupportedCapability, NonBoxNodeRoleError, SizingAlgorithm,
        SizingBehavior, SizingProperty,
    },
    geometry::{Edges, FlowAxes, LogicalAxis, PhysicalAxis, PhysicalSide, Point, Size},
    grid::GridAxisKind,
    inline_wrapping::WrappingInput,
    measurement::{LeafMeasureInput, LeafMeasureInputOf, compute_leaf},
    node_input::{
        AlignContent, AlignItems, AtomicInlineParticipation, BoxSizing, Clear, ComputedOverflow,
        Direction, Display, FlexDirection, FlexGrowOf, FlexShrink, FlexShrinkOf, FlexWrap, Float,
        FloatExclusion, FloatExclusionIntervalOf, FloatExclusionQueryOf, GridAutoFlow,
        GridPlacement, InlineBoundaryEdgeOf, InlineBoundaryInput, InlineBoundaryInputOf,
        InlineBoundaryKind, InlineBreakOpportunity, InlineMetrics, InlineMetricsOf,
        ItemAlignmentPosition, LayoutInput, LayoutInputOf, LayoutInputViewOf, LineBreakInput,
        LineBreakInputOf, NodeInput, NodeInputOf, Overflow, OverflowClipBox, OverflowClipMargin,
        OverflowClipMarginOf, Position, ScrollMargin, ScrollMarginOf, ScrollPadding,
        ScrollPaddingOf, ScrollPaddingValue, ScrollPaddingValueOf, ScrollSnapAlign,
        ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType,
        ScrollbarGutter, ScrollbarWidth, ScrollbarWidthOf, TextAlign, TextAlignLast,
        TextJustification, VerticalAlignOf, WritingMode,
    },
    output::{
        BaselinesOf, CompletedLayoutBatch, CompletedLayoutBatchOf, ComputeInput, ComputeInputOf,
        ComputeOutput, ComputeOutputOf, ContainingLayoutContext, FlexItemRootContext,
        FlexItemRootContextOf, InlineFragmentOutputEntryOf, InlineFragmentOutputOf,
        InlineFragmentOwner, LayoutCacheStoreEntryOf, LayoutOutputEntry, LayoutOutputEntryOf,
        LayoutRootContext, LayoutRootContextOf, LayoutRootRequest, LayoutRootRequestOf,
        LayoutRoundingMode, NodeOutput, NodeOutputOf, ParentFormattingContext,
        PositionedRootContextOf, RequestedAxis, RootContextMismatch, RootLocalPositionedContextOf,
        RunMode, SizingMode, SourceIndex,
    },
    scalar::{DefaultScalar, LayoutScalar},
    scroll::{
        PhysicalScrollOffset, PhysicalScrollRangeOf, ScrollGeometry, ScrollGeometryOf, ScrollRect,
        ScrollRectOf, ScrollbarGutterRects,
    },
    sizing::{
        CalcSizeCalculation, FlexBasisOf, MaxSize, MaxSizeOf, MinSize, MinSizeOf, PreferredSize,
        PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation,
    },
    text_layout::{
        LayoutFlowInput, TextControlParticipation, TextParagraphBinding, TextParticipation,
    },
    tree::{LayoutBatchSink, LayoutTree, Traverse},
    value::{
        AspectRatioOf, AvailableSpaceOf, AxisSizingRequestOf, FiniteScalarErrorOf, Length,
        LengthAuto, LengthAutoOf, LengthOf, LengthPercentageOf, NearestScrollportOf,
        NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf, OrthogonalFallbackErrorOf,
        OrthogonalFallbackLimitOf, OrthogonalFallbackOf, SizingOperation, SizingRequestOf,
        SubgridTrack, TrackComponent, TrackComponentOf, TrackFlexFactorOf, TrackRepetition,
        TrackRepetitionOf, TrackSizing,
    },
};
use surgeist_text::supplied_line::BidiLevel;
use surgeist_text::supplied_line::InlineWhitespaceEdge;

#[path = "root_tests/fixtures_tests.rs"]
mod fixtures;

#[path = "root_tests/containing_contexts_tests.rs"]
mod containing_contexts;
#[path = "root_tests/inline_boundary_edges.rs"]
mod inline_boundary_edges;
#[path = "root_tests/measurement_tests.rs"]
mod measurement;
#[path = "root_tests/requests_tests.rs"]
mod requests;
#[path = "root_tests/root_local_positioned.rs"]
mod root_local_positioned;
#[path = "root_tests/rounding_tests.rs"]
mod rounding;
#[path = "root_tests/transaction_cache_tests.rs"]
mod transaction_cache;

#[path = "root_tests/orthogonal_intrinsic_width.rs"]
mod orthogonal_intrinsic_width;
