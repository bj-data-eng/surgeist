use super::*;
use crate::{
    JsValue,
    job::{JobStatus, JobWait},
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Mutex, atomic::AtomicUsize},
    task::{Context as TaskContext, Wake, Waker},
};

struct CaptureDrop {
    queues: Weak<RuntimeJobQueues>,
    drops: Rc<Cell<usize>>,
    outside_store: Rc<Cell<bool>>,
}
impl Drop for CaptureDrop {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        if let Some(queues) = self.queues.upgrade() {
            self.outside_store
                .set(self.outside_store.get() && queues.store.try_borrow_mut().is_ok());
        }
    }
}
struct Accepted {
    slot: Rc<Slot>,
    observer: JobObserver,
    calls: Rc<Cell<usize>>,
    drops: Rc<Cell<usize>>,
    outside_store: Rc<Cell<bool>>,
}
fn owner(executor: Rc<SimpleJobExecutor>) -> (Context, Rc<RuntimeJobQueues>) {
    let builder = Context::builder().job_executor(executor.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    let mut context = builder.build().unwrap();
    let queues = executor.attach_context(&mut context).unwrap();
    (context, queues)
}
fn accept(context: &mut Context, queues: &Rc<RuntimeJobQueues>) -> Accepted {
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let outside_store = Rc::new(Cell::new(true));
    let seen = calls.clone();
    let probe = CaptureDrop {
        queues: Rc::downgrade(queues),
        drops: drops.clone(),
        outside_store: outside_store.clone(),
    };
    let job = GenericJob::new(
        move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Ok(JsValue::undefined())
        },
        context.realm().clone(),
    );
    let observer = job.observer();
    assert!(matches!(
        context.enqueue_job(job.into()).unwrap(),
        JobAcceptance::Accepted
    ));
    let slot = {
        let store = queues.store.borrow();
        store
            .node(store.ordinary.generic.tail.unwrap())
            .unwrap()
            .clone()
    };
    Accepted {
        slot,
        observer,
        calls,
        drops,
        outside_store,
    }
}
fn pending(wait: &mut JobWait, waker: &Waker) {
    assert!(
        Pin::new(wait)
            .poll(&mut TaskContext::from_waker(waker))
            .is_pending()
    );
}
type ObserverSnapshot = Vec<Result<JobStatus, JobFault>>;
struct PeerWake {
    peers: Vec<JobObserver>,
    snapshots: Arc<Mutex<Vec<ObserverSnapshot>>>,
    wakes: Arc<AtomicUsize>,
}
impl PeerWake {
    fn observe(&self) {
        let current = self.peers.iter().map(JobObserver::status).collect();
        self.snapshots.lock().unwrap().push(current);
        self.wakes.fetch_add(1, Ordering::Relaxed);
    }
}
impl Wake for PeerWake {
    fn wake(self: Arc<Self>) {
        self.observe();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.observe();
    }
}

#[test]
fn inaccessible_executor_registry_marks_its_entire_real_recipient_set_before_the_first_wake() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut left, left_queue) = owner(executor.clone());
    let (mut right, right_queue) = owner(executor.clone());
    assert!(!left.runtime().same_heap(right.runtime()));
    let first = accept(&mut left, &left_queue);
    let second = accept(&mut right, &right_queue);
    let unrelated_executor = Rc::new(SimpleJobExecutor::new());
    let (mut unrelated, unrelated_queue) = owner(unrelated_executor);
    let independent = accept(&mut unrelated, &unrelated_queue);
    let peers = vec![first.observer.clone(), second.observer.clone()];
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let wakes = Arc::new(AtomicUsize::new(0));
    let waker = Waker::from(Arc::new(PeerWake {
        peers,
        snapshots: snapshots.clone(),
        wakes: wakes.clone(),
    }));
    let mut first_wait = first.observer.wait().unwrap();
    let mut second_wait = second.observer.wait().unwrap();
    pending(&mut first_wait, &waker);
    pending(&mut second_wait, &waker);
    let expected = JobFault::new(
        JobFaultOwner::ExecutorRegistry,
        JobFaultCause::InaccessibleRegistry,
    );

    // Inject actual private registry contention while an actual owner release
    // guard still owns completion. No notification is invented under this loan.
    let release = RegistryRelease {
        registry: executor.registry.clone(),
        completed: true,
        unwinding_at_entry: std::thread::panicking(),
    };
    let protected = executor.registry.queues.borrow_mut();
    let Err(QueueError::Fault(fault)) = executor.queues_for(&left) else {
        panic!("the inaccessible real registry must return its typed first fault")
    };
    assert_eq!(fault, expected);
    assert_eq!(executor.registry.first_fault.get(), Some(expected));
    assert!(snapshots.lock().unwrap().is_empty());
    assert_eq!(first.observer.status(), Ok(JobStatus::Pending));
    assert_eq!(second.observer.status(), Ok(JobStatus::Pending));
    drop(protected);
    drop(release);
    let first_notification = snapshots
        .lock()
        .unwrap()
        .first()
        .cloned()
        .expect("the actual pending native waiters must be notified");
    assert!(wakes.load(Ordering::Relaxed) >= 2);
    for accepted in [&first, &second] {
        assert_eq!(accepted.observer.status(), Err(expected));
        assert_eq!(accepted.observer.fault(), Some(expected));
        assert_eq!(accepted.calls.get(), 0);
        assert!(accepted.slot.status.get() != Status::Retired);
        assert_eq!(
            accepted.drops.get(),
            0,
            "fault quarantine does not dispose accepted payloads"
        );
    }
    assert_eq!(independent.observer.status(), Ok(JobStatus::Pending));
    assert_eq!(independent.observer.fault(), None);
    // A second actual cleanup fault cannot replace the registry's first cause.
    drop(DrainGuard(left_queue.clone()));
    assert_eq!(left_queue.first_fault.get(), Some(expected));
    left_queue.clear().unwrap();
    right_queue.clear().unwrap();
    for accepted in [&first, &second] {
        assert_eq!(accepted.drops.get(), 1);
        assert_eq!(accepted.calls.get(), 0);
        assert!(accepted.outside_store.get());
        assert_eq!(accepted.observer.status(), Err(expected));
    }
    assert_eq!(
        Pin::new(&mut first_wait).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Err(expected))
    );
    assert_eq!(
        Pin::new(&mut second_wait).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Err(expected))
    );
    unrelated.run_jobs().unwrap();
    assert_eq!(independent.calls.get(), 1);
    assert_eq!(independent.observer.status(), Ok(JobStatus::Completed));
    assert_eq!(independent.observer.fault(), None);
    // Assert after deliberate cleanup so a failing mark-order probe retains no
    // test-owned blocked driver or undisposed accepted payload.
    assert_eq!(
        first_notification,
        vec![Err(expected), Err(expected)],
        "every observer in this executor registry's actual recipient set must be fault-marked before any user wake"
    );
}

use crate::{execution::ExecutionOptions, job::NativeJob};
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};

fn engine_fault(error: &crate::JsError) -> JobFault {
    let Some(EngineError::Job(crate::error::JobError::Fault(fault))) = error.as_engine() else {
        panic!("the queue must expose its actual typed structural fault: {error:?}")
    };
    *fault
}
fn quarantined(accepted: &Accepted, expected: JobFault) {
    assert_eq!(accepted.observer.status(), Err(expected));
    assert_eq!(accepted.observer.fault(), Some(expected));
    assert_eq!(accepted.calls.get(), 0);
    assert_eq!(accepted.drops.get(), 0);
}

#[test]
fn invalid_index_reciprocal_neighbor_and_live_slot_identity_fail_during_priced_inspection() {
    for family in 0..3 {
        let executor = Rc::new(SimpleJobExecutor::new());
        let (mut context, queues) = owner(executor);
        let first = accept(&mut context, &queues);
        let second = accept(&mut context, &queues);
        let cause = match family {
            0 => {
                let mut store = queues.store.borrow_mut();
                let invalid = store.entries.len();
                store.ordinary.generic.head = Some(NodeId(invalid));
                store.ordinary.generic.tail = Some(NodeId(invalid));
                JobFaultCause::InvalidIndex { index: invalid }
            }
            1 => {
                second.slot.previous.set(None);
                JobFaultCause::InvalidQueueLink {
                    index: first.slot.index.0,
                }
            }
            2 => {
                queues
                    .store
                    .borrow_mut()
                    .entries
                    .swap(first.slot.index.0, second.slot.index.0);
                JobFaultCause::InvalidSlotIdentity {
                    index: first.slot.index.0,
                }
            }
            _ => unreachable!(),
        };
        let expected = defect(cause);
        let outcome = context
            .with_execution(ExecutionOptions::new(0), |access| {
                queues.claim(ClaimSelection::Generic, &access.into_host_access())
            })
            .unwrap();
        let Err(error) = outcome.result else {
            panic!("the corrupt real selected node cannot be claimed")
        };
        assert_eq!(engine_fault(&error), expected);
        assert!(!outcome.report.effects_started());
        quarantined(&first, expected);
        quarantined(&second, expected);
        drop(DrainGuard(queues.clone()));
        assert_eq!(queues.first_fault.get(), Some(expected));
        queues.clear().unwrap();
        for accepted in [&first, &second] {
            assert_eq!(accepted.drops.get(), 1);
            assert_eq!(accepted.calls.get(), 0);
            assert!(accepted.outside_store.get());
            assert_eq!(accepted.observer.status(), Err(expected));
        }
    }
}

fn complete_first(context: &mut Context, queues: &Rc<RuntimeJobQueues>, accepted: &Accepted) {
    let host = context.host_access();
    let batch = queues.claim(ClaimSelection::Generic, &host).unwrap();
    let mut payload = Payload::take(&accepted.slot).unwrap().unwrap();
    let Resident::Generic(job) = payload.command_slot().unwrap().take().unwrap() else {
        panic!("the fixture's actual accepted generic command")
    };
    let result = host.with(|access| match job.call(access) {
        JobCall::Complete(result) => result,
        JobCall::Deferred { .. } => {
            panic!("the ordinary accepted control must execute before unlink")
        }
    });
    queues.finish_action(&accepted.slot, &host, result).unwrap();
    drop(payload);
    drop(batch);
    assert_eq!(accepted.calls.get(), 1);
    assert_eq!(accepted.drops.get(), 1);
    assert_eq!(accepted.observer.status(), Ok(JobStatus::Completed));
}

#[test]
fn invalid_free_link_returns_the_original_unaccepted_command_and_preserves_other_residents() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor);
    let completed = accept(&mut context, &queues);
    let resident = accept(&mut context, &queues);
    complete_first(&mut context, &queues, &completed);
    {
        let mut store = queues.store.borrow_mut();
        let Entry::Free(next) = &mut store.entries[completed.slot.index.0] else {
            panic!("real completed work leaves its reusable free entry")
        };
        *next = Some(completed.slot.index);
    }
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let outside_store = Rc::new(Cell::new(true));
    let seen = calls.clone();
    let probe = CaptureDrop {
        queues: Rc::downgrade(&queues),
        drops: drops.clone(),
        outside_store: outside_store.clone(),
    };
    let proposed = GenericJob::new(
        move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Ok(JsValue::undefined())
        },
        context.realm().clone(),
    );
    let observer = proposed.observer();
    let refused = context.enqueue_job(proposed.into()).unwrap_err();
    let expected = defect(JobFaultCause::InvalidFreeLink {
        index: completed.slot.index.0,
    });
    assert_eq!(engine_fault(refused.cause().error()), expected);
    quarantined(&resident, expected);
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 0);
    assert_eq!(completed.observer.status(), Ok(JobStatus::Completed));
    let (original, _) = refused.into_parts();
    drop(original);
    assert_eq!(drops.get(), 1);
    assert!(outside_store.get());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    queues.clear().unwrap();
    assert_eq!(resident.drops.get(), 1);
    assert_eq!(resident.calls.get(), 0);
}

#[test]
fn an_actual_stale_slot_cannot_unlink_a_new_allocation_that_reused_its_index() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor);
    let completed = accept(&mut context, &queues);
    complete_first(&mut context, &queues, &completed);
    let replacement = accept(&mut context, &queues);
    assert_eq!(completed.slot.index, replacement.slot.index);
    let result = queues.with_store(|store| {
        store
            .unlink(&completed.slot)
            .map(|_| ())
            .map_err(QueueError::Fault)
    });
    let Err(QueueError::Fault(fault)) = result else {
        panic!("the actual old allocation cannot authorize removal of its replacement")
    };
    let expected = defect(JobFaultCause::InvalidSlotIdentity {
        index: completed.slot.index.0,
    });
    assert_eq!(fault, expected);
    quarantined(&replacement, expected);
    assert_eq!(completed.observer.status(), Ok(JobStatus::Completed));
    assert_eq!(completed.observer.fault(), None);
    queues.clear().unwrap();
    assert_eq!(replacement.drops.get(), 1);
    assert_eq!(completed.drops.get(), 1);
}

#[test]
fn missing_existing_clock_bucket_preserves_the_real_timer_payload_until_cancellation() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor);
    let resident = accept(&mut context, &queues);
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let outside_store = Rc::new(Cell::new(true));
    let seen = calls.clone();
    let probe = CaptureDrop {
        queues: Rc::downgrade(&queues),
        drops: drops.clone(),
        outside_store: outside_store.clone(),
    };
    let callback = NativeJob::with_realm(
        move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Ok(JsValue::undefined())
        },
        context.realm().clone(),
    );
    let observer = callback.observer();
    let timer = super::super::TimeoutJob::new(callback, 1);
    let token = timer.cancellation_token().clone();
    assert!(matches!(
        context.enqueue_job(timer.into()).unwrap(),
        JobAcceptance::Accepted
    ));
    let slot = {
        let mut store = queues.store.borrow_mut();
        let bucket = store
            .clocks
            .pop()
            .expect("the actual timer created its clock bucket");
        store.node(bucket.bucket.head.unwrap()).unwrap().clone()
    };
    let result =
        queues.with_store(|store| store.unlink(&slot).map(|_| ()).map_err(QueueError::Fault));
    let Err(QueueError::Fault(fault)) = result else {
        panic!("a missing existing bucket is a typed queue defect")
    };
    let expected = defect(JobFaultCause::MissingClockBucket);
    assert_eq!(fault, expected);
    quarantined(&resident, expected);
    assert_eq!(observer.status(), Err(expected));
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 0);
    assert!(!token.revoked());
    queues.clear().unwrap();
    assert_eq!(drops.get(), 1);
    assert!(outside_store.get());
    assert!(token.revoked());
    assert_eq!(resident.drops.get(), 1);
}

#[test]
fn real_protected_store_contention_defers_notifications_until_its_owner_releases_access() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor);
    let first = accept(&mut context, &queues);
    let second = accept(&mut context, &queues);
    let expected = defect(JobFaultCause::InaccessibleRegistry);
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let wakes = Arc::new(AtomicUsize::new(0));
    let waker = Waker::from(Arc::new(PeerWake {
        peers: vec![first.observer.clone(), second.observer.clone()],
        snapshots: snapshots.clone(),
        wakes: wakes.clone(),
    }));
    let mut first_wait = first.observer.wait().unwrap();
    let mut second_wait = second.observer.wait().unwrap();
    pending(&mut first_wait, &waker);
    pending(&mut second_wait, &waker);
    let result: QueueResult<()> = queues.with_store(|_| {
        let result: QueueResult<()> = queues.with_store(|_| Ok(()));
        assert!(matches!(&result, Err(QueueError::Fault(fault)) if *fault == expected));
        assert_eq!(queues.first_fault.get(), Some(expected));
        assert!(snapshots.lock().unwrap().is_empty());
        assert_eq!(first.observer.status(), Ok(JobStatus::Pending));
        assert_eq!(second.observer.status(), Ok(JobStatus::Pending));
        result
    });
    assert!(matches!(result, Err(QueueError::Fault(fault)) if fault == expected));
    quarantined(&first, expected);
    quarantined(&second, expected);
    assert!(wakes.load(Ordering::Relaxed) >= 2);
    assert!(
        snapshots
            .lock()
            .unwrap()
            .iter()
            .all(|snapshot| snapshot == &vec![Err(expected), Err(expected)])
    );
    queues.clear().unwrap();
    assert_eq!(first.drops.get(), 1);
    assert_eq!(second.drops.get(), 1);
    assert!(first.outside_store.get());
    assert!(second.outside_store.get());
}

#[test]
fn checked_drain_and_reservation_overflow_refuse_without_faulting_or_consuming_resident_work() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor.clone());
    let resident = accept(&mut context, &queues);
    queues.active_drains.set(usize::MAX);
    let error = executor.run_jobs(context.access()).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::Resource(EngineResourceError::SizeOverflow))
    );
    assert_eq!(queues.active_drains.get(), usize::MAX);
    assert_eq!(queues.first_fault.get(), None);
    assert_eq!(resident.observer.status(), Ok(JobStatus::Pending));
    assert_eq!(resident.calls.get(), 0);
    assert_eq!(resident.drops.get(), 0);
    queues.active_drains.set(0);
    let time = JsInstant::new(7, 0);
    let reservation = queues.reserve_clock(time).unwrap();
    {
        let mut store = queues.store.borrow_mut();
        let index = store.clock_index(time).unwrap();
        store.clocks[index].bucket.reservations = usize::MAX;
    }
    let overflow = queues.reserve_clock(time);
    assert!(matches!(
        overflow,
        Err(QueueError::Resource(EngineResourceError::SizeOverflow))
    ));
    assert_eq!(queues.first_fault.get(), None);
    assert_eq!(resident.drops.get(), 0);
    {
        let mut store = queues.store.borrow_mut();
        let index = store.clock_index(time).unwrap();
        assert_eq!(store.clocks[index].bucket.reservations, usize::MAX);
        // Restore the one real outstanding guard after the injected quote.
        store.clocks[index].bucket.reservations = 1;
    }
    drop(reservation);
    context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap()
        .result
        .unwrap();
    assert_eq!(resident.calls.get(), 1);
    assert_eq!(resident.drops.get(), 1);
    assert_eq!(resident.observer.status(), Ok(JobStatus::Completed));
}

#[test]
fn reservation_counter_underflow_is_a_first_fault_without_premature_payload_disposal() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor);
    let resident = accept(&mut context, &queues);
    let time = JsInstant::new(8, 0);
    let reservation = queues.reserve_clock(time).unwrap();
    {
        let mut store = queues.store.borrow_mut();
        let index = store.clock_index(time).unwrap();
        store.clocks[index].bucket.reservations = 0;
    }
    drop(reservation);
    let expected = defect(JobFaultCause::CounterUnderflow);
    assert_eq!(queues.first_fault.get(), Some(expected));
    quarantined(&resident, expected);
    queues.clear().unwrap();
    assert_eq!(resident.drops.get(), 1);
    assert!(resident.outside_store.get());
}

#[derive(Debug, Eq, PartialEq)]
struct CollisionUnwind(&'static str);
struct PanickingPeerWake {
    observe: PeerWake,
    tag: &'static str,
}
impl Wake for PanickingPeerWake {
    fn wake(self: Arc<Self>) {
        self.observe.observe();
        panic_any(CollisionUnwind(self.tag));
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.observe.observe();
        panic_any(CollisionUnwind(self.tag));
    }
}

#[test]
fn payload_restore_collision_retains_both_real_commands_and_the_initiating_external_unwind() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut context, queues) = owner(executor);
    let first = accept(&mut context, &queues);
    let second = accept(&mut context, &queues);
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let wakes = Arc::new(AtomicUsize::new(0));
    let peers = vec![first.observer.clone(), second.observer.clone()];
    let first_waker = Waker::from(Arc::new(PanickingPeerWake {
        observe: PeerWake {
            peers: peers.clone(),
            snapshots: snapshots.clone(),
            wakes: wakes.clone(),
        },
        tag: "first notification",
    }));
    let second_waker = Waker::from(Arc::new(PanickingPeerWake {
        observe: PeerWake {
            peers,
            snapshots: snapshots.clone(),
            wakes: wakes.clone(),
        },
        tag: "second notification",
    }));
    let mut first_wait = first.observer.wait().unwrap();
    let mut second_wait = second.observer.wait().unwrap();
    pending(&mut first_wait, &first_waker);
    pending(&mut second_wait, &second_waker);
    let batch = queues
        .claim(ClaimSelection::Generic, &context.host_access())
        .unwrap();
    let first_payload = Payload::take(&first.slot).unwrap().unwrap();
    let mut second_payload = Payload::take(&second.slot).unwrap().unwrap();
    // Move, never copy, the second actual accepted record into the first slot.
    // Both real commands must survive the owning restoration's collision path.
    let second_record = second_payload.record.take().unwrap();
    first
        .slot
        .payload
        .set(PayloadState::Resident(second_record));
    drop(second_payload);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _restoring = first_payload;
        panic_any(CollisionUnwind("initiating external unwind"));
    }))
    .unwrap_err();
    assert_eq!(
        unwind.downcast_ref::<CollisionUnwind>(),
        Some(&CollisionUnwind("initiating external unwind"))
    );
    let expected = defect(JobFaultCause::StateInconsistency);
    quarantined(&first, expected);
    quarantined(&second, expected);
    assert_eq!(queues.first_fault.get(), Some(expected));
    assert_eq!(wakes.load(Ordering::Relaxed), 2);
    assert!(
        snapshots
            .lock()
            .unwrap()
            .iter()
            .all(|snapshot| snapshot == &vec![Err(expected), Err(expected)])
    );
    drop(batch);
    queues.clear().unwrap();
    for accepted in [&first, &second] {
        assert_eq!(accepted.calls.get(), 0);
        assert_eq!(accepted.drops.get(), 1);
        assert!(accepted.outside_store.get());
        assert_eq!(accepted.observer.status(), Err(expected));
    }
}

struct PriorityMaximumClock;
impl crate::context::time::Clock for PriorityMaximumClock {
    fn now(&self) -> JsInstant {
        JsInstant::new(u64::MAX, 999_999_999)
    }
    fn system_time_millis(&self) -> i64 {
        0
    }
}

fn registry_fault_precedes_new_timer_refusal(foreign_selected_realm: bool) {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut left, queues) = owner(executor.clone());
    let resident = accept(&mut left, &queues);
    // This valid receiving Context must attach while its shared registry is
    // healthy. A pure constant clock makes MAX+1ns unrepresentable.
    let builder = Context::builder()
        .runtime(left.runtime().clone())
        .job_executor(executor.clone())
        .clock(Rc::new(PriorityMaximumClock));
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    let mut maximum = builder.build().unwrap();
    assert!(maximum.runtime().same_heap(left.runtime()));
    let (mut unrelated, unrelated_queue) = owner(Rc::new(SimpleJobExecutor::new()));
    let independent = accept(&mut unrelated, &unrelated_queue);
    assert!(!unrelated.runtime().same_heap(left.runtime()));

    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let outside_store = Rc::new(Cell::new(true));
    let seen = calls.clone();
    let probe = CaptureDrop {
        queues: Rc::downgrade(&queues),
        drops: drops.clone(),
        outside_store: outside_store.clone(),
    };
    let delay = std::time::Duration::from_nanos(if foreign_selected_realm { 7 } else { 1 });
    let selected_realm = if foreign_selected_realm {
        unrelated.realm().clone()
    } else {
        maximum.realm().clone()
    };
    let proposed = super::super::TimeoutJob::with_realm(
        move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Ok(JsValue::undefined())
        },
        selected_realm,
        delay,
    );
    let observer = proposed.observer();
    let token = proposed.cancellation_token().clone();

    // Cause the registry's actual first fault through its protected owner,
    // rather than writing the latch or synthesizing an error for publication.
    let release = RegistryRelease {
        registry: executor.registry.clone(),
        completed: true,
        unwinding_at_entry: std::thread::panicking(),
    };
    let protected = executor.registry.queues.borrow_mut();
    let expected = JobFault::new(
        JobFaultOwner::ExecutorRegistry,
        JobFaultCause::InaccessibleRegistry,
    );
    let Err(QueueError::Fault(first)) = executor.queues_for(&left) else {
        panic!("actual protected registry access must establish the first owner fault")
    };
    assert_eq!(first, expected);
    drop(protected);
    drop(release);
    quarantined(&resident, expected);

    let refusal = if foreign_selected_realm {
        left.enqueue_job(proposed.into())
    } else {
        maximum.enqueue_job(proposed.into())
    }
    .unwrap_err();
    assert_eq!(
        engine_fault(refusal.cause().error()),
        expected,
        "a preexisting registry fault precedes new deadline and selected-realm refusal"
    );
    let Job::TimeoutJob(returned) = refusal.job() else {
        panic!("refusal must retain the actual original timer wrapper")
    };
    assert_eq!(returned.timeout().as_nanos(), delay.as_nanos());
    assert_eq!(returned.observer().status(), Ok(JobStatus::Pending));
    assert_eq!(returned.observer().fault(), None);
    assert!(!returned.cancellation_token().revoked());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert!(!token.revoked());
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 0);
    quarantined(&resident, expected);

    let (original, cause) = refusal.into_parts();
    assert_eq!(engine_fault(cause.error()), expected);
    drop(original);
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1);
    assert!(outside_store.get());
    assert!(token.revoked());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
    quarantined(&resident, expected);
    queues.clear().unwrap();
    assert_eq!(resident.calls.get(), 0);
    assert_eq!(resident.drops.get(), 1);
    assert!(resident.outside_store.get());
    assert_eq!(resident.observer.status(), Err(expected));
    assert_eq!(executor.registry.first_fault.get(), Some(expected));
    unrelated.run_jobs().unwrap();
    assert_eq!(independent.calls.get(), 1);
    assert_eq!(independent.drops.get(), 1);
    assert_eq!(independent.observer.status(), Ok(JobStatus::Completed));
    assert_eq!(independent.observer.fault(), None);
}

#[test]
fn existing_registry_fault_precedes_a_new_overflowing_timer_and_returns_the_original_wrapper() {
    registry_fault_precedes_new_timer_refusal(false);
}

#[test]
fn existing_registry_fault_precedes_a_new_foreign_selected_timer_and_returns_the_original_wrapper()
{
    registry_fault_precedes_new_timer_refusal(true);
}

#[test]
fn one_unavailable_real_bank_defers_every_registry_notification_until_its_owning_release() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let (mut left, left_queue) = owner(executor.clone());
    let (mut right, right_queue) = owner(executor.clone());
    assert!(!left.runtime().same_heap(right.runtime()));
    let first = accept(&mut left, &left_queue);
    let second = accept(&mut right, &right_queue);
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let wakes = Arc::new(AtomicUsize::new(0));
    let waker = Waker::from(Arc::new(PeerWake {
        peers: vec![first.observer.clone(), second.observer.clone()],
        snapshots: snapshots.clone(),
        wakes: wakes.clone(),
    }));
    let mut first_wait = first.observer.wait().unwrap();
    let mut second_wait = second.observer.wait().unwrap();
    pending(&mut first_wait, &waker);
    pending(&mut second_wait, &waker);

    let mut bank_release = StoreRelease::new(&left_queue);
    bank_release.completed = true;
    let bank = left_queue.store.borrow_mut();
    let registry_release = RegistryRelease {
        registry: executor.registry.clone(),
        completed: true,
        unwinding_at_entry: std::thread::panicking(),
    };
    let protected_registry = executor.registry.queues.borrow_mut();
    let expected = JobFault::new(
        JobFaultOwner::ExecutorRegistry,
        JobFaultCause::InaccessibleRegistry,
    );
    let Err(QueueError::Fault(fault)) = executor.queues_for(&left) else {
        panic!("real protected registry access must establish its first fault")
    };
    assert_eq!(fault, expected);
    drop(protected_registry);
    drop(registry_release);
    assert_eq!(executor.registry.first_fault.get(), Some(expected));
    assert_eq!(first.observer.status(), Ok(JobStatus::Pending));
    assert_eq!(wakes.load(Ordering::Relaxed), 0);
    assert!(snapshots.lock().unwrap().is_empty());
    for accepted in [&first, &second] {
        assert_eq!(accepted.calls.get(), 0);
        assert_eq!(accepted.drops.get(), 0);
    }

    drop(bank);
    // Releasing the raw loan does not itself invent notification authority.
    assert_eq!(wakes.load(Ordering::Relaxed), 0);
    drop(bank_release);
    assert!(wakes.load(Ordering::Relaxed) >= 2);
    let observations = snapshots.lock().unwrap();
    assert!(!observations.is_empty());
    assert!(
        observations
            .iter()
            .all(|peers| peers == &vec![Err(expected), Err(expected)])
    );
    drop(observations);
    quarantined(&first, expected);
    quarantined(&second, expected);
    for wait in [&mut first_wait, &mut second_wait] {
        assert_eq!(
            Pin::new(wait).poll(&mut TaskContext::from_waker(Waker::noop())),
            Poll::Ready(Err(expected))
        );
    }
    left_queue.clear().unwrap();
    right_queue.clear().unwrap();
    for accepted in [&first, &second] {
        assert_eq!(accepted.calls.get(), 0);
        assert_eq!(accepted.drops.get(), 1);
        assert!(accepted.outside_store.get());
        assert_eq!(accepted.observer.status(), Err(expected));
    }
}

#[test]
fn the_actual_store_release_pin_completes_deferred_registry_fault_and_final_executor_shutdown() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let executor_gone = Rc::downgrade(&executor);
    let registry_gone = Rc::downgrade(&executor.registry);
    let (mut left, left_queue) = owner(executor.clone());
    let (mut right, right_queue) = owner(executor.clone());
    assert!(!left.runtime().same_heap(right.runtime()));
    let first = accept(&mut left, &left_queue);
    let second = accept(&mut right, &right_queue);
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let wakes = Arc::new(AtomicUsize::new(0));
    let waker = Waker::from(Arc::new(PeerWake {
        peers: vec![first.observer.clone(), second.observer.clone()],
        snapshots: snapshots.clone(),
        wakes: wakes.clone(),
    }));
    let mut first_wait = first.observer.wait().unwrap();
    let mut second_wait = second.observer.wait().unwrap();
    pending(&mut first_wait, &waker);
    pending(&mut second_wait, &waker);

    let mut bank_release = StoreRelease::new(&left_queue);
    bank_release.completed = true;
    let bank = left_queue.store.borrow_mut();
    let registry_release = RegistryRelease {
        registry: executor.registry.clone(),
        completed: true,
        unwinding_at_entry: std::thread::panicking(),
    };
    let protected_registry = executor.registry.queues.borrow_mut();
    let expected = JobFault::new(
        JobFaultOwner::ExecutorRegistry,
        JobFaultCause::InaccessibleRegistry,
    );
    let Err(QueueError::Fault(fault)) = executor.queues_for(&left) else {
        panic!("real protected registry access must establish its first fault")
    };
    assert_eq!(fault, expected);
    drop(protected_registry);
    drop(registry_release);
    assert_eq!(first.observer.status(), Ok(JobStatus::Pending));
    assert_eq!(wakes.load(Ordering::Relaxed), 0);
    assert!(snapshots.lock().unwrap().is_empty());

    // Generic commands have no prepared native Context owner. Their physical
    // queue custody survives Context destruction. These are all actual strong
    // executor owners; queues, waiters, observers and captures own no registry.
    drop(executor);
    drop(left);
    drop(right);
    assert!(executor_gone.upgrade().is_none());
    // Only the real StoreRelease field now pins the deferred registry. Do not
    // upgrade registry_gone or introduce a separate controller before release.
    assert_eq!(first.observer.status(), Ok(JobStatus::Pending));
    assert_eq!(wakes.load(Ordering::Relaxed), 0);
    assert!(snapshots.lock().unwrap().is_empty());
    for accepted in [&first, &second] {
        assert_eq!(accepted.calls.get(), 0);
        assert_eq!(accepted.drops.get(), 0);
    }
    drop(bank);
    assert_eq!(wakes.load(Ordering::Relaxed), 0);
    drop(bank_release);

    assert!(wakes.load(Ordering::Relaxed) >= 2);
    let observations = snapshots.lock().unwrap();
    assert!(!observations.is_empty());
    assert!(
        observations
            .iter()
            .all(|peers| peers == &vec![Err(expected), Err(expected)])
    );
    drop(observations);
    for accepted in [&first, &second] {
        assert_eq!(accepted.observer.status(), Err(expected));
        assert_eq!(accepted.observer.fault(), Some(expected));
        assert_eq!(accepted.calls.get(), 0);
        assert_eq!(accepted.drops.get(), 1);
        assert!(accepted.outside_store.get());
    }
    for wait in [&mut first_wait, &mut second_wait] {
        assert_eq!(
            Pin::new(wait).poll(&mut TaskContext::from_waker(Waker::noop())),
            Poll::Ready(Err(expected))
        );
    }
    // No executor/Context action or explicit clear drives this cleanup. Actual
    // final SimpleJobExecutor destruction requested it, and the guard completed it.
    assert!(left_queue.store.borrow().entries.is_empty());
    assert!(right_queue.store.borrow().entries.is_empty());
    assert!(registry_gone.upgrade().is_none());
}
