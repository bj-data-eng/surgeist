//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
//!
//! # Errors
//!
//! Every type that a public function returns in an `Err` implements `Debug`,
//! `Display` and [`std::error::Error`]. `Display` states the cause in prose and
//! never prints `Debug`. A wrapper's text describes its own operation and
//! returns the wrapped failure from `source`, so a reporter that walks the chain
//! never prints a cause twice. A public error has one of four shapes. Apply the
//! questions in order to predict the shape of any error:
//!
//! 1. Does the refusal hand back recoverable owners? Then it is a carrier
//!    struct with the reason and an `into_parts` or accessor, such as
//!    [`bounded::HtmlConstructionActionFailure`] and
//!    [`bounded::HtmlValuePoolSealRefusal`].
//! 2. Is it a misuse of an owner or cursor protocol (a consumed, not-ready,
//!    stale, wrong-reply or already-complete state), or does one type merge the
//!    refusals of many operations? Then it is a struct with a private cause,
//!    queried through `is_*` predicates and fact accessors, such as
//!    [`HtmlConstructionError`], [`HtmlCspProtocolError`], the interpretation
//!    protocol errors and the cursor errors like [`HtmlFocusingError`]. New
//!    causes never break a caller.
//! 3. Does a driver charge or branch on every variant, and does a consumer
//!    match it exhaustively? Then it is an exhaustive enum: the resource
//!    accounting enums such as [`bounded::IntegerUnitStepError`] and
//!    [`bounded::FloatingPointAllocationError`], and the two-source wrappers
//!    [`HtmlCspError`] and `bounded::HtmlRelActionError`, which separate the
//!    caller's own refusal from this crate's.
//! 4. Otherwise it names a rule of the pinned standard, a syntax class, an
//!    unavailability reason or a vocabulary that a driver constructs. Then it is
//!    a `#[non_exhaustive]` enum, such as [`HtmlTokenError`],
//!    [`HtmlAttributeConformanceError`] and [`bounded::HtmlValuePoolError`].
//!
//! An owned cause converts with `From`, so `?` applies it. `map_err` remains
//! where the wrapped value is the caller's generic error, because a blanket
//! `From` for it would overlap the protocol conversion.
//!
//! Errors do not capture a [`std::backtrace::Backtrace`]. Most are small `Clone`
//! and `Eq` values that bounded operations return under exact allocation
//! accounting, and a capture allocates outside that accounting and cannot be
//! compared. A recoverable parse diagnostic is a value, not an error, and never
//! pays for a capture. A caller that wants a backtrace captures it where it
//! converts this crate's error into its own.
//!
//! # API evolution
//!
//! Public enums are closed or open by what their variants enumerate:
//!
//! - Closed by the pinned standard: keyword and state sets that the HTML
//!   Standard enumerates, such as input types and token comparison. They are
//!   exhaustive. A new edition of the standard is a deliberate breaking change
//!   that must fail to compile in every consumer.
//! - Closed by a controlled-operation protocol: the action, reply, outcome,
//!   step and phase enums that an owner and its driver exchange, and the
//!   resource vocabularies a driver charges. They are exhaustive, because
//!   matching every variant is what proves that a driver services every action
//!   and charges every resource.
//! - Open: failure causes, rule verdicts, conformance violations and
//!   unavailability reasons grow with the standard and its checks. They are
//!   `#[non_exhaustive]`, so a downstream `match` needs a wildcard arm.
//!
//! Before the root `surgeist` API reaches 1.0 no compatibility alias or
//! migration shim is kept for a changed enum.
//!
//! ```
//! use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
//!
//! // One UTF-16 unit of text fits; eight do not.
//! let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 2, 2))?;
//! let root = builder.root();
//! let refusal = builder.append_text(&root, "too long").unwrap_err();
//!
//! // The cause is private; a caller asks about it and reads its facts.
//! assert!(refusal.is_limit_exceeded());
//! assert_eq!(refusal.attempted(), Some(8));
//! assert_eq!(
//!     refusal.to_string(),
//!     "UTF-16 unit limit of 2 cannot admit a total of 8"
//! );
//! # Ok::<(), surgeist_html::HtmlConstructionError>(())
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![warn(missing_docs)]

mod amortized;
mod attributes;
mod autofocus;
/// Controlled operations over HTML-owned immutable values.
pub mod bounded;
mod canvas;
mod child_navigable;
mod color;
mod construction;
mod control_values;
mod csp_policy;
mod current_focus;
mod custom_element_state;
mod custom_names;
mod design_mode;
mod details_rules;
mod dialog;
mod focus_chain;
mod focus_delegate;
mod focus_event;
mod focus_update;
mod focusing;
mod form_association;
mod form_native;
mod form_owner;
mod fragment_initialization;
mod frame;
mod get_focusable_area;
mod hyperlink;
mod iframe;
mod image;
mod input;
mod input_color;
mod input_image;
mod input_range;
mod integration_points;
mod interpretation;
mod marquee;
mod media;
mod native_initialization;
mod newlines;
mod nonce;
mod numeric;
mod opener_policy;
mod pi_pseudo_attributes;
mod reflected_element;
mod rel;
mod script_flags;
mod script_preparation;
mod script_result;
mod script_type;
mod select;
mod semantic_input;
mod slot_assign;
mod svg_script_flags;
mod template_markers;
mod template_state;
mod temporal;
mod tokens;
mod values;
mod vocabulary;

pub use attributes::*;
pub use autofocus::*;
pub use canvas::{HtmlCanvasBitmapDimension, HtmlCanvasNoneState};
pub use child_navigable::{
    HtmlChildNavigableAccessDecision, HtmlChildNavigableLoadDelayDecision,
    HtmlChildNavigablePresence, HtmlFrameIframeAfterAncestors, HtmlFrameIframeSrcDecision,
};
pub use color::*;
pub use construction::*;
pub use control_values::*;
pub use csp_policy::*;
pub use current_focus::*;
pub use custom_element_state::{HtmlCustomElementState, custom_element_state_without_definition};
pub use custom_names::{HtmlCustomElementName, HtmlCustomElementNameError};
pub use design_mode::*;
pub use details_rules::*;
pub use dialog::*;
pub use focus_chain::*;
pub use focus_delegate::*;
pub use focus_event::*;
pub use focus_update::*;
pub use focusing::*;
pub use form_association::HtmlNativeFormAssociationKind;
pub use form_native::HtmlFormNativeState;
pub use fragment_initialization::{
    HtmlFragmentInsertionMode, HtmlFragmentTokenizerState, HtmlParserScriptingMode,
    fragment_insertion_mode, fragment_tokenizer_state,
};
pub use frame::{
    HtmlFramePostConnectionDecision, frame_post_connection, frame_potentially_delays_load,
};
pub use get_focusable_area::*;
pub use hyperlink::*;
pub use iframe::{
    HtmlIframeAttributeBranch, HtmlIframeInitialState, HtmlIframePostConnectionDecision,
    iframe_attribute_branch, iframe_loading_became_eager, iframe_post_connection,
};
pub use image::*;
pub use input::*;
pub use input_color::{
    HtmlInputColorValueSource, input_color_standard_failure_fallback, input_color_value_source,
};
pub use input_image::{HtmlInputImageHistory, HtmlInputImageUpdateGuard, HtmlInputImagesDisabled};
pub use integration_points::{is_html_integration_point, is_mathml_text_integration_point};
pub use interpretation::*;
pub use marquee::HtmlMarqueeState;
pub use media::*;
pub use native_initialization::{
    HtmlClosedElementState, HtmlCommonHtmlState, HtmlCommonScope, HtmlCommonState,
    HtmlElementClass, HtmlNativeFreshNonce, HtmlNativeFreshSlot, HtmlNativeInitialization,
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
pub use opener_policy::{HtmlOpenerPolicy, HtmlOpenerPolicyValue};
pub use reflected_element::{
    HtmlReflectedElementSetter, HtmlReflectedElementSetterEffect,
    reflected_element_attribute_change,
};
pub use rel::*;
pub use script_flags::HtmlScriptFlags;
pub use script_result::{HtmlScriptReadySteps, HtmlScriptResult};
pub use script_type::HtmlScriptKind;
pub use semantic_input::{HtmlAttributeInput, HtmlAttributesInput, HtmlNameInput};
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
pub use values::{
    Attribute, DocumentString, DuplicateAttributeError, Namespace, OrderedAttributes,
    QualifiedName, Utf8ConversionError,
};
pub use vocabulary::*;

mod style;
pub use style::*;
