//! Parley-backed text layout boundary for Surgeist.
//!
//! This module owns text layout facts: shaping, line breaking, glyph runs,
//! inline boxes, cursor geometry, selection geometry, and low-level movement.
//! It does not own rendering, widgets, document identity, style resolution, or
//! application commands.

#[cfg(feature = "text-accessibility")]
mod accessibility;
mod cache;
mod catalogue;
mod composer;
mod edit;
mod error;
mod font;
mod geometry;
mod id;
mod layout;
mod options;
mod processing;
mod projection;
mod range;
#[path = "source.rs"]
mod source_model;
mod style;
mod style_support;
mod system;
mod text;
mod views;

#[cfg(feature = "text-accessibility")]
pub use accessibility::Accessibility;
pub use cache::{FontGeneration, Key, OptionsKey, SourceKey, Stats, StyleKey};
pub use catalogue::{FontCatalogueIdentity, FontCatalogueMode};
pub use composer::{Composer, Mark, compose, source};
pub use edit::{Edit, TextEdit};
pub use error::{
    DocumentSegmentBreakError, Error, ErrorCode, ErrorDetail, FontMetricField,
    FontMetricNumericError, FontMetricPhase, FontMetricTable, FontMetricTableError,
    FontSynthesisKind, NumericRequirement, ProcessingCapacityError, Result, ShapingBoundary,
};
pub use font::{EffectiveFontInstance, FontData, FontFace, FontMetrics, StrikeoutMetrics};
pub use geometry::{Point, Rect, Size};
pub use id::Id;
pub use layout::{
    Affinity, BaselineShiftFact, Cluster, Cursor, CursorGeometry, DecorationKind, DecorationRun,
    FontRef, Glyph, Hit, InlineBoxMetricFact, InlineMetricFacts, Layout, Line, LineMetricFact,
    Metrics, Movement, PaintedSelectionGeometry, PaintedSelectionRect, PositionedInlineBox, Run,
    RunMetricFact, RunMetrics, Selection, SelectionGeometry, SelectionRect,
};
pub use options::{Alignment, Indent, Options, TextAlignLast, TextOverflow, ValidatedOptions};
pub use processing::{
    DocumentSegmentBreaks, PlaintextItem, PlaintextItems, ProcessedInlineBox, ProcessedPosition,
    ProcessedRange, ProcessedSource, ProcessedSpan, ProcessedText, process_source_whitespace,
};
pub use range::{Range, SourcePosition, SourceRange};
pub use source_model::{
    BaselineShift, InlineBox, InlineBoxKind, Source, SourceIdentity, SourceRevision, Span,
    ValidatedSource, VerticalAlign,
};
pub use style::{
    Brush, Decoration, DecorationBrush, DecorationOffset, DecorationOffsetValue,
    DecorationThickness, DecorationThicknessValue, Direction, Font, FontVariant, FontWeightValue,
    FontWidthRatio, LineHeight, OverflowWrap, SelectionPaint, Slant, Style, TextTransform,
    ValidatedStyle, Weight, WhiteSpace, Width, WordBreak, Wrap,
};
pub use style_support::{TextStyleFeature, TextStyleSupport, UnsupportedTextStyleReason};
pub use system::{Builder, System, SystemOptions};
pub use text::Text;
pub use views::{GlyphPlacement, RunView};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_fonts;
