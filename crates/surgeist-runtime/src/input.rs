//! What enters the runtime queues.
//!
//! [`AppInput`] pairs an application payload with the provenance that explains its origin. [`UiInput`], [`TaskInput`]
//! and [`ServiceInput`] validate that provenance against their lane ([`RuntimeLane`]) and are refused with a
//! [`RuntimeInputError`] otherwise. A full queue refuses an input with a [`RuntimeQueueError`] (see
//! [`RuntimeQueueErrorCode`]) that returns it for an exact later retry. [`RuntimeQueuePolicy`] sets the capacity of the
//! UI, task and service queues.

pub use crate::internal::input::AppInput;
pub use crate::internal::runtime::{
    RuntimeInputError, RuntimeLane, RuntimeQueueError, RuntimeQueueErrorCode, RuntimeQueuePolicy,
    ServiceInput, TaskInput, UiInput,
};
