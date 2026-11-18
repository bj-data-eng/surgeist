//! Measurement flow, semantic window and scope-incidence observations.
//!
//! [`PreparedMeasurementFlow`], [`PreparedFlowSegment`], [`FlowObservationRequest`], [`FlowObservation`] and
//! [`FlowSegmentRole`] describe measurement flow. [`SemanticWindowRequest`], [`SemanticWindowEvidence`],
//! [`SemanticWindowFailure`], [`SemanticPartitionEvidence`], [`SemanticConditionalEvidence`],
//! [`SemanticAtomicAllocation`] and [`SemanticControlAllocation`] describe a semantic window and its
//! evidence. [`CapturedScopeIncidence`], [`PreparedBoundaryIncidence`] and [`WrappingScopeQueryError`] relate a
//! placement to an inline box.

pub use crate::protocol::{
    CapturedScopeIncidence, FlowObservation, FlowObservationRequest, FlowSegmentRole,
    PreparedBoundaryIncidence, PreparedFlowSegment, PreparedMeasurementFlow,
    SemanticAtomicAllocation, SemanticConditionalEvidence, SemanticControlAllocation,
    SemanticPartitionEvidence, SemanticWindowEvidence, SemanticWindowFailure,
    SemanticWindowRequest, WrappingScopeQueryError,
};
