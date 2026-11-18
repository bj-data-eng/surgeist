//! Retained UI surfaces.
//!
//! A [`UiSurface`] is a registered surface with a generation-qualified identity ([`SurfaceRef`]) and a
//! [`SurfaceLifecycle`]. Its [`SurfaceRoot`] retains the root identity and the registered event-target elements
//! ([`SurfaceElements`], each an [`ElementRegistration`] of [`ElementPhase`]s); a [`SurfaceElementRef`] refers to one
//! element on one surface by generation. A [`SurfaceRoute`] is a validated
//! ordered route of [`SurfaceRouteStep`]s. A [`SurfaceInvalidation`] (of a [`SurfaceInvalidationKind`]) records why rendering
//! work is needed, a render begins with a [`SurfaceRenderFrame`], [`SurfaceRenderState`] pairs the application state view
//! with its frame, and [`SurfaceRenderAck`] is the result of accepting an acknowledgement. [`SurfaceMutation`] is the
//! observable result of a local update, [`SurfacePoint`] and [`SurfaceSize`] are logical coordinates and dimensions, and a
//! rejection is a [`SurfaceError`] classified by a [`SurfaceErrorCode`].

pub use crate::internal::surface::{
    ElementPhase, ElementRegistration, SurfaceElementRef, SurfaceElements, SurfaceError,
    SurfaceErrorCode, SurfaceInvalidation, SurfaceInvalidationKind, SurfaceLifecycle,
    SurfaceMutation, SurfacePoint, SurfaceRef, SurfaceRenderAck, SurfaceRenderFrame,
    SurfaceRenderState, SurfaceRoot, SurfaceRoute, SurfaceRouteStep, SurfaceSize, UiSurface,
};
