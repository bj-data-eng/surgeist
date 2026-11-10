#![allow(unused_crate_dependencies)]

use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll, Waker},
    time::Duration,
};

use surgeist_js::{
    Context, JsValue, NativeFunction, Script, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ProfileMode},
    js_string,
};

thread_local! {
    static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static CALLBACKS: Cell<usize> = const { Cell::new(0) };
    static CLOCK_READS: Cell<usize> = const { Cell::new(0) };
}

fn clock() -> Duration {
    CLOCK_READS.set(CLOCK_READS.get() + 1);
    TIME.get()
}

fn modes() -> [ProfileMode; 6] {
    [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::All,
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 64,
            filter: EventFilter::ScopeName("vm-active"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 2,
            filter: EventFilter::ScopeName("vm-active"),
        },
    ]
}

fn context_with_clock_callback() -> Context {
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("advanceClock"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                CALLBACKS.set(CALLBACKS.get() + 1);
                TIME.set(TIME.get() + Duration::from_millis(7));
                Ok(JsValue::new(41))
            }),
        )
        .unwrap();
    context
}

#[test]
fn profiled_evaluation_observes_vm_active_time_across_a_native_callback() {
    const SOURCE: &str = "advanceClock() + 1";
    TIME.set(Duration::ZERO);
    CALLBACKS.set(0);
    let control = context_with_clock_callback()
        .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    assert_eq!(CALLBACKS.get(), 1);
    let instructions = control.report.instructions().unwrap();
    assert!(instructions > 0 && instructions <= 64);

    TIME.set(Duration::ZERO);
    CALLBACKS.set(0);
    let outcome = context_with_clock_callback()
        .eval_with_budget(
            Source::from_bytes(SOURCE),
            ExecutionOptions::new(64)
                .with_profile(ProfileMode::Summary { timing: true })
                .with_profile_clock(clock),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_eq!(CALLBACKS.get(), 1);
    assert_eq!(outcome.report.instructions().unwrap(), instructions);
    assert!(outcome.report.effects_started());
    let profile = outcome.report.profile().unwrap().unwrap();
    assert_eq!(profile.elapsed, Some(Duration::from_millis(7)));
    // Only executing the registered callback advances time. Parsing and
    // compilation finish at zero, before this single synchronous VM slice.
    let active = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "vm-active")
        .expect("a profiled evaluation must expose VM active execution");
    assert_eq!(active.phases, 1);
    assert_eq!(active.phase_elapsed, Some(Duration::from_millis(7)));
}

#[test]
fn task_profiles_separate_vm_active_slices_scheduling_wait_and_inclusive_lifetime() {
    const SOURCE: &str = "advanceClock() + 1";
    let control = context_with_clock_callback()
        .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    let instructions = control.report.instructions().unwrap();
    for mode in modes() {
        TIME.set(Duration::ZERO);
        CALLBACKS.set(0);
        CLOCK_READS.set(0);
        let mut context = context_with_clock_callback();
        let mut task = context
            .execution_task(
                ExecutionOptions::new(64)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                async |mut context| {
                    let script =
                        Script::parse(Source::from_bytes(SOURCE), None, context.reborrow())?;
                    script
                        .evaluate_async_with_budget(context.reborrow(), 1)
                        .await
                },
            )
            .unwrap();
        // Setup has happened, but the callback and lifetime observation have not.
        // Each real Pending interval below contributes exactly 100ms of host wait.
        TIME.set(Duration::from_millis(10));
        let mut completed = None;
        for pending_polls in 0..128 {
            match Pin::new(&mut task).poll(&mut TaskContext::from_waker(Waker::noop())) {
                Poll::Ready(outcome) => {
                    completed = Some((outcome, pending_polls));
                    break;
                }
                Poll::Pending => TIME.set(TIME.get() + Duration::from_millis(100)),
            }
        }
        let (outcome, pending_polls) =
            completed.expect("script did not complete within 128 cooperative polls");
        assert!(pending_polls > 1);
        assert_eq!(outcome.result.unwrap(), JsValue::new(42));
        assert_eq!(CALLBACKS.get(), 1);
        assert_eq!(outcome.report.instructions().unwrap(), instructions);
        if let ProfileMode::Off = mode {
            assert!(outcome.report.profile().unwrap().is_none());
            assert_eq!(CLOCK_READS.get(), 0);
            continue;
        }
        let timed = matches!(
            mode,
            ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
        );
        let wait = Duration::from_millis(100 * pending_polls as u64);
        let profile = outcome.report.profile().unwrap().unwrap();
        assert_eq!(
            profile.elapsed,
            timed.then_some(Duration::from_millis(17) + wait)
        );
        for (name, phases, elapsed) in [
            ("execution-lifetime", 1, Duration::from_millis(7) + wait),
            (
                "vm-active",
                pending_polls as u64 + 1,
                Duration::from_millis(7),
            ),
            ("vm-waiting", pending_polls as u64, wait),
        ] {
            let scope = profile
                .scopes
                .iter()
                .find(|scope| scope.name == name)
                .unwrap();
            assert_eq!(scope.phases, phases, "{name}");
            assert_eq!(scope.phase_elapsed, timed.then_some(elapsed), "{name}");
        }
        if let ProfileMode::Detailed {
            timing: true,
            event_limit,
            ..
        } = mode
        {
            let active_slices = pending_polls + 1;
            assert_eq!(profile.events.len(), active_slices.min(event_limit));
            assert_eq!(
                profile.dropped_events,
                active_slices.saturating_sub(event_limit) as u64
            );
            // Filtering suppresses frontend, lifetime, wait and instruction
            // events without counting them as lost selected detail.
            assert!(profile.events.iter().all(|event| matches!(event, surgeist_ledger::Event::Phase { scope, .. } if profile.scopes.iter().any(|report| report.scope == *scope && report.name == "vm-active"))));
        }
        if !timed {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}

#[test]
fn every_profile_mode_preserves_async_first_refusal_and_clock_silence_when_untimed() {
    const SOURCE: &str = "advanceClock() + 1";
    let control = context_with_clock_callback()
        .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    let limit = control.report.instructions().unwrap() - 1;
    for mode in modes() {
        TIME.set(Duration::ZERO);
        CALLBACKS.set(0);
        CLOCK_READS.set(0);
        let mut context = context_with_clock_callback();
        let mut task = context
            .execution_task(
                ExecutionOptions::new(limit)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                async |mut context| {
                    let script =
                        Script::parse(Source::from_bytes(SOURCE), None, context.reborrow())?;
                    script
                        .evaluate_async_with_budget(context.reborrow(), 1)
                        .await
                },
            )
            .unwrap();
        let mut completed = None;
        for _ in 0..128 {
            if let Poll::Ready(outcome) =
                Pin::new(&mut task).poll(&mut TaskContext::from_waker(Waker::noop()))
            {
                completed = Some(outcome);
                break;
            }
        }
        let outcome =
            completed.expect("limited script did not finish within 128 cooperative polls");
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                limit
            }))
        );
        assert_eq!(outcome.report.instructions().unwrap(), limit);
        assert_eq!(CALLBACKS.get(), 1);
        if !matches!(
            mode,
            ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
        ) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}
