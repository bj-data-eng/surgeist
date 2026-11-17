//! Progress values reported by running tasks.
//!
//! A [`TaskProgressEvent`] pairs a [`CoalescingKey`](crate::identity::CoalescingKey)
//! with a [`TaskProgressValue`]: a count of units with an optional total, a
//! [`PercentComplete`], or a message. [`PercentComplete`] is stored in basis
//! points, where 10000 is 100 percent, and rejects larger values with
//! [`PercentCompleteError`].
//!
//! A [`TaskEventQueue`](crate::queue::TaskEventQueue) keeps only the latest
//! progress event for each task, attempt and coalescing key.

pub use crate::internal::event::{
    PercentComplete, PercentCompleteError, PercentCompleteErrorKind, TaskProgressEvent,
    TaskProgressValue,
};
