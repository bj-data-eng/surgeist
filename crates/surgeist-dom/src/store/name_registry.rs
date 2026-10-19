//! Owner-side exact name reservations. Semantic atoms retain no registry or
//! accounting owner. Pending rows and their strong operation guards survive
//! semantic rollback; only selected publication promotes them.
use super::{DomError, DomResource, bank, custody};
use crate::name::NameRecord;
use crate::{DocumentString, DomNameAtom, DomNameLimits, DomOwnerId};
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    mem::size_of,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

mod index;
mod ingress;
pub(in crate::store) use ingress::Normalized;
#[cfg(test)]
mod tests;
use index::{Index, Prepared};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Resource {
    RetainedBytes,
    Entries,
    Scratch,
}

type Result<T> = std::result::Result<T, DomError>;

fn check(limits: DomNameLimits, resource: Resource, actual: usize) -> Result<()> {
    let (resource, limit) = match resource {
        Resource::RetainedBytes => (DomResource::NameRetainedBytes, limits.retained_bytes()),
        Resource::Entries => (DomResource::NameEntries, limits.entries()),
        Resource::Scratch => (DomResource::NameScratch, limits.operation_scratch()),
    };
    if actual > limit {
        Err(DomError::ResourceLimit { resource, limit })
    } else {
        Ok(())
    }
}

pub(in crate::store) struct Admission<'a> {
    pub global: &'a mut dyn FnMut(custody::Usage) -> std::result::Result<(), DomError>,
    pub step: &'a mut dyn FnMut() -> std::result::Result<(), DomError>,
}

#[derive(Clone)]
pub(in crate::store) struct Registry(Arc<Mutex<State>>);
struct State {
    index: Index,
    index_leased: bool,
    index_entries: usize,
    leased_storage: custody::Usage,
    records: custody::Usage,
    controls: usize,
    limits: DomNameLimits,
    owner: DomOwnerId,
    hash: RandomState,
    registry: bank::CustodyRegistry,
    // Covers the actual Mutex<State> target and unique index allocations.
    // Semantic index data is destroyed before this affine owner.
    storage: custody::Debit,
}
impl State {
    fn usage(&self) -> std::result::Result<custody::Usage, DomError> {
        self.storage
            .usage()
            .checked_add(self.leased_storage)?
            .checked_add(self.records)?
            .checked_add(custody::Usage::new(self.controls, 0))
    }
}

struct IndexLease {
    index: Index,
    storage: custody::Debit,
    owner: Registry,
}
impl IndexLease {
    fn take(owner: &Registry) -> Result<Self> {
        let mut state = owner.lock();
        if state.index_leased {
            return Err(DomError::InvalidSourceOperation);
        }
        state.index_entries = state.index.entries();
        let usage = state.storage.usage();
        let storage = state
            .storage
            .split(usage)
            .expect("whole original index receipt");
        state.leased_storage = usage;
        state.index_leased = true;
        Ok(Self {
            index: std::mem::take(&mut state.index),
            storage,
            owner: owner.clone(),
        })
    }
}
impl Drop for IndexLease {
    fn drop(&mut self) {
        let mut state = self.owner.lock();
        assert!(state.index_leased, "one exclusive operational index lease");
        state.index = std::mem::take(&mut self.index);
        state.index_entries = state.index.entries();
        let storage = self
            .storage
            .split(self.storage.usage())
            .expect("whole index receipt return");
        state
            .storage
            .absorb(storage)
            .expect("original index ledger");
        state.leased_storage = custody::Usage::ZERO;
        state.index_leased = false;
    }
}

/// One directly measured owner-side allocation extent, constrained locally in
/// addition to its original global debit. Data fields precede this receipt.
pub(in crate::store) struct Attribution {
    state: Weak<Mutex<State>>,
    scope: Weak<Scope>,
    bytes: usize,
}
impl Attribution {
    fn new(owner: &Registry, scope: &Arc<Scope>, bytes: usize) -> Result<Self> {
        let mut value = Self {
            state: Arc::downgrade(&owner.0),
            scope: Arc::downgrade(scope),
            bytes: 0,
        };
        value.grow(bytes)?;
        Ok(value)
    }
    pub(in crate::store) fn grow(&mut self, bytes: usize) -> Result<()> {
        let Some(owner) = self.state.upgrade() else {
            return Ok(());
        };
        let scope = self.scope.upgrade();
        let mut state = owner.lock().expect("name receipt owner invariant");
        let next = state
            .usage()?
            .bytes()
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        check(state.limits, Resource::RetainedBytes, next)?;
        if let Some(scope) = &scope {
            scope.check_scratch(bytes)?;
        }
        state.controls = state
            .controls
            .checked_add(bytes)
            .expect("admitted name extent");
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .expect("admitted name receipt");
        if let Some(scope) = &scope {
            scope.add_scratch(bytes);
        }
        Ok(())
    }
    pub(in crate::store) fn absorb(&mut self, mut other: Self) {
        assert!(
            Weak::ptr_eq(&self.state, &other.state) && Weak::ptr_eq(&self.scope, &other.scope),
            "one original name formation"
        );
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .expect("admitted receipt union");
        other.bytes = 0;
    }
    fn transfer_pending(mut self, own: custody::Usage, moved: usize) {
        let bytes = own
            .bytes()
            .checked_add(moved)
            .expect("admitted pending extent");
        self.shrink(bytes);
        let owner = self
            .state
            .upgrade()
            .expect("pending operation retains owner");
        let mut state = owner.lock().expect("name receipt owner invariant");
        state.controls = state
            .controls
            .checked_sub(bytes)
            .expect("candidate local extent");
        state.records = state
            .records
            .checked_add(own)
            .expect("admitted record extent");
        state.leased_storage = state
            .leased_storage
            .checked_add(custody::Usage::new(moved, 0))
            .expect("admitted moved index extent");
        self.bytes = 0;
    }
    fn shrink(&mut self, bytes: usize) {
        let removed = self
            .bytes
            .checked_sub(bytes)
            .expect("owned measured subset");
        if let Some(owner) = self.state.upgrade() {
            owner.lock().expect("name receipt owner invariant").controls -= removed;
        }
        if let Some(scope) = self.scope.upgrade() {
            scope.release_scratch(removed);
        }
        self.bytes = bytes;
    }
}
impl Drop for Attribution {
    fn drop(&mut self) {
        self.shrink(0);
    }
}

/// Work and installed pending resources survive a Draft's Restore. Children
/// retain their ancestry, so resetting a child does not replenish its parent.
struct Scope {
    parent: Option<Arc<Scope>>,
    state: Weak<Mutex<State>>,
    identity: custody::MaterializationId,
    limits: Mutex<DomNameLimits>,
    work: Mutex<u64>,
    scratch: Mutex<usize>,
    _storage: custody::Debit,
}
impl Scope {
    fn limits(&self) -> DomNameLimits {
        *self.limits.lock().expect("name operation policy invariant")
    }
    fn step(&self, admission: &mut Admission<'_>) -> Result<()> {
        let mut scope = Some(self);
        while let Some(current) = scope {
            let mut work = current.work.lock().expect("name work invariant");
            let next = work
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            let limit = current.limits().operation_work();
            if next > limit {
                return Err(DomError::WorkLimit {
                    resource: DomResource::NameWork,
                    limit,
                });
            }
            *work = next;
            scope = current.parent.as_deref();
        }
        (admission.step)()
    }
    fn check_scratch(&self, extra: usize) -> Result<()> {
        let mut scope = Some(self);
        while let Some(current) = scope {
            let scratch = *current.scratch.lock().expect("name scratch invariant");
            check(
                current.limits(),
                Resource::Scratch,
                scratch
                    .checked_add(extra)
                    .ok_or(DomError::RepresentationExhausted)?,
            )?;
            scope = current.parent.as_deref();
        }
        Ok(())
    }
    fn add_scratch(&self, extra: usize) {
        let mut scope = Some(self);
        while let Some(current) = scope {
            let mut scratch = current.scratch.lock().expect("name scratch invariant");
            *scratch = scratch.checked_add(extra).expect("admitted name scratch");
            scope = current.parent.as_deref();
        }
    }
    fn release_scratch(&self, amount: usize) {
        let mut scope = Some(self);
        while let Some(current) = scope {
            let mut scratch = current.scratch.lock().expect("name scratch invariant");
            *scratch = scratch.checked_sub(amount).expect("owned name scratch");
            scope = current.parent.as_deref();
        }
    }
    fn ancestor(
        self: &Arc<Self>,
        identity: custody::MaterializationId,
        admission: &mut Admission<'_>,
    ) -> Result<Option<Arc<Self>>> {
        let mut scope = Some(self);
        while let Some(current) = scope {
            self.step(admission)?;
            if current.identity == identity {
                return Ok(Some(current.clone()));
            }
            scope = current.parent.as_ref();
        }
        Ok(None)
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        if let Some(parent) = &self.parent {
            parent.release_scratch(*self.scratch.lock().expect("closing name scratch"));
        }
        if let Some(state) = self.state.upgrade() {
            let mut state = state.lock().expect("name registry invariant");
            state.controls = state
                .controls
                .checked_sub(size_of::<Self>())
                .expect("owned scope");
        }
        // Owner-only attribution may outlive this scope through Weak. Closing
        // the scope ends all remaining formation scratch in its ancestors;
        // later capsule release only removes retained name extent.
        // Ancestor data dies before its affine token.
        drop(self.parent.take());
    }
}

#[derive(Clone)]
pub(in crate::store) struct Operation {
    retained: Arc<Mutex<Retained>>,
    scope: Arc<Scope>,
    owner: Registry,
}

/// Directly attributable name-owned allocations only. This is an affine
/// receipt in the existing DOM ledger, never a second accounting domain.
struct LocalStorage {
    scope: Arc<Scope>,
    owner: Registry,
    debit: custody::Debit,
}
impl LocalStorage {
    fn new(owner: &Registry, scope: &Arc<Scope>, admission: &mut Admission<'_>) -> Result<Self> {
        let ledger = owner.lock().registry.ledger().clone();
        Ok(Self {
            scope: scope.clone(),
            owner: owner.clone(),
            debit: ledger.retain(custody::Usage::ZERO, &mut *admission.global)?,
        })
    }
    fn grow(&mut self, bytes: usize, admission: &mut Admission<'_>) -> Result<()> {
        let local = Attribution::new(&self.owner, &self.scope, bytes)?;
        self.debit
            .grow(custody::Usage::new(bytes, 0), &mut *admission.global)?;
        // This storage already carries the local receipt through its own drop.
        // Transfer only the accounting obligation, never a second global debit.
        let mut local = local;
        local.bytes = 0;
        Ok(())
    }
    /// Caller has physically discarded unused paths/backing before shrinking.
    fn shrink(&mut self, bytes: usize) {
        let removed = self
            .debit
            .usage()
            .bytes()
            .checked_sub(bytes)
            .expect("owned admitted subset");
        let released = self
            .debit
            .split(custody::Usage::new(removed, 0))
            .expect("owned admitted subset");
        self.owner.lock().controls -= removed;
        self.scope.release_scratch(removed);
        drop(released);
    }
    fn absorb(&mut self, mut other: Self) {
        let debit = other
            .debit
            .split(other.debit.usage())
            .expect("whole admitted extent");
        self.debit.absorb(debit).expect("one DOM name ledger");
    }
}
impl Drop for LocalStorage {
    fn drop(&mut self) {
        let bytes = self.debit.usage().bytes();
        self.owner.lock().controls -= bytes;
        self.scope.release_scratch(bytes);
    }
}

/// Shared only by aliases of one original operation, outside Draft inverses.
/// Replays reuse the occupied guard slot; they do not allocate another overlay.
struct Retained {
    values: bank::owned::OwnedBank<Reservation>,
    storage: Option<LocalStorage>,
}
struct RetainedLease {
    values: bank::owned::OwnedBank<Reservation>,
    storage: Option<LocalStorage>,
    owner: Arc<Mutex<Retained>>,
}
impl RetainedLease {
    fn take(owner: &Arc<Mutex<Retained>>) -> Result<Self> {
        let mut value = owner.lock().expect("name reservation owner invariant");
        let storage = value
            .storage
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        Ok(Self {
            values: std::mem::take(&mut value.values),
            storage: Some(storage),
            owner: owner.clone(),
        })
    }
}
impl Drop for RetainedLease {
    fn drop(&mut self) {
        let mut value = self.owner.lock().expect("name reservation owner invariant");
        assert!(
            value.storage.is_none(),
            "exclusive reservation overlay lease"
        );
        value.values = std::mem::take(&mut self.values);
        value.storage = self.storage.take();
    }
}
impl Registry {
    pub(in crate::store) fn new(
        owner: DomOwnerId,
        registry: &bank::CustodyRegistry,
        limits: DomNameLimits,
        admission: &mut Admission<'_>,
    ) -> Result<Self> {
        let own = custody::Usage::new(size_of::<Mutex<State>>(), 0);
        check(limits, Resource::RetainedBytes, own.bytes())?;
        let storage = registry.ledger().retain(own, &mut *admission.global)?;
        Ok(Self(Arc::new(Mutex::new(State {
            index: Index::default(),
            index_leased: false,
            index_entries: 0,
            leased_storage: custody::Usage::ZERO,
            records: custody::Usage::ZERO,
            controls: 0,
            limits,
            owner,
            hash: RandomState::new(),
            registry: registry.clone(),
            storage,
        }))))
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().expect("name registry invariant")
    }
    pub(in crate::store) fn begin(
        &self,
        parent: Option<&Operation>,
        admission: &mut Admission<'_>,
    ) -> Result<Operation> {
        if parent.is_some_and(|parent| !Arc::ptr_eq(&parent.owner.0, &self.0)) {
            return Err(DomError::ForeignOwner);
        }
        let (limits, ledger) = {
            let state = self.lock();
            (state.limits, state.registry.ledger().clone())
        };
        let bytes = size_of::<Scope>();
        {
            let mut state = self.lock();
            check(
                limits,
                Resource::RetainedBytes,
                state
                    .usage()?
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(DomError::RepresentationExhausted)?,
            )?;
            if let Some(parent) = parent {
                parent.scope.check_scratch(bytes)?;
            }
            check(limits, Resource::Scratch, bytes)?;
            state.controls += bytes;
        }
        if let Some(parent) = parent {
            parent.scope.add_scratch(bytes);
        }
        let retained = (|| {
            let identity = custody::MaterializationId::fresh()?;
            let storage = ledger.retain(custody::Usage::new(bytes, 0), &mut *admission.global)?;
            Ok::<_, DomError>((identity, storage))
        })();
        let (identity, storage) = match retained {
            Ok(value) => value,
            Err(error) => {
                self.lock().controls -= bytes;
                if let Some(parent) = parent {
                    parent.scope.release_scratch(bytes);
                }
                return Err(error);
            }
        };
        let scope = Arc::new(Scope {
            parent: parent.map(|parent| parent.scope.clone()),
            state: Arc::downgrade(&self.0),
            identity,
            limits: Mutex::new(limits),
            work: Mutex::new(0),
            scratch: Mutex::new(bytes),
            _storage: storage,
        });
        let mut storage = LocalStorage::new(self, &scope, admission)?;
        storage.grow(size_of::<Mutex<Retained>>(), admission)?;
        Ok(Operation {
            retained: Arc::new(Mutex::new(Retained {
                values: bank::owned::OwnedBank::default(),
                storage: Some(storage),
            })),
            scope,
            owner: self.clone(),
        })
    }
    #[cfg(test)]
    pub(in crate::store) fn set_retained_limits(&self, bytes: usize, entries: usize) -> Result<()> {
        let mut state = self.lock();
        let limits = DomNameLimits::new(
            bytes,
            entries,
            state.limits.operation_work(),
            state.limits.operation_scratch(),
        )?;
        check(limits, Resource::RetainedBytes, state.usage()?.bytes())?;
        check(
            limits,
            Resource::Entries,
            if state.index_leased {
                state.index_entries
            } else {
                state.index.entries()
            },
        )?;
        state.limits = limits;
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::store) fn inspect_usage(&self) -> custody::Usage {
        self.lock().usage().unwrap()
    }
    pub(in crate::store) fn admit_limits(&self, limits: DomNameLimits) -> Result<()> {
        let state = self.lock();
        check(limits, Resource::RetainedBytes, state.usage()?.bytes())?;
        check(
            limits,
            Resource::Entries,
            if state.index_leased {
                state.index_entries
            } else {
                state.index.entries()
            },
        )
    }
    pub(in crate::store) fn replace_limits(&self, limits: DomNameLimits) {
        self.lock().limits = limits;
    }
}

impl Operation {
    fn pending_origin(
        &self,
        origin: &Weak<Scope>,
        admission: &mut Admission<'_>,
    ) -> Result<Arc<Scope>> {
        self.scope.step(admission)?;
        let origin = origin.upgrade().ok_or(DomError::InvalidSourceOperation)?;
        // Enclosing operations retain their existing ancestry and cumulative
        // scratch. Independent prepared edits may pin the same private record;
        // their guards retain its exact original receipt until promotion/drop.
        Ok(self
            .scope
            .ancestor(origin.identity, admission)?
            .unwrap_or(origin))
    }

    pub(in crate::store) fn step(&self, step: &mut impl FnMut() -> Result<()>) -> Result<()> {
        self.scope.step(&mut Admission {
            global: &mut |_| Ok(()),
            step,
        })
    }
    pub(in crate::store) fn retain(
        &self,
        text: DocumentString,
        admission: &mut Admission<'_>,
    ) -> Result<DomNameAtom> {
        self.cleanup(1, admission)?;
        let reservation = self.reserve(text, admission)?;
        self.retain_reservation(reservation, admission)
    }
    fn bind_output(
        &self,
        text: DocumentString,
        admission: &mut Admission<'_>,
    ) -> Result<DomNameAtom> {
        // The explicit single-token output has a finite value-formation extent,
        // distinct from arbitrary caller clones after this operation returns.
        let mut output = LocalStorage::new(&self.owner, &self.scope, admission)?;
        output.grow(size_of::<DomNameAtom>(), admission)?;
        self.cleanup(1, admission)?;
        let reserved = self.reserve(text, admission)?;
        Ok(reserved.publish())
    }
    pub(in crate::store) fn attribute(&self, bytes: usize) -> Result<Attribution> {
        Attribution::new(&self.owner, &self.scope, bytes)
    }
    pub(in crate::store) fn retain_atom(
        &self,
        atom: &DomNameAtom,
        admission: &mut Admission<'_>,
    ) -> Result<DomNameAtom> {
        if atom.owner() != self.owner.lock().owner {
            return self.retain(atom.text().clone(), admission);
        }
        self.scope.step(admission)?;
        let slot = atom.record().row();
        let key = atom.identity();
        let mut lease = IndexLease::take(&self.owner)?;
        let row = lease
            .index
            .find_row_mut(slot, key)
            .ok_or(DomError::InvalidSourceOperation)?;
        let origin = if let Some(pending) = &mut row.pending {
            let origin = self.pending_origin(&pending.origin, admission)?;
            pending.pins = pending
                .pins
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Some(origin)
        } else {
            None
        };
        drop(lease);
        // This guard owns the exact row pin before any later work can refuse.
        let reservation = Reservation {
            atom: Some(atom.clone()),
            origin,
            key,
            slot,
            owner: self.owner.clone(),
        };
        for _ in 0..index::CLOSING_STEPS {
            self.scope.step(admission)?;
        }
        self.retain_reservation(reservation, admission)
    }
    fn retain_reservation(
        &self,
        reservation: Reservation,
        admission: &mut Admission<'_>,
    ) -> Result<DomNameAtom> {
        let atom = reservation
            .atom
            .as_ref()
            .expect("retained private atom")
            .clone();
        let mut retained = RetainedLease::take(&self.retained)?;
        if let Some(existing) = retained.values.get(reservation.slot) {
            assert_eq!(
                existing.key, reservation.key,
                "a pinned row cannot be reused"
            );
            drop(reservation);
            return Ok(atom);
        }
        let mut storage = LocalStorage::new(&self.owner, &self.scope, admission)?;
        let plan = retained.values.reserve(reservation.slot, |charge| {
            self.scope.step(admission)?;
            storage.grow(charge.bytes, admission)?;
            Ok::<_, DomError>(None)
        })?;
        let moved = retained.values.insert(plan, reservation, false).bytes;
        storage.shrink(moved);
        retained
            .storage
            .as_mut()
            .expect("leased overlay receipt")
            .absorb(storage);
        Ok(atom)
    }
    pub(in crate::store) fn admit_limits(&self, limits: DomNameLimits) -> Result<()> {
        let work = *self.scope.work.lock().expect("name work invariant");
        if work > limits.operation_work() {
            return Err(DomError::WorkLimit {
                resource: DomResource::NameWork,
                limit: limits.operation_work(),
            });
        }
        check(
            limits,
            Resource::Scratch,
            *self.scope.scratch.lock().expect("name scratch invariant"),
        )
    }
    pub(in crate::store) fn replace_limits(&self, limits: DomNameLimits) {
        *self
            .scope
            .limits
            .lock()
            .expect("name operation policy invariant") = limits;
    }
    pub(in crate::store) fn reserve(
        &self,
        text: DocumentString,
        admission: &mut Admission<'_>,
    ) -> Result<Reservation> {
        let mut hash = self.owner.lock().hash.build_hasher();
        for unit in text.code_units() {
            self.scope.step(admission)?;
            hash.write_u16(*unit);
        }
        self.reserve_hashed(text, hash.finish(), admission)
    }
    fn reserve_hashed(
        &self,
        text: DocumentString,
        fingerprint: u64,
        admission: &mut Admission<'_>,
    ) -> Result<Reservation> {
        let mut lease = IndexLease::take(&self.owner)?;
        let (limits, owner, registry) = {
            let state = self.owner.lock();
            (state.limits, state.owner, state.registry.clone())
        };
        let mut current = lease.index.first(fingerprint);
        while let Some(slot) = current {
            self.scope.step(admission)?;
            let row = lease.index.row(slot);
            current = row.next;
            if row.fingerprint != fingerprint || row.len != text.len() {
                continue;
            }
            let Some(record) = row.record.upgrade() else {
                continue;
            };
            let mut equal = true;
            for (left, right) in record.text().code_units().iter().zip(text.code_units()) {
                self.scope.step(admission)?;
                if left != right {
                    equal = false;
                    break;
                }
            }
            if equal {
                let origin = match row.pending.as_ref() {
                    Some(pending) => Some(self.pending_origin(&pending.origin, admission)?),
                    None => None,
                };
                // Prepay the fixed, allocation-free promotion or unlink path
                // before returning a retained reservation to a mandatory tail.
                for _ in 0..index::CLOSING_STEPS {
                    self.scope.step(admission)?;
                }
                let key = row.key;
                if let Some(pending) = lease.index.row_mut(slot).pending.as_mut() {
                    pending.pins = pending
                        .pins
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
                return Ok(Reservation {
                    atom: Some(DomNameAtom::from_record(record)),
                    origin,
                    key,
                    slot,
                    owner: self.owner.clone(),
                });
            }
        }
        check(
            limits,
            Resource::Entries,
            lease
                .index
                .entries()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        let payload = NameRecord::usage(&text)?;
        // The per-record shared payload receipt is directly attributable. The
        // generic registration/path/release machinery remains in the original
        // ledger, outside the name-local ceiling.
        let own = payload.checked_add(custody::SharedDebit::administration_usage())?;
        let key = custody::MaterializationId::fresh()?;
        let mut prepared = Prepared::new(&self.owner, &self.scope, own, admission)?;
        lease.index.prepare(&mut prepared, &self.scope, admission)?;
        for _ in 0..index::CLOSING_STEPS {
            self.scope.step(admission)?;
        }
        let custody = registry.adopt_object(key, payload, &mut *admission.global)?;
        let record = Arc::new(NameRecord::new(owner, text, key, prepared.slot()));
        #[cfg(test)]
        metric(super::scale_metrics::Event::StructuralAllocation {
            bytes: size_of::<NameRecord>(),
        });
        let row = index::Row {
            fingerprint,
            len: record.text().len(),
            record: Arc::downgrade(&record),
            key,
            next: None,
            previous: None,
            bucket: 0,
            pending: Some(index::Pending {
                origin: Arc::downgrade(&self.scope),
                pins: 1,
                bytes: 0,
            }),
            own,
            _custody: custody,
        };
        let (slot, moved) = lease.index.install(prepared, row, &mut lease.storage);
        let pending_bytes = own
            .bytes()
            .checked_add(moved)
            .expect("admitted pending extent");
        lease.index.row_mut(slot).pending.as_mut().unwrap().bytes = pending_bytes;
        Ok(Reservation {
            atom: Some(DomNameAtom::from_record(record)),
            origin: Some(self.scope.clone()),
            key,
            slot,
            owner: self.owner.clone(),
        })
    }
    pub(in crate::store) fn check_owner(&self, atom: &DomNameAtom) -> Result<()> {
        if atom.owner() == self.owner.lock().owner {
            Ok(())
        } else {
            Err(DomError::ForeignOwner)
        }
    }
    pub(in crate::store) fn cleanup(
        &self,
        count: usize,
        admission: &mut Admission<'_>,
    ) -> Result<usize> {
        let mut lease = IndexLease::take(&self.owner)?;
        let mut removed = 0;
        for _ in 0..count {
            self.scope.step(admission)?;
            if let Some(row) = lease.index.cleanup_one() {
                {
                    let mut state = self.owner.lock();
                    state.records = state.records.checked_sub(row.own)?;
                }
                drop(row);
                let registry = self.owner.lock().registry.clone();
                registry.reconcile_releases();
                removed += 1;
            }
        }
        Ok(removed)
    }
}

/// Fixed aliases of already admitted publication fields. Closing never needs
/// a freshly allocated name-selection directory or a new private identity.
pub(in crate::store) struct Promotions {
    slots: bank::Bank<super::Slot>,
    touched: bank::Bank<bool>,
    before: bank::Bank<super::Slot>,
    operation: Operation,
}
impl Promotions {
    pub(in crate::store) fn prepare(
        operation: &Operation,
        slots: &bank::Bank<super::Slot>,
        touched: &bank::Bank<bool>,
        before: &bank::Bank<super::Slot>,
        admission: &mut Admission<'_>,
    ) -> Result<Self> {
        let value = Self {
            slots: slots.clone(),
            touched: touched.clone(),
            before: before.clone(),
            operation: operation.clone(),
        };
        value.visit(|atom| {
            operation.check_owner(atom)?;
            operation.scope.step(admission)?;
            let origin = {
                let state = operation.owner.lock();
                state
                    .index
                    .find_row(atom.record().row(), atom.identity())
                    .ok_or(DomError::InvalidSourceOperation)?
                    .pending
                    .as_ref()
                    .map(|pending| pending.origin.clone())
            };
            if let Some(origin) = origin {
                operation.pending_origin(&origin, admission)?;
                for _ in 0..index::CLOSING_STEPS {
                    operation.scope.step(admission)?;
                }
            }
            Ok(())
        })?;
        Ok(value)
    }
    fn visit(&self, mut visit: impl FnMut(&DomNameAtom) -> Result<()>) -> Result<()> {
        for (index, _) in self.touched.iter() {
            let Some(element) = self
                .slots
                .get(index)
                .and_then(|slot| slot.node.as_ref())
                .and_then(|node| node.element())
            else {
                continue;
            };
            let name = element.name();
            if let Some(namespace) = name.namespace() {
                visit(namespace)?;
            }
            visit(name.local_name_atom())?;
            let unchanged = self
                .before
                .get(index)
                .and_then(|slot| slot.node.as_ref())
                .is_some_and(|node| {
                    node.element()
                        .is_some_and(|old| old.same_attribute_storage(element))
                });
            if !unchanged {
                for attribute in element.attributes() {
                    if let Some(namespace) = attribute.name().namespace() {
                        visit(namespace)?;
                    }
                    visit(attribute.name().local_name_atom())?;
                }
            }
        }
        Ok(())
    }
    pub(in crate::store) fn commit(self) {
        self.visit(|atom| {
            let pending = {
                let mut state = self.operation.owner.lock();
                state
                    .index
                    .find_row_mut(atom.record().row(), atom.identity())
                    .expect("preadmitted pinned publication row")
                    .pending
                    .take()
            };
            if let Some(pending) = pending {
                pending
                    .origin
                    .upgrade()
                    .expect("prepared reservation retains its original scope")
                    .release_scratch(pending.bytes);
            }
            Ok(())
        })
        .expect("publication name transition is infallible");
    }
}

#[cfg(test)]
fn metric(event: super::scale_metrics::Event) {
    if super::scale_metrics::ENABLED {
        super::scale_metrics::record(event);
    }
}

pub(in crate::store) struct Reservation {
    atom: Option<DomNameAtom>,
    origin: Option<Arc<Scope>>,
    key: custody::MaterializationId,
    slot: usize,
    owner: Registry,
}
impl Reservation {
    pub(in crate::store) fn publish(mut self) -> DomNameAtom {
        let mut state = self.owner.lock();
        let row = state.index.row_mut(self.slot);
        assert_eq!(row.key, self.key, "pinned nonreusable pending row");
        if let Some(pending) = row.pending.take() {
            self.origin
                .as_ref()
                .expect("pending reservation ancestry")
                .release_scratch(pending.bytes);
        }
        drop(state);
        self.atom.take().expect("unexposed admitted atom")
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        // The strong semantic target dies before a last private row/custody.
        drop(self.atom.take());
        let mut lease = IndexLease::take(&self.owner)
            .expect("a reservation closes outside an admission callback");
        let Some(row) = lease.index.find_row_mut(self.slot, self.key) else {
            return;
        };
        if let Some(pending) = &mut row.pending {
            pending.pins -= 1;
            if pending.pins == 0 {
                let row = lease.index.remove(self.slot);
                self.origin
                    .as_ref()
                    .expect("owned pending ancestry")
                    .release_scratch(row.pending.as_ref().unwrap().bytes);
                {
                    let mut state = self.owner.lock();
                    state.records = state
                        .records
                        .checked_sub(row.own)
                        .expect("owned record receipt");
                }
                drop(row);
                let registry = self.owner.lock().registry.clone();
                registry.reconcile_releases();
            }
        }
    }
}

impl super::Dom {
    /// Bind exact UTF-16 to this owner's live name identity without changing
    /// tree revision or history. No language validation or case folding occurs.
    pub fn bind_name(
        &mut self,
        text: &DocumentString,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<DomNameAtom> {
        self.require_source_entry()?;
        let formation = self.dispatch_registration_formation(cancelled)?;
        let mut global = |next| match &formation {
            Some(formation) => formation.admit_custody_usage(next, cancelled),
            None => super::check_cancel(cancelled),
        };
        let mut step = || super::source::formation::step(formation.as_ref(), cancelled);
        let mut admission = Admission {
            global: &mut global,
            step: &mut step,
        };
        let parent = self
            .source_operation
            .as_ref()
            .or_else(|| self.source_parents.last())
            .map(|operation| &operation.names);
        let operation = self.names.begin(parent, &mut admission)?;
        operation.bind_output(text.clone(), &mut admission)
    }
}
