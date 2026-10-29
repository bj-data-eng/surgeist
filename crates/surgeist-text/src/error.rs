use std::{error, fmt};

use super::{FontCatalogueIdentity, TextStyleFeature, UnsupportedTextStyleReason, WhiteSpace};

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
        local: super::Range,
        aggregate: super::Range,
    },
    Alternative {
        alternative: usize,
        local: super::Range,
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
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref().map(|error| error as _)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// Invalid complete contextual source-processing capture.
    InvalidSourceProcessing,
    /// Final semantic phrase analysis received invalid retained model data.
    InvalidPhraseAnalysis,
    /// Font-independent text analysis could not retain its complete result.
    TextAnalysisCapacity,
    FontSystemUnavailable,
    /// Font resolution found no usable face for required shaping or metrics.
    NoUsableFontFace,
    /// Actual shaping declined required work or representability.
    ShapingRefused,
    /// A logical or canonical geometry conversion lost its numeric category.
    InvalidTextNumeric,
    /// Invalid phase capability, captured occurrence, or complete line context.
    InvalidTextProtocol,
    FontLoadFailed,
    FontGenerationExhausted,
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
        reason: crate::HyphenationError,
    },
    /// Admission or retained-association failure for a nonempty source range.
    SourceRangeFragments {
        reason: crate::SourceRangeFragmentError,
    },
    /// Atomic rejection of contextual source facts or view membership.
    SourceProcessing {
        reason: super::SourceProcessingError,
    },
    /// Actual phase association or request admission failure.
    InvalidTextProtocol {
        reason: super::ProtocolError,
    },
    /// Actual glyph outline admission failed without publishing geometry.
    FontOutline {
        /// Requested glyph in the retained face.
        glyph: u32,
        /// Structural, capability, provider or storage refusal.
        reason: super::FontOutlineError,
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
        kind: super::DecorationKind,
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
        reference: super::UprightMetricReference,
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
        expected: FontCatalogueIdentity,
        /// Actual resource publication.
        actual: FontCatalogueIdentity,
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
        range: super::Range,
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
        id: super::Id,
        /// Original source anchor.
        anchor: usize,
    },
    /// One actual document delimiter crosses a change of effective processing value.
    SegmentBreakCrossesProcessingMode {
        /// Actual delimiter-vector ordinal.
        occurrence: usize,
        /// Complete original delimiter extent.
        range: super::Range,
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
        range: super::Range,
        bytes: usize,
    },
    AccessibilityBindingMismatch,
    AccessibilityIdentityConflict {
        id: u64,
    },
    AccessibilityIdentityExhausted,
    InvalidFontContainer,
    InvalidFontFace {
        index: u32,
    },
    FontFacesNotAdmitted {
        expected: usize,
        admitted: usize,
    },
    FontGenerationExhausted {
        generation: u64,
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
    InvalidPhraseData,
    MissingPhraseTopology {
        at: usize,
    },
    MissingPhraseScopeStyle {
        scope: usize,
        at: usize,
    },
    MissingSeparatorBoundaryContext {
        control: usize,
        at: usize,
        connection: usize,
    },
    /// A genuine original separator's forced adjacency depends on an uncaptured
    /// connection side. The range identifies original text, never a control.
    MissingOriginalSeparatorBoundaryContext {
        source: crate::Range,
        at: usize,
        connection: usize,
    },
    UnrepresentableSeparatorSeam {
        control: usize,
        at: usize,
        retained: crate::Range,
    },
    InvalidTextNodeRange {
        range: crate::Range,
        text_len: usize,
    },
    TextNodeMismatch {
        origin: usize,
    },
    MissingTextNodeContext {
        origin: usize,
    },
    CasingDataUnavailable,
    InvalidCasingData,
    InvalidContentLocale,
    Capacity,
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
    /// The occurrence precedes the previous supplied occurrence.
    Unordered,
    /// Ordered occurrences overlap.
    Overlap,
}

/// Capacity failure during off-side source processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessingCapacityError {
    /// Checked count or byte arithmetic exceeded the representable domain.
    Overflow,
    /// A fallible buffer reservation was refused.
    Allocation,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricTableError {
    /// Required table is absent.
    Missing,
    /// A present member lacks its required companion.
    MissingCompanion {
        /// Required absent table.
        required: FontMetricTable,
    },
    /// Long metric count is outside the glyph inventory.
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
        /// Actual record ordinal.
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
    /// Region inventory differs from the actual face axes.
    VariationAxisCount {
        /// Actual fvar axis count.
        expected: usize,
        /// Declared region-list axis count.
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
    BaselineOutline {
        glyph: u16,
        point: u16,
        reason: super::FontOutlineError,
    },
    /// The point is outside the genuine glyph's stored point inventory.
    BaselinePointRange {
        glyph: u16,
        point: u16,
        count: usize,
    },
    /// Actual run script metadata has no canonical Unicode short name.
    BaselineScriptUnavailable,
    /// OpenType units-per-em is invalid.
    InvalidUnitsPerEm {
        /// Actual rejected value.
        value: u16,
    },
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
    NonFinite,
    PositiveLost,
    NonZeroLost,
    OutOfRange,
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
