//! Where an input came from and what caused it.
//!
//! [`InputProvenance`] records an input's source ([`InputOrigin`], with [`InputSourceId`], [`ServiceProvenance`],
//! [`SurfaceProvenance`] and [`TaskProvenance`]) and its current and parent causal [`Correlation`], which are independent.
//! Attaching a surface to a provenance can be rejected with a [`ProvenanceError`] classified by a
//! [`ProvenanceErrorCode`].

pub use crate::internal::provenance::{
    Correlation, InputOrigin, InputProvenance, InputSourceId, ProvenanceError, ProvenanceErrorCode,
    ServiceProvenance, SurfaceProvenance, TaskProvenance,
};
