//! Rootless job-owner failures shared by local and synchronized custody.

use std::fmt;
use thiserror::Error;

pub(crate) fn reserve_registration<T>(values: &mut Vec<T>) -> Result<(), JobResourceError> {
    values
        .len()
        .checked_add(1)
        .and_then(|count| count.checked_mul(std::mem::size_of::<T>()))
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(JobResourceError::SizeOverflow)?;
    values
        .try_reserve(1)
        .map_err(|_| JobResourceError::Allocation)
}

/// Finishes an already-owned notification/disposal set before resuming unwind.
#[derive(Default)]
pub(crate) struct UnwindCleanup {
    first: Option<Box<dyn std::any::Any + Send>>,
}
impl UnwindCleanup {
    fn discard_secondary(payload: Box<dyn std::any::Any + Send>) {
        if let Err(nested) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(payload)))
        {
            // Leak only this recursively panicking secondary host payload;
            // running its destructor again could replace the initiating unwind.
            std::mem::forget(nested);
        }
    }

    pub(crate) fn run(&mut self, cleanup: impl FnOnce()) {
        if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(cleanup)) {
            if self.first.is_none() {
                self.first = Some(payload);
            } else {
                Self::discard_secondary(payload);
            }
        }
    }
    pub(crate) fn finish(self) {
        if let Some(payload) = self.first {
            if std::thread::panicking() {
                // An outer native unwind already owns propagation. This cleanup
                // set must finish without replacing it with a secondary panic.
                Self::discard_secondary(payload);
            } else {
                std::panic::resume_unwind(payload);
            }
        }
    }

    /// Returns an owned result, or disposes it before resuming cleanup's unwind.
    /// During an existing outer unwind, secondary cleanup failure is discarded
    /// under the same policy as `finish`, and the result remains caller-owned.
    pub(crate) fn finish_with<T>(mut self, value: T) -> T {
        match self.first.take() {
            Some(original) if !std::thread::panicking() => {
                self.run(move || drop(value));
                self.resume(original)
            }
            Some(secondary) => {
                Self::discard_secondary(secondary);
                value
            }
            None => value,
        }
    }

    /// Resumes an initiating unwind already caught by the owning action.
    pub(crate) fn resume(self, original: Box<dyn std::any::Any + Send>) -> ! {
        if let Some(secondary) = self.first {
            Self::discard_secondary(secondary);
        }
        std::panic::resume_unwind(original)
    }
}

/// The owner that detected a job protocol invariant failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobFaultOwner {
    /// Native disposition and listener registration.
    Observer,
    /// A native observer's parked waiter.
    ObserverWaiter,
    /// Exact Context task liveness and registration.
    ContextLifetime,
    /// Owned asynchronous task state and action cleanup.
    Task,
    /// An asynchronous task's parked driver.
    TaskWake,
    /// Protected Context access custody.
    HostContext,
    /// Native promise settlement state and subscriptions.
    NativeSignal,
    /// A native notification domain and its membership.
    NativeDomain,
    /// Native dependency binding authority.
    NativeEpoch,
    /// A source-module evaluation's local custody.
    ModuleEvaluation,
    /// Resident job storage, links, payload custody and drain state.
    Queue,
    /// The executor's runtime queue registry.
    ExecutorRegistry,
}

/// A finite structural failure, with only rootless scalar diagnostic facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobFaultCause {
    /// A registry mutex was poisoned.
    PoisonedRegistry,
    /// A parked waker mutex was poisoned.
    PoisonedWaker,
    /// A notification work-link mutex was poisoned.
    PoisonedLink,
    /// Private registry state was already borrowed.
    InaccessibleRegistry,
    /// An index did not name an existing physical entry.
    InvalidIndex {
        /// The rejected index.
        index: usize,
    },
    /// A free-list link did not name the expected free entry.
    InvalidFreeLink {
        /// The rejected free-list index.
        index: usize,
    },
    /// A registration key did not match its original allocation.
    InvalidSlotIdentity {
        /// The mismatched physical slot.
        index: usize,
    },
    /// An atomic disposition was not a defined encoding.
    InvalidDisposition {
        /// The observed encoding.
        raw: u8,
    },
    /// Faulted disposition had no initialized first cause.
    MissingCause,
    /// The owning state transition was inconsistent.
    StateInconsistency,
    /// A cleanup counter had no matching acquisition.
    CounterUnderflow,
    /// A key or work item belonged to a different owner.
    OwnerMismatch,
    /// A resident queue link did not match its reciprocal link or bucket.
    InvalidQueueLink {
        /// The queue node whose link was rejected.
        index: usize,
    },
    /// A required existing clock bucket was absent.
    MissingClockBucket,
}

/// The first detected invariant failure of a job owner.
///
/// This Copy value owns no Context, engine value, callback, report or budget.
/// Only engine owners can construct it after detecting a protocol failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JobFault {
    owner: JobFaultOwner,
    cause: JobFaultCause,
}

impl JobFault {
    pub(crate) const fn new(owner: JobFaultOwner, cause: JobFaultCause) -> Self {
        Self { owner, cause }
    }

    /// Returns the owner that detected the invariant failure.
    pub const fn owner(self) -> JobFaultOwner {
        self.owner
    }

    /// Returns its typed structural cause and scalar facts.
    pub const fn cause(self) -> JobFaultCause {
        self.cause
    }
}

impl fmt::Display for JobFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.owner, self.cause)
    }
}

impl std::error::Error for JobFault {}

/// A registration request that cannot obtain its rootless storage.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum JobResourceError {
    /// The requested storage layout cannot be represented.
    #[error("job registration size cannot be represented")]
    SizeOverflow,
    /// A fallible storage reservation failed.
    #[error("could not allocate job registration storage")]
    Allocation,
}

/// Registration refusal without any engine access or guest payload.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum JobRegistrationError {
    /// The new registration could not reserve storage; existing custody is intact.
    #[error(transparent)]
    Resource(#[from] JobResourceError),
    /// The owner is quarantined by its first structural fault.
    #[error(transparent)]
    Fault(#[from] JobFault),
}

#[cfg(test)]
mod cleanup_tests;
