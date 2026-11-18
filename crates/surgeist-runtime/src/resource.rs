//! Resource state and its transitions.
//!
//! A [`ResourceState`] holds a resource value, its current operation and a checked generation. Starting a load or refresh
//! issues a [`ResourceOperation`], which the completion and cancellation methods accept. The lifecycle is a [`ResourceStatus`],
//! whether a retained value is current is its [`Freshness`], and [`FailureVisibility`] controls whether a failed
//! operation keeps its previous value. A [`ResourceSnapshot`] is an immutable clone of the observable state; a rejected
//! transition is a [`ResourceStateError`] classified by a [`ResourceStateErrorCode`].

pub use crate::internal::resource::{
    FailureVisibility, Freshness, ResourceOperation, ResourceSnapshot, ResourceState,
    ResourceStateError, ResourceStateErrorCode, ResourceStatus,
};
