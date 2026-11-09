#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsResult, JsValue, NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{EngineError, EngineResourceError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ProfileMode},
    job::{Job, JobCall, JobExecutor},
    js_string,
    object::builtins::{JsAsyncGenerator, JsFunction, JsGenerator, JsPromise},
};

const LIMIT: u64 = 100_000;

#[derive(Clone, Copy, Debug)]
enum Request {
    Next,
    Return,
    Throw,
}
impl Request {
    const ALL: [Self; 3] = [Self::Next, Self::Return, Self::Throw];
    fn name(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Return => "return",
            Self::Throw => "throw",
        }
    }
    fn argument(self) -> JsValue {
        match self {
            Self::Next => JsValue::undefined(),
            Self::Return => 91.into(),
            Self::Throw => 42.into(),
        }
    }
}

enum Generator {
    Sync(JsGenerator),
    Async(JsAsyncGenerator),
}
enum Route {
    Native,
    Guest(JsFunction),
}
#[derive(Debug)]
enum Output {
    Sync(JsValue),
    Async(JsPromise),
}

impl Generator {
    fn call(&self, request: Request, route: &Route, access: ContextAccess<'_>) -> JsResult<Output> {
        if let Route::Guest(function) = route {
            let value = function.call(&JsValue::undefined(), &[], access)?;
            return match self {
                Self::Sync(_) => Ok(Output::Sync(value)),
                Self::Async(_) => Ok(Output::Async(JsPromise::from_object(
                    value.as_object().unwrap(),
                )?)),
            };
        }
        match self {
            Self::Sync(generator) => match request {
                Request::Next => generator.next(request.argument(), access),
                Request::Return => generator.r#return(request.argument(), access),
                Request::Throw => generator.throw(request.argument(), access),
            }
            .map(Output::Sync),
            Self::Async(generator) => match request {
                Request::Next => generator.next(request.argument(), access),
                Request::Return => generator.r#return(request.argument(), access),
                Request::Throw => generator.throw(request.argument(), access),
            }
            .map(Output::Async),
        }
    }
    fn value(&self) -> JsValue {
        match self {
            Self::Sync(value) => value.clone().into(),
            Self::Async(value) => value.clone().into(),
        }
    }
}

fn options(requests: u64) -> ExecutionOptions {
    ExecutionOptions::new(LIMIT).with_generator_request_limit(requests)
}
fn refusal(error: &JsError, limit: u64) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::GeneratorRequest { limit }
        ))
    );
}

#[derive(Default)]
struct ManualExecutor(RefCell<VecDeque<Job>>);
impl JobExecutor for ManualExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, _: ContextAccess<'_>) {
        self.0.borrow_mut().push_back(job);
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches selected jobs explicitly")
    }
}
impl ManualExecutor {
    fn settle(&self, promise: &JsPromise, context: &mut Context) -> JsResult<JsValue> {
        for _ in 0..32 {
            match promise.state().unwrap() {
                PromiseState::Fulfilled(value) => return Ok(value),
                PromiseState::Rejected(reason) => return Err(JsError::from_opaque(reason)),
                PromiseState::Pending => {}
            }
            let queued = self.0.borrow_mut().pop_front();
            let Some(Job::PromiseJob(job)) = queued else {
                panic!("ready generator work supplies a real promise job: {queued:?}")
            };
            let dispatched = context
                .with_execution(options(0), |access| match job.call(access) {
                    JobCall::Complete(result) => result,
                    JobCall::Deferred { job, error } => {
                        self.0.borrow_mut().push_front(Job::PromiseJob(job));
                        Err(error)
                    }
                })
                .unwrap();
            assert_eq!(
                dispatched.report.generator_requests().unwrap(),
                0,
                "resume and settlement are not new requests"
            );
            dispatched.result.unwrap();
        }
        panic!("the accepted request did not settle after thirty-two bounded dispatches")
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
fn generator(context: &mut Context, asynchronous: bool, gated: bool, completed: bool) -> Generator {
    let prefix = if asynchronous {
        "async function*"
    } else {
        "function*"
    };
    let body = if completed {
        "return 30;"
    } else if gated {
        "starts += 1; try { await requestGate; yield 10; resumes += 1; yield 20; return 30; } finally { closes += 1; }"
    } else {
        "starts += 1; try { yield 10; resumes += 1; yield 20; return 30; } finally { closes += 1; }"
    };
    let source = format!("globalThis.starts = 0; globalThis.resumes = 0; globalThis.closes = 0;
        globalThis.requestGate = new Promise(resolve => {{ globalThis.releaseRequestGate = resolve; }});
        ({prefix} () {{ {body} }})()");
    let object = eval(context, &source).as_object().unwrap();
    if asynchronous {
        Generator::Async(JsAsyncGenerator::from_object(object).unwrap())
    } else {
        Generator::Sync(JsGenerator::from_object(object).unwrap())
    }
}
fn route(context: &mut Context, generator: &Generator, request: Request, guest: bool) -> Route {
    if !guest {
        return Route::Native;
    }
    context
        .global_object()
        .set(
            js_string!("requestSubject"),
            generator.value(),
            true,
            context.access(),
        )
        .unwrap();
    let method = request.name();
    Route::Guest(
        JsFunction::from_object(
            eval(
                context,
                &format!(
                    "(function () {{ return requestSubject.{method}({}); }})",
                    match request {
                        Request::Next => "undefined",
                        Request::Return => "91",
                        Request::Throw => "42",
                    }
                ),
            )
            .as_object()
            .unwrap(),
        )
        .unwrap(),
    )
}
fn settle(
    result: JsResult<Output>,
    executor: &ManualExecutor,
    context: &mut Context,
) -> JsResult<JsValue> {
    match result? {
        Output::Sync(value) => Ok(value),
        Output::Async(promise) => executor.settle(&promise, context),
    }
}
fn iterator(value: JsValue, expected: JsValue, done: bool, context: &mut Context) {
    let object = value.as_object().unwrap();
    assert_eq!(
        object.get(js_string!("value"), context.access()).unwrap(),
        expected
    );
    assert_eq!(
        object.get(js_string!("done"), context.access()).unwrap(),
        done.into()
    );
}
fn property(context: &mut Context, name: &str) -> JsValue {
    context
        .global_object()
        .get(surgeist_js::JsString::from(name), context.access())
        .unwrap()
}
fn release(context: &mut Context) {
    let released = context
        .eval_with_budget(Source::from_bytes("releaseRequestGate(1)"), options(0))
        .unwrap();
    released.result.unwrap();
    assert_eq!(released.report.generator_requests().unwrap(), 0);
}

#[test]
fn every_native_and_guest_request_kind_has_one_attempt_unit_for_sync_and_async_generators() {
    for asynchronous in [false, true] {
        for guest in [false, true] {
            for request in Request::ALL {
                let (mut context, executor) = context();
                let generator = generator(&mut context, asynchronous, false, false);
                let route = route(&mut context, &generator, request, guest);
                let outcome = context
                    .with_execution(options(1), |access| generator.call(request, &route, access))
                    .unwrap();
                assert_eq!(
                    outcome.report.generator_requests().unwrap(),
                    1,
                    "{asynchronous}/{guest}/{request:?}"
                );
                assert!(outcome.report.effects_started());
                let result = settle(outcome.result, &executor, &mut context);
                match request {
                    Request::Next => iterator(result.unwrap(), 10.into(), false, &mut context),
                    Request::Return => iterator(result.unwrap(), 91.into(), true, &mut context),
                    Request::Throw => {
                        assert_eq!(result.unwrap_err().as_opaque(), Some(&JsValue::new(42)))
                    }
                }
            }
        }
    }
}

#[test]
fn zero_request_cap_preserves_unstarted_state_and_opens_no_request_phase() {
    for asynchronous in [false, true] {
        for guest in [false, true] {
            for request in Request::ALL {
                let (mut context, executor) = context();
                let generator = generator(&mut context, asynchronous, false, false);
                let route = route(&mut context, &generator, request, guest);
                let outcome = context
                    .with_execution(
                        options(0).with_profile(ProfileMode::Summary { timing: false }),
                        |access| generator.call(request, &route, access),
                    )
                    .unwrap();
                refusal(&outcome.result.unwrap_err(), 0);
                assert_eq!(outcome.report.generator_requests().unwrap(), 0);
                assert_eq!(
                    outcome
                        .report
                        .profile()
                        .unwrap()
                        .unwrap()
                        .scopes
                        .iter()
                        .filter(|scope| scope.name == "generator-request")
                        .map(|scope| scope.phases)
                        .sum::<u64>(),
                    0
                );
                if !guest {
                    assert!(!outcome.report.effects_started());
                    assert_eq!(outcome.report.instructions().unwrap(), 0);
                }
                assert_eq!(property(&mut context, "starts"), 0.into());
                let fresh = context
                    .with_execution(options(1), |access| {
                        generator.call(Request::Next, &Route::Native, access)
                    })
                    .unwrap();
                assert_eq!(fresh.report.generator_requests().unwrap(), 1);
                iterator(
                    settle(fresh.result, &executor, &mut context).unwrap(),
                    10.into(),
                    false,
                    &mut context,
                );
            }
        }
    }
}

#[test]
fn one_accepted_request_then_refusal_keeps_the_exact_first_stop_and_accepted_async_work() {
    for asynchronous in [false, true] {
        let (mut context, executor) = context();
        let generator = generator(&mut context, asynchronous, false, false);
        let mut accepted = None;
        let mut stopped = None;
        let outcome = context
            .with_execution(options(1), |mut access| {
                accepted =
                    Some(generator.call(Request::Next, &Route::Native, access.reborrow())?);
                let first = generator
                    .call(Request::Return, &Route::Native, access.reborrow())
                    .unwrap_err();
                refusal(&first, 1);
                let repeated = generator
                    .call(Request::Throw, &Route::Native, access)
                    .unwrap_err();
                assert_eq!(repeated, first);
                stopped = Some(first);
                Ok(())
            })
            .unwrap();
        assert_eq!(outcome.result.unwrap_err(), stopped.unwrap());
        assert_eq!(outcome.report.generator_requests().unwrap(), 1);
        iterator(
            settle(Ok(accepted.unwrap()), &executor, &mut context).unwrap(),
            10.into(),
            false,
            &mut context,
        );
    }
}

#[test]
fn a_preexisting_instruction_stop_precedes_a_zero_request_cap_without_an_attempt() {
    for asynchronous in [false, true] {
        for request in Request::ALL {
            let (mut context, _) = context();
            let generator = generator(&mut context, asynchronous, false, false);
            let mut first = None;
            let outcome = context
                .with_execution(
                    ExecutionOptions::new(0).with_generator_request_limit(0),
                    |mut access| {
                        let stopped = access.eval(Source::from_bytes("0")).unwrap_err();
                        assert_eq!(
                            stopped.as_engine(),
                            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                                limit: 0
                            }))
                        );
                        let direct = generator.call(request, &Route::Native, access).unwrap_err();
                        assert_eq!(direct, stopped);
                        first = Some(stopped);
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(outcome.result.unwrap_err(), first.unwrap());
            assert_eq!(outcome.report.generator_requests().unwrap(), 0);
            assert_eq!(property(&mut context, "starts"), 0.into());
        }
    }
}

#[test]
fn completed_requests_still_enter_the_algorithm_and_count_once_without_vm_work() {
    for asynchronous in [false, true] {
        for request in Request::ALL {
            let (mut context, executor) = context();
            let generator = generator(&mut context, asynchronous, false, true);
            let completed = context
                .with_execution(options(1), |access| {
                    generator.call(Request::Next, &Route::Native, access)
                })
                .unwrap();
            iterator(
                settle(completed.result, &executor, &mut context).unwrap(),
                30.into(),
                true,
                &mut context,
            );
            let outcome = context
                .with_execution(options(1), |access| {
                    generator.call(request, &Route::Native, access)
                })
                .unwrap();
            assert_eq!(outcome.report.generator_requests().unwrap(), 1);
            assert!(outcome.report.effects_started());
            assert_eq!(outcome.report.instructions().unwrap(), 0);
            let result = settle(outcome.result, &executor, &mut context);
            match request {
                Request::Next => {
                    iterator(result.unwrap(), JsValue::undefined(), true, &mut context)
                }
                Request::Return => iterator(result.unwrap(), 91.into(), true, &mut context),
                Request::Throw => {
                    assert_eq!(result.unwrap_err().as_opaque(), Some(&JsValue::new(42)))
                }
            }
        }
    }
}

#[test]
fn the_default_request_limit_accepts_three_completed_requests() {
    let (mut context, _) = context();
    let generator = generator(&mut context, false, false, true);
    context
        .with_execution(options(1), |access| {
            generator.call(Request::Next, &Route::Native, access)
        })
        .unwrap()
        .result
        .unwrap();
    let outcome = context
        .with_execution(ExecutionOptions::new(LIMIT), |mut access| {
            for _ in 0..3 {
                generator.call(Request::Next, &Route::Native, access.reborrow())?;
            }
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    assert_eq!(outcome.report.generator_requests().unwrap(), 3);
}

#[test]
fn wrong_brand_requests_are_charged_before_validation_for_every_prototype_method() {
    for asynchronous in [false, true] {
        for request in Request::ALL {
            for cap in [0, 1] {
                let (mut context, executor) = context();
                let prefix = if asynchronous {
                    "async function*"
                } else {
                    "function*"
                };
                let function = JsFunction::from_object(eval(&mut context, &format!("(() => {{ const method = Object.getPrototypeOf(({prefix} () {{}}).prototype).{}; return function () {{ return method.call({{}}, 42); }}; }})()", request.name())).as_object().unwrap()).unwrap();
                let outcome = context
                    .with_execution(options(cap), |access| {
                        function.call(&JsValue::undefined(), &[], access)
                    })
                    .unwrap();
                assert_eq!(outcome.report.generator_requests().unwrap(), cap);
                if cap == 0 {
                    refusal(&outcome.result.unwrap_err(), 0);
                    continue;
                }
                let error = if asynchronous {
                    let promise =
                        JsPromise::from_object(outcome.result.unwrap().as_object().unwrap())
                            .unwrap();
                    executor.settle(&promise, &mut context).unwrap_err()
                } else {
                    outcome.result.unwrap_err()
                };
                assert!(error.try_native(context.access()).unwrap().is_type());
            }
        }
    }
}

#[test]
fn already_executing_sync_requests_count_before_their_guest_type_error() {
    for request in Request::ALL {
        let (mut context, _) = context();
        let value = eval(
            &mut context,
            &format!(
                "globalThis.busyType = false; globalThis.busyGenerator = (function* () {{ try {{ busyGenerator.{}(42); }} catch (error) {{ busyType = error instanceof TypeError; }} yield 10; }})(); busyGenerator",
                request.name()
            ),
        );
        let generator = JsGenerator::from_object(value.as_object().unwrap()).unwrap();
        let outcome = context
            .with_execution(options(2), |access| {
                generator.next(JsValue::undefined(), access)
            })
            .unwrap();
        assert_eq!(outcome.report.generator_requests().unwrap(), 2);
        iterator(outcome.result.unwrap(), 10.into(), false, &mut context);
        assert_eq!(property(&mut context, "busyType"), true.into());
    }
}

#[test]
fn reentrant_requests_consume_the_same_operation_cap_without_consuming_a_refused_inner_state() {
    for cap in [1, 2] {
        let (mut context, _) = context();
        eval(
            &mut context,
            "globalThis.innerRequest = (function* () { yield 99; })(); globalThis.outerRequest = (function* () { innerRequest.next(); yield 10; })();",
        );
        let outer =
            JsGenerator::from_object(property(&mut context, "outerRequest").as_object().unwrap())
                .unwrap();
        let inner =
            JsGenerator::from_object(property(&mut context, "innerRequest").as_object().unwrap())
                .unwrap();
        let outcome = context
            .with_execution(options(cap), |access| {
                outer.next(JsValue::undefined(), access)
            })
            .unwrap();
        assert_eq!(outcome.report.generator_requests().unwrap(), cap);
        if cap == 1 {
            refusal(&outcome.result.unwrap_err(), 1);
            let retry = context
                .with_execution(options(1), |access| {
                    inner.next(JsValue::undefined(), access)
                })
                .unwrap();
            iterator(retry.result.unwrap(), 99.into(), false, &mut context);
        } else {
            iterator(outcome.result.unwrap(), 10.into(), false, &mut context);
        }
    }
}

#[test]
fn async_requests_enqueued_while_executing_are_counted_once_before_later_cap_zero_dispatches() {
    for request in Request::ALL {
        let (mut context, executor) = context();
        let generator = generator(&mut context, true, true, false);
        let outcome = context
            .with_execution(options(2), |mut access| {
                let Output::Async(first) =
                    generator.call(Request::Next, &Route::Native, access.reborrow())?
                else {
                    unreachable!()
                };
                let Output::Async(second) = generator.call(request, &Route::Native, access)? else {
                    unreachable!()
                };
                assert_eq!(first.state().unwrap(), PromiseState::Pending);
                assert_eq!(second.state().unwrap(), PromiseState::Pending);
                Ok((first, second))
            })
            .unwrap();
        assert_eq!(outcome.report.generator_requests().unwrap(), 2);
        let (first, second) = outcome.result.unwrap();
        assert_eq!(property(&mut context, "starts"), 1.into());
        release(&mut context);
        iterator(
            executor.settle(&first, &mut context).unwrap(),
            10.into(),
            false,
            &mut context,
        );
        let result = executor.settle(&second, &mut context);
        match request {
            Request::Next => {
                iterator(result.unwrap(), 20.into(), false, &mut context);
                assert_eq!(property(&mut context, "resumes"), 1.into());
            }
            Request::Return => {
                iterator(result.unwrap(), 91.into(), true, &mut context);
                assert_eq!(property(&mut context, "closes"), 1.into());
            }
            Request::Throw => {
                assert_eq!(result.unwrap_err().as_opaque(), Some(&JsValue::new(42)));
                assert_eq!(property(&mut context, "closes"), 1.into());
            }
        }
    }
}

#[test]
fn refusing_a_new_async_request_does_not_revoke_an_older_pending_await_continuation() {
    for request in Request::ALL {
        let (mut context, executor) = context();
        let generator = generator(&mut context, true, true, false);
        let accepted = context
            .with_execution(options(1), |access| {
                generator.call(Request::Next, &Route::Native, access)
            })
            .unwrap();
        let Output::Async(first) = accepted.result.unwrap() else {
            unreachable!()
        };
        assert_eq!(first.state().unwrap(), PromiseState::Pending);
        let refused = context
            .with_execution(options(0), |access| {
                generator.call(request, &Route::Native, access)
            })
            .unwrap();
        refusal(&refused.result.unwrap_err(), 0);
        assert_eq!(refused.report.generator_requests().unwrap(), 0);
        assert!(!refused.report.effects_started());
        release(&mut context);
        iterator(
            executor.settle(&first, &mut context).unwrap(),
            10.into(),
            false,
            &mut context,
        );
        assert_eq!(property(&mut context, "starts"), 1.into());
        assert_eq!(property(&mut context, "resumes"), 0.into());
        assert_eq!(property(&mut context, "closes"), 0.into());
    }
}

#[test]
fn an_admitted_body_host_error_remains_counted_and_preserves_its_original_payload() {
    let (mut context, _) = context();
    context
        .register_global_builtin_callable(
            js_string!("failRequestBody"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                Err(EngineError::Resource(EngineResourceError::Allocation).into())
            }),
        )
        .unwrap();
    let generator = JsGenerator::from_object(
        eval(&mut context, "(function* () { failRequestBody(); })()")
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let outcome = context
        .with_execution(options(1), |access| {
            generator.next(JsValue::undefined(), access)
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::Resource(EngineResourceError::Allocation))
    );
    assert_eq!(outcome.report.generator_requests().unwrap(), 1);
    assert!(outcome.report.effects_started());
}

thread_local! {
    static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static CLOCK_READS: Cell<usize> = const { Cell::new(0) };
}
fn clock() -> Duration {
    CLOCK_READS.set(CLOCK_READS.get() + 1);
    TIME.get()
}
fn modes() -> [ProfileMode; 5] {
    [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::ScopeName("generator-request"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::ScopeName("generator-request"),
        },
    ]
}
fn advance_callback(context: &mut Context) {
    context
        .register_global_builtin_callable(
            js_string!("advanceRequestClock"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                TIME.set(TIME.get() + Duration::from_millis(7));
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
}

#[test]
fn request_phases_end_on_return_before_pending_wait_and_all_untimed_modes_read_no_clock() {
    for mode in modes() {
        let (mut context, executor) = context();
        advance_callback(&mut context);
        let value = eval(
            &mut context,
            "globalThis.requestGate = new Promise(resolve => { globalThis.releaseRequestGate = resolve; }); (async function* () { advanceRequestClock(); await requestGate; yield 10; })()",
        );
        let generator = JsAsyncGenerator::from_object(value.as_object().unwrap()).unwrap();
        TIME.set(Duration::ZERO);
        CLOCK_READS.set(0);
        let outcome = context
            .with_execution(
                options(1).with_profile(mode).with_profile_clock(clock),
                |access| {
                    let promise = generator.next(JsValue::undefined(), access)?;
                    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
                    TIME.set(TIME.get() + Duration::from_millis(100));
                    Ok(promise)
                },
            )
            .unwrap();
        assert_eq!(outcome.report.generator_requests().unwrap(), 1);
        if mode == ProfileMode::Off {
            assert!(outcome.report.profile().unwrap().is_none());
        } else {
            let timed = matches!(
                mode,
                ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
            );
            let profile = outcome.report.profile().unwrap().unwrap();
            assert_eq!(profile.elapsed, timed.then_some(Duration::from_millis(107)));
            let request = profile
                .scopes
                .iter()
                .find(|scope| scope.name == "generator-request")
                .unwrap();
            assert_eq!(request.phases, 1);
            assert_eq!(
                request.phase_elapsed,
                timed.then_some(Duration::from_millis(7))
            );
        }
        if !matches!(
            mode,
            ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
        ) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
        release(&mut context);
        iterator(
            executor
                .settle(&outcome.result.unwrap(), &mut context)
                .unwrap(),
            10.into(),
            false,
            &mut context,
        );
    }
}

#[test]
fn nested_inline_requests_have_two_inclusive_phases_and_two_admitted_units() {
    let (mut context, _) = context();
    advance_callback(&mut context);
    let value = eval(
        &mut context,
        "globalThis.innerRequest = (function* () { advanceRequestClock(); yield 99; })(); (function* () { innerRequest.next(); yield 10; })()",
    );
    let generator = JsGenerator::from_object(value.as_object().unwrap()).unwrap();
    TIME.set(Duration::ZERO);
    let outcome = context
        .with_execution(
            options(2)
                .with_profile(ProfileMode::Summary { timing: true })
                .with_profile_clock(clock),
            |access| generator.next(JsValue::undefined(), access),
        )
        .unwrap();
    assert_eq!(outcome.report.generator_requests().unwrap(), 2);
    iterator(outcome.result.unwrap(), 10.into(), false, &mut context);
    let profile = outcome.report.profile().unwrap().unwrap();
    assert_eq!(profile.elapsed, Some(Duration::from_millis(7)));
    let request = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "generator-request")
        .unwrap();
    assert_eq!(request.phases, 2);
    assert_eq!(request.phase_elapsed, Some(Duration::from_millis(14)));
}

#[test]
fn admitted_request_unwind_closes_its_phase_before_a_later_host_gap() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let (mut context, _) = context();
    context
        .register_global_builtin_callable(
            js_string!("panicRequestBody"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                TIME.set(TIME.get() + Duration::from_millis(7));
                panic!("authored request unwind")
            }),
        )
        .unwrap();
    let generator = JsGenerator::from_object(
        eval(&mut context, "(function* () { panicRequestBody(); })()")
            .as_object()
            .unwrap(),
    )
    .unwrap();
    TIME.set(Duration::ZERO);
    let mut task = context
        .execution_task(
            options(1)
                .with_profile(ProfileMode::Summary { timing: true })
                .with_profile_clock(clock),
            async move |access| generator.next(JsValue::undefined(), access),
        )
        .unwrap();
    let receipt = task.receipt();
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _ = Pin::new(&mut task).poll(&mut TaskContext::from_waker(Waker::noop()));
    }))
    .unwrap_err();
    assert_eq!(
        unwind.downcast_ref::<&str>(),
        Some(&"authored request unwind")
    );
    TIME.set(TIME.get() + Duration::from_millis(100));
    drop(task);
    let report = receipt.report().unwrap();
    assert_eq!(report.generator_requests().unwrap(), 1);
    let profile = report.profile().unwrap().unwrap();
    let request = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "generator-request")
        .unwrap();
    assert_eq!(request.phases, 1);
    assert_eq!(request.phase_elapsed, Some(Duration::from_millis(7)));
}

#[test]
fn an_admitted_completed_async_runtime_refusal_keeps_its_request_count_and_phase() {
    let (mut context, executor) = context();
    let generator = generator(&mut context, true, false, true);
    let completed = context
        .with_execution(options(1), |access| {
            generator.call(Request::Next, &Route::Native, access)
        })
        .unwrap();
    iterator(
        settle(completed.result, &executor, &mut context).unwrap(),
        30.into(),
        true,
        &mut context,
    );
    let saved = context.runtime_limits();
    context.runtime_limits_mut().set_recursion_limit(0);
    let refused = context
        .with_execution(
            options(1).with_profile(ProfileMode::Summary { timing: false }),
            |access| generator.call(Request::Next, &Route::Native, access),
        )
        .unwrap();
    context.set_runtime_limits(saved);
    assert_eq!(
        refused.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Recursion))
    );
    assert_eq!(refused.report.generator_requests().unwrap(), 1);
    assert!(refused.report.effects_started());
    assert_eq!(refused.report.instructions().unwrap(), 0);
    let profile = refused.report.profile().unwrap().unwrap();
    let request = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "generator-request")
        .unwrap();
    assert_eq!(request.phases, 1);
}
