use std::{fmt, sync::Arc};

use super::{
    BaselineShift, Brush, Direction, Edit, FontCatalogueIdentity, FontData, Id, InlineBoxKind, Key,
    Options, Point, Range, Rect, Result, SelectionPaint, Size, Source, Style, VerticalAlign,
};
use crate::engine::{
    PlacedLine, editing,
    layout::data::{FinalizedRun, PlacedLayout, PreparedRun},
};

/// Immutable committed rows and their sole original-source preparation owner.
#[derive(Clone)]
pub struct Layout {
    pub(crate) inner: Arc<PlacedLayout>,
    #[cfg(feature = "text-accessibility")]
    pub(crate) identity: Arc<()>,
    pub(crate) basis: LayoutBasis,
    pub(crate) key: Key,
}

/// The actual operation which produced this placement; explicit composition
/// does not acquire manufactured standalone options.
#[derive(Clone, Copy)]
pub(crate) enum LayoutBasis {
    Standalone(Options),
    Explicit,
}

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layout")
            .field("metrics", &self.metrics())
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl Layout {
    pub(crate) fn from_placed(inner: PlacedLayout, key: Key, basis: LayoutBasis) -> Self {
        Self {
            inner: Arc::new(inner),
            key,
            basis,
            #[cfg(feature = "text-accessibility")]
            identity: Arc::new(()),
        }
    }

    pub(crate) fn matches_request(
        &self,
        source: &Source,
        style: &Style,
        options: Options,
        catalogue: &FontCatalogueIdentity,
    ) -> bool {
        self.source() == source
            && self.default_style() == style
            && matches!(self.basis, LayoutBasis::Standalone(actual) if actual == options)
            && &self.inner.prepared.font_catalogue == catalogue
    }

    #[must_use]
    pub fn source(&self) -> &Source {
        self.inner.prepared.analyzed.processed.source()
    }

    /// Retains the exact original analysis shared by this placement and its preparation.
    #[must_use]
    pub fn analyzed_source(&self) -> crate::AnalyzedSource {
        crate::AnalyzedSource::from_inner(Arc::clone(&self.inner.prepared.analyzed))
    }

    pub(crate) fn default_style(&self) -> &Style {
        self.inner
            .prepared
            .analyzed
            .paragraph
            .base_style(&self.inner.prepared.analyzed.processed)
    }

    /// Actual admitted physical bounds, including the explicit line origins.
    ///
    /// These are placed metric bounds, not representation-selected paint ink.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.inner.bounds
    }

    #[must_use]
    pub fn metrics(&self) -> Metrics {
        Metrics::new(
            Size::new(self.inner.width, self.inner.height),
            self.inner.full_width,
            self.inner.lines.len(),
            self.inner.lines.first().map(line_baseline),
            self.inner.lines.last().map(line_baseline),
            match self.basis {
                LayoutBasis::Standalone(options) => options
                    .width
                    .is_some_and(|width| self.inner.full_width > width),
                LayoutBasis::Explicit => self.inner.lines.iter().any(|line| {
                    let metrics = line.finalized.data.metrics;
                    line.finalized
                        .selected
                        .request
                        .context
                        .available_inline
                        .is_some_and(|available| {
                            metrics.occupied_start < 0.0 || metrics.occupied_end > available
                        })
                }),
            },
        )
    }

    #[must_use]
    pub fn key(&self) -> Key {
        self.key.clone()
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        if self.inner.prepared.analyzed.analysis.base_level & 1 != 0 {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        }
    }

    pub fn lines(&self) -> Vec<Line> {
        self.inner
            .lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let metrics = &line.finalized.data.metrics;
                Line::new(
                    index,
                    line.finalized.selected.presentation.source,
                    line_baseline(line),
                    line_rect(
                        line,
                        Rect::new(
                            metrics.inline_min,
                            metrics.block_min,
                            metrics.inline_max - metrics.inline_min,
                            metrics.block_max - metrics.block_min,
                        ),
                    ),
                )
            })
            .collect()
    }

    pub(crate) fn positioned_runs(
        &self,
    ) -> impl Iterator<Item = (usize, &PlacedLine, &FinalizedRun, &PreparedRun)> {
        self.inner
            .lines
            .iter()
            .enumerate()
            .flat_map(|(index, line)| {
                line.finalized
                    .data
                    .runs
                    .iter()
                    .map(move |run| (index, line, run, &line.finalized.data.font_runs[run.run]))
            })
    }

    pub fn glyph_runs(&self) -> Vec<Run> {
        self.positioned_runs()
            .map(|(index, line, run, shaped)| {
                let glyphs = line.finalized.data.glyphs[run.glyphs.clone()]
                    .iter()
                    .map(|glyph| {
                        Glyph::new(
                            glyph.glyph.id,
                            glyph.glyph.x,
                            glyph.glyph.y,
                            glyph.glyph.advance,
                            glyph.glyph.source,
                            glyph.pen,
                        )
                    })
                    .collect();
                let authored_style = if self.inner.prepared.default_style_index == Some(run.style) {
                    self.inner
                        .prepared
                        .analyzed
                        .paragraph
                        .base_style(&self.inner.prepared.analyzed.processed)
                } else {
                    self.inner.prepared.analyzed.styles[run.style]
                        .style
                        .authored()
                };
                Run::new(
                    index,
                    run.source,
                    shaped.font.clone(),
                    authored_style.clone(),
                    self.inner.prepared.styles[run.style].style.brush,
                    RunMetrics::new(
                        shaped.font_size,
                        line_baseline(line),
                        line_point(
                            line,
                            Point::new(run.x, line.finalized.data.metrics.baseline),
                        )
                        .x,
                        run.advance,
                    ),
                    glyphs,
                )
            })
            .collect()
    }

    #[must_use]
    pub fn inline_metric_facts(&self) -> InlineMetricFacts {
        let lines = self.lines().into_iter().map(LineMetricFact::new).collect();
        let runs = self
            .positioned_runs()
            .map(|(index, line, run, shaped)| {
                RunMetricFact::new(
                    index,
                    run.source,
                    shaped.font_size,
                    line_baseline(line),
                    line_point(
                        line,
                        Point::new(run.x, line.finalized.data.metrics.baseline),
                    )
                    .x,
                    run.advance,
                )
            })
            .collect();
        let inline_boxes = self
            .inline_boxes()
            .into_iter()
            .map(|box_| {
                InlineBoxMetricFact::new(
                    box_.id(),
                    box_.kind(),
                    box_.index(),
                    box_.rect(),
                    box_.line(),
                    box_.vertical_align(),
                    match box_.vertical_align() {
                        VerticalAlign::Shift(shift) => BaselineShiftFact::Requested(shift),
                        _ => BaselineShiftFact::BottomOnBaseline,
                    },
                )
            })
            .collect();
        InlineMetricFacts::new(self.metrics(), lines, runs, inline_boxes)
    }

    pub fn inline_boxes(&self) -> Vec<PositionedInlineBox> {
        self.inner
            .lines
            .iter()
            .enumerate()
            .flat_map(|(index, line)| {
                line.atomics.iter().map(move |box_| {
                    let occurrence = box_.occurrence.view();
                    let source = occurrence.original_box();
                    PositionedInlineBox::new(
                        source.id(),
                        source.kind(),
                        box_.border,
                        source.index(),
                        source.vertical_align(),
                        index,
                    )
                })
            })
            .collect()
    }

    pub fn clusters(&self) -> Vec<Cluster> {
        self.inner
            .lines
            .iter()
            .flat_map(|line| {
                let metrics = &line.finalized.data.metrics;
                line.finalized.data.clusters.iter().map(move |cluster| {
                    Cluster::new(
                        cluster.source,
                        line_rect(
                            line,
                            Rect::new(
                                cluster.x,
                                metrics.block_min,
                                cluster.advance,
                                metrics.block_max - metrics.block_min,
                            ),
                        ),
                    )
                })
            })
            .collect()
    }

    pub fn decorations(&self) -> Vec<DecorationRun> {
        self.decoration_views().collect()
    }

    #[must_use]
    pub fn hit(&self, point: Point) -> Hit {
        editing::hit(&self.inner, point)
    }

    /// Ordered positions in this exact immutable placement. Source projection
    /// does not collapse distinct native-control or source-less line occurrences.
    pub fn positions(&self) -> impl ExactSizeIterator<Item = PlacedPosition> + '_ {
        (0..self.inner.interaction.positions.len()).map(|index| PlacedPosition {
            placement: Arc::clone(&self.inner),
            index,
        })
    }

    fn admit_position(&self, position: &PlacedPosition) -> Result<usize> {
        if !Arc::ptr_eq(&self.inner, &position.placement) {
            return Err(crate::protocol::protocol_error(
                crate::ProtocolError::ForeignPlacement,
            ));
        }
        if position.index >= self.inner.interaction.positions.len() {
            return Err(crate::protocol::protocol_error(
                crate::ProtocolError::InvalidPlacedPosition {
                    position: position.index,
                },
            ));
        }
        Ok(position.index)
    }

    /// Geometry of an actual occurrence in this placement.
    pub fn position_geometry(&self, position: &PlacedPosition) -> Result<CursorGeometry> {
        editing::position::geometry(&self.inner, self.admit_position(position)?)
    }

    pub fn move_position(
        &self,
        position: &PlacedPosition,
        movement: Movement,
    ) -> Result<PlacedPosition> {
        let index =
            editing::position::movement(&self.inner, self.admit_position(position)?, movement)?;
        Ok(PlacedPosition {
            placement: Arc::clone(&self.inner),
            index,
        })
    }

    /// Selects between actual placement cuts, preserving exact original
    /// occurrence coverage separately from geometry and processed plaintext.
    pub fn position_selection(
        &self,
        anchor: &PlacedPosition,
        focus: &PlacedPosition,
    ) -> Result<PlacedSelection> {
        editing::position::selection(
            &self.inner,
            self.admit_position(anchor)?,
            self.admit_position(focus)?,
        )
    }

    pub fn cursor(&self, cursor: Cursor) -> Result<CursorGeometry> {
        editing::cursor(&self.inner, cursor)
    }

    pub fn selection(&self, selection: Selection) -> Result<SelectionGeometry> {
        editing::selection(&self.inner, selection)
    }

    pub fn painted_selection(
        &self,
        selection: Selection,
        paint: SelectionPaint,
    ) -> Result<PaintedSelectionGeometry> {
        Ok(PaintedSelectionGeometry::new(
            self.selection(selection)?
                .rects()
                .iter()
                .map(|rect| PaintedSelectionRect::new(rect.rect(), rect.line(), paint))
                .collect(),
        ))
    }

    pub fn move_cursor(&self, cursor: Cursor, movement: Movement) -> Result<Cursor> {
        editing::move_cursor(&self.inner, cursor, movement)
    }

    pub fn move_selection(
        &self,
        selection: Selection,
        movement: Movement,
        extend: bool,
    ) -> Result<Selection> {
        super::range::validate(self.source().text(), selection.range())?;
        self.inner
            .prepared
            .analyzed
            .paragraph
            .admit(self.source(), selection.range())?;
        editing::position::validate_source(&self.inner, selection.anchor())?;
        editing::position::validate_source(&self.inner, selection.focus())?;
        let moved = self.move_cursor(selection.focus(), movement)?;
        let result = if extend {
            Selection::new(selection.anchor(), moved)
        } else {
            Selection::collapsed(moved)
        };
        self.inner
            .prepared
            .analyzed
            .paragraph
            .admit(self.source(), result.range())?;
        Ok(result)
    }

    pub fn apply(&self, edit: Edit) -> Result<Source> {
        let edit = edit.normalize(self.source())?;
        self.inner
            .prepared
            .analyzed
            .paragraph
            .admit(self.source(), edit.range())?;
        edit.apply_to(self.source().clone())
    }
}

pub(crate) fn line_baseline(line: &PlacedLine) -> f64 {
    line.anchor.y
}

/// The one physical translation of this line's immutable local geometry.
pub(crate) fn line_point(line: &PlacedLine, local: Point) -> Point {
    Point::new(
        line.anchor.x + local.x,
        line.anchor.y + (local.y - line.finalized.data.metrics.baseline),
    )
}

pub(crate) fn line_rect(line: &PlacedLine, local: Rect) -> Rect {
    let origin = line_point(line, local.origin);
    Rect::new(origin.x, origin.y, local.size.width, local.size.height)
}

pub(crate) fn decoration_top(baseline: f64, offset_from_baseline: f64) -> f64 {
    baseline - offset_from_baseline
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    size: Size,
    full_width: f64,
    line_count: usize,
    first_baseline: Option<f64>,
    last_baseline: Option<f64>,
    overflow: bool,
}

impl Metrics {
    #[must_use]
    pub const fn new(
        size: Size,
        full_width: f64,
        line_count: usize,
        first_baseline: Option<f64>,
        last_baseline: Option<f64>,
        overflow: bool,
    ) -> Self {
        Self {
            size,
            full_width,
            line_count,
            first_baseline,
            last_baseline,
            overflow,
        }
    }

    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[must_use]
    pub const fn full_width(self) -> f64 {
        self.full_width
    }

    #[must_use]
    pub const fn line_count(self) -> usize {
        self.line_count
    }

    #[must_use]
    pub const fn first_baseline(self) -> Option<f64> {
        self.first_baseline
    }

    #[must_use]
    pub const fn last_baseline(self) -> Option<f64> {
        self.last_baseline
    }

    #[must_use]
    pub const fn overflow(self) -> bool {
        self.overflow
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    index: usize,
    range: Range,
    baseline: f64,
    bounds: Rect,
}

impl Line {
    #[must_use]
    pub const fn new(index: usize, range: Range, baseline: f64, bounds: Rect) -> Self {
        Self {
            index,
            range,
            baseline,
            bounds,
        }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn range(self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn baseline(self) -> f64 {
        self.baseline
    }

    #[must_use]
    pub const fn bounds(self) -> Rect {
        self.bounds
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    line: usize,
    range: Range,
    font: FontData,
    style: Style,
    brush: Brush,
    metrics: RunMetrics,
    glyphs: Vec<Glyph>,
}

impl Run {
    #[must_use]
    pub fn new(
        line: usize,
        range: Range,
        font: FontData,
        style: Style,
        brush: Brush,
        metrics: RunMetrics,
        glyphs: Vec<Glyph>,
    ) -> Self {
        Self {
            line,
            range,
            font,
            style,
            brush,
            metrics,
            glyphs,
        }
    }

    #[must_use]
    pub fn font(&self) -> &FontData {
        &self.font
    }

    #[must_use]
    pub const fn style(&self) -> &Style {
        &self.style
    }

    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.metrics.font_size()
    }

    #[must_use]
    pub const fn brush(&self) -> Brush {
        self.brush
    }

    #[must_use]
    pub const fn baseline(&self) -> f64 {
        self.metrics.baseline()
    }

    #[must_use]
    pub const fn offset(&self) -> f64 {
        self.metrics.offset()
    }

    #[must_use]
    pub const fn advance(&self) -> f64 {
        self.metrics.advance()
    }

    #[must_use]
    pub const fn origin(&self) -> Point {
        Point::new(self.metrics.offset, self.metrics.baseline)
    }

    #[must_use]
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InlineMetricFacts {
    metrics: Metrics,
    lines: Vec<LineMetricFact>,
    runs: Vec<RunMetricFact>,
    inline_boxes: Vec<InlineBoxMetricFact>,
}

impl InlineMetricFacts {
    #[must_use]
    pub fn new(
        metrics: Metrics,
        lines: Vec<LineMetricFact>,
        runs: Vec<RunMetricFact>,
        inline_boxes: Vec<InlineBoxMetricFact>,
    ) -> Self {
        Self {
            metrics,
            lines,
            runs,
            inline_boxes,
        }
    }

    #[must_use]
    pub const fn metrics(&self) -> Metrics {
        self.metrics
    }

    #[must_use]
    pub fn lines(&self) -> &[LineMetricFact] {
        &self.lines
    }

    #[must_use]
    pub fn runs(&self) -> &[RunMetricFact] {
        &self.runs
    }

    #[must_use]
    pub fn inline_boxes(&self) -> &[InlineBoxMetricFact] {
        &self.inline_boxes
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineMetricFact {
    line: Line,
}

impl LineMetricFact {
    #[must_use]
    pub const fn new(line: Line) -> Self {
        Self { line }
    }

    #[must_use]
    pub const fn line(self) -> Line {
        self.line
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunMetricFact {
    line: usize,
    range: Range,
    font_size: f32,
    baseline: f64,
    offset: f64,
    advance: f64,
}

impl RunMetricFact {
    #[must_use]
    pub const fn new(
        line: usize,
        range: Range,
        font_size: f32,
        baseline: f64,
        offset: f64,
        advance: f64,
    ) -> Self {
        Self {
            line,
            range,
            font_size,
            baseline,
            offset,
            advance,
        }
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn range(self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        self.font_size
    }

    #[must_use]
    pub const fn baseline(self) -> f64 {
        self.baseline
    }

    #[must_use]
    pub const fn offset(self) -> f64 {
        self.offset
    }

    #[must_use]
    pub const fn advance(self) -> f64 {
        self.advance
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoxMetricFact {
    id: Id,
    kind: InlineBoxKind,
    index: usize,
    rect: Rect,
    line: usize,
    vertical_align: VerticalAlign,
    baseline_shift: BaselineShiftFact,
}

impl InlineBoxMetricFact {
    #[must_use]
    pub const fn new(
        id: Id,
        kind: InlineBoxKind,
        index: usize,
        rect: Rect,
        line: usize,
        vertical_align: VerticalAlign,
        baseline_shift: BaselineShiftFact,
    ) -> Self {
        Self {
            id,
            kind,
            index,
            rect,
            line,
            vertical_align,
            baseline_shift,
        }
    }

    #[must_use]
    pub const fn id(self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoxKind {
        self.kind
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn baseline_shift(self) -> BaselineShiftFact {
        self.baseline_shift
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BaselineShiftFact {
    BottomOnBaseline,
    Requested(BaselineShift),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunMetrics {
    font_size: f32,
    baseline: f64,
    offset: f64,
    advance: f64,
}

impl RunMetrics {
    #[must_use]
    pub const fn new(font_size: f32, baseline: f64, offset: f64, advance: f64) -> Self {
        Self {
            font_size,
            baseline,
            offset,
            advance,
        }
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        self.font_size
    }

    #[must_use]
    pub const fn baseline(self) -> f64 {
        self.baseline
    }

    #[must_use]
    pub const fn offset(self) -> f64 {
        self.offset
    }

    #[must_use]
    pub const fn advance(self) -> f64 {
        self.advance
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    id: u32,
    x: f32,
    y: f32,
    advance: f32,
    range: Range,
    pen: f64,
}

impl Glyph {
    #[must_use]
    pub const fn new(id: u32, x: f32, y: f32, advance: f32, range: Range, pen: f64) -> Self {
        Self {
            id,
            x,
            y,
            advance,
            range,
            pen,
        }
    }

    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }

    /// Raw canonical horizontal provider offset, added to the logical pen and run origin.
    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }

    /// Raw canonical vertical provider offset, positive downward from the run baseline.
    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }

    /// Raw canonical provider advance, before logical spacing allocation.
    #[must_use]
    pub const fn advance(self) -> f32 {
        self.advance
    }

    /// Actual logical run-local pen, including line spacing allocation.
    #[must_use]
    pub const fn pen(self) -> f64 {
        self.pen
    }

    #[must_use]
    pub const fn range(self) -> Range {
        self.range
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cluster {
    range: Range,
    bounds: Rect,
}

impl Cluster {
    #[must_use]
    pub const fn new(range: Range, bounds: Rect) -> Self {
        Self { range, bounds }
    }

    #[must_use]
    pub const fn range(self) -> Range {
        self.range
    }

    #[must_use]
    pub const fn bounds(self) -> Rect {
        self.bounds
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionedInlineBox {
    id: Id,
    kind: InlineBoxKind,
    rect: Rect,
    index: usize,
    vertical_align: VerticalAlign,
    line: usize,
}

impl PositionedInlineBox {
    #[must_use]
    pub const fn new(
        id: Id,
        kind: InlineBoxKind,
        rect: Rect,
        index: usize,
        vertical_align: VerticalAlign,
        line: usize,
    ) -> Self {
        Self {
            id,
            kind,
            rect,
            index,
            vertical_align,
            line,
        }
    }

    #[must_use]
    pub const fn id(self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn kind(self) -> InlineBoxKind {
        self.kind
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn vertical_align(self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }
}

/// A checked interaction occurrence of one actual immutable placement.
/// Its source cursor is an explicit projection and cannot identify its geometry.
#[derive(Clone)]
pub struct PlacedPosition {
    pub(crate) placement: Arc<PlacedLayout>,
    pub(crate) index: usize,
}
impl PlacedPosition {
    #[must_use]
    pub fn source_cursor(&self) -> Cursor {
        self.placement.interaction.positions[self.index].source
    }
}
impl fmt::Debug for PlacedPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlacedPosition")
            .field("source_cursor", &self.source_cursor())
            .finish_non_exhaustive()
    }
}
impl PartialEq for PlacedPosition {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.placement, &other.placement)
    }
}
impl Eq for PlacedPosition {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Hit {
    Text(PlacedPosition),
    InlineBox(Id),
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecorationRun {
    rect: Rect,
    brush: Brush,
    kind: DecorationKind,
}

impl DecorationRun {
    #[must_use]
    pub const fn new(rect: Rect, brush: Brush, kind: DecorationKind) -> Self {
        Self { rect, brush, kind }
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub const fn brush(self) -> Brush {
        self.brush
    }

    #[must_use]
    pub const fn kind(self) -> DecorationKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecorationKind {
    Underline,
    Strikethrough,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cursor {
    index: usize,
    affinity: Affinity,
}

impl Cursor {
    #[must_use]
    pub const fn new(index: usize, affinity: Affinity) -> Self {
        Self { index, affinity }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn affinity(self) -> Affinity {
        self.affinity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Affinity {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorGeometry {
    rect: Rect,
}

impl CursorGeometry {
    #[must_use]
    pub const fn new(rect: Rect) -> Self {
        Self { rect }
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    anchor: Cursor,
    focus: Cursor,
}

impl Selection {
    #[must_use]
    pub const fn new(anchor: Cursor, focus: Cursor) -> Self {
        Self { anchor, focus }
    }

    #[must_use]
    pub const fn collapsed(cursor: Cursor) -> Self {
        Self {
            anchor: cursor,
            focus: cursor,
        }
    }

    #[must_use]
    pub const fn anchor(self) -> Cursor {
        self.anchor
    }

    #[must_use]
    pub const fn focus(self) -> Cursor {
        self.focus
    }

    #[must_use]
    pub const fn is_collapsed(self) -> bool {
        self.anchor.index == self.focus.index
    }

    #[must_use]
    pub fn range(self) -> Range {
        Range::new(
            self.anchor.index.min(self.focus.index),
            self.anchor.index.max(self.focus.index),
        )
    }
}

/// One selected original interval in an actual captured text occurrence.
/// Equal text in a different occurrence does not identify this piece.
#[derive(Clone, Debug)]
pub struct PlacedSelectionPiece {
    pub(crate) origin: crate::PreparedOrigin,
    pub(crate) local: Range,
    pub(crate) aggregate: Range,
}

impl PlacedSelectionPiece {
    #[must_use]
    pub fn origin(&self) -> &crate::PreparedOrigin {
        &self.origin
    }

    #[must_use]
    pub const fn local_range(&self) -> Range {
        self.local
    }

    #[must_use]
    pub const fn aggregate_range(&self) -> Range {
        self.aggregate
    }

    /// Exact original UTF-16, including isolated units; never transformed text.
    #[must_use]
    pub fn code_units(&self) -> &[u16] {
        let prepared = &self.origin.prepared;
        let source = prepared.analyzed.paragraph.origins[self.origin.index]
            .original(&prepared.analyzed.processed);
        &source.text().code_units()[self.local.start..self.local.end]
    }
}

/// An exact selection issued by one immutable placement's relation cuts.
/// Geometry, original extraction and plaintext are distinct observations.
#[derive(Clone)]
pub struct PlacedSelection {
    // Retain the issuing snapshot even when the selected cuts cover no source.
    pub(crate) _placement: Arc<PlacedLayout>,
    pub(crate) geometry: SelectionGeometry,
    pub(crate) pieces: Vec<PlacedSelectionPiece>,
    pub(crate) source_text: crate::Text,
    pub(crate) plaintext: crate::Text,
    pub(crate) collapsed: bool,
}

impl PlacedSelection {
    #[must_use]
    pub fn geometry(&self) -> &SelectionGeometry {
        &self.geometry
    }

    /// Normalized original pieces in original logical order, without holes filled.
    #[must_use]
    pub fn source_pieces(&self) -> &[PlacedSelectionPiece] {
        &self.pieces
    }

    /// Covered original UTF-16 once per occurrence, in original logical order.
    #[must_use]
    pub fn source_text(&self) -> &crate::Text {
        &self.source_text
    }

    /// The processing owner's plaintext policy over this original coverage.
    #[must_use]
    pub fn plaintext(&self) -> &crate::Text {
        &self.plaintext
    }

    /// Whether both endpoints alias one actual cut. A removal-only selection
    /// can have no rectangle while remaining noncollapsed.
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.collapsed
    }
}

impl fmt::Debug for PlacedSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlacedSelection")
            .field("geometry", &self.geometry)
            .field("source_pieces", &self.pieces)
            .field("source_text", &self.source_text)
            .field("plaintext", &self.plaintext)
            .field("collapsed", &self.collapsed)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectionGeometry {
    rects: Vec<SelectionRect>,
}

impl SelectionGeometry {
    #[must_use]
    pub fn new(rects: Vec<SelectionRect>) -> Self {
        Self { rects }
    }

    #[must_use]
    pub fn rects(&self) -> &[SelectionRect] {
        &self.rects
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintedSelectionGeometry {
    rects: Vec<PaintedSelectionRect>,
}

impl PaintedSelectionGeometry {
    #[must_use]
    pub fn new(rects: Vec<PaintedSelectionRect>) -> Self {
        Self { rects }
    }

    #[must_use]
    pub fn rects(&self) -> &[PaintedSelectionRect] {
        &self.rects
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRect {
    rect: Rect,
    line: usize,
}

impl SelectionRect {
    #[must_use]
    pub const fn new(rect: Rect, line: usize) -> Self {
        Self { rect, line }
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintedSelectionRect {
    rect: Rect,
    line: usize,
    paint: SelectionPaint,
}

impl PaintedSelectionRect {
    #[must_use]
    pub const fn new(rect: Rect, line: usize, paint: SelectionPaint) -> Self {
        Self { rect, line, paint }
    }

    #[must_use]
    pub const fn rect(self) -> Rect {
        self.rect
    }

    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn paint(self) -> SelectionPaint {
        self.paint
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Movement {
    PreviousCluster,
    NextCluster,
    PreviousWord,
    NextWord,
    LineStart,
    LineEnd,
    PreviousLine,
    NextLine,
    DocumentStart,
    DocumentEnd,
}
