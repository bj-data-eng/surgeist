//! Hosted HTML element tasks.
//!
//! [`DomElementTaskRuntime`] is the weak execution permission for the session's shared task owner, bounded by
//! [`DomElementTaskRuntimeLimits`] and refused with a [`DomElementTaskRuntimeError`]. A queued [`DomElementTask`] holds
//! a Details, Dialog, Object representation, text selection, file selection, or Select notification payload.
//! Running one takes explicit [`DomElementTaskRunLimits`], reports
//! [`DomElementTaskProgress`] and uses a caller context that implements [`DomElementTaskEventContext`].

pub use crate::internal::element_tasks::{
    DomDetailsTask, DomElementTask, DomElementTaskEventContext, DomElementTaskProgress,
    DomElementTaskRunLimits, DomElementTaskRuntime, DomElementTaskRuntimeError,
    DomElementTaskRuntimeLimits,
};
