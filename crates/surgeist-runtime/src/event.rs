//! Native DOM event dispatch.
//!
//! The [`DomEventDispatcher`] is the runtime's native listener registry and path and invocation owner; listeners are
//! [`DomEventCallback`]s registered with [`DomEventListenerOptions`] for a [`DomEventTarget`] and run against a scoped
//! [`DomEventDom`]. A [`DomEventDispatch`] is prepared for an exact event, target and registry and executes as far as the
//! per-call [`DomEventDispatchLimits`] permit, returning a [`DomEventDispatchOutcome`] and finally a
//! [`DomEventDispatchReceipt`]. [`NativeDomEvent`] is the shared native event body, [`DomEventType`] an exact event type, [`DomEventInit`] the flags the Event
//! constructor accepts and [`DomStandaloneEventTarget`] an authored EventTarget. [`DomEventRegistryLimits`] and
//! [`DomEventStorageLimits`] bound the registry and a session's event objects; refusals and failures are reported as
//! [`DomEventDispatchError`], [`DomEventConstructionError`], [`DomEventResource`], [`DomEventTimestampUnavailable`],
//! [`DomEventCallbackError`] and [`DomEventHostError`].

pub use crate::internal::dom_event::{
    DomEventCallback, DomEventCallbackError, DomEventComposedPath, DomEventConstructionError,
    DomEventCreationObservation, DomEventDispatch, DomEventDispatchError, DomEventDispatchLimits,
    DomEventDispatchOutcome, DomEventDispatchReceipt, DomEventDispatcher, DomEventDom,
    DomEventHostError, DomEventHostInvocation, DomEventInit, DomEventListenerFailure,
    DomEventListenerFailureKind, DomEventListenerOptions, DomEventPhase, DomEventRegistrationId,
    DomEventRegistryLimits, DomEventResource, DomEventStorageLimits, DomEventTarget,
    DomEventTimestampUnavailable, DomEventType, DomStandaloneEventTarget, DomToggleEvent,
    NativeDomEvent,
};
