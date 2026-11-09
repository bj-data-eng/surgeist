use std::{
    cell::Cell,
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
};
use surgeist_js::{
    Context, HostContext, JsError, JsObject, JsResult, JsValue,
    error::EngineError,
    execution::ExecutionOptions,
    job::{ContextTask, JobObserver, JobRetirement, JobStatus, JobStep, JobWait, NativeAsyncJob},
};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

fn step(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> JobStep {
    job.run_step(&mut TaskContext::from_waker(Waker::noop()), host)
}

fn poll_wait(
    wait: &mut JobWait,
    waker: &Waker,
) -> Poll<Result<JobStatus, surgeist_js::job::JobFault>> {
    Pin::new(wait).poll(&mut TaskContext::from_waker(waker))
}

#[derive(Default)]
struct WakeCount(AtomicUsize);
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

fn subscribed(observer: &JobObserver) -> (JobWait, Arc<WakeCount>) {
    let mut wait = observer.wait().unwrap();
    let wakes = Arc::new(WakeCount::default());
    assert!(poll_wait(&mut wait, &Waker::from(wakes.clone())).is_pending());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    (wait, wakes)
}

#[derive(Clone, Default)]
struct Counts {
    factories: Rc<Cell<usize>>,
    polls: Rc<Cell<usize>>,
    captures_dropped: Rc<Cell<usize>>,
    returned_ready: Rc<Cell<bool>>,
}
impl Counts {
    fn assert_once(&self) {
        assert_eq!(self.factories.get(), 1);
        assert_eq!(self.polls.get(), 1);
        assert_eq!(self.captures_dropped.get(), 1);
    }
}

#[derive(Debug)]
struct HostPanic {
    site: &'static str,
}

struct TaskCapture {
    counts: Counts,
    disposal_payload: Option<Arc<HostPanic>>,
}
impl Drop for TaskCapture {
    fn drop(&mut self) {
        self.counts
            .captures_dropped
            .set(self.counts.captures_dropped.get() + 1);
        // Arm only after the poll closure has produced its Ready result. A
        // failed setup assertion cannot accidentally create a second unwind.
        if self.counts.returned_ready.get()
            && let Some(payload) = self.disposal_payload.take()
        {
            panic_any(payload);
        }
    }
}

fn ready_job(
    result: JsResult<JsValue>,
    counts: Counts,
    disposal_payload: Option<Arc<HostPanic>>,
) -> NativeAsyncJob {
    let capture = TaskCapture {
        counts: counts.clone(),
        disposal_payload,
    };
    NativeAsyncJob::new(move |_| {
        counts.factories.set(counts.factories.get() + 1);
        let mut result = Some(result);
        Ok(ContextTask::from_poll(move |_, _| {
            // The live closure owns this capture; it is neither explicitly
            // dropped nor panicked from inside the source poll.
            let _keep = &capture;
            counts.polls.set(counts.polls.get() + 1);
            let result = result.take().expect("the owned task source cannot replay");
            counts.returned_ready.set(true);
            Poll::Ready(result)
        }))
    })
}

fn assert_same_panic(payload: &(dyn std::any::Any + Send), expected: &Arc<HostPanic>) {
    let actual = payload
        .downcast_ref::<Arc<HostPanic>>()
        .expect("preserve the authored host payload type");
    assert!(
        Arc::ptr_eq(actual, expected),
        "resume the original host payload, without replacement"
    );
    assert_eq!(actual.site, expected.site);
}

fn assert_terminal_steps(
    job: &mut NativeAsyncJob,
    observer: &JobObserver,
    expected: JobStatus,
    undelivered_retirement: bool,
) {
    let mut receiver = context();
    let outcome = receiver.with_execution(
        ExecutionOptions::new(0)
            .with_job_preparation_limit(0)
            .with_job_invocation_limit(0)
            .with_job_poll_limit(0),
        |access| {
            let host = access.into_host_access();
            if undelivered_retirement {
                let JobStatus::Retired(reason) = expected else { panic!("this path owns an undelivered retirement") };
                let JobStep::Complete(result) = step(job, &host) else {
                    panic!("the first post-unwind observation must deliver its typed retirement once")
                };
                assert_eq!(result.unwrap_err().as_engine(), Some(&EngineError::JobRetired(reason)));
            }
            for _ in 0..3 {
                let JobStep::Finished(result) = step(job, &host) else {
                    panic!("later terminal observation must not defer, reenter, or invent a state fault")
                };
                assert_eq!(result, Ok(expected));
                assert_eq!(observer.status(), Ok(expected));
                assert_eq!(observer.fault(), None);
            }
            Ok(())
        },
    ).unwrap();
    outcome.result.unwrap();
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(outcome.report.job_preparations().unwrap(), 0);
    assert_eq!(outcome.report.job_invocations().unwrap(), 0);
    assert_eq!(outcome.report.job_polls().unwrap(), 0);
    assert!(!outcome.report.effects_started());
}

#[test]
fn ordinary_ready_success_and_guest_error_dispose_once_and_publish_completed() {
    for guest_error in [false, true] {
        let mut owner = context();
        let object = JsObject::with_null_proto(owner.runtime());
        let original = JsError::from_opaque(object.clone().into()).unwrap();
        let result = if guest_error {
            Err(original.clone())
        } else {
            Ok(42.into())
        };
        let counts = Counts::default();
        let mut job = ready_job(result, counts.clone(), None);
        let observer = job.observer();
        let (mut wait, wakes) = subscribed(&observer);
        assert!(matches!(
            step(&mut job, &owner.host_access()),
            JobStep::Prepared
        ));
        assert_eq!(counts.polls.get(), 0);
        assert_eq!(counts.captures_dropped.get(), 0);

        let JobStep::Complete(result) = step(&mut job, &owner.host_access()) else {
            panic!("ordinary Ready result")
        };
        if guest_error {
            let error = result.unwrap_err();
            assert_eq!(error, original);
            assert_eq!(error.as_opaque().unwrap().as_object().unwrap(), object);
        } else {
            assert_eq!(result.unwrap(), 42.into());
        }
        assert!(counts.returned_ready.get());
        counts.assert_once();
        assert_eq!(observer.status(), Ok(JobStatus::Completed));
        assert_eq!(observer.fault(), None);
        assert!(wakes.0.load(Ordering::Relaxed) > 0);
        assert_eq!(
            poll_wait(&mut wait, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
        assert_terminal_steps(&mut job, &observer, JobStatus::Completed, false);
        drop(job);
        drop(owner);
        counts.assert_once();
        assert_eq!(observer.status(), Ok(JobStatus::Completed));
        assert_eq!(observer.fault(), None);
        assert_eq!(
            poll_wait(&mut wait, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
    }
}

fn ready_disposal_unwind(guest_error: bool) {
    let mut owner = context();
    let original = JsError::from_opaque(7.into()).unwrap();
    let result = if guest_error {
        Err(original)
    } else {
        Ok(42.into())
    };
    let counts = Counts::default();
    let payload = Arc::new(HostPanic {
        site: "Ready task capture disposal",
    });
    let mut job = ready_job(result, counts.clone(), Some(payload.clone()));
    let observer = job.observer();
    let alias = observer.clone();
    let (mut wait, wakes) = subscribed(&observer);
    assert!(matches!(
        step(&mut job, &owner.host_access()),
        JobStep::Prepared
    ));
    assert_eq!(counts.polls.get(), 0);
    assert_eq!(counts.captures_dropped.get(), 0);
    let before = wakes.0.load(Ordering::Relaxed);

    let external =
        catch_unwind(AssertUnwindSafe(|| step(&mut job, &owner.host_access()))).unwrap_err();
    assert_same_panic(external.as_ref(), &payload);
    assert!(
        counts.returned_ready.get(),
        "the source produced Ready before its capture Drop unwound"
    );
    counts.assert_once();
    let terminal = JobStatus::Retired(JobRetirement::Interrupted);
    assert_eq!(observer.status(), Ok(terminal));
    assert_eq!(alias.status(), Ok(terminal));
    assert_eq!(
        observer.fault(),
        None,
        "an authored host Drop unwind is not a structural job defect"
    );
    assert!(
        wakes.0.load(Ordering::Relaxed) > before,
        "the already parked native waiter must be notified"
    );
    assert_eq!(
        poll_wait(&mut wait, Waker::noop()),
        Poll::Ready(Ok(terminal))
    );
    let mut late = alias.wait().unwrap();
    assert_eq!(
        poll_wait(&mut late, Waker::noop()),
        Poll::Ready(Ok(terminal))
    );
    assert_terminal_steps(&mut job, &observer, terminal, true);
    drop(job);
    drop(owner);
    counts.assert_once();
    assert_eq!(observer.status(), Ok(terminal));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        poll_wait(&mut wait, Waker::noop()),
        Poll::Ready(Ok(terminal))
    );
}

#[test]
fn ready_success_capture_disposal_unwind_retires_and_notifies_without_replay() {
    ready_disposal_unwind(false);
}

#[test]
fn ready_guest_error_capture_disposal_unwind_retires_and_notifies_without_replay() {
    ready_disposal_unwind(true);
}

#[test]
fn source_poll_unwind_control_already_retires_and_preserves_the_original_payload() {
    let mut owner = context();
    let counts = Counts::default();
    let observed = counts.clone();
    let payload = Arc::new(HostPanic {
        site: "source poll",
    });
    let supplied = payload.clone();
    let capture = TaskCapture {
        counts: counts.clone(),
        disposal_payload: None,
    };
    let mut job = NativeAsyncJob::new(move |_| {
        observed.factories.set(observed.factories.get() + 1);
        Ok(ContextTask::from_poll(
            move |_, _| -> Poll<JsResult<JsValue>> {
                let _keep = &capture;
                observed.polls.set(observed.polls.get() + 1);
                panic_any(supplied.clone());
            },
        ))
    });
    let observer = job.observer();
    let (mut wait, wakes) = subscribed(&observer);
    assert!(matches!(
        step(&mut job, &owner.host_access()),
        JobStep::Prepared
    ));
    let external =
        catch_unwind(AssertUnwindSafe(|| step(&mut job, &owner.host_access()))).unwrap_err();
    assert_same_panic(external.as_ref(), &payload);
    assert!(
        !counts.returned_ready.get(),
        "this control unwinds inside polling, before Ready"
    );
    counts.assert_once();
    let terminal = JobStatus::Retired(JobRetirement::Interrupted);
    assert_eq!(observer.status(), Ok(terminal));
    assert_eq!(observer.fault(), None);
    assert!(wakes.0.load(Ordering::Relaxed) > 0);
    assert_eq!(
        poll_wait(&mut wait, Waker::noop()),
        Poll::Ready(Ok(terminal))
    );
    assert_terminal_steps(&mut job, &observer, terminal, true);
    drop(job);
    counts.assert_once();
}

struct PanickingWake {
    calls: AtomicUsize,
    armed: AtomicBool,
    payload: Arc<HostPanic>,
}
impl PanickingWake {
    fn notify(&self) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.armed.swap(false, Ordering::AcqRel) {
            panic_any(self.payload.clone());
        }
    }
}
impl Wake for PanickingWake {
    fn wake(self: Arc<Self>) {
        self.notify();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.notify();
    }
}

#[test]
fn completed_waiter_notification_unwind_keeps_completed_without_fabricating_a_fault() {
    let mut owner = context();
    let counts = Counts::default();
    let mut job = ready_job(Ok(42.into()), counts.clone(), None);
    let observer = job.observer();
    let alias = observer.clone();
    let payload = Arc::new(HostPanic {
        site: "completed observer wake",
    });
    let wake = Arc::new(PanickingWake {
        calls: AtomicUsize::new(0),
        armed: AtomicBool::new(true),
        payload: payload.clone(),
    });
    let mut panicking_wait = observer.wait().unwrap();
    assert!(poll_wait(&mut panicking_wait, &Waker::from(wake.clone())).is_pending());
    let (mut passive_wait, passive_wakes) = subscribed(&observer);
    assert!(matches!(
        step(&mut job, &owner.host_access()),
        JobStep::Prepared
    ));

    let external =
        catch_unwind(AssertUnwindSafe(|| step(&mut job, &owner.host_access()))).unwrap_err();
    assert_same_panic(external.as_ref(), &payload);
    assert!(counts.returned_ready.get());
    counts.assert_once();
    assert_eq!(wake.calls.load(Ordering::Relaxed), 1);
    assert!(
        passive_wakes.0.load(Ordering::Relaxed) > 0,
        "finish the other already-owned notice despite the host wake unwind"
    );
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(alias.status(), Ok(JobStatus::Completed));
    assert_eq!(
        observer.fault(),
        None,
        "notification unwind cannot retroactively invent a Task fault"
    );
    for wait in [&mut panicking_wait, &mut passive_wait] {
        assert_eq!(
            poll_wait(wait, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
    }
    assert_terminal_steps(&mut job, &observer, JobStatus::Completed, false);
    drop(job);
    drop(owner);
    counts.assert_once();
    assert_eq!(
        wake.calls.load(Ordering::Relaxed),
        1,
        "the completed notice cannot replay"
    );
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        poll_wait(&mut alias.wait().unwrap(), Waker::noop()),
        Poll::Ready(Ok(JobStatus::Completed))
    );
}
