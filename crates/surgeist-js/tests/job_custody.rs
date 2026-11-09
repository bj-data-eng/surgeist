#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, JsResult, JsValue, Source,
    context::time::FixedClock,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{
        ContextTask, GenericJob, JobExecutor, NativeAsyncJob, NativeJob, PromiseJob,
        SimpleJobExecutor, TimeoutJob,
    },
};

fn context_with_executor() -> Context {
    Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap()
}

fn instruction_stop() -> EngineError {
    EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 0 })
}

fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}

fn drain_in_zero_instruction_scope(context: &mut Context) -> JsResult<()> {
    let actual_drain = RefCell::new(None);
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            *actual_drain.borrow_mut() = Some(access.run_jobs());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&instruction_stop())
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    actual_drain
        .into_inner()
        .expect("the scoped host must call the executor")
}

#[test]
fn unrelated_queued_callback_keeps_its_captures_and_order_after_another_jobs_stop() {
    let mut context = context_with_executor();
    let order = Rc::new(RefCell::new(Vec::new()));
    let refusal_entries = Rc::new(Cell::new(0));
    let unrelated_entries = Rc::new(Cell::new(0));
    let observed_capture = Rc::new(RefCell::new(None));
    let first_order = order.clone();
    let first_entries = refusal_entries.clone();
    context
        .enqueue_job(
            PromiseJob::new(move |mut access| {
                first_entries.set(first_entries.get() + 1);
                first_order.borrow_mut().push("refusal");
                access.eval(Source::from_bytes("0"))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let second_order = order.clone();
    let second_entries = unrelated_entries.clone();
    let second_capture = observed_capture.clone();
    let tag = String::from("accepted callback payload");
    let values = (40, 2);
    context
        .enqueue_job(
            PromiseJob::new(move |_| {
                second_entries.set(second_entries.get() + 1);
                second_order.borrow_mut().push("unrelated");
                *second_capture.borrow_mut() = Some((tag, values.0 + values.1));
                Ok(JsValue::new(values.0 + values.1))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();

    let actual_drain = drain_in_zero_instruction_scope(&mut context);
    assert_eq!(
        actual_drain
            .as_ref()
            .err()
            .and_then(|error| error.as_engine()),
        Some(&instruction_stop()),
        "actual driver result={actual_drain:?}; callback order={:?}",
        order.borrow()
    );
    assert_eq!(refusal_entries.get(), 1);
    assert_eq!(unrelated_entries.get(), 0);
    assert!(observed_capture.borrow().is_none());
    assert_eq!(*order.borrow(), ["refusal"]);

    let fresh_drain = context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    fresh_drain.result.unwrap();
    assert_eq!(unrelated_entries.get(), 1);
    assert_eq!(
        observed_capture.borrow().as_ref(),
        Some(&(String::from("accepted callback payload"), 42))
    );
    assert_eq!(*order.borrow(), ["refusal", "unrelated"]);
    context.run_jobs().unwrap();
    assert_eq!(refusal_entries.get(), 1);
    assert_eq!(unrelated_entries.get(), 1);
    assert_eq!(*order.borrow(), ["refusal", "unrelated"]);
}

#[derive(Default)]
struct PendingState {
    factories: Cell<usize>,
    polls: Cell<usize>,
    completions: Cell<usize>,
    drops: Cell<usize>,
    ready: Cell<bool>,
    waker: RefCell<Option<Waker>>,
    order: RefCell<Vec<&'static str>>,
    observed_capture: RefCell<Option<(String, i32)>>,
}

struct AcceptedFuture {
    state: Rc<PendingState>,
    tag: String,
    values: (i32, i32),
}

impl AcceptedFuture {
    fn new(state: Rc<PendingState>, tag: String, values: (i32, i32)) -> Self {
        state.factories.set(state.factories.get() + 1);
        state.order.borrow_mut().push("factory");
        Self { state, tag, values }
    }
}

impl Future for AcceptedFuture {
    type Output = JsResult<JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.state.polls.set(this.state.polls.get() + 1);
        if !this.state.ready.get() {
            *this.state.waker.borrow_mut() = Some(cx.waker().clone());
            this.state.order.borrow_mut().push("pending");
            return Poll::Pending;
        }
        this.state.completions.set(this.state.completions.get() + 1);
        this.state.order.borrow_mut().push("ready");
        let value = this.values.0 + this.values.1;
        *this.state.observed_capture.borrow_mut() = Some((this.tag.clone(), value));
        Poll::Ready(Ok(JsValue::new(value)))
    }
}

impl Drop for AcceptedFuture {
    fn drop(&mut self) {
        self.state.drops.set(self.state.drops.get() + 1);
        self.state.order.borrow_mut().push("drop");
    }
}

#[test]
fn unrelated_pending_future_survives_another_jobs_stop_and_resumes_without_restart() {
    let mut context = context_with_executor();
    let state = Rc::new(PendingState::default());
    let callback_state = state.clone();
    let tag = String::from("accepted future payload");
    context
        .enqueue_job(
            NativeAsyncJob::new(move |_| {
                Ok(ContextTask::from_future(AcceptedFuture::new(
                    callback_state,
                    tag,
                    (40, 2),
                )))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let refusal_entries = Rc::new(Cell::new(0));
    let callback_entries = refusal_entries.clone();
    let callback_state = state.clone();
    context
        .enqueue_job(
            GenericJob::new(
                move |mut access| {
                    callback_entries.set(callback_entries.get() + 1);
                    callback_state.order.borrow_mut().push("refusal");
                    access.eval(Source::from_bytes("0"))
                },
                context.realm().clone(),
            )
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();

    let actual_drain = drain_in_zero_instruction_scope(&mut context);
    assert_eq!(
        actual_drain
            .as_ref()
            .err()
            .and_then(|error| error.as_engine()),
        Some(&instruction_stop()),
        "actual driver result={actual_drain:?}; future order={:?}",
        state.order.borrow()
    );
    assert_eq!(refusal_entries.get(), 1);
    assert_eq!(state.factories.get(), 1);
    assert_eq!(state.polls.get(), 1);
    assert_eq!(state.completions.get(), 0);
    assert_eq!(
        state.drops.get(),
        0,
        "the unrelated accepted future must remain owned after the driver returns"
    );
    assert_eq!(*state.order.borrow(), ["factory", "pending", "refusal"]);

    state.ready.set(true);
    let waker = state.waker.borrow_mut().take();
    if let Some(waker) = waker {
        waker.wake();
    }
    let fresh_drain = context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    fresh_drain.result.unwrap();
    assert_eq!(state.factories.get(), 1);
    assert_eq!(state.polls.get(), 2);
    assert_eq!(state.completions.get(), 1);
    assert_eq!(state.drops.get(), 1);
    assert_eq!(
        state.observed_capture.borrow().as_ref(),
        Some(&(String::from("accepted future payload"), 42))
    );
    assert_eq!(
        *state.order.borrow(),
        ["factory", "pending", "refusal", "ready", "drop"]
    );
    context.run_jobs().unwrap();
    assert_eq!(refusal_entries.get(), 1);
    assert_eq!(state.factories.get(), 1);
    assert_eq!(state.polls.get(), 2);
    assert_eq!(state.completions.get(), 1);
    assert_eq!(state.drops.get(), 1);
}

#[test]
fn dropping_a_pending_drain_preserves_the_started_future_for_the_next_drain() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let state = Rc::new(PendingState::default());
    let callback_state = state.clone();
    context
        .enqueue_job(
            NativeAsyncJob::new(move |_| {
                Ok(ContextTask::from_future(AcceptedFuture::new(
                    callback_state,
                    String::from("accepted temporary-drain payload"),
                    (40, 2),
                )))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |access| {
            let holder = access.into_host_access();
            let mut first_drain = Box::pin(executor.clone().run_jobs_async(&holder));
            assert!(poll(&mut first_drain).is_pending());
            assert_eq!(state.factories.get(), 1);
            assert_eq!(state.polls.get(), 1);
            assert_eq!(state.completions.get(), 0);
            drop(first_drain);
            assert_eq!(
                state.drops.get(),
                0,
                "ending a temporary drain must retain its accepted future"
            );
            assert_eq!(*state.order.borrow(), ["factory", "pending"]);
            state.ready.set(true);
            let waker = state.waker.borrow_mut().take();
            if let Some(waker) = waker {
                waker.wake();
            }
            let mut next_drain = Box::pin(executor.clone().run_jobs_async(&holder));
            for _ in 0..16 {
                if let Poll::Ready(result) = poll(&mut next_drain) {
                    return result;
                }
            }
            panic!("the ready accepted future must finish within sixteen cooperative drain polls")
        })
        .unwrap();
    outcome.result.unwrap();
    assert_eq!(state.factories.get(), 1);
    assert_eq!(state.polls.get(), 2);
    assert_eq!(state.completions.get(), 1);
    assert_eq!(state.drops.get(), 1);
    assert_eq!(
        state.observed_capture.borrow().as_ref(),
        Some(&(String::from("accepted temporary-drain payload"), 42))
    );
    assert_eq!(
        *state.order.borrow(),
        ["factory", "pending", "ready", "drop"]
    );
    context.run_jobs().unwrap();
    assert_eq!(state.factories.get(), 1);
    assert_eq!(state.polls.get(), 2);
    assert_eq!(state.completions.get(), 1);
    assert_eq!(state.drops.get(), 1);
}

#[test]
fn an_untouched_due_timeout_keeps_its_captures_after_another_timeout_stops() {
    let clock = Rc::new(FixedClock::from_millis(100));
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .clock(clock.clone())
        .build()
        .unwrap();
    let order = Rc::new(RefCell::new(Vec::new()));
    let refusal_entries = Rc::new(Cell::new(0));
    let tail_entries = Rc::new(Cell::new(0));
    let observed_capture = Rc::new(RefCell::new(None));
    let first_order = order.clone();
    let first_entries = refusal_entries.clone();
    context
        .enqueue_job(
            TimeoutJob::new(
                NativeJob::new(move |mut access| {
                    first_entries.set(first_entries.get() + 1);
                    first_order.borrow_mut().push("timeout-refusal");
                    access.eval(Source::from_bytes("0"))
                }),
                0,
            )
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let second_order = order.clone();
    let second_entries = tail_entries.clone();
    let second_capture = observed_capture.clone();
    let tag = String::from("accepted timeout payload");
    let values = (40, 2);
    context
        .enqueue_job(
            TimeoutJob::new(
                NativeJob::new(move |_| {
                    second_entries.set(second_entries.get() + 1);
                    second_order.borrow_mut().push("timeout-tail");
                    *second_capture.borrow_mut() = Some((tag, values.0 + values.1));
                    Ok(JsValue::new(values.0 + values.1))
                }),
                0,
            )
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    // Both requests have the same deadline, and it is strictly in the past
    // before either drain starts. No later clock advancement is needed.
    clock.forward(1);

    let actual_drain = drain_in_zero_instruction_scope(&mut context);
    assert_eq!(
        actual_drain
            .as_ref()
            .err()
            .and_then(|error| error.as_engine()),
        Some(&instruction_stop()),
        "actual driver result={actual_drain:?}; timeout order={:?}",
        order.borrow()
    );
    assert_eq!(refusal_entries.get(), 1);
    assert_eq!(tail_entries.get(), 0);
    assert!(observed_capture.borrow().is_none());
    assert_eq!(*order.borrow(), ["timeout-refusal"]);

    let fresh_drain = context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    fresh_drain.result.unwrap();
    assert_eq!(tail_entries.get(), 1);
    assert_eq!(
        observed_capture.borrow().as_ref(),
        Some(&(String::from("accepted timeout payload"), 42))
    );
    assert_eq!(*order.borrow(), ["timeout-refusal", "timeout-tail"]);
    context.run_jobs().unwrap();
    assert_eq!(refusal_entries.get(), 1);
    assert_eq!(tail_entries.get(), 1);
    assert_eq!(*order.borrow(), ["timeout-refusal", "timeout-tail"]);
}

#[test]
fn reentrant_drain_runs_new_work_without_losing_or_running_the_outer_promise_tail() {
    let mut context = context_with_executor();
    let order = Rc::new(RefCell::new(Vec::new()));
    let first_entries = Rc::new(Cell::new(0));
    let new_entries = Rc::new(Cell::new(0));
    let tail_entries = Rc::new(Cell::new(0));
    let observed_capture = Rc::new(RefCell::new(None));
    let actual_inner_drain = Rc::new(RefCell::new(None));
    let callback_order = order.clone();
    let callback_first_entries = first_entries.clone();
    let callback_new_entries = new_entries.clone();
    let callback_inner_drain = actual_inner_drain.clone();
    context
        .enqueue_job(
            PromiseJob::new(move |mut access| {
                callback_first_entries.set(callback_first_entries.get() + 1);
                callback_order.borrow_mut().push("A");
                let new_order = callback_order.clone();
                access
                    .enqueue_job(
                        PromiseJob::new(move |_| {
                            callback_new_entries.set(callback_new_entries.get() + 1);
                            new_order.borrow_mut().push("N");
                            Ok(JsValue::new(7))
                        })
                        .into(),
                    )
                    .unwrap()
                    .into_result()
                    .unwrap();
                let inner = access.run_jobs();
                *callback_inner_drain.borrow_mut() = Some(inner.clone());
                inner?;
                callback_order.borrow_mut().push("A-stop");
                access.eval(Source::from_bytes("0"))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let callback_order = order.clone();
    let callback_tail_entries = tail_entries.clone();
    let callback_capture = observed_capture.clone();
    let tag = String::from("old promise tail payload");
    let values = (40, 2);
    context
        .enqueue_job(
            PromiseJob::new(move |_| {
                callback_tail_entries.set(callback_tail_entries.get() + 1);
                callback_order.borrow_mut().push("B");
                *callback_capture.borrow_mut() = Some((tag, values.0 + values.1));
                Ok(JsValue::new(values.0 + values.1))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();

    let actual_outer_drain = drain_in_zero_instruction_scope(&mut context);
    assert_eq!(
        actual_outer_drain
            .as_ref()
            .err()
            .and_then(|error| error.as_engine()),
        Some(&instruction_stop()),
        "actual outer driver result={actual_outer_drain:?}; callback order={:?}",
        order.borrow()
    );
    assert!(
        actual_inner_drain
            .borrow()
            .as_ref()
            .is_some_and(Result::is_ok),
        "the nested driver must run the new work before A requests its stop"
    );
    assert_eq!(first_entries.get(), 1);
    assert_eq!(new_entries.get(), 1);
    assert_eq!(tail_entries.get(), 0);
    assert!(observed_capture.borrow().is_none());
    assert_eq!(*order.borrow(), ["A", "N", "A-stop"]);

    let fresh_drain = context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    fresh_drain.result.unwrap();
    assert_eq!(tail_entries.get(), 1);
    assert_eq!(
        observed_capture.borrow().as_ref(),
        Some(&(String::from("old promise tail payload"), 42))
    );
    assert_eq!(*order.borrow(), ["A", "N", "A-stop", "B"]);
    context.run_jobs().unwrap();
    assert_eq!(first_entries.get(), 1);
    assert_eq!(new_entries.get(), 1);
    assert_eq!(tail_entries.get(), 1);
    assert_eq!(*order.borrow(), ["A", "N", "A-stop", "B"]);
}
