//! Events a task emits, and the sink that receives them.
//!
//! A [`TaskEvent`] pairs a [`TaskProvenance`](crate::identity::TaskProvenance)
//! with a [`TaskEventKind`]: a lifecycle change, a progress report, an output
//! value or a diagnostic. A running job emits [`TaskJobEvent`] values through its
//! [`TaskContext`](crate::job::TaskContext); the executor adds the task and
//! attempt identity and delivers the resulting events to a [`TaskEventSink`].
//!
//! Terminal lifecycle events are emitted when the executor is drained, not when
//! the job returns. A sink failure is reported as [`TaskEventSinkError`]. Use
//! [`queue`](crate::queue) for a bounded, coalescing buffer of events.

pub use crate::internal::event::{
    TaskDiagnosticEvent, TaskDiagnosticLevel, TaskEvent, TaskEventKind, TaskJobEvent,
    TaskLifecycleEvent, TaskOutputEvent,
};
pub use crate::internal::executor::{TaskEventSink, TaskEventSinkError, TaskEventSinkErrorKind};
