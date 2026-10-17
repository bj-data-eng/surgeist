//! Affine ownership of the actual shared element-task bank and payloads.
use crate::{DomOwnerId, NodeId};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub(crate) struct Binding {
    pub(crate) owner: DomOwnerId,
    pub(crate) bytes: AtomicUsize,
}
/// HTML element tasks preserve FIFO independently within each task source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomElementTaskSourceKind {
    DomManipulation,
    UserInteraction,
}
/// The one actual host task owner bound to a canonical DOM. This capability is
/// trusted to publish only its real prepared tasks, never a synthetic queue ack.
pub struct DomElementTaskSource {
    pub(crate) binding: Arc<Binding>,
}
impl std::fmt::Debug for DomElementTaskSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomElementTaskSource")
            .finish_non_exhaustive()
    }
}
impl DomElementTaskSource {
    /// Producer-owned fixed binding payload, excluding allocator metadata.
    pub const fn binding_payload_bytes() -> usize {
        std::mem::size_of::<Binding>()
    }
    /// Qualify this owner's checked task-bank serial and target. This scalar
    /// identity grants neither publication nor task execution permission.
    pub fn task_id(
        &self,
        serial: u64,
        target: NodeId,
    ) -> Result<DomElementTaskId, crate::DomError> {
        if serial == 0 || target.owner() != self.binding.owner {
            return Err(crate::DomError::InvalidSourceOperation);
        }
        Ok(DomElementTaskId {
            owner: self.binding.owner,
            serial,
            target,
        })
    }
}
/// Exact source issuer, nonwrapping occurrence and target generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomElementTaskId {
    pub(crate) owner: DomOwnerId,
    pub(crate) serial: u64,
    pub(crate) target: NodeId,
}
impl DomElementTaskId {
    /// Actual queued receiver.
    pub const fn target(self) -> NodeId {
        self.target
    }
}
/// Continuous admission of actual task-owner storage. Drop the represented
/// allocation before this guard. It remains observable after any Source closes.
pub struct DomElementTaskRetention {
    pub(crate) binding: Arc<Binding>,
    pub(crate) bytes: usize,
}
impl DomElementTaskRetention {
    /// Transfer a checked subextent to its actual allocation owner, without
    /// changing aggregate admission or creating credit.
    pub fn split(&mut self, bytes: usize) -> Result<Self, crate::DomError> {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or(crate::DomError::RepresentationExhausted)?;
        Ok(Self {
            binding: self.binding.clone(),
            bytes,
        })
    }
    /// Merge admitted extents owned by this same original task source.
    pub fn absorb(&mut self, mut other: Self) -> Result<(), crate::DomError> {
        if !Arc::ptr_eq(&self.binding, &other.binding) {
            return Err(crate::DomError::InvalidSourceOperation);
        }
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .ok_or(crate::DomError::RepresentationExhausted)?;
        other.bytes = 0;
        Ok(())
    }
}
impl Drop for DomElementTaskRetention {
    fn drop(&mut self) {
        self.binding.bytes.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}
pub(crate) fn retained(binding: &Option<Weak<Binding>>) -> Result<usize, crate::DomError> {
    binding
        .as_ref()
        .and_then(Weak::upgrade)
        .map_or(Ok(0), |binding| {
            DomElementTaskSource::binding_payload_bytes()
                .checked_add(binding.bytes.load(Ordering::Relaxed))
                .ok_or(crate::DomError::RepresentationExhausted)
        })
}
