//! Whole-record preservation through the public Paint integration front door.
use crate::{
    Color, RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
    RenderAccountingSetupError, RenderAttemptStatus, RenderLimits, RenderPhaseFilter,
    RenderProfile, RenderWork, prepare_repeating_linear_gradient,
};
use std::{cell::Cell, mem::size_of, time::Duration};
use surgeist_layout as layout;
use surgeist_paint as paint;

thread_local! { static CLOCK: Cell<u64> = const { Cell::new(0) }; }
fn clock() -> Duration {
    CLOCK.with(|clock| {
        let now = clock.get();
        clock.set(now + 1);
        Duration::from_nanos(now)
    })
}
fn forbidden_clock() -> Duration {
    panic!("untimed preparation must not read the host clock")
}

fn red() -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
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

fn line(length: f64) -> paint::LinearGradientLine {
    paint::resolve_linear_gradient(
        layout::Size::<f64>::new(length, 1.0),
        paint::LinearGradientDirection::Right,
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

fn context(limits: RenderLimits) -> RenderAccountingContext {
    RenderAccountingContext::new(
        RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Summary { timing: false }),
    )
}

#[test]
fn invalid_limit_and_report_setup_failure_preserve_the_previous_completed_attempt() {
    let geometry = line(8.0);
    let source = [at(0.0, red()), at(8.0, blue())];
    let used = stops(8.0, &source);
    let mut account = context(RenderLimits::new());
    prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    let previous = account.last_attempt().unwrap().snapshot_for_test();
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
    assert_eq!(account.last_attempt(), Some(&previous));
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
    assert_eq!(account.last_attempt(), Some(&previous));
    account.set_options(RenderAccountingOptions::new());
    prepare_repeating_linear_gradient(&geometry, &used, &mut account).unwrap();
    assert_eq!(
        account.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
}

#[test]
fn mandatory_head_hint_and_final_output_frontiers_preserve_reports_across_profiles() {
    use crate::{
        Gradient, GradientStop, RenderClock, RenderPhase, RenderQuantity, RenderResource,
        paint::GradientKind, prepare_linear_gradient,
    };
    let geometry = line(1.0);
    let source = [
        paint::GradientStopItem::Stop {
            payload: red(),
            positions: paint::GradientStopPositions::Two(position(-4.0, 0.0), position(0.0, 0.0)),
        },
        paint::GradientStopItem::Hint {
            position: position(2.0, 0.0),
        },
        paint::GradientStopItem::Stop {
            payload: blue(),
            positions: paint::GradientStopPositions::Two(position(4.0, 0.0), position(12.0, 0.0)),
        },
    ];
    let used = stops(1.0, &source);
    let mut account = context(RenderLimits::new());
    let expected = prepare_linear_gradient(&geometry, &used, &mut account).unwrap();
    let GradientKind::Linear { stops: backing, .. } = expected.kind() else {
        panic!("linear output")
    };
    let backing_bytes = (backing.capacity() * size_of::<GradientStop>()) as u64;
    let fixed_bytes = size_of::<Gradient>() as u64;
    let total_bytes = backing_bytes + fixed_bytes;
    let baseline = account.take_last_attempt().unwrap();
    let baseline = baseline.attempt().unwrap();
    assert_eq!(baseline.usage().work(RenderWork::GradientStopVisits), 10);
    // Linear geometry/basis8, five capture/divide/domain actions24, four pairs8.
    assert_eq!(baseline.usage().work(RenderWork::CoordinateVisits), 40);
    assert_eq!(baseline.usage().work(RenderWork::PaintVisits), 3);
    assert_eq!(
        baseline.usage().peak(RenderResource::RetainedCpuBytes),
        total_bytes
    );
    assert_eq!(
        baseline.usage().work(RenderWork::PreparedTransferredBytes),
        total_bytes
    );
    let mut failures = Vec::new();
    for cap in 0..10 {
        failures.push((
            RenderLimits::new()
                .with_work_limit(RenderWork::GradientStopVisits, cap)
                .unwrap(),
            RenderQuantity::Work(RenderWork::GradientStopVisits),
            cap,
            1,
        ));
    }
    for cap in [0, 7, 8, 12, 13, 18, 19, 25, 26, 32, 33, 39] {
        failures.push((
            RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, cap)
                .unwrap(),
            RenderQuantity::Work(RenderWork::CoordinateVisits),
            cap,
            1,
        ));
    }
    for cap in 0..3 {
        failures.push((
            RenderLimits::new()
                .with_work_limit(RenderWork::PaintVisits, cap)
                .unwrap(),
            RenderQuantity::Work(RenderWork::PaintVisits),
            cap,
            1,
        ));
    }
    for (cap, current, amount) in [
        (backing_bytes - 1, 0, backing_bytes),
        (backing_bytes, backing_bytes, fixed_bytes),
        (total_bytes - 1, backing_bytes, fixed_bytes),
    ] {
        failures.push((
            RenderLimits::new().with_retained_bytes(cap),
            RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
            current,
            amount,
        ));
    }
    let profiles = [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 2,
            filter: RenderPhaseFilter::Phase(RenderPhase::Coordinates),
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
    ];
    let mut refusal_oracles = Vec::new();
    for (profile_index, profile) in profiles.into_iter().enumerate() {
        let timed = matches!(profile, RenderProfile::Summary { timing: true });
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(if timed {
                clock
            } else {
                forbidden_clock
            }));
        account.set_options(options);
        assert_eq!(
            prepare_linear_gradient(&geometry, &used, &mut account).unwrap(),
            expected
        );
        let report = account.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage(), baseline.usage());
        assert_eq!(report.elapsed().is_some(), timed);
        for phase in [
            RenderPhase::Preparation,
            RenderPhase::Coordinates,
            RenderPhase::Output,
            RenderPhase::Cleanup,
        ] {
            assert_eq!(report.phase_usage(phase), baseline.phase_usage(phase));
        }
        for (index, &(limits, quantity, current, amount)) in failures.iter().enumerate() {
            account.set_options(options.with_limits(limits));
            let error = prepare_linear_gradient(&geometry, &used, &mut account).unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("real quota refusal")
            };
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(refusal.quantity(), quantity);
            assert_eq!(refusal.current(), current);
            assert_eq!(refusal.amount(), amount);
            let report = account.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
            assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
            assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
            let phase_usage = [
                RenderPhase::Preparation,
                RenderPhase::Coordinates,
                RenderPhase::Output,
                RenderPhase::Cleanup,
            ]
            .map(|phase| report.phase_usage(phase));
            if profile_index == 0 {
                refusal_oracles.push((
                    error.accounting_diagnostic().unwrap(),
                    report.usage(),
                    phase_usage,
                ));
            } else {
                let (expected_error, expected_usage, expected_phases) = &refusal_oracles[index];
                assert_eq!(error.accounting_diagnostic(), Some(*expected_error));
                assert_eq!(&report.usage(), expected_usage);
                assert_eq!(&phase_usage, expected_phases);
            }
            account.set_options(options);
            assert_eq!(
                prepare_linear_gradient(&geometry, &used, &mut account).unwrap(),
                expected
            );
        }
    }
}
