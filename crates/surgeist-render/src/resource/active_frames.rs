use super::{FrameIdentity, lease::FrameScratch};
use crate::{
    Error, RenderCpuScope, RenderResource, RenderWork, Result,
    accounting::{AccountedVec, CpuAccount, StorageAccount},
};

type Record = (FrameIdentity, FrameScratch);

/// Device-owned index kept in frame-identity order for nested scratch growth.
#[derive(Default)]
pub(super) struct ActiveFrames {
    records: Option<AccountedVec<Record>>,
    last_identity: u64,
}

impl ActiveFrames {
    pub(super) fn open(&mut self, entry_count: usize, cpu: &CpuAccount) -> Result<FrameIdentity> {
        let next = self.last_identity.checked_add(1).ok_or_else(|| {
            Error::invalid_value(
                "resource frame identity",
                self.last_identity,
                "must have remaining identity space",
            )
        })?;
        let frame = FrameIdentity(next);
        let scratch = FrameScratch::new(cpu, entry_count)?;
        let records = match &mut self.records {
            Some(records) => {
                records.grow(1)?;
                records
            }
            vacant @ None => vacant.insert(AccountedVec::new_in(
                &StorageAccount::Renderer(cpu.at(RenderCpuScope::DeviceBacking)),
                1,
                RenderResource::RetainedCpuBytes,
            )?),
        };
        records.append_slot()?.write((frame, scratch));
        self.last_identity = next;
        Ok(frame)
    }

    fn position(records: &[Record], frame: &FrameIdentity) -> Option<usize> {
        records
            .binary_search_by_key(frame, |(identity, _)| *identity)
            .ok()
    }

    pub(super) fn contains_key(&self, frame: &FrameIdentity) -> bool {
        self.records
            .as_ref()
            .is_some_and(|records| Self::position(records, frame).is_some())
    }

    pub(super) fn get_mut(&mut self, frame: &FrameIdentity) -> Option<&mut FrameScratch> {
        let records = self.records.as_mut()?;
        let index = Self::position(records, frame)?;
        records.get_mut(index).map(|(_, scratch)| scratch)
    }

    pub(super) fn values_mut(&mut self) -> impl Iterator<Item = &mut FrameScratch> {
        self.records
            .iter_mut()
            .flat_map(|records| records.iter_mut().map(|(_, scratch)| scratch))
    }

    pub(super) fn remove(&mut self, frame: &FrameIdentity) -> Option<FrameScratch> {
        let records = self.records.as_mut()?;
        let index = Self::position(records, frame)?;
        let (_, scratch) = records.remove(index)?;
        let moves = records.len().saturating_sub(index);
        if moves != 0 {
            records.renderer_maintenance(RenderWork::ActiveFrameRecordMoves, moves as u64);
        }
        Some(scratch)
    }

    #[cfg(test)]
    pub(super) fn get(&self, frame: &FrameIdentity) -> Option<&FrameScratch> {
        let records = self.records.as_ref()?;
        let index = Self::position(records, frame)?;
        records.get(index).map(|(_, scratch)| scratch)
    }

    #[cfg(test)]
    pub(super) fn iter(&self) -> impl Iterator<Item = (&FrameIdentity, &FrameScratch)> {
        self.records
            .iter()
            .flat_map(|records| records.iter().map(|(frame, scratch)| (frame, scratch)))
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.records.as_ref().map_or(0, |records| records.len())
    }

    #[cfg(test)]
    pub(super) const fn last_identity(&self) -> u64 {
        self.last_identity
    }
}
