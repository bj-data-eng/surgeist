//! Admission, priority, retry, blocking and unobserved-task policy.
//!
//! A [`TaskPolicy`] combines deduplication, an [`UnobservedPolicy`], a
//! [`TaskPriority`], a [`RetryPolicy`] and a [`BlockingPolicy`]. The default,
//! [`TaskPolicy::continue_when_unobserved`], uses no deduplication, normal
//! priority, one attempt and an abortable job.
//!
//! These values describe a broader scheduling boundary than the Tokio executor
//! implements: they are carried as data, and callers act on them.

pub use crate::internal::policy::{
    BlockingPolicy, RetryPolicy, TaskPolicy, TaskPriority, UnobservedPolicy,
};
