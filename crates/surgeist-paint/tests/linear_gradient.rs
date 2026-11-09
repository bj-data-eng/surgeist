#![forbid(unsafe_code)]

use std::{cell::Cell, mem::size_of, time::Duration};
use surgeist_layout::{Point, Size};
use surgeist_paint::{
    EventFilter, GradientGeometryError, LinearGradientDirection as Direction, LinearGradientLine,
    PaintChange, PaintError, PaintLimits, PaintOptions, PaintPhase, PaintReport, ProfileMode,
    ResourceKind, WorkKind, resolve_linear_gradient,
};

fn options(limits: PaintLimits) -> PaintOptions {
    PaintOptions::new(limits, ProfileMode::Summary { timing: false })
}

fn line(size: Size<f64>, direction: Direction) -> LinearGradientLine {
    resolve_linear_gradient(size, direction, options(PaintLimits::unlimited()))
        .result
        .unwrap()
}

fn close(actual: f64, expected: f64) {
    // The anchors below use ordinary pixel-scale arithmetic. Thirty-two binary
    // rounding units allow trig evaluation without hiding coordinate mistakes.
    let tolerance = 32.0 * f64::EPSILON * expected.abs().max(1.0);
    assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "actual {actual:?}, expected {expected:?}, tolerance {tolerance:?}"
    );
}

fn point_close(actual: Point<f64>, expected: Point<f64>) {
    // A zero coordinate still inherits the scale of the physical point.
    let scale = expected.x.abs().max(expected.y.abs()).max(1.0);
    close(actual.x / scale, expected.x / scale);
    close(actual.y / scale, expected.y / scale);
}

fn line_close(actual: LinearGradientLine, start: Point<f64>, end: Point<f64>, length: f64) {
    point_close(actual.start(), start);
    point_close(actual.end(), end);
    close(actual.length(), length);
}

const DIRECTIONS: [Direction; 8] = [
    Direction::Top,
    Direction::Right,
    Direction::Bottom,
    Direction::Left,
    Direction::TopLeft,
    Direction::TopRight,
    Direction::BottomLeft,
    Direction::BottomRight,
];

#[test]
fn physical_directions_resolve_to_independent_rectangular_anchors() {
    // Images3's perpendicular projections give corner displacement (40, 80)
    // about (100, 50): the adjacent-corner diagonal has slope 1/2.
    let corner_length = 80.0 * 5.0_f64.sqrt();
    for (direction, start, end, length) in [
        (Direction::Top, (100.0, 100.0), (100.0, 0.0), 100.0),
        (Direction::Right, (0.0, 50.0), (200.0, 50.0), 200.0),
        (Direction::Bottom, (100.0, 0.0), (100.0, 100.0), 100.0),
        (Direction::Left, (200.0, 50.0), (0.0, 50.0), 200.0),
        (
            Direction::TopLeft,
            (140.0, 130.0),
            (60.0, -30.0),
            corner_length,
        ),
        (
            Direction::TopRight,
            (60.0, 130.0),
            (140.0, -30.0),
            corner_length,
        ),
        (
            Direction::BottomLeft,
            (140.0, -30.0),
            (60.0, 130.0),
            corner_length,
        ),
        (
            Direction::BottomRight,
            (60.0, -30.0),
            (140.0, 130.0),
            corner_length,
        ),
    ] {
        line_close(
            line(Size::new(200.0, 100.0), direction),
            Point::new(start.0, start.1),
            Point::new(end.0, end.1),
            length,
        );
    }
}

#[test]
fn forty_five_degrees_and_corner_directions_have_distinct_rectangular_geometry() {
    let size = Size::new(200.0, 100.0);
    let diagonal = line(size, Direction::AngleDegrees(45.0));
    line_close(
        diagonal,
        Point::new(25.0, 125.0),
        Point::new(175.0, -25.0),
        150.0 * 2.0_f64.sqrt(),
    );
    assert_ne!(diagonal, line(size, Direction::TopRight));
    // A square's adjacent-corner diagonal really is perpendicular to 45deg.
    let square = Size::new(100.0, 100.0);
    let corner = line(square, Direction::TopRight);
    line_close(
        corner,
        Point::new(0.0, 100.0),
        Point::new(100.0, 0.0),
        100.0 * 2.0_f64.sqrt(),
    );
    let square_angle = line(square, Direction::AngleDegrees(45.0));
    point_close(square_angle.start(), corner.start());
    point_close(square_angle.end(), corner.end());
    close(square_angle.length(), corner.length());
}

#[test]
fn neighboring_corners_project_to_the_midpoint_of_each_corner_line() {
    for (direction, neighbors) in [
        (
            Direction::TopRight,
            [Point::new(0.0, 0.0), Point::new(200.0, 100.0)],
        ),
        (
            Direction::BottomLeft,
            [Point::new(0.0, 0.0), Point::new(200.0, 100.0)],
        ),
        (
            Direction::TopLeft,
            [Point::new(200.0, 0.0), Point::new(0.0, 100.0)],
        ),
        (
            Direction::BottomRight,
            [Point::new(200.0, 0.0), Point::new(0.0, 100.0)],
        ),
    ] {
        let result = line(Size::new(200.0, 100.0), direction);
        let start: Point<f64> = result.start();
        let end: Point<f64> = result.end();
        let center = Point::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
        point_close(center, Point::new(100.0, 50.0));
        // Orthogonal projection of either neighboring corner lands at 50%,
        // independently of the producer's trigonometry/normalization method.
        for neighbor in neighbors {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            close(
                ((neighbor.x - start.x) * dx + (neighbor.y - start.y) * dy) / (dx * dx + dy * dy),
                0.5,
            );
        }
    }
}

#[test]
fn oblique_angles_follow_literal_quadrant_and_thirty_degree_anchors() {
    let size = Size::new(200.0, 100.0);
    for (degrees, start, end) in [
        (45.0, (25.0, 125.0), (175.0, -25.0)),
        (135.0, (25.0, -25.0), (175.0, 125.0)),
        (225.0, (175.0, -25.0), (25.0, 125.0)),
        (315.0, (175.0, 125.0), (25.0, -25.0)),
    ] {
        line_close(
            line(size, Direction::AngleDegrees(degrees)),
            Point::new(start.0, start.1),
            Point::new(end.0, end.1),
            150.0 * 2.0_f64.sqrt(),
        );
    }
    // sin30=1/2 and cos30=sqrt3/2 independently fix both sides of
    // the nearest-cardinal reduction boundary, without calling trig here.
    let root_three = 3.0_f64.sqrt();
    line_close(
        line(size, Direction::AngleDegrees(30.0)),
        Point::new(75.0 - 12.5 * root_three, 87.5 + 25.0 * root_three),
        Point::new(125.0 + 12.5 * root_three, 12.5 - 25.0 * root_three),
        100.0 + 50.0 * root_three,
    );
    line_close(
        line(size, Direction::AngleDegrees(60.0)),
        Point::new(25.0 - 12.5 * root_three, 62.5 + 25.0 * root_three),
        Point::new(175.0 + 12.5 * root_three, 37.5 - 25.0 * root_three),
        50.0 + 100.0 * root_three,
    );
}

#[test]
fn opposite_directions_reverse_the_same_line_and_transposition_exchanges_corners() {
    for (forward, reverse) in [
        (Direction::Top, Direction::Bottom),
        (Direction::Right, Direction::Left),
        (Direction::TopRight, Direction::BottomLeft),
        (Direction::TopLeft, Direction::BottomRight),
    ] {
        let forward = line(Size::new(200.0, 100.0), forward);
        let reverse = line(Size::new(200.0, 100.0), reverse);
        point_close(forward.start(), reverse.end());
        point_close(forward.end(), reverse.start());
        close(forward.length(), reverse.length());
    }
    let original = line(Size::new(200.0, 100.0), Direction::TopRight);
    let transposed = line(Size::new(100.0, 200.0), Direction::BottomLeft);
    point_close(original.start().transpose(), transposed.start());
    point_close(original.end().transpose(), transposed.end());
    close(original.length(), transposed.length());
}

#[test]
fn defaults_cardinals_negative_angles_and_full_turns_preserve_the_field() {
    let size = Size::new(200.0, 100.0);
    assert_eq!(
        line(size, Direction::default()),
        line(size, Direction::Bottom)
    );
    for (direction, degrees) in [
        (Direction::Top, 0.0),
        (Direction::Right, 90.0),
        (Direction::Bottom, 180.0),
        (Direction::Left, 270.0),
    ] {
        for turns in [-720.0, -360.0, 0.0, 360.0, 720.0] {
            assert_eq!(
                line(size, Direction::AngleDegrees(degrees + turns)),
                line(size, direction)
            );
        }
    }
    for degrees in [45.0, 135.0, 225.0, 315.0] {
        let expected = line(size, Direction::AngleDegrees(degrees));
        for turns in [-720.0, -360.0, 360.0, 720.0] {
            let actual = line(size, Direction::AngleDegrees(degrees + turns));
            point_close(actual.start(), expected.start());
            point_close(actual.end(), expected.end());
            close(actual.length(), expected.length());
        }
    }
    // Reduction precedes radian conversion, even at the largest finite input.
    for degrees in [f64::MAX, -f64::MAX] {
        assert_eq!(
            line(size, Direction::AngleDegrees(degrees)),
            line(size, Direction::AngleDegrees(degrees % 360.0)),
        );
    }
}

#[test]
fn zero_dimensions_preserve_nonzero_projections_and_true_zero_centers() {
    for direction in DIRECTIONS {
        let result = line(Size::new(0.0, 0.0), direction);
        assert_eq!(result.start(), Point::new(0.0, 0.0));
        assert_eq!(result.end(), Point::new(0.0, 0.0));
        assert_eq!(result.length(), 0.0);
    }
    for angle in [
        -f64::MAX,
        -45.0,
        -f64::from_bits(1),
        0.0,
        f64::from_bits(1),
        45.0,
        f64::MAX,
    ] {
        let result = line(Size::new(0.0, 0.0), Direction::AngleDegrees(angle));
        assert_eq!(result.start(), Point::new(0.0, 0.0));
        assert_eq!(result.end(), Point::new(0.0, 0.0));
        assert_eq!(result.length(), 0.0);
    }
    for (size, direction, start, end, length) in [
        (
            Size::new(0.0, 100.0),
            Direction::Top,
            (0.0, 100.0),
            (0.0, 0.0),
            100.0,
        ),
        (
            Size::new(200.0, 0.0),
            Direction::Right,
            (0.0, 0.0),
            (200.0, 0.0),
            200.0,
        ),
        (
            Size::new(0.0, 100.0),
            Direction::Right,
            (0.0, 50.0),
            (0.0, 50.0),
            0.0,
        ),
        (
            Size::new(200.0, 0.0),
            Direction::Top,
            (100.0, 0.0),
            (100.0, 0.0),
            0.0,
        ),
        (
            Size::new(0.0, 100.0),
            Direction::TopRight,
            (0.0, 50.0),
            (0.0, 50.0),
            0.0,
        ),
        (
            Size::new(200.0, 0.0),
            Direction::TopRight,
            (100.0, 0.0),
            (100.0, 0.0),
            0.0,
        ),
        (
            Size::new(0.0, 100.0),
            Direction::AngleDegrees(45.0),
            (-25.0, 75.0),
            (25.0, 25.0),
            50.0 * 2.0_f64.sqrt(),
        ),
        (
            Size::new(200.0, 0.0),
            Direction::AngleDegrees(45.0),
            (50.0, 50.0),
            (150.0, -50.0),
            100.0 * 2.0_f64.sqrt(),
        ),
    ] {
        line_close(
            line(size, direction),
            Point::new(start.0, start.1),
            Point::new(end.0, end.1),
            length,
        );
    }
    assert_eq!(line(Size::new(-0.0, -0.0), Direction::Bottom).length(), 0.0,);
}

#[test]
fn exact_cardinals_preserve_the_smallest_positive_and_largest_finite_dimensions() {
    for extent in [f64::from_bits(1), f64::MIN_POSITIVE, f64::MAX] {
        let size = Size::new(extent, extent);
        for (direction, start, end) in [
            (
                Direction::Top,
                Point::new(extent / 2.0, extent),
                Point::new(extent / 2.0, 0.0),
            ),
            (
                Direction::Bottom,
                Point::new(extent / 2.0, 0.0),
                Point::new(extent / 2.0, extent),
            ),
            (
                Direction::Right,
                Point::new(0.0, extent / 2.0),
                Point::new(extent, extent / 2.0),
            ),
            (
                Direction::Left,
                Point::new(extent, extent / 2.0),
                Point::new(0.0, extent / 2.0),
            ),
        ] {
            let result = line(size, direction);
            assert_eq!(result.start(), start);
            assert_eq!(result.end(), end);
            assert_eq!(result.length(), extent);
            assert_ne!(result.start(), result.end());
        }
    }
}

fn assert_no_custody(report: &PaintReport) {
    for &resource in ResourceKind::ALL {
        assert_eq!(report.live(resource), 0, "{resource:?}");
    }
}

#[test]
fn invalid_inputs_and_unrepresentable_positive_lines_return_typed_failures() {
    for invalid in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        for (size, expected) in [
            (
                Size::new(invalid, 100.0),
                GradientGeometryError::InvalidWidth,
            ),
            (
                Size::new(200.0, invalid),
                GradientGeometryError::InvalidHeight,
            ),
        ] {
            let outcome =
                resolve_linear_gradient(size, Direction::Top, options(PaintLimits::unlimited()));
            assert_eq!(outcome.result, Err(PaintError::Gradient(expected)));
            assert_no_custody(&outcome.report.unwrap());
        }
    }
    for angle in [f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        let outcome = resolve_linear_gradient(
            Size::new(200.0, 100.0),
            Direction::AngleDegrees(angle),
            options(PaintLimits::unlimited()),
        );
        assert_eq!(
            outcome.result,
            Err(PaintError::Gradient(GradientGeometryError::NonFiniteAngle))
        );
        assert_no_custody(&outcome.report.unwrap());
    }
    for (size, direction) in [
        (Size::new(f64::MAX, f64::MAX), Direction::AngleDegrees(45.0)),
        (Size::new(f64::MAX, f64::MAX), Direction::TopRight),
        // A positive horizontal projection smaller than one subnormal unit
        // cannot become a faithfully represented positive line.
        (
            Size::new(f64::from_bits(1), 0.0),
            Direction::AngleDegrees(1.0),
        ),
        (
            Size::new(1.0, 0.0),
            Direction::AngleDegrees(f64::from_bits(1)),
        ),
    ] {
        let outcome = resolve_linear_gradient(size, direction, options(PaintLimits::unlimited()));
        assert_eq!(
            outcome.result,
            Err(PaintError::Gradient(
                GradientGeometryError::UnrepresentableLine
            ))
        );
        assert_no_custody(&outcome.report.unwrap());
    }
}

#[test]
fn named_work_and_fixed_result_custody_belong_to_their_declared_phases() {
    for (direction, validations) in [(Direction::TopRight, 3), (Direction::AngleDegrees(45.0), 4)] {
        let outcome = resolve_linear_gradient(
            Size::new(200.0, 100.0),
            direction,
            options(PaintLimits::unlimited()),
        );
        let _result = outcome.result.unwrap();
        let report = outcome.report.unwrap();
        // Width, height, and direction discriminant; angles add a scalar.
        assert_eq!(report.work(WorkKind::GradientValidation), validations);
        assert!(report.work(WorkKind::GradientDirection) > 0);
        assert!(report.work(WorkKind::GradientCoordinate) > 0);
        assert_eq!(report.work(WorkKind::OutputRecord), 1);
        assert_eq!(
            report.live(ResourceKind::RetainedResultBytes),
            size_of::<LinearGradientLine>() as u64
        );
        assert_eq!(
            report.peak(ResourceKind::RetainedResultBytes),
            size_of::<LinearGradientLine>() as u64
        );
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
        for (phase, kinds) in [
            (
                PaintPhase::GradientValidation,
                &[WorkKind::GradientValidation][..],
            ),
            (
                PaintPhase::GradientGeometry,
                &[WorkKind::GradientDirection, WorkKind::GradientCoordinate][..],
            ),
            (PaintPhase::Publication, &[WorkKind::OutputRecord][..]),
        ] {
            let attribution = report
                .phases()
                .find(|entry| entry.phase() == phase)
                .unwrap();
            assert_eq!(attribution.invocations(), 1);
            for &kind in kinds {
                assert_eq!(attribution.work(kind), report.work(kind));
            }
        }
    }
}

#[test]
fn zero_work_and_result_budgets_refuse_without_partial_geometry_or_live_custody() {
    for kind in [
        WorkKind::GradientValidation,
        WorkKind::GradientDirection,
        WorkKind::GradientCoordinate,
        WorkKind::OutputRecord,
    ] {
        let outcome = resolve_linear_gradient(
            Size::new(200.0, 100.0),
            Direction::TopRight,
            options(PaintLimits::unlimited().with_work_limit(kind, 0)),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("expected refusal for {kind:?}");
        };
        assert!(
            matches!(refusal.change, PaintChange::Work { kind: refused, amount } if refused == kind && amount > 0)
        );
        assert_eq!(refusal.limiting_phase, PaintPhase::Operation);
        let report = outcome.report.unwrap();
        assert_eq!(report.work(kind), 0);
        assert_no_custody(&report);
    }
    for limit in [0, size_of::<LinearGradientLine>() as u64 - 1] {
        let outcome = resolve_linear_gradient(
            Size::new(200.0, 100.0),
            Direction::TopRight,
            options(
                PaintLimits::unlimited()
                    .with_resource_limit(ResourceKind::RetainedResultBytes, limit),
            ),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("expected fixed-result refusal");
        };
        assert_eq!(
            refusal.change,
            PaintChange::Acquire {
                kind: ResourceKind::RetainedResultBytes,
                amount: size_of::<LinearGradientLine>() as u64
            }
        );
        assert_eq!(refusal.limiting_phase, PaintPhase::Operation);
        assert_no_custody(&outcome.report.unwrap());
    }
    let exact = resolve_linear_gradient(
        Size::new(200.0, 100.0),
        Direction::TopRight,
        options(PaintLimits::unlimited().with_resource_limit(
            ResourceKind::RetainedResultBytes,
            size_of::<LinearGradientLine>() as u64,
        )),
    );
    line_close(
        exact.result.unwrap(),
        Point::new(60.0, 130.0),
        Point::new(140.0, -30.0),
        80.0 * 5.0_f64.sqrt(),
    );
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
    panic!("Off and untimed profiles must not read the supplied clock")
}

fn accounting_equal(actual: &PaintReport, expected: &PaintReport) {
    for &kind in WorkKind::ALL {
        assert_eq!(actual.work(kind), expected.work(kind), "{kind:?}");
    }
    for &resource in ResourceKind::ALL {
        assert_eq!(actual.live(resource), expected.live(resource));
        assert_eq!(actual.peak(resource), expected.peak(resource));
    }
}

#[test]
fn profiling_and_bounded_detail_preserve_geometry_admission_and_usage() {
    let size = Size::new(200.0, 100.0);
    CLOCK_READS.with(|reads| reads.set(0));
    let timed = resolve_linear_gradient(
        size,
        Direction::TopRight,
        PaintOptions::new(
            PaintLimits::unlimited(),
            ProfileMode::Summary { timing: true },
        )
        .with_clock(clock),
    );
    let expected = timed.result.unwrap();
    let baseline = timed.report.unwrap();
    let refusal_limits = PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 0);
    let timed_refusal = resolve_linear_gradient(
        size,
        Direction::TopRight,
        PaintOptions::new(refusal_limits, ProfileMode::Summary { timing: true }).with_clock(clock),
    );
    assert!(matches!(timed_refusal.result, Err(PaintError::Budget(_))));
    let refusal_report = timed_refusal.report.unwrap();
    assert_no_custody(&refusal_report);
    assert!(baseline.elapsed().unwrap() > Duration::ZERO);
    for phase in [
        PaintPhase::GradientValidation,
        PaintPhase::GradientGeometry,
        PaintPhase::Publication,
    ] {
        let phase = baseline
            .phases()
            .find(|entry| entry.phase() == phase)
            .unwrap();
        assert_eq!(phase.invocations(), 1);
        assert!(phase.elapsed().unwrap() > Duration::ZERO);
    }
    for profile in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::All,
        },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 2,
            filter: EventFilter::All,
        },
    ] {
        let outcome = resolve_linear_gradient(
            size,
            Direction::TopRight,
            PaintOptions::new(PaintLimits::unlimited(), profile).with_clock(forbidden_clock),
        );
        assert_eq!(outcome.result, Ok(expected));
        if profile == ProfileMode::Off {
            assert!(outcome.report.is_none());
        } else {
            let report = outcome.report.unwrap();
            accounting_equal(&report, &baseline);
            assert_eq!(report.elapsed(), None);
            assert!(report.phases().all(|phase| phase.elapsed().is_none()));
            if let ProfileMode::Detailed { event_limit, .. } = profile {
                assert!(report.events().len() <= event_limit);
                assert!(report.dropped_events() > 0);
            }
        }
        let failed = resolve_linear_gradient(
            size,
            Direction::TopRight,
            PaintOptions::new(refusal_limits, profile).with_clock(forbidden_clock),
        );
        assert_eq!(failed.result, timed_refusal.result);
        if let Some(report) = failed.report {
            accounting_equal(&report, &refusal_report);
            assert_no_custody(&report);
        }
    }
}
