//! Element task sources and the native checkpoint that releases queued tasks.
//!
//! `DomElementTaskSource` is the one host task owner bound to a DOM, `DomElementTaskId`
//! names an exact task occurrence, `DomElementTaskSourceKind` names a task source (HTML
//! keeps FIFO order within each source), `DomElementTaskRetention` is the continuous
//! admission of the owner's storage, and `DomElementTaskAssociation` records the relevant
//! Global and Document at queueing time. `Dom::native_checkpoint_step` takes a
//! `DomNativeCheckpointLimits` and returns a `DomNativeCheckpoint`, one native checkpoint
//! turn (idle, a Select task, a child-list notification or Slot notifications);
//! `DomNativeTaskId` is an exact operation identity.
//!
//! A checkpoint turn executes the oldest native FIFO work: child-list observer deliveries
//! precede the captured Slot signals. No callback and no browser event loop is installed.
//! `docs/slots-and-checkpoints.md` owns the contract.

pub use crate::internal::details::DomElementTaskAssociation;
pub use crate::internal::element_tasks::{
    DomElementTaskId, DomElementTaskRetention, DomElementTaskSource, DomElementTaskSourceKind,
};
pub use crate::internal::select::{
    DomNativeCheckpoint, DomNativeCheckpointLimits, DomNativeTaskId,
};
pub use crate::internal::store::source::operation::element_queue::{
    DomFileSelectionTask, DomFileSelectionTaskInfo, DomObjectRepresentationTask,
    DomObjectRepresentationTaskInfo, DomQueuedSourceElementTask, DomQueuedSourceElementTasks,
    DomSourceElementTaskInfo, DomSourceElementTaskQueueCommit, DomSourceElementTaskQueueRequest,
    DomTextSelectionTaskInfo,
};
