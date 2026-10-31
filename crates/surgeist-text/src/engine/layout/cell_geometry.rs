//! Executed Cell geometry in explicit intrinsic, line-local and placed domains.
//!
//! Getters evaluate immutable admitted operands. Producers use the same evaluator
//! with numeric qualification before publishing logical or physical observations.
use super::{
    CellFrame, FinalizedCell, FinalizedCluster, FinalizedGlyph, FinalizedLineData, FinalizedRun,
    FinalizedRunFrame, LineMetrics, RunMetrics,
};
use crate::engine::PlacedLine;
use crate::{
    Error, ErrorCode, LineOrientation, Point, Rect, Result, TextNumericError, TextNumericPhase,
};

struct Arithmetic {
    phase: Option<TextNumericPhase>,
    error: Option<Error>,
}
impl Arithmetic {
    fn published() -> Self {
        Self {
            phase: None,
            error: None,
        }
    }
    fn checked(phase: TextNumericPhase) -> Self {
        Self {
            phase: Some(phase),
            error: None,
        }
    }
    fn reject(&mut self, field: &'static str, reason: TextNumericError, value: f64) {
        if self.error.is_none()
            && let Some(phase) = self.phase
        {
            self.error = Some(crate::error::text_numeric_error(
                field, phase, reason, value,
            ));
        }
    }
    fn finite(&mut self, field: &'static str, value: f64) -> f64 {
        if !value.is_finite() {
            self.reject(field, TextNumericError::NonFinite, value);
        }
        value
    }
    fn add(&mut self, field: &'static str, base: f64, delta: f64) -> f64 {
        let value = self.finite(field, base + delta);
        if delta != 0.0 && value == base {
            self.reject(field, TextNumericError::NonZeroLost, delta);
        }
        value
    }
    // Metric corners are rounded coordinates, not independently required moves.
    // Their true allocation spans are qualified after the complete mapping.
    fn coordinate(&mut self, field: &'static str, base: f64, offset: f64) -> f64 {
        self.finite(field, base + offset)
    }
    fn mul(&mut self, field: &'static str, a: f64, b: f64) -> f64 {
        let value = self.finite(field, a * b);
        if a != 0.0 && b != 0.0 && value == 0.0 {
            self.reject(field, TextNumericError::NonZeroLost, b);
        }
        value
    }
    fn rect(&mut self, left: f64, top: f64, right: f64, bottom: f64) -> Rect {
        let width = self.add("cell rectangle width", right, -left);
        let height = self.add("cell rectangle height", bottom, -top);
        self.add("cell rectangle right", left, width);
        self.add("cell rectangle bottom", top, height);
        Rect::new(left, top, width, height)
    }
    fn metric_rect(&mut self, left: f64, top: f64, right: f64, bottom: f64) -> Rect {
        let width = self.finite("cell metric rectangle width", right - left);
        let height = self.finite("cell metric rectangle height", bottom - top);
        self.add("cell metric rectangle right", left, width);
        self.add("cell metric rectangle bottom", top, height);
        Rect::new(left, top, width, height)
    }
    fn frame(&mut self, frame: CellFrame) {
        self.finite("cell frame x scale", frame.x_scale);
        self.finite("cell frame x offset", frame.x_offset);
        self.finite("cell frame baseline", frame.baseline);
        if frame.x_scale <= 0.0 {
            self.reject(
                "cell frame x scale",
                if frame.x_scale == 0.0 {
                    TextNumericError::PositiveLost
                } else {
                    TextNumericError::OutOfRange
                },
                frame.x_scale,
            );
        }
    }
    fn layout_transform(&mut self, transform: super::CellTransform) {
        self.finite("cell tab scale", transform.x_scale);
        self.finite("cell tab offset", transform.x_offset);
        if transform.x_scale <= 0.0 {
            self.reject(
                "cell tab scale",
                TextNumericError::PositiveLost,
                transform.x_scale,
            );
        }
    }
    fn span(&mut self, field: &'static str, first: Point, last: Point, extent: f64) {
        if extent != 0.0 && first == last {
            self.reject(field, TextNumericError::PositiveLost, extent.abs());
        }
    }
    fn finish(self) -> Result<()> {
        self.error.map_or(Ok(()), Err)
    }
}

fn extrema(points: [Point; 4], math: &mut Arithmetic) -> (f64, f64, f64, f64) {
    let mut left = points[0].x;
    let mut right = left;
    let mut top = points[0].y;
    let mut bottom = top;
    for point in points {
        math.finite("cell rectangle x", point.x);
        math.finite("cell rectangle y", point.y);
        left = left.min(point.x);
        right = right.max(point.x);
        top = top.min(point.y);
        bottom = bottom.max(point.y);
    }
    (left, top, right, bottom)
}
fn hull(points: [Point; 4], math: &mut Arithmetic) -> Rect {
    let (left, top, right, bottom) = extrema(points, math);
    math.rect(left, top, right, bottom)
}
fn metric_hull(points: [Point; 4], math: &mut Arithmetic) -> Rect {
    let (left, top, right, bottom) = extrema(points, math);
    math.metric_rect(left, top, right, bottom)
}

/// Square-relative horizontal coordinates to actual line-local inline/under.
fn square_point(
    cell: &FinalizedCell,
    orientation: LineOrientation,
    point: Point,
    mut add: impl FnMut(&'static str, f64, f64) -> f64,
) -> Point {
    match orientation {
        LineOrientation::Horizontal => Point::new(
            add("cell line inline", cell.inline.start, point.x),
            add("cell line under", cell.under.start, point.y),
        ),
        LineOrientation::Vertical | LineOrientation::SidewaysClockwise => Point::new(
            add("cell line inline", cell.inline.start, point.y),
            add("cell line under", cell.under.end, -point.x),
        ),
        LineOrientation::SidewaysCounterclockwise => Point::new(
            add("cell line inline", cell.inline.end, -point.y),
            add("cell line under", cell.under.start, point.x),
        ),
    }
}
fn metric_points(edges: (f64, f64), metrics: RunMetrics, math: &mut Arithmetic) -> [Point; 4] {
    let (x, end) = edges;
    let top = -metrics.ascent;
    let bottom = metrics.descent;
    math.finite("cell cluster ascent", top);
    math.finite("cell cluster descent", bottom);
    [
        Point::new(x, top),
        Point::new(end, top),
        Point::new(end, bottom),
        Point::new(x, bottom),
    ]
}
fn metric_rect(
    data: &FinalizedLineData,
    cluster: &FinalizedCluster,
    orientation: LineOrientation,
    placed: Option<&PlacedLine>,
    math: &mut Arithmetic,
) -> Rect {
    let FinalizedRunFrame::Cell(index) = cluster.frame else {
        return data.cell_rect(cluster.x, cluster.advance, cluster.reference);
    };
    let cell = &data.cells[index];
    if matches!(cluster.content, super::ClusterContent::CssTab(_)) {
        return tab_cluster_rect(cell, cluster, orientation, placed, math);
    }
    let frame = cell
        .frame
        .expect("published inner cluster has an executed Cell frame");
    math.frame(frame);
    let run = cluster
        .content
        .font_run()
        .expect("published inner cluster has retained font metrics");
    let end = math.add("cell cluster allocation end", cluster.x, cluster.advance);
    font_metric_rect(
        cell,
        data.font_runs[run].metrics,
        (cluster.x, end),
        cluster.advance,
        orientation,
        placed,
        math,
    )
}

// Both callers qualify the genuine retained frame before entering this mapper.
// The original whole-cluster advance stays independent of its rounded end;
// directed Range edges instead supply their own actual coordinate difference.
fn font_metric_rect(
    cell: &FinalizedCell,
    metrics: RunMetrics,
    edges: (f64, f64),
    advance: f64,
    orientation: LineOrientation,
    placed: Option<&PlacedLine>,
    math: &mut Arithmetic,
) -> Rect {
    let frame = cell
        .frame
        .expect("font metric geometry has an executed Cell frame");
    let points = metric_points(edges, metrics, math);
    let points = if let Some(line) = placed {
        // Use the real placed horizontal frame; no inverse/forward anchor roundtrip.
        let glyph_frame = crate::layout::cell_glyph_frame(line, cell, Point::new(0.0, 0.0));
        let origin = glyph_frame.origin();
        points.map(|point| {
            let x = math.finite("placed cell cluster x scale", frame.x_scale * point.x);
            Point::new(
                math.coordinate("placed cell cluster x", origin.x, x),
                math.coordinate("placed cell cluster y", origin.y, point.y),
            )
        })
    } else {
        points.map(|point| {
            let x = math.finite("cell cluster x scale", frame.x_scale * point.x);
            let horizontal = math.coordinate("cell cluster centered x", frame.x_offset, x);
            let vertical = math.coordinate("cell cluster inner baseline", frame.baseline, point.y);
            square_point(
                cell,
                orientation,
                Point::new(horizontal, vertical),
                |field, base, offset| math.coordinate(field, base, offset),
            )
        })
    };
    let horizontal = math.mul("cell cluster metric advance scale", frame.x_scale, advance);
    let vertical = math.finite(
        "cell cluster font metric span",
        metrics.ascent + metrics.descent,
    );
    math.span(
        "cell cluster horizontal metric span",
        points[0],
        points[1],
        horizontal,
    );
    math.span(
        "cell cluster vertical metric span",
        points[0],
        points[3],
        vertical,
    );
    metric_hull(points, math)
}
/// Map genuine directed inner-horizontal metric edges through an actual placed
/// Cell. These are metric coordinates; only their complete mapped spans are
/// required displacements. No provider cluster or new baseline is constructed.
pub(crate) fn placed_metric_rect(
    line: &PlacedLine,
    cell: &FinalizedCell,
    edges: (f64, f64),
    metrics: RunMetrics,
    phase: TextNumericPhase,
) -> Result<Rect> {
    let frame = cell
        .frame
        .ok_or_else(|| invalid("Cell font metric geometry has no executed frame"))?;
    let mut math = Arithmetic::checked(phase);
    math.frame(frame);
    math.finite("cell metric directed start", edges.0);
    math.finite("cell metric directed end", edges.1);
    let advance = math.finite("cell metric directed advance", edges.1 - edges.0);
    let rect = font_metric_rect(
        cell,
        metrics,
        edges,
        advance,
        line.orientation,
        Some(line),
        &mut math,
    );
    math.finish()?;
    Ok(rect)
}
pub(crate) fn placed_cluster_rect(line: &PlacedLine, cluster: &FinalizedCluster) -> Rect {
    match cluster.frame {
        FinalizedRunFrame::Line => crate::layout::line_rect(
            line,
            line.finalized
                .data
                .cell_rect(cluster.x, cluster.advance, cluster.reference),
        ),
        FinalizedRunFrame::Cell(_) => metric_rect(
            &line.finalized.data,
            cluster,
            line.orientation,
            Some(line),
            &mut Arithmetic::published(),
        ),
    }
}
pub(crate) fn qualify_cluster_rect(
    data: &FinalizedLineData,
    cluster: &FinalizedCluster,
    orientation: LineOrientation,
    placed: Option<&PlacedLine>,
    phase: TextNumericPhase,
) -> Result<()> {
    if let FinalizedRunFrame::Cell(index) = cluster.frame {
        let cell = data
            .cells
            .get(index)
            .ok_or_else(|| invalid("inner cluster has no actual Cell"))?;
        let tab = matches!(cluster.content, super::ClusterContent::CssTab(_));
        if (tab && cell.transform.is_none())
            || (!tab
                && (cell.frame.is_none()
                    || cluster
                        .content
                        .font_run()
                        .is_none_or(|run| run >= data.font_runs.len())))
        {
            return Err(invalid("inner cluster has no executed frame/font metrics"));
        }
        let mut math = Arithmetic::checked(phase);
        metric_rect(data, cluster, orientation, placed, &mut math);
        math.finish()?;
    }
    Ok(())
}

fn allocation_rect(cell: &FinalizedCell, metrics: &LineMetrics, math: &mut Arithmetic) -> Rect {
    let end = math.add("cell edge flow end", cell.inline.start, cell.advance);
    if end != cell.flow_end {
        math.reject(
            "cell edge flow association",
            TextNumericError::OutOfRange,
            end,
        );
    }
    let left = cell.inline.start.min(cell.flow_end);
    let width = math.finite("cell edge allocation width", cell.advance.abs());
    let height = math.add("cell edge line band", metrics.block_max, -metrics.block_min);
    math.add("cell edge allocation right", left, width);
    math.add("cell edge allocation bottom", metrics.block_min, height);
    Rect::new(left, metrics.block_min, width, height)
}
pub(crate) fn edge_rect(cell: &FinalizedCell, metrics: &LineMetrics) -> Rect {
    allocation_rect(cell, metrics, &mut Arithmetic::published())
}
pub(crate) fn qualify_edge_rect(
    cell: &FinalizedCell,
    metrics: &LineMetrics,
    phase: TextNumericPhase,
) -> Result<()> {
    let mut math = Arithmetic::checked(phase);
    allocation_rect(cell, metrics, &mut math);
    math.finish()
}

/// Actual retained run/glyph and canonical outline box, mapped into line-local
/// metric axes. This is a fallible requested outline observation, not paint ink.
pub(crate) fn outline_rect(
    cell: &FinalizedCell,
    run: &FinalizedRun,
    glyph: &FinalizedGlyph,
    bounds: Rect,
    orientation: LineOrientation,
) -> Result<Rect> {
    let frame = cell
        .frame
        .ok_or_else(|| invalid("Cell outline has no executed frame"))?;
    let mut math = Arithmetic::checked(TextNumericPhase::GlyphOutline);
    math.frame(frame);
    let rx = math.mul("cell outline run x scale", frame.x_scale, run.x);
    let rx = math.add("cell outline run x", frame.x_offset, rx);
    let ry = math.add("cell outline run y", frame.baseline, run.y);
    let pen = math.add(
        "cell outline raw x offset",
        glyph.pen,
        f64::from(glyph.glyph.x),
    );
    let pen = math.mul("cell outline pen scale", frame.x_scale, pen);
    let x = math.add("cell outline glyph x", rx, pen);
    let y = math.add("cell outline glyph y", ry, f64::from(glyph.glyph.y));
    let left = math.mul("cell outline left scale", frame.x_scale, bounds.origin.x);
    let left = math.add("cell outline left", x, left);
    let top = math.add("cell outline top", y, bounds.origin.y);
    let width = math.mul("cell outline width scale", frame.x_scale, bounds.size.width);
    let right = math.add("cell outline right", left, width);
    let bottom = math.add("cell outline bottom", top, bounds.size.height);
    let points = [
        Point::new(left, top),
        Point::new(right, top),
        Point::new(right, bottom),
        Point::new(left, bottom),
    ]
    .map(|point| {
        square_point(cell, orientation, point, |field, base, delta| {
            math.add(field, base, delta)
        })
    });
    math.span(
        "cell outline horizontal extent",
        points[0],
        points[1],
        width,
    );
    math.span(
        "cell outline vertical extent",
        points[0],
        points[3],
        bounds.size.height,
    );
    let rect = hull(points, &mut math);
    math.finish()?;
    Ok(rect)
}
fn invalid(message: &'static str) -> Error {
    Error::new(ErrorCode::LayoutFailed, message)
}

fn layout_point(
    cell: &FinalizedCell,
    transform: super::CellTransform,
    point: Point,
    orientation: LineOrientation,
    placed: Option<&PlacedLine>,
    math: &mut Arithmetic,
) -> Point {
    let x = math.mul("cell layout x scale", transform.x_scale, point.x);
    let x = math.coordinate("cell layout x offset", transform.x_offset, x);
    if let Some(line) = placed {
        let origin = crate::layout::cell_square_origin(line, cell);
        Point::new(
            math.coordinate("placed cell layout x", origin.x, x),
            math.coordinate("placed cell layout y", origin.y, point.y),
        )
    } else {
        square_point(
            cell,
            orientation,
            Point::new(x, point.y),
            |field, base, offset| math.coordinate(field, base, offset),
        )
    }
}
/// Map an actual retained native-layout point without borrowing a provider
/// baseline. The square already contains its enclosing scope displacement.
pub(crate) fn placed_layout_point(
    line: &PlacedLine,
    cell: &FinalizedCell,
    point: Point,
    phase: TextNumericPhase,
) -> Result<Point> {
    let transform = cell
        .transform
        .ok_or_else(|| invalid("Cell layout point has no executed transform"))?;
    let mut math = Arithmetic::checked(phase);
    math.layout_transform(transform);
    let point = layout_point(
        cell,
        transform,
        point,
        line.orientation,
        Some(line),
        &mut math,
    );
    math.finish()?;
    Ok(point)
}
fn tab_cluster_rect(
    cell: &FinalizedCell,
    cluster: &FinalizedCluster,
    orientation: LineOrientation,
    placed: Option<&PlacedLine>,
    math: &mut Arithmetic,
) -> Rect {
    let transform = cell
        .transform
        .expect("a published TAB has its executed Cell layout");
    let end = math.add("cell tab allocation end", cluster.x, cluster.advance);
    let extent = math.finite(
        "cell tab square extent",
        cell.inline.end - cell.inline.start,
    );
    let points = [
        Point::new(cluster.x, 0.0),
        Point::new(end, 0.0),
        Point::new(end, extent),
        Point::new(cluster.x, extent),
    ]
    .map(|point| layout_point(cell, transform, point, orientation, placed, math));
    metric_hull(points, math)
}
/// Qualify the actual executed endpoint pairs, independent of glyph baselines.
pub(crate) fn qualify_tabs(
    cell: &FinalizedCell,
    orientation: LineOrientation,
    placed: Option<&PlacedLine>,
    phase: TextNumericPhase,
) -> Result<()> {
    if cell.tabs.is_empty() {
        return Ok(());
    }
    let transform = cell
        .transform
        .ok_or_else(|| invalid("Cell TAB spans lost their layout"))?;
    let mut math = Arithmetic::checked(phase);
    math.layout_transform(transform);
    for span in &cell.tabs {
        let before = layout_point(
            cell,
            transform,
            Point::new(span.before, 0.0),
            orientation,
            placed,
            &mut math,
        );
        let after = layout_point(
            cell,
            transform,
            Point::new(span.after, 0.0),
            orientation,
            placed,
            &mut math,
        );
        if span.amount != 0.0 && before == after {
            math.reject(
                "cell tab displacement",
                TextNumericError::NonZeroLost,
                span.amount,
            );
        }
    }
    math.finish()
}
