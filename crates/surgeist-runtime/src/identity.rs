//! Identities, generations and versions.
//!
//! The identifier types ([`AppId`], [`CalcId`], [`CustomScopeId`], [`ElementId`], [`ExpressionId`], [`ResourceId`],
//! [`RootId`], [`ServiceId`], [`SurfaceId`], [`ValueExprId`], [`WindowId`]), the generation counters
//! ([`ResourceGeneration`], [`SurfaceGeneration`], [`SurfaceInvalidationGeneration`]) and [`StateVersion`] are the plain
//! values the other modules name things and order changes with. [`CorrelationId`] is nonzero, so a present correlation is
//! never ambiguous with absence; [`CorrelationError`] rejects zero. [`ResourceOperationId`] is a nonzero operation
//! identity issued by a [`ResourceState`](crate::resource::ResourceState). [`VersionError`] reports that a checked counter
//! cannot advance further.

pub use crate::internal::ids::{
    AppId, CalcId, CorrelationError, CorrelationId, CustomScopeId, ElementId, ExpressionId,
    ResourceGeneration, ResourceId, ResourceOperationId, RootId, ServiceId, SurfaceGeneration,
    SurfaceId, SurfaceInvalidationGeneration, ValueExprId, VersionError, WindowId,
};
pub use crate::internal::snapshot::StateVersion;
