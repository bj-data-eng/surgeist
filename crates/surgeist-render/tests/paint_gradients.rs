#![cfg(feature = "render-paint")]
#![forbid(unsafe_code)]

use std::{cell::Cell, mem::size_of, time::Duration};
use surgeist_layout as layout;
use surgeist_paint as paint;
use surgeist_render::{
    Color, Error, ErrorCode, GradientStop, PaintGradientError, RenderAccountingContext,
    RenderAccountingError, RenderAccountingOptions, RenderAccountingSetupError, RenderAttemptKind,
    RenderAttemptReport, RenderAttemptStatus, RenderClock, RenderEvent, RenderLimits, RenderPhase,
    RenderPhaseFilter, RenderProfile, RenderQuantity, RenderResource, RenderWork,
    RepeatingGradient, prepare_linear_gradient, prepare_radial_gradient,
    prepare_repeating_linear_gradient, prepare_repeating_radial_gradient,
};

fn red() -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
}
fn green() -> Color {
    Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap()
}
fn blue() -> Color {
    Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()
}

fn paint_options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}

fn position(length: f64, fraction: f64) -> paint::GradientPosition {
    paint::GradientPosition { length, fraction }
}

fn at(value: f64, color: Color) -> paint::GradientStopItem<Color> {
    paint::GradientStopItem::Stop {
        payload: color,
        positions: paint::GradientStopPositions::One(position(value, 0.0)),
    }
}

fn omitted(color: Color) -> paint::GradientStopItem<Color> {
    paint::GradientStopItem::Stop {
        payload: color,
        positions: paint::GradientStopPositions::Omitted,
    }
}

fn line(length: f64) -> paint::LinearGradientLine {
    paint::resolve_linear_gradient(
        layout::Size::<f64>::new(length, 1.0),
        paint::LinearGradientDirection::Right,
        paint_options(),
    )
    .result
    .unwrap()
}

fn radial(x: f64, y: f64) -> paint::RadialGradientGeometry {
    paint::resolve_radial_gradient(
        layout::Size::<f64>::new(16.0, 8.0),
        Some(layout::Point::new(8.0, 4.0)),
        paint::RadialGradientSize::EllipseRadii(layout::Size::new(
            position(x, 0.0),
            position(y, 0.0),
        )),
        paint_options(),
    )
    .result
    .unwrap()
}

fn stops(
    length: f64,
    source: &[paint::GradientStopItem<Color>],
) -> paint::GradientStops<'_, Color> {
    paint::resolve_gradient_stops(length, source, paint_options())
        .result
        .unwrap()
}

fn observed(gradient: &RepeatingGradient) -> Vec<(f64, Color)> {
    gradient
        .stops()
        .iter()
        .map(|stop| (stop.position(), stop.color()))
        .collect()
}

fn context(limits: RenderLimits) -> RenderAccountingContext {
    RenderAccountingContext::new(
        RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Summary { timing: false }),
    )
}

fn diagnostic(error: &Error) -> &PaintGradientError {
    assert_eq!(error.code(), ErrorCode::PaintGradient);
    assert!(error.glyph_preparation_diagnostic().is_none());
    assert!(error.accounting_diagnostic().is_none());
    error.paint_gradient_diagnostic().unwrap()
}

fn no_custody(report: &RenderAttemptReport) {
    assert_eq!(report.kind(), RenderAttemptKind::PaintGradient);
    for resource in [
        RenderResource::RetainedCpuBytes,
        RenderResource::ScratchCpuBytes,
    ] {
        assert_eq!(report.usage().live(resource), 0, "{resource:?}");
    }
}

#[test]
fn actual_paint_line_and_shared_fixup_preserve_geometry_colors_and_expanded_order() {
    let geometry = paint::resolve_linear_gradient(
        layout::Size::new(200.0, 100.0),
        paint::LinearGradientDirection::Top,
        paint_options(),
    )
    .result
    .unwrap();
    let source = [
        paint::GradientStopItem::Stop {
            payload: red(),
            positions: paint::GradientStopPositions::Two(position(-50.0, 0.0), position(0.0, 0.0)),
        },
        at(0.0, green()),
        at(150.0, blue()),
    ];
    let used = stops(geometry.length(), &source);
    let mut account = context(RenderLimits::new());
    let prepared = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    let (start, end) = prepared.linear_points().unwrap();
    assert_eq!((start.x(), start.y()), (100.0, 100.0));
    assert_eq!((end.x(), end.y()), (100.0, 0.0));
    assert!(prepared.radial_geometry().is_none());
    assert_eq!(
        observed(&prepared),
        [(-0.5, red()), (0.0, red()), (0.0, green()), (1.5, blue())]
    );
    let report = account.last_attempt().unwrap().attempt().unwrap();
    no_custody(report);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
}

#[test]
fn normal_radial_geometry_uses_the_horizontal_ray_basis_and_keeps_both_actual_axes() {
    let geometry = radial(4.0, 2.0);
    let source = [at(-1.0, red()), omitted(green()), at(4.0, blue())];
    let used = stops(4.0, &source);
    let mut account = RenderAccountingContext::default();
    let prepared = prepare_repeating_radial_gradient(&geometry, &used, &mut account).unwrap();
    let (center, x, y) = prepared.radial_geometry().unwrap();
    assert_eq!((center.x(), center.y(), x, y), (8.0, 4.0, 4.0, 2.0));
    assert!(prepared.linear_points().is_none());
    assert_eq!(
        observed(&prepared),
        [(-0.25, red()), (0.375, green()), (1.0, blue())]
    );
}

#[test]
fn returned_gradient_and_its_clone_outlive_all_source_storage_and_the_context() {
    let prepared = {
        let geometry = line(8.0);
        let source = vec![at(0.0, red()), at(8.0, blue())];
        let used = stops(8.0, &source);
        let mut account = context(RenderLimits::new());
        prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap()
    };
    let cloned = prepared.clone();
    drop(prepared);
    assert_eq!(observed(&cloned), [(0.0, red()), (1.0, blue())]);
    let (_, end) = cloned.linear_points().unwrap();
    assert_eq!(end.x(), 8.0);
    let _: surgeist_render::Paint = cloned.into();
}

#[test]
fn zero_geometry_precedes_basis_and_source_conversion_and_radial_zero_cases_are_explicit() {
    let source = [
        at(0.0, red()),
        paint::GradientStopItem::Hint {
            position: position(5.0, 0.0),
        },
        at(10.0, blue()),
    ];
    let mismatched = stops(10.0, &source);
    let mut account = context(RenderLimits::new());
    let error =
        prepare_repeating_linear_gradient(&line(0.0), &mismatched, &mut account).unwrap_err();
    assert!(matches!(
        diagnostic(&error),
        PaintGradientError::UnsupportedZeroLengthLinear
    ));
    for geometry in [
        paint::resolve_radial_gradient(
            layout::Size::new(16.0, 8.0),
            None,
            paint::RadialGradientSize::CircleRadius(0.0),
            paint_options(),
        )
        .result
        .unwrap(),
        radial(2.0, 0.0),
    ] {
        let error =
            prepare_repeating_radial_gradient(&geometry, &mismatched, &mut account).unwrap_err();
        let PaintGradientError::UnsupportedRadialCase { case } = diagnostic(&error) else {
            panic!("expected a typed radial case")
        };
        assert_eq!(*case, geometry.case());
        no_custody(account.last_attempt().unwrap().attempt().unwrap());
    }
    for geometry in [radial(0.0, 2.0), radial(0.0, 0.0)] {
        let error =
            prepare_repeating_radial_gradient(&geometry, &mismatched, &mut account).unwrap_err();
        assert_eq!(
            diagnostic(&error),
            &PaintGradientError::BasisMismatch {
                expected: 0.0,
                actual: 10.0,
            }
        );
        let report = account.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 0);
        no_custody(report);
    }
}

#[test]
fn exact_basis_equality_precedes_source_item_conversion() {
    let source = [
        at(0.0, red()),
        paint::GradientStopItem::Hint {
            position: position(5.0, 0.0),
        },
        at(10.0, blue()),
    ];
    let actual = f64::from_bits(8.0_f64.to_bits() + 1);
    let used = stops(actual, &source);
    let mut account = context(RenderLimits::new());
    for error in [
        prepare_repeating_linear_gradient(&line(8.0), &used, &mut account).unwrap_err(),
        prepare_repeating_radial_gradient(&radial(8.0, 2.0), &used, &mut account).unwrap_err(),
    ] {
        let PaintGradientError::BasisMismatch {
            expected,
            actual: reported,
        } = diagnostic(&error)
        else {
            panic!("expected exact basis mismatch")
        };
        assert_eq!(*expected, 8.0);
        assert_eq!(*reported, actual);
    }
}

#[test]
fn adopted_hint_bounded_fixup_and_outgoing_associations_survive_all_four_consumers() {
    let source = [
        at(0.0, red()),
        omitted(green()),
        paint::GradientStopItem::Hint {
            position: position(20.0, 0.0),
        },
        at(100.0, blue()),
    ];
    let used = stops(100.0, &source);
    let positions: Vec<_> = used
        .items()
        .map(|item| match item {
            paint::UsedGradientItem::Stop { position, .. }
            | paint::UsedGradientItem::Hint { position, .. } => position,
        })
        .collect();
    assert_eq!(positions, [0.0, 10.0, 20.0, 100.0]);
    let mut account = context(RenderLimits::new());
    let prepared = [
        prepare_linear_gradient(&line(100.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
        prepare_radial_gradient(&radial(100.0, 50.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
        prepare_repeating_linear_gradient(&line(100.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
        prepare_repeating_radial_gradient(&radial(100.0, 50.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
    ];
    for output in prepared {
        assert_eq!(
            output
                .iter()
                .map(|stop| (stop.position(), stop.color()))
                .collect::<Vec<_>>(),
            [(0.0, red()), (0.1, green()), (1.0, blue())]
        );
        assert!(output[0].hint_after().is_none());
        assert_eq!(output[1].hint_after().unwrap().position(), 0.2);
        assert!(output[2].hint_after().is_none());
    }
    no_custody(account.last_attempt().unwrap().attempt().unwrap());
    assert_eq!(
        account.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
}

#[test]
fn source_numeric_refusals_keep_hint_and_two_ordinals_instead_of_old_hint_preflight() {
    let converted: Error = PaintGradientError::UnrepresentablePosition {
        source_index: 7,
        position_index: 1,
    }
    .into();
    assert_eq!(
        converted.paint_gradient_diagnostic(),
        Some(&PaintGradientError::UnrepresentablePosition {
            source_index: 7,
            position_index: 1
        })
    );
    // The first color loses its positive quotient. A later authentic hint is
    // accepted during classification, so the real first numeric refusal wins.
    for (source, source_index) in [
        (
            [
                at(f64::from_bits(1), red()),
                paint::GradientStopItem::Hint {
                    position: position(1.0, 0.0),
                },
                at(2.0, blue()),
            ],
            0,
        ),
        (
            [
                at(0.0, red()),
                paint::GradientStopItem::Hint {
                    position: position(f64::from_bits(1), 0.0),
                },
                at(2.0, blue()),
            ],
            1,
        ),
    ] {
        let used = stops(2.0, &source);
        let mut account = context(RenderLimits::new());
        for error in [
            prepare_linear_gradient(&line(2.0), &used, &mut account).unwrap_err(),
            prepare_radial_gradient(&radial(2.0, 1.0), &used, &mut account).unwrap_err(),
            prepare_repeating_linear_gradient(&line(2.0), &used, &mut account).unwrap_err(),
            prepare_repeating_radial_gradient(&radial(2.0, 1.0), &used, &mut account).unwrap_err(),
        ] {
            assert_eq!(
                diagnostic(&error),
                &PaintGradientError::UnrepresentablePosition {
                    source_index,
                    position_index: 0
                }
            );
        }
        no_custody(account.last_attempt().unwrap().attempt().unwrap());
        assert_eq!(
            account.last_attempt().unwrap().attempt().unwrap().status(),
            RenderAttemptStatus::Failed(ErrorCode::PaintGradient)
        );
        assert_eq!(
            account
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::PreparedTransferredBytes),
            0
        );
    }
}

#[test]
fn hint_topology_collapse_reports_the_actual_hint_or_following_two_ordinal() {
    let first = f64::from_bits(1.5_f64.to_bits() + 1);
    let second = f64::from_bits(1.5_f64.to_bits() + 2);
    assert_eq!(first / 3.0, second / 3.0);
    for (source, source_index) in [
        (
            vec![
                at(first, red()),
                paint::GradientStopItem::Hint {
                    position: position(second, 0.0),
                },
                at(3.0, blue()),
            ],
            1,
        ),
        (
            vec![
                at(0.0, red()),
                paint::GradientStopItem::Hint {
                    position: position(first, 0.0),
                },
                paint::GradientStopItem::Stop {
                    payload: blue(),
                    positions: paint::GradientStopPositions::Two(
                        position(second, 0.0),
                        position(3.0, 0.0),
                    ),
                },
            ],
            2,
        ),
    ] {
        let used = stops(3.0, &source);
        let mut account = context(RenderLimits::new());
        for error in [
            prepare_linear_gradient(&line(3.0), &used, &mut account).unwrap_err(),
            prepare_radial_gradient(&radial(3.0, 1.0), &used, &mut account).unwrap_err(),
            prepare_repeating_linear_gradient(&line(3.0), &used, &mut account).unwrap_err(),
            prepare_repeating_radial_gradient(&radial(3.0, 1.0), &used, &mut account).unwrap_err(),
        ] {
            assert_eq!(
                diagnostic(&error),
                &PaintGradientError::UnrepresentablePosition {
                    source_index,
                    position_index: 0
                }
            );
        }
        no_custody(account.last_attempt().unwrap().attempt().unwrap());
    }
}

#[test]
fn hint_after_two_attaches_to_ordinal_one_before_following_ordinal_zero() {
    let source = [
        paint::GradientStopItem::Stop {
            payload: red(),
            positions: paint::GradientStopPositions::Two(position(0.0, 0.0), position(0.2, 0.0)),
        },
        paint::GradientStopItem::Hint {
            position: position(0.3, 0.0),
        },
        paint::GradientStopItem::Stop {
            payload: blue(),
            positions: paint::GradientStopPositions::Two(position(0.8, 0.0), position(1.0, 0.0)),
        },
    ];
    let used = stops(1.0, &source);
    let mut account = context(RenderLimits::new());
    let prepared = [
        prepare_linear_gradient(&line(1.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
        prepare_radial_gradient(&radial(1.0, 2.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
        prepare_repeating_linear_gradient(&line(1.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
        prepare_repeating_radial_gradient(&radial(1.0, 2.0), &used, &mut account)
            .unwrap()
            .stops()
            .to_vec(),
    ];
    for output in prepared {
        assert_eq!(
            output
                .iter()
                .map(|stop| stop.position())
                .collect::<Vec<_>>(),
            [0.0, 0.2, 0.8, 1.0]
        );
        assert_eq!(
            output
                .iter()
                .map(|stop| stop.hint_after().map(|hint| hint.position()))
                .collect::<Vec<_>>(),
            [None, Some(0.3), None, None]
        );
    }
}

#[test]
fn quotient_underflow_overflow_and_distinct_stop_collapse_are_typed_refusals() {
    for (length, value) in [
        (2.0, f64::from_bits(1)),
        (2.0, -f64::from_bits(1)),
        (f64::from_bits(2), f64::MAX),
    ] {
        let source = [at(value, red())];
        let used = stops(length, &source);
        let mut account = context(RenderLimits::new());
        for error in [
            prepare_repeating_linear_gradient(&line(length), &used, &mut account).unwrap_err(),
            prepare_repeating_radial_gradient(&radial(length, 1.0), &used, &mut account)
                .unwrap_err(),
        ] {
            assert!(matches!(
                diagnostic(&error),
                PaintGradientError::UnrepresentablePosition {
                    source_index: 0,
                    position_index: 0
                }
            ));
        }
        no_custody(account.last_attempt().unwrap().attempt().unwrap());
    }
    // In this binade, division by 3 maps these adjacent f64 values to one
    // quotient. A distinct later source position must not silently coalesce.
    let first = f64::from_bits(1.5_f64.to_bits() + 1);
    let second = f64::from_bits(1.5_f64.to_bits() + 2);
    assert_eq!(first / 3.0, second / 3.0);
    let source = [
        at(0.0, red()),
        paint::GradientStopItem::Stop {
            payload: blue(),
            positions: paint::GradientStopPositions::Two(
                position(first, 0.0),
                position(second, 0.0),
            ),
        },
    ];
    let used = stops(3.0, &source);
    let mut account = context(RenderLimits::new());
    for error in [
        prepare_repeating_linear_gradient(&line(3.0), &used, &mut account).unwrap_err(),
        prepare_repeating_radial_gradient(&radial(3.0, 1.0), &used, &mut account).unwrap_err(),
    ] {
        assert!(matches!(
            diagnostic(&error),
            PaintGradientError::UnrepresentablePosition {
                source_index: 1,
                position_index: 1
            }
        ));
    }
    let accepted_source = [at(second, blue())];
    let accepted =
        prepare_repeating_linear_gradient(&line(3.0), &stops(3.0, &accepted_source), &mut account)
            .unwrap();
    assert_eq!(accepted.stops()[0].position(), second / 3.0);
}

#[test]
fn ordinary_nonzero_quotient_rounding_is_accepted_and_true_equal_stops_remain_distinct() {
    for (length, physical, expected) in [
        (100.0, 0.9, 0.009000000000000001),
        (3.0, 0.1, 0.03333333333333333),
    ] {
        let source = [at(physical, red()), at(physical, blue())];
        let used = stops(length, &source);
        let mut account = RenderAccountingContext::default();
        let prepared =
            prepare_repeating_linear_gradient(&line(length), &used, &mut account).unwrap();
        assert_eq!(observed(&prepared), [(expected, red()), (expected, blue())]);
        let prepared =
            prepare_repeating_radial_gradient(&radial(length, 1.0), &used, &mut account).unwrap();
        assert_eq!(observed(&prepared), [(expected, red()), (expected, blue())]);
    }
    let source = [at(-0.0, red()), at(0.0, blue())];
    let prepared = prepare_repeating_linear_gradient(
        &line(8.0),
        &stops(8.0, &source),
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    assert_eq!(observed(&prepared), [(0.0, red()), (0.0, blue())]);
}

#[test]
fn only_the_converted_span_must_be_finite_not_the_original_physical_span() {
    let source = [
        paint::GradientStopItem::Stop {
            payload: red(),
            positions: paint::GradientStopPositions::Two(
                position(-f64::MAX, 0.0),
                position(0.0, 0.0),
            ),
        },
        at(f64::MAX, blue()),
    ];
    let mut account = context(RenderLimits::new());
    let accepted =
        prepare_repeating_linear_gradient(&line(f64::MAX), &stops(f64::MAX, &source), &mut account)
            .unwrap();
    assert_eq!(
        observed(&accepted),
        [(-1.0, red()), (0.0, red()), (1.0, blue())]
    );
    let radial_accepted = prepare_repeating_radial_gradient(
        &radial(f64::MAX, 1.0),
        &stops(f64::MAX, &source),
        &mut account,
    )
    .unwrap();
    assert_eq!(
        observed(&radial_accepted),
        [(-1.0, red()), (0.0, red()), (1.0, blue())]
    );
    for error in [
        prepare_repeating_linear_gradient(&line(1.0), &stops(1.0, &source), &mut account)
            .unwrap_err(),
        prepare_repeating_radial_gradient(&radial(1.0, 1.0), &stops(1.0, &source), &mut account)
            .unwrap_err(),
    ] {
        assert!(matches!(
            diagnostic(&error),
            PaintGradientError::UnrepresentableSpan {
                first_source_index: 0,
                last_source_index: 1
            }
        ));
    }
    no_custody(account.last_attempt().unwrap().attempt().unwrap());
}

#[test]
fn retained_backing_is_transferred_once_and_every_named_refusal_releases_its_prefix() {
    let geometry = line(8.0);
    let source = [at(-4.0, red()), at(0.0, green()), at(12.0, blue())];
    let used = stops(8.0, &source);
    let mut account = context(RenderLimits::new());
    let expected = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    let baseline_record = account.take_last_attempt().unwrap();
    let baseline = baseline_record.attempt().unwrap();
    no_custody(baseline);
    let transferred = baseline.usage().work(RenderWork::PreparedTransferredBytes);
    assert!(transferred >= (size_of::<RepeatingGradient>() + 3 * size_of::<GradientStop>()) as u64);
    assert_eq!(
        transferred,
        baseline.usage().peak(RenderResource::RetainedCpuBytes)
    );
    assert_eq!(baseline.usage().peak(RenderResource::ScratchCpuBytes), 0);
    assert_eq!(baseline.usage().work(RenderWork::Submissions), 0);
    for work in [
        RenderWork::PaintVisits,
        RenderWork::GradientStopVisits,
        RenderWork::CoordinateVisits,
    ] {
        let total = baseline.usage().work(work);
        assert!(total > 0);
        for limit in [0, total - 1] {
            let limits = RenderLimits::new().with_work_limit(work, limit).unwrap();
            account.set_options(RenderAccountingOptions::new().with_limits(limits));
            let error =
                prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected a named Render refusal")
            };
            assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
            assert!(error.glyph_preparation_diagnostic().is_none());
            let report = account.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert!(report.usage().work(work) <= limit);
            assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
            no_custody(report);
        }
    }
    for limit in [0, transferred - 1] {
        account.set_options(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(limit)),
        );
        let error = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
        assert!(
            matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(refusal)) if refusal.quantity() == RenderQuantity::Resource(RenderResource::RetainedCpuBytes))
        );
        no_custody(account.last_attempt().unwrap().attempt().unwrap());
    }
    account.set_options(
        RenderAccountingOptions::new()
            .with_limits(RenderLimits::new().with_retained_bytes(transferred)),
    );
    assert_eq!(
        prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap(),
        expected
    );
}

#[test]
fn invalid_limit_and_report_setup_failure_preserve_the_previous_completed_attempt() {
    let geometry = line(8.0);
    let source = [at(0.0, red()), at(8.0, blue())];
    let used = stops(8.0, &source);
    let mut account = context(RenderLimits::new());
    prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    let observe = |account: &RenderAccountingContext| {
        let report = account.last_attempt().unwrap();
        (
            report.kind(),
            report.status(),
            report.attempt().cloned(),
            report.cleanup_fault(),
        )
    };
    let previous = observe(&account);
    account.set_options(
        RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::GlyphVisits, 0)
                .unwrap(),
        ),
    );
    let error = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::InvalidLimit(RenderWork::GlyphVisits))
    ));
    assert_eq!(observe(&account), previous);
    account.set_options(
        RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
            timing: false,
            event_limit: usize::MAX,
            filter: RenderPhaseFilter::All,
        }),
    );
    let error = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Setup(
            RenderAccountingSetupError::Allocation
        ))
    ));
    assert_eq!(observe(&account), previous);
    account.set_options(RenderAccountingOptions::new());
    prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    assert_eq!(
        account.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
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
    panic!("Off and untimed preparation must not read a clock")
}

fn usage_equal(actual: &RenderAttemptReport, expected: &RenderAttemptReport) {
    assert_eq!(actual.kind(), expected.kind());
    assert_eq!(actual.status(), expected.status());
    assert_eq!(actual.refusal(), expected.refusal());
    assert_eq!(actual.usage(), expected.usage());
    for phase in [
        RenderPhase::Preparation,
        RenderPhase::Coordinates,
        RenderPhase::Output,
        RenderPhase::Cleanup,
    ] {
        assert_eq!(actual.phase_usage(phase), expected.phase_usage(phase));
    }
}

#[test]
fn off_untimed_timed_and_filtered_profiles_preserve_output_admission_and_usage() {
    let geometry = line(8.0);
    let source = [at(0.0, red()), at(8.0, blue())];
    let used = stops(8.0, &source);
    CLOCK_READS.with(|reads| reads.set(0));
    let mut account = RenderAccountingContext::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock)),
    );
    let expected = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    let baseline_record = account.take_last_attempt().unwrap();
    let baseline = baseline_record.attempt().unwrap();
    assert!(baseline.elapsed().unwrap() > Duration::ZERO);
    for phase in [
        RenderPhase::Preparation,
        RenderPhase::Coordinates,
        RenderPhase::Output,
        RenderPhase::Cleanup,
    ] {
        assert!(baseline.phase_count(phase) > 0);
        assert!(baseline.phase_elapsed(phase).unwrap() > Duration::ZERO);
    }
    assert_eq!(baseline.phase_count(RenderPhase::GradientPrepare), 0);
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CoordinateVisits, 0)
        .unwrap();
    account.set_options(
        RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock)),
    );
    let baseline_error =
        prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
    let baseline_diagnostic = baseline_error.accounting_diagnostic().unwrap();
    let failure_record = account.take_last_attempt().unwrap();
    let failure = failure_record.attempt().unwrap();
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 2,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 8,
            filter: RenderPhaseFilter::Phase(RenderPhase::Coordinates),
        },
    ] {
        let timing = matches!(profile, RenderProfile::Detailed { timing: true, .. });
        let selected_clock = if timing { clock } else { forbidden_clock };
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(selected_clock));
        account.set_options(options);
        assert_eq!(
            prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap(),
            expected
        );
        let actual = account.last_attempt().unwrap().attempt().unwrap();
        usage_equal(actual, baseline);
        assert_eq!(actual.elapsed().is_some(), timing);
        if let RenderProfile::Detailed {
            event_limit,
            filter,
            ..
        } = profile
        {
            assert!(actual.events().count() <= event_limit);
            if filter == RenderPhaseFilter::All {
                assert!(actual.dropped_events() > 0);
            }
            if let RenderPhaseFilter::Phase(expected) = filter {
                assert!(actual.events().next().is_some());
                for event in actual.events() {
                    let phase = match event {
                        RenderEvent::Change { phase, .. }
                        | RenderEvent::Phase { phase, .. }
                        | RenderEvent::Refused { phase, .. } => phase,
                    };
                    assert_eq!(phase, expected);
                }
            }
        }
        account.set_options(options.with_limits(limits));
        let error = prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap_err();
        assert_eq!(error.accounting_diagnostic(), Some(baseline_diagnostic));
        usage_equal(account.last_attempt().unwrap().attempt().unwrap(), failure);
        no_custody(account.last_attempt().unwrap().attempt().unwrap());
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod gpu {
    use super::*;
    use surgeist_render::{
        EffectQualityPolicy, FontData, FontRef, GlyphRunLimits, GlyphSynthesis, Image, ImageBuffer,
        Layer, Options, Parameters, Rect, RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene,
        Shape, Size, Stroke, Surface, TextGlyph, TextInstance, TextPaint, TextRun, TextRunBounds,
        Transform,
    };

    fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect::try_new(x, y, width, height).unwrap()
    }

    fn linear_field(length: f64) -> RepeatingGradient {
        let geometry = line(length);
        let source = [at(0.0, red()), at(length, blue())];
        let used = stops(length, &source);
        prepare_repeating_linear_gradient(&geometry, &used, &mut RenderAccountingContext::default())
            .unwrap()
    }

    fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
        let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
        image.rgba()[index..index + 4].try_into().unwrap()
    }

    fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, expected: [u8; 4], tolerance: u8) {
        let actual = pixel(image, x, y);
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
            "pixel ({x},{y}) actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
        );
    }

    fn read(
        renderer: &mut Renderer,
        surface: &mut Surface,
        scene: &Scene,
        graph: bool,
    ) -> ImageBuffer {
        let stats =
            pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
        assert_eq!(
            stats.route,
            Some(if graph {
                RenderRoute::GpuGraph
            } else {
                RenderRoute::DirectVello
            })
        );
        pollster::block_on(renderer.read_headless(surface)).unwrap()
    }

    fn scene(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
        let mut scene = Scene::new();
        if graph {
            // A constant half-alpha image forces the public graph route and
            // makes mask execution observable independently of gradient RGB.
            let mask = Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128])
                .unwrap();
            let layer = Layer::new().with_resolved_alpha_mask(
                ResolvedLayerAlphaMask::try_new(mask, rect(0.0, 0.0, 32.0, 8.0)).unwrap(),
            );
            scene.layer(layer, draw);
        } else {
            draw(&mut scene);
        }
        scene
    }

    #[test]
    fn tiny_public_gpu_composition_preserves_fields_for_fill_stroke_glyphs_masks_and_scale() {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let linear = linear_field(4.0);
        let glyph_paint = TextPaint::try_fill(linear_field(0.75).into()).unwrap();
        let font = FontData::try_from_bytes(
            include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        // Ahem X is glyph 58, with a one-em square and baseline .8.
        let glyphs = [
            TextGlyph::try_new(58, 0.0, 0.8, 1.0).unwrap(),
            TextGlyph::try_new(58, 2.0, 0.8, 1.0).unwrap(),
        ];
        let prepared_glyphs = font
            .prepare_glyph_run(
                &TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap(),
                &glyphs,
                GlyphRunLimits::new(2, 16, 1 << 20, 1 << 20),
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
        let radial_geometry = radial(4.0, 2.0);
        let radial_source = [at(0.0, red()), at(4.0, blue())];
        let radial_field = prepare_repeating_radial_gradient(
            &radial_geometry,
            &stops(4.0, &radial_source),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let transparent_green = Color::try_rgba(0.0, 1.0, 0.0, 0.0).unwrap();
        let average_cases = [
            ([0.0, 0.25, 1.0], [64, 0, 191, 128]),
            ([0.0, 0.0, 0.0], [128, 0, 128, 128]),
        ]
        .map(|(positions, expected)| {
            let geometry = line(4.0);
            let source = [
                at(positions[0], red()),
                at(positions[1], transparent_green),
                at(positions[2], blue()),
            ];
            let gradient = prepare_repeating_linear_gradient(
                &geometry,
                &stops(4.0, &source),
                &mut RenderAccountingContext::default(),
            )
            .unwrap();
            (gradient, expected)
        });
        // One Renderer and two tiny surfaces execute 28 frames: the original
        // 20 geometry/command frames plus eight independent average frames.
        for scale in [1_u32, 2] {
            let mut surface = pollster::block_on(
                renderer.create_headless(Size::try_new(32.0, 8.0).unwrap(), f64::from(scale)),
            )
            .unwrap();
            for graph in [false, true] {
                let alpha = if graph { 128 } else { 255 };
                let tolerance = if graph { 5 } else { 3 };
                for stroke in [false, true] {
                    let scene = scene(graph, |scene| {
                        scene.transform(Transform::translation(2.0, 0.0).unwrap(), |scene| {
                            scene.transform(Transform::scale(2.0, 1.0).unwrap(), |scene| {
                                if stroke {
                                    scene.stroke(
                                        Shape::rect(rect(0.0, 1.0, 12.0, 2.0)),
                                        Stroke::try_new(2.0).unwrap(),
                                        linear.clone(),
                                    );
                                } else {
                                    scene.fill(rect(0.0, 0.0, 12.0, 4.0), linear.clone());
                                }
                            });
                        });
                    });
                    let image = read(&mut renderer, &mut surface, &scene, graph);
                    // Integer physical indices sample local x=(output_x-2)/2.
                    // Dividing by L=4 gives phases 1/4,1/2,3/4 at x=4,6,8.
                    for (x, r, b) in [(4, 191, 64), (6, 128, 128), (8, 64, 191), (12, 191, 64)] {
                        assert_pixel(&image, x * scale, scale, [r, 0, b, alpha], tolerance);
                    }
                }
                for native in [false, true] {
                    let scene = scene(graph, |scene| {
                        scene.transform(Transform::translation(2.0, 0.0).unwrap(), |scene| {
                            let transform = Transform::scale(8.0, 4.0).unwrap();
                            if native {
                                scene.text_run(
                                    TextRun::try_new(
                                        FontRef::new(2085).with_data(font.clone()),
                                        1.0,
                                        transform,
                                        glyph_paint.clone(),
                                        &glyphs,
                                        TextRunBounds::try_ink(rect(0.0, 0.0, 3.0, 1.0)).unwrap(),
                                    )
                                    .unwrap(),
                                );
                            } else {
                                scene.prepared_glyph_run(
                                    prepared_glyphs.clone(),
                                    transform,
                                    glyph_paint.clone(),
                                );
                            }
                        });
                    });
                    let image = read(&mut renderer, &mut surface, &scene, graph);
                    // x=3,19 map to run-local 1/8,17/8. With L=3/4,
                    // phases are 1/6 and 5/6, so per-glyph restart cannot pass.
                    assert_pixel(&image, 3 * scale, scale, [213, 0, 43, alpha], tolerance);
                    assert_pixel(&image, 19 * scale, scale, [43, 0, 213, alpha], tolerance);
                    assert_pixel(&image, 12 * scale, scale, [0, 0, 0, 0], 1);
                }
                for (gradient, expected) in &average_cases {
                    let scene = scene(graph, |scene| {
                        // The positive physical period 1 becomes .5 or 1 device
                        // pixel at these scales, strictly below the threshold.
                        scene.transform(Transform::scale(0.5, 1.0).unwrap(), |scene| {
                            scene.fill(rect(0.0, 0.0, 64.0, 8.0), gradient.clone());
                        });
                    });
                    let image = read(&mut renderer, &mut surface, &scene, graph);
                    // Segment integration gives premultiplied (.125,0,.375,.5).
                    // Equal positions use uniform weights (.25,.5,.25), giving
                    // premultiplied (.25,0,.25,.5). The mask halves alpha only.
                    let mut expected = *expected;
                    if graph {
                        expected[3] = 64;
                    }
                    for (x, y) in [(4, 2), (24, 6)] {
                        assert_pixel(&image, x * scale, y * scale, expected, tolerance);
                    }
                }
                let scene = scene(graph, |scene| {
                    scene.transform(Transform::translation(2.0, 0.0).unwrap(), |scene| {
                        scene.transform(Transform::scale(2.0, 1.0).unwrap(), |scene| {
                            scene.fill(rect(0.0, 0.0, 16.0, 8.0), radial_field.clone());
                        });
                    });
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                // Transformed center=(18,4), radii=(8,2); quarter and half
                // ellipse rays give identical phases on the matching axes.
                for (x, y, r, b) in [(20, 4, 191, 64), (22, 4, 128, 128), (18, 5, 128, 128)] {
                    assert_pixel(&image, x * scale, y * scale, [r, 0, b, alpha], tolerance);
                }
            }
        }
    }
}
