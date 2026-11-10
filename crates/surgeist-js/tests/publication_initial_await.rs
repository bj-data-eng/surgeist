use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeError, JsObject, JsResult, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::{DiagnosticKind, DiagnosticNativeKind, EngineError, EngineResourceError, PanicError},
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobStatus,
    },
    js_string,
    object::builtins::JsPromise,
    property::Attribute,
};

const ALLOWANCE: u64 = 100_000;

#[derive(Default)]
struct PostStoreExecutor {
    jobs: RefCell<VecDeque<Job>>,
    hook: RefCell<Option<JsError>>,
    accepted: RefCell<Option<JobObserver>>,
}
impl JobExecutor for PostStoreExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let hook = self.hook.borrow_mut().take();
        if hook.is_some() {
            let Job::PromiseJob(reaction) = &job else {
                panic!("the armed initial Await publishes its actual source reaction")
            };
            assert!(self.accepted.replace(Some(reaction.observer())).is_none());
        }
        // Physical acceptance precedes the independent host hook. No reaction
        // is called inline, rejected before entry, copied, or returned for replay.
        self.jobs.borrow_mut().push_back(job);
        match hook {
            Some(error) => Ok(JobAcceptance::AcceptedWithError(
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
            None => Ok(JobAcceptance::Accepted),
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches only its retained real wrappers explicitly")
    }
}
fn context(executor: Rc<PostStoreExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_property(js_string!("postStoreEffects"), 0, Attribute::all())
        .unwrap();
    context
}
fn linked(context: &mut Context) -> Module {
    let module = Module::parse(
        Source::from_bytes(
            "globalThis.postStoreEffects += 1; await 0; globalThis.postStoreEffects += 10;",
        ),
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
        .unwrap()
        .get(js_string!("postStoreEffects"), context.access())
        .unwrap()
}
fn fresh_evaluation(module: &Module, context: &mut Context) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(ALLOWANCE), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .expect("an accepted initial suspension remains a viable module instance")
}
fn finish_saved_continuation(executor: &PostStoreExecutor, module: &Module, context: &mut Context) {
    let observer = executor.accepted.borrow().as_ref().unwrap().clone();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert_eq!(effects(context), JsValue::new(1));
    let evaluation = fresh_evaluation(module, context);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    let cached_pending = fresh_evaluation(module, context);
    assert_eq!(cached_pending.identity(), evaluation.identity());
    assert_eq!(effects(context), JsValue::new(1));
    let mut native = evaluation.clone().into_js_future(context.access()).unwrap();
    assert!(
        Pin::new(&mut native)
            .poll(&mut TaskContext::from_waker(Waker::noop()))
            .is_pending()
    );
    for _ in 0..16 {
        if evaluation.state().unwrap() != PromiseState::Pending {
            break;
        }
        let queued = executor.jobs.borrow_mut().pop_front().expect(
            "actual accepted continuation and completion reactions retain dispatch custody",
        );
        let Job::PromiseJob(reaction) = queued else {
            panic!("this import-free module produces actual promise reactions")
        };
        context
            .with_execution(ExecutionOptions::new(ALLOWANCE), |access| {
                match reaction.call(access) {
                    JobCall::Complete(result) => result,
                    JobCall::Deferred { job, error } => {
                        drop(job);
                        panic!("fresh operation must admit the retained real wrapper: {error:?}")
                    }
                }
            })
            .unwrap()
            .result
            .unwrap();
    }
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        Pin::new(&mut native).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Ok(JsValue::undefined()))
    );
    assert_eq!(effects(context), JsValue::new(11));
    assert_eq!(
        fresh_evaluation(module, context).identity(),
        evaluation.identity()
    );
    assert_eq!(effects(context), JsValue::new(11));
    context
        .eval_with_budget(
            Source::from_bytes("40 + 2"),
            ExecutionOptions::new(ALLOWANCE),
        )
        .unwrap()
        .result
        .map(|value| assert_eq!(value, JsValue::new(42)))
        .unwrap();
}

#[test]
fn an_unscoped_initial_await_retains_its_stored_reaction_after_the_post_store_resource_error() {
    let executor = Rc::new(PostStoreExecutor::default());
    let mut context = context(executor.clone());
    let module = linked(&mut context);
    assert!(executor.jobs.borrow().is_empty());
    let authored: JsError = EngineResourceError::Allocation.into();
    *executor.hook.borrow_mut() = Some(authored.clone());
    let actual = module.evaluate(context.access()).unwrap_err();
    assert_eq!(actual, authored);
    assert_eq!(
        actual.as_engine(),
        Some(&EngineError::Resource(EngineResourceError::Allocation))
    );
    assert!(executor.hook.borrow().is_none());
    finish_saved_continuation(&executor, &module, &mut context);
}

#[test]
fn a_scoped_initial_await_retains_its_stored_reaction_and_exact_post_store_panic_diagnostic() {
    let executor = Rc::new(PostStoreExecutor::default());
    let mut context = context(executor.clone());
    let module = linked(&mut context);
    assert!(executor.jobs.borrow().is_empty());
    let opaque: JsValue = JsObject::with_null_proto(context.runtime()).unwrap().into();
    let authored: JsError = PanicError::new("authored post-storage hook")
        .with_source(
            JsNativeError::range()
                .with_message("authored hook cause")
                .with_cause(JsError::from_opaque(opaque.clone()).unwrap()),
        )
        .into();
    *executor.hook.borrow_mut() = Some(authored.clone());
    let mut direct = None;
    let outcome = context
        .with_execution(ExecutionOptions::new(ALLOWANCE), |access| {
            let result = module.evaluate(access);
            direct = Some(result.as_ref().unwrap_err().clone());
            result
        })
        .unwrap();
    let actual = outcome.result.unwrap_err();
    assert_eq!(direct.as_ref(), Some(&authored));
    assert_eq!(actual, authored);
    assert_eq!(outcome.report.job_invocations().unwrap(), 0);
    assert!(outcome.report.instructions().unwrap() < ALLOWANCE);
    let Some(EngineError::Panic(panic)) = actual.as_engine() else {
        panic!("the actual post-store engine Panic must remain the first cause")
    };
    assert_eq!(panic.message(), "authored post-storage hook");
    let DiagnosticKind::Native(cause) = panic.diagnostic().unwrap().kind() else {
        panic!("the original native diagnostic source must survive")
    };
    assert_eq!(cause.kind(), DiagnosticNativeKind::Range);
    assert_eq!(cause.message(), "authored hook cause");
    let DiagnosticKind::Opaque(leaf) = cause.cause().unwrap().kind() else {
        panic!("the authored local opaque identity must remain visible")
    };
    assert_eq!(leaf.value().unwrap(), opaque);
    finish_saved_continuation(&executor, &module, &mut context);
}

thread_local! {
    static BODY_HOOK: RefCell<Option<(Rc<PostStoreExecutor>, JsError)>> = const { RefCell::new(None) };
}
struct BodyHook;
impl Drop for BodyHook {
    fn drop(&mut self) {
        BODY_HOOK.with(|slot| drop(slot.borrow_mut().take()));
    }
}
fn arm_from_module_body(
    context: &mut Context,
    executor: Rc<PostStoreExecutor>,
    error: JsError,
) -> BodyHook {
    BODY_HOOK.with(|slot| assert!(slot.replace(Some((executor, error))).is_none()));
    context
        .register_global_builtin_callable(
            js_string!("armPostStoreHook"),
            0,
            surgeist_js::NativeFunction::from_fn_ptr(|_, _, _| {
                BODY_HOOK.with(|slot| {
                    let (executor, error) = slot
                        .borrow_mut()
                        .take()
                        .expect("one authored module-body arm");
                    assert!(executor.hook.replace(Some(error)).is_none());
                });
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
    BodyHook
}
fn dispatch_stored_job(executor: &PostStoreExecutor, context: &mut Context) -> JsResult<JsValue> {
    let job = executor
        .jobs
        .borrow_mut()
        .pop_front()
        .expect("actual staged adapter work remains queued");
    let mut direct = None;
    let outcome = context.with_execution(ExecutionOptions::new(ALLOWANCE), |access| {
        let result = match job {
            Job::PromiseJob(reaction) => match reaction.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!("the fresh operation admits its real stored reaction: {error:?}")
                }
            },
            Job::AsyncJob(mut loader) => {
                let host = access.into_host_access();
                let mut task = TaskContext::from_waker(Waker::noop());
                assert!(matches!(loader.run_step(&mut task, &host), surgeist_js::job::JobStep::Prepared));
                let surgeist_js::job::JobStep::Complete(result) = loader.run_step(&mut task, &host) else {
                    panic!("the in-memory MapModuleLoader completes its actual prepared native task")
                };
                result
            }
            other => panic!("the source adapters produce their real loader/promise wrappers: {other:?}"),
        };
        direct = Some(result.clone());
        result
    }).unwrap();
    assert_eq!(
        outcome.result,
        direct.unwrap(),
        "operation completion preserves the actual callback's first error"
    );
    outcome.result
}

#[derive(Default)]
struct PostStoreWake {
    count: std::sync::Mutex<usize>,
    changed: std::sync::Condvar,
}
impl std::task::Wake for PostStoreWake {
    fn wake(self: std::sync::Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        *self.count.lock().unwrap() += 1;
        self.changed.notify_all();
    }
}
impl PostStoreWake {
    fn after(&self, baseline: usize) {
        let (count, _) = self
            .changed
            .wait_timeout_while(
                self.count.lock().unwrap(),
                std::time::Duration::from_secs(5),
                |count| *count <= baseline,
            )
            .unwrap();
        assert!(
            *count > baseline,
            "the preexisting re-armed adapter waiter must receive its actual settlement notice"
        );
    }
}

fn adapter_post_store_failure(dynamic_import: bool) {
    let executor = Rc::new(PostStoreExecutor::default());
    let loader = Rc::new(surgeist_js::module::MapModuleLoader::new());
    let builder = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_property(js_string!("postStoreEffects"), 0, Attribute::all())
        .unwrap();
    let authored: JsError = PanicError::new("adapter post-storage failure")
        .with_source(JsNativeError::range().with_message("adapter retained cause"))
        .into();
    let _body_hook = arm_from_module_body(&mut context, executor.clone(), authored.clone());
    let module = Module::parse(
        Source::from_bytes(
            "armPostStoreHook(); globalThis.postStoreEffects += 1;
         await 0; globalThis.postStoreEffects += 10; export const answer = 42;",
        ),
        None,
        context.access(),
    )
    .unwrap();
    assert!(loader.insert("post-store-entry", module.clone()).is_none());
    // Loading/linking are real adapter stages. Only the admitted module body
    // arms the hook, immediately before its initial Await's source publication.
    let adapter = if dynamic_import {
        JsPromise::from_object(
            context
                .eval_with_budget(
                    Source::from_bytes("import('post-store-entry')"),
                    ExecutionOptions::new(ALLOWANCE),
                )
                .unwrap()
                .result
                .unwrap()
                .as_object()
                .unwrap(),
        )
        .unwrap()
    } else {
        module.load_link_evaluate(context.access()).unwrap()
    };
    assert!(executor.accepted.borrow().is_none());
    let mut native = adapter.clone().into_js_future(context.access()).unwrap();
    let wake = std::sync::Arc::new(PostStoreWake::default());
    let waker = Waker::from(wake.clone());
    assert!(
        Pin::new(&mut native)
            .poll(&mut TaskContext::from_waker(&waker))
            .is_pending()
    );
    let mut initial_error = None;
    for _ in 0..16 {
        match dispatch_stored_job(&executor, &mut context) {
            Ok(_) => assert!(
                executor.accepted.borrow().is_none(),
                "the hook must expose its actual error at the initiating callback"
            ),
            Err(error) => {
                initial_error = Some(error);
                break;
            }
        }
    }
    assert_eq!(initial_error, Some(authored));
    assert!(BODY_HOOK.with(|slot| slot.borrow().is_none()));
    let observer = executor.accepted.borrow().as_ref().unwrap().clone();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert_eq!(effects(&mut context), JsValue::new(1));
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    // Earlier handoff notices can consume a waker. Re-arm the same actual
    // waiter while still Pending before measuring the later settlement wake.
    assert!(
        Pin::new(&mut native)
            .poll(&mut TaskContext::from_waker(&waker))
            .is_pending()
    );
    let baseline = *wake.count.lock().unwrap();
    let evaluation = fresh_evaluation(&module, &mut context);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    for _ in 0..24 {
        if adapter.state().unwrap() != PromiseState::Pending {
            break;
        }
        dispatch_stored_job(&executor, &mut context).unwrap();
    }
    let PromiseState::Fulfilled(value) = adapter.state().unwrap() else {
        panic!("the original adapter must remain viable after accepted publication")
    };
    if dynamic_import {
        assert_eq!(
            value
                .as_object()
                .unwrap()
                .get(js_string!("answer"), context.access())
                .unwrap(),
            JsValue::new(42)
        );
    } else {
        assert_eq!(value, JsValue::undefined());
    }
    wake.after(baseline);
    assert_eq!(
        Pin::new(&mut native).poll(&mut TaskContext::from_waker(&waker)),
        Poll::Ready(Ok(value))
    );
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(effects(&mut context), JsValue::new(11));
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        fresh_evaluation(&module, &mut context).identity(),
        evaluation.identity()
    );
    assert_eq!(effects(&mut context), JsValue::new(11));
}

#[test]
fn combined_adapter_preserves_its_preexisting_native_waiter_after_initial_await_post_store_failure()
{
    adapter_post_store_failure(false);
}

#[test]
fn dynamic_import_preserves_its_preexisting_native_waiter_after_initial_await_post_store_failure() {
    adapter_post_store_failure(true);
}

#[derive(Default)]
struct UnacceptedExecutor {
    refusal: RefCell<Option<JsError>>,
    observer: RefCell<Option<JobObserver>>,
}
impl JobExecutor for UnacceptedExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Job::PromiseJob(reaction) = &job else {
            panic!("the linked import-free control publishes its actual Await reaction")
        };
        assert!(self.observer.replace(Some(reaction.observer())).is_none());
        let error = self
            .refusal
            .borrow_mut()
            .take()
            .expect("one unaccepted installation");
        // No queue or alternate owner accepts the command. The actual original
        // wrapper is returned to Promise for its owning rollback/disposal path.
        Err(JobPublicationError::new(
            job,
            JobPublicationCause::try_new(error, access).unwrap(),
        ))
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the unaccepted control owns no dispatch queue")
    }
}

#[test]
fn an_actually_unaccepted_initial_await_returns_its_original_job_and_interrupts_without_a_ghost_successor()
 {
    let executor = Rc::new(UnacceptedExecutor::default());
    let builder = Context::builder().job_executor(executor.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_property(js_string!("postStoreEffects"), 0, Attribute::all())
        .unwrap();
    let module = linked(&mut context);
    let authored: JsError = EngineResourceError::Allocation.into();
    *executor.refusal.borrow_mut() = Some(authored.clone());
    assert_eq!(module.evaluate(context.access()).unwrap_err(), authored);
    assert!(executor.refusal.borrow().is_none());
    let observer = executor.observer.borrow().as_ref().unwrap().clone();
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(surgeist_js::job::JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
    assert_eq!(effects(&mut context), JsValue::new(1));
    let first = context
        .with_execution(ExecutionOptions::new(ALLOWANCE), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(first.report.instructions().unwrap(), 0);
    let first = first.result.unwrap_err();
    let Some(EngineError::ModuleInterrupted(interrupted)) = first.as_engine() else {
        panic!("an actually unaccepted Await must not leave a protected ghost suspension")
    };
    assert_eq!(
        interrupted.cause().kind(),
        DiagnosticKind::Engine(surgeist_js::error::DiagnosticEngine::Resource(
            EngineResourceError::Allocation
        ))
    );
    let repeated = context
        .with_execution(ExecutionOptions::new(ALLOWANCE), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(repeated.report.instructions().unwrap(), 0);
    let repeated = repeated.result.unwrap_err();
    let Some(EngineError::ModuleInterrupted(repeated)) = repeated.as_engine() else {
        panic!("the actually interrupted module remains terminal")
    };
    assert_eq!(repeated.identity(), interrupted.identity());
    assert_eq!(effects(&mut context), JsValue::new(1));
}

#[test]
fn a_resumed_module_retains_its_new_stored_await_after_an_independent_post_store_error() {
    let executor = Rc::new(PostStoreExecutor::default());
    let mut context = context(executor.clone());
    let (gate, resolvers) = JsPromise::new_pending(context.access()).unwrap();
    context
        .register_global_property(js_string!("postStoreFirstGate"), gate, Attribute::all())
        .unwrap();
    let module = Module::parse(
        Source::from_bytes(
            "await postStoreFirstGate; globalThis.postStoreEffects += 1;
         await 0; globalThis.postStoreEffects += 10;",
        ),
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
    let evaluation = fresh_evaluation(&module, &mut context);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    assert!(executor.jobs.borrow().is_empty());
    let mut preexisting = evaluation.clone().into_js_future(context.access()).unwrap();
    assert!(
        Pin::new(&mut preexisting)
            .poll(&mut TaskContext::from_waker(Waker::noop()))
            .is_pending()
    );
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
        .unwrap();
    let authored: JsError = EngineResourceError::Allocation.into();
    // The first gate's publication is already complete. Only its admitted
    // callback's new immediate Await can hit the armed post-store hook.
    *executor.hook.borrow_mut() = Some(authored.clone());
    assert_eq!(
        dispatch_stored_job(&executor, &mut context).unwrap_err(),
        authored
    );
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    assert!(
        Pin::new(&mut preexisting)
            .poll(&mut TaskContext::from_waker(Waker::noop()))
            .is_pending()
    );
    assert_eq!(
        fresh_evaluation(&module, &mut context).identity(),
        evaluation.identity()
    );
    finish_saved_continuation(&executor, &module, &mut context);
    assert_eq!(
        Pin::new(&mut preexisting).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Ok(JsValue::undefined()))
    );
    assert_eq!(
        fresh_evaluation(&module, &mut context).identity(),
        evaluation.identity()
    );
    assert_eq!(effects(&mut context), JsValue::new(11));
}
