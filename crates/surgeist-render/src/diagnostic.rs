//! Typed diagnostics that an [`Error`](crate::Error) carries.
//!
//! [`InvalidValue`], [`UnresolvedResource`] (with [`UnresolvedResourceKind`]) and
//! [`DegradedQuality`] (with [`DegradedQualityKind`]) are the payloads of the matching
//! [`ErrorCode`](crate::ErrorCode) diagnostics. [`UnsupportedPrimitive`] names an unsupported
//! authored operation by its [`PrimitiveFamily`] and [`PrimitiveOperation`].
//! [`RuntimeCapabilityUnavailable`] reports runtime evidence with a
//! [`RuntimeCapabilityUnavailableReason`] and a [`RuntimeOperation`]. [`DeviceLossReason`],
//! [`GpuFaultKind`], [`RenderSurfaceAvailability`] and [`SurfaceIdentityMismatchKind`] classify device,
//! fault and surface conditions, and [`GeometryRealizationError`] reports valid geometry that cannot
//! qualify for its native representation.

pub use crate::error::{
    DegradedQuality, DegradedQualityKind, DeviceLossReason, GeometryRealizationError, GpuFaultKind,
    InvalidValue, PrimitiveFamily, PrimitiveOperation, RenderSurfaceAvailability,
    RuntimeCapabilityUnavailable, RuntimeCapabilityUnavailableReason, RuntimeOperation,
    SurfaceIdentityMismatchKind, UnresolvedResource, UnresolvedResourceKind, UnsupportedPrimitive,
};
