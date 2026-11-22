use super::*;
use crate::accounting::{PaintChange, PaintLimits, PaintPhase};
use crate::gradient_stops::{GradientPositionComponent, GradientStopPositions, GradientStopsError};
use crate::radial_gradient::{RadialGradientSize, resolve_radial_gradient};
use std::ptr;
use surgeist_layout::Size;
use surgeist_ledger::ProfileMode;

fn options(limits: PaintLimits) -> PaintOptions {
    PaintOptions::new(limits, ProfileMode::Summary { timing: false })
}

fn radial_geometry(radius: f64) -> RadialGradientGeometry {
    resolve_radial_gradient(
        Size::new(20.0, 10.0),
        Some(Point::new(6.0, 7.0)),
        RadialGradientSize::CircleRadius(radius),
        options(PaintLimits::unlimited()),
    )
    .result
    .unwrap()
}

fn explicit(length: f64, fraction: f64) -> GradientStopPositions {
    GradientStopPositions::One(GradientPosition { length, fraction })
}

fn stop<C>(payload: C, positions: GradientStopPositions) -> GradientStopItem<C> {
    GradientStopItem::Stop { payload, positions }
}

fn unit_stops(
    source: &[GradientStopItem<()>],
) -> Result<ZeroCircleGradientStops<'_, ()>, PaintError> {
    resolve_zero_circle_gradient_stops(
        &radial_geometry(0.0),
        source,
        options(PaintLimits::unlimited()),
    )
    .result
}

fn positions<C>(result: &ZeroCircleGradientStops<'_, C>) -> Vec<(f64, f64)> {
    result
        .records()
        .map(|record| (record.position().length(), record.position().fraction()))
        .collect()
}

#[test]
fn negative_lengths_dominate_fractions_and_hints_clamp_following_records() {
    let source = [
        stop((), explicit(-7.0, 1000.0)),
        stop((), explicit(-5.0, -1000.0)),
        GradientStopItem::Hint {
            position: GradientPosition {
                length: -5.0,
                fraction: 9.0,
            },
        },
        stop((), explicit(-5.0, 8.0)),
        stop((), explicit(-7.0, 99.0)),
        stop((), GradientStopPositions::Omitted),
    ];
    let result = unit_stops(&source).unwrap();
    assert_eq!(result.center(), Point::new(6.0, 7.0));
    assert_eq!(
        positions(&result),
        [
            (-7.0, 1000.0),
            (-5.0, -1000.0),
            (-5.0, 9.0),
            (-5.0, 9.0),
            (-5.0, 9.0),
            (0.0, 1.0),
        ]
    );
    let hint = result.records().nth(2).unwrap();
    assert!(matches!(hint.kind(), ZeroCircleGradientRecordKind::Hint));
    assert_eq!((hint.source_index(), hint.position_index()), (2, 0));
}

#[test]
fn omitted_positions_interpolate_both_components_on_each_side_of_a_hint() {
    let source = [
        stop((), explicit(-8.0, 6.0)),
        stop((), GradientStopPositions::Omitted),
        GradientStopItem::Hint {
            position: GradientPosition {
                length: -4.0,
                fraction: 2.0,
            },
        },
        stop((), GradientStopPositions::Omitted),
        stop((), GradientStopPositions::Omitted),
    ];
    let result = unit_stops(&source).unwrap();
    assert_eq!(
        positions(&result),
        [
            (-8.0, 6.0),
            (-6.0, 4.0),
            (-4.0, 2.0),
            (-2.0, 1.5),
            (0.0, 1.0)
        ]
    );
}

#[test]
fn one_omitted_stop_uses_the_first_default() {
    let source = [stop((), GradientStopPositions::Omitted)];
    let result = unit_stops(&source).unwrap();
    assert_eq!(positions(&result), [(0.0, 0.0)]);
}

#[test]
fn equal_positions_preserve_each_ordinal_and_the_exact_payload() {
    let position = GradientPosition {
        length: 3.0,
        fraction: 0.0,
    };
    let source = [
        stop(
            String::from("one"),
            GradientStopPositions::Two(position, position),
        ),
        stop(String::from("two"), GradientStopPositions::Omitted),
        stop(String::from("three"), explicit(3.0, 0.0)),
    ];
    let result = resolve_zero_circle_gradient_stops(
        &radial_geometry(0.0),
        &source,
        options(PaintLimits::unlimited()),
    )
    .result
    .unwrap();
    assert!(ptr::eq(result.source(), source.as_slice()));
    let mut records = result.records();
    assert_eq!(records.len(), 4);
    for (expected_item, expected_ordinal) in [(0, 0), (0, 1), (1, 0), (2, 0)] {
        let record = records.next().unwrap();
        assert_eq!(record.source_index(), expected_item);
        assert_eq!(record.position_index(), expected_ordinal);
        assert_eq!(
            (record.position().length(), record.position().fraction()),
            (3.0, 0.0)
        );
        let GradientStopItem::Stop {
            payload: expected, ..
        } = &source[expected_item]
        else {
            panic!("fixture must contain only stops");
        };
        let ZeroCircleGradientRecordKind::Stop { payload } = record.kind() else {
            panic!("stop must retain its payload");
        };
        assert!(ptr::eq(payload, expected));
    }
    assert_eq!(records.len(), 0);
}

#[test]
fn omitted_run_refuses_a_collapsed_length_or_fraction_separation() {
    for (left, right) in [
        (explicit(1.0, 1.0), explicit(1.0_f64.next_up(), 2.0)),
        (explicit(1.0, 1.0), explicit(1.0, 1.0_f64.next_up())),
    ] {
        let source = [
            stop((), left),
            stop((), GradientStopPositions::Omitted),
            stop((), right),
        ];
        assert_eq!(
            unit_stops(&source).unwrap_err(),
            PaintError::ZeroCircleGradient(ZeroCircleGradientError::UnrepresentableDistribution {
                left_source_index: 0,
                left_position_index: 0,
                right_source_index: 1,
                right_position_index: 0,
            })
        );
    }
}

#[test]
fn another_radial_case_refuses_before_scanning_the_stop_list() {
    let outcome = resolve_zero_circle_gradient_stops::<()>(
        &radial_geometry(1.0),
        &[],
        options(PaintLimits::unlimited()),
    );
    assert_eq!(
        outcome.result.unwrap_err(),
        PaintError::ZeroCircleGradient(ZeroCircleGradientError::ExpectedZeroCircle)
    );
    let report = outcome.report.unwrap();
    assert_eq!(report.work(WorkKind::GradientStopVisit), 0);
    assert_eq!(report.peak(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.peak(ResourceKind::RetainedResultBytes), 0);
}

#[test]
fn malformed_stop_retains_original_component_diagnostic() {
    let source = [stop((), explicit(0.0, f64::INFINITY))];
    assert_eq!(
        unit_stops(&source).unwrap_err(),
        PaintError::GradientStops(GradientStopsError::NonFiniteComponent {
            source_index: 0,
            position_index: 0,
            component: GradientPositionComponent::Fraction,
        })
    );
}

#[test]
fn final_owner_refusal_releases_prepared_records_and_fixed_custody() {
    let source = [
        stop((), GradientStopPositions::Omitted),
        stop((), GradientStopPositions::Omitted),
    ];
    // Two staging records, two prepared records and one result owner.
    let accepted_outputs = 4;
    let outcome = resolve_zero_circle_gradient_stops(
        &radial_geometry(0.0),
        &source,
        options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, accepted_outputs)),
    );
    let PaintError::Budget(refusal) = outcome.result.unwrap_err() else {
        panic!("the final owner must refuse");
    };
    assert_eq!(refusal.origin, PaintPhase::Publication);
    assert_eq!(refusal.current, accepted_outputs);
    assert_eq!(
        refusal.change,
        PaintChange::Work {
            kind: WorkKind::OutputRecord,
            amount: 1
        }
    );
    let report = outcome.report.unwrap();
    assert_eq!(report.work(WorkKind::OutputRecord), accepted_outputs);
    assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
    assert_eq!(
        report.peak(ResourceKind::ScratchBytes),
        bytes::<AffineRecord>(2).unwrap()
    );
    assert_eq!(
        report.peak(ResourceKind::RetainedResultBytes),
        bytes::<PreparedRecord<'_, ()>>(2).unwrap()
            + bytes::<ZeroCircleGradientStops<'_, ()>>(1).unwrap()
    );
}

#[test]
fn thirds_keep_exact_zero_and_the_small_nonzero_residual() {
    // For the first of two omitted records, (2 * -1 + b) / 3 is
    // exactly zero at b=2 and exactly next_binary64_after_2_minus_2 / 3
    // at b=2.next_up(). The same rule applies to both coefficients.
    let after_two = 2.0_f64.next_up();
    for (left, right, expected) in [
        (explicit(-1.0, -1.0), explicit(2.0, 2.0), (0.0, 0.0)),
        (
            explicit(-1.0, -1.0),
            explicit(after_two, after_two),
            ((after_two - 2.0) / 3.0, (after_two - 2.0) / 3.0),
        ),
    ] {
        let source = [
            stop((), left),
            stop((), GradientStopPositions::Omitted),
            stop((), GradientStopPositions::Omitted),
            stop((), right),
        ];
        let result = unit_stops(&source).unwrap();
        assert_eq!(positions(&result)[1], expected);
    }
}

#[test]
fn crossing_omission_handles_maximum_lengths_without_a_finite_difference() {
    let source = [
        stop((), explicit(-f64::MAX, 0.0)),
        stop((), GradientStopPositions::Omitted),
        stop((), GradientStopPositions::Omitted),
        stop((), GradientStopPositions::Omitted),
        stop((), explicit(f64::MAX, 0.0)),
    ];
    let result = unit_stops(&source).unwrap();
    assert_eq!(
        positions(&result),
        [
            (-f64::MAX, 0.0),
            (-f64::MAX / 2.0, 0.0),
            (0.0, 0.0),
            (f64::MAX / 2.0, 0.0),
            (f64::MAX, 0.0),
        ]
    );
}

#[test]
fn underflow_prevents_a_rounded_stop_at_the_center() {
    let tiny = f64::from_bits(1);
    for (left, right) in [
        (explicit(-tiny, 0.0), explicit(2.0 * tiny, 0.0)),
        (explicit(-1.0, 0.0), explicit(1.0, tiny)),
    ] {
        let source = [
            stop((), left),
            stop((), GradientStopPositions::Omitted),
            stop((), right),
        ];
        assert_eq!(
            unit_stops(&source).unwrap_err(),
            PaintError::ZeroCircleGradient(ZeroCircleGradientError::UnrepresentableDistribution {
                left_source_index: 0,
                left_position_index: 0,
                right_source_index: 1,
                right_position_index: 0,
            })
        );
    }
}

#[test]
fn exact_half_subnormal_uses_even_rounding_before_accepting_a_coefficient() {
    let tiny = f64::from_bits(1);
    for sign in [1.0, -1.0] {
        let source = |right: f64| {
            let mut source = vec![stop((), explicit(0.0, 0.0_f64.copysign(sign)))];
            source.extend((0..9).map(|_| stop((), GradientStopPositions::Omitted)));
            source.push(stop((), explicit(10.0, right * sign)));
            source
        };
        // The first omitted coefficient is right / 10, while its length is
        // strictly between its neighbors. 5 * tiny / 10 ties at even zero.
        let tie = source(5.0 * tiny);
        let Err(PaintError::ZeroCircleGradient(
            ZeroCircleGradientError::UnrepresentableDistribution {
                left_source_index: 0,
                right_source_index: 1,
                ..
            },
        )) = unit_stops(&tie)
        else {
            panic!("an exact half-subnormal must refuse before publication");
        };
        // The next value is above the threshold. At 1.5 and 2.5 units the
        // same tie rule rounds up from odd and down from even, respectively.
        for (endpoint_units, expected_units) in [(6.0, 1.0), (15.0, 2.0), (25.0, 2.0)] {
            let above = source(endpoint_units * tiny);
            let result = unit_stops(&above).unwrap();
            assert_eq!(positions(&result)[1], (1.0, expected_units * tiny * sign));
        }
    }
}
