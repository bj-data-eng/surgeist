use super::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Barrier, Condvar, TryLockError, atomic::AtomicUsize},
    task::Wake,
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
            assert!(
                !remaining.is_zero(),
                "expected {expected} native wakes; observed {}",
                *count
            );
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
fn poll(wait: &mut JobWait) -> Poll<Observation> {
    Pin::new(wait).poll(&mut Context::from_waker(Waker::noop()))
}
fn park<W: Wake + Send + Sync + 'static>(observer: &JobObserver, wake: Arc<W>) -> JobWait {
    let mut wait = observer.wait().unwrap();
    let waker = Waker::from(wake);
    assert_eq!(
        Pin::new(&mut wait).poll(&mut Context::from_waker(&waker)),
        Poll::Pending
    );
    wait
}
fn expected(owner: JobFaultOwner, cause: JobFaultCause) -> JobFault {
    JobFault::new(owner, cause)
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
fn malformed_disposition_and_missing_cause_notify_all_original_waiters_with_typed_faults() {
    for (raw, cause) in [
        (255, JobFaultCause::InvalidDisposition { raw: 255 }),
        (FAULTED, JobFaultCause::MissingCause),
    ] {
        let observer = JobObserver::new();
        let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
        let mut waits: Vec<_> = counts
            .iter()
            .map(|count| park(&observer, count.clone()))
            .collect();
        observer.inner.status.store(raw, Ordering::Release);
        let fault = expected(JobFaultOwner::Observer, cause);
        assert_eq!(observer.status(), Err(fault));
        assert_eq!(observer.fault(), Some(fault));
        for (count, wait) in counts.iter().zip(&mut waits) {
            count.await_count(1);
            assert_eq!(poll(wait), Poll::Ready(Err(fault)));
        }
        observer.complete();
        observer.retire(JobRetirement::Cancelled);
        observer.inner.status.store(254, Ordering::Release);
        assert_eq!(observer.status(), Err(fault));
        assert_eq!(observer.fault(), Some(fault));
        assert_eq!(
            observer.wait().unwrap_err(),
            JobRegistrationError::Fault(fault)
        );
    }
}

#[test]
fn invalid_free_links_quarantine_the_physical_listener_bank_including_entries_after_holes() {
    for mode in 0..3 {
        let observer = JobObserver::new();
        let first_count = Arc::new(WakeCount::default());
        let last_count = Arc::new(WakeCount::default());
        let mut first = park(&observer, first_count.clone());
        let hole = observer.wait().unwrap();
        let hole_index = hole._registration.as_ref().unwrap().index;
        let mut last = park(&observer, last_count.clone());
        drop(hole);
        let index = {
            let mut listeners = observer.inner.listeners.lock().unwrap();
            let index = match mode {
                0 => listeners.entries.len(),
                1 => first._registration.as_ref().unwrap().index,
                _ => hole_index,
            };
            if mode == 2 {
                listeners.entries[index] = Entry::Free(Some(index));
            }
            listeners.free = Some(index);
            index
        };
        let fault = expected(
            JobFaultOwner::Observer,
            JobFaultCause::InvalidFreeLink { index },
        );
        assert_eq!(
            observer.wait().unwrap_err(),
            JobRegistrationError::Fault(fault)
        );
        assert_eq!(observer.status(), Err(fault));
        first_count.await_count(1);
        last_count.await_count(1);
        assert_eq!(poll(&mut first), Poll::Ready(Err(fault)));
        assert_eq!(poll(&mut last), Poll::Ready(Err(fault)));
        assert_eq!(
            observer.wait().unwrap_err(),
            JobRegistrationError::Fault(fault)
        );
    }
}

#[test]
fn registry_poison_is_a_fault_and_not_permission_to_follow_a_damaged_free_head() {
    let observer = JobObserver::new();
    let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
    let mut waits: Vec<_> = counts
        .iter()
        .map(|count| park(&observer, count.clone()))
        .collect();
    {
        observer.inner.listeners.lock().unwrap().free = Some(usize::MAX);
    }
    poison(&observer.inner.listeners);
    let fault = expected(JobFaultOwner::Observer, JobFaultCause::PoisonedRegistry);
    assert_eq!(
        observer.wait().unwrap_err(),
        JobRegistrationError::Fault(fault)
    );
    assert_eq!(observer.fault(), Some(fault));
    for (count, wait) in counts.iter().zip(&mut waits) {
        count.await_count(1);
        assert_eq!(poll(wait), Poll::Ready(Err(fault)));
    }
    assert_eq!(
        observer.wait().unwrap_err(),
        JobRegistrationError::Fault(fault)
    );
}

#[test]
fn a_stale_registration_cannot_free_the_replacement_listener() {
    let observer = JobObserver::new();
    let original = observer.wait().unwrap();
    let registration = original._registration.as_ref().unwrap();
    let index = registration.index;
    let stale = Registration {
        observer: registration.observer.clone(),
        index,
        expected: registration.expected.clone(),
        _listener: registration._listener.clone(),
    };
    drop(original);
    let count = Arc::new(WakeCount::default());
    let mut replacement = park(&observer, count.clone());
    drop(stale);
    let fault = expected(
        JobFaultOwner::Observer,
        JobFaultCause::InvalidSlotIdentity { index },
    );
    assert_eq!(observer.status(), Err(fault));
    count.await_count(1);
    assert_eq!(poll(&mut replacement), Poll::Ready(Err(fault)));
    assert_eq!(
        observer.wait().unwrap_err(),
        JobRegistrationError::Fault(fault)
    );
}

#[test]
fn drop_only_invalid_registration_is_retained_for_the_next_fallible_entry() {
    let observer = JobObserver::new();
    let mut invalid = observer.wait().unwrap();
    invalid._registration.as_mut().unwrap().index = usize::MAX;
    let count = Arc::new(WakeCount::default());
    let mut original = park(&observer, count.clone());
    drop(invalid);
    let fault = expected(
        JobFaultOwner::Observer,
        JobFaultCause::InvalidIndex { index: usize::MAX },
    );
    assert_eq!(
        observer.wait().unwrap_err(),
        JobRegistrationError::Fault(fault)
    );
    assert_eq!(observer.status(), Err(fault));
    count.await_count(1);
    assert_eq!(poll(&mut original), Poll::Ready(Err(fault)));
}

#[test]
fn normal_terminal_results_survive_a_later_detected_waiter_fault() {
    for terminal in [
        JobStatus::Completed,
        JobStatus::Retired(JobRetirement::OwnerDropped),
    ] {
        let observer = JobObserver::new();
        let mut original = observer.wait().unwrap();
        observer.publish(terminal);
        assert_eq!(poll(&mut original), Poll::Ready(Ok(terminal)));
        poison(&original.signal.waker);
        // A late notification detects a real poisoned slot after commitment.
        original.signal.notify(Ok(terminal));
        let fault = expected(JobFaultOwner::ObserverWaiter, JobFaultCause::PoisonedWaker);
        assert_eq!(observer.status(), Ok(terminal));
        assert_eq!(observer.fault(), Some(fault));
        assert_eq!(poll(&mut original), Poll::Ready(Ok(terminal)));
        assert_eq!(
            observer.wait().unwrap_err(),
            JobRegistrationError::Fault(fault)
        );
    }
}

#[test]
fn preparing_a_fault_cause_does_not_win_a_normal_terminal_transition() {
    let observer = JobObserver::new();
    let count = Arc::new(WakeCount::default());
    let mut original = park(&observer, count.clone());
    let index = original._registration.as_ref().unwrap().index;
    observer.inner.listeners.lock().unwrap().free = Some(index);
    let fault = expected(
        JobFaultOwner::Observer,
        JobFaultCause::InvalidFreeLink { index },
    );
    // Pause a legal publication transient after detecting the bad link and
    // initializing its immutable cause, before attempting the Faulted CAS.
    observer.inner.first_fault.set(fault).unwrap();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    observer.complete();
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), Some(fault));
    count.await_count(1);
    assert_eq!(poll(&mut original), Poll::Ready(Ok(JobStatus::Completed)));
    assert_eq!(
        observer.wait().unwrap_err(),
        JobRegistrationError::Fault(fault)
    );
}

#[test]
fn racing_terminal_publications_never_expose_an_uninitialized_fault_cause() {
    for _ in 0..16 {
        let observer = JobObserver::new();
        let count = Arc::new(WakeCount::default());
        let mut original = park(&observer, count.clone());
        let index = original._registration.as_ref().unwrap().index;
        observer.inner.listeners.lock().unwrap().free = Some(index);
        let fault = expected(
            JobFaultOwner::Observer,
            JobFaultCause::InvalidFreeLink { index },
        );
        let barrier = Arc::new(Barrier::new(3));
        std::thread::scope(|scope| {
            let completing = observer.clone();
            let start = barrier.clone();
            scope.spawn(move || {
                start.wait();
                completing.complete();
            });
            let faulting = observer.clone();
            let start = barrier.clone();
            scope.spawn(move || {
                start.wait();
                faulting.record_fault(fault);
            });
            barrier.wait();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match observer.status() {
                    Ok(JobStatus::Pending) => {
                        assert!(
                            Instant::now() < deadline,
                            "both terminal publishers remained pending"
                        );
                        std::thread::yield_now();
                    }
                    Ok(JobStatus::Completed) => break,
                    Err(actual) => {
                        assert_eq!(actual, fault);
                        assert_eq!(observer.fault(), Some(fault));
                        break;
                    }
                    other => panic!("unexpected terminal race result: {other:?}"),
                }
            }
        });
        let result = observer.status();
        assert!(result == Ok(JobStatus::Completed) || result == Err(fault));
        assert_eq!(observer.fault(), Some(fault));
        count.await_count(1);
        assert_eq!(poll(&mut original), Poll::Ready(result));
    }
}

#[test]
fn a_poisoned_pending_waiter_notifies_the_full_original_wait_set() {
    let observer = JobObserver::new();
    let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
    let mut waits: Vec<_> = counts
        .iter()
        .map(|count| park(&observer, count.clone()))
        .collect();
    poison(&waits[0].signal.waker);
    let fault = expected(JobFaultOwner::ObserverWaiter, JobFaultCause::PoisonedWaker);
    assert_eq!(poll(&mut waits[0]), Poll::Ready(Err(fault)));
    for (count, wait) in counts.iter().zip(&mut waits) {
        count.await_count(1);
        assert_eq!(poll(wait), Poll::Ready(Err(fault)));
    }
    assert_eq!(observer.fault(), Some(fault));
}

struct DropReentry {
    observer: JobObserver,
    slot: Weak<WaitSignal>,
    drops: Arc<AtomicUsize>,
    valid: Arc<AtomicBool>,
}
impl Wake for DropReentry {
    fn wake(self: Arc<Self>) {
        drop(self);
    }
}
impl Drop for DropReentry {
    fn drop(&mut self) {
        let free = unlocked(&self.observer.inner.listeners)
            && self.slot.upgrade().is_none_or(|slot| unlocked(&slot.waker));
        self.valid.fetch_and(free, Ordering::SeqCst);
        if free {
            self.valid
                .fetch_and(self.observer.wait().is_ok(), Ordering::SeqCst);
        }
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn replacement_and_final_waker_disposal_can_reenter_after_owner_locks_are_released() {
    for replace in [false, true] {
        let observer = JobObserver::new();
        let mut wait = observer.wait().unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        let valid = Arc::new(AtomicBool::new(true));
        let waker = Waker::from(Arc::new(DropReentry {
            observer: observer.clone(),
            slot: Arc::downgrade(&wait.signal),
            drops: drops.clone(),
            valid: valid.clone(),
        }));
        assert_eq!(
            Pin::new(&mut wait).poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
        drop(waker);
        if replace {
            assert_eq!(poll(&mut wait), Poll::Pending);
        }
        drop(wait);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(valid.load(Ordering::SeqCst));
        assert_eq!(observer.status(), Ok(JobStatus::Pending));
        observer.complete();
    }
}

#[derive(Debug)]
struct WakePanic(usize);
struct PanickingWake {
    observer: JobObserver,
    tag: usize,
    first: Arc<OnceLock<usize>>,
    mask: Arc<AtomicUsize>,
    valid: Arc<AtomicBool>,
    completion: Arc<WakeCount>,
}
impl Wake for PanickingWake {
    fn wake(self: Arc<Self>) {
        self.valid
            .fetch_and(unlocked(&self.observer.inner.listeners), Ordering::SeqCst);
        self.valid.fetch_and(
            self.observer.status() == Ok(JobStatus::Completed),
            Ordering::SeqCst,
        );
        self.mask.fetch_or(1 << self.tag, Ordering::SeqCst);
        self.completion.record();
        self.first.get_or_init(|| self.tag);
        std::panic::panic_any(WakePanic(self.tag));
    }
}

#[test]
fn external_waker_panics_finish_all_notifications_then_resume_the_first_unwind() {
    let observer = JobObserver::new();
    let first = Arc::new(OnceLock::new());
    let mask = Arc::new(AtomicUsize::new(0));
    let valid = Arc::new(AtomicBool::new(true));
    let completion = Arc::new(WakeCount::default());
    let mut waits = Vec::new();
    for tag in [0, 1] {
        waits.push(park(
            &observer,
            Arc::new(PanickingWake {
                observer: observer.clone(),
                tag,
                first: first.clone(),
                mask: mask.clone(),
                valid: valid.clone(),
                completion: completion.clone(),
            }),
        ));
    }
    let count = Arc::new(WakeCount::default());
    waits.push(park(&observer, count.clone()));
    let panic = catch_unwind(AssertUnwindSafe(|| observer.complete())).unwrap_err();
    assert_eq!(
        panic.downcast_ref::<WakePanic>().unwrap().0,
        *first.get().unwrap()
    );
    completion.await_count(2);
    count.await_count(1);
    assert_eq!(mask.load(Ordering::SeqCst), 3);
    assert!(valid.load(Ordering::SeqCst));
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    for wait in &mut waits {
        assert_eq!(poll(wait), Poll::Ready(Ok(JobStatus::Completed)));
    }
    let mut late = observer.wait().unwrap();
    assert_eq!(poll(&mut late), Poll::Ready(Ok(JobStatus::Completed)));
}

#[test]
fn observer_fault_cleanup_finishes_waits_inside_an_already_stopped_operation() {
    use crate::{
        Context as EngineContext, Source,
        error::{EngineError, RuntimeLimitError},
        execution::ExecutionOptions,
    };
    let observer = JobObserver::new();
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&observer, count.clone());
    let builder = EngineContext::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let fault = expected(
        JobFaultOwner::Observer,
        JobFaultCause::InvalidDisposition { raw: 255 },
    );
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let first = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                first.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            observer.inner.status.store(255, Ordering::Release);
            assert_eq!(observer.status(), Err(fault));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    count.await_count(1);
    assert_eq!(poll(&mut wait), Poll::Ready(Err(fault)));
}

#[test]
fn a_registry_fault_wakes_the_real_native_task_driver_and_all_original_waiters() {
    use crate::job::NativeAsyncJob;
    let job = NativeAsyncJob::new(|_| panic!("a parked driver must not execute the task factory"));
    let observer = job.observer();
    let driver = Arc::new(WakeCount::default());
    job.register_driver(&Waker::from(driver.clone()));
    let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
    let mut waits: Vec<_> = counts
        .iter()
        .map(|count| park(&observer, count.clone()))
        .collect();
    observer.inner.listeners.lock().unwrap().free = Some(usize::MAX);
    let fault = expected(
        JobFaultOwner::Observer,
        JobFaultCause::InvalidFreeLink { index: usize::MAX },
    );
    assert_eq!(
        observer.wait().unwrap_err(),
        JobRegistrationError::Fault(fault)
    );
    driver.await_count(1);
    for (count, wait) in counts.iter().zip(&mut waits) {
        count.await_count(1);
        assert_eq!(poll(wait), Poll::Ready(Err(fault)));
    }
    drop(job);
    assert_eq!(observer.status(), Err(fault));
    driver.await_count(1);
}

#[test]
fn a_panicking_real_task_driver_does_not_skip_fault_notification_of_original_waiters() {
    use crate::job::NativeAsyncJob;
    struct DriverPanic;
    impl Wake for DriverPanic {
        fn wake(self: Arc<Self>) {
            std::panic::panic_any(PoisonTag);
        }
    }
    let job = NativeAsyncJob::new(|_| panic!("notification must not execute the task factory"));
    let observer = job.observer();
    job.register_driver(&Waker::from(Arc::new(DriverPanic)));
    let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
    let mut waits: Vec<_> = counts
        .iter()
        .map(|count| park(&observer, count.clone()))
        .collect();
    observer.inner.status.store(255, Ordering::Release);
    let panic = catch_unwind(AssertUnwindSafe(|| observer.status())).unwrap_err();
    assert!(panic.is::<PoisonTag>());
    let fault = expected(
        JobFaultOwner::Observer,
        JobFaultCause::InvalidDisposition { raw: 255 },
    );
    for (count, wait) in counts.iter().zip(&mut waits) {
        count.await_count(1);
        assert_eq!(poll(wait), Poll::Ready(Err(fault)));
    }
    assert_eq!(observer.status(), Err(fault));
    assert_eq!(observer.fault(), Some(fault));
    drop(job);
    assert_eq!(observer.status(), Err(fault));
}
