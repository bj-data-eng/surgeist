//! Task, attempt, correlation and observer identity, names, keys and provenance.
//!
//! [`TaskId`], [`TaskAttemptId`] and [`CorrelationId`] are numeric identities.
//! [`TaskName`], [`TaskKey`], [`ObserverId`], [`ResourceClassId`] and
//! [`CoalescingKey`] are validated strings: construction trims surrounding
//! whitespace and rejects an empty value or one containing an ASCII control
//! character, reporting [`TaskIdError`].
//!
//! A task has a semantic identity (its key) and an active attempt; attempt ids
//! only advance, which is how stale results are recognized. [`TaskProvenance`]
//! names the task and attempt an event belongs to, with optional correlation,
//! parent correlation and sequence.

pub use crate::internal::id::{
    CoalescingKey, CorrelationId, ObserverId, ResourceClassId, TaskAttemptId, TaskId, TaskIdError,
    TaskIdErrorKind, TaskKey, TaskName,
};
pub use crate::internal::provenance::TaskProvenance;
