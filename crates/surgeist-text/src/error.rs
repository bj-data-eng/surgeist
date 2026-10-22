use std::{error, fmt};

use super::{TextStyleFeature, UnsupportedTextStyleReason, WhiteSpace};

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
    InvalidRange,
    InvalidStyle,
    LayoutFailed,
    HitTestFailed,
    UnsupportedFeature,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorDetail {
    /// The first nonempty effective interval outside the admitted processing mode.
    UnsupportedProcessingMode {
        /// Original UTF-16 interval.
        range: super::Range,
        /// Actual effective mode.
        mode: WhiteSpace,
        /// Winning original span ordinal, or the captured base style.
        span: Option<usize>,
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
    /// The provider still requires a concrete synthesis operation.
    UnresolvedFontSynthesis {
        /// Actual unresolved suggestion.
        kind: FontSynthesisKind,
    },
    IsolatedSurrogateLayout {
        index: usize,
        unit: u16,
    },
    UnsupportedShapingBoundary {
        index: usize,
        kind: ShapingBoundary,
    },
    ProviderSourceRange {
        start: usize,
        end: usize,
    },
    UnsupportedAccessibilityUnit {
        range: super::Range,
        bytes: usize,
    },
    AccessibilityLayoutMismatch,
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
        value: f32,
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

/// A shaping suggestion for which no concrete outline operation was supplied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontSynthesisKind {
    /// Faux bold requires a concrete emboldening operation.
    Embolden,
    /// Faux italic or oblique requires a concrete skew operation.
    Skew,
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
