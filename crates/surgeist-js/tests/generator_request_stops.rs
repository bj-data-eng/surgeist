#![allow(unused_crate_dependencies)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};

use surgeist_js::{
    Context, ContextAccess, JsResult, JsValue, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{Job, JobAcceptance, JobCall, JobExecutor, JobPublicationError},
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
    fn call_sync(self, generator: &JsGenerator, access: ContextAccess<'_>) -> JsResult<JsValue> {
        match self {
            Self::Next => generator.next(JsValue::undefined(), access),
            Self::Return => generator.r#return(91, access),
            Self::Throw => generator.throw(42, access),
        }
    }

    fn call_async(
        self,
        generator: &JsAsyncGenerator,
        access: ContextAccess<'_>,
    ) -> JsResult<JsPromise> {
        match self {
            Self::Next => generator.next(JsValue::undefined(), access),
            Self::Return => generator.r#return(91, access),
            Self::Throw => generator.throw(42, access),
        }
    }
}

#[derive(Default)]
struct ManualExecutor(RefCell<VecDeque<Job>>);

impl JobExecutor for ManualExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.0.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches captured jobs explicitly")
    }
}

impl ManualExecutor {
    fn settle(&self, promise: &JsPromise, context: &mut Context) -> PromiseState {
        for _ in 0..32 {
            let state = promise.state().unwrap();
            if state != PromiseState::Pending {
                return state;
            }
            let queued = self.0.borrow_mut().pop_front();
            let Some(Job::PromiseJob(job)) = queued else {
                panic!("a ready async request must supply its promise job: {queued:?}")
            };
            context
                .with_execution(ExecutionOptions::new(LIMIT), |access| {
                    match job.call(access) {
                        JobCall::Complete(result) => result,
                        JobCall::Deferred { job, error } => {
                            self.0.borrow_mut().push_front(job.into());
                            Err(error)
                        }
                    }
                })
                .unwrap()
                .result
                .unwrap();
        }
        panic!("async request remained pending after thirty-two bounded dispatches")
    }
}

fn context() -> (Context, Rc<ManualExecutor>) {
    let executor = Rc::new(ManualExecutor::default());
    let builder = Context::builder().job_executor(executor.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    (builder.build().unwrap(), executor)
}

fn eval(context: &mut Context, source: &str) -> JsValue {
    context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(LIMIT))
        .unwrap()
        .result
        .unwrap()
}

fn sync_generator(context: &mut Context) -> JsGenerator {
    let value = eval(
        context,
        "globalThis.starts = 0; globalThis.resumes = 0; globalThis.closes = 0;
         (function* () {
             starts += 1;
             try { yield 10; resumes += 1; yield 20; return 30; }
             finally { closes += 1; }
         })()",
    );
    JsGenerator::from_object(value.as_object().unwrap()).unwrap()
}

fn async_generator(context: &mut Context, gated: bool) -> JsAsyncGenerator {
    let source = if gated {
        "globalThis.starts = 0; globalThis.resumes = 0; globalThis.closes = 0;
         globalThis.requestGate = new Promise(resolve => { globalThis.releaseRequestGate = resolve; });
         (async function* () {
             starts += 1;
             try { await requestGate; yield 10; resumes += 1; yield 20; return 30; }
             finally { closes += 1; }
         })()"
    } else {
        "globalThis.starts = 0; globalThis.resumes = 0; globalThis.closes = 0;
         (async function* () {
             starts += 1;
             try { yield 10; resumes += 1; yield 20; return 30; }
             finally { closes += 1; }
         })()"
    };
    JsAsyncGenerator::from_object(eval(context, source).as_object().unwrap()).unwrap()
}

fn after_retained_stop<T: Debug>(
    context: &mut Context,
    request: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
) {
    let mut direct = None;
    let mut first = None;
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let error = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            first = Some(error.clone());
            // Capture direct entry independently of the outer first-stop result.
            direct = Some(request(access));
            Ok(())
        })
        .unwrap();
    let first = first.unwrap();
    assert_eq!(outcome.result.unwrap_err(), first);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    let direct = direct.expect("the direct request must actually be attempted");
    assert_eq!(
        direct.as_ref().err(),
        Some(&first),
        "direct request: {direct:?}"
    );
}

fn next_sync(generator: &JsGenerator, context: &mut Context) -> JsValue {
    context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            Request::Next.call_sync(generator, access)
        })
        .unwrap()
        .result
        .unwrap()
}

fn request_async(
    request: Request,
    generator: &JsAsyncGenerator,
    context: &mut Context,
) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            request.call_async(generator, access)
        })
        .unwrap()
        .result
        .unwrap()
}

fn assert_result(value: JsValue, expected: JsValue, done: bool, context: &mut Context) {
    let object = value
        .as_object()
        .expect("a request returns an iterator result");
    assert_eq!(
        object.get(js_string!("value"), context.access()).unwrap(),
        expected
    );
    assert_eq!(
        object.get(js_string!("done"), context.access()).unwrap(),
        JsValue::new(done)
    );
}

fn assert_async_result(
    executor: &ManualExecutor,
    promise: &JsPromise,
    expected: JsValue,
    done: bool,
    context: &mut Context,
) {
    let PromiseState::Fulfilled(value) = executor.settle(promise, context) else {
        panic!("the accepted request must fulfill its iterator result")
    };
    assert_result(value, expected, done, context);
}

fn assert_counts(context: &mut Context, starts: i32, resumes: i32, closes: i32) {
    for (name, expected) in [("starts", starts), ("resumes", resumes), ("closes", closes)] {
        assert_eq!(
            context
                .global_object()
                .get(surgeist_js::JsString::from(name), context.access())
                .unwrap(),
            JsValue::new(expected),
            "{name}"
        );
    }
}

#[test]
fn retained_stop_preserves_a_sync_generators_unstarted_body_before_any_new_request() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut context, _) = context();
        let generator = sync_generator(&mut context);
        after_retained_stop(&mut context, |access| request.call_sync(&generator, access));
        assert_counts(&mut context, 0, 0, 0);
        let first = next_sync(&generator, &mut context);
        assert_result(first, 10.into(), false, &mut context);
        assert_counts(&mut context, 1, 0, 0);
    }
}

#[test]
fn retained_stop_preserves_a_sync_yield_before_return_or_throw_requests() {
    for request in [Request::Return, Request::Throw] {
        let (mut context, _) = context();
        let generator = sync_generator(&mut context);
        let first = next_sync(&generator, &mut context);
        assert_result(first, 10.into(), false, &mut context);
        after_retained_stop(&mut context, |access| request.call_sync(&generator, access));
        assert_counts(&mut context, 1, 0, 0);
        let second = next_sync(&generator, &mut context);
        assert_result(second, 20.into(), false, &mut context);
        let final_result = next_sync(&generator, &mut context);
        assert_result(final_result, 30.into(), true, &mut context);
        assert_counts(&mut context, 1, 1, 1);
    }
}

#[test]
fn retained_stop_precedes_a_completed_sync_next_result() {
    let (mut context, _) = context();
    let generator = sync_generator(&mut context);
    for (value, done) in [(10, false), (20, false), (30, true)] {
        let result = next_sync(&generator, &mut context);
        assert_result(result, value.into(), done, &mut context);
    }
    assert_counts(&mut context, 1, 1, 1);
    after_retained_stop(&mut context, |access| {
        Request::Next.call_sync(&generator, access)
    });
    let completed = next_sync(&generator, &mut context);
    assert_result(completed, JsValue::undefined(), true, &mut context);
    assert_counts(&mut context, 1, 1, 1);
}

#[test]
fn retained_stop_preserves_an_async_generators_unstarted_body_before_any_new_request() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut context, executor) = context();
        let generator = async_generator(&mut context, false);
        after_retained_stop(&mut context, |access| {
            request.call_async(&generator, access)
        });
        assert_counts(&mut context, 0, 0, 0);
        let first = request_async(Request::Next, &generator, &mut context);
        assert_async_result(&executor, &first, 10.into(), false, &mut context);
        assert_counts(&mut context, 1, 0, 0);
    }
}

#[test]
fn retained_stop_preserves_an_async_yield_before_a_return_request() {
    let (mut context, executor) = context();
    let generator = async_generator(&mut context, false);
    let first = request_async(Request::Next, &generator, &mut context);
    assert_async_result(&executor, &first, 10.into(), false, &mut context);
    after_retained_stop(&mut context, |access| {
        Request::Return.call_async(&generator, access)
    });
    assert_counts(&mut context, 1, 0, 0);
    let second = request_async(Request::Next, &generator, &mut context);
    assert_async_result(&executor, &second, 20.into(), false, &mut context);
    let final_result = request_async(Request::Next, &generator, &mut context);
    assert_async_result(&executor, &final_result, 30.into(), true, &mut context);
    assert_counts(&mut context, 1, 1, 1);
}

#[test]
fn retained_stop_precedes_a_completed_async_throw_rejection() {
    let (mut context, executor) = context();
    let generator = async_generator(&mut context, false);
    for (value, done) in [(10, false), (20, false), (30, true)] {
        let result = request_async(Request::Next, &generator, &mut context);
        assert_async_result(&executor, &result, value.into(), done, &mut context);
    }
    after_retained_stop(&mut context, |access| {
        Request::Throw.call_async(&generator, access)
    });
    let completed = request_async(Request::Next, &generator, &mut context);
    assert_async_result(
        &executor,
        &completed,
        JsValue::undefined(),
        true,
        &mut context,
    );
    assert_counts(&mut context, 1, 1, 1);
}

#[test]
fn retained_stop_adds_no_async_request_behind_a_waiting_body() {
    for request in [Request::Next, Request::Return, Request::Throw] {
        let (mut context, executor) = context();
        let generator = async_generator(&mut context, true);
        let first = request_async(Request::Next, &generator, &mut context);
        assert_eq!(first.state().unwrap(), PromiseState::Pending);
        assert_counts(&mut context, 1, 0, 0);
        after_retained_stop(&mut context, |access| {
            request.call_async(&generator, access)
        });
        assert_counts(&mut context, 1, 0, 0);
        eval(&mut context, "releaseRequestGate(1)");
        assert_async_result(&executor, &first, 10.into(), false, &mut context);
        assert_counts(&mut context, 1, 0, 0);
        let second = request_async(Request::Next, &generator, &mut context);
        assert_async_result(&executor, &second, 20.into(), false, &mut context);
        let final_result = request_async(Request::Next, &generator, &mut context);
        assert_async_result(&executor, &final_result, 30.into(), true, &mut context);
        assert_counts(&mut context, 1, 1, 1);
    }
}

#[test]
fn ordinary_sync_return_and_completed_throw_keep_guest_results() {
    let (mut context, _) = context();
    let generator = sync_generator(&mut context);
    let first = next_sync(&generator, &mut context);
    assert_result(first, 10.into(), false, &mut context);
    let returned = context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            Request::Return.call_sync(&generator, access)
        })
        .unwrap()
        .result
        .unwrap();
    assert_result(returned, 91.into(), true, &mut context);
    let thrown = context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            Request::Throw.call_sync(&generator, access)
        })
        .unwrap()
        .result
        .unwrap_err();
    assert_eq!(thrown.as_opaque(), Some(&JsValue::new(42)));
    let completed = next_sync(&generator, &mut context);
    assert_result(completed, JsValue::undefined(), true, &mut context);
    assert_counts(&mut context, 1, 0, 1);
}

#[test]
fn ordinary_async_return_and_completed_throw_keep_guest_results() {
    let (mut context, executor) = context();
    let generator = async_generator(&mut context, false);
    let first = request_async(Request::Next, &generator, &mut context);
    assert_async_result(&executor, &first, 10.into(), false, &mut context);
    let returned = request_async(Request::Return, &generator, &mut context);
    assert_async_result(&executor, &returned, 91.into(), true, &mut context);
    let thrown = request_async(Request::Throw, &generator, &mut context);
    assert_eq!(
        executor.settle(&thrown, &mut context),
        PromiseState::Rejected(42.into())
    );
    let completed = request_async(Request::Next, &generator, &mut context);
    assert_async_result(
        &executor,
        &completed,
        JsValue::undefined(),
        true,
        &mut context,
    );
    assert_counts(&mut context, 1, 0, 1);
}
