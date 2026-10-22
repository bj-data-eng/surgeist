use std::{error, fmt};

use super::{TextStyleFeature, UnsupportedTextStyleReason};

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
    InvalidRange,
    InvalidStyle,
    LayoutFailed,
    HitTestFailed,
    UnsupportedFeature,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorDetail {
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
