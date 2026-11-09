//! Engine-independent native task disposition and notification.

use super::{
    JobFault, JobFaultCause, JobFaultOwner, JobRegistrationError, UnwindCleanup,
    reserve_registration,
};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    task::{Context, Poll, Waker},
};

/// Why owned work was retired without ordinary completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobRetirement {
    /// The executor explicitly cancelled this runtime's work.
    Cancelled,
    /// The exact Context required by this task was destroyed.
    OwnerDropped,
    /// Admitted work was interrupted by a host failure or native unwind.
    Interrupted,
    /// The last direct owner disposed unfinished work.
    Dropped,
}

/// Result-free native task lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobStatus {
    /// Work remains owned and viable.
    Pending,
    /// An ordinary result was delivered to its caller.
    Completed,
    /// Work was explicitly retired.
    Retired(JobRetirement),
}
impl JobStatus {
    const fn encode(self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Completed => 1,
            Self::Retired(JobRetirement::Cancelled) => 2,
            Self::Retired(JobRetirement::OwnerDropped) => 3,
            Self::Retired(JobRetirement::Interrupted) => 4,
            Self::Retired(JobRetirement::Dropped) => 5,
        }
    }
    const fn decode(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Completed),
            2 => Some(Self::Retired(JobRetirement::Cancelled)),
            3 => Some(Self::Retired(JobRetirement::OwnerDropped)),
            4 => Some(Self::Retired(JobRetirement::Interrupted)),
            5 => Some(Self::Retired(JobRetirement::Dropped)),
            _ => None,
        }
    }
}
const FAULTED: u8 = 6;

type Observation = Result<JobStatus, JobFault>;
trait StatusListener: Send + Sync {
    fn notify(&self, status: Observation);
}
pub(super) trait FaultWake: Send + Sync {
    fn faulted(&self, fault: JobFault);
}
enum Entry {
    Free(Option<usize>),
    Listener(Weak<dyn StatusListener>),
}
#[derive(Default)]
struct Listeners {
    entries: Vec<Entry>,
    free: Option<usize>,
}
struct ObserverState {
    status: AtomicU8,
    first_fault: OnceLock<JobFault>,
    listeners: Mutex<Listeners>,
    task_driver: OnceLock<Weak<dyn FaultWake>>,
    driver_fault_notified: AtomicBool,
}
impl ObserverState {
    fn latch_fault(&self, fault: JobFault) -> JobFault {
        let first = *self.first_fault.get_or_init(|| fault);
        loop {
            let raw = self.status.load(Ordering::Acquire);
            if raw == FAULTED
                || matches!(JobStatus::decode(raw), Some(status) if status != JobStatus::Pending)
            {
                return first;
            }
            if self
                .status
                .compare_exchange(raw, FAULTED, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return first;
            }
        }
    }
    fn observation(&self) -> (Observation, bool) {
        let raw = self.status.load(Ordering::Acquire);
        if let Some(status) = JobStatus::decode(raw) {
            return (Ok(status), false);
        }
        if raw == FAULTED
            && let Some(fault) = self.first_fault.get()
        {
            return (Err(*fault), false);
        }
        let cause = if raw == FAULTED {
            JobFaultCause::MissingCause
        } else {
            JobFaultCause::InvalidDisposition { raw }
        };
        let fault = self.latch_fault(JobFault::new(JobFaultOwner::Observer, cause));
        let status = JobStatus::decode(self.status.load(Ordering::Acquire));
        (status.ok_or(fault), true)
    }
    fn notify_terminal(&self) {
        let (entries, status) = {
            let mut list = match self.listeners.lock() {
                Ok(list) => list,
                Err(poisoned) => {
                    self.latch_fault(JobFault::new(
                        JobFaultOwner::Observer,
                        JobFaultCause::PoisonedRegistry,
                    ));
                    poisoned.into_inner()
                }
            };
            let status = self.observation().0;
            if status == Ok(JobStatus::Pending) {
                return;
            }
            list.free = None;
            (std::mem::take(&mut list.entries), status)
        };
        // The physical bank, not its free links, owns this complete cleanup set.
        let mut cleanup = UnwindCleanup::default();
        if let Some(fault) = self.first_fault.get().copied()
            && let Some(driver) = self.task_driver.get().and_then(Weak::upgrade)
            && !self.driver_fault_notified.swap(true, Ordering::AcqRel)
        {
            cleanup.run(move || driver.faulted(fault));
        }
        for entry in entries {
            if let Entry::Listener(listener) = entry
                && let Some(listener) = listener.upgrade()
            {
                cleanup.run(move || {
                    listener.notify(status);
                    drop(listener);
                });
            }
        }
        cleanup.finish();
    }
}

/// A thread-safe native observation handle containing no engine access or guest values.
#[derive(Clone)]
pub struct JobObserver {
    inner: Arc<ObserverState>,
}
impl fmt::Debug for JobObserver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JobObserver").field(&self.status()).finish()
    }
}
impl JobObserver {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(ObserverState {
                status: AtomicU8::new(0),
                first_fault: OnceLock::new(),
                listeners: Mutex::new(Listeners::default()),
                task_driver: OnceLock::new(),
                driver_fault_notified: AtomicBool::new(false),
            }),
        }
    }
    /// Reads the monotonic disposition or the fault that won its terminal transition.
    /// A fault found after normal completion remains available through [`Self::fault`].
    pub fn status(&self) -> Result<JobStatus, JobFault> {
        let (status, repaired) = self.inner.observation();
        if repaired {
            self.inner.notify_terminal();
        }
        status
    }
    /// Returns the first structural defect, including one found after normal completion.
    pub fn fault(&self) -> Option<JobFault> {
        self.inner.first_fault.get().copied()
    }
    // Atomic-only preparation for owners detaching a poisoned protected slot.
    // The caller completes record_fault after releasing that slot.
    pub(crate) fn latch_fault(&self, fault: JobFault) -> JobFault {
        self.inner.latch_fault(fault)
    }
    // One immutable, constructor-installed weak route to the actual task driver.
    pub(super) fn bind_task_driver(&self, driver: Weak<dyn FaultWake>) {
        if self.inner.task_driver.set(driver).is_err() {
            self.record_fault(JobFault::new(
                JobFaultOwner::Observer,
                JobFaultCause::StateInconsistency,
            ));
        } else if self.fault().is_some() {
            self.inner.notify_terminal();
        }
    }
    pub(crate) fn record_fault(&self, fault: JobFault) -> JobFault {
        let first = self.inner.latch_fault(fault);
        self.inner.notify_terminal();
        first
    }
    /// Reserves a rootless native waiter. Known terminal observation needs no registry slot.
    pub fn wait(&self) -> Result<JobWait, JobRegistrationError> {
        if let Some(fault) = self.fault() {
            return Err(fault.into());
        }
        let signal = Arc::new(WaitSignal {
            observer: self.clone(),
            waker: Mutex::new(None),
        });
        let registration = self.register(signal.clone())?;
        Ok(JobWait {
            observer: self.clone(),
            signal,
            _registration: registration,
        })
    }
    fn register(
        &self,
        listener: Arc<dyn StatusListener>,
    ) -> Result<Option<Registration>, JobRegistrationError> {
        if let Some(fault) = self.fault() {
            return Err(fault.into());
        }
        let status = self.status();
        if status != Ok(JobStatus::Pending) {
            if let Err(fault) = status {
                return Err(fault.into());
            }
            listener.notify(status);
            return Ok(None);
        }
        let mut list = match self.inner.listeners.lock() {
            Ok(list) => list,
            Err(poisoned) => {
                drop(poisoned.into_inner());
                let fault = self.record_fault(JobFault::new(
                    JobFaultOwner::Observer,
                    JobFaultCause::PoisonedRegistry,
                ));
                return Err(fault.into());
            }
        };
        if let Some(fault) = self.fault() {
            drop(list);
            self.record_fault(fault);
            return Err(fault.into());
        }
        let status = self.inner.observation().0;
        if status != Ok(JobStatus::Pending) {
            drop(list);
            match status {
                Err(fault) => {
                    self.record_fault(fault);
                    return Err(fault.into());
                }
                Ok(_) => {
                    listener.notify(status);
                    return Ok(None);
                }
            }
        }
        let candidate = if let Some(index) = list.free {
            match list.entries.get(index) {
                Some(Entry::Free(next))
                    if next.is_none_or(|next| {
                        next != index && matches!(list.entries.get(next), Some(Entry::Free(_)))
                    }) =>
                {
                    Ok((index, *next))
                }
                _ => Err(JobFaultCause::InvalidFreeLink { index }),
            }
        } else {
            reserve_registration(&mut list.entries)?;
            Ok((list.entries.len(), None))
        };
        let (index, next) = match candidate {
            Ok(candidate) => candidate,
            Err(cause) => {
                drop(list);
                return Err(self
                    .record_fault(JobFault::new(JobFaultOwner::Observer, cause))
                    .into());
            }
        };
        let expected = Arc::downgrade(&listener);
        if index == list.entries.len() {
            list.entries.push(Entry::Listener(expected.clone()));
        } else if let Some(entry) = list.entries.get_mut(index) {
            *entry = Entry::Listener(expected.clone());
        } else {
            drop(list);
            return Err(self
                .record_fault(JobFault::new(
                    JobFaultOwner::Observer,
                    JobFaultCause::InvalidIndex { index },
                ))
                .into());
        }
        list.free = next;
        drop(list);
        let registration = Registration {
            observer: Arc::downgrade(&self.inner),
            index,
            expected,
            _listener: listener.clone(),
        };
        let status = self.status();
        if status != Ok(JobStatus::Pending) {
            listener.notify(status);
        }
        if let Some(fault) = self.fault() {
            return Err(fault.into());
        }
        Ok(Some(registration))
    }
    pub(crate) fn complete(&self) {
        self.publish(JobStatus::Completed);
    }
    pub(crate) fn retire(&self, reason: JobRetirement) {
        self.publish(JobStatus::Retired(reason));
    }
    fn publish(&self, status: JobStatus) {
        if status == JobStatus::Pending {
            self.record_fault(JobFault::new(
                JobFaultOwner::Observer,
                JobFaultCause::StateInconsistency,
            ));
            return;
        }
        if self
            .inner
            .status
            .compare_exchange(0, status.encode(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.inner.notify_terminal();
        } else {
            // Malformed encodings and missing causes still require checked repair.
            let _ = self.status();
        }
    }
    pub(crate) fn register_retirement(
        &self,
        relay: Arc<dyn RetirementRelay>,
    ) -> Result<RetirementRegistration, JobRegistrationError> {
        Ok(RetirementRegistration {
            _registration: self.register(Arc::new(RelayListener {
                relay,
                delivered: AtomicBool::new(false),
            }))?,
        })
    }
}
struct Registration {
    observer: Weak<ObserverState>,
    index: usize,
    expected: Weak<dyn StatusListener>,
    _listener: Arc<dyn StatusListener>,
}
impl Drop for Registration {
    fn drop(&mut self) {
        let Some(inner) = self.observer.upgrade() else {
            return;
        };
        let observer = JobObserver { inner };
        if observer.status() != Ok(JobStatus::Pending) {
            return;
        }
        if let Some(fault) = observer.fault() {
            observer.record_fault(fault);
            return;
        }
        let mut list = match observer.inner.listeners.lock() {
            Ok(list) => list,
            Err(poisoned) => {
                drop(poisoned.into_inner());
                observer.record_fault(JobFault::new(
                    JobFaultOwner::Observer,
                    JobFaultCause::PoisonedRegistry,
                ));
                return;
            }
        };
        // Publication may have detached this registration while Drop waited.
        if observer.inner.observation().0 != Ok(JobStatus::Pending) {
            return;
        }
        if let Some(fault) = observer.fault() {
            drop(list);
            observer.record_fault(fault);
            return;
        }
        let cause = match list.entries.get(self.index) {
            Some(Entry::Listener(listener)) if Weak::ptr_eq(listener, &self.expected) => {
                match list.free {
                    Some(index)
                        if index == self.index
                            || !matches!(list.entries.get(index), Some(Entry::Free(_))) =>
                    {
                        Some(JobFaultCause::InvalidFreeLink { index })
                    }
                    _ => None,
                }
            }
            Some(_) => Some(JobFaultCause::InvalidSlotIdentity { index: self.index }),
            None => Some(JobFaultCause::InvalidIndex { index: self.index }),
        };
        if let Some(cause) = cause {
            drop(list);
            observer.record_fault(JobFault::new(JobFaultOwner::Observer, cause));
            return;
        }
        let next = list.free;
        if let Some(entry) = list.entries.get_mut(self.index) {
            *entry = Entry::Free(next);
            list.free = Some(self.index);
        } else {
            drop(list);
            observer.record_fault(JobFault::new(
                JobFaultOwner::Observer,
                JobFaultCause::InvalidIndex { index: self.index },
            ));
        }
    }
}

/// A rootless native outcome relay; no implementation may retain engine values.
pub(crate) trait RetirementRelay: Send + Sync {
    fn fault(&self, fault: JobFault);
    fn retire(&self, reason: JobRetirement);
    fn completed(&self) {}
}
struct RelayListener {
    relay: Arc<dyn RetirementRelay>,
    delivered: AtomicBool,
}
impl StatusListener for RelayListener {
    fn notify(&self, status: Observation) {
        if self.delivered.swap(true, Ordering::AcqRel) {
            return;
        }
        match status {
            Err(fault) => self.relay.fault(fault),
            Ok(JobStatus::Retired(reason)) => self.relay.retire(reason),
            Ok(JobStatus::Completed) => self.relay.completed(),
            Ok(JobStatus::Pending) => self.relay.fault(JobFault::new(
                JobFaultOwner::Observer,
                JobFaultCause::StateInconsistency,
            )),
        }
    }
}
/// Keeps a registered retirement relay alive until its association hands off.
pub(crate) struct RetirementRegistration {
    _registration: Option<Registration>,
}
struct WaitSignal {
    observer: JobObserver,
    waker: Mutex<Option<Waker>>,
}
impl StatusListener for WaitSignal {
    fn notify(&self, _: Observation) {
        let (waker, fault) = match self.waker.lock() {
            Ok(mut slot) => (slot.take(), None),
            Err(poisoned) => {
                let fault = self.observer.inner.latch_fault(JobFault::new(
                    JobFaultOwner::ObserverWaiter,
                    JobFaultCause::PoisonedWaker,
                ));
                (poisoned.into_inner().take(), Some(fault))
            }
        };
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            if let Some(fault) = fault {
                self.observer.record_fault(fault);
            }
        });
        cleanup.run(|| {
            if let Some(waker) = waker {
                waker.wake();
            }
        });
        cleanup.finish();
    }
}
/// A pre-registered, thread-safe native lifecycle waiter.
pub struct JobWait {
    observer: JobObserver,
    signal: Arc<WaitSignal>,
    _registration: Option<Registration>,
}
impl fmt::Debug for JobWait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobWait")
            .field("status", &self.observer.status())
            .finish_non_exhaustive()
    }
}
impl Future for JobWait {
    type Output = Result<JobStatus, JobFault>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = self.observer.status();
        if status != Ok(JobStatus::Pending) {
            return Poll::Ready(status);
        }
        let waker = cx.waker().clone();
        let (old, poisoned) = match self.signal.waker.lock() {
            Ok(mut slot) => (slot.replace(waker), false),
            Err(poisoned) => {
                self.observer.inner.latch_fault(JobFault::new(
                    JobFaultOwner::ObserverWaiter,
                    JobFaultCause::PoisonedWaker,
                ));
                let old = poisoned.into_inner().take();
                drop(waker);
                (old, true)
            }
        };
        if poisoned {
            let mut cleanup = UnwindCleanup::default();
            cleanup.run(|| self.observer.inner.notify_terminal());
            cleanup.run(|| {
                if let Some(old) = old {
                    old.wake();
                }
            });
            cleanup.finish();
        } else {
            drop(old);
        }
        let status = self.observer.status();
        if status == Ok(JobStatus::Pending) {
            Poll::Pending
        } else {
            Poll::Ready(status)
        }
    }
}

#[cfg(test)]
mod fault_tests;
