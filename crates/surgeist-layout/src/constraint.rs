//! Available space, sizing operations and requests, orthogonal fallback, and the nearest scrollport.
//!
//! [`AvailableSpaceOf`] is the physical space supplied by the containing boundary, independent of the
//! [`SizingOperation`]. An [`AxisSizingRequestOf`] pairs the two for one axis, and a [`SizingRequestOf`] carries both
//! physical axes with an [`OrthogonalFallbackOf`]: the original initial-containing-block authority, the current
//! [`NearestScrollportOf`] and the local containing limits ([`OrthogonalFallbackLimitOf`]). A
//! [`ScrollportFallbackChangeOf`] says whether a container replaces the nearest scrollport.
//! [`ContentSizingRequestOf`] is the final content-space projection of a request and cannot be requalified for
//! descendants.

pub use crate::value::{
    AvailableSpace, AvailableSpaceOf, AxisSizingRequest, AxisSizingRequestOf, ContentSizingRequest,
    ContentSizingRequestOf, NearestScrollport, NearestScrollportOf, OrthogonalFallback,
    OrthogonalFallbackError, OrthogonalFallbackErrorOf, OrthogonalFallbackLimit,
    OrthogonalFallbackLimitOf, OrthogonalFallbackOf, ScrollportFallbackChange,
    ScrollportFallbackChangeOf, SizingOperation, SizingRequest, SizingRequestOf,
};
