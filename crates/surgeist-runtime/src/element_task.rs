//! Details and Dialog element tasks.
//!
//! [`DomElementTaskRuntime`] is the weak execution permission for the session's shared task owner, bounded by
//! [`DomElementTaskRuntimeLimits`] and refused with a [`DomElementTaskRuntimeError`]. A queued [`DomElementTask`] holds
//! a [`DomDetailsTask`] or a Dialog task payload; running one takes explicit [`DomElementTaskRunLimits`], reports
//! [`DomElementTaskProgress`] and uses a caller context that implements [`DomElementTaskEventContext`].

pub use crate::internal::element_tasks::{
    DomDetailsTask, DomElementTask, DomElementTaskEventContext, DomElementTaskProgress,
    DomElementTaskRunLimits, DomElementTaskRuntime, DomElementTaskRuntimeError,
    DomElementTaskRuntimeLimits,
};
