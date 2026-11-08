#![allow(unused_crate_dependencies)]

use std::path::Path;

use surgeist_js::{
    Context, JsError, JsObject, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    js_string,
    object::builtins::JsPromise,
};

const INTERRUPTING_LIMIT: u64 = 64;
const LATER_LIMIT: u64 = 100_000;

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .global_object()
        .set(js_string!("moduleEffects"), 0, true, context.access())
        .unwrap();
    context
}

fn linked_module(source: &str, path: &str, context: &mut Context) -> Module {
    // Parsing, loading and linking precede the selected evaluation operation.
    let module = Module::parse(
        Source::from_bytes(source).with_path(Path::new(path)),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = module.load(context.access()).unwrap();
    assert_eq!(
        loaded.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    module
}

fn effects(context: &mut Context) -> JsValue {
    context
        .global_object()
        .get(js_string!("moduleEffects"), context.access())
        .unwrap()
}

fn assert_instruction_stop(error: &JsError, limit: u64) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit
        }))
    );
}

fn stop_evaluation(module: &Module, context: &mut Context, limit: u64) {
    let outcome = context
        .with_execution(ExecutionOptions::new(limit), |access| {
            let result = module.evaluate(access);
            assert_instruction_stop(
                result
                    .as_ref()
                    .expect_err("entered evaluation must return its original instruction stop"),
                limit,
            );
            result
        })
        .unwrap();
    assert_instruction_stop(&outcome.result.unwrap_err(), limit);
    assert_eq!(outcome.report.instructions(), limit);
}

fn assert_later_evaluation_is_terminal(module: &Module, context: &mut Context) {
    let outcome = context
        .with_execution(ExecutionOptions::new(LATER_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    let error = outcome
        .result
        .expect_err("an interrupted instance must refuse a fresh higher-budget evaluation");
    assert!(error.as_engine().is_some());
    assert_eq!(outcome.report.instructions(), 0);
}

#[test]
fn synchronous_stop_preserves_its_committed_effect_and_refuses_higher_budget_reentry() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1;
         for (let step = 0; step < 1000; step++) {}
         export const completed = true;",
        "sync-interruption.js",
        &mut context,
    );

    stop_evaluation(&module, &mut context, INTERRUPTING_LIMIT);
    assert_eq!(effects(&mut context), 1.into());
    assert_later_evaluation_is_terminal(&module, &mut context);
    assert_eq!(
        effects(&mut context),
        1.into(),
        "evaluation must never replay"
    );
}

#[test]
fn zero_instruction_stop_after_evaluation_entry_cannot_be_retried_as_unstarted_work() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1; export const completed = true;",
        "zero-instruction-interruption.js",
        &mut context,
    );

    // There is no previously retained stop. Evaluate enters its lifecycle before
    // attempting to admit the body's first instruction.
    stop_evaluation(&module, &mut context, 0);
    assert_eq!(effects(&mut context), 0.into());
    assert_later_evaluation_is_terminal(&module, &mut context);
    assert_eq!(effects(&mut context), 0.into());
}

#[test]
fn initial_async_module_body_returns_its_original_typed_host_stop() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1;
         for (let step = 0; step < 1000; step++) {}
         await 0;",
        "initial-tla-interruption.js",
        &mut context,
    );

    stop_evaluation(&module, &mut context, INTERRUPTING_LIMIT);
    assert_eq!(effects(&mut context), 1.into());
    assert_later_evaluation_is_terminal(&module, &mut context);
    assert_eq!(effects(&mut context), 1.into());
}

#[test]
fn ordinary_guest_throw_keeps_the_same_cached_rejected_promise_and_reason() {
    let mut context = context();
    let reason = JsValue::from(JsObject::with_null_proto(context.runtime()));
    context
        .global_object()
        .set(
            js_string!("moduleGuestError"),
            reason.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let module = linked_module(
        "globalThis.moduleEffects += 1; throw moduleGuestError;",
        "ordinary-module-rejection.js",
        &mut context,
    );

    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.state(), PromiseState::Rejected(reason.clone()));
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(second.state(), PromiseState::Rejected(reason));
    assert_eq!(effects(&mut context), 1.into());
}

fn evaluate_with_fresh_budget(module: &Module, context: &mut Context) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(LATER_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap()
}

#[test]
fn stop_retained_before_evaluation_preserves_the_linked_instances_first_valid_attempt() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1; export const completed = true;",
        "pre-entry-refusal.js",
        &mut context,
    );
    let stopped = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let prior = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_instruction_stop(&prior, 0);
            module.evaluate(access)
        })
        .unwrap();
    assert_instruction_stop(&stopped.result.unwrap_err(), 0);
    assert_eq!(stopped.report.instructions(), 0);
    assert_eq!(effects(&mut context), 0.into());

    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.state(), PromiseState::Fulfilled(JsValue::undefined()));
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(effects(&mut context), 1.into());
}

#[test]
fn completed_success_is_preserved_after_an_unrelated_operations_stop() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1; export const completed = true;",
        "completed-before-unrelated-stop.js",
        &mut context,
    );
    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.state(), PromiseState::Fulfilled(JsValue::undefined()));

    let unrelated = context
        .eval_with_budget(Source::from_bytes("0"), ExecutionOptions::new(0))
        .unwrap();
    assert_instruction_stop(&unrelated.result.unwrap_err(), 0);
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        second.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(effects(&mut context), 1.into());
}

mod continuation {
    use std::{
        cell::RefCell,
        collections::VecDeque,
        future::Future,
        pin::Pin,
        rc::Rc,
        sync::{Arc, Condvar, Mutex},
        task::{Context as TaskContext, Poll, Wake, Waker},
        time::{Duration, Instant},
    };

    use surgeist_js::{
        Context, ContextAccess, JsResult, JsValue, Module, Source,
        builtins::promise::{PromiseState, ResolvingFunctions},
        execution::{ExecutionOptions, ExecutionOutcome},
        job::{Job, JobCall, JobExecutor},
        js_string,
        object::builtins::{JsFuture, JsPromise},
    };

    use super::{
        INTERRUPTING_LIMIT, LATER_LIMIT, assert_instruction_stop,
        assert_later_evaluation_is_terminal, effects, evaluate_with_fresh_budget, linked_module,
    };

    // Scheduling stays explicit, while callback custody and dispatch retain the
    // real public job wrappers. Queue borrows never span a callback or wake.
    #[derive(Default)]
    struct ManualExecutor {
        jobs: RefCell<VecDeque<Job>>,
    }

    impl JobExecutor for ManualExecutor {
        fn enqueue_job(self: Rc<Self>, job: Job, _: ContextAccess<'_>) {
            self.jobs.borrow_mut().push_back(job);
        }

        fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
            panic!("the fixture dispatches selected jobs explicitly")
        }
    }

    impl ManualExecutor {
        fn run_next(
            &self,
            context: &mut Context,
            limit: u64,
        ) -> (JsResult<JsValue>, ExecutionOutcome<JsValue>) {
            let queued = self.jobs.borrow_mut().pop_front();
            let Some(Job::PromiseJob(job)) = queued else {
                panic!("the pending await must supply a real promise reaction: {queued:?}")
            };
            let mut direct = None;
            let outcome = context
                .with_execution(ExecutionOptions::new(limit), |access| {
                    match job.call(access) {
                        JobCall::Complete(result) => {
                            direct = Some(result.clone());
                            result
                        }
                        JobCall::Deferred { job, error } => {
                            self.jobs.borrow_mut().push_front(Job::PromiseJob(job));
                            Err(error)
                        }
                    }
                })
                .unwrap();
            (
                direct.expect("the selected continuation must be admitted rather than deferred"),
                outcome,
            )
        }
    }

    #[derive(Default)]
    struct WakeCount {
        count: Mutex<usize>,
        changed: Condvar,
    }

    impl WakeCount {
        fn record(&self) {
            *self.count.lock().unwrap() += 1;
            self.changed.notify_all();
        }

        fn await_notification(&self) {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut count = self.count.lock().unwrap();
            while *count == 0 {
                let remaining = deadline.saturating_duration_since(Instant::now());
                assert!(
                    !remaining.is_zero(),
                    "the original native module waiter received no interruption wake within 5 seconds; last count={}",
                    *count
                );
                (count, _) = self.changed.wait_timeout(count, remaining).unwrap();
            }
        }
    }

    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.record();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.record();
        }
    }

    struct NativeWait {
        future: Pin<Box<JsFuture>>,
        wakes: Arc<WakeCount>,
    }

    impl NativeWait {
        fn subscribe(promise: &JsPromise, context: &mut Context) -> Self {
            let mut wait = Self {
                future: Box::pin(promise.clone().into_js_future(context.access()).unwrap()),
                wakes: Arc::new(WakeCount::default()),
            };
            wait.assert_pending();
            wait
        }

        fn poll(&mut self) -> Poll<JsResult<JsValue>> {
            let waker = Waker::from(self.wakes.clone());
            self.future
                .as_mut()
                .poll(&mut TaskContext::from_waker(&waker))
        }

        fn assert_pending(&mut self) {
            assert!(
                self.poll().is_pending(),
                "the module is genuinely suspended"
            );
            // Only a notification after this rearmed poll can establish that
            // the impending interruption woke this original native waiter.
            *self.wakes.count.lock().unwrap() = 0;
        }

        fn assert_interrupted(&mut self) {
            self.wakes.await_notification();
            let Poll::Ready(Err(error)) = self.poll() else {
                panic!("the original native waiter must promptly return the module's host error")
            };
            assert!(error.as_engine().is_some(), "{error:?}");
        }

        fn assert_fulfilled(&mut self) {
            let Poll::Ready(result) = self.poll() else {
                panic!("ordinary module settlement must complete its native waiter")
            };
            assert_eq!(result.unwrap(), JsValue::undefined());
        }
    }

    struct SuspendedModule {
        context: Context,
        executor: Rc<ManualExecutor>,
        module: Module,
        gate: JsPromise,
        resolvers: ResolvingFunctions,
        evaluation: JsPromise,
        wait: NativeWait,
    }

    impl SuspendedModule {
        fn new(source: &str, path: &str) -> Self {
            let executor = Rc::new(ManualExecutor::default());
            let builder = Context::builder().job_executor(executor.clone());
            #[cfg(feature = "fuzz")]
            let builder = builder.instructions_remaining(1_000_000);
            let mut context = builder.build().unwrap();
            context
                .global_object()
                .set(js_string!("moduleEffects"), 0, true, context.access())
                .unwrap();
            let (gate, resolvers) = JsPromise::new_pending(context.access());
            context
                .global_object()
                .set(
                    js_string!("moduleGate"),
                    gate.clone(),
                    true,
                    context.access(),
                )
                .unwrap();
            let module = linked_module(source, path, &mut context);
            let evaluation = evaluate_with_fresh_budget(&module, &mut context);
            assert_eq!(evaluation.state(), PromiseState::Pending);
            assert_eq!(gate.state(), PromiseState::Pending);
            assert_eq!(effects(&mut context), 0.into());
            let wait = NativeWait::subscribe(&evaluation, &mut context);
            Self {
                context,
                executor,
                module,
                gate,
                resolvers,
                evaluation,
                wait,
            }
        }

        fn release_gate(&mut self) {
            self.resolvers
                .resolve
                .call(&JsValue::undefined(), &[7.into()], self.context.access())
                .unwrap();
            assert_eq!(self.gate.state(), PromiseState::Fulfilled(7.into()));
            assert_eq!(self.evaluation.state(), PromiseState::Pending);
            self.wait.assert_pending();
        }

        fn interrupt_resume(&mut self) {
            self.release_gate();
            let (direct, outcome) = self
                .executor
                .run_next(&mut self.context, INTERRUPTING_LIMIT);
            assert_instruction_stop(&direct.unwrap_err(), INTERRUPTING_LIMIT);
            assert_instruction_stop(&outcome.result.unwrap_err(), INTERRUPTING_LIMIT);
            assert_eq!(outcome.report.instructions(), INTERRUPTING_LIMIT);
            assert_eq!(effects(&mut self.context), 1.into());
            assert_eq!(self.evaluation.state(), PromiseState::Pending);
        }
    }

    const INTERRUPTED_SOURCE: &str = "await moduleGate;
         globalThis.moduleEffects += 1;
         for (let step = 0; step < 1000; step++) {}
         export const completed = true;";

    #[test]
    fn post_await_host_stop_preserves_pending_guest_state_and_refuses_native_reentry() {
        let mut fixture = SuspendedModule::new(INTERRUPTED_SOURCE, "post-await-interruption.js");
        fixture.interrupt_resume();
        assert_later_evaluation_is_terminal(&fixture.module, &mut fixture.context);
        assert_eq!(fixture.evaluation.state(), PromiseState::Pending);
        assert_eq!(effects(&mut fixture.context), 1.into());
    }

    #[test]
    fn native_waiter_registered_before_resume_is_not_stranded_by_the_host_stop() {
        let mut fixture = SuspendedModule::new(INTERRUPTED_SOURCE, "native-wait-interruption.js");
        fixture.interrupt_resume();
        fixture.wait.assert_interrupted();
        assert_eq!(fixture.evaluation.state(), PromiseState::Pending);
        assert_eq!(effects(&mut fixture.context), 1.into());
    }

    #[test]
    fn independently_suspended_module_survives_an_unrelated_operation_stop() {
        let mut fixture = SuspendedModule::new(
            "await moduleGate;
             globalThis.moduleEffects += 1;
             export const completed = true;",
            "independent-suspended-module.js",
        );
        let unrelated = fixture
            .context
            .eval_with_budget(Source::from_bytes("0"), ExecutionOptions::new(0))
            .unwrap();
        assert_instruction_stop(&unrelated.result.unwrap_err(), 0);
        assert_eq!(fixture.evaluation.state(), PromiseState::Pending);
        assert_eq!(effects(&mut fixture.context), 0.into());
        fixture.wait.assert_pending();

        fixture.release_gate();
        // Follow semantic settlement rather than assuming how many delivery
        // reactions the implementation needs. Every selected job has a finite
        // operation budget and the drain itself has a bounded action count.
        for _ in 0..8 {
            if fixture.evaluation.state() != PromiseState::Pending {
                break;
            }
            let (direct, outcome) = fixture.executor.run_next(&mut fixture.context, LATER_LIMIT);
            direct.unwrap();
            outcome.result.unwrap();
        }
        assert_eq!(
            fixture.evaluation.state(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        fixture.wait.assert_fulfilled();
        let cached = evaluate_with_fresh_budget(&fixture.module, &mut fixture.context);
        assert_eq!(cached.identity(), fixture.evaluation.identity());
        assert_eq!(effects(&mut fixture.context), 1.into());
    }
}
