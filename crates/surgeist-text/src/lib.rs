//! Owned text preparation and standalone layout for Surgeist.
//!
//! This module owns text layout facts: shaping, line breaking, glyph runs,
//! inline boxes, cursor geometry, selection geometry, and low-level movement.
//! It does not own rendering, widgets, document identity, style resolution, or
//! application commands.

#[cfg(feature = "text-accessibility")]
mod accessibility;
mod analyzed;
mod cache;
mod catalogue;
mod composer;
mod control;
mod edit;
mod engine;
mod error;
mod font;
mod geometry;
mod hyphenation;
mod id;
mod layout;
mod options;
mod phrase;
mod processing;
mod projection;
mod protocol;
mod range;
#[path = "source.rs"]
mod source_model;
mod style;
mod style_support;
mod supplied_line;
mod system;
mod text;
mod transform;
mod views;
mod wrapping;

#[cfg(feature = "text-accessibility")]
pub use accessibility::Accessibility;
pub use analyzed::{AnalysisIdentity, AnalysisTypography, AnalyzedSource};
pub use cache::{FontGeneration, Key, OptionsKey, SourceKey, Stats, StyleKey};
pub use catalogue::{FontCatalogueIdentity, FontCatalogueMode};
pub use composer::{Composer, Mark, compose, source};
pub use edit::{Edit, TextEdit};
pub use error::{
    DocumentSegmentBreakError, Error, ErrorCode, ErrorDetail, FontMetricField,
    FontMetricNumericError, FontMetricPhase, FontMetricTable, FontMetricTableError,
    NumericRequirement, ProcessingCapacityError, Result, ShapingBoundary, ShapingRefusalReason,
    SpacingConstraint, SpacingConstraintKind, SpacingContext, SpacingFailureReason,
    SpacingOperation, SpacingPiece, SpacingStage, TextNumericError, TextNumericPhase, TextScript,
    TextStyleOwner, TextTransformError, UprightContent, UprightContentPiece, UprightDecorationRole,
    UprightPresentation, UprightScriptContext, UprightScriptContextFailure, UprightScriptFailure,
    UprightTabMeasurement, VerticalOriginReferenceReason, VerticalPositionStage,
    VerticalShapingContextReason,
};
pub use font::{
    EffectiveFontInstance, FontBaseline, FontBaselineAbsence, FontBaselineAxis,
    FontBaselineMeasurement, FontBaselineOutcome, FontBaselineProvenance, FontBaselineScript,
    FontBaselineUnavailable, FontBaselineUnavailableReason, FontBaselineValue, FontData, FontFace,
    FontLineMetricSource, FontMetrics, FontOutlineCapability, FontOutlineError, FontSynthesis,
    GlyphOutlineGeometry, StrikeoutMetrics, UprightFontMetrics, UprightMetricProvenance,
    UprightMetricReference, VerticalGlyphMetricProvenance, VerticalGlyphMetrics,
};
pub use geometry::{GlyphBasis, Point, Rect, Size};
pub use id::Id;
pub use layout::{
    Affinity, BaselineShiftFact, Cluster, Cursor, CursorGeometry, DecorationKind, DecorationRun,
    Glyph, Hit, InlineBoxMetricFact, InlineMetricFacts, Layout, Line, LineMetricFact, Metrics,
    Movement, PaintedSelectionGeometry, PaintedSelectionRect, PlacedPosition, PlacedSelection,
    PlacedSelectionPiece, PositionedInlineBox, Run, RunMetricFact, RunMetrics, Selection,
    SelectionGeometry, SelectionRect,
};
pub use options::{Alignment, Indent, Options, TextAlignLast, TextOverflow, ValidatedOptions};
pub use processing::{
    CapturedComputedLineBoundary, CapturedComputedLineBoundaryTarget, CapturedControl,
    CapturedDomainDefinition, CapturedEventPosition, CapturedFlowEvent, CapturedParagraph,
    CapturedScope, CapturedScopeIntersection, ComputedLineBoundaryError, ComputedLineBoundaryInput,
    ContextualSourceInput, DocumentSegmentBreaks, PlaintextItem, PlaintextItems,
    ProcessedInlineBox, ProcessedParagraph, ProcessedPosition, ProcessedRange, ProcessedSource,
    ProcessedSpan, ProcessedText, ScopeAlignmentBoundary, ScopeAlignmentError,
    ScopeAlignmentLimitKind, ScopeAlignmentLimits, ScopeAlignmentUsage, ScopeFontReference,
    ScopeRole, SourceBoxSide, SourceProcessingConnection, SourceProcessingEnvelope,
    SourceProcessingError, SourceProcessingEvent, SourceProcessingParagraph, SourceProcessingScope,
    SourceProcessingScopeBoundaries, SourceProcessingTopology, WhiteSpaceTrim,
    process_contextual_source, process_source_whitespace,
};
pub use protocol::paragraph::{
    CompleteTextNodeContribution, ParagraphAnchor, ParagraphAnchorItem,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnection,
    ParagraphConnectionKind, ParagraphControl, ParagraphControlKind, ParagraphOrigin,
    ParagraphOriginKind, ParagraphReplacement, ParagraphReplacementDefinition, ParagraphStrut,
};
pub use protocol::prefix::{FinalizedPrefixView, FinalizedSharedUnitView, TextSeam};
pub use protocol::readback::{
    FinalizedControlView, FinalizedFragmentOwner, FinalizedFragmentView, FinalizedGlyphView,
    FinalizedRunView, LineOutlineInk, PreparedAlternativeDefinition, PreparedAlternativeView,
    PreparedAtomicView, PreparedBoundaryView, PreparedComputedLineBoundaryView,
    PreparedControlView, PreparedOpportunityView, PreparedOriginIntersectionView,
    PreparedOriginView, PreparedScopeView, PreparedUnitView, PreparedVirtualSeparatorView,
    PresentationMappingKind, SelectedMappingView,
};
pub use protocol::{
    AtomicAllocation, AtomicBoxAllocation, AtomicFlowAllocation, AtomicLineAllocation,
    AtomicPlacement, AtomicSlotFeedback, CapturedScopeIncidence, ConditionalDisposition,
    ConditionalPurpose, ConditionalStage, ControlAllocation, ControlStrut, FinalizedFragment,
    FinalizedLine, FlowObservation, FlowObservationRequest, FlowSegmentRole, FormattedLineContext,
    FormattedLineUsage, HyphenationContinuation, HyphenationCounts, HyphenationProvenance,
    HyphenationResource, HyphenationSourcePieceView, InlineBoundaryAllocation,
    InlineBoundaryContextAllocation, InlineMeasure, LineAlignmentInsets, LineAllocations,
    LineAnchor, LineBoxAlignment, LineBoxDirection, LineBoxStrut, LineContext, LineDirectionFacts,
    LineEdgeDisposition, LineEdgeOwner, LineEdgeSide, LineEdgeUnitView, LineEnd, LineFeedback,
    LineMeasurements, LineOrientation, LinePlacement, LineSelectionRequest, LineStart, LineStrut,
    NonFormattedLine, NonFormattedLinePlacement, ParagraphBaseDirection, ParagraphInput,
    ParagraphTerminalEdge, PhaseTwoContext, PhraseOverflowDecision, PlacementCoverage,
    PlacementEntry, PlacementRequest, PreparationOptions, PreparedAlternative,
    PreparedAlternativeAvailability, PreparedAtomic, PreparedBoundary, PreparedBoundaryIncidence,
    PreparedBreakKind, PreparedComputedLineBoundary, PreparedConditionalBatch,
    PreparedConditionalOpportunity, PreparedConditionalSearch, PreparedControl,
    PreparedFlowSegment, PreparedHyphenationCut, PreparedHyphenationCutView,
    PreparedMeasurementFlow, PreparedOpportunity, PreparedOrigin, PreparedParticipant,
    PreparedPhrase, PreparedPhrasePiece, PreparedPhrasePieceOwner, PreparedPhrasePortion,
    PreparedPhraseProvenance, PreparedScope, PreparedScopeMetrics, PreparedTermination,
    PreparedText, PreparedUnit, PreparedVirtualSeparator, PreparedWord, PreparedWordPortion,
    PreparedWordView, PresentationRange, ProtocolContextField, ProtocolError, ProtocolHandleKind,
    ScopeAlignmentAllowance, ScopeAlignmentCensus, ScopeAlignmentCensusInput,
    ScopeAlignmentProduct, ScopeAlignmentProducts, ScopeAlignmentRetention,
    ScopeAlignmentWeakProduct, ScopeAllocation, ScopeAllocations, ScopeFrameAllocation,
    ScopeMetricContribution, SelectedLine, SelectedScope, SpacingAllocation, SpacingAllocations,
    SpacingBoundaryGeometry, SpacingDeferredCause, SpacingPolicy, SpacingSlot,
    SpacingSlotDisposition, SpacingSlotFeedback, SpacingSlotGeometry, SpacingSlotOwner,
    SpacingSlotPiece, SpacingSlotTarget, SpacingStretchability, SpacingUnit, TabContext,
    WrappingScopeQueryError, scope_alignment_census,
};
pub use range::{Range, SourcePosition, SourceRange};
pub use source_model::{
    BaselineShift, InlineBox, InlineBoxKind, Source, SourceIdentity, SourceRevision, Span,
    ValidatedSource, VerticalAlign,
};
pub use style::{
    Brush, CaseTransform, Decoration, DecorationBrush, DecorationOffset, DecorationOffsetValue,
    DecorationThickness, DecorationThicknessValue, Direction, Font, FontVariant, FontWeightValue,
    FontWidthRatio, HangingPunctuation, HangingPunctuationEnd, LineBreak, LineHeight, OverflowWrap,
    SelectionPaint, Slant, Style, TextTransform, TextTransformSet, ValidatedStyle, Weight,
    WhiteSpace, Width, WordBreak, WordSpaceTransform, Wrap,
};
pub use style_support::{
    TextStyleFeature, TextStylePhase, TextStyleSupport, UnsupportedTextStyleReason,
};
pub use supplied_line::{
    BidiLevel, BidiLevelError, InlineWhitespaceEdge, SuppliedInlineParticipant, SuppliedLine,
    SuppliedLineEdges, SuppliedLineError, SuppliedVisualOrder,
};
pub use system::{Builder, System, SystemOptions};
pub use text::Text;
pub use views::{GlyphPlacement, RunView};
pub use wrapping::{
    AnalyzedWrappingRelation, AnalyzedWrappingRelationView, WrappingNeighbor, WrappingPolicy,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_fonts;
#[cfg(test)]
mod test_wrapping;

#[cfg(test)]
#[path = "test_fonts/shaping.rs"]
mod shaping_test_fonts;

#[cfg(test)]
#[path = "test_fonts/baseline.rs"]
mod baseline_test_fonts;

#[cfg(test)]
#[path = "test_fonts/vertical.rs"]
mod vertical_test_fonts;

#[cfg(test)]
#[path = "test_fonts/mixed.rs"]
mod mixed_test_fonts;

mod range_fragments;
pub use range_fragments::{
    SourceRangeFragmentError, SourceRangeFragments, SourceRangePiece, SourceRangeUnit,
};

pub use hyphenation::{
    HyphenateCharacter, HyphenateLimitChars, HyphenationError, HyphenationMinimum,
    HyphenationPlacement, Hyphens, NativeHyphenationConvention,
};

mod tabs;
pub use tabs::{ParagraphTabReference, TabPreparation, TabSize};
