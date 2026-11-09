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
        loaded.state().unwrap(),
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
    assert_eq!(outcome.report.instructions().unwrap(), limit);
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
    assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    assert_eq!(
        first.state().unwrap(),
        PromiseState::Rejected(reason.clone())
    );
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(second.state().unwrap(), PromiseState::Rejected(reason));
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
    assert_eq!(stopped.report.instructions().unwrap(), 0);
    assert_eq!(effects(&mut context), 0.into());

    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(
        first.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
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
    assert_eq!(
        first.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    let unrelated = context
        .eval_with_budget(Source::from_bytes("0"), ExecutionOptions::new(0))
        .unwrap();
    assert_instruction_stop(&unrelated.result.unwrap_err(), 0);
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        second.state().unwrap(),
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
            assert!(self.poll().is_pending(), "the module is suspended");
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
            assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
            assert_eq!(gate.state().unwrap(), PromiseState::Pending);
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
            assert_eq!(
                self.gate.state().unwrap(),
                PromiseState::Fulfilled(7.into())
            );
            assert_eq!(self.evaluation.state().unwrap(), PromiseState::Pending);
            self.wait.assert_pending();
        }

        fn interrupt_resume(&mut self) {
            self.release_gate();
            let (direct, outcome) = self
                .executor
                .run_next(&mut self.context, INTERRUPTING_LIMIT);
            assert_instruction_stop(&direct.unwrap_err(), INTERRUPTING_LIMIT);
            assert_instruction_stop(&outcome.result.unwrap_err(), INTERRUPTING_LIMIT);
            assert_eq!(outcome.report.instructions().unwrap(), INTERRUPTING_LIMIT);
            assert_eq!(effects(&mut self.context), 1.into());
            assert_eq!(self.evaluation.state().unwrap(), PromiseState::Pending);
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
        assert_eq!(fixture.evaluation.state().unwrap(), PromiseState::Pending);
        assert_eq!(effects(&mut fixture.context), 1.into());
    }

    #[test]
    fn native_waiter_registered_before_resume_is_not_stranded_by_the_host_stop() {
        let mut fixture = SuspendedModule::new(INTERRUPTED_SOURCE, "native-wait-interruption.js");
        fixture.interrupt_resume();
        fixture.wait.assert_interrupted();
        assert_eq!(fixture.evaluation.state().unwrap(), PromiseState::Pending);
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
        assert_eq!(fixture.evaluation.state().unwrap(), PromiseState::Pending);
        assert_eq!(effects(&mut fixture.context), 0.into());
        fixture.wait.assert_pending();

        fixture.release_gate();
        // Follow semantic settlement rather than assuming how many delivery
        // reactions the implementation needs. Every selected job has a finite
        // operation budget and the drain itself has a bounded action count.
        for _ in 0..8 {
            if fixture.evaluation.state().unwrap() != PromiseState::Pending {
                break;
            }
            let (direct, outcome) = fixture.executor.run_next(&mut fixture.context, LATER_LIMIT);
            direct.unwrap();
            outcome.result.unwrap();
        }
        assert_eq!(
            fixture.evaluation.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        fixture.wait.assert_fulfilled();
        let cached = evaluate_with_fresh_budget(&fixture.module, &mut fixture.context);
        assert_eq!(cached.identity(), fixture.evaluation.identity());
        assert_eq!(effects(&mut fixture.context), 1.into());
    }

    struct GraphFixture {
        context: Context,
        executor: Rc<ManualExecutor>,
        loader: Rc<surgeist_js::module::MapModuleLoader>,
    }

    impl GraphFixture {
        fn new() -> Self {
            let executor = Rc::new(ManualExecutor::default());
            let loader = Rc::new(surgeist_js::module::MapModuleLoader::new());
            let builder = Context::builder()
                .job_executor(executor.clone())
                .module_loader(loader.clone());
            #[cfg(feature = "fuzz")]
            let builder = builder.instructions_remaining(1_000_000);
            let mut context = builder.build().unwrap();
            for name in [
                "moduleEffects",
                "independentEffects",
                "parentEffects",
                "grandEffects",
                "importerEffects",
            ] {
                context
                    .global_object()
                    .set(js_string!(name), 0, true, context.access())
                    .unwrap();
            }
            Self {
                context,
                executor,
                loader,
            }
        }

        fn insert(&mut self, name: &str, source: &str) -> Module {
            let module = Module::parse(
                Source::from_bytes(source).with_path(std::path::Path::new(name)),
                None,
                self.context.access(),
            )
            .unwrap();
            assert!(self.loader.insert(name, module.clone()).is_none());
            module
        }

        fn load_and_link(&mut self, module: &Module) {
            let loaded = module.load(self.context.access()).unwrap();
            for _ in 0..128 {
                if loaded.state().unwrap() != PromiseState::Pending {
                    break;
                }
                let queued = self.executor.jobs.borrow_mut().pop_front();
                let Some(Job::AsyncJob(mut job)) = queued else {
                    panic!("the map loader must supply its actual native task: {queued:?}")
                };
                let host = self.context.host_access();
                let mut task = TaskContext::from_waker(Waker::noop());
                assert!(matches!(
                    job.run_step(&mut task, &host),
                    surgeist_js::job::JobStep::Prepared
                ));
                let surgeist_js::job::JobStep::Complete(result) = job.run_step(&mut task, &host)
                else {
                    panic!("the in-memory map loader returns an immediately ready result")
                };
                result.unwrap();
            }
            assert_eq!(
                loaded.state().unwrap(),
                PromiseState::Fulfilled(JsValue::undefined())
            );
            module.link(self.context.access()).unwrap();
        }

        fn count(&mut self, name: &str) -> JsValue {
            self.context
                .global_object()
                .get(js_string!(name), self.context.access())
                .unwrap()
        }

        fn pending_gate(&mut self, name: &str) -> (JsPromise, ResolvingFunctions) {
            let (gate, resolvers) = JsPromise::new_pending(self.context.access());
            self.context
                .global_object()
                .set(js_string!(name), gate.clone(), true, self.context.access())
                .unwrap();
            (gate, resolvers)
        }
    }

    #[test]
    fn interrupted_static_cycle_refuses_root_and_member_aliases_but_preserves_completed_dependency()
    {
        let mut graph = GraphFixture::new();
        let independent = graph.insert(
            "independent",
            "globalThis.independentEffects += 1; export const value = 42;",
        );
        let root = graph.insert(
            "cycle-root",
            "import 'independent'; import 'cycle-member'; globalThis.parentEffects += 1;",
        );
        let member = graph.insert("cycle-member", "import 'cycle-root'; globalThis.moduleEffects += 1; for (let step = 0; step < 1000; step++) {}");
        graph.load_and_link(&root);
        let completed = evaluate_with_fresh_budget(&independent, &mut graph.context);
        assert_eq!(
            completed.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        assert_eq!(graph.count("independentEffects"), 1.into());
        let root_alias = root.clone();
        let member_alias = member.clone();
        super::stop_evaluation(&root, &mut graph.context, INTERRUPTING_LIMIT);
        assert_eq!(effects(&mut graph.context), 1.into());
        assert_eq!(graph.count("parentEffects"), 0.into());
        let still_completed = evaluate_with_fresh_budget(&independent, &mut graph.context);
        assert_eq!(completed.identity(), still_completed.identity());
        assert_eq!(graph.count("independentEffects"), 1.into());
        let observations: Vec<_> = [&root, &root_alias, &member, &member_alias]
            .into_iter()
            .map(|entry| {
                graph
                    .context
                    .with_execution(ExecutionOptions::new(LATER_LIMIT), |access| {
                        entry.evaluate(access)
                    })
                    .unwrap()
            })
            .collect();
        for observation in observations {
            let error = observation
                .result
                .expect_err("every root and member alias must refuse native evaluation");
            assert!(error.as_engine().is_some());
            assert_eq!(observation.report.instructions().unwrap(), 0);
        }
        assert_eq!(effects(&mut graph.context), 1.into());
        assert_eq!(graph.count("parentEffects"), 0.into());
    }

    struct WaitingGraph {
        graph: GraphFixture,
        parent: Module,
        grandparent: Module,
        parent_promise: JsPromise,
        grand_promise: JsPromise,
        sibling_promise: JsPromise,
        child_gate: JsPromise,
        child_resolvers: ResolvingFunctions,
        sibling_gate: JsPromise,
        sibling_resolvers: ResolvingFunctions,
        parent_wait: NativeWait,
        grand_wait: NativeWait,
    }

    impl WaitingGraph {
        fn new() -> Self {
            let mut graph = GraphFixture::new();
            let (child_gate, child_resolvers) = graph.pending_gate("childGate");
            let (sibling_gate, sibling_resolvers) = graph.pending_gate("siblingGate");
            // The child is reached only through its importers. There is no
            // independently requested child evaluation promise or native wait.
            graph.insert("child", "await childGate; globalThis.moduleEffects += 1; for (let step = 0; step < 1000; step++) {}");
            let parent = graph.insert("parent", "import 'child'; globalThis.parentEffects += 1;");
            let sibling = graph.insert(
                "sibling",
                "await siblingGate; globalThis.independentEffects += 1;",
            );
            let grandparent = graph.insert(
                "grandparent",
                "import 'parent'; import 'sibling'; globalThis.grandEffects += 1;",
            );
            graph.load_and_link(&grandparent);
            let sibling_promise = evaluate_with_fresh_budget(&sibling, &mut graph.context);
            let parent_promise = evaluate_with_fresh_budget(&parent, &mut graph.context);
            let grand_promise = evaluate_with_fresh_budget(&grandparent, &mut graph.context);
            for promise in [&sibling_promise, &parent_promise, &grand_promise] {
                assert_eq!(promise.state().unwrap(), PromiseState::Pending);
            }
            assert_eq!(graph.count("parentEffects"), 0.into());
            assert_eq!(graph.count("grandEffects"), 0.into());
            let parent_wait = NativeWait::subscribe(&parent_promise, &mut graph.context);
            let grand_wait = NativeWait::subscribe(&grand_promise, &mut graph.context);
            Self {
                graph,
                parent,
                grandparent,
                parent_promise,
                grand_promise,
                sibling_promise,
                child_gate,
                child_resolvers,
                sibling_gate,
                sibling_resolvers,
                parent_wait,
                grand_wait,
            }
        }

        fn interrupt_child(&mut self) {
            assert_eq!(self.child_gate.state().unwrap(), PromiseState::Pending);
            self.child_resolvers
                .resolve
                .call(
                    &JsValue::undefined(),
                    &[1.into()],
                    self.graph.context.access(),
                )
                .unwrap();
            self.parent_wait.assert_pending();
            self.grand_wait.assert_pending();
            let (direct, outcome) = self
                .graph
                .executor
                .run_next(&mut self.graph.context, INTERRUPTING_LIMIT);
            assert_instruction_stop(&direct.unwrap_err(), INTERRUPTING_LIMIT);
            assert_instruction_stop(&outcome.result.unwrap_err(), INTERRUPTING_LIMIT);
            assert_eq!(outcome.report.instructions().unwrap(), INTERRUPTING_LIMIT);
            assert_eq!(effects(&mut self.graph.context), 1.into());
            assert_eq!(self.graph.count("parentEffects"), 0.into());
            assert_eq!(self.graph.count("grandEffects"), 0.into());
            assert_eq!(self.parent_promise.state().unwrap(), PromiseState::Pending);
            assert_eq!(self.grand_promise.state().unwrap(), PromiseState::Pending);
        }

        fn finish_independent_sibling(&mut self) {
            // This is a new settlement of still-pending required
            // work, not an ignored second resolution of the interrupted gate.
            assert_eq!(self.sibling_gate.state().unwrap(), PromiseState::Pending);
            assert_eq!(self.sibling_promise.state().unwrap(), PromiseState::Pending);
            self.sibling_resolvers
                .resolve
                .call(
                    &JsValue::undefined(),
                    &[2.into()],
                    self.graph.context.access(),
                )
                .unwrap();
            for _ in 0..8 {
                if self.sibling_promise.state().unwrap() != PromiseState::Pending {
                    break;
                }
                let (direct, outcome) = self
                    .graph
                    .executor
                    .run_next(&mut self.graph.context, LATER_LIMIT);
                direct.unwrap();
                outcome.result.unwrap();
            }
            assert_eq!(
                self.sibling_promise.state().unwrap(),
                PromiseState::Fulfilled(JsValue::undefined())
            );
            assert_eq!(self.graph.count("independentEffects"), 1.into());
            assert_eq!(self.graph.count("parentEffects"), 0.into());
            assert_eq!(self.graph.count("grandEffects"), 0.into());
            assert_eq!(self.parent_promise.state().unwrap(), PromiseState::Pending);
            assert_eq!(self.grand_promise.state().unwrap(), PromiseState::Pending);
        }
    }

    #[test]
    fn interruption_wakes_transitive_waiting_parent_and_grandparent_without_a_child_native_promise()
    {
        let mut fixture = WaitingGraph::new();
        fixture.interrupt_child();
        fixture.parent_wait.assert_interrupted();
        fixture.grand_wait.assert_interrupted();
        assert_later_evaluation_is_terminal(&fixture.parent, &mut fixture.graph.context);
        assert_later_evaluation_is_terminal(&fixture.grandparent, &mut fixture.graph.context);
        assert_eq!(fixture.graph.count("parentEffects"), 0.into());
        assert_eq!(fixture.graph.count("grandEffects"), 0.into());
    }

    #[test]
    fn late_independent_dependency_settlement_does_not_revive_an_interrupted_importer() {
        let mut fixture = WaitingGraph::new();
        fixture.interrupt_child();
        fixture.finish_independent_sibling();
        assert_later_evaluation_is_terminal(&fixture.grandparent, &mut fixture.graph.context);
        assert_eq!(fixture.graph.count("grandEffects"), 0.into());
        assert_eq!(effects(&mut fixture.graph.context), 1.into());
    }

    #[test]
    fn new_importer_of_an_interrupted_dependency_is_refused_before_its_body() {
        let mut fixture = WaitingGraph::new();
        fixture.interrupt_child();
        let importer = fixture.graph.insert(
            "new-importer",
            "import 'parent'; globalThis.importerEffects += 1;",
        );
        fixture.graph.load_and_link(&importer);
        assert_later_evaluation_is_terminal(&importer, &mut fixture.graph.context);
        assert_eq!(fixture.graph.count("importerEffects"), 0.into());
        assert_eq!(fixture.graph.count("parentEffects"), 0.into());
        assert_eq!(effects(&mut fixture.graph.context), 1.into());
    }

    #[test]
    fn late_sibling_rejection_does_not_publish_guest_rejection_for_interrupted_ancestors() {
        let mut fixture = WaitingGraph::new();
        fixture.interrupt_child();
        assert_eq!(fixture.sibling_gate.state().unwrap(), PromiseState::Pending);
        assert_eq!(
            fixture.sibling_promise.state().unwrap(),
            PromiseState::Pending
        );
        let reason = JsValue::from(js_string!("authored late sibling rejection"));
        fixture
            .sibling_resolvers
            .reject
            .call(
                &JsValue::undefined(),
                std::slice::from_ref(&reason),
                fixture.graph.context.access(),
            )
            .unwrap();
        assert_eq!(
            fixture.sibling_gate.state().unwrap(),
            PromiseState::Rejected(reason.clone())
        );
        for _ in 0..8 {
            if fixture.sibling_promise.state().unwrap() != PromiseState::Pending {
                break;
            }
            let (direct, outcome) = fixture
                .graph
                .executor
                .run_next(&mut fixture.graph.context, LATER_LIMIT);
            direct.unwrap();
            outcome.result.unwrap();
        }
        assert_eq!(
            fixture.sibling_promise.state().unwrap(),
            PromiseState::Rejected(reason)
        );
        assert_eq!(
            fixture.parent_promise.state().unwrap(),
            PromiseState::Pending
        );
        assert_eq!(
            fixture.grand_promise.state().unwrap(),
            PromiseState::Pending
        );
        assert_eq!(fixture.graph.count("independentEffects"), 0.into());
        assert_eq!(fixture.graph.count("parentEffects"), 0.into());
        assert_eq!(fixture.graph.count("grandEffects"), 0.into());
        assert_eq!(effects(&mut fixture.graph.context), 1.into());
        assert_later_evaluation_is_terminal(&fixture.parent, &mut fixture.graph.context);
        assert_later_evaluation_is_terminal(&fixture.grandparent, &mut fixture.graph.context);
    }
}
