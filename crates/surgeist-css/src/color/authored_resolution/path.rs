//! One caller-owned contiguous path allocation with explicit reservation effects.

use crate::color::controlled::CssColorOperationOwner;
use crate::color::controlled::protocol::ColorProtocolError;
use crate::component::CssValueOrigin;
use std::mem::{size_of, take};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorPathStep {
    Source,
    Channel(usize),
    Alpha,
    First,
    Second,
    MixColor(usize),
    MixWeight(usize),
    ExpressionOperand(usize),
}

/// A checked path whose allocation transfers with an authored failure.
#[derive(Debug, Eq, PartialEq)]
pub struct CssColorComponentPath {
    steps: Vec<CssColorPathStep>,
}
impl CssColorComponentPath {
    pub fn steps(&self) -> &[CssColorPathStep] {
        &self.steps
    }
    pub fn capacity(&self) -> usize {
        self.steps.capacity()
    }
    pub fn capacity_bytes(&self) -> usize {
        capacity_bytes(self.capacity())
    }
}
fn capacity_bytes(capacity: usize) -> usize {
    capacity
        .checked_mul(size_of::<CssColorPathStep>())
        .expect("an existing nonzero-sized Vec allocation has a representable layout")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssColorPathReservationStatus {
    Reserved,
    AllocationFailed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CssColorPathCapacityObservation {
    epoch: usize,
    revision: usize,
    old_len: usize,
    old_capacity: usize,
    required_len: usize,
    actual_len: usize,
    actual_capacity: usize,
    status: CssColorPathReservationStatus,
}
impl CssColorPathCapacityObservation {
    #[cfg(test)]
    pub(crate) const fn old_len(self) -> usize {
        self.old_len
    }
    #[cfg(test)]
    pub(crate) const fn old_capacity(self) -> usize {
        self.old_capacity
    }
    #[cfg(test)]
    pub(crate) const fn required_len(self) -> usize {
        self.required_len
    }
    #[cfg(test)]
    pub(crate) const fn actual_len(self) -> usize {
        self.actual_len
    }
    pub const fn actual_capacity(self) -> usize {
        self.actual_capacity
    }
    pub const fn status(self) -> CssColorPathReservationStatus {
        self.status
    }
}

/// The allocation has no retained anchor/root/fact borrow, including after cleanup.
pub(crate) struct CssColorPathBuffer {
    steps: Vec<CssColorPathStep>,
    epoch: usize,
    observation: Option<CssColorPathCapacityObservation>,
}
impl Default for CssColorPathBuffer {
    fn default() -> Self {
        Self::new()
    }
}
impl CssColorPathBuffer {
    pub const fn new() -> Self {
        Self {
            steps: Vec::new(),
            epoch: 0,
            observation: None,
        }
    }
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.steps.len()
    }
    pub fn capacity(&self) -> usize {
        self.steps.capacity()
    }
    pub fn capacity_bytes(&self) -> usize {
        capacity_bytes(self.capacity())
    }
    /// Fixed observation of this buffer's own outstanding recovered reservation.
    /// The caller admits this header read and accounts its actual capacity delta
    /// before acknowledging it. This is an observation, never an identity token.
    pub const fn capacity_observation(&self) -> Option<CssColorPathCapacityObservation> {
        self.observation
    }
    /// Acknowledges only this buffer's stored recovered reservation. It performs
    /// no allocation, path extension, producer work, or first-stop replacement.
    /// An absent or already acknowledged observation refuses without mutation.
    pub fn capacity_observed(&mut self) -> Result<(), ColorProtocolError> {
        if self.observation.is_none() {
            return Err(ColorProtocolError::WrongAction);
        }
        self.observation = None;
        Ok(())
    }
    /// Caller admission precedes this fixed header mutation; Copy steps have no
    /// payload destructors. Capacity and epoch remain owned by the caller.
    pub fn cleanup(&mut self) -> Result<(), ColorProtocolError> {
        if self.observation.is_some() {
            return Err(ColorProtocolError::WrongAction);
        }
        self.steps.clear();
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CssColorPathHandle<'css> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    epoch: usize,
    revision: usize,
    length: usize,
}
impl std::fmt::Debug for CssColorPathHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssColorPathHandle")
            .field("phase", &self.phase)
            .field("epoch", &self.epoch)
            .field("revision", &self.revision)
            .field("length", &self.length)
            .finish_non_exhaustive()
    }
}
impl PartialEq for CssColorPathHandle<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.owner, other.owner)
            && self.phase == other.phase
            && self.epoch == other.epoch
            && self.revision == other.revision
            && self.length == other.length
    }
}
impl Eq for CssColorPathHandle<'_> {}
impl CssColorPathHandle<'_> {
    pub const fn length(self) -> usize {
        self.length
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CssColorPathInspectionLayout {
    pub handle_bytes: usize,
    pub path_header_bytes: usize,
    pub step_read_bytes: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CssColorPathReservationLayout {
    pub header_bytes: usize,
    pub observation_bytes: usize,
    pub step_bytes: usize,
    pub old_capacity_bytes: usize,
    pub requested_capacity_bytes: usize,
    pub required_len: usize,
}

pub(super) struct PreparedSite {
    pub(super) path: CssColorComponentPath,
    pub(super) origin: Option<CssValueOrigin>,
    pub(super) site: usize,
}
pub(super) struct PathState<'path, 'css> {
    buffer: &'path mut CssColorPathBuffer,
    owner: &'css CssColorOperationOwner,
    phase: usize,
    revision: usize,
    prepared: Option<PreparedSite>,
}
impl<'path, 'css> PathState<'path, 'css> {
    pub(super) fn initialize(
        owner: &'css mut CssColorOperationOwner,
        buffer: &'path mut CssColorPathBuffer,
    ) -> Result<Self, ColorProtocolError> {
        if !buffer.steps.is_empty() || buffer.observation.is_some() {
            return Err(ColorProtocolError::WrongAction);
        }
        let phase = owner.current_phase();
        let next_phase = owner.prepare_advance(phase)?;
        let next_epoch = buffer
            .epoch
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        owner.commit_phase(phase, next_phase);
        buffer.epoch = next_epoch;
        Ok(Self {
            buffer,
            owner,
            phase: next_phase,
            revision: 0,
            prepared: None,
        })
    }
    #[cfg(test)]
    pub(super) fn retained_layout(&self) -> (usize, usize, usize) {
        match &self.prepared {
            Some(site) => (
                site.path.steps.len(),
                site.path.capacity_bytes(),
                if site.origin.is_some() {
                    size_of::<CssValueOrigin>()
                } else {
                    0
                },
            ),
            None => (self.buffer.steps.len(), self.buffer.capacity_bytes(), 0),
        }
    }
    pub(super) fn owner(&self) -> &'css CssColorOperationOwner {
        self.owner
    }
    pub(super) const fn phase(&self) -> usize {
        self.phase
    }
    fn steps(&self) -> &[CssColorPathStep] {
        self.prepared
            .as_ref()
            .map_or(&self.buffer.steps, |site| &site.path.steps)
    }
    pub(super) fn handle(&self) -> CssColorPathHandle<'css> {
        CssColorPathHandle {
            owner: self.owner,
            phase: self.phase,
            epoch: self.buffer.epoch,
            revision: self.revision,
            length: self.steps().len(),
        }
    }
    #[cfg(test)]
    fn qualify(&self, handle: CssColorPathHandle<'css>) -> Result<(), ColorProtocolError> {
        if handle != self.handle() || self.owner.current_phase() != self.phase {
            return Err(ColorProtocolError::ForeignSlot);
        }
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn inspection_layout(
        &self,
        handle: CssColorPathHandle<'css>,
    ) -> Result<CssColorPathInspectionLayout, ColorProtocolError> {
        self.qualify(handle)?;
        Ok(CssColorPathInspectionLayout {
            handle_bytes: size_of::<CssColorPathHandle<'_>>(),
            path_header_bytes: if self.prepared.is_some() {
                size_of::<PreparedSite>()
            } else {
                size_of::<CssColorPathBuffer>()
            },
            step_read_bytes: handle
                .length
                .checked_mul(size_of::<CssColorPathStep>())
                .ok_or(ColorProtocolError::Overflow)?,
        })
    }
    #[cfg(test)]
    pub(super) fn view(
        &self,
        handle: CssColorPathHandle<'css>,
    ) -> Result<&[CssColorPathStep], ColorProtocolError> {
        self.qualify(handle)?;
        Ok(self.steps())
    }
    pub(super) fn reservation_layout(
        &self,
    ) -> Result<CssColorPathReservationLayout, ColorProtocolError> {
        if self.prepared.is_some() || self.buffer.observation.is_some() {
            return Err(ColorProtocolError::WrongAction);
        }
        let required_len = self
            .buffer
            .steps
            .len()
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let requested_capacity_bytes = required_len
            .checked_mul(size_of::<CssColorPathStep>())
            .ok_or(ColorProtocolError::Overflow)?;
        Ok(CssColorPathReservationLayout {
            header_bytes: size_of::<CssColorPathBuffer>(),
            observation_bytes: size_of::<CssColorPathCapacityObservation>(),
            step_bytes: size_of::<CssColorPathStep>(),
            old_capacity_bytes: self.buffer.capacity_bytes(),
            requested_capacity_bytes,
            required_len,
        })
    }
    pub(super) fn reserve(
        &mut self,
    ) -> Result<CssColorPathCapacityObservation, ColorProtocolError> {
        let layout = self.reservation_layout()?;
        let old_len = self.buffer.steps.len();
        let old_capacity = self.buffer.steps.capacity();
        let status = if old_capacity >= layout.required_len
            || self
                .buffer
                .steps
                .try_reserve_exact(layout.required_len - old_len)
                .is_ok()
        {
            CssColorPathReservationStatus::Reserved
        } else {
            CssColorPathReservationStatus::AllocationFailed
        };
        let observation = CssColorPathCapacityObservation {
            epoch: self.buffer.epoch,
            revision: self.revision,
            old_len,
            old_capacity,
            required_len: layout.required_len,
            actual_len: self.buffer.steps.len(),
            actual_capacity: self.buffer.steps.capacity(),
            status,
        };
        self.buffer.observation = Some(observation);
        Ok(observation)
    }
    pub(super) fn capacity_observed(
        &mut self,
    ) -> Result<CssColorPathReservationStatus, ColorProtocolError> {
        let observation = self
            .buffer
            .observation
            .ok_or(ColorProtocolError::WrongAction)?;
        if observation.revision != self.revision {
            return Err(ColorProtocolError::WrongAction);
        }
        self.buffer.capacity_observed()?;
        Ok(observation.status)
    }
    pub(super) fn append(&mut self, step: CssColorPathStep) -> Result<(), ColorProtocolError> {
        if self.prepared.is_some()
            || self.buffer.observation.is_some()
            || self.buffer.steps.len() == self.buffer.steps.capacity()
        {
            return Err(ColorProtocolError::WrongAction);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.buffer.steps.push(step);
        self.revision = revision;
        Ok(())
    }
    pub(super) fn truncate(&mut self, length: usize) -> Result<(), ColorProtocolError> {
        if self.prepared.is_some()
            || self.buffer.observation.is_some()
            || length > self.buffer.steps.len()
        {
            return Err(ColorProtocolError::WrongAction);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.buffer.steps.truncate(length);
        self.revision = revision;
        Ok(())
    }
    pub(super) fn prepare(
        &mut self,
        site: usize,
        origin: Option<&CssValueOrigin>,
    ) -> Result<(), ColorProtocolError> {
        if self.prepared.is_some() || self.buffer.observation.is_some() {
            return Err(ColorProtocolError::WrongAction);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let origin = origin.cloned();
        self.prepared = Some(PreparedSite {
            path: CssColorComponentPath {
                steps: take(&mut self.buffer.steps),
            },
            origin,
            site,
        });
        self.revision = revision;
        Ok(())
    }
    pub(super) fn restore(&mut self) -> Result<(), ColorProtocolError> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let site = self
            .prepared
            .take()
            .ok_or(ColorProtocolError::WrongAction)?;
        self.buffer.steps = site.path.steps;
        self.revision = revision;
        Ok(())
    }
    pub(super) fn prepared_site(&self) -> Option<usize> {
        self.prepared.as_ref().map(|site| site.site)
    }
    pub(super) fn take_site(&mut self) -> PreparedSite {
        self.prepared
            .take()
            .expect("semantic action preflight proved a prepared site")
    }
    pub(super) fn reclaim_completed(
        &mut self,
        path: CssColorComponentPath,
        origin: Option<CssValueOrigin>,
        site: usize,
    ) {
        assert!(
            self.prepared.is_none()
                && self.buffer.steps.is_empty()
                && self.buffer.steps.capacity() == 0
        );
        self.prepared = Some(PreparedSite { path, origin, site });
    }
    pub(super) fn recover(mut self) -> Option<CssColorPathCapacityObservation> {
        if let Some(site) = self.prepared.take() {
            self.buffer.steps = site.path.steps;
        }
        self.buffer.observation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreign_fact_preflight_does_not_change_phase_epoch_or_capacity() {
        use crate::color::authored_resolution::controlled::AuthoredError;
        use crate::color::{
            CssColorFactDefect, CssColorFactIdentity, CssColorFacts, CssColorPropertyFacts,
            CssColorSchemeFact, CssDeviceColorFact,
        };
        let mut owner = CssColorOperationOwner::new();
        let mut buffer = CssColorPathBuffer::new();
        let root = crate::color::CssColor::transparent();
        let facts = CssColorFacts::new(
            CssColorFactIdentity::new(1, 2),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        );
        let phase = owner.current_phase();
        let epoch = buffer.epoch;
        let result = crate::color::authored_resolution::controlled::CssAuthoredColorOperation::resolve_authored(
            &mut owner,
            &root,
            &facts,
            CssColorFactIdentity::new(1, 3),
            &mut buffer,
        );
        assert!(matches!(
            result,
            Err(AuthoredError::Color(ref error))
                if error.fact_defect() == Some(CssColorFactDefect::Foreign)
        ));
        drop(result);
        assert_eq!(owner.current_phase(), phase);
        assert_eq!(buffer.epoch, epoch);
        assert_eq!(buffer.capacity(), 0);
        assert!(buffer.observation.is_none());
    }
}
