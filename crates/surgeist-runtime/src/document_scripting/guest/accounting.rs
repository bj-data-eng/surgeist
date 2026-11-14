//! Adapter admission and observation for the original attached guest owner.

use super::*;
use crate::document_scripting::preparation::CapReservation;
use surgeist_ledger::{
    Change, Limits, Operation, PhaseObservation, ProfileMode, Scope as LedgerScope,
    SharedOperation, Tariff, Tariffs,
};

surgeist_ledger::dimensions! {
    /// Work owned by the Runtime guest adapter.
    pub enum DocumentGuestWork => WorkTable {
        /// One admitted unit at an existing Runtime guest admission point.
        Adapter,
        /// One attempt to convert a JS value through a selected IDL type.
        ConversionValue,
        /// One payload code unit copied, validated, or normalized.
        ConversionCodeUnit,
        /// One lookup in a bounded declared enumeration.
        ConversionEnumerationLookup,
        /// One declared argument presence/default decision.
        ConversionArgument,
        /// One dictionary member visited, before reading guest properties.
        ConversionDictionaryMember,
        /// One iterator property read or invocation, before entering guest code.
        ConversionIteratorOperation,
        /// One yielded sequence item, before reading its value.
        ConversionSequenceItem,
        /// One previously accepted element moved to a growing Rust buffer.
        ConversionSequenceMove,
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
    conversion: LedgerScope,
    window_css_conversion: LedgerScope,
    dom_exception_conversion: LedgerScope,
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
pub(in crate::document_scripting) enum ConversionOrigin {
    General,
    WindowCss,
    DomException,
}

#[derive(Clone, Copy)]
pub(in crate::document_scripting) enum AccountScope {
    Adapter,
    Projection,
    Preparation,
    Conversion(ConversionOrigin),
}

fn adapter_work_limits(work: u64) -> WorkTable<u64> {
    WorkTable {
        Adapter: work,
        ..WorkTable::splat(0)
    }
}

fn conversion_tariffs() -> Tariffs<DocumentGuestWork, DocumentGuestResource> {
    Tariffs::from_fn(
        |_| Tariff::new(DocumentGuestWork::Adapter, 1),
        |resource| Tariff::new(resource, 1),
    )
}

fn conversion_work(work: surgeist_bindings_js::conversion::Work) -> DocumentGuestWork {
    use surgeist_bindings_js::conversion::Work;
    match work {
        Work::Value => DocumentGuestWork::ConversionValue,
        Work::CodeUnit => DocumentGuestWork::ConversionCodeUnit,
        Work::EnumerationLookup => DocumentGuestWork::ConversionEnumerationLookup,
        Work::Argument => DocumentGuestWork::ConversionArgument,
        Work::DictionaryMember => DocumentGuestWork::ConversionDictionaryMember,
        Work::IteratorOperation => DocumentGuestWork::ConversionIteratorOperation,
        Work::SequenceItem => DocumentGuestWork::ConversionSequenceItem,
        Work::SequenceMove => DocumentGuestWork::ConversionSequenceMove,
    }
}

#[derive(Clone)]
pub(in crate::document_scripting) struct GuestBudget(Rc<Accounting>);

impl GuestBudget {
    const SCOPES: usize = 6;

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
            Limits::new(
                adapter_work_limits(limits.work),
                ResourceTable::splat(bytes),
            ),
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
        let conversion = operation.child(
            root,
            "js-conversion",
            Limits::unlimited(),
            conversion_tariffs(),
        )?;
        let window_css_conversion = operation.child(
            root,
            "window-css",
            Limits::unlimited(),
            conversion_tariffs(),
        )?;
        let dom_exception_conversion = operation.child(
            root,
            "dom-exception",
            Limits::unlimited(),
            conversion_tariffs(),
        )?;
        Ok(Self(Rc::new(Accounting {
            ledger: operation.share(),
            root,
            projection,
            preparation_scope,
            conversion,
            window_css_conversion,
            dom_exception_conversion,
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

    pub(in crate::document_scripting) fn charge_conversion(
        &self,
        origin: ConversionOrigin,
        work: surgeist_bindings_js::conversion::Work,
        units: u64,
    ) -> Result<(), DocumentGuestEventError> {
        self.0.apply(
            AccountScope::Conversion(origin),
            Change::Work(conversion_work(work), units),
        )
    }

    pub(in crate::document_scripting) fn begin_conversion_phase(
        &self,
        origin: ConversionOrigin,
    ) -> Result<PhaseObservation, AccountingError> {
        if let Some(error) = self.0.cleanup_error.get() {
            return Err(error);
        }
        self.0
            .ledger
            .begin_phase(self.0.scope(AccountScope::Conversion(origin)))
    }

    pub(in crate::document_scripting) fn finish_conversion_phase(
        &self,
        marker: PhaseObservation,
    ) -> Result<(), AccountingError> {
        // Completion must consume the original marker even after an earlier
        // cleanup fault; that fault retains precedence over later defects.
        let result = self
            .0
            .ledger
            .end_phase(marker)
            .map_err(|_| AccountingError::ForeignScope)
            .and_then(|()| self.0.ledger.usage(self.0.root).map(|_| ()));
        if let Err(error) = result
            && self.0.cleanup_error.get().is_none()
        {
            self.0.cleanup_error.set(Some(error));
        }
        match self.0.cleanup_error.get() {
            Some(error) => Err(error),
            None => Ok(()),
        }
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
            local: GuestSourceInputBacking {
                account: self.0.clone(),
                scope,
                bytes,
            },
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
            local: GuestSourceInputBacking {
                account: self.0.clone(),
                scope,
                bytes: canonical.retained_bytes(),
            },
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
            adapter_work_limits(work),
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
            AccountScope::Conversion(origin) => match origin {
                ConversionOrigin::General => self.conversion,
                ConversionOrigin::WindowCss => self.window_css_conversion,
                ConversionOrigin::DomException => self.dom_exception_conversion,
            },
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
        match backing.release() {
            Err(error) => Err(match refusal.upgrade() {
                Some(refusal) => capture_refusal(&refusal, error.into()),
                None => Error::ExpiredIdentity,
            }),
            Ok(()) => Ok(()),
        }
    }
}

impl surgeist_bindings_js::preparation::Retention for CaptureBacking {
    fn release(self: Box<Self>) -> Result<(), Error> {
        (*self).release()
    }
}

pub(crate) struct GuestBacking {
    local: GuestSourceInputBacking,
    canonical: DomHostedRuntimeRetention,
}

/// Original guest-adapter accounting, released when its actual input becomes
/// native Source storage. It carries no native or Source execution authority.
pub(in crate::document_scripting) struct GuestSourceInputBacking {
    account: Rc<Accounting>,
    scope: AccountScope,
    bytes: usize,
}

impl GuestBacking {
    pub(in crate::document_scripting) fn release(self) -> Result<(), AccountingError> {
        let account = self.local.account.clone();
        drop(self);
        match account.cleanup_error.get() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub(in crate::document_scripting) fn retained_bytes(&self) -> usize {
        self.canonical.retained_bytes()
    }

    /// Transfer both existing receipts without reserving or releasing storage.
    /// The prepared native input keeps them together until Source installation.
    pub(in crate::document_scripting) fn into_source_input(
        self,
    ) -> (DomHostedRuntimeRetention, GuestSourceInputBacking) {
        (self.canonical, self.local)
    }

    pub(in crate::document_scripting) fn split_off(
        &mut self,
        bytes: usize,
    ) -> Result<Self, DocumentScriptNativeError> {
        let remaining = self
            .local
            .bytes
            .checked_sub(bytes)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let canonical = self
            .canonical
            .split_off(bytes)
            .map_err(|error| match error {
                DomError::RepresentationExhausted => {
                    DocumentScriptNativeError::RepresentationExhausted
                }
                other => other.into(),
            })?;
        self.local.bytes = remaining;
        Ok(Self {
            local: GuestSourceInputBacking {
                account: self.local.account.clone(),
                scope: self.local.scope,
                bytes,
            },
            canonical,
        })
    }
}

impl Drop for GuestSourceInputBacking {
    fn drop(&mut self) {
        if let Ok(units) = u64::try_from(self.bytes)
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

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_bindings_js::conversion::Work;
    use surgeist_ledger::{Dimension, EventFilter, RefusalReason};

    // Independent conversion contract: eight distinct semantic counts retain
    // their own child units while each consumes one original adapter unit.
    const CONVERSION_COUNTS: [(Work, DocumentGuestWork, u64); 8] = [
        (Work::Value, DocumentGuestWork::ConversionValue, 1),
        (Work::CodeUnit, DocumentGuestWork::ConversionCodeUnit, 2),
        (
            Work::EnumerationLookup,
            DocumentGuestWork::ConversionEnumerationLookup,
            3,
        ),
        (Work::Argument, DocumentGuestWork::ConversionArgument, 4),
        (
            Work::DictionaryMember,
            DocumentGuestWork::ConversionDictionaryMember,
            5,
        ),
        (
            Work::IteratorOperation,
            DocumentGuestWork::ConversionIteratorOperation,
            6,
        ),
        (
            Work::SequenceItem,
            DocumentGuestWork::ConversionSequenceItem,
            7,
        ),
        (
            Work::SequenceMove,
            DocumentGuestWork::ConversionSequenceMove,
            8,
        ),
    ];

    fn budget() -> (GuestBudget, DomSession, Arc<AtomicBool>) {
        let cancellation = Arc::new(AtomicBool::new(false));
        let (mut session, _) =
            crate::document_scripting::cross_realm_tests::hosted_session(&cancellation, 0);
        let mode = ProfileMode::Summary { timing: false };
        let canonical = session
            .hosted_execution()
            .retain_script_payload(
                &mut session.dom(),
                GuestBudget::setup_bytes(mode).unwrap(),
                &cancellation,
            )
            .unwrap();
        let budget = GuestBudget::new(
            DocumentGuestEventLimits {
                storage_bytes: 10,
                work: 20,
                pending_exception_reports: 0,
                dispatch: DomEventDispatchLimits {
                    path_items: 0,
                    snapshot_listeners: 0,
                    failure_records: 0,
                    callbacks: 0,
                    work: 0,
                },
                node_retirement: DomNodeRetirementLimits::new(0, 0),
                source_edits: DomSourceLimits::new(0, 0, 0, 0, 0, 0, 0),
            },
            mode,
            canonical,
        )
        .unwrap();
        (budget, session, cancellation)
    }

    #[test]
    fn source_input_transfer_keeps_both_original_receipts_without_another_admission() {
        let (budget, mut session, cancellation) = budget();
        let backing = budget
            .reserve_with(AccountScope::Adapter, 10, || {
                session
                    .hosted_execution()
                    .retain_script_payload(&mut session.dom(), 10, &cancellation)
                    .map_err(DocumentScriptNativeError::from)
            })
            .unwrap();
        let work = budget.work().unwrap();
        let (canonical, local) = backing.into_source_input();
        assert_eq!(budget.used(), 10);
        assert_eq!(canonical.retained_bytes(), 10);
        assert_eq!(budget.work().unwrap(), work);

        // Adoption ends adapter ownership; the native receipt independently
        // survives until the consuming Source performs its equal-size move.
        drop(local);
        assert_eq!(budget.used(), 0);
        assert_eq!(canonical.retained_bytes(), 10);
        assert_eq!(budget.work().unwrap(), work);
        drop(canonical);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn original_conversion_scopes_complete_after_cleanup_failure() {
        let (budget, mut session, cancellation) = budget();
        for (origin, name) in [
            (ConversionOrigin::General, "js-conversion"),
            (ConversionOrigin::WindowCss, "window-css"),
            (ConversionOrigin::DomException, "dom-exception"),
        ] {
            let marker = budget.begin_conversion_phase(origin).unwrap();
            budget.charge_conversion(origin, Work::Value, 1).unwrap();
            budget.finish_conversion_phase(marker).unwrap();
            let report = budget.observation().snapshot().unwrap().unwrap();
            let scope = report
                .scopes
                .iter()
                .find(|scope| scope.name == name)
                .unwrap();
            assert_eq!(scope.usage.work(DocumentGuestWork::ConversionValue), 1);
            assert_eq!(scope.phases, 1);
        }
        assert_eq!(budget.work().unwrap(), 3);
        let report = budget.observation().snapshot().unwrap().unwrap();
        assert_eq!(report.scopes.len(), 6);
        let limits = budget.0.ledger.limits(budget.0.root).unwrap();
        for dimension in DocumentGuestWork::ALL {
            assert_eq!(
                limits.work(*dimension),
                if *dimension == DocumentGuestWork::Adapter {
                    20
                } else {
                    0
                }
            );
        }

        let scope = AccountScope::Conversion(ConversionOrigin::General);
        let mut backing = budget
            .reserve_with(scope, 10, || {
                session
                    .hosted_execution()
                    .retain_script_payload(&mut session.dom(), 10, &cancellation)
                    .map_err(DocumentScriptNativeError::from)
            })
            .unwrap();
        let split = backing.split_off(4).unwrap();
        assert_eq!(budget.used(), 10);
        assert_eq!(budget.work().unwrap(), 4);
        split.release().unwrap();
        assert_eq!(budget.used(), 6);
        backing.release().unwrap();
        assert_eq!(budget.used(), 0);

        let marker = budget
            .begin_conversion_phase(ConversionOrigin::General)
            .unwrap();
        let backing = budget
            .reserve_with(scope, 4, || {
                session
                    .hosted_execution()
                    .retain_script_payload(&mut session.dom(), 4, &cancellation)
                    .map_err(DocumentScriptNativeError::from)
            })
            .unwrap();
        // Make the release receipt disagree with its real Ledger custody. The
        // drop must expose that typed defect rather than erase it or panic.
        budget
            .0
            .ledger
            .apply(
                budget.0.scope(scope),
                Change::Release(DocumentGuestResource::BackingBytes, 4),
            )
            .unwrap();
        let error = backing.release().unwrap_err();
        assert!(matches!(
            error,
            AccountingError::Refused(refusal) if refusal.reason == RefusalReason::Underflow
        ));
        assert_eq!(budget.finish_conversion_phase(marker), Err(error));
        let report = budget.0.ledger.snapshot().unwrap().unwrap();
        let general = report
            .scopes
            .iter()
            .find(|scope| scope.name == "js-conversion")
            .unwrap();
        assert_eq!(general.phases, 2);
        assert_eq!(budget.observation().snapshot(), Err(error));
    }

    #[test]
    fn foreign_conversion_phase_keeps_its_typed_ownership_error() {
        let (first, _session, _cancellation) = budget();
        let (second, _other_session, _other_cancellation) = budget();
        let marker = first
            .begin_conversion_phase(ConversionOrigin::General)
            .unwrap();
        assert_eq!(
            second.finish_conversion_phase(marker),
            Err(AccountingError::ForeignScope)
        );
        let report = second.0.ledger.snapshot().unwrap().unwrap();
        assert!(report.scopes.iter().all(|scope| scope.phases == 0));
    }

    #[test]
    fn conversion_dimensions_share_the_exact_original_work_allowance() {
        for mode in [
            ProfileMode::Off,
            ProfileMode::Summary { timing: false },
            ProfileMode::Summary { timing: true },
            ProfileMode::Detailed {
                timing: true,
                event_limit: 32,
                filter: EventFilter::All,
            },
        ] {
            for limit in [37, 36] {
                let mut operation = Operation::with_reserved_profile(
                    "runtime-guest",
                    Limits::new(adapter_work_limits(limit), ResourceTable::splat(0)),
                    2,
                    mode,
                )
                .unwrap();
                let root = operation.root();
                let child = operation
                    .child(
                        root,
                        "js-conversion",
                        Limits::unlimited(),
                        conversion_tariffs(),
                    )
                    .unwrap();
                let marker = operation.begin_phase(child).unwrap();
                // Buffer reservation retains its existing allocation work.
                operation
                    .apply(child, Change::Work(DocumentGuestWork::Adapter, 1))
                    .unwrap();
                for (work, expected, units) in CONVERSION_COUNTS {
                    let result = operation.apply(child, Change::Work(conversion_work(work), units));
                    if limit == 36 && work == Work::SequenceMove {
                        let Err(AccountingError::Refused(refusal)) = result else {
                            panic!("one unit short must refuse the aggregate conversion work");
                        };
                        assert_eq!(refusal.scope, root);
                        assert_eq!(refusal.change, Change::Work(DocumentGuestWork::Adapter, 8));
                        assert_eq!(refusal.current, 29);
                        assert_eq!(refusal.reason, RefusalReason::Limit { limit: 36 });
                        assert_eq!(operation.usage(child).unwrap().work(expected), 0);
                    } else {
                        result.unwrap();
                        assert_eq!(operation.usage(child).unwrap().work(expected), units);
                    }
                }
                operation.end_phase(marker).unwrap();
                let aggregate = operation.usage(root).unwrap();
                assert_eq!(
                    aggregate.work(DocumentGuestWork::Adapter),
                    if limit == 37 { 37 } else { 29 }
                );
                for dimension in DocumentGuestWork::ALL {
                    if *dimension != DocumentGuestWork::Adapter {
                        assert_eq!(aggregate.work(*dimension), 0);
                    }
                }
                let report = operation.snapshot().unwrap();
                if mode == ProfileMode::Off {
                    assert!(report.is_none());
                } else {
                    let report = report.unwrap();
                    assert_eq!(report.scopes[1].phases, 1);
                    for (work, expected, units) in CONVERSION_COUNTS {
                        assert_eq!(
                            report.scopes[1].usage.work(expected),
                            if limit == 36 && work == Work::SequenceMove {
                                0
                            } else {
                                units
                            }
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn conversion_origins_keep_raw_attribution_and_share_live_backing() {
        let mut operation = Operation::with_reserved_profile(
            "runtime-guest",
            Limits::new(adapter_work_limits(6), ResourceTable::splat(10)),
            4,
            ProfileMode::Summary { timing: false },
        )
        .unwrap();
        let root = operation.root();
        let mut children = Vec::new();
        for (name, work, bytes) in [
            ("js-conversion", 1, 2),
            ("window-css", 2, 3),
            ("dom-exception", 3, 5),
        ] {
            let child = operation
                .child(root, name, Limits::unlimited(), conversion_tariffs())
                .unwrap();
            operation
                .apply(child, Change::Work(conversion_work(Work::Value), work))
                .unwrap();
            operation
                .apply(
                    child,
                    Change::Acquire(DocumentGuestResource::BackingBytes, bytes),
                )
                .unwrap();
            children.push((child, work, bytes));
        }
        assert_eq!(
            operation
                .usage(root)
                .unwrap()
                .work(DocumentGuestWork::Adapter),
            6
        );
        assert_eq!(
            operation
                .usage(root)
                .unwrap()
                .live(DocumentGuestResource::BackingBytes),
            10
        );
        assert!(matches!(
            operation.apply(children[0].0, Change::Acquire(DocumentGuestResource::BackingBytes, 1)),
            Err(AccountingError::Refused(refusal))
                if refusal.scope == root
                    && refusal.reason == RefusalReason::Limit { limit: 10 }
        ));
        let report = operation.snapshot().unwrap().unwrap();
        for (child, work, bytes) in children {
            let origin = report
                .scopes
                .iter()
                .find(|scope| scope.scope == child)
                .unwrap();
            assert_eq!(origin.usage.work(DocumentGuestWork::ConversionValue), work);
            assert_eq!(origin.usage.work(DocumentGuestWork::Adapter), 0);
            assert_eq!(
                origin.usage.live(DocumentGuestResource::BackingBytes),
                bytes
            );
            operation
                .apply(
                    child,
                    Change::Release(DocumentGuestResource::BackingBytes, bytes),
                )
                .unwrap();
        }
        assert_eq!(
            operation
                .usage(root)
                .unwrap()
                .live(DocumentGuestResource::BackingBytes),
            0
        );
    }
}
