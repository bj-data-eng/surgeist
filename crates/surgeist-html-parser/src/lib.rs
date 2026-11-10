//! Decoded-Unicode HTML ingestion, recovery and source-owning immutable reports.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
// No run-time panic is reachable from parsing, so production code carries none
// of the operations that can panic. A site that is provably total says why in a
// narrow `#[expect]`; test code is outside the gate.
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing,
        clippy::string_slice,
        clippy::arithmetic_side_effects
    )
)]

use surgeist_html_semantics::*;
mod accounting;
mod construction;
mod keyed;
mod meta_csp;
mod parser;
pub use meta_csp::{HtmlParseCspFailure, HtmlParseCspFailureKind};
mod request;
#[cfg(test)]
mod schema_support;
mod settings;
mod source;
pub use accounting::HtmlParseAccounting;
pub use request::*;
pub use settings::*;
pub use source::*;

pub use construction::read::{
    HtmlInputImageSource, HtmlParseRetiredDestination, HtmlParseUnresolvedKind,
    HtmlParseUnresolvedReference,
};
pub use construction::{HtmlParseCompletion, HtmlParseDocumentState, HtmlParseReadiness};
pub use parser::control::{
    HtmlColorProcessingError, HtmlParseInvariant, HtmlParseResourceUsage, HtmlParseStop,
    HtmlParseUnsupportedState, HtmlParseUsage, HtmlParseWorkCategory,
};
pub use parser::tree_builder::head::HtmlParseConstructionBoundary;
pub use parser::tree_builder::parse::{
    HtmlParseEngineCondition, HtmlParseFailure, HtmlParseFailureReason,
};
pub use parser::tree_builder::report::HtmlParseReport;

pub use construction::HtmlParseSvgScriptOrigin;
pub use construction::associated::HtmlParseHyperlinkUnavailable;
pub use construction::context::{
    HtmlParseCapturedRegistry, HtmlParseEffect, HtmlParseEffectHost, HtmlParseEffects,
    HtmlParseRegistry, HtmlParseRegistryBinding,
};
pub use construction::evidence::{
    HtmlParseDiagnostic, HtmlParseDiagnosticContext, HtmlParseDiagnosticKind,
    HtmlParseDiagnosticLocation, HtmlParseDiagnosticLocationRole, HtmlParseDiagnosticRule,
    HtmlParseDiagnosticSeverity, HtmlParseDiagnostics, HtmlParseTokenFamily,
};
pub use construction::history::{
    HtmlParseHistoryBody, HtmlParseHistoryCursor, HtmlParseHistoryError, HtmlParseHistoryFrame,
    HtmlParseHistoryLeaf, HtmlParseHistoryOrigin, HtmlParseHistoryStep, HtmlParseOriginGroup,
};
pub use construction::meta::{HtmlParseMetaDomain, HtmlParseMetaMissingKind};
pub use construction::pending_association::{
    HtmlParseAssociationCaptureDisposition, HtmlParseAssociationCapturedObservation,
    HtmlParseAssociationContext, HtmlParseAssociationContextEpochId,
    HtmlParseAssociationContextRequirement, HtmlParseAssociationContextRole,
    HtmlParseAssociationContextSlotId, HtmlParseAssociationDocument,
    HtmlParseAssociationMissingKind, HtmlParseAssociationOccurrence,
    HtmlParseAssociationOccurrenceId, HtmlParseAssociationOccurrenceKind,
    HtmlParseAssociationStyleInputs, HtmlParsePendingAssociationCursor,
    HtmlParsePendingAssociationStep,
};
pub use construction::provenance::reverse::{HtmlParseAssociationRelation, HtmlParseReverseError};
pub use construction::provenance::{
    HtmlParseCapturedCauseRole, HtmlParseConstructionOperation, HtmlParseRawField, HtmlParserRule,
};
pub use construction::provenance::{
    HtmlParseColorOperandRole, HtmlParseColorValueSource, HtmlParseRangeBranch,
    HtmlParseRangeOperandKind, HtmlParseRangeOperandRole,
};
pub use construction::provenance::{
    HtmlParseLexicalDisposition, HtmlParseLexicalKind, HtmlParseLexicalRole, HtmlParseOmissionKind,
};
pub use construction::read::{
    HtmlParseAssociation, HtmlParseAssociations, HtmlParseCapturedCause, HtmlParseCause,
    HtmlParseDestination, HtmlParseField, HtmlParseHistory, HtmlParseHistoryHeader,
    HtmlParseLexicalOccurrence, HtmlParseLexicalPart, HtmlParseOmission, HtmlParseReverseCursor,
    HtmlParseReverseStep, HtmlParseScript,
};
pub use construction::read::{
    HtmlParseColorComputation, HtmlParseRangeComputation, HtmlParseRangeOperand,
};
pub use construction::report::{
    HtmlParseBase, HtmlParseDialog, HtmlParseDialogDisposition, HtmlParseDialogMissing,
    HtmlParseDocumentInteraction, HtmlParseMetadata, HtmlParseMetadataCursor,
    HtmlParseMetadataDocument, HtmlParseMetadataStep, HtmlParseTemplate,
};
pub use construction::report::{
    HtmlParseBaseState, HtmlParseDocumentMeta, HtmlParseMetaDisposition, HtmlParseMetaMissing,
    HtmlParseMetaProfile, HtmlParseMetaRefresh,
};
pub use construction::report::{
    HtmlParseHyperlink, HtmlParseHyperlinkState, HtmlParseImage, HtmlParseImageUpdate,
    HtmlParseStyle,
};
pub use construction::style::HtmlParseStyleAssociation;
pub use construction::{HtmlParseFormEvidence, HtmlParseOutputOverride};
pub use parser::tokenizer::states::{
    HtmlTokenizerEscape, HtmlTokenizerIdentifier, HtmlTokenizerQuote, HtmlTokenizerRawState,
    HtmlTokenizerState,
};
pub use parser::tokenizer::token::{
    HtmlParseOriginKind, HtmlParseTextOrigin, HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticPhase,
};
pub use parser::tree_builder::head::{HtmlInsertionMode, HtmlTreeDiagnostic};
pub use parser::tree_builder::prologue::{HtmlPrologueDiagnostic, HtmlPrologueMode};

pub use construction::context::HtmlParseOpaqueOrigin;
pub use construction::fragment_document::HtmlParseFragmentParserDocument;

#[doc(hidden)]
pub fn tally_report() -> String {
    parser::control::tally::report()
}
#[doc(hidden)]
pub fn tally_reset() {
    parser::control::tally::reset()
}
