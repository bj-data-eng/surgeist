//! The live invocation owner and its exact work-identity index. Neither bank
//! enters immutable DOM snapshots or supplies execution authority to callers.
use std::{
    mem::size_of,
    ops::Index,
    sync::{Arc, atomic::AtomicU64},
};

use super::exit::Invocation;
use crate::store::{
    DomError,
    bank::{Bank, BankCustody, ControlledRecord, CustodyRegistry, Totals},
    custody::Usage,
};
use crate::{DomFullscreenExitId, DomOwnerId};

pub(in crate::store) struct Invocations {
    records: Bank<Invocation>,
    work: Bank<DomFullscreenExitId>,
    record_custody: BankCustody<Invocation>,
    work_custody: BankCustody<DomFullscreenExitId>,
    registry: CustodyRegistry,
    owner: Option<DomOwnerId>,
    closing_credit: u64,
}

impl Clone for Invocations {
    fn clone(&self) -> Self {
        Self {
            records: self.records.clone(),
            work: self.work.clone(),
            record_custody: self.record_custody.clone(),
            work_custody: self.work_custody.clone(),
            registry: self.registry.clone(),
            owner: self.owner,
            closing_credit: self.closing_credit,
        }
    }
}

impl Drop for Invocations {
    fn drop(&mut self) {
        // Discard real data before releasing the associated controlled debit.
        drop(std::mem::take(&mut self.records));
        drop(std::mem::take(&mut self.work));
        drop(std::mem::take(&mut self.record_custody));
        drop(std::mem::take(&mut self.work_custody));
        self.registry.reconcile_releases();
    }
}

impl Invocations {
    pub fn new(registry: &CustodyRegistry) -> Self {
        Self {
            records: Bank::default(),
            work: Bank::default(),
            record_custody: BankCustody::default(),
            work_custody: BankCustody::default(),
            registry: registry.clone(),
            owner: None,
            closing_credit: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn get(&self, slot: usize) -> Option<&Invocation> {
        self.records.get(slot)
    }

    pub fn closing_credit(&self) -> u64 {
        self.closing_credit
    }

    pub fn work_custody(
        &self,
        work: &Arc<AtomicU64>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<DomFullscreenExitId>, DomError> {
        step()?;
        Ok(self.work.get(work_key(work)).copied())
    }

    /// Prepare both roots before their common publication fence. Existing
    /// records, paths and work atoms remain shared with their actual custodians.
    pub fn prepare_append(
        &self,
        invocation: Invocation,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let id = invocation.id;
        if self.owner.is_some_and(|owner| owner != id.owner) {
            return Err(DomError::ForeignOwner);
        }
        if id.slot != self.len() || id.generation == 0 {
            return Err(DomError::InvalidSourceOperation);
        }
        let closing_credit = self
            .closing_credit
            .checked_add(invocation.closing_credit)
            .ok_or(DomError::RepresentationExhausted)?;
        step()?;
        let work = invocation.work_custody.work().clone();
        let mut ancestor = invocation.retained_ancestors.clone();
        let mut candidate = self.clone();
        let record = ControlledRecord::new(
            invocation,
            Usage::new(size_of::<Invocation>(), 0),
            &self.registry,
            &mut admit,
        )?;
        let (records, custody) = self.records.set_controlled(
            &self.record_custody,
            id.slot,
            record,
            false,
            Totals {
                bytes: size_of::<Invocation>(),
                units: 0,
                occurrences: 1,
            },
            &self.registry,
            &mut admit,
        )?;
        candidate.records = records;
        candidate.record_custody = custody;
        candidate.owner = Some(id.owner);
        candidate.closing_credit = closing_credit;
        candidate.index_work(&work, id, &mut admit, &mut step)?;
        while let Some(event) = ancestor {
            candidate.index_work(event.source().work(), id, &mut admit, &mut step)?;
            ancestor = event.ancestor();
        }
        Ok(candidate)
    }

    fn index_work(
        &mut self,
        work: &Arc<AtomicU64>,
        id: DomFullscreenExitId,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        step()?;
        let key = work_key(work);
        // Invocations append in slot order. Preserve the first original owner
        // when several invocations share the same counter or event ancestor.
        if self.work.get(key).is_some() {
            return Ok(());
        }
        let record = ControlledRecord::new(
            id,
            Usage::new(size_of::<DomFullscreenExitId>(), 0),
            &self.registry,
            &mut *admit,
        )?;
        let (work, custody) = self.work.set_controlled(
            &self.work_custody,
            key,
            record,
            false,
            Totals {
                bytes: size_of::<DomFullscreenExitId>(),
                units: 0,
                occurrences: 1,
            },
            &self.registry,
            admit,
        )?;
        self.work = work;
        self.work_custody = custody;
        Ok(())
    }
}

impl Index<usize> for Invocations {
    type Output = Invocation;

    fn index(&self, slot: usize) -> &Self::Output {
        self.get(slot)
            .expect("an issued invocation slot remains live")
    }
}

fn work_key(work: &Arc<AtomicU64>) -> usize {
    // The invocation owns this counter, directly or through its immutable
    // ancestor chain, for the complete lifetime of this index entry. No address
    // can be reused while its key is present. Historical input adoption uses
    // materialization IDs in the custody registry, independently of this key.
    Arc::as_ptr(work) as usize
}
