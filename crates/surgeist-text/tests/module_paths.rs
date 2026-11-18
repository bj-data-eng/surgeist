//! Every public item of the crate resolves through its one module path.
//!
//! The imports are the assertion: a missing or misplaced export fails to compile. The crate root holds
//! only `System`, `Error`, `ErrorCode`, `ErrorDetail` and `Result`; every other item lives in a module.
#![expect(unused_imports, reason = "the imports are the assertion")]

use surgeist_text::{Error, ErrorCode, ErrorDetail, Result, System};

#[cfg(feature = "text-accessibility")]
use surgeist_text::accessibility::Accessibility;
use surgeist_text::allocation::{
    AtomicAllocation, AtomicBoxAllocation, AtomicFitAllocation, AtomicFlowAllocation,
    AtomicLineAllocation, ControlAllocation, ControlStrut, CssMarginBoxFit,
    InlineBoundaryAllocation, InlineBoundaryContextAllocation, LineAlignmentInsets,
    LineBoxAlignment, LineBoxStrut, LineStrut,
};
use surgeist_text::analyzed::{
    AnalysisIdentity, AnalysisTypography, AnalyzedCombinedCell, AnalyzedCombinedSourcePiece,
    AnalyzedDomain, AnalyzedSource, AnalyzedWrappingRelation, AnalyzedWrappingRelationView,
    CapturedAnalysisOwner, TextCombineError, WrappingNeighbor, WrappingPolicy,
};
use surgeist_text::cache::{Key, OptionsKey, SourceKey, Stats, StyleKey};
use surgeist_text::capture::{
    CapturedComputedLineBoundary, CapturedComputedLineBoundaryTarget, CapturedConnection,
    CapturedControl, CapturedDomainDefinition, CapturedEventPosition, CapturedFlowEvent,
    CapturedJointFlow, CapturedParagraph, CapturedScope, CapturedScopeIntersection,
    ComputedLineBoundaryError, ComputedLineBoundaryInput, ContextualSourceInput,
    ProcessedParagraph, ScopeRole, SourceBoxSide, SourceContinuityOperation,
    SourceProcessingConnection, SourceProcessingConnectionError, SourceProcessingEnvelope,
    SourceProcessingError, SourceProcessingEvent, SourceProcessingJointFlow,
    SourceProcessingJointFlowError, SourceProcessingParagraph, SourceProcessingScope,
    SourceProcessingScopeBoundaries, SourceProcessingTopology, SourceWritingAxis, WhiteSpaceTrim,
    process_contextual_source,
};
use surgeist_text::composer::{Composer, Mark, compose, source};
use surgeist_text::conditional::{
    ConditionalDisposition, ConditionalLineContext, ConditionalPurpose, ConditionalStage,
    PhraseOverflowDecision, PreparedConditionalBatch, PreparedConditionalOpportunity,
    PreparedConditionalSearch, PreparedPhrase, PreparedPhrasePiece, PreparedPhrasePieceOwner,
    PreparedPhrasePortion, PreparedPhraseProvenance,
};
use surgeist_text::finalized::{
    AtomicSlotFeedback, FinalizedCellView, FinalizedControlView, FinalizedFragment,
    FinalizedFragmentOwner, FinalizedFragmentView, FinalizedGlyphView, FinalizedLine,
    FinalizedPrefixView, FinalizedRunView, FinalizedSharedUnitView, FormattedLineUsage,
    InlineMeasure, LineEdgeDisposition, LineEdgeOwner, LineEdgeSide, LineEdgeUnitView,
    LineFeedback, LineMeasurements, LineOutlineInk, MarginEdgeFit, MarginEdgeFitAuthority,
    PresentationMappingKind, SelectedMappingView, TextSeam,
};
use surgeist_text::font::{
    EffectiveFontInstance, FontBaseline, FontBaselineAbsence, FontBaselineAxis,
    FontBaselineMeasurement, FontBaselineOutcome, FontBaselineProvenance, FontBaselineScript,
    FontBaselineUnavailable, FontBaselineUnavailableReason, FontBaselineValue, FontData, FontFace,
    FontLineMetricSource, FontMetricField, FontMetricNumericError, FontMetricPhase,
    FontMetricTable, FontMetricTableError, FontMetrics, FontOutlineCapability, FontOutlineError,
    FontSynthesis, GlyphOutlineGeometry, StrikeoutMetrics, UprightFontMetrics,
    UprightMetricProvenance, UprightMetricReference, VerticalGlyphMetricProvenance,
    VerticalGlyphMetrics,
};
use surgeist_text::geometry::{
    GlyphBasis, GlyphFrame, NumericRequirement, Point, Rect, Size, TextNumericError,
    TextNumericPhase,
};
use surgeist_text::hyphenation::{
    HyphenateCharacter, HyphenateLimitChars, HyphenationContinuation, HyphenationCounts,
    HyphenationError, HyphenationMinimum, HyphenationPlacement, HyphenationProvenance,
    HyphenationResource, HyphenationSourcePieceView, Hyphens, NativeHyphenationConvention,
    PreparedHyphenationCut, PreparedHyphenationCutView, PreparedWord, PreparedWordPortion,
    PreparedWordView,
};
use surgeist_text::inline_box::{
    FinalizedInlineBoxPiece, InlineBoxComponentParticipation, InlineBoxEdgeComponents,
    InlineBoxEdgeContext, InlineBoxEdgeError, InlineBoxFragment, InlineBoxFragmentationError,
    InlineBoxPieceMember, InlineBoxSource, PlacedInlineBoxPiece, PreparedInlineBoxFamily,
    SelectedInlineBoxPiece, SuppliedInlineBoxComponent, SuppliedInlineBoxComponentRow,
    SuppliedInlineBoxComponents, SuppliedInlineBoxComponentsInput,
};
use surgeist_text::layout::{
    Affinity, BaselineShiftFact, Builder, Cluster, Cursor, CursorGeometry, DecorationKind,
    DecorationRun, Glyph, GlyphPlacement, Hit, InlineBoxMetricFact, InlineMetricFacts, Layout,
    Line, LineMetricFact, Metrics, Movement, PaintedSelectionGeometry, PaintedSelectionRect,
    PlacedPosition, PlacedSelection, PlacedSelectionPiece, PositionedInlineBox, Run, RunMetricFact,
    RunMetrics, RunView, Selection, SelectionGeometry, SelectionRect, ShapingBoundary,
    ShapingRefusalReason,
};
use surgeist_text::leader::{
    FinalizedLeaderPattern, LeaderClusterView, LeaderCombinationContext, LeaderPaintGroup,
    LeaderPattern, LeaderPatternInput, LeaderRunView,
};
use surgeist_text::observation::{
    CapturedScopeIncidence, FlowObservation, FlowObservationRequest, FlowSegmentRole,
    PreparedBoundaryIncidence, PreparedFlowSegment, PreparedMeasurementFlow,
    SemanticAtomicAllocation, SemanticConditionalEvidence, SemanticControlAllocation,
    SemanticPartitionEvidence, SemanticWindowEvidence, SemanticWindowFailure,
    SemanticWindowRequest, WrappingScopeQueryError,
};
use surgeist_text::options::{
    Alignment, Indent, Options, TextAlignLast, TextOverflow, ValidatedOptions,
};
use surgeist_text::paragraph::{
    CompleteTextNodeContribution, ParagraphAnchor, ParagraphAnchorItem, ParagraphBaseDirection,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnection,
    ParagraphConnectionKind, ParagraphControl, ParagraphControlKind, ParagraphInput,
    ParagraphOrigin, ParagraphOriginKind, ParagraphReplacement, ParagraphReplacementDefinition,
    ParagraphStrut, ParagraphTerminalEdge,
};
use surgeist_text::placement::{
    AtomicPlacement, BreakFontMetricBasis, LineAnchor, LinePlacement, NonFormattedLine,
    NonFormattedLinePlacement, PlacedBreakFontMetrics, PlacementCoverage, PlacementEntry,
    PlacementRequest,
};
use surgeist_text::prepared::{
    LineDirectionFacts, PhaseTwoContext, PreparationOptions, PreparedAlternative,
    PreparedAlternativeAvailability, PreparedAlternativeDefinition, PreparedAlternativeView,
    PreparedAtomic, PreparedAtomicView, PreparedBoundary, PreparedBoundaryView, PreparedBreakKind,
    PreparedCell, PreparedCellView, PreparedComputedLineBoundary, PreparedComputedLineBoundaryView,
    PreparedControl, PreparedControlView, PreparedOpportunity, PreparedOpportunityView,
    PreparedOrigin, PreparedOriginIntersectionView, PreparedOriginView, PreparedParticipant,
    PreparedScope, PreparedScopeView, PreparedSourceSegment, PreparedText, PreparedUnit,
    PreparedUnitView, PreparedVirtualSeparator, PreparedVirtualSeparatorView, PresentationRange,
    ProtocolContextField, ProtocolError, ProtocolHandleKind,
};
use surgeist_text::processing::{
    DocumentSegmentBreakError, DocumentSegmentBreaks, PlaintextItem, PlaintextItems,
    ProcessedInlineBox, ProcessedPosition, ProcessedRange, ProcessedSource, ProcessedSpan,
    ProcessedText, ProcessingCapacityError, TextTransformError, process_source_whitespace,
};
use surgeist_text::range_fragments::{
    SourceRangeCellFontPiece, SourceRangeCellLayoutFrame, SourceRangeCellMetricFrame,
    SourceRangeCellTabPiece, SourceRangeCellUnit, SourceRangeCellUnitKind,
    SourceRangeFragmentError, SourceRangeFragments, SourceRangeGeometry, SourceRangePiece,
    SourceRangeUnit,
};
use surgeist_text::scope_alignment::{
    LineAllocations, PreparedScopeMetrics, ScopeAlignmentAllowance, ScopeAlignmentBoundary,
    ScopeAlignmentCensus, ScopeAlignmentCensusAccounting, ScopeAlignmentCensusFailure,
    ScopeAlignmentCensusInput, ScopeAlignmentError, ScopeAlignmentLimitKind, ScopeAlignmentLimits,
    ScopeAlignmentProduct, ScopeAlignmentProducts, ScopeAlignmentRetention, ScopeAlignmentUsage,
    ScopeAlignmentWeakProduct, ScopeAllocation, ScopeAllocations, ScopeFontReference,
    ScopeFrameAllocation, ScopeMetricContribution, SelectedScope, scope_alignment_census,
};
use surgeist_text::selection::{
    FormattedLineContext, LineBoxDirection, LineContext, LineEnd, LineOrientation,
    LineSelectionRequest, LineStart, PreparedTermination, SelectedLine,
};
use surgeist_text::source::{
    BaselineShift, Edit, FirstLetterSelection, Id, InlineBox, InlineBoxKind, Range, Source,
    SourceIdentity, SourcePosition, SourceRange, SourceRevision, Span, Text, TextEdit,
    ValidatedSource, VerticalAlign,
};
use surgeist_text::spacing::{
    SpacingAllocation, SpacingAllocations, SpacingBoundaryGeometry, SpacingConstraint,
    SpacingConstraintKind, SpacingContext, SpacingDeferredCause, SpacingFailureReason,
    SpacingOperation, SpacingPiece, SpacingPolicy, SpacingSlot, SpacingSlotDisposition,
    SpacingSlotFeedback, SpacingSlotGeometry, SpacingSlotOwner, SpacingSlotPiece,
    SpacingSlotTarget, SpacingStage, SpacingStretchability, SpacingUnit,
};
use surgeist_text::style::{
    Brush, CaseTransform, Decoration, DecorationBrush, DecorationOffset, DecorationOffsetValue,
    DecorationThickness, DecorationThicknessValue, Direction, Font, FontVariant, FontWeightValue,
    FontWidthRatio, HangingPunctuation, HangingPunctuationEnd, LineBreak, LineHeight, OverflowWrap,
    SelectionPaint, Slant, Style, TextCombineUpright, TextCombineUprightDigits, TextStyleFeature,
    TextStylePhase, TextStyleSupport, TextTransform, TextTransformSet, UnsupportedTextStyleReason,
    ValidatedStyle, Weight, WhiteSpace, Width, WordBreak, WordSpaceTransform, Wrap,
};
use surgeist_text::supplied_line::{
    BidiLevel, BidiLevelError, InlineWhitespaceEdge, SuppliedInlineParticipant, SuppliedLine,
    SuppliedLineEdges, SuppliedLineError, SuppliedVisualOrder,
};
use surgeist_text::tabs::{ParagraphTabReference, TabContext, TabPreparation, TabSize};
use surgeist_text::upright::{
    TextScript, TextStyleOwner, UprightContent, UprightContentPiece, UprightDecorationRole,
    UprightPresentation, UprightScriptContext, UprightScriptContextFailure, UprightScriptFailure,
    UprightTabMeasurement, VerticalOriginReferenceReason, VerticalPositionStage,
    VerticalShapingContextReason,
};

#[test]
fn root_items_construct_and_report() {
    let system = System::default();
    let _ = &system;
    let error = Error::new(ErrorCode::NoUsableFontFace, "no usable face");
    assert_eq!(error.code, ErrorCode::NoUsableFontFace);
    let result: Result<()> = Err(error);
    assert!(result.is_err());
}
