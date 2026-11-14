//! Producer-local limits with admission delegated to the original operation.

use std::any::Any;
use std::cell::{Cell, OnceCell};
use std::fmt;
use std::rc::{Rc, Weak};

use surgeist_ledger::{Limits, PhaseObservation};

use crate::owner::OwnerState;
use crate::{Error, Scope};

use super::account::{Account, Storage};
use super::local::LocalState;
use super::registry::{CleanupCustody, CleanupRecord, Registrations};
use super::{
    CleanupFault, CleanupObservation, CleanupStage, CleanupStatus, ConversionAccount, Resource,
    Work,
};

/// An owned charge for one live Rust conversion buffer and its release record.
/// Releasing consumes the receipt once and preserves the original typed cause.
pub trait BufferRetention: 'static {
    type Error: std::error::Error + 'static;
    fn release(self) -> Result<(), Self::Error>;
}

/// Exact requested payload and admission-record bodies for one buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferRequest {
    payload: u64,
    backing: usize,
}

impl BufferRequest {
    pub fn payload_bytes(self) -> u64 {
        self.payload
    }

    /// Payload plus the concrete release-record body. Allocator and Rc control
    /// bookkeeping and opaque engine allocations are excluded.
    pub fn backing_bytes(self) -> usize {
        self.backing
    }
}

/// Adapter to the enclosing operation's admission and optional phase observer.
/// Each call begins after binding Ledger loans end. The host admits before
/// effects, returns custody for accepted storage and finishes each phase marker.
/// Host callbacks may release existing converted buffers. Admission reentry on
/// this same account refuses before invoking the host again.
pub trait Host: 'static {
    type CleanupError: std::error::Error + 'static;
    type Buffer: BufferRetention<Error = Self::CleanupError>;

    fn charge(&self, work: Work, units: u64) -> Result<(), Error>;
    fn retain_buffer(&self, request: BufferRequest) -> Result<Self::Buffer, Error>;
    fn begin_phase(&self) -> Result<Option<PhaseObservation>, Error>;
    fn end_phase(&self, marker: PhaseObservation) -> Result<(), Self::CleanupError>;
}

/// Allocation-free setup, allowing the enclosing host to fund metadata first.
pub struct Hosted<H> {
    host: H,
}

impl<H: Host> Hosted<H> {
    pub fn new(host: H) -> Self {
        Self { host }
    }

    /// Requested setup bodies, including the owner registration and concrete
    /// host, cleanup error slot and shared backing B. B retains this admitted
    /// extent until the last cleanup observation or result custody retires.
    pub fn storage_bytes<B: 'static>(&self) -> Option<usize> {
        size_of::<Account>()
            .checked_add(size_of::<Target<H>>())?
            .checked_add(size_of::<StorageState>())?
            .checked_add(size_of::<Record<H::CleanupError>>())?
            .checked_add(Registrations::registration_bytes())?
            .checked_add(size_of::<B>())
    }

    /// Installs exact producer-local caps. The ordinary backing B must fund
    /// `storage_bytes::<B>()` before this call. It must contain no strong edge
    /// back to this account, its cleanup record or the execution owner.
    pub fn install<B: 'static>(
        self,
        scope: &mut Scope<'_>,
        limits: Limits<Work, Resource>,
        backing: B,
    ) -> Result<ConversionAccount, Error> {
        scope.check_active()?;
        self.storage_bytes::<B>().ok_or(Error::Allocation)?;
        let owner = scope.operations.owner_state();
        owner.reap_conversion_cleanup();
        let backing: Rc<dyn Any> = Rc::new(backing);
        let record = Rc::new(Record {
            owner: owner.id(),
            first: OnceCell::new(),
            _backing: backing.clone(),
        });
        let erased: Rc<dyn CleanupRecord> = record.clone();
        let state = Rc::new(StorageState {
            local: LocalState::new(limits),
            cleanup: Cell::new(CleanupStatus::default()),
            owner: Rc::downgrade(&owner),
            record: Rc::downgrade(&erased),
            _backing: backing,
        });
        let custody: Rc<dyn CleanupCustody> = state.clone();
        owner.register_conversion_cleanup(Rc::downgrade(&custody), erased);
        owner.available()?;
        Ok(ConversionAccount(Rc::new(Account::Hosted(Box::new(
            Target {
                host: self.host,
                state,
                record,
            },
        )))))
    }
}

pub(super) trait Factory {
    fn owner(&self) -> &Weak<OwnerState>;
    fn work(&self, work: Work, units: u64) -> Result<(), Error>;
    fn reserve(&self, bytes: u64) -> Result<Storage, Error>;
    fn empty(&self) -> Storage;
    fn begin_phase(&self) -> Result<Option<PhaseObservation>, Error>;
    fn end_phase(&self, marker: PhaseObservation);
    fn cleanup_status(&self) -> CleanupStatus;
    fn observation(&self) -> CleanupObservation;
}

struct Target<H: Host> {
    host: H,
    state: Rc<StorageState>,
    record: Rc<Record<H::CleanupError>>,
}

impl<H: Host> Factory for Target<H> {
    fn owner(&self) -> &Weak<OwnerState> {
        &self.state.owner
    }

    fn work(&self, work: Work, units: u64) -> Result<(), Error> {
        let pending = self
            .state
            .local
            .stage_work(work, units)
            .map_err(Error::ConversionAdmission)?;
        {
            let _host = self.state.local.enter_host();
            self.host.charge(work, units)?;
        }
        pending.commit();
        self.state.available()
    }

    fn reserve(&self, bytes: u64) -> Result<Storage, Error> {
        if bytes == 0 {
            return Ok(self.empty());
        }
        let pending = self
            .state
            .local
            .stage_buffer(bytes)
            .map_err(Error::ConversionAdmission)?;
        let backing = usize::try_from(bytes)
            .ok()
            .and_then(|bytes| bytes.checked_add(size_of::<Retained<H::Buffer>>()))
            .ok_or(Error::Allocation)?;
        let buffer = {
            let _host = self.state.local.enter_host();
            self.host.retain_buffer(BufferRequest {
                payload: bytes,
                backing,
            })?
        };
        pending.commit();
        let storage = Storage::Hosted {
            state: self.state.clone(),
            bytes,
            receipt: Some(Box::new(Retained {
                buffer,
                record: Rc::downgrade(&self.record),
            })),
        };
        self.state.available()?;
        Ok(storage)
    }

    fn empty(&self) -> Storage {
        Storage::Hosted {
            state: self.state.clone(),
            bytes: 0,
            receipt: None,
        }
    }

    fn begin_phase(&self) -> Result<Option<PhaseObservation>, Error> {
        self.state
            .local
            .check_admission()
            .map_err(Error::ConversionAdmission)?;
        let _host = self.state.local.enter_host();
        let marker = self.host.begin_phase()?;
        if let Err(error) = self.state.available() {
            if let Some(marker) = marker {
                self.end_phase(marker);
            }
            return Err(error);
        }
        Ok(marker)
    }

    fn end_phase(&self, marker: PhaseObservation) {
        let _host = self.state.local.enter_host();
        if let Err(error) = self.host.end_phase(marker) {
            self.record
                .record(&self.state, CleanupStage::PhaseCompletion, error);
        }
    }

    fn cleanup_status(&self) -> CleanupStatus {
        self.state.cleanup_status()
    }

    fn observation(&self) -> CleanupObservation {
        CleanupObservation::hosted(self.state.clone(), self.record.clone())
    }
}

pub(super) struct StorageState {
    local: LocalState,
    cleanup: Cell<CleanupStatus>,
    owner: Weak<OwnerState>,
    record: Weak<dyn CleanupRecord>,
    _backing: Rc<dyn Any>,
}

impl CleanupCustody for StorageState {}

impl StorageState {
    pub(super) fn owner(&self) -> &Weak<OwnerState> {
        &self.owner
    }

    fn available(&self) -> Result<(), Error> {
        self.owner
            .upgrade()
            .ok_or(Error::ExpiredIdentity)?
            .available()
    }

    pub(super) fn cleanup_status(&self) -> CleanupStatus {
        self.cleanup.get()
    }

    pub(super) fn release(&self, bytes: u64, receipt: Option<Box<dyn BufferReceipt>>) {
        let _host = self.local.enter_host();
        if bytes != 0
            && let Err(error) = self.local.release(bytes)
        {
            let fault = CleanupFault::LocalStorageRelease(error);
            self.cleanup.set(self.cleanup.get().record(fault));
            if let Some(owner) = self.owner.upgrade() {
                owner.record_cleanup(fault);
            }
        }
        if let Some(receipt) = receipt {
            receipt.release(self);
        }
    }
}

impl Drop for StorageState {
    fn drop(&mut self) {
        // Registration holds only Weak custody, so reaping cannot retain us.
        // Drop the upgrade after reaping, outside any owner registry loan.
        if let Some(owner) = self.owner.upgrade() {
            owner.reap_conversion_cleanup();
        }
    }
}

struct Record<E> {
    owner: u64,
    first: OnceCell<(CleanupStage, E)>,
    _backing: Rc<dyn Any>,
}

impl<E: std::error::Error + 'static> Record<E> {
    fn record(&self, state: &StorageState, stage: CleanupStage, error: E) {
        let _later = self.first.set((stage, error));
        let fault = match stage {
            CleanupStage::StorageRelease => CleanupFault::HostStorageRelease,
            CleanupStage::PhaseCompletion => CleanupFault::HostPhaseCompletion,
        };
        state.cleanup.set(state.cleanup.get().record(fault));
        if let Some(owner) = state.owner.upgrade()
            && let Some(record) = state.record.upgrade()
        {
            owner.record_host_cleanup(record);
        }
    }
}

impl<E: fmt::Debug> fmt::Debug for Record<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConversionCleanup")
            .field("owner", &self.owner)
            .field("first", &self.first)
            .finish_non_exhaustive()
    }
}

impl<E: std::error::Error + 'static> CleanupRecord for Record<E> {
    fn owner(&self) -> u64 {
        self.owner
    }
    fn error(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.first
            .get()
            .map(|(_, error)| error as &dyn std::error::Error)
    }
    fn stage(&self) -> Option<CleanupStage> {
        self.first.get().map(|(stage, _)| *stage)
    }
}

pub(super) trait BufferReceipt {
    fn release(self: Box<Self>, state: &StorageState);
}

struct Retained<B: BufferRetention> {
    buffer: B,
    record: Weak<Record<B::Error>>,
}

impl<B: BufferRetention> BufferReceipt for Retained<B> {
    fn release(self: Box<Self>, state: &StorageState) {
        let Self { buffer, record } = *self;
        if let Err(error) = buffer.release()
            && let Some(record) = record.upgrade()
        {
            record.record(state, CleanupStage::StorageRelease, error);
        }
    }
}

#[cfg(test)]
mod tests;
