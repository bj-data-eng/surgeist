use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeError, JsObject, JsResult, JsValue, Source,
    builtins::promise::{OperationType, Promise, PromiseState},
    context::HostHooks,
    error::PanicError,
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobStatus,
    },
    js_string,
    object::builtins::{JsAsyncGenerator, JsFuture, JsPromise},
    property::Attribute,
};

const FINITE: u64 = 100_000;
enum Hook {
    Store(JsError),
    Inline(JsError),
}
#[derive(Default)]
struct AdapterExecutor {
    queue: RefCell<VecDeque<Job>>,
    hook: RefCell<Option<Hook>>,
    observed: RefCell<Option<JobObserver>>,
}
impl AdapterExecutor {
    fn arm(&self, hook: Hook) {
        assert!(self.hook.replace(Some(hook)).is_none());
        self.observed.borrow_mut().take();
    }
    fn observer(&self) -> JobObserver {
        self.observed.borrow().as_ref().unwrap().clone()
    }
    fn invoke(job: Job, access: ContextAccess<'_>) -> JsResult<JsValue> {
        let Job::PromiseJob(reaction) = job else {
            panic!("the real adapter publishes a promise reaction")
        };
        match reaction.call(access) {
            JobCall::Complete(result) => result,
            JobCall::Deferred { job, error } => {
                drop(job);
                panic!(
                    "the configured host hook follows actual callback entry, never Deferred: {error:?}"
                )
            }
        }
    }
    fn step(&self, context: &mut Context) -> JsResult<JsValue> {
        let job = self
            .queue
            .borrow_mut()
            .pop_front()
            .expect("the real accepted wrapper remains queued");
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
    fn finish(&self, promises: &[&JsPromise], context: &mut Context) {
        for _ in 0..32 {
            if promises
                .iter()
                .all(|promise| promise.state().unwrap() != PromiseState::Pending)
            {
                return;
            }
            self.step(context).unwrap();
        }
        assert!(
            promises
                .iter()
                .all(|promise| promise.state().unwrap() != PromiseState::Pending),
            "the actual finite adapter chain must finish"
        );
    }
}
impl JobExecutor for AdapterExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Some(hook) = self.hook.borrow_mut().take() else {
            self.queue.borrow_mut().push_back(job);
            return Ok(JobAcceptance::Accepted);
        };
        let Job::PromiseJob(reaction) = &job else {
            panic!("only the selected actual promise reaction receives the hook")
        };
        *self.observed.borrow_mut() = Some(reaction.observer());
        let error = match hook {
            Hook::Store(error) => {
                self.queue.borrow_mut().push_back(job);
                error
            }
            Hook::Inline(error) => {
                Self::invoke(job, access.reborrow()).unwrap();
                error
            }
        };
        Ok(JobAcceptance::AcceptedWithError(
            JobPublicationCause::try_new(error, access).unwrap(),
        ))
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture explicitly dispatches real saved wrappers")
    }
}
#[derive(Default)]
struct Rejections(Cell<usize>);
impl HostHooks for Rejections {
    fn promise_rejection_tracker(
        &self,
        _: &JsObject<Promise>,
        operation: OperationType,
        _: ContextAccess<'_>,
    ) {
        if operation == OperationType::Reject {
            self.0.set(self.0.get() + 1);
        }
    }
}
fn context(executor: Rc<AdapterExecutor>, rejections: Rc<Rejections>) -> Context {
    let builder = Context::builder()
        .job_executor(executor)
        .host_hooks(rejections);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    for name in [
        "adapterEffects",
        "adapterFinally",
        "adapterClosed",
        "adapterNext",
        "adapterMapped",
    ] {
        context
            .register_global_property(surgeist_js::JsString::from(name), 0, Attribute::all())
            .unwrap();
    }
    context
}
fn error() -> JsError {
    PanicError::new("authored adapter post-publication failure")
        .with_source(JsNativeError::range().with_message("authored adapter cause"))
        .into()
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
        .unwrap()
        .get(surgeist_js::JsString::from(name), context.access())
        .unwrap()
}
fn generator(source: &str, context: &mut Context) -> JsAsyncGenerator {
    JsAsyncGenerator::from_object(eval(source, context).unwrap().as_object().unwrap()).unwrap()
}
fn request(
    context: &mut Context,
    run: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsPromise>,
) -> JsResult<JsPromise> {
    let mut direct = None;
    let outcome = context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            let result = run(access);
            direct = Some(result.clone());
            result
        })
        .unwrap();
    match (&outcome.result, direct.as_ref().unwrap()) {
        (Ok(result), Ok(direct)) => assert_eq!(result.identity(), direct.identity()),
        (Err(result), Err(direct)) => assert_eq!(result, direct),
        _ => panic!("the operation must preserve the actual admitted request result"),
    }
    outcome.result
}
fn native(promise: &JsPromise, context: &mut Context) -> JsFuture {
    let mut future = promise.clone().into_js_future(context.access()).unwrap();
    assert!(poll(&mut future).is_pending());
    future
}
fn poll(future: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn result(promise: &JsPromise, expected: JsValue, done: bool, context: &mut Context) -> JsValue {
    let PromiseState::Fulfilled(record) = promise.state().unwrap() else {
        panic!("the actual request must fulfill without a fabricated rejection")
    };
    let object = record.as_object().unwrap();
    assert_eq!(
        object.get(js_string!("value"), context.access()).unwrap(),
        expected
    );
    assert_eq!(
        object.get(js_string!("done"), context.access()).unwrap(),
        JsValue::new(done)
    );
    record
}

#[test]
fn async_generator_await_return_keeps_its_accepted_request_and_drains_following_requests_once() {
    let executor = Rc::new(AdapterExecutor::default());
    let rejections = Rc::new(Rejections::default());
    let mut context = context(executor.clone(), rejections.clone());
    let generator = generator(
        "(async function* () { globalThis.adapterEffects += 100; yield 1; })()",
        &mut context,
    );
    executor.arm(Hook::Store(error()));
    assert_eq!(
        request(&mut context, |access| generator.r#return(41, access)).unwrap_err(),
        error()
    );
    let observed = executor.observer();
    assert_eq!(observed.status(), Ok(JobStatus::Pending));
    assert_eq!(observed.fault(), None);
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(0));
    let following_next = request(&mut context, |access| {
        generator.next(JsValue::undefined(), access)
    })
    .unwrap();
    let following_return = request(&mut context, |access| generator.r#return(42, access)).unwrap();
    let mut next_wait = native(&following_next, &mut context);
    let mut return_wait = native(&following_return, &mut context);
    executor.finish(&[&following_next, &following_return], &mut context);
    let next_result = result(&following_next, JsValue::undefined(), true, &mut context);
    let return_result = result(&following_return, JsValue::new(42), true, &mut context);
    assert_eq!(poll(&mut next_wait), Poll::Ready(Ok(next_result)));
    assert_eq!(poll(&mut return_wait), Poll::Ready(Ok(return_result)));
    assert_eq!(observed.status(), Ok(JobStatus::Completed));
    assert_eq!(observed.fault(), None);
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(0));
    assert_eq!(rejections.0.get(), 0);
}

#[test]
fn async_generator_finish_resume_preserves_the_stored_await_and_its_original_front_request() {
    let executor = Rc::new(AdapterExecutor::default());
    let rejections = Rc::new(Rejections::default());
    let mut context = context(executor.clone(), rejections.clone());
    let generator = generator(
        "(async function* () {
            globalThis.adapterEffects += 1;
            try { await 0; yield 2; globalThis.adapterEffects += 10; return 3; }
            finally { globalThis.adapterFinally += 1; }
         })()",
        &mut context,
    );
    executor.arm(Hook::Store(error()));
    assert_eq!(
        request(&mut context, |access| generator
            .next(JsValue::undefined(), access))
        .unwrap_err(),
        error()
    );
    let observed = executor.observer();
    assert_eq!(observed.status(), Ok(JobStatus::Pending));
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(1));
    assert_eq!(value("adapterFinally", &mut context), JsValue::new(0));
    let following = request(&mut context, |access| {
        generator.next(JsValue::undefined(), access)
    })
    .unwrap();
    let mut wait = native(&following, &mut context);
    executor.finish(&[&following], &mut context);
    let record = result(&following, JsValue::new(3), true, &mut context);
    assert_eq!(poll(&mut wait), Poll::Ready(Ok(record)));
    assert_eq!(observed.status(), Ok(JobStatus::Completed));
    assert_eq!(observed.fault(), None);
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(11));
    assert_eq!(value("adapterFinally", &mut context), JsValue::new(1));
    let completed = request(&mut context, |access| {
        generator.next(JsValue::undefined(), access)
    })
    .unwrap();
    result(&completed, JsValue::undefined(), true, &mut context);
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(11));
    assert_eq!(rejections.0.get(), 0);
}

#[test]
fn async_generator_inline_consumption_keeps_a_newer_pending_await_after_the_enclosing_hook_error() {
    let executor = Rc::new(AdapterExecutor::default());
    let rejections = Rc::new(Rejections::default());
    let mut context = context(executor.clone(), rejections.clone());
    let (gate, resolvers) = JsPromise::new_pending(context.access()).unwrap();
    context
        .register_global_property(js_string!("adapterGate"), gate.clone(), Attribute::all())
        .unwrap();
    let generator = generator(
        "(async function* () {
            globalThis.adapterEffects += 1; await 0;
            globalThis.adapterEffects += 10; await adapterGate;
            yield 2; globalThis.adapterEffects += 100; return 3;
         })()",
        &mut context,
    );
    executor.arm(Hook::Inline(error()));
    assert_eq!(
        request(&mut context, |access| generator
            .next(JsValue::undefined(), access))
        .unwrap_err(),
        error()
    );
    let observed = executor.observer();
    assert_eq!(observed.status(), Ok(JobStatus::Completed));
    assert_eq!(observed.fault(), None);
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(11));
    assert_eq!(gate.state().unwrap(), PromiseState::Pending);
    let following = request(&mut context, |access| {
        generator.next(JsValue::undefined(), access)
    })
    .unwrap();
    let mut wait = native(&following, &mut context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
        .unwrap();
    executor.finish(&[&following], &mut context);
    let record = result(&following, JsValue::new(3), true, &mut context);
    assert_eq!(poll(&mut wait), Poll::Ready(Ok(record)));
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(111));
    assert_eq!(observed.status(), Ok(JobStatus::Completed));
    assert_eq!(observed.fault(), None);
    assert_eq!(rejections.0.get(), 0);
}

#[test]
fn async_from_sync_acceptance_does_not_close_the_iterator_or_reject_its_owned_result() {
    let executor = Rc::new(AdapterExecutor::default());
    let rejections = Rc::new(Rejections::default());
    let mut context = context(executor.clone(), rejections.clone());
    eval(
        "globalThis.adapterIterable = {
            [Symbol.iterator]() { return this; },
            next() { globalThis.adapterNext += 1; return { value: Promise.resolve(5), done: false }; },
            return() { globalThis.adapterClosed += 1; return { done: true }; }
         };", &mut context).unwrap();
    executor.arm(Hook::Store(error()));
    assert_eq!(eval(
        "(async function consumeSync() {
            try { for await (const value of adapterIterable) { globalThis.adapterEffects += value; break; } }
            finally { globalThis.adapterFinally += 1; }
         })()", &mut context).unwrap_err(), error());
    let observed = executor.observer();
    assert_eq!(observed.status(), Ok(JobStatus::Pending));
    assert_eq!(value("adapterNext", &mut context), JsValue::new(1));
    assert_eq!(value("adapterClosed", &mut context), JsValue::new(0));
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(0));
    assert_eq!(value("adapterFinally", &mut context), JsValue::new(0));
    assert_eq!(rejections.0.get(), 0);
    executor.step(&mut context).unwrap();
    assert_eq!(observed.status(), Ok(JobStatus::Completed));
    assert_eq!(observed.fault(), None);
    assert!(executor.queue.borrow().is_empty());
    assert_eq!(value("adapterNext", &mut context), JsValue::new(1));
    assert_eq!(value("adapterClosed", &mut context), JsValue::new(0));
    assert_eq!(value("adapterEffects", &mut context), JsValue::new(0));
    assert_eq!(value("adapterFinally", &mut context), JsValue::new(0));
    assert_eq!(rejections.0.get(), 0);
    assert_eq!(eval("40 + 2", &mut context).unwrap(), JsValue::new(42));
}

#[cfg(feature = "experimental")]
#[test]
fn a_native_array_coroutine_keeps_its_recursive_stored_await_after_the_post_store_hook_error() {
    let executor = Rc::new(AdapterExecutor::default());
    let rejections = Rc::new(Rejections::default());
    let mut context = context(executor.clone(), rejections.clone());
    let (gate, resolvers) = JsPromise::new_pending(context.access()).unwrap();
    context
        .register_global_property(js_string!("adapterGate"), gate, Attribute::all())
        .unwrap();
    let array_result = eval(
        "Array.fromAsync({ 0: adapterGate, length: 1 }, function mapNative(value) {
            globalThis.adapterMapped += 1; return value + 1;
         })",
        &mut context,
    )
    .unwrap();
    let promise = JsPromise::from_object(array_result.as_object().unwrap()).unwrap();
    let mut wait = native(&promise, &mut context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(41)], context.access())
        .unwrap();
    executor.arm(Hook::Store(error()));
    assert_eq!(executor.step(&mut context).unwrap_err(), error());
    let observed = executor.observer();
    assert_eq!(observed.status(), Ok(JobStatus::Pending));
    assert_eq!(observed.fault(), None);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    assert!(poll(&mut wait).is_pending());
    assert_eq!(value("adapterMapped", &mut context), JsValue::new(1));
    assert_eq!(rejections.0.get(), 0);
    executor.finish(&[&promise], &mut context);
    let PromiseState::Fulfilled(array) = promise.state().unwrap() else {
        panic!("the original native coroutine result must finish normally")
    };
    assert_eq!(
        array
            .as_object()
            .unwrap()
            .get(js_string!("0"), context.access())
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        array
            .as_object()
            .unwrap()
            .get(js_string!("length"), context.access())
            .unwrap(),
        JsValue::new(1)
    );
    assert_eq!(poll(&mut wait), Poll::Ready(Ok(array)));
    assert_eq!(observed.status(), Ok(JobStatus::Completed));
    assert_eq!(observed.fault(), None);
    assert_eq!(value("adapterMapped", &mut context), JsValue::new(1));
    assert_eq!(rejections.0.get(), 0);
    assert_eq!(eval("40 + 2", &mut context).unwrap(), JsValue::new(42));
}
