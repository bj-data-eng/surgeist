//! Configuration, refusal and shutdown of the durable DOM session.
//!
//! [`DomSession`](crate::DomSession) owns the canonical DOM and its composed obligations. [`DomSessionLimits`] gives finite
//! admission for each owner of the composition. A refused creation is a [`DomSessionCreateError`] that returns the input
//! DOM, and its [`DomSessionCreateFailure`] names the owner that refused. Shutdown ends in a [`DomShutdownOutcome`]: complete,
//! or pending with the session retained and a [`DomShutdownReason`] naming the unresolved obligation.

pub use crate::internal::dom_style::{
    DomSessionCreateError, DomSessionCreateFailure, DomSessionLimits, DomShutdownOutcome,
    DomShutdownReason,
};
