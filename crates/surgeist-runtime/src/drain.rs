//! A drain turn of the runtime.
//!
//! [`RuntimeBudget`] limits the inputs one turn drains, in total and for each lane. A successful turn returns a
//! [`RuntimeDrainReport`]; a failed checked transaction returns a [`RuntimeDrainError`] (see
//! [`RuntimeDrainErrorCode`]) that carries the work already committed. [`AppLoop`] is a host-independent owner of a
//! [`Runtime`](crate::Runtime) that delegates one drain step at a time.

pub use crate::internal::loop_::AppLoop;
pub use crate::internal::runtime::{
    RuntimeBudget, RuntimeDrainError, RuntimeDrainErrorCode, RuntimeDrainReport,
};
