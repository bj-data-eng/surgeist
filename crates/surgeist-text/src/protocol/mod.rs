//! Resource-retaining public preparation, selection, finalization and placement.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::engine::PreparedData;
use crate::{Alignment, FontCatalogueIdentity, Point, Range, Rect, Result};
use selection::SelectedData;

pub(crate) mod boundary;
mod conditional;
pub(crate) use conditional::request_context;
mod continuation;
mod direction;
pub(crate) mod flow;
pub(crate) mod input;
pub use direction::LineDirectionFacts;
pub use flow::{
    FlowObservation, FlowObservationRequest, FlowSegmentRole, PreparedFlowSegment,
    PreparedMeasurementFlow,
};
pub(crate) mod finalized;
mod hyphenation;
pub(crate) mod measurements;
mod phrase;
pub use phrase::{
    PreparedPhrase, PreparedPhrasePiece, PreparedPhrasePieceOwner, PreparedPhrasePortion,
    PreparedPhraseProvenance,
};
pub(crate) mod paragraph;
pub(crate) mod placement;
pub(crate) mod prefix;
pub(crate) mod prepared;
pub(crate) mod readback;
pub(crate) mod selection;
pub use continuation::HyphenationContinuation;

pub use conditional::{
    ConditionalDisposition, ConditionalPurpose, ConditionalStage, PhraseOverflowDecision,
    PreparedConditionalBatch, PreparedConditionalOpportunity, PreparedConditionalSearch,
};
pub use hyphenation::{
    HyphenationCounts, HyphenationProvenance, HyphenationResource, HyphenationSourcePieceView,
    PreparedHyphenationCutView, PreparedWordPortion, PreparedWordView,
};
pub use measurements::{
    InlineMeasure, LineEdgeDisposition, LineEdgeOwner, LineEdgeSide, LineEdgeUnitView,
    LineMeasurements,
};
pub use paragraph::{ParagraphBaseDirection, ParagraphInput, ParagraphTerminalEdge};
pub use prepared::PreparedBreakKind;

/// Provider-dependent preparation inputs, independent of line choice/placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparationOptions {
    scale: f32,
    tabs: crate::TabPreparation,
}

impl PreparationOptions {
    /// Select one tab resource operation for the complete original/alternative inventory.
    /// CSS preparation fails atomically if a potentially selectable tab lacks required capture or metrics.
    pub fn try_new(scale: f32, tabs: crate::TabPreparation) -> Result<Self> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(crate::error::text_numeric_error(
                "preparation scale",
                crate::TextNumericPhase::Preparation,
                if scale.is_finite() {
                    crate::TextNumericError::OutOfRange
                } else {
                    crate::TextNumericError::NonFinite
                },
                f64::from(scale),
            ));
        }
        Ok(Self { scale, tabs })
    }
    #[must_use]
    pub const fn tabs(self) -> crate::TabPreparation {
        self.tabs
    }
    #[must_use]
    pub const fn scale(self) -> f32 {
        self.scale
    }
}
impl Default for PreparationOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            tabs: crate::TabPreparation::FontAdvance,
        }
    }
}

/// Actual immutable preparation; clones retain the same allocation and fonts.
#[derive(Clone)]
pub struct PreparedText {
    pub(crate) inner: Arc<PreparedData>,
}
impl fmt::Debug for PreparedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedText").finish_non_exhaustive()
    }
}
impl PartialEq for PreparedText {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for PreparedText {}
impl Hash for PreparedText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state);
    }
}
impl PreparedText {
    /// Exact preparation demand retained with this immutable allocation.
    #[must_use]
    pub fn preparation_options(&self) -> PreparationOptions {
        self.inner.options
    }
    /// Actual captured nearest block authority, when supplied by the producer.
    #[must_use]
    pub fn tab_reference(&self) -> Option<&crate::ParagraphTabReference> {
        self.inner.analyzed.paragraph.tab_reference.as_ref()
    }
    /// Actual captured end role, independent of numeric coverage and completion.
    #[must_use]
    pub fn terminal_edge(&self) -> ParagraphTerminalEdge {
        self.inner.analyzed.paragraph.terminal_edge
    }
    #[must_use]
    pub fn analyzed_source(&self) -> crate::AnalyzedSource {
        crate::AnalyzedSource::from_inner(Arc::clone(&self.inner.analyzed))
    }
    #[must_use]
    pub fn font_catalogue_identity(&self) -> &FontCatalogueIdentity {
        &self.inner.font_catalogue
    }
}

// Private constructors are used only after the owning preparation issued and
// validated an actual table index. Public integers cannot create capabilities.
macro_rules! prepared_handle {
    ($($name:ident),+ $(,)?) => {$ (
        #[derive(Clone)]
        pub struct $name {
            pub(crate) prepared: Arc<PreparedData>,
            pub(crate) index: usize,
        }
        impl $name {
            /// Retains the actual preparation that issued this capability.
            #[must_use]
            pub fn prepared_text(&self) -> PreparedText {
                PreparedText { inner: Arc::clone(&self.prepared) }
            }
        }
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.index == other.index && Arc::ptr_eq(&self.prepared, &other.prepared)
            }
        }
        impl Eq for $name {}
        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                Arc::as_ptr(&self.prepared).hash(state);
                self.index.hash(state);
            }
        }
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name)).field("ordinal", &self.index).finish_non_exhaustive()
            }
        }
    )+};
}
prepared_handle!(
    PreparedOrigin,
    PreparedScope,
    PreparedVirtualSeparator,
    PreparedUnit,
    PreparedAtomic,
    PreparedControl,
    PreparedBoundary,
    PreparedOpportunity,
    PreparedAlternative,
    PreparedWord,
    PreparedHyphenationCut
);

/// The complete logical mixed sequence, using distinct issued occurrence roles.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PreparedParticipant {
    Text(PreparedUnit),
    Atomic(PreparedAtomic),
    Control(PreparedControl),
}

/// Supported physical shaping basis. Other requested bases refuse explicitly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineOrientation {
    Horizontal,
    Vertical,
    Sideways,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineStart {
    Paragraph,
    Continuation,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineEnd {
    Soft,
    Forced,
    Paragraph,
}

/// Resolved inline direction of the line box, independent of body bidi order.
///
/// The consumer resolves this basis for alignment and indentation. It does not
/// replace the captured paragraph base used for Unicode bidi resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineBoxDirection {
    LeftToRight,
    RightToLeft,
}

/// Actual tab context; `NoTabs` is validated against the selected sequence.
#[derive(Clone, Debug, PartialEq)]
pub enum TabContext {
    NoTabs,
    /// Actual retained-font shaping advance, without a tab-stop algorithm.
    FontAdvance,
    /// Physical nearest-block start relative to unaligned paragraph start.
    /// None means the actual used origin is unavailable; only nonzero tabs consume it.
    CssStops {
        origin: Option<f64>,
    },
    /// Logical grid at the unindented paragraph line-start for unconstrained
    /// intrinsic measurement, advancing in paragraph progression.
    IntrinsicCssStops,
}
/// The selected profile exposes only actual supported inter-space allocation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpacingPolicy {
    None,
    Spaces,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineContext {
    pub formatted_line: FormattedLineContext,
    pub base_direction: ParagraphBaseDirection,
    /// Independently resolved alignment and indentation basis.
    pub line_box_direction: LineBoxDirection,
    pub orientation: LineOrientation,
    pub start: LineStart,
    pub end: LineEnd,
    pub available_inline: Option<f64>,
    pub indent: f64,
    pub alignment: Alignment,
    pub tabs: TabContext,
    pub spacing: SpacingPolicy,
}

/// Actual formatting-container roles; unknown is distinct from affirmative false.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct FormattedLineContext {
    first: Option<bool>,
    last: Option<bool>,
}
impl FormattedLineContext {
    pub const fn new(first: Option<bool>, last: Option<bool>) -> Self {
        Self { first, last }
    }
    pub const fn first(self) -> Option<bool> {
        self.first
    }
    pub const fn last(self) -> Option<bool> {
        self.last
    }
}

/// Roles actually consumed by emitted line measurements, including false roles.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FormattedLineUsage {
    pub(crate) first: bool,
    pub(crate) last: bool,
}
impl FormattedLineUsage {
    pub const fn first(self) -> bool {
        self.first
    }
    pub const fn last(self) -> bool {
        self.last
    }
}

/// Signed flow contributions, distinct from nonnegative border/content sizes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicFlowAllocation {
    pub advance: f64,
    pub before_baseline: f64,
    pub after_baseline: f64,
    pub occupied_start: f64,
    pub occupied_end: f64,
}
/// Box geometry relative to the actual final flow slot, including its resolved
/// baseline/top/bottom baseline coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtomicBoxAllocation {
    pub border: Rect,
    pub content: Rect,
    pub baseline: f64,
}
/// Line-time geometry exists only for actual in-flow participation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtomicLineAllocation {
    InFlow {
        flow: AtomicFlowAllocation,
        box_geometry: AtomicBoxAllocation,
        alignment: LineBoxAlignment,
    },
    OutOfFlow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomicAllocation {
    pub occurrence: PreparedAtomic,
    pub allocation: AtomicLineAllocation,
}

/// Alignment of an actual Layout-allocated atomic box or control strut.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LineBoxAlignment {
    Baseline,
    Top,
    Bottom,
}

/// Coupled allocated control geometry; these are Layout facts, not font metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlStrut {
    pub baseline: f64,
    pub line_extent: f64,
    pub alignment: LineBoxAlignment,
}

/// One exact selected control, including explicit absence of a strut.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlAllocation {
    pub occurrence: PreparedControl,
    pub strut: Option<ControlStrut>,
    pub boundary: Option<InlineBoundaryAllocation>,
}

/// One endpoint's actual used inline advance, already resolved by its consumer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryAllocation {
    advance: f64,
}
impl InlineBoundaryAllocation {
    pub const ZERO: Self = Self { advance: 0.0 };

    pub fn try_new(advance: f64) -> Result<Self> {
        let category = if !advance.is_finite() {
            Some(crate::TextNumericError::NonFinite)
        } else if advance < 0.0 {
            Some(crate::TextNumericError::OutOfRange)
        } else {
            None
        };
        if let Some(category) = category {
            return Err(crate::error::text_numeric_error(
                "inline boundary advance",
                crate::TextNumericPhase::Selection,
                category,
                advance,
            ));
        }
        Ok(Self {
            advance: if advance == 0.0 { 0.0 } else { advance },
        })
    }

    #[must_use]
    pub const fn advance(&self) -> f64 {
        self.advance
    }
}

/// A demanded off-selection endpoint fact; it allocates no flow or output.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineBoundaryContextAllocation {
    pub occurrence: PreparedControl,
    pub boundary: InlineBoundaryAllocation,
}

/// Actual per-line allocated geometry, combined with body and paragraph struts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineStrut {
    pub baseline: f64,
    pub line_extent: f64,
}

/// Whether an indication is an ordinary offer or only staged restoration data.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PreparedAlternativeAvailability {
    Ordinary,
    AutoPhraseSuppressed,
}

/// An actual ordinary opportunity or an interval-bound conditional capability.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PreparedTermination {
    Ordinary(PreparedOpportunity),
    Conditional(PreparedConditionalOpportunity),
    /// Exact lexical portion trial through the shared finalizer; never placeable.
    Measurement(PreparedWordPortion),
    /// Authentic whole remaining phrase trial; never a placeable termination.
    PhraseMeasurement(PreparedPhrasePortion),
}

/// Complete request; all capabilities and numerical/context facts are checked
/// together by `PreparedText::select` before a selected result is issued.
#[derive(Clone, Debug, PartialEq)]
pub struct LineSelectionRequest {
    pub start: PreparedBoundary,
    pub end: PreparedBoundary,
    pub participants: Vec<PreparedParticipant>,
    pub termination: PreparedTermination,
    pub alternative: Option<PreparedAlternative>,
    /// Exact choice whose nonempty After emission is owed by the preceding selection.
    pub incoming: Option<HyphenationContinuation>,
    pub context: LineContext,
    pub atomics: Vec<AtomicAllocation>,
    pub controls: Vec<ControlAllocation>,
    pub boundary_context: Vec<InlineBoundaryContextAllocation>,
    pub line_strut: Option<LineStrut>,
}

#[derive(Clone)]
pub struct SelectedLine {
    pub(crate) inner: Arc<SelectedData>,
}
impl fmt::Debug for SelectedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectedLine").finish_non_exhaustive()
    }
}
impl PartialEq for SelectedLine {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for SelectedLine {}

/// Slot issued by an exact selected request, never by a glyph-array ordinal.
#[derive(Clone)]
pub struct SpacingSlot {
    pub(crate) selected: Arc<SelectedData>,
    pub(crate) index: usize,
}
impl fmt::Debug for SpacingSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpacingSlot")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}
impl PartialEq for SpacingSlot {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.selected, &other.selected)
    }
}
impl Eq for SpacingSlot {}

/// The actual preparation-issued owner of one selected presentation SPACE.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpacingSlotOwner {
    Unit(PreparedUnit),
    Alternative(PreparedAlternative),
}

/// The executed line-edge and trailing classification of a selected SPACE.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpacingSlotDisposition {
    DiscardedAtLineEdge,
    Retained { trailing: bool },
}

/// Immutable finalization feedback for one exact selected slot. `advance` is
/// the resulting logical contribution, including its admitted allocation;
/// canonical provider glyph advances remain available through glyph readback.
#[derive(Clone, Debug, PartialEq)]
pub struct SpacingSlotFeedback {
    pub slot: SpacingSlot,
    pub disposition: SpacingSlotDisposition,
    pub advance: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpacingAllocation {
    pub slot: SpacingSlot,
    pub amount: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SpacingAllocations {
    /// The actual zero-extra first pass; no implicit automatic justification.
    None,
    /// Exactly one actual amount, including zeros, for every issued slot.
    Complete(Vec<SpacingAllocation>),
}

/// One immutable selected-line result, retaining its exact request/preparation.
#[derive(Clone)]
pub struct FinalizedLine {
    pub(crate) inner: Arc<crate::engine::FinalizedLine>,
}
impl fmt::Debug for FinalizedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FinalizedLine").finish_non_exhaustive()
    }
}
impl PartialEq for FinalizedLine {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for FinalizedLine {}
#[derive(Clone)]
pub struct FinalizedFragment {
    pub(crate) finalized: Arc<crate::engine::FinalizedLine>,
    pub(crate) index: usize,
}
impl fmt::Debug for FinalizedFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FinalizedFragment")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}
impl PartialEq for FinalizedFragment {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.finalized, &other.finalized)
    }
}
impl Eq for FinalizedFragment {}

/// Physical location of local x=0 on the line baseline, not its top-left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineAnchor {
    pub inline_origin: f64,
    pub baseline: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AtomicPlacement {
    pub occurrence: PreparedAtomic,
    pub border: Rect,
    pub content: Rect,
    pub baseline: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LinePlacement {
    pub line: FinalizedLine,
    pub anchor: LineAnchor,
    pub orientation: LineOrientation,
    pub atomics: Vec<AtomicPlacement>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PlacementCoverage {
    WholeParagraph,
    Range {
        start: PreparedBoundary,
        end: PreparedBoundary,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlacementRequest {
    pub prepared: PreparedText,
    pub coverage: PlacementCoverage,
    pub lines: Vec<LinePlacement>,
}

/// Feedback is line-local, and never declares that a consumer's candidate fits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineFeedback {
    pub advance: f64,
    pub occupied_start: f64,
    pub occupied_end: f64,
    /// Descriptive retained trailing whitespace; never a general subtraction
    /// instruction. Use the owner-issued purpose-specific measurements.
    pub trailing_whitespace: f64,
    /// Signed excluded contribution at the physical start edge. Physical
    /// outward overlap is observed separately through `edge_units()`.
    pub hanging_start: f64,
    /// Signed excluded contribution at the physical end edge.
    pub hanging_end: f64,
    pub ascent: f64,
    pub descent: f64,
    pub line_height: f64,
    pub baseline: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AtomicSlotFeedback {
    pub occurrence: PreparedAtomic,
    /// Actual line-local flow baseline, before physical line-anchor mapping.
    pub origin: Point,
    pub allocation: AtomicLineAllocation,
}

/// Semantic capability kind used by an exact protocol admission diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolHandleKind {
    WordPortion,
    Origin,
    Unit,
    Atomic,
    Control,
    Boundary,
    Opportunity,
    ConditionalOpportunity,
    Alternative,
    Fragment,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolContextField {
    BaseDirection,
    Orientation,
    LineStart,
    LineEnd,
    TabStops,
    TabPreparation,
    TabReference,
    TabMetrics,
    CssParagraphBoundary,
    Spacing,
}
/// Actual invalid association or incomplete input; no caller ordinal issues a capability.
#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolError {
    InvalidMeasurementFlow,
    /// The queried range does not belong to one canonical body bidi context.
    InvalidLineDirectionQuery,
    InvalidWordPortion,
    MeasurementNotPlaceable,
    InvalidPhrasePortion,
    InvalidConditionalStage,
    /// Missing actual context for CSS Text Phase II: Trimming and Positioning.
    MissingPhaseTwoContext(PhaseTwoContext),
    /// The occurrence-qualified original seam is not represented by this line.
    OutsideFinalizedLine,
    InvalidOriginSeam {
        position: usize,
    },
    ForeignPreparation {
        kind: ProtocolHandleKind,
        occurrence: usize,
    },
    ForeignSelectedLine,
    ForeignFinalizedLine {
        line: usize,
    },
    InvalidOriginRange {
        origin: usize,
        local: Range,
        aggregate: Range,
    },
    OriginTextMismatch {
        origin: usize,
        local: usize,
        aggregate: usize,
    },
    IncompleteOriginCoverage {
        expected: usize,
        actual: usize,
    },
    MissingConnection {
        before: usize,
        after: usize,
    },
    InvalidConnection {
        connection: usize,
        at: usize,
    },
    InvalidControl {
        control: usize,
        at: usize,
    },
    InvalidBoundaryPolicy {
        policy: usize,
        at: usize,
    },
    /// A second group claims the same genuine defining occurrence.
    DuplicateReplacement {
        alternative: usize,
        prior: usize,
    },
    InvalidReplacement {
        alternative: usize,
        source: Range,
    },
    InvalidBoundaryRange {
        start: usize,
        end: usize,
        participants: usize,
    },
    ParticipantMismatch {
        position: usize,
    },
    MissingAtomic {
        occurrence: usize,
    },
    DuplicateAtomic {
        occurrence: usize,
    },
    IllegalOpportunity {
        opportunity: usize,
    },
    InvalidConditionalQuery {
        start: usize,
        ceiling: usize,
    },
    InvalidConditionalSelection {
        start: usize,
        end: usize,
    },
    InvalidConditionalContext {
        purpose: ConditionalPurpose,
    },
    InvalidConditionalReceipt {
        receipt: usize,
        opportunity: usize,
    },
    RetainedConditionalPrefix {
        opportunity: usize,
        alternative: Option<usize>,
    },
    UnsupportedWrappingAnalysis,
    MissingWrappingPolicy {
        relation: usize,
    },
    ForeignWrappingRelation {
        relation: usize,
    },
    DuplicateWrappingPolicy {
        relation: usize,
    },
    ConflictingWrappingPolicy {
        relation: usize,
    },
    ForeignAlternative {
        alternative: usize,
        opportunity: usize,
    },
    UnsupportedContext {
        field: ProtocolContextField,
    },
    MismatchedContext {
        field: ProtocolContextField,
    },
    InvalidAtomicGeometry {
        occurrence: usize,
    },
    DuplicateSpacingSlot {
        slot: usize,
    },
    MissingSpacingSlot {
        slot: usize,
    },
    IneligibleSpacingSlot {
        slot: usize,
    },
    UnexpectedAtomic {
        occurrence: usize,
    },
    MissingAlternative {
        opportunity: usize,
    },
    InvalidAnchorOrder {
        position: usize,
        kind: ProtocolHandleKind,
        occurrence: usize,
    },
    MissingControl {
        occurrence: usize,
    },
    MissingControlBoundaryGeometry {
        occurrence: usize,
    },
    MissingBoundaryContext {
        occurrence: usize,
    },
    DuplicateBoundaryContext {
        occurrence: usize,
    },
    UnexpectedBoundaryContext {
        occurrence: usize,
    },
    DuplicateControl {
        occurrence: usize,
    },
    UnexpectedControl {
        occurrence: usize,
    },
    InvalidControlGeometry {
        occurrence: usize,
    },
    MissingLineMetrics,
    InvalidLineStrut,
    UnsupportedStandaloneAnalysis,
    ForeignPlacement,
    OutsidePlacement {
        index: usize,
        coverage: Range,
    },
    AmbiguousSourcePosition {
        index: usize,
        affinity: crate::Affinity,
    },
    InvalidPlacedPosition {
        position: usize,
    },
    EmptyPlacement,
    IncompleteLineCoverage {
        line: usize,
    },
    DuplicateLine {
        line: usize,
    },
    InvalidFragment {
        line: usize,
        fragment: usize,
    },
    AtomicPlacementMismatch {
        line: usize,
        occurrence: usize,
    },
}

/// Actual structural fact consumed by native CSS Text Phase II (Trimming and
/// Positioning) measurements, not an implementation or delivery-stage marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhaseTwoContext {
    FirstFormattedLine,
    LastFormattedLine,
    TerminalBlockEdge,
    TabOrigin,
}

pub(crate) fn protocol_error(reason: ProtocolError) -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::InvalidTextProtocol,
        "invalid text phase association or context",
    )
    .with_detail(crate::ErrorDetail::InvalidTextProtocol { reason })
}

/// Selected-presentation code-unit extent, distinct from authored/processed ranges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentationRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}
impl PresentationRange {
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[cfg(test)]
mod option_tests {
    use super::*;
    #[test]
    fn preparation_options_classifies_actual_input_domain_without_conversion_loss() {
        for value in [0.0_f32, -0.0, -1.0] {
            let error =
                PreparationOptions::try_new(value, crate::TabPreparation::FontAdvance).unwrap_err();
            assert!(
                matches!(error.detail(), Some(crate::ErrorDetail::InvalidTextNumeric {
                category: crate::TextNumericError::OutOfRange, value: actual, ..
            }) if *actual == f64::from(value))
            );
        }
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let error =
                PreparationOptions::try_new(value, crate::TabPreparation::FontAdvance).unwrap_err();
            assert!(matches!(
                error.detail(),
                Some(crate::ErrorDetail::InvalidTextNumeric {
                    category: crate::TextNumericError::NonFinite,
                    ..
                })
            ));
        }
        assert_eq!(
            PreparationOptions::try_new(0.5, crate::TabPreparation::FontAdvance)
                .unwrap()
                .scale(),
            0.5
        );
    }
}

impl SelectedLine {
    #[must_use]
    pub fn prepared_text(&self) -> PreparedText {
        PreparedText {
            inner: Arc::clone(&self.inner.prepared),
        }
    }
}
impl FinalizedLine {
    #[must_use]
    pub fn prepared_text(&self) -> PreparedText {
        PreparedText {
            inner: Arc::clone(&self.inner.selected.prepared),
        }
    }
}
impl Hash for FinalizedLine {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state);
    }
}

#[cfg(test)]
mod tests;
