//! Admission and custody of binding-owned Rust preparation, excluding engine storage.

use std::cell::OnceCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use crate::conversion::CleanupPrimary;
use crate::owner::OwnerState;
use crate::{Error, Process, Value};

/// Selected callback arguments together with custody of their Rust allocation.
/// Moving out values keeps the original capacity admitted until this container
/// or its consuming iterator is dropped. Individual retained roots keep their
/// own backing until both their Rust identity and backend record retire.
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

/// A producer-owned value together with admission of its Rust allocation.
/// Borrowing exposes the value while its backing remains alive. An allocation
/// cannot be extracted from this container and outlive that backing.
pub struct Admitted<T> {
    pub(crate) value: T,
    pub(crate) backing: Option<Guard>,
}

impl<T> std::ops::Deref for Admitted<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Admitted<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(formatter)
    }
}

impl<T: PartialEq<U>, U> PartialEq<U> for Admitted<T> {
    fn eq(&self, other: &U) -> bool {
        self.value.eq(other)
    }
}

impl<T> Admitted<T> {
    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> Admitted<U> {
        Admitted {
            value: map(self.value),
            backing: self.backing,
        }
    }
}

pub(crate) fn buffer<T>(
    owner: &Rc<OwnerState>,
    kind: BufferKind,
    elements: usize,
) -> Result<Admitted<Vec<T>>, Error> {
    owner.available()?;
    let bytes = elements
        .checked_mul(size_of::<T>())
        .ok_or_else(|| owner.fail(Error::Allocation))?;
    let backing = if elements == 0 {
        None
    } else {
        owner
            .preparation
            .get()
            .map(|account| {
                account.admit(Request::Buffer {
                    kind,
                    bytes,
                    elements,
                })
            })
            .transpose()?
    };
    let mut value = Vec::new();
    value
        .try_reserve_exact(elements)
        .map_err(|_| owner.fail(Error::Allocation))?;
    Ok(Admitted { value, backing })
}

pub(crate) fn charge(owner: &Rc<OwnerState>, work: Work) -> Result<(), Error> {
    owner.available()?;
    if let Some(account) = owner.preparation.get() {
        account.charge(work)?;
    }
    Ok(())
}

/// One reached preparation allocation. Incoming requests preserve one unit of
/// incoming-capture work each; other work is reported separately through [`Work`].
/// Bytes describe requested Rust bodies, excluding allocator metadata, tree-node
/// packing and engine storage. They include the caller's retention body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// A nonempty selected-argument buffer, before reserving its capacity.
    IncomingBuffer { bytes: usize },
    /// One receiver, selected argument or NewTarget root, before retaining it.
    IncomingRoot { bytes: usize },
    /// One general scoped root record, before preparing its Rust custody.
    RootRecord { bytes: usize },
    /// One Rust buffer, before reserving its requested capacity.
    Buffer {
        kind: BufferKind,
        bytes: usize,
        elements: usize,
    },
    /// One backend callable registration, excluding the caller's callback body.
    CallableRegistration { bytes: usize },
    /// One native callback's panic-custody record, before host invocation.
    /// Normal records are released after native frames return. A panic keeps
    /// its record until owner teardown; the user's panic payload is excluded.
    CallbackFrame { bytes: usize },
    /// Process-owned Rust setup, before constructing its records.
    Process { bytes: usize },
    /// Realm-owned Rust setup, before constructing its records.
    Realm { bytes: usize },
}

/// The physical Rust buffer being prepared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferKind {
    /// Binding-owned copies of outbound value roots.
    OutboundRoots,
    /// Backend-owned copies of outbound local handles.
    OutboundLocals,
    /// An exact UTF-16 output buffer.
    Utf16,
    /// An exact IDL ByteString output buffer.
    Bytes,
}

/// Reached preparation work, reported before its traversal or payload copy.
/// Charging retains no backing and performs no producer allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// One element copied or visited in the selected physical buffer.
    BufferElement(BufferKind),
    /// One explicit payload pass over this many UTF-16 code units.
    CodeUnits { units: usize },
}

impl Request {
    pub fn bytes(self) -> usize {
        match self {
            Self::IncomingBuffer { bytes }
            | Self::IncomingRoot { bytes }
            | Self::RootRecord { bytes }
            | Self::Buffer { bytes, .. }
            | Self::CallableRegistration { bytes }
            | Self::CallbackFrame { bytes }
            | Self::Process { bytes }
            | Self::Realm { bytes } => bytes,
        }
    }

    fn with_retention(self, retention: usize) -> Result<Self, Error> {
        let bytes = self
            .bytes()
            .checked_add(retention)
            .ok_or(Error::Allocation)?;
        Ok(match self {
            Self::IncomingBuffer { .. } => Self::IncomingBuffer { bytes },
            Self::IncomingRoot { .. } => Self::IncomingRoot { bytes },
            Self::RootRecord { .. } => Self::RootRecord { bytes },
            Self::Buffer { kind, elements, .. } => Self::Buffer {
                kind,
                bytes,
                elements,
            },
            Self::CallableRegistration { .. } => Self::CallableRegistration { bytes },
            Self::CallbackFrame { .. } => Self::CallbackFrame { bytes },
            Self::Process { .. } => Self::Process { bytes },
            Self::Realm { .. } => Self::Realm { bytes },
        })
    }
}

/// Caller-owned backing for an accepted preparation step. Release is called once,
/// after the corresponding Rust bodies are destroyed. Root backing covers both
/// the public Rust identity and the backend record, including surviving tokens
/// after process destruction.
/// Implementations finish release before returning its failure, retain no
/// native loan or engine custody, and follow ordinary Rust destructor behavior
/// on panic.
pub trait Retention: 'static {
    fn release(self: Box<Self>) -> Result<(), Error>;
}

/// An allocation-free prepared caller policy. The caller owns its work tariffs,
/// Ledger composition, native quota and cancellation checks. Returning an error
/// refuses preparation before materialization and is a terminal host refusal,
/// including an `Error::Exception`. Admission never produces a catchable throw.
pub struct Admission<F, W, T> {
    admit: F,
    charge: W,
    retention: PhantomData<fn() -> T>,
}

impl<F, W, T> Admission<F, W, T>
where
    F: Fn(Request) -> Result<T, Error> + 'static,
    W: Fn(Work) -> Result<(), Error> + 'static,
    T: Retention,
{
    pub fn new(admit: F, charge: W) -> Self {
        Self {
            admit,
            charge,
            retention: PhantomData,
        }
    }

    /// Requested setup bodies, including the policy, supplied setup backing and
    /// preallocated first-cleanup slot. Admit before installing the policy.
    pub fn storage_bytes<B: 'static>(&self) -> Option<usize> {
        size_of::<Account>()
            .checked_add(size_of::<F>())?
            .checked_add(size_of::<W>())?
            .checked_add(size_of::<StoredCleanup<B>>())
    }

    /// Installs once for subsequent preparation in this process, including
    /// incoming capture by functions created earlier. This does not account for
    /// allocations reached before installation. Setup backing follows the
    /// policy and surviving controllers, cleanup observations and reports.
    /// Accepted backing independently follows its own Rust bodies; it does not
    /// keep the admission policy alive. A duplicate install preserves the first
    /// policy.
    /// The returned controller owns no engine.
    pub fn install<B: 'static>(
        self,
        process: &mut Process,
        backing: B,
    ) -> Result<Controller<T>, Error> {
        let owner = process.engine.owner_state();
        owner.available()?;
        if owner.preparation.get().is_some() {
            return Err(Error::PreparationAlreadyAdmitted);
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
            charge: Box::new(self.charge),
            cleanup,
        });
        owner
            .preparation
            .set(account.clone())
            .map_err(|_| Error::PreparationAlreadyAdmitted)?;
        Ok(Controller {
            account,
            retention: PhantomData,
        })
    }
}

/// Installed caller policy identity, without engine or native-loan custody.
/// Binding preparation APIs use this controller to accept owned backing from
/// the caller's current native context. The funding closure runs immediately;
/// it is never retained or called during guest entry.
pub struct Controller<T> {
    account: Rc<Account>,
    retention: PhantomData<fn() -> T>,
}

impl<T> Clone for Controller<T> {
    fn clone(&self) -> Self {
        Self {
            account: self.account.clone(),
            retention: PhantomData,
        }
    }
}

impl<T: Retention> Controller<T> {
    pub fn observation(&self) -> Observation {
        Observation {
            cleanup: self.account.cleanup.clone(),
        }
    }

    pub(crate) fn check_owner(&self, owner: &OwnerState) -> Result<(), Error> {
        owner.available()?;
        let account_owner = self.account.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        if !std::ptr::eq(account_owner.as_ref(), owner) {
            return Err(Error::ForeignOwner);
        }
        if !owner
            .preparation
            .get()
            .is_some_and(|installed| Rc::ptr_eq(installed, &self.account))
        {
            return Err(owner.fail(Error::RegistryConflict));
        }
        Ok(())
    }

    pub(crate) fn fund(
        &self,
        request: Request,
        fund: impl FnOnce(Request) -> Result<T, Error>,
    ) -> Result<Guard, Error> {
        let owner = self.account.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        owner.available()?;
        if !owner
            .preparation
            .get()
            .is_some_and(|installed| Rc::ptr_eq(installed, &self.account))
        {
            return Err(owner.fail(Error::RegistryConflict));
        }
        let retention = request
            .with_retention(size_of::<T>())
            .and_then(fund)
            .map_err(|error| owner.fail(error))?;
        Ok(Guard {
            retention: Some(Box::new(retention)),
            cleanup: Rc::downgrade(&self.account.cleanup),
            owner: self.account.owner.clone(),
        })
    }
}

/// Read-only first-cleanup observation, usable after process destruction.
#[derive(Clone)]
pub struct Observation {
    cleanup: Rc<dyn CleanupRecord>,
}

impl Observation {
    /// `None` means no failed preparation release has been recorded.
    pub fn cleanup_error(&self) -> Option<&Error> {
        self.cleanup.error()
    }
}

/// A terminal preparation-release failure with its selected preceding guest cause.
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
        formatter.write_str("binding preparation release failed")
    }
}

impl std::error::Error for CleanupFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cleanup_error()
            .map(|error| error as &dyn std::error::Error)
    }
}

type Admit = dyn Fn(Request) -> Result<Box<dyn Retention>, Error>;
type Charge = dyn Fn(Work) -> Result<(), Error>;

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
            .debug_struct("PreparationCleanup")
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
    charge: Box<Charge>,
    cleanup: Rc<dyn CleanupRecord>,
}

impl Account {
    pub(crate) fn admit(self: &Rc<Self>, request: Request) -> Result<Guard, Error> {
        let owner = self.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        owner.available()?;
        let retention = (self.admit)(request).map_err(|error| owner.fail(error))?;
        Ok(Guard {
            retention: Some(retention),
            cleanup: Rc::downgrade(&self.cleanup),
            owner: self.owner.clone(),
        })
    }

    pub(crate) fn charge(&self, work: Work) -> Result<(), Error> {
        let owner = self.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        owner.available()?;
        (self.charge)(work).map_err(|error| owner.fail(error))
    }
}

pub(crate) struct Guard {
    retention: Option<Box<dyn Retention>>,
    // Neither link retains policy closures or a root-bearing cleanup error.
    cleanup: Weak<dyn CleanupRecord>,
    owner: Weak<OwnerState>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(retention) = self.retention.take()
            && let Err(error) = retention.release()
        {
            // Release completes even after every cleanup consumer has retired.
            let Some(cleanup) = self.cleanup.upgrade() else {
                return;
            };
            // Later faults never replace the immutable first cleanup cause.
            let _later = cleanup.record(error);
            if let Some(owner) = self.owner.upgrade() {
                owner.record_preparation_cleanup(CleanupFailure {
                    owner: owner.id(),
                    primary: None,
                    cleanup,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    struct Retained {
        released: Rc<Cell<usize>>,
        failure: Option<Error>,
    }

    impl Retention for Retained {
        fn release(self: Box<Self>) -> Result<(), Error> {
            self.released.set(self.released.get() + 1);
            match self.failure {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }

    fn account(
        owner: &Rc<OwnerState>,
        charge: impl Fn(Work) -> Result<(), Error> + 'static,
    ) -> Rc<Account> {
        Rc::new(Account {
            owner: Rc::downgrade(owner),
            admit: Box::new(|_| Err(Error::BackendUnavailable)),
            charge: Box::new(charge),
            cleanup: Rc::new(StoredCleanup {
                error: OnceCell::new(),
                _backing: (),
            }),
        })
    }

    fn installed(
        charge: impl Fn(Work) -> Result<(), Error> + 'static,
    ) -> (Rc<OwnerState>, Controller<Retained>) {
        let owner = Rc::new(OwnerState::new(1));
        let account = account(&owner, charge);
        assert!(owner.preparation.set(account.clone()).is_ok());
        (
            owner,
            Controller {
                account,
                retention: PhantomData,
            },
        )
    }

    #[test]
    fn controlled_funding_preserves_request_metadata_and_retains_accepted_backing() {
        let (_owner, controller) = installed(|_| Ok(()));
        let released = Rc::new(Cell::new(0));
        let seen = Cell::new(None);
        let guard = controller
            .fund(
                Request::Buffer {
                    kind: BufferKind::OutboundLocals,
                    bytes: 19,
                    elements: 3,
                },
                |request| {
                    seen.set(Some(request));
                    Ok(Retained {
                        released: released.clone(),
                        failure: None,
                    })
                },
            )
            .unwrap();
        assert_eq!(
            seen.get(),
            Some(Request::Buffer {
                kind: BufferKind::OutboundLocals,
                bytes: 19 + size_of::<Retained>(),
                elements: 3,
            })
        );
        assert_eq!(released.get(), 0);
        drop(guard);
        assert_eq!(released.get(), 1);
    }

    #[test]
    fn controlled_funding_rejects_an_uninstalled_account_before_calling_the_funder() {
        let (owner, _installed) = installed(|_| Ok(()));
        let controller = Controller::<Retained> {
            account: account(&owner, |_| Ok(())),
            retention: PhantomData,
        };
        let called = Cell::new(false);
        let result = controller.fund(Request::RootRecord { bytes: 1 }, |_| {
            called.set(true);
            Err(Error::Allocation)
        });
        assert!(matches!(result, Err(Error::RegistryConflict)));
        assert!(!called.get());
        assert!(matches!(owner.available(), Err(Error::RegistryConflict)));
    }

    #[test]
    fn work_refusal_is_terminal_and_stops_later_work_before_its_callback() {
        let calls = Rc::new(Cell::new(0));
        let charged = calls.clone();
        let (_owner, controller) = installed(move |_| {
            charged.set(charged.get() + 1);
            Err(Error::Allocation)
        });
        assert!(matches!(
            controller
                .account
                .charge(Work::BufferElement(BufferKind::OutboundRoots)),
            Err(Error::Allocation)
        ));
        assert!(matches!(
            controller.account.charge(Work::CodeUnits { units: 7 }),
            Err(Error::Allocation)
        ));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn controlled_funding_refuses_overflow_before_calling_the_funder() {
        let (owner, controller) = installed(|_| Ok(()));
        let called = Cell::new(false);
        let result = controller.fund(Request::RootRecord { bytes: usize::MAX }, |_| {
            called.set(true);
            Err(Error::BackendUnavailable)
        });
        assert!(matches!(result, Err(Error::Allocation)));
        assert!(!called.get());
        assert!(matches!(owner.available(), Err(Error::Allocation)));
    }

    #[test]
    fn controlled_funding_preserves_an_existing_refusal_without_calling_the_funder() {
        let (owner, controller) = installed(|_| Ok(()));
        owner.fail(Error::RegistryConflict);
        let called = Cell::new(false);
        let result = controller.fund(Request::RootRecord { bytes: 1 }, |_| {
            called.set(true);
            Err(Error::Allocation)
        });
        assert!(matches!(result, Err(Error::RegistryConflict)));
        assert!(!called.get());
    }

    #[test]
    fn observation_preserves_first_release_failure_after_controller_and_owner_drop() {
        let (owner, controller) = installed(|_| Ok(()));
        let observation = controller.observation();
        let released = Rc::new(Cell::new(0));
        let first = controller
            .fund(Request::RootRecord { bytes: 1 }, |_| {
                Ok(Retained {
                    released: released.clone(),
                    failure: Some(Error::Allocation),
                })
            })
            .unwrap();
        let later = controller
            .fund(Request::RootRecord { bytes: 1 }, |_| {
                Ok(Retained {
                    released: released.clone(),
                    failure: Some(Error::RegistryConflict),
                })
            })
            .unwrap();
        drop(first);
        drop(later);
        drop(controller);
        drop(owner);
        assert_eq!(released.get(), 2);
        assert!(matches!(
            observation.cleanup_error(),
            Some(Error::Allocation)
        ));
    }

    #[test]
    fn accepted_backing_does_not_keep_policy_alive_and_reports_to_a_late_observer() {
        let (owner, controller) = installed(|_| Ok(()));
        let policy = Rc::downgrade(&controller.account);
        let observation = controller.observation();
        let released = Rc::new(Cell::new(0));
        let guard = controller
            .fund(Request::RootRecord { bytes: 1 }, |_| {
                Ok(Retained {
                    released: released.clone(),
                    failure: Some(Error::Allocation),
                })
            })
            .unwrap();
        drop(controller);
        drop(owner);
        assert!(policy.upgrade().is_none());
        assert!(observation.cleanup_error().is_none());
        drop(guard);
        assert_eq!(released.get(), 1);
        assert!(matches!(
            observation.cleanup_error(),
            Some(Error::Allocation)
        ));
    }

    #[test]
    fn accepted_backing_finishes_release_after_every_cleanup_consumer_retires() {
        let (owner, controller) = installed(|_| Ok(()));
        let cleanup = Rc::downgrade(&controller.account.cleanup);
        let released = Rc::new(Cell::new(0));
        let guard = controller
            .fund(Request::RootRecord { bytes: 1 }, |_| {
                Ok(Retained {
                    released: released.clone(),
                    failure: Some(Error::Allocation),
                })
            })
            .unwrap();
        drop(controller);
        drop(owner);
        assert!(cleanup.upgrade().is_none());
        drop(guard);
        assert_eq!(released.get(), 1);
    }
}
