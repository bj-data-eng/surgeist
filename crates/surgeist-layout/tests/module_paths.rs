//! Every public item of the crate resolves through its one module path.
//!
//! The imports are the assertion: a missing or misplaced export fails to compile. The crate root holds
//! only the four compute functions; every other item lives in a module. Types whose generic arguments
//! follow from their parameter names are also named in an annotation.
#![expect(unused_imports, reason = "the imports are the assertion")]

use surgeist_layout::{
    compute_layout, compute_layout_invalidated, compute_layout_with_flow,
    compute_layout_with_flow_invalidated,
};

use surgeist_layout::baseline::{
    BaselineCoordinateOf, BaselineFactOf, BaselineOffset, BaselineOffsetOf,
    BaselineOffsetResolutionError, BaselinePairOf, BaselineSource, Baselines, BaselinesOf,
    GridBaselineError, GridBaselineReference, GridBaselineReferences, InlineBaselineContext,
    InlineBaselineError, NamedBaselinesOf,
};
use surgeist_layout::batch::{
    CompletedLayoutBatch, CompletedLayoutBatchOf, InlineFragmentOutputEntry,
    InlineFragmentOutputEntryOf, LayoutBatchApplyError, LayoutBatchCommit, LayoutBatchPreparation,
    LayoutCacheClearEntry, LayoutCacheStoreEntry, LayoutCacheStoreEntryOf, LayoutOutputEntry,
    LayoutOutputEntryOf, LayoutPublicationTargets, LayoutSourceBasis, LayoutTargetBasis,
};
use surgeist_layout::box_model::{
    AlignContent, AlignItems, BlockFlowContext, BlockGeneration, BoxSizing, Clear, Display, Float,
    FloatExclusion, FloatExclusionInterval, FloatExclusionIntervalError,
    FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf, FloatExclusionQuery,
    FloatExclusionQueryOf, ItemAlignmentPosition, Position, SizeContainment,
};
use surgeist_layout::cache::{
    Cache, CacheKeyContext, CacheOf, ClearState, ComputeInput, ComputeInputOf, ComputeOutput,
    ComputeOutputOf,
};
use surgeist_layout::constraint::{
    AvailableSpace, AvailableSpaceOf, AxisSizingRequest, AxisSizingRequestOf, ContentSizingRequest,
    ContentSizingRequestOf, NearestScrollport, NearestScrollportOf, OrthogonalFallback,
    OrthogonalFallbackError, OrthogonalFallbackErrorOf, OrthogonalFallbackLimit,
    OrthogonalFallbackLimitOf, OrthogonalFallbackOf, ScrollportFallbackChange,
    ScrollportFallbackChangeOf, SizingOperation, SizingRequest, SizingRequestOf,
};
use surgeist_layout::error::{
    AtomicInlineParticipationRoleError, CalcSizeBehaviorBasis, FloatExclusionRoleError,
    InvalidMeasurementOutput, InvalidMeasurementOutputOf, LayoutError, LayoutErrorKind,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSite, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutInvalidInput, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation, LayoutResult,
    LayoutResultOf, LayoutScrollGeometryError, LayoutScrollGeometryErrorOf, LayoutTextNumericError,
    LayoutUnsupportedCapability, NonBoxNodeRoleError, SizingAlgorithm, SizingBehavior,
    SizingProperty, UnsupportedSizingBehavior,
};
use surgeist_layout::flex::{
    FlexDirection, FlexGrow, FlexGrowOf, FlexItemCollapse, FlexShrink, FlexShrinkOf, FlexWrap,
};
use surgeist_layout::fragmentation::{
    BlockFragmentainerOf, BlockFragmentationFailure, BoxFragmentGeometryOf, BoxGeometryPairOf,
    FragmentCoordinateSpace, FragmentOutputOf, FragmentPayloadOf, FragmentationBreak,
    FragmentationBreakInside, FragmentationBreakKinds, FragmentationGraphicalCoverageOf,
    FragmentationInputError, FragmentationKind, FragmentationLogicalPageSide,
    FragmentationPageSide, FragmentationPhysicalPageSide, LayoutBoundRequestOf, LayoutFragmentId,
    LayoutFragmentainerId, LayoutFragmentainerOutputOf, LayoutHandleError, NodeLayoutRefOf,
    NonBoxGeometryOf, NonBoxKind, NonBoxOutputRefOf, OverflowFragmentGeometryOf,
    OverflowGeometryPairOf, PageProgression, SingleBoxOutputRefOf,
};
use surgeist_layout::geometry::{
    Direction, Edges, FlowAxes, LogicalAxis, PhysicalAxis, PhysicalSide, Point, Size, WritingMode,
};
use surgeist_layout::grid::{
    GridAutoFlow, GridAxisKind, GridFlowTolerance, GridFlowToleranceOf, GridLanesFlow, GridLine,
    GridPlacement, GridSpan, GridTemplateAreaRow, GridTemplateAreas, MaxTrackSizing,
    MaxTrackSizingOf, MinTrackSizing, MinTrackSizingOf, NamedGridError, RawGridLine,
    RawGridPlacement, SubgridLineNameComponent, SubgridLineNameRepeatCount, SubgridTrack,
    TrackComponent, TrackComponentList, TrackComponentListOf, TrackComponentOf, TrackFlexFactor,
    TrackFlexFactorOf, TrackRepeat, TrackRepeatCount, TrackRepetition, TrackRepetitionError,
    TrackRepetitionOf, TrackSizing, TrackSizingOf, track_sizing_components,
    track_sizing_components_of,
};
use surgeist_layout::inline::{
    AtomicInlineParticipation, InlineBoundaryEdge, InlineBoundaryEdgeError,
    InlineBoundaryEdgeErrorOf, InlineBoundaryEdgeOf, InlineBoundaryInput, InlineBoundaryInputOf,
    InlineBoundaryKind, InlineMetrics, InlineMetricsError, InlineMetricsOf, VerticalAlign,
    VerticalAlignOf,
};
use surgeist_layout::inline_wrapping::{
    ChildOrderCompleteness, GeneratedBoxParticipation, GeneratedBoxRef, GeneratedChildBoxInput,
    InlineBoxMetricContribution, InlineBoxRef, InlineChildOrderInput, InlineDelimiterSide,
    InlineFormatterEdge, InlineGapAffinity, InlineGapAnchor, InlineGapGroupInput,
    NativeInlineBoxEdgeView, NativeInlineBoxFamily, NativeInlineBoxFamilyInput,
    NativeInlineBoxFragment, NativeInlineBoxMemberView, NativeInlineBoxPieceView,
    ProcessedWrappingSource, ResolvedWrapEdges, SourceOccurrenceAssociations,
    SourceScopeAssociation, SourceWrappingScopeInput, WrapEdge, WrapEdgeSource,
    WrapEdgeSourceNodeView, WrapEdges, WrapInside, WrappingInput, WrappingScopeError,
    WrappingScopeInput, process_wrapping_source,
};
use surgeist_layout::length::{
    AspectRatio, AspectRatioOf, FiniteScalarErrorOf, Length, LengthAuto, LengthAutoOf, LengthOf,
    LengthPercentageErrorOf, LengthPercentageOf, LengthResolution, LengthResolutionOf,
    LengthResolutionStatus, NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf,
    NumericResolutionOf, PercentageBasisOf, ResolvedLengthAuto, ResolvedLengthAutoOf,
    UnresolvedLengthReason,
};
use surgeist_layout::node::{
    CollapsibleMargin, CollapsibleMarginOf, ItemOrder, LayoutInput, LayoutInputOf, LayoutInputView,
    LayoutInputViewOf, NodeInput, NodeInputOf, NodeOutput, NodeOutputOf,
    PhysicalBlockMarginCollapse, PhysicalBlockMarginCollapseOf, SourceIndex,
};
use surgeist_layout::overflow::{
    ComputedOverflow, ComputedOverflowError, Overflow, OverflowClipBox, OverflowClipMargin,
    OverflowClipMarginOf, ScrollMargin, ScrollMarginError, ScrollMarginErrorOf, ScrollMarginOf,
    ScrollPadding, ScrollPaddingOf, ScrollPaddingValue, ScrollPaddingValueOf, ScrollSnapAlign,
    ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType,
    ScrollbarGutter, ScrollbarWidth, ScrollbarWidthOf,
};
use surgeist_layout::request::{
    BlockFlowRootContext, BlockFlowRootContextOf, ContainingLayoutContext, FlexItemRootContext,
    FlexItemRootContextOf, LayoutRootContext, LayoutRootContextOf, LayoutRootRequest,
    LayoutRootRequestOf, LayoutRoundingMode, LayoutSizeInput, LayoutSizeInputError,
    LayoutSizeInputErrorOf, ParentFormattingContext, PositionedRootContext,
    PositionedRootContextOf, RootContextMismatch, RootLocalPositionedContext,
    RootLocalPositionedContextOf,
};
use surgeist_layout::resize_observer::{
    ResizeObserverBoxSize, ResizeObserverContentRectOf, ResizeObserverMeasurementError,
    ResizeObserverMeasurementOf, ResizeObserverPixelGrid,
};
use surgeist_layout::scalar::{DefaultScalar, LayoutScalar};
use surgeist_layout::scroll::anchor_selection::{
    ScrollAnchorBox, ScrollAnchorBoxOf, ScrollAnchorContainingBlock, ScrollAnchorNode,
    ScrollAnchorNodeOf, ScrollAnchorPolicy, ScrollAnchorPosition, ScrollAnchorPriority,
    ScrollAnchorRequest, ScrollAnchorRequestOf, ScrollAnchorSelectionError, ScrollAnchorSubject,
    ScrollAnchorSubjectOf, select_scroll_anchor,
};
use surgeist_layout::scroll::{
    FlowRelativeScrollAxisRange, FlowRelativeScrollAxisRangeOf, FlowRelativeScrollOffset,
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRange, FlowRelativeScrollRangeOf, OverflowClip,
    OverflowClipOf, PhysicalClipAxis, PhysicalClipAxisOf, PhysicalScrollAxisRange,
    PhysicalScrollAxisRangeOf, PhysicalScrollOffset, PhysicalScrollOffsetOf, PhysicalScrollRange,
    PhysicalScrollRangeOf, ScrollCoordinateError, ScrollCoordinateErrorOf, ScrollGeometry,
    ScrollGeometryOf, ScrollRect, ScrollRectError, ScrollRectErrorOf, ScrollRectOf,
    ScrollSnapContext, ScrollSnapContextOf, ScrollSnapGeometry, ScrollSnapGeometryOf,
    ScrollTargetGeometry, ScrollTargetGeometryOf, ScrollbarGutterRects, ScrollbarGutterRectsOf,
};
use surgeist_layout::sizing::{
    CalcSizeCalculation, CalcSizeCalculationErrorOf, CalcSizeCalculationOf,
    CalcSizeConstructionError, FlexBasis, FlexBasisCalcBasis, FlexBasisOf, MaxSize,
    MaxSizeCalcBasis, MaxSizeOf, MinSize, MinSizeCalcBasis, MinSizeOf, PreferredSize,
    PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation, SizingCalculationError,
    SizingCalculationOf,
};
use surgeist_layout::text_flow::{
    HyphenateLimitLast, HyphenateLimitLines, InlineBreakOpportunity, LineBoxDirectionPolicy,
    LineBreakDisplay, LineBreakInput, LineBreakInputOf, TextAlign, TextAlignLast, TextGroupAlign,
    TextIndent, TextIndentOf, TextJustification, TextWrapStyle,
};
use surgeist_layout::text_layout::{
    InlineBoxFragmentationDemand, InlineFragmentOutput, InlineFragmentOutputOf,
    InlineFragmentOwner, LayoutFlowInput, PlacedInlineBoxFragments, PlacedTextOutput,
    TextAtomicBinding, TextBindingError, TextControlBinding, TextControlParticipation,
    TextOriginBinding, TextParagraphBinding, TextParticipation, TextScopeAlignmentInput,
    TextScopeAlignmentInputOf, TextScopeAssociation, TextScopeBinding,
};
use surgeist_layout::tree::{
    LayoutBatchSink, LayoutTree, LeafMeasureInput, LeafMeasureInputOf, Traverse, compute_leaf,
    measure_definite_inline_atomic_size,
};

// Each module itself.
use surgeist_layout::scroll::anchor_selection;
use surgeist_layout::{
    baseline, batch, box_model, cache, constraint, error, flex, fragmentation, geometry, grid,
    inline, inline_wrapping, length, node, overflow, request, resize_observer, scalar, scroll,
    sizing, text_flow, text_layout, tree,
};

#[test]
fn every_type_is_named_through_its_module() {
    let _: Option<baseline::BaselineCoordinateOf<f32>> = None;
    let _: Option<baseline::BaselineFactOf<f32>> = None;
    let _: Option<baseline::BaselineOffset> = None;
    let _: Option<baseline::BaselineOffsetOf<f32>> = None;
    let _: Option<baseline::BaselineOffsetResolutionError> = None;
    let _: Option<baseline::BaselinePairOf<f32>> = None;
    let _: Option<baseline::BaselineSource> = None;
    let _: Option<baseline::Baselines> = None;
    let _: Option<baseline::BaselinesOf<f32>> = None;
    let _: Option<baseline::GridBaselineError> = None;
    let _: Option<baseline::GridBaselineReference> = None;
    let _: Option<baseline::GridBaselineReferences> = None;
    let _: Option<baseline::InlineBaselineContext> = None;
    let _: Option<baseline::InlineBaselineError> = None;
    let _: Option<baseline::NamedBaselinesOf<f32>> = None;
    let _: Option<batch::CompletedLayoutBatch<usize>> = None;
    let _: Option<batch::CompletedLayoutBatchOf<usize, f32>> = None;
    let _: Option<batch::InlineFragmentOutputEntry<usize>> = None;
    let _: Option<batch::InlineFragmentOutputEntryOf<usize, f32>> = None;
    let _: Option<batch::LayoutBatchApplyError<core::convert::Infallible>> = None;
    let _: Option<batch::LayoutBatchPreparation<'static, usize, f32>> = None;
    let _: Option<batch::LayoutCacheClearEntry<usize>> = None;
    let _: Option<batch::LayoutCacheStoreEntry<usize>> = None;
    let _: Option<batch::LayoutCacheStoreEntryOf<usize, f32>> = None;
    let _: Option<batch::LayoutOutputEntry<usize>> = None;
    let _: Option<batch::LayoutOutputEntryOf<usize, f32>> = None;
    let _: Option<batch::LayoutPublicationTargets<usize>> = None;
    let _: Option<batch::LayoutSourceBasis> = None;
    let _: Option<batch::LayoutTargetBasis> = None;
    let _: Option<box_model::AlignContent> = None;
    let _: Option<box_model::AlignItems> = None;
    let _: Option<box_model::BlockFlowContext> = None;
    let _: Option<box_model::BlockGeneration> = None;
    let _: Option<box_model::BoxSizing> = None;
    let _: Option<box_model::Clear> = None;
    let _: Option<box_model::Display> = None;
    let _: Option<box_model::Float> = None;
    let _: Option<box_model::FloatExclusion> = None;
    let _: Option<box_model::FloatExclusionInterval> = None;
    let _: Option<box_model::FloatExclusionIntervalError> = None;
    let _: Option<box_model::FloatExclusionIntervalErrorOf<f32>> = None;
    let _: Option<box_model::FloatExclusionIntervalOf<f32>> = None;
    let _: Option<box_model::FloatExclusionQuery> = None;
    let _: Option<box_model::FloatExclusionQueryOf<f32>> = None;
    let _: Option<box_model::ItemAlignmentPosition> = None;
    let _: Option<box_model::Position> = None;
    let _: Option<box_model::SizeContainment> = None;
    let _: Option<cache::Cache> = None;
    let _: Option<cache::CacheKeyContext> = None;
    let _: Option<cache::CacheOf<f32>> = None;
    let _: Option<cache::ClearState> = None;
    let _: Option<cache::ComputeInput> = None;
    let _: Option<cache::ComputeInputOf<f32>> = None;
    let _: Option<cache::ComputeOutput> = None;
    let _: Option<cache::ComputeOutputOf<f32>> = None;
    let _: Option<constraint::AvailableSpace> = None;
    let _: Option<constraint::AvailableSpaceOf<f32>> = None;
    let _: Option<constraint::AxisSizingRequest> = None;
    let _: Option<constraint::AxisSizingRequestOf<f32>> = None;
    let _: Option<constraint::ContentSizingRequest> = None;
    let _: Option<constraint::ContentSizingRequestOf<f32>> = None;
    let _: Option<constraint::NearestScrollport> = None;
    let _: Option<constraint::NearestScrollportOf<f32>> = None;
    let _: Option<constraint::OrthogonalFallback> = None;
    let _: Option<constraint::OrthogonalFallbackError> = None;
    let _: Option<constraint::OrthogonalFallbackErrorOf<f32>> = None;
    let _: Option<constraint::OrthogonalFallbackLimit> = None;
    let _: Option<constraint::OrthogonalFallbackLimitOf<f32>> = None;
    let _: Option<constraint::OrthogonalFallbackOf<f32>> = None;
    let _: Option<constraint::ScrollportFallbackChange> = None;
    let _: Option<constraint::ScrollportFallbackChangeOf<f32>> = None;
    let _: Option<constraint::SizingOperation> = None;
    let _: Option<constraint::SizingRequest> = None;
    let _: Option<constraint::SizingRequestOf<f32>> = None;
    let _: Option<error::AtomicInlineParticipationRoleError> = None;
    let _: Option<error::CalcSizeBehaviorBasis> = None;
    let _: Option<error::FloatExclusionRoleError> = None;
    let _: Option<error::InvalidMeasurementOutput> = None;
    let _: Option<error::InvalidMeasurementOutputOf<f32>> = None;
    let _: Option<error::LayoutError<usize, core::convert::Infallible>> = None;
    let _: Option<error::LayoutErrorKind<core::convert::Infallible>> = None;
    let _: Option<error::LayoutErrorKindOf<f32, core::convert::Infallible>> = None;
    let _: Option<error::LayoutErrorOf<usize, f32, core::convert::Infallible>> = None;
    let _: Option<error::LayoutErrorSite<usize>> = None;
    let _: Option<error::LayoutErrorSiteOf<usize>> = None;
    let _: Option<error::LayoutInternalInvariant> = None;
    let _: Option<error::LayoutInvalidInput> = None;
    let _: Option<error::LayoutInvalidInputOf<f32>> = None;
    let _: Option<error::LayoutMissingContext> = None;
    let _: Option<error::LayoutOperation> = None;
    let _: Option<error::LayoutResult<usize, f32, core::convert::Infallible>> = None;
    let _: Option<error::LayoutResultOf<usize, f32, f32, core::convert::Infallible>> = None;
    let _: Option<error::LayoutScrollGeometryError> = None;
    let _: Option<error::LayoutScrollGeometryErrorOf<f32>> = None;
    let _: Option<error::LayoutTextNumericError> = None;
    let _: Option<error::LayoutUnsupportedCapability> = None;
    let _: Option<error::NonBoxNodeRoleError> = None;
    let _: Option<error::SizingAlgorithm> = None;
    let _: Option<error::SizingBehavior> = None;
    let _: Option<error::SizingProperty> = None;
    let _: Option<error::UnsupportedSizingBehavior> = None;
    let _: Option<flex::FlexDirection> = None;
    let _: Option<flex::FlexGrow> = None;
    let _: Option<flex::FlexGrowOf<f32>> = None;
    let _: Option<flex::FlexItemCollapse> = None;
    let _: Option<flex::FlexShrink> = None;
    let _: Option<flex::FlexShrinkOf<f32>> = None;
    let _: Option<flex::FlexWrap> = None;
    let _: Option<fragmentation::BlockFragmentainerOf<f32>> = None;
    let _: Option<fragmentation::BlockFragmentationFailure> = None;
    let _: Option<fragmentation::BoxFragmentGeometryOf<f32>> = None;
    let _: Option<fragmentation::BoxGeometryPairOf<usize, f32>> = None;
    let _: Option<fragmentation::FragmentCoordinateSpace> = None;
    let _: Option<fragmentation::FragmentOutputOf<usize, f32>> = None;
    let _: Option<fragmentation::FragmentPayloadOf<usize, f32>> = None;
    let _: Option<fragmentation::FragmentationBreak> = None;
    let _: Option<fragmentation::FragmentationBreakInside> = None;
    let _: Option<fragmentation::FragmentationBreakKinds> = None;
    let _: Option<fragmentation::FragmentationGraphicalCoverageOf<f32>> = None;
    let _: Option<fragmentation::FragmentationInputError> = None;
    let _: Option<fragmentation::FragmentationKind> = None;
    let _: Option<fragmentation::FragmentationLogicalPageSide> = None;
    let _: Option<fragmentation::FragmentationPageSide> = None;
    let _: Option<fragmentation::FragmentationPhysicalPageSide> = None;
    let _: Option<fragmentation::LayoutBoundRequestOf<f32>> = None;
    let _: Option<fragmentation::LayoutFragmentId> = None;
    let _: Option<fragmentation::LayoutFragmentainerId> = None;
    let _: Option<fragmentation::LayoutFragmentainerOutputOf<f32>> = None;
    let _: Option<fragmentation::LayoutHandleError> = None;
    let _: Option<fragmentation::NodeLayoutRefOf<'static, usize, f32>> = None;
    let _: Option<fragmentation::NonBoxGeometryOf<f32>> = None;
    let _: Option<fragmentation::NonBoxKind> = None;
    let _: Option<fragmentation::NonBoxOutputRefOf<'static, usize, f32>> = None;
    let _: Option<fragmentation::OverflowFragmentGeometryOf<f32>> = None;
    let _: Option<fragmentation::OverflowGeometryPairOf<f32>> = None;
    let _: Option<fragmentation::PageProgression> = None;
    let _: Option<fragmentation::SingleBoxOutputRefOf<'static, usize, f32>> = None;
    let _: Option<geometry::Direction> = None;
    let _: Option<geometry::Edges<f32>> = None;
    let _: Option<geometry::FlowAxes> = None;
    let _: Option<geometry::LogicalAxis> = None;
    let _: Option<geometry::PhysicalAxis> = None;
    let _: Option<geometry::PhysicalSide> = None;
    let _: Option<geometry::Point<f32>> = None;
    let _: Option<geometry::Size<f32>> = None;
    let _: Option<geometry::WritingMode> = None;
    let _: Option<grid::GridAutoFlow> = None;
    let _: Option<grid::GridAxisKind> = None;
    let _: Option<grid::GridFlowTolerance> = None;
    let _: Option<grid::GridFlowToleranceOf<f32>> = None;
    let _: Option<grid::GridLanesFlow> = None;
    let _: Option<grid::GridLine> = None;
    let _: Option<grid::GridPlacement> = None;
    let _: Option<grid::GridSpan> = None;
    let _: Option<grid::GridTemplateAreaRow> = None;
    let _: Option<grid::GridTemplateAreas> = None;
    let _: Option<grid::MaxTrackSizing> = None;
    let _: Option<grid::MaxTrackSizingOf<f32>> = None;
    let _: Option<grid::MinTrackSizing> = None;
    let _: Option<grid::MinTrackSizingOf<f32>> = None;
    let _: Option<grid::NamedGridError> = None;
    let _: Option<grid::RawGridLine> = None;
    let _: Option<grid::RawGridPlacement> = None;
    let _: Option<grid::SubgridLineNameComponent> = None;
    let _: Option<grid::SubgridLineNameRepeatCount> = None;
    let _: Option<grid::SubgridTrack> = None;
    let _: Option<grid::TrackComponent> = None;
    let _: Option<grid::TrackComponentList> = None;
    let _: Option<grid::TrackComponentListOf<f32>> = None;
    let _: Option<grid::TrackComponentOf<f32>> = None;
    let _: Option<grid::TrackFlexFactor> = None;
    let _: Option<grid::TrackFlexFactorOf<f32>> = None;
    let _: Option<grid::TrackRepeat> = None;
    let _: Option<grid::TrackRepeatCount> = None;
    let _: Option<grid::TrackRepetition> = None;
    let _: Option<grid::TrackRepetitionError> = None;
    let _: Option<grid::TrackRepetitionOf<f32>> = None;
    let _: Option<grid::TrackSizing> = None;
    let _: Option<grid::TrackSizingOf<f32>> = None;
    let _: Option<inline::AtomicInlineParticipation> = None;
    let _: Option<inline::InlineBoundaryEdge> = None;
    let _: Option<inline::InlineBoundaryEdgeError> = None;
    let _: Option<inline::InlineBoundaryEdgeErrorOf<f32>> = None;
    let _: Option<inline::InlineBoundaryEdgeOf<f32>> = None;
    let _: Option<inline::InlineBoundaryInput> = None;
    let _: Option<inline::InlineBoundaryInputOf<f32>> = None;
    let _: Option<inline::InlineBoundaryKind> = None;
    let _: Option<inline::InlineMetrics> = None;
    let _: Option<inline::InlineMetricsError<f32>> = None;
    let _: Option<inline::InlineMetricsOf<f32>> = None;
    let _: Option<inline::VerticalAlign> = None;
    let _: Option<inline::VerticalAlignOf<f32>> = None;
    let _: Option<inline_wrapping::ChildOrderCompleteness> = None;
    let _: Option<inline_wrapping::GeneratedBoxParticipation> = None;
    let _: Option<inline_wrapping::InlineBoxMetricContribution> = None;
    let _: Option<inline_wrapping::InlineDelimiterSide> = None;
    let _: Option<inline_wrapping::InlineFormatterEdge> = None;
    let _: Option<inline_wrapping::InlineGapAffinity> = None;
    let _: Option<inline_wrapping::InlineGapAnchor<usize>> = None;
    let _: Option<inline_wrapping::InlineGapGroupInput<usize>> = None;
    let _: Option<inline_wrapping::NativeInlineBoxEdgeView<'static, usize, f32>> = None;
    let _: Option<inline_wrapping::NativeInlineBoxFamily<usize>> = None;
    let _: Option<inline_wrapping::NativeInlineBoxFamilyInput<usize>> = None;
    let _: Option<inline_wrapping::NativeInlineBoxFragment<usize>> = None;
    let _: Option<inline_wrapping::NativeInlineBoxMemberView<'static, usize>> = None;
    let _: Option<inline_wrapping::NativeInlineBoxPieceView<'static, usize, f32>> = None;
    let _: Option<inline_wrapping::ProcessedWrappingSource<usize>> = None;
    let _: Option<inline_wrapping::ResolvedWrapEdges<usize>> = None;
    let _: Option<inline_wrapping::SourceOccurrenceAssociations<usize>> = None;
    let _: Option<inline_wrapping::SourceScopeAssociation<usize>> = None;
    let _: Option<inline_wrapping::SourceWrappingScopeInput<usize>> = None;
    let _: Option<inline_wrapping::WrapEdge> = None;
    let _: Option<inline_wrapping::WrapEdgeSourceNodeView<'static, f32>> = None;
    let _: Option<inline_wrapping::WrapEdges> = None;
    let _: Option<inline_wrapping::WrapInside> = None;
    let _: Option<inline_wrapping::WrappingInput<usize>> = None;
    let _: Option<inline_wrapping::WrappingScopeError> = None;
    let _: Option<inline_wrapping::WrappingScopeInput<usize>> = None;
    let _: Option<length::AspectRatio> = None;
    let _: Option<length::AspectRatioOf<f32>> = None;
    let _: Option<length::FiniteScalarErrorOf<f32>> = None;
    let _: Option<length::Length> = None;
    let _: Option<length::LengthAuto> = None;
    let _: Option<length::LengthAutoOf<f32>> = None;
    let _: Option<length::LengthOf<f32>> = None;
    let _: Option<length::LengthPercentageErrorOf<f32>> = None;
    let _: Option<length::LengthPercentageOf<f32>> = None;
    let _: Option<length::LengthResolution> = None;
    let _: Option<length::LengthResolutionOf<f32>> = None;
    let _: Option<length::LengthResolutionStatus<f32>> = None;
    let _: Option<length::NonNegativeFiniteOf<f32>> = None;
    let _: Option<length::NonNegativeFiniteScalarErrorOf<f32>> = None;
    let _: Option<length::NumericResolutionOf<f32>> = None;
    let _: Option<length::PercentageBasisOf<f32>> = None;
    let _: Option<length::ResolvedLengthAuto> = None;
    let _: Option<length::ResolvedLengthAutoOf<f32>> = None;
    let _: Option<length::UnresolvedLengthReason> = None;
    let _: Option<node::CollapsibleMargin> = None;
    let _: Option<node::CollapsibleMarginOf<f32>> = None;
    let _: Option<node::ItemOrder> = None;
    let _: Option<node::LayoutInput> = None;
    let _: Option<node::LayoutInputOf<f32>> = None;
    let _: Option<node::LayoutInputView<'static>> = None;
    let _: Option<node::LayoutInputViewOf<'static, f32>> = None;
    let _: Option<node::NodeInput> = None;
    let _: Option<node::NodeInputOf<f32>> = None;
    let _: Option<node::NodeOutput> = None;
    let _: Option<node::NodeOutputOf<f32>> = None;
    let _: Option<node::PhysicalBlockMarginCollapse> = None;
    let _: Option<node::PhysicalBlockMarginCollapseOf<f32>> = None;
    let _: Option<node::SourceIndex> = None;
    let _: Option<overflow::ComputedOverflow> = None;
    let _: Option<overflow::ComputedOverflowError> = None;
    let _: Option<overflow::Overflow> = None;
    let _: Option<overflow::OverflowClipBox> = None;
    let _: Option<overflow::OverflowClipMargin> = None;
    let _: Option<overflow::OverflowClipMarginOf<f32>> = None;
    let _: Option<overflow::ScrollMargin> = None;
    let _: Option<overflow::ScrollMarginError> = None;
    let _: Option<overflow::ScrollMarginErrorOf<f32>> = None;
    let _: Option<overflow::ScrollMarginOf<f32>> = None;
    let _: Option<overflow::ScrollPadding> = None;
    let _: Option<overflow::ScrollPaddingOf<f32>> = None;
    let _: Option<overflow::ScrollPaddingValue> = None;
    let _: Option<overflow::ScrollPaddingValueOf<f32>> = None;
    let _: Option<overflow::ScrollSnapAlign> = None;
    let _: Option<overflow::ScrollSnapAlignValue> = None;
    let _: Option<overflow::ScrollSnapAxis> = None;
    let _: Option<overflow::ScrollSnapStop> = None;
    let _: Option<overflow::ScrollSnapStrictness> = None;
    let _: Option<overflow::ScrollSnapType> = None;
    let _: Option<overflow::ScrollbarGutter> = None;
    let _: Option<overflow::ScrollbarWidth> = None;
    let _: Option<overflow::ScrollbarWidthOf<f32>> = None;
    let _: Option<request::BlockFlowRootContext> = None;
    let _: Option<request::BlockFlowRootContextOf<f32>> = None;
    let _: Option<request::ContainingLayoutContext> = None;
    let _: Option<request::FlexItemRootContext> = None;
    let _: Option<request::FlexItemRootContextOf<f32>> = None;
    let _: Option<request::LayoutRootContext> = None;
    let _: Option<request::LayoutRootContextOf<f32>> = None;
    let _: Option<request::LayoutRootRequest> = None;
    let _: Option<request::LayoutRootRequestOf<f32>> = None;
    let _: Option<request::LayoutRoundingMode> = None;
    let _: Option<request::LayoutSizeInput> = None;
    let _: Option<request::LayoutSizeInputError> = None;
    let _: Option<request::LayoutSizeInputErrorOf<f32>> = None;
    let _: Option<request::ParentFormattingContext> = None;
    let _: Option<request::PositionedRootContext> = None;
    let _: Option<request::PositionedRootContextOf<f32>> = None;
    let _: Option<request::RootContextMismatch> = None;
    let _: Option<request::RootLocalPositionedContext> = None;
    let _: Option<request::RootLocalPositionedContextOf<f32>> = None;
    let _: Option<resize_observer::ResizeObserverBoxSize<f32>> = None;
    let _: Option<resize_observer::ResizeObserverContentRectOf<f32>> = None;
    let _: Option<resize_observer::ResizeObserverMeasurementError> = None;
    let _: Option<resize_observer::ResizeObserverMeasurementOf<f32>> = None;
    let _: Option<resize_observer::ResizeObserverPixelGrid> = None;
    let _: Option<scalar::DefaultScalar> = None;
    let _: Option<scroll::FlowRelativeScrollAxisRange> = None;
    let _: Option<scroll::FlowRelativeScrollAxisRangeOf<f32>> = None;
    let _: Option<scroll::FlowRelativeScrollOffset> = None;
    let _: Option<scroll::FlowRelativeScrollOffsetOf<f32>> = None;
    let _: Option<scroll::FlowRelativeScrollRange> = None;
    let _: Option<scroll::FlowRelativeScrollRangeOf<f32>> = None;
    let _: Option<scroll::OverflowClip> = None;
    let _: Option<scroll::OverflowClipOf<f32>> = None;
    let _: Option<scroll::PhysicalClipAxis> = None;
    let _: Option<scroll::PhysicalClipAxisOf<f32>> = None;
    let _: Option<scroll::PhysicalScrollAxisRange> = None;
    let _: Option<scroll::PhysicalScrollAxisRangeOf<f32>> = None;
    let _: Option<scroll::PhysicalScrollOffset> = None;
    let _: Option<scroll::PhysicalScrollOffsetOf<f32>> = None;
    let _: Option<scroll::PhysicalScrollRange> = None;
    let _: Option<scroll::PhysicalScrollRangeOf<f32>> = None;
    let _: Option<scroll::ScrollCoordinateError> = None;
    let _: Option<scroll::ScrollCoordinateErrorOf<f32>> = None;
    let _: Option<scroll::ScrollGeometry> = None;
    let _: Option<scroll::ScrollGeometryOf<f32>> = None;
    let _: Option<scroll::ScrollRect> = None;
    let _: Option<scroll::ScrollRectError> = None;
    let _: Option<scroll::ScrollRectErrorOf<f32>> = None;
    let _: Option<scroll::ScrollRectOf<f32>> = None;
    let _: Option<scroll::ScrollSnapContext> = None;
    let _: Option<scroll::ScrollSnapContextOf<f32>> = None;
    let _: Option<scroll::ScrollSnapGeometry> = None;
    let _: Option<scroll::ScrollSnapGeometryOf<f32>> = None;
    let _: Option<scroll::ScrollTargetGeometry> = None;
    let _: Option<scroll::ScrollTargetGeometryOf<f32>> = None;
    let _: Option<scroll::ScrollbarGutterRects> = None;
    let _: Option<scroll::ScrollbarGutterRectsOf<f32>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorBox> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorBoxOf<f32>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorContainingBlock<usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorNode<'static, usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorNodeOf<'static, usize, f32>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorPolicy> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorPosition<usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorPriority<usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorRequest<usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorRequestOf<usize, f32>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorSelectionError<usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorSubject<usize>> = None;
    let _: Option<scroll::anchor_selection::ScrollAnchorSubjectOf<usize, f32>> = None;
    let _: Option<sizing::CalcSizeCalculation> = None;
    let _: Option<sizing::CalcSizeCalculationErrorOf<f32>> = None;
    let _: Option<sizing::CalcSizeCalculationOf<f32>> = None;
    let _: Option<sizing::CalcSizeConstructionError> = None;
    let _: Option<sizing::FlexBasis> = None;
    let _: Option<sizing::FlexBasisCalcBasis> = None;
    let _: Option<sizing::FlexBasisOf<f32>> = None;
    let _: Option<sizing::MaxSize> = None;
    let _: Option<sizing::MaxSizeCalcBasis> = None;
    let _: Option<sizing::MaxSizeOf<f32>> = None;
    let _: Option<sizing::MinSize> = None;
    let _: Option<sizing::MinSizeCalcBasis> = None;
    let _: Option<sizing::MinSizeOf<f32>> = None;
    let _: Option<sizing::PreferredSize> = None;
    let _: Option<sizing::PreferredSizeCalcBasis> = None;
    let _: Option<sizing::PreferredSizeOf<f32>> = None;
    let _: Option<sizing::SizingCalculation> = None;
    let _: Option<sizing::SizingCalculationError> = None;
    let _: Option<sizing::SizingCalculationOf<f32>> = None;
    let _: Option<text_flow::HyphenateLimitLast> = None;
    let _: Option<text_flow::HyphenateLimitLines> = None;
    let _: Option<text_flow::InlineBreakOpportunity> = None;
    let _: Option<text_flow::LineBoxDirectionPolicy> = None;
    let _: Option<text_flow::LineBreakDisplay> = None;
    let _: Option<text_flow::LineBreakInput> = None;
    let _: Option<text_flow::LineBreakInputOf<f32>> = None;
    let _: Option<text_flow::TextAlign> = None;
    let _: Option<text_flow::TextAlignLast> = None;
    let _: Option<text_flow::TextGroupAlign> = None;
    let _: Option<text_flow::TextIndent> = None;
    let _: Option<text_flow::TextIndentOf<f32>> = None;
    let _: Option<text_flow::TextJustification> = None;
    let _: Option<text_flow::TextWrapStyle> = None;
    let _: Option<text_layout::InlineBoxFragmentationDemand> = None;
    let _: Option<text_layout::InlineFragmentOutput> = None;
    let _: Option<text_layout::InlineFragmentOutputOf<f32>> = None;
    let _: Option<text_layout::InlineFragmentOwner<usize>> = None;
    let _: Option<text_layout::PlacedInlineBoxFragments<'static>> = None;
    let _: Option<text_layout::PlacedTextOutput<usize>> = None;
    let _: Option<text_layout::TextAtomicBinding<usize>> = None;
    let _: Option<text_layout::TextBindingError> = None;
    let _: Option<text_layout::TextControlBinding<usize>> = None;
    let _: Option<text_layout::TextControlParticipation> = None;
    let _: Option<text_layout::TextOriginBinding<usize>> = None;
    let _: Option<text_layout::TextParagraphBinding<usize>> = None;
    let _: Option<text_layout::TextParticipation> = None;
    let _: Option<text_layout::TextScopeAlignmentInput> = None;
    let _: Option<text_layout::TextScopeAlignmentInputOf<f32>> = None;
    let _: Option<text_layout::TextScopeAssociation<usize>> = None;
    let _: Option<text_layout::TextScopeBinding<usize>> = None;
    let _: Option<tree::LeafMeasureInput> = None;
    let _: Option<tree::LeafMeasureInputOf<f32>> = None;
}

#[expect(
    dead_code,
    reason = "type-checking the function items is the assertion"
)]
fn functions_are_named_as_values<T, W>()
where
    T: tree::LayoutTree,
    W: inline_wrapping::WrapEdgeSource,
{
    let _ = compute_layout::<T>;
    let _ = compute_layout_invalidated::<T>;
    let _ = compute_layout_with_flow::<T>;
    let _ = compute_layout_with_flow_invalidated::<T>;
    let _ = tree::measure_definite_inline_atomic_size::<T>;
    let _ = tree::compute_leaf::<f32, ()>
        as fn(
            cache::ComputeInputOf<f32>,
            &node::NodeInputOf<f32>,
            fn(tree::LeafMeasureInputOf<f32>) -> Result<geometry::Size<f32>, ()>,
        ) -> _;
    let _ = inline_wrapping::process_wrapping_source::<W>;
    let _ = scroll::anchor_selection::select_scroll_anchor::<usize, f32>
        as fn(
            scroll::anchor_selection::ScrollAnchorRequestOf<usize, f32>,
            fn(usize) -> Option<scroll::anchor_selection::ScrollAnchorNodeOf<'static, usize, f32>>,
        ) -> _;
    let _ = grid::track_sizing_components;
    let _ = grid::track_sizing_components_of::<f32>;
}
