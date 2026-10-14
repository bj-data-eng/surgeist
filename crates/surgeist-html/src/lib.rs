//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

mod attributes;
mod base;
mod bidi;
/// Controlled operations over HTML-owned immutable values.
pub mod bounded;
mod capture;
mod color;
mod construction;
mod control_values;
mod custom_names;
mod details_rules;
mod fragment_initialization;
mod input;
mod integration_points;
mod interpretation;
mod media;
mod nonce;
mod numeric;
mod parser;
mod pi_pseudo_attributes;
mod rel;
mod request;
mod script_type;
mod semantics;
mod settings;
mod source;
mod template_markers;
mod temporal;
mod tokens;
mod values;
mod vocabulary;

pub use attributes::*;
pub use base::*;
pub use capture::*;
pub use color::*;
pub use construction::*;
pub use control_values::*;
pub use custom_names::{HtmlCustomElementName, HtmlCustomElementNameError};
pub use details_rules::*;
pub use fragment_initialization::{
    HtmlFragmentInsertionMode, HtmlFragmentTokenizerState, HtmlParserScriptingMode,
    fragment_insertion_mode, fragment_tokenizer_state,
};
pub use input::*;
pub use integration_points::{is_html_integration_point, is_mathml_text_integration_point};
pub use interpretation::*;
pub use media::*;
pub use nonce::{
    HtmlBrowsingContextPresence, HtmlCapturedNonce, HtmlHeaderDeliveredCsp,
    HtmlNonceAttributeEffect, HtmlNonceConnection, HtmlNonceDocumentEnvironment,
    HtmlNonceInsertionEffect, HtmlNonceMissing, nonce_attribute_change, nonce_insertion_effect,
};
pub use numeric::{
    HtmlDecimalDigitsError, HtmlDimension, HtmlDimensionList, HtmlFloatingPointList,
    HtmlFloatingPointNumber, HtmlInteger, HtmlNegativeIntegerError, HtmlNonFiniteNumberError,
    HtmlNonNegativeDecimal, HtmlNonNegativeInteger, HtmlNonNegativeRational, HtmlNumericRangeError,
    HtmlRationalError, HtmlSignedDecimal,
};
pub use rel::*;
pub use request::*;
pub use script_type::HtmlScriptKind;
pub use semantics::*;
pub use settings::*;
pub use source::*;
pub use temporal::*;
pub use tokens::{
    HtmlCommaSeparatedTokens, HtmlCommaToken, HtmlDuplicateTokenError, HtmlOrderedTokenSet,
    HtmlSpaceSeparatedTokens, HtmlSpaceToken, HtmlTokenComparison, HtmlTokenError,
    HtmlUnorderedTokenSet,
};
pub use values::*;
pub use vocabulary::*;

pub use construction::parser_session::read::{HtmlInputImageSource, HtmlParseRetiredDestination};
pub use construction::parser_session::{
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

pub use construction::parser_session::SvgScriptOrigin as HtmlParseSvgScriptOrigin;
pub use construction::parser_session::context::{
    FrozenCapturedRegistry as HtmlParseCapturedRegistry, FrozenFragmentEffect as HtmlParseEffect,
    FrozenFragmentEffects as HtmlParseEffects, FrozenFragmentHost as HtmlParseEffectHost,
    FrozenFragmentRegistry as HtmlParseRegistry,
    FrozenFragmentRegistryBinding as HtmlParseRegistryBinding,
};
pub use construction::parser_session::evidence::{
    DiagnosticContext as HtmlParseDiagnosticContext,
    DiagnosticEvidenceView as HtmlParseDiagnostics, DiagnosticKind as HtmlParseDiagnosticKind,
    DiagnosticLocation as HtmlParseDiagnosticLocation,
    DiagnosticLocationRole as HtmlParseDiagnosticLocationRole,
    DiagnosticRecord as HtmlParseDiagnostic, DiagnosticRule as HtmlParseDiagnosticRule,
    DiagnosticSeverity as HtmlParseDiagnosticSeverity, DiagnosticToken as HtmlParseTokenFamily,
};
pub use construction::parser_session::history::{
    HistoryBody as HtmlParseHistoryBody, HistoryCursor as HtmlParseHistoryCursor,
    HistoryFrame as HtmlParseHistoryFrame, HistoryGroup as HtmlParseOriginGroup,
    HistoryLeaf as HtmlParseHistoryLeaf, HistoryOrigin as HtmlParseHistoryOrigin,
    HistoryReadError as HtmlParseHistoryError, HistoryStep as HtmlParseHistoryStep,
};
pub use construction::parser_session::provenance::reverse::{
    AssociationRelation as HtmlParseAssociationRelation, ReverseQueryError as HtmlParseReverseError,
};
pub use construction::parser_session::provenance::{
    CapturedCauseRole as HtmlParseCapturedCauseRole,
    ConstructionOperation as HtmlParseConstructionOperation, ParserRule as HtmlParserRule,
    RawFieldKind as HtmlParseRawField,
};
pub use construction::parser_session::provenance::{
    ColorOperandRole as HtmlParseColorOperandRole, ColorValueSource as HtmlParseColorValueSource,
    RangeBranch as HtmlParseRangeBranch, RangeOperandKind as HtmlParseRangeOperandKind,
    RangeOperandRole as HtmlParseRangeOperandRole,
};
pub use construction::parser_session::provenance::{
    LexicalDisposition as HtmlParseLexicalDisposition, LexicalKind as HtmlParseLexicalKind,
    LexicalRole as HtmlParseLexicalRole, OmissionKind as HtmlParseOmissionKind,
};
pub use construction::parser_session::read::{
    HtmlParseAssociation, HtmlParseAssociations, HtmlParseCapturedCause, HtmlParseCause,
    HtmlParseDestination, HtmlParseField, HtmlParseHistory, HtmlParseHistoryHeader,
    HtmlParseLexicalOccurrence, HtmlParseLexicalPart, HtmlParseOmission, HtmlParseReverseCursor,
    HtmlParseReverseStep, HtmlParseScript,
};
pub use construction::parser_session::read::{
    HtmlParseColorComputation, HtmlParseRangeComputation, HtmlParseRangeOperand,
};
pub use construction::parser_session::report::{
    MetadataBase as HtmlParseBase, MetadataCursor as HtmlParseMetadataCursor,
    MetadataDocument as HtmlParseMetadataDocument, MetadataRef as HtmlParseMetadata,
    MetadataStep as HtmlParseMetadataStep, MetadataTemplate as HtmlParseTemplate,
};
pub use construction::parser_session::{
    FormAssociationEvidence as HtmlParseFormEvidence, FormAssociationKind as HtmlParseFormKind,
    OutputDefaultValueOverride as HtmlParseOutputOverride,
    ScriptReadySteps as HtmlParseScriptReadySteps, ScriptResult as HtmlParseScriptResult,
    ScriptType as HtmlParseScriptType,
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
