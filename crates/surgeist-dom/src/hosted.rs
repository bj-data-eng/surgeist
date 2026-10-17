//! Controlled native-host ownership and checked lifecycle identities.
//!
//! The source issues provenance for an external owner. It does not implement an
//! event loop, an agent, a clock or a second Document/activity table.

mod destruction;
pub use destruction::{
    DomHostedChildRemovalHandoff, DomHostedChildRemovalPublication, DomHostedDestructionInvocation,
    DomHostedDestructionInvocationRequest,
};
mod child;
pub use child::{
    DomHostedChildHistoryRequest, DomHostedChildPreparation, DomHostedChildRemovalRequest,
    DomHostedChildRequest, DomHostedChildStep, DomHostedEmbedderKind,
    DomHostedEmbedderPolicyInputs, DomHostedPreparedChild,
};
mod queue;
pub use queue::{
    DomHostedChildHistoryPublication, DomHostedHistoryJobReceipt, DomHostedHistoryJobRequest,
    DomHostedHistoryStep, DomHostedQueuedHistory,
};
mod history;
pub use history::{DomHostedEntry, DomHostedEntryInitialization, DomHostedNavigable};

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::{DomError, DomOwnerId};

/// Identity of the unique hosted issuer registered with one canonical owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomHostedSourceId {
    pub(crate) owner: DomOwnerId,
    pub(crate) serial: u64,
}

macro_rules! identity {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name {
            pub(crate) source: DomHostedSourceId,
            pub(crate) slot: usize,
            pub(crate) generation: u64,
        }
        impl $name {
            /// The original issuer; this observation cannot keep it alive.
            pub const fn source(self) -> DomHostedSourceId {
                self.source
            }
        }
    };
}
identity!(DomHostedHostId, "One actual hosted owner record.");
identity!(
    DomHostedBrowsingContextId,
    "One actual browsing-context record."
);
identity!(DomHostedNavigableId, "One actual navigable record.");
identity!(DomHostedEntryId, "One actual session-history entry record.");
identity!(
    DomHostedGlobalId,
    "One actual native Global EventTarget record."
);
identity!(
    DomHostedOperationId,
    "One retained lifecycle operation occurrence."
);

/// Independent checked revision of the original external hosted owner.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DomHostedSourceRevision(pub(crate) u64);
impl DomHostedSourceRevision {
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Explicit bounds on live issued role identities and retained registries.
/// Exhausted identity tombstones and spare backing remain byte-charged even
/// when they no longer consume a live role count. Zero rejects new records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedRecordLimits {
    hosts: usize,
    browsing_contexts: usize,
    navigables: usize,
    entries: usize,
    globals: usize,
    operations: usize,
    registries: usize,
}
impl DomHostedRecordLimits {
    pub const fn new(
        hosts: usize,
        browsing_contexts: usize,
        navigables: usize,
        entries: usize,
        globals: usize,
        operations: usize,
        registries: usize,
    ) -> Self {
        Self {
            hosts,
            browsing_contexts,
            navigables,
            entries,
            globals,
            operations,
            registries,
        }
    }
    pub const fn hosts(self) -> usize {
        self.hosts
    }
    pub const fn browsing_contexts(self) -> usize {
        self.browsing_contexts
    }
    pub const fn navigables(self) -> usize {
        self.navigables
    }
    pub const fn entries(self) -> usize {
        self.entries
    }
    pub const fn globals(self) -> usize {
        self.globals
    }
    pub const fn operations(self) -> usize {
        self.operations
    }
    pub const fn registries(self) -> usize {
        self.registries
    }
}

/// Canonical hosted storage and per-operation work admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedLimits {
    records: DomHostedRecordLimits,
    retained_bytes: usize,
    scratch_bytes: usize,
    work: u64,
}
impl DomHostedLimits {
    pub const fn new(
        records: DomHostedRecordLimits,
        retained_bytes: usize,
        scratch_bytes: usize,
        work: u64,
    ) -> Self {
        Self {
            records,
            retained_bytes,
            scratch_bytes,
            work,
        }
    }
    pub const fn records(self) -> DomHostedRecordLimits {
        self.records
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub const fn scratch_bytes(self) -> usize {
        self.scratch_bytes
    }
    pub const fn work(self) -> u64 {
        self.work
    }
}

/// Actual finite admission domain, independent of canonical tree limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostedResource {
    Hosts,
    BrowsingContexts,
    Navigables,
    Entries,
    Globals,
    Operations,
    Registries,
    RetainedBytes,
    ScratchBytes,
    Work,
}

/// A refusal preserves the original operation and unconsumed publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomHostedError {
    Dom(DomError),
    AlreadyRegistered,
    PriorDetailsTransfer,
    ForeignSource,
    SourceExpired,
    SourcePoisoned,
    StaleIdentity,
    WrongPhase,
    MissingPublication,
    StaleBasis,
    IncompleteOperation,
    UnsupportedProfile,
    PendingDocumentObligations,
    ClockRegression,
    InvalidClockSample,
    InvalidEntryInitialization,
    RepresentationExhausted,
    AllocationFailed,
    Limit {
        resource: DomHostedResource,
        limit: usize,
    },
    WorkLimit {
        limit: u64,
    },
}
impl From<DomError> for DomHostedError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}
impl fmt::Display for DomHostedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DomHostedError {}

/// The initial inherited result; it does not establish platform fullscreen support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFullscreenInherited {
    Enabled,
    Disabled,
}
/// The selected fixed native Permissions Policy roster and origin profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomInitialPermissionsPolicyProfile {
    FullscreenSelfOpaqueInitial,
}
/// Actual initial inherited value, with empty declared/reporting maps by representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomInitialPermissionsPolicy {
    pub(crate) inherited: DomFullscreenInherited,
}
impl DomInitialPermissionsPolicy {
    pub const fn inherited_fullscreen(self) -> DomFullscreenInherited {
        self.inherited
    }
    pub const fn profile(self) -> DomInitialPermissionsPolicyProfile {
        DomInitialPermissionsPolicyProfile::FullscreenSelfOpaqueInitial
    }
}

/// An actual clock sample from this source's single epoch, floored to 100µs.
/// Construction is confined to the original source's phase publications.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedMoment {
    pub(crate) source: DomHostedSourceId,
    pub(crate) microseconds: u64,
}
impl DomHostedMoment {
    pub const fn source(self) -> DomHostedSourceId {
        self.source
    }
    pub const fn microseconds(self) -> u64 {
        self.microseconds
    }
}

#[derive(Clone, Debug)]
struct IdentitySlot {
    generation: u64,
    live: bool,
    retired: bool,
    phase: u64,
    issued: u8,
    creator: Option<RequestKey>,
    work: Option<WorkObservation>,
    reserved_revisions: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Role {
    Host,
    BrowsingContext,
    Navigable,
    Entry,
    Global,
    Operation,
}
impl Role {
    fn index(self) -> usize {
        self as usize
    }
    fn bound(self, limits: DomHostedRecordLimits) -> (usize, DomHostedResource) {
        match self {
            Self::Host => (limits.hosts, DomHostedResource::Hosts),
            Self::BrowsingContext => (
                limits.browsing_contexts,
                DomHostedResource::BrowsingContexts,
            ),
            Self::Navigable => (limits.navigables, DomHostedResource::Navigables),
            Self::Entry => (limits.entries, DomHostedResource::Entries),
            Self::Global => (limits.globals, DomHostedResource::Globals),
            Self::Operation => (limits.operations, DomHostedResource::Operations),
        }
    }
}

#[derive(Debug)]
struct Issuance {
    alive: bool,
    revision: DomHostedSourceRevision,
    banks: [Vec<IdentitySlot>; 6],
    reserved_revisions: u64,
}
#[derive(Debug)]
pub(crate) struct Shared {
    id: DomHostedSourceId,
    issuance: Mutex<Issuance>,
    bytes: AtomicUsize,
    registries: AtomicUsize,
    pub(crate) runtime_bytes: AtomicUsize,
}

/// Weak access to the actual counters owned by the original operation. These
/// observations live in its existing issuer role rows, never in a second ledger.
#[derive(Clone, Debug)]
struct WorkObservation {
    spent: Weak<AtomicU64>,
    limit: Weak<AtomicU64>,
}
impl WorkObservation {
    fn new(work: &Work) -> Self {
        Self {
            spent: Arc::downgrade(&work.spent),
            limit: Arc::downgrade(&work.limit),
        }
    }
    fn validate_limit(&self, limit: u64) -> Result<(), DomHostedError> {
        if self
            .spent
            .upgrade()
            .is_some_and(|spent| spent.load(Ordering::Relaxed) > limit)
        {
            return Err(DomHostedError::WorkLimit { limit });
        }
        Ok(())
    }
    fn set_limit(&self, limit: u64) {
        if let Some(current) = self.limit.upgrade() {
            current.store(limit, Ordering::Relaxed);
        }
    }
}

/// The one cumulative counter retained from preparation through completion.
#[derive(Debug)]
pub(crate) struct Work {
    pub(crate) spent: Arc<AtomicU64>,
    limit: Arc<AtomicU64>,
    _retained: Option<Retained>,
}
impl Work {
    pub(crate) fn step(&self, cancel: &AtomicBool) -> Result<(), DomHostedError> {
        self.spend(1, cancel)
    }
    pub(crate) fn spend(&self, amount: usize, cancel: &AtomicBool) -> Result<(), DomHostedError> {
        crate::store::check_cancel(cancel)?;
        let amount = u64::try_from(amount).map_err(|_| DomHostedError::RepresentationExhausted)?;
        let limit = self.limit.load(Ordering::Relaxed);
        self.spent
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_add(amount).filter(|next| *next <= limit)
            })
            .map(|_| ())
            .map_err(|old| {
                if old == u64::MAX {
                    DomHostedError::RepresentationExhausted
                } else {
                    DomHostedError::WorkLimit { limit }
                }
            })
    }
}

/// An actual retained occurrence in the original issuer's finite storage.
#[derive(Debug)]
pub(crate) struct Retained {
    shared: Weak<Shared>,
    bytes: usize,
}
impl Retained {
    pub(crate) fn append(&mut self, mut other: Self) -> Result<(), DomHostedError> {
        if !self.shared.ptr_eq(&other.shared) {
            return Err(DomHostedError::ForeignSource);
        }
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        other.bytes = 0;
        Ok(())
    }
}
impl Drop for Retained {
    fn drop(&mut self) {
        if let Some(shared) = self.shared.upgrade() {
            shared.bytes.fetch_sub(self.bytes, Ordering::Relaxed);
        }
    }
}

/// Unique issuer retained by the actual external lifecycle owner. Identity
/// observations and canonical snapshots retain only weak live-use permission.
#[derive(Debug)]
pub struct DomHostedSource {
    pub(crate) shared: Arc<Shared>,
    limits: DomHostedLimits,
    pub(crate) active_work: Weak<Work>,
}

/// Observation used for live-use validation, never for historical activity reads.
#[derive(Clone, Debug)]
pub(crate) struct SourceObservation {
    shared: Weak<Shared>,
}
impl PartialEq for SourceObservation {
    fn eq(&self, other: &Self) -> bool {
        self.shared.ptr_eq(&other.shared)
    }
}
impl Eq for SourceObservation {}

impl DomHostedSource {
    pub(crate) fn next_operation_slot(
        &self,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<usize, DomHostedError> {
        let state = self.lock()?;
        let rows = &state.banks[Role::Operation.index()];
        for (index, row) in rows.iter().enumerate() {
            work.step(cancel)?;
            if !row.live && !row.retired {
                return Ok(index);
            }
        }
        Ok(rows.len())
    }
    pub(crate) fn begin_operation(
        &mut self,
        work: &Work,
        revisions: u64,
        scratch_base: usize,
        cancel: &AtomicBool,
    ) -> Result<DomHostedOperationId, DomHostedError> {
        {
            let mut state = self.lock()?;
            let reserved = state
                .reserved_revisions
                .checked_add(revisions)
                .ok_or(DomHostedError::RepresentationExhausted)?;
            state
                .revision
                .0
                .checked_add(reserved)
                .ok_or(DomHostedError::RepresentationExhausted)?;
            state.reserved_revisions = reserved;
        }
        let issued = self.issue(Role::Operation, work, true, None, scratch_base, cancel);
        match issued {
            Ok((slot, generation)) => {
                self.lock()?.banks[Role::Operation.index()][slot].reserved_revisions =
                    revisions - 1;
                Ok(DomHostedOperationId {
                    source: self.id(),
                    slot,
                    generation,
                })
            }
            Err(error) => {
                self.lock()?.reserved_revisions -= revisions;
                Err(error)
            }
        }
    }
    /// Remaining revision credit owned by this exact original operation/phase.
    /// Callers preflight canonical headroom before this metered issuer read.
    pub(crate) fn operation_reserved_revisions(
        &self,
        request: &Request,
        cancel: &AtomicBool,
    ) -> Result<u64, DomHostedError> {
        request.work.step(cancel)?;
        if request.key.operation.source != self.id() {
            return Err(DomHostedError::ForeignSource);
        }
        let state = self.lock()?;
        let row = state.banks[Role::Operation.index()]
            .get(request.key.operation.slot)
            .filter(|row| {
                row.live
                    && row.generation == request.key.operation.generation
                    && row.phase == request.key.phase
            })
            .ok_or(DomHostedError::WrongPhase)?;
        Ok(row.reserved_revisions)
    }
    pub(crate) fn reserve_operation_revisions(
        &self,
        request: &Request,
        additional: u64,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        request.work.step(cancel)?;
        if request.key.operation.source != self.id() {
            return Err(DomHostedError::ForeignSource);
        }
        let mut state = self.lock()?;
        // An admitted C/H can still need bookkeeping after its original BC
        // retires. Reserve only for the exact live operation and phase; this
        // grants no current BC/Global execution authority. Keep qualification
        // and credit mutation under this same issuer lock.
        let operation = state.banks[Role::Operation.index()]
            .get(request.key.operation.slot)
            .filter(|row| {
                row.live
                    && row.generation == request.key.operation.generation
                    && row.phase == request.key.phase
            })
            .ok_or(DomHostedError::WrongPhase)?;
        let owned = operation
            .reserved_revisions
            .checked_add(additional)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let reserved = state
            .reserved_revisions
            .checked_add(additional)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        state
            .revision
            .0
            .checked_add(reserved)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        state.banks[Role::Operation.index()][request.key.operation.slot].reserved_revisions = owned;
        state.reserved_revisions = reserved;
        Ok(())
    }
    pub(crate) fn permit<'a>(
        &'a self,
        request: &Request,
        revision: DomHostedSourceRevision,
        finish: bool,
        cancel: &AtomicBool,
    ) -> Result<PhasePermit<'a>, DomHostedError> {
        request.work.step(cancel)?;
        if request.key.operation.source != self.id() {
            return Err(DomHostedError::ForeignSource);
        }
        let state = self.lock()?;
        if state.revision != revision {
            return Err(DomHostedError::StaleBasis);
        }
        state.banks[Role::Operation.index()]
            .get(request.key.operation.slot)
            .filter(|slot| {
                slot.live
                    && slot.generation == request.key.operation.generation
                    && slot.phase == request.key.phase
            })
            .ok_or(DomHostedError::WrongPhase)?;
        let owned =
            state.banks[Role::Operation.index()][request.key.operation.slot].reserved_revisions;
        let consumed = if finish { owned } else { 1 };
        if owned == 0 {
            return Err(DomHostedError::WrongPhase);
        }
        let reserved = state
            .reserved_revisions
            .checked_sub(consumed)
            .ok_or(DomHostedError::WrongPhase)?;
        let next = state
            .revision
            .0
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        next.checked_add(reserved)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let phase = request
            .key
            .phase
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        Ok(PhasePermit {
            state,
            slot: request.key.operation.slot,
            next,
            phase,
            finish,
            consumed,
            retire: None,
        })
    }
    pub(crate) fn new(
        id: DomHostedSourceId,
        limits: DomHostedLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomHostedError> {
        crate::store::check_cancel(cancel)?;
        let bytes = std::mem::size_of::<Shared>();
        admit(
            bytes,
            limits.retained_bytes,
            DomHostedResource::RetainedBytes,
        )?;
        Ok(Self {
            limits,
            active_work: Weak::new(),
            shared: Arc::new(Shared {
                id,
                bytes: AtomicUsize::new(bytes),
                registries: AtomicUsize::new(0),
                runtime_bytes: AtomicUsize::new(0),
                issuance: Mutex::new(Issuance {
                    alive: true,
                    revision: DomHostedSourceRevision(0),
                    banks: std::array::from_fn(|_| Vec::new()),
                    reserved_revisions: 0,
                }),
            }),
        })
    }
    pub fn id(&self) -> DomHostedSourceId {
        self.shared.id
    }
    pub fn limits(&self) -> DomHostedLimits {
        self.limits
    }
    pub fn revision(&self) -> Result<DomHostedSourceRevision, DomHostedError> {
        Ok(self.lock()?.revision)
    }
    pub(crate) fn observe(&self) -> SourceObservation {
        SourceObservation {
            shared: Arc::downgrade(&self.shared),
        }
    }
    pub(crate) fn retain(
        &self,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<Retained, DomHostedError> {
        crate::store::check_cancel(cancel)?;
        drop(self.lock()?);
        self.shared
            .bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_add(bytes)
                    .filter(|next| *next <= self.limits.retained_bytes)
            })
            .map_err(|old| {
                if old.checked_add(bytes).is_none() {
                    DomHostedError::RepresentationExhausted
                } else {
                    DomHostedError::Limit {
                        resource: DomHostedResource::RetainedBytes,
                        limit: self.limits.retained_bytes,
                    }
                }
            })?;
        Ok(Retained {
            shared: Arc::downgrade(&self.shared),
            bytes,
        })
    }
    pub(crate) fn remaining_bytes(&self) -> Result<usize, DomHostedError> {
        drop(self.lock()?);
        Ok(self.limits.retained_bytes - self.shared.bytes.load(Ordering::Relaxed))
    }
    pub(crate) fn work(&self, cancel: &AtomicBool) -> Result<Arc<Work>, DomHostedError> {
        let retained = self.retain(
            std::mem::size_of::<Work>() + 2 * std::mem::size_of::<AtomicU64>(),
            cancel,
        )?;
        Ok(Arc::new(Work {
            spent: Arc::new(AtomicU64::new(0)),
            limit: Arc::new(AtomicU64::new(self.limits.work)),
            _retained: Some(retained),
        }))
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Issuance>, DomHostedError> {
        let state = self
            .shared
            .issuance
            .lock()
            .map_err(|_| DomHostedError::SourcePoisoned)?;
        if !state.alive {
            return Err(DomHostedError::SourceExpired);
        }
        Ok(state)
    }
    fn issue(
        &mut self,
        role: Role,
        work: &Work,
        consume_reserved: bool,
        request: Option<(RequestKey, u8)>,
        scratch_base: usize,
        cancel: &AtomicBool,
    ) -> Result<(usize, u64), DomHostedError> {
        work.step(cancel)?;
        let limits = self.limits;
        admit(
            scratch_base,
            limits.scratch_bytes,
            DomHostedResource::ScratchBytes,
        )?;
        let mut state = self.lock()?;
        if let Some((key, bit)) = request {
            let operation = state.banks[Role::Operation.index()]
                .get(key.operation.slot)
                .filter(|slot| {
                    slot.live
                        && slot.generation == key.operation.generation
                        && slot.phase == key.phase
                })
                .ok_or(DomHostedError::WrongPhase)?;
            if operation.issued & bit != 0
                || (consume_reserved && operation.reserved_revisions == 0)
            {
                return Err(DomHostedError::WrongPhase);
            }
        }
        let next_revision = state
            .revision
            .0
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let reserved = state
            .reserved_revisions
            .checked_sub(u64::from(consume_reserved))
            .ok_or(DomHostedError::WrongPhase)?;
        next_revision
            .checked_add(reserved)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let bank = &state.banks[role.index()];
        let mut free = None;
        let mut live = 0usize;
        for (index, slot) in bank.iter().enumerate() {
            work.step(cancel)?;
            if slot.live {
                live += 1;
            } else if !slot.retired && free.is_none() {
                free = Some(index);
            }
        }
        let (limit, resource) = role.bound(limits.records);
        admit(
            live.checked_add(1)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            limit,
            resource,
        )?;
        let result = if let Some(index) = free {
            let slot = &mut state.banks[role.index()][index];
            slot.live = true;
            slot.phase = 0;
            slot.issued = 0;
            slot.creator = request.map(|(key, _)| key);
            slot.reserved_revisions = 0;
            slot.work = matches!(role, Role::BrowsingContext | Role::Operation)
                .then(|| WorkObservation::new(work));
            (index, slot.generation)
        } else {
            let required = bank
                .len()
                .checked_add(1)
                .ok_or(DomHostedError::RepresentationExhausted)?;
            if required > bank.capacity() {
                let bytes = required
                    .checked_mul(std::mem::size_of::<IdentitySlot>())
                    .ok_or(DomHostedError::RepresentationExhausted)?;
                admit(
                    bytes
                        .checked_add(std::mem::size_of::<Vec<IdentitySlot>>())
                        .and_then(|bytes| bytes.checked_add(scratch_base))
                        .ok_or(DomHostedError::RepresentationExhausted)?,
                    limits.scratch_bytes,
                    DomHostedResource::ScratchBytes,
                )?;
                admit(
                    self.shared
                        .bytes
                        .load(Ordering::Relaxed)
                        .checked_add(bytes)
                        .ok_or(DomHostedError::RepresentationExhausted)?,
                    limits.retained_bytes,
                    DomHostedResource::RetainedBytes,
                )?;
                let mut replacement = Vec::new();
                replacement
                    .try_reserve_exact(required)
                    .map_err(|_| DomHostedError::AllocationFailed)?;
                let actual = replacement
                    .capacity()
                    .checked_mul(std::mem::size_of::<IdentitySlot>())
                    .ok_or(DomHostedError::RepresentationExhausted)?;
                admit(
                    actual
                        .checked_add(std::mem::size_of::<Vec<IdentitySlot>>())
                        .and_then(|bytes| bytes.checked_add(scratch_base))
                        .ok_or(DomHostedError::RepresentationExhausted)?,
                    limits.scratch_bytes,
                    DomHostedResource::ScratchBytes,
                )?;
                admit(
                    self.shared
                        .bytes
                        .load(Ordering::Relaxed)
                        .checked_add(actual)
                        .ok_or(DomHostedError::RepresentationExhausted)?,
                    limits.retained_bytes,
                    DomHostedResource::RetainedBytes,
                )?;
                for slot in &state.banks[role.index()] {
                    work.step(cancel)?;
                    replacement.push(slot.clone());
                }
                let old_bytes = state.banks[role.index()]
                    .capacity()
                    .checked_mul(std::mem::size_of::<IdentitySlot>())
                    .ok_or(DomHostedError::RepresentationExhausted)?;
                state.banks[role.index()] = replacement;
                self.shared
                    .bytes
                    .fetch_add(actual - old_bytes, Ordering::Relaxed);
            }
            let bank = &mut state.banks[role.index()];
            let index = bank.len();
            bank.push(IdentitySlot {
                generation: 1,
                live: true,
                retired: false,
                phase: 0,
                issued: 0,
                creator: request.map(|(key, _)| key),
                work: matches!(role, Role::BrowsingContext | Role::Operation)
                    .then(|| WorkObservation::new(work)),
                reserved_revisions: 0,
            });
            (index, 1)
        };
        state.revision = DomHostedSourceRevision(next_revision);
        state.reserved_revisions = reserved;
        if let Some((key, bit)) = request {
            let operation = &mut state.banks[Role::Operation.index()][key.operation.slot];
            operation.issued |= bit;
            if consume_reserved {
                operation.reserved_revisions -= 1;
            }
        }
        Ok(result)
    }
    pub(crate) fn retire(
        &mut self,
        role: Role,
        source: DomHostedSourceId,
        index: usize,
        generation: u64,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        crate::store::check_cancel(cancel)?;
        if source != self.id() {
            return Err(DomHostedError::ForeignSource);
        }
        let mut state = self.lock()?;
        let next = state
            .revision
            .0
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        next.checked_add(state.reserved_revisions)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let slot = state.banks[role.index()]
            .get_mut(index)
            .filter(|slot| slot.live && slot.generation == generation)
            .ok_or(DomHostedError::StaleIdentity)?;
        slot.live = false;
        if let Some(next) = generation.checked_add(1) {
            slot.generation = next;
        } else {
            slot.retired = true;
        }
        state.revision = DomHostedSourceRevision(next);
        Ok(())
    }
}

impl Drop for DomHostedSource {
    fn drop(&mut self) {
        if let Ok(mut state) = self.shared.issuance.lock() {
            state.alive = false;
        }
    }
}

impl DomHostedSource {
    /// Retire the actual Host only after its owner proved empty membership and
    /// completed all retained creation, task and destruction obligations.
    pub fn retire_host(
        &mut self,
        id: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        self.retire(Role::Host, id.source, id.slot, id.generation, cancel)
    }
}

impl DomHostedSource {
    /// Identity for an admitted actual host. This is a complete host-creation
    /// operation; a later browsing-context creation owns a distinct work cursor.
    pub fn issue_host(&mut self, cancel: &AtomicBool) -> Result<DomHostedHostId, DomHostedError> {
        let work = self.work(cancel)?;
        let (slot, generation) = self.issue(Role::Host, &work, false, None, 0, cancel)?;
        Ok(DomHostedHostId {
            source: self.id(),
            slot,
            generation,
        })
    }

    /// Reserve the actual new browsing context before its native construction.
    /// The returned preparation retains all spent work across later phases.
    pub fn prepare_top_level(
        &mut self,
        host: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTopLevelPreparation, DomHostedError> {
        if self.active_work.upgrade().is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        self.observe()
            .validate(host.source, Role::Host, host.slot, host.generation)?;
        let work = self.work(cancel)?;
        let (slot, generation) =
            self.issue(Role::BrowsingContext, &work, false, None, 0, cancel)?;
        self.active_work = Arc::downgrade(&work);
        Ok(DomHostedTopLevelPreparation {
            host,
            context: DomHostedBrowsingContextId {
                source: self.id(),
                slot,
                generation,
            },
            work,
        })
    }

    /// Attest the actual native browsing-context construction prefix.
    /// Possession of the original issuer is producer authority; this method
    /// does not sample a clock or instantiate an external resource.
    pub fn seal_top_level(
        &self,
        preparation: &mut Option<DomHostedTopLevelPreparation>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedPreparedTopLevel, DomHostedError> {
        let current = preparation
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        current.work.step(cancel)?;
        self.observe().validate(
            current.context.source,
            Role::BrowsingContext,
            current.context.slot,
            current.context.generation,
        )?;
        Ok(DomHostedPreparedTopLevel {
            preparation: preparation.take().expect("validated preparation"),
        })
    }
}

impl DomHostedSource {
    /// Changes admission without replacing the original issuer, cursor or spent
    /// work. Rejected lowering leaves every old bound and counter unchanged.
    pub fn set_limits(
        &mut self,
        limits: DomHostedLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        crate::store::check_cancel(cancel)?;
        let state = self.lock()?;
        let work = self.active_work.upgrade();
        if let Some(work) = &work
            && work.spent.load(Ordering::Relaxed) > limits.work
        {
            return Err(DomHostedError::WorkLimit { limit: limits.work });
        }
        admit(
            self.shared.bytes.load(Ordering::Relaxed),
            limits.retained_bytes,
            DomHostedResource::RetainedBytes,
        )?;
        admit(
            self.shared.registries.load(Ordering::Relaxed),
            limits.records.registries,
            DomHostedResource::Registries,
        )?;
        for role in [
            Role::Host,
            Role::BrowsingContext,
            Role::Navigable,
            Role::Entry,
            Role::Global,
            Role::Operation,
        ] {
            let mut live = 0usize;
            for slot in &state.banks[role.index()] {
                crate::store::check_cancel(cancel)?;
                if slot.live {
                    if let Some(work) = &slot.work {
                        work.validate_limit(limits.work)?;
                    }
                    live = live
                        .checked_add(1)
                        .ok_or(DomHostedError::RepresentationExhausted)?;
                }
            }
            let (limit, resource) = role.bound(limits.records);
            admit(live, limit, resource)?;
        }
        // Every fallible check is complete before any original counter changes.
        for bank in &state.banks {
            for slot in bank {
                if slot.live
                    && let Some(work) = &slot.work
                {
                    work.set_limit(limits.work);
                }
            }
        }
        drop(state);
        if let Some(work) = work {
            work.limit.store(limits.work, Ordering::Relaxed);
        }
        self.limits = limits;
        Ok(())
    }
    pub fn retained_storage_bytes(&self) -> usize {
        self.shared.bytes.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub(crate) struct RegistryQuota {
    shared: Weak<Shared>,
}
impl Drop for RegistryQuota {
    fn drop(&mut self) {
        if let Some(shared) = self.shared.upgrade() {
            shared.registries.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
impl DomHostedSource {
    pub(crate) fn retain_registry(
        &self,
        cancel: &AtomicBool,
    ) -> Result<RegistryQuota, DomHostedError> {
        crate::store::check_cancel(cancel)?;
        self.shared
            .registries
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_add(1)
                    .filter(|next| *next <= self.limits.records.registries)
            })
            .map_err(|old| {
                if old == usize::MAX {
                    DomHostedError::RepresentationExhausted
                } else {
                    DomHostedError::Limit {
                        resource: DomHostedResource::Registries,
                        limit: self.limits.records.registries,
                    }
                }
            })?;
        Ok(RegistryQuota {
            shared: Arc::downgrade(&self.shared),
        })
    }
}

/// Original pre-policy creation custody. It is neither a live Document nor a
/// completed browsing-context/Global association.
#[derive(Debug)]
pub struct DomHostedTopLevelPreparation {
    pub(crate) host: DomHostedHostId,
    pub(crate) context: DomHostedBrowsingContextId,
    pub(crate) work: Arc<Work>,
}
impl DomHostedTopLevelPreparation {
    pub const fn host(&self) -> DomHostedHostId {
        self.host
    }
    pub const fn browsing_context(&self) -> DomHostedBrowsingContextId {
        self.context
    }
}
/// The same preparation after the original producer constructed its native BC.
#[derive(Debug)]
pub struct DomHostedPreparedTopLevel {
    pub(crate) preparation: DomHostedTopLevelPreparation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestKey {
    pub(crate) operation: DomHostedOperationId,
    pub(crate) phase: u64,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct ParentContext {
    pub(crate) document: crate::DomDocumentStamp,
    pub(crate) navigable: DomHostedNavigableId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) container: crate::NodeId,
    pub(crate) traversable: DomHostedNavigableId,
}
#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) key: RequestKey,
    pub(crate) host: DomHostedHostId,
    pub(crate) context: DomHostedBrowsingContextId,
    pub(crate) work: Arc<Work>,
    pub(crate) parent: Option<ParentContext>,
}
macro_rules! request {
    ($name:ident, $description:literal, $($field:ident : $ty:ty),* $(,)?) => {
        #[doc = $description]
        #[derive(Debug)]
        pub struct $name { pub(crate) request: Request, $(pub(crate) $field: $ty),* }
        impl $name {
            pub fn operation(&self) -> DomHostedOperationId { self.request.key.operation }
            pub fn host(&self) -> DomHostedHostId { self.request.host }
            pub fn browsing_context(&self) -> DomHostedBrowsingContextId { self.request.context }
        }
    };
}
request!(DomHostedPolicyRequest, "The exact source-qualified Permissions Policy phase.", origin: crate::DomOpaqueOrigin, embedder_policy:Option<Arc<DomHostedEmbedderPolicyInputs>>);
request!(DomHostedExecutionRequest, "Creation of the actual native agent, loop, Global and settings.",
    origin: crate::DomOpaqueOrigin, inputs: crate::document::HostedDocumentInputs);
request!(DomHostedMakeActiveRequest, "Make the actual Document/Global association active.",
    document: crate::DocumentId, global: DomHostedGlobalId);
request!(DomHostedFinishLoadingRequest, "The fresh completion sample from the same original epoch.",
    document: crate::DocumentId, global: DomHostedGlobalId, navigation_start: DomHostedMoment);
request!(DomHostedEntryRequest, "Actual initial entry, traversable and top-level host membership.",
    document: crate::DocumentId, global: DomHostedGlobalId);

impl DomHostedPolicyRequest {
    pub fn embedder_policy(&self) -> Option<&DomHostedEmbedderPolicyInputs> {
        self.embedder_policy.as_deref()
    }
    pub fn target_origin(&self) -> &crate::DomOpaqueOrigin {
        &self.origin
    }
    pub const fn profile(&self) -> DomInitialPermissionsPolicyProfile {
        DomInitialPermissionsPolicyProfile::FullscreenSelfOpaqueInitial
    }
    pub fn parent_document(&self) -> Option<crate::DocumentId> {
        self.request.parent.map(|parent| parent.document.document())
    }
    pub fn embedder(&self) -> Option<crate::NodeId> {
        self.request.parent.map(|parent| parent.container)
    }
}
impl DomHostedExecutionRequest {
    pub fn parent_global(&self) -> Option<DomHostedGlobalId> {
        self.request.parent.map(|parent| parent.global)
    }
    pub fn top_level_origin(&self) -> &crate::DomOpaqueOrigin {
        &self.inputs.0.top_origin
    }
    pub fn creation_url(&self) -> &surgeist_html_semantics::HtmlBaseUrlRecord {
        self.inputs.url()
    }
    pub fn top_level_creation_url(&self) -> &surgeist_html_semantics::HtmlBaseUrlRecord {
        self.inputs
            .0
            .top_creation_url
            .as_ref()
            .unwrap_or_else(|| self.inputs.url())
    }
    pub fn base_document_facts(&self) -> &surgeist_html_semantics::HtmlBaseDocumentFacts {
        &self.inputs.0.facts
    }
    pub fn opener_policy(&self) -> &surgeist_html::HtmlOpenerPolicy {
        &self.inputs.0.opener
    }
    pub fn origin(&self) -> &crate::DomOpaqueOrigin {
        &self.origin
    }
}
macro_rules! document_request {
    ($name:ident) => {
        impl $name {
            pub fn document(&self) -> crate::DocumentId {
                self.document
            }
            pub fn global(&self) -> DomHostedGlobalId {
                self.global
            }
        }
    };
}
document_request!(DomHostedMakeActiveRequest);
document_request!(DomHostedFinishLoadingRequest);
document_request!(DomHostedEntryRequest);
impl DomHostedFinishLoadingRequest {
    pub fn navigation_start(&self) -> DomHostedMoment {
        self.navigation_start
    }
}

/// Non-Clone completed policy attestation from the original private producer.
#[derive(Debug)]
pub struct DomHostedPolicyPublication {
    pub(crate) key: RequestKey,
    pub(crate) revision: DomHostedSourceRevision,
    pub(crate) inherited: DomFullscreenInherited,
}
#[derive(Debug)]
pub(crate) enum PublicationValue {
    NativeExecution {
        global: DomHostedGlobalId,
        navigation_start: DomHostedMoment,
    },
    MakeActive,
    FinishLoading(DomHostedMoment),
    InitialEntry {
        entry: DomHostedEntryId,
        navigable: DomHostedNavigableId,
        initialization: DomHostedEntryInitialization,
        _retained: Retained,
    },
}
/// One non-Clone original phase result. Successful acceptance consumes it once.
#[derive(Debug)]
pub struct DomHostedCreationPublication {
    pub(crate) key: RequestKey,
    pub(crate) revision: DomHostedSourceRevision,
    pub(crate) value: PublicationValue,
}

impl DomHostedSource {
    fn validate_request(
        &self,
        request: &Request,
        cancel: &AtomicBool,
    ) -> Result<DomHostedSourceRevision, DomHostedError> {
        request.work.step(cancel)?;
        let source = self.observe();
        source.validate(
            request.key.operation.source,
            Role::Operation,
            request.key.operation.slot,
            request.key.operation.generation,
        )?;
        let state = self.lock()?;
        if state.banks[Role::Operation.index()][request.key.operation.slot].phase
            != request.key.phase
        {
            return Err(DomHostedError::WrongPhase);
        }
        drop(state);
        source.validate(
            request.host.source,
            Role::Host,
            request.host.slot,
            request.host.generation,
        )?;
        source.validate(
            request.context.source,
            Role::BrowsingContext,
            request.context.slot,
            request.context.generation,
        )
    }
    pub(crate) fn validate_created(
        &self,
        role: Role,
        source: DomHostedSourceId,
        slot: usize,
        generation: u64,
        key: RequestKey,
    ) -> Result<(), DomHostedError> {
        if source != self.id() {
            return Err(DomHostedError::ForeignSource);
        }
        let state = self.lock()?;
        state.banks[role.index()]
            .get(slot)
            .filter(|slot| slot.live && slot.generation == generation && slot.creator == Some(key))
            .ok_or(DomHostedError::StaleIdentity)?;
        Ok(())
    }
    pub fn seal_permissions_policy(
        &self,
        request: &DomHostedPolicyRequest,
        inherited: DomFullscreenInherited,
        cancel: &AtomicBool,
    ) -> Result<DomHostedPolicyPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        Ok(DomHostedPolicyPublication {
            key: request.request.key,
            revision,
            inherited,
        })
    }
    pub fn issue_global(
        &mut self,
        request: &DomHostedExecutionRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedGlobalId, DomHostedError> {
        self.validate_request(&request.request, cancel)?;
        let (slot, generation) = self.issue(
            Role::Global,
            &request.request.work,
            true,
            Some((request.request.key, 1)),
            0,
            cancel,
        )?;
        Ok(DomHostedGlobalId {
            source: self.id(),
            slot,
            generation,
        })
    }
    pub fn seal_native_execution(
        &self,
        request: &DomHostedExecutionRequest,
        global: DomHostedGlobalId,
        navigation_start_microseconds: u64,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreationPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        self.observe()
            .validate(global.source, Role::Global, global.slot, global.generation)?;
        self.validate_created(
            Role::Global,
            global.source,
            global.slot,
            global.generation,
            request.request.key,
        )?;
        if !navigation_start_microseconds.is_multiple_of(100) {
            return Err(DomHostedError::InvalidClockSample);
        }
        Ok(DomHostedCreationPublication {
            key: request.request.key,
            revision,
            value: PublicationValue::NativeExecution {
                global,
                navigation_start: DomHostedMoment {
                    source: self.id(),
                    microseconds: navigation_start_microseconds,
                },
            },
        })
    }
    pub fn seal_make_active(
        &self,
        request: &DomHostedMakeActiveRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreationPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        self.observe().validate(
            request.global.source,
            Role::Global,
            request.global.slot,
            request.global.generation,
        )?;
        Ok(DomHostedCreationPublication {
            key: request.request.key,
            revision,
            value: PublicationValue::MakeActive,
        })
    }
    pub fn seal_finish_loading(
        &self,
        request: &DomHostedFinishLoadingRequest,
        sampled_microseconds: u64,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreationPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        if !sampled_microseconds.is_multiple_of(100) {
            return Err(DomHostedError::InvalidClockSample);
        }
        if sampled_microseconds < request.navigation_start.microseconds {
            return Err(DomHostedError::ClockRegression);
        }
        Ok(DomHostedCreationPublication {
            key: request.request.key,
            revision,
            value: PublicationValue::FinishLoading(DomHostedMoment {
                source: self.id(),
                microseconds: sampled_microseconds,
            }),
        })
    }
    pub fn issue_entry(
        &mut self,
        request: &DomHostedEntryRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedEntryId, DomHostedError> {
        self.validate_request(&request.request, cancel)?;
        let (slot, generation) = self.issue(
            Role::Entry,
            &request.request.work,
            true,
            Some((request.request.key, 1)),
            0,
            cancel,
        )?;
        Ok(DomHostedEntryId {
            source: self.id(),
            slot,
            generation,
        })
    }
    pub fn issue_navigable(
        &mut self,
        request: &DomHostedEntryRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedNavigableId, DomHostedError> {
        self.validate_request(&request.request, cancel)?;
        let (slot, generation) = self.issue(
            Role::Navigable,
            &request.request.work,
            true,
            Some((request.request.key, 2)),
            0,
            cancel,
        )?;
        Ok(DomHostedNavigableId {
            source: self.id(),
            slot,
            generation,
        })
    }
    pub fn seal_initial_entry(
        &self,
        request: &DomHostedEntryRequest,
        entry: DomHostedEntryId,
        navigable: DomHostedNavigableId,
        initialization: &mut Option<DomHostedEntryInitialization>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedCreationPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        self.observe()
            .validate(entry.source, Role::Entry, entry.slot, entry.generation)?;
        self.validate_created(
            Role::Entry,
            entry.source,
            entry.slot,
            entry.generation,
            request.request.key,
        )?;
        self.observe().validate(
            navigable.source,
            Role::Navigable,
            navigable.slot,
            navigable.generation,
        )?;
        self.validate_created(
            Role::Navigable,
            navigable.source,
            navigable.slot,
            navigable.generation,
            request.request.key,
        )?;
        let input = initialization
            .as_ref()
            .ok_or(DomHostedError::MissingPublication)?;
        for value in [&input.navigation_key, &input.navigation_id] {
            for (index, unit) in value.iter().enumerate() {
                request.request.work.step(cancel)?;
                let valid = if [8, 13, 18, 23].contains(&index) {
                    *unit == u16::from(b'-')
                } else {
                    matches!(*unit, 48..=57 | 97..=102)
                };
                if !valid
                    || (index == 14 && *unit != u16::from(b'4'))
                    || (index == 19 && !matches!(*unit, 56 | 57 | 97 | 98))
                {
                    return Err(DomHostedError::InvalidEntryInitialization);
                }
            }
        }
        let retained = self.retain(size_of::<DomHostedEntryInitialization>(), cancel)?;
        Ok(DomHostedCreationPublication {
            key: request.request.key,
            revision,
            value: PublicationValue::InitialEntry {
                entry,
                navigable,
                initialization: initialization
                    .take()
                    .expect("validated actual initial entry strings"),
                _retained: retained,
            },
        })
    }
}

pub(crate) struct PhasePermit<'a> {
    state: std::sync::MutexGuard<'a, Issuance>,
    slot: usize,
    next: u64,
    phase: u64,
    finish: bool,
    consumed: u64,
    retire: Option<DomHostedCreation>,
}
impl PhasePermit<'_> {
    pub(crate) fn retire_navigable(
        mut self,
        creation: DomHostedCreation,
    ) -> Result<Self, DomHostedError> {
        for (role, source, slot, generation) in [
            (
                Role::BrowsingContext,
                creation.context.source,
                creation.context.slot,
                creation.context.generation,
            ),
            (
                Role::Global,
                creation.global.source,
                creation.global.slot,
                creation.global.generation,
            ),
            (
                Role::Entry,
                creation.entry.source,
                creation.entry.slot,
                creation.entry.generation,
            ),
            (
                Role::Navigable,
                creation.navigable.source,
                creation.navigable.slot,
                creation.navigable.generation,
            ),
        ] {
            if source != creation.host.source {
                return Err(DomHostedError::ForeignSource);
            }
            self.state.banks[role.index()]
                .get(slot)
                .filter(|row| row.live && row.generation == generation)
                .ok_or(DomHostedError::StaleIdentity)?;
        }
        self.retire = Some(creation);
        Ok(self)
    }
    pub(crate) fn commit(mut self) {
        if let Some(creation) = self.retire {
            for (role, slot) in [
                (Role::BrowsingContext, creation.context.slot),
                (Role::Global, creation.global.slot),
                (Role::Entry, creation.entry.slot),
                (Role::Navigable, creation.navigable.slot),
            ] {
                let row = &mut self.state.banks[role.index()][slot];
                row.live = false;
                if let Some(next) = row.generation.checked_add(1) {
                    row.generation = next;
                } else {
                    row.retired = true;
                }
            }
        }
        let operation = &mut self.state.banks[Role::Operation.index()][self.slot];
        if self.finish {
            operation.live = false;
            if let Some(next) = operation.generation.checked_add(1) {
                operation.generation = next;
            } else {
                operation.retired = true;
            }
        } else {
            operation.phase = self.phase;
            operation.issued = 0;
        }
        self.state.banks[Role::Operation.index()][self.slot].reserved_revisions -= self.consumed;
        self.state.reserved_revisions -= self.consumed;
        self.state.revision = DomHostedSourceRevision(self.next);
    }
}

/// Borrowed actual next phase; no request can be independently constructed.
#[derive(Debug)]
pub enum DomHostedCreationStep<'a> {
    PermissionsPolicy(&'a DomHostedPolicyRequest),
    NativeExecution(&'a DomHostedExecutionRequest),
    MakeActive(&'a DomHostedMakeActiveRequest),
    FinishLoading(&'a DomHostedFinishLoadingRequest),
    InitialEntry(&'a DomHostedEntryRequest),
    Complete(crate::DocumentId),
}
/// Permission to resume the one operation retained by the original Dom.
/// Dropping it does not complete, cancel or discard the original continuation.
#[derive(Debug)]
pub struct DomHostedCreationLease {
    pub(crate) operation: DomHostedOperationId,
    pub(crate) completed: Option<DomHostedCreation>,
}
impl DomHostedCreationLease {
    pub fn operation(&self) -> DomHostedOperationId {
        self.operation
    }
}
/// Completed original creation identities. These observations do not retain or
/// replace their mutable external owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedCreation {
    pub(crate) document: crate::DocumentId,
    pub(crate) host: DomHostedHostId,
    pub(crate) context: DomHostedBrowsingContextId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) entry: DomHostedEntryId,
    pub(crate) navigable: DomHostedNavigableId,
}
impl DomHostedCreation {
    pub const fn document(self) -> crate::DocumentId {
        self.document
    }
    pub const fn host(self) -> DomHostedHostId {
        self.host
    }
    pub const fn browsing_context(self) -> DomHostedBrowsingContextId {
        self.context
    }
    pub const fn global(self) -> DomHostedGlobalId {
        self.global
    }
    pub const fn entry(self) -> DomHostedEntryId {
        self.entry
    }
    pub const fn navigable(self) -> DomHostedNavigableId {
        self.navigable
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentPhase {
    Constructed,
    Active,
    Loaded(DomHostedMoment),
    InitialEntry {
        entry: DomHostedEntryId,
        navigable: DomHostedNavigableId,
        loaded: DomHostedMoment,
    },
    Retired {
        operation: DomHostedOperationId,
    },
}

/// The one captured canonical association. Runtime's mutable objects are not
/// mirrored here. Only controlled phase acceptance can establish these joins.
#[derive(Debug)]
pub(crate) struct DocumentAssociation {
    pub(crate) source: SourceObservation,
    pub(crate) host: DomHostedHostId,
    pub(crate) context: DomHostedBrowsingContextId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) navigation_start: DomHostedMoment,
    pub(crate) policy: DomInitialPermissionsPolicy,
    pub(crate) phase: DocumentPhase,
    pub(crate) _retained: Retained,
}
impl PartialEq for DocumentAssociation {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.host == other.host
            && self.context == other.context
            && self.global == other.global
            && self.navigation_start == other.navigation_start
            && self.policy == other.policy
            && self.phase == other.phase
    }
}
impl Eq for DocumentAssociation {}
impl DocumentAssociation {
    pub(crate) fn validate_current(&self) -> Result<DomHostedSourceRevision, DomHostedError> {
        if matches!(self.phase, DocumentPhase::Retired { .. }) {
            return Err(DomHostedError::StaleIdentity);
        }
        let source = self.global.source;
        self.source
            .validate(source, Role::Host, self.host.slot, self.host.generation)?;
        self.source.validate(
            source,
            Role::BrowsingContext,
            self.context.slot,
            self.context.generation,
        )?;
        if let DocumentPhase::InitialEntry {
            entry, navigable, ..
        } = self.phase
        {
            self.source
                .validate(source, Role::Entry, entry.slot, entry.generation)?;
            self.source.validate(
                source,
                Role::Navigable,
                navigable.slot,
                navigable.generation,
            )?;
        }
        self.source.validate(
            source,
            Role::Global,
            self.global.slot,
            self.global.generation,
        )
    }
    pub(crate) fn validate_live(&self) -> Result<DomHostedSourceRevision, DomHostedError> {
        let revision = self.validate_current()?;
        if !matches!(self.phase, DocumentPhase::InitialEntry { .. }) {
            return Err(DomHostedError::IncompleteOperation);
        }
        Ok(revision)
    }
}

impl SourceObservation {
    /// Qualify an existing original association's issuer without granting a
    /// live role or substituting current membership for historical provenance.
    pub(crate) fn validate_issuer(
        &self,
        id: DomHostedSourceId,
    ) -> Result<DomHostedSourceRevision, DomHostedError> {
        let shared = self.shared.upgrade().ok_or(DomHostedError::SourceExpired)?;
        if shared.id != id {
            return Err(DomHostedError::ForeignSource);
        }
        let state = shared
            .issuance
            .lock()
            .map_err(|_| DomHostedError::SourcePoisoned)?;
        if !state.alive {
            return Err(DomHostedError::SourceExpired);
        }
        Ok(state.revision)
    }
    pub(crate) fn validate(
        &self,
        id: DomHostedSourceId,
        role: Role,
        slot: usize,
        generation: u64,
    ) -> Result<DomHostedSourceRevision, DomHostedError> {
        let shared = self.shared.upgrade().ok_or(DomHostedError::SourceExpired)?;
        if shared.id != id {
            return Err(DomHostedError::ForeignSource);
        }
        let state = shared
            .issuance
            .lock()
            .map_err(|_| DomHostedError::SourcePoisoned)?;
        if !state.alive {
            return Err(DomHostedError::SourceExpired);
        }
        state.banks[role.index()]
            .get(slot)
            .filter(|slot| slot.live && slot.generation == generation)
            .ok_or(DomHostedError::StaleIdentity)?;
        Ok(state.revision)
    }
}

pub(crate) fn admit(
    value: usize,
    limit: usize,
    resource: DomHostedResource,
) -> Result<(), DomHostedError> {
    if value > limit {
        Err(DomHostedError::Limit { resource, limit })
    } else {
        Ok(())
    }
}

/// Sealed observation of actual successful cleanup of this exact Document and
/// Global generation. Duplicating evidence does not run or acknowledge a job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostedDocumentDestructionProof {
    pub(crate) source: DomHostedSourceId,
    pub(crate) document: crate::DocumentId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) operation: DomHostedOperationId,
}
impl DomHostedDocumentDestructionProof {
    pub fn document(&self) -> crate::DocumentId {
        self.document
    }
    pub fn global(&self) -> DomHostedGlobalId {
        self.global
    }
    pub(crate) fn validate(
        &self,
        source: &DomHostedSource,
        document: crate::DocumentId,
        global: DomHostedGlobalId,
    ) -> Result<(), DomHostedError> {
        source.revision()?;
        if self.source != source.id() {
            return Err(DomHostedError::ForeignSource);
        }
        if self.document != document || self.global != global {
            return Err(DomHostedError::WrongPhase);
        }
        Ok(())
    }
}

/// The original invocation's captured child documents, independent from any
/// later unlink or nulling of the current active DocumentState.
#[derive(Debug)]
pub(crate) struct CapturedChildren {
    pub(crate) values: Vec<DomHostedCreation>,
    pub(crate) _retained: Retained,
}

/// Borrowed source-qualified task/cleanup input for the actual no-child top level.
#[derive(Debug)]
pub struct DomHostedDestructionRequest {
    pub(crate) request: Request,
    pub(crate) creation: DomHostedCreation,
    pub(crate) children: Arc<CapturedChildren>,
    pub(crate) invocation: DomHostedOperationId,
    pub(crate) traversable: DomHostedNavigableId,
}
impl DomHostedDestructionRequest {
    pub fn operation(&self) -> DomHostedOperationId {
        self.request.key.operation
    }
    pub fn host(&self) -> DomHostedHostId {
        self.creation.host
    }
    pub fn browsing_context(&self) -> DomHostedBrowsingContextId {
        self.creation.context
    }
    pub fn global(&self) -> DomHostedGlobalId {
        self.creation.global
    }
    pub fn document(&self) -> crate::DocumentId {
        self.creation.document
    }
    pub fn entry(&self) -> DomHostedEntryId {
        self.creation.entry
    }
    pub fn navigable(&self) -> DomHostedNavigableId {
        self.creation.navigable
    }
    pub fn traversable(&self) -> DomHostedNavigableId {
        self.traversable
    }
    pub fn child_navigable_count(&self) -> usize {
        self.children.values.len()
    }
    /// These are the exact created identities captured by this invocation, not
    /// a later query of current active Documents after cleanup.
    pub fn child_documents(&self) -> &[DomHostedCreation] {
        &self.children.values
    }
}
#[derive(Debug)]
pub struct DomHostedDestructionPublication {
    pub(crate) key: RequestKey,
    pub(crate) revision: DomHostedSourceRevision,
    pub(crate) cleanup: bool,
}
#[derive(Debug)]
pub enum DomHostedDestructionStep<'a> {
    EnqueueParentTask(&'a DomHostedDestructionRequest),
    Cleanup(&'a DomHostedDestructionRequest),
    Complete,
}
/// Dropping this permission does not discard or acknowledge retained task work.
#[derive(Debug)]
pub struct DomHostedDestructionLease {
    pub(crate) operation: DomHostedOperationId,
    pub(crate) completed: Option<DomHostedCreation>,
    pub(crate) proof: Option<DomHostedDocumentDestructionProof>,
}
impl DomHostedDestructionLease {
    pub fn destruction_proof(&self) -> Option<&DomHostedDocumentDestructionProof> {
        self.proof.as_ref()
    }
    pub fn operation(&self) -> DomHostedOperationId {
        self.operation
    }
}
impl DomHostedSource {
    /// Attests actual publication of the parent Global navigation task.
    pub fn seal_destruction_queued(
        &self,
        request: &DomHostedDestructionRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        Ok(DomHostedDestructionPublication {
            key: request.request.key,
            revision,
            cleanup: false,
        })
    }
    /// Attests the actual queued task body, checkpoint and external cleanup.
    /// Canonical acceptance independently checks its remaining owned obligations.
    pub fn seal_top_level_cleanup(
        &self,
        request: &DomHostedDestructionRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        Ok(DomHostedDestructionPublication {
            key: request.request.key,
            revision,
            cleanup: true,
        })
    }
}

#[cfg(test)]
mod tests;

/// Relative Source projection of actual original Host/Window/focus/watcher allocations.
/// This weak storage observation grants no execution authority and cannot keep
/// a disposed issuer alive. Drop represented backing before this guard.
#[derive(Debug)]
pub struct DomHostedRuntimeRetention {
    pub(crate) shared: Weak<Shared>,
    pub(crate) bytes: usize,
}
impl DomHostedRuntimeRetention {
    /// Merge two actual simultaneous extents of the same original issuer.
    pub fn absorb(&mut self, mut other: Self) -> Result<(), DomError> {
        if !self.shared.ptr_eq(&other.shared) {
            return Err(DomError::HostedSourceUnavailable);
        }
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        other.bytes = 0;
        Ok(())
    }
}
impl Drop for DomHostedRuntimeRetention {
    fn drop(&mut self) {
        if let Some(shared) = self.shared.upgrade() {
            shared
                .runtime_bytes
                .fetch_sub(self.bytes, Ordering::Relaxed);
        }
    }
}
impl SourceObservation {
    pub(crate) fn runtime_bytes(&self) -> usize {
        self.shared
            .upgrade()
            .map_or(0, |shared| shared.runtime_bytes.load(Ordering::Relaxed))
    }
}
