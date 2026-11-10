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
// A parse records its detail only when its request selects it, and without the
// `detail` feature no request can. The recording code and the data it keeps
// stay compiled, constant-folded away by the unselected control, and the types
// that read them do not exist, so what only those readers use is unused.
#![cfg_attr(not(feature = "detail"), allow(dead_code))]

use surgeist_html_semantics::*;
mod accounting;
mod construction;
mod detail;
#[cfg(feature = "detail")]
mod detail_view;
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
pub use detail::{DetailSet, HtmlParseDetailUnavailable};
#[cfg(feature = "detail")]
pub use detail_view::{
    HtmlParseCauseUnselected, HtmlParseCaused, HtmlParseDetail, HtmlParseLexical,
    HtmlParseOmissions,
};
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
#[cfg(any(feature = "detail", test))]
pub use construction::evidence::{
    HtmlParseDiagnostic, HtmlParseDiagnosticContext, HtmlParseDiagnosticKind,
    HtmlParseDiagnosticLocation, HtmlParseDiagnosticLocationRole, HtmlParseDiagnosticRule,
    HtmlParseDiagnosticSeverity, HtmlParseDiagnostics, HtmlParseTokenFamily,
};
#[cfg(any(feature = "detail", test))]
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
#[cfg(any(feature = "detail", test))]
pub use construction::provenance::reverse::{HtmlParseAssociationRelation, HtmlParseReverseError};
#[cfg(any(feature = "detail", test))]
pub use construction::provenance::{
    HtmlParseCapturedCauseRole, HtmlParseConstructionOperation, HtmlParseRawField, HtmlParserRule,
};
#[cfg(any(feature = "detail", test))]
pub use construction::provenance::{
    HtmlParseColorOperandRole, HtmlParseColorValueSource, HtmlParseRangeBranch,
    HtmlParseRangeOperandKind, HtmlParseRangeOperandRole,
};
#[cfg(any(feature = "detail", test))]
pub use construction::provenance::{
    HtmlParseLexicalDisposition, HtmlParseLexicalKind, HtmlParseLexicalRole, HtmlParseOmissionKind,
};
pub use construction::read::HtmlParseScript;
#[cfg(any(feature = "detail", test))]
pub use construction::read::{
    HtmlParseAssociation, HtmlParseAssociations, HtmlParseCapturedCause, HtmlParseCause,
    HtmlParseDestination, HtmlParseField, HtmlParseHistory, HtmlParseHistoryHeader,
    HtmlParseLexicalOccurrence, HtmlParseLexicalPart, HtmlParseOmission, HtmlParseReverseCursor,
    HtmlParseReverseStep,
};
#[cfg(any(feature = "detail", test))]
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
#[cfg(any(feature = "detail", test))]
pub use parser::tokenizer::states::{
    HtmlTokenizerEscape, HtmlTokenizerIdentifier, HtmlTokenizerQuote, HtmlTokenizerRawState,
    HtmlTokenizerState,
};
#[cfg(any(feature = "detail", test))]
pub use parser::tokenizer::token::{
    HtmlParseOriginKind, HtmlParseTextOrigin, HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticPhase,
};
#[cfg(any(feature = "detail", test))]
pub use parser::tree_builder::head::{HtmlInsertionMode, HtmlTreeDiagnostic};
#[cfg(any(feature = "detail", test))]
pub use parser::tree_builder::prologue::{HtmlPrologueDiagnostic, HtmlPrologueMode};

pub use construction::context::HtmlParseOpaqueOrigin;
pub use construction::fragment_document::HtmlParseFragmentParserDocument;
