//! Element task sources and the native checkpoint that releases queued tasks.
//!
//! `DomElementTaskSource` is the one host task owner bound to a DOM, `DomElementTaskId`
//! names an exact task occurrence, `DomElementTaskSourceKind` names a task source (HTML
//! keeps FIFO order within each source), `DomElementTaskRetention` is the continuous
//! admission of the owner's storage, and `DomElementTaskAssociation` records the relevant
//! Global and Document at queueing time. `Dom::native_checkpoint_step` takes a
//! `DomNativeCheckpointLimits` and returns a `DomNativeCheckpoint`, one native checkpoint
//! turn; `DomNativeTaskId` is an exact operation identity.
//!
//! No callback and no browser event loop is installed: the runtime calls the checkpoint
//! explicitly. `docs/slots-and-checkpoints.md` owns the contract.

pub use crate::internal::details::DomElementTaskAssociation;
pub use crate::internal::element_tasks::{
    DomElementTaskId, DomElementTaskRetention, DomElementTaskSource, DomElementTaskSourceKind,
};
pub use crate::internal::select::{
    DomNativeCheckpoint, DomNativeCheckpointLimits, DomNativeTaskId,
};
