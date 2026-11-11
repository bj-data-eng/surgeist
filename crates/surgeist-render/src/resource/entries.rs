use super::{ResourceIdentity, manager::ResourceEntry};
use crate::{
    RenderAccountingError, RenderCpuScope, RenderRefusalReason, RenderResource, RenderWork, Result,
    accounting::{AccountedVec, AppendSlot, CpuAccount, StorageAccount},
};

type Record = (ResourceIdentity, Option<ResourceEntry>);

#[cfg(test)]
pub(super) const RECORD_BYTES: u64 = std::mem::size_of::<Record>() as u64;

/// Identity-ordered records with vacant positions retained between compactions.
#[derive(Default)]
pub(super) struct ResourceEntries {
    records: Option<AccountedVec<Record>>,
    vacant: usize,
}

impl ResourceEntries {
    pub(super) fn len(&self) -> usize {
        self.records
            .as_ref()
            .map_or(0, |records| records.len() - self.vacant)
    }

    pub(super) fn reserve(&mut self, additional: usize, cpu: &CpuAccount) -> Result<()> {
        if additional == 0 {
            return Ok(());
        }
        let resource = RenderResource::RetainedCpuBytes;
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::DeviceBacking));
        let Some(records) = &mut self.records else {
            self.records = Some(AccountedVec::new_in(&storage, additional, resource)?);
            return Ok(());
        };
        let needed = records
            .len()
            .checked_add(additional)
            .ok_or_else(|| storage.capacity_error(resource))?;
        if needed <= records.capacity() {
            return Ok(());
        }
        let live_needed = needed - self.vacant;
        let target = records
            .capacity()
            .checked_mul(2)
            .filter(|&capacity| capacity > live_needed);
        let capacity = if live_needed <= records.capacity() {
            records.capacity()
        } else if let Some(target) = target {
            match storage
                .bytes::<Record>(target, resource)
                .and_then(|bytes| storage.preview(resource, bytes))
            {
                Ok(()) => target,
                Err(error)
                    if matches!(
                        error.accounting_diagnostic(),
                        Some(
                            RenderAccountingError::CpuRefused {
                                reason: RenderRefusalReason::Limit { .. },
                                ..
                            } | RenderAccountingError::CpuCapacityOverflow { .. }
                        )
                    ) =>
                {
                    live_needed
                }
                Err(error) => return Err(error),
            }
        } else {
            live_needed
        };
        if self.vacant != 0 {
            cpu.at(RenderCpuScope::Cleanup)
                .maintenance_phase(|| Self::compact(records, &mut self.vacant));
        }
        records.grow(capacity - records.len())
    }

    pub(super) fn append_slot(
        &mut self,
        identity: ResourceIdentity,
    ) -> Result<ResourceEntrySlot<'_>> {
        let records = self
            .records
            .as_mut()
            .ok_or(RenderAccountingError::Invariant)?;
        if records.last().is_some_and(|(last, _)| identity <= *last) {
            return Err(RenderAccountingError::Invariant.into());
        }
        Ok(ResourceEntrySlot {
            identity,
            slot: records.append_slot()?,
        })
    }

    fn position(records: &[Record], identity: &ResourceIdentity) -> Option<usize> {
        records
            .binary_search_by_key(identity, |(identity, _)| *identity)
            .ok()
    }

    pub(super) fn get(&self, identity: &ResourceIdentity) -> Option<&ResourceEntry> {
        let records = self.records.as_ref()?;
        let index = Self::position(records, identity)?;
        records.get(index)?.1.as_ref()
    }

    pub(super) fn get_mut(&mut self, identity: &ResourceIdentity) -> Option<&mut ResourceEntry> {
        let records = self.records.as_mut()?;
        let index = Self::position(records, identity)?;
        records.get_mut(index)?.1.as_mut()
    }

    pub(super) fn remove(&mut self, identity: &ResourceIdentity) -> Option<ResourceEntry> {
        let records = self.records.as_mut()?;
        let index = Self::position(records, identity)?;
        let entry = records.get_mut(index)?.1.take()?;
        self.vacant += 1;
        if self.vacant > records.len() / 2 {
            Self::compact(records, &mut self.vacant);
        }
        Some(entry)
    }

    fn compact(records: &mut AccountedVec<Record>, vacant: &mut usize) {
        if *vacant == 0 {
            return;
        }
        let mut hole = false;
        let mut moves = 0_u64;
        records.retain(|(_, entry)| {
            if entry.is_none() {
                hole = true;
                return false;
            }
            moves += u64::from(hole);
            true
        });
        *vacant = 0;
        if moves != 0 {
            records.renderer_maintenance(RenderWork::ResourceCacheRecordMoves, moves);
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&ResourceIdentity, &ResourceEntry)> {
        self.records
            .iter()
            .flat_map(|records| records.iter())
            .filter_map(|(identity, entry)| entry.as_ref().map(|entry| (identity, entry)))
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&ResourceIdentity, &mut ResourceEntry)> {
        self.records
            .iter_mut()
            .flat_map(|records| records.iter_mut())
            .filter_map(|(identity, entry)| entry.as_mut().map(|entry| (&*identity, entry)))
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &ResourceEntry> {
        self.iter().map(|(_, entry)| entry)
    }

    #[cfg(test)]
    pub(super) fn keys(&self) -> impl Iterator<Item = &ResourceIdentity> {
        self.iter().map(|(identity, _)| identity)
    }
}

pub(super) struct ResourceEntrySlot<'a> {
    identity: ResourceIdentity,
    slot: AppendSlot<'a, Record>,
}

impl ResourceEntrySlot<'_> {
    pub(super) fn write(self, entry: ResourceEntry) {
        self.slot.write((self.identity, Some(entry)));
    }
}
