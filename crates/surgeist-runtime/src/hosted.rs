//! The native hosted lifecycle of a session.
//!
//! [`HostedRuntime`] is a weak execution permission for the session's native lifecycle owner, bounded by
//! [`DomHostedRuntimeLimits`] and refused with a [`DomHostedRuntimeError`]. It creates and destroys hosts and
//! top-level Documents; a created top-level is observed as an immutable [`DomHostedTopLevel`], and its settings and
//! initial history entry as [`DomHostedSettings`] and [`DomHostedHistoryEntry`]. Queued teardown is observed as
//! [`DomHostedDestructionQueued`], and task execution as [`DomHostedTaskProgress`], [`DomHostedTaskKind`] and
//! [`DomHostedTaskWait`]. [`DomSourceExecutionScope`] is a weak permission naming one host.

pub use crate::internal::dom_lifecycle::{
    DomHostedDestructionQueued, DomHostedHistoryEntry, DomHostedRuntimeError,
    DomHostedRuntimeLimits, DomHostedSettings, DomHostedTaskKind, DomHostedTaskProgress,
    DomHostedTaskWait, DomHostedTopLevel, DomSourceExecutionScope, HostedRuntime,
};
