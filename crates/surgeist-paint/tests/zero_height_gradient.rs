#![forbid(unsafe_code)]

use std::{cell::Cell, ptr, rc::Rc, time::Duration};
use surgeist_layout::{Point, Size};
use surgeist_paint::{
    EventFilter, GradientPosition, GradientPositionComponent, GradientRepeat, GradientStopItem,
    GradientStopPositions, GradientStopsError, PaintChange, PaintError, PaintLimits, PaintOptions,
    PaintPhase, PaintReport, ProfileMode, RadialGradientGeometry, RadialGradientSize, ResourceKind,
    WorkKind, ZeroHeightGradientColors, ZeroHeightGradientError, resolve_radial_gradient,
    resolve_zero_height_gradient_colors,
};

fn options() -> PaintOptions {
    PaintOptions::new(
        PaintLimits::unlimited(),
        ProfileMode::Summary { timing: false },
    )
}

fn position(length: f64, fraction: f64) -> GradientPosition {
    GradientPosition { length, fraction }
}

fn at(length: f64, fraction: f64) -> GradientStopItem<()> {
    GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::One(position(length, fraction)),
    }
}

fn omitted() -> GradientStopItem<()> {
    GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::Omitted,
    }
}

fn hint(length: f64, fraction: f64) -> GradientStopItem<()> {
    GradientStopItem::Hint {
        position: position(length, fraction),
    }
}

fn geometry(x: f64, y: f64) -> RadialGradientGeometry {
    resolve_radial_gradient(
        Size::new(32.0, 16.0),
        Some(Point::new(8.0, 8.0)),
        RadialGradientSize::EllipseRadii(Size::new(position(x, 0.0), position(y, 0.0))),
        options(),
    )
    .result
    .unwrap()
}

fn resolved<C>(
    radius: f64,
    source: &[GradientStopItem<C>],
    repeat: GradientRepeat,
) -> ZeroHeightGradientColors<'_, C> {
    resolve_zero_height_gradient_colors(&geometry(radius, 0.0), source, repeat, options())
        .result
        .unwrap()
}

fn observed<C>(measure: &ZeroHeightGradientColors<'_, C>) -> Vec<(usize, usize, f64)> {
    let mut colors = measure.colors();
    let count = colors.len();
    let observed: Vec<_> = colors
        .by_ref()
        .map(|color| (color.source_index, color.position_index, color.weight))
        .collect();
    assert_eq!(observed.len(), count);
    assert_eq!(colors.len(), 0);
    observed
}

fn weights<C>(measure: &ZeroHeightGradientColors<'_, C>, expected: &[f64]) {
    let actual = observed(measure);
    assert_eq!(actual.len(), expected.len());
    for ((_, _, actual), expected) in actual.iter().zip(expected) {
        let tolerance = 8.0 * f64::EPSILON;
        assert!(
            actual.is_finite() && *actual >= 0.0 && (actual - expected).abs() <= tolerance,
            "weight {actual:?}, expected {expected:?}, tolerance {tolerance:?}"
        );
    }
    assert!((actual.iter().map(|entry| entry.2).sum::<f64>() - 1.0).abs() <= 8.0 * f64::EPSILON);
}

#[test]
fn substituted_width_limit_discards_lower_order_lengths_and_ignores_original_radius() {
    let source = [at(0.0, 0.0), at(10.0, 0.0), at(0.0, 1.0)];
    for radius in [1.0, 20.0, 100.0] {
        let repeat = resolved(radius, &source, GradientRepeat::Repeat);
        weights(&repeat, &[0.0, 0.5, 0.5]);
        assert_eq!(
            observed(&repeat)
                .iter()
                .map(|x| (x.0, x.1))
                .collect::<Vec<_>>(),
            [(0, 0), (1, 0), (2, 0)]
        );
        let ordinary = resolved(radius, &source, GradientRepeat::None);
        assert_eq!(observed(&ordinary), [(2, 0, 1.0)]);
    }
    // A finite basis20 would instead place these at0,.5,1 and yield
    // [.25,.5,.25]. These literal weights distinguish the adopted limit.
}

#[test]
fn tied_fractions_keep_lengths_and_lexicographic_fixup_copies_the_complete_pair() {
    for source in [
        [at(0.0, 5.0), at(10.0, 5.0), at(40.0, 5.0)],
        [at(0.0, 0.0), at(10.0, 0.0), at(40.0, 0.0)],
        [at(0.0, -2.0), at(0.0, 0.0), at(0.0, 6.0)],
    ] {
        weights(
            &resolved(20.0, &source, GradientRepeat::Repeat),
            &[0.125, 0.5, 0.375],
        );
    }
    for source in [
        [at(0.0, 0.0), at(100.0, -1.0), at(0.0, 1.0)],
        // The middle (1,100) must become (2,10), not (2,100).
        // Independent component clamping would produce [.5,.5,0].
        [at(10.0, 2.0), at(100.0, 1.0), at(40.0, 2.0)],
    ] {
        weights(
            &resolved(20.0, &source, GradientRepeat::Repeat),
            &[0.0, 0.5, 0.5],
        );
    }
}

#[test]
fn normalization_precedes_omission_distribution_for_subnormal_and_overflowing_spans() {
    let tiny = f64::from_bits(1);
    for source in [
        [omitted(), omitted(), omitted(), omitted()],
        [at(0.0, 0.0), omitted(), omitted(), at(0.0, tiny)],
        [at(0.0, 5.0), omitted(), omitted(), at(tiny, 5.0)],
        [at(-f64::MAX, 0.0), omitted(), omitted(), at(f64::MAX, 0.0)],
    ] {
        weights(
            &resolved(20.0, &source, GradientRepeat::Repeat),
            &[1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0],
        );
    }
    for source in [
        [at(0.0, -f64::MAX), at(0.0, 0.0), at(0.0, f64::MAX)],
        [at(-f64::MAX, 0.0), at(0.0, 0.0), at(f64::MAX, 0.0)],
    ] {
        weights(
            &resolved(20.0, &source, GradientRepeat::Repeat),
            &[0.25, 0.5, 0.25],
        );
    }
}

#[test]
fn hints_bound_omitted_colors_but_unused_hint_precision_has_no_color_measure() {
    let omitted_source = [at(0.0, 0.0), omitted(), hint(0.0, 0.75), at(0.0, 1.0)];
    let result = resolved(20.0, &omitted_source, GradientRepeat::Repeat);
    weights(&result, &[3.0 / 16.0, 0.5, 5.0 / 16.0]);
    assert_eq!(
        observed(&result).iter().map(|x| x.0).collect::<Vec<_>>(),
        [0, 1, 3]
    );
    let explicit = [at(0.0, 0.0), at(0.0, 0.375), at(0.0, 1.0)];
    weights(
        &resolved(20.0, &explicit, GradientRepeat::Repeat),
        &[3.0 / 16.0, 0.5, 5.0 / 16.0],
    );
    for h in [0.375, 0.5, 1.0] {
        let explicit = [at(0.0, 0.0), at(0.0, 0.375), hint(0.0, h), at(0.0, 1.0)];
        weights(
            &resolved(20.0, &explicit, GradientRepeat::Repeat),
            &[3.0 / 16.0, 0.5, 5.0 / 16.0],
        );
    }
    for source in [
        [
            at(0.0, 0.0),
            hint(f64::from_bits(1), 0.0),
            at(f64::MAX, 0.0),
        ],
        [
            at(-f64::MAX, 0.0),
            hint(0.0, 0.0),
            at(f64::from_bits(1), 0.0),
        ],
    ] {
        weights(
            &resolved(20.0, &source, GradientRepeat::Repeat),
            &[0.5, 0.5],
        );
    }
}

struct Opaque {
    label: String,
    drops: Rc<Cell<usize>>,
}

#[test]
fn single_authored_two_and_hinted_pairs_keep_arity_during_partial_iteration() {
    let drops = Rc::new(Cell::new(0));
    for (first, last) in [(0.0, 0.0), (-f64::MAX, f64::MAX)] {
        let source = [GradientStopItem::Stop {
            payload: Opaque {
                label: "one payload, two occurrences".to_owned(),
                drops: Rc::clone(&drops),
            },
            positions: GradientStopPositions::Two(position(first, 0.0), position(last, 0.0)),
        }];
        let GradientStopItem::Stop { payload, .. } = &source[0] else {
            unreachable!()
        };
        let average = resolved(20.0, &source, GradientRepeat::Repeat);
        let mut colors = average.colors();
        assert_eq!(colors.len(), 2);
        let first = colors.next().unwrap();
        assert_eq!(
            (first.source_index, first.position_index, first.weight),
            (0, 0, 0.5)
        );
        assert!(ptr::eq(first.payload, payload));
        assert_eq!(colors.len(), 1);
        drop(colors);
        assert_eq!(observed(&average), [(0, 0, 0.5), (0, 1, 0.5)]);
        let last = resolved(20.0, &source, GradientRepeat::None);
        let mut colors = last.colors();
        assert_eq!(colors.len(), 1);
        let color = colors.next().unwrap();
        assert_eq!(
            (color.source_index, color.position_index, color.weight),
            (0, 1, 1.0)
        );
        assert!(ptr::eq(color.payload, payload));
        assert_eq!(colors.len(), 0);
        assert!(colors.next().is_none());
        assert!(colors.next().is_none());
    }
    assert_eq!(drops.get(), 2);
    for source in [
        vec![at(0.0, 0.0), at(0.0, 1.0)],
        vec![at(0.0, 0.0), hint(0.0, 0.25), at(0.0, 1.0)],
        vec![at(0.0, 1.0), hint(0.0, 1.0), at(0.0, 1.0)],
    ] {
        let result = resolved(20.0, &source, GradientRepeat::Repeat);
        assert_eq!(observed(&result), [(0, 0, 0.5), (source.len() - 1, 0, 0.5)]);
        let mut colors = result.colors();
        assert_eq!(colors.len(), 2);
        assert_eq!(colors.next().unwrap().source_index, 0);
        assert_eq!(colors.len(), 1);
        drop(colors);
        assert_eq!(result.colors().len(), 2);
    }
}

impl Drop for Opaque {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn expanded_color_occurrences_borrow_opaque_payloads_with_exact_two_ordinals() {
    let drops = Rc::new(Cell::new(0));
    // Opaque deliberately implements none of Clone, Copy, Debug or equality.
    let source = [
        GradientStopItem::Stop {
            payload: Opaque {
                label: "red".to_owned(),
                drops: Rc::clone(&drops),
            },
            positions: GradientStopPositions::Two(position(0.0, 0.0), position(0.0, 0.1)),
        },
        GradientStopItem::Hint {
            position: position(0.0, 0.2),
        },
        GradientStopItem::Stop {
            payload: Opaque {
                label: "blue".to_owned(),
                drops: Rc::clone(&drops),
            },
            positions: GradientStopPositions::Two(position(0.0, 0.8), position(0.0, 1.0)),
        },
    ];
    {
        let result = resolved(20.0, &source, GradientRepeat::Repeat);
        assert!(ptr::eq(result.source(), source.as_slice()));
        weights(&result, &[0.05, 0.4, 0.45, 0.1]);
        let mut colors = result.colors();
        for (index, (source_index, position_index)) in
            [(0, 0), (0, 1), (2, 0), (2, 1)].into_iter().enumerate()
        {
            assert_eq!(colors.len(), 4 - index);
            let record = colors.next().unwrap();
            assert_eq!(
                (record.source_index, record.position_index),
                (source_index, position_index)
            );
            let GradientStopItem::Stop { payload, .. } = &source[source_index] else {
                unreachable!()
            };
            assert!(ptr::eq(record.payload, payload));
            assert_eq!(
                record.payload.label,
                if source_index == 0 { "red" } else { "blue" }
            );
        }
        assert_eq!(colors.len(), 0);
        let ordinary = resolved(20.0, &source, GradientRepeat::None);
        assert_eq!(observed(&ordinary), [(2, 1, 1.0)]);
    }
    assert_eq!(drops.get(), 0);
    drop(source);
    assert_eq!(drops.get(), 2);
}

#[test]
fn equal_whole_spans_count_actual_colors_and_last_ignores_unused_arithmetic() {
    let two = |a, b| GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::Two(position(0.0, a), position(0.0, b)),
    };
    let source = [two(0.0, 0.1), at(0.0, 1.0)];
    weights(
        &resolved(20.0, &source, GradientRepeat::Repeat),
        &[0.05, 0.5, 0.45],
    );
    let source = [two(0.0, 0.0), hint(0.0, 0.0), at(0.0, 0.0)];
    weights(
        &resolved(20.0, &source, GradientRepeat::Repeat),
        &[0.25, 0.5, 0.25],
    );
    let single = [at(f64::MAX, f64::MAX)];
    assert_eq!(
        observed(&resolved(20.0, &single, GradientRepeat::Repeat)),
        [(0, 0, 1.0)]
    );
    let loss = [at(0.0, 0.0), at(f64::from_bits(1), 0.0), at(f64::MAX, 0.0)];
    assert_eq!(
        observed(&resolved(20.0, &loss, GradientRepeat::None)),
        [(2, 0, 1.0)]
    );
}

fn no_custody(report: &PaintReport) {
    for &resource in ResourceKind::ALL {
        assert_eq!(report.live(resource), 0, "{resource:?}");
    }
}

#[test]
fn case_priority_raw_grammar_and_real_normalized_loss_have_exact_diagnostics() {
    for geometry in [
        geometry(20.0, 8.0),
        geometry(0.0, 8.0),
        geometry(0.0, 0.0),
        resolve_radial_gradient(
            Size::new(32.0, 16.0),
            None,
            RadialGradientSize::CircleRadius(0.0),
            options(),
        )
        .result
        .unwrap(),
    ] {
        let invalid = [at(f64::NAN, f64::INFINITY)];
        for repeat in [GradientRepeat::None, GradientRepeat::Repeat] {
            let outcome =
                resolve_zero_height_gradient_colors(&geometry, &invalid, repeat, options());
            assert_eq!(
                outcome.result.unwrap_err(),
                PaintError::ZeroHeightGradient(ZeroHeightGradientError::ExpectedZeroHeight)
            );
            no_custody(&outcome.report.unwrap());
        }
    }
    for (source, expected) in [
        (vec![], GradientStopsError::EmptyList),
        (
            vec![at(f64::NAN, 0.0), at(0.0, 1.0)],
            GradientStopsError::NonFiniteComponent {
                source_index: 0,
                position_index: 0,
                component: GradientPositionComponent::Length,
            },
        ),
        (
            vec![hint(f64::NAN, 0.0)],
            GradientStopsError::EndpointHint { source_index: 0 },
        ),
        (
            vec![at(0.0, 0.0), hint(0.0, 0.25), hint(0.0, 0.5), at(0.0, 1.0)],
            GradientStopsError::AdjacentHints {
                previous_source_index: 1,
                source_index: 2,
            },
        ),
        (
            vec![
                at(0.0, 0.0),
                GradientStopItem::Stop {
                    payload: (),
                    positions: GradientStopPositions::Two(
                        position(0.0, 0.0),
                        position(0.0, f64::NAN),
                    ),
                },
            ],
            GradientStopsError::NonFiniteComponent {
                source_index: 1,
                position_index: 1,
                component: GradientPositionComponent::Fraction,
            },
        ),
    ] {
        for repeat in [GradientRepeat::None, GradientRepeat::Repeat] {
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(20.0, 0.0),
                &source,
                repeat,
                options(),
            );
            assert_eq!(
                outcome.result.unwrap_err(),
                PaintError::GradientStops(expected)
            );
            no_custody(&outcome.report.unwrap());
        }
    }
    for (source, left, right) in [
        (
            vec![at(0.0, 0.0), at(f64::from_bits(1), 0.0), at(f64::MAX, 0.0)],
            (0, 0),
            (1, 0),
        ),
        // The normalized interval survives as the minimum subnormal, but its
        // required positive half-weight does not survive binary64 rounding.
        (
            vec![at(0.0, 0.0), at(f64::from_bits(1), 0.0), at(1.0, 0.0)],
            (0, 0),
            (1, 0),
        ),
        // The hint carries strictness; the following color equals that hint.
        // Losing the tiny color interval still names colors, excluding the hint.
        (
            vec![
                at(0.0, 0.0),
                hint(f64::from_bits(1), 0.0),
                at(f64::from_bits(1), 0.0),
                at(f64::MAX, 0.0),
            ],
            (0, 0),
            (2, 0),
        ),
        (
            vec![
                GradientStopItem::Stop {
                    payload: (),
                    positions: GradientStopPositions::Two(
                        position(0.0, 0.0),
                        position(f64::from_bits(1), 0.0),
                    ),
                },
                at(f64::MAX, 0.0),
            ],
            (0, 0),
            (0, 1),
        ),
    ] {
        let outcome = resolve_zero_height_gradient_colors(
            &geometry(20.0, 0.0),
            &source,
            GradientRepeat::Repeat,
            options(),
        );
        assert_eq!(
            outcome.result.unwrap_err(),
            PaintError::ZeroHeightGradient(ZeroHeightGradientError::UnrepresentableWeight {
                left_source_index: left.0,
                left_position_index: left.1,
                right_source_index: right.0,
                right_position_index: right.1,
            })
        );
        no_custody(&outcome.report.unwrap());
    }
}

thread_local! { static CLOCK: Cell<u64> = const { Cell::new(0) }; }
fn clock() -> Duration {
    CLOCK.with(|value| {
        let now = value.get();
        value.set(now + 1);
        Duration::from_nanos(now)
    })
}
fn forbidden_clock() -> Duration {
    panic!("untimed zero-height resolution must not read a clock")
}

fn usage_equal(actual: &PaintReport, expected: &PaintReport) {
    for &work in WorkKind::ALL {
        assert_eq!(actual.work(work), expected.work(work), "{work:?}");
    }
    for &resource in ResourceKind::ALL {
        assert_eq!(actual.live(resource), expected.live(resource));
        assert_eq!(actual.peak(resource), expected.peak(resource));
    }
    assert_eq!(actual.phases().len(), expected.phases().len());
    for (actual, expected) in actual.phases().zip(expected.phases()) {
        assert_eq!(actual.phase(), expected.phase());
        for &work in WorkKind::ALL {
            assert_eq!(actual.work(work), expected.work(work));
        }
        for &resource in ResourceKind::ALL {
            assert_eq!(actual.live(resource), expected.live(resource));
            assert_eq!(actual.peak(resource), expected.peak(resource));
        }
    }
}

#[test]
fn actual_work_and_capacity_caps_clean_partial_measures_and_profiles_preserve_admission() {
    let source = [at(0.0, 0.0), omitted(), hint(0.0, 0.75), at(0.0, 1.0)];
    let geometry = geometry(20.0, 0.0);
    let baseline =
        resolve_zero_height_gradient_colors(&geometry, &source, GradientRepeat::Repeat, options());
    let expected = observed(&baseline.result.unwrap());
    let report = baseline.report.unwrap();
    for profile in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 2,
            filter: EventFilter::ScopeName("gradient_stop_distribution"),
        },
    ] {
        let timed = matches!(profile, ProfileMode::Summary { timing: true });
        let options = PaintOptions::new(PaintLimits::unlimited(), profile).with_clock(if timed {
            clock
        } else {
            forbidden_clock
        });
        let actual = resolve_zero_height_gradient_colors(
            &geometry,
            &source,
            GradientRepeat::Repeat,
            options,
        );
        assert_eq!(observed(&actual.result.unwrap()), expected);
        if let Some(actual_report) = actual.report {
            usage_equal(&actual_report, &report);
            assert_eq!(actual_report.elapsed().is_some(), timed);
        }
        for work in [
            WorkKind::GradientValidation,
            WorkKind::GradientStopVisit,
            WorkKind::GradientCoordinate,
            WorkKind::OutputRecord,
        ] {
            let cap = report.work(work) - 1;
            let failed = resolve_zero_height_gradient_colors(
                &geometry,
                &source,
                GradientRepeat::Repeat,
                options.with_limits(PaintLimits::unlimited().with_work_limit(work, cap)),
            );
            let Err(PaintError::Budget(refusal)) = failed.result else {
                panic!("expected {work:?} cap")
            };
            assert!(matches!(refusal.change, PaintChange::Work { kind, .. } if kind == work));
            if let Some(failed_report) = failed.report {
                assert!(failed_report.work(work) <= cap);
                no_custody(&failed_report);
            }
            let retry = resolve_zero_height_gradient_colors(
                &geometry,
                &source,
                GradientRepeat::Repeat,
                options,
            );
            assert_eq!(observed(&retry.result.unwrap()), expected);
        }
        for &resource in ResourceKind::ALL {
            let peak = report.peak(resource);
            assert!(peak > 0);
            let failed = resolve_zero_height_gradient_colors(
                &geometry,
                &source,
                GradientRepeat::Repeat,
                options
                    .with_limits(PaintLimits::unlimited().with_resource_limit(resource, peak - 1)),
            );
            assert!(matches!(failed.result, Err(PaintError::Budget(_))));
            if let Some(failed_report) = failed.report {
                no_custody(&failed_report);
            }
        }
        let before_distribution: u64 = report
            .phases()
            .filter(|phase| {
                matches!(
                    phase.phase(),
                    PaintPhase::GradientValidation
                        | PaintPhase::GradientStopResolution
                        | PaintPhase::GradientStopFixup
                )
            })
            .map(|phase| phase.work(WorkKind::GradientCoordinate))
            .sum();
        let failed = resolve_zero_height_gradient_colors(
            &geometry,
            &source,
            GradientRepeat::Repeat,
            options.with_limits(
                PaintLimits::unlimited()
                    .with_work_limit(WorkKind::GradientCoordinate, before_distribution),
            ),
        );
        let Err(PaintError::Budget(refusal)) = failed.result else {
            panic!("expected actual distribution/weight work refusal")
        };
        assert_eq!(refusal.origin, PaintPhase::GradientStopDistribution);
        if let Some(report) = failed.report {
            no_custody(&report);
        }
    }
}
