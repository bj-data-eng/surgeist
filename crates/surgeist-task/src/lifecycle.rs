//! Task records, snapshots, statuses and transition errors.
//!
//! A [`TaskRecord`] is the state model of one task: identity, key, scope, policy,
//! cancellation token, observer count and the active attempt, with a
//! [`TaskStatus`]. Starting an attempt leaves the task queued; attempt ids must
//! advance, transitions reject stale attempts, and a terminal record stays
//! terminal. A [`TaskSnapshot`] is an immutable copy of the observable state.
//!
//! A record is separate from the executor's events: the integrator applies the
//! lifecycle changes it wants to a record. A rejected transition is reported as
//! [`TaskLifecycleError`].

pub use crate::internal::lifecycle::{
    TaskLifecycleError, TaskLifecycleErrorKind, TaskRecord, TaskSnapshot, TaskStatus,
    TaskTerminalStatus,
};
