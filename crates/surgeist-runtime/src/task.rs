//! Abstract task intents.
//!
//! A [`TaskIntentId`] and a [`TaskIntentAttemptId`] identify a task intent and one attempt of it; a [`TaskIntentHandle`]
//! pairs them. [`TaskIntentKey`] correlates related intents, [`TaskIntentName`] names the kind of work the runtime asks
//! the root to start, and [`TaskPriorityHint`] is a scheduling preference that root adapters may interpret. This crate does
//! not execute tasks.

pub use crate::internal::task::{
    TaskIntentAttemptId, TaskIntentHandle, TaskIntentId, TaskIntentKey, TaskIntentName,
    TaskPriorityHint,
};
