//! One bounded weak index, independent of historical snapshots and change logs.
use super::*;
use crate::DomDocumentStamp;
use crate::dispatch::{
    DomDispatchAssociation, DomDispatchCustody, DomDispatchCustodyLimits, Row, Scope,
};
use source::formation::Formation;
use std::{
    cell::Cell,
    sync::{OnceLock, Weak},
};

#[derive(Default)]
pub(super) struct CustodyBank {
    scopes: Vec<Weak<Scope>>,
}

/// Actual staged association, resolved by the original Draft's final mapping.
pub(super) struct Retirement {
    pub target: NodeId,
    pub document: DocumentId,
}

struct Processing<'a> {
    limits: DomDispatchCustodyLimits,
    spent: Cell<u64>,
    formation: Option<&'a Formation>,
    cancel: &'a AtomicBool,
}
impl<'a> Processing<'a> {
    fn new(
        limits: DomDispatchCustodyLimits,
        formation: Option<&'a Formation>,
        cancel: &'a AtomicBool,
    ) -> Self {
        Self {
            limits,
            spent: Cell::new(0),
            formation,
            cancel,
        }
    }
    fn step(&self) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        if let Some(formation) = self.formation {
            formation.step(self.cancel)?;
        }
        let next = self
            .spent
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomError::DispatchCustodyWorkLimit {
                limit: self.limits.work,
            });
        }
        self.spent.set(next);
        Ok(())
    }
    fn scratch(&self, bytes: usize) -> Result<(), DomError> {
        admit(
            bytes,
            self.limits.scratch_bytes,
            DomResource::DispatchCustodyScratch,
        )
    }
}

#[derive(Default)]
struct Extent {
    scopes: usize,
    references: usize,
    bytes: usize,
}
impl Extent {
    fn admit(&self, limits: DomDispatchCustodyLimits) -> Result<(), DomError> {
        admit(
            self.scopes,
            limits.scopes,
            DomResource::DispatchCustodyScopes,
        )?;
        admit(
            self.references,
            limits.references,
            DomResource::DispatchCustodyReferences,
        )?;
        admit(
            self.bytes,
            limits.retained_bytes,
            DomResource::DispatchCustodyRetainedBytes,
        )
    }
}
fn bytes<T>(capacity: usize) -> Result<usize, DomError> {
    capacity
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)
}
fn add(a: usize, b: usize) -> Result<usize, DomError> {
    a.checked_add(b).ok_or(DomError::RepresentationExhausted)
}

impl CustodyBank {
    /// Whether admission must scan actual index entries, including dead Weak
    /// entries. An empty index can still retain capacity requiring admission.
    pub(super) fn has_indexed_scopes(&self) -> bool {
        !self.scopes.is_empty()
    }

    /// Only an actual dependent scope needs a public retirement association.
    /// The caller supplies its original work admission for every reached scan.
    pub(super) fn tracks(
        &self,
        target: NodeId,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        for scope in &self.scopes {
            step()?;
            let Some(scope) = scope.upgrade() else {
                continue;
            };
            for row in &scope.rows {
                step()?;
                if row.target == target {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    /// Source envelopes charge all bodies through this one canonical basis.
    /// Event payloads exclude that same producer term only in their Source
    /// debit; their actual Runtime storage still includes the complete scope.
    pub(super) fn retained_bytes(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        Ok(self.extent_with(step)?.bytes)
    }
    /// Pre-admit the complete reconciliation before deterministic no-callback
    /// release. Dead Weak headers remain charged until this actual pruning.
    fn prune(&mut self, processing: &Processing<'_>) -> Result<(), DomError> {
        for _ in &self.scopes {
            processing.step()?;
        }
        self.scopes.retain(|scope| scope.strong_count() != 0);
        Ok(())
    }
    fn extent(&self, processing: &Processing<'_>) -> Result<Extent, DomError> {
        self.extent_with(|| processing.step())
    }
    fn extent_with(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Extent, DomError> {
        let mut extent = Extent {
            scopes: self.scopes.capacity(),
            references: 0,
            bytes: bytes::<Weak<Scope>>(self.scopes.capacity())?,
        };
        for scope in &self.scopes {
            step()?;
            // The Arc allocation's Scope payload remains until its final Weak
            // drops, even when the Scope destructor already released its rows.
            extent.bytes = add(extent.bytes, size_of::<Scope>())?;
            if let Some(scope) = scope.upgrade() {
                extent.references = add(extent.references, scope.rows.len())?;
                extent.bytes = add(extent.bytes, bytes::<Row>(scope.rows.len())?)?;
            }
        }
        Ok(extent)
    }
    pub(super) fn admit_limits(
        &mut self,
        limits: DomDispatchCustodyLimits,
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let processing = Processing::new(limits, formation, cancel);
        self.prune(&processing)?;
        self.extent(&processing)?.admit(limits)
    }
    fn register(
        &mut self,
        owner: DomOwnerId,
        state: &State,
        targets: &[NodeId],
        processing: &Processing<'_>,
    ) -> Result<DomDispatchCustody, DomError> {
        let limits = processing.limits;
        let formation = processing.formation;
        let cancel = processing.cancel;
        self.prune(processing)?;
        let mut extent = self.extent(processing)?;
        let count = add(self.scopes.len(), 1)?;
        admit(count, limits.scopes, DomResource::DispatchCustodyScopes)?;
        processing.step()?; // Scope/header registration and eventual release.
        extent.references = add(extent.references, targets.len())?;
        admit(
            extent.references,
            limits.references,
            DomResource::DispatchCustodyReferences,
        )?;
        let payload = DomDispatchCustody::maximum_retained_bytes(targets.len())?;
        extent.bytes = add(extent.bytes, payload)?;
        for target in targets {
            processing.step()?;
            state.node(owner, *target)?;
        }
        // The old bank, replacement bank, row construction Vec and boxed rows
        // can coexist. Admit that whole frontier before allocating either bank.
        let row_bytes = bytes::<Row>(targets.len())?;
        let bank_bytes = if count > self.scopes.capacity() {
            bytes::<Weak<Scope>>(count)?
        } else {
            0
        };
        let construction = add(add(bank_bytes, row_bytes)?, row_bytes)?;
        processing.scratch(construction)?;
        admit(
            add(extent.bytes, bank_bytes)?,
            limits.retained_bytes,
            DomResource::DispatchCustodyRetainedBytes,
        )?;
        let guard = formation.map(Formation::fork);
        if let Some(guard) = &guard {
            guard.retain_bytes(
                add(bank_bytes, payload.max(add(row_bytes, row_bytes)?))?,
                cancel,
            )?;
        }
        let mut replacement = Vec::new();
        if bank_bytes != 0 {
            replacement
                .try_reserve_exact(count)
                .map_err(|_| DomError::AllocationFailed)?;
            let actual = bytes::<Weak<Scope>>(replacement.capacity())?;
            admit(
                replacement.capacity(),
                limits.scopes,
                DomResource::DispatchCustodyScopes,
            )?;
            processing.scratch(add(add(actual, row_bytes)?, row_bytes)?)?;
            admit(
                add(extent.bytes, actual)?,
                limits.retained_bytes,
                DomResource::DispatchCustodyRetainedBytes,
            )?;
            if let Some(guard) = &guard {
                guard.retain_bytes(actual - bank_bytes, cancel)?;
            }
            for scope in &self.scopes {
                processing.step()?;
                replacement.push(scope.clone());
            }
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(targets.len())
            .map_err(|_| DomError::AllocationFailed)?;
        let actual_rows = bytes::<Row>(rows.capacity())?;
        let actual_bank = bytes::<Weak<Scope>>(replacement.capacity())?;
        processing.scratch(add(add(actual_bank, actual_rows)?, row_bytes)?)?;
        if let Some(guard) = &guard {
            guard.retain_bytes(actual_rows - row_bytes, cancel)?;
        }
        for target in targets {
            processing.step()?; // Row construction.
            processing.step()?; // Whole-scope eventual release, pre-admitted.
            rows.push(Row {
                target: *target,
                retired: OnceLock::new(),
            });
        }
        let scope = Arc::new(Scope {
            owner,
            rows: rows.into_boxed_slice(),
        });
        check_cancel(cancel)?;
        // No fallible work follows publication of the registration.
        if bank_bytes != 0 {
            replacement.push(Arc::downgrade(&scope));
            self.scopes = replacement;
        } else {
            self.scopes.push(Arc::downgrade(&scope));
        }
        Ok(DomDispatchCustody { scope })
    }
    pub(super) fn prepare_publication(
        &mut self,
        retirements: &[Retirement],
        stamp: DomStamp,
        limits: DomDispatchCustodyLimits,
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<PublicationPlan, DomError> {
        // Disabled custody is not an extra condition on ordinary destruction.
        if self.scopes.is_empty() || retirements.is_empty() {
            return Ok(PublicationPlan::default());
        }
        let processing = Processing::new(limits, formation, cancel);
        self.prune(&processing)?;
        self.extent(&processing)?.admit(limits)?;
        let mut count = 0;
        for scope in &self.scopes {
            processing.step()?;
            let Some(scope) = scope.upgrade() else {
                continue;
            };
            for row in &scope.rows {
                for retirement in retirements {
                    processing.step()?;
                    if row.target == retirement.target {
                        if row.retired.get().is_some() {
                            return Err(DomError::InvalidDispatchCustody);
                        }
                        count = add(count, 1)?;
                    }
                }
            }
        }
        let guard = formation.map(Formation::fork);
        let requested = bytes::<PlannedRow>(count)?;
        processing.scratch(requested)?;
        if let Some(guard) = &guard {
            guard.retain_bytes(requested, cancel)?;
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = bytes::<PlannedRow>(rows.capacity())?;
        processing.scratch(actual)?;
        if let Some(guard) = &guard {
            guard.retain_bytes(actual - requested, cancel)?;
        }
        for scope in &self.scopes {
            processing.step()?;
            let Some(scope) = scope.upgrade() else {
                continue;
            };
            for (index, row) in scope.rows.iter().enumerate() {
                for retirement in retirements {
                    processing.step()?;
                    if row.target == retirement.target {
                        if retirement.document.node().owner() != stamp.owner()
                            || row.retired.get().is_some()
                        {
                            return Err(DomError::InvalidDispatchCustody);
                        }
                        processing.step()?; // Pre-admit the final infallible cell publication.
                        rows.push(PlannedRow {
                            scope: scope.clone(),
                            index,
                            stamp: DomDocumentStamp {
                                store: stamp,
                                document: retirement.document,
                            },
                        });
                    }
                }
            }
        }
        // Duplicate packet identities would otherwise try to set one cell twice.
        for (index, row) in rows.iter().enumerate() {
            for prior in &rows[..index] {
                processing.step()?;
                if Arc::ptr_eq(&row.scope, &prior.scope) && row.index == prior.index {
                    return Err(DomError::InvalidDispatchCustody);
                }
            }
        }
        check_cancel(cancel)?;
        Ok(PublicationPlan {
            rows,
            _formation: guard,
        })
    }
}

struct PlannedRow {
    scope: Arc<Scope>,
    index: usize,
    stamp: DomDocumentStamp,
}
#[derive(Default)]
pub(super) struct PublicationPlan {
    rows: Vec<PlannedRow>,
    _formation: Option<Formation>,
}
impl PublicationPlan {
    /// Called only at the exclusive, infallible canonical publication fence.
    pub(super) fn publish(self) {
        for row in self.rows {
            assert!(
                row.scope.rows[row.index].retired.set(row.stamp).is_ok(),
                "preflighted unpublished retirement cell"
            );
        }
    }
}

impl Dom {
    /// Actual aggregate producer-owned payload across every registered scope
    /// and the canonical weak index, including retained physical capacity.
    pub fn dispatch_custody_retained_bytes(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        let formation = self.dispatch_registration_formation(cancel)?;
        let processing = Processing::new(self.limits.dispatch_custody, formation.as_ref(), cancel);
        Ok(self.dispatch_custody.extent(&processing)?.bytes)
    }
    /// Register all actual Node invocation targets before exposing a dispatch.
    pub fn register_dispatch_custody(
        &mut self,
        targets: &[NodeId],
        cancel: &AtomicBool,
    ) -> Result<DomDispatchCustody, DomError> {
        let formation = self.dispatch_registration_formation(cancel)?;
        self.register_dispatch_custody_with_formation(targets, formation.as_ref(), cancel)
    }
    pub(in crate::store) fn register_dispatch_custody_with_formation(
        &mut self,
        targets: &[NodeId],
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomDispatchCustody, DomError> {
        let processing = Processing::new(self.limits.dispatch_custody, formation, cancel);
        self.dispatch_custody
            .register(self.owner, &self.state, targets, &processing)
    }
    /// Qualify exactly one original row against this canonical owner. Live rows
    /// always use current association; a missing retirement cell remains refusal.
    pub fn dispatch_custody_association(
        &self,
        custody: &DomDispatchCustody,
        row: usize,
        target: NodeId,
    ) -> Result<DomDispatchAssociation, DomError> {
        if custody.scope.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let row = custody
            .scope
            .rows
            .get(row)
            .filter(|row| row.target == target)
            .ok_or(DomError::InvalidDispatchCustody)?;
        match self.state.node(self.owner, target) {
            Ok(node) => Ok(DomDispatchAssociation::Current(node.document)),
            Err(DomError::StaleNode) => row
                .retired
                .get()
                .copied()
                .map(DomDispatchAssociation::Retired)
                .ok_or(DomError::StaleNode),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod incremental_tests;
#[cfg(test)]
mod tests;
