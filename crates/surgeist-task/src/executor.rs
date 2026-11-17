//! The executor contract, spawn requests, handles and the Tokio executor.
//!
//! [`TaskSpawnRequest`] carries everything an executor needs to run one attempt
//! of a task: identity, scope, policy, cancellation token, input, runnable and
//! event sink. Build it with [`TaskSpawnRequest::builder`], which reports missing
//! fields as an [`ExecutorError`] when built.
//!
//! [`TaskExecutor`] is the contract integrators depend on, and
//! [`TokioTaskExecutor`] is the concrete implementation: it owns a multi-thread
//! Tokio runtime, runs async jobs on it and blocking jobs on Tokio's blocking
//! pool. The runtime and join handles stay inside the implementation. Terminal
//! lifecycle events are emitted by [`TaskExecutor::drain_finished`], so the owning
//! runtime must call it; a job returning does not deliver its terminal event.
//!
//! Priority, deduplication and retry values in a request's policy are carried as
//! data; the Tokio executor does not enforce them.

pub use crate::internal::executor::{
    ExecutorDrainReport, ExecutorError, ExecutorErrorKind, ExecutorTaskHandle, TaskExecutor,
    TaskHandle, TaskSpawnRequest, TaskSpawnRequestBuilder,
};
pub use crate::internal::runtime_tokio::TokioTaskExecutor;
