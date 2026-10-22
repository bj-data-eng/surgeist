use surgeist_text::BidiLevel;

use super::super::{DefaultScalar, LayoutScalar};
use super::{Clear, Direction};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineBreakDisplay {
    #[default]
    Break,
    None,
}

impl LineBreakDisplay {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlign {
    #[default]
    Auto,
    LegacyLeft,
    LegacyRight,
    LegacyCenter,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
    Bottom,
}

/// The writing-mode state supplied to layout.
///
/// The five supported values are `HorizontalTb`, `VerticalRl`, `VerticalLr`,
/// `SidewaysRl`, and `SidewaysLr`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

impl WritingMode {
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        !matches!(self, Self::HorizontalTb)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineMetricsOf<S: LayoutScalar = DefaultScalar> {
    baseline: S,
    line_extent: S,
}

pub type InlineMetrics = InlineMetricsOf<DefaultScalar>;

/// Normalized break permission for a box-only inline participant.
/// Text candidates use preparation-issued opportunities and alternatives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBreakOpportunity {
    Prohibited,
    Allowed,
    Mandatory,
}

/// Actual resolved level and break participation of an atomic inline box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AtomicInlineParticipation {
    bidi_level: BidiLevel,
    following_break: InlineBreakOpportunity,
}

impl AtomicInlineParticipation {
    #[must_use]
    pub const fn new(bidi_level: BidiLevel, following_break: InlineBreakOpportunity) -> Self {
        Self {
            bidi_level,
            following_break,
        }
    }
    #[must_use]
    pub const fn bidi_level(self) -> BidiLevel {
        self.bidi_level
    }
    #[must_use]
    pub const fn following_break(self) -> InlineBreakOpportunity {
        self.following_break
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineMetricsError<S: LayoutScalar = DefaultScalar> {
    NonFinite { value: S },
    Negative { value: S },
    BaselineExceedsLineExtent { baseline: S, line_extent: S },
    BaselineExceedsLineHeight { baseline: S, line_height: S },
}

impl<S: LayoutScalar> InlineMetricsOf<S> {
    pub fn try_new(baseline: S, line_extent: S) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(baseline)?;
        validate_non_negative_finite(line_extent)?;

        if baseline > line_extent {
            return Err(InlineMetricsError::BaselineExceedsLineExtent {
                baseline,
                line_extent,
            });
        }

        Ok(Self {
            baseline,
            line_extent,
        })
    }

    pub fn from_ascent_descent(ascent: S, descent: S) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(ascent)?;
        validate_non_negative_finite(descent)?;
        Self::try_new(ascent, ascent + descent)
    }

    pub fn from_line_height_and_baseline(
        line_height: S,
        baseline: S,
    ) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(line_height)?;
        validate_non_negative_finite(baseline)?;

        if baseline > line_height {
            return Err(InlineMetricsError::BaselineExceedsLineHeight {
                baseline,
                line_height,
            });
        }

        Ok(Self {
            baseline,
            line_extent: line_height,
        })
    }

    #[must_use]
    pub const fn baseline(self) -> S {
        self.baseline
    }

    #[must_use]
    pub const fn line_extent(self) -> S {
        self.line_extent
    }

    #[must_use]
    pub fn after_baseline(self) -> S {
        self.line_extent - self.baseline
    }
}

impl<S: LayoutScalar> Default for InlineMetricsOf<S> {
    fn default() -> Self {
        Self::from_line_height_and_baseline(S::from_f64(16.0), S::from_f64(12.0))
            .expect("default inline metrics are valid")
    }
}

fn validate_non_negative_finite<S: LayoutScalar>(value: S) -> Result<(), InlineMetricsError<S>> {
    if !value.is_finite() {
        return Err(InlineMetricsError::NonFinite { value });
    }
    if value < S::ZERO {
        return Err(InlineMetricsError::Negative { value });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineBreakInputOf<S: LayoutScalar = DefaultScalar> {
    display: LineBreakDisplay,
    direction: Direction,
    writing_mode: WritingMode,
    vertical_align: VerticalAlign,
    clear: Clear,
    metrics: InlineMetricsOf<S>,
}

pub type LineBreakInput = LineBreakInputOf<DefaultScalar>;

impl<S: LayoutScalar> LineBreakInputOf<S> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_metrics(mut self, metrics: InlineMetricsOf<S>) -> Self {
        self.metrics = metrics;
        self
    }

    #[must_use]
    pub const fn hidden(mut self) -> Self {
        self.display = LineBreakDisplay::None;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn with_clear(mut self, clear: Clear) -> Self {
        self.clear = clear;
        self
    }

    #[must_use]
    pub const fn display(self) -> LineBreakDisplay {
        self.display
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn clear(self) -> Clear {
        self.clear
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}

impl<S: LayoutScalar> Default for LineBreakInputOf<S> {
    fn default() -> Self {
        Self {
            display: LineBreakDisplay::Break,
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            vertical_align: VerticalAlign::Baseline,
            clear: Clear::None,
            metrics: InlineMetricsOf::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoundaryKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryInputOf<S: LayoutScalar = DefaultScalar> {
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    vertical_align: VerticalAlign,
    metrics: InlineMetricsOf<S>,
}

pub type InlineBoundaryInput = InlineBoundaryInputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBoundaryInputOf<S> {
    #[must_use]
    pub const fn new(kind: InlineBoundaryKind, metrics: InlineMetricsOf<S>) -> Self {
        Self {
            kind,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            vertical_align: VerticalAlign::Baseline,
            metrics,
        }
    }

    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[must_use]
    pub const fn writing_mode(self) -> WritingMode {
        self.writing_mode
    }

    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}
