//! Engine-independent native task disposition and notification.

use crate::{JsResult, error::EngineResourceError};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex, Weak,
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
    fn encode(self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Completed => 1,
            Self::Retired(JobRetirement::Cancelled) => 2,
            Self::Retired(JobRetirement::OwnerDropped) => 3,
            Self::Retired(JobRetirement::Interrupted) => 4,
            Self::Retired(JobRetirement::Dropped) => 5,
        }
    }
    fn decode(value: u8) -> Self {
        match value {
            0 => Self::Pending,
            1 => Self::Completed,
            2 => Self::Retired(JobRetirement::Cancelled),
            3 => Self::Retired(JobRetirement::OwnerDropped),
            4 => Self::Retired(JobRetirement::Interrupted),
            5 => Self::Retired(JobRetirement::Dropped),
            _ => unreachable!("valid job status"),
        }
    }
}

trait StatusListener: Send + Sync {
    fn notify(&self, status: JobStatus);
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
    listeners: Mutex<Listeners>,
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
                listeners: Mutex::new(Listeners::default()),
            }),
        }
    }
    /// Reads this task's monotonic disposition without allocation.
    pub fn status(&self) -> JobStatus {
        JobStatus::decode(self.inner.status.load(Ordering::Acquire))
    }
    /// Reserves a native waiter before returning its asynchronous observation.
    pub fn wait(&self) -> JsResult<JobWait> {
        let signal = Arc::new(WaitSignal {
            waker: Mutex::new(None),
        });
        let registration = self.register(signal.clone())?;
        Ok(JobWait {
            observer: self.clone(),
            signal,
            _registration: registration,
        })
    }
    fn register(&self, listener: Arc<dyn StatusListener>) -> JsResult<Registration> {
        let index = {
            let mut list = self
                .inner
                .listeners
                .lock()
                .expect("observer mutex poisoned");
            if let Some(index) = list.free {
                let Entry::Free(next) = list.entries[index] else {
                    unreachable!("free listener slot")
                };
                list.free = next;
                list.entries[index] = Entry::Listener(Arc::downgrade(&listener));
                index
            } else {
                list.entries
                    .try_reserve(1)
                    .map_err(|_| EngineResourceError::Allocation)?;
                let index = list.entries.len();
                list.entries
                    .push(Entry::Listener(Arc::downgrade(&listener)));
                index
            }
        };
        let registration = Registration {
            observer: Arc::downgrade(&self.inner),
            index,
            _listener: listener.clone(),
        };
        let status = self.status();
        if status != JobStatus::Pending {
            listener.notify(status);
        }
        Ok(registration)
    }
    pub(crate) fn complete(&self) {
        self.publish(JobStatus::Completed);
    }
    pub(crate) fn retire(&self, reason: JobRetirement) {
        self.publish(JobStatus::Retired(reason));
    }
    fn publish(&self, status: JobStatus) {
        if self
            .inner
            .status
            .compare_exchange(0, status.encode(), Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let len = self
            .inner
            .listeners
            .lock()
            .expect("observer mutex poisoned")
            .entries
            .len();
        for index in 0..len {
            let listener = {
                let list = self
                    .inner
                    .listeners
                    .lock()
                    .expect("observer mutex poisoned");
                match &list.entries[index] {
                    Entry::Listener(listener) => listener.upgrade(),
                    Entry::Free(_) => None,
                }
            };
            if let Some(listener) = listener {
                listener.notify(status);
            }
        }
    }
    pub(crate) fn register_retirement(
        &self,
        relay: Arc<dyn RetirementRelay>,
    ) -> JsResult<RetirementRegistration> {
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
    _listener: Arc<dyn StatusListener>,
}
impl Drop for Registration {
    fn drop(&mut self) {
        if let Some(observer) = self.observer.upgrade() {
            let mut list = observer.listeners.lock().expect("observer mutex poisoned");
            let next = list.free;
            list.entries[self.index] = Entry::Free(next);
            list.free = Some(self.index);
        }
    }
}

/// Private status-only relay; implementations must not retain local engine state.
pub(crate) trait RetirementRelay: Send + Sync {
    fn retire(&self, reason: JobRetirement);
    fn completed(&self) {}
}
struct RelayListener {
    relay: Arc<dyn RetirementRelay>,
    delivered: AtomicBool,
}
impl StatusListener for RelayListener {
    fn notify(&self, status: JobStatus) {
        if self.delivered.swap(true, Ordering::AcqRel) {
            return;
        }
        match status {
            JobStatus::Retired(reason) => self.relay.retire(reason),
            JobStatus::Completed => self.relay.completed(),
            JobStatus::Pending => unreachable!("only terminal status is notified"),
        }
    }
}
/// Keeps one pre-registered retirement relay alive until its association hands off.
pub(crate) struct RetirementRegistration {
    _registration: Registration,
}

struct WaitSignal {
    waker: Mutex<Option<Waker>>,
}
impl StatusListener for WaitSignal {
    fn notify(&self, _: JobStatus) {
        let waker = self.waker.lock().expect("waiter mutex poisoned").take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}
/// A registered, thread-safe native lifecycle waiter.
pub struct JobWait {
    observer: JobObserver,
    signal: Arc<WaitSignal>,
    _registration: Registration,
}
impl fmt::Debug for JobWait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobWait")
            .field("status", &self.observer.status())
            .finish_non_exhaustive()
    }
}
impl Future for JobWait {
    type Output = JobStatus;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = self.observer.status();
        if status != JobStatus::Pending {
            return Poll::Ready(status);
        }
        let waker = cx.waker().clone();
        let old = self
            .signal
            .waker
            .lock()
            .expect("waiter mutex poisoned")
            .replace(waker);
        drop(old);
        let status = self.observer.status();
        if status == JobStatus::Pending {
            Poll::Pending
        } else {
            Poll::Ready(status)
        }
    }
}
