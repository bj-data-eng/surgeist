//! Admission of Rust-owned incoming callback capture, excluding the engine heap.

use std::cell::OnceCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use crate::conversion::CleanupPrimary;
use crate::owner::OwnerState;
use crate::{Error, Process, Value};

/// Selected callback arguments together with custody of their Rust allocation.
/// Moving out values keeps the original capacity admitted until this container
/// or its consuming iterator is dropped. Individual retained roots have their
/// own backing through backend reaping.
pub struct ArgumentValues<'scope> {
    pub(crate) values: Vec<Value<'scope>>,
    pub(crate) backing: Option<Guard>,
}

impl<'scope> ArgumentValues<'scope> {
    pub fn pop(&mut self) -> Option<Value<'scope>> {
        self.values.pop()
    }

    /// Removes an existing argument, preserving the remaining order.
    pub fn remove(&mut self, index: usize) -> Option<Value<'scope>> {
        (index < self.values.len()).then(|| self.values.remove(index))
    }
}

impl<'scope> std::ops::Deref for ArgumentValues<'scope> {
    type Target = [Value<'scope>];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

/// Consuming traversal that retains the original argument buffer's admission,
/// including after the last item is taken and until the iterator is dropped.
pub struct ArgumentValuesIntoIter<'scope> {
    values: std::vec::IntoIter<Value<'scope>>,
    _backing: Option<Guard>,
}

impl<'scope> IntoIterator for ArgumentValues<'scope> {
    type Item = Value<'scope>;
    type IntoIter = ArgumentValuesIntoIter<'scope>;

    fn into_iter(self) -> Self::IntoIter {
        ArgumentValuesIntoIter {
            values: self.values.into_iter(),
            _backing: self.backing,
        }
    }
}

impl<'scope> Iterator for ArgumentValuesIntoIter<'scope> {
    type Item = Value<'scope>;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.values.size_hint()
    }
}

impl ExactSizeIterator for ArgumentValuesIntoIter<'_> {}

/// One reached capture step. Each request represents one unit of capture work.
/// Bytes describe requested Rust bodies, excluding allocator metadata, tree-node
/// packing and engine storage. They include the caller's retention body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// A nonempty selected-argument buffer, before reserving its capacity.
    Buffer { bytes: usize },
    /// One receiver, selected argument or NewTarget root, before retaining it.
    RootRecord { bytes: usize },
}

impl Request {
    pub fn bytes(self) -> usize {
        match self {
            Self::Buffer { bytes } | Self::RootRecord { bytes } => bytes,
        }
    }

    fn with_retention(self, retention: usize) -> Result<Self, Error> {
        let bytes = self
            .bytes()
            .checked_add(retention)
            .ok_or(Error::Allocation)?;
        Ok(match self {
            Self::Buffer { .. } => Self::Buffer { bytes },
            Self::RootRecord { .. } => Self::RootRecord { bytes },
        })
    }
}

/// Caller-owned backing for an accepted capture step. Release is called once,
/// after the corresponding Rust buffer or backend root record is destroyed.
/// Implementations finish release before returning its failure, retain no
/// native loan, and follow ordinary Rust destructor behavior on panic.
pub trait Retention: 'static {
    fn release(self: Box<Self>) -> Result<(), Error>;
}

/// An allocation-free prepared caller policy. The caller owns its work tariffs,
/// Ledger composition, native quota and cancellation checks. Returning an error
/// refuses capture before materialization and is a terminal host refusal,
/// including an `Error::Exception`. Admission never produces a catchable throw.
pub struct Admission<F, T> {
    admit: F,
    retention: PhantomData<fn() -> T>,
}

impl<F, T> Admission<F, T>
where
    F: Fn(Request) -> Result<T, Error> + 'static,
    T: Retention,
{
    pub fn new(admit: F) -> Self {
        Self {
            admit,
            retention: PhantomData,
        }
    }

    /// Requested setup bodies, including the policy, supplied setup backing and
    /// preallocated first-cleanup slot. Admit before installing the policy.
    pub fn storage_bytes<B: 'static>(&self) -> Option<usize> {
        size_of::<Account>()
            .checked_add(size_of::<F>())?
            .checked_add(size_of::<StoredCleanup<B>>())
    }

    /// Installs once for every callable and constructor in this process,
    /// including functions created earlier. Setup backing follows the policy
    /// through accepted capture and surviving cleanup observations/reports.
    /// A duplicate install is refused before
    /// replacing the original policy. The returned observation owns no engine.
    pub fn install<B: 'static>(
        self,
        process: &mut Process,
        backing: B,
    ) -> Result<Observation, Error> {
        let owner = process.engine.owner_state();
        owner.available()?;
        if owner.capture.get().is_some() {
            return Err(Error::CallbackCaptureAlreadyAdmitted);
        }
        let cleanup: Rc<dyn CleanupRecord> = Rc::new(StoredCleanup {
            error: OnceCell::new(),
            _backing: backing,
        });
        let account = Rc::new(Account {
            owner: Rc::downgrade(&owner),
            admit: Box::new(move |request| {
                let retained = (self.admit)(request.with_retention(size_of::<T>())?)?;
                Ok(Box::new(retained) as Box<dyn Retention>)
            }),
            cleanup: cleanup.clone(),
        });
        owner
            .capture
            .set(account)
            .map_err(|_| Error::CallbackCaptureAlreadyAdmitted)?;
        Ok(Observation { cleanup })
    }
}

/// Read-only first-cleanup observation, usable after process destruction.
#[derive(Clone)]
pub struct Observation {
    cleanup: Rc<dyn CleanupRecord>,
}

impl Observation {
    /// `None` means no failed capture release has been recorded.
    pub fn cleanup_error(&self) -> Option<&Error> {
        self.cleanup.error()
    }
}

/// A terminal capture-release failure with its selected preceding guest cause.
/// The original cleanup error was stored in admitted setup storage. Reporting
/// or cloning these facts allocates nothing and performs no guest coercion.
#[derive(Clone, Debug)]
pub struct CleanupFailure {
    pub(crate) owner: u64,
    pub(crate) primary: Option<CleanupPrimary>,
    cleanup: Rc<dyn CleanupRecord>,
}

impl CleanupFailure {
    pub fn primary(&self) -> Option<&CleanupPrimary> {
        self.primary.as_ref()
    }

    pub fn cleanup_error(&self) -> Option<&Error> {
        self.cleanup.error()
    }

    pub(crate) fn same_fault(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.cleanup, &other.cleanup)
    }
}

impl std::fmt::Display for CleanupFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("incoming callback capture release failed")
    }
}

impl std::error::Error for CleanupFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cleanup_error()
            .map(|error| error as &dyn std::error::Error)
    }
}

type Admit = dyn Fn(Request) -> Result<Box<dyn Retention>, Error>;

trait CleanupRecord: std::fmt::Debug {
    fn error(&self) -> Option<&Error>;
    fn record(&self, error: Error) -> Result<(), Error>;
}

struct StoredCleanup<B> {
    error: OnceCell<Error>,
    _backing: B,
}

impl<B> std::fmt::Debug for StoredCleanup<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CaptureCleanup")
            .field("error", &self.error.get())
            .finish_non_exhaustive()
    }
}

impl<B> CleanupRecord for StoredCleanup<B> {
    fn error(&self) -> Option<&Error> {
        self.error.get()
    }
    fn record(&self, error: Error) -> Result<(), Error> {
        self.error.set(error)
    }
}

pub(crate) struct Account {
    owner: Weak<OwnerState>,
    admit: Box<Admit>,
    cleanup: Rc<dyn CleanupRecord>,
}

impl Account {
    pub(crate) fn admit(self: &Rc<Self>, request: Request) -> Result<Guard, Error> {
        let owner = self.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        owner.available()?;
        let retention = (self.admit)(request).map_err(|error| owner.fail(error))?;
        Ok(Guard {
            retention: Some(retention),
            account: self.clone(),
        })
    }
}

pub(crate) struct Guard {
    retention: Option<Box<dyn Retention>>,
    account: Rc<Account>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(retention) = self.retention.take()
            && let Err(error) = retention.release()
        {
            // The first slot cannot already refer to itself. Later faults are
            // discarded without replacing the immutable first cleanup cause.
            let _later = self.account.cleanup.record(error);
            if let Some(owner) = self.account.owner.upgrade() {
                owner.record_capture_cleanup(CleanupFailure {
                    owner: owner.id(),
                    primary: None,
                    cleanup: self.account.cleanup.clone(),
                });
            }
        }
    }
}
