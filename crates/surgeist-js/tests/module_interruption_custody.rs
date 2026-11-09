#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, ContextAccess, JsError, JsResult, JsValue, Module, Source,
    builtins::promise::{PromiseState, ResolvingFunctions},
    error::{DiagnosticEngine, DiagnosticKind, EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobPublicationCause, JobPublicationError,
        JobRetirement, JobStatus, JobStep, PromiseJob,
    },
    js_string,
    module::MapModuleLoader,
    object::builtins::{JsFuture, JsPromise},
};

const STOP_LIMIT: u64 = 256;
const FRESH_LIMIT: u64 = 100_000;

// Scheduling is explicit, but selected work retains its real public wrapper
// and callback custody. No queue borrow spans invocation or disposal.
#[derive(Default)]
struct ManualExecutor {
    jobs: RefCell<VecDeque<Job>>,
}

impl JobExecutor for ManualExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.jobs.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the test dispatches captured jobs explicitly")
    }
}

impl ManualExecutor {
    fn take_promise(&self) -> PromiseJob {
        let queued = self.jobs.borrow_mut().pop_front();
        match queued {
            Some(Job::PromiseJob(job)) => job,
            other => panic!("expected the generated await/completion reaction: {other:?}"),
        }
    }

    fn finish(&self, promise: &JsPromise, context: &mut Context) {
        for _ in 0..8 {
            if promise.state().unwrap() != PromiseState::Pending {
                return;
            }
            invoke(self.take_promise(), context);
        }
        assert_ne!(
            promise.state().unwrap(),
            PromiseState::Pending,
            "the selected reactions must finish within eight dispatches"
        );
    }
}

fn context<Q: JobExecutor + 'static>(executor: Rc<Q>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.moduleEffects = 0; globalThis.innerStarts = 0;
             globalThis.innerFinishes = 0;",
        ))
        .unwrap();
    context
}

fn linked(source: &str, context: &mut Context) -> Module {
    let module = Module::parse(Source::from_bytes(source), None, context.access()).unwrap();
    let loaded = module.load(context.access()).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    module
}

fn pending_gate(context: &mut Context) -> (JsPromise, ResolvingFunctions) {
    let (gate, resolvers) = JsPromise::new_pending(context.access());
    context
        .global_object()
        .set(js_string!("gate"), gate.clone(), true, context.access())
        .unwrap();
    (gate, resolvers)
}

fn count(name: &str, context: &mut Context) -> JsValue {
    context
        .global_object()
        .get(surgeist_js::JsString::from(name), context.access())
        .unwrap()
}

fn evaluate(module: &Module, context: &mut Context) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap()
}

fn assert_terminal(module: &Module, context: &mut Context) {
    let outcome = context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    let error = outcome
        .result
        .expect_err("fresh admission cannot revive the interrupted module instance");
    assert!(error.as_engine().is_some(), "{error:?}");
    assert_eq!(outcome.report.instructions().unwrap(), 0);
}

fn assert_limit(error: &JsError, expected: RuntimeLimitError) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(expected))
    );
}

fn invoke(job: PromiseJob, context: &mut Context) {
    let outcome = context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            match job.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!("fresh invocation unexpectedly deferred: {error:?}")
                }
            }
        })
        .unwrap();
    outcome.result.unwrap();
}

struct NativeWait(Pin<Box<JsFuture>>);

impl NativeWait {
    fn subscribe(promise: &JsPromise, context: &mut Context) -> Self {
        let mut wait = Self(Box::pin(
            promise.clone().into_js_future(context.access()).unwrap(),
        ));
        wait.assert_pending();
        wait
    }

    fn poll(&mut self) -> Poll<JsResult<JsValue>> {
        self.0
            .as_mut()
            .poll(&mut TaskContext::from_waker(Waker::noop()))
    }

    fn assert_pending(&mut self) {
        assert!(
            self.poll().is_pending(),
            "native evaluation must remain viable"
        );
    }

    fn assert_fulfilled(&mut self, value: JsValue) {
        let Poll::Ready(result) = self.poll() else {
            panic!("actual settlement must complete the original native wait")
        };
        assert_eq!(result.unwrap(), value);
    }

    fn assert_interrupted(&mut self) {
        let Poll::Ready(Err(error)) = self.poll() else {
            panic!("the invalidated module must terminate the original native wait")
        };
        assert!(error.as_engine().is_some(), "{error:?}");
    }
}

#[test]
fn suspended_async_function_created_by_a_module_survives_that_modules_own_stop() {
    let executor = Rc::new(ManualExecutor::default());
    let mut context = context(executor.clone());
    let (gate, resolvers) = pending_gate(&mut context);
    let module = linked(
        "globalThis.moduleEffects += 1;
         globalThis.inner = (async function independent() {
             globalThis.innerStarts += 1;
             await gate;
             globalThis.innerFinishes += 1;
             return 42;
         })();
         for (let step = 0; step < 1000; step++) {}
         globalThis.moduleEffects += 100;",
        &mut context,
    );
    let stopped = context
        .with_execution(ExecutionOptions::new(STOP_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_limit(
        &stopped.result.unwrap_err(),
        RuntimeLimitError::Instruction { limit: STOP_LIMIT },
    );
    assert_eq!(stopped.report.instructions().unwrap(), STOP_LIMIT);
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_eq!(count("innerStarts", &mut context), 1.into());
    assert_eq!(count("innerFinishes", &mut context), 0.into());
    assert_eq!(gate.state().unwrap(), PromiseState::Pending);
    let inner = JsPromise::from_object(
        context
            .global_object()
            .get(js_string!("inner"), context.access())
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let mut wait = NativeWait::subscribe(&inner, &mut context);
    assert_terminal(&module, &mut context);

    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    executor.finish(&inner, &mut context);
    assert_eq!(inner.state().unwrap(), PromiseState::Fulfilled(42.into()));
    wait.assert_fulfilled(42.into());
    assert_eq!(count("innerStarts", &mut context), 1.into());
    assert_eq!(count("innerFinishes", &mut context), 1.into());
    assert_terminal(&module, &mut context);
    assert_eq!(count("moduleEffects", &mut context), 1.into());
}

#[test]
fn outer_await_job_admission_refusal_returns_the_same_viable_continuation() {
    let executor = Rc::new(ManualExecutor::default());
    let mut context = context(executor.clone());
    let (gate, resolvers) = pending_gate(&mut context);
    let module = linked(
        "globalThis.moduleEffects += 1; await gate; globalThis.moduleEffects += 1;",
        &mut context,
    );
    let evaluation = evaluate(&module, &mut context);
    let mut wait = NativeWait::subscribe(&evaluation, &mut context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    assert_eq!(gate.state().unwrap(), PromiseState::Fulfilled(1.into()));
    let job = executor.take_promise();
    let observer = job.observer();
    let mut returned = None;
    let refused = context
        .with_execution(
            ExecutionOptions::new(FRESH_LIMIT).with_job_invocation_limit(0),
            |access| match job.call(access) {
                JobCall::Deferred { job, error } => {
                    returned = Some(job);
                    Err::<JsValue, _>(error)
                }
                JobCall::Complete(result) => {
                    panic!("outer admission must preserve uncalled custody: {result:?}")
                }
            },
        )
        .unwrap();
    assert_limit(
        &refused.result.unwrap_err(),
        RuntimeLimitError::JobInvocation { limit: 0 },
    );
    assert_eq!(refused.report.job_invocations().unwrap(), 0);
    assert_eq!(refused.report.instructions().unwrap(), 0);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    wait.assert_pending();
    let cached = evaluate(&module, &mut context);
    assert_eq!(cached.identity(), evaluation.identity());

    // This is the exact wrapper returned by Deferred, not a recreated callback.
    invoke(returned.unwrap(), &mut context);
    executor.finish(&evaluation, &mut context);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    wait.assert_fulfilled(JsValue::undefined());
    assert_eq!(count("moduleEffects", &mut context), 2.into());
    let cached = evaluate(&module, &mut context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(count("moduleEffects", &mut context), 2.into());
}

#[test]
fn consumed_await_reaction_whose_handler_entry_is_refused_interrupts_its_module() {
    let executor = Rc::new(ManualExecutor::default());
    let mut context = context(executor.clone());
    let (_gate, resolvers) = pending_gate(&mut context);
    let module = linked(
        "globalThis.moduleEffects += 1; await gate; globalThis.moduleEffects += 100;",
        &mut context,
    );
    let evaluation = evaluate(&module, &mut context);
    let mut wait = NativeWait::subscribe(&evaluation, &mut context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    let job = executor.take_promise();
    let observer = job.observer();
    let recursion = context.runtime_limits().recursion_limit();
    // Outer native-job admission remains available. Its consumed callback then
    // reaches the real builtin reaction handler's runtime entry check.
    context.runtime_limits_mut().set_recursion_limit(0);
    let outcome = context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            match job.call(access) {
                JobCall::Complete(result) => {
                    assert_limit(result.as_ref().unwrap_err(), RuntimeLimitError::Recursion);
                    result
                }
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!("this case requires consumed outer callback custody: {error:?}")
                }
            }
        })
        .unwrap();
    context.runtime_limits_mut().set_recursion_limit(recursion);
    assert_limit(&outcome.result.unwrap_err(), RuntimeLimitError::Recursion);
    assert_eq!(outcome.report.job_invocations().unwrap(), 1);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(
        observer.status().unwrap(),
        JobStatus::Retired(JobRetirement::Interrupted)
    );
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    wait.assert_interrupted();
    assert_terminal(&module, &mut context);
    assert_eq!(count("moduleEffects", &mut context), 1.into());
}

// Dispatch the first await reaction inline and stop only after that action has
// successfully published a newer suspension. Later work stays in the real queue.
#[derive(Default)]
struct InlineTransferExecutor {
    queue: ManualExecutor,
    inline_first: Cell<bool>,
    publication_stop: RefCell<Option<JsError>>,
}

impl JobExecutor for InlineTransferExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        if !self.inline_first.replace(false) {
            self.queue.jobs.borrow_mut().push_back(job);
            return Ok(JobAcceptance::Accepted);
        }
        let Job::PromiseJob(job) = job else {
            panic!("the first immediate await supplies a promise reaction")
        };
        match job.call(access.reborrow()) {
            JobCall::Complete(result) => {
                result.unwrap();
            }
            JobCall::Deferred { job, error } => {
                self.queue.jobs.borrow_mut().push_back(job.into());
                panic!("the authored inline resume must actually run: {error:?}")
            }
        }
        let error = access
            .eval(Source::from_bytes(
                "for (let step = 0; step < 1000; step++) {}",
            ))
            .expect_err("the enclosing publication hook exhausts the finite operation budget");
        let cause = JobPublicationCause::try_new(error.clone(), access.reborrow())
            .expect("the authored instruction stop is engine-valued");
        assert!(self.publication_stop.replace(Some(error)).is_none());
        Ok(JobAcceptance::AcceptedWithError(cause))
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the test dispatches captured jobs explicitly")
    }
}

#[test]
fn inline_resume_transfers_a_new_module_await_before_enclosing_publication_stops() {
    let executor = Rc::new(InlineTransferExecutor::default());
    let mut context = context(executor.clone());
    let (gate, resolvers) = pending_gate(&mut context);
    let module = linked(
        "await 0; globalThis.moduleEffects += 1;
         await gate; globalThis.moduleEffects += 1;",
        &mut context,
    );
    executor.inline_first.set(true);
    let stopped = context
        .with_execution(ExecutionOptions::new(STOP_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    let original = executor.publication_stop.borrow().clone().unwrap();
    assert_limit(
        &original,
        RuntimeLimitError::Instruction { limit: STOP_LIMIT },
    );
    assert_eq!(stopped.result.unwrap_err(), original);
    assert_eq!(stopped.report.instructions().unwrap(), STOP_LIMIT);
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_eq!(gate.state().unwrap(), PromiseState::Pending);

    // The inline action already transferred a valid newer module continuation.
    let evaluation = evaluate(&module, &mut context);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    let mut wait = NativeWait::subscribe(&evaluation, &mut context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    executor.queue.finish(&evaluation, &mut context);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    wait.assert_fulfilled(JsValue::undefined());
    assert_eq!(count("moduleEffects", &mut context), 2.into());
    let cached = evaluate(&module, &mut context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(count("moduleEffects", &mut context), 2.into());
}

// Native adapters must retain the actual module cause even when evaluation
// stops before returning the evaluation promise to the adapter callback.
fn assert_interruption_cause(error: &JsError) {
    let Some(EngineError::ModuleInterrupted(interruption)) = error.as_engine() else {
        panic!("the source-module cause must win over job retirement: {error:?}")
    };
    assert_eq!(
        interruption.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(
            RuntimeLimitError::Instruction { limit: STOP_LIMIT }
        ))
    );
}

fn interrupt_next_reaction(executor: &ManualExecutor, context: &mut Context) {
    let job = executor.take_promise();
    let mut direct = None;
    let outcome = context
        .with_execution(ExecutionOptions::new(STOP_LIMIT), |access| {
            match job.call(access) {
                JobCall::Complete(result) => {
                    direct = Some(result.as_ref().unwrap_err().clone());
                    result
                }
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!("evaluation must enter the real source body: {error:?}")
                }
            }
        })
        .unwrap();
    let direct = direct.unwrap();
    assert_limit(
        &direct,
        RuntimeLimitError::Instruction { limit: STOP_LIMIT },
    );
    assert_eq!(outcome.result.unwrap_err(), direct);
    assert_eq!(outcome.report.instructions().unwrap(), STOP_LIMIT);
    assert_eq!(count("moduleEffects", context), 1.into());
}

fn assert_adapter_interrupted(
    adapter: &JsPromise,
    wait: &mut NativeWait,
    module: &Module,
    context: &mut Context,
) {
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    let Poll::Ready(Err(error)) = wait.poll() else {
        panic!("the preexisting native adapter wait must observe the module interruption")
    };
    assert_interruption_cause(&error);
    let original_identity = match error.as_engine().unwrap() {
        EngineError::ModuleInterrupted(interruption) => interruption.identity(),
        _ => unreachable!(),
    };
    let later = context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(later.report.instructions().unwrap(), 0);
    let later_error = later.result.unwrap_err();
    assert_interruption_cause(&later_error);
    let Some(EngineError::ModuleInterrupted(interruption)) = later_error.as_engine() else {
        unreachable!()
    };
    assert_eq!(interruption.identity(), original_identity);
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    assert_eq!(count("moduleEffects", context), 1.into());
}

fn adapter_source(context: &mut Context) -> Module {
    context
        .global_object()
        .set(js_string!("moduleEffects"), 0, true, context.access())
        .unwrap();
    Module::parse(
        Source::from_bytes(
            "globalThis.moduleEffects += 1;
             for (let step = 0; step < 1000; step++) {}
             globalThis.moduleEffects += 100; export const answer = 42;",
        ),
        None,
        context.access(),
    )
    .unwrap()
}

#[test]
fn combined_evaluation_native_wait_keeps_a_module_cause_when_the_initial_body_stops() {
    let executor = Rc::new(ManualExecutor::default());
    let mut context = context(executor.clone());
    let module = adapter_source(&mut context);
    let adapter = module.load_link_evaluate(context.access()).unwrap();
    let mut wait = NativeWait::subscribe(&adapter, &mut context);
    assert_eq!(count("moduleEffects", &mut context), 0.into());

    // Complete only linking; evaluation remains a real queued reaction.
    invoke(executor.take_promise(), &mut context);
    assert_eq!(count("moduleEffects", &mut context), 0.into());
    wait.assert_pending();
    interrupt_next_reaction(&executor, &mut context);
    assert_adapter_interrupted(&adapter, &mut wait, &module, &mut context);
}

#[test]
fn dynamic_import_native_wait_keeps_a_module_cause_when_the_initial_body_stops() {
    let executor = Rc::new(ManualExecutor::default());
    let loader = Rc::new(MapModuleLoader::new());
    let builder = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let module = adapter_source(&mut context);
    assert!(loader.insert("entry", module.clone()).is_none());
    let adapter = JsPromise::from_object(
        context
            .eval(Source::from_bytes("import('entry')"))
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let mut wait = NativeWait::subscribe(&adapter, &mut context);
    assert_eq!(count("moduleEffects", &mut context), 0.into());

    let queued = executor.jobs.borrow_mut().pop_front();
    let Some(Job::AsyncJob(mut producer)) = queued else {
        panic!("dynamic import must enqueue its real module loader: {queued:?}")
    };
    {
        let host = context.host_access();
        let mut task = TaskContext::from_waker(Waker::noop());
        assert!(matches!(
            producer.run_step(&mut task, &host),
            JobStep::Prepared
        ));
        let JobStep::Complete(result) = producer.run_step(&mut task, &host) else {
            panic!("the real in-memory MapModuleLoader is immediately ready")
        };
        result.unwrap();
    }
    assert_eq!(count("moduleEffects", &mut context), 0.into());
    wait.assert_pending();
    interrupt_next_reaction(&executor, &mut context);
    assert_adapter_interrupted(&adapter, &mut wait, &module, &mut context);
}

#[test]
fn combined_evaluation_rejects_foreign_runtime_preparation_with_a_guest_type_error() {
    let mut owner = context(Rc::new(ManualExecutor::default()));
    let mut receiver = context(Rc::new(ManualExecutor::default()));
    let module = Module::parse(
        Source::from_bytes("globalThis.moduleEffects += 1;"),
        None,
        owner.access(),
    )
    .unwrap();

    let adapter = module
        .load_link_evaluate(receiver.access())
        .expect("catchable preparation failure must return a guest promise");
    let PromiseState::Rejected(reason) = adapter.state().unwrap() else {
        panic!("a foreign-runtime module must reject the combined evaluation promise")
    };
    let error = JsError::from_opaque(reason)
        .try_native(receiver.access())
        .unwrap();
    assert!(error.is_type());
    assert_eq!(count("moduleEffects", &mut owner), 0.into());
    assert_eq!(count("moduleEffects", &mut receiver), 0.into());
}

// Promise jobs really run inline. A refused invocation retains the exact public
// wrapper returned by Deferred, while native loader jobs remain manually driven.
#[derive(Default)]
struct FullyInlineExecutor {
    inline: Cell<bool>,
    queued: ManualExecutor,
    deferred: RefCell<VecDeque<PromiseJob>>,
    errors: RefCell<Vec<JsError>>,
}

impl JobExecutor for FullyInlineExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        match job {
            Job::PromiseJob(job) if self.inline.get() => match job.call(access.reborrow()) {
                JobCall::Complete(Ok(_)) => Ok(JobAcceptance::Accepted),
                JobCall::Complete(Err(error)) => {
                    let cause = JobPublicationCause::try_new(error.clone(), access.reborrow());
                    self.errors.borrow_mut().push(error);
                    match cause {
                        Ok(cause) => Ok(JobAcceptance::AcceptedWithError(cause)),
                        // Preserve the fixture's recorded catchable callback-error policy.
                        Err(_catchable) => Ok(JobAcceptance::Accepted),
                    }
                }
                JobCall::Deferred { job, error } => {
                    self.deferred.borrow_mut().push_back(job);
                    self.errors.borrow_mut().push(error);
                    // The refused attempt precedes fallback acceptance. The
                    // existing operation stop and recorded diagnostic remain.
                    Ok(JobAcceptance::Accepted)
                }
            },
            job => {
                self.queued.jobs.borrow_mut().push_back(job);
                Ok(JobAcceptance::Accepted)
            }
        }
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the test explicitly drives saved and native jobs")
    }
}

impl FullyInlineExecutor {
    fn take_deferred(&self) -> PromiseJob {
        self.deferred
            .borrow_mut()
            .pop_front()
            .expect("initial await publication must retain its actual deferred reaction")
    }

    fn finish_deferred(&self, promise: &JsPromise, context: &mut Context) {
        for _ in 0..16 {
            let next = self.deferred.borrow_mut().pop_front();
            let Some(job) = next else {
                assert_ne!(
                    promise.state().unwrap(),
                    PromiseState::Pending,
                    "the original promise must settle after its accepted continuations finish"
                );
                return;
            };
            invoke(job, context);
        }
        assert!(
            self.deferred.borrow().is_empty(),
            "saved dispatch bound exhausted"
        );
        assert_ne!(
            promise.state().unwrap(),
            PromiseState::Pending,
            "saved accepted continuations must settle within sixteen fresh dispatches"
        );
    }

    fn assert_clean(&self) {
        assert!(
            self.errors.borrow().is_empty(),
            "{:?}",
            self.errors.borrow()
        );
        assert!(self.deferred.borrow().is_empty());
    }
}

fn fully_inline_context() -> (Context, Rc<FullyInlineExecutor>) {
    let executor = Rc::new(FullyInlineExecutor::default());
    executor.inline.set(true);
    (context(executor.clone()), executor)
}

fn assert_vm_usable(context: &mut Context) {
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("20 + 22"),
            ExecutionOptions::new(FRESH_LIMIT),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), 42.into());
}

fn assert_inline_completion(source: &str) {
    let (mut context, executor) = fully_inline_context();
    let module = linked(source, &mut context);
    let evaluation = evaluate(&module, &mut context);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    executor.assert_clean();
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    let cached = evaluate(&module, &mut context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(
        cached.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_vm_usable(&mut context);
}

#[test]
fn async_marked_module_without_executed_await_completes_with_fully_inline_jobs() {
    assert_inline_completion("if (false) await 0; globalThis.moduleEffects += 1;");
}

#[test]
fn initial_immediate_await_completes_with_fully_inline_jobs_and_preserves_its_cache() {
    assert_inline_completion("await Promise.resolve(0); globalThis.moduleEffects += 1;");
}

#[test]
fn fully_inline_initial_await_preserves_the_authored_guest_rejection_and_cached_promise() {
    let (mut context, executor) = fully_inline_context();
    let reason = JsValue::from(surgeist_js::JsObject::with_null_proto(context.runtime()));
    context
        .global_object()
        .set(
            js_string!("moduleGuestError"),
            reason.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let module = linked(
        "await Promise.resolve(0); globalThis.moduleEffects += 1; throw moduleGuestError;",
        &mut context,
    );
    let evaluation = evaluate(&module, &mut context);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Rejected(reason.clone())
    );
    executor.assert_clean();
    let cached = evaluate(&module, &mut context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(cached.state().unwrap(), PromiseState::Rejected(reason));
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_vm_usable(&mut context);
}

#[test]
fn initial_immediate_await_keeps_the_exact_deferred_job_viable_after_publication_refusal() {
    let (mut context, executor) = fully_inline_context();
    let module = linked(
        "await Promise.resolve(0); globalThis.moduleEffects += 1;",
        &mut context,
    );
    let stopped = context
        .with_execution(
            ExecutionOptions::new(FRESH_LIMIT).with_job_invocation_limit(0),
            |access| module.evaluate(access),
        )
        .unwrap();
    assert_limit(
        &stopped.result.unwrap_err(),
        RuntimeLimitError::JobInvocation { limit: 0 },
    );
    assert_eq!(stopped.report.job_invocations().unwrap(), 0);
    assert_eq!(count("moduleEffects", &mut context), 0.into());
    let continuation = executor.take_deferred();
    let observer = continuation.observer();
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    let evaluation = evaluate(&module, &mut context);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    let mut wait = NativeWait::subscribe(&evaluation, &mut context);

    executor.errors.borrow_mut().clear();
    invoke(continuation, &mut context);
    executor.finish_deferred(&evaluation, &mut context);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    wait.assert_fulfilled(JsValue::undefined());
    executor.assert_clean();
    let cached = evaluate(&module, &mut context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_vm_usable(&mut context);
}

fn initial_transfer_source(context: &mut Context) -> Module {
    Module::parse(
        Source::from_bytes(
            "await Promise.resolve(0); globalThis.moduleEffects += 1; export const answer = 42;",
        ),
        None,
        context.access(),
    )
    .unwrap()
}

fn refuse_initial_adapter_resume(executor: &FullyInlineExecutor, context: &mut Context) {
    let evaluation_job = executor.queued.take_promise();
    executor.inline.set(true);
    let mut direct = None;
    let outcome = context
        .with_execution(
            ExecutionOptions::new(FRESH_LIMIT).with_job_invocation_limit(1),
            |access| match evaluation_job.call(access) {
                JobCall::Complete(result) => {
                    direct = Some(result.as_ref().unwrap_err().clone());
                    result
                }
                JobCall::Deferred { job, error } => {
                    executor.queued.jobs.borrow_mut().push_front(job.into());
                    panic!("the outer evaluation action must be admitted: {error:?}")
                }
            },
        )
        .unwrap();
    let direct = direct.unwrap();
    assert_limit(&direct, RuntimeLimitError::JobInvocation { limit: 1 });
    assert_eq!(outcome.result.unwrap_err(), direct);
    assert_eq!(outcome.report.job_invocations().unwrap(), 1);
    assert_eq!(count("moduleEffects", context), 0.into());
}

fn complete_initial_transfer(
    executor: &FullyInlineExecutor,
    module: &Module,
    context: &mut Context,
) {
    let continuation = executor.take_deferred();
    let observer = continuation.observer();
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    let evaluation = evaluate(module, context);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    executor.errors.borrow_mut().clear();
    invoke(continuation, context);
    executor.finish_deferred(&evaluation, context);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    let cached = evaluate(module, context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(count("moduleEffects", context), 1.into());
    executor.assert_clean();
}

#[test]
fn combined_native_adapter_survives_initial_await_transfer_followed_by_outer_refusal() {
    let executor = Rc::new(FullyInlineExecutor::default());
    let mut context = context(executor.clone());
    let module = initial_transfer_source(&mut context);
    let adapter = module.load_link_evaluate(context.access()).unwrap();
    let mut wait = NativeWait::subscribe(&adapter, &mut context);
    assert_eq!(count("moduleEffects", &mut context), 0.into());
    // Preserve load -> link -> evaluate staging, then permit only the outer
    // evaluation invocation. The initial await's actual inline job is deferred.
    invoke(executor.queued.take_promise(), &mut context);
    refuse_initial_adapter_resume(&executor, &mut context);
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    wait.assert_pending();

    complete_initial_transfer(&executor, &module, &mut context);
    executor.finish_deferred(&adapter, &mut context);
    assert_eq!(
        adapter.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    wait.assert_fulfilled(JsValue::undefined());
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_vm_usable(&mut context);
}

#[test]
fn dynamic_import_native_adapter_survives_initial_await_transfer_followed_by_outer_refusal() {
    let executor = Rc::new(FullyInlineExecutor::default());
    let loader = Rc::new(MapModuleLoader::new());
    let builder = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .global_object()
        .set(js_string!("moduleEffects"), 0, true, context.access())
        .unwrap();
    let module = initial_transfer_source(&mut context);
    assert!(loader.insert("entry", module.clone()).is_none());
    let adapter = JsPromise::from_object(
        context
            .eval_with_budget(
                Source::from_bytes("import('entry')"),
                ExecutionOptions::new(FRESH_LIMIT),
            )
            .unwrap()
            .result
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let mut wait = NativeWait::subscribe(&adapter, &mut context);
    let queued = executor.queued.jobs.borrow_mut().pop_front();
    let Some(Job::AsyncJob(mut producer)) = queued else {
        panic!("dynamic import must enqueue its actual native loader: {queued:?}")
    };
    let loaded = context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            let host = access.into_host_access();
            let mut task = TaskContext::from_waker(Waker::noop());
            assert!(matches!(
                producer.run_step(&mut task, &host),
                JobStep::Prepared
            ));
            let JobStep::Complete(result) = producer.run_step(&mut task, &host) else {
                panic!("the actual in-memory MapModuleLoader must be immediately ready")
            };
            result
        })
        .unwrap();
    loaded.result.unwrap();
    assert_eq!(count("moduleEffects", &mut context), 0.into());
    refuse_initial_adapter_resume(&executor, &mut context);
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    wait.assert_pending();

    complete_initial_transfer(&executor, &module, &mut context);
    executor.finish_deferred(&adapter, &mut context);
    let PromiseState::Fulfilled(namespace) = adapter.state().unwrap() else {
        panic!("the original import adapter must publish its canonical module namespace")
    };
    assert_eq!(
        namespace
            .as_object()
            .unwrap()
            .get(js_string!("answer"), context.access())
            .unwrap(),
        42.into()
    );
    wait.assert_fulfilled(namespace);
    assert_eq!(count("moduleEffects", &mut context), 1.into());
    assert_vm_usable(&mut context);
}

struct InlineGraph {
    context: Context,
    executor: Rc<FullyInlineExecutor>,
    loader: Rc<MapModuleLoader>,
}

impl InlineGraph {
    fn new() -> Self {
        let executor = Rc::new(FullyInlineExecutor::default());
        executor.inline.set(true);
        let loader = Rc::new(MapModuleLoader::new());
        let builder = Context::builder()
            .job_executor(executor.clone())
            .module_loader(loader.clone());
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(1_000_000);
        let mut context = builder.build().unwrap();
        context
            .eval_with_budget(
                Source::from_bytes(
                    "globalThis.graphOrder = ''; globalThis.graphResult = 0;
                     globalThis.childEffects = 0; globalThis.parentEffects = 0;
                     globalThis.childReady = false; globalThis.releaserReady = false;
                     globalThis.lastReady = false;
                     globalThis.graphGate = new Promise(resolve => {
                         globalThis.releaseGraphGate = resolve;
                     });",
                ),
                ExecutionOptions::new(FRESH_LIMIT),
            )
            .unwrap()
            .result
            .unwrap();
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

    fn load_and_link(&mut self, root: &Module) {
        let loaded = self
            .context
            .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
                root.load(access)
            })
            .unwrap()
            .result
            .unwrap();
        for _ in 0..32 {
            if loaded.state().unwrap() != PromiseState::Pending {
                break;
            }
            let queued = self.executor.queued.jobs.borrow_mut().pop_front();
            let Some(Job::AsyncJob(mut job)) = queued else {
                panic!("the in-memory graph must supply a real ready loader job: {queued:?}")
            };
            let outcome = self
                .context
                .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
                    let host = access.into_host_access();
                    let mut task = TaskContext::from_waker(Waker::noop());
                    for _ in 0..8 {
                        match job.run_step(&mut task, &host) {
                            JobStep::Prepared | JobStep::Pending => {}
                            JobStep::Complete(result) => return result,
                            JobStep::Deferred(error) => return Err(error),
                            JobStep::Finished(status) => {
                                panic!("loader ended without its result: {status:?}")
                            }
                        }
                    }
                    panic!("the in-memory ready loader exceeded eight bounded steps")
                })
                .unwrap();
            outcome.result.unwrap();
        }
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        self.context
            .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
                root.link(access)
            })
            .unwrap()
            .result
            .unwrap();
        assert_eq!(
            count("graphOrder", &mut self.context),
            js_string!("").into()
        );
    }

    fn assert_success_once(&mut self, root: &Module, child: &Module, order: &str) {
        let evaluation = evaluate(root, &mut self.context);
        assert_eq!(
            evaluation.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        let cached = evaluate(root, &mut self.context);
        assert_eq!(cached.identity(), evaluation.identity());
        assert_eq!(
            cached.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        let child_result = evaluate(child, &mut self.context);
        assert_eq!(
            child_result.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        assert_eq!(
            evaluate(child, &mut self.context).identity(),
            child_result.identity()
        );
        assert_eq!(
            count("graphOrder", &mut self.context),
            surgeist_js::JsString::from(order).into()
        );
        assert_eq!(count("graphResult", &mut self.context), 42.into());
        assert_eq!(count("childEffects", &mut self.context), 1.into());
        assert_eq!(count("parentEffects", &mut self.context), 1.into());
        self.executor.assert_clean();
        assert_vm_usable(&mut self.context);
    }

    fn assert_gate_fulfilled(&mut self) {
        let gate =
            JsPromise::from_object(count("graphGate", &mut self.context).as_object().unwrap())
                .unwrap();
        assert_eq!(gate.state().unwrap(), PromiseState::Fulfilled(1.into()));
    }
}

#[test]
fn fully_inline_async_child_finishes_before_its_dependent_parent_and_caches_both() {
    let mut graph = InlineGraph::new();
    let child = graph.insert(
        "child",
        "await Promise.resolve(0); globalThis.childEffects += 1;
         globalThis.graphOrder += 'c'; export const value = 40;",
    );
    let parent = graph.insert(
        "parent",
        "import { value } from 'child'; globalThis.parentEffects += 1;
         globalThis.graphOrder += 'p'; globalThis.graphResult = value + 2;",
    );
    graph.load_and_link(&parent);
    graph.assert_success_once(&parent, &child, "cp");
}

#[test]
fn fully_inline_async_cycle_member_publishes_its_export_before_the_cycle_parent_body() {
    let mut graph = InlineGraph::new();
    let child = graph.insert(
        "cycle-child",
        "import 'cycle-parent'; await Promise.resolve(0);
         globalThis.childEffects += 1; globalThis.graphOrder += 'c';
         export const value = 40;",
    );
    let parent = graph.insert(
        "cycle-parent",
        "import { value } from 'cycle-child'; globalThis.parentEffects += 1;
         globalThis.graphOrder += 'p'; globalThis.graphResult = value + 2;
         export const parentValue = 2;",
    );
    graph.load_and_link(&parent);
    graph.assert_success_once(&parent, &child, "cp");
}

#[test]
fn inline_completion_of_an_older_child_waits_for_the_importers_remaining_dependencies() {
    let mut graph = InlineGraph::new();
    let child = graph.insert(
        "older",
        "globalThis.graphOrder += 's'; await graphGate;
         globalThis.childEffects += 1; globalThis.childReady = true;
         globalThis.graphOrder += 'c'; export const value = 40;",
    );
    graph.insert(
        "releaser",
        "if (globalThis.childReady) throw 99;
         globalThis.graphOrder += 'r'; releaseGraphGate(1);
         globalThis.graphOrder += 't'; globalThis.releaserReady = true;
         export const released = true;",
    );
    graph.insert(
        "last",
        "globalThis.lastReady = true; globalThis.graphOrder += 'l';
         export const lastValue = 2;",
    );
    let parent = graph.insert(
        "parent",
        "import { value } from 'older'; import { released } from 'releaser';
         import { lastValue } from 'last';
         if (!globalThis.releaserReady || !globalThis.lastReady || !released) throw 99;
         globalThis.parentEffects += 1; globalThis.graphOrder += 'p';
         globalThis.graphResult = value + lastValue;",
    );
    graph.load_and_link(&parent);
    graph.assert_success_once(&parent, &child, "srctlp");
    graph.assert_gate_fulfilled();
    assert_eq!(count("releaserReady", &mut graph.context), true.into());
    assert_eq!(count("lastReady", &mut graph.context), true.into());
}

#[test]
fn inline_rejection_of_an_older_child_preserves_the_reason_and_never_runs_its_importer() {
    let mut graph = InlineGraph::new();
    let reason = JsValue::from(surgeist_js::JsObject::with_null_proto(
        graph.context.runtime(),
    ));
    graph
        .context
        .global_object()
        .set(
            js_string!("graphGuestError"),
            reason.clone(),
            true,
            graph.context.access(),
        )
        .unwrap();
    let child = graph.insert(
        "older",
        "globalThis.graphOrder += 's'; await graphGate;
         globalThis.childEffects += 1; globalThis.graphOrder += 'e';
         throw graphGuestError;",
    );
    graph.insert(
        "releaser",
        "globalThis.graphOrder += 'r'; releaseGraphGate(1);
         globalThis.graphOrder += 't'; globalThis.releaserReady = true;",
    );
    let parent = graph.insert(
        "parent",
        "import 'older'; import 'releaser';
         globalThis.parentEffects += 1; globalThis.graphOrder += 'p';",
    );
    graph.load_and_link(&parent);
    let evaluation = evaluate(&parent, &mut graph.context);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Rejected(reason.clone())
    );
    let cached = evaluate(&parent, &mut graph.context);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(
        cached.state().unwrap(),
        PromiseState::Rejected(reason.clone())
    );
    let child_result = evaluate(&child, &mut graph.context);
    assert_eq!(
        child_result.state().unwrap(),
        PromiseState::Rejected(reason)
    );
    assert_eq!(
        evaluate(&child, &mut graph.context).identity(),
        child_result.identity()
    );
    assert_eq!(
        count("graphOrder", &mut graph.context),
        js_string!("sret").into()
    );
    assert_eq!(count("childEffects", &mut graph.context), 1.into());
    assert_eq!(count("parentEffects", &mut graph.context), 0.into());
    assert_eq!(count("releaserReady", &mut graph.context), true.into());
    graph.assert_gate_fulfilled();
    graph.executor.assert_clean();
    assert_vm_usable(&mut graph.context);
}

#[test]
fn native_waiter_finds_the_retained_module_cause_through_a_public_promise_follow_cycle() {
    let executor = Rc::new(ManualExecutor::default());
    let mut context = context(executor.clone());
    let (_gate, resolvers) = pending_gate(&mut context);
    let module = linked(
        "await gate; globalThis.moduleEffects += 1;
         for (let step = 0; step < 1000; step++) {}",
        &mut context,
    );
    let d = evaluate(&module, &mut context);
    let (a, _a_resolvers) = JsPromise::new_pending(context.access());
    let (b, _b_resolvers) = JsPromise::new_pending(context.access());
    let (c, _c_resolvers) = JsPromise::new_pending(context.access());
    let mut a_wait = NativeWait::subscribe(&a, &mut context);
    let _b_wait = NativeWait::subscribe(&b, &mut context);
    let _c_wait = NativeWait::subscribe(&c, &mut context);
    for (name, promise) in [
        ("cycleA", &a),
        ("cycleB", &b),
        ("cycleC", &c),
        ("cycleD", &d),
    ] {
        context
            .global_object()
            .set(
                surgeist_js::JsString::from(name),
                promise.clone(),
                true,
                context.access(),
            )
            .unwrap();
    }
    context
        .eval_with_budget(
            Source::from_bytes(
                "function followNative(source, target) {
                 source.constructor = {
                     [Symbol.species]: function(executor) {
                         executor(() => {}, () => {});
                         return target;
                     }
                 };
                 source.then();
             }
             followNative(cycleB, cycleD);
             followNative(cycleD, cycleC);
             followNative(cycleC, cycleB);
             followNative(cycleB, cycleA);",
            ),
            ExecutionOptions::new(FRESH_LIMIT),
        )
        .unwrap()
        .result
        .unwrap();
    for promise in [&a, &b, &c, &d] {
        assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    }
    a_wait.assert_pending();

    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    interrupt_next_reaction(&executor, &mut context);
    let Poll::Ready(Err(observed)) = a_wait.poll() else {
        panic!("the original native waiter must expose its reachable module interruption")
    };
    assert_interruption_cause(&observed);
    let retained = context
        .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(retained.report.instructions().unwrap(), 0);
    let retained = retained.result.unwrap_err();
    assert_interruption_cause(&retained);
    let Some(EngineError::ModuleInterrupted(observed)) = observed.as_engine() else {
        unreachable!()
    };
    let Some(EngineError::ModuleInterrupted(retained)) = retained.as_engine() else {
        unreachable!()
    };
    assert_eq!(observed.identity(), retained.identity());
    assert_eq!(observed.cause().kind(), retained.cause().kind());
    for promise in [&a, &b, &c, &d] {
        assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    }
    assert_eq!(count("moduleEffects", &mut context), 1.into());
}
