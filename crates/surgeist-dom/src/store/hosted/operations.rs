//! The original hosted controller's finite generation-checked operation rows.
//! Permissions never own the only copy of a parked mandatory continuation.
use super::*;

#[derive(Default)]
pub(in crate::store) struct Operations {
    rows: Vec<Option<Box<Operation>>>,
    retained: Option<Retained>,
    synchronous: Option<DomHostedOperationId>,
}
impl Operations {
    pub(in crate::store) fn get(
        &self,
        id: DomHostedOperationId,
    ) -> Result<&Operation, DomHostedError> {
        self.rows
            .get(id.slot)
            .and_then(Option::as_deref)
            .filter(|operation| operation.request().key.operation == id)
            .ok_or(DomHostedError::StaleIdentity)
    }
    pub(in crate::store) fn get_mut(
        &mut self,
        id: DomHostedOperationId,
    ) -> Result<&mut Operation, DomHostedError> {
        self.rows
            .get_mut(id.slot)
            .and_then(Option::as_deref_mut)
            .filter(|operation| operation.request().key.operation == id)
            .ok_or(DomHostedError::StaleIdentity)
    }
    pub(super) fn find(
        &self,
        work: &Work,
        cancel: &AtomicBool,
        mut predicate: impl FnMut(&Operation) -> bool,
    ) -> Result<Option<DomHostedOperationId>, DomHostedError> {
        for row in &self.rows {
            work.step(cancel)?;
            if let Some(row) = row
                && predicate(row)
            {
                return Ok(Some(row.request().key.operation));
            }
        }
        Ok(None)
    }
    pub(in crate::store) fn synchronous(&self) -> Option<DomHostedOperationId> {
        self.synchronous
    }
    /// A private candidate remains independently charged until original role
    /// issuance and every other fallible prefix check succeeds.
    pub(in crate::store) fn prepare(
        &self,
        slot: usize,
        work: &Work,
        source: &DomHostedSource,
        scratch_base: usize,
        formation: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<PreparedRows, DomHostedError> {
        work.step(cancel)?;
        let scratch_header = scratch_base
            .checked_add(size_of::<PreparedRows>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            scratch_header,
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        let source_header = if let Some(formation) = formation {
            // A growing candidate's permanent debit below already includes its
            // Operations header, which is inline in PreparedRows. Keep only the
            // remaining temporary header here; its charge ends at handoff while
            // the candidate's original debit follows the installed bank.
            let candidate_header = if slot >= self.rows.capacity() {
                size_of::<Operations>()
            } else {
                0
            };
            let payload = source::storage::Payload {
                bytes: size_of::<PreparedRows>()
                    .checked_sub(candidate_header)
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                units: 0,
            };
            formation.retain_payload(payload, cancel)?;
            Some(formation.split_payload(payload))
        } else {
            None
        };
        if self.rows.get(slot).is_some_and(Option::is_some) {
            return Err(DomHostedError::IncompleteOperation);
        }
        let required = slot
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let candidate = if required > self.rows.capacity() {
            let extent = |capacity: usize| {
                capacity
                    .checked_mul(size_of::<Option<Box<Operation>>>())
                    .and_then(|bytes| bytes.checked_add(size_of::<Operations>()))
                    .ok_or(DomHostedError::RepresentationExhausted)
            };
            let requested = extent(required)?;
            crate::hosted::admit(
                scratch_header
                    .checked_add(
                        required
                            .checked_mul(size_of::<Option<Box<Operation>>>())
                            .ok_or(DomHostedError::RepresentationExhausted)?,
                    )
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                source.limits().scratch_bytes(),
                DomHostedResource::ScratchBytes,
            )?;
            if let Some(formation) = formation {
                formation.retain_bytes(requested, cancel)?;
            }
            let retained = source.retain(requested, cancel)?;
            let mut candidate = Operations {
                rows: Vec::new(),
                retained: Some(retained),
                synchronous: None,
            };
            candidate
                .rows
                .try_reserve_exact(required)
                .map_err(|_| DomHostedError::AllocationFailed)?;
            let actual = extent(candidate.rows.capacity())?;
            crate::hosted::admit(
                scratch_header
                    .checked_add(
                        candidate
                            .rows
                            .capacity()
                            .checked_mul(size_of::<Option<Box<Operation>>>())
                            .ok_or(DomHostedError::RepresentationExhausted)?,
                    )
                    .ok_or(DomHostedError::RepresentationExhausted)?,
                source.limits().scratch_bytes(),
                DomHostedResource::ScratchBytes,
            )?;
            if let Some(formation) = formation {
                formation.retain_bytes(
                    actual
                        .checked_sub(requested)
                        .ok_or(DomError::RepresentationExhausted)?,
                    cancel,
                )?;
            }
            candidate
                .retained
                .as_mut()
                .expect("candidate owner debit")
                .append(
                    source.retain(
                        actual
                            .checked_sub(requested)
                            .ok_or(DomHostedError::RepresentationExhausted)?,
                        cancel,
                    )?,
                )?;
            Some(candidate)
        } else {
            None
        };
        for _ in &self.rows {
            work.step(cancel)?;
        }
        for _ in self.rows.len()..required {
            work.step(cancel)?;
        }
        Ok(PreparedRows {
            candidate,
            slot,
            required,
            _source_header: source_header,
        })
    }
    pub(in crate::store) fn insert_prepared(
        &mut self,
        prepared: PreparedRows,
        operation: Operation,
        synchronous: bool,
    ) {
        let PreparedRows {
            candidate,
            slot,
            required,
            _source_header,
        } = prepared;
        let id = operation.request().key.operation;
        assert_eq!(slot, id.slot, "same original issuer reservation");
        assert!(
            self.rows.get(slot).is_none_or(Option::is_none),
            "reserved empty operation slot"
        );
        if let Some(mut candidate) = candidate {
            for row in &mut self.rows {
                candidate.rows.push(row.take());
            }
            candidate.synchronous = self.synchronous;
            let old = std::mem::replace(self, candidate);
            // The old backing is destroyed before its original debit.
            drop(old);
        }
        while self.rows.len() < required {
            self.rows.push(None);
        }
        self.rows[slot] = Some(Box::new(operation));
        if synchronous {
            self.synchronous = Some(id);
        }
    }
    fn take(&mut self, id: DomHostedOperationId) -> Result<Box<Operation>, DomHostedError> {
        self.get(id)?;
        Ok(self.rows[id.slot]
            .take()
            .expect("validated original operation"))
    }
    fn restore(&mut self, operation: Box<Operation>) {
        let id = operation.request().key.operation;
        debug_assert!(self.rows[id.slot].is_none());
        self.rows[id.slot] = Some(operation);
    }
    pub(in crate::store) fn remove(
        &mut self,
        id: DomHostedOperationId,
    ) -> Result<(), DomHostedError> {
        self.get(id)?;
        self.rows[id.slot].take();
        if self.synchronous == Some(id) {
            self.synchronous = None;
        }
        Ok(())
    }
}

pub(in crate::store) struct PreparedRows {
    candidate: Option<Operations>,
    slot: usize,
    required: usize,
    _source_header: Option<source::checkpoint::Charge>,
}

impl PreparedRows {
    pub(in crate::store) fn backing_bytes(&self) -> Result<usize, DomHostedError> {
        self.candidate.as_ref().map_or(Ok(0), |candidate| {
            candidate
                .rows
                .capacity()
                .checked_mul(size_of::<Option<Box<Operation>>>())
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
impl Operations {
    pub(super) fn row_capacity(&self) -> usize {
        self.rows.capacity()
    }
}

#[cfg(test)]
impl PreparedRows {
    pub(super) fn candidate_capacity(&self) -> Option<usize> {
        self.candidate
            .as_ref()
            .map(|candidate| candidate.rows.capacity())
    }
}

pub(in crate::store) enum OperationResult<T> {
    Retain(T),
    Complete(T),
}
/// The real row remains recoverable if a fixed internal operation panics. No
/// callback or alternate controller can observe the temporarily borrowed slot.
struct Borrowed<'a> {
    dom: &'a mut Dom,
    operation: Option<Box<Operation>>,
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
                .as_deref_mut()
                .expect("original operation guard"),
        )? {
            OperationResult::Retain(result) => Ok(result),
            OperationResult::Complete(result) => {
                borrowed.operation.take();
                if borrowed.dom.hosted_operations.synchronous == Some(id) {
                    borrowed.dom.hosted_operations.synchronous = None;
                }
                Ok(result)
            }
        }
    }
}
