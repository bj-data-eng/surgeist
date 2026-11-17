//! Cooperative cancellation tokens, views and reasons.
//!
//! A [`CancellationToken`] is the owning handle: whoever holds it can request
//! cancellation with a [`CancelReason`]. A [`CancellationView`] is a read-only
//! observer of the same state that a running job can poll. The first
//! cancellation request wins; later requests do not replace the recorded reason.
//!
//! Cancellation is a request, not an abort. A blocking function keeps running
//! until it checks its view; see [`BlockingPolicy`](crate::policy::BlockingPolicy)
//! for how an executor reports work that cannot be aborted.

pub use crate::internal::cancel::{
    CancelReason, CancelReasonKind, CancellationToken, CancellationView,
};
