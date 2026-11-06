#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    path::Path,
    rc::Rc,
    time::Duration,
};

use surgeist_js::{
    Context, JsError, JsResult, JsValue, NativeFunction, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ExecutionSetupError, ProfileMode},
    job::{Job, JobExecutor},
    js_string,
};

#[test]
fn an_empty_script_admits_its_return_check_and_return() {
    let mut context = Context::default();

    // The empty script emits only CheckReturn and Return.
    let denied = context
        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(1))
        .unwrap();
    assert_eq!(denied.report.instructions(), 1);
    assert_eq!(
        denied.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 1,
        }))
    );

    let exact = context
        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(2))
        .unwrap();
    assert_eq!(exact.report.instructions(), 2);
    assert_eq!(exact.result.unwrap(), JsValue::undefined());
}

#[test]
fn a_host_stop_preserves_prior_effects_and_bypasses_guest_handlers() {
    let mut context = Context::default();
    let source = "globalThis.started = true;
        try { while (true) {} }
        catch { globalThis.caught = true; }
        finally { globalThis.finalized = true; }";
    let outcome = context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(outcome.report.instructions(), 64);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 64,
        }))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "started === true && typeof caught === 'undefined' && typeof finalized === 'undefined'",
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn reentrant_evaluation_cannot_hide_the_first_instruction_stop() {
    let mut context = Context::default();
    context
        .register_global_builtin_callable(
            js_string!("reenter"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                assert_eq!(
                    context
                        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(200))
                        .unwrap_err(),
                    ExecutionSetupError::AlreadyActive
                );
                let nested = context.eval(
                    Source::from_bytes("while (true) {}").with_path(Path::new("ledger-inner.js")),
                );
                assert_eq!(
                    nested.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 32,
                    }))
                );
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes("reenter(); globalThis.after = true;")
                .with_path(Path::new("ledger-outer.js")),
            ExecutionOptions::new(32),
        )
        .unwrap();
    assert_eq!(outcome.report.instructions(), 32);
    let error = outcome.result.unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 32,
        }))
    );
    assert!(error.to_string().contains("ledger-inner.js"), "{error}");
    assert_eq!(
        context
            .eval(Source::from_bytes("typeof after === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn a_nested_legacy_stop_preserves_the_callers_pending_throw() {
    let mut context = Context::default();
    context.runtime_limits_mut().set_loop_iteration_limit(0);
    context
        .register_global_builtin_callable(
            js_string!("swallowStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                let stop = context
                    .eval(Source::from_bytes("while (true) {}"))
                    .unwrap_err();
                assert_eq!(
                    stop.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::LoopIteration))
                );
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();

    let error = context
        .eval(Source::from_bytes(
            "try { throw 1; } catch { throw 2; } finally { swallowStop(); }",
        ))
        .unwrap_err();
    assert_eq!(error.as_opaque(), Some(&JsValue::new(2)));
}

#[derive(Default)]
struct InlinePromiseExecutor {
    error: RefCell<Option<JsError>>,
}

impl JobExecutor for InlinePromiseExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        let Job::PromiseJob(job) = job else {
            panic!("unexpected job kind");
        };
        if let Err(error) = job.call(context) {
            self.error.replace(Some(error));
        }
    }

    fn run_jobs(self: Rc<Self>, _: &mut Context) -> JsResult<()> {
        Ok(())
    }
}

#[test]
fn an_inline_await_stop_reaches_the_executor_and_releases_the_generator() {
    let executor = Rc::new(InlinePromiseExecutor::default());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes(
            "const generator = (async function* () { await 0; while (true) {} })();",
        ))
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes("generator.next()"),
            ExecutionOptions::new(256),
        )
        .unwrap();
    let expected = EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 256 });
    assert_eq!(outcome.result.unwrap_err().as_engine(), Some(&expected));
    assert_eq!(
        executor
            .error
            .borrow()
            .as_ref()
            .and_then(JsError::as_engine),
        Some(&expected)
    );
    context
        .eval(Source::from_bytes(
            "generator.next().then(result => { globalThis.finished = result.done; });",
        ))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("globalThis.finished"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn bounded_profiling_preserves_admission_and_reports_trace_loss() {
    let mut context = Context::default();
    let source = Source::from_bytes("");
    let outcome = context
        .eval_with_budget(
            source,
            ExecutionOptions::new(2).with_profile(ProfileMode::Detailed {
                timing: false,
                event_limit: 0,
                filter: EventFilter::All,
            }),
        )
        .unwrap();

    assert_eq!(outcome.result.unwrap(), JsValue::undefined());
    assert_eq!(outcome.report.instructions(), 2);
    let profile = outcome.report.profile().unwrap();
    assert_eq!(profile.dropped_events, 2);
    assert!(profile.events.is_empty());
    assert_eq!(profile.elapsed, None);
}

#[test]
fn the_configured_clock_runs_only_for_timed_profiling() {
    thread_local! {
        static TICKS: Cell<u64> = const { Cell::new(0) };
    }
    fn clock() -> Duration {
        TICKS.with(|ticks| {
            let elapsed = Duration::from_millis(ticks.get());
            ticks.set(ticks.get() + 1);
            elapsed
        })
    }

    let mut context = Context::default();
    for timing in [false, true] {
        TICKS.set(0);
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(""),
                ExecutionOptions::new(2)
                    .with_profile(ProfileMode::Summary { timing })
                    .with_profile_clock(clock),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::undefined());
        assert_eq!(outcome.report.instructions(), 2);
        assert_eq!(
            outcome.report.profile().unwrap().elapsed,
            timing.then_some(Duration::from_millis(1))
        );
        assert_eq!(TICKS.get(), if timing { 2 } else { 0 });
    }
}

#[test]
fn an_instruction_stop_preserves_an_async_generator_return_already_given_to_a_job() {
    // Sweep the checkpoints of this small call so the fixture exercises the
    // stop between closing the generator and returning from its VM frame.
    for limit in 0..128 {
        let mut context = Context::default();
        context
            .eval(Source::from_bytes(
                "globalThis.generator = (async function* () {
                    generator.return(7).then(result => {
                        globalThis.drained = result.done && result.value === 7;
                    });
                })();",
            ))
            .unwrap();

        let outcome = context
            .eval_with_budget(
                Source::from_bytes("generator.next()"),
                ExecutionOptions::new(limit),
            )
            .unwrap();
        if outcome.result.is_ok() {
            break;
        }
        context.run_jobs().unwrap();
        let drained = context
            .eval(Source::from_bytes("globalThis.drained === true"))
            .unwrap();
        if drained == JsValue::new(true) {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit,
                }))
            );
            return;
        }
    }
    panic!("no stopped evaluation preserved an already scheduled generator return");
}

#[test]
fn an_instruction_stop_before_async_return_registration_discards_the_unowned_queue() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "const value = Promise.resolve(7);
             Object.defineProperty(value, 'constructor', { get() { while (true) {} } });
             const generator = (async function* () { generator.return(value); })();",
        ))
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes("generator.next()"),
            ExecutionOptions::new(256),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 256,
        }))
    );

    context
        .eval(Source::from_bytes(
            "generator.next().then(result => { globalThis.finished = result.done; });",
        ))
        .unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("globalThis.finished"))
            .unwrap(),
        JsValue::new(true)
    );
}
