//! Actual temporary publication banks while the private Draft/checkpoint remain
//! live. Their charge ends only at discard or canonical ownership transfer.
use super::*;
use std::cell::{Cell, RefCell};

#[cfg(test)]
mod custody_tests;

/// Record an actual admitted control-target allocation at its construction.
pub(crate) fn allocated_arc<T>(value: T) -> Arc<T> {
    let target = Arc::new(value);
    #[cfg(test)]
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: size_of::<T>(),
        });
    }
    target
}
pub(super) fn allocated_box<T>(value: T) -> Box<T> {
    let result = Box::new(value);
    #[cfg(test)]
    scale_metrics::record(scale_metrics::Event::StructuralAllocation {
        bytes: size_of::<T>(),
    });
    result
}

/// Lease-free access to the original controlled storage union. External event
/// requests may retain this counter handle, but it owns no record/page debit.
#[derive(Clone, Debug)]
pub(crate) struct SourceStorage(custody::Ledger);
impl SourceStorage {
    #[inline]
    pub(crate) fn record_entry() {
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::Entered(scale_metrics::Domain::Source));
        }
    }
    #[inline]
    pub(crate) fn record_step() {
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::Entered(scale_metrics::Domain::Source));
            scale_metrics::record(scale_metrics::Event::SourceStep);
        }
    }
    pub(in crate::store) fn new(ledger: custody::Ledger) -> Self {
        Self(ledger)
    }
    pub(in crate::store) fn ledger(&self) -> &custody::Ledger {
        &self.0
    }
}

/// Affine owner-side custody of an explicitly retained Source event payload.
/// This lives in the retention capability, never in an immutable request.
#[derive(Debug)]
pub(crate) struct SourceRetention {
    debit: custody::Debit,
    _event: Arc<EventWorkCustody>,
}
impl SourceRetention {
    pub(in crate::store) fn retain(
        event: Arc<EventWorkCustody>,
        usage: custody::Usage,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let debit = event.work().storage.ledger().retain(usage, admit)?;
        Ok(Self {
            debit,
            _event: event,
        })
    }
    pub(crate) fn absorb(&mut self, other: Self) -> Result<(), DomError> {
        self.debit
            .absorb(other.debit)
            .map_err(|_| DomError::InvalidSourceEvent)
    }
}

/// Lease-free identity of an original cumulative counter, minted by its real
/// issuer before the target allocation and preserved by every later alias.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceWorkIdentity(custody::MaterializationId);
impl SourceWorkIdentity {
    pub(crate) fn fresh() -> Result<Self, DomError> {
        custody::MaterializationId::fresh().map(Self)
    }
    pub(in crate::store) fn materialization(self) -> custody::MaterializationId {
        self.0
    }
}

/// Owner-side custody of one original cumulative Source counter. Aliases retain
/// the same actual atom once; loading its numerical value never creates custody.
#[derive(Clone)]
pub(in crate::store) struct SourceWorkCustody {
    work: Arc<AtomicU64>,
    identity: SourceWorkIdentity,
    _custody: bank::ObjectCustody,
}
impl std::fmt::Debug for SourceWorkCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceWorkCustody")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
impl SourceWorkCustody {
    pub(in crate::store) fn allocate(
        spent: u64,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(Arc<AtomicU64>, Self), DomError> {
        let identity = SourceWorkIdentity::fresh()?;
        let custody = registry.adopt_object(
            identity.materialization(),
            custody::Usage::new(size_of::<AtomicU64>(), 0),
            admit,
        )?;
        let work = allocated_arc(AtomicU64::new(spent));
        Ok((
            work.clone(),
            Self {
                work,
                identity,
                _custody: custody,
            },
        ))
    }
    pub(in crate::store) fn work(&self) -> &Arc<AtomicU64> {
        &self.work
    }
    pub(in crate::store) fn identity(&self) -> SourceWorkIdentity {
        self.identity
    }
    pub(in crate::store) fn adopt_existing(
        identity: SourceWorkIdentity,
        work: &Arc<AtomicU64>,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let custody = registry.adopt_object(
            identity.materialization(),
            custody::Usage::new(size_of::<AtomicU64>(), 0),
            admit,
        )?;
        Ok(Self {
            work: work.clone(),
            identity,
            _custody: custody,
        })
    }
}

/// Controlled dependency capsule for an original event's fixed atoms. External
/// EventWork/request values intentionally hold no copy of this owner authority.
#[derive(Debug)]
pub(in crate::store) struct EventWorkCustody {
    // Destroy represented targets/dependencies before their own debit.
    work: Arc<crate::source::event::EventWork>,
    source: SourceWorkCustody,
    ancestor: Option<Arc<EventWorkCustody>>,
    _debit: custody::SharedDebit,
}
pub(in crate::store) struct PreparedEventWorkCustody {
    source: SourceWorkCustody,
    ancestor: Option<Arc<EventWorkCustody>>,
    debit: custody::SharedDebit,
}
impl EventWorkCustody {
    pub(in crate::store) fn prepare(
        ledger: &custody::Ledger,
        source: SourceWorkCustody,
        ancestor: Option<Arc<Self>>,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<PreparedEventWorkCustody, DomError> {
        let bytes = size_of::<crate::source::event::EventWork>()
            .checked_add(2 * size_of::<AtomicUsize>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let debit = ledger.retain_shared(custody::Usage::new(bytes, 0), admit)?;
        // The executor is held only weakly by EventWork. Its external target is
        // not a controlled allocation; the Weak holder is in EventWork's size.
        Ok(PreparedEventWorkCustody {
            source,
            ancestor,
            debit,
        })
    }
    pub(in crate::store) fn work(&self) -> &Arc<crate::source::event::EventWork> {
        &self.work
    }
    pub(in crate::store) fn source(&self) -> SourceWorkCustody {
        self.source.clone()
    }
    pub(in crate::store) fn ancestor(&self) -> Option<Arc<Self>> {
        self.ancestor.clone()
    }
}
impl PreparedEventWorkCustody {
    pub(in crate::store) fn finish(
        self,
        work: Arc<crate::source::event::EventWork>,
    ) -> Arc<EventWorkCustody> {
        allocated_arc(EventWorkCustody {
            work,
            source: self.source,
            ancestor: self.ancestor,
            _debit: self.debit,
        })
    }
}

impl Draft {
    /// Admit the paired ledger's absolute prospective union. This gate owns no
    /// debit and adds only obligations explicitly outside that ledger.
    pub(in crate::store) fn admit_custody_usage(
        &self,
        prospective: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if let Some(parsed) = self.source_work.parsed() {
            return parsed.controlled_unpolled(prospective);
        }
        let Some(limits) = self.source_limits else {
            return Ok(());
        };
        admit(
            prospective
                .bytes()
                .checked_add(self.source_other_bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            limits.retained_bytes,
            DomResource::SourceRetainedBytes,
        )?;
        admit(
            prospective
                .units()
                .checked_add(self.source_other_units)
                .ok_or(DomError::RepresentationExhausted)?,
            limits.retained_units,
            DomResource::SourceRetainedUnits,
        )
    }
}

/// Owner-side lifetime of the original diagnostic mirror targets. Immutable
/// external requests may share numerical counters without retaining this lease.
#[derive(Clone)]
pub(in crate::store) struct SourceMirrors(Arc<MirrorTargets>);
struct MirrorTargets {
    bytes: Arc<AtomicUsize>,
    units: Arc<AtomicUsize>,
    temporary: Arc<AtomicUsize>,
    _debit: custody::SharedDebit,
}
impl SourceMirrors {
    pub(in crate::store) fn allocate(
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let bytes = size_of::<MirrorTargets>()
            .checked_add(
                size_of::<AtomicUsize>()
                    .checked_mul(3)
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        let debit = registry
            .ledger()
            .retain_shared(custody::Usage::new(bytes, 0), admit)?;
        Ok(Self(allocated_arc(MirrorTargets {
            bytes: allocated_arc(AtomicUsize::new(0)),
            units: allocated_arc(AtomicUsize::new(0)),
            temporary: allocated_arc(AtomicUsize::new(0)),
            _debit: debit,
        })))
    }
    pub(in crate::store) fn bytes(&self) -> Arc<AtomicUsize> {
        self.0.bytes.clone()
    }
    pub(in crate::store) fn units(&self) -> Arc<AtomicUsize> {
        self.0.units.clone()
    }
    pub(in crate::store) fn temporary(&self) -> Arc<AtomicUsize> {
        self.0.temporary.clone()
    }
}

/// One actual work owner. Parsed construction reuses its original Entry atom;
/// it never allocates or resets a Source counter to impersonate that authority.
#[derive(Clone)]
pub(in crate::store) enum WorkOwner {
    Source {
        work: Arc<AtomicU64>,
        custody: SourceWorkCustody,
    },
    Parsed(crate::store::pending_associations::control::Admission),
}
impl WorkOwner {
    pub(in crate::store) fn load(&self, order: Ordering) -> u64 {
        match self {
            Self::Source { work, .. } => work.load(order),
            Self::Parsed(a) => a.spent_work(),
        }
    }
    pub(in crate::store) fn store(&self, value: u64, order: Ordering) {
        self.source_arc().store(value, order);
    }
    pub(in crate::store) fn source_arc(&self) -> &Arc<AtomicU64> {
        match self {
            Self::Source { work, .. } => work,
            Self::Parsed(_) => {
                unreachable!("parsed work is never reset or exported as Source authority")
            }
        }
    }
    pub(in crate::store) fn source_custody(&self) -> SourceWorkCustody {
        match self {
            Self::Source { custody, .. } => custody.clone(),
            Self::Parsed(_) => unreachable!("actual Source custody required"),
        }
    }
    pub(in crate::store) fn parsed(
        &self,
    ) -> Option<&crate::store::pending_associations::control::Admission> {
        match self {
            Self::Parsed(value) => Some(value),
            Self::Source { .. } => None,
        }
    }
}
/// Actual owner dependencies of counters retained by a temporary guard. These
/// aliases keep their original atom receipts alive without duplicating debit.
#[derive(Clone)]
pub(super) struct WorkDependencies {
    _source: Option<SourceWorkCustody>,
    _parsed: Option<crate::store::pending_associations::control::Admission>,
    _event: Option<Arc<EventWorkCustody>>,
    _mirrors: Option<SourceMirrors>,
}
impl WorkDependencies {
    pub(super) fn same_source(&self, other: &Self) -> bool {
        match (&self._source, &other._source) {
            (Some(a), Some(b)) => a.identity() == b.identity() && Arc::ptr_eq(a.work(), b.work()),
            _ => false,
        }
    }
    pub(super) fn from_draft(draft: &Draft) -> Self {
        Self {
            _source: draft
                .source_work
                .parsed()
                .is_none()
                .then(|| draft.source_work_custody()),
            _parsed: draft.source_work.parsed().cloned(),
            _event: draft.source_ancestor_custody(),
            _mirrors: Some(draft.source_mirrors()),
        }
    }
    fn from_event(event: Arc<EventWorkCustody>) -> Self {
        Self {
            _source: Some(event.source()),
            _parsed: None,
            _event: Some(event),
            _mirrors: None,
        }
    }
}

pub(crate) struct Formation {
    // Only obligations outside the actual shared controlled union.
    base: usize,
    base_units: usize,
    storage: SourceStorage,
    debit: RefCell<Option<custody::Debit>>,
    limit: usize,
    unit_limit: usize,
    // Protocol/diagnostic mirrors; never added back into Ledger.total().
    live: Arc<AtomicUsize>,
    own: Cell<usize>,
    live_units: Arc<AtomicUsize>,
    own_units: Cell<usize>,
    work: WorkOwner,
    ancestors: Option<Arc<crate::source::event::EventWork>>,
    work_limit: u64,
    dependencies: WorkDependencies,
}
impl Drop for Formation {
    fn drop(&mut self) {
        self.live.fetch_sub(self.own.get(), Ordering::Relaxed);
        self.live_units
            .fetch_sub(self.own_units.get(), Ordering::Relaxed);
    }
}
impl Formation {
    pub(in crate::store) fn new(
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Option<Self>, DomError> {
        let limits = match (draft.source_limits, draft.source_work.parsed()) {
            (Some(limits), _) => limits,
            (None, Some(parsed)) => {
                parsed.step(cancel)?;
                return Ok(Some(Self {
                    base: 0,
                    base_units: 0,
                    storage: SourceStorage::new(draft.custody_ledger().clone()),
                    debit: RefCell::new(None),
                    limit: parsed.limits().retained_bytes(),
                    unit_limit: parsed.limits().retained_units(),
                    live: draft.source_formation_bytes.clone(),
                    own: Cell::new(0),
                    live_units: draft.source_temporary_units.clone(),
                    own_units: Cell::new(0),
                    work: draft.source_work.clone(),
                    ancestors: None,
                    work_limit: 0,
                    dependencies: WorkDependencies::from_draft(draft),
                }));
            }
            (None, None) => return Ok(None),
        };
        draft.source_admit(0, cancel)?;
        Ok(Some(Self {
            base: draft.source_other_bytes,
            base_units: draft.source_other_units,
            storage: SourceStorage::new(draft.custody_ledger().clone()),
            debit: RefCell::new(None),
            limit: limits.retained_bytes,
            unit_limit: limits.retained_units,
            live: draft.source_formation_bytes.clone(),
            own: Cell::new(0),
            live_units: draft.source_temporary_units.clone(),
            own_units: Cell::new(0),
            work: draft.source_work.clone(),
            ancestors: draft.source_ancestors.clone(),
            work_limit: limits.work,
            dependencies: WorkDependencies::from_draft(draft),
        }))
    }
    pub(in crate::store) fn from_event(
        event_custody: Arc<EventWorkCustody>,
        external: storage::Payload,
        limits: crate::DomSourceLimits,
    ) -> Self {
        let work = event_custody.work();
        Self {
            base: external.bytes,
            base_units: external.units,
            storage: work.storage.clone(),
            debit: RefCell::new(None),
            limit: limits.retained_bytes,
            unit_limit: limits.retained_units,
            live: work.bytes.clone(),
            live_units: work.units.clone(),
            own: Cell::new(0),
            own_units: Cell::new(0),
            work: WorkOwner::Source {
                work: work.work.clone(),
                custody: event_custody.source(),
            },
            work_limit: limits.work,
            ancestors: work.ancestors.clone(),
            dependencies: WorkDependencies::from_event(event_custody),
        }
    }
    pub(in crate::store) fn fork(&self) -> Self {
        Self {
            base: self.base,
            base_units: self.base_units,
            storage: self.storage.clone(),
            debit: RefCell::new(None),
            limit: self.limit,
            unit_limit: self.unit_limit,
            live: self.live.clone(),
            own: Cell::new(0),
            live_units: self.live_units.clone(),
            own_units: Cell::new(0),
            work: self.work.clone(),
            ancestors: self.ancestors.clone(),
            work_limit: self.work_limit,
            dependencies: self.dependencies.clone(),
        }
    }
    fn absorb_debit(&self, incoming: Option<custody::Debit>) {
        if let Some(incoming) = incoming {
            let mut debit = self.debit.borrow_mut();
            if let Some(owned) = debit.as_mut() {
                owned
                    .absorb(incoming)
                    .expect("same original Source custody ledger");
            } else {
                *debit = Some(incoming);
            }
        }
    }
    /// Transfer an already admitted payload occurrence from its constructing
    /// holder. The caller keeps the represented data alive through this move;
    /// this does not allocate, admit, or change the original work owner.
    pub(in crate::store) fn absorb_payload_debit(&self, incoming: custody::Debit) {
        assert!(
            incoming.belongs_to(self.storage.ledger()),
            "same original Source payload ledger"
        );
        let usage = incoming.usage();
        let own = self
            .own
            .get()
            .checked_add(usage.bytes())
            .expect("admitted payload bytes");
        let units = self
            .own_units
            .get()
            .checked_add(usage.units())
            .expect("admitted payload units");
        self.live
            .load(Ordering::Relaxed)
            .checked_add(usage.bytes())
            .expect("admitted live bytes");
        self.live_units
            .load(Ordering::Relaxed)
            .checked_add(usage.units())
            .expect("admitted live units");
        self.absorb_debit(Some(incoming));
        self.own.set(own);
        self.own_units.set(units);
        self.live.fetch_add(usage.bytes(), Ordering::Relaxed);
        self.live_units.fetch_add(usage.units(), Ordering::Relaxed);
    }
    pub(in crate::store) fn absorb(&self, other: Self) {
        assert!(
            Arc::ptr_eq(&self.live, &other.live)
                && Arc::ptr_eq(&self.live_units, &other.live_units),
            "same original Source formation"
        );
        self.own.set(
            self.own
                .get()
                .checked_add(other.own.replace(0))
                .expect("admitted bytes"),
        );
        self.own_units.set(
            self.own_units
                .get()
                .checked_add(other.own_units.replace(0))
                .expect("admitted units"),
        );
        self.absorb_debit(other.debit.borrow_mut().take());
    }
    pub(crate) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if let Some(parsed) = self.work.parsed() {
            return parsed.step(cancel);
        }
        check_cancel(cancel)?;
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.work_limit {
            return Err(DomError::SourceWorkLimit {
                limit: self.work_limit,
            });
        }
        self.work.store(next, Ordering::Relaxed);
        SourceStorage::record_step();
        crate::source::event::step_ancestors(self.ancestors.as_ref(), cancel)
    }
    fn admit_total(&self, prospective: custody::Usage) -> Result<(), DomError> {
        if let Some(parsed) = self.work.parsed() {
            return parsed.controlled_unpolled(prospective);
        }
        admit(
            self.base
                .checked_add(prospective.bytes())
                .ok_or(DomError::RepresentationExhausted)?,
            self.limit,
            DomResource::SourceRetainedBytes,
        )?;
        admit(
            self.base_units
                .checked_add(prospective.units())
                .ok_or(DomError::RepresentationExhausted)?,
            self.unit_limit,
            DomResource::SourceRetainedUnits,
        )
    }
    pub(crate) fn admit_custody_usage(
        &self,
        prospective: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        self.admit_total(prospective)
    }
    pub(in crate::store) fn retained_byte_limit(&self) -> usize {
        self.limit
    }
    pub(in crate::store) fn remaining_bytes(&self) -> Result<usize, DomError> {
        if let Some(parsed) = self.work.parsed() {
            return parsed.remaining_bytes();
        }
        self.limit
            .checked_sub(self.base)
            .and_then(|bytes| bytes.checked_sub(self.storage.ledger().total().bytes()))
            .ok_or(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: self.limit,
            })
    }
    pub(crate) fn retain_bytes(&self, bytes: usize, cancel: &AtomicBool) -> Result<(), DomError> {
        self.retain_payload(storage::Payload { bytes, units: 0 }, cancel)
    }
    pub(crate) fn retain_payload(
        &self,
        payload: storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let own = self
            .own
            .get()
            .checked_add(payload.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let own_units = self
            .own_units
            .get()
            .checked_add(payload.units)
            .ok_or(DomError::RepresentationExhausted)?;
        let debit = self
            .storage
            .ledger()
            .retain(custody::Usage::new(payload.bytes, payload.units), |next| {
                self.admit_custody_usage(next, cancel)
            })?;
        self.absorb_debit(Some(debit));
        self.live.fetch_add(payload.bytes, Ordering::Relaxed);
        self.live_units.fetch_add(payload.units, Ordering::Relaxed);
        self.own.set(own);
        self.own_units.set(own_units);
        Ok(())
    }
    fn before(&self, bytes: usize) -> Result<(), DomError> {
        self.admit_total(
            self.storage
                .ledger()
                .total()
                .checked_add(custody::Usage::new(bytes, 0))?,
        )
    }
    fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
        utf16: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let required = values
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= values.capacity() {
            return self.before(0);
        }
        let maximum = isize::MAX as usize / size_of::<T>().max(1);
        let target = values
            .capacity()
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let requested = target
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        // Keep the original bank unchanged until allocation, actual capacity,
        // overlap, copying work and cancellation have all been admitted.
        let replacement_budget = self.fork();
        replacement_budget.retain_payload(
            storage::Payload {
                bytes: requested,
                units: if utf16 { target } else { 0 },
            },
            cancel,
        )?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(target)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = replacement
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        #[cfg(test)]
        scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes: actual });
        replacement_budget.retain_payload(
            storage::Payload {
                bytes: actual
                    .checked_sub(requested)
                    .ok_or(DomError::RepresentationExhausted)?,
                units: if utf16 {
                    replacement.capacity() - target
                } else {
                    0
                },
            },
            cancel,
        )?;
        for _ in values.iter() {
            self.step(cancel)?;
        }
        self.step(cancel)?;
        let old_bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        let old_units = if utf16 { values.capacity() } else { 0 };
        #[cfg(test)]
        let moved_bytes = values
            .len()
            .checked_mul(size_of::<T>())
            .expect("actual materialized Vec extent");
        replacement.append(values);
        #[cfg(test)]
        if moved_bytes != 0 {
            scale_metrics::record(scale_metrics::Event::SequenceMaterialized {
                bytes: moved_bytes,
            });
        }
        drop(std::mem::replace(values, replacement));
        replacement_budget.release_payload(storage::Payload {
            bytes: old_bytes,
            units: old_units,
        });
        self.absorb(replacement_budget);
        Ok(())
    }
    /// Transfer already admitted produced occurrences into their selected-field
    /// owner. The represented values must have moved before this classification
    /// ends. The affine debit stays live while temporary observer mirrors end.
    pub(in crate::store) fn into_debit(self) -> Option<custody::Debit> {
        let bytes = self.own.replace(0);
        let units = self.own_units.replace(0);
        let debit = self.debit.borrow_mut().take();
        self.live.fetch_sub(bytes, Ordering::Relaxed);
        self.live_units.fetch_sub(units, Ordering::Relaxed);
        debit
    }

    pub(in crate::store) fn into_charge(self) -> super::checkpoint::Charge {
        let bytes = self.own.replace(0);
        let units = self.own_units.replace(0);
        let debit = self.debit.borrow_mut().take();
        super::checkpoint::Charge::payload(
            self.storage.clone(),
            self.live.clone(),
            bytes,
            self.live_units.clone(),
            units,
            debit,
            self.dependencies.clone(),
        )
    }
    pub(in crate::store) fn owned_payload(&self) -> storage::Payload {
        storage::Payload {
            bytes: self.own.get(),
            units: self.own_units.get(),
        }
    }
    pub(in crate::store) fn split_payload(
        &self,
        payload: storage::Payload,
    ) -> super::checkpoint::Charge {
        self.own.set(
            self.own
                .get()
                .checked_sub(payload.bytes)
                .expect("owned payload bytes"),
        );
        self.own_units.set(
            self.own_units
                .get()
                .checked_sub(payload.units)
                .expect("owned payload units"),
        );
        let debit = self.take_debit(payload);
        super::checkpoint::Charge::payload(
            self.storage.clone(),
            self.live.clone(),
            payload.bytes,
            self.live_units.clone(),
            payload.units,
            debit,
            self.dependencies.clone(),
        )
    }
    fn take_debit(&self, payload: storage::Payload) -> Option<custody::Debit> {
        if payload.bytes == 0 && payload.units == 0 {
            return None;
        }
        Some(
            self.debit
                .borrow_mut()
                .as_mut()
                .expect("owned custody debit")
                .split(custody::Usage::new(payload.bytes, payload.units))
                .expect("owned custody component"),
        )
    }
    pub(crate) fn release_payload(&self, payload: storage::Payload) {
        self.own.set(
            self.own
                .get()
                .checked_sub(payload.bytes)
                .expect("owned payload bytes"),
        );
        self.own_units.set(
            self.own_units
                .get()
                .checked_sub(payload.units)
                .expect("owned payload units"),
        );
        self.live.fetch_sub(payload.bytes, Ordering::Relaxed);
        self.live_units.fetch_sub(payload.units, Ordering::Relaxed);
        drop(self.take_debit(payload));
    }
    pub(in crate::store) fn release_bytes(&self, bytes: usize) {
        self.release_payload(storage::Payload { bytes, units: 0 });
    }
}
fn reserve_unformed<T>(values: &mut Vec<T>, additional: usize) -> Result<(), DomError> {
    super::super::reserve(values, additional)
}
pub(crate) fn reserve<T>(
    values: &mut Vec<T>,
    additional: usize,
    formation: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match formation {
        Some(f) => f.reserve(values, additional, false, cancel),
        None => reserve_unformed(values, additional),
    }
}

/// Grow an actual UTF16 bank with the same byte policy and an explicit unit
/// extent. Other two-byte element types do not acquire UTF16 classification.
pub(crate) fn reserve_utf16(
    values: &mut Vec<u16>,
    additional: usize,
    formation: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    match formation {
        Some(f) => f.reserve(values, additional, true, cancel),
        None => reserve_unformed(values, additional),
    }
}

/// Materialize the original native factory's exact text under its existing
/// Source formation custody. Ordinary factories retain their prior path.
pub(crate) fn string(
    text: &str,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    let Some(budget) = budget else {
        let value = DocumentString::from(text);
        #[cfg(test)]
        {
            let bytes = value
                .shared_code_units()
                .capacity()
                .checked_mul(size_of::<u16>())
                .expect("actual UTF16 allocation extent");
            if bytes != 0 {
                scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes });
            }
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<Vec<u16>>(),
            });
            if !value.is_empty() {
                scale_metrics::record(scale_metrics::Event::SequenceMaterialized {
                    bytes: value.len() * size_of::<u16>(),
                });
            }
        }
        return Ok(value);
    };
    budget.step(cancel)?;
    // Admit the UTF8 scan before deriving its exact UTF16 output length.
    for _ in 0..text.len() {
        budget.step(cancel)?;
    }
    let length = text.encode_utf16().count();
    budget.retain_payload(
        storage::Payload {
            bytes: size_of::<Vec<u16>>(),
            units: length,
        },
        cancel,
    )?;
    let mut units = Vec::new();
    reserve(&mut units, length, Some(budget), cancel)?;
    budget.retain_payload(
        storage::Payload {
            bytes: 0,
            units: units
                .capacity()
                .checked_sub(length)
                .ok_or(DomError::RepresentationExhausted)?,
        },
        cancel,
    )?;
    #[cfg(test)]
    let mut copied = transaction::CopyObservation::default();
    for unit in text.encode_utf16() {
        budget.step(cancel)?;
        units.push(unit);
        #[cfg(test)]
        copied.pushed::<u16>();
    }
    Ok(owned_units(units))
}

/// Fresh exact UTF16 output under its actual private owner. Its Vec header and
/// capacity remain owned by the caller's formation through immutable transfer.
pub(crate) fn utf16_buffer(
    length: usize,
    limit: usize,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<Vec<u16>, DomError> {
    admit(length, limit, DomResource::StringCodeUnits)?;
    length
        .checked_mul(size_of::<u16>())
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(DomError::RepresentationExhausted)?;
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: size_of::<Vec<u16>>(),
                units: length,
            },
            cancel,
        )?;
    }
    let mut output = Vec::new();
    if budget.is_some() {
        reserve(&mut output, length, budget, cancel)?;
    } else {
        output
            .try_reserve_exact(length)
            .map_err(|_| DomError::AllocationFailed)?;
        #[cfg(test)]
        if output.capacity() != 0 {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: output
                    .capacity()
                    .checked_mul(size_of::<u16>())
                    .expect("actual UTF16 allocation extent"),
            });
        }
    }
    admit(output.capacity(), limit, DomResource::StringCodeUnits)?;
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: 0,
                units: output
                    .capacity()
                    .checked_sub(length)
                    .ok_or(DomError::RepresentationExhausted)?,
            },
            cancel,
        )?;
    }
    Ok(output)
}

fn owned_units(units: Vec<u16>) -> DocumentString {
    let value = DocumentString::from_utf16(units);
    #[cfg(test)]
    scale_metrics::record(scale_metrics::Event::StructuralAllocation {
        bytes: size_of::<Vec<u16>>(),
    });
    value
}

/// Copies an actual borrowed UTF16 operand under the same private formation
/// lease. The returned string remains covered until its caller transfers it.
pub(crate) fn units(
    input: &[u16],
    lowercase: bool,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: size_of::<Vec<u16>>(),
                units: input.len(),
            },
            cancel,
        )?;
    }
    let mut output = Vec::new();
    reserve(&mut output, input.len(), budget, cancel)?;
    if let Some(budget) = budget {
        budget.retain_payload(
            storage::Payload {
                bytes: 0,
                units: output
                    .capacity()
                    .checked_sub(input.len())
                    .ok_or(DomError::RepresentationExhausted)?,
            },
            cancel,
        )?;
    }
    #[cfg(test)]
    let mut copied = transaction::CopyObservation::default();
    for unit in input {
        step(budget, cancel)?;
        output.push(if lowercase {
            crate::names::ascii_lower(*unit)
        } else {
            *unit
        });
        #[cfg(test)]
        copied.pushed::<u16>();
    }
    Ok(owned_units(output))
}
pub(crate) fn step(budget: Option<&Formation>, cancel: &AtomicBool) -> Result<(), DomError> {
    match budget {
        Some(budget) => budget.step(cancel),
        None => check_cancel(cancel),
    }
}

pub(in crate::store) fn merge_charge(
    retained: &mut Option<super::checkpoint::Charge>,
    incoming: Option<super::checkpoint::Charge>,
) {
    if let Some(incoming) = incoming {
        if let Some(retained) = retained {
            retained.merge(incoming);
        } else {
            *retained = Some(incoming);
        }
    }
}
pub(crate) fn share(
    value: &DocumentString,
    budget: Option<&Formation>,
    cancel: &AtomicBool,
) -> Result<DocumentString, DomError> {
    if let Some(budget) = budget {
        let mut payload = storage::Payload::default();
        payload.string(value)?;
        budget.retain_payload(payload, cancel)?;
    }
    Ok(value.clone())
}
pub(in crate::store) fn shared_string(
    draft: &Draft,
    value: &DocumentString,
    cancel: &AtomicBool,
) -> Result<(DocumentString, Option<super::checkpoint::Charge>), DomError> {
    let formation = Formation::new(draft, cancel)?;
    let value = share(value, formation.as_ref(), cancel)?;
    Ok((value, formation.map(Formation::into_charge)))
}

#[cfg(test)]
pub(in crate::store) mod mutation_observation {
    use super::*;
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Site {
        Children,
        SelectTasks,
        Metadata,
        Output,
        TextareaApi,
        InputString,
        CustomValidity,
        ReplaceData,
        InputAlias,
        InputRadioName,
        HyperlinkSnapshot,
        EncodingFacts,
        BaseFacts,
    }
    #[derive(Clone, Copy, Debug)]
    pub struct Growth {
        pub required: usize,
        pub limit: usize,
        pub required_units: usize,
        pub unit_limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<Option<Site>> = const { Cell::new(None) };
        static GROWTH: Cell<Option<Growth>> = const { Cell::new(None) };
    }
    pub fn start(site: Site) {
        ACTIVE.set(Some(site));
        GROWTH.set(None);
    }
    pub fn finish() -> Option<Growth> {
        ACTIVE.set(None);
        GROWTH.take()
    }
    pub fn before(
        draft: &Draft,
        site: Site,
        cancel: &AtomicBool,
    ) -> Result<Option<Growth>, DomError> {
        if ACTIVE.get() != Some(site) || GROWTH.get().is_some() {
            return Ok(None);
        }
        let Some(limits) = draft.source_limits else {
            return Ok(None);
        };
        Ok(Some(Growth {
            required: draft
                .source_storage_bytes(cancel)?
                .checked_add(draft.source_other_bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            limit: limits.retained_bytes,
            required_units: draft
                .source_storage_units(cancel)?
                .checked_add(draft.source_other_units)
                .ok_or(DomError::RepresentationExhausted)?,
            unit_limit: limits.retained_units,
        }))
    }
    pub fn grew(before: Option<Growth>, new_bank: usize) {
        grew_payload(before, new_bank, 0);
    }
    pub fn grew_payload(before: Option<Growth>, bytes: usize, units: usize) {
        if let Some(mut before) = before {
            before.required = before.required.checked_add(bytes).unwrap();
            before.required_units = before.required_units.checked_add(units).unwrap();
            GROWTH.set(Some(before));
        }
    }
}
