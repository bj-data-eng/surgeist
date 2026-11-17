//! Typed diagnostics that an [`Error`](crate::Error) carries.

pub use crate::error::{
    DegradedQuality, DegradedQualityKind, DeviceLossReason, GeometryRealizationError, GpuFaultKind,
    InvalidValue, PrimitiveFamily, PrimitiveOperation, RenderSurfaceAvailability,
    RuntimeCapabilityUnavailable, RuntimeCapabilityUnavailableReason, RuntimeOperation,
    SurfaceIdentityMismatchKind, UnresolvedResource, UnresolvedResourceKind, UnsupportedPrimitive,
};
