//! Task scheduling and work-plane contracts for Surgeist.
//!
//! This crate owns the task subsystem boundary for Surgeist. Keep app authors
//! oriented around Surgeist tasks, task context, cancellation, progress,
//! resource classes, and scheduler policy. Concrete executors such as Tokio,
//! thread pools, or process workers belong behind these contracts.
//!
//! # Where things live
//!
//! The crate root holds only the public modules and [`CRATE_NAME`]. Every public
//! item has exactly one path, in the module that owns its domain, and a root path
//! to a module item does not resolve.
//!
//! | Module | Domain |
//! | --- | --- |
//! | [`cancellation`] | Cancellation tokens, views and reasons. |
//! | [`coordination`] | Task registration and observer tracking. |
//! | [`event`] | Task events and the sink that receives them. |
//! | [`executor`] | The executor contract, spawn requests, handles and the Tokio executor. |
//! | [`identity`] | Task, attempt, correlation and observer identity, names, keys and provenance. |
//! | [`job`] | Blocking and async jobs, their context and their failures. |
//! | [`lifecycle`] | Task records, snapshots, statuses and transition errors. |
//! | [`policy`] | Admission, priority, retry, blocking and unobserved-task policy. |
//! | [`progress`] | Progress values reported by running tasks. |
//! | [`queue`] | The bounded event queue, drain budgets and reports. |
//! | [`scope`] | Task scopes, segments and their validation errors. |
//!
//! # Entry points
//!
//! A task author builds a [`executor::TaskSpawnRequest`] from identity
//! ([`identity::TaskId`], [`identity::TaskAttemptId`], [`identity::TaskKey`]), a
//! [`scope::TaskScope`], a [`cancellation::CancellationToken`], an
//! [`event::TaskEventSink`], an input and a [`job::TaskRunnable`], and hands it to
//! an [`executor::TaskExecutor`]. [`executor::TokioTaskExecutor`] is the concrete
//! executor. The job reports through its [`job::TaskContext`], and the sink
//! receives [`event::TaskEvent`] values.

#![forbid(unsafe_code)]

pub mod cancellation;
pub mod coordination;
pub mod event;
pub mod executor;
pub mod identity;
pub mod job;
pub mod lifecycle;
pub mod policy;
pub mod progress;
pub mod queue;
pub mod scope;

mod internal;

/// Crate identity string used by smoke tests and API artifacts.
pub const CRATE_NAME: &str = "surgeist-task";

#[cfg(test)]
mod tests {
    use super::CRATE_NAME;

    #[test]
    fn exposes_crate_identity() {
        assert_eq!(CRATE_NAME, "surgeist-task");
    }
}
