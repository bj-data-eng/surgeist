use super::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Condvar, TryLockError, atomic::AtomicUsize},
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
                "native wake deadline: expected {expected}, observed {}",
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
fn isolated() -> Arc<Domain> {
    Arc::new(Domain::new())
}
fn signal(domain: &Arc<Domain>) -> Arc<Signal> {
    Signal::in_isolated_domain(domain.clone()).unwrap()
}
fn park<W: Wake + Send + Sync + 'static>(signal: &Arc<Signal>, wake: Arc<W>) -> SettlementWait {
    let mut wait = signal.wait().unwrap();
    assert_eq!(
        wait.poll(&mut TaskContext::from_waker(&Waker::from(wake))),
        Poll::Pending
    );
    wait
}
fn poll(wait: &mut SettlementWait) -> Poll<Result<SettlementStatus, JobFault>> {
    wait.poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn registration_error<T>(result: Result<T, JobRegistrationError>) -> JobRegistrationError {
    match result {
        Err(error) => error,
        Ok(_) => panic!("registration unexpectedly accepted"),
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
fn connect(source: &Arc<Signal>, target: &Arc<Signal>) -> DependencyGuard {
    let epoch = Epoch::new();
    epoch.active.store(true, Ordering::Release);
    source.subscribe(target, epoch).unwrap()
}

#[test]
fn malformed_signal_state_notifies_all_waiters_and_retains_the_original_typed_cause() {
    for (raw, cause) in [
        (255, JobFaultCause::InvalidDisposition { raw: 255 }),
        (FAULTED, JobFaultCause::MissingCause),
    ] {
        let domain = isolated();
        let signal = signal(&domain);
        let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
        let mut waits: Vec<_> = counts
            .iter()
            .map(|count| park(&signal, count.clone()))
            .collect();
        let expected = signal_fault(cause);
        signal.status.store(raw, Ordering::Release);
        assert_eq!(signal.status(), Err(expected));
        for (count, wait) in counts.iter().zip(&mut waits) {
            count.await_count(1);
            assert_eq!(poll(wait), Poll::Ready(Err(expected)));
        }
        signal.status.store(254, Ordering::Release);
        assert_eq!(signal.status(), Err(expected));
        assert_eq!(
            registration_error(signal.wait()),
            JobRegistrationError::Fault(expected)
        );
        assert!(domain.fault.get().is_none());
    }
}

#[test]
fn poisoned_signal_registry_and_waker_slots_are_typed_faults_with_complete_original_notifications()
{
    for waker_slot in [false, true] {
        let domain = isolated();
        let signal = signal(&domain);
        let counts: Vec<_> = (0..3).map(|_| Arc::new(WakeCount::default())).collect();
        let mut waits: Vec<_> = counts
            .iter()
            .map(|count| park(&signal, count.clone()))
            .collect();
        let cause = if waker_slot {
            poison(&waits[0].waiter.as_ref().unwrap().waker);
            JobFaultCause::PoisonedWaker
        } else {
            poison(&signal.state);
            JobFaultCause::PoisonedRegistry
        };
        let expected = signal_fault(cause);
        assert_eq!(poll(&mut waits[0]), Poll::Ready(Err(expected)));
        for (count, wait) in counts.iter().zip(&mut waits) {
            count.await_count(1);
            assert_eq!(poll(wait), Poll::Ready(Err(expected)));
        }
        assert_eq!(
            registration_error(signal.wait()),
            JobRegistrationError::Fault(expected)
        );
        assert!(domain.fault.get().is_none());
    }
}

#[test]
fn invalid_and_stale_waiter_keys_quarantine_without_unregistering_the_replacement() {
    for stale_identity in [false, true] {
        let domain = isolated();
        let signal = signal(&domain);
        let mut invalid = signal.wait().unwrap();
        let index = invalid.index;
        let cause = if stale_identity {
            // A stale key keeps the old allocation alive while the real slot is
            // released and reused by a different waiter.
            let stale = SettlementWait {
                signal: signal.clone(),
                index,
                waiter: invalid.waiter.clone(),
            };
            drop(invalid);
            invalid = stale;
            JobFaultCause::InvalidSlotIdentity { index }
        } else {
            invalid.index = usize::MAX;
            JobFaultCause::InvalidIndex { index: usize::MAX }
        };
        let count = Arc::new(WakeCount::default());
        let mut replacement = park(&signal, count.clone());
        let expected = signal_fault(cause);
        assert_eq!(poll(&mut invalid), Poll::Ready(Err(expected)));
        count.await_count(1);
        assert_eq!(poll(&mut replacement), Poll::Ready(Err(expected)));
        assert_eq!(
            registration_error(signal.wait()),
            JobRegistrationError::Fault(expected)
        );
    }
}

#[test]
fn dropped_invalid_waiter_key_preserves_its_fault_for_later_fallible_entries() {
    let domain = isolated();
    let signal = signal(&domain);
    let mut invalid = signal.wait().unwrap();
    invalid.index = usize::MAX;
    let count = Arc::new(WakeCount::default());
    let mut original = park(&signal, count.clone());
    drop(invalid);
    let expected = signal_fault(JobFaultCause::InvalidIndex { index: usize::MAX });
    count.await_count(1);
    assert_eq!(poll(&mut original), Poll::Ready(Err(expected)));
    assert_eq!(
        registration_error(signal.wait()),
        JobRegistrationError::Fault(expected)
    );
}

struct DomainWake {
    signals: Vec<Arc<Signal>>,
    domain: Arc<Domain>,
    expected: JobFault,
    all_marked: AtomicBool,
    unlocked: AtomicBool,
    completion: Arc<WakeCount>,
}
impl Wake for DomainWake {
    fn wake(self: Arc<Self>) {
        self.all_marked.fetch_and(
            self.signals
                .iter()
                .all(|signal| signal.status() == Err(self.expected)),
            Ordering::SeqCst,
        );
        self.unlocked.fetch_and(
            unlocked(&self.domain.members)
                && unlocked(&self.domain.queue)
                && self.signals.iter().all(|signal| {
                    unlocked(&signal.state)
                        && unlocked(&signal.next)
                        && unlocked(&signal.module_next)
                }),
            Ordering::SeqCst,
        );
        self.completion.record();
    }
}

#[test]
fn poisoned_domain_scans_the_physical_bank_marks_all_members_before_wakes_and_severs_cycles() {
    let domain = isolated();
    let signals: Vec<_> = (0..3).map(|_| signal(&domain)).collect();
    let hole = signal(&domain);
    drop(hole);
    let expected = domain_fault(JobFaultCause::PoisonedRegistry);
    let completion = Arc::new(WakeCount::default());
    let probe = Arc::new(DomainWake {
        signals: signals.clone(),
        domain: domain.clone(),
        expected,
        all_marked: AtomicBool::new(true),
        unlocked: AtomicBool::new(true),
        completion: completion.clone(),
    });
    let mut waits: Vec<_> = signals
        .iter()
        .map(|signal| park(signal, probe.clone()))
        .collect();
    for pair in signals.windows(2) {
        *pair[0].next.lock().unwrap() = Some(pair[1].clone());
    }
    *signals.last().unwrap().next.lock().unwrap() = Some(signals[0].clone());
    domain.members.lock().unwrap().free = Some(usize::MAX);
    poison(&domain.members);
    assert_eq!(
        registration_error(Signal::in_isolated_domain(domain.clone())),
        JobRegistrationError::Fault(expected)
    );
    completion.await_count(3);
    assert!(probe.all_marked.load(Ordering::SeqCst));
    assert!(probe.unlocked.load(Ordering::SeqCst));
    for wait in &mut waits {
        assert_eq!(poll(wait), Poll::Ready(Err(expected)));
    }
    let weak: Vec<_> = signals.iter().map(Arc::downgrade).collect();
    drop(waits);
    drop(probe);
    drop(signals);
    assert!(
        weak.iter().all(|weak| weak.upgrade().is_none()),
        "notification links must not retain quarantined members"
    );
}

#[test]
fn damaged_domain_free_links_and_counters_retain_typed_faults_and_notify_all_members() {
    for mode in 0..5 {
        let domain = isolated();
        let signal = signal(&domain);
        let hole = Signal::in_isolated_domain(domain.clone()).unwrap();
        let hole_index = hole.memberships.get().unwrap().ordinary;
        drop(hole);
        let count = Arc::new(WakeCount::default());
        let mut wait = park(&signal, count.clone());
        let cause = {
            let mut members = domain.members.lock().unwrap();
            match mode {
                0 => {
                    members.free = Some(usize::MAX);
                    JobFaultCause::InvalidIndex { index: usize::MAX }
                }
                1 => {
                    members.free = Some(signal.memberships.get().unwrap().ordinary);
                    JobFaultCause::InvalidFreeLink {
                        index: signal.memberships.get().unwrap().ordinary,
                    }
                }
                2 => {
                    members.entries[hole_index] = MemberEntry::Vacant(Some(hole_index));
                    JobFaultCause::InvalidFreeLink { index: hole_index }
                }
                3 => {
                    members.live = members.entries.len() + 1;
                    JobFaultCause::StateInconsistency
                }
                _ => {
                    members.live = 0;
                    members.free = None;
                    JobFaultCause::CounterUnderflow
                }
            }
        };
        let expected = domain_fault(cause);
        if mode == 4 {
            domain.unregister(signal.memberships.get().unwrap().ordinary, &signal);
        } else {
            assert_eq!(
                registration_error(Signal::in_isolated_domain(domain.clone())),
                JobRegistrationError::Fault(expected)
            );
        }
        count.await_count(1);
        assert_eq!(poll(&mut wait), Poll::Ready(Err(expected)));
        assert_eq!(
            registration_error(Signal::in_isolated_domain(domain.clone())),
            JobRegistrationError::Fault(expected)
        );
    }
}

#[test]
fn domain_unregister_requires_both_index_and_current_signal_witness() {
    for wrong_index in [false, true] {
        let domain = isolated();
        let a = signal(&domain);
        let b = signal(&domain);
        let count = Arc::new(WakeCount::default());
        let mut wait = park(&b, count.clone());
        let index = if wrong_index {
            usize::MAX
        } else {
            b.memberships.get().unwrap().ordinary
        };
        let expected = domain_fault(if wrong_index {
            JobFaultCause::InvalidIndex { index }
        } else {
            JobFaultCause::InvalidSlotIdentity { index }
        });
        domain.unregister(index, &a);
        count.await_count(1);
        assert_eq!(poll(&mut wait), Poll::Ready(Err(expected)));
        assert_eq!(a.status(), Err(expected));
    }
}

#[test]
fn second_membership_resource_refusal_rolls_back_without_poisoning_either_domain() {
    let ordinary = isolated();
    let original = signal(&ordinary);
    let module = ordinary.isolated_module.get().unwrap().clone();
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&original, count.clone());
    // This per-domain failpoint refuses the actual append reservation without
    // changing storage. It simulates allocation refusal rather than physical OOM.
    module.fail_reservation.store(true, Ordering::Release);
    assert_eq!(
        registration_error(Signal::in_isolated_domain(ordinary.clone())),
        JobRegistrationError::Resource(crate::job::JobResourceError::Allocation)
    );
    assert!(ordinary.fault.get().is_none());
    assert!(module.fault.get().is_none());
    assert_eq!(original.status(), Ok(SettlementStatus::Pending));
    assert_eq!(
        wait.poll(&mut TaskContext::from_waker(&Waker::from(count.clone()))),
        Poll::Pending
    );
    for domain in [&ordinary, &module] {
        let members = domain.members.lock().unwrap();
        assert_eq!(members.live, 1);
        assert_eq!(
            members
                .entries
                .iter()
                .filter_map(MemberEntry::signal)
                .filter(|weak| weak.upgrade().is_some())
                .count(),
            1
        );
    }
    let next = signal(&ordinary);
    let next_count = Arc::new(WakeCount::default());
    let mut next_wait = park(&next, next_count.clone());
    original.settle();
    next.settle();
    count.await_count(1);
    next_count.await_count(1);
    assert_eq!(poll(&mut wait), Poll::Ready(Ok(SettlementStatus::Settled)));
    assert_eq!(
        poll(&mut next_wait),
        Poll::Ready(Ok(SettlementStatus::Settled))
    );
    let weak = Arc::downgrade(&next);
    drop(next_wait);
    drop(next);
    assert!(weak.upgrade().is_none());
}

#[test]
fn second_membership_registry_fault_rolls_back_first_membership_and_retains_fault_kind() {
    let ordinary = isolated();
    let original = signal(&ordinary);
    let module = ordinary.isolated_module.get().unwrap().clone();
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&original, count.clone());
    poison(&module.members);
    let expected = domain_fault(JobFaultCause::PoisonedRegistry);
    assert_eq!(
        registration_error(Signal::in_isolated_domain(ordinary.clone())),
        JobRegistrationError::Fault(expected)
    );
    count.await_count(1);
    assert_eq!(poll(&mut wait), Poll::Ready(Err(expected)));
    assert!(ordinary.fault.get().is_none());
    assert_eq!(ordinary.members.lock().unwrap().live, 1);
    let fresh_module = isolated();
    let accepted = Signal::construct(
        DomainRef::Isolated(ordinary.clone()),
        DomainRef::Isolated(fresh_module),
    )
    .unwrap();
    accepted.settle();
    assert_eq!(accepted.status(), Ok(SettlementStatus::Settled));
    assert_eq!(
        registration_error(Signal::in_isolated_domain(ordinary)),
        JobRegistrationError::Fault(expected)
    );
}

#[test]
fn affiliation_is_checked_for_each_route_and_isolated_quarantine_skips_foreign_members() {
    let domain = isolated();
    let own = signal(&domain);
    let foreign_domain = isolated();
    let foreign = signal(&foreign_domain);
    let foreign_slot = domain.register(&foreign).unwrap();
    let expected = domain_fault(JobFaultCause::OwnerMismatch);
    // The actual route guard rejects a signal sent to a different pair. A
    // poisoned bank's forged foreign entry grants no ownership over that signal.
    foreign.notice_in(&domain, false);
    assert_eq!(domain.fault.get(), Some(&expected));
    assert_eq!(own.status(), Err(expected));
    assert_eq!(foreign.status(), Ok(SettlementStatus::Pending));
    assert!(foreign_domain.fault.get().is_none());
    foreign.settle();
    assert_eq!(foreign.status(), Ok(SettlementStatus::Settled));
    let _ = foreign_slot;
    let shared = isolated();
    assert_eq!(
        registration_error(Signal::construct(
            DomainRef::Isolated(shared.clone()),
            DomainRef::Isolated(shared)
        )),
        JobRegistrationError::Fault(expected)
    );
}

#[test]
fn module_route_mismatch_and_poisoned_work_links_are_typed_domain_faults() {
    for mode in 0..3 {
        let ordinary = isolated();
        let signal = signal(&ordinary);
        let module = ordinary.isolated_module.get().unwrap().clone();
        let count = Arc::new(WakeCount::default());
        let mut wait = park(&signal, count.clone());
        let expected = if mode == 0 {
            signal.notice_in(&ordinary, true);
            domain_fault(JobFaultCause::OwnerMismatch)
        } else if mode == 1 {
            poison(&signal.next);
            signal.notice_in(&ordinary, false);
            domain_fault(JobFaultCause::PoisonedLink)
        } else {
            signal.ordinary_work.store(3, Ordering::Release);
            signal.notice_in(&ordinary, false);
            domain_fault(JobFaultCause::StateInconsistency)
        };
        count.await_count(1);
        assert_eq!(poll(&mut wait), Poll::Ready(Err(expected)));
        assert_eq!(ordinary.fault.get(), Some(&expected));
        assert!(module.fault.get().is_none());
    }
}

struct ChainWake {
    signals: Vec<Arc<Signal>>,
    expected: JobFault,
    all_marked: AtomicBool,
    completion: Arc<WakeCount>,
}
impl Wake for ChainWake {
    fn wake(self: Arc<Self>) {
        self.all_marked.fetch_and(
            self.signals
                .iter()
                .all(|signal| signal.status() == Err(self.expected)),
            Ordering::SeqCst,
        );
        self.completion.record();
    }
}
#[test]
fn a_signal_fault_reaches_the_full_active_dependency_chain_before_any_native_wake() {
    let domain = isolated();
    let signals: Vec<_> = (0..128).map(|_| signal(&domain)).collect();
    let _dependencies: Vec<_> = signals
        .windows(2)
        .map(|pair| connect(&pair[0], &pair[1]))
        .collect();
    let expected = signal_fault(JobFaultCause::InvalidDisposition { raw: 255 });
    let completion = Arc::new(WakeCount::default());
    let probe = Arc::new(ChainWake {
        signals: signals.clone(),
        expected,
        all_marked: AtomicBool::new(true),
        completion: completion.clone(),
    });
    let mut waits: Vec<_> = signals
        .iter()
        .map(|signal| park(signal, probe.clone()))
        .collect();
    signals[0].status.store(255, Ordering::Release);
    assert_eq!(signals[0].status(), Err(expected));
    completion.await_count(signals.len());
    assert!(probe.all_marked.load(Ordering::SeqCst));
    for wait in &mut waits {
        assert_eq!(poll(wait), Poll::Ready(Err(expected)));
    }
    assert!(domain.fault.get().is_none());
}

#[test]
fn stale_epoch_fault_and_module_notices_do_not_poison_an_active_successor() {
    let domain = isolated();
    let signal = signal(&domain);
    let stale = Epoch::new();
    let successor = Epoch::new();
    successor.active.store(true, Ordering::Release);
    let old = Dependency {
        target: Arc::downgrade(&signal),
        epoch: stale.clone(),
    };
    let current = Dependency {
        target: Arc::downgrade(&signal),
        epoch: successor.clone(),
    };
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&signal, count.clone());
    let fault = signal_fault(JobFaultCause::PoisonedRegistry);
    old.fault(fault);
    old.module_interrupt();
    assert_eq!(stale.fault.get(), Some(&fault));
    assert!(stale.module_pending.load(Ordering::Acquire));
    assert_eq!(signal.status(), Ok(SettlementStatus::Pending));
    assert_eq!(
        wait.poll(&mut TaskContext::from_waker(&Waker::from(count.clone()))),
        Poll::Pending
    );
    current.retire(JobRetirement::Cancelled);
    count.await_count(1);
    assert_eq!(
        poll(&mut wait),
        Poll::Ready(Ok(SettlementStatus::Retired(JobRetirement::Cancelled)))
    );
    assert!(signal.fault.get().is_none());
}

#[test]
fn signal_settlement_and_explicit_module_markers_keep_their_selected_priority_over_faults() {
    let domain = isolated();
    for generic_retirement in [false, true] {
        for module in [false, true] {
            let signal = signal(&domain);
            let fault = if generic_retirement {
                signal.retire_pending(JobRetirement::Dropped);
                poison(&signal.state);
                let expected = signal_fault(JobFaultCause::PoisonedRegistry);
                assert_eq!(signal.state_access().err(), Some(expected));
                assert_eq!(
                    signal.status(),
                    Ok(SettlementStatus::Retired(JobRetirement::Dropped))
                );
                expected
            } else {
                signal.status.store(255, Ordering::Release);
                let expected = signal_fault(JobFaultCause::InvalidDisposition { raw: 255 });
                assert_eq!(signal.status(), Err(expected));
                expected
            };
            if module {
                let epoch = Epoch::new();
                epoch.active.store(true, Ordering::Release);
                signal.mark_module_epoch(&epoch);
                signal.settle();
                assert_eq!(signal.status(), Ok(SettlementStatus::ModuleInterrupted));
            } else {
                signal.settle();
                assert_eq!(signal.status(), Ok(SettlementStatus::Settled));
                signal.mark_module();
                assert_eq!(signal.status(), Ok(SettlementStatus::Settled));
            }
            assert_eq!(signal.fault.get(), Some(&fault));
            assert_eq!(signal.check(), Err(fault));
            let mut late = signal.wait().unwrap();
            assert_eq!(poll(&mut late), Poll::Ready(signal.status()));
        }
    }
}

#[derive(Debug)]
struct WakePanic(usize);
struct PanickingDomainWake {
    probe: DomainWake,
    tag: usize,
    first: Arc<OnceLock<usize>>,
    mask: Arc<AtomicUsize>,
    reentry_valid: Arc<AtomicBool>,
}
impl Wake for PanickingDomainWake {
    fn wake(self: Arc<Self>) {
        self.probe.all_marked.fetch_and(
            self.probe
                .signals
                .iter()
                .all(|signal| signal.status() == Err(self.probe.expected)),
            Ordering::SeqCst,
        );
        self.reentry_valid.fetch_and(
            unlocked(&self.probe.domain.members)
                && unlocked(&self.probe.domain.queue)
                && self
                    .probe
                    .signals
                    .iter()
                    .all(|signal| unlocked(&signal.state)),
            Ordering::SeqCst,
        );
        let rejected = registration_error(Signal::in_isolated_domain(self.probe.domain.clone()));
        self.reentry_valid.fetch_and(
            rejected == JobRegistrationError::Fault(self.probe.expected),
            Ordering::SeqCst,
        );
        self.mask.fetch_or(1 << self.tag, Ordering::SeqCst);
        self.probe.completion.record();
        self.first.get_or_init(|| self.tag);
        std::panic::panic_any(WakePanic(self.tag));
    }
}
#[test]
fn domain_quarantine_finishes_all_panicking_wakes_then_resumes_the_first_unwind() {
    let domain = isolated();
    let signals: Vec<_> = (0..3).map(|_| signal(&domain)).collect();
    let expected = domain_fault(JobFaultCause::PoisonedRegistry);
    let first = Arc::new(OnceLock::new());
    let mask = Arc::new(AtomicUsize::new(0));
    let valid = Arc::new(AtomicBool::new(true));
    let completion = Arc::new(WakeCount::default());
    let mut waits = Vec::new();
    let mut probes = Vec::new();
    for tag in [0, 1] {
        let probe = Arc::new(PanickingDomainWake {
            probe: DomainWake {
                signals: signals.clone(),
                domain: domain.clone(),
                expected,
                all_marked: AtomicBool::new(true),
                unlocked: AtomicBool::new(true),
                completion: completion.clone(),
            },
            tag,
            first: first.clone(),
            mask: mask.clone(),
            reentry_valid: valid.clone(),
        });
        waits.push(park(&signals[tag], probe.clone()));
        probes.push(probe);
    }
    let count = Arc::new(WakeCount::default());
    waits.push(park(&signals[2], count.clone()));
    poison(&domain.members);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = Signal::in_isolated_domain(domain.clone());
    }))
    .unwrap_err();
    assert_eq!(
        panic.downcast_ref::<WakePanic>().unwrap().0,
        *first.get().unwrap()
    );
    completion.await_count(2);
    count.await_count(1);
    assert_eq!(mask.load(Ordering::SeqCst), 3);
    assert!(valid.load(Ordering::SeqCst));
    assert!(
        probes
            .iter()
            .all(|probe| probe.probe.all_marked.load(Ordering::SeqCst))
    );
    for wait in &mut waits {
        assert_eq!(poll(wait), Poll::Ready(Err(expected)));
    }
    assert_eq!(
        registration_error(Signal::in_isolated_domain(domain)),
        JobRegistrationError::Fault(expected)
    );
    let healthy = signal(&isolated());
    healthy.settle();
    assert_eq!(healthy.status(), Ok(SettlementStatus::Settled));
}

#[test]
fn an_actual_module_origin_keeps_identity_and_cause_after_a_signal_registry_fault() {
    use crate::{
        JsValue, Module, Source,
        error::{DiagnosticEngine, DiagnosticKind, EngineError, RuntimeLimitError},
        execution::ExecutionOptions,
    };
    const LIMIT: u64 = 64;
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let module = Module::parse(
        Source::from_bytes("for (let step = 0; step < 1000; step++) {}"),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = module.load(context.access()).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        crate::builtins::promise::PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    let stopped = context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: LIMIT
        }))
    );
    let retained = module.evaluate(context.access()).unwrap_err();
    let Some(EngineError::ModuleInterrupted(retained)) = retained.as_engine() else {
        panic!("the module must retain its interruption");
    };
    assert_eq!(
        retained.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(
            RuntimeLimitError::Instruction { limit: LIMIT }
        ))
    );
    let owner = Association::new(&context).unwrap();
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&owner.signal, count.clone());
    owner.observe_module(module.kind().as_source_text().unwrap().evaluation.clone());
    count.await_count(1);
    assert_eq!(
        poll(&mut wait),
        Poll::Ready(Ok(SettlementStatus::ModuleInterrupted))
    );
    poison(&owner.signal.state);
    let expected = signal_fault(JobFaultCause::PoisonedRegistry);
    assert_eq!(owner.signal.state_access().err(), Some(expected));
    assert_eq!(owner.signal.fault.get(), Some(&expected));
    assert_eq!(
        owner.signal.status(),
        Ok(SettlementStatus::ModuleInterrupted)
    );
    let discovered = Association::module_error(&owner)
        .expect("the retained source origin remains authoritative");
    let Some(EngineError::ModuleInterrupted(discovered)) = discovered.as_engine() else {
        panic!("the native signal fault must not replace the module cause");
    };
    assert_eq!(discovered.identity(), retained.identity());
    assert_eq!(discovered.cause().kind(), retained.cause().kind());
    assert_eq!(
        poll(&mut wait),
        Poll::Ready(Ok(SettlementStatus::ModuleInterrupted))
    );
}

#[test]
fn native_fault_cleanup_preserves_an_existing_operation_stop_and_performs_no_membership_admission()
{
    use crate::{
        Source,
        error::{EngineError, RuntimeLimitError},
        execution::ExecutionOptions,
    };
    let domain = isolated();
    let signal = signal(&domain);
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&signal, count.clone());
    domain.fail_reservation.store(true, Ordering::Release);
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let expected = signal_fault(JobFaultCause::InvalidDisposition { raw: 255 });
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let first = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                first.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            signal.status.store(255, Ordering::Release);
            assert_eq!(signal.status(), Err(expected));
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
    assert_eq!(poll(&mut wait), Poll::Ready(Err(expected)));
    assert_eq!(
        registration_error(Signal::in_isolated_domain(domain.clone())),
        JobRegistrationError::Resource(crate::job::JobResourceError::Allocation)
    );
    assert!(domain.fault.get().is_none());
    let next = Signal::in_isolated_domain(domain).unwrap();
    next.settle();
    assert_eq!(next.status(), Ok(SettlementStatus::Settled));
}

struct SignalDropReentry {
    signal: Weak<Signal>,
    waiter: Weak<Waiter>,
    drops: Arc<AtomicUsize>,
    valid: Arc<AtomicBool>,
}
impl Wake for SignalDropReentry {
    fn wake(self: Arc<Self>) {
        drop(self);
    }
}
impl Drop for SignalDropReentry {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
        if let Some(signal) = self.signal.upgrade() {
            let free = unlocked(&signal.state)
                && self
                    .waiter
                    .upgrade()
                    .is_none_or(|waiter| unlocked(&waiter.waker));
            self.valid.fetch_and(free, Ordering::SeqCst);
            if free {
                self.valid
                    .fetch_and(signal.wait().is_ok(), Ordering::SeqCst);
            }
        }
    }
}
#[test]
fn native_waiter_replacement_and_final_drop_allow_safe_reentry_outside_owner_locks() {
    for replace in [false, true] {
        let domain = isolated();
        let signal = signal(&domain);
        let mut wait = signal.wait().unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        let valid = Arc::new(AtomicBool::new(true));
        let old = Waker::from(Arc::new(SignalDropReentry {
            signal: Arc::downgrade(&signal),
            waiter: Arc::downgrade(wait.waiter.as_ref().unwrap()),
            drops: drops.clone(),
            valid: valid.clone(),
        }));
        assert_eq!(wait.poll(&mut TaskContext::from_waker(&old)), Poll::Pending);
        drop(old);
        if replace {
            assert_eq!(poll(&mut wait), Poll::Pending);
        }
        drop(wait);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(valid.load(Ordering::SeqCst));
        assert_eq!(signal.status(), Ok(SettlementStatus::Pending));
        assert!(domain.fault.get().is_none());
        signal.settle();
    }
}

#[test]
fn module_domain_poison_notifies_its_members_without_quarantining_the_ordinary_or_other_pair() {
    let ordinary = isolated();
    let own = signal(&ordinary);
    let module = ordinary.isolated_module.get().unwrap().clone();
    let other_pair = isolated();
    let other = signal(&other_pair);
    let count = Arc::new(WakeCount::default());
    let mut wait = park(&own, count.clone());
    poison(&module.queue);
    let expected = domain_fault(JobFaultCause::PoisonedRegistry);
    assert_eq!(module.access().err(), Some(expected));
    count.await_count(1);
    assert_eq!(poll(&mut wait), Poll::Ready(Err(expected)));
    assert!(ordinary.fault.get().is_none());
    assert_eq!(module.fault.get(), Some(&expected));
    assert!(other_pair.fault.get().is_none());
    assert_eq!(other.status(), Ok(SettlementStatus::Pending));
    other.settle();
    assert_eq!(other.status(), Ok(SettlementStatus::Settled));
}

#[test]
fn direct_signal_registry_fault_marks_both_active_siblings_before_any_native_wake() {
    let domain = isolated();
    let signals: Vec<_> = (0..3).map(|_| signal(&domain)).collect();
    let _first = connect(&signals[0], &signals[1]);
    let _second = connect(&signals[0], &signals[2]);
    let expected = signal_fault(JobFaultCause::PoisonedRegistry);
    let completion = Arc::new(WakeCount::default());
    let probe = Arc::new(ChainWake {
        signals: signals.clone(),
        expected,
        all_marked: AtomicBool::new(true),
        completion: completion.clone(),
    });
    let mut waits: Vec<_> = signals
        .iter()
        .map(|signal| park(signal, probe.clone()))
        .collect();

    // All original waits are subscribed and parked before the actual owner
    // mutex is poisoned. This checked entry detects the real poisoning; the
    // first external wake inspects all three actual Signal statuses.
    poison(&signals[0].state);
    assert_eq!(signals[0].state_access().err(), Some(expected));
    completion.await_count(signals.len());
    assert!(
        probe.all_marked.load(Ordering::SeqCst),
        "each active sibling must be faulted before the first native wake"
    );
    for wait in &mut waits {
        assert_eq!(poll(wait), Poll::Ready(Err(expected)));
    }
    assert!(domain.fault.get().is_none());
}
