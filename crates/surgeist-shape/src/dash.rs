use super::*;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dash {
    density: f64,
    phase: f64,
    rounded: bool,
    sides: SideSet,
    anchors: [DashAnchor; 4],
    anchor_count: usize,
    constraints: [Option<DashConstraint>; 1],
}

impl Dash {
    pub fn try_new(density: f64) -> Result<Self> {
        validate_non_negative_kind(density, "dash density", NumericKind::Dash)?;
        if density <= f64::EPSILON {
            return Err(Error::new(
                ErrorCode::InvalidDash,
                "dash density must be greater than zero",
            ));
        }
        Ok(Self::new_unchecked(density))
    }

    fn new_unchecked(density: f64) -> Self {
        let anchor = DashAnchor::corner(Corner::TopLeft);
        Self {
            density,
            phase: 0.0,
            rounded: false,
            sides: SideSet::all(),
            anchors: [anchor; 4],
            anchor_count: 0,
            constraints: [None],
        }
    }

    #[must_use]
    pub fn dashed() -> Self {
        Self::new_unchecked(1.0)
    }

    #[must_use]
    pub fn dotted() -> Self {
        Self::dashed().rounded().circular()
    }

    pub fn try_with_density(mut self, density: f64) -> Result<Self> {
        validate_non_negative_kind(density, "dash density", NumericKind::Dash)?;
        if density <= f64::EPSILON {
            return Err(Error::new(
                ErrorCode::InvalidDash,
                "dash density must be greater than zero",
            ));
        }
        self.density = density;
        Ok(self)
    }

    pub fn with_sides(mut self, sides: SideSet) -> Result<Self> {
        if sides.is_empty() {
            return Err(Error::new(ErrorCode::InvalidDash, "dash side set is empty"));
        }
        self.sides = sides;
        Ok(self)
    }

    #[must_use]
    pub fn rounded(mut self) -> Self {
        self.rounded = true;
        self
    }

    #[must_use]
    pub fn circular(mut self) -> Self {
        self.constraints[0] = Some(DashConstraint::Circular);
        self.rounded = true;
        self
    }

    #[must_use]
    pub fn with_corner_anchors(mut self) -> Self {
        self.anchor_count = 4;
        self.anchors = [
            DashAnchor::corner(Corner::TopLeft),
            DashAnchor::corner(Corner::TopRight),
            DashAnchor::corner(Corner::BottomRight),
            DashAnchor::corner(Corner::BottomLeft),
        ];
        self
    }

    pub fn with_anchor(mut self, anchor: DashAnchor) -> Result<Self> {
        if self.anchor_count >= self.anchors.len() {
            return Err(Error::new(
                ErrorCode::InvalidDash,
                "dash supports at most four anchors",
            ));
        }
        self.anchors[self.anchor_count] = anchor;
        self.anchor_count += 1;
        Ok(self)
    }

    pub fn try_with_phase(mut self, phase: f64) -> Result<Self> {
        validate_finite(phase, "dash phase")?;
        self.phase = phase;
        Ok(self)
    }

    #[must_use]
    pub fn density(self) -> f64 {
        self.density
    }

    #[must_use]
    pub fn phase(self) -> f64 {
        self.phase
    }

    #[must_use]
    pub fn sides(self) -> SideSet {
        self.sides
    }

    #[must_use]
    pub fn is_rounded(self) -> bool {
        self.rounded
    }

    #[must_use]
    pub fn anchors(&self) -> &[DashAnchor] {
        &self.anchors[..self.anchor_count]
    }

    #[must_use]
    pub fn has_constraint(self, constraint: DashConstraint) -> bool {
        self.constraints
            .iter()
            .flatten()
            .any(|value| *value == constraint)
    }

    pub fn validate(self) -> Result<()> {
        validate_non_negative_kind(self.density, "dash density", NumericKind::Dash)?;
        validate_finite(self.phase, "dash phase")?;
        if self.density <= f64::EPSILON {
            return Err(Error::new(
                ErrorCode::InvalidDash,
                "dash density must be greater than zero",
            ));
        }
        if self.sides.is_empty() {
            return Err(Error::new(ErrorCode::InvalidDash, "dash side set is empty"));
        }
        for anchor in self.anchors.iter().take(self.anchor_count) {
            match anchor.kind {
                DashAnchorKind::Corner(_) => {}
                DashAnchorKind::ContourOffset(offset) => {
                    let _ = offset.get();
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashAnchor {
    kind: DashAnchorKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DashAnchorKind {
    Corner(Corner),
    ContourOffset(NonNegative),
}

impl DashAnchor {
    #[must_use]
    pub const fn corner(corner: Corner) -> Self {
        Self {
            kind: DashAnchorKind::Corner(corner),
        }
    }

    pub fn try_contour_offset(offset: f64) -> Result<Self> {
        Ok(Self {
            kind: DashAnchorKind::ContourOffset(NonNegative::try_new(offset, NumericKind::Dash)?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DashConstraint {
    Circular,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DashGeometry {
    segments: Vec<DashSegment>,
}

impl DashGeometry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    #[must_use]
    pub fn segments(&self) -> &[DashSegment] {
        &self.segments
    }

    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.segments
            .iter()
            .map(DashSegment::bounds)
            .fold(Rect::empty(), Rect::union)
    }

    #[must_use]
    pub fn contour_length(&self) -> f64 {
        self.segments.iter().map(DashSegment::contour_length).sum()
    }

    fn push(&mut self, segment: DashSegment) {
        self.segments.push(segment);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DashSegment {
    path: Path,
    width: f64,
    rounded: bool,
}

impl DashSegment {
    pub fn try_new(path: Path, width: f64, rounded: bool) -> Result<Self> {
        path.validate()?;
        validate_non_negative_kind(width, "dash segment width", NumericKind::Stroke)?;
        Ok(Self::new_unchecked(path, width, rounded))
    }

    #[must_use]
    pub(crate) const fn new_unchecked(path: Path, width: f64, rounded: bool) -> Self {
        Self {
            path,
            width,
            rounded,
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn rounded(&self) -> bool {
        self.rounded
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    #[must_use]
    pub fn contour_length(&self) -> f64 {
        path_polyline_length(&self.path)
    }

    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.path.bounds().outset(Insets::all(self.width * 0.5))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SideSet {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl SideSet {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            top: true,
            right: true,
            bottom: true,
            left: true,
        }
    }

    #[must_use]
    pub const fn none() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: false,
        }
    }

    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            top: true,
            right: false,
            bottom: true,
            left: false,
        }
    }

    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            top: false,
            right: true,
            bottom: false,
            left: true,
        }
    }

    #[must_use]
    pub const fn top() -> Self {
        Self {
            top: true,
            ..Self::none()
        }
    }

    #[must_use]
    pub const fn right() -> Self {
        Self {
            right: true,
            ..Self::none()
        }
    }

    #[must_use]
    pub const fn bottom() -> Self {
        Self {
            bottom: true,
            ..Self::none()
        }
    }

    #[must_use]
    pub const fn left() -> Self {
        Self {
            left: true,
            ..Self::none()
        }
    }

    #[must_use]
    pub const fn includes(self, side: Side) -> bool {
        match side {
            Side::Top => self.top,
            Side::Right => self.right,
            Side::Bottom => self.bottom,
            Side::Left => self.left,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        !self.top && !self.right && !self.bottom && !self.left
    }
}

impl Default for SideSet {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}
pub(crate) fn dash_rect(rect: Rect, radii: Radii, stroke: Stroke) -> Result<DashGeometry> {
    let dash = stroke
        .dash()
        .ok_or_else(|| Error::new(ErrorCode::InvalidDash, "stroke has no dash geometry"))?;
    let mut geometry = DashGeometry::new();
    let metrics = DashMetrics::resolve(stroke, dash);
    let radii = radii.normalized_for(rect)?;
    let contour = RectContour::new(rect, radii, metrics);

    for run in side_runs(dash.sides()) {
        if run.closed() {
            add_centered_closed_dashes(
                &mut geometry,
                &contour.points,
                &contour.measure_points,
                &contour.corner_offsets(),
                metrics,
            );
        } else {
            let (points, measure_points, anchors) = contour.run(run);
            add_centered_open_dashes(&mut geometry, &points, &measure_points, &anchors, metrics);
        }
    }

    Ok(geometry)
}

pub(crate) fn dash_ellipse(center: Point, radii: Size, stroke: Stroke) -> Result<DashGeometry> {
    radii.validate("dash ellipse radii")?;
    let dash = stroke
        .dash()
        .ok_or_else(|| Error::new(ErrorCode::InvalidDash, "stroke has no dash geometry"))?;
    let metrics = DashMetrics::resolve(stroke, dash);
    let steps = ellipse_steps(radii);
    let points = ellipse_polyline_with_steps(center, radii, steps);
    let measure_points = ellipse_polyline(
        center,
        Size::new(
            (radii.width() - metrics.measure_inset).max(0.001),
            (radii.height() - metrics.measure_inset).max(0.001),
        ),
        steps,
    );
    let mut geometry = DashGeometry::new();
    add_closed_path_dashes(&mut geometry, &points, &measure_points, metrics);
    Ok(geometry)
}

#[derive(Clone, Copy)]
pub(crate) struct DashMetrics {
    width: f64,
    mark: f64,
    gap: f64,
    phase: f64,
    pub(crate) measure_inset: f64,
    rounded: bool,
    circular: bool,
}

impl DashMetrics {
    pub(crate) fn resolve(stroke: Stroke, dash: Dash) -> Self {
        let circular = dash.has_constraint(DashConstraint::Circular);
        let width = stroke.width();
        let mark = if circular {
            width
        } else {
            (width * 3.0 / dash.density()).max(width)
        };
        let gap = if circular {
            (width / dash.density()).max(width * 0.5)
        } else {
            (width * 1.5 / dash.density()).max(width * 0.5)
        };
        let measure_inset = match stroke.align() {
            StrokeAlign::Center => width * 0.5,
            StrokeAlign::Inside => width,
            StrokeAlign::Outside => 0.0,
        };
        Self {
            width,
            mark,
            gap,
            phase: dash.phase(),
            measure_inset,
            rounded: dash.is_rounded() || circular,
            circular,
        }
    }
}

fn side_corners(side: Side) -> (Corner, Corner) {
    match side {
        Side::Top => (Corner::TopLeft, Corner::TopRight),
        Side::Right => (Corner::TopRight, Corner::BottomRight),
        Side::Bottom => (Corner::BottomRight, Corner::BottomLeft),
        Side::Left => (Corner::BottomLeft, Corner::TopLeft),
    }
}

#[derive(Clone, Copy)]
struct SideRun {
    start: usize,
    count: usize,
}

impl SideRun {
    fn closed(self) -> bool {
        self.count == 4
    }

    fn side_at(self, offset: usize) -> Side {
        side_from_index((self.start + offset) % 4)
    }
}

pub(crate) struct RectContour {
    points: Vec<Point>,
    measure_points: Vec<Point>,
    side_start: [f64; 4],
    side_end: [f64; 4],
    pub(crate) total: f64,
}

impl RectContour {
    pub(crate) fn new(rect: Rect, radii: Radii, metrics: DashMetrics) -> Self {
        let mut points = Vec::new();
        let mut measure_points = Vec::new();
        let mut side_start = [0.0; 4];
        let mut side_end = [0.0; 4];
        let mut cursor = 0.0;
        let measure_rect = rect.inset(Insets::all(metrics.measure_inset));
        let measure_radii = concave_radii(radii, metrics.measure_inset);

        for side in [Side::Top, Side::Right, Side::Bottom, Side::Left] {
            let index = side_index(side);
            let side_points = rounded_side_path(rect, radii, side);
            let side_measure_points = rounded_side_path(measure_rect, measure_radii, side);
            side_start[index] = cursor;
            cursor += polyline_length(&side_measure_points);
            side_end[index] = cursor;

            if points.is_empty() {
                points.extend(side_points);
                measure_points.extend(side_measure_points);
            } else {
                for (point, measure_point) in
                    side_points.into_iter().zip(side_measure_points).skip(1)
                {
                    if side == Side::Left
                        && points
                            .first()
                            .is_some_and(|first| first.distance_to(point) <= 0.001)
                    {
                        continue;
                    }
                    push_unique(&mut points, point);
                    push_unique(&mut measure_points, measure_point);
                }
            }
        }

        Self {
            points,
            measure_points,
            side_start,
            side_end,
            total: cursor,
        }
    }

    pub(crate) fn corner_offsets(&self) -> [f64; 4] {
        [
            0.0,
            self.side_end[side_index(Side::Top)],
            self.side_end[side_index(Side::Right)],
            self.side_end[side_index(Side::Bottom)],
        ]
    }

    fn run(&self, run: SideRun) -> (Vec<Point>, Vec<Point>, Vec<f64>) {
        let start_side = run.side_at(0);
        let end_side = run.side_at(run.count - 1);
        let start = self.side_start[side_index(start_side)];
        let end = self.unwrapped_side_end(end_side, start);
        let (points, measure_points) = self.slice(start, end);
        let mut anchors = vec![0.0];

        for offset in 0..run.count {
            let side = run.side_at(offset);
            anchors.push(self.unwrapped_side_end(side, start) - start);
        }

        (points, measure_points, anchors)
    }

    fn unwrapped_side_end(&self, side: Side, start: f64) -> f64 {
        let mut end = self.side_end[side_index(side)];
        if end <= start + f64::EPSILON {
            end += self.total;
        }
        end
    }

    fn slice(&self, start: f64, end: f64) -> (Vec<Point>, Vec<Point>) {
        let (closed, closed_measure) = self.closed_points();
        if end <= self.total + f64::EPSILON {
            return mapped_polyline_slice(&closed, &closed_measure, start, end);
        }

        let (mut points, mut measure_points) =
            mapped_polyline_slice(&closed, &closed_measure, start, self.total);
        let (wrapped_points, wrapped_measure_points) =
            mapped_polyline_slice(&closed, &closed_measure, 0.0, end - self.total);
        extend_unique(&mut points, wrapped_points);
        extend_unique(&mut measure_points, wrapped_measure_points);
        (points, measure_points)
    }

    fn closed_points(&self) -> (Vec<Point>, Vec<Point>) {
        let mut points = self.points.clone();
        let mut measure_points = self.measure_points.clone();
        if let Some(first) = self.points.first() {
            points.push(*first);
        }
        if let Some(first) = self.measure_points.first() {
            measure_points.push(*first);
        }
        (points, measure_points)
    }
}

fn side_runs(sides: SideSet) -> Vec<SideRun> {
    let included = [sides.top, sides.right, sides.bottom, sides.left];
    if included.iter().all(|included| *included) {
        return vec![SideRun { start: 0, count: 4 }];
    }

    let mut runs = Vec::new();
    for index in 0..4 {
        if !included[index] || included[(index + 3) % 4] {
            continue;
        }

        let mut count = 0;
        while count < 4 && included[(index + count) % 4] {
            count += 1;
        }
        runs.push(SideRun {
            start: index,
            count,
        });
    }
    runs
}

fn side_index(side: Side) -> usize {
    match side {
        Side::Top => 0,
        Side::Right => 1,
        Side::Bottom => 2,
        Side::Left => 3,
    }
}

fn side_from_index(index: usize) -> Side {
    match index % 4 {
        0 => Side::Top,
        1 => Side::Right,
        2 => Side::Bottom,
        _ => Side::Left,
    }
}

fn corner_point(rect: Rect, corner: Corner) -> Point {
    let min = rect.min();
    let max = rect.max();
    match corner {
        Corner::TopLeft => min,
        Corner::TopRight => Point::new(max.x(), min.y()),
        Corner::BottomRight => max,
        Corner::BottomLeft => Point::new(min.x(), max.y()),
    }
}

pub(crate) fn corner_radius(radii: Radii, corner: Corner) -> f64 {
    match corner {
        Corner::TopLeft => radii.top_left(),
        Corner::TopRight => radii.top_right(),
        Corner::BottomRight => radii.bottom_right(),
        Corner::BottomLeft => radii.bottom_left(),
    }
}

fn concave_radii(radii: Radii, inset: f64) -> Radii {
    let radius = |value: f64| {
        if value <= f64::EPSILON {
            0.0
        } else {
            (value - inset).max(0.001)
        }
    };
    Radii::new(
        radius(radii.top_left()),
        radius(radii.top_right()),
        radius(radii.bottom_right()),
        radius(radii.bottom_left()),
    )
}

fn rounded_side_path(rect: Rect, radii: Radii, side: Side) -> Vec<Point> {
    let (start_corner, end_corner) = side_corners(side);
    let mut points = corner_arc_points(
        rect,
        radii,
        start_corner,
        corner_anchor_angle(start_corner),
        corner_side_angle(start_corner, side),
    );
    push_unique(
        &mut points,
        corner_side_point(rect, radii, end_corner, side),
    );
    extend_unique(
        &mut points,
        corner_arc_points(
            rect,
            radii,
            end_corner,
            corner_side_angle(end_corner, side),
            corner_anchor_angle(end_corner),
        ),
    );
    points
}

fn corner_arc_points(
    rect: Rect,
    radii: Radii,
    corner: Corner,
    start_angle: f64,
    end_angle: f64,
) -> Vec<Point> {
    let radius = corner_radius(radii, corner);
    if radius <= f64::EPSILON {
        return vec![corner_point(rect, corner)];
    }
    let center = corner_center(rect, radii, corner);
    arc_points(
        center,
        Size::new(radius, radius),
        start_angle,
        end_angle,
        CORNER_ARC_STEPS,
    )
}

pub(crate) fn corner_center(rect: Rect, radii: Radii, corner: Corner) -> Point {
    let min = rect.min();
    let max = rect.max();
    match corner {
        Corner::TopLeft => Point::new(min.x() + radii.top_left(), min.y() + radii.top_left()),
        Corner::TopRight => Point::new(max.x() - radii.top_right(), min.y() + radii.top_right()),
        Corner::BottomRight => Point::new(
            max.x() - radii.bottom_right(),
            max.y() - radii.bottom_right(),
        ),
        Corner::BottomLeft => {
            Point::new(min.x() + radii.bottom_left(), max.y() - radii.bottom_left())
        }
    }
}

pub(crate) fn corner_anchor_angle(corner: Corner) -> f64 {
    match corner {
        Corner::TopLeft => PI * 1.25,
        Corner::TopRight => PI * 1.75,
        Corner::BottomRight => PI * 0.25,
        Corner::BottomLeft => PI * 0.75,
    }
}

fn corner_side_angle(corner: Corner, side: Side) -> f64 {
    match (corner, side) {
        (Corner::TopLeft, Side::Top) | (Corner::TopRight, Side::Top) => PI * 1.5,
        (Corner::TopRight, Side::Right) => PI * 2.0,
        (Corner::BottomRight, Side::Right) => 0.0,
        (Corner::BottomRight, Side::Bottom) | (Corner::BottomLeft, Side::Bottom) => PI * 0.5,
        (Corner::BottomLeft, Side::Left) | (Corner::TopLeft, Side::Left) => PI,
        _ => corner_anchor_angle(corner),
    }
}

fn corner_side_point(rect: Rect, radii: Radii, corner: Corner, side: Side) -> Point {
    let radius = corner_radius(radii, corner);
    if radius <= f64::EPSILON {
        corner_point(rect, corner)
    } else {
        ellipse_point(
            corner_center(rect, radii, corner),
            Size::new(radius, radius),
            corner_side_angle(corner, side),
        )
    }
}

fn add_closed_path_dashes(
    geometry: &mut DashGeometry,
    points: &[Point],
    measure_points: &[Point],
    metrics: DashMetrics,
) {
    add_centered_closed_dashes(geometry, points, measure_points, &[], metrics);
}

fn add_centered_closed_dashes(
    geometry: &mut DashGeometry,
    points: &[Point],
    measure_points: &[Point],
    anchors: &[f64],
    metrics: DashMetrics,
) {
    if points.len() < 2 || measure_points.len() < 2 {
        return;
    }
    let mut closed = points.to_vec();
    closed.push(points[0]);
    let mut closed_measure = measure_points.to_vec();
    closed_measure.push(measure_points[0]);
    let total = polyline_length(&closed_measure);
    if total <= f64::EPSILON {
        return;
    }

    let centers = if anchors.is_empty() {
        centered_closed_points(total, metrics)
    } else {
        centered_points_from_anchors(total, anchors, true, metrics)
    };
    for center in centers {
        emit_centered_mark(geometry, &closed, &closed_measure, center, metrics, true);
    }
}

fn add_centered_open_dashes(
    geometry: &mut DashGeometry,
    points: &[Point],
    measure_points: &[Point],
    anchors: &[f64],
    metrics: DashMetrics,
) {
    let total = polyline_length(measure_points);
    if total <= f64::EPSILON {
        return;
    }

    for center in centered_points_from_anchors(total, anchors, false, metrics) {
        emit_centered_mark(geometry, points, measure_points, center, metrics, false);
    }
}

pub(crate) fn centered_closed_points(total: f64, metrics: DashMetrics) -> Vec<f64> {
    let spacing = metrics.mark + metrics.gap;
    let count = (total / spacing).round().max(1.0) as usize;
    (0..count)
        .map(|index| (total * index as f64 / count as f64 + metrics.phase).rem_euclid(total))
        .collect()
}

pub(crate) fn centered_points_from_anchors(
    total: f64,
    anchors: &[f64],
    closed: bool,
    metrics: DashMetrics,
) -> Vec<f64> {
    let mut anchors: Vec<f64> = anchors
        .iter()
        .map(|distance| distance.clamp(0.0, total))
        .collect();
    anchors.sort_by(f64::total_cmp);
    anchors.dedup_by(|a, b| (*a - *b).abs() <= 0.001);
    if anchors.is_empty() {
        return centered_closed_points(total, metrics);
    }

    let mut centers = anchors.clone();
    for pair in anchors.windows(2) {
        add_interior_centers(&mut centers, pair[0], pair[1], metrics);
    }

    if closed {
        add_interior_centers(
            &mut centers,
            *anchors.last().unwrap(),
            total + anchors[0],
            metrics,
        );
    }

    centers.sort_by(f64::total_cmp);
    centers.dedup_by(|a, b| (*a - *b).abs() <= 0.001);
    centers
        .into_iter()
        .map(|distance| {
            if closed {
                (distance + metrics.phase).rem_euclid(total)
            } else {
                (distance + metrics.phase).clamp(0.0, total)
            }
        })
        .collect()
}

fn add_interior_centers(centers: &mut Vec<f64>, start: f64, end: f64, metrics: DashMetrics) {
    let length = end - start;
    if length <= f64::EPSILON {
        return;
    }
    let gaps = center_gap_count(length, metrics);
    let spacing = length / gaps as f64;
    for index in 1..gaps {
        centers.push(start + spacing * index as f64);
    }
}

fn center_gap_count(length: f64, metrics: DashMetrics) -> usize {
    let desired = (metrics.mark + metrics.gap).max(0.001);
    let mut count = (length / desired).round().max(1.0) as usize;
    while count > 1 && length / count as f64 <= metrics.mark {
        count -= 1;
    }
    count
}

fn emit_centered_mark(
    geometry: &mut DashGeometry,
    points: &[Point],
    measure_points: &[Point],
    center: f64,
    metrics: DashMetrics,
    closed: bool,
) {
    if metrics.circular {
        emit_dot(geometry, points, measure_points, center, metrics);
        return;
    }

    let half = metrics.mark * 0.5;
    let segment = if closed {
        closed_mapped_polyline_slice(points, measure_points, center - half, center + half)
    } else {
        let total = polyline_length(measure_points);
        mapped_polyline_slice(
            points,
            measure_points,
            (center - half).max(0.0),
            (center + half).min(total),
        )
        .0
    };

    if segment.len() >= 2 {
        geometry.push(DashSegment::new_unchecked(
            path_from_points(&segment),
            metrics.width,
            metrics.rounded,
        ));
    }
}

fn emit_dot(
    geometry: &mut DashGeometry,
    points: &[Point],
    measure_points: &[Point],
    distance: f64,
    metrics: DashMetrics,
) {
    let point = mapped_polyline_point_at(points, measure_points, distance);
    let tangent = mapped_polyline_tangent_at(points, measure_points, distance);
    let epsilon = 0.001;
    let half = epsilon * 0.5;
    let start = point.translate(-tangent.x() * half, -tangent.y() * half);
    let end = point.translate(tangent.x() * half, tangent.y() * half);
    geometry.push(DashSegment::new_unchecked(
        path_from_points(&[start, end]),
        metrics.width,
        true,
    ));
}

fn arc_points(
    center: Point,
    radii: Size,
    start_angle: f64,
    end_angle: f64,
    steps: usize,
) -> Vec<Point> {
    let steps = steps.max(1);
    (0..=steps)
        .map(|index| {
            let t = index as f64 / steps as f64;
            ellipse_point(center, radii, start_angle + (end_angle - start_angle) * t)
        })
        .collect()
}

pub(crate) fn ellipse_point(center: Point, radii: Size, angle: f64) -> Point {
    Point::new(
        center.x() + radii.width() * angle.cos(),
        center.y() + radii.height() * angle.sin(),
    )
}

pub(crate) fn ellipse_steps(radii: Size) -> usize {
    let max_radius = radii.width().max(radii.height());
    ((max_radius * 2.0 * PI) / 2.0).ceil().clamp(32.0, 192.0) as usize
}

pub(crate) fn ellipse_polyline(center: Point, radii: Size, steps: usize) -> Vec<Point> {
    ellipse_polyline_with_steps(center, radii, steps)
}

fn ellipse_polyline_with_steps(center: Point, radii: Size, steps: usize) -> Vec<Point> {
    (0..steps)
        .map(|index| ellipse_point(center, radii, index as f64 / steps as f64 * 2.0 * PI))
        .collect()
}

fn push_unique(points: &mut Vec<Point>, point: Point) {
    if points
        .last()
        .is_none_or(|last| last.distance_to(point) > 0.001)
    {
        points.push(point);
    }
}

fn extend_unique(points: &mut Vec<Point>, next: Vec<Point>) {
    for point in next {
        push_unique(points, point);
    }
}

pub(crate) fn polyline_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| pair[0].distance_to(pair[1]))
        .sum()
}

fn mapped_polyline_slice(
    points: &[Point],
    measure_points: &[Point],
    start: f64,
    end: f64,
) -> (Vec<Point>, Vec<Point>) {
    let mut out = Vec::new();
    let mut measure_out = Vec::new();
    let mut cursor = 0.0;
    for (pair, measure_pair) in points.windows(2).zip(measure_points.windows(2)) {
        let a = pair[0];
        let b = pair[1];
        let measure_a = measure_pair[0];
        let measure_b = measure_pair[1];
        let length = measure_a.distance_to(measure_b);
        if length <= f64::EPSILON {
            continue;
        }
        let next = cursor + length;
        if next < start {
            cursor = next;
            continue;
        }
        if cursor > end {
            break;
        }

        let from = ((start - cursor) / length).clamp(0.0, 1.0);
        let to = ((end - cursor) / length).clamp(0.0, 1.0);
        if from <= to {
            let p0 = lerp_point(a, b, from);
            let p1 = lerp_point(a, b, to);
            let measure_p0 = lerp_point(measure_a, measure_b, from);
            let measure_p1 = lerp_point(measure_a, measure_b, to);
            push_unique(&mut out, p0);
            push_unique(&mut measure_out, measure_p0);
            if p0.distance_to(p1) > 0.001 || measure_p0.distance_to(measure_p1) > 0.001 {
                push_unique(&mut out, p1);
                push_unique(&mut measure_out, measure_p1);
            }
        }
        cursor = next;
    }
    (out, measure_out)
}

fn closed_mapped_polyline_slice(
    points: &[Point],
    measure_points: &[Point],
    start: f64,
    end: f64,
) -> Vec<Point> {
    let total = polyline_length(measure_points);
    if total <= f64::EPSILON {
        return Vec::new();
    }
    let mut start = start;
    let mut end = end;
    while start < 0.0 {
        start += total;
        end += total;
    }
    while start >= total {
        start -= total;
        end -= total;
    }
    if end <= total {
        return mapped_polyline_slice(points, measure_points, start, end).0;
    }

    let (mut out, _) = mapped_polyline_slice(points, measure_points, start, total);
    let (wrapped, _) = mapped_polyline_slice(points, measure_points, 0.0, end - total);
    extend_unique(&mut out, wrapped);
    out
}

fn mapped_polyline_point_at(points: &[Point], measure_points: &[Point], distance: f64) -> Point {
    if points.is_empty() {
        return Point::ZERO;
    }
    let mut cursor = 0.0;
    for (pair, measure_pair) in points.windows(2).zip(measure_points.windows(2)) {
        let length = measure_pair[0].distance_to(measure_pair[1]);
        if length <= f64::EPSILON {
            continue;
        }
        let next = cursor + length;
        if distance <= next {
            return lerp_point(
                pair[0],
                pair[1],
                ((distance - cursor) / length).clamp(0.0, 1.0),
            );
        }
        cursor = next;
    }
    *points.last().unwrap_or(&Point::ZERO)
}

fn mapped_polyline_tangent_at(points: &[Point], measure_points: &[Point], distance: f64) -> Point {
    let mut cursor = 0.0;
    for (pair, measure_pair) in points.windows(2).zip(measure_points.windows(2)) {
        let length = measure_pair[0].distance_to(measure_pair[1]);
        if length <= f64::EPSILON {
            continue;
        }
        let next = cursor + length;
        if distance <= next {
            let render_length = pair[0].distance_to(pair[1]);
            if render_length > f64::EPSILON {
                return Point::new(
                    (pair[1].x() - pair[0].x()) / render_length,
                    (pair[1].y() - pair[0].y()) / render_length,
                );
            }
        }
        cursor = next;
    }
    points
        .windows(2)
        .rev()
        .find_map(|pair| {
            let length = pair[0].distance_to(pair[1]);
            (length > f64::EPSILON).then(|| {
                Point::new(
                    (pair[1].x() - pair[0].x()) / length,
                    (pair[1].y() - pair[0].y()) / length,
                )
            })
        })
        .unwrap_or(Point::new(1.0, 0.0))
}

fn lerp_point(a: Point, b: Point, t: f64) -> Point {
    Point::new(a.x() + (b.x() - a.x()) * t, a.y() + (b.y() - a.y()) * t)
}
