#![forbid(unsafe_code)]

use std::{cell::Cell, mem::size_of, time::Duration};
use surgeist_layout::{PhysicalAxis, Point, Size};
use surgeist_paint::{
    EventFilter, GradientPosition, GradientPositionComponent, GradientStopItem,
    GradientStopPositions, PaintChange, PaintError, PaintEvent, PaintLimits, PaintOptions,
    PaintPhase, PaintReport, ProfileMode, RadialGradientCase as Case, RadialGradientError,
    RadialGradientExtent as Extent, RadialGradientGeometry, RadialGradientSize as EndingSize,
    ResourceKind, UsedGradientItem, WorkKind, resolve_gradient_stops, resolve_radial_gradient,
};

fn options(limits: PaintLimits) -> PaintOptions {
    PaintOptions::new(limits, ProfileMode::Summary { timing: false })
}

fn position(length: f64, fraction: f64) -> GradientPosition {
    GradientPosition { length, fraction }
}

fn explicit(x: f64, y: f64) -> EndingSize {
    EndingSize::EllipseRadii(Size::new(position(x, 0.0), position(y, 0.0)))
}

fn geometry(
    box_size: Size<f64>,
    center: Option<Point<f64>>,
    ending_size: EndingSize,
) -> RadialGradientGeometry {
    resolve_radial_gradient(
        box_size,
        center,
        ending_size,
        options(PaintLimits::unlimited()),
    )
    .result
    .unwrap()
}

fn close(actual: f64, expected: f64) {
    let tolerance = 32.0 * f64::EPSILON * expected.abs().max(1.0);
    assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "actual {actual:?}, expected {expected:?}, tolerance {tolerance:?}"
    );
}

fn ellipse(result: RadialGradientGeometry, expected_center: Point<f64>, x: f64, y: f64) {
    assert_eq!(result.center(), expected_center);
    let Case::Ellipse { radii } = result.case() else {
        panic!("expected a positive ellipse")
    };
    assert!(radii.width > 0.0 && radii.height > 0.0);
    close(radii.width, x);
    close(radii.height, y);
}

const EXTENTS: [Extent; 4] = [
    Extent::ClosestSide,
    Extent::FarthestSide,
    Extent::ClosestCorner,
    Extent::FarthestCorner,
];

#[test]
fn default_box_center_and_every_extent_match_centered_rectangular_anchors() {
    let box_size = Size::new(200.0, 100.0);
    let center = Point::new(100.0, 50.0);
    let root_two = 2.0_f64.sqrt();
    ellipse(
        geometry(box_size, None, EndingSize::default()),
        center,
        100.0 * root_two,
        50.0 * root_two,
    );
    assert_eq!(
        geometry(box_size, None, EndingSize::default()),
        geometry(
            box_size,
            Some(center),
            EndingSize::EllipseExtent(Extent::FarthestCorner)
        ),
    );
    for (extent, x, y, circle_radius) in [
        (Extent::ClosestSide, 100.0, 50.0, 50.0),
        (Extent::FarthestSide, 100.0, 50.0, 100.0),
        (
            Extent::ClosestCorner,
            100.0 * root_two,
            50.0 * root_two,
            50.0 * 5.0_f64.sqrt(),
        ),
        (
            Extent::FarthestCorner,
            100.0 * root_two,
            50.0 * root_two,
            50.0 * 5.0_f64.sqrt(),
        ),
    ] {
        ellipse(
            geometry(box_size, None, EndingSize::EllipseExtent(extent)),
            center,
            x,
            y,
        );
        ellipse(
            geometry(box_size, None, EndingSize::CircleExtent(extent)),
            center,
            circle_radius,
            circle_radius,
        );
    }
}

#[test]
fn off_center_sides_and_corners_use_the_selected_pairs_and_required_aspect_ratio() {
    let box_size = Size::new(200.0, 100.0);
    let center = Point::new(30.0, 40.0);
    let root_two = 2.0_f64.sqrt();
    // Near pair (30,40), far pair (170,60). Corner ellipses pass through
    // that selected corner with the corresponding side-pair aspect ratio.
    for (extent, x, y, circle_radius) in [
        (Extent::ClosestSide, 30.0, 40.0, 30.0),
        (Extent::FarthestSide, 170.0, 60.0, 170.0),
        (
            Extent::ClosestCorner,
            30.0 * root_two,
            40.0 * root_two,
            50.0,
        ),
        (
            Extent::FarthestCorner,
            170.0 * root_two,
            60.0 * root_two,
            50.0 * 13.0_f64.sqrt(),
        ),
    ] {
        ellipse(
            geometry(box_size, Some(center), EndingSize::EllipseExtent(extent)),
            center,
            x,
            y,
        );
        ellipse(
            geometry(box_size, Some(center), EndingSize::CircleExtent(extent)),
            center,
            circle_radius,
            circle_radius,
        );
    }
}

#[test]
fn outside_centers_measure_infinite_supporting_lines_and_separable_corners() {
    let box_size = Size::new(200.0, 100.0);
    let center = Point::new(-30.0, 140.0);
    let root_two = 2.0_f64.sqrt();
    // Finite-edge distance would give 50 for the nearest side, rather than 30.
    for (extent, x, y, circle_radius) in [
        (Extent::ClosestSide, 30.0, 40.0, 30.0),
        (Extent::FarthestSide, 230.0, 140.0, 230.0),
        (
            Extent::ClosestCorner,
            30.0 * root_two,
            40.0 * root_two,
            50.0,
        ),
        (
            Extent::FarthestCorner,
            230.0 * root_two,
            140.0 * root_two,
            50.0 * 29.0_f64.sqrt(),
        ),
    ] {
        ellipse(
            geometry(box_size, Some(center), EndingSize::EllipseExtent(extent)),
            center,
            x,
            y,
        );
        ellipse(
            geometry(box_size, Some(center), EndingSize::CircleExtent(extent)),
            center,
            circle_radius,
            circle_radius,
        );
    }
}

#[test]
fn reflections_transposition_and_side_ties_preserve_physical_geometry() {
    let box_size = Size::new(200.0, 100.0);
    for extent in EXTENTS {
        for request in [
            EndingSize::EllipseExtent(extent),
            EndingSize::CircleExtent(extent),
        ] {
            let original = geometry(box_size, Some(Point::new(30.0, 40.0)), request);
            for center in [
                Point::new(170.0, 40.0),
                Point::new(30.0, 60.0),
                Point::new(170.0, 60.0),
            ] {
                let reflected = geometry(box_size, Some(center), request);
                assert_eq!(reflected.center(), center);
                assert_eq!(reflected.case(), original.case());
            }
            let outside = geometry(box_size, Some(Point::new(-30.0, 140.0)), request);
            assert_eq!(
                outside.case(),
                geometry(box_size, Some(Point::new(230.0, -40.0)), request).case()
            );
            let Case::Ellipse { radii } = original.case() else {
                panic!("positive geometry required")
            };
            let transposed = geometry(
                Size::new(box_size.height, box_size.width),
                Some(Point::new(40.0, 30.0)),
                request,
            );
            ellipse(
                transposed,
                Point::new(40.0, 30.0),
                radii.height,
                radii.width,
            );
        }
    }
    let tied = Point::new(50.0, 50.0);
    for extent in EXTENTS {
        let radius = if matches!(extent, Extent::ClosestSide | Extent::FarthestSide) {
            50.0
        } else {
            50.0 * 2.0_f64.sqrt()
        };
        ellipse(
            geometry(
                Size::new(100.0, 100.0),
                None,
                EndingSize::CircleExtent(extent),
            ),
            tied,
            radius,
            radius,
        );
    }
}

#[test]
fn explicit_radii_use_full_matching_axis_dimensions_and_fused_signed_contributions() {
    let box_size = Size::new(200.0, 100.0);
    let center = Point::new(100.0, 50.0);
    ellipse(
        geometry(
            box_size,
            None,
            EndingSize::EllipseRadii(Size::new(position(0.0, 0.25), position(0.0, 0.5))),
        ),
        center,
        50.0,
        50.0,
    );
    ellipse(
        geometry(
            box_size,
            None,
            EndingSize::EllipseRadii(Size::new(position(10.0, 0.25), position(-10.0, 0.5))),
        ),
        center,
        60.0,
        40.0,
    );
    ellipse(
        geometry(
            box_size,
            Some(Point::new(-30.0, 140.0)),
            EndingSize::CircleRadius(25.0),
        ),
        Point::new(-30.0, 140.0),
        25.0,
        25.0,
    );
    let huge = Size::new(f64::MAX, 100.0);
    let used_center = Point::new(0.0, 0.0);
    ellipse(
        geometry(
            huge,
            Some(used_center),
            EndingSize::EllipseRadii(Size::new(position(-f64::MAX, 2.0), position(1.0, 0.0))),
        ),
        used_center,
        f64::MAX,
        1.0,
    );
}

#[test]
fn explicit_zero_cases_preserve_the_ordered_circle_width_height_split() {
    let box_size = Size::new(200.0, 100.0);
    for (request, expected) in [
        (EndingSize::CircleRadius(0.0), Case::ZeroCircle),
        (EndingSize::CircleRadius(-0.0), Case::ZeroCircle),
        (explicit(0.0, 20.0), Case::ZeroWidth),
        (explicit(0.0, 0.0), Case::ZeroWidth),
        (explicit(-0.0, -0.0), Case::ZeroWidth),
        (explicit(20.0, 0.0), Case::ZeroHeight),
    ] {
        let result = geometry(box_size, None, request);
        assert_eq!(result.center(), Point::new(100.0, 50.0));
        assert_eq!(result.case(), expected);
    }
}

#[test]
fn edge_corner_and_zero_box_cases_do_not_invent_an_ordinary_zero_aspect_ratio() {
    let box_size = Size::new(200.0, 100.0);
    for extent in [Extent::ClosestSide, Extent::ClosestCorner] {
        for (center, expected) in [
            (Point::new(0.0, 40.0), Case::ZeroWidth),
            (Point::new(200.0, 40.0), Case::ZeroWidth),
            (Point::new(30.0, 0.0), Case::ZeroHeight),
            (Point::new(30.0, 100.0), Case::ZeroHeight),
            (Point::new(0.0, 0.0), Case::ZeroWidth),
        ] {
            assert_eq!(
                geometry(box_size, Some(center), EndingSize::EllipseExtent(extent)).case(),
                expected
            );
        }
    }
    assert_eq!(
        geometry(
            box_size,
            Some(Point::new(0.0, 40.0)),
            EndingSize::CircleExtent(Extent::ClosestSide)
        )
        .case(),
        Case::ZeroCircle
    );
    ellipse(
        geometry(
            box_size,
            Some(Point::new(0.0, 40.0)),
            EndingSize::CircleExtent(Extent::ClosestCorner),
        ),
        Point::new(0.0, 40.0),
        40.0,
        40.0,
    );
    for extent in EXTENTS {
        assert_eq!(
            geometry(Size::new(0.0, 0.0), None, EndingSize::CircleExtent(extent)).case(),
            Case::ZeroCircle
        );
        assert_eq!(
            geometry(Size::new(0.0, 0.0), None, EndingSize::EllipseExtent(extent)).case(),
            Case::ZeroWidth
        );
        assert_eq!(
            geometry(
                Size::new(0.0, 100.0),
                None,
                EndingSize::EllipseExtent(extent)
            )
            .case(),
            Case::ZeroWidth
        );
        assert_eq!(
            geometry(
                Size::new(200.0, 0.0),
                None,
                EndingSize::EllipseExtent(extent)
            )
            .case(),
            Case::ZeroHeight
        );
        for (box_size, center, radius) in [
            (Size::new(0.0, 100.0), Point::new(0.0, 50.0), 50.0),
            (Size::new(200.0, 0.0), Point::new(100.0, 0.0), 100.0),
        ] {
            let result = geometry(box_size, None, EndingSize::CircleExtent(extent));
            if extent == Extent::ClosestSide {
                assert_eq!(result.case(), Case::ZeroCircle);
            } else {
                ellipse(result, center, radius, radius);
            }
        }
        let corner = matches!(extent, Extent::ClosestCorner | Extent::FarthestCorner);
        let (x, y) = if corner {
            (30.0 * 2.0_f64.sqrt(), 40.0 * 2.0_f64.sqrt())
        } else {
            (30.0, 40.0)
        };
        ellipse(
            geometry(
                Size::new(0.0, 0.0),
                Some(Point::new(30.0, 40.0)),
                EndingSize::EllipseExtent(extent),
            ),
            Point::new(30.0, 40.0),
            x,
            y,
        );
    }
}

#[test]
fn proved_zero_axes_precede_unused_overflowing_extent_distances_and_scales() {
    for (box_size, center, extent, expected) in [
        (
            Size::new(200.0, f64::MAX),
            Point::new(0.0, -f64::MAX),
            Extent::ClosestCorner,
            Case::ZeroWidth,
        ),
        (
            Size::new(0.0, f64::MAX),
            Point::new(0.0, -f64::MAX),
            Extent::FarthestCorner,
            Case::ZeroWidth,
        ),
        (
            Size::new(f64::MAX, 0.0),
            Point::new(-f64::MAX, 0.0),
            Extent::FarthestCorner,
            Case::ZeroHeight,
        ),
    ] {
        assert_eq!(
            geometry(box_size, Some(center), EndingSize::EllipseExtent(extent)).case(),
            expected
        );
    }
}

#[test]
fn selected_distances_hypot_and_finite_fields_avoid_unused_overflow_restrictions() {
    let box_size = Size::new(f64::MAX, 100.0);
    let center = Point::new(-f64::MAX, 50.0);
    ellipse(
        geometry(
            box_size,
            Some(center),
            EndingSize::EllipseExtent(Extent::ClosestSide),
        ),
        center,
        f64::MAX,
        50.0,
    );
    ellipse(
        geometry(
            box_size,
            Some(center),
            EndingSize::CircleExtent(Extent::ClosestSide),
        ),
        center,
        50.0,
        50.0,
    );
    let huge_center = Point::new(f64::MAX, f64::MAX);
    ellipse(
        geometry(
            Size::new(200.0, 100.0),
            Some(huge_center),
            EndingSize::CircleRadius(f64::MAX),
        ),
        huge_center,
        f64::MAX,
        f64::MAX,
    );
    // The required hypot is finite although squaring either component overflows.
    let far_center = Point::new(3.0e200, 4.0e200);
    ellipse(
        geometry(
            Size::new(0.0, 0.0),
            Some(far_center),
            EndingSize::CircleExtent(Extent::ClosestCorner),
        ),
        far_center,
        5.0e200,
        5.0e200,
    );
    let both = Point::new(-f64::MAX, -f64::MAX);
    ellipse(
        geometry(
            Size::new(f64::MAX, f64::MAX),
            Some(both),
            EndingSize::CircleExtent(Extent::ClosestSide),
        ),
        both,
        f64::MAX,
        f64::MAX,
    );
}

#[test]
fn true_affine_cancellation_and_smallest_positive_radii_remain_distinct() {
    let tiny = f64::from_bits(1);
    let center = Point::new(0.0, 0.0);
    for (box_size, request, expected) in [
        (
            Size::new(2.0, 100.0),
            EndingSize::EllipseRadii(Size::new(position(-1.0, 0.5), position(1.0, 0.0))),
            Case::ZeroWidth,
        ),
        (
            Size::new(3.0, 100.0),
            EndingSize::EllipseRadii(Size::new(position(-9.0, 3.0), position(1.0, 0.0))),
            Case::ZeroWidth,
        ),
        (
            Size::new(tiny, 100.0),
            EndingSize::EllipseRadii(Size::new(position(-tiny, 1.0), position(1.0, 0.0))),
            Case::ZeroWidth,
        ),
        (
            Size::new(tiny, 100.0),
            EndingSize::EllipseRadii(Size::new(position(0.0, 0.0), position(1.0, 0.0))),
            Case::ZeroWidth,
        ),
        (
            Size::new(100.0, 2.0),
            EndingSize::EllipseRadii(Size::new(position(1.0, 0.0), position(-1.0, 0.5))),
            Case::ZeroHeight,
        ),
    ] {
        assert_eq!(geometry(box_size, Some(center), request).case(), expected);
    }
    let circle = geometry(
        Size::new(200.0, 100.0),
        None,
        EndingSize::CircleRadius(tiny),
    );
    assert_eq!(
        circle.case(),
        Case::Ellipse {
            radii: Size::new(tiny, tiny)
        }
    );
    let ellipse = geometry(Size::new(tiny, tiny), Some(center), explicit(tiny, tiny));
    assert_eq!(
        ellipse.case(),
        Case::Ellipse {
            radii: Size::new(tiny, tiny)
        }
    );
    let small_corner = geometry(
        Size::new(0.0, 0.0),
        Some(Point::new(tiny, 0.0)),
        EndingSize::CircleExtent(Extent::ClosestCorner),
    );
    assert_eq!(
        small_corner.case(),
        Case::Ellipse {
            radii: Size::new(tiny, tiny)
        }
    );
}

fn no_custody(report: &PaintReport) {
    for &resource in ResourceKind::ALL {
        assert_eq!(report.live(resource), 0, "{resource:?}");
    }
}

fn error(
    box_size: Size<f64>,
    center: Option<Point<f64>>,
    request: EndingSize,
    expected: RadialGradientError,
) {
    let outcome =
        resolve_radial_gradient(box_size, center, request, options(PaintLimits::unlimited()));
    assert_eq!(outcome.result, Err(PaintError::RadialGradient(expected)));
    no_custody(&outcome.report.unwrap());
}

#[test]
fn needed_overflow_and_nonzero_affine_underflow_refuse_without_fabricating_zero_cases() {
    let box_size = Size::new(f64::MAX, 100.0);
    let center = Point::new(-f64::MAX, 50.0);
    for request in [
        EndingSize::CircleExtent(Extent::FarthestSide),
        EndingSize::EllipseExtent(Extent::FarthestSide),
        EndingSize::EllipseExtent(Extent::ClosestCorner),
    ] {
        error(
            box_size,
            Some(center),
            request,
            RadialGradientError::UnrepresentableGeometry,
        );
    }
    error(
        Size::new(f64::MAX, f64::MAX),
        Some(Point::new(-f64::MAX, -f64::MAX)),
        EndingSize::CircleExtent(Extent::ClosestCorner),
        RadialGradientError::UnrepresentableGeometry,
    );
    error(
        box_size,
        Some(Point::new(0.0, 0.0)),
        EndingSize::EllipseRadii(Size::new(position(f64::MAX, 2.0), position(1.0, 0.0))),
        RadialGradientError::UnrepresentableGeometry,
    );
    let tiny = f64::from_bits(1);
    for x in [
        position(0.0, 0.5),
        position(0.0, -0.5),
        position(-tiny, 1.5),
        position(-tiny, 0.5),
    ] {
        error(
            Size::new(tiny, 100.0),
            Some(Point::new(0.0, 0.0)),
            EndingSize::EllipseRadii(Size::new(x, position(1.0, 0.0))),
            RadialGradientError::UnrepresentableGeometry,
        );
    }
    for y in [position(0.0, 0.5), position(0.0, -0.5)] {
        error(
            Size::new(100.0, tiny),
            Some(Point::new(0.0, 0.0)),
            EndingSize::EllipseRadii(Size::new(position(1.0, 0.0), y)),
            RadialGradientError::UnrepresentableGeometry,
        );
    }
    // No finite binary fraction equals one third: this near cancellation is
    // nonzero even though its residual is far smaller than one subnormal unit.
    error(
        Size::new(3.0 * tiny, 100.0),
        Some(Point::new(0.0, 0.0)),
        EndingSize::EllipseRadii(Size::new(position(-tiny, 1.0 / 3.0), position(1.0, 0.0))),
        RadialGradientError::UnrepresentableGeometry,
    );
}

#[test]
fn a_collapsed_default_center_refuses_while_an_explicit_used_zero_center_is_valid() {
    let tiny = f64::from_bits(1);
    for box_size in [
        Size::new(tiny, 100.0),
        Size::new(200.0, tiny),
        Size::new(tiny, tiny),
    ] {
        error(
            box_size,
            None,
            EndingSize::CircleRadius(1.0),
            RadialGradientError::UnrepresentableGeometry,
        );
        let result = geometry(
            box_size,
            Some(Point::new(0.0, 0.0)),
            EndingSize::CircleRadius(1.0),
        );
        assert_eq!(result.center(), Point::new(0.0, 0.0));
        assert_eq!(
            result.case(),
            Case::Ellipse {
                radii: Size::new(1.0, 1.0)
            }
        );
    }
}

#[test]
fn invalid_input_diagnostics_preserve_axis_component_and_validation_priority() {
    let box_size = Size::new(200.0, 100.0);
    let valid = EndingSize::CircleRadius(1.0);
    for invalid in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        error(
            Size::new(invalid, f64::NAN),
            Some(Point::new(f64::NAN, f64::NAN)),
            EndingSize::CircleRadius(f64::NAN),
            RadialGradientError::InvalidWidth,
        );
        error(
            Size::new(200.0, invalid),
            Some(Point::new(f64::NAN, f64::NAN)),
            EndingSize::CircleRadius(f64::NAN),
            RadialGradientError::InvalidHeight,
        );
        error(
            box_size,
            None,
            EndingSize::CircleRadius(invalid),
            RadialGradientError::InvalidCircleRadius,
        );
    }
    for invalid in [f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        error(
            box_size,
            Some(Point::new(invalid, f64::NAN)),
            EndingSize::CircleRadius(f64::NAN),
            RadialGradientError::NonFiniteCenter {
                axis: PhysicalAxis::Horizontal,
            },
        );
        error(
            box_size,
            Some(Point::new(0.0, invalid)),
            valid,
            RadialGradientError::NonFiniteCenter {
                axis: PhysicalAxis::Vertical,
            },
        );
        for (request, axis, component) in [
            (
                EndingSize::EllipseRadii(Size::new(
                    position(invalid, f64::NAN),
                    position(f64::NAN, f64::NAN),
                )),
                PhysicalAxis::Horizontal,
                GradientPositionComponent::Length,
            ),
            (
                EndingSize::EllipseRadii(Size::new(
                    position(0.0, invalid),
                    position(f64::NAN, f64::NAN),
                )),
                PhysicalAxis::Horizontal,
                GradientPositionComponent::Fraction,
            ),
            (
                EndingSize::EllipseRadii(Size::new(
                    position(1.0, 0.0),
                    position(invalid, f64::NAN),
                )),
                PhysicalAxis::Vertical,
                GradientPositionComponent::Length,
            ),
            (
                EndingSize::EllipseRadii(Size::new(position(1.0, 0.0), position(0.0, invalid))),
                PhysicalAxis::Vertical,
                GradientPositionComponent::Fraction,
            ),
        ] {
            error(
                box_size,
                None,
                request,
                RadialGradientError::NonFiniteRadiusComponent { axis, component },
            );
        }
    }
    error(
        box_size,
        None,
        explicit(-1.0, -1.0),
        RadialGradientError::NegativeRadius {
            axis: PhysicalAxis::Horizontal,
        },
    );
    error(
        box_size,
        None,
        explicit(1.0, -1.0),
        RadialGradientError::NegativeRadius {
            axis: PhysicalAxis::Vertical,
        },
    );
    error(
        box_size,
        None,
        explicit(0.0, -1.0),
        RadialGradientError::NegativeRadius {
            axis: PhysicalAxis::Vertical,
        },
    );
    error(
        box_size,
        None,
        EndingSize::EllipseRadii(Size::new(position(0.0, 0.0), position(0.0, f64::NAN))),
        RadialGradientError::NonFiniteRadiusComponent {
            axis: PhysicalAxis::Vertical,
            component: GradientPositionComponent::Fraction,
        },
    );
    error(
        box_size,
        None,
        EndingSize::EllipseRadii(Size::new(position(-1.0, 0.0), position(0.0, f64::NAN))),
        RadialGradientError::NonFiniteRadiusComponent {
            axis: PhysicalAxis::Vertical,
            component: GradientPositionComponent::Fraction,
        },
    );
}

#[test]
fn layout_to_positive_radial_geometry_composes_with_shared_stop_resolution() {
    let box_size = Size::<f64>::new(200.0, 100.0);
    let result = geometry(
        box_size,
        None,
        EndingSize::EllipseRadii(Size::new(position(10.0, 0.25), position(0.0, 0.5))),
    );
    let Case::Ellipse { radii } = result.case() else {
        panic!("normal geometry required for this composition")
    };
    assert_eq!(radii, Size::new(60.0, 50.0));
    let source = [
        GradientStopItem::Stop {
            payload: "near",
            positions: GradientStopPositions::One(position(-10.0, 0.0)),
        },
        GradientStopItem::Stop {
            payload: "middle",
            positions: GradientStopPositions::Omitted,
        },
        GradientStopItem::Stop {
            payload: "far",
            positions: GradientStopPositions::Two(position(0.0, 0.5), position(0.0, 1.0)),
        },
    ];
    let stops = resolve_gradient_stops(radii.width, &source, options(PaintLimits::unlimited()))
        .result
        .unwrap();
    assert_eq!(stops.line_length(), 60.0);
    let observed: Vec<_> = stops
        .items()
        .map(|item| match item {
            UsedGradientItem::Stop {
                source_index,
                position_index,
                payload,
                position,
            } => (source_index, position_index, *payload, position),
            UsedGradientItem::Hint { .. } => panic!("the source contains only stops"),
        })
        .collect();
    assert_eq!(
        observed,
        [
            (0, 0, "near", -10.0),
            (1, 0, "middle", 10.0),
            (2, 0, "far", 30.0),
            (2, 1, "far", 60.0)
        ]
    );
}

const PHASES: [PaintPhase; 4] = [
    PaintPhase::GradientValidation,
    PaintPhase::GradientGeometry,
    PaintPhase::Publication,
    PaintPhase::Cleanup,
];

fn phase_work(report: &PaintReport, phase: PaintPhase, kind: WorkKind) -> u64 {
    report
        .phases()
        .find(|entry| entry.phase() == phase)
        .unwrap()
        .work(kind)
}

fn usage_equal(actual: &PaintReport, expected: &PaintReport) {
    for &kind in WorkKind::ALL {
        assert_eq!(actual.work(kind), expected.work(kind), "{kind:?}");
        for phase in PHASES {
            assert_eq!(
                phase_work(actual, phase, kind),
                phase_work(expected, phase, kind),
                "{phase:?}/{kind:?}"
            );
        }
    }
    for &resource in ResourceKind::ALL {
        assert_eq!(actual.live(resource), expected.live(resource));
        assert_eq!(actual.peak(resource), expected.peak(resource));
    }
}

#[test]
fn fixed_result_custody_and_named_work_are_attributed_without_geometry_heap_storage() {
    let requests = [
        EndingSize::default(),
        EndingSize::CircleRadius(1.0),
        explicit(0.0, 1.0),
        explicit(1.0, 0.0),
        EndingSize::CircleRadius(0.0),
    ];
    for request in requests {
        let outcome = resolve_radial_gradient(
            Size::new(200.0, 100.0),
            None,
            request,
            options(PaintLimits::unlimited()),
        );
        let _result = outcome.result.unwrap();
        let report = outcome.report.unwrap();
        assert_eq!(
            report.live(ResourceKind::RetainedResultBytes),
            size_of::<RadialGradientGeometry>() as u64
        );
        assert_eq!(
            report.peak(ResourceKind::RetainedResultBytes),
            size_of::<RadialGradientGeometry>() as u64
        );
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
        assert_eq!(report.peak(ResourceKind::ScratchBytes), 0);
        assert_eq!(report.work(WorkKind::OutputRecord), 1);
        assert!(report.work(WorkKind::GradientValidation) > 0);
        assert!(report.work(WorkKind::GradientCoordinate) > 0);
        for &kind in WorkKind::ALL {
            if !matches!(
                kind,
                WorkKind::GradientValidation
                    | WorkKind::GradientCoordinate
                    | WorkKind::OutputRecord
            ) {
                assert_eq!(report.work(kind), 0, "{kind:?}");
            }
        }
        for (phase, kind) in [
            (PaintPhase::GradientValidation, WorkKind::GradientValidation),
            (PaintPhase::GradientGeometry, WorkKind::GradientCoordinate),
            (PaintPhase::Publication, WorkKind::OutputRecord),
        ] {
            assert_eq!(phase_work(&report, phase, kind), report.work(kind));
        }
        for phase in PHASES {
            assert_eq!(
                report
                    .phases()
                    .find(|entry| entry.phase() == phase)
                    .unwrap()
                    .invocations(),
                1
            );
        }
    }
}

#[test]
fn work_and_fixed_owner_refusals_publish_no_geometry_release_custody_and_allow_retry() {
    let box_size = Size::new(200.0, 100.0);
    let request = EndingSize::default();
    let expected = geometry(box_size, None, request);
    for kind in [
        WorkKind::GradientValidation,
        WorkKind::GradientCoordinate,
        WorkKind::OutputRecord,
    ] {
        let outcome = resolve_radial_gradient(
            box_size,
            None,
            request,
            options(PaintLimits::unlimited().with_work_limit(kind, 0)),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("expected {kind:?} refusal")
        };
        assert!(
            matches!(refusal.change, PaintChange::Work { kind: actual, amount } if actual == kind && amount > 0)
        );
        assert_eq!(refusal.limiting_phase, PaintPhase::Operation);
        let report = outcome.report.unwrap();
        assert_eq!(report.work(kind), 0);
        no_custody(&report);
        if kind == WorkKind::OutputRecord {
            assert_eq!(
                report.peak(ResourceKind::RetainedResultBytes),
                size_of::<RadialGradientGeometry>() as u64
            );
            assert_eq!(refusal.origin, PaintPhase::Publication);
        }
    }
    for limit in [0, size_of::<RadialGradientGeometry>() as u64 - 1] {
        let outcome = resolve_radial_gradient(
            box_size,
            None,
            request,
            options(
                PaintLimits::unlimited()
                    .with_resource_limit(ResourceKind::RetainedResultBytes, limit),
            ),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("expected fixed-owner refusal")
        };
        assert_eq!(
            refusal.change,
            PaintChange::Acquire {
                kind: ResourceKind::RetainedResultBytes,
                amount: size_of::<RadialGradientGeometry>() as u64
            }
        );
        let report = outcome.report.unwrap();
        no_custody(&report);
        assert_eq!(report.peak(ResourceKind::RetainedResultBytes), 0);
    }
    let exact = resolve_radial_gradient(
        box_size,
        None,
        request,
        options(PaintLimits::unlimited().with_resource_limit(
            ResourceKind::RetainedResultBytes,
            size_of::<RadialGradientGeometry>() as u64,
        )),
    );
    assert_eq!(exact.result, Ok(expected));
    assert_eq!(geometry(box_size, None, request), expected);
}

thread_local! { static CLOCK_READS: Cell<u64> = const { Cell::new(0) }; }

fn clock() -> Duration {
    CLOCK_READS.with(|reads| {
        let now = reads.get();
        reads.set(now + 1);
        Duration::from_nanos(now)
    })
}

fn forbidden_clock() -> Duration {
    panic!("Off and untimed operations must not read the supplied clock")
}

#[test]
fn profile_modes_timing_and_bounded_filtered_events_preserve_cases_usage_and_refusals() {
    let box_size = Size::new(200.0, 100.0);
    CLOCK_READS.with(|reads| reads.set(0));
    for request in [
        EndingSize::default(),
        explicit(0.0, 1.0),
        explicit(1.0, 0.0),
        EndingSize::CircleRadius(0.0),
    ] {
        let baseline = resolve_radial_gradient(
            box_size,
            None,
            request,
            PaintOptions::new(
                PaintLimits::unlimited(),
                ProfileMode::Summary { timing: true },
            )
            .with_clock(clock),
        );
        let expected = baseline.result.unwrap();
        let report = baseline.report.unwrap();
        assert!(report.elapsed().unwrap() > Duration::ZERO);
        for phase in PHASES {
            let measured = report
                .phases()
                .find(|entry| entry.phase() == phase)
                .unwrap();
            assert_eq!(measured.invocations(), 1);
            assert!(measured.elapsed().unwrap() > Duration::ZERO);
        }
        let refusal_limits =
            PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 0);
        let baseline_failure = resolve_radial_gradient(
            box_size,
            None,
            request,
            PaintOptions::new(refusal_limits, ProfileMode::Summary { timing: true })
                .with_clock(clock),
        );
        let Err(expected_error) = baseline_failure.result else {
            panic!("expected work refusal")
        };
        let failure_report = baseline_failure.report.unwrap();
        for profile in [
            ProfileMode::Off,
            ProfileMode::Summary { timing: false },
            ProfileMode::Detailed {
                timing: false,
                event_limit: 0,
                filter: EventFilter::All,
            },
            ProfileMode::Detailed {
                timing: true,
                event_limit: 2,
                filter: EventFilter::All,
            },
            ProfileMode::Detailed {
                timing: false,
                event_limit: 8,
                filter: EventFilter::ScopeName("gradient_geometry"),
            },
        ] {
            let timing = matches!(profile, ProfileMode::Detailed { timing: true, .. });
            let selected_clock = if timing { clock } else { forbidden_clock };
            let outcome = resolve_radial_gradient(
                box_size,
                None,
                request,
                PaintOptions::new(PaintLimits::unlimited(), profile).with_clock(selected_clock),
            );
            assert_eq!(outcome.result, Ok(expected));
            if profile == ProfileMode::Off {
                assert!(outcome.report.is_none());
            } else {
                let actual = outcome.report.unwrap();
                usage_equal(&actual, &report);
                assert_eq!(actual.elapsed().is_some(), timing);
                if let ProfileMode::Detailed {
                    event_limit,
                    filter,
                    ..
                } = profile
                {
                    assert!(actual.events().len() <= event_limit);
                    if filter == EventFilter::All {
                        assert!(actual.dropped_events() > 0);
                    }
                    if matches!(filter, EventFilter::ScopeName(_)) {
                        assert!(actual.events().next().is_some());
                        for event in actual.events() {
                            let origin = match event {
                                PaintEvent::Accepted { phase, .. }
                                | PaintEvent::PhaseCompleted { phase, .. } => phase,
                                PaintEvent::Refused { refusal, .. } => refusal.origin,
                            };
                            assert_eq!(origin, PaintPhase::GradientGeometry);
                        }
                    }
                }
            }
            let failed = resolve_radial_gradient(
                box_size,
                None,
                request,
                PaintOptions::new(refusal_limits, profile).with_clock(selected_clock),
            );
            assert_eq!(failed.result, Err(expected_error.clone()));
            if let Some(actual) = failed.report {
                usage_equal(&actual, &failure_report);
                no_custody(&actual);
            }
        }
    }
}
