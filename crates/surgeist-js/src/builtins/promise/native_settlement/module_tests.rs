use super::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Condvar, atomic::AtomicUsize},
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
        *lock(&self.count) += 1;
        self.changed.notify_all();
    }

    fn await_count(&self, expected: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut count = lock(&self.count);
        while *count < expected {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "native notification deadline: expected {expected} wakes, observed {}",
                *count
            );
            let (next, _) = self
                .changed
                .wait_timeout(count, remaining)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            count = next;
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

fn arm<W: Wake + Send + Sync + 'static>(signal: &Arc<Signal>, wake: Arc<W>) -> SettlementWait {
    let mut waiter = signal.wait().unwrap();
    let waker = Waker::from(wake);
    assert_eq!(
        waiter.poll(&mut TaskContext::from_waker(&waker)),
        Poll::Pending
    );
    waiter
}

fn observed(waiter: &mut SettlementWait) -> Poll<SettlementStatus> {
    waiter
        .poll(&mut TaskContext::from_waker(Waker::noop()))
        .map(|status| status.expect("the healthy signal must retain its ordinary result"))
}

fn connect(source: &Arc<Signal>, target: &Arc<Signal>) -> DependencyGuard {
    let epoch = Epoch::new();
    epoch.active.store(true, Ordering::Release);
    source.subscribe(target, epoch).unwrap()
}

fn isolated_domain() -> Arc<Domain> {
    Arc::new(Domain::new())
}

// Module notices may be driven by another thread. The real driver prioritizes
// all queued notices, including their descendants, before this barrier's wake.
fn await_module_notices() {
    let barrier = Signal::new().unwrap();
    let wakes = Arc::new(WakeCount::default());
    let mut waiter = arm(&barrier, wakes.clone());
    barrier.mark_module();
    barrier.notify_module();
    wakes.await_count(1);
    assert_eq!(
        observed(&mut waiter),
        Poll::Ready(SettlementStatus::ModuleInterrupted)
    );
}

struct CheckModuleWake {
    signals: Vec<Arc<Signal>>,
    all_terminal: AtomicBool,
    calls: AtomicUsize,
    completion: Arc<WakeCount>,
}

impl Wake for CheckModuleWake {
    fn wake(self: Arc<Self>) {
        // Record on whichever thread owns the driver; assert on the test thread.
        let all_terminal = self
            .signals
            .iter()
            .all(|signal| signal.status().unwrap() == SettlementStatus::ModuleInterrupted);
        self.all_terminal.fetch_and(all_terminal, Ordering::SeqCst);
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.completion.record();
    }
}

#[test]
fn module_notice_marks_every_active_descendant_before_any_native_wake() {
    let signals: Vec<_> = (0..16).map(|_| Signal::new().unwrap()).collect();
    let _subscriptions: Vec<_> = signals
        .windows(2)
        .map(|pair| connect(&pair[0], &pair[1]))
        .collect();
    let completion = Arc::new(WakeCount::default());
    let probes: Vec<_> = signals
        .iter()
        .map(|_| {
            Arc::new(CheckModuleWake {
                signals: signals.clone(),
                all_terminal: AtomicBool::new(true),
                calls: AtomicUsize::new(0),
                completion: completion.clone(),
            })
        })
        .collect();
    let mut waiters: Vec<_> = signals
        .iter()
        .zip(&probes)
        .map(|(signal, probe)| arm(signal, probe.clone()))
        .collect();

    signals[0].mark_module();
    signals[0].notify_module();
    completion.await_count(signals.len());
    for (signal, (probe, waiter)) in signals.iter().zip(probes.iter().zip(&mut waiters)) {
        assert_eq!(
            signal.status().unwrap(),
            SettlementStatus::ModuleInterrupted
        );
        assert!(probe.all_terminal.load(Ordering::SeqCst));
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            observed(waiter),
            Poll::Ready(SettlementStatus::ModuleInterrupted)
        );
    }

    signals[0].mark_module();
    signals[0].notify_module();
    await_module_notices();
    completion.await_count(signals.len());
}

#[test]
fn an_explicit_module_notice_upgrades_prior_generic_retirement_through_the_active_chain() {
    let domain = isolated_domain();
    let signals: Vec<_> = (0..4)
        .map(|_| Signal::in_isolated_domain(domain.clone()).unwrap())
        .collect();
    let _subscriptions: Vec<_> = signals
        .windows(2)
        .map(|pair| connect(&pair[0], &pair[1]))
        .collect();
    signals[0].retire_pending(JobRetirement::Dropped);
    for signal in &signals {
        assert_eq!(
            signal.status().unwrap(),
            SettlementStatus::Retired(JobRetirement::Dropped)
        );
    }

    // Retirement alone did not establish module terminality. This explicit
    // signal-owner event is a separate stimulus with its own propagation path.
    signals[0].mark_module();
    signals[0].notify_module();
    await_module_notices();
    for signal in &signals {
        assert_eq!(
            signal.status().unwrap(),
            SettlementStatus::ModuleInterrupted
        );
    }
}

#[test]
fn module_terminal_status_survives_late_successor_handoff_and_settlement() {
    let context = Context::default();
    let owner = Association::new(&context).unwrap();
    let predecessor = JobObserver::new();
    owner.bind_job(&predecessor, true).unwrap();
    let wakes = Arc::new(WakeCount::default());
    let mut waiter = arm(&owner.signal, wakes.clone());
    owner.signal.mark_module();
    owner.signal.notify_module();
    wakes.await_count(1);

    let successor = JobObserver::new();
    let notification = Association::prepare_handoff(&owner, &successor)
        .unwrap()
        .commit();
    notification.notify();
    successor.retire(JobRetirement::Cancelled);
    predecessor.retire(JobRetirement::OwnerDropped);
    owner.settle();
    await_module_notices();
    assert_eq!(
        owner.signal.status().unwrap(),
        SettlementStatus::ModuleInterrupted
    );
    assert_eq!(
        observed(&mut waiter),
        Poll::Ready(SettlementStatus::ModuleInterrupted)
    );
    wakes.await_count(1);
}

fn selected_delivery(prepared: &PreparedHandoff) -> Arc<Dependency> {
    prepared
        .binding
        ._dependency
        .as_ref()
        .unwrap()
        .dependency
        .clone()
}

#[test]
fn abandoned_follow_preparation_cannot_deliver_a_module_notice_to_the_valid_successor() {
    let context = Context::default();
    let owner = Association::new(&context).unwrap();
    let obsolete = Association::new(&context).unwrap();
    let prepared = Association::prepare_follow(&owner, &obsolete).unwrap();
    let delivery = selected_delivery(&prepared);
    drop(prepared);
    let successor = JobObserver::new();
    drop(
        Association::prepare_handoff(&owner, &successor)
            .unwrap()
            .commit(),
    );

    obsolete.signal.mark_module();
    obsolete.signal.notify_module();
    // A selected delivery can outlive its registration. Its abandoned epoch
    // must still reject this actual source signal's module event.
    delivery.module_interrupt();
    await_module_notices();
    assert_eq!(
        obsolete.signal.status().unwrap(),
        SettlementStatus::ModuleInterrupted
    );
    assert_eq!(owner.signal.status().unwrap(), SettlementStatus::Pending);
    successor.complete();
    owner.settle();
    assert_eq!(owner.signal.status().unwrap(), SettlementStatus::Settled);
}

#[test]
fn replaced_follow_epoch_cannot_deliver_a_module_notice_to_the_committed_successor() {
    let context = Context::default();
    let owner = Association::new(&context).unwrap();
    let obsolete = Association::new(&context).unwrap();
    let prepared = Association::prepare_follow(&owner, &obsolete).unwrap();
    let delivery = selected_delivery(&prepared);
    drop(prepared.commit());
    let successor = JobObserver::new();
    drop(
        Association::prepare_handoff(&owner, &successor)
            .unwrap()
            .commit(),
    );

    obsolete.signal.mark_module();
    obsolete.signal.notify_module();
    delivery.module_interrupt();
    await_module_notices();
    assert_eq!(
        obsolete.signal.status().unwrap(),
        SettlementStatus::ModuleInterrupted
    );
    assert_eq!(owner.signal.status().unwrap(), SettlementStatus::Pending);
    successor.complete();
    owner.settle();
    assert_eq!(owner.signal.status().unwrap(), SettlementStatus::Settled);
}

#[derive(Debug)]
struct AuthoredWakePanic;

struct ReentrantPanickingWake {
    source: Arc<Signal>,
    successor: Arc<Signal>,
    domain: Arc<Domain>,
    effects: AtomicUsize,
}

impl Wake for ReentrantPanickingWake {
    fn wake(self: Arc<Self>) {
        assert!(self.domain.queue.try_lock().is_ok());
        for signal in [&self.source, &self.successor] {
            assert!(signal.state.try_lock().is_ok());
            assert!(signal.next.try_lock().is_ok());
            assert!(signal.module_next.try_lock().is_ok());
        }
        self.effects.fetch_add(1, Ordering::SeqCst);
        self.successor.mark_module();
        self.successor.notice_in(&self.domain, true);
        std::panic::panic_any(AuthoredWakePanic);
    }
}

#[test]
fn panicking_reentrant_module_waker_preserves_remaining_notices_and_driver_reuse() {
    // Isolate driver/unwind ownership. This exercises the actual module notice
    // driver, independently of global notify_module routing or module causes.
    let domain = isolated_domain();
    let source = Signal::in_isolated_domain(domain.clone()).unwrap();
    let successor = Signal::in_isolated_domain(domain.clone()).unwrap();
    let module_domain = domain.isolated_module.get().unwrap().clone();
    let panic_wake = Arc::new(ReentrantPanickingWake {
        source: source.clone(),
        successor: successor.clone(),
        domain: module_domain.clone(),
        effects: AtomicUsize::new(0),
    });
    let completion = Arc::new(WakeCount::default());
    let mut first = arm(&source, panic_wake.clone());
    let mut remaining = arm(&source, completion.clone());
    let mut reentrant = arm(&successor, completion.clone());

    source.mark_module();
    let panic = catch_unwind(AssertUnwindSafe(|| source.notice_in(&module_domain, true)))
        .expect_err("the authored waker panic must resume after notification cleanup");
    assert!(panic.is::<AuthoredWakePanic>());
    assert_eq!(panic_wake.effects.load(Ordering::SeqCst), 1);
    completion.await_count(2);
    for waiter in [&mut first, &mut remaining, &mut reentrant] {
        assert_eq!(
            observed(waiter),
            Poll::Ready(SettlementStatus::ModuleInterrupted)
        );
    }

    let later = Signal::in_isolated_domain(domain.clone()).unwrap();
    let later_wakes = Arc::new(WakeCount::default());
    let mut later_waiter = arm(&later, later_wakes.clone());
    later.mark_module();
    later.notice_in(&module_domain, true);
    later_wakes.await_count(1);
    assert_eq!(
        observed(&mut later_waiter),
        Poll::Ready(SettlementStatus::ModuleInterrupted)
    );
}

#[test]
fn native_association_finds_a_retained_module_cause_at_the_last_vertex_of_a_follow_cycle() {
    use crate::{
        JsValue, Module, Source,
        error::{DiagnosticEngine, DiagnosticKind, EngineError, RuntimeLimitError},
        execution::ExecutionOptions,
        js_string,
    };

    const LIMIT: u64 = 64;
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let module = Module::parse(
        Source::from_bytes(
            "globalThis.moduleCauseEffects = 1;
             for (let step = 0; step < 1000; step++) {}",
        ),
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
    assert_eq!(stopped.report.instructions().unwrap(), LIMIT);
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: LIMIT
        }))
    );
    assert_eq!(
        context
            .global_object()
            .get(js_string!("moduleCauseEffects"), context.access())
            .unwrap(),
        1.into()
    );
    let retained = context
        .with_execution(ExecutionOptions::new(4096), |access| {
            module.evaluate(access)
        })
        .unwrap();
    assert_eq!(retained.report.instructions().unwrap(), 0);
    let retained = retained.result.unwrap_err();
    let Some(EngineError::ModuleInterrupted(retained)) = retained.as_engine() else {
        panic!("fresh source evaluation must expose its retained interruption")
    };
    assert_eq!(
        retained.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(
            RuntimeLimitError::Instruction { limit: LIMIT }
        ))
    );
    let origin = module.kind().as_source_text().unwrap().evaluation.clone();

    // The parsed module and all four native associations retain legitimate roots.
    // The only module origin is D; the real handoffs form A -> B -> C -> D -> B.
    let a = Association::new(&context).unwrap();
    let b = Association::new(&context).unwrap();
    let c = Association::new(&context).unwrap();
    let d = Association::new(&context).unwrap();
    drop(Association::prepare_follow(&a, &b).unwrap().commit());
    drop(Association::prepare_follow(&b, &c).unwrap().commit());
    drop(Association::prepare_follow(&c, &d).unwrap().commit());
    drop(Association::prepare_follow(&d, &b).unwrap().commit());
    let wakes = Arc::new(WakeCount::default());
    let mut wait = arm(&a.signal, wakes.clone());
    d.observe_module(origin);
    wakes.await_count(1);
    assert_eq!(
        observed(&mut wait),
        Poll::Ready(SettlementStatus::ModuleInterrupted)
    );

    let discovered = Association::module_error(&a)
        .expect("a reachable interrupted origin must be found throughout the follow cycle");
    let Some(EngineError::ModuleInterrupted(discovered)) = discovered.as_engine() else {
        panic!("native observation must preserve the module interruption")
    };
    assert_eq!(discovered.identity(), retained.identity());
    assert_eq!(discovered.cause().kind(), retained.cause().kind());
}
