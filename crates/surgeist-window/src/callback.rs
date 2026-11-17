//! The scope values a handler receives and the work they request.
//!
//! Each [`Handler`](crate::Handler) callback receives one scope: [`Ready`],
//! [`Frame`], [`Resize`], [`Input`], [`Event`], [`Close`] or [`Closed`]. The
//! [`Scope`] trait is their shared live-window access. Work requested through
//! a scope is queued on its [`Context`], and a [`Target`] addresses one window,
//! chosen with a [`Selector`], to author commands for it. Queued work commits
//! only when the callback returns `Ok`.

pub use super::context::Context;
pub use super::dsl::{Close, Closed, Event, Frame, Input, Ready, Resize, Scope, Selector, Target};
