use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, ContextId, HostContext, JsData, JsError, JsObject, JsValue,
    error::{EngineError, EngineResourceError},
    execution::ExecutionOptions,
    gc::heap::Trace,
    job::{
        ContextTask, JobCall, JobFault, JobObserver, JobRegistrationError, JobResourceError,
        JobRetirement, JobStatus, JobStep, JobWait, NativeAsyncJob, NativeJob,
    },
};

const DEADLINE: Duration = Duration::from_secs(5);
fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}
fn poll<F: Future + Unpin>(future: &mut F, waker: &Waker) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(waker))
}
fn step(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> JobStep {
    job.run_step(&mut TaskContext::from_waker(Waker::noop()), host)
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
    let count = Arc::new(WakeCount::default());
    assert!(poll(&mut wait, &Waker::from(count.clone())).is_pending());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    (wait, count)
}

#[test]
fn rootless_observation_types_are_thread_safe_and_registration_resources_keep_their_category() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<JobObserver>();
    send_sync::<JobWait>();
    send_sync::<JobFault>();
    send_sync::<JobRegistrationError>();
    send_sync::<ContextId>();
    for (resource, expected) in [
        (
            JobResourceError::Allocation,
            EngineResourceError::Allocation,
        ),
        (
            JobResourceError::SizeOverflow,
            EngineResourceError::SizeOverflow,
        ),
    ] {
        let refusal = JobRegistrationError::Resource(resource);
        let error = JsError::from(refusal);
        assert_eq!(error.as_engine(), Some(&EngineError::Resource(expected)));
        let mut owner = context();
        assert_eq!(
            error.clone().into_opaque(owner.access()).unwrap_err(),
            error
        );
    }
}

#[test]
fn ordinary_success_and_guest_failure_both_publish_completed_and_ready_wait_results() {
    for guest_error in [false, true] {
        let mut owner = context();
        let original = JsError::from_opaque(7.into()).unwrap();
        let supplied = original.clone();
        let calls = Rc::new(Cell::new(0));
        let seen = calls.clone();
        let job = NativeJob::new(move |_| {
            seen.set(seen.get() + 1);
            if guest_error {
                Err(supplied)
            } else {
                Ok(JsValue::new(42))
            }
        });
        let observer = job.observer();
        let alias = observer.clone();
        let (mut wait, wakes) = subscribed(&observer);
        match job.call(owner.access()) {
            JobCall::Complete(result) => {
                if guest_error {
                    assert_eq!(result.unwrap_err(), original);
                } else {
                    assert_eq!(result.unwrap(), JsValue::new(42));
                }
            }
            JobCall::Deferred { .. } => panic!("ordinary direct invocation must enter"),
        }
        assert_eq!(calls.get(), 1);
        assert!(wakes.0.load(Ordering::Relaxed) > 0);
        assert_eq!(observer.status(), Ok(JobStatus::Completed));
        assert_eq!(
            poll(&mut wait, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
        drop(owner);
        assert_eq!(alias.status(), Ok(JobStatus::Completed));
        assert_eq!(alias.fault(), None);
        let mut late = alias.wait().unwrap();
        assert_eq!(
            poll(&mut late, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
        assert_eq!(
            poll(&mut wait, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
    }
}

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn dropping_unbound_work_notifies_waiters_without_retaining_or_running_its_factory() {
    let drops = Rc::new(Cell::new(0));
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let probe = DropProbe(drops.clone());
    let job = NativeAsyncJob::new(move |_| {
        let _keep = &probe;
        seen.set(seen.get() + 1);
        Ok(ContextTask::from_future(std::future::ready(Ok(
            JsValue::undefined(),
        ))))
    });
    let observer = job.observer();
    let (mut wait, wakes) = subscribed(&observer);
    drop(job);
    let expected = JobStatus::Retired(JobRetirement::Dropped);
    assert_eq!(drops.get(), 1);
    assert_eq!(calls.get(), 0);
    assert!(wakes.0.load(Ordering::Relaxed) > 0);
    assert_eq!(observer.status(), Ok(expected));
    assert_eq!(observer.fault(), None);
    assert_eq!(poll(&mut wait, Waker::noop()), Poll::Ready(Ok(expected)));
    assert_eq!(
        poll(&mut observer.wait().unwrap(), Waker::noop()),
        Poll::Ready(Ok(expected))
    );
}

#[derive(Debug, JsData, Trace)]
struct NativeLeaf {
    #[heap_trace(skip)]
    drops: Rc<Cell<usize>>,
}
impl Drop for NativeLeaf {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn observers_and_waiters_do_not_keep_the_exact_context_or_a_retired_tasks_heap_roots_alive() {
    let mut owner = context();
    let runtime = owner.runtime().clone();
    let peer = Context::builder().runtime(runtime.clone()).build().unwrap();
    let required = owner.id();
    assert_ne!(peer.id(), required);
    let drops = Rc::new(Cell::new(0));
    let leaf = JsObject::from_proto_and_data(
        &runtime,
        None,
        NativeLeaf {
            drops: drops.clone(),
        },
    )
    .unwrap();
    let polls = Rc::new(Cell::new(0));
    let seen = polls.clone();
    let mut job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, _| {
            let _root = &leaf;
            seen.set(seen.get() + 1);
            Poll::Pending
        }))
    });
    let observer = job.observer();
    let (mut wait, wakes) = subscribed(&observer);
    assert!(matches!(
        step(&mut job, &owner.host_access()),
        JobStep::Prepared
    ));
    assert!(matches!(
        step(&mut job, &owner.host_access()),
        JobStep::Pending
    ));
    runtime.collect().unwrap();
    assert_eq!(drops.get(), 0, "the live task owns its authored leaf");
    assert_eq!(polls.get(), 1);
    drop(owner);
    assert!(wakes.0.load(Ordering::Relaxed) > 0);
    let expected = JobStatus::Retired(JobRetirement::OwnerDropped);
    assert_eq!(observer.status(), Ok(expected));
    assert_eq!(observer.fault(), None);
    assert_eq!(poll(&mut wait, Waker::noop()), Poll::Ready(Ok(expected)));
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "the rootless observations and live peer cannot retain the old task leaf"
    );
    drop(job);
    drop(peer);
    assert_eq!(observer.status(), Ok(expected));
    assert_eq!(poll(&mut wait, Waker::noop()), Poll::Ready(Ok(expected)));
}

struct ChannelWake(mpsc::SyncSender<()>);
impl Wake for ChannelWake {
    fn wake(self: Arc<Self>) {
        let _ = self.0.try_send(());
    }
    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.0.try_send(());
    }
}

#[test]
fn cross_thread_wait_registration_before_or_after_terminal_publication_cannot_lose_the_result() {
    for completed in [false, true] {
        for register_first in [false, true] {
            let mut owner = context();
            let calls = Rc::new(Cell::new(0));
            let seen = calls.clone();
            let job = NativeJob::new(move |_| {
                seen.set(seen.get() + 1);
                Ok(JsValue::new(42))
            });
            let observer = job.observer();
            let expected = if completed {
                JobStatus::Completed
            } else {
                JobStatus::Retired(JobRetirement::Dropped)
            };
            let remote = observer.clone();
            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            let (done_tx, done_rx) = mpsc::sync_channel(1);
            let thread = std::thread::spawn(move || {
                let (wake_tx, wake_rx) = mpsc::sync_channel(1);
                let waker = Waker::from(Arc::new(ChannelWake(wake_tx)));
                let mut wait = if register_first {
                    let mut wait = remote.wait().unwrap();
                    assert!(poll(&mut wait, &waker).is_pending());
                    ready_tx.send(()).unwrap();
                    release_rx
                        .recv_timeout(DEADLINE)
                        .expect("terminal publication command");
                    wake_rx
                        .recv_timeout(DEADLINE)
                        .expect("the previously subscribed waiter must be notified");
                    wait
                } else {
                    ready_tx.send(()).unwrap();
                    release_rx
                        .recv_timeout(DEADLINE)
                        .expect("terminal publication precedes registration");
                    remote.wait().unwrap()
                };
                assert_eq!(poll(&mut wait, &waker), Poll::Ready(Ok(expected)));
                assert_eq!(remote.status(), Ok(expected));
                assert_eq!(remote.fault(), None);
                done_tx.send(()).unwrap();
            });
            ready_rx
                .recv_timeout(DEADLINE)
                .expect("the rootless worker must reach the selected registration boundary");
            if completed {
                match job.call(owner.access()) {
                    JobCall::Complete(result) => {
                        result.unwrap();
                    }
                    JobCall::Deferred { .. } => panic!("ordinary terminal producer"),
                }
            } else {
                drop(job);
            }
            release_tx.send(()).unwrap();
            done_rx
                .recv_timeout(DEADLINE)
                .expect("the rootless worker must observe the winning terminal result");
            thread.join().unwrap();
            assert_eq!(calls.get(), usize::from(completed));
            assert_eq!(observer.status(), Ok(expected));
        }
    }
}

#[test]
fn concurrent_registration_and_completion_observe_one_terminal_result_without_a_lost_wake() {
    let mut owner = context();
    let job = NativeJob::new(|_| Ok(JsValue::new(42)));
    let observer = job.observer();
    let remote = observer.clone();
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (start_tx, start_rx) = mpsc::sync_channel(1);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || {
        ready_tx.send(()).unwrap();
        start_rx
            .recv_timeout(DEADLINE)
            .expect("the concurrent registration command must arrive");
        let mut wait = remote.wait().unwrap();
        let (wake_tx, wake_rx) = mpsc::sync_channel(1);
        let waker = Waker::from(Arc::new(ChannelWake(wake_tx)));
        if poll(&mut wait, &waker).is_pending() {
            wake_rx
                .recv_timeout(DEADLINE)
                .expect("a registration racing completion must not lose the terminal wake");
        }
        assert_eq!(
            poll(&mut wait, &waker),
            Poll::Ready(Ok(JobStatus::Completed))
        );
        done_tx.send(()).unwrap();
    });
    ready_rx
        .recv_timeout(DEADLINE)
        .expect("the rootless registration worker must be ready");
    start_tx.send(()).unwrap();
    match job.call(owner.access()) {
        JobCall::Complete(result) => {
            result.unwrap();
        }
        JobCall::Deferred { .. } => panic!("ordinary concurrent completion"),
    }
    done_rx
        .recv_timeout(DEADLINE)
        .expect("racing terminal observation must finish");
    thread.join().unwrap();
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
}

struct ReentrantWake {
    requested: AtomicBool,
    failed: AtomicBool,
    command: mpsc::SyncSender<()>,
    reply: Mutex<mpsc::Receiver<()>>,
}
impl ReentrantWake {
    fn run(&self) {
        if self.requested.swap(true, Ordering::AcqRel) {
            return;
        }
        if self.command.try_send(()).is_err() {
            self.failed.store(true, Ordering::Release);
            return;
        }
        if self.reply.lock().unwrap().recv_timeout(DEADLINE).is_err() {
            self.failed.store(true, Ordering::Release);
        }
    }
}
impl Wake for ReentrantWake {
    fn wake(self: Arc<Self>) {
        self.run();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.run();
    }
}

#[test]
fn terminal_wake_can_finish_another_registration_before_returning_to_the_publisher() {
    let mut owner = context();
    let job = NativeJob::new(|_| Ok(JsValue::new(42)));
    let observer = job.observer();
    let mut first = observer.wait().unwrap();
    let second = observer.wait().unwrap();
    let remote = observer.clone();
    let (command_tx, command_rx) = mpsc::sync_channel(1);
    let (reply_tx, reply_rx) = mpsc::sync_channel(1);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || {
        command_rx
            .recv_timeout(DEADLINE)
            .expect("the original waiter must request lawful registration cleanup");
        drop(second);
        assert_eq!(remote.status(), Ok(JobStatus::Completed));
        assert_eq!(remote.fault(), None);
        let mut late = remote.wait().unwrap();
        assert_eq!(
            poll(&mut late, Waker::noop()),
            Poll::Ready(Ok(JobStatus::Completed))
        );
        reply_tx.send(()).unwrap();
        done_tx.send(()).unwrap();
    });
    let wake = Arc::new(ReentrantWake {
        requested: AtomicBool::new(false),
        failed: AtomicBool::new(false),
        command: command_tx,
        reply: Mutex::new(reply_rx),
    });
    assert!(poll(&mut first, &Waker::from(wake.clone())).is_pending());
    match job.call(owner.access()) {
        JobCall::Complete(result) => {
            result.unwrap();
        }
        JobCall::Deferred { .. } => panic!("ordinary completion notifying native waiters"),
    }
    // If a protected publisher blocked cleanup, the bounded wake returns first;
    // the worker can then finish once publication releases its access.
    done_rx
        .recv_timeout(DEADLINE)
        .expect("the registration cleanup worker must finish after notification returns");
    thread.join().unwrap();
    assert!(wake.requested.load(Ordering::Acquire));
    assert!(
        !wake.failed.load(Ordering::Acquire),
        "registration cleanup must finish while the notification callback is active"
    );
    assert_eq!(
        poll(&mut first, Waker::noop()),
        Poll::Ready(Ok(JobStatus::Completed))
    );
    assert_eq!(observer.fault(), None);
}

#[test]
fn terminal_native_observation_spends_no_entry_budget_and_never_turns_into_ready_pending() {
    let mut owner = context();
    let job = NativeJob::new(|_| Ok(JsValue::undefined()));
    let observer = job.observer();
    let mut wait = observer.wait().unwrap();
    drop(job);
    let expected = JobStatus::Retired(JobRetirement::Dropped);
    let outcome = owner
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(0)
                .with_job_invocation_limit(0)
                .with_job_poll_limit(0),
            |_| {
                for _ in 0..3 {
                    assert_eq!(observer.status(), Ok(expected));
                    assert_eq!(observer.fault(), None);
                    assert_eq!(poll(&mut wait, Waker::noop()), Poll::Ready(Ok(expected)));
                    let mut late = observer.wait().unwrap();
                    assert_eq!(poll(&mut late, Waker::noop()), Poll::Ready(Ok(expected)));
                }
                Ok(())
            },
        )
        .unwrap();
    outcome.result.unwrap();
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(outcome.report.job_preparations().unwrap(), 0);
    assert_eq!(outcome.report.job_invocations().unwrap(), 0);
    assert_eq!(outcome.report.job_polls().unwrap(), 0);
    assert!(!outcome.report.effects_started());
}

#[test]
fn finished_native_steps_preserve_terminal_results_under_a_fresh_contexts_zero_budgets() {
    for ordinary_completion in [false, true] {
        let mut original = context();
        let factories = Rc::new(Cell::new(0));
        let polls = Rc::new(Cell::new(0));
        let made = factories.clone();
        let polled = polls.clone();
        let mut job = NativeAsyncJob::new(move |_| {
            made.set(made.get() + 1);
            Ok(ContextTask::from_poll(move |_, _| {
                polled.set(polled.get() + 1);
                Poll::Ready(Ok(JsValue::new(42)))
            }))
        });
        let observer = job.observer();
        assert!(matches!(
            step(&mut job, &original.host_access()),
            JobStep::Prepared
        ));
        if ordinary_completion {
            let JobStep::Complete(result) = step(&mut job, &original.host_access()) else {
                panic!("authored ordinary completion")
            };
            assert_eq!(result.unwrap(), JsValue::new(42));
        }
        drop(original);
        let expected = if ordinary_completion {
            JobStatus::Completed
        } else {
            JobStatus::Retired(JobRetirement::OwnerDropped)
        };
        assert_eq!(observer.status(), Ok(expected));
        let mut receiver = context();
        let outcome = receiver
            .with_execution(
                ExecutionOptions::new(0)
                    .with_job_preparation_limit(0)
                    .with_job_invocation_limit(0)
                    .with_job_poll_limit(0),
                |access| {
                    let host = access.into_host_access();
                    if !ordinary_completion {
                        let JobStep::Complete(result) = step(&mut job, &host) else {
                            panic!("the undelivered retirement is returned once")
                        };
                        assert_eq!(
                            result.unwrap_err().as_engine(),
                            Some(&EngineError::JobRetired(JobRetirement::OwnerDropped))
                        );
                    }
                    for _ in 0..2 {
                        let JobStep::Finished(result) = step(&mut job, &host) else {
                            panic!("terminal observation cannot dispatch another action")
                        };
                        assert_eq!(result, Ok(expected));
                    }
                    Ok(())
                },
            )
            .unwrap();
        outcome.result.unwrap();
        assert_eq!(outcome.report.instructions().unwrap(), 0);
        assert_eq!(outcome.report.job_preparations().unwrap(), 0);
        assert_eq!(outcome.report.job_invocations().unwrap(), 0);
        assert_eq!(outcome.report.job_polls().unwrap(), 0);
        assert!(!outcome.report.effects_started());
        assert_eq!(factories.get(), 1);
        assert_eq!(polls.get(), usize::from(ordinary_completion));
        assert_eq!(observer.status(), Ok(expected));
        assert_eq!(observer.fault(), None);
    }
}
