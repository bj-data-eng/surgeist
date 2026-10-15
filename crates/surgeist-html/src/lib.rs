//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

mod attributes;
/// Controlled operations over HTML-owned immutable values.
pub mod bounded;
mod canvas;
mod child_navigable;
mod color;
mod construction;
mod control_values;
mod custom_element_state;
mod custom_names;
mod details_rules;
mod form_association;
mod form_native;
mod form_owner;
mod fragment_initialization;
mod frame;
mod iframe;
mod input;
mod integration_points;
mod interpretation;
mod marquee;
mod media;
mod native_initialization;
mod newlines;
mod nonce;
mod numeric;
mod pi_pseudo_attributes;
mod reflected_element;
mod rel;
mod script_flags;
mod script_preparation;
mod script_result;
mod script_type;
mod select;
mod slot_assign;
mod svg_script_flags;
mod template_markers;
mod template_state;
mod temporal;
mod tokens;
mod values;
mod vocabulary;

pub use attributes::*;
pub use canvas::{HtmlCanvasBitmapDimension, HtmlCanvasNoneState};
pub use color::*;
pub use construction::*;
pub use control_values::*;
pub use custom_element_state::{HtmlCustomElementState, custom_element_state_without_definition};
pub use custom_names::{HtmlCustomElementName, HtmlCustomElementNameError};
pub use details_rules::*;
pub use form_association::HtmlNativeFormAssociationKind;
pub use form_native::HtmlFormNativeState;
pub use fragment_initialization::{
    HtmlFragmentInsertionMode, HtmlFragmentTokenizerState, HtmlParserScriptingMode,
    fragment_insertion_mode, fragment_tokenizer_state,
};
pub use frame::{HtmlFrameNullContent, HtmlFramePostConnectionDecision, frame_post_connection};
pub use iframe::{
    HtmlIframeNativeState, HtmlIframePostConnectionDecision, iframe_loading_became_eager,
    iframe_post_connection,
};
pub use input::*;
pub use integration_points::{is_html_integration_point, is_mathml_text_integration_point};
pub use interpretation::*;
pub use marquee::HtmlMarqueeState;
pub use media::*;
pub use native_initialization::{
    HtmlNativeClosedElementState, HtmlNativeClosedHtmlState, HtmlNativeCommonState,
    HtmlNativeFreshNonce, HtmlNativeFreshSlot, HtmlNativeInitialization,
    HtmlNativeInitializationRequirement,
};
pub use newlines::HtmlNewlineNormalizer;
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
pub use reflected_element::{
    HtmlReflectedElementSetter, HtmlReflectedElementSetterEffect,
    reflected_element_attribute_change,
};
pub use rel::*;
pub use script_flags::HtmlScriptFlags;
pub use script_result::{HtmlScriptReadySteps, HtmlScriptResult};
pub use script_type::HtmlScriptKind;
pub use svg_script_flags::SvgScriptFlags;
pub use template_state::{
    HtmlTemplateContentsOwnerDocument, HtmlTemplateFreshInsertionState,
    template_contents_owner_document,
};
pub use temporal::*;
pub use tokens::{
    HtmlCommaSeparatedTokens, HtmlCommaToken, HtmlDuplicateTokenError, HtmlOrderedTokenSet,
    HtmlSpaceSeparatedTokens, HtmlSpaceToken, HtmlTokenComparison, HtmlTokenError,
    HtmlUnorderedTokenSet,
};
pub use values::*;
pub use vocabulary::*;
