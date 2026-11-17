//! The native event loop and the validated transitions a native host reports.
//!
//! A [`Loop`] owns one handler, its registry and its startup requests, and
//! runs the native event loop. A native host reports window changes as a
//! [`NativeEventTransition`], which validates against the window's snapshot
//! and derives the event handlers observe. [`WindowStatePatch`] is the state
//! change such a transition commits, and [`MetricsEvent`] the event a metrics
//! patch selects.

pub use super::loop_::Loop;
pub use super::transition::{MetricsEvent, NativeEventTransition, WindowStatePatch};
