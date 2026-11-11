use std::cell::Cell;
use std::rc::{Rc, Weak};

use surgeist_ledger::{
    Change, Limits, Operation, PhaseObservation, ScopeLedger, SharedOperation, Tariffs,
};

use crate::owner::OwnerState;
use crate::{Error, Scope};

use super::{CleanupFault, CleanupStage, Conversion, Failure};

surgeist_ledger::dimensions! {
    /// Binding-owned semantic work, excluding engine instructions/allocation.
    pub enum Work => WorkTable {
        /// One attempt to convert a JS value through a selected IDL type.
        Value,
        /// One payload code unit copied, validated, or normalized.
        CodeUnit,
        /// One lookup in a bounded declared enumeration.
        EnumerationLookup,
        /// One declared argument presence/default decision.
        Argument,
        /// One dictionary member visited, before reading guest properties.
        DictionaryMember,
        /// One iterator property read or invocation, before entering guest code.
        IteratorOperation,
        /// One yielded sequence item, before reading its value.
        SequenceItem,
        /// One previously accepted element moved to a growing Rust buffer.
        SequenceMove,
    }
}

surgeist_ledger::dimensions! {
    /// Requested payload storage, excluding allocator metadata and engine heaps.
    pub enum Resource => ResourceTable {
        /// Requested capacity of live Rust payload buffers in bytes.
        BufferBytes,
    }
}

pub type LedgerError = surgeist_ledger::Error<Work, Resource>;

/// Cleanup observations remain readable even after the execution owner dies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CleanupStatus {
    first: Option<CleanupFault>,
    failures: u64,
    overflowed: bool,
}

impl CleanupStatus {
    pub(crate) fn record(mut self, fault: CleanupFault) -> Self {
        self.first = self.first.or(Some(fault));
        match self.failures.checked_add(1) {
            Some(count) => self.failures = count,
            None => self.overflowed = true,
        }
        self
    }
    pub fn first(self) -> Option<CleanupFault> {
        self.first
    }
    /// `None` explicitly reports that the count exceeded its representation.
    pub fn failures(self) -> Option<u64> {
        (!self.overflowed).then_some(self.failures)
    }
}

pub(super) struct Account {
    pub(super) owner: Weak<OwnerState>,
    pub(super) ledger: SharedOperation<Work, Resource>,
    pub(super) scope: surgeist_ledger::Scope,
    cleanup: Cell<CleanupStatus>,
}

impl Account {
    pub(super) fn fault(&self, stage: CleanupStage, ledger: LedgerError) {
        let fault = match stage {
            CleanupStage::StorageRelease => CleanupFault::StorageRelease(ledger),
            CleanupStage::PhaseCompletion => CleanupFault::PhaseLedger(ledger),
        };
        self.record_fault(fault);
    }

    fn record_fault(&self, fault: CleanupFault) {
        self.cleanup.set(self.cleanup.get().record(fault));
        if let Some(owner) = self.owner.upgrade() {
            owner.record_cleanup(fault);
        }
    }

    pub(super) fn release(&self, bytes: u64) {
        if bytes != 0
            && let Err(error) = self
                .ledger
                .apply_batch(self.scope, &[Change::Release(Resource::BufferBytes, bytes)])
        {
            self.fault(CleanupStage::StorageRelease, error);
        }
    }

    fn end_phase(&self, marker: PhaseObservation) {
        if self.ledger.end_phase(marker).is_err() {
            // The marker and participant are exclusively constructed together.
            // A rejected marker is a typed ownership defect, never a panic.
            self.record_fault(CleanupFault::PhaseOwnerMismatch);
        } else if let Err(error) = self.ledger.usage(self.scope) {
            self.fault(CleanupStage::PhaseCompletion, error);
        }
    }
}

/// Cloneable binding participant. It retains Ledger custody, never the heap.
/// Limits and tariffs belong to its caller; native consumers keep their own
/// charging contracts. All Ledger locks end before guest code is entered.
#[derive(Clone)]
pub struct ConversionAccount(pub(super) Rc<Account>);

impl ConversionAccount {
    /// Requested allocation body retained by the account's Rc.
    pub const fn retained_body_bytes() -> usize {
        size_of::<Account>()
    }

    /// Requested backing of a standalone conversion account, including its
    /// account body, shared Ledger body, and reserved scope storage.
    pub fn standalone_backing_bytes() -> Result<usize, Error> {
        SharedOperation::<Work, Resource>::reserved_backing_bytes(1)
            .and_then(|bytes| bytes.checked_add(Self::retained_body_bytes()))
            .ok_or(Error::Allocation)
    }

    /// Create a bounded conversion participant with no enclosing operation.
    /// Work is cumulative per dimension and resource limits bound live buffers.
    /// A host accounting for setup storage admits [`Self::standalone_backing_bytes`]
    /// through the lifetime of this account and its converted values.
    /// Use [`Self::new`] when binding work belongs to an enclosing Ledger.
    pub fn standalone(
        scope: &mut Scope<'_>,
        work: WorkTable<u64>,
        resources: ResourceTable<u64>,
    ) -> Result<Self, Error> {
        scope.check_active()?;
        let limits = Limits::new(work, resources);
        let ledger = Operation::with_reserved_scopes("js-conversion", limits, 1)
            .map_err(Error::ConversionLedger)?
            .share();
        let owner = scope.operations.owner_state();
        let participant = ledger.root();
        Ok(Self(Rc::new(Account {
            owner: Rc::downgrade(&owner),
            ledger,
            scope: participant,
            cleanup: Cell::new(CleanupStatus::default()),
        })))
    }

    pub fn new(
        scope: &mut Scope<'_>,
        mut ledger: SharedOperation<Work, Resource>,
        parent: surgeist_ledger::Scope,
        limits: Limits<Work, Resource>,
        tariffs: Tariffs<Work, Resource>,
    ) -> Result<Self, Error> {
        scope.check_active()?;
        let owner = scope.operations.owner_state();
        let child = ScopeLedger::child(&mut ledger, parent, "js-conversion", limits, tariffs)
            .map_err(|error| scope.operations.refuse(Error::ConversionLedger(error)))?;
        Ok(Self(Rc::new(Account {
            owner: Rc::downgrade(&owner),
            ledger,
            scope: child,
            cleanup: Cell::new(CleanupStatus::default()),
        })))
    }

    pub fn cleanup_status(&self) -> CleanupStatus {
        self.0.cleanup.get()
    }

    /// Describes the conversion result after its local storage and phase guard
    /// finish. The actual host/callback exit selects the outward result and
    /// also includes later local drops and post-SDK reaping.
    ///
    /// The borrowed conversion context cannot escape:
    /// ```compile_fail
    /// use surgeist_bindings_js::{Scope, conversion::ConversionAccount};
    /// fn escape(scope: &mut Scope<'_>, account: &ConversionAccount) {
    ///     let escaped = account.run(scope, |conversion| Ok(conversion));
    /// }
    /// ```
    pub fn run<'scope, T>(
        &self,
        scope: &mut Scope<'scope>,
        action: impl FnOnce(&mut Conversion<'_, 'scope>) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let owner = scope.operations.owner_state();
        owner.available().map_err(Failure)?;
        if !self.0.owner.ptr_eq(&Rc::downgrade(&owner)) {
            return Err(Failure(scope.operations.refuse(Error::ForeignOwner)));
        }
        scope.check_active().map_err(Failure)?;
        let marker =
            self.0.ledger.begin_phase(self.0.scope).map_err(|error| {
                Failure(scope.operations.refuse(Error::ConversionLedger(error)))
            })?;
        let mut phase = PhaseGuard {
            account: self.0.clone(),
            marker: Some(marker),
        };
        let result = action(&mut Conversion {
            scope,
            account: self.0.clone(),
        });
        phase.finish();
        owner
            .operation_result(result.map_err(Error::from))
            .map_err(Failure)
    }
}

struct PhaseGuard {
    account: Rc<Account>,
    marker: Option<PhaseObservation>,
}

impl PhaseGuard {
    fn finish(&mut self) {
        if let Some(marker) = self.marker.take() {
            self.account.end_phase(marker);
        }
    }
}

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        self.finish();
    }
}

pub(super) struct Storage {
    pub(super) account: Rc<Account>,
    pub(super) bytes: u64,
}

impl Drop for Storage {
    fn drop(&mut self) {
        self.account.release(self.bytes);
    }
}

/// Owns converted payload and its original resource charge together. The value
/// is dropped before release, including ordinary Rust unwinding. Host-made
/// copies from the borrowed value remain the host's work and custody.
/// The owned charge cannot be duplicated by cloning the wrapper:
/// ```compile_fail
/// use surgeist_bindings_js::{DomString, conversion::Converted};
/// fn duplicate(value: &Converted<DomString>) -> Converted<DomString> {
///     value.clone()
/// }
/// ```
pub struct Converted<T> {
    pub(super) value: T,
    pub(super) storage: Storage,
}

impl<T> Converted<T> {
    pub fn get(&self) -> &T {
        &self.value
    }
}
