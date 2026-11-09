use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsData, JsError, JsObject, JsResult, JsValue, Module, NativeFunction,
    Source,
    builtins::promise::{PromiseState, ResolvingFunctions},
    error::{
        DiagnosticEngine, DiagnosticKind, EngineError, EngineResourceError, RuntimeLimitError,
    },
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobRetirement, JobStatus,
    },
    js_string,
    object::builtins::{JsFuture, JsPromise},
    property::Attribute,
};
use surgeist_js_gc::heap::Trace;

const FINITE: u64 = 100_000;
enum Policy {
    StoreThenFail(JsError),
    Discard,
    Refuse(JsError),
    InlineTerminal(Option<JsError>),
    EnterBeforeHandlerFails,
}
#[derive(Default)]
struct LifecycleExecutor {
    queue: RefCell<VecDeque<Job>>,
    policy: RefCell<Option<Policy>>,
    observed: RefCell<Option<JobObserver>>,
    inline_nested: Cell<bool>,
    terminal: RefCell<Option<JsPromise>>,
    probe_drops: RefCell<Option<Rc<Cell<usize>>>>,
}
impl LifecycleExecutor {
    fn arm(&self, policy: Policy) {
        assert!(self.policy.replace(Some(policy)).is_none());
        self.observed.borrow_mut().take();
    }
    fn observer(&self) -> JobObserver {
        self.observed.borrow().as_ref().unwrap().clone()
    }
    fn invoke(job: Job, access: ContextAccess<'_>) -> JsResult<JsValue> {
        let Job::PromiseJob(reaction) = job else {
            panic!("the fixture dispatches its real import-free promise reactions")
        };
        match reaction.call(access) {
            JobCall::Complete(result) => result,
            JobCall::Deferred { job, error } => {
                drop(job);
                panic!("this admitted callback must not be a pre-entry Deferred result: {error:?}")
            }
        }
    }
    fn step(&self, context: &mut Context) -> JsResult<JsValue> {
        let job = self
            .queue
            .borrow_mut()
            .pop_front()
            .expect("actual accepted wrapper remains queued");
        let mut direct = None;
        let outcome = context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                let result = Self::invoke(job, access);
                direct = Some(result.clone());
                result
            })
            .unwrap();
        assert_eq!(outcome.result, direct.unwrap());
        outcome.result
    }
    fn finish(&self, promise: &JsPromise, context: &mut Context) {
        for _ in 0..16 {
            if promise.state().unwrap() != PromiseState::Pending {
                return;
            }
            self.step(context).unwrap();
        }
        assert_ne!(
            promise.state().unwrap(),
            PromiseState::Pending,
            "the retained finite reaction chain must finish"
        );
    }
}
impl JobExecutor for LifecycleExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        if self.inline_nested.get() {
            Self::invoke(job, access).unwrap();
            return Ok(JobAcceptance::Accepted);
        }
        let Some(policy) = self.policy.borrow_mut().take() else {
            self.queue.borrow_mut().push_back(job);
            return Ok(JobAcceptance::Accepted);
        };
        let Job::PromiseJob(reaction) = &job else {
            panic!("the configured policy belongs to the actual selected Await reaction")
        };
        *self.observed.borrow_mut() = Some(reaction.observer());
        match policy {
            Policy::StoreThenFail(error) => {
                self.queue.borrow_mut().push_back(job);
                Ok(JobAcceptance::AcceptedWithError(
                    JobPublicationCause::try_new(error, access).unwrap(),
                ))
            }
            Policy::Discard => {
                drop(job);
                Ok(JobAcceptance::Discarded)
            }
            Policy::Refuse(error) => Err(JobPublicationError::new(
                job,
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
            Policy::InlineTerminal(error) => {
                self.inline_nested.set(true);
                Self::invoke(job, access.reborrow()).unwrap();
                self.inline_nested.set(false);
                assert_eq!(self.observer().status(), Ok(JobStatus::Completed));
                assert_eq!(
                    self.terminal.borrow().as_ref().unwrap().state().unwrap(),
                    PromiseState::Fulfilled(JsValue::undefined())
                );
                // NativeJob::call and every nested completion have returned.
                // Collection happens before Promise receives/maps this receipt.
                access.runtime().collect().unwrap();
                assert_eq!(self.probe_drops.borrow().as_ref().unwrap().get(), 1);
                match error {
                    Some(error) => Ok(JobAcceptance::AcceptedWithError(
                        JobPublicationCause::try_new(error, access).unwrap(),
                    )),
                    None => Ok(JobAcceptance::Accepted),
                }
            }
            Policy::EnterBeforeHandlerFails => {
                // Native job admission checks stop/realm/invocation, not this
                // function recursion quote. The handler Call fails only AFTER
                // JobCall has consumed the actual admitted wrapper.
                let saved = access.runtime_limits();
                let mut failing = saved;
                failing.set_recursion_limit(0);
                access.set_runtime_limits(failing);
                let result = Self::invoke(job, access.reborrow());
                access.set_runtime_limits(saved);
                let error = result.unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Recursion))
                );
                assert_eq!(
                    self.observer().status(),
                    Ok(JobStatus::Retired(JobRetirement::Interrupted))
                );
                Ok(JobAcceptance::AcceptedWithError(
                    JobPublicationCause::try_new(error, access).unwrap(),
                ))
            }
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("actual saved wrappers are dispatched explicitly")
    }
}

thread_local! {
    static PROBE: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}
#[derive(Debug, JsData, Trace)]
struct Leaf(#[heap_trace(skip)] Rc<Cell<usize>>);
impl Drop for Leaf {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
struct ProbeInput;
impl Drop for ProbeInput {
    fn drop(&mut self) {
        PROBE.with(|slot| drop(slot.borrow_mut().take()));
    }
}
fn put_probe(context: &Context, drops: Rc<Cell<usize>>) -> ProbeInput {
    let leaf = JsObject::from_proto_and_data(context.runtime(), None, Leaf(drops)).unwrap();
    PROBE.with(|slot| assert!(slot.replace(Some(leaf.into())).is_none()));
    ProbeInput
}
fn context(executor: Rc<LifecycleExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    for name in ["lifecycleEffects", "lifecycleFinally"] {
        context
            .register_global_property(surgeist_js::JsString::from(name), 0, Attribute::all())
            .unwrap();
    }
    context
        .register_global_builtin_callable(
            js_string!("takeLifecycleProbe"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                Ok(PROBE.with(|slot| {
                    slot.borrow_mut()
                        .take()
                        .expect("one actual continuation-only native leaf")
                }))
            }),
        )
        .unwrap();
    context
}
fn eval(source: &str, context: &mut Context) -> JsResult<JsValue> {
    context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(FINITE))
        .unwrap()
        .result
}
fn value(name: &str, context: &mut Context) -> JsValue {
    context
        .global_object()
        .get(surgeist_js::JsString::from(name), context.access())
        .unwrap()
}
fn gate(context: &mut Context) -> ResolvingFunctions {
    let (gate, resolvers) = JsPromise::new_pending(context.access());
    context
        .register_global_property(js_string!("lifecycleGate"), gate, Attribute::all())
        .unwrap();
    resolvers
}
fn native(promise: &JsPromise, context: &mut Context) -> JsFuture {
    let mut future = promise.clone().into_js_future(context.access()).unwrap();
    assert!(poll(&mut future).is_pending());
    future
}
fn poll(future: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn linked(source: &str, context: &mut Context) -> Module {
    let module = Module::parse(Source::from_bytes(source), None, context.access()).unwrap();
    assert_eq!(
        module.load(context.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    module
}
fn evaluation(module: &Module, context: &mut Context) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap()
}
fn allocation() -> JsError {
    EngineResourceError::Allocation.into()
}

#[test]
fn an_ordinary_async_initial_await_keeps_the_stored_reaction_without_premature_finally() {
    let executor = Rc::new(LifecycleExecutor::default());
    let mut context = context(executor.clone());
    executor.arm(Policy::StoreThenFail(allocation()));
    let result = eval(
        "(async function initialStored() {
             globalThis.lifecycleEffects += 1;
             try { await 0; globalThis.lifecycleEffects += 10; }
             finally { globalThis.lifecycleFinally += 1; }
         })()",
        &mut context,
    );
    assert_eq!(result.unwrap_err(), allocation());
    assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(1));
    assert_eq!(value("lifecycleFinally", &mut context), JsValue::new(0));
    let observer = executor.observer();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    executor.step(&mut context).unwrap();
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(11));
    assert_eq!(value("lifecycleFinally", &mut context), JsValue::new(1));
    assert!(executor.queue.borrow().is_empty());
    assert_eq!(eval("40 + 2", &mut context).unwrap(), JsValue::new(42));
}

#[test]
fn an_ordinary_async_resumed_await_preserves_the_original_promise_and_native_wait() {
    let executor = Rc::new(LifecycleExecutor::default());
    let mut context = context(executor.clone());
    let resolvers = gate(&mut context);
    let result = eval(
        "(async function resumedStored() {
             await lifecycleGate; globalThis.lifecycleEffects += 1;
             try { await 0; globalThis.lifecycleEffects += 10; }
             finally { globalThis.lifecycleFinally += 1; }
             return 42;
         })()",
        &mut context,
    )
    .unwrap();
    let promise = JsPromise::from_object(result.as_object().unwrap()).unwrap();
    let mut wait = native(&promise, &mut context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
        .unwrap();
    executor.arm(Policy::StoreThenFail(allocation()));
    assert_eq!(executor.step(&mut context).unwrap_err(), allocation());
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    assert!(poll(&mut wait).is_pending());
    assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(1));
    assert_eq!(value("lifecycleFinally", &mut context), JsValue::new(0));
    let observer = executor.observer();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    executor.finish(&promise, &mut context);
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    assert_eq!(poll(&mut wait), Poll::Ready(Ok(JsValue::new(42))));
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(11));
    assert_eq!(value("lifecycleFinally", &mut context), JsValue::new(1));
}

#[test]
fn later_real_discard_or_refusal_releases_source_pending_ordinary_and_module_captures() {
    for module_case in [false, true] {
        for refused in [false, true] {
            let executor = Rc::new(LifecycleExecutor::default());
            let mut context = context(executor.clone());
            let resolvers = gate(&mut context);
            let drops = Rc::new(Cell::new(0));
            let _input = put_probe(&context, drops.clone());
            let source = "globalThis.lifecycleEffects += 1;
                function useProbe(leaf, value) { globalThis.lifecycleEffects += 10; }
                useProbe(takeLifecycleProbe(), await lifecycleGate);";
            let module = module_case.then(|| linked(source, &mut context));
            let promise = match &module {
                Some(module) => evaluation(module, &mut context),
                None => {
                    let result = eval(
                        &format!("(async function pendingOrdinary() {{ {source} }})()"),
                        &mut context,
                    )
                    .unwrap();
                    JsPromise::from_object(result.as_object().unwrap()).unwrap()
                }
            };
            assert!(PROBE.with(|slot| slot.borrow().is_none()));
            context.runtime().collect().unwrap();
            assert_eq!(
                drops.get(),
                0,
                "the first argument is actually retained across the pending Await"
            );
            executor.arm(if refused {
                Policy::Refuse(allocation())
            } else {
                Policy::Discard
            });
            let result =
                resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[JsValue::new(7)], context.access());
            if refused {
                assert_eq!(result.unwrap_err(), allocation());
            } else {
                result.unwrap();
            }
            assert_eq!(
                executor.observer().status(),
                Ok(JobStatus::Retired(JobRetirement::Dropped))
            );
            assert_eq!(executor.observer().fault(), None);
            assert!(executor.queue.borrow().is_empty());
            assert_eq!(promise.state().unwrap(), PromiseState::Pending);
            assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(1));
            context.runtime().collect().unwrap();
            assert_eq!(
                drops.get(),
                1,
                "neither a dropped selected job nor the old source-retained qualification owns a runnable capture"
            );
            if let Some(module) = &module {
                if refused {
                    let stopped = context
                        .with_execution(ExecutionOptions::new(FINITE), |access| {
                            module.evaluate(access)
                        })
                        .unwrap();
                    assert_eq!(stopped.report.instructions().unwrap(), 0);
                    let error = stopped.result.unwrap_err();
                    let Some(EngineError::ModuleInterrupted(cause)) = error.as_engine() else {
                        panic!(
                            "required module reaction refusal must preserve its actual terminal cause"
                        )
                    };
                    assert_eq!(
                        cause.cause().kind(),
                        DiagnosticKind::Engine(DiagnosticEngine::Resource(
                            EngineResourceError::Allocation
                        ))
                    );
                }
            }
            assert_eq!(eval("40 + 2", &mut context).unwrap(), JsValue::new(42));
            assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(1));
        }
    }
}

#[test]
fn inline_normal_ordinary_and_module_completion_survives_collection_before_healthy_or_failed_receipt()
 {
    for module_case in [false, true] {
        for failed_receipt in [false, true] {
            let executor = Rc::new(LifecycleExecutor::default());
            let mut context = context(executor.clone());
            let resolvers = gate(&mut context);
            let drops = Rc::new(Cell::new(0));
            let _input = put_probe(&context, drops.clone());
            let source = "globalThis.lifecycleEffects += 1;
             function useProbe(leaf, value) { globalThis.lifecycleEffects += 10; }
             useProbe(takeLifecycleProbe(), await lifecycleGate);";
            let module = module_case.then(|| linked(source, &mut context));
            let promise = match &module {
                Some(module) => evaluation(module, &mut context),
                None => {
                    let result = eval(
                        &format!("(async function terminalOrdinary() {{ {source} }})()"),
                        &mut context,
                    )
                    .unwrap();
                    JsPromise::from_object(result.as_object().unwrap()).unwrap()
                }
            };
            let mut wait = native(&promise, &mut context);
            context.runtime().collect().unwrap();
            assert_eq!(drops.get(), 0);
            *executor.terminal.borrow_mut() = Some(promise.clone());
            *executor.probe_drops.borrow_mut() = Some(drops.clone());
            executor.arm(Policy::InlineTerminal(failed_receipt.then(allocation)));
            let result =
                resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[JsValue::new(7)], context.access());
            if failed_receipt {
                assert_eq!(result.unwrap_err(), allocation());
            } else {
                result.unwrap();
            }
            assert_eq!(executor.observer().status(), Ok(JobStatus::Completed));
            assert_eq!(executor.observer().fault(), None);
            assert_eq!(drops.get(), 1);
            assert_eq!(
                promise.state().unwrap(),
                PromiseState::Fulfilled(JsValue::undefined())
            );
            assert_eq!(poll(&mut wait), Poll::Ready(Ok(JsValue::undefined())));
            if let Some(module) = &module {
                assert_eq!(
                    evaluation(module, &mut context).identity(),
                    promise.identity()
                );
            }
            assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(11));
            assert_eq!(eval("40 + 2", &mut context).unwrap(), JsValue::new(42));
        }
    }
}

#[test]
fn admitted_handler_entry_failure_cannot_be_revived_by_its_later_accepted_error_receipt() {
    let executor = Rc::new(LifecycleExecutor::default());
    let mut context = context(executor.clone());
    let resolvers = gate(&mut context);
    let drops = Rc::new(Cell::new(0));
    let _input = put_probe(&context, drops.clone());
    let module = linked(
        "globalThis.lifecycleEffects += 1;
         function useProbe(leaf, value) { globalThis.lifecycleEffects += 10; }
         useProbe(takeLifecycleProbe(), await lifecycleGate);",
        &mut context,
    );
    let promise = evaluation(&module, &mut context);
    let mut wait = native(&promise, &mut context);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 0);
    executor.arm(Policy::EnterBeforeHandlerFails);
    let actual = resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
        .unwrap_err();
    assert_eq!(
        actual.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Recursion))
    );
    assert_eq!(
        executor.observer().status(),
        Ok(JobStatus::Retired(JobRetirement::Interrupted))
    );
    assert_eq!(executor.observer().fault(), None);
    assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(1));
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
    let Poll::Ready(Err(observed)) = poll(&mut wait) else {
        panic!("the original module waiter must retain the actual lifecycle interruption")
    };
    let Some(EngineError::ModuleInterrupted(observed)) = observed.as_engine() else {
        panic!("native observation must preserve the module's typed cause")
    };
    assert_eq!(
        observed.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(RuntimeLimitError::Recursion))
    );
    let later = context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(later.report.instructions().unwrap(), 0);
    let later = later.result.unwrap_err();
    let Some(EngineError::ModuleInterrupted(later)) = later.as_engine() else {
        panic!("later acceptance cannot resurrect the already interrupted module")
    };
    assert_eq!(later.identity(), observed.identity());
    assert_eq!(eval("40 + 2", &mut context).unwrap(), JsValue::new(42));
    assert_eq!(value("lifecycleEffects", &mut context), JsValue::new(1));
}
