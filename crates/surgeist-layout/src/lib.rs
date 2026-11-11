//! Native layout algorithm boundary for Surgeist.
//!
//! The public physical geometry contract uses x/y points, width/height sizes,
//! and top/right/bottom/left edges. Public layout outputs, cached geometry, and
//! scroll geometry remain physical. Layout algorithms may use
//! crate-private logical algorithm geometry while working in inline/block
//! coordinates. Those carriers stay private until the owning [`FlowAxes`]
//! projects them to physical geometry at a contextual boundary.
//!
//! [`FlowAxes`] is the sole production owner of writing-mode mapping for
//! [`WritingMode::HorizontalTb`], [`WritingMode::VerticalRl`],
//! [`WritingMode::VerticalLr`], [`WritingMode::SidewaysRl`], and
//! [`WritingMode::SidewaysLr`]. Its [`Direction`] is the already-resolved
//! used inline direction, not authored or otherwise unresolved CSS. Layout owns
//! its consuming computed-style adapter; callers supply the resolved value at
//! the current normalized input boundary.
//!
//! Scroll input is normalized, computed or otherwise layout-ready input rather
//! than authored CSS. [`ComputedOverflow`] is an atomic canonical pair; layout
//! privately derives used overflow from that pair and box-generation facts.
//! [`ScrollbarWidthOf`] is the explicit finite thickness selected by the caller's
//! scrollbar environment, including zero for overlay or disabled scrollbars.
//! Layout does not discover host scrollbar metrics or guess a missing policy.
//!
//! [`ScrollGeometryOf`] is immutable, layout-produced physical geometry. Its
//! signed x/y range contains the zero initial anchor, and an axis's scroll size
//! is the span `maximum - minimum`, including an explicit zero span. Nested
//! [`ScrollTargetGeometryOf`] preserves the target border box, physical scroll
//! margin, flow axes, and snap metadata used to integrate that target later.
//! The general physical and flow-relative range types also keep finite ordered
//! bounds; when an axis runs in reverse, [`FlowAxes`] swaps and negates endpoints
//! so signed minima and maxima retain their meaning.
//!
//! [`PreferredSizeOf`], [`MinSizeOf`], [`MaxSizeOf`], and [`FlexBasisOf`] are
//! distinct closed property domains with only their role-valid keywords.
//! Direct [`FlexBasisOf::MIN_CONTENT`] and [`FlexBasisOf::MAX_CONTENT`] values
//! retain their distinct intrinsic measurement constraints through the public
//! layout front door; neither is normalized to the generic content basis.
//! [`SizingCalculationOf`] preserves finite affine leaves and nested `min`,
//! `max`, and `clamp` in a validated program evaluated iteratively. Percentages
//! remain symbolic until an explicit [`PercentageBasisOf`] is available; a
//! required missing basis remains unresolved rather than being guessed.
//!
//! [`NodeInputOf::flex_item_collapse`] is a normalized, layout-ready flex
//! effect, and [`FlexItemCollapse::Normal`] is its default. A collapsed in-flow
//! flex item participates through a finite cross-size strut replay, publishes
//! zero committed collapsed geometry, and hides its descendants. Layout
//! owns lowering a flex item's computed `visibility: collapse` into this
//! normalized state, while rendering owns painting. This leaf does not parse
//! authored CSS or provide a general visibility model.
//!
//! Canonical layout-ready `calc-size()` input pairs the property's calc-size
//! basis with a validated [`CalcSizeCalculationOf`] containing finite length,
//! percentage, and size coefficients. Track flex remains track-only: callers
//! validate a finite, non-negative [`TrackFlexFactorOf`] and place it only in a
//! [`MaxTrackSizingOf`] breadth. Later-owned valid behavior returns
//! [`LayoutUnsupportedCapability::SizingBehavior`] with an exact
//! [`UnsupportedSizingBehavior`] property, behavior, algorithm, and axis.
//!
//! [`LayoutRootRequestOf`] carries validated dimensions and explicit root context
//! for [`compute_layout`], which
//! returns either a complete [`CompletedLayoutBatchOf`] or a typed
//! [`LayoutErrorOf`] with no partial public result. Recursive compute modes are
//! internal.
//!
//! [`LayoutFlowInput`] binds immutable Text paragraphs and declared
//! [`WrappingScopeInput`] boxes to one borrowed tree. Numeric box/control flow
//! can supply no Text paragraphs. [`compute_layout_with_flow`] consumes that
//! checked flow and selects Text-issued candidates when present. It retains the exact
//! placed [`surgeist_text::Layout`] in each [`PlacedTextOutput`], alongside
//! source-associated fragment handles and canonical atomic allocations.
//! Text owns shaping and source mappings; Layout owns fitting and placement,
//! Render owns paint conversion, and Runtime owns coherent publication.
//! The prepared-text profile supports horizontal LTR and RTL flow, signed
//! indentation, and font-advance or CSS-stop tabs. [`TextAlign`] and
//! [`TextAlignLast`] select line alignment. [`TextWrapStyle`] selects ordinary
//! or balanced breaks; [`TextWrapStyle::Stable`] retains ordinary line-local
//! quality selection. [`TextWrapStyle::AvoidShortLastLine`] attempts bounded
//! final-pair repair in a Paragraph-ending group. [`TextWrapStyle::Pretty`]
//! composes that repair with bounded body-line improvement at each actual group
//! entry. See the [native quality policy](../docs/reference.md#node-input).
//! These choices do not
//! change the real containing geometry
//! or wrapping permission. [`TextJustification::Spaces`] permits
//! explicit expansion of actual retained U+0020 slots during justification.
//! [`TextJustification::InterWord`] consumes Text-qualified separators and exact
//! alignment feedback, including actual tab response and unexpandable fallback.
//! [`TextJustification::InterCharacter`] uses the same allocator over Text-qualified
//! visual character boundaries, retaining protected/constrained provider geometry.
//! [`TextJustification::None`]
//! adds no expansion. Text owns line-edge and trailing-space decisions.
//! Other Text orientations return a precise unsupported error.
//! [`NodeInputOf::non_box`] is the canonical tree companion for text and inline
//! controls.
//!
//! Completed batches keep unrounded and final [`InlineFragmentOutputEntryOf`]
//! phases alongside node and cache state. [`compute_layout_invalidated`] stages
//! the exact invalidation closure, and [`CompletedLayoutBatchOf::apply_to`]
//! uses [`LayoutBatchSink`] to prepare an owned replacement immutably before an
//! infallible exclusive commit.
//!
//! Floating boxes select the closed [`FloatExclusion`] contract. `MarginBox` is
//! the default; `Shape` promises a tree provider that accepts a validated
//! physical [`FloatExclusionQueryOf`] and returns a clipped
//! [`FloatExclusionIntervalOf`]. Float placement and independent block sizing
//! and collision use the floats' physical margin boxes for both variants,
//! without shape queries. Layout invokes the provider for inline wrapping
//! across an overlapping finite candidate band. Missing providers, provider
//! failures, and mismatched interval queries retain typed float/query diagnostics
//! and return no partial batch. Required shape intervals never fall back to
//! rectangular exclusion.
//! Provider facts change through explicit tree invalidation rather than a
//! cache-key revision.
//!
//! `compute_leaf` is the direct fallible measurement boundary: providers receive
//! non-negative content-space constraints and provider failures or invalid output
//! become typed layout errors. `DefaultScalar` and `Scalar` use `f32`; generic
//! `*Of<S>` contracts support end-to-end `f32` and `f64` scalar lanes.
//!
//! [`ItemOrder`] is the layout-ready signed order value. [`SourceIndex`] is
//! stable source-sibling identity: outputs remain source-associated while flex,
//! ordinary grid, and grid-lanes consume one stable order-modified traversal
//! sorted by item order and then source index.
//!
//! [`NodeInputOf::item_is_replaced`] is an independent box-generation fact, not
//! inferred from table role, measurement, aspect ratio, or stretch. Block and
//! root sizing use it to avoid ordinary auto-inline fill; flex uses it for
//! automatic main-size suggestion selection; and grid and grid-lanes use it for
//! normal alignment while preserving explicit stretch.
//!
//! [`ContainingLayoutContext`] and [`ParentFormattingContext`] form the complete
//! containing context and cache identity, including explicit containing flow and
//! parent role. [`SizingRequestOf`] independently carries physical available space,
//! per-axis sizing operations, and independent ICB, nearest-scrollport and local
//! containing-bound authority for orthogonal fallback.
//! [`FlexItemRootContext`] keeps actual parent flow axes, imposed host allocation,
//! and percentage bases separate from those request dimensions and operations.
//!
//! Browser-parity generation is crate-local tooling. `generate` is the
//! managed-pinned mode and may use the configured fetcher for the exact manifest
//! pin. `generate-existing` is the existing-pinned, no-fetch mode and accepts
//! only a repository-relative executable under that manifest cache whose exact
//! `--version` matches the pin. Both use the shared headless launch profile,
//! including the mock-keychain argument; corpus freshness checks remain
//! browser-free.
//!
//! Layout owns its direct adapters from lower-level producers. Callers supply
//! normalized style/box facts, host scrollbar policy and retained association;
//! the respective consumers own live scrolling, CSSOM, rendering and host events.
//! Runtime coordinates publication; root `surgeist` provides optional facade
//! composition and bootstrap. This crate emits canonical geometry and target
//! metadata; it does not parse authored CSS, resolve computed style or own a live
//! scrolling runtime.
//!
//! The normalized scroll input and read-only output surface composes through the
//! crate root in either scalar lane without constructing derived geometry:
//!
//! ```
//! use surgeist_layout::{
//! ComputedOverflow, NodeInputOf, NodeOutputOf, Overflow, OverflowClipMarginOf,
//! ScrollGeometryOf, ScrollMarginOf, ScrollPaddingOf, ScrollbarWidthOf,
//! };
//!
//! fn normalized_input<S: surgeist_layout::LayoutScalar>() -> NodeInputOf<S> {
//! let mut input = NodeInputOf::<S>::default();
//! input.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Scroll).unwrap();
//! input.overflow_clip_margin = OverflowClipMarginOf::try_new(
//! surgeist_layout::OverflowClipBox::PaddingBox,
//! S::ZERO,
//! ).unwrap();
//! input.scrollbar_width = ScrollbarWidthOf::try_new(S::ZERO).unwrap();
//! input.scroll_padding = ScrollPaddingOf::default();
//! input.scroll_margin = ScrollMarginOf::try_new(S::ZERO, S::ZERO, S::ZERO, S::ZERO).unwrap();
//! input
//! }
//!
//! fn inspect<S: surgeist_layout::LayoutScalar>(output: NodeOutputOf<S>) {
//! let geometry: Option<ScrollGeometryOf<S>> = output.scroll_geometry;
//! if let Some(geometry) = geometry {
//! let x = geometry.physical_range().x();
//! let target = geometry.target();
//! let _ = (x.minimum(), x.maximum(), target.border_box(), target.snap_align());
//! }
//! }
//!
//! let _ = normalized_input::<f32>();
//! let _ = normalized_input::<f64>();
//! inspect(NodeOutputOf::<f64>::default());
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{NodeInput, Overflow, Point};
//! let mut input = NodeInput::DEFAULT;
//! input.overflow = Point::new(Overflow::Visible, Overflow::Visible);
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{NodeOutput, Size};
//! let mut output = NodeOutput::default();
//! output.scrollbar_size = Size::ZERO;
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{ScrollGeometry, ScrollbarGutterRects};
//! let _ = ScrollGeometry::default();
//! let _ = ScrollbarGutterRects::default();
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::ScrollPaddingValue;
//! let _ = ScrollPaddingValue::Deferred;
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::NodeOutput;
//! let output = NodeOutput::default();
//! let _ = output.current_scroll_offset;
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{LogicalEdgesOf, LogicalPointOf, LogicalRectOf, LogicalSizeOf};
//! ```

#[cfg(test)]
#[path = "../tests/layout/browser_parity/fixture_variants.rs"]
mod fixture_variants;

#[cfg(test)]
use surgeist_text::{BidiLevel, BidiLevelError, InlineWhitespaceEdge};

#[cfg(test)]
#[path = "../tests/layout/browser_parity/atomic_paragraph_metrics.rs"]
mod atomic_paragraph_metrics;
mod baseline_offset;
mod block;
mod block_fragmentation;
pub use block_fragmentation::{
    BlockFragmentainerOf, BlockFragmentationFailure, BoxFragmentGeometryOf, BoxGeometryPairOf,
    FragmentCoordinateSpace, FragmentOutputOf, FragmentPayloadOf, LayoutBatchApplyError,
    LayoutBatchCommit, LayoutBatchPreparation, LayoutBoundRequestOf, LayoutFragmentId,
    LayoutFragmentainerId, LayoutFragmentainerOutputOf, LayoutHandleError,
    LayoutPublicationTargets, LayoutSourceBasis, LayoutTargetBasis, NodeLayoutRefOf,
    NonBoxGeometryOf, NonBoxKind, NonBoxOutputRefOf, OverflowFragmentGeometryOf,
    OverflowGeometryPairOf, SingleBoxOutputRefOf,
};
mod cache;
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/empty_line_br_font_band_profile.rs"]
mod empty_line_br_font_band_profile;
mod engine;
mod error;
mod flex;
mod fragmentation;
pub use fragmentation::{
    FragmentainerOf, FragmentationBoundaryControls, FragmentationBreak, FragmentationBreakControl,
    FragmentationBreakInside, FragmentationBreakKinds, FragmentationBreakReason,
    FragmentationBreakRecord, FragmentationContext, FragmentationControlKind,
    FragmentationCoverage, FragmentationCutOf, FragmentationEmergencySliceOf, FragmentationError,
    FragmentationFallback, FragmentationFlowOf, FragmentationForcedSide, FragmentationFrameOf,
    FragmentationGraphicalCoverageOf, FragmentationGraphicalProgressOf, FragmentationInputError,
    FragmentationInputOf, FragmentationKind, FragmentationLogicalPageSide,
    FragmentationMeasuredHeadOf, FragmentationOccurrence, FragmentationOpportunityKindOf,
    FragmentationOpportunityOf, FragmentationPageSide, FragmentationPhysicalPageSide,
    FragmentationPieceKindOf, FragmentationPieceOf, FragmentationPosition, FragmentationProvider,
    FragmentationQueryOf, FragmentationReplyOf, FragmentationReportOf, FragmentationRole,
    FragmentationStage, FragmentationStart, FragmentationWindowEnd, FragmentationWindowOf,
    PageProgression, resolve_fragmentation,
};
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/capture_helper_identity.rs"]
mod capture_helper_identity;
mod geometry;
mod grid;
#[cfg(test)]
#[path = "../tests/layout/browser_parity/grid_alignment_reference.rs"]
mod grid_alignment_reference;
mod group_alignment;
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/horizontal_br_font_band_profile.rs"]
mod horizontal_br_font_band_profile;
mod hyphenation;
mod inline;
mod layout_math;
mod measurement;
mod node_input;
mod node_projection;
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/original_flex_parent_profile.rs"]
mod original_flex_parent_profile;
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/original_subgrid_body_profile.rs"]
mod original_subgrid_body_profile;
mod output;
#[cfg(test)]
#[path = "../tests/layout/browser_parity/pinned_font.rs"]
mod pinned_font;
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/prepared_atomic_percentage_profile.rs"]
mod prepared_atomic_percentage_profile;
#[cfg(test)]
#[path = "../tests/bin/surgeist-layout-generate/measurement/vertical_br_font_band_profile.rs"]
mod vertical_br_font_band_profile;
pub use baseline_offset::{BaselineOffset, BaselineOffsetOf, BaselineOffsetResolutionError};
pub use hyphenation::{HyphenateLimitLast, HyphenateLimitLines};
mod inline_wrapping;
mod line_box;
mod text_indent;
mod text_layout;
pub use inline_wrapping::{
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
pub use text_layout::{
    InlineBoxFragmentationDemand, LayoutFlowInput, TextAtomicBinding, TextBindingError,
    TextControlBinding, TextControlParticipation, TextOriginBinding, TextParagraphBinding,
    TextParticipation, TextScopeAlignmentInput, TextScopeAlignmentInputOf, TextScopeAssociation,
    TextScopeBinding,
};
mod scalar;
mod scroll;
mod sizing;
mod tree;
mod value;

/// Default scalar precision used by the non-generic public layout aliases.
///
/// This is intentionally `f32`, matching the crate's browser-parity fixture
/// boundary and the default Surgeist layout coordinate contract.
pub type DefaultScalar = f32;

/// Convenience alias for the default scalar precision.
///
/// Use explicit `*Of<S>` types with `S: LayoutScalar` when one layout tree
/// needs to run end-to-end with a different supported precision such as `f64`.
pub type Scalar = DefaultScalar;

#[cfg(test)]
pub(crate) use block::compute_block;
pub use cache::{Cache, CacheKeyContext, CacheOf, ClearState};
pub(crate) use engine::contracts::{CacheAccess, Compute};
#[cfg(test)]
pub(crate) use engine::contracts::{Round, compute_cached};
#[cfg(test)]
pub(crate) use engine::round_layout;
#[cfg(test)]
pub(crate) use engine::{compute_hidden, compute_root};
pub use engine::{
    compute_layout, compute_layout_invalidated, compute_layout_with_flow,
    compute_layout_with_flow_invalidated, measure_definite_inline_atomic_size,
};
pub use error::{
    AtomicInlineParticipationRoleError, CalcSizeBehaviorBasis, FloatExclusionRoleError,
    InvalidMeasurementOutput, InvalidMeasurementOutputOf, LayoutError, LayoutErrorKind,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSite, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutInvalidInput, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation, LayoutResult,
    LayoutResultOf, LayoutTextNumericError, LayoutUnsupportedCapability, NonBoxNodeRoleError,
    SizingAlgorithm, SizingBehavior, SizingProperty, UnsupportedSizingBehavior,
};
#[cfg(test)]
pub(crate) use flex::compute_flex;
pub use geometry::{Edges, FlowAxes, LogicalAxis, PhysicalAxis, PhysicalSide, Point, Size};
pub use grid::{
    DefiniteLaneIntrinsicItem, DefiniteLaneIntrinsicItemOf, GridAxisKind, GridComputation,
    GridComputationOf, GridComputationReport, IndefiniteLaneContributionGroup,
    IndefiniteLaneContributionGroupOf, LaneContributionFacts, LaneContributionFactsOf,
    LaneIntrinsicItem, LaneIntrinsicItemKind, LaneIntrinsicItemOf, LaneIntrinsicSizingInput,
    LaneIntrinsicSizingInputOf, LaneIntrinsicSizingReport, LaneIntrinsicSizingReportOf, LaneItem,
    LaneItemOf, LaneItemOffset, LaneItemOffsetOf, LanePlacementError, LanePlacementInput,
    LanePlacementInputOf, LanePlacementReport, LanePlacementReportOf, LaneTrackSpan,
    LaneTrackSpanLength, NamedGridErrorReport, NamedGridReport, lane_intrinsic_sizing, place_lanes,
};
#[cfg(test)]
pub(crate) use grid::{compute_grid, compute_grid_with_report};
pub use measurement::{
    LeafMeasureError, LeafMeasureErrorOf, LeafMeasureInput, LeafMeasureInputOf, compute_leaf,
};
pub use node_input::{
    AlignContent, AlignItems, AtomicInlineParticipation, BaselineSource, BlockFlowContext,
    BlockGeneration, BoxSizing, Clear, ComputedOverflow, ComputedOverflowError, Direction, Display,
    FlexDirection, FlexGrow, FlexGrowOf, FlexItemCollapse, FlexShrink, FlexShrinkOf, FlexWrap,
    Float, FloatExclusion, FloatExclusionInterval, FloatExclusionIntervalError,
    FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf, FloatExclusionQuery,
    FloatExclusionQueryOf, GridAutoFlow, GridBaselineError, GridBaselineReference,
    GridBaselineReferences, GridFlowTolerance, GridFlowToleranceOf, GridLanesFlow, GridPlacement,
    InlineBaselineContext, InlineBaselineError, InlineBoundaryEdge, InlineBoundaryEdgeError,
    InlineBoundaryEdgeErrorOf, InlineBoundaryEdgeOf, InlineBoundaryInput, InlineBoundaryInputOf,
    InlineBoundaryKind, InlineBreakOpportunity, InlineMetrics, InlineMetricsError, InlineMetricsOf,
    ItemAlignmentPosition, ItemOrder, LayoutInput, LayoutInputOf, LayoutInputView,
    LayoutInputViewOf, LineBoxDirectionPolicy, LineBreakDisplay, LineBreakInput, LineBreakInputOf,
    NodeInput, NodeInputOf, Overflow, OverflowClipBox, OverflowClipMargin, OverflowClipMarginOf,
    Position, RawGridLine, RawGridPlacement, ScrollMargin, ScrollMarginError, ScrollMarginErrorOf,
    ScrollMarginOf, ScrollPadding, ScrollPaddingOf, ScrollPaddingValue, ScrollPaddingValueOf,
    ScrollSnapAlign, ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness,
    ScrollSnapType, ScrollbarGutter, ScrollbarWidth, ScrollbarWidthOf, SizeContainment, TextAlign,
    TextAlignLast, TextGroupAlign, TextIndent, TextIndentOf, TextJustification, TextWrapStyle,
    VerticalAlign, VerticalAlignOf, WritingMode,
};
pub use output::{
    BaselineCoordinateOf, BaselineFactOf, BaselinePairOf, Baselines, BaselinesOf,
    BlockFlowRootContext, BlockFlowRootContextOf, CollapsibleMargin, CollapsibleMarginOf,
    CompletedLayoutBatch, CompletedLayoutBatchOf, ComputeInput, ComputeInputOf, ComputeOutput,
    ComputeOutputOf, ContainingLayoutContext, FlexItemRootContext, FlexItemRootContextOf,
    InlineFragmentOutput, InlineFragmentOutputEntry, InlineFragmentOutputEntryOf,
    InlineFragmentOutputOf, InlineFragmentOwner, LayoutCacheClearEntry, LayoutCacheStoreEntry,
    LayoutCacheStoreEntryOf, LayoutOutputEntry, LayoutOutputEntryOf, LayoutRootContext,
    LayoutRootContextOf, LayoutRootRequest, LayoutRootRequestOf, LayoutRoundingMode,
    LayoutSizeInput, LayoutSizeInputError, LayoutSizeInputErrorOf, NamedBaselinesOf, NodeOutput,
    NodeOutputOf, ParentFormattingContext, PhysicalBlockMarginCollapse,
    PhysicalBlockMarginCollapseOf, PlacedInlineBoxFragments, PlacedTextOutput,
    PositionedRootContext, PositionedRootContextOf, RootContextMismatch,
    RootLocalPositionedContext, RootLocalPositionedContextOf, SourceIndex,
};
pub(crate) use output::{RequestedAxis, RunMode, SizingMode};
/// Supported scalar contract for generic layout APIs.
pub use scalar::LayoutScalar;
pub use scroll::{
    FlowRelativeScrollAxisRange, FlowRelativeScrollAxisRangeOf, FlowRelativeScrollOffset,
    FlowRelativeScrollOffsetOf, FlowRelativeScrollRange, FlowRelativeScrollRangeOf, OverflowClip,
    OverflowClipOf, PhysicalClipAxis, PhysicalClipAxisOf, PhysicalScrollAxisRange,
    PhysicalScrollAxisRangeOf, PhysicalScrollOffset, PhysicalScrollOffsetOf, PhysicalScrollRange,
    PhysicalScrollRangeOf, ScrollAnchorBox, ScrollAnchorBoxOf, ScrollAnchorContainingBlock,
    ScrollAnchorNode, ScrollAnchorNodeOf, ScrollAnchorPolicy, ScrollAnchorPosition,
    ScrollAnchorPriority, ScrollAnchorRequest, ScrollAnchorRequestOf, ScrollAnchorSelectionError,
    ScrollAnchorSubject, ScrollAnchorSubjectOf, ScrollCoordinateError, ScrollCoordinateErrorOf,
    ScrollGeometry, ScrollGeometryOf, ScrollRect, ScrollRectError, ScrollRectErrorOf, ScrollRectOf,
    ScrollTargetGeometry, ScrollTargetGeometryOf, ScrollbarGutterRects, ScrollbarGutterRectsOf,
    select_scroll_anchor,
};
pub use sizing::{
    CalcSizeCalculation, CalcSizeCalculationErrorOf, CalcSizeCalculationOf,
    CalcSizeConstructionError, FlexBasis, FlexBasisCalcBasis, FlexBasisOf, MaxSize,
    MaxSizeCalcBasis, MaxSizeOf, MinSize, MinSizeCalcBasis, MinSizeOf, PreferredSize,
    PreferredSizeCalcBasis, PreferredSizeOf, SizingCalculation, SizingCalculationError,
    SizingCalculationOf,
};
/// Baseline names, independent of any font-design or placed-box coordinate.
pub use surgeist_text::FontBaseline as BaselineName;
pub use tree::{LayoutBatchSink, LayoutTree, Traverse};
pub use value::{
    AspectRatio, AspectRatioOf, AvailableSpace, AvailableSpaceOf, AxisSizingRequest,
    AxisSizingRequestOf, ContentSizingRequest, ContentSizingRequestOf, FiniteScalarErrorOf, Length,
    LengthAuto, LengthAutoOf, LengthOf, LengthPercentageErrorOf, LengthPercentageOf,
    LengthResolution, LengthResolutionOf, LengthResolutionStatus, NearestScrollport,
    NearestScrollportOf, NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf, NumericResolutionOf,
    OrthogonalFallback, OrthogonalFallbackError, OrthogonalFallbackErrorOf,
    OrthogonalFallbackLimit, OrthogonalFallbackLimitOf, OrthogonalFallbackOf, PercentageBasisOf,
    ResolvedLengthAuto, ResolvedLengthAutoOf, ScrollportFallbackChange, ScrollportFallbackChangeOf,
    SizingOperation, SizingRequest, SizingRequestOf, UnresolvedLengthReason,
};
pub use value::{
    GridLine, GridSpan, GridTemplateAreaRow, GridTemplateAreas, MaxTrackSizing, MaxTrackSizingOf,
    MinTrackSizing, MinTrackSizingOf, SubgridLineNameComponent, SubgridLineNameRepeatCount,
    SubgridTrack, TrackComponent, TrackComponentList, TrackComponentListOf, TrackComponentOf,
    TrackFlexFactor, TrackFlexFactorOf, TrackRepeat, TrackRepeatCount, TrackRepetition,
    TrackRepetitionError, TrackRepetitionOf, TrackSizing, TrackSizingOf, track_sizing_components,
    track_sizing_components_of,
};

#[cfg(test)]
mod block_tests;
#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod compute_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod flex_tests;
#[cfg(test)]
mod inline_tests;
#[cfg(test)]
mod invalidation_transaction_tests;
#[cfg(test)]
mod leaf_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod root_tests;
#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod test_support;
