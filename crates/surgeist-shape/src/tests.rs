use super::dash::{
    DashMetrics, RectContour, centered_closed_points, centered_points_from_anchors,
    corner_anchor_angle, corner_center, corner_radius, ellipse_point, ellipse_polyline,
    ellipse_steps, polyline_length,
};
use super::*;

fn mk_point(x: f64, y: f64) -> Point {
    Point::try_new(x, y).unwrap()
}

fn mk_size(width: f64, height: f64) -> Size {
    Size::try_new(width, height).unwrap()
}

fn mk_rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect::try_new(x, y, width, height).unwrap()
}

fn mk_insets(top: f64, right: f64, bottom: f64, left: f64) -> Insets {
    Insets::try_new(top, right, bottom, left).unwrap()
}

fn radii_all(radius: f64) -> Radii {
    Radii::try_all(radius).unwrap()
}

fn radii(top_left: f64, top_right: f64, bottom_right: f64, bottom_left: f64) -> Radii {
    Radii::try_new(top_left, top_right, bottom_right, bottom_left).unwrap()
}

fn stroke_with_dash(width: f64, dash: Dash) -> Stroke {
    Stroke::try_new(width).unwrap().with_dash(dash)
}

#[test]
fn non_negative_scalar_rejects_negative_values_with_semantic_code() {
    let error = NonNegative::try_new(-1.0, NumericKind::Size).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeSize);
}

#[test]
fn non_negative_scalar_rejects_negative_radius_with_radius_code() {
    let error = NonNegative::try_new(-1.0, NumericKind::Radius).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeRadius);
}

#[test]
fn finite_scalar_rejects_nan() {
    let error = Finite::try_new(f64::NAN, "point x").unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn size_constructor_rejects_negative_dimensions() {
    let error = Size::try_new(-1.0, 2.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeSize);
}

#[test]
fn radii_constructor_rejects_negative_radius() {
    let error = Radii::try_all(-1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeRadius);
}

#[test]
fn transform_constructor_rejects_non_finite_matrix() {
    let error = Transform::try_new([1.0, 0.0, 0.0, 1.0, f64::NAN, 0.0]).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn kurbo_point_conversion_rejects_non_finite_coordinates() {
    let error = Point::try_from(kurbo::Point::new(f64::NAN, 0.0)).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn transform_helpers_reject_non_finite_inputs() {
    let error = Transform::try_translate(f64::NAN, 0.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn path_builder_rejects_line_before_move_on_build() {
    let mut builder = PathBuilder::new();
    builder.line_to(Point::try_new(1.0, 1.0).unwrap());

    assert_eq!(builder.build().unwrap_err().code, ErrorCode::InvalidPath);
}

#[test]
fn validated_path_is_not_empty_after_build() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point::try_new(0.0, 0.0).unwrap())
        .line_to(Point::try_new(1.0, 0.0).unwrap());

    let path = builder.build().unwrap();

    assert!(!path.is_empty());
}

#[test]
fn empty_path_is_explicitly_valid_empty_geometry() {
    let path = PathBuilder::new().build().unwrap();

    assert!(path.is_empty());
    assert_eq!(path.bounds(), Rect::empty());
}

#[test]
fn shape_circle_rejects_negative_radius_at_construction() {
    let error = Shape::try_circle(Point::zero(), -1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::NegativeRadius);
}

#[test]
fn shape_ellipse_accepts_valid_radii_through_front_door() {
    let shape = Shape::try_ellipse(Point::zero(), Size::try_new(2.0, 3.0).unwrap()).unwrap();

    assert_eq!(shape.bounds(), Rect::try_new(-2.0, -3.0, 4.0, 6.0).unwrap());
}

#[test]
fn shape_path_accepts_only_validated_path() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point::zero())
        .line_to(Point::try_new(1.0, 0.0).unwrap())
        .line_to(Point::try_new(1.0, 1.0).unwrap())
        .close();
    let path = builder.build().unwrap();
    let shape = Shape::try_path(path, FillRule::NonZero).unwrap();

    assert!(!shape.bounds().is_empty());
}

#[test]
fn shape_path_bounds_preserve_degenerate_path_bounds() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(Point::zero())
        .line_to(Point::try_new(1.0, 0.0).unwrap());
    let path = builder.build().unwrap();
    let expected = path.bounds();
    let shape = Shape::try_path(path, FillRule::NonZero).unwrap();

    assert_eq!(shape.bounds(), expected);
}

#[test]
fn stroke_constructor_rejects_negative_width() {
    let error = Stroke::try_new(-1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidStroke);
}

#[test]
fn stroke_style_accessors_return_configured_values() {
    let stroke = Stroke::try_new(2.0)
        .unwrap()
        .with_join(LineJoin::Bevel)
        .with_caps(LineCap::Round, LineCap::Square)
        .try_with_miter_limit(12.0)
        .unwrap();

    assert_eq!(stroke.join(), LineJoin::Bevel);
    assert_eq!(stroke.start_cap(), LineCap::Round);
    assert_eq!(stroke.end_cap(), LineCap::Square);
    assert_eq!(stroke.miter_limit(), 12.0);
}

#[test]
fn dash_constructor_rejects_zero_density() {
    let error = Dash::try_new(0.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidDash);
}

#[test]
fn dash_segment_constructor_rejects_negative_width() {
    let path = {
        let mut builder = PathBuilder::new();
        builder
            .move_to(Point::zero())
            .line_to(Point::try_new(1.0, 0.0).unwrap());
        builder.build().unwrap()
    };
    let error = DashSegment::try_new(path, -1.0, false).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidStroke);
}

#[test]
fn dash_anchor_rejects_negative_contour_offset() {
    let error = DashAnchor::try_contour_offset(-1.0).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidDash);
}

#[test]
fn dash_rejects_empty_side_set_but_side_set_none_is_valid() {
    let sides = SideSet::none();

    assert!(sides.is_empty());
    assert!(!sides.includes(Side::Top));

    let error = Dash::dashed().with_sides(sides).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidDash);
}

#[test]
fn dash_with_anchor_rejects_fifth_anchor() {
    let dash = Dash::dashed()
        .with_anchor(DashAnchor::corner(Corner::TopLeft))
        .unwrap()
        .with_anchor(DashAnchor::corner(Corner::TopRight))
        .unwrap()
        .with_anchor(DashAnchor::corner(Corner::BottomRight))
        .unwrap()
        .with_anchor(DashAnchor::corner(Corner::BottomLeft))
        .unwrap();

    let error = dash
        .with_anchor(DashAnchor::corner(Corner::TopLeft))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidDash);
}

#[test]
fn public_front_doors_construct_valid_shape_and_dash_geometry() {
    let rect = Rect::try_new(0.0, 0.0, 80.0, 40.0).unwrap();
    let shape = Shape::try_rounded_rect(rect, Radii::try_all(8.0).unwrap()).unwrap();
    let dash = Dash::dashed().with_corner_anchors();
    let stroke = Stroke::try_centered(2.0).unwrap().with_dash(dash);
    let geometry = shape.dashed_stroke(stroke).unwrap();

    assert!(!geometry.is_empty());
}

#[test]
fn radii_normalization_reduces_asymmetric_corners() {
    let rect = mk_rect(0.0, 0.0, 10.0, 8.0);
    let radii = radii(8.0, 8.0, 4.0, 4.0).normalized_for(rect).unwrap();

    assert_eq!(radii.top_left(), 5.0);
    assert_eq!(radii.top_right(), 5.0);
    assert_eq!(radii.bottom_right(), 2.5);
    assert_eq!(radii.bottom_left(), 2.5);
}

#[test]
fn path_rejects_line_before_move() {
    let error = Path::from_commands(vec![Command::LineTo(mk_point(1.0, 1.0))]).unwrap_err();

    assert_eq!(error.code, ErrorCode::InvalidPath);
}

#[test]
fn path_fill_rule_affects_key() {
    let mut builder = PathBuilder::new();
    builder
        .move_to(mk_point(0.0, 0.0))
        .line_to(mk_point(10.0, 0.0))
        .line_to(mk_point(10.0, 10.0))
        .close();
    let path = builder.build().unwrap();

    assert_ne!(path.key(FillRule::NonZero), path.key(FillRule::EvenOdd));
    assert_ne!(
        Shape::try_path(path.clone(), FillRule::NonZero)
            .unwrap()
            .key(),
        Shape::try_path(path, FillRule::EvenOdd).unwrap().key()
    );
}

#[test]
fn shape_keys_are_stable_and_distinguish_normalized_geometry() {
    let rect = Shape::try_rounded_rect(mk_rect(0.0, 0.0, 10.0, 8.0), radii_all(12.0)).unwrap();
    let same = Shape::try_rounded_rect(mk_rect(0.0, 0.0, 10.0, 8.0), radii_all(20.0)).unwrap();
    let different = Shape::try_rounded_rect(mk_rect(0.0, 0.0, 12.0, 8.0), radii_all(20.0)).unwrap();

    assert_eq!(rect.key(), rect.key());
    assert_eq!(rect.key(), same.key());
    assert_ne!(rect.key(), different.key());
}

#[test]
fn visual_bounds_respect_stroke_alignment() {
    let shape = Shape::rect(mk_rect(10.0, 20.0, 30.0, 40.0));

    assert_eq!(
        shape
            .visual_bounds(Some(Stroke::try_inside(8.0).unwrap()))
            .unwrap(),
        mk_rect(10.0, 20.0, 30.0, 40.0)
    );
    assert_eq!(
        shape
            .visual_bounds(Some(Stroke::try_centered(8.0).unwrap()))
            .unwrap(),
        mk_rect(6.0, 16.0, 38.0, 48.0)
    );
    assert_eq!(
        shape
            .visual_bounds(Some(Stroke::try_outside(8.0).unwrap()))
            .unwrap(),
        mk_rect(2.0, 12.0, 46.0, 56.0)
    );
}

#[test]
fn dash_rejects_non_finite_phase() {
    let error = Dash::dashed().try_with_phase(f64::NAN).unwrap_err();

    assert_eq!(error.code, ErrorCode::NonFinite);
}

#[test]
fn support_bounds_are_explicit_outsets() {
    let shape = Shape::rect(mk_rect(10.0, 20.0, 30.0, 40.0));
    let bounds = shape.support_bounds(mk_insets(1.0, 2.0, 3.0, 4.0)).unwrap();

    assert_eq!(bounds, mk_rect(6.0, 19.0, 36.0, 44.0));
}

#[test]
fn converts_rect_to_path() {
    let path = Shape::rect(mk_rect(0.0, 0.0, 10.0, 20.0))
        .to_path()
        .unwrap();

    assert!(!path.is_empty());
}

#[test]
fn side_scoped_dashes_emit_only_included_sides() {
    let dash = Dash::dashed()
        .with_sides(SideSet::top())
        .unwrap()
        .with_corner_anchors();
    let stroke = stroke_with_dash(2.0, dash);
    let geometry = Shape::rect(mk_rect(0.0, 0.0, 80.0, 40.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert!(!geometry.is_empty());
    assert!(geometry.segments().iter().all(|segment| {
        segment
            .path()
            .commands()
            .iter()
            .all(|command| match *command {
                Command::MoveTo(point) | Command::LineTo(point) => point.y() <= 2.0,
                _ => true,
            })
    }));
}

#[test]
fn dash_phase_offsets_resolved_geometry() {
    let base = stroke_with_dash(4.0, Dash::dashed());
    let phased = stroke_with_dash(4.0, Dash::dashed().try_with_phase(10.0).unwrap());
    let shape = Shape::rect(mk_rect(0.0, 0.0, 80.0, 40.0));

    assert_ne!(
        shape.dashed_stroke(base).unwrap(),
        shape.dashed_stroke(phased).unwrap()
    );
}

#[test]
fn corner_dash_directions_stay_inside_rect_bounds() {
    let dash = Dash::dashed().with_corner_anchors();
    let stroke = stroke_with_dash(4.0, dash);
    let rect = mk_rect(0.0, 0.0, 80.0, 40.0);
    let geometry = Shape::rect(rect).dashed_stroke(stroke).unwrap();

    assert!(geometry.segments().iter().all(|segment| {
        segment
            .path()
            .commands()
            .iter()
            .all(|command| match *command {
                Command::MoveTo(point) | Command::LineTo(point) => rect.contains(point),
                Command::QuadTo { control, end } => rect.contains(control) && rect.contains(end),
                Command::CubicTo {
                    control_a,
                    control_b,
                    end,
                } => rect.contains(control_a) && rect.contains(control_b) && rect.contains(end),
                Command::Close => true,
            })
    }));
}

#[test]
fn circular_one_sided_corner_dots_are_omitted() {
    let dash = Dash::dotted()
        .with_sides(SideSet::top())
        .unwrap()
        .with_corner_anchors();
    let stroke = stroke_with_dash(4.0, dash);
    let geometry = Shape::rect(mk_rect(0.0, 0.0, 80.0, 40.0))
        .dashed_stroke(stroke)
        .unwrap();

    assert!(
        geometry
            .segments()
            .iter()
            .all(|segment| segment.contour_length() > 0.0)
    );
}

#[test]
fn ellipse_dashes_have_stable_output() {
    let dash = Dash::dashed().try_with_density(1.5).unwrap();
    let stroke = stroke_with_dash(2.0, dash);
    let geometry = Shape::try_ellipse(mk_point(20.0, 20.0), mk_size(15.0, 10.0))
        .unwrap()
        .dashed_stroke(stroke)
        .unwrap();

    assert!(!geometry.is_empty());
    assert_eq!(
        geometry,
        Shape::try_ellipse(mk_point(20.0, 20.0), mk_size(15.0, 10.0))
            .unwrap()
            .dashed_stroke(stroke)
            .unwrap()
    );
}

#[test]
fn ellipse_dashes_use_consistent_concave_arc_lengths() {
    let dash = Dash::dashed().try_with_density(1.2).unwrap().rounded();
    let stroke = stroke_with_dash(4.0, dash);
    let metrics = DashMetrics::resolve(stroke, dash);
    let radii = mk_size(40.0, 18.0);
    let steps = ellipse_steps(radii);
    let measure = ellipse_polyline(
        mk_point(20.0, 20.0),
        mk_size(
            (radii.width() - metrics.measure_inset).max(0.001),
            (radii.height() - metrics.measure_inset).max(0.001),
        ),
        steps,
    );
    let measure_total = polyline_length(&closed_measure_points(&measure));
    let centers = centered_closed_points(measure_total, metrics);
    let geometry = Shape::try_ellipse(mk_point(20.0, 20.0), mk_size(40.0, 18.0))
        .unwrap()
        .dashed_stroke(stroke)
        .unwrap();

    assert_eq!(geometry.segments().len(), centers.len());
    assert!(centers.windows(2).all(|pair| pair[1] > pair[0]));
}

#[test]
fn rounded_rect_dashes_include_corner_anchors_without_overlap() {
    let dash = Dash::dashed().try_with_density(1.1).unwrap();
    let stroke = stroke_with_dash(6.0, dash);
    let rect = mk_rect(0.0, 0.0, 120.0, 80.0);
    let geometry = Shape::try_rounded_rect(rect, radii_all(20.0))
        .unwrap()
        .dashed_stroke(stroke)
        .unwrap();

    assert!(geometry.segments().len() > 8);
    assert!(
        geometry
            .segments()
            .iter()
            .all(|segment| match segment.path().commands()[0] {
                Command::MoveTo(point) => rect.contains(point),
                _ => false,
            })
    );
}

#[test]
fn rounded_rect_corner_points_are_distribution_anchors() {
    let dash = Dash::dashed().try_with_density(1.0).unwrap();
    let stroke = stroke_with_dash(6.0, dash);
    let metrics = DashMetrics::resolve(stroke, dash);
    let contour = RectContour::new(mk_rect(0.0, 0.0, 120.0, 80.0), radii_all(24.0), metrics);
    let anchors = contour.corner_offsets();
    let centers = centered_points_from_anchors(contour.total, &anchors, true, metrics);

    for anchor in anchors {
        assert!(
            centers
                .iter()
                .any(|center| (center - anchor).abs() <= 0.001)
        );
    }
}

#[test]
fn rounded_rect_dashes_contain_owned_corner_points() {
    let dash = Dash::dashed().try_with_density(1.0).unwrap();
    let stroke = stroke_with_dash(6.0, dash);
    let rect = mk_rect(0.0, 0.0, 120.0, 80.0);
    let radii = radii_all(24.0);
    let geometry = Shape::try_rounded_rect(rect, radii)
        .unwrap()
        .dashed_stroke(stroke)
        .unwrap();

    for corner in [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomRight,
        Corner::BottomLeft,
    ] {
        let expected = ellipse_point(
            corner_center(rect, radii, corner),
            mk_size(corner_radius(radii, corner), corner_radius(radii, corner)),
            corner_anchor_angle(corner),
        );
        assert!(
            geometry.segments().iter().any(|segment| path_visits_point(
                segment.path(),
                expected,
                0.75
            )),
            "missing dash at {corner:?}"
        );
    }
}

fn closed_measure_points(points: &[Point]) -> Vec<Point> {
    let mut closed = points.to_vec();
    if let Some(first) = points.first() {
        closed.push(*first);
    }
    closed
}

fn path_visits_point(path: &Path, expected: Point, tolerance: f64) -> bool {
    let mut current = None;
    for command in path.commands() {
        match *command {
            Command::MoveTo(point) => current = Some(point),
            Command::LineTo(point) => {
                if let Some(start) = current
                    && point_to_segment_distance(expected, start, point) <= tolerance
                {
                    return true;
                }
                current = Some(point);
            }
            _ => {}
        }
    }
    false
}

fn point_to_segment_distance(point: Point, start: Point, end: Point) -> f64 {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length_sq = dx * dx + dy * dy;
    if length_sq <= f64::EPSILON {
        return point.distance_to(start);
    }
    let t =
        (((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / length_sq).clamp(0.0, 1.0);
    point.distance_to(mk_point(start.x() + dx * t, start.y() + dy * t))
}
