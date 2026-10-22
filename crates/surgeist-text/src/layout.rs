use std::{fmt, sync::Arc};

use parley::PositionedLayoutItem;

use super::geometry::rect_from_bounds;
use super::projection::{ProviderIndex, ScalarProjection};
use super::{
    BaselineShift, Brush, Direction, Edit, Id, InlineBoxKind, Key, Point, Range, Rect, Result,
    SelectionPaint, Size, Source, Style, VerticalAlign,
};

/// Immutable shaped and line-broken layout.
#[derive(Clone)]
pub struct Layout {
    pub(crate) inner: parley::Layout<Brush>,
    pub(crate) projection: Arc<ScalarProjection>,
    #[cfg(feature = "text-accessibility")]
    pub(crate) identity: Arc<()>,
    #[cfg(feature = "text-accessibility")]
    pub(crate) alignment: super::Alignment,
    pub(crate) source: Source,
    pub(crate) default_style: Style,
    pub(crate) key: Key,
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
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }

    #[must_use]
    pub fn metrics(&self) -> Metrics {
        Metrics::new(
            Size::new(self.inner.width(), self.inner.height()),
            self.inner.full_width(),
            self.inner.len(),
            self.inner.get(0).map(|line| line.metrics().baseline),
            self.inner
                .get(self.inner.len().saturating_sub(1))
                .map(|line| line.metrics().baseline),
            self.key
                .options_width
                .is_some_and(|width| self.inner.full_width() > width.0),
        )
    }

    #[must_use]
    pub fn key(&self) -> Key {
        self.key.clone()
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        if self.inner.is_rtl() {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        }
    }

    pub fn lines(&self) -> Vec<Line> {
        self.inner
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let metrics = line.metrics();
                Line::new(
                    index,
                    self.projection.line_range(index),
                    metrics.baseline,
                    Rect::new(
                        metrics.inline_min_coord,
                        metrics.block_min_coord,
                        metrics.inline_max_coord - metrics.inline_min_coord,
                        metrics.block_max_coord - metrics.block_min_coord,
                    ),
                )
            })
            .collect()
    }

    pub(crate) fn positioned_runs(
        &self,
    ) -> impl Iterator<Item = (usize, parley::GlyphRun<'_, Brush>)> {
        self.inner
            .lines()
            .enumerate()
            .flat_map(|(line_index, line)| {
                line.items().filter_map(move |item| match item {
                    PositionedLayoutItem::GlyphRun(run) => Some((line_index, run)),
                    PositionedLayoutItem::InlineBox(_) => None,
                })
            })
    }

    pub fn glyph_runs(&self) -> Vec<Run> {
        self.positioned_runs()
            .map(|(line_index, run)| {
                let run_range = run.run().text_range();
                let fallback_range = self.projection.range(run_range);
                let brush = run.style().brush;
                let mut ranges = run.run().visual_clusters().flat_map(|cluster| {
                    let range = cluster.text_range();
                    let range = self.projection.range(range);
                    cluster.glyphs().map(move |_| range)
                });
                let glyphs: Vec<Glyph> = run
                    .positioned_glyphs()
                    .map(|glyph| {
                        Glyph::new(
                            glyph.id,
                            glyph.x,
                            glyph.y,
                            glyph.advance,
                            ranges.next().unwrap_or(fallback_range),
                        )
                    })
                    .collect();
                let style = self.style_for_run(fallback_range, &glyphs, brush);
                Run::new(
                    line_index,
                    fallback_range,
                    FontRef::from_parley(run.run().font()),
                    style,
                    brush,
                    RunMetrics::new(
                        run.run().font_size(),
                        run.baseline(),
                        run.offset(),
                        run.advance(),
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
            .glyph_runs()
            .into_iter()
            .map(|run| {
                RunMetricFact::new(
                    run.line(),
                    run.range(),
                    run.font_size(),
                    run.baseline(),
                    run.offset(),
                    run.advance(),
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
                        _ => BaselineShiftFact::BackendBottomOnBaseline,
                    },
                )
            })
            .collect();

        InlineMetricFacts::new(self.metrics(), lines, runs, inline_boxes)
    }

    fn style_for_run(&self, fallback_range: Range, glyphs: &[Glyph], brush: Brush) -> Style {
        glyphs
            .iter()
            .map(|glyph| self.style_for_range(glyph.range))
            .find(|style| style.brush == brush)
            .or_else(|| {
                glyphs
                    .first()
                    .map(|glyph| self.style_for_range(glyph.range))
            })
            .unwrap_or_else(|| self.style_for_range(fallback_range))
    }

    pub(crate) fn style_for_range(&self, range: Range) -> Style {
        if range.start < range.end {
            self.style_at(range.start)
        } else if range.start > 0 {
            self.style_at(range.start - 1)
        } else {
            self.default_style.clone()
        }
    }

    fn style_at(&self, index: usize) -> Style {
        self.source
            .spans
            .iter()
            .rfind(|span| span.range.contains(index))
            .map(|span| span.style.clone())
            .unwrap_or_else(|| self.default_style.clone())
    }

    pub fn inline_boxes(&self) -> Vec<PositionedInlineBox> {
        self.inner
            .lines()
            .enumerate()
            .flat_map(|(line_index, line)| {
                line.items().filter_map(move |item| {
                    let PositionedLayoutItem::InlineBox(box_) = item else {
                        return None;
                    };
                    let source_box = self
                        .source
                        .boxes
                        .iter()
                        .find(|source_box| source_box.id.as_u64() == box_.id);
                    Some(PositionedInlineBox::new(
                        Id::from_u64(box_.id),
                        match box_.kind {
                            parley::InlineBoxKind::InFlow => InlineBoxKind::InFlow,
                            parley::InlineBoxKind::OutOfFlow
                            | parley::InlineBoxKind::CustomOutOfFlow => InlineBoxKind::OutOfFlow,
                        },
                        Rect::new(box_.x, box_.y, box_.width, box_.height),
                        source_box.map_or(0, |source_box| source_box.index),
                        source_box.map_or(VerticalAlign::Baseline, |source_box| {
                            source_box.vertical_align
                        }),
                        line_index,
                    ))
                })
            })
            .collect()
    }

    pub fn clusters(&self) -> Vec<Cluster> {
        let mut clusters = Vec::new();
        for line in self.inner.lines() {
            let line_metrics = line.metrics();
            for run in line.runs() {
                for cluster in run.clusters() {
                    let range = cluster.text_range();
                    let x = cluster.visual_offset().unwrap_or(line_metrics.offset);
                    clusters.push(Cluster::new(
                        self.projection.range(range),
                        Rect::new(
                            x,
                            line_metrics.block_min_coord,
                            cluster.advance(),
                            line_metrics.block_max_coord - line_metrics.block_min_coord,
                        ),
                    ));
                }
            }
        }
        clusters
    }

    pub fn decorations(&self) -> Vec<DecorationRun> {
        self.decoration_views().collect()
    }

    #[must_use]
    pub fn hit(&self, point: Point) -> Hit {
        for box_ in self.inline_boxes() {
            if box_.rect.contains(point) {
                return Hit::InlineBox(box_.id);
            }
        }
        if self.inner.is_empty() {
            return Hit::None;
        }
        let metrics = self.metrics();
        if !Rect::new(0.0, 0.0, metrics.size.width, metrics.size.height).contains(point) {
            return Hit::None;
        }
        Hit::Text(self.mapped_cursor(parley::Cursor::from_point(&self.inner, point.x, point.y)))
    }

    fn provider_cursor(&self, cursor: Cursor) -> Result<parley::Cursor> {
        super::range::validate_index(self.source.text(), cursor.index, "cursor")?;
        let stop = self.projection.caret(cursor.index, cursor.affinity);
        Ok(parley::Cursor::from_byte_index(
            &self.inner,
            self.projection.byte(stop).0,
            cursor.affinity.into(),
        ))
    }

    fn mapped_cursor(&self, cursor: parley::Cursor) -> Cursor {
        let unit = self
            .projection
            .unit(ProviderIndex(cursor.index()))
            .expect("provider cursor uses validated scalar boundaries");
        let affinity = cursor.affinity().into();
        Cursor::new(self.projection.caret(unit, affinity), affinity)
    }

    pub fn cursor(&self, cursor: Cursor) -> Result<CursorGeometry> {
        let cursor = self.provider_cursor(cursor)?;
        Ok(CursorGeometry::new(rect_from_bounds(
            cursor.geometry(&self.inner, 1.0),
        )))
    }

    pub fn selection(&self, selection: Selection) -> Result<SelectionGeometry> {
        super::range::validate(self.source.text(), selection.range())?;
        if selection.is_collapsed() {
            return Ok(SelectionGeometry::default());
        }
        let range = self.projection.selection_range(selection.range());
        let start = self.provider_cursor(Cursor::new(range.start, Affinity::After))?;
        let end = self.provider_cursor(Cursor::new(range.end, Affinity::Before))?;
        let projected = parley::Selection::new(start, end);
        Ok(SelectionGeometry::new(
            projected
                .geometry(&self.inner)
                .into_iter()
                .map(|(bounds, line)| SelectionRect::new(rect_from_bounds(bounds), line))
                .collect(),
        ))
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
        let cursor = self.provider_cursor(cursor)?;
        let mut moved = match movement {
            Movement::PreviousCluster => cursor.previous_visual(&self.inner),
            Movement::NextCluster => cursor.next_visual(&self.inner),
            Movement::PreviousWord => cursor.previous_logical_word(&self.inner),
            Movement::NextWord => cursor.next_logical_word(&self.inner),
            Movement::LineStart => self.line_boundary(cursor.index(), LineBoundary::Start),
            Movement::LineEnd => self.line_boundary(cursor.index(), LineBoundary::End),
            Movement::PreviousLine => self.vertical_movement(cursor, VerticalMovement::Previous),
            Movement::NextLine => self.vertical_movement(cursor, VerticalMovement::Next),
            Movement::DocumentStart => {
                parley::Cursor::from_byte_index(&self.inner, 0, parley::Affinity::Downstream)
            }
            Movement::DocumentEnd => parley::Cursor::from_byte_index(
                &self.inner,
                self.projection.text.len(),
                parley::Affinity::Upstream,
            ),
        };
        if matches!(movement, Movement::PreviousCluster | Movement::NextCluster) {
            // Each scalar boundary has at most two affinities. Traverse actual provider stops,
            // never return a caret inside an indivisible source grapheme.
            for _ in 0..=self.source.text.len().saturating_mul(2) {
                let unit = self.projection.unit(ProviderIndex(moved.index()))?;
                if self.projection.is_stop(unit) {
                    break;
                }
                let next = if movement == Movement::PreviousCluster {
                    moved.previous_visual(&self.inner)
                } else {
                    moved.next_visual(&self.inner)
                };
                if next.index() == moved.index() && next.affinity() == moved.affinity() {
                    break;
                }
                moved = next;
            }
        }
        Ok(self.mapped_cursor(moved))
    }

    pub fn move_selection(
        &self,
        selection: Selection,
        movement: Movement,
        extend: bool,
    ) -> Result<Selection> {
        super::range::validate(self.source.text(), selection.range())?;
        let moved = self.move_cursor(selection.focus, movement)?;
        Ok(if extend {
            Selection::new(selection.anchor, moved)
        } else {
            Selection::collapsed(moved)
        })
    }

    fn line_boundary(&self, index: usize, boundary: LineBoundary) -> parley::Cursor {
        let unit = self
            .projection
            .unit(ProviderIndex(index))
            .expect("validated cursor boundary");
        let Some(line) = self.line_for_index(unit) else {
            return parley::Cursor::from_byte_index(
                &self.inner,
                match boundary {
                    LineBoundary::Start => 0,
                    LineBoundary::End => self.projection.text.len(),
                },
                match boundary {
                    LineBoundary::Start => parley::Affinity::Downstream,
                    LineBoundary::End => parley::Affinity::Upstream,
                },
            );
        };
        match boundary {
            LineBoundary::Start => parley::Cursor::from_byte_index(
                &self.inner,
                self.projection.byte(line.range.start).0,
                parley::Affinity::Downstream,
            ),
            LineBoundary::End => parley::Cursor::from_byte_index(
                &self.inner,
                self.projection.byte(self.line_visible_end(line.range)).0,
                parley::Affinity::Upstream,
            ),
        }
    }

    fn vertical_movement(
        &self,
        cursor: parley::Cursor,
        movement: VerticalMovement,
    ) -> parley::Cursor {
        let lines = self.lines();
        let Some(current_index) = self.line_index_for_cursor(
            self.projection
                .unit(ProviderIndex(cursor.index()))
                .expect("validated cursor boundary"),
            &lines,
        ) else {
            return cursor;
        };
        let next_index = match movement {
            VerticalMovement::Previous => current_index.checked_sub(1),
            VerticalMovement::Next => {
                let index = current_index + 1;
                (index < lines.len()).then_some(index)
            }
        };
        let Some(next_index) = next_index else {
            return cursor;
        };
        let geometry = cursor.geometry(&self.inner, 1.0);
        let line = lines[next_index];
        let y = line.bounds.origin.y + line.bounds.size.height * 0.5;
        parley::Cursor::from_point(&self.inner, geometry.x0 as f32, y)
    }

    fn line_for_index(&self, index: usize) -> Option<Line> {
        let lines = self.lines();
        self.line_index_for_cursor(index, &lines)
            .and_then(|line_index| lines.get(line_index).copied())
    }

    fn line_index_for_cursor(&self, index: usize, lines: &[Line]) -> Option<usize> {
        lines
            .iter()
            .position(|line| index >= line.range.start && index < line.range.end)
            .or_else(|| {
                lines
                    .iter()
                    .rposition(|line| index == line.range.end && line.range.end != line.range.start)
            })
            .or_else(|| {
                (!lines.is_empty() && index >= self.source.text.len()).then_some(lines.len() - 1)
            })
    }

    fn line_visible_end(&self, range: Range) -> usize {
        let mut end = range.end;
        while end > range.start {
            if matches!(self.source.text.code_units()[end - 1], 0x0a | 0x0d) {
                end -= 1;
            } else {
                break;
            }
        }
        end
    }

    pub fn apply(&self, edit: Edit) -> Result<Source> {
        edit.normalize(&self.source)?.apply_to(self.source.clone())
    }
}

pub(crate) fn decoration_top(baseline: f32, offset_from_baseline: f32) -> f32 {
    baseline - offset_from_baseline
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    size: Size,
    full_width: f32,
    line_count: usize,
    first_baseline: Option<f32>,
    last_baseline: Option<f32>,
    overflow: bool,
}

impl Metrics {
    #[must_use]
    pub const fn new(
        size: Size,
        full_width: f32,
        line_count: usize,
        first_baseline: Option<f32>,
        last_baseline: Option<f32>,
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
    pub const fn full_width(self) -> f32 {
        self.full_width
    }

    #[must_use]
    pub const fn line_count(self) -> usize {
        self.line_count
    }

    #[must_use]
    pub const fn first_baseline(self) -> Option<f32> {
        self.first_baseline
    }

    #[must_use]
    pub const fn last_baseline(self) -> Option<f32> {
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
    baseline: f32,
    bounds: Rect,
}

impl Line {
    #[must_use]
    pub const fn new(index: usize, range: Range, baseline: f32, bounds: Rect) -> Self {
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
    pub const fn baseline(self) -> f32 {
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
    font: FontRef,
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
        font: FontRef,
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
    pub fn font(&self) -> &FontRef {
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
    pub const fn baseline(&self) -> f32 {
        self.metrics.baseline()
    }

    #[must_use]
    pub const fn offset(&self) -> f32 {
        self.metrics.offset()
    }

    #[must_use]
    pub const fn advance(&self) -> f32 {
        self.metrics.advance()
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
    baseline: f32,
    offset: f32,
    advance: f32,
}

impl RunMetricFact {
    #[must_use]
    pub const fn new(
        line: usize,
        range: Range,
        font_size: f32,
        baseline: f32,
        offset: f32,
        advance: f32,
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
    pub const fn baseline(self) -> f32 {
        self.baseline
    }

    #[must_use]
    pub const fn offset(self) -> f32 {
        self.offset
    }

    #[must_use]
    pub const fn advance(self) -> f32 {
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
    BackendBottomOnBaseline,
    Requested(BaselineShift),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunMetrics {
    font_size: f32,
    baseline: f32,
    offset: f32,
    advance: f32,
}

impl RunMetrics {
    #[must_use]
    pub const fn new(font_size: f32, baseline: f32, offset: f32, advance: f32) -> Self {
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
    pub const fn baseline(self) -> f32 {
        self.baseline
    }

    #[must_use]
    pub const fn offset(self) -> f32 {
        self.offset
    }

    #[must_use]
    pub const fn advance(self) -> f32 {
        self.advance
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontRef {
    id: u64,
    data: Option<FontData>,
}

impl FontRef {
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self { id, data: None }
    }

    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    #[must_use]
    pub fn with_data(mut self, data: FontData) -> Self {
        self.data = Some(data);
        self
    }

    #[must_use]
    pub fn data(&self) -> Option<&FontData> {
        self.data.as_ref()
    }

    fn from_parley(data: &parley::FontData) -> Self {
        Self {
            id: data.data.id(),
            data: Some(FontData::from_parley(data.clone())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontData {
    data: parley::FontData,
}

impl FontData {
    fn from_parley(data: parley::FontData) -> Self {
        Self { data }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.data.data.as_ref()
    }

    #[must_use]
    pub const fn index(&self) -> u32 {
        self.data.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    id: u32,
    x: f32,
    y: f32,
    advance: f32,
    range: Range,
}

impl Glyph {
    #[must_use]
    pub const fn new(id: u32, x: f32, y: f32, advance: f32, range: Range) -> Self {
        Self {
            id,
            x,
            y,
            advance,
            range,
        }
    }

    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }

    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }

    #[must_use]
    pub const fn advance(self) -> f32 {
        self.advance
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hit {
    Text(Cursor),
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

impl From<parley::Affinity> for Affinity {
    fn from(affinity: parley::Affinity) -> Self {
        match affinity {
            parley::Affinity::Upstream => Self::Before,
            parley::Affinity::Downstream => Self::After,
        }
    }
}

impl From<Affinity> for parley::Affinity {
    fn from(affinity: Affinity) -> Self {
        match affinity {
            Affinity::Before => Self::Upstream,
            Affinity::After => Self::Downstream,
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineBoundary {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VerticalMovement {
    Previous,
    Next,
}
