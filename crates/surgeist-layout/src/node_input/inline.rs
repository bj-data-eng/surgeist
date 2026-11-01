use surgeist_text::BidiLevel;

use super::super::{DefaultScalar, LayoutScalar};
use super::{Clear, Direction};

/// Resolved alignment reference of a native inline formatting context.
/// Physical coordinates do not assert a font or named baseline origin.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum InlineBaselineContext {
    #[default]
    Physical,
    Named(crate::BaselineName),
}

/// The child's own, non-inherited choice of alignment baseline set.
/// This preference applies to named matching, not raw Physical projections.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BaselineSource {
    #[default]
    Auto,
    First,
    Last,
}

impl BaselineSource {
    pub(crate) fn prefers_last(self, display: super::Display) -> bool {
        match self {
            Self::Auto => display == super::Display::InlineBlock,
            Self::First => false,
            Self::Last => true,
        }
    }
}

/// A supplied baseline fact cannot be used in this containing context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBaselineError {
    /// A finite named coordinate cannot be represented in its output frame.
    UnrepresentableCoordinate,
    AxisMismatch {
        expected: crate::PhysicalAxis,
        actual: crate::PhysicalAxis,
    },
    ReferenceMismatch {
        expected: InlineBaselineContext,
        actual: InlineBaselineContext,
    },
}

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

/// The actual generation role of a normalized block box.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlockGeneration {
    #[default]
    Principal,
    Anonymous,
}

/// Computed signed indentation. Percentages remain unresolved until Layout
/// has the block container own inner inline size; intrinsic contributions use
/// only the absolute component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextIndentOf<S: LayoutScalar = DefaultScalar> {
    pub amount: crate::LengthPercentageOf<S>,
    pub each_line: bool,
    pub hanging: bool,
}

pub type TextIndent = TextIndentOf<DefaultScalar>;

impl<S: LayoutScalar> TextIndentOf<S> {
    pub const ZERO: Self = Self {
        amount: crate::LengthPercentageOf::ZERO,
        each_line: false,
        hanging: false,
    };
}

impl<S: LayoutScalar> Default for TextIndentOf<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    LegacyLeft,
    LegacyRight,
    LegacyCenter,
}

/// Normalized last/forced-line alignment; legacy block alignment is not a role
/// of this input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlignLast {
    #[default]
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

/// Preference for selecting legal line breaks in a native inline formatting run.
///
/// This normalized block-container value does not change wrapping permission,
/// available geometry or intrinsic sizing. Balance operates independently on
/// forced-break groups and may retain Auto for groups above ten formatted lines.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextWrapStyle {
    /// Retain the ordinary line-selection preference.
    #[default]
    Auto,
    /// Reduce variation in natural remaining inline space without changing the
    /// ordinary formatted-line count in the supported bounded group profile.
    Balance,
    /// Use ordinary line-local quality preference, retaining actual legality,
    /// geometry and element-terminal constraints rather than prior edit results.
    Stable,
    /// Repair the final paragraph pair when its shorter natural line can grow
    /// without transferring the deficiency to its predecessor.
    AvoidShortLastLine,
    /// Improve body raggedness after preserving the paragraph-tail preference.
    Pretty,
}

/// The implemented native expansion method, rather than CSS-wide `auto`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextJustification {
    #[default]
    None,
    /// Existing native expansion of retained U+0020 participants.
    Spaces,
    /// Text-qualified word-separator expansion with exact alignment fill.
    InterWord,
    /// Text-qualified visual character-boundary expansion with exact alignment fill.
    InterCharacter,
}

impl TextAlignLast {
    pub(crate) fn resolve(self, normal: TextAlign) -> TextAlign {
        match self {
            Self::Auto if normal == TextAlign::Justify => TextAlign::Start,
            Self::Auto => normal,
            Self::Start => TextAlign::Start,
            Self::End => TextAlign::End,
            Self::Left => TextAlign::Left,
            Self::Right => TextAlign::Right,
            Self::Center => TextAlign::Center,
            Self::Justify => TextAlign::Justify,
        }
    }
}

impl TextAlign {
    pub(crate) fn for_line(self, last: TextAlignLast, terminal: bool) -> Self {
        if terminal { last.resolve(self) } else { self }
    }

    /// CSS Text3 §6.4.3: an unexpandable line uses last-line alignment,
    /// with explicit last-line Justify falling back to Center.
    pub(crate) fn unexpandable(self, last: TextAlignLast) -> Self {
        match last.resolve(self) {
            Self::Justify => Self::Center,
            alignment => alignment,
        }
    }
}

/// Resolved line alignment. Shift matches the actual baseline first, then moves
/// toward line-over without changing the child's intrinsic/local metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VerticalAlignOf<S: LayoutScalar = DefaultScalar> {
    #[default]
    Baseline,
    Top,
    Bottom,
    Shift(crate::BaselineOffsetOf<S>),
}

pub type VerticalAlign = VerticalAlignOf<DefaultScalar>;

impl<S: LayoutScalar> VerticalAlignOf<S> {
    pub(crate) const fn is_baseline_relative(self) -> bool {
        matches!(self, Self::Baseline | Self::Shift(_))
    }
}

/// The writing-mode state supplied to layout.
///
/// The five supported values are `HorizontalTb`, `VerticalRl`, `VerticalLr`,
/// `SidewaysRl`, and `SidewaysLr`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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
    reference: InlineBaselineContext,
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
    pub fn try_new(
        reference: InlineBaselineContext,
        baseline: S,
        line_extent: S,
    ) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(baseline)?;
        validate_non_negative_finite(line_extent)?;

        if baseline > line_extent {
            return Err(InlineMetricsError::BaselineExceedsLineExtent {
                baseline,
                line_extent,
            });
        }

        Ok(Self {
            reference,
            baseline,
            line_extent,
        })
    }

    pub fn from_ascent_descent(
        reference: InlineBaselineContext,
        ascent: S,
        descent: S,
    ) -> Result<Self, InlineMetricsError<S>> {
        validate_non_negative_finite(ascent)?;
        validate_non_negative_finite(descent)?;
        Self::try_new(reference, ascent, ascent + descent)
    }

    pub fn from_line_height_and_baseline(
        reference: InlineBaselineContext,
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
            reference,
            baseline,
            line_extent: line_height,
        })
    }

    #[must_use]
    pub const fn reference(self) -> InlineBaselineContext {
        self.reference
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
        Self::from_line_height_and_baseline(
            InlineBaselineContext::Physical,
            S::from_f64(16.0),
            S::from_f64(12.0),
        )
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
    vertical_align: VerticalAlignOf<S>,
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
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlignOf<S>) -> Self {
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
    pub const fn vertical_align(self) -> VerticalAlignOf<S> {
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
            vertical_align: VerticalAlignOf::Baseline,
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

/// Actual used border and padding on this endpoint's own inline side.
/// Components and their sum are finite, nonnegative Layout coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryEdgeOf<S: LayoutScalar = DefaultScalar> {
    border: S,
    padding: S,
}

pub type InlineBoundaryEdge = InlineBoundaryEdgeOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBoundaryEdgeOf<S> {
    /// An affirmative used zero edge, rather than missing geometry.
    pub const ZERO: Self = Self {
        border: S::ZERO,
        padding: S::ZERO,
    };

    /// Validates the used components and their sum in the actual scalar lane.
    ///
    /// # Errors
    /// Rejects negative/nonfinite components or a nonfinite component sum.
    pub fn try_new(
        border: S,
        padding: S,
    ) -> Result<Self, crate::NonNegativeFiniteScalarErrorOf<S>> {
        let border = crate::NonNegativeFiniteOf::new(border)?.get();
        let padding = crate::NonNegativeFiniteOf::new(padding)?.get();
        crate::NonNegativeFiniteOf::new(border + padding)?;
        Ok(Self { border, padding })
    }

    #[must_use]
    pub const fn border(self) -> S {
        self.border
    }

    #[must_use]
    pub const fn padding(self) -> S {
        self.padding
    }

    /// The constructor-proved sum in S; consumers lower this result once.
    #[must_use]
    pub fn advance(self) -> S {
        self.border + self.padding
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoundaryInputOf<S: LayoutScalar = DefaultScalar> {
    kind: InlineBoundaryKind,
    writing_mode: WritingMode,
    direction: Direction,
    vertical_align: VerticalAlignOf<S>,
    metrics: InlineMetricsOf<S>,
    edge: InlineBoundaryEdgeOf<S>,
    operation: InlineBoundaryOperation,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum InlineBoundaryOperation {
    Contribution,
    BoxEdge { parent_axes: crate::FlowAxes },
}

pub type InlineBoundaryInput = InlineBoundaryInputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineBoundaryInputOf<S> {
    /// Contribute actual border/padding advance and metrics without requesting
    /// paired physical box geometry. Unpaired and zero-valued controls remain valid.
    #[must_use]
    pub const fn contribution(
        kind: InlineBoundaryKind,
        metrics: InlineMetricsOf<S>,
        edge: InlineBoundaryEdgeOf<S>,
    ) -> Self {
        Self {
            kind,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            vertical_align: VerticalAlignOf::Baseline,
            metrics,
            edge,
            operation: InlineBoundaryOperation::Contribution,
        }
    }

    /// Request physical box-edge placement with the actual immediate parent's
    /// used basis. The containing formatter frame remains independently checked.
    #[must_use]
    pub const fn box_edge(
        kind: InlineBoundaryKind,
        metrics: InlineMetricsOf<S>,
        edge: InlineBoundaryEdgeOf<S>,
        parent_axes: crate::FlowAxes,
    ) -> Self {
        let mut value = Self::contribution(kind, metrics, edge);
        value.operation = InlineBoundaryOperation::BoxEdge { parent_axes };
        value
    }

    /// The actual immediate parent basis for this explicit BoxEdge operation.
    /// A contribution carries no parent-side claim.
    #[must_use]
    pub const fn parent_edge_axes(self) -> Option<crate::FlowAxes> {
        match self.operation {
            InlineBoundaryOperation::Contribution => None,
            InlineBoundaryOperation::BoxEdge { parent_axes } => Some(parent_axes),
        }
    }

    /// Set the containing block formatter frame, independently of parent edges.
    #[must_use]
    pub const fn with_writing_mode(mut self, writing_mode: WritingMode) -> Self {
        self.writing_mode = writing_mode;
        self
    }

    /// Set the containing block formatter direction, independently of parent edges.
    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn with_vertical_align(mut self, vertical_align: VerticalAlignOf<S>) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoundaryKind {
        self.kind
    }

    #[must_use]
    pub const fn edge(self) -> InlineBoundaryEdgeOf<S> {
        self.edge
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
    pub const fn vertical_align(self) -> VerticalAlignOf<S> {
        self.vertical_align
    }

    #[must_use]
    pub const fn metrics(self) -> InlineMetricsOf<S> {
        self.metrics
    }
}
