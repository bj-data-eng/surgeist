//! Measurement flow, semantic window and scope-incidence observations.

pub use crate::protocol::{
    CapturedScopeIncidence, FlowObservation, FlowObservationRequest, FlowSegmentRole,
    PreparedBoundaryIncidence, PreparedFlowSegment, PreparedMeasurementFlow,
    SemanticAtomicAllocation, SemanticConditionalEvidence, SemanticControlAllocation,
    SemanticPartitionEvidence, SemanticWindowEvidence, SemanticWindowFailure,
    SemanticWindowRequest, WrappingScopeQueryError,
};
