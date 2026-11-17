//! Task registration and observer tracking.
//!
//! A [`TaskRegistration`] names a kind of task and derives each task's scope and
//! key from its input. [`TaskCoordination`] counts the distinct observers of each
//! task key. Detaching the last observer returns an [`ObservationChange`] that
//! reflects the task's unobserved policy; it is a decision for the caller to
//! apply, and coordination does not itself cancel, pause or lower the task.
//!
//! Registration failures are reported as [`TaskRegistrationError`], which wraps
//! the identity or scope validation error that caused it.

pub use crate::internal::coordination::{
    ObservationChange, ObserverCountSnapshot, TaskCoordination, TaskRegistration,
    TaskRegistrationError, TaskRegistrationErrorKind,
};
