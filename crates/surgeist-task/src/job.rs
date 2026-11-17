//! The work a task author writes, and what it can report.
//!
//! A [`TaskRunnable`] is either a blocking or an async job, built from a closure
//! with [`TaskRunnable::blocking_fn`] and [`TaskRunnable::async_fn`] or from an
//! implementation of [`BlockingTaskJob`] or [`AsyncTaskJob`]. The job receives its
//! input and a [`TaskContext`], through which it checks for cancellation and emits
//! progress, output and diagnostics.
//!
//! A job ends with `Ok(())` or a [`TaskFailure`]. Returning
//! `TaskFailure::cancelled` reports that work stopped because of cancellation;
//! the executor maps each [`TaskFailureKind`] to a terminal lifecycle event.

pub use crate::internal::executor::{
    AsyncTaskJob, BlockingTaskJob, TaskContext, TaskFailure, TaskFailureKind, TaskRunnable,
};
