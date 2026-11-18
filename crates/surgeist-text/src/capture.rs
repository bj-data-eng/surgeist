//! Contextual source capture and its processed paragraph views.

pub use crate::processing::contextual::{
    CapturedComputedLineBoundary, CapturedComputedLineBoundaryTarget, CapturedConnection,
    CapturedControl, CapturedDomainDefinition, CapturedEventPosition, CapturedFlowEvent,
    CapturedJointFlow, CapturedParagraph, CapturedScope, CapturedScopeIntersection,
    ComputedLineBoundaryError, ComputedLineBoundaryInput, ContextualSourceInput,
    ProcessedParagraph, ScopeRole, SourceBoxSide, SourceContinuityOperation,
    SourceProcessingConnection, SourceProcessingConnectionError, SourceProcessingEnvelope,
    SourceProcessingError, SourceProcessingEvent, SourceProcessingJointFlow,
    SourceProcessingJointFlowError, SourceProcessingParagraph, SourceProcessingScope,
    SourceProcessingScopeBoundaries, SourceProcessingTopology, SourceWritingAxis, WhiteSpaceTrim,
    process_contextual_source,
};
