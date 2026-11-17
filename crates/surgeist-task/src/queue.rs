//! A bounded event queue with progress coalescing, drain budgets and reports.
//!
//! [`TaskEventQueue`] buffers [`TaskEvent`](crate::event::TaskEvent) values for
//! the integration that consumes them; it is separate from the event sink and
//! needs its own synchronization. Capacity counts retained non-progress events
//! plus distinct progress slots, where a slot is a task id, attempt id and
//! coalescing key. Replacing an existing progress slot succeeds even at capacity;
//! adding an event at capacity returns [`QueueError`] and increments the dropped
//! count.
//!
//! A drain returns non-progress events first, in order, then progress slots in
//! insertion order, up to a [`DrainBudget`]. The dropped and coalesced counters in
//! a [`QueueReport`] are cumulative.

pub use crate::internal::queue::{
    DrainBudget, DrainReport, QueueError, QueueErrorCode, QueuePolicy, QueueReport, TaskEventQueue,
};
