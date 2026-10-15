//! Decoded-Unicode HTML ingestion, recovery and source-owning immutable reports.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

use surgeist_html::bounded;
use surgeist_html::*;
use surgeist_html_semantics::*;
mod construction;
mod parser;
mod request;
mod settings;
mod source;
pub use request::*;
pub use settings::*;
pub use source::*;
pub use surgeist_html::HtmlScriptKind;

pub use construction::read::{HtmlInputImageSource, HtmlParseRetiredDestination};
pub use construction::{
    ConstructionCompletion as HtmlParseCompletion, ConstructionReadiness as HtmlParseReadiness,
    StandaloneDocumentState as HtmlParseDocumentState,
};
pub use parser::control::{
    HtmlColorProcessingError, Invariant as HtmlParseInvariant, ParseStop as HtmlParseStop,
    ParseUsage as HtmlParseUsage, ResourceUsage as HtmlParseResourceUsage,
    StandaloneUnsupported as HtmlParseUnsupportedState, WorkCategory as HtmlParseWorkCategory,
};
pub use parser::tree_builder::head::HeadBoundary as HtmlParseConstructionBoundary;
pub use parser::tree_builder::parse::{
    HtmlParseEngineCondition, HtmlParseFailure, HtmlParseFailureReason,
};
pub use parser::tree_builder::report::ImmutableParseReport as HtmlParseReport;

pub use construction::SvgScriptOrigin as HtmlParseSvgScriptOrigin;
pub use construction::context::{
    FrozenCapturedRegistry as HtmlParseCapturedRegistry, FrozenFragmentEffect as HtmlParseEffect,
    FrozenFragmentEffects as HtmlParseEffects, FrozenFragmentHost as HtmlParseEffectHost,
    FrozenFragmentRegistry as HtmlParseRegistry,
    FrozenFragmentRegistryBinding as HtmlParseRegistryBinding,
};
pub use construction::evidence::{
    DiagnosticContext as HtmlParseDiagnosticContext,
    DiagnosticEvidenceView as HtmlParseDiagnostics, DiagnosticKind as HtmlParseDiagnosticKind,
    DiagnosticLocation as HtmlParseDiagnosticLocation,
    DiagnosticLocationRole as HtmlParseDiagnosticLocationRole,
    DiagnosticRecord as HtmlParseDiagnostic, DiagnosticRule as HtmlParseDiagnosticRule,
    DiagnosticSeverity as HtmlParseDiagnosticSeverity, DiagnosticToken as HtmlParseTokenFamily,
};
pub use construction::history::{
    HistoryBody as HtmlParseHistoryBody, HistoryCursor as HtmlParseHistoryCursor,
    HistoryFrame as HtmlParseHistoryFrame, HistoryGroup as HtmlParseOriginGroup,
    HistoryLeaf as HtmlParseHistoryLeaf, HistoryOrigin as HtmlParseHistoryOrigin,
    HistoryReadError as HtmlParseHistoryError, HistoryStep as HtmlParseHistoryStep,
};
pub use construction::provenance::reverse::{
    AssociationRelation as HtmlParseAssociationRelation, ReverseQueryError as HtmlParseReverseError,
};
pub use construction::provenance::{
    CapturedCauseRole as HtmlParseCapturedCauseRole,
    ConstructionOperation as HtmlParseConstructionOperation, ParserRule as HtmlParserRule,
    RawFieldKind as HtmlParseRawField,
};
pub use construction::provenance::{
    ColorOperandRole as HtmlParseColorOperandRole, ColorValueSource as HtmlParseColorValueSource,
    RangeBranch as HtmlParseRangeBranch, RangeOperandKind as HtmlParseRangeOperandKind,
    RangeOperandRole as HtmlParseRangeOperandRole,
};
pub use construction::provenance::{
    LexicalDisposition as HtmlParseLexicalDisposition, LexicalKind as HtmlParseLexicalKind,
    LexicalRole as HtmlParseLexicalRole, OmissionKind as HtmlParseOmissionKind,
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
    MetadataBase as HtmlParseBase, MetadataCursor as HtmlParseMetadataCursor,
    MetadataDocument as HtmlParseMetadataDocument, MetadataRef as HtmlParseMetadata,
    MetadataStep as HtmlParseMetadataStep, MetadataTemplate as HtmlParseTemplate,
};
pub use construction::{
    FormAssociationEvidence as HtmlParseFormEvidence, FormAssociationKind as HtmlParseFormKind,
    OutputDefaultValueOverride as HtmlParseOutputOverride,
    ScriptReadySteps as HtmlParseScriptReadySteps, ScriptResult as HtmlParseScriptResult,
};
pub use parser::tokenizer::states::{
    Escape as HtmlTokenizerEscape, Identifier as HtmlTokenizerIdentifier,
    Quote as HtmlTokenizerQuote, Raw as HtmlTokenizerRawState, State as HtmlTokenizerState,
};
pub use parser::tokenizer::token::{
    DiagnosticCode as HtmlTokenizerDiagnostic, DiagnosticPhase as HtmlTokenizerDiagnosticPhase,
    OriginKind as HtmlParseOriginKind, TextOrigin as HtmlParseTextOrigin,
};
pub use parser::tree_builder::head::{
    HeadCondition as HtmlTreeDiagnostic, InsertionMode as HtmlInsertionMode,
};
pub use parser::tree_builder::prologue::{
    PrologueCondition as HtmlPrologueDiagnostic, PrologueMode as HtmlPrologueMode,
};
