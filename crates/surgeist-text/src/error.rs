use std::{error, fmt};

use crate::{
    style::WhiteSpace,
    style_support::{TextStyleFeature, UnsupportedTextStyleReason},
};
use surgeist_font::CataloguePublication;

mod upright;
pub use upright::{
    TextScript, TextStyleOwner, UprightContent, UprightContentPiece, UprightDecorationRole,
    UprightPresentation, UprightScriptContext, UprightScriptContextFailure, UprightScriptFailure,
    UprightTabMeasurement, VerticalOriginReferenceReason, VerticalPositionStage,
    VerticalShapingContextReason,
};

/// Text crate result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Provider-observation domain behind a material spacing demand.
/// SelectedShaping also identifies retained selected output reused without reshaping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpacingStage {
    /// Admission of a source-owned complete-basis request or its receipt-qualified topology.
    Selection,
    Preparation,
    SelectedShaping,
}

/// The material spacing contribution, independently of its source ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpacingOperation {
    InterCharacterBasis,
    AuthoredWord,
    AuthoredTracking,
    InterCharacterAllocation,
    InterWordAllocation,
}

/// Ordered actual content of an unsupported spacing target. Ordinals describe
/// the executed phase's inventory and do not issue capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpacingPiece {
    Atomic {
        occurrence: usize,
    },
    Source {
        origin: usize,
        local: crate::range::Range,
        aggregate: crate::range::Range,
    },
    Alternative {
        alternative: usize,
        local: crate::range::Range,
        insertion_anchor: usize,
    },
    Control {
        occurrence: usize,
    },
    VirtualSeparator {
        occurrence: usize,
    },
}

/// Capability needed by an actual material spacing operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpacingFailureReason {
    InterleavedUnit,
    UnqualifiedWritingSystem,
    UnqualifiedProviderPath,
    PreservedOptionalLigature,
    CrossUnitConstraint,
}

/// Actual relation or observed context scope constraining independent movement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpacingConstraintKind {
    Ligature,
    MarkAttachment,
    CursiveAttachment,
    UnqualifiedProviderPath(surgeist_shaper::ComponentPath),
    /// Joining actions used context across separately emitted invocations. This
    /// identifies an unqualified movement dependency, not an observed attachment.
    /// Both ranges use the actual preparation or selected-presentation atoms.
    ExternalJoiningContext {
        body: std::ops::Range<usize>,
        context: std::ops::Range<usize>,
    },
}

/// Governing source context of one endpoint in a rejected spacing operation.
#[derive(Clone, Debug, PartialEq)]
pub struct SpacingContext {
    pub pieces: Box<[SpacingPiece]>,
    pub locale: Option<String>,
    pub candidates: Box<[TextScript]>,
    pub writing_systems: Box<[TextScript]>,
    pub resolved_scripts: Box<[TextScript]>,
    pub letter_spacing_before: f64,
    pub letter_spacing_after: f64,
}

/// Actual provider relation retained for a rejected spacing target.
#[derive(Clone, Debug, PartialEq)]
pub struct SpacingConstraint {
    pub kind: SpacingConstraintKind,
    pub run: usize,
    pub components: Box<[std::ops::Range<usize>]>,
    pub provenance: Box<[surgeist_shaper::FeatureProvenance]>,
}

/// Stable text diagnostic.
///
/// `Display` prints [`Self::message`] and nothing else. The classification is
/// read through [`Self::code`] and the typed context through [`Self::detail`];
/// [`Self::source`] holds the underlying error, which a reporter that walks the
/// chain prints separately.
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    detail: Option<Box<ErrorDetail>>,
    pub source: Option<Box<dyn error::Error + Send + Sync>>,
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            source: None,
        }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: ErrorDetail) -> Self {
        self.detail = Some(Box::new(detail));
        self
    }

    #[must_use]
    pub const fn detail(&self) -> Option<&ErrorDetail> {
        match &self.detail {
            Some(detail) => Some(detail),
            None => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref().map(|error| error as _)
    }
}

impl From<surgeist_font::DecodeError> for Error {
    fn from(error: surgeist_font::DecodeError) -> Self {
        Self::new(ErrorCode::FontLoadFailed, error.to_string())
            .with_detail(ErrorDetail::FontDecode(error))
    }
}

impl From<surgeist_font::CatalogueError> for Error {
    fn from(error: surgeist_font::CatalogueError) -> Self {
        Self::new(ErrorCode::FontLoadFailed, error.to_string())
            .with_detail(ErrorDetail::FontCatalogue(error))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// Invalid complete contextual source-processing capture.
    InvalidSourceProcessing,
    /// Captured-scope alignment could not admit its facts, resources or allocation.
    ScopeAlignmentRefused,
    /// Final semantic phrase analysis received invalid retained model data.
    InvalidPhraseAnalysis,
    /// Font-independent text analysis could not retain its complete result.
    TextAnalysisCapacity,
    /// Font resolution found no usable face for required shaping or metrics.
    NoUsableFontFace,
    /// Actual shaping declined required work or representability.
    ShapingRefused,
    /// A logical or canonical geometry conversion lost its numeric category.
    InvalidTextNumeric,
    /// Invalid phase capability, captured occurrence, or complete line context.
    InvalidTextProtocol,
    FontLoadFailed,
    /// An edit cannot publish another revision of the source.
    SourceRevisionExhausted,
    /// Source or mapped index arithmetic exceeded its representable domain.
    SourceLengthOverflow,
    /// Actual shaping metadata cannot describe a complete font instance.
    InvalidFontInstance,
    /// Source processing could not represent or allocate its complete result.
    SourceProcessingCapacity,
    /// Resource provenance and the expected operation basis differ.
    FontCatalogueMismatch,
    /// Used font-metric metadata or numeric output is invalid.
    InvalidFontMetrics,
    /// Font metric scratch size or allocation cannot be admitted.
    FontMetricCapacity,
    /// Same-instance glyph outline geometry could not be admitted.
    FontOutlineRefused,
    InvalidRange,
    InvalidStyle,
    LayoutFailed,
    HitTestFailed,
    UnsupportedFeature,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorDetail {
    /// Source-owned text-combination context or preparation capability refusal.
    TextCombine {
        reason: crate::transform::TextCombineError,
    },
    /// A typed captured-scope alignment refusal from its owning phase.
    ScopeAlignment {
        reason: crate::processing::contextual::ScopeAlignmentError,
    },
    /// A material spacing operation needs unavailable profile or movement capability.
    UnsupportedSpacing {
        stage: SpacingStage,
        operation: SpacingOperation,
        pieces: Box<[SpacingPiece]>,
        /// Exact atom pieces in the actual preparation or selected presentation.
        provider_atoms: Box<[std::ops::Range<usize>]>,
        reason: SpacingFailureReason,
        contexts: Box<[SpacingContext]>,
        constraints: Box<[SpacingConstraint]>,
    },
    /// Executed pre-analysis transformation could not retain its operation.
    TextTransform {
        reason: TextTransformError,
    },
    /// Exact native hyphenation input/resource refusal.
    Hyphenation {
        reason: crate::hyphenation::HyphenationError,
    },
    /// Admission or retained-association failure for a nonempty source range.
    SourceRangeFragments {
        reason: crate::range_fragments::SourceRangeFragmentError,
    },
    /// Atomic rejection of contextual source facts or view membership.
    SourceProcessing {
        reason: crate::processing::contextual::SourceProcessingError,
    },
    /// Actual phase association or request admission failure.
    InvalidTextProtocol {
        reason: crate::protocol::ProtocolError,
    },
    /// Actual glyph outline admission failed without publishing geometry.
    FontOutline {
        /// Requested glyph in the retained face.
        glyph: u32,
        /// Structural, capability, provider or storage refusal.
        reason: crate::font::FontOutlineError,
    },
    /// Nominal glyph request is outside the retained face inventory.
    FontGlyphRange {
        /// Requested glyph.
        glyph: u32,
        /// Actual glyph count.
        count: u32,
    },
    /// Vertical metrics for an actual variable face are not qualified.
    UnsupportedVerticalGlyphVariation {
        /// Requested glyph.
        glyph: u32,
        /// Actual variation axis count, including default coordinates.
        axis_count: usize,
    },
    /// The actual shaping invocation returned incomplete output.
    ShapingRefused {
        reason: ShapingRefusalReason,
    },
    /// Effective script candidates cannot use the unrotated upright profile.
    UprightScript {
        content: Box<UprightContent>,
        candidates: Box<[TextScript]>,
        contexts: Box<[UprightScriptContext]>,
        reason: UprightScriptFailure,
    },
    /// An enabled decoration needs an unavailable automatic upright metric.
    UprightDecoration {
        owner: TextStyleOwner,
        kind: crate::layout::DecorationKind,
        role: UprightDecorationRole,
    },
    /// Actual selected provider context cannot use supplied vertical positioning.
    VerticalShapingContext {
        content: UprightContent,
        reason: VerticalShapingContextReason,
    },
    /// Actual unscaled origin reference cannot enter the provider input grid.
    VerticalOriginReference {
        content: UprightContent,
        reference: crate::font::UprightMetricReference,
        value: f64,
        reason: VerticalOriginReferenceReason,
    },
    /// Actual positioning exceeded the provider's half-design-unit range.
    VerticalPositionRange {
        content: UprightContent,
        stage: VerticalPositionStage,
    },
    /// The actual post-substitution glyph's metric supplier failed.
    ///
    /// The unchanged original Text error is retained in [`Error::source`].
    VerticalMetricSupplier {
        content: UprightContent,
        glyph_id: u32,
    },
    /// Logical or canonical geometry could not be represented truthfully.
    InvalidTextNumeric {
        field: &'static str,
        phase: TextNumericPhase,
        category: TextNumericError,
        value: f64,
    },
    /// The host requires scalar UTF-8 and cannot represent this original unit.
    AccessibilityIllFormedSource {
        index: usize,
        unit: u16,
    },
    /// Associated font provenance does not match the captured operation basis.
    FontCatalogueMismatch {
        /// Expected publication.
        expected: CataloguePublication,
        /// Actual resource publication.
        actual: CataloguePublication,
    },
    /// Structural admission failed for a used metric table.
    FontMetricTable {
        /// Actual table.
        table: FontMetricTable,
        /// Precise refusal.
        reason: FontMetricTableError,
    },
    /// Neither hhea nor OS/2 provides a readable line-metric source.
    MissingFontLineMetrics,
    /// Font metric scratch size arithmetic overflowed.
    FontMetricCapacityOverflow,
    /// Fallible reservation of font metric scratch was refused.
    FontMetricAllocationRefused,
    /// Checked variation accumulation or rounding exceeded its proven domain.
    FontMetricVariationArithmetic {
        /// Actual value tag whose delta could not be represented.
        tag: [u8; 4],
    },
    /// Required metric or arithmetic result is outside its numeric domain.
    InvalidFontMetric {
        /// Named field.
        field: FontMetricField,
        /// Validation stage.
        phase: FontMetricPhase,
        /// Numeric category.
        category: FontMetricNumericError,
        /// Actual rejected value before clamping or substitution.
        value: f64,
    },
    /// An actual supplied document delimiter is invalid.
    InvalidDocumentSegmentBreak {
        /// Original delimiter-vector ordinal.
        occurrence: usize,
        /// Supplied original source range.
        range: crate::range::Range,
        /// Exact structural refusal.
        reason: DocumentSegmentBreakError,
    },
    /// A delimiter would cross a real atomic inline occurrence.
    SegmentBreakCrossesInlineBox {
        /// Original delimiter-vector ordinal.
        occurrence: usize,
        /// Original box-vector ordinal.
        box_ordinal: usize,
        /// Authentic box ID.
        id: crate::id::Id,
        /// Original source anchor.
        anchor: usize,
    },
    /// One actual document delimiter crosses a change of effective processing value.
    SegmentBreakCrossesProcessingMode {
        /// Actual delimiter-vector ordinal.
        occurrence: usize,
        /// Complete original delimiter extent.
        range: crate::range::Range,
        /// Original offset at which the effective value changes.
        at: usize,
        /// Effective value before the boundary.
        before: WhiteSpace,
        /// Effective value after the boundary.
        after: WhiteSpace,
    },
    /// Result-relative processed bounds are invalid.
    InvalidProcessedRange {
        /// Requested start or position.
        start: usize,
        /// Requested end or position.
        end: usize,
        /// Actual processed length.
        text_len: usize,
    },
    /// Complete-result capacity could not be admitted.
    ProcessingCapacity {
        /// Distinguishes arithmetic overflow from allocation refusal.
        reason: ProcessingCapacityError,
    },
    /// A present variation table failed structural or axis-range validation.
    MalformedFontVariations,
    /// Stored normalized coordinates do not cover the actual axis inventory.
    FontCoordinateCount {
        /// Actual font axes.
        expected: usize,
        /// Stored provider coordinates.
        actual: usize,
    },
    /// A stored F2Dot14 coordinate is outside the canonical normalized interval.
    FontCoordinate {
        /// Actual axis ordinal.
        axis: usize,
        /// Unchanged stored coordinate bits.
        bits: i16,
    },
    UnsupportedShapingBoundary {
        index: usize,
        kind: ShapingBoundary,
    },
    ProviderSourceRange {
        start: usize,
        end: usize,
    },
    /// A completed shaping result returned inconsistent parallel arrays.
    ProviderOutputLength {
        infos: usize,
        positions: usize,
    },
    /// A returned glyph identifier is outside the selected resource.
    ProviderGlyphBounds {
        glyph_id: u32,
        num_glyphs: u16,
    },
    UnsupportedAccessibilityUnit {
        range: crate::range::Range,
        bytes: usize,
    },
    /// Cell source membership cannot be represented by original AX units.
    UnsupportedAccessibilityCellMapping {
        /// Actual Cell binding ordinal in the analyzed output, in either phase.
        cell: usize,
        /// The original aggregate source interval whose mapping is unsupported.
        range: crate::range::Range,
    },
    AccessibilityBindingMismatch,
    AccessibilityIdentityConflict {
        id: u64,
    },
    AccessibilityIdentityExhausted,
    /// Shared Font refused the encoded container under its admission contract.
    FontDecode(surgeist_font::DecodeError),
    /// Shared Font refused a registration, publication or candidate query.
    FontCatalogue(surgeist_font::CatalogueError),
    InvalidFontFace {
        index: u32,
    },
    /// The source revision that has no successor.
    SourceRevisionExhausted {
        /// The exhausted revision.
        revision: u64,
    },
    InvalidSourceRange {
        start: usize,
        end: usize,
        text_len: usize,
    },
    InvalidSourceIndex {
        name: &'static str,
        index: usize,
        text_len: usize,
    },
    InvalidNumericField {
        field: &'static str,
        value: f64,
        requirement: NumericRequirement,
    },
    UnsupportedTextStyle {
        feature: TextStyleFeature,
        reason: UnsupportedTextStyleReason,
    },
    UnsupportedCombination {
        feature: &'static str,
        reason: &'static str,
    },
}

/// Exact source-transform admission or execution refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextTransformError {
    /// Required original-unit envelope for diagnostics. Actual original pieces,
    /// rather than this envelope, establish membership in the requested view.
    /// The transform cannot keep that indivisible unit whole: a structural
    /// frontier would divide it, or its transformed unit exceeds its exact
    /// output coverage.
    IndivisibleCoverage { source: crate::range::Range },
    /// The retained phrase model data is invalid.
    InvalidPhraseData,
    /// Phrase separator insertion at `at` has no captured scope topology.
    MissingPhraseTopology { at: usize },
    /// The captured scope has no style from which to insert a phrase separator.
    MissingPhraseScopeStyle { scope: usize, at: usize },
    /// A separator control lacks the boundary event order of the connection.
    MissingSeparatorBoundaryContext {
        control: usize,
        at: usize,
        connection: usize,
    },
    /// An original separator's forced adjacency depends on an uncaptured
    /// connection side. The range identifies original text, never a control;
    /// `at` is the position of the connection.
    MissingOriginalSeparatorBoundaryContext {
        source: crate::range::Range,
        at: usize,
        connection: usize,
    },
    /// The separator control sits inside the retained original unit, so it has
    /// no exact scalar seam.
    UnrepresentableSeparatorSeam {
        control: usize,
        at: usize,
        retained: crate::range::Range,
    },
    /// The text-node range is reversed or extends beyond the node's content,
    /// measured in UTF-16 code units.
    InvalidTextNodeRange {
        range: crate::range::Range,
        text_len: usize,
    },
    /// The text-node content differs from its original occurrence.
    TextNodeMismatch { origin: usize },
    /// The transform requires complete original text-node context that the
    /// origin does not supply.
    MissingTextNodeContext { origin: usize },
    /// The casing data is unavailable.
    CasingDataUnavailable,
    /// The casing data holds an invalid exception or scalar.
    InvalidCasingData,
    /// The content locale is not a valid language tag.
    InvalidContentLocale,
    /// The transform could not reserve storage for its result.
    Capacity,
}

impl fmt::Display for TextTransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndivisibleCoverage { source } => write!(
                f,
                "the transform cannot keep the indivisible original unit {}..{} whole",
                source.start, source.end
            ),
            Self::InvalidPhraseData => f.write_str("the retained phrase model data is invalid"),
            Self::MissingPhraseTopology { at } => write!(
                f,
                "phrase separator insertion at {at} needs the captured scope topology"
            ),
            Self::MissingPhraseScopeStyle { scope, at } => write!(
                f,
                "phrase separator insertion at {at} needs the style of scope {scope}"
            ),
            Self::MissingSeparatorBoundaryContext {
                control,
                at,
                connection,
            } => write!(
                f,
                "separator control {control} at {at} lacks the boundary event order of connection {connection}"
            ),
            Self::MissingOriginalSeparatorBoundaryContext {
                source,
                at,
                connection,
            } => write!(
                f,
                "the original separator {}..{} depends on an uncaptured side of connection {connection} at {at}",
                source.start, source.end
            ),
            Self::UnrepresentableSeparatorSeam {
                control,
                at,
                retained,
            } => write!(
                f,
                "separator control {control} at {at} lies inside the retained unit {}..{} and has no exact scalar seam",
                retained.start, retained.end
            ),
            Self::InvalidTextNodeRange { range, text_len } => write!(
                f,
                "text-node range {}..{} is reversed or extends beyond the node's UTF-16 length {text_len}",
                range.start, range.end
            ),
            Self::TextNodeMismatch { origin } => write!(
                f,
                "the text-node content for origin {origin} differs from its original occurrence"
            ),
            Self::MissingTextNodeContext { origin } => write!(
                f,
                "the transform requires complete original text-node context for origin {origin}"
            ),
            Self::CasingDataUnavailable => f.write_str("the casing data is unavailable"),
            Self::InvalidCasingData => f.write_str("the casing data is invalid"),
            Self::InvalidContentLocale => {
                f.write_str("the content locale is not a valid language tag")
            }
            Self::Capacity => f.write_str("the transform could not reserve storage for its result"),
        }
    }
}

impl error::Error for TextTransformError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IndivisibleCoverage { .. }
            | Self::InvalidPhraseData
            | Self::MissingPhraseTopology { .. }
            | Self::MissingPhraseScopeStyle { .. }
            | Self::MissingSeparatorBoundaryContext { .. }
            | Self::MissingOriginalSeparatorBoundaryContext { .. }
            | Self::UnrepresentableSeparatorSeam { .. }
            | Self::InvalidTextNodeRange { .. }
            | Self::TextNodeMismatch { .. }
            | Self::MissingTextNodeContext { .. }
            | Self::CasingDataUnavailable
            | Self::InvalidCasingData
            | Self::InvalidContentLocale
            | Self::Capacity => None,
        }
    }
}

/// Structural invalidity of one supplied document delimiter occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentSegmentBreakError {
    /// The occurrence has no source units.
    Empty,
    /// The endpoints are reversed.
    Reversed,
    /// At least one endpoint exceeds captured source bounds.
    OutOfBounds,
    /// The occurrence starts before the previous supplied occurrence starts.
    Unordered,
    /// The occurrence starts inside the previous supplied occurrence.
    Overlap,
}

impl fmt::Display for DocumentSegmentBreakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "the delimiter occurrence covers no source units",
            Self::Reversed => "the delimiter occurrence ends before it starts",
            Self::OutOfBounds => "the delimiter occurrence extends beyond the source",
            Self::Unordered => {
                "the delimiter occurrence starts before the previous occurrence starts"
            }
            Self::Overlap => "the delimiter occurrence starts inside the previous occurrence",
        })
    }
}

impl error::Error for DocumentSegmentBreakError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Empty | Self::Reversed | Self::OutOfBounds | Self::Unordered | Self::Overlap => {
                None
            }
        }
    }
}

/// Capacity failure during off-side source processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessingCapacityError {
    /// Checked count or byte arithmetic exceeded the representable domain.
    Overflow,
    /// A fallible buffer reservation was refused.
    Allocation,
}

impl fmt::Display for ProcessingCapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Overflow => {
                "a source-processing count or byte size exceeded the representable range"
            }
            Self::Allocation => "a source-processing buffer reservation was refused",
        })
    }
}

impl error::Error for ProcessingCapacityError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Overflow | Self::Allocation => None,
        }
    }
}

/// A shaping input boundary that must not divide a surrogate pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapingBoundary {
    SpanStart,
    SpanEnd,
    InlineBox,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericRequirement {
    Finite,
    FiniteNonNegative,
    FiniteGreaterThanZero,
    UnitInterval,
}

/// Tables consumed by same-instance font metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricTable {
    /// `head`.
    Head,
    /// `hhea`.
    Hhea,
    /// `OS/2`.
    Os2,
    /// `MVAR`.
    Mvar,
    /// `maxp`.
    Maxp,
    /// `hmtx`.
    Hmtx,
    /// `vhea`.
    Vhea,
    /// `vmtx`.
    Vmtx,
    /// `loca`.
    Loca,
    /// `glyf` nominal header reference.
    Glyf,
    /// `VVAR`.
    Vvar,
    /// `BASE` named design baselines.
    Base,
}
/// Used metric-table refusal.
///
/// The enum names the cause. The table that was refused is named by the
/// [`ErrorDetail::FontMetricTable`] value that carries it, so the messages
/// speak of "the table".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricTableError {
    /// Required table is absent.
    Missing,
    /// A present member lacks its required companion.
    MissingCompanion {
        /// Required absent table.
        required: FontMetricTable,
    },
    /// Long metric count is outside the glyph inventory: it is zero or exceeds
    /// the glyph count.
    LongMetricCount {
        /// Declared long metric count.
        count: u16,
        /// Actual glyph count.
        glyph_count: u32,
    },
    /// Multiple directory entries claim the table.
    Duplicate,
    /// Recorded range is inaccessible.
    Inaccessible,
    /// Contents are truncated or malformed.
    Malformed,
    /// Version is outside the admitted domain.
    UnsupportedVersion,
    /// A used metric variation source has no real fvar axis inventory.
    VariationWithoutAxes,
    /// Value-record layout exceeds the supported eight-byte format.
    UnsupportedRecordSize {
        /// Actual declared stride.
        size: u16,
    },
    /// Value tags are duplicated or not strictly ascending.
    VariationRecordOrder {
        /// Actual ordinal of the record whose tag does not follow its predecessor's.
        record: u16,
    },
    /// A declared store or data offset cannot be resolved completely.
    VariationOffset {
        /// Actual relative offset.
        offset: u32,
    },
    /// Store format is outside the admitted format1 domain.
    VariationStoreFormat {
        /// Actual declared format.
        format: u16,
    },
    /// Two axis counts that must agree differ: the face's axes and the region
    /// list's axes, or the region list's axes and the face's coordinates.
    VariationAxisCount {
        /// The count that was required.
        expected: usize,
        /// The count that was found.
        actual: u16,
    },
    /// Region count is outside the format's domain.
    VariationRegionCount {
        /// Actual declared count.
        count: u16,
    },
    /// A tuple coordinate is outside the canonical F2Dot14 domain.
    VariationRegionCoordinate {
        /// Actual region ordinal.
        region: u16,
        /// Actual axis ordinal.
        axis: u16,
        /// Rejected coordinate bits.
        value: i16,
    },
    /// A record's outer or nonnull inner index is outside its declaration.
    VariationDeltaIndex {
        /// Actual value record ordinal.
        record: u16,
        /// Actual outer index.
        outer: u16,
        /// Actual inner index.
        inner: u16,
    },
    /// A data subtable refers outside the actual region list.
    VariationRegionIndex {
        /// Store-relative data offset.
        offset: u32,
        /// Rejected region index.
        index: u16,
    },
    /// A data row declares more wide deltas than region entries.
    VariationWordCount {
        /// Store-relative data offset.
        offset: u32,
        /// Declared wide-delta count, excluding the LONG_WORDS flag.
        words: u16,
        /// Declared region-index count.
        regions: u16,
    },
    /// A referenced row does not yield every declared delta.
    VariationRow {
        /// Store-relative data offset.
        offset: u32,
        /// Referenced row index.
        inner: u16,
        /// Declared delta count.
        expected: u16,
        /// Actual accessible delta count.
        actual: usize,
    },
    /// A qualified provider region scalar is outside zero through one.
    VariationScalar {
        /// Actual region ordinal.
        region: u16,
        /// Rejected 16.16 scalar bits.
        value: i32,
    },
    /// BASE baseline tags are duplicated or not strictly sorted.
    BaselineTagOrder,
    /// BASE script tags are duplicated or not strictly sorted.
    BaselineScriptOrder,
    /// The selected script's coordinates do not cover its tag inventory.
    BaselineValueCount { expected: u16, actual: u16 },
    /// A present default index is outside the actual coordinate inventory.
    BaselineDefaultIndex { index: u16, count: u16 },
    /// The used coordinate representation is not qualified.
    BaselineCoordinateFormat { format: u16 },
    /// The used Device/VariationIndex format is invalid.
    BaselineDeviceFormat { format: u16 },
    /// A real Device is forbidden in a variable BASE table.
    BaselineVariableDevice,
    /// A used VariationIndex has no actual BASE variation store.
    BaselineVariationStoreMissing,
    /// A used stored outline cannot validate the actual point reference.
    ///
    /// `source()` returns the outline refusal; the message names only the
    /// glyph and point that could not be validated.
    BaselineOutline {
        glyph: u16,
        point: u16,
        reason: crate::font::FontOutlineError,
    },
    /// The point is outside the glyph's stored point inventory.
    BaselinePointRange {
        glyph: u16,
        point: u16,
        count: usize,
    },
    /// Actual run script metadata has no canonical four-letter Unicode short name.
    BaselineScriptUnavailable,
    /// OpenType units-per-em is outside 16 through 16384.
    InvalidUnitsPerEm {
        /// Actual rejected value.
        value: u16,
    },
}

impl fmt::Display for FontMetricTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("a required table is absent"),
            Self::MissingCompanion { .. } => {
                f.write_str("a present table lacks its required companion table")
            }
            Self::LongMetricCount { count, glyph_count } => write!(
                f,
                "long metric count {count} must be at least 1 and at most the glyph count {glyph_count}"
            ),
            Self::Duplicate => f.write_str("more than one directory entry claims the table"),
            Self::Inaccessible => f.write_str("the table's recorded range cannot be read"),
            Self::Malformed => f.write_str("the table is truncated or malformed"),
            Self::UnsupportedVersion => f.write_str("the table version is not supported"),
            Self::VariationWithoutAxes => f.write_str(
                "a metric variation source is present but the face has no variation axes",
            ),
            Self::UnsupportedRecordSize { size } => write!(
                f,
                "the value-record size {size} is larger than the supported eight bytes"
            ),
            Self::VariationRecordOrder { record } => write!(
                f,
                "value record {record} has a tag that is not greater than the previous record's tag"
            ),
            Self::VariationOffset { offset } => {
                write!(f, "variation offset {offset} cannot be resolved")
            }
            Self::VariationStoreFormat { format } => write!(
                f,
                "variation store format {format} is not the supported format 1"
            ),
            Self::VariationAxisCount { expected, actual } => write!(
                f,
                "the variation axis count is {actual} where {expected} was expected"
            ),
            Self::VariationRegionCount { count } => {
                write!(f, "variation region count {count} is not below 32768")
            }
            Self::VariationRegionCoordinate { region, axis, .. } => write!(
                f,
                "variation region {region} axis {axis} has a coordinate outside the range minus one to one"
            ),
            Self::VariationDeltaIndex {
                record,
                outer,
                inner,
            } => write!(
                f,
                "record {record} refers to delta set {outer}:{inner}, which its variation store does not contain"
            ),
            Self::VariationRegionIndex { offset, index } => write!(
                f,
                "the data subtable at offset {offset} refers to region {index}, outside the region list"
            ),
            Self::VariationWordCount {
                offset,
                words,
                regions,
            } => write!(
                f,
                "the data subtable at offset {offset} declares a wide-delta count of {words} but a region count of only {regions}"
            ),
            Self::VariationRow {
                offset,
                inner,
                expected,
                actual,
            } => write!(
                f,
                "row {inner} of the data subtable at offset {offset} yields a delta count of {actual} where {expected} is declared"
            ),
            Self::VariationScalar { region, .. } => write!(
                f,
                "the scalar of variation region {region} is outside zero through one"
            ),
            Self::BaselineTagOrder => {
                f.write_str("baseline tags are duplicated or not strictly sorted")
            }
            Self::BaselineScriptOrder => {
                f.write_str("baseline script tags are duplicated or not strictly sorted")
            }
            Self::BaselineValueCount { expected, actual } => write!(
                f,
                "the selected script's coordinate count {actual} differs from its baseline tag count {expected}"
            ),
            Self::BaselineDefaultIndex { index, count } => write!(
                f,
                "default baseline index {index} is outside the coordinate count {count}"
            ),
            Self::BaselineCoordinateFormat { format } => {
                write!(f, "baseline coordinate format {format} is not 1, 2 or 3")
            }
            Self::BaselineDeviceFormat { format } => {
                write!(f, "baseline device format {format} is invalid")
            }
            Self::BaselineVariableDevice => f.write_str(
                "a device table is not allowed in the baseline table of a variable face",
            ),
            Self::BaselineVariationStoreMissing => f.write_str(
                "a variation index is used but the baseline table has no variation store",
            ),
            Self::BaselineOutline { glyph, point, .. } => write!(
                f,
                "the stored outline of glyph {glyph} cannot validate point {point}"
            ),
            Self::BaselinePointRange {
                glyph,
                point,
                count,
            } => write!(
                f,
                "point {point} of glyph {glyph} is outside its stored point count {count}"
            ),
            Self::BaselineScriptUnavailable => {
                f.write_str("the run's script has no canonical four-letter Unicode short name")
            }
            Self::InvalidUnitsPerEm { value } => {
                write!(f, "units-per-em {value} is outside 16 to 16384")
            }
        }
    }
}

impl error::Error for FontMetricTableError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::BaselineOutline { reason, .. } => Some(reason),
            Self::Missing
            | Self::MissingCompanion { .. }
            | Self::LongMetricCount { .. }
            | Self::Duplicate
            | Self::Inaccessible
            | Self::Malformed
            | Self::UnsupportedVersion
            | Self::VariationWithoutAxes
            | Self::UnsupportedRecordSize { .. }
            | Self::VariationRecordOrder { .. }
            | Self::VariationOffset { .. }
            | Self::VariationStoreFormat { .. }
            | Self::VariationAxisCount { .. }
            | Self::VariationRegionCount { .. }
            | Self::VariationRegionCoordinate { .. }
            | Self::VariationDeltaIndex { .. }
            | Self::VariationRegionIndex { .. }
            | Self::VariationWordCount { .. }
            | Self::VariationRow { .. }
            | Self::VariationScalar { .. }
            | Self::BaselineTagOrder
            | Self::BaselineScriptOrder
            | Self::BaselineValueCount { .. }
            | Self::BaselineDefaultIndex { .. }
            | Self::BaselineCoordinateFormat { .. }
            | Self::BaselineDeviceFormat { .. }
            | Self::BaselineVariableDevice
            | Self::BaselineVariationStoreMissing
            | Self::BaselinePointRange { .. }
            | Self::BaselineScriptUnavailable
            | Self::InvalidUnitsPerEm { .. } => None,
        }
    }
}

/// Required same-instance metric fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricField {
    /// Actual instance size.
    Size,
    /// Nonnegative baseline ascent.
    Ascent,
    /// Signed nonpositive descent.
    Descent,
    /// Signed additional line spacing.
    Leading,
    /// Strike center above baseline.
    StrikeoutCenter,
    /// Positive strike thickness.
    StrikeoutThickness,
    /// Named font-design baseline coordinate.
    BaselineCoordinate,
    /// Used upright em-under coordinate or nominal-pen distance.
    UprightEmUnder,
    /// Used upright em-over coordinate or nominal-pen distance.
    UprightEmOver,
    /// Used upright Central coordinate or nominal-pen distance.
    UprightCentral,
    /// Em-under distance from used upright Central.
    UprightUsedUnder,
    /// Em-over distance from used upright Central.
    UprightUsedOver,
    /// Nominal horizontal advance.
    HorizontalAdvance,
    /// Nonnegative vertical advance magnitude.
    VerticalAdvance,
    /// Vertical origin relative to horizontal design origin, positive right.
    VerticalOriginX,
    /// Vertical origin relative to horizontal design origin, positive up.
    VerticalOriginY,
    /// Exact CSS-facing midpoint in one design reference.
    BaselineMidpoint,
    /// Vertical ideographic em-box origin, required to be zero.
    BaselineIdeographicOrigin,
}
/// Numeric validation stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricPhase {
    /// Resolved unscaled font-design units.
    Unscaled,
    /// Same-instance f64 scaled values.
    Scaled,
    /// Strike normalization or fallback.
    StrikeoutNormalization,
}
/// Numeric refusal category; no clamping is applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricNumericError {
    /// Required finite value is nonfinite.
    NonFinite,
    /// Nonnegative metric is negative.
    Negative,
    /// Nonpositive metric is positive.
    Positive,
    /// Positive metric is zero or negative.
    NonPositive,
    /// A required zero reference is nonzero.
    NonZero,
}

impl fmt::Display for FontMetricNumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "a required finite metric value is not finite",
            Self::Negative => "a metric value that must not be negative is negative",
            Self::Positive => "a metric value that must not be positive is positive",
            Self::NonPositive => "a metric value that must be positive is zero or negative",
            Self::NonZero => "a metric value that must be zero is not zero",
        })
    }
}

impl error::Error for FontMetricNumericError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NonFinite
            | Self::Negative
            | Self::Positive
            | Self::NonPositive
            | Self::NonZero => None,
        }
    }
}

/// Actual reason a shaping invocation could not complete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapingRefusalReason {
    GrowthLimit,
    WorkLimit,
    LookupNestingLimit,
    ContextLimit,
    AttachmentNestingLimit,
    FeatureCapacityLimit,
    ComponentHistoryLimit,
    JoiningInput,
    JoiningCorrespondence,
    JoiningProfile,
    /// The actual native symbol-presentation input could not be admitted.
    OrdinarySymbolInput,
}

/// Operation at which geometry was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextNumericPhase {
    SourceRangeFragments,
    Selection,
    Preparation,
    Shaping,
    Finalization,
    Placement,
    Accessibility,
    FontSynthesis,
    GlyphOutline,
}

/// Category which a geometry value failed to preserve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextNumericError {
    /// The value, or the result of converting or combining it, is not finite;
    /// this includes a finite value whose narrower representation overflows.
    NonFinite,
    /// A positive value did not stay positive: the result is zero or negative.
    PositiveLost,
    /// A nonzero value was lost: it became zero or no longer changed its result.
    NonZeroLost,
    /// The value is outside the domain the operation admits: negative where a
    /// nonnegative value is required, zero or negative where a positive one is
    /// required, too large for its representation, or inconsistent with its
    /// recorded counterpart.
    OutOfRange,
}

impl fmt::Display for TextNumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => {
                "a geometry value, or the result of converting or combining it, is not finite"
            }
            Self::PositiveLost => "a positive geometry value did not stay positive",
            Self::NonZeroLost => {
                "a nonzero geometry value was lost: it became zero or no longer changed its result"
            }
            Self::OutOfRange => "a geometry value is outside the domain the operation admits",
        })
    }
}

impl error::Error for TextNumericError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NonFinite | Self::PositiveLost | Self::NonZeroLost | Self::OutOfRange => None,
        }
    }
}

pub(crate) fn shaping_refused(reason: surgeist_shaper::ShapingRefusal) -> Error {
    use surgeist_shaper::ShapingRefusal as Provider;
    let reason = match reason {
        Provider::GrowthLimit => ShapingRefusalReason::GrowthLimit,
        Provider::WorkLimit => ShapingRefusalReason::WorkLimit,
        Provider::LookupNestingLimit => ShapingRefusalReason::LookupNestingLimit,
        Provider::ContextLimit => ShapingRefusalReason::ContextLimit,
        Provider::AttachmentNestingLimit => ShapingRefusalReason::AttachmentNestingLimit,
        Provider::FeatureCapacityLimit => ShapingRefusalReason::FeatureCapacityLimit,
        Provider::ComponentHistoryLimit => ShapingRefusalReason::ComponentHistoryLimit,
        Provider::JoiningInput => ShapingRefusalReason::JoiningInput,
        Provider::JoiningCorrespondence => ShapingRefusalReason::JoiningCorrespondence,
        Provider::JoiningProfile => ShapingRefusalReason::JoiningProfile,
        Provider::OrdinarySymbolInput => ShapingRefusalReason::OrdinarySymbolInput,
    };
    Error::new(ErrorCode::ShapingRefused, "shaping did not complete")
        .with_detail(ErrorDetail::ShapingRefused { reason })
}

pub(crate) fn text_numeric_error(
    field: &'static str,
    phase: TextNumericPhase,
    category: TextNumericError,
    value: f64,
) -> Error {
    Error::new(
        ErrorCode::InvalidTextNumeric,
        "text geometry cannot preserve its numeric category",
    )
    .with_detail(ErrorDetail::InvalidTextNumeric {
        field,
        phase,
        category,
        value,
    })
}
