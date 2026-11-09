#![allow(unused_crate_dependencies)]
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    time::Duration,
};
use surgeist_js::{
    Context, JsValue, Runtime,
    context::time::{Clock, JsInstant},
    execution::ExecutionOptions,
    job::{CancellationToken, IntervalJob, SimpleJobExecutor},
};

// Each Context owns a pure constant clock. Runtime shares object/queue custody,
// without imposing a clock instance or epoch on its participating contexts.
struct ConstantClock(JsInstant);
impl Clock for ConstantClock {
    fn now(&self) -> JsInstant {
        self.0
    }
    fn system_time_millis(&self) -> i64 {
        0
    }
}
fn context(runtime: Runtime, executor: Rc<SimpleJobExecutor>, now: JsInstant) -> Context {
    let builder = Context::builder()
        .runtime(runtime)
        .job_executor(executor)
        .clock(Rc::new(ConstantClock(now)));
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn self_cancelling_interval(
    owner: &Context,
    calls: Rc<Cell<usize>>,
    drops: Rc<Cell<usize>>,
) -> (IntervalJob, CancellationToken) {
    let token_slot: Rc<RefCell<Option<CancellationToken>>> = Rc::new(RefCell::new(None));
    let captured = token_slot.clone();
    let probe = DropProbe(drops);
    let interval = IntervalJob::with_realm(
        move |access| {
            let _keep = &probe;
            calls.set(calls.get() + 1);
            let token = captured.borrow().as_ref().unwrap().clone();
            token.cancel(access);
            Ok(JsValue::undefined())
        },
        owner.realm().clone(),
        Duration::from_millis(1),
    );
    let token = interval.cancellation_token().clone();
    *token_slot.borrow_mut() = Some(token.clone());
    (interval, token)
}

#[test]
fn interval_rescheduling_overflow_returns_a_host_error_without_losing_the_resident_callback() {
    let runtime = Runtime::default();
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut publisher = context(runtime.clone(), executor.clone(), JsInstant::new(1, 0));
    let mut overflowing = context(
        runtime.clone(),
        executor.clone(),
        JsInstant::new(u64::MAX, 999_999_999),
    );
    let mut retry = context(runtime, executor, JsInstant::new(2, 0));
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let (interval, token) = self_cancelling_interval(&publisher, calls.clone(), drops.clone());
    publisher
        .enqueue_job(interval.into())
        .unwrap()
        .into_result()
        .unwrap(); // Initial 1s + 1ms is representable.
    assert!(!token.revoked());
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        overflowing.with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
    }));
    assert_eq!(
        calls.get(),
        0,
        "deadline preparation precedes callback dispatch"
    );
    assert_eq!(
        drops.get(),
        0,
        "a previously accepted interval must retain its actual callback for retry"
    );
    assert!(
        !token.revoked(),
        "overflow cannot silently dispose the accepted interval"
    );
    let failed = outcome
        .expect("deadline arithmetic must return through the fallible drain boundary")
        .unwrap();
    let error = failed.result.unwrap_err();
    // The current API exposes the uncatchable host-error family. A precise
    // deadline cause belongs to the later functional typed-error assertions.
    assert!(error.as_engine().is_some());
    assert_eq!(failed.report.job_invocations().unwrap(), 0);
    assert!(!failed.report.effects_started());

    let completed = retry
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    completed.result.unwrap();
    assert_eq!(completed.report.job_invocations().unwrap(), 1);
    assert!(completed.report.effects_started());
    assert_eq!(calls.get(), 1);
    assert!(token.revoked());
    assert_eq!(drops.get(), 1);
    retry
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap()
        .result
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(drops.get(), 1);
}

#[test]
fn a_representable_peer_deadline_executes_the_same_self_cancelling_interval_once() {
    let runtime = Runtime::default();
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut publisher = context(runtime.clone(), executor.clone(), JsInstant::new(1, 0));
    let mut receiver = context(runtime, executor, JsInstant::new(2, 0));
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let (interval, token) = self_cancelling_interval(&publisher, calls.clone(), drops.clone());
    publisher
        .enqueue_job(interval.into())
        .unwrap()
        .into_result()
        .unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 0);
    let completed = receiver
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    completed.result.unwrap();
    assert_eq!(completed.report.job_invocations().unwrap(), 1);
    assert_eq!(calls.get(), 1);
    assert!(token.revoked());
    assert_eq!(drops.get(), 1);
    receiver
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap()
        .result
        .unwrap();
    assert_eq!(calls.get(), 1);
}
