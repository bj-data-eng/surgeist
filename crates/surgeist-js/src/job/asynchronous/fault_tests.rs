use super::*;
use std::{
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    sync::{Condvar, TryLockError, atomic::AtomicUsize},
    time::{Duration, Instant},
};

#[derive(Default)]
struct WakeCount {
    count: Mutex<usize>,
    changed: Condvar,
}
impl WakeCount {
    fn record(&self) {
        *self.count.lock().unwrap() += 1;
        self.changed.notify_all();
    }
    fn await_count(&self, expected: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut count = self.count.lock().unwrap();
        while *count < expected {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "native driver wake deadline");
            count = self.changed.wait_timeout(count, remaining).unwrap().0;
        }
        assert_eq!(*count, expected);
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
#[derive(Debug)]
struct PoisonTag;
fn poison<T>(mutex: &Mutex<T>) {
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _guard = mutex.lock().unwrap();
        std::panic::panic_any(PoisonTag);
    }))
    .unwrap_err();
    assert!(panic.is::<PoisonTag>());
}
fn unlocked<T>(mutex: &Mutex<T>) -> bool {
    match mutex.try_lock() {
        Ok(guard) => {
            drop(guard);
            true
        }
        Err(TryLockError::Poisoned(error)) => {
            drop(error.into_inner());
            true
        }
        Err(TryLockError::WouldBlock) => false,
    }
}

#[test]
fn actual_task_driver_poison_faults_original_observers_and_wakes_the_detached_driver() {
    for through_register in [false, true] {
        let job = NativeAsyncJob::new(|_| panic!("notification must not execute the task factory"));
        let observer = job.observer();
        let driver = Arc::new(WakeCount::default());
        job.register_driver(&Waker::from(driver.clone()));
        job.control.wake.ready.store(false, Ordering::Release);
        let wait_count = Arc::new(WakeCount::default());
        let mut wait = observer.wait().unwrap();
        assert_eq!(
            Pin::new(&mut wait).poll(&mut TaskContext::from_waker(&Waker::from(
                wait_count.clone()
            ))),
            Poll::Pending
        );
        poison(&job.control.wake.driver);
        if through_register {
            job.register_driver(Waker::noop());
        } else {
            Wake::wake_by_ref(&job.control.wake);
        }
        let expected = JobFault::new(JobFaultOwner::TaskWake, JobFaultCause::PoisonedWaker);
        driver.await_count(1);
        wait_count.await_count(1);
        assert!(job.is_ready());
        assert_eq!(observer.status(), Err(expected));
        assert_eq!(observer.fault(), Some(expected));
        assert_eq!(
            Pin::new(&mut wait).poll(&mut TaskContext::from_waker(Waker::noop())),
            Poll::Ready(Err(expected))
        );
        job.register_driver(Waker::noop());
        assert_eq!(observer.status(), Err(expected));
        drop(job);
        driver.await_count(1);
        assert_eq!(observer.status(), Err(expected));
    }
}

struct DriverDrop {
    owner: std::sync::Weak<TaskWake>,
    calls: Arc<AtomicUsize>,
    valid: Arc<AtomicBool>,
}
impl Wake for DriverDrop {
    fn wake(self: Arc<Self>) {
        drop(self);
    }
}
impl Drop for DriverDrop {
    fn drop(&mut self) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(owner) = self.owner.upgrade() {
            let free = unlocked(&owner.driver);
            self.valid.fetch_and(free, Ordering::SeqCst);
            if free {
                // Exercise actual replacement after the previous guard released.
                owner.register(Waker::noop());
            }
        }
    }
}
#[test]
fn replacing_or_retiring_a_driver_disposes_the_previous_waker_outside_its_slot_lock() {
    for retire in [false, true] {
        let job = NativeAsyncJob::new(|_| panic!("replacement must not execute the task factory"));
        let calls = Arc::new(AtomicUsize::new(0));
        let valid = Arc::new(AtomicBool::new(true));
        let old = Waker::from(Arc::new(DriverDrop {
            owner: Arc::downgrade(&job.control.wake),
            calls: calls.clone(),
            valid: valid.clone(),
        }));
        job.register_driver(&old);
        drop(old);
        if retire {
            job.control.wake.retire();
        } else {
            job.register_driver(Waker::noop());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(valid.load(Ordering::SeqCst));
        assert_eq!(job.observer().status(), Ok(JobStatus::Pending));
        assert_eq!(job.observer().fault(), None);
    }
}

struct DriverWake {
    owner: std::sync::Weak<TaskWake>,
    calls: Arc<AtomicUsize>,
    valid: Arc<AtomicBool>,
}
impl Wake for DriverWake {
    fn wake(self: Arc<Self>) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let owner = self.owner.upgrade().unwrap();
        let free = unlocked(&owner.driver);
        self.valid.fetch_and(free, Ordering::SeqCst);
        if free {
            owner.register(Waker::noop());
        }
    }
}
#[test]
fn an_ordinary_driver_wake_may_reenter_registration_without_retiring_the_task() {
    let job = NativeAsyncJob::new(|_| panic!("waking must not execute the task factory"));
    let calls = Arc::new(AtomicUsize::new(0));
    let valid = Arc::new(AtomicBool::new(true));
    job.register_driver(&Waker::from(Arc::new(DriverWake {
        owner: Arc::downgrade(&job.control.wake),
        calls: calls.clone(),
        valid: valid.clone(),
    })));
    job.control.wake.ready.store(false, Ordering::Release);
    Wake::wake_by_ref(&job.control.wake);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(valid.load(Ordering::SeqCst));
    assert!(job.is_ready());
    assert_eq!(job.observer().status(), Ok(JobStatus::Pending));
    assert_eq!(job.observer().fault(), None);
}

struct PayloadDrop(Rc<Cell<usize>>);
impl Drop for PayloadDrop {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn a_faulted_prepared_task_is_not_polled_again_and_keeps_payload_until_explicit_disposal() {
    use crate::{error::JobError, execution::ExecutionOptions};
    for owner_drop in [false, true] {
        let polls = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let payload = PayloadDrop(drops.clone());
        let observed_polls = polls.clone();
        let mut job = NativeAsyncJob::new(move |_| {
            Ok(ContextTask::from_poll(move |_, _| {
                let _keep = &payload;
                observed_polls.set(observed_polls.get() + 1);
                Poll::Pending
            }))
        });
        let observer = job.observer();
        let builder = Context::builder();
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(1_000_000);
        let mut context = builder.build().unwrap();
        let expected = JobFault::new(JobFaultOwner::TaskWake, JobFaultCause::PoisonedWaker);
        let outcome = context.with_execution(ExecutionOptions::new(4096), |access| {
            let host = access.into_host_access();
            let mut cx = TaskContext::from_waker(Waker::noop());
            assert!(matches!(job.run_step(&mut cx, &host), JobStep::Prepared));
            assert!(matches!(job.run_step(&mut cx, &host), JobStep::Pending));
            assert_eq!(polls.get(), 1);
            poison(&job.control.wake.driver);
            Wake::wake_by_ref(&job.control.wake);
            match job.run_step(&mut cx, &host) {
                JobStep::Complete(Err(error)) => assert_eq!(error.as_engine(), Some(&EngineError::Job(JobError::Fault(expected)))),
                other => panic!("the first direct fault result must be delivered: {other:?}"),
            }
            assert!(matches!(job.run_step(&mut cx, &host), JobStep::Finished(Err(error)) if error == expected));
            assert_eq!(polls.get(), 1);
            assert_eq!(drops.get(), 0);
            Ok(())
        }).unwrap();
        outcome.result.unwrap();
        assert_eq!(observer.status(), Err(expected));
        assert_eq!(drops.get(), 0);
        if owner_drop {
            drop(context);
        } else {
            job.control.retire(JobRetirement::Cancelled);
        }
        assert_eq!(drops.get(), 1);
        assert_eq!(polls.get(), 1);
        assert_eq!(observer.status(), Err(expected));
        drop(job);
        assert_eq!(drops.get(), 1);
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CollisionDisposalUnwind(u8);

struct CollisionDisposalCapture {
    tag: u8,
    index: usize,
    counts: Rc<[Cell<usize>; 2]>,
    mask: Rc<Cell<u8>>,
    first: Rc<Cell<Option<u8>>>,
}
impl Drop for CollisionDisposalCapture {
    fn drop(&mut self) {
        self.counts[self.index].set(self.counts[self.index].get() + 1);
        self.mask.set(self.mask.get() | self.tag);
        if self.first.get().is_none() {
            self.first.set(Some(self.tag));
        }
        std::panic::panic_any(CollisionDisposalUnwind(self.tag));
    }
}

fn collision_disposal_job(
    counts: Rc<[Cell<usize>; 2]>,
    mask: Rc<Cell<u8>>,
    first: Rc<Cell<Option<u8>>>,
) -> NativeAsyncJob {
    let retained = CollisionDisposalCapture {
        tag: 1,
        index: 0,
        counts: counts.clone(),
        mask: mask.clone(),
        first: first.clone(),
    };
    let job = NativeAsyncJob::new(move |_| {
        let _keep = &retained;
        panic!("collision cleanup cannot execute the retained factory")
    });
    let active = CollisionDisposalCapture {
        tag: 2,
        index: 1,
        counts,
        mask,
        first,
    };
    // Exercise the actual engine-owned restoration-collision route. It must
    // quarantine both independently owned payloads without disposing either.
    job.control
        .restore_state(State::Task(ContextTask::from_poll(move |_, _| {
            let _keep = &active;
            panic!("collision cleanup cannot poll the quarantined task")
        })));
    job
}

#[test]
fn collision_retirement_finishes_both_payload_disposals_and_resumes_the_first_unwind() {
    let counts = Rc::new([Cell::new(0), Cell::new(0)]);
    let mask = Rc::new(Cell::new(0));
    let first = Rc::new(Cell::new(None));
    let job = collision_disposal_job(counts.clone(), mask.clone(), first.clone());
    let observer = job.observer();
    let expected = JobFault::new(JobFaultOwner::Task, JobFaultCause::StateInconsistency);
    assert_eq!(observer.status(), Err(expected));
    assert_eq!(mask.get(), 0);
    assert_eq!(first.get(), None);

    let unwind = catch_unwind(AssertUnwindSafe(|| {
        job.control.retire(JobRetirement::Cancelled);
    }))
    .unwrap_err();
    assert_eq!(mask.get(), 3, "both actual owner payloads must dispose");
    assert_eq!([counts[0].get(), counts[1].get()], [1, 1]);
    let first_tag = first
        .get()
        .expect("an actual destructor must initiate unwind");
    assert_eq!(
        unwind.downcast_ref::<CollisionDisposalUnwind>(),
        Some(&CollisionDisposalUnwind(first_tag))
    );
    assert_eq!(observer.status(), Err(expected));
    assert_eq!(observer.fault(), Some(expected));
    drop(job);
    assert_eq!(mask.get(), 3);
    assert_eq!([counts[0].get(), counts[1].get()], [1, 1]);
}

#[test]
fn collision_final_drop_finishes_both_payload_disposals_and_resumes_the_first_unwind() {
    let counts = Rc::new([Cell::new(0), Cell::new(0)]);
    let mask = Rc::new(Cell::new(0));
    let first = Rc::new(Cell::new(None));
    let job = collision_disposal_job(counts.clone(), mask.clone(), first.clone());
    let observer = job.observer();
    let expected = JobFault::new(JobFaultOwner::Task, JobFaultCause::StateInconsistency);
    assert_eq!(observer.status(), Err(expected));
    assert_eq!(mask.get(), 0);
    assert_eq!(first.get(), None);

    let unwind = catch_unwind(AssertUnwindSafe(|| drop(job))).unwrap_err();
    assert_eq!(mask.get(), 3, "both actual owner payloads must dispose");
    assert_eq!([counts[0].get(), counts[1].get()], [1, 1]);
    let first_tag = first
        .get()
        .expect("an actual destructor must initiate unwind");
    assert_eq!(
        unwind.downcast_ref::<CollisionDisposalUnwind>(),
        Some(&CollisionDisposalUnwind(first_tag))
    );
    assert_eq!(observer.status(), Err(expected));
    assert_eq!(observer.fault(), Some(expected));
}
