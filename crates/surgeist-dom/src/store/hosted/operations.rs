//! Exclusive generation-qualified controller rows. Preparations own only empty
//! paths and a destination holder; no resident Operation is shared or copied.
use super::*;
use bank::owned::{EmptyPath, OwnedBank, Taken};
mod index;
use index::{Index, KeyWords};

struct Resident {
    holder: Box<Option<Operation>>,
    _holder_extra: Retained,
    allowed: Projection,
}
impl Resident {
    fn operation(&self) -> &Operation {
        self.holder
            .as_ref()
            .as_ref()
            .expect("installed sole operation")
    }
    fn operation_mut(&mut self) -> &mut Operation {
        self.holder
            .as_mut()
            .as_mut()
            .expect("installed sole operation")
    }
}
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(in crate::store) struct Projection {
    retry: Option<KeyWords>,
    history: Option<KeyWords>,
    cleanup: Option<KeyWords>,
}
impl Projection {
    pub(super) fn invocation(
        creation: DomHostedCreation,
        history: Option<DomHostedOperationId>,
    ) -> Self {
        Self {
            retry: Some(KeyWords::retry(creation, history)),
            history: history.map(KeyWords::history),
            cleanup: None,
        }
    }
    pub(super) fn without_retry(&mut self) {
        self.retry = None;
    }
    pub(super) fn cleanup(creation: DomHostedCreation) -> Self {
        Self {
            cleanup: Some(KeyWords::cleanup(creation)),
            ..Default::default()
        }
    }
    fn operation(operation: &Operation) -> Self {
        match &operation.phase {
            CreationPhase::Invocation(row) => row.projection(),
            CreationPhase::DestroyEnqueue(row) | CreationPhase::DestroyCleanup(row) => {
                Self::cleanup(row.creation)
            }
            CreationPhase::Destroyed(_, creation) => Self::cleanup(*creation),
            _ => Self::default(),
        }
    }
}
#[derive(Default)]
pub(in crate::store) struct Operations {
    rows: OwnedBank<Resident>,
    retry: Index,
    history: Index,
    cleanup: Index,
    synchronous: Option<DomHostedOperationId>,
}
impl Operations {
    pub(in crate::store) fn get(
        &self,
        id: DomHostedOperationId,
    ) -> Result<&Operation, DomHostedError> {
        self.rows
            .get(id.slot)
            .map(Resident::operation)
            .filter(|op| op.request().key.operation == id)
            .ok_or(DomHostedError::StaleIdentity)
    }
    pub(in crate::store) fn get_mut(
        &mut self,
        id: DomHostedOperationId,
    ) -> Result<OperationMut<'_>, DomHostedError> {
        let taken = self.take(id)?;
        Ok(OperationMut {
            owner: self,
            taken: Some(taken),
        })
    }
    pub(super) fn retry(
        &self,
        creation: DomHostedCreation,
        history: Option<DomHostedOperationId>,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedOperationId>, DomHostedError> {
        work.step(cancel)?;
        Ok(self.retry.first(KeyWords::retry(creation,history)).filter(|&id|self.get(id).is_ok_and(|op|matches!(&op.phase,CreationPhase::Invocation(row) if row.matches_retry(creation,history)))))
    }
    pub(super) fn cleanup(
        &self,
        creation: DomHostedCreation,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedOperationId>, DomHostedError> {
        work.step(cancel)?;
        Ok(self
            .cleanup
            .first(KeyWords::cleanup(creation))
            .filter(|&id| {
                self.get(id).is_ok_and(|op| match &op.phase {
                    CreationPhase::DestroyEnqueue(row) | CreationPhase::DestroyCleanup(row) => {
                        row.creation.document == creation.document
                            && row.creation.global == creation.global
                    }
                    CreationPhase::Destroyed(_, actual) => {
                        actual.document == creation.document && actual.global == creation.global
                    }
                    _ => false,
                })
            }))
    }
    pub(super) fn history(
        &self,
        history: DomHostedOperationId,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedOperationId>, DomHostedError> {
        work.step(cancel)?;
        Ok(self.history.first(KeyWords::history(history)).filter(|&id|self.get(id).is_ok_and(|op|matches!(&op.phase,CreationPhase::Invocation(row) if row.matches_history(history)))))
    }
    pub(in crate::store) fn synchronous(&self) -> Option<DomHostedOperationId> {
        self.synchronous
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "One preparation admits the exact row, empty destination, original body credit, projection paths and both existing producer budgets before issuance"
    )]
    pub(in crate::store) fn prepare<'a>(
        &self,
        slot: usize,
        work: &Work,
        source: &DomHostedSource,
        scratch_base: usize,
        formation: Option<&source::formation::Formation>,
        credit: BodyCredit<'a>,
        projection: Projection,
        cancel: &AtomicBool,
    ) -> Result<PreparedRows<'a>, DomHostedError> {
        work.step(cancel)?;
        credit
            .receipt()
            .require_body_credit(source, size_of::<Operation>())?;
        if self.rows.get(slot).is_some() {
            return Err(DomHostedError::IncompleteOperation);
        }
        let header = size_of::<PreparedRows<'_>>();
        let mut staged = scratch_base
            .checked_add(header)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            staged,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let source_header = if let Some(f) = formation {
            f.retain_bytes(header, cancel)?;
            Some(f.split_payload(source::storage::Payload {
                bytes: header,
                units: 0,
            }))
        } else {
            None
        };
        let empty_held = self.rows.is_held(slot);
        let extra = if empty_held {
            0
        } else {
            size_of::<Option<Operation>>()
                .checked_sub(size_of::<Operation>())
                .ok_or(DomHostedError::RepresentationExhausted)?
        };
        // Original body credit is borrowed/owned separately; new storage only.
        if !empty_held {
            staged = staged
                .checked_add(size_of::<Option<Operation>>())
                .ok_or(DomHostedError::RepresentationExhausted)?;
            crate::hosted::admit(
                staged,
                source.limits().scratch_bytes(),
                DomHostedResource::ScratchBytes,
            )?;
        }
        let resident = if empty_held {
            None
        } else {
            let holder_extra = source.retain(extra, cancel)?;
            if let Some(f) = formation {
                f.retain_bytes(size_of::<Option<Operation>>(), cancel)?;
            }
            let resident = Resident {
                holder: Box::new(None),
                _holder_extra: holder_extra,
                allowed: projection,
            };
            #[cfg(test)]
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<Option<Operation>>(),
            });
            Some(resident)
        };
        let mut admit = |charge: bank::IndexCharge| {
            work.step(cancel)?;
            staged = staged
                .checked_add(charge.bytes)
                .ok_or(DomHostedError::RepresentationExhausted)?;
            crate::hosted::admit(
                staged,
                source.limits().scratch_bytes(),
                DomHostedResource::ScratchBytes,
            )?;
            if let Some(f) = formation {
                f.retain_bytes(charge.bytes, cancel)?;
            }
            source.retain(charge.bytes, cancel).map(Some)
        };
        let path = self.rows.reserve(slot, &mut admit)?;
        let retry = projection
            .retry
            .map(|key| {
                self.retry
                    .prepare(key, slot, &mut admit, formation, work, cancel)
            })
            .transpose()?;
        let history = projection
            .history
            .map(|key| {
                self.history
                    .prepare(key, slot, &mut admit, formation, work, cancel)
            })
            .transpose()?;
        let cleanup = projection
            .cleanup
            .map(|key| {
                self.cleanup
                    .prepare(key, slot, &mut admit, formation, work, cancel)
            })
            .transpose()?;
        let components = [
            path.charge().bytes,
            retry.as_ref().map_or(0, index::Plan::component_bytes),
            history.as_ref().map_or(0, index::Plan::component_bytes),
            cleanup.as_ref().map_or(0, index::Plan::component_bytes),
            resident
                .as_ref()
                .map_or(0, |_| size_of::<Option<Operation>>()),
        ]
        .into_iter()
        .try_fold(0usize, |sum, bytes| {
            sum.checked_add(bytes)
                .ok_or(DomHostedError::RepresentationExhausted)
        })?;
        let source_storage = formation.map(|f| {
            f.split_payload(source::storage::Payload {
                bytes: components,
                units: 0,
            })
        });
        Ok(PreparedRows {
            resident,
            path,
            retry,
            history,
            cleanup,
            _source_header: source_header,
            issuer: source.observe(),
            source_storage,
            credit,
        })
    }
    pub(in crate::store) fn insert_prepared(
        &mut self,
        transfer: Transfer,
        synchronous: bool,
    ) -> Option<source::checkpoint::Charge> {
        let Transfer {
            mut ready,
            operation,
        } = transfer;
        let id = operation.request().key.operation;
        assert_eq!(
            ready.path_index, id.slot,
            "same original issuer reservation"
        );
        assert!(
            self.rows.get(id.slot).is_none(),
            "original destination remains empty"
        );
        let mut source_storage = ready.source_storage.take();
        *ready.resident.holder = Some(operation);
        let moved = self.rows.insert(ready.path, ready.resident, false);
        let mut used = size_of::<Option<Operation>>()
            .checked_add(moved.bytes)
            .expect("moved admitted holder/row subset");
        for (index, plan) in [
            (&mut self.retry, ready.retry),
            (&mut self.history, ready.history),
            (&mut self.cleanup, ready.cleanup),
        ] {
            if let Some(plan) = plan {
                used = used
                    .checked_add(index.install(plan, id).bytes)
                    .expect("moved admitted projection subset");
            }
        }
        // Every unused spare and plan backing has now actually been destroyed.
        // The remaining affine subset follows the existing Source handoff.
        if let Some(charge) = &mut source_storage {
            charge.shrink_to_payload(source::storage::Payload {
                bytes: used,
                units: 0,
            });
        }
        if synchronous {
            self.synchronous = Some(id);
        }
        source_storage
    }
    fn retire_projections(&mut self, projection: Projection, id: DomHostedOperationId) {
        if let Some(key) = projection.retry {
            self.retry.retire(key, id);
        }
        if let Some(key) = projection.history {
            self.history.retire(key, id);
        }
        if let Some(key) = projection.cleanup {
            self.cleanup.retire(key, id);
        }
    }
    fn unindex(&mut self, projection: Projection, id: DomHostedOperationId) {
        if let Some(key) = projection.retry {
            self.retry.remove(key, id);
        }
        if let Some(key) = projection.history {
            self.history.remove(key, id);
        }
        if let Some(key) = projection.cleanup {
            self.cleanup.remove(key, id);
        }
    }
    fn reindex(&mut self, projection: Projection, id: DomHostedOperationId) {
        if let Some(key) = projection.retry {
            self.retry.restore(key, id);
        }
        if let Some(key) = projection.history {
            self.history.restore(key, id);
        }
        if let Some(key) = projection.cleanup {
            self.cleanup.restore(key, id);
        }
    }
    fn take(&mut self, id: DomHostedOperationId) -> Result<Taken<Resident>, DomHostedError> {
        let projection = Projection::operation(self.get(id)?);
        self.unindex(projection, id);
        Ok(self
            .rows
            .take(id.slot)
            .expect("validated original operation"))
    }
    fn restore(&mut self, taken: Taken<Resident>) {
        let op = taken.value().operation();
        let id = op.request().key.operation;
        let projection = Projection::operation(op);
        let allowed = taken.value().allowed;
        assert!(
            projection.retry.is_none() || projection.retry == allowed.retry,
            "retry projection only narrows after installation"
        );
        assert!(
            projection.history.is_none() || projection.history == allowed.history,
            "history projection only narrows after installation"
        );
        assert!(
            projection.cleanup.is_none() || projection.cleanup == allowed.cleanup,
            "cleanup projection only narrows after installation"
        );
        self.rows.restore(taken);
        self.reindex(projection, id);
    }
    fn finish_taken(&mut self, taken: Taken<Resident>) {
        let id = taken.value().operation().request().key.operation;
        self.retire_projections(taken.value().allowed, id);
        drop(self.rows.finish_taken(taken));
        if self.synchronous == Some(id) {
            self.synchronous = None;
        }
    }
    pub(in crate::store) fn remove(
        &mut self,
        id: DomHostedOperationId,
    ) -> Result<(), DomHostedError> {
        self.get(id)?;
        let allowed = self.rows.get(id.slot).expect("original row").allowed;
        self.retire_projections(allowed, id);
        drop(self.rows.remove(id.slot));
        if self.synchronous == Some(id) {
            self.synchronous = None;
        }
        Ok(())
    }
}
/// A fresh body credit moves here; a Source body credit remains borrowed from
/// its original pending phase until the infallible data-first transfer guard.
pub(in crate::store) enum BodyCredit<'a> {
    Owned(Retained),
    Borrowed(&'a Retained),
}
impl BodyCredit<'_> {
    fn receipt(&self) -> &Retained {
        match self {
            Self::Owned(r) => r,
            Self::Borrowed(r) => r,
        }
    }
}
pub(in crate::store) struct PreparedRows<'a> {
    resident: Option<Resident>,
    path: EmptyPath<Resident>,
    retry: Option<index::Plan>,
    history: Option<index::Plan>,
    cleanup: Option<index::Plan>,
    _source_header: Option<source::checkpoint::Charge>,
    issuer: SourceObservation,
    source_storage: Option<source::checkpoint::Charge>,
    credit: BodyCredit<'a>,
}
pub(in crate::store) struct ReadyRows {
    resident: Resident,
    path: EmptyPath<Resident>,
    path_index: usize,
    retry: Option<index::Plan>,
    history: Option<index::Plan>,
    cleanup: Option<index::Plan>,
    _source_header: Option<source::checkpoint::Charge>,
    issuer: SourceObservation,
    source_storage: Option<source::checkpoint::Charge>,
}
/// Data comes first on every refusal/panic/drop. No fallible operation follows
/// construction: install moves the original body into the preallocated holder.
pub(in crate::store) struct Transfer {
    ready: ReadyRows,
    operation: Operation,
}
pub(in crate::store) struct SourceTransfer<P> {
    ready: ReadyRows,
    payload: P,
}
impl ReadyRows {
    pub(in crate::store) fn with_payload<P>(self, payload: P) -> SourceTransfer<P> {
        SourceTransfer {
            ready: self,
            payload,
        }
    }
}
impl<P> SourceTransfer<P> {
    pub(in crate::store) fn payload(&self) -> &P {
        &self.payload
    }
    pub(in crate::store) fn into_operation_and<R>(
        self,
        build: impl FnOnce(P) -> (Operation, R),
    ) -> (Transfer, R) {
        let Self { ready, payload } = self;
        let (operation, result) = build(payload);
        (Transfer::new(ready, operation), result)
    }
    pub(in crate::store) fn into_operation(self, build: impl FnOnce(P) -> Operation) -> Transfer {
        let Self { ready, payload } = self;
        Transfer::new(ready, build(payload))
    }
}
impl Transfer {
    pub fn new(ready: ReadyRows, operation: Operation) -> Self {
        operation
            ._retained
            .require_observed_body_credit(&ready.issuer, size_of::<Operation>())
            .expect("same original admitted body credit at infallible transfer");
        assert!(
            Projection::operation(&operation) == ready.resident.allowed,
            "same preadmitted operation projections"
        );
        Self { ready, operation }
    }
}
impl<'a> PreparedRows<'a> {
    fn ready(self) -> (ReadyRows, BodyCredit<'a>) {
        let Self {
            resident,
            path,
            retry,
            history,
            cleanup,
            _source_header,
            issuer,
            source_storage,
            credit,
        } = self;
        let path_index = path.index();
        (
            ReadyRows {
                resident: resident.expect("held-only empty reservation cannot install"),
                path,
                path_index,
                retry,
                history,
                cleanup,
                _source_header,
                issuer,
                source_storage,
            },
            credit,
        )
    }
    pub(in crate::store) fn into_owned(self) -> (ReadyRows, Retained) {
        let (ready, credit) = self.ready();
        let BodyCredit::Owned(credit) = credit else {
            panic!("fresh-owned transfer")
        };
        (ready, credit)
    }
    pub(in crate::store) fn into_ready(self) -> ReadyRows {
        let (ready, credit) = self.ready();
        assert!(
            matches!(credit, BodyCredit::Borrowed(_)),
            "Source borrowed transfer"
        );
        ready
    }
    pub(in crate::store) fn backing_bytes(&self) -> Result<usize, DomHostedError> {
        [
            self.path.charge().bytes,
            self.retry.as_ref().map_or(0, index::Plan::backing_bytes),
            self.history.as_ref().map_or(0, index::Plan::backing_bytes),
            self.cleanup.as_ref().map_or(0, index::Plan::backing_bytes),
            self.resident
                .as_ref()
                .map_or(0, |_| size_of::<Option<Operation>>()),
        ]
        .into_iter()
        .try_fold(0usize, |sum, bytes| {
            sum.checked_add(bytes)
                .ok_or(DomHostedError::RepresentationExhausted)
        })
    }
    pub(in crate::store) fn scratch_bytes(&self) -> Result<usize, DomHostedError> {
        size_of::<Self>()
            .checked_add(self.backing_bytes()?)
            .ok_or(DomHostedError::RepresentationExhausted)
    }
    pub(in crate::store) fn operation_scratch_bytes(&self) -> Result<usize, DomHostedError> {
        size_of::<Operation>()
            .checked_add(self.scratch_bytes()?)
            .ok_or(DomHostedError::RepresentationExhausted)
    }
}
#[cfg(test)]
impl PreparedRows<'_> {
    pub(super) fn inspect_new_storage_bytes(&self) -> usize {
        self.path.inspect_storage_bytes()
            + self
                .retry
                .as_ref()
                .map_or(0, index::Plan::inspect_storage_bytes)
            + self
                .history
                .as_ref()
                .map_or(0, index::Plan::inspect_storage_bytes)
            + self
                .cleanup
                .as_ref()
                .map_or(0, index::Plan::inspect_storage_bytes)
            + self
                .resident
                .as_ref()
                .map_or(0, |_| size_of::<Option<Operation>>())
    }
}
#[cfg(test)]
impl Operations {
    pub(super) fn inspect_storage_bytes(&self) -> usize {
        self.rows.inspect_storage_bytes()
            + self.rows.values().count() * size_of::<Option<Operation>>()
            + self.retry.inspect_storage_bytes()
            + self.history.inspect_storage_bytes()
            + self.cleanup.inspect_storage_bytes()
    }
}
pub(in crate::store) struct OperationMut<'a> {
    owner: &'a mut Operations,
    taken: Option<Taken<Resident>>,
}
impl std::ops::Deref for OperationMut<'_> {
    type Target = Operation;
    fn deref(&self) -> &Operation {
        self.taken.as_ref().unwrap().value().operation()
    }
}
impl std::ops::DerefMut for OperationMut<'_> {
    fn deref_mut(&mut self) -> &mut Operation {
        self.taken.as_mut().unwrap().value_mut().operation_mut()
    }
}
impl Drop for OperationMut<'_> {
    fn drop(&mut self) {
        if let Some(taken) = self.taken.take() {
            self.owner.restore(taken);
        }
    }
}
pub(in crate::store) enum OperationResult<T> {
    Retain(T),
    Complete(T),
}
struct Borrowed<'a> {
    dom: &'a mut Dom,
    operation: Option<Taken<Resident>>,
}
impl Drop for Borrowed<'_> {
    fn drop(&mut self) {
        if let Some(operation) = self.operation.take() {
            self.dom.hosted_operations.restore(operation);
        }
    }
}
impl Dom {
    pub(in crate::store) fn with_hosted_operation<T>(
        &mut self,
        id: DomHostedOperationId,
        action: impl FnOnce(&mut Dom, &mut Operation) -> Result<OperationResult<T>, DomHostedError>,
    ) -> Result<T, DomHostedError> {
        let operation = self.hosted_operations.take(id)?;
        let mut borrowed = Borrowed {
            dom: self,
            operation: Some(operation),
        };
        match action(
            borrowed.dom,
            borrowed
                .operation
                .as_mut()
                .unwrap()
                .value_mut()
                .operation_mut(),
        )? {
            OperationResult::Retain(result) => Ok(result),
            OperationResult::Complete(result) => {
                let taken = borrowed.operation.take().unwrap();
                borrowed.dom.hosted_operations.finish_taken(taken);
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests;
