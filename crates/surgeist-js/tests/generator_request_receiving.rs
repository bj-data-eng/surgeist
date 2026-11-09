#![allow(unused_crate_dependencies)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeErrorKind, JsObject, JsResult, JsValue, Source,
    builtins::promise::PromiseState,
    execution::ExecutionOptions,
    job::{Job, JobCall, JobExecutor},
    js_string,
    object::builtins::{JsAsyncGenerator, JsGenerator, JsPromise},
};

const LIMIT: u64 = 100_000;

#[derive(Clone, Copy, Debug)]
enum Request {
    Next,
    Return,
    Throw,
}

impl Request {
    fn call_sync(
        self,
        generator: &JsGenerator,
        value: JsValue,
        access: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        match self {
            Self::Next => generator.next(value, access),
            Self::Return => generator.r#return(value, access),
            Self::Throw => generator.throw(value, access),
        }
    }

    fn call_async(
        self,
        generator: &JsAsyncGenerator,
        value: JsValue,
        access: ContextAccess<'_>,
    ) -> JsResult<JsPromise> {
        match self {
            Self::Next => generator.next(value, access),
            Self::Return => generator.r#return(value, access),
            Self::Throw => generator.throw(value, access),
        }
    }
}

#[derive(Default)]
struct ManualExecutor(RefCell<VecDeque<Job>>);

impl JobExecutor for ManualExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, _: ContextAccess<'_>) {
        self.0.borrow_mut().push_back(job);
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches captured jobs explicitly")
    }
}

impl ManualExecutor {
    fn settle(&self, promise: &JsPromise, context: &mut Context) -> JsValue {
        for _ in 0..32 {
            match promise.state().unwrap() {
                PromiseState::Fulfilled(value) => return value,
                PromiseState::Rejected(reason) => panic!("accepted request rejected: {reason:?}"),
                PromiseState::Pending => {}
            }
            let queued = self.0.borrow_mut().pop_front();
            let Some(Job::PromiseJob(job)) = queued else {
                panic!("the ready request must supply a real promise job: {queued:?}")
            };
            in_scope(context, |access| match job.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { job, error } => {
                    self.0.borrow_mut().push_front(job.into());
                    Err(error)
                }
            })
            .unwrap();
        }
        panic!("request remained pending after thirty-two bounded dispatches")
    }
}

fn context() -> (Context, Rc<ManualExecutor>) {
    let executor = Rc::new(ManualExecutor::default());
    let builder = Context::builder().job_executor(executor.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    (builder.build().unwrap(), executor)
}

fn peer_context(owner: &Context, executor: Rc<ManualExecutor>) -> Context {
    let builder = Context::builder()
        .runtime(owner.runtime().clone())
        .job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

fn in_scope<T: Debug>(
    context: &mut Context,
    call: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
) -> JsResult<T> {
    let mut actual = None;
    let outcome = context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            actual = Some(call(access));
            Ok(())
        })
        .unwrap();
    assert!(
        outcome.result.is_ok(),
        "operation: {:?}; direct entry: {actual:?}",
        outcome.result
    );
    actual.expect("the public entry must return its actual result")
}

fn eval(context: &mut Context, source: &str) -> JsValue {
    in_scope(context, |mut access| {
        access.eval(Source::from_bytes(source))
    })
    .unwrap()
}

fn sync_generator(context: &mut Context) -> JsGenerator {
    let value = eval(
        context,
        "globalThis.starts = 0; globalThis.resumes = 0;
         globalThis.catches = 0; globalThis.closes = 0;
         (function* () {
             starts += 1;
             try { const value = yield 10; resumes += 1; yield value; return 30; }
             catch (error) { catches += 1; yield error; return 31; }
             finally { closes += 1; }
         })()",
    );
    JsGenerator::from_object(value.as_object().unwrap()).unwrap()
}

fn async_generator(context: &mut Context, gated: bool) -> JsAsyncGenerator {
    eval(context,
        "globalThis.starts = 0; globalThis.resumes = 0;
         globalThis.catches = 0; globalThis.closes = 0;
         globalThis.requestGate = new Promise(resolve => { globalThis.releaseRequestGate = resolve; });",
    );
    let source = if gated {
        "(async function* () {
             starts += 1;
             try { await requestGate; const value = yield 10; resumes += 1; yield value; return 30; }
             catch (error) { catches += 1; yield error; return 31; }
             finally { closes += 1; }
         })()"
    } else {
        "(async function* () {
             starts += 1;
             try { const value = yield 10; resumes += 1; yield value; return 30; }
             catch (error) { catches += 1; yield error; return 31; }
             finally { closes += 1; }
         })()"
    };
    JsAsyncGenerator::from_object(eval(context, source).as_object().unwrap()).unwrap()
}

fn assert_foreign_runtime(error: JsError) {
    let native = error
        .as_native()
        .expect("runtime admission returns a native error");
    assert_eq!(native.kind(), &JsNativeErrorKind::Type);
    assert_eq!(native.message(), "object belongs to a different runtime");
}

fn assert_result(value: JsValue, expected: JsValue, done: bool, context: &mut Context) {
    let result = value.as_object().unwrap();
    assert_eq!(
        result.get(js_string!("value"), context.access()).unwrap(),
        expected
    );
    assert_eq!(
        result.get(js_string!("done"), context.access()).unwrap(),
        JsValue::new(done)
    );
}

fn assert_counts(context: &mut Context, starts: i32, resumes: i32, catches: i32, closes: i32) {
    for (name, value) in [
        ("starts", starts),
        ("resumes", resumes),
        ("catches", catches),
        ("closes", closes),
    ] {
        assert_eq!(
            context
                .global_object()
                .get(surgeist_js::JsString::from(name), context.access())
                .unwrap(),
            JsValue::new(value),
            "{name}"
        );
    }
}

fn sync_request(
    request: Request,
    generator: &JsGenerator,
    value: JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    in_scope(context, |access| {
        request.call_sync(generator, value, access)
    })
}

fn async_request(
    request: Request,
    generator: &JsAsyncGenerator,
    value: JsValue,
    context: &mut Context,
) -> JsResult<JsPromise> {
    in_scope(context, |access| {
        request.call_async(generator, value, access)
    })
}

fn async_result(
    request: Request,
    generator: &JsAsyncGenerator,
    value: JsValue,
    executor: &ManualExecutor,
    context: &mut Context,
) -> JsValue {
    let promise = async_request(request, generator, value, context).unwrap();
    executor.settle(&promise, context)
}

#[test]
fn sync_requests_refuse_a_foreign_receiving_context_before_consuming_start_state() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut owner, _) = context();
        let (mut receiver, _) = context();
        let generator = sync_generator(&mut owner);
        assert_foreign_runtime(
            sync_request(request, &generator, 42.into(), &mut receiver).unwrap_err(),
        );
        assert_counts(&mut owner, 0, 0, 0, 0);
        let first =
            sync_request(Request::Next, &generator, JsValue::undefined(), &mut owner).unwrap();
        assert_result(first, 10.into(), false, &mut owner);
        assert_counts(&mut owner, 1, 0, 0, 0);
    }
}

#[test]
fn async_requests_refuse_a_foreign_receiving_context_before_consuming_start_state() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut owner, executor) = context();
        let (mut receiver, _) = context();
        let generator = async_generator(&mut owner, false);
        assert_foreign_runtime(
            async_request(request, &generator, 42.into(), &mut receiver).unwrap_err(),
        );
        assert_counts(&mut owner, 0, 0, 0, 0);
        let first = async_result(
            Request::Next,
            &generator,
            JsValue::undefined(),
            &executor,
            &mut owner,
        );
        assert_result(first, 10.into(), false, &mut owner);
        assert_counts(&mut owner, 1, 0, 0, 0);
    }
}

#[test]
fn sync_requests_refuse_foreign_payloads_before_resuming_or_replacing_a_yield() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut owner, _) = context();
        let (foreign, _) = context();
        let payload: JsValue = JsObject::with_null_proto(foreign.runtime()).into();
        let generator = sync_generator(&mut owner);
        let first =
            sync_request(Request::Next, &generator, JsValue::undefined(), &mut owner).unwrap();
        assert_result(first, 10.into(), false, &mut owner);
        assert_foreign_runtime(sync_request(request, &generator, payload, &mut owner).unwrap_err());
        assert_counts(&mut owner, 1, 0, 0, 0);
        let second = sync_request(Request::Next, &generator, 20.into(), &mut owner).unwrap();
        assert_result(second, 20.into(), false, &mut owner);
        let completed =
            sync_request(Request::Next, &generator, JsValue::undefined(), &mut owner).unwrap();
        assert_result(completed, 30.into(), true, &mut owner);
        assert_counts(&mut owner, 1, 1, 0, 1);
    }
}

#[test]
fn async_requests_refuse_foreign_payloads_without_adding_a_waiting_request() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut owner, executor) = context();
        let (foreign, _) = context();
        let payload: JsValue = JsObject::with_null_proto(foreign.runtime()).into();
        let generator = async_generator(&mut owner, true);
        let first =
            async_request(Request::Next, &generator, JsValue::undefined(), &mut owner).unwrap();
        assert_eq!(first.state().unwrap(), PromiseState::Pending);
        assert_foreign_runtime(
            async_request(request, &generator, payload, &mut owner).unwrap_err(),
        );
        assert_counts(&mut owner, 1, 0, 0, 0);
        eval(&mut owner, "releaseRequestGate(1)");
        let first_result = executor.settle(&first, &mut owner);
        assert_result(first_result, 10.into(), false, &mut owner);
        assert_counts(&mut owner, 1, 0, 0, 0);
        let second = async_result(Request::Next, &generator, 20.into(), &executor, &mut owner);
        assert_result(second, 20.into(), false, &mut owner);
        let completed = async_result(
            Request::Next,
            &generator,
            JsValue::undefined(),
            &executor,
            &mut owner,
        );
        assert_result(completed, 30.into(), true, &mut owner);
        assert_counts(&mut owner, 1, 1, 0, 1);
    }
}

#[test]
fn sync_requests_accept_a_same_runtime_peer_and_preserve_its_object_payload() {
    let (mut owner, executor) = context();
    let mut peer = peer_context(&owner, executor);
    let peer_realm = peer.realm().clone();
    let payload: JsValue = JsObject::with_null_proto(peer.runtime()).into();
    let generator = sync_generator(&mut owner);
    let first = sync_request(Request::Next, &generator, JsValue::undefined(), &mut peer).unwrap();
    assert_result(first, 10.into(), false, &mut peer);
    let echoed = sync_request(Request::Next, &generator, payload.clone(), &mut peer).unwrap();
    assert_result(echoed, payload.clone(), false, &mut peer);
    let caught = sync_request(Request::Throw, &generator, payload.clone(), &mut peer).unwrap();
    assert_result(caught, payload.clone(), false, &mut peer);
    let returned = sync_request(Request::Return, &generator, payload.clone(), &mut peer).unwrap();
    assert_result(returned, payload, true, &mut peer);
    assert_eq!(peer.realm(), &peer_realm);
    assert_counts(&mut owner, 1, 1, 1, 1);
}

#[test]
fn async_requests_accept_a_same_runtime_peer_and_preserve_its_object_payload() {
    let (mut owner, executor) = context();
    let mut peer = peer_context(&owner, executor.clone());
    let peer_realm = peer.realm().clone();
    let payload: JsValue = JsObject::with_null_proto(peer.runtime()).into();
    let generator = async_generator(&mut owner, false);
    let first = async_result(
        Request::Next,
        &generator,
        JsValue::undefined(),
        &executor,
        &mut peer,
    );
    assert_result(first, 10.into(), false, &mut peer);
    let echoed = async_result(
        Request::Next,
        &generator,
        payload.clone(),
        &executor,
        &mut peer,
    );
    assert_result(echoed, payload.clone(), false, &mut peer);
    let caught = async_result(
        Request::Throw,
        &generator,
        payload.clone(),
        &executor,
        &mut peer,
    );
    assert_result(caught, payload.clone(), false, &mut peer);
    let returned = async_result(
        Request::Return,
        &generator,
        payload.clone(),
        &executor,
        &mut peer,
    );
    assert_result(returned, payload, true, &mut peer);
    assert_eq!(peer.realm(), &peer_realm);
    assert_counts(&mut owner, 1, 1, 1, 1);
}
