//! One preparation policy across native setup and the original guest attachment.

use super::guest::{AccountScope, CaptureBacking, DispatchFailure, GuestBudget, capture_refusal};
use super::*;
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

impl State {
    fn native(&self) -> Result<Rc<NativeOwner>, Error> {
        if let Some(error) = self.quota.cleanup.get() {
            return Err(error.clone());
        }
        self.native.upgrade().ok_or(Error::ExpiredIdentity)
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
