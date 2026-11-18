//! Subscription coordination.
//!
//! A [`CoordinationState`] stores subscription references. A [`Subscription`] requests adding a reference to a
//! [`SubscriptionKey`], whose identity is made of its target ([`SubscriptionTarget`], [`SubscriptionTargetKindId`]),
//! [`AppScope`] (built from [`ScopePathSegment`]s), observer and [`SubscriptionPriority`]. A
//! [`SubscriptionAggregate`] is a deterministic view of the active keys for one target, [`SubscriptionChange`] is the
//! result of adding or removing a reference, and a failure is a [`SubscriptionError`] with a
//! [`SubscriptionErrorCode`].

pub use crate::internal::coord::{
    AppScope, CoordinationState, ScopePathSegment, Subscription, SubscriptionAggregate,
    SubscriptionChange, SubscriptionError, SubscriptionErrorCode, SubscriptionKey,
    SubscriptionPriority, SubscriptionTarget, SubscriptionTargetKindId,
};
