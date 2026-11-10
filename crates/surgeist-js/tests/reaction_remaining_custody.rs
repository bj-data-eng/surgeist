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
    Context, ContextAccess, JsData, JsError, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{EngineError, ObjectAccessError, PanicError},
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobRetirement, JobStatus,
    },
    js_string,
    object::builtins::{JsAsyncGenerator, JsFunction, JsFuture, JsPromise},
    property::Attribute,
};
use surgeist_js_gc::heap::Trace;
const FINITE: u64 = 100_000;
enum Policy {
    Queue,
    Inline,
    Discard,
    Refuse(JsError),
    InlineThenFail(JsError),
}
#[derive(Default)]
struct Executor {
    plan: RefCell<VecDeque<Policy>>,
    queue: RefCell<VecDeque<Job>>,
    observers: RefCell<Vec<JobObserver>>,
    inline: Cell<bool>,
}
impl Executor {
    fn arm(&self, plan: impl IntoIterator<Item = Policy>) {
        assert!(self.plan.borrow().is_empty());
        *self.plan.borrow_mut() = plan.into_iter().collect();
        self.observers.borrow_mut().clear();
    }
    fn invoke(job: Job, access: ContextAccess<'_>) -> JsResult<JsValue> {
        let Job::PromiseJob(job) = job else {
            panic!("the fixture dispatches actual promise wrappers")
        };
        match job.call(access) {
            JobCall::Complete(result) => result,
            JobCall::Deferred { job, error } => {
                drop(job);
                panic!("actual inline entry cannot be relabeled from Deferred: {error:?}")
            }
        }
    }
    fn step(&self, context: &mut Context) {
        let job = self
            .queue
            .borrow_mut()
            .pop_front()
            .expect("the real accepted wrapper remains queued");
        context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                Self::invoke(job, access)
            })
            .unwrap()
            .result
            .unwrap();
    }
    fn finish(&self, promise: &JsPromise, context: &mut Context) {
        for _ in 0..24 {
            if promise.state().unwrap() != PromiseState::Pending {
                return;
            }
            self.step(context);
        }
        assert_ne!(promise.state().unwrap(), PromiseState::Pending);
    }
}
impl JobExecutor for Executor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        if self.inline.get() {
            Self::invoke(job, access).unwrap();
            return Ok(JobAcceptance::Accepted);
        }
        let policy = self.plan.borrow_mut().pop_front().unwrap_or(Policy::Queue);
        let Job::PromiseJob(reaction) = &job else {
            panic!("the authored plans select real promise reactions")
        };
        self.observers.borrow_mut().push(reaction.observer());
        match policy {
            Policy::Queue => {
                self.queue.borrow_mut().push_back(job);
                Ok(JobAcceptance::Accepted)
            }
            Policy::Discard => {
                drop(job);
                Ok(JobAcceptance::Discarded)
            }
            Policy::Refuse(error) => Err(JobPublicationError::new(
                job,
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
            Policy::Inline | Policy::InlineThenFail(_) => {
                self.inline.set(true);
                Self::invoke(job, access.reborrow()).unwrap();
                self.inline.set(false);
                match policy {
                    Policy::InlineThenFail(error) => Ok(JobAcceptance::AcceptedWithError(
                        JobPublicationCause::try_new(error, access).unwrap(),
                    )),
                    _ => Ok(JobAcceptance::Accepted),
                }
            }
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture drives its actual wrappers explicitly")
    }
}
thread_local! { static LEAF: RefCell<Option<JsValue>> = const { RefCell::new(None) }; }
#[derive(Debug, JsData, Trace)]
struct Leaf(#[heap_trace(skip)] Rc<Cell<usize>>);
impl Drop for Leaf {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
struct LeafInput;
impl Drop for LeafInput {
    fn drop(&mut self) {
        LEAF.with(|slot| drop(slot.borrow_mut().take()));
    }
}
fn leaf(context: &Context, drops: Rc<Cell<usize>>) -> LeafInput {
    let object = JsObject::from_proto_and_data(context.runtime(), None, Leaf(drops)).unwrap();
    LEAF.with(|slot| assert!(slot.replace(Some(object.into())).is_none()));
    LeafInput
}
fn context(executor: Rc<Executor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    for name in [
        "remainingEffects",
        "remainingFinally",
        "remainingIndependent",
    ] {
        context
            .register_global_property(surgeist_js::JsString::from(name), 0, Attribute::all())
            .unwrap();
    }
    context
        .register_global_builtin_callable(
            js_string!("takeRemainingLeaf"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                Ok(LEAF.with(|slot| slot.borrow_mut().take().unwrap()))
            }),
        )
        .unwrap();
    context
}
fn eval(source: &str, context: &mut Context) -> JsValue {
    context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(FINITE))
        .unwrap()
        .result
        .unwrap()
}
fn function(source: &str, context: &mut Context) -> JsFunction {
    JsFunction::from_object(eval(source, context).as_object().unwrap()).unwrap()
}
fn value(name: &str, context: &mut Context) -> JsValue {
    context
        .global_object()
        .unwrap()
        .get(surgeist_js::JsString::from(name), context.access())
        .unwrap()
}
fn first_error() -> JsError {
    PanicError::new("authored later bookkeeping precedence")
        .with_source(JsNativeError::range().with_message("retained first cause"))
        .into()
}
fn poll(wait: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(wait).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn native(promise: &JsPromise, context: &mut Context) -> JsFuture {
    let mut wait = promise.clone().into_js_future(context.access()).unwrap();
    assert!(poll(&mut wait).is_pending());
    wait
}
fn record(promise: &JsPromise, expected: JsValue, done: bool, context: &mut Context) -> JsValue {
    let PromiseState::Fulfilled(record) = promise.state().unwrap() else {
        panic!("actual request must fulfill normally")
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
fn actual_discard_then_handled_write_refusal_preserves_prior_effects_and_the_first_accepted_error()
{
    let executor = Rc::new(Executor::default());
    let mut context = context(executor.clone());
    let prefix_source = JsPromise::resolve(11, context.access()).unwrap();
    let callback = function(
        "() => { globalThis.remainingEffects += 1; return 42; }",
        &mut context,
    );
    executor.arm([Policy::Inline]);
    let prefix = prefix_source
        .then(Some(callback.clone()), None, context.access())
        .unwrap();
    let prefix_identity = prefix.identity();
    assert_eq!(
        prefix.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    assert_eq!(value("remainingEffects", &mut context), JsValue::new(1));
    // This separate source is still unhandled: both refused handled writes
    // are required bookkeeping, not a redundant already-true write.
    let source = JsPromise::resolve(42, context.access()).unwrap();
    executor.arm([Policy::Discard]);
    let shared_source = source.borrow().unwrap();
    let discarded = source
        .then(Some(callback.clone()), None, context.access())
        .unwrap_err();
    assert_eq!(
        discarded.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
    );
    assert_eq!(
        executor.observers.borrow().last().unwrap().status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    drop(shared_source);
    assert_eq!(value("remainingEffects", &mut context), JsValue::new(1));
    assert_eq!(prefix.identity(), prefix_identity);
    assert_eq!(
        prefix.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    executor.arm([Policy::InlineThenFail(first_error())]);
    let shared_source = source.borrow().unwrap();
    let actual = source
        .then(Some(callback.clone()), None, context.access())
        .unwrap_err();
    assert_eq!(
        actual,
        first_error(),
        "later handled-write Busy cannot replace the actual accepted-error cause"
    );
    assert_eq!(
        executor.observers.borrow().last().unwrap().status(),
        Ok(JobStatus::Completed)
    );
    assert_eq!(executor.observers.borrow().last().unwrap().fault(), None);
    drop(shared_source);
    assert_eq!(value("remainingEffects", &mut context), JsValue::new(2));
    assert_eq!(
        source.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    executor.arm([Policy::Queue]);
    let retry = source.then(Some(callback), None, context.access()).unwrap();
    executor.finish(&retry, &mut context);
    assert_eq!(
        retry.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    assert_eq!(value("remainingEffects", &mut context), JsValue::new(3));
    assert_eq!(prefix.identity(), prefix_identity);
    assert_eq!(
        prefix.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
}

#[test]
fn later_discard_or_refusal_of_an_async_generator_await_releases_only_its_actual_continuation() {
    for refused in [false, true] {
        let executor = Rc::new(Executor::default());
        let mut context = context(executor.clone());
        let (gate, resolvers) = JsPromise::new_pending(context.access()).unwrap();
        context
            .register_global_property(js_string!("remainingGate"), gate, Attribute::all())
            .unwrap();
        let drops = Rc::new(Cell::new(0));
        let _input = leaf(&context, drops.clone());
        let generator = JsAsyncGenerator::from_object(
            eval(
                "(async function* () { globalThis.remainingEffects += 1;
                function useLeaf(leaf, value) { globalThis.remainingEffects += 10; }
                try { useLeaf(takeRemainingLeaf(), await remainingGate); yield 2; }
                finally { globalThis.remainingFinally += 1; } })()",
                &mut context,
            )
            .as_object()
            .unwrap(),
        )
        .unwrap();
        let request = context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                generator.next(JsValue::undefined(), access)
            })
            .unwrap()
            .result
            .unwrap();
        let identity = request.identity();
        assert_eq!(request.state().unwrap(), PromiseState::Pending);
        assert!(LEAF.with(|slot| slot.borrow().is_none()));
        context.runtime().collect().unwrap();
        assert_eq!(drops.get(), 0);
        executor.arm([if refused {
            Policy::Refuse(first_error())
        } else {
            Policy::Discard
        }]);
        let result =
            resolvers
                .resolve
                .call(&JsValue::undefined(), &[JsValue::new(7)], context.access());
        if refused {
            assert_eq!(result.unwrap_err(), first_error());
        } else {
            result.unwrap();
        }
        assert_eq!(
            executor.observers.borrow().last().unwrap().status(),
            Ok(JobStatus::Retired(JobRetirement::Dropped))
        );
        assert_eq!(executor.observers.borrow().last().unwrap().fault(), None);
        assert!(executor.queue.borrow().is_empty());
        assert_eq!(request.identity(), identity);
        assert_eq!(request.state().unwrap(), PromiseState::Pending);
        context.runtime().collect().unwrap();
        assert_eq!(drops.get(), 1);
        assert_eq!(value("remainingEffects", &mut context), JsValue::new(1));
        assert_eq!(value("remainingFinally", &mut context), JsValue::new(0));
        assert_eq!(eval("40 + 2", &mut context), JsValue::new(42));
        assert_eq!(request.state().unwrap(), PromiseState::Pending);
        assert_eq!(value("remainingFinally", &mut context), JsValue::new(0));
    }
}

#[test]
fn later_fanout_discard_or_refusal_cannot_erase_a_generators_actual_new_successor_or_terminal() {
    for terminal in [false, true] {
        for refused in [false, true] {
            let executor = Rc::new(Executor::default());
            let mut context = context(executor.clone());
            let (first_gate, first_resolvers) = JsPromise::new_pending(context.access()).unwrap();
            let (second_gate, second_resolvers) = JsPromise::new_pending(context.access()).unwrap();
            context
                .register_global_property(
                    js_string!("remainingFirstGate"),
                    first_gate.clone(),
                    Attribute::all(),
                )
                .unwrap();
            context
                .register_global_property(
                    js_string!("remainingSecondGate"),
                    second_gate,
                    Attribute::all(),
                )
                .unwrap();
            let source = if terminal {
                "(async function* () { globalThis.remainingEffects += 1;
                try { await remainingFirstGate; globalThis.remainingEffects += 10; return 41; }
                finally { globalThis.remainingFinally += 1; } })()"
            } else {
                "(async function* () { globalThis.remainingEffects += 1;
                try { await remainingFirstGate; globalThis.remainingEffects += 10;
                    await remainingSecondGate; yield 2; globalThis.remainingEffects += 100; return 3; }
                finally { globalThis.remainingFinally += 1; } })()"
            };
            let generator =
                JsAsyncGenerator::from_object(eval(source, &mut context).as_object().unwrap())
                    .unwrap();
            let first_request = generator
                .next(JsValue::undefined(), context.access())
                .unwrap();
            let mut first_wait = native(&first_request, &mut context);
            let independent_handler = function(
                "() => { globalThis.remainingIndependent += 1; }",
                &mut context,
            );
            let independent = first_gate
                .then(Some(independent_handler), None, context.access())
                .unwrap();
            executor.arm([
                Policy::Inline,
                if refused {
                    Policy::Refuse(first_error())
                } else {
                    Policy::Discard
                },
            ]);
            let result = first_resolvers.resolve.call(
                &JsValue::undefined(),
                &[JsValue::new(7)],
                context.access(),
            );
            if refused {
                assert_eq!(result.unwrap_err(), first_error());
            } else {
                result.unwrap();
            }
            assert!(executor.plan.borrow().is_empty());
            assert_eq!(
                executor.observers.borrow()[0].status(),
                Ok(JobStatus::Completed)
            );
            assert_eq!(executor.observers.borrow()[0].fault(), None);
            assert_eq!(
                executor.observers.borrow()[1].status(),
                Ok(JobStatus::Retired(JobRetirement::Dropped))
            );
            assert_eq!(executor.observers.borrow()[1].fault(), None);
            assert_eq!(independent.state().unwrap(), PromiseState::Pending);
            assert_eq!(value("remainingIndependent", &mut context), JsValue::new(0));
            assert_eq!(value("remainingEffects", &mut context), JsValue::new(11));
            if terminal {
                let result = record(&first_request, JsValue::new(41), true, &mut context);
                assert_eq!(poll(&mut first_wait), Poll::Ready(Ok(result)));
                assert_eq!(value("remainingFinally", &mut context), JsValue::new(1));
                let next = generator
                    .next(JsValue::undefined(), context.access())
                    .unwrap();
                record(&next, JsValue::undefined(), true, &mut context);
                assert_eq!(value("remainingEffects", &mut context), JsValue::new(11));
            } else {
                assert_eq!(first_request.state().unwrap(), PromiseState::Pending);
                assert!(poll(&mut first_wait).is_pending());
                assert_eq!(value("remainingFinally", &mut context), JsValue::new(0));
                second_resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[JsValue::new(9)], context.access())
                    .unwrap();
                executor.finish(&first_request, &mut context);
                let result = record(&first_request, JsValue::new(2), false, &mut context);
                assert_eq!(poll(&mut first_wait), Poll::Ready(Ok(result)));
                let next = generator
                    .next(JsValue::undefined(), context.access())
                    .unwrap();
                executor.finish(&next, &mut context);
                record(&next, JsValue::new(3), true, &mut context);
                assert_eq!(value("remainingEffects", &mut context), JsValue::new(111));
                assert_eq!(value("remainingFinally", &mut context), JsValue::new(1));
            }
        }
    }
}
