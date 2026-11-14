//! Adapter admission and observation for the original attached guest owner.

use super::*;
use crate::document_scripting::preparation::CapReservation;
use surgeist_ledger::{
    Change, Limits, Operation, ProfileMode, Scope as LedgerScope, SharedOperation, Tariffs,
};

surgeist_ledger::dimensions! {
    /// Work owned by the Runtime guest adapter.
    pub enum DocumentGuestWork => WorkTable {
        /// One admitted unit at an existing Runtime guest admission point.
        Adapter,
    }
}

surgeist_ledger::dimensions! {
    /// Live Rust backing owned by the Runtime guest adapter.
    pub enum DocumentGuestResource => ResourceTable {
        /// Requested bytes retained by the adapter.
        BackingBytes,
    }
}

type Ledger = SharedOperation<DocumentGuestWork, DocumentGuestResource>;
pub(in crate::document_scripting) type AccountingError =
    surgeist_ledger::Error<DocumentGuestWork, DocumentGuestResource>;
type Usage = surgeist_ledger::Usage<DocumentGuestWork, DocumentGuestResource>;

/// An observation of one attachment's Runtime guest admission.
pub type DocumentGuestAccountingReport =
    surgeist_ledger::Report<DocumentGuestWork, DocumentGuestResource>;

/// Read-only observation custody for one attachment. This holds no execution
/// authority and can outlive the attachment to observe release.
#[derive(Clone)]
pub struct DocumentGuestAccounting(Rc<Accounting>);

impl DocumentGuestAccounting {
    /// Read accepted usage and the configured observations. Profiling off
    /// returns `None`. A snapshot does not change the admission state.
    pub fn snapshot(&self) -> Result<Option<DocumentGuestAccountingReport>, AccountingError> {
        if let Some(error) = self.0.cleanup_error.get() {
            return Err(error);
        }
        self.0.ledger.snapshot()
    }
}

pub(in crate::document_scripting) struct Accounting {
    ledger: Ledger,
    root: LedgerScope,
    projection: LedgerScope,
    preparation_scope: LedgerScope,
    profiling: bool,
    reserved_storage: usize,
    cleanup_error: Cell<Option<AccountingError>>,
    #[cfg(test)]
    measured_peak: Cell<Option<usize>>,
    // The original native cap covers every surviving guest body and report.
    preparation_allowance: OnceCell<CapReservation>,
    _backing: DomHostedRuntimeRetention,
}

#[derive(Clone, Copy)]
pub(in crate::document_scripting) enum AccountScope {
    Adapter,
    Projection,
    Preparation,
}

#[derive(Clone)]
pub(in crate::document_scripting) struct GuestBudget(Rc<Accounting>);

impl GuestBudget {
    const SCOPES: usize = 3;

    pub(in crate::document_scripting) fn setup_bytes(
        mode: ProfileMode,
    ) -> Result<usize, DocumentScriptNativeError> {
        Ledger::profiled_backing_bytes(Self::SCOPES, mode)
            .and_then(|bytes| bytes.checked_add(size_of::<Accounting>()))
            .ok_or(DocumentScriptNativeError::StorageLimit)
    }

    pub(in crate::document_scripting) fn new(
        limits: DocumentGuestEventLimits,
        mode: ProfileMode,
        backing: DomHostedRuntimeRetention,
    ) -> Result<Self, DocumentGuestEventError> {
        let bytes = u64::try_from(limits.storage_bytes)
            .map_err(|_| DocumentScriptNativeError::StorageLimit)?;
        let mut operation = Operation::with_reserved_profile(
            "runtime-guest",
            Limits::new(WorkTable::splat(limits.work), ResourceTable::splat(bytes)),
            Self::SCOPES,
            mode,
        )?;
        let root = operation.root();
        let projection = operation.child(
            root,
            "node-projection",
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        let preparation_scope = operation.child(
            root,
            "binding-preparation",
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        Ok(Self(Rc::new(Accounting {
            ledger: operation.share(),
            root,
            projection,
            preparation_scope,
            profiling: mode != ProfileMode::Off,
            reserved_storage: limits.storage_bytes,
            cleanup_error: Cell::new(None),
            #[cfg(test)]
            measured_peak: Cell::new(None),
            preparation_allowance: OnceCell::new(),
            _backing: backing,
        })))
    }

    pub(in crate::document_scripting) fn reserved_storage_bytes(
        &self,
    ) -> Result<usize, DocumentGuestEventError> {
        self.0
            .reserved_storage
            .checked_add(self.0._backing.retained_bytes())
            .ok_or_else(|| DocumentScriptNativeError::StorageLimit.into())
    }

    pub(in crate::document_scripting) fn observation(&self) -> DocumentGuestAccounting {
        DocumentGuestAccounting(self.0.clone())
    }

    pub(in crate::document_scripting) fn attach_preparation_allowance(
        &self,
        allowance: CapReservation,
    ) -> Result<(), DocumentGuestEventError> {
        self.0
            .preparation_allowance
            .set(allowance)
            .map_err(|_| DocumentGuestEventError::RegistryState)
    }

    pub(in crate::document_scripting) fn phase<T>(
        &self,
        run: impl FnOnce() -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        if !self.0.profiling {
            return run();
        }
        let result = self.0.ledger.with_phase(self.0.projection, run)?;
        match result {
            Ok(value) => {
                self.0.usage()?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::document_scripting) fn charge_work(
        &self,
        scope: AccountScope,
        units: u64,
    ) -> Result<(), DocumentGuestEventError> {
        self.0
            .apply(scope, Change::Work(DocumentGuestWork::Adapter, units))
    }

    pub(in crate::document_scripting) fn check_work(
        &self,
        units: u64,
    ) -> Result<(), DocumentGuestEventError> {
        self.0.check(
            AccountScope::Adapter,
            Change::Work(DocumentGuestWork::Adapter, units),
        )
    }

    #[cfg(test)]
    pub(in crate::document_scripting) fn work(&self) -> Result<u64, DocumentGuestEventError> {
        Ok(self.0.usage()?.work(DocumentGuestWork::Adapter))
    }

    pub(in crate::document_scripting) fn reserve(
        &self,
        scope: AccountScope,
        owner: &NativeOwner,
        bytes: usize,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        self.reserve_with(scope, bytes, || {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .retain_script_payload(&mut session.dom(), bytes, &owner.cancellation)
                .map_err(DocumentScriptNativeError::from)
        })
    }

    pub(in crate::document_scripting) fn reserve_with(
        &self,
        scope: AccountScope,
        bytes: usize,
        retain: impl FnOnce() -> Result<DomHostedRuntimeRetention, DocumentScriptNativeError>,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        self.charge_work(scope, 1)?;
        let units = u64::try_from(bytes).map_err(|_| DocumentScriptNativeError::StorageLimit)?;
        let change = Change::Acquire(DocumentGuestResource::BackingBytes, units);
        self.0.check(scope, change)?;
        let canonical = retain()?;
        self.0.apply(scope, change)?;
        Ok(GuestBacking {
            account: self.0.clone(),
            scope,
            canonical,
        })
    }

    pub(in crate::document_scripting) fn adopt_fixed(
        &self,
        canonical: DomHostedRuntimeRetention,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        let units = u64::try_from(canonical.retained_bytes())
            .map_err(|_| DocumentScriptNativeError::StorageLimit)?;
        let scope = AccountScope::Adapter;
        self.0.apply(
            scope,
            Change::Acquire(DocumentGuestResource::BackingBytes, units),
        )?;
        Ok(GuestBacking {
            account: self.0.clone(),
            scope,
            canonical,
        })
    }

    #[cfg(test)]
    pub(in crate::document_scripting) fn used(&self) -> usize {
        usize::try_from(
            self.0
                .usage()
                .unwrap()
                .live(DocumentGuestResource::BackingBytes),
        )
        .unwrap()
    }

    #[cfg(test)]
    pub(in crate::document_scripting) fn peak_used(&self) -> usize {
        usize::try_from(
            self.0
                .usage()
                .unwrap()
                .peak(DocumentGuestResource::BackingBytes),
        )
        .unwrap()
    }

    /// Observe accepted live storage within this test operation, independently
    /// of earlier setup peaks. Nested windows contribute to their outer peak.
    #[cfg(test)]
    pub(in crate::document_scripting) fn measure_peak<T>(
        &self,
        run: impl FnOnce() -> T,
    ) -> (T, usize) {
        struct Window<'a> {
            account: &'a Accounting,
            previous: Option<usize>,
        }
        impl Drop for Window<'_> {
            fn drop(&mut self) {
                let peak = self.account.measured_peak.get().unwrap();
                self.account
                    .measured_peak
                    .set(self.previous.map(|previous| previous.max(peak)));
            }
        }
        let previous = self.0.measured_peak.replace(Some(self.used()));
        let window = Window {
            account: &self.0,
            previous,
        };
        let result = run();
        let peak = self.0.measured_peak.get().unwrap();
        drop(window);
        (result, peak)
    }

    #[cfg(test)]
    pub(in crate::document_scripting) fn capture_used(&self) -> usize {
        usize::try_from(
            self.0
                .ledger
                .usage(self.0.preparation_scope)
                .unwrap()
                .live(DocumentGuestResource::BackingBytes),
        )
        .unwrap()
    }

    pub(in crate::document_scripting) fn work_remaining(
        &self,
    ) -> Result<u64, DocumentGuestEventError> {
        let limit = self
            .0
            .ledger
            .limits(self.0.root)?
            .work(DocumentGuestWork::Adapter);
        Ok(limit.saturating_sub(self.0.usage()?.work(DocumentGuestWork::Adapter)))
    }

    #[cfg(test)]
    pub(in crate::document_scripting) fn set_work_remaining(&self, remaining: u64) {
        let limits = self.0.ledger.limits(self.0.root).unwrap();
        let work = self.work().unwrap().checked_add(remaining).unwrap();
        let limits = Limits::new(
            WorkTable::splat(work),
            ResourceTable::splat(limits.live(DocumentGuestResource::BackingBytes)),
        );
        self.0.ledger.set_limits(self.0.root, limits).unwrap();
    }
}

impl Accounting {
    fn scope(&self, scope: AccountScope) -> LedgerScope {
        match scope {
            AccountScope::Adapter => self.root,
            AccountScope::Projection => self.projection,
            AccountScope::Preparation => self.preparation_scope,
        }
    }

    fn usage(&self) -> Result<Usage, DocumentGuestEventError> {
        if let Some(error) = self.cleanup_error.get() {
            return Err(error.into());
        }
        Ok(self.ledger.usage(self.root)?)
    }

    fn check(
        &self,
        scope: AccountScope,
        change: Change<DocumentGuestWork, DocumentGuestResource>,
    ) -> Result<(), DocumentGuestEventError> {
        if let Some(error) = self.cleanup_error.get() {
            return Err(error.into());
        }
        match self.ledger.check_batch(self.scope(scope), &[change]) {
            Ok(()) => Ok(()),
            Err(error @ AccountingError::Refused(_)) => {
                // Only this attachment mutates the Ledger. No mutation can
                // occur between this check and recording the refused item.
                match self.apply(scope, change) {
                    Err(error) => Err(error),
                    Ok(()) => Err(error.into()),
                }
            }
            Err(error) => Err(error.into()),
        }
    }

    fn apply(
        &self,
        scope: AccountScope,
        change: Change<DocumentGuestWork, DocumentGuestResource>,
    ) -> Result<(), DocumentGuestEventError> {
        if let Some(error) = self.cleanup_error.get() {
            return Err(error.into());
        }
        self.ledger
            .apply(self.scope(scope), change)
            .map_err(|error| match error {
                AccountingError::Refused(refusal) => match refusal.change {
                    Change::Work(..) => DocumentGuestEventError::WorkLimit,
                    Change::Acquire(..) => DocumentScriptNativeError::StorageLimit.into(),
                    Change::Release(..) => error.into(),
                },
                error => error.into(),
            })?;
        #[cfg(test)]
        if matches!(change, Change::Acquire(..))
            && let Some(peak) = self.measured_peak.get()
        {
            let live = usize::try_from(self.usage()?.live(DocumentGuestResource::BackingBytes))
                .map_err(|_| DocumentScriptNativeError::StorageLimit)?;
            self.measured_peak.set(Some(peak.max(live)));
        }
        Ok(())
    }
}

/// Preparation release must finish before the binding boundary chooses its result.
pub(in crate::document_scripting) struct CaptureBacking {
    backing: GuestBacking,
    refusal: Weak<reports::DispatchFailure>,
}

impl CaptureBacking {
    pub(in crate::document_scripting) fn new(
        backing: GuestBacking,
        refusal: Rc<reports::DispatchFailure>,
    ) -> Self {
        Self {
            backing,
            refusal: Rc::downgrade(&refusal),
        }
    }
}

pub(in crate::document_scripting) fn capture_refusal(
    slot: &reports::DispatchFailure,
    error: DocumentGuestEventError,
) -> Error {
    match slot.record(error) {
        Ok(failure) | Err(RecordFailure::Occupied { first: failure, .. }) => {
            Error::HostFailure(failure)
        }
        Err(RecordFailure::Inactive { error, .. }) => error,
    }
}

impl CaptureBacking {
    pub(in crate::document_scripting) fn release(self) -> Result<(), Error> {
        let Self { backing, refusal } = self;
        let account = backing.account.clone();
        drop(backing);
        match account.cleanup_error.get() {
            Some(error) => Err(match refusal.upgrade() {
                Some(refusal) => capture_refusal(&refusal, error.into()),
                None => Error::ExpiredIdentity,
            }),
            None => Ok(()),
        }
    }
}

impl surgeist_bindings_js::preparation::Retention for CaptureBacking {
    fn release(self: Box<Self>) -> Result<(), Error> {
        (*self).release()
    }
}

pub(crate) struct GuestBacking {
    account: Rc<Accounting>,
    scope: AccountScope,
    canonical: DomHostedRuntimeRetention,
}

impl GuestBacking {
    pub(in crate::document_scripting) fn retained_bytes(&self) -> usize {
        self.canonical.retained_bytes()
    }

    pub(in crate::document_scripting) fn split_off(
        &mut self,
        bytes: usize,
    ) -> Result<Self, DocumentScriptNativeError> {
        let canonical = self
            .canonical
            .split_off(bytes)
            .map_err(|error| match error {
                DomError::RepresentationExhausted => {
                    DocumentScriptNativeError::RepresentationExhausted
                }
                other => other.into(),
            })?;
        Ok(Self {
            account: self.account.clone(),
            scope: self.scope,
            canonical,
        })
    }
}

impl Drop for GuestBacking {
    fn drop(&mut self) {
        if let Ok(units) = u64::try_from(self.retained_bytes())
            && let Err(error) = self.account.ledger.apply(
                self.account.scope(self.scope),
                Change::Release(DocumentGuestResource::BackingBytes, units),
            )
            && self.account.cleanup_error.get().is_none()
        {
            self.account.cleanup_error.set(Some(error));
        }
    }
}
