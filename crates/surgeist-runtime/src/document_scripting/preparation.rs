//! One preparation policy across native setup and the original guest attachment.

use super::guest::{
    AccountScope, CaptureBacking, ConversionOrigin, DispatchFailure, GuestBudget, capture_refusal,
};
use super::*;
use surgeist_bindings_js::conversion::{BufferRequest, BufferRetention, Host};
use surgeist_bindings_js::preparation::{Admission, Controller, Request, Retention, Work};

/// Installed policy and its attachment transition. Neither owns the native
/// Session or an engine handle; the Process retains the policy state through
/// its admitted cleanup storage.
pub(super) struct Prepared {
    controller: Controller<Backing>,
    state: Rc<State>,
}

struct State {
    native: Weak<NativeOwner>,
    authority: RealmAuthority,
    failures: Rc<FailureSlot>,
    quota: Rc<Quota>,
    guest: OnceCell<GuestPhase>,
    // Includes the State body and the binding policy's admitted setup. It
    // follows the policy's cleanup storage through surviving observations.
    _backing: DomHostedRuntimeRetention,
}

struct GuestPhase {
    budget: GuestBudget,
    refusal: Rc<DispatchFailure>,
}

struct Quota {
    limit: usize,
    fixed: usize,
    live: Cell<usize>,
    #[cfg(test)]
    peak: Cell<usize>,
    failures: Rc<FailureSlot>,
    cleanup: OnceCell<Error>,
    _backing: DomHostedRuntimeRetention,
}

/// Local-cap custody for actual additional native backing. This does not
/// replace canonical DOM retention. Keep it with the physical caller-owned
/// body and its canonical backing, including during replacement overlap.
pub(super) struct CapReservation {
    quota: Rc<Quota>,
    bytes: usize,
}

impl CapReservation {
    pub(super) fn retained_bytes(&self) -> usize {
        self.bytes
    }

    /// Transfer an already reserved subextent without acquiring it again.
    pub(super) fn split_off(&mut self, bytes: usize) -> Result<Self, DocumentScriptNativeError> {
        let remaining = self
            .bytes
            .checked_sub(bytes)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        self.bytes = remaining;
        Ok(Self {
            quota: self.quota.clone(),
            bytes,
        })
    }

    fn release(mut self) -> Result<(), Error> {
        let bytes = std::mem::replace(&mut self.bytes, 0);
        self.quota.release(bytes)
    }
}

impl Drop for CapReservation {
    fn drop(&mut self) {
        let bytes = std::mem::replace(&mut self.bytes, 0);
        let _recorded = self.quota.release(bytes);
    }
}

pub(super) enum Backing {
    Native(NativeBacking),
    Guest(CaptureBacking),
}

pub(super) struct NativeBacking {
    canonical: DomHostedRuntimeRetention,
    cap: CapReservation,
}

/// Original Runtime admission for binding conversion. The descriptor uses the
/// existing policy state and follows its one-time guest attachment transition.
pub(super) struct NativeConversionHost {
    state: Rc<State>,
    origin: ConversionOrigin,
}

impl NativeConversionHost {
    pub(super) fn for_origin(&self, origin: ConversionOrigin) -> Self {
        Self {
            state: self.state.clone(),
            origin,
        }
    }
}

pub(super) enum ConversionBacking {
    Native(NativeBacking),
    Guest(guest::GuestBacking),
}

#[derive(Debug)]
pub(super) enum ConversionCleanupError {
    Accounting(guest::AccountingError),
    Native(Error),
}

impl fmt::Display for ConversionCleanupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accounting(error) => error.fmt(formatter),
            Self::Native(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ConversionCleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Accounting(error) => Some(error),
            Self::Native(error) => Some(error),
        }
    }
}

impl BufferRetention for ConversionBacking {
    type Error = ConversionCleanupError;

    fn release(self) -> Result<(), Self::Error> {
        match self {
            Self::Native(NativeBacking { canonical, cap }) => {
                drop(canonical);
                cap.release().map_err(ConversionCleanupError::Native)
            }
            Self::Guest(backing) => backing
                .release()
                .map_err(ConversionCleanupError::Accounting),
        }
    }
}

impl Host for NativeConversionHost {
    type CleanupError = ConversionCleanupError;
    type Buffer = ConversionBacking;

    fn charge(
        &self,
        work: surgeist_bindings_js::conversion::Work,
        units: u64,
    ) -> Result<(), Error> {
        let native = self.state.native()?;
        if let Some(guest) = self.state.guest.get() {
            if native
                .cancellation
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return Err(capture_refusal(&guest.refusal, DomError::Cancelled.into()));
            }
            // Conversion can run under NativeRead's existing Session loan.
            guest
                .budget
                .charge_conversion(self.origin, work, units)
                .map_err(|error| capture_refusal(&guest.refusal, error))?;
            #[cfg(test)]
            observe_conversion_admission(&self.state, work);
            return Ok(());
        }
        (|| {
            let units = usize::try_from(units)
                .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
            let mut session = native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            native_admit_work(&mut session, &native, units)
        })()
        .map_err(|error| native_refusal(&self.state.failures, error))
    }

    fn retain_buffer(&self, request: BufferRequest) -> Result<Self::Buffer, Error> {
        self.state
            .conversion_backing(self.origin, request.backing_bytes())
    }

    fn begin_phase(&self) -> Result<Option<surgeist_ledger::PhaseObservation>, Error> {
        self.state.guest.get().map_or(Ok(None), |guest| {
            guest
                .budget
                .begin_conversion_phase(self.origin)
                .map(Some)
                .map_err(|error| capture_refusal(&guest.refusal, error.into()))
        })
    }

    fn end_phase(
        &self,
        marker: surgeist_ledger::PhaseObservation,
    ) -> Result<(), Self::CleanupError> {
        let guest = self
            .state
            .guest
            .get()
            .ok_or(ConversionCleanupError::Accounting(
                surgeist_ledger::Error::ForeignScope,
            ))?;
        guest
            .budget
            .finish_conversion_phase(marker)
            .map_err(ConversionCleanupError::Accounting)
    }
}

fn native_admit_work(
    session: &mut DomSession,
    native: &NativeOwner,
    units: usize,
) -> Result<(), DocumentScriptNativeError> {
    session
        .hosted_execution()
        .admit_script_work(&session.dom(), units, &native.cancellation)
        .map_err(DocumentScriptNativeError::from)
}

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        match *self {
            Self::Native(NativeBacking { canonical, cap }) => {
                drop(canonical);
                cap.release()
            }
            Self::Guest(backing) => backing.release(),
        }
    }
}

impl Quota {
    fn total_with(&self, additional: usize) -> Option<usize> {
        self.fixed
            .checked_add(self.live.get())?
            .checked_add(additional)
    }

    fn reserve(self: &Rc<Self>, bytes: usize) -> Result<CapReservation, DocumentScriptNativeError> {
        if self.cleanup.get().is_some() {
            return Err(DocumentScriptNativeError::ProjectionCustody);
        }
        if self
            .total_with(bytes)
            .is_none_or(|total| total > self.limit)
        {
            return Err(DocumentScriptNativeError::StorageLimit);
        }
        let live = self
            .live
            .get()
            .checked_add(bytes)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        self.live.set(live);
        #[cfg(test)]
        self.peak.set(self.peak.get().max(self.fixed + live));
        Ok(CapReservation {
            quota: self.clone(),
            bytes,
        })
    }

    fn release(&self, bytes: usize) -> Result<(), Error> {
        if let Some(live) = self.live.get().checked_sub(bytes) {
            self.live.set(live);
            return self.cleanup.get().cloned().map_or(Ok(()), Err);
        }
        let error = native_refusal(&self.failures, DocumentScriptNativeError::ProjectionCustody);
        let _later = self.cleanup.set(error.clone());
        Err(self.cleanup.get().cloned().unwrap_or(error))
    }
}

fn native_refusal(slot: &FailureSlot, error: DocumentScriptNativeError) -> Error {
    match slot.record(error) {
        Ok(failure) | Err(RecordFailure::Occupied { first: failure, .. }) => {
            Error::HostFailure(failure)
        }
        Err(RecordFailure::Inactive { error, .. }) => error,
    }
}

type PreparationResult<T> = Result<T, Error>;

fn policy(
    state: Weak<State>,
) -> Admission<
    impl Fn(Request) -> PreparationResult<Backing>,
    impl Fn(Work) -> PreparationResult<()>,
    Backing,
> {
    let work_state = state.clone();
    Admission::new(
        move |request| {
            state
                .upgrade()
                .ok_or(Error::ExpiredIdentity)?
                .admit(request)
        },
        move |work| {
            work_state
                .upgrade()
                .ok_or(Error::ExpiredIdentity)?
                .charge(work)
        },
    )
}

impl Prepared {
    #[cfg(test)]
    pub(super) fn observe_conversion_admission(
        &self,
        accepted: impl Fn(surgeist_bindings_js::conversion::Work) + 'static,
    ) -> ConversionAdmissionObservation {
        let observer = ConversionAdmissionObserver {
            state: Rc::downgrade(&self.state),
            accepted: Rc::new(accepted),
        };
        ConversionAdmissionObservation {
            previous: CONVERSION_ADMISSION_OBSERVER.with(|slot| slot.replace(Some(observer))),
        }
    }

    /// Exact additional Rust bodies for this policy. The caller's existing
    /// fixed reservation is separate and must not already include these bytes.
    pub(super) fn setup_storage_bytes() -> Result<usize, DocumentScriptNativeError> {
        let policy_bytes = policy(Weak::new())
            .storage_bytes::<Rc<State>>()
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        size_of::<State>()
            .checked_add(size_of::<Quota>())
            .and_then(|bytes| bytes.checked_add(size_of::<FailureSlot>()))
            .and_then(|bytes| bytes.checked_add(FailureSlot::storage_bytes()))
            .and_then(|bytes| bytes.checked_add(policy_bytes))
            .ok_or(DocumentScriptNativeError::StorageLimit)
    }

    /// Install before hook creation. `fixed_prepaid` is the actual existing
    /// fixed/reserved extent, whose canonical backing was already accepted.
    /// This constructor admits every additional setup body before allocating it.
    pub(super) fn install(
        process: &mut Process,
        native: &Rc<NativeOwner>,
        authority: &RealmAuthority,
        fixed_prepaid: usize,
    ) -> Result<Self, DocumentScriptError> {
        authority.check_owner_active()?;
        let setup = Self::setup_storage_bytes()?;
        let fixed = fixed_prepaid
            .checked_add(setup)
            .filter(|total| *total <= native.storage_limit)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let mut backing = {
            let mut session = native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .retain_script_payload(&mut session.dom(), setup, &native.cancellation)
                .map_err(DocumentScriptNativeError::from)?
        };
        let failure_bytes = size_of::<FailureSlot>()
            .checked_add(FailureSlot::storage_bytes())
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let failure_backing = backing
            .split_off(failure_bytes)
            .map_err(DocumentScriptNativeError::from)?;
        let quota_backing = backing
            .split_off(size_of::<Quota>())
            .map_err(DocumentScriptNativeError::from)?;
        let failures = Rc::new(FailureSlot::new(authority, failure_backing)?);
        let quota = Rc::new(Quota {
            limit: native.storage_limit,
            fixed,
            live: Cell::new(0),
            #[cfg(test)]
            peak: Cell::new(fixed),
            failures: failures.clone(),
            cleanup: OnceCell::new(),
            _backing: quota_backing,
        });
        let state = Rc::new(State {
            native: Rc::downgrade(native),
            authority: authority.clone(),
            failures,
            quota,
            guest: OnceCell::new(),
            _backing: backing,
        });
        let controller = policy(Rc::downgrade(&state)).install(process, state.clone())?;
        Ok(Self { controller, state })
    }

    pub(super) fn controller(&self) -> &Controller<Backing> {
        &self.controller
    }

    pub(super) fn conversion_host(&self, origin: ConversionOrigin) -> NativeConversionHost {
        NativeConversionHost {
            state: self.state.clone(),
            origin,
        }
    }

    pub(super) fn reserve_conversion_metadata(
        &self,
        origin: ConversionOrigin,
        bytes: usize,
    ) -> Result<ConversionBacking, Error> {
        self.state.conversion_backing(origin, bytes)
    }

    pub(super) fn total_with(&self, additional: usize) -> Option<usize> {
        self.state.quota.total_with(additional)
    }

    /// Test-only accepted peak, including this build's actual setup bodies.
    #[cfg(test)]
    pub(super) fn peak_storage_bytes(&self) -> usize {
        self.state.quota.peak.get()
    }

    /// Reserve only caller-owned native bodies. Producer preparation requests
    /// use their own exact sizes and acquire this same cap internally.
    pub(super) fn reserve_external(
        &self,
        bytes: usize,
    ) -> Result<CapReservation, DocumentScriptNativeError> {
        self.state.quota.reserve(bytes)
    }

    /// Attach the original guest operation once. The allowance is acquired
    /// before guest setup and must cover its actual configured limit plus setup.
    pub(super) fn activate(
        &self,
        budget: &GuestBudget,
        refusal: Rc<DispatchFailure>,
        allowance: CapReservation,
    ) -> Result<(), DocumentGuestEventError> {
        self.state.authority.check_owner_active()?;
        self.state.native()?;
        if self.state.guest.get().is_some()
            || !Rc::ptr_eq(&allowance.quota, &self.state.quota)
            || allowance.retained_bytes() != budget.reserved_storage_bytes()?
        {
            return Err(DocumentGuestEventError::RegistryState);
        }
        // Accounting follows every guest body and observation after the policy
        // retires. Individual allocations remain within this original allowance.
        budget.attach_preparation_allowance(allowance)?;
        self.state
            .guest
            .set(GuestPhase {
                budget: budget.clone(),
                refusal,
            })
            .map_err(|_| DocumentGuestEventError::RegistryState)
    }
}

#[cfg(test)]
struct ConversionAdmissionObserver {
    state: Weak<State>,
    accepted: Rc<dyn Fn(surgeist_bindings_js::conversion::Work)>,
}

#[cfg(test)]
thread_local! {
    static CONVERSION_ADMISSION_OBSERVER: RefCell<Option<ConversionAdmissionObserver>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(super) struct ConversionAdmissionObservation {
    previous: Option<ConversionAdmissionObserver>,
}

#[cfg(test)]
impl Drop for ConversionAdmissionObservation {
    fn drop(&mut self) {
        CONVERSION_ADMISSION_OBSERVER.with(|slot| {
            slot.replace(self.previous.take());
        });
    }
}

#[cfg(test)]
fn observe_conversion_admission(state: &Rc<State>, work: surgeist_bindings_js::conversion::Work) {
    let accepted = CONVERSION_ADMISSION_OBSERVER.with(|slot| {
        slot.borrow()
            .as_ref()
            .filter(|observer| observer.state.ptr_eq(&Rc::downgrade(state)))
            .map(|observer| observer.accepted.clone())
    });
    // Neither the observer registry nor the original Ledger is loaned here.
    if let Some(accepted) = accepted {
        accepted(work);
    }
}

impl State {
    fn native(&self) -> Result<Rc<NativeOwner>, Error> {
        if let Some(error) = self.quota.cleanup.get() {
            return Err(error.clone());
        }
        self.native.upgrade().ok_or(Error::ExpiredIdentity)
    }

    fn conversion_backing(
        &self,
        origin: ConversionOrigin,
        bytes: usize,
    ) -> Result<ConversionBacking, Error> {
        let native = self.native()?;
        if let Some(guest) = self.guest.get() {
            return guest
                .budget
                .reserve(AccountScope::Conversion(origin), &native, bytes)
                .map(ConversionBacking::Guest)
                .map_err(|error| capture_refusal(&guest.refusal, error));
        }
        let cap = self
            .quota
            .reserve(bytes)
            .map_err(|error| native_refusal(&self.failures, error))?;
        let canonical = (|| {
            let mut session = native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .retain_script_payload(&mut session.dom(), bytes, &native.cancellation)
                .map_err(DocumentScriptNativeError::from)
        })()
        .map_err(|error| native_refusal(&self.failures, error))?;
        Ok(ConversionBacking::Native(NativeBacking { canonical, cap }))
    }

    fn admit(&self, request: Request) -> Result<Backing, Error> {
        let native = self.native()?;
        if let Some(guest) = self.guest.get() {
            let backing = guest
                .budget
                .reserve(AccountScope::Preparation, &native, request.bytes())
                .map_err(|error| capture_refusal(&guest.refusal, error))?;
            return Ok(Backing::Guest(CaptureBacking::new(
                backing,
                guest.refusal.clone(),
            )));
        }
        let cap = self
            .quota
            .reserve(request.bytes())
            .map_err(|error| native_refusal(&self.failures, error))?;
        let canonical = (|| {
            let mut session = native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .retain_script_payload(&mut session.dom(), request.bytes(), &native.cancellation)
                .map_err(DocumentScriptNativeError::from)
        })()
        .map_err(|error| native_refusal(&self.failures, error))?;
        Ok(Backing::Native(NativeBacking { canonical, cap }))
    }

    fn charge(&self, work: Work) -> Result<(), Error> {
        let native = self.native()?;
        let units = match work {
            Work::BufferElement(_) => 1,
            Work::CodeUnits { units } => units,
        };
        let cancelled = native
            .cancellation
            .load(std::sync::atomic::Ordering::Relaxed);
        if let Some(guest) = self.guest.get() {
            if cancelled {
                return Err(capture_refusal(&guest.refusal, DomError::Cancelled.into()));
            }
            let units = u64::try_from(units)
                .map_err(|_| capture_refusal(&guest.refusal, DocumentGuestEventError::WorkLimit))?;
            return guest
                .budget
                .charge_work(AccountScope::Preparation, units)
                .map_err(|error| capture_refusal(&guest.refusal, error));
        }
        if cancelled {
            return Err(native_refusal(&self.failures, DomError::Cancelled.into()));
        }
        (|| {
            let mut session = native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .admit_script_work(&session.dom(), units, &native.cancellation)
                .map_err(DocumentScriptNativeError::from)
        })()
        .map_err(|error| native_refusal(&self.failures, error))
    }
}
