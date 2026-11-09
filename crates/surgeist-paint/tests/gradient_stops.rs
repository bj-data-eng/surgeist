#![forbid(unsafe_code)]

use std::{cell::Cell, mem::size_of, ptr, rc::Rc, time::Duration};
use surgeist_layout::Size;
use surgeist_paint::{
    EventFilter, GradientPosition, GradientPositionComponent, GradientStopItem,
    GradientStopPositions, GradientStops, GradientStopsError, LinearGradientDirection, PaintChange,
    PaintError, PaintEvent, PaintLimits, PaintOptions, PaintPhase, PaintReport, ProfileMode,
    ResourceKind, UsedGradientItem, WorkKind, resolve_gradient_stops, resolve_linear_gradient,
};

fn options(limits: PaintLimits) -> PaintOptions {
    PaintOptions::new(limits, ProfileMode::Summary { timing: false })
}

fn position(length: f64, fraction: f64) -> GradientPosition {
    GradientPosition { length, fraction }
}

fn omitted() -> GradientStopItem<()> {
    GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::Omitted,
    }
}

fn at(length: f64, fraction: f64) -> GradientStopItem<()> {
    GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::One(position(length, fraction)),
    }
}

fn hint(length: f64, fraction: f64) -> GradientStopItem<()> {
    GradientStopItem::Hint {
        position: position(length, fraction),
    }
}

fn two(first: GradientPosition, second: GradientPosition) -> GradientStopItem<()> {
    GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::Two(first, second),
    }
}

fn resolved<C>(length: f64, source: &[GradientStopItem<C>]) -> GradientStops<'_, C> {
    resolve_gradient_stops(length, source, options(PaintLimits::unlimited()))
        .result
        .unwrap()
}

#[test]
fn opaque_two_and_hint_keep_identity_across_partial_and_repeated_iteration() {
    struct Payload(u8);
    let source = [
        GradientStopItem::Stop {
            payload: Payload(7),
            positions: GradientStopPositions::Two(position(10.0, 0.0), position(30.0, 0.0)),
        },
        GradientStopItem::Hint {
            position: position(50.0, 0.0),
        },
        GradientStopItem::Stop {
            payload: Payload(9),
            positions: GradientStopPositions::Omitted,
        },
    ];
    let GradientStopItem::Stop { payload: first, .. } = &source[0] else {
        unreachable!()
    };
    let GradientStopItem::Stop { payload: last, .. } = &source[2] else {
        unreachable!()
    };
    let stops = resolved(100.0, &source);
    assert!(ptr::eq(stops.source(), &source));
    let mut partial = stops.items();
    assert_eq!(partial.len(), 4);
    let Some(UsedGradientItem::Stop {
        source_index,
        position_index,
        payload,
        position,
    }) = partial.next()
    else {
        panic!("first occurrence must be a stop");
    };
    assert_eq!(
        (source_index, position_index, position, payload.0),
        (0, 0, 10.0, 7)
    );
    assert!(ptr::eq(payload, first));
    assert_eq!(partial.len(), 3);
    let mut independent = stops.items();
    assert_eq!(independent.len(), 4);
    assert!(matches!(
        independent.next(),
        Some(UsedGradientItem::Stop {
            position_index: 0,
            ..
        })
    ));
    let Some(UsedGradientItem::Stop {
        source_index,
        position_index,
        payload,
        position,
    }) = partial.next()
    else {
        panic!("second occurrence must be a stop");
    };
    assert_eq!((source_index, position_index, position), (0, 1, 30.0));
    assert!(ptr::eq(payload, first));
    assert_eq!(partial.len(), 2);
    assert!(matches!(
        partial.next(),
        Some(UsedGradientItem::Hint {
            source_index: 1,
            position: 50.0
        })
    ));
    assert_eq!(partial.len(), 1);
    let Some(UsedGradientItem::Stop {
        source_index,
        position_index,
        payload,
        position,
    }) = partial.next()
    else {
        panic!("last occurrence must be a stop");
    };
    assert_eq!(
        (source_index, position_index, position, payload.0),
        (2, 0, 100.0, 9)
    );
    assert!(ptr::eq(payload, last));
    for _ in 0..2 {
        assert_eq!(partial.len(), 0);
        assert!(partial.next().is_none());
    }
    assert_eq!(independent.count(), 3);
    assert_eq!(stops.items().count(), 4);
}

fn positions<C>(stops: &GradientStops<'_, C>) -> Vec<f64> {
    stops
        .items()
        .map(|item| match item {
            UsedGradientItem::Stop { position, .. } | UsedGradientItem::Hint { position, .. } => {
                position
            }
        })
        .collect()
}

#[derive(Debug, PartialEq)]
enum Observation {
    Stop {
        source: usize,
        ordinal: usize,
        position: f64,
    },
    Hint {
        source: usize,
        position: f64,
    },
}

fn observed<C>(stops: &GradientStops<'_, C>) -> Vec<Observation> {
    let mut items = stops.items();
    let expected_len = items.len();
    let result: Vec<_> = items
        .by_ref()
        .map(|item| match item {
            UsedGradientItem::Stop {
                source_index,
                position_index,
                position,
                ..
            } => Observation::Stop {
                source: source_index,
                ordinal: position_index,
                position,
            },
            UsedGradientItem::Hint {
                source_index,
                position,
            } => Observation::Hint {
                source: source_index,
                position,
            },
        })
        .collect();
    assert_eq!(result.len(), expected_len);
    assert_eq!(items.len(), 0);
    result
}

#[test]
fn endpoint_defaults_prefix_maxima_and_omission_runs_match_literal_fixup_examples() {
    // Pinned Images3 3.4.3 examples, in physical lengths at L=100.
    for (source, expected) in [
        (
            vec![omitted(), at(0.0, 0.2), omitted()],
            vec![0.0, 20.0, 100.0],
        ),
        (
            vec![at(0.0, 0.4), omitted(), omitted(), omitted()],
            vec![40.0, 60.0, 80.0, 100.0],
        ),
        (
            vec![at(0.0, -0.5), omitted(), omitted()],
            vec![-50.0, 25.0, 100.0],
        ),
        (
            vec![at(-50.0, 0.0), omitted(), omitted()],
            vec![-50.0, 25.0, 100.0],
        ),
        (
            vec![at(20.0, 0.0), at(0.0, 0.0), at(40.0, 0.0)],
            vec![20.0, 20.0, 40.0],
        ),
        (
            vec![omitted(), at(0.0, -0.5), at(0.0, 1.5), omitted()],
            vec![0.0, 0.0, 150.0, 150.0],
        ),
        (
            vec![at(80.0, 0.0), at(0.0, 0.0), omitted(), at(100.0, 0.0)],
            vec![80.0, 80.0, 90.0, 100.0],
        ),
        (
            vec![omitted(), omitted(), omitted(), omitted(), omitted()],
            vec![0.0, 25.0, 50.0, 75.0, 100.0],
        ),
    ] {
        assert_eq!(positions(&resolved(100.0, &source)), expected);
    }
}

#[test]
fn a_single_omitted_stop_uses_the_first_default_and_long_runs_remain_evenly_spaced() {
    let single = [omitted()];
    for length in [0.0, 100.0, f64::MAX] {
        let result = resolved(length, &single);
        assert_eq!(positions(&result), [0.0]);
        assert_eq!(result.line_length(), length);
        assert_eq!(
            observed(&result),
            [Observation::Stop {
                source: 0,
                ordinal: 0,
                position: 0.0
            }]
        );
    }
    let run = [
        at(0.0, 0.0),
        omitted(),
        omitted(),
        omitted(),
        omitted(),
        omitted(),
        at(96.0, 0.0),
    ];
    assert_eq!(
        positions(&resolved(100.0, &run)),
        [0.0, 16.0, 32.0, 48.0, 64.0, 80.0, 96.0]
    );
}

#[test]
fn positioned_hints_bound_omissions_on_both_sides_and_remain_distinct_items() {
    // The adopted CSSWG #3931 resolution makes a hint a distribution boundary.
    let source = [
        at(0.0, 0.0),
        omitted(),
        hint(0.0, 0.2),
        omitted(),
        at(0.0, 1.0),
    ];
    let result = resolved(100.0, &source);
    assert_eq!(
        observed(&result),
        [
            Observation::Stop {
                source: 0,
                ordinal: 0,
                position: 0.0
            },
            Observation::Stop {
                source: 1,
                ordinal: 0,
                position: 10.0
            },
            Observation::Hint {
                source: 2,
                position: 20.0
            },
            Observation::Stop {
                source: 3,
                ordinal: 0,
                position: 60.0
            },
            Observation::Stop {
                source: 4,
                ordinal: 0,
                position: 100.0
            },
        ]
    );
    for (source, expected) in [
        (
            [at(0.0, 0.0), hint(80.0, 0.0), at(20.0, 0.0)],
            [0.0, 80.0, 80.0],
        ),
        (
            [at(80.0, 0.0), hint(20.0, 0.0), at(100.0, 0.0)],
            [80.0, 80.0, 100.0],
        ),
        (
            [at(0.0, 0.0), hint(0.0, 0.0), at(0.0, 0.0)],
            [0.0, 0.0, 0.0],
        ),
    ] {
        let result = resolved(100.0, &source);
        assert_eq!(positions(&result), expected);
        assert!(matches!(
            result.items().nth(1),
            Some(UsedGradientItem::Hint {
                source_index: 1,
                ..
            })
        ));
        assert_eq!(result.items().len(), 3);
    }
}

#[test]
fn affine_resolution_precedes_ordering_and_depends_on_the_actual_line_length() {
    // The Images3 mixed-unit example changes ordering when layout changes L.
    let source = [at(100.0, 0.0), at(0.0, 0.5)];
    assert_eq!(positions(&resolved(150.0, &source)), [100.0, 100.0]);
    assert_eq!(positions(&resolved(300.0, &source)), [100.0, 150.0]);
    let signed = [at(-20.0, -0.5), at(10.0, 1.5)];
    assert_eq!(positions(&resolved(100.0, &signed)), [-70.0, 160.0]);
    let equal = [at(5.0, 0.0), omitted(), at(5.0, 0.0)];
    assert_eq!(positions(&resolved(100.0, &equal)), [5.0, 5.0, 5.0]);
}

struct OpaquePayload<'a> {
    label: &'a str,
    drops: Rc<Cell<usize>>,
}

impl Drop for OpaquePayload<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn double_positions_borrow_the_same_opaque_payload_with_source_and_position_ordinals() {
    // No Clone, Copy, Debug, Eq, Send or Sync implementation is available.
    let drops = Rc::new(Cell::new(0));
    let label = "original".to_owned();
    let source = [GradientStopItem::Stop {
        payload: OpaquePayload {
            label: label.as_str(),
            drops: Rc::clone(&drops),
        },
        positions: GradientStopPositions::Two(position(0.0, 0.2), position(0.0, 0.8)),
    }];
    let result = resolved(100.0, &source);
    assert!(ptr::eq(result.source(), source.as_slice()));
    let GradientStopItem::Stop {
        payload: original, ..
    } = &source[0]
    else {
        unreachable!()
    };
    for (ordinal, item) in result.items().enumerate() {
        let UsedGradientItem::Stop {
            source_index,
            position_index,
            payload,
            position,
        } = item
        else {
            panic!("double positions must remain stops")
        };
        assert_eq!(source_index, 0);
        assert_eq!(position_index, ordinal);
        assert!(ptr::eq(payload, original));
        assert_eq!(payload.label, "original");
        assert_eq!(position, [20.0, 80.0][ordinal]);
    }
    assert_eq!(result.items().len(), 2);
    drop(result);
    assert_eq!(drops.get(), 0);
    let failed = resolve_gradient_stops(
        100.0,
        &source,
        options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 0)),
    );
    assert!(matches!(failed.result, Err(PaintError::Budget(_))));
    assert_eq!(drops.get(), 0);
    drop(failed);
    drop(source);
    assert_eq!(drops.get(), 1);
}

#[test]
fn zero_sized_payload_identity_is_the_source_slot_and_two_positions_keep_their_order() {
    let source = [
        two(position(80.0, 0.0), position(20.0, 0.0)),
        hint(80.0, 0.0),
        two(position(80.0, 0.0), position(120.0, 0.0)),
    ];
    assert_eq!(
        observed(&resolved(100.0, &source)),
        [
            Observation::Stop {
                source: 0,
                ordinal: 0,
                position: 80.0
            },
            Observation::Stop {
                source: 0,
                ordinal: 1,
                position: 80.0
            },
            Observation::Hint {
                source: 1,
                position: 80.0
            },
            Observation::Stop {
                source: 2,
                ordinal: 0,
                position: 80.0
            },
            Observation::Stop {
                source: 2,
                ordinal: 1,
                position: 120.0
            },
        ]
    );
}

#[test]
fn zero_length_preserves_absolutes_and_never_normalizes_or_divides_positions() {
    let source = [
        at(-10.0, 0.0),
        omitted(),
        hint(0.0, 0.5),
        omitted(),
        at(10.0, 0.0),
    ];
    assert_eq!(
        positions(&resolved(0.0, &source)),
        [-10.0, -5.0, 0.0, 5.0, 10.0]
    );
    let percentages = [at(0.0, -2.0), omitted(), at(0.0, 3.0)];
    assert_eq!(positions(&resolved(0.0, &percentages)), [0.0, 0.0, 0.0]);
    assert_eq!(positions(&resolved(-0.0, &percentages)), [0.0, 0.0, 0.0]);
    let negative = [at(-50.0, 0.0), omitted(), omitted()];
    assert_eq!(positions(&resolved(0.0, &negative)), [-50.0, -25.0, 0.0]);
}

#[test]
fn finite_extremes_allow_overflowing_spans_fused_cancellation_and_rounded_coincidences() {
    let span = [at(-f64::MAX, 0.0), omitted(), at(f64::MAX, 0.0)];
    assert_eq!(
        positions(&resolved(100.0, &span)),
        [-f64::MAX, 0.0, f64::MAX]
    );
    let endpoints = [at(-f64::MAX, 0.0), at(f64::MAX, 0.0)];
    assert_eq!(
        positions(&resolved(100.0, &endpoints)),
        [-f64::MAX, f64::MAX]
    );
    let cancellation = [at(-f64::MAX, 2.0)];
    assert_eq!(positions(&resolved(f64::MAX, &cancellation)), [f64::MAX]);
    let next = f64::from_bits(1.0_f64.to_bits() + 1);
    let rounded = [at(1.0, 0.0), omitted(), at(next, 0.0)];
    assert_eq!(positions(&resolved(100.0, &rounded)), [1.0, 1.0, next]);
    let tiny = [at(0.0, 0.0), omitted(), at(f64::from_bits(1), 0.0)];
    assert_eq!(
        positions(&resolved(0.0, &tiny)),
        [0.0, 0.0, f64::from_bits(1)]
    );
}

fn no_custody(report: &PaintReport) {
    for &resource in ResourceKind::ALL {
        assert_eq!(report.live(resource), 0, "{resource:?}");
    }
}

fn error<C>(length: f64, source: &[GradientStopItem<C>], expected: GradientStopsError) {
    let outcome = resolve_gradient_stops(length, source, options(PaintLimits::unlimited()));
    match outcome.result {
        Err(actual) => assert_eq!(actual, PaintError::GradientStops(expected)),
        Ok(_) => panic!("invalid input must not publish stops"),
    }
    no_custody(&outcome.report.unwrap());
}

#[test]
fn grammar_and_component_failures_follow_the_declared_validation_priority() {
    for invalid in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        error::<()>(invalid, &[], GradientStopsError::InvalidLineLength);
    }
    error::<()>(100.0, &[], GradientStopsError::EmptyList);
    error(
        100.0,
        &[hint(f64::NAN, f64::NAN)],
        GradientStopsError::EndpointHint { source_index: 0 },
    );
    error(
        100.0,
        &[at(0.0, 0.0), hint(f64::NAN, 0.0)],
        GradientStopsError::EndpointHint { source_index: 1 },
    );
    error(
        100.0,
        &[
            at(0.0, 0.0),
            hint(10.0, 0.0),
            hint(f64::NAN, 0.0),
            at(100.0, 0.0),
        ],
        GradientStopsError::AdjacentHints {
            previous_source_index: 1,
            source_index: 2,
        },
    );
    // Source order takes priority over a later endpoint grammar failure.
    error(
        100.0,
        &[at(f64::NAN, 0.0), hint(0.0, 0.0)],
        GradientStopsError::NonFiniteComponent {
            source_index: 0,
            position_index: 0,
            component: GradientPositionComponent::Length,
        },
    );
    error(
        100.0,
        &[two(
            position(f64::NAN, f64::INFINITY),
            position(f64::INFINITY, 0.0),
        )],
        GradientStopsError::NonFiniteComponent {
            source_index: 0,
            position_index: 0,
            component: GradientPositionComponent::Length,
        },
    );
    error(
        100.0,
        &[two(position(0.0, f64::NAN), position(f64::INFINITY, 0.0))],
        GradientStopsError::NonFiniteComponent {
            source_index: 0,
            position_index: 0,
            component: GradientPositionComponent::Fraction,
        },
    );
    error(
        100.0,
        &[two(position(0.0, 0.0), position(0.0, f64::INFINITY))],
        GradientStopsError::NonFiniteComponent {
            source_index: 0,
            position_index: 1,
            component: GradientPositionComponent::Fraction,
        },
    );
    error(
        100.0,
        &[at(0.0, 0.0), hint(0.0, f64::NAN), at(100.0, 0.0)],
        GradientStopsError::NonFiniteComponent {
            source_index: 1,
            position_index: 0,
            component: GradientPositionComponent::Fraction,
        },
    );
    // Component validation still runs at L=0; percentages cannot hide infinity.
    for invalid in [f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        error(
            0.0,
            &[at(0.0, invalid)],
            GradientStopsError::NonFiniteComponent {
                source_index: 0,
                position_index: 0,
                component: GradientPositionComponent::Fraction,
            },
        );
        error(
            0.0,
            &[at(invalid, 0.0)],
            GradientStopsError::NonFiniteComponent {
                source_index: 0,
                position_index: 0,
                component: GradientPositionComponent::Length,
            },
        );
    }
}

#[test]
fn affine_overflow_reports_its_source_and_expanded_ordinal_after_raw_validation() {
    let source = [
        at(0.0, 0.0),
        two(position(0.0, 0.0), position(f64::MAX, 2.0)),
        at(0.0, 1.0),
    ];
    error(
        f64::MAX,
        &source,
        GradientStopsError::UnrepresentablePosition {
            source_index: 1,
            position_index: 1,
        },
    );
    let hint_overflow = [at(0.0, 0.0), hint(f64::MAX, 2.0), at(0.0, 1.0)];
    error(
        f64::MAX,
        &hint_overflow,
        GradientStopsError::UnrepresentablePosition {
            source_index: 1,
            position_index: 0,
        },
    );
    let invalid_later = [at(f64::MAX, 2.0), at(0.0, f64::NAN)];
    error(
        f64::MAX,
        &invalid_later,
        GradientStopsError::NonFiniteComponent {
            source_index: 1,
            position_index: 0,
            component: GradientPositionComponent::Fraction,
        },
    );
}

#[test]
fn numeric_failure_precedes_prepared_allocation_and_both_buffers_retry_cleanly() {
    let overflow = [at(f64::MAX, 2.0)];
    let failed = resolve_gradient_stops(
        f64::MAX,
        &overflow,
        options(PaintLimits::unlimited().with_resource_limit(ResourceKind::RetainedResultBytes, 0)),
    );
    assert!(matches!(
        failed.result,
        Err(PaintError::GradientStops(
            GradientStopsError::UnrepresentablePosition {
                source_index: 0,
                position_index: 0
            }
        ))
    ));
    no_custody(&failed.report.unwrap());

    let source = accounting_fixture();
    let baseline = resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
    let expected = observed(&baseline.result.unwrap());
    let report = baseline.report.unwrap();
    let scratch = report.peak(ResourceKind::ScratchBytes);
    let retained = report.peak(ResourceKind::RetainedResultBytes);
    assert!(scratch > 0);
    for limits in [
        PaintLimits::unlimited().with_resource_limit(ResourceKind::ScratchBytes, 0),
        PaintLimits::unlimited().with_resource_limit(ResourceKind::RetainedResultBytes, 0),
        // Seven staging records precede preparation; refuse its second emission.
        PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 8),
        // All fourteen records are emitted; refuse the final fixed result emission.
        PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 14),
    ] {
        let failed = resolve_gradient_stops(100.0, &source, options(limits));
        assert!(matches!(failed.result, Err(PaintError::Budget(_))));
        no_custody(&failed.report.unwrap());
        let retry = resolve_gradient_stops(
            100.0,
            &source,
            options(
                PaintLimits::unlimited()
                    .with_resource_limit(ResourceKind::ScratchBytes, scratch)
                    .with_resource_limit(ResourceKind::RetainedResultBytes, retained),
            ),
        );
        assert_eq!(observed(&retry.result.unwrap()), expected);
        assert_eq!(retry.report.unwrap().live(ResourceKind::ScratchBytes), 0);
    }
}

#[test]
fn layout_size_and_real_linear_geometry_supply_the_used_stop_length() {
    let geometry = resolve_linear_gradient(
        Size::<f64>::new(200.0, 100.0),
        LinearGradientDirection::Top,
        options(PaintLimits::unlimited()),
    )
    .result
    .unwrap();
    assert_eq!(geometry.length(), 100.0);
    let source = [omitted(), at(10.0, 0.25), omitted()];
    let result = resolved(geometry.length(), &source);
    assert_eq!(result.line_length(), geometry.length());
    assert_eq!(positions(&result), [0.0, 35.0, 100.0]);
}

fn accounting_fixture() -> [GradientStopItem<()>; 6] {
    [
        at(0.0, 0.0),
        omitted(),
        hint(20.0, 0.0),
        two(position(80.0, 0.0), position(40.0, 0.0)),
        omitted(),
        at(120.0, 0.0),
    ]
}

const PHASES: [PaintPhase; 6] = [
    PaintPhase::GradientValidation,
    PaintPhase::GradientStopResolution,
    PaintPhase::GradientStopFixup,
    PaintPhase::GradientStopDistribution,
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
fn reports_charge_expanded_records_and_exclude_caller_payload_storage() {
    let source = accounting_fixture();
    let outcome = resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
    assert_eq!(
        positions(&outcome.result.unwrap()),
        [0.0, 10.0, 20.0, 80.0, 80.0, 100.0, 120.0]
    );
    let report = outcome.report.unwrap();
    // Seven numeric staging emissions, seven prepared emissions, one owner.
    assert_eq!(report.work(WorkKind::OutputRecord), 15);
    for kind in [
        WorkKind::GradientValidation,
        WorkKind::GradientStopVisit,
        WorkKind::GradientCoordinate,
    ] {
        assert!(report.work(kind) > 0);
    }
    assert!(
        report.live(ResourceKind::RetainedResultBytes) > size_of::<GradientStops<'_, ()>>() as u64
    );
    assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
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
    let small = [GradientStopItem::Stop {
        payload: (),
        positions: GradientStopPositions::Two(position(20.0, 0.0), position(80.0, 0.0)),
    }];
    let large = [GradientStopItem::Stop {
        payload: [7_u8; 8192],
        positions: GradientStopPositions::Two(position(20.0, 0.0), position(80.0, 0.0)),
    }];
    let small_outcome = resolve_gradient_stops(100.0, &small, options(PaintLimits::unlimited()));
    let large_outcome = resolve_gradient_stops(100.0, &large, options(PaintLimits::unlimited()));
    assert_eq!(
        observed(&small_outcome.result.unwrap()),
        observed(&large_outcome.result.unwrap())
    );
    usage_equal(
        &small_outcome.report.unwrap(),
        &large_outcome.report.unwrap(),
    );
}

#[test]
fn work_and_result_refusals_release_every_admitted_byte_and_allow_a_clean_retry() {
    let source = accounting_fixture();
    let baseline = resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
    let expected = observed(&baseline.result.unwrap());
    let report = baseline.report.unwrap();
    for kind in [
        WorkKind::GradientValidation,
        WorkKind::GradientStopVisit,
        WorkKind::GradientCoordinate,
        WorkKind::OutputRecord,
    ] {
        let outcome = resolve_gradient_stops(
            100.0,
            &source,
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
    }
    // A measured accepted prefix locates admission at each semantic stage;
    // no literal helper/comparison counts become part of the public oracle.
    for (index, phase) in PHASES.into_iter().enumerate().take(5) {
        let kind = [
            WorkKind::GradientValidation,
            WorkKind::GradientStopVisit,
            WorkKind::GradientCoordinate,
            WorkKind::OutputRecord,
        ]
        .into_iter()
        .find(|&kind| phase_work(&report, phase, kind) > 0)
        .unwrap();
        let prefix: u64 = PHASES[..index]
            .iter()
            .map(|&earlier| phase_work(&report, earlier, kind))
            .sum();
        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            options(PaintLimits::unlimited().with_work_limit(kind, prefix)),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("expected refusal in {phase:?}")
        };
        assert_eq!(refusal.origin, phase);
        assert_eq!(refusal.limiting_phase, PaintPhase::Operation);
        no_custody(&outcome.report.unwrap());
    }
    let retained = report.peak(ResourceKind::RetainedResultBytes);
    for limit in [
        0,
        retained - 1,
        retained - size_of::<GradientStops<'_, ()>>() as u64,
    ] {
        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            options(
                PaintLimits::unlimited()
                    .with_resource_limit(ResourceKind::RetainedResultBytes, limit),
            ),
        );
        assert!(matches!(outcome.result, Err(PaintError::Budget(_))));
        no_custody(&outcome.report.unwrap());
    }
    let exact = resolve_gradient_stops(
        100.0,
        &source,
        options(
            PaintLimits::unlimited()
                .with_resource_limit(ResourceKind::RetainedResultBytes, retained),
        ),
    );
    assert_eq!(observed(&exact.result.unwrap()), expected);
    no_custody(
        &resolve_gradient_stops(f64::NAN, &source, options(PaintLimits::unlimited()))
            .report
            .unwrap(),
    );
    assert_eq!(observed(&resolved(100.0, &source)), expected);
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
fn profiling_timing_and_bounded_filtered_events_preserve_results_and_admission() {
    let source = accounting_fixture();
    CLOCK_READS.with(|reads| reads.set(0));
    let baseline = resolve_gradient_stops(
        100.0,
        &source,
        PaintOptions::new(
            PaintLimits::unlimited(),
            ProfileMode::Summary { timing: true },
        )
        .with_clock(clock),
    );
    let expected = observed(&baseline.result.unwrap());
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
    let limits = PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 0);
    let baseline_failure = resolve_gradient_stops(
        100.0,
        &source,
        PaintOptions::new(limits, ProfileMode::Summary { timing: true }).with_clock(clock),
    );
    let Err(expected_error) = baseline_failure.result else {
        panic!("expected work refusal")
    };
    let failure_report = baseline_failure.report.unwrap();
    no_custody(&failure_report);
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
            filter: EventFilter::ScopeName("gradient_stop_distribution"),
        },
    ] {
        let timing = matches!(profile, ProfileMode::Detailed { timing: true, .. });
        let selected_clock = if timing { clock } else { forbidden_clock };
        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            PaintOptions::new(PaintLimits::unlimited(), profile).with_clock(selected_clock),
        );
        assert_eq!(observed(&outcome.result.unwrap()), expected);
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
                        assert_eq!(origin, PaintPhase::GradientStopDistribution);
                    }
                }
            }
        }
        let failed = resolve_gradient_stops(
            100.0,
            &source,
            PaintOptions::new(limits, profile).with_clock(selected_clock),
        );
        match failed.result {
            Err(actual) => assert_eq!(actual, expected_error),
            Ok(_) => panic!("profiling must preserve admission refusal"),
        }
        if let Some(actual) = failed.report {
            usage_equal(&actual, &failure_report);
            no_custody(&actual);
        }
    }
}
