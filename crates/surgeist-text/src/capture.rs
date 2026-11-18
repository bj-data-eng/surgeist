//! Contextual source capture and its processed paragraph views.
//!
//! A [`ContextualSourceInput`] is a complete immutable envelope; its tables hold paragraphs, scopes, connections
//! and computed line boundaries. [`process_contextual_source`] trims it, executes the shared Phase I lexer once
//! and freezes the views, among them a [`ProcessedParagraph`]. [`SourceProcessingEvent`] is the sole ordered
//! occurrence stream and refers to the capture-local table ordinals by index; the `Captured*` handle types are
//! separate, each pairing an ordinal with its capture. The table
//! rows are [`SourceProcessingParagraph`], [`SourceProcessingScope`] (with [`SourceProcessingScopeBoundaries`]),
//! [`SourceProcessingConnection`], [`SourceProcessingJointFlow`] and [`ComputedLineBoundaryInput`];
//! [`CapturedComputedLineBoundaryTarget`] is the canonical target of one input computed mandatory cause and
//! [`WhiteSpaceTrim`] holds computed non-inherited trim facts.
//! [`SourceProcessingEnvelope`] and [`SourceProcessingTopology`] say how much context the producer captured,
//! [`SourceWritingAxis`] and [`ScopeRole`] classify a domain and a scope, and [`SourceProcessingError`] and the
//! other `*Error` types name the typed rejections.

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
