use std::{error, fmt};

use super::{FontCatalogueIdentity, TextStyleFeature, UnsupportedTextStyleReason, WhiteSpace};

/// Text crate result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Stable text diagnostic.
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    detail: Option<ErrorDetail>,
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
        self.detail = Some(detail);
        self
    }

    #[must_use]
    pub const fn detail(&self) -> Option<&ErrorDetail> {
        self.detail.as_ref()
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
    FontSystemUnavailable,
    /// Font resolution found no usable face for required shaping or metrics.
    NoUsableFontFace,
    /// Actual shaping declined required work or representability.
    ShapingRefused,
    /// A logical or canonical geometry conversion lost its numeric category.
    InvalidTextNumeric,
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
    /// Actual glyph outline admission failed without publishing geometry.
    FontOutline {
        /// Requested glyph in the retained face.
        glyph: u32,
        /// Structural, capability, provider or storage refusal.
        reason: super::FontOutlineError,
    },
    /// The actual shaping invocation returned incomplete output.
    ShapingRefused {
        reason: ShapingRefusalReason,
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

/// Tables used by glyph-free same-instance metrics.
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
}
/// Used metric-table refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricTableError {
    /// Required table is absent.
    Missing,
    /// Multiple directory entries claim the table.
    Duplicate,
    /// Recorded range is inaccessible.
    Inaccessible,
    /// Contents are truncated or malformed.
    Malformed,
    /// Version is outside the admitted domain.
    UnsupportedVersion,
    /// MVAR is present without a real fvar axis inventory.
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
}
/// Numeric validation stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontMetricPhase {
    /// Resolved unscaled provider-selected units.
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
}

/// Operation at which geometry was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextNumericPhase {
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

pub(crate) fn shaping_refused(reason: harfrust::ShapingRefusal) -> Error {
    use harfrust::ShapingRefusal as Provider;
    let reason = match reason {
        Provider::GrowthLimit => ShapingRefusalReason::GrowthLimit,
        Provider::WorkLimit => ShapingRefusalReason::WorkLimit,
        Provider::LookupNestingLimit => ShapingRefusalReason::LookupNestingLimit,
        Provider::ContextLimit => ShapingRefusalReason::ContextLimit,
        Provider::AttachmentNestingLimit => ShapingRefusalReason::AttachmentNestingLimit,
        Provider::FeatureCapacityLimit => ShapingRefusalReason::FeatureCapacityLimit,
        Provider::ComponentHistoryLimit => ShapingRefusalReason::ComponentHistoryLimit,
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
