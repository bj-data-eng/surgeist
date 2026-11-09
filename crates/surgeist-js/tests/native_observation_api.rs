use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Condvar, Mutex},
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, ContextAccess, HostContext, JsObject, JsResult, JsValue, Source,
    builtins::promise::PromiseState,
    error::{EngineError, JsNativeErrorKind, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionReport},
    job::{
        ContextTask, Job, JobAcceptance, JobExecutor, JobPublicationError, JobRetirement,
        JobStatus, JobStep, NativeAsyncJob,
    },
    object::builtins::{JsFuture, JsPromise},
};

// Scheduling alone is explicit. All admission, task ownership, retirement and
// resolver behavior use the actual public job and promise implementations.
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
        panic!("this test dispatches the real wrapper explicitly")
    }
}
impl ManualExecutor {
    fn take_async(&self) -> NativeAsyncJob {
        let job = self.0.borrow_mut().pop_front();
        let Some(Job::AsyncJob(job)) = job else {
            panic!("the authored native promise task must own its real async wrapper")
        };
        job
    }
}
fn context(executor: Rc<ManualExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}
fn step(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> JobStep {
    job.run_step(&mut TaskContext::from_waker(Waker::noop()), host)
}
fn poll(future: &mut JsFuture, waker: &Waker) -> Poll<JsResult<JsValue>> {
    Pin::new(future).poll(&mut TaskContext::from_waker(waker))
}
#[derive(Default)]
struct WakeCount {
    count: Mutex<usize>,
    changed: Condvar,
}
impl WakeCount {
    fn record(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        drop(count);
        self.changed.notify_all();
    }
    fn current(&self) -> usize {
        *self.count.lock().unwrap()
    }
    fn wait_after(&self, baseline: usize) {
        let count = self.count.lock().unwrap();
        let (count, result) = self
            .changed
            .wait_timeout_while(count, Duration::from_secs(5), |count| *count <= baseline)
            .unwrap();
        assert!(
            *count > baseline,
            "the parked native waiter received no fresh retirement wake within five seconds: baseline={baseline}, observed={}, timed_out={}",
            *count,
            result.timed_out(),
        );
    }
}
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.record();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.record();
    }
}
fn no_action(report: &ExecutionReport) {
    assert_eq!(report.instructions().unwrap(), 0);
    assert_eq!(report.job_preparations().unwrap(), 0);
    assert_eq!(report.job_invocations().unwrap(), 0);
    assert_eq!(report.job_polls().unwrap(), 0);
    assert!(!report.effects_started());
}
fn limits() -> ExecutionOptions {
    ExecutionOptions::new(0)
        .with_job_preparation_limit(0)
        .with_job_invocation_limit(0)
        .with_job_poll_limit(0)
}

#[test]
fn a_preexisting_native_waiter_receives_retirement_without_action_admission_or_guest_rejection() {
    let executor = Rc::new(ManualExecutor::default());
    let mut owner = context(executor.clone());
    let factories = Rc::new(Cell::new(0));
    let made = factories.clone();
    let promise = JsPromise::from_task(
        move |_| {
            made.set(made.get() + 1);
            Ok(ContextTask::from_future(std::future::ready(Ok(
                JsValue::new(42),
            ))))
        },
        owner.access(),
    )
    .unwrap();
    let job = executor.take_async();
    let observer = job.observer();
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    let wakes = Arc::new(WakeCount::default());
    assert!(poll(&mut future, &Waker::from(wakes.clone())).is_pending());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let before_retirement = wakes.current();
    drop(job);
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
    // Another normal-domain driver may own the queued notification. Wait only
    // for this already parked waker; do not repoll the future to create progress.
    wakes.wait_after(before_retirement);
    let outcome = owner
        .with_execution(limits(), |_| {
            for _ in 0..2 {
                let Poll::Ready(Err(error)) = poll(&mut future, Waker::noop()) else {
                    panic!("the registered native wait must not remain parked")
                };
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::JobRetired(JobRetirement::Dropped))
                );
            }
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    no_action(&outcome.report);
    assert_eq!(factories.get(), 0);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
}

#[test]
fn actual_guest_settlement_keeps_its_value_identity_and_completed_observer_after_final_job_drop() {
    let executor = Rc::new(ManualExecutor::default());
    let mut owner = context(executor.clone());
    let value = JsObject::with_null_proto(owner.runtime());
    let supplied = value.clone();
    let promise = JsPromise::from_task(
        move |_| {
            Ok(ContextTask::from_future(std::future::ready(Ok(
                supplied.into()
            ))))
        },
        owner.access(),
    )
    .unwrap();
    let mut job = executor.take_async();
    let observer = job.observer();
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    assert!(poll(&mut future, Waker::noop()).is_pending());
    assert!(matches!(
        step(&mut job, &owner.host_access()),
        JobStep::Prepared
    ));
    let JobStep::Complete(result) = step(&mut job, &owner.host_access()) else {
        panic!("the real ready task must complete its resolver stage")
    };
    result.unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(value.clone().into())
    );
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    drop(job);
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    let outcome = owner
        .with_execution(limits(), |_| {
            for _ in 0..2 {
                let Poll::Ready(Ok(actual)) = poll(&mut future, Waker::noop()) else {
                    panic!("published guest settlement remains ready")
                };
                assert_eq!(actual.as_object().unwrap(), value);
            }
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    no_action(&outcome.report);
    assert_eq!(observer.fault(), None);
}

#[test]
fn foreign_runtime_future_registration_is_rejected_before_native_observation_is_exposed() {
    let mut original = context(Rc::new(ManualExecutor::default()));
    let mut receiver = context(Rc::new(ManualExecutor::default()));
    assert!(!original.runtime().same_heap(receiver.runtime()));
    let (promise, _) = JsPromise::new_pending(original.access());
    let refused = promise
        .clone()
        .into_js_future(receiver.access())
        .unwrap_err();
    assert!(matches!(
        refused.as_native().unwrap().kind(),
        JsNativeErrorKind::Type
    ));
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    let mut legitimate = promise.clone().into_js_future(original.access()).unwrap();
    assert!(poll(&mut legitimate, Waker::noop()).is_pending());
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
}

#[test]
fn rootless_retirement_observation_does_not_replace_the_receiving_operations_first_stop() {
    let executor = Rc::new(ManualExecutor::default());
    let mut owner = context(executor.clone());
    let promise = JsPromise::from_task(
        |_| {
            Ok(ContextTask::from_future(std::future::ready(Ok(
                JsValue::new(42),
            ))))
        },
        owner.access(),
    )
    .unwrap();
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    assert!(poll(&mut future, Waker::noop()).is_pending());
    drop(executor.take_async());
    let outcome = owner
        .with_execution(limits(), |mut access| {
            let original = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                original.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            let Poll::Ready(Err(historical)) = poll(&mut future, Waker::noop()) else {
                panic!("native observation remains rootless and terminal")
            };
            assert_eq!(
                historical.as_engine(),
                Some(&EngineError::JobRetired(JobRetirement::Dropped))
            );
            Err::<(), _>(historical)
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    no_action(&outcome.report);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
}
