//! Native settlement custody, independent of guest reactions and task results.

use crate::{
    Context, JsResult,
    error::PanicError,
    job::{
        JobFault, JobFaultCause, JobFaultOwner, JobObserver, JobRegistrationError, JobRetirement,
        RetirementRegistration, RetirementRelay, UnwindCleanup,
    },
};
use std::{
    cell::Cell,
    rc::{Rc, Weak as LocalWeak},
    sync::{
        Arc, Mutex, MutexGuard, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js_gc::heap::{RecordRef, Rooted, Trace};

// Test fixtures have their own synchronization assertions. Production poison is
// always a fault followed by cleanup, never permission to resume registry work.
#[cfg(test)]
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().expect("test fixture mutex")
}
fn reserve<T>(values: &mut Vec<T>) -> JsResult<()> {
    crate::job::reserve_registration(values)
        .map_err(|error| JobRegistrationError::Resource(error).into())
}
fn fault(owner: JobFaultOwner, cause: JobFaultCause) -> JobFault {
    JobFault::new(owner, cause)
}
fn signal_fault(cause: JobFaultCause) -> JobFault {
    fault(JobFaultOwner::NativeSignal, cause)
}
fn domain_fault(cause: JobFaultCause) -> JobFault {
    fault(JobFaultOwner::NativeDomain, cause)
}
fn engine_fault(value: JobFault) -> crate::JsError {
    JobRegistrationError::Fault(value).into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettlementStatus {
    Pending,
    Settled,
    Retired(JobRetirement),
    ModuleInterrupted,
}
impl SettlementStatus {
    fn encode(self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Settled => 1,
            Self::ModuleInterrupted => 2,
            Self::Retired(JobRetirement::Cancelled) => 3,
            Self::Retired(JobRetirement::OwnerDropped) => 4,
            Self::Retired(JobRetirement::Interrupted) => 5,
            Self::Retired(JobRetirement::Dropped) => 6,
        }
    }
    fn decode(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Self::Pending),
            1 => Some(Self::Settled),
            2 => Some(Self::ModuleInterrupted),
            3 => Some(Self::Retired(JobRetirement::Cancelled)),
            4 => Some(Self::Retired(JobRetirement::OwnerDropped)),
            5 => Some(Self::Retired(JobRetirement::Interrupted)),
            6 => Some(Self::Retired(JobRetirement::Dropped)),
            _ => None,
        }
    }
}
const FAULTED: u8 = 7;
struct Waiter {
    waker: Mutex<Option<Waker>>,
}
struct SignalState {
    waiters: Vec<Option<Arc<Waiter>>>,
    dependents: Vec<Option<Weak<Dependency>>>,
}
struct NoticeQueue {
    head: Option<Arc<Signal>>,
    tail: Option<Arc<Signal>>,
    wake_head: Option<Arc<Signal>>,
    wake_tail: Option<Arc<Signal>>,
    driving: bool,
}
impl NoticeQueue {
    fn new() -> Self {
        Self {
            head: None,
            tail: None,
            wake_head: None,
            wake_tail: None,
            driving: false,
        }
    }
}
enum MemberEntry {
    Vacant(Option<usize>),
    Live(Weak<Signal>),
}
impl MemberEntry {
    fn signal(&self) -> Option<&Weak<Signal>> {
        match self {
            Self::Live(signal) => Some(signal),
            Self::Vacant(_) => None,
        }
    }
}
struct Members {
    entries: Vec<MemberEntry>,
    free: Option<usize>,
    live: usize,
}
impl Members {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: None,
            live: 0,
        }
    }
    fn free_next(&self, index: usize) -> Result<Option<usize>, JobFault> {
        let next = match self.entries.get(index) {
            Some(MemberEntry::Vacant(next)) => *next,
            Some(MemberEntry::Live(_)) => {
                return Err(domain_fault(JobFaultCause::InvalidFreeLink { index }));
            }
            None => return Err(domain_fault(JobFaultCause::InvalidIndex { index })),
        };
        if let Some(next) = next
            && (next == index || !matches!(self.entries.get(next), Some(MemberEntry::Vacant(_))))
        {
            return Err(domain_fault(JobFaultCause::InvalidFreeLink { index: next }));
        }
        // A null next is exactly the final vacant slot. Validate the count
        // correspondence without traversing the free chain.
        let after_one = self
            .live
            .checked_add(1)
            .ok_or_else(|| domain_fault(JobFaultCause::StateInconsistency))?;
        if next.is_none() != (after_one == self.entries.len()) {
            return Err(domain_fault(JobFaultCause::StateInconsistency));
        }
        Ok(next)
    }
    fn insert(
        &mut self,
        signal: &Arc<Signal>,
        #[cfg(test)] fail_reservation: &AtomicBool,
    ) -> Result<usize, JobRegistrationError> {
        if self.live > self.entries.len() {
            return Err(domain_fault(JobFaultCause::StateInconsistency).into());
        }
        let live = self
            .live
            .checked_add(1)
            .ok_or_else(|| domain_fault(JobFaultCause::StateInconsistency))?;
        let index = if let Some(index) = self.free {
            if self.live == self.entries.len() {
                return Err(domain_fault(JobFaultCause::StateInconsistency).into());
            }
            let next = self.free_next(index)?;
            // All identity/link/count validation precedes either mutation.
            let entry = self
                .entries
                .get_mut(index)
                .ok_or_else(|| domain_fault(JobFaultCause::InvalidIndex { index }))?;
            *entry = MemberEntry::Live(Arc::downgrade(signal));
            self.free = next;
            index
        } else {
            if self.live != self.entries.len() {
                return Err(domain_fault(JobFaultCause::StateInconsistency).into());
            }
            #[cfg(test)]
            if fail_reservation.swap(false, Ordering::AcqRel) {
                return Err(crate::job::JobResourceError::Allocation.into());
            }
            crate::job::reserve_registration(&mut self.entries)?;
            let index = self.entries.len();
            self.entries.push(MemberEntry::Live(Arc::downgrade(signal)));
            index
        };
        self.live = live;
        Ok(index)
    }
    fn remove(&mut self, index: usize, signal: &Signal) -> Result<MemberEntry, JobFault> {
        match self.entries.get(index) {
            // This private removal entry requires the actual live Signal witness:
            // constructor rollback retains its Arc, and Signal::drop executes
            // before that same allocation can be reclaimed/reused. A bare index
            // or address is never accepted from an independently lived key.
            Some(MemberEntry::Live(entry)) if std::ptr::eq(entry.as_ptr(), signal) => {}
            Some(_) => return Err(domain_fault(JobFaultCause::InvalidSlotIdentity { index })),
            None => return Err(domain_fault(JobFaultCause::InvalidIndex { index })),
        }
        let live = self
            .live
            .checked_sub(1)
            .ok_or_else(|| domain_fault(JobFaultCause::CounterUnderflow))?;
        if self.live > self.entries.len() {
            return Err(domain_fault(JobFaultCause::StateInconsistency));
        }
        if let Some(free) = self.free {
            if self.live == self.entries.len() {
                return Err(domain_fault(JobFaultCause::StateInconsistency));
            }
            self.free_next(free)?;
        } else if self.live != self.entries.len() {
            return Err(domain_fault(JobFaultCause::StateInconsistency));
        }
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| domain_fault(JobFaultCause::InvalidIndex { index }))?;
        let removed = std::mem::replace(entry, MemberEntry::Vacant(self.free));
        self.free = Some(index);
        self.live = live;
        Ok(removed)
    }
}
struct Domain {
    fault: OnceLock<JobFault>,
    members: Mutex<Members>,
    queue: Mutex<NoticeQueue>,
    #[cfg(test)]
    empty_notice_hook: Option<EmptyNoticeHook>,
    #[cfg(test)]
    fail_reservation: AtomicBool,
    #[cfg(test)]
    isolated_module: OnceLock<Arc<Domain>>,
}
#[cfg(test)]
struct EmptyNoticeHook {
    visits: std::sync::atomic::AtomicUsize,
    reached: std::sync::mpsc::Sender<()>,
    resume: Mutex<std::sync::mpsc::Receiver<()>>,
}
impl Domain {
    fn new() -> Self {
        Self {
            fault: OnceLock::new(),
            members: Mutex::new(Members::new()),
            queue: Mutex::new(NoticeQueue::new()),
            #[cfg(test)]
            empty_notice_hook: None,
            #[cfg(test)]
            fail_reservation: AtomicBool::new(false),
            #[cfg(test)]
            isolated_module: OnceLock::new(),
        }
    }
    fn register(&self, signal: &Arc<Signal>) -> Result<usize, JobRegistrationError> {
        if let Some(error) = signal.fault.get() {
            return Err((*error).into());
        }
        if let Some(error) = self.fault.get() {
            return Err((*error).into());
        }
        let mut members = match self.members.lock() {
            Ok(members) => members,
            Err(poison) => {
                drop(poison.into_inner());
                let error = domain_fault(JobFaultCause::PoisonedRegistry);
                self.quarantine(error);
                return Err(self.first_fault(error).into());
            }
        };
        if let Some(error) = self.fault.get() {
            return Err((*error).into());
        }
        if let Some(error) = signal.fault.get() {
            return Err((*error).into());
        }
        #[cfg(test)]
        let result = members.insert(signal, &self.fail_reservation);
        #[cfg(not(test))]
        let result = members.insert(signal);
        drop(members);
        if let Err(JobRegistrationError::Fault(error)) = result {
            self.quarantine(error);
            return Err(self.first_fault(error).into());
        }
        result
    }
    fn unregister(&self, index: usize, signal: &Signal) {
        if self.fault.get().is_some() {
            return;
        }
        let removed = match self.members.lock() {
            Ok(mut members) => {
                if self.fault.get().is_some() {
                    return;
                }
                members.remove(index, signal)
            }
            Err(poison) => {
                drop(poison.into_inner());
                Err(domain_fault(JobFaultCause::PoisonedRegistry))
            }
        };
        match removed {
            Ok(removed) => drop(removed),
            Err(error) => self.quarantine(error),
        }
    }
    fn first_fault(&self, candidate: JobFault) -> JobFault {
        *self.fault.get_or_init(|| candidate)
    }
    fn quarantine(&self, error: JobFault) {
        // The first caller owns the physical cleanup bank. Concurrent callers
        // may still publish the same diagnostic, but cannot reopen admission.
        if self.fault.set(error).is_err() {
            return;
        }
        let members = {
            let mut guard = match self.members.lock() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            std::mem::replace(&mut *guard, Members::new())
        };
        let queue = {
            let mut guard = match self.queue.lock() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            std::mem::replace(&mut *guard, NoticeQueue::new())
        };
        // Physical entries are authoritative even if every work link is corrupt.
        // Mark the whole domain before detaching any owned links or invoking
        // arbitrary native callbacks.
        let mut unwind = UnwindCleanup::default();
        for entry in &members.entries {
            unwind.run(|| {
                if let Some(signal) = entry.signal().and_then(Weak::upgrade)
                    && signal.belongs_to(self)
                {
                    signal.mark_fault(error);
                }
            });
        }
        for entry in &members.entries {
            unwind.run(|| {
                if let Some(signal) = entry.signal().and_then(Weak::upgrade)
                    && signal.belongs_to(self)
                {
                    signal.detach_links(self);
                }
            });
        }
        unwind.run(|| drop(queue));
        for entry in &members.entries {
            unwind.run(|| {
                if let Some(signal) = entry.signal().and_then(Weak::upgrade)
                    && signal.belongs_to(self)
                {
                    signal.notify_direct();
                }
            });
        }
        drop(members);
        unwind.finish();
    }
    fn access(&self) -> Result<MutexGuard<'_, NoticeQueue>, JobFault> {
        if let Some(error) = self.fault.get() {
            return Err(*error);
        }
        match self.queue.lock() {
            Ok(queue) => {
                if let Some(error) = self.fault.get() {
                    drop(queue);
                    Err(*error)
                } else {
                    Ok(queue)
                }
            }
            Err(poison) => {
                drop(poison.into_inner());
                let error = domain_fault(JobFaultCause::PoisonedRegistry);
                self.quarantine(error);
                Err(self.first_fault(error))
            }
        }
    }
    #[cfg(test)]
    fn after_empty_notice(&self) {
        if let Some(hook) = &self.empty_notice_hook
            && hook.visits.fetch_add(1, Ordering::SeqCst) == 1
        {
            hook.reached
                .send(())
                .expect("second retirement thread is alive");
            lock(&hook.resume)
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("concurrent retirement must return while the first driver owns the domain");
        }
    }
}
fn domain() -> &'static Domain {
    static DOMAIN: OnceLock<Domain> = OnceLock::new();
    DOMAIN.get_or_init(Domain::new)
}
fn module_domain() -> &'static Domain {
    static DOMAIN: OnceLock<Domain> = OnceLock::new();
    DOMAIN.get_or_init(Domain::new)
}
enum DomainRef {
    Shared(&'static Domain),
    #[cfg(test)]
    Isolated(Arc<Domain>),
}
impl DomainRef {
    fn get(&self) -> &Domain {
        match self {
            Self::Shared(domain) => domain,
            #[cfg(test)]
            Self::Isolated(domain) => domain,
        }
    }
}
struct Memberships {
    ordinary: usize,
    module: usize,
}
struct MemberRollback<'a> {
    domain: &'a Domain,
    signal: &'a Signal,
    index: usize,
    armed: bool,
}
impl Drop for MemberRollback<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.domain.unregister(self.index, self.signal);
        }
    }
}
/// Status and prepared notification links only; never owns engine values.
pub(crate) struct Signal {
    ordinary_domain: DomainRef,
    module_domain: DomainRef,
    memberships: OnceLock<Memberships>,
    status: AtomicU8,
    fault: OnceLock<JobFault>,
    notified: AtomicBool,
    module_notified: AtomicBool,
    ordinary_work: AtomicU8,
    module_work: AtomicU8,
    module_next: Mutex<Option<Arc<Signal>>>,
    state: Mutex<SignalState>,
    next: Mutex<Option<Arc<Signal>>>,
}
impl std::fmt::Debug for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NativeSettlementSignal")
            .field(&self.status())
            .finish()
    }
}
impl Signal {
    fn new() -> Result<Arc<Self>, JobRegistrationError> {
        Self::construct(
            DomainRef::Shared(domain()),
            DomainRef::Shared(module_domain()),
        )
    }
    fn construct(
        ordinary_domain: DomainRef,
        module_domain: DomainRef,
    ) -> Result<Arc<Self>, JobRegistrationError> {
        if std::ptr::eq(ordinary_domain.get(), module_domain.get()) {
            return Err(domain_fault(JobFaultCause::OwnerMismatch).into());
        }
        if let Some(error) = ordinary_domain
            .get()
            .fault
            .get()
            .or_else(|| module_domain.get().fault.get())
        {
            return Err((*error).into());
        }
        // Both routes exist before a Signal can become visible to engine state.
        let signal = Arc::new(Self {
            ordinary_domain,
            module_domain,
            memberships: OnceLock::new(),
            status: AtomicU8::new(0),
            fault: OnceLock::new(),
            notified: AtomicBool::new(false),
            module_notified: AtomicBool::new(false),
            ordinary_work: AtomicU8::new(0),
            module_work: AtomicU8::new(0),
            module_next: Mutex::new(None),
            next: Mutex::new(None),
            state: Mutex::new(SignalState {
                waiters: Vec::new(),
                dependents: Vec::new(),
            }),
        });
        let ordinary = signal.notification_domain().register(&signal)?;
        let mut rollback = MemberRollback {
            domain: signal.notification_domain(),
            signal: &signal,
            index: ordinary,
            armed: true,
        };
        let module = signal.module_notification_domain().register(&signal)?;
        if signal
            .memberships
            .set(Memberships { ordinary, module })
            .is_err()
        {
            signal
                .module_notification_domain()
                .unregister(module, &signal);
            return Err(signal_fault(JobFaultCause::StateInconsistency).into());
        }
        rollback.armed = false;
        drop(rollback);
        if let Some(error) = signal.fault.get() {
            return Err((*error).into());
        }
        Ok(signal)
    }
    fn notification_domain(&self) -> &Domain {
        self.ordinary_domain.get()
    }
    fn module_notification_domain(&self) -> &Domain {
        self.module_domain.get()
    }
    fn belongs_to(&self, domain: &Domain) -> bool {
        std::ptr::eq(self.notification_domain(), domain)
            || std::ptr::eq(self.module_notification_domain(), domain)
    }
    #[cfg(test)]
    fn in_isolated_domain(domain: Arc<Domain>) -> Result<Arc<Self>, JobRegistrationError> {
        if let Some(error) = domain.fault.get() {
            return Err((*error).into());
        }
        let module = domain
            .isolated_module
            .get_or_init(|| Arc::new(Domain::new()))
            .clone();
        Self::construct(DomainRef::Isolated(domain), DomainRef::Isolated(module))
    }
    fn mark_fault(&self, error: JobFault) {
        self.fault.get_or_init(|| error);
        loop {
            let raw = self.status.load(Ordering::Acquire);
            if raw == FAULTED
                || SettlementStatus::decode(raw)
                    .is_some_and(|status| status != SettlementStatus::Pending)
            {
                return;
            }
            if self
                .status
                .compare_exchange(raw, FAULTED, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }
    fn fail(&self, error: JobFault) {
        self.mark_fault(error);
        self.notify_direct();
    }
    fn reject_waker(&self, error: JobFault, waker: Waker) {
        self.mark_fault(error);
        let mut unwind = UnwindCleanup::default();
        unwind.run(|| drop(waker));
        unwind.run(|| self.notify_direct());
        unwind.finish();
    }
    pub(crate) fn status(&self) -> Result<SettlementStatus, JobFault> {
        let raw = self.status.load(Ordering::Acquire);
        if let Some(status) = SettlementStatus::decode(raw) {
            return Ok(status);
        }
        let error = if raw == FAULTED {
            if let Some(error) = self.fault.get() {
                return Err(*error);
            }
            signal_fault(JobFaultCause::MissingCause)
        } else {
            signal_fault(JobFaultCause::InvalidDisposition { raw })
        };
        self.fail(error);
        self.observation()
    }
    fn check(&self) -> Result<(), JobFault> {
        self.status()?;
        match self.fault.get() {
            Some(error) => Err(*error),
            None => Ok(()),
        }
    }
    fn state_access(&self) -> Result<MutexGuard<'_, SignalState>, JobFault> {
        if let Some(error) = self.fault.get() {
            return Err(*error);
        }
        match self.state.lock() {
            Ok(state) => {
                if let Some(error) = self.fault.get() {
                    drop(state);
                    Err(*error)
                } else {
                    Ok(state)
                }
            }
            Err(poison) => {
                drop(poison.into_inner());
                let error = signal_fault(JobFaultCause::PoisonedRegistry);
                self.fail(error);
                Err(*self.fault.get_or_init(|| error))
            }
        }
    }
    fn publish_pending(&self, status: SettlementStatus) -> bool {
        self.status
            .compare_exchange(0, status.encode(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    fn retire_pending(self: &Arc<Self>, reason: JobRetirement) {
        if self.publish_pending(SettlementStatus::Retired(reason)) {
            self.notify();
        }
    }
    fn retire_epoch(self: &Arc<Self>, epoch: &Epoch, reason: JobRetirement) {
        let Ok(_state) = self.state_access() else {
            return;
        };
        let changed = epoch.active.load(Ordering::Acquire)
            && self.publish_pending(SettlementStatus::Retired(reason));
        drop(_state);
        if changed {
            self.notify();
        }
    }
    fn fault_epoch(self: &Arc<Self>, epoch: &Epoch, error: JobFault) {
        // The active binding is checked under the same guard as handoff commit.
        let Ok(state) = self.state_access() else {
            return;
        };
        let active = epoch.active.load(Ordering::Acquire);
        if active {
            self.mark_fault(error);
        }
        drop(state);
        if active {
            self.notify();
        }
    }
    fn notify(self: &Arc<Self>) {
        match self.status() {
            Ok(SettlementStatus::Retired(_)) | Err(_)
                if !self.notified.swap(true, Ordering::AcqRel) =>
            {
                self.notice_in(self.notification_domain(), false)
            }
            Ok(SettlementStatus::Retired(_)) | Err(_) => {}
            Ok(SettlementStatus::ModuleInterrupted) => self.notify_module(),
            _ => self.wake(),
        }
    }
    pub(crate) fn settle(self: &Arc<Self>) {
        // Actual guest settlement keeps the established precedence over generic
        // retirement and observation defects. A real module cause remains final.
        let mut raw = self.status.load(Ordering::Acquire);
        loop {
            if raw == SettlementStatus::ModuleInterrupted.encode() {
                return;
            }
            if raw != FAULTED && SettlementStatus::decode(raw).is_none() {
                self.mark_fault(signal_fault(JobFaultCause::InvalidDisposition { raw }));
                raw = self.status.load(Ordering::Acquire);
                continue;
            }
            match self.status.compare_exchange(
                raw,
                SettlementStatus::Settled.encode(),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(current) => raw = current,
            }
        }
        self.wake();
    }
    fn wake(&self) {
        if self.fault.get().is_some() {
            let waiters = {
                let mut state = match self.state.lock() {
                    Ok(state) => state,
                    Err(poison) => poison.into_inner(),
                };
                std::mem::take(&mut state.waiters)
            };
            let mut unwind = UnwindCleanup::default();
            for waiter in waiters.iter().filter_map(|entry| entry.as_ref()) {
                unwind.run(|| self.wake_waiter(waiter));
            }
            drop(waiters);
            unwind.finish();
            return;
        }
        let count = match self.state_access() {
            Ok(state) => state.waiters.len(),
            Err(_) => return,
        };
        let mut unwind = UnwindCleanup::default();
        for index in 0..count {
            let waiter = match self.state_access() {
                Ok(state) => state.waiters.get(index).and_then(Clone::clone),
                Err(_) => break,
            };
            if let Some(waiter) = waiter {
                unwind.run(|| self.wake_waiter(&waiter));
            }
        }
        unwind.finish();
    }
    fn wake_waiter(&self, waiter: &Waiter) {
        let (waker, poison) = match waiter.waker.lock() {
            Ok(mut slot) => (slot.take(), false),
            Err(poison) => (poison.into_inner().take(), true),
        };
        if poison {
            self.mark_fault(signal_fault(JobFaultCause::PoisonedWaker));
        }
        let mut unwind = UnwindCleanup::default();
        if poison {
            unwind.run(|| self.notify_direct());
        }
        if let Some(waker) = waker {
            unwind.run(|| waker.wake());
        }
        unwind.finish();
    }
    fn notify_direct(&self) {
        // A failed signal's physical banks remain the cleanup authority.
        let state = {
            let mut state = match self.state.lock() {
                Ok(state) => state,
                Err(poison) => poison.into_inner(),
            };
            SignalState {
                waiters: std::mem::take(&mut state.waiters),
                dependents: std::mem::take(&mut state.dependents),
            }
        };
        let observation = self.observation();
        let mut unwind = UnwindCleanup::default();
        for dependent in state
            .dependents
            .iter()
            .filter_map(|entry| entry.as_ref())
            .filter_map(Weak::upgrade)
        {
            unwind.run(|| dependent.deliver(observation));
        }
        for waiter in state.waiters.iter().filter_map(|entry| entry.as_ref()) {
            unwind.run(|| self.wake_waiter(waiter));
        }
        unwind.run(|| drop(state));
        unwind.finish();
    }
    fn observation(&self) -> Result<SettlementStatus, JobFault> {
        let raw = self.status.load(Ordering::Acquire);
        match SettlementStatus::decode(raw) {
            Some(status) => Ok(status),
            None => Err(*self.fault.get_or_init(|| {
                signal_fault(if raw == FAULTED {
                    JobFaultCause::MissingCause
                } else {
                    JobFaultCause::InvalidDisposition { raw }
                })
            })),
        }
    }
    pub(crate) fn mark_module(&self) {
        let mut raw = self.status.load(Ordering::Acquire);
        while raw != SettlementStatus::Settled.encode()
            && raw != SettlementStatus::ModuleInterrupted.encode()
        {
            if raw != FAULTED && SettlementStatus::decode(raw).is_none() {
                self.mark_fault(signal_fault(JobFaultCause::InvalidDisposition { raw }));
                raw = self.status.load(Ordering::Acquire);
                continue;
            }
            match self.status.compare_exchange(
                raw,
                SettlementStatus::ModuleInterrupted.encode(),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(current) => raw = current,
            }
        }
    }
    fn mark_module_epoch(self: &Arc<Self>, epoch: &Arc<Epoch>) {
        // This publishes an actual source-module marker, not ordinary registry
        // progress. Preserve its priority even when observation already failed;
        // the same guard still serializes active-epoch handoff authority.
        let (state, poisoned) = match self.state.lock() {
            Ok(state) => (state, false),
            Err(poison) => (poison.into_inner(), true),
        };
        if poisoned {
            self.mark_fault(signal_fault(JobFaultCause::PoisonedRegistry));
        }
        let active = epoch.active.load(Ordering::Acquire);
        if active {
            self.mark_module();
        }
        drop(state);
        let mut unwind = UnwindCleanup::default();
        if active {
            unwind.run(|| self.notify_module());
        }
        if poisoned {
            unwind.run(|| self.notify_direct());
        }
        unwind.finish();
    }
    pub(crate) fn notify_module(self: &Arc<Self>) {
        if self.observation() != Ok(SettlementStatus::ModuleInterrupted)
            || self.module_notified.swap(true, Ordering::AcqRel)
        {
            return;
        }
        self.notice_in(self.module_notification_domain(), true);
    }
    fn work_state(&self, module: bool) -> &AtomicU8 {
        if module {
            &self.module_work
        } else {
            &self.ordinary_work
        }
    }
    fn valid_route(&self, domain: &Domain, module: bool) -> bool {
        std::ptr::eq(
            if module {
                self.module_notification_domain()
            } else {
                self.notification_domain()
            },
            domain,
        )
    }
    fn link(&self, module: bool) -> &Mutex<Option<Arc<Signal>>> {
        if module {
            &self.module_next
        } else {
            &self.next
        }
    }
    fn detach_links(&self, domain: &Domain) {
        let module = std::ptr::eq(self.module_notification_domain(), domain);
        let old = {
            let mut link = match self.link(module).lock() {
                Ok(link) => link,
                Err(poison) => poison.into_inner(),
            };
            link.take()
        };
        drop(old);
    }
    fn notice_in(self: &Arc<Self>, domain: &Domain, module: bool) {
        // Queue then link is the only nested lock order. No state access,
        // callbacks or final Arc disposal occur while either guard is held.
        let insertion = (|| {
            let mut queue = domain.access()?;
            if !self.valid_route(domain, module) {
                drop(queue);
                return Err(domain_fault(JobFaultCause::OwnerMismatch));
            }
            if queue.head.is_some() != queue.tail.is_some() {
                drop(queue);
                return Err(domain_fault(JobFaultCause::StateInconsistency));
            }
            if self
                .work_state(module)
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                drop(queue);
                return Err(domain_fault(JobFaultCause::StateInconsistency));
            }
            let previous = queue.tail.clone();
            if let Some(tail) = &previous {
                if !tail.valid_route(domain, module)
                    || tail.work_state(module).load(Ordering::Acquire) != 1
                {
                    drop(queue);
                    return Err(domain_fault(JobFaultCause::StateInconsistency));
                }
                match tail.link(module).lock() {
                    Ok(mut link) => {
                        if link.is_some() {
                            drop(link);
                            drop(queue);
                            return Err(domain_fault(JobFaultCause::StateInconsistency));
                        }
                        *link = Some(self.clone());
                    }
                    Err(poison) => {
                        drop(poison.into_inner());
                        drop(queue);
                        return Err(domain_fault(JobFaultCause::PoisonedLink));
                    }
                }
            } else {
                queue.head = Some(self.clone());
            }
            let old_tail = queue.tail.replace(self.clone());
            let drive = !queue.driving;
            queue.driving = true;
            drop(queue);
            drop(old_tail);
            drop(previous);
            Ok(drive)
        })();
        let drive = match insertion {
            Ok(drive) => drive,
            Err(error) => {
                domain.quarantine(error);
                return;
            }
        };
        if !drive {
            return;
        }
        enum Work {
            Notice(Arc<Signal>),
            Wake(Arc<Signal>),
            Idle,
        }
        let mut unwind = UnwindCleanup::default();
        loop {
            #[cfg(test)]
            {
                let empty = match domain.access() {
                    Ok(queue) => queue.head.is_none(),
                    Err(_) => break,
                };
                if empty {
                    domain.after_empty_notice();
                }
            }
            let selection = (|| {
                let mut queue = domain.access()?;
                if queue.head.is_some() != queue.tail.is_some()
                    || queue.wake_head.is_some() != queue.wake_tail.is_some()
                {
                    drop(queue);
                    return Err(domain_fault(JobFaultCause::StateInconsistency));
                }
                let module_work = queue.head.is_some();
                let next = if module_work {
                    queue.head.as_ref()
                } else {
                    queue.wake_head.as_ref()
                }
                .cloned();
                if let Some(next) = next {
                    if !next.valid_route(domain, module)
                        || next
                            .work_state(module)
                            .compare_exchange(
                                if module_work { 1 } else { 3 },
                                if module_work { 2 } else { 4 },
                                Ordering::AcqRel,
                                Ordering::Acquire,
                            )
                            .is_err()
                    {
                        drop(queue);
                        return Err(domain_fault(JobFaultCause::StateInconsistency));
                    }
                    let following = match next.link(module).lock() {
                        Ok(mut link) => link.take(),
                        Err(poison) => {
                            drop(poison.into_inner());
                            drop(queue);
                            return Err(domain_fault(JobFaultCause::PoisonedLink));
                        }
                    };
                    let tail = if module_work {
                        queue.tail.as_ref()
                    } else {
                        queue.wake_tail.as_ref()
                    };
                    let last = tail.is_some_and(|tail| Arc::ptr_eq(tail, &next));
                    if last != following.is_none() {
                        drop(queue);
                        drop(following);
                        return Err(domain_fault(JobFaultCause::StateInconsistency));
                    }
                    let (old_head, old_tail) = if module_work {
                        (
                            std::mem::replace(&mut queue.head, following),
                            if queue.head.is_none() {
                                queue.tail.take()
                            } else {
                                None
                            },
                        )
                    } else {
                        (
                            std::mem::replace(&mut queue.wake_head, following),
                            if queue.wake_head.is_none() {
                                queue.wake_tail.take()
                            } else {
                                None
                            },
                        )
                    };
                    drop(queue);
                    drop(old_head);
                    drop(old_tail);
                    Ok(if module_work {
                        Work::Notice(next)
                    } else {
                        Work::Wake(next)
                    })
                } else {
                    queue.driving = false;
                    Ok(Work::Idle)
                }
            })();
            let work = match selection {
                Ok(work) => work,
                Err(error) => {
                    unwind.run(|| domain.quarantine(error));
                    break;
                }
            };
            match work {
                Work::Notice(next) => {
                    unwind.run(|| next.deliver_dependents());
                    if domain.fault.get().is_some() {
                        break;
                    }
                    let queued = (|| {
                        let mut queue = domain.access()?;
                        if queue.wake_head.is_some() != queue.wake_tail.is_some()
                            || next
                                .work_state(module)
                                .compare_exchange(2, 3, Ordering::AcqRel, Ordering::Acquire)
                                .is_err()
                        {
                            drop(queue);
                            return Err(domain_fault(JobFaultCause::StateInconsistency));
                        }
                        let previous = queue.wake_tail.clone();
                        if let Some(tail) = &previous {
                            if !tail.valid_route(domain, module)
                                || tail.work_state(module).load(Ordering::Acquire) != 3
                            {
                                drop(queue);
                                return Err(domain_fault(JobFaultCause::StateInconsistency));
                            }
                            match tail.link(module).lock() {
                                Ok(mut link) => {
                                    if link.is_some() {
                                        drop(link);
                                        drop(queue);
                                        return Err(domain_fault(
                                            JobFaultCause::StateInconsistency,
                                        ));
                                    }
                                    *link = Some(next.clone());
                                }
                                Err(poison) => {
                                    drop(poison.into_inner());
                                    drop(queue);
                                    return Err(domain_fault(JobFaultCause::PoisonedLink));
                                }
                            }
                        } else {
                            queue.wake_head = Some(next.clone());
                        }
                        let old_tail = queue.wake_tail.replace(next);
                        drop(queue);
                        drop(old_tail);
                        drop(previous);
                        Ok(())
                    })();
                    if let Err(error) = queued {
                        unwind.run(|| domain.quarantine(error));
                        break;
                    }
                }
                Work::Wake(wake) => unwind.run(|| wake.wake()),
                Work::Idle => break,
            }
        }
        unwind.finish();
    }
    fn deliver_dependents(&self) {
        if self.fault.get().is_some() {
            let dependents = {
                let mut state = match self.state.lock() {
                    Ok(state) => state,
                    Err(poison) => poison.into_inner(),
                };
                std::mem::take(&mut state.dependents)
            };
            let mut unwind = UnwindCleanup::default();
            for dependent in dependents
                .iter()
                .filter_map(|entry| entry.as_ref())
                .filter_map(Weak::upgrade)
            {
                unwind.run(|| dependent.deliver(self.observation()));
            }
            drop(dependents);
            unwind.finish();
            return;
        }
        let count = match self.state_access() {
            Ok(state) => state.dependents.len(),
            Err(_) => return,
        };
        let mut unwind = UnwindCleanup::default();
        for index in 0..count {
            let dependent = match self.state_access() {
                Ok(state) => state
                    .dependents
                    .get(index)
                    .and_then(Clone::clone)
                    .and_then(|entry| entry.upgrade()),
                Err(_) => break,
            };
            if let Some(dependent) = dependent {
                unwind.run(|| dependent.deliver(self.observation()));
            }
        }
        unwind.finish();
    }
    pub(crate) fn wait(self: &Arc<Self>) -> Result<SettlementWait, JobRegistrationError> {
        match self.status() {
            Ok(status) if status != SettlementStatus::Pending => {
                return Ok(SettlementWait {
                    signal: self.clone(),
                    index: 0,
                    waiter: None,
                });
            }
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        let waiter = Arc::new(Waiter {
            waker: Mutex::new(None),
        });
        let index = {
            let mut state = self.state_access()?;
            if let Some((index, slot)) = state
                .waiters
                .iter_mut()
                .enumerate()
                .find(|(_, slot)| slot.is_none())
            {
                *slot = Some(waiter.clone());
                index
            } else {
                crate::job::reserve_registration(&mut state.waiters)?;
                let index = state.waiters.len();
                state.waiters.push(Some(waiter.clone()));
                index
            }
        };
        Ok(SettlementWait {
            signal: self.clone(),
            index,
            waiter: Some(waiter),
        })
    }
    fn subscribe(
        self: &Arc<Self>,
        target: &Arc<Signal>,
        epoch: Arc<Epoch>,
    ) -> Result<DependencyGuard, JobRegistrationError> {
        self.check()?;
        target.check()?;
        if let Some(error) = target.fault.get() {
            return Err((*error).into());
        }
        let dependency = Arc::new(Dependency {
            target: Arc::downgrade(target),
            epoch,
        });
        let index = {
            let mut state = self.state_access()?;
            if let Some((index, slot)) = state
                .dependents
                .iter_mut()
                .enumerate()
                .find(|(_, slot)| slot.is_none())
            {
                *slot = Some(Arc::downgrade(&dependency));
                index
            } else {
                crate::job::reserve_registration(&mut state.dependents)?;
                let index = state.dependents.len();
                state.dependents.push(Some(Arc::downgrade(&dependency)));
                index
            }
        };
        dependency.deliver(self.status());
        Ok(DependencyGuard {
            source: Arc::downgrade(self),
            index,
            dependency,
        })
    }
}
impl Drop for Signal {
    fn drop(&mut self) {
        if let Some(memberships) = self.memberships.get() {
            let mut unwind = UnwindCleanup::default();
            unwind.run(|| {
                self.notification_domain()
                    .unregister(memberships.ordinary, self)
            });
            unwind.run(|| {
                self.module_notification_domain()
                    .unregister(memberships.module, self)
            });
            unwind.finish();
        }
    }
}
pub(crate) struct SettlementWait {
    signal: Arc<Signal>,
    index: usize,
    waiter: Option<Arc<Waiter>>,
}
impl SettlementWait {
    pub(crate) fn poll(
        &mut self,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Result<SettlementStatus, JobFault>> {
        let status = self.signal.status();
        if status != Ok(SettlementStatus::Pending) {
            return Poll::Ready(status);
        }
        let Some(waiter) = &self.waiter else {
            let error = signal_fault(JobFaultCause::StateInconsistency);
            self.signal.fail(error);
            return Poll::Ready(self.signal.status());
        };
        let new_waker = cx.waker().clone();
        let replacement = {
            let state = match self.signal.state_access() {
                Ok(state) => state,
                Err(error) => {
                    drop(new_waker);
                    return Poll::Ready(Err(error));
                }
            };
            match state.waiters.get(self.index) {
                Some(Some(slot)) if Arc::ptr_eq(slot, waiter) => {}
                Some(_) => {
                    drop(state);
                    let error =
                        signal_fault(JobFaultCause::InvalidSlotIdentity { index: self.index });
                    self.signal.reject_waker(error, new_waker);
                    return Poll::Ready(self.signal.status());
                }
                None => {
                    drop(state);
                    let error = signal_fault(JobFaultCause::InvalidIndex { index: self.index });
                    self.signal.reject_waker(error, new_waker);
                    return Poll::Ready(self.signal.status());
                }
            }
            // Keep only the checked registration identity, not a registry guard,
            // across waker-slot access or disposal.
            drop(state);
            match waiter.waker.lock() {
                Ok(mut slot) => Ok(slot.replace(new_waker)),
                Err(poison) => {
                    drop(poison.into_inner());
                    Err(new_waker)
                }
            }
        };
        match replacement {
            Ok(old) => drop(old),
            Err(new_waker) => {
                self.signal
                    .reject_waker(signal_fault(JobFaultCause::PoisonedWaker), new_waker);
            }
        }
        let status = self.signal.status();
        if status == Ok(SettlementStatus::Pending) {
            Poll::Pending
        } else {
            Poll::Ready(status)
        }
    }
}
impl Drop for SettlementWait {
    fn drop(&mut self) {
        let Some(waiter) = &self.waiter else {
            return;
        };
        if self.signal.fault.get().is_some() {
            return;
        }
        let removed = match self.signal.state_access() {
            Ok(mut state) => match state.waiters.get_mut(self.index) {
                Some(entry) if entry.as_ref().is_some_and(|slot| Arc::ptr_eq(slot, waiter)) => {
                    Ok(entry.take())
                }
                Some(_) => Err(signal_fault(JobFaultCause::InvalidSlotIdentity {
                    index: self.index,
                })),
                None => Err(signal_fault(JobFaultCause::InvalidIndex {
                    index: self.index,
                })),
            },
            Err(_) => return,
        };
        match removed {
            Ok(removed) => drop(removed),
            Err(error) => self.signal.fail(error),
        }
    }
}
struct Epoch {
    module_pending: AtomicBool,
    active: AtomicBool,
    pending: OnceLock<JobRetirement>,
    fault: OnceLock<JobFault>,
    completed: AtomicBool,
}
impl Epoch {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            module_pending: AtomicBool::new(false),
            active: AtomicBool::new(false),
            pending: OnceLock::new(),
            fault: OnceLock::new(),
            completed: AtomicBool::new(false),
        })
    }
}
struct Relay {
    target: Weak<Signal>,
    epoch: Arc<Epoch>,
}
impl RetirementRelay for Relay {
    fn retire(&self, reason: JobRetirement) {
        let reason = *self.epoch.pending.get_or_init(|| reason);
        if self.epoch.active.load(Ordering::Acquire)
            && let Some(target) = self.target.upgrade()
        {
            target.retire_epoch(&self.epoch, reason);
        }
    }
    fn fault(&self, error: JobFault) {
        let error = *self.epoch.fault.get_or_init(|| error);
        if self.epoch.active.load(Ordering::Acquire)
            && let Some(target) = self.target.upgrade()
        {
            target.fault_epoch(&self.epoch, error);
        }
    }
    fn completed(&self) {
        if let Some(target) = self.target.upgrade() {
            // Completion disables this epoch even if observation already failed.
            let state = match target.state.lock() {
                Ok(state) => state,
                Err(poison) => {
                    drop(poison.into_inner());
                    target.fail(signal_fault(JobFaultCause::PoisonedRegistry));
                    self.epoch.completed.store(true, Ordering::Release);
                    self.epoch.active.store(false, Ordering::Release);
                    return;
                }
            };
            self.epoch.completed.store(true, Ordering::Release);
            self.epoch.active.store(false, Ordering::Release);
            drop(state);
        }
    }
}
struct Dependency {
    target: Weak<Signal>,
    epoch: Arc<Epoch>,
}
impl Dependency {
    fn deliver(&self, observation: Result<SettlementStatus, JobFault>) {
        match observation {
            Ok(SettlementStatus::Retired(reason)) => self.retire(reason),
            Ok(SettlementStatus::ModuleInterrupted) => self.module_interrupt(),
            Err(error) => self.fault(error),
            _ => {}
        }
    }
    fn module_interrupt(&self) {
        self.epoch.module_pending.store(true, Ordering::Release);
        if let Some(target) = self.target.upgrade() {
            target.mark_module_epoch(&self.epoch);
        }
    }
    fn retire(&self, reason: JobRetirement) {
        let reason = *self.epoch.pending.get_or_init(|| reason);
        if self.epoch.active.load(Ordering::Acquire)
            && let Some(target) = self.target.upgrade()
        {
            target.retire_epoch(&self.epoch, reason);
        }
    }
    fn fault(&self, error: JobFault) {
        let error = *self.epoch.fault.get_or_init(|| error);
        if self.epoch.active.load(Ordering::Acquire)
            && let Some(target) = self.target.upgrade()
        {
            target.fault_epoch(&self.epoch, error);
        }
    }
}
struct DependencyGuard {
    source: Weak<Signal>,
    index: usize,
    dependency: Arc<Dependency>,
}
impl Drop for DependencyGuard {
    fn drop(&mut self) {
        if let Some(source) = self.source.upgrade() {
            if source.fault.get().is_some() {
                return;
            }
            let error = match source.state_access() {
                Ok(mut state) => match state.dependents.get_mut(self.index) {
                    Some(entry)
                        if entry.as_ref().is_some_and(|entry| {
                            entry.ptr_eq(&Arc::downgrade(&self.dependency))
                        }) =>
                    {
                        *entry = None;
                        None
                    }
                    Some(_) => Some(signal_fault(JobFaultCause::InvalidSlotIdentity {
                        index: self.index,
                    })),
                    None => Some(signal_fault(JobFaultCause::InvalidIndex {
                        index: self.index,
                    })),
                },
                Err(_) => return,
            };
            if let Some(error) = error {
                source.fail(error);
            }
        }
    }
}
struct Binding {
    epoch: Arc<Epoch>,
    _job: Option<RetirementRegistration>,
    _dependency: Option<DependencyGuard>,
}
impl Drop for Binding {
    fn drop(&mut self) {
        self.epoch.active.store(false, Ordering::Release);
    }
}

pub(crate) struct PreparedHandoff {
    owner: Rooted<Association>,
    binding: Binding,
    dependency: Option<AssociationEdge>,
}
/// A committed handoff retains its outgoing custody until the caller releases
/// every object/heap borrow and completes the associated publication.
#[must_use = "notify or dispose the handoff after releasing publication borrows"]
pub(crate) struct HandoffNotification {
    owner: Rooted<Association>,
    previous: Option<Binding>,
    previous_dependency: Option<AssociationEdge>,
}
impl HandoffNotification {
    pub(crate) fn notify(self) {
        drop(self);
    }
}
impl Drop for HandoffNotification {
    fn drop(&mut self) {
        let previous = self.previous.take();
        let previous_dependency = self.previous_dependency.take();
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| drop(previous));
        cleanup.run(|| drop(previous_dependency));
        cleanup.run(|| self.owner.notify());
        cleanup.finish();
    }
}
impl PreparedHandoff {
    pub(crate) fn commit(self) -> HandoffNotification {
        let Self {
            owner,
            binding,
            dependency,
        } = self;
        if owner.module_cause().is_some() {
            owner.signal.mark_module();
        }
        // Commit can run under a caller's Promise borrow. Record any lock fault
        // without invoking cleanup there; the returned notification owns it.
        let state = match owner.signal.state.lock() {
            Ok(state) => state,
            Err(poison) => {
                drop(poison.into_inner());
                owner
                    .signal
                    .mark_fault(signal_fault(JobFaultCause::PoisonedRegistry));
                return HandoffNotification {
                    owner: owner.clone(),
                    previous: Some(binding),
                    previous_dependency: dependency,
                };
            }
        };
        let observation = owner.signal.observation();
        if owner.signal.fault.get().is_some()
            || observation == Ok(SettlementStatus::ModuleInterrupted)
        {
            drop(state);
            return HandoffNotification {
                owner: owner.clone(),
                previous: Some(binding),
                previous_dependency: dependency,
            };
        }
        // These cells contain private local ownership only. Their take/mutate/
        // restore interval has no callback, allocation or fallible boundary.
        let epoch = binding.epoch.clone();
        let mut bindings = owner.bindings.take();
        let previous = bindings.primary.replace(binding);
        if let Some(previous) = &previous {
            previous.epoch.active.store(false, Ordering::Release);
        }
        owner.bindings.set(bindings);
        let previous_dependency = owner.view.dependency.replace(dependency);
        let active = !epoch.completed.load(Ordering::Acquire);
        epoch.active.store(active, Ordering::Release);
        if active && epoch.module_pending.load(Ordering::Acquire) {
            owner.signal.mark_module();
        } else if active {
            if let Some(error) = epoch.fault.get() {
                owner.signal.mark_fault(*error);
            } else if let Some(reason) = epoch.pending.get() {
                owner
                    .signal
                    .publish_pending(SettlementStatus::Retired(*reason));
            }
        }
        drop(state);
        HandoffNotification {
            owner,
            previous,
            previous_dependency,
        }
    }
}

pub(crate) struct ResolutionLease(Option<Arc<Epoch>>);

// Only these two local traced edge facts may inhabit a LocalSlot. Their
// Clone operations cannot call the host, acquire the heap, or notify a Signal.
trait LocalFact: Clone + Trace {}
impl LocalFact for crate::module::lifecycle::EvaluationRef {}
impl LocalFact for AssociationEdge {}
struct LocalSlot<T: LocalFact>(Cell<Option<T>>);
impl<T: LocalFact> Default for LocalSlot<T> {
    fn default() -> Self {
        Self(Cell::new(None))
    }
}
impl<T: LocalFact> LocalSlot<T> {
    fn get(&self) -> Option<T> {
        let value = self.0.take();
        let copy = value.clone();
        self.0.set(value);
        copy
    }
    fn replace(&self, value: Option<T>) -> Option<T> {
        self.0.replace(value)
    }
    fn take(&self) -> Option<T> {
        self.0.take()
    }
}
impl<T: LocalFact> Trace for LocalSlot<T> {
    fn trace(
        &self,
        edges: &mut surgeist_js_gc::heap::EdgeVisitor<'_>,
    ) -> Result<(), surgeist_js_gc::heap::AccessError> {
        // Restore the retained facts before invoking the heap visitor.
        self.get().trace(edges)
    }
}
#[derive(Default, Trace)]
struct AssociationView {
    #[heap_trace(skip)]
    completes_origin: Cell<bool>,
    origin: LocalSlot<crate::module::lifecycle::EvaluationRef>,
    dependency: LocalSlot<AssociationEdge>,
}
#[derive(Trace)]
pub(crate) struct Association {
    #[heap_trace(skip)]
    signal: Arc<Signal>,
    #[heap_trace(skip)]
    bindings: Cell<Bindings>,
    view: Rc<AssociationView>,
}
#[derive(Default)]
struct Bindings {
    primary: Option<Binding>,
    required: Vec<Binding>,
}
impl std::fmt::Debug for Association {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NativeSettlementAssociation")
            .field(&self.signal)
            .finish()
    }
}
#[derive(Clone, Debug, Trace)]
pub(crate) struct AssociationEdge(
    RecordRef<Association>,
    #[heap_trace(skip)] LocalWeak<AssociationView>,
    #[heap_trace(skip)] Arc<Signal>,
);
impl AssociationEdge {
    pub(crate) fn root(&self) -> JsResult<Rooted<Association>> {
        match self.0.upgrade() {
            Ok(Some(owner)) => Ok(owner),
            Ok(None) => {
                let error = signal_fault(JobFaultCause::OwnerMismatch);
                self.2.fail(error);
                Err(engine_fault(error))
            }
            Err(_) => {
                let error = signal_fault(JobFaultCause::InaccessibleRegistry);
                self.2.fail(error);
                Err(engine_fault(error))
            }
        }
    }
}
impl Association {
    pub(crate) fn check(&self) -> JsResult<()> {
        self.signal.check().map_err(crate::JsError::from)
    }
    pub(crate) fn attach_module(&self, evaluation: crate::module::lifecycle::EvaluationRef) {
        self.view.completes_origin.set(true);
        let previous = self.view.origin.replace(Some(evaluation));
        drop(previous);
    }
    pub(crate) fn bind_module(&self, evaluation: crate::module::lifecycle::EvaluationRef) {
        if self.view.origin.get().is_none() {
            let previous = self.view.origin.replace(Some(evaluation));
            drop(previous);
        }
    }
    pub(crate) fn observe_module(&self, evaluation: crate::module::lifecycle::EvaluationRef) {
        self.bind_module(evaluation);
        if self.module_cause().is_some() {
            self.signal.mark_module();
            self.signal.notify_module();
        }
    }
    fn next_view(view: &Rc<AssociationView>) -> Option<(Rc<AssociationView>, Arc<Signal>)> {
        let next = view.dependency.get()?;
        Some((next.1.upgrade()?, next.2.clone()))
    }
    pub(crate) fn module_cause(&self) -> Option<Rc<crate::error::RetainedInterruption>> {
        let mut cursor = (self.view.clone(), self.signal.clone());
        let mut slow = Some(self.view.clone());
        let mut fast = Some(self.view.clone());
        let mut collision = None;
        loop {
            if cursor.1.observation() == Ok(SettlementStatus::Settled) {
                return None;
            }
            if let Some(origin) = cursor.0.origin.get()
                && let Some(cause) = origin.cause()
            {
                return Some(cause);
            }
            cursor = Self::next_view(&cursor.0)?;
            if let Some(collision) = &collision {
                if Rc::ptr_eq(&cursor.0, collision) {
                    return None;
                }
                continue;
            }
            slow = slow.and_then(|view| Self::next_view(&view).map(|next| next.0));
            fast = fast
                .and_then(|view| Self::next_view(&view))
                .and_then(|next| Self::next_view(&next.0))
                .map(|next| next.0);
            if let (Some(slow), Some(fast)) = (&slow, &fast)
                && Rc::ptr_eq(slow, fast)
            {
                collision = Some(cursor.0.clone());
            }
        }
    }
    pub(crate) fn module_error(owner: &Rooted<Self>) -> Option<crate::JsError> {
        owner.module_cause().map(|cause| {
            crate::error::EngineError::ModuleInterrupted(
                crate::error::ModuleInterruption::association(cause, owner.clone()),
            )
            .into()
        })
    }
    pub(crate) fn new(context: &Context) -> JsResult<Rooted<Self>> {
        context
            .runtime()
            .heap
            .allocate(Self {
                signal: Signal::new().map_err(crate::JsError::from)?,
                bindings: Cell::new(Bindings {
                    primary: None,
                    required: Vec::new(),
                }),
                view: Rc::new(AssociationView::default()),
            })
            .map_err(|error| {
                PanicError::new(format!(
                    "native settlement association publication: {}",
                    error.reason
                ))
                .into()
            })
    }
    pub(crate) fn edge(owner: &Rooted<Self>) -> AssociationEdge {
        AssociationEdge(
            owner.record_ref(),
            Rc::downgrade(&owner.view),
            owner.signal.clone(),
        )
    }
    pub(crate) fn signal(&self) -> Arc<Signal> {
        self.signal.clone()
    }
    pub(crate) fn prepare_handoff(
        owner: &Rooted<Self>,
        observer: &JobObserver,
    ) -> JsResult<PreparedHandoff> {
        owner.check()?;
        if let Some(error) = observer.fault() {
            return Err(error.into());
        }
        let epoch = Epoch::new();
        let relay: Arc<dyn RetirementRelay> = Arc::new(Relay {
            target: Arc::downgrade(&owner.signal),
            epoch: epoch.clone(),
        });
        let registration = observer.register_retirement(relay)?;
        Ok(PreparedHandoff {
            owner: owner.clone(),
            binding: Binding {
                epoch,
                _job: Some(registration),
                _dependency: None,
            },
            dependency: None,
        })
    }
    pub(crate) fn prepare_follow(
        owner: &Rooted<Self>,
        source: &Rooted<Self>,
    ) -> JsResult<PreparedHandoff> {
        owner.check()?;
        source.check()?;
        let epoch = Epoch::new();
        let subscription = source.signal.subscribe(&owner.signal, epoch.clone())?;
        Ok(PreparedHandoff {
            owner: owner.clone(),
            binding: Binding {
                epoch,
                _job: None,
                _dependency: Some(subscription),
            },
            dependency: Some(Self::edge(source)),
        })
    }
    pub(crate) fn resolution_lease(&self) -> JsResult<ResolutionLease> {
        if let Some(error) = self.signal.fault.get() {
            return Err(engine_fault(*error));
        }
        let bindings = self.bindings.take();
        let lease = ResolutionLease(
            bindings
                .primary
                .as_ref()
                .map(|binding| binding.epoch.clone()),
        );
        self.bindings.set(bindings);
        Ok(lease)
    }
    pub(crate) fn interrupt_resolution(&self, expected: ResolutionLease) {
        let state = match self.signal.state_access() {
            Ok(state) => state,
            Err(_) => return,
        };
        let bindings = self.bindings.take();
        let current = bindings.primary.as_ref().map(|binding| &binding.epoch);
        let same = match (current, expected.0.as_ref()) {
            (None, None) => true,
            (Some(current), Some(expected)) => Arc::ptr_eq(current, expected),
            _ => false,
        };
        let changed = same
            && self
                .signal
                .publish_pending(SettlementStatus::Retired(JobRetirement::Interrupted));
        self.bindings.set(bindings);
        drop(state);
        if changed {
            self.signal.notify();
        }
    }
    pub(crate) fn retire_pending(&self, reason: JobRetirement) {
        self.signal.retire_pending(reason);
    }
    pub(crate) fn notify(&self) {
        self.signal.notify();
    }
    pub(crate) fn bind_job(&self, observer: &JobObserver, replace: bool) -> JsResult<()> {
        self.check()?;
        if let Some(error) = observer.fault() {
            return Err(error.into());
        }
        if let Some(error) = self.signal.fault.get() {
            return Err(engine_fault(*error));
        }
        let epoch = Epoch::new();
        let relay: Arc<dyn RetirementRelay> = Arc::new(Relay {
            target: Arc::downgrade(&self.signal),
            epoch: epoch.clone(),
        });
        let registration = observer.register_retirement(relay)?;
        let binding = Binding {
            epoch: epoch.clone(),
            _job: Some(registration),
            _dependency: None,
        };
        self.commit(binding, replace)?;
        if replace && self.signal.status() != Ok(SettlementStatus::ModuleInterrupted) {
            let previous = self.view.dependency.take();
            drop(previous);
        }
        if let Some(error) = epoch.fault.get() {
            self.signal.fault_epoch(&epoch, *error);
        } else if let Some(reason) = epoch.pending.get().copied() {
            self.signal.retire_epoch(&epoch, reason);
        }
        if matches!(observer.status(), Ok(crate::job::JobStatus::Completed)) {
            epoch.active.store(false, Ordering::Release);
        }
        self.signal.notify();
        if let Some(error) = self.signal.fault.get() {
            return Err(engine_fault(*error));
        }
        Ok(())
    }
    fn commit(&self, binding: Binding, replace: bool) -> JsResult<()> {
        if let Some(error) = self.signal.fault.get() {
            return Err(engine_fault(*error));
        }
        let epoch = binding.epoch.clone();
        let mut bindings = self.bindings.take();
        if !replace
            && !bindings
                .required
                .iter()
                .any(|slot| !slot.epoch.active.load(Ordering::Acquire))
        {
            let reserved = reserve(&mut bindings.required);
            if let Err(error) = reserved {
                self.bindings.set(bindings);
                return Err(error);
            }
        }
        // Restore the complete owner before acquiring any fallible native lock.
        self.bindings.set(bindings);
        let state = self.signal.state_access().map_err(crate::JsError::from)?;
        let mut bindings = self.bindings.take();
        let old = if replace {
            bindings.primary.replace(binding)
        } else if let Some(slot) = bindings
            .required
            .iter_mut()
            .find(|slot| !slot.epoch.active.load(Ordering::Acquire))
        {
            Some(std::mem::replace(slot, binding))
        } else {
            bindings.required.push(binding);
            None
        };
        if let Some(old) = &old {
            old.epoch.active.store(false, Ordering::Release);
        }
        epoch
            .active
            .store(!epoch.completed.load(Ordering::Acquire), Ordering::Release);
        self.bindings.set(bindings);
        drop(state);
        drop(old);
        Ok(())
    }
    pub(crate) fn settle(&self) {
        if self.module_cause().is_some()
            || self.signal.observation() == Ok(SettlementStatus::ModuleInterrupted)
        {
            self.signal.mark_module();
            self.signal.notify_module();
            return;
        }
        let old_bindings = self.bindings.take();
        let old_dependency = self.view.dependency.take();
        if self.view.completes_origin.get()
            && let Some(origin) = self.view.origin.get()
        {
            origin.complete();
        }
        self.signal.settle();
        drop(old_bindings);
        drop(old_dependency);
    }
}

#[cfg(test)]
mod module_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::atomic::AtomicUsize, task::Wake};

    #[derive(Default)]
    struct WakeCount(AtomicUsize, Mutex<()>, std::sync::Condvar);
    impl WakeCount {
        fn signal(&self) {
            let _guard = lock(&self.1);
            self.0.fetch_add(1, Ordering::SeqCst);
            self.2.notify_all();
        }
        fn wait_for_fresh_wake(&self, previous: usize) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            let mut guard = lock(&self.1);
            while self.0.load(Ordering::SeqCst) <= previous {
                let now = std::time::Instant::now();
                assert!(
                    now < deadline,
                    "registered native waiter was not woken before its deadline; last count {}",
                    self.0.load(Ordering::SeqCst)
                );
                let (next, _) = self
                    .2
                    .wait_timeout(guard, deadline.saturating_duration_since(now))
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard = next;
            }
        }
    }
    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.signal();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.signal();
        }
    }
    fn pending(wait: &mut SettlementWait, count: &Arc<WakeCount>) {
        let waker = Waker::from(count.clone());
        assert_eq!(
            wait.poll(&mut TaskContext::from_waker(&waker)),
            Poll::Pending
        );
    }

    #[test]
    fn abandoned_successor_preparation_keeps_the_predecessor_authoritative() {
        let context = Context::default();
        let association = Association::new(&context).unwrap();
        let first = JobObserver::new();
        let successor = JobObserver::new();
        association.bind_job(&first, true).unwrap();
        let prepared = Association::prepare_handoff(&association, &successor).unwrap();
        successor.retire(JobRetirement::Dropped);
        assert_eq!(
            association.signal.status().unwrap(),
            SettlementStatus::Pending
        );
        drop(prepared);
        first.retire(JobRetirement::OwnerDropped);
        assert_eq!(
            association.signal.status().unwrap(),
            SettlementStatus::Retired(JobRetirement::OwnerDropped)
        );
    }

    #[test]
    fn committed_successor_retires_existing_waiters_and_ignores_the_old_epoch() {
        let context = Context::default();
        let association = Association::new(&context).unwrap();
        let first = JobObserver::new();
        let successor = JobObserver::new();
        association.bind_job(&first, true).unwrap();
        let count = Arc::new(WakeCount::default());
        let mut wait = association.signal.wait().unwrap();
        pending(&mut wait, &count);
        let notification = Association::prepare_handoff(&association, &successor)
            .unwrap()
            .commit();
        notification.notify();
        count.wait_for_fresh_wake(0);
        assert!(count.0.load(Ordering::SeqCst) > 0);
        pending(&mut wait, &count);
        count.0.store(0, Ordering::SeqCst);
        first.retire(JobRetirement::OwnerDropped);
        assert_eq!(
            association.signal.status().unwrap(),
            SettlementStatus::Pending
        );
        assert_eq!(count.0.load(Ordering::SeqCst), 0);
        successor.retire(JobRetirement::Cancelled);
        count.wait_for_fresh_wake(0);
        assert!(count.0.load(Ordering::SeqCst) > 0);
        let waker = Waker::from(count);
        assert_eq!(
            wait.poll(&mut TaskContext::from_waker(&waker)),
            Poll::Ready(Ok(SettlementStatus::Retired(JobRetirement::Cancelled)))
        );
    }

    #[test]
    fn completing_one_required_child_does_not_release_another_child() {
        let context = Context::default();
        let association = Association::new(&context).unwrap();
        let first = JobObserver::new();
        let second = JobObserver::new();
        association.bind_job(&first, false).unwrap();
        association.bind_job(&second, false).unwrap();
        first.complete();
        assert_eq!(
            association.signal.status().unwrap(),
            SettlementStatus::Pending
        );
        second.retire(JobRetirement::OwnerDropped);
        assert_eq!(
            association.signal.status().unwrap(),
            SettlementStatus::Retired(JobRetirement::OwnerDropped)
        );
        assert_eq!(first.status().unwrap(), crate::job::JobStatus::Completed);
    }

    struct CheckDescendant {
        last: Arc<Signal>,
        observed: AtomicBool,
    }
    impl Wake for CheckDescendant {
        fn wake(self: Arc<Self>) {
            assert_eq!(
                self.last.status().unwrap(),
                SettlementStatus::Retired(JobRetirement::Interrupted)
            );
            self.observed.store(true, Ordering::SeqCst);
        }
    }
    #[test]
    fn retirement_marks_the_complete_dependency_chain_before_waking() {
        let context = Context::default();
        let chain: Vec<_> = (0..1024)
            .map(|_| Association::new(&context).unwrap())
            .collect();
        for pair in chain.windows(2) {
            drop(
                Association::prepare_follow(&pair[1], &pair[0])
                    .unwrap()
                    .commit(),
            );
        }
        let wake = Arc::new(CheckDescendant {
            last: chain.last().unwrap().signal(),
            observed: AtomicBool::new(false),
        });
        let mut waiter = chain[0].signal.wait().unwrap();
        let waker = Waker::from(wake.clone());
        assert_eq!(
            waiter.poll(&mut TaskContext::from_waker(&waker)),
            Poll::Pending
        );
        chain[0].retire_pending(JobRetirement::Interrupted);
        assert!(wake.observed.load(Ordering::SeqCst));
        assert!(chain.iter().all(|value| value.signal.status().unwrap()
            == SettlementStatus::Retired(JobRetirement::Interrupted)));
    }

    #[test]
    fn local_dependency_cycles_are_collectable_even_with_native_signals_alive() {
        let context = Context::default();
        let left = Association::new(&context).unwrap();
        let right = Association::new(&context).unwrap();
        let left_edge = Association::edge(&left);
        let right_edge = Association::edge(&right);
        let signals = (left.signal(), right.signal());
        drop(Association::prepare_follow(&left, &right).unwrap().commit());
        drop(Association::prepare_follow(&right, &left).unwrap().commit());
        context.runtime().collect().unwrap();
        assert!(left_edge.0.is_live().unwrap());
        assert!(right_edge.0.is_live().unwrap());
        drop((left, right));
        context.runtime().collect().unwrap();
        assert!(!left_edge.0.is_live().unwrap());
        assert!(!right_edge.0.is_live().unwrap());
        assert_eq!(signals.0.status().unwrap(), SettlementStatus::Pending);
        assert_eq!(signals.1.status().unwrap(), SettlementStatus::Pending);
    }
    #[test]
    fn concurrent_retirement_wakes_both_registered_native_waiters() {
        use std::{sync::mpsc, time::Duration};
        let (reached_tx, reached_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let domain = Arc::new(Domain {
            queue: Mutex::new(NoticeQueue {
                head: None,
                tail: None,
                wake_head: None,
                wake_tail: None,
                driving: false,
            }),
            empty_notice_hook: Some(EmptyNoticeHook {
                visits: AtomicUsize::new(0),
                reached: reached_tx,
                resume: Mutex::new(resume_rx),
            }),
            ..Domain::new()
        });
        let first = Signal::in_isolated_domain(domain.clone()).unwrap();
        let second = Signal::in_isolated_domain(domain).unwrap();
        let first_count = Arc::new(WakeCount::default());
        let second_count = Arc::new(WakeCount::default());
        let mut first_wait = first.wait().unwrap();
        let mut second_wait = second.wait().unwrap();
        pending(&mut first_wait, &first_count);
        pending(&mut second_wait, &second_count);
        std::thread::scope(|threads| {
            let first_thread = threads.spawn(|| first.retire_pending(JobRetirement::Dropped));
            let second_signal = second.clone();
            let second_thread = threads.spawn(move || {
                reached_rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("first driver reaches its final empty-notice boundary");
                second_signal.retire_pending(JobRetirement::Dropped);
                resume_tx
                    .send(())
                    .expect("first driver waits for the concurrent retirement");
            });
            first_thread
                .join()
                .expect("first retirement finishes without timeout");
            second_thread
                .join()
                .expect("second retirement finishes without timeout");
        });
        // Retire the isolated failed-run queue itself after preserving the
        // observable wake counts; no test-owned notice cycle survives failure.
        let residue = {
            let mut queue = lock(&second.notification_domain().queue);
            (
                queue.head.take(),
                queue.tail.take(),
                queue.wake_head.take(),
                queue.wake_tail.take(),
            )
        };
        drop(residue);
        assert_eq!(
            first.status().unwrap(),
            SettlementStatus::Retired(JobRetirement::Dropped)
        );
        assert_eq!(
            second.status().unwrap(),
            SettlementStatus::Retired(JobRetirement::Dropped)
        );
        assert_eq!(first_count.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            second_count.0.load(Ordering::SeqCst),
            1,
            "a concurrently queued native notice must wake its preexisting waiter without a later unrelated retirement"
        );
    }
}

#[cfg(test)]
mod fault_tests;
