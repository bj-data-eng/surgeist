//! Intrinsic interpretation of selected HTML attributes over exact raw records.
//!
//! The module root holds the attribute report, applicability and conformance types and
//! the keyword enumerations. Each submodule covers one attribute family.

pub mod accept;
pub mod area;
pub mod autocomplete;
pub mod command;
pub mod control_length;
pub mod control_text;
pub mod details_name;
pub mod encoding_label;
pub mod form_name;
pub mod hash_name;
pub mod image_values;
pub mod input_bounds;
pub mod input_colorspace;
pub mod link_destination;
pub mod media;
pub mod meta_pragma;
pub mod meta_processing;
pub mod meter;
pub mod microdata;
pub mod modification;
pub mod navigable_target;
pub mod progress;
pub mod sandbox;
pub mod scalar;
pub mod script_type;
pub mod srcset;
pub mod table_header_scope;
pub mod textarea_wrap;
pub mod token_declarations;

pub use crate::internal::attributes::autocomplete::{
    HtmlAutocompleteContactHint, HtmlAutocompleteMode,
};
pub use crate::internal::attributes::command::{HtmlPopoverTargetAction, HtmlStandardCommand};
pub use crate::internal::attributes::link_destination::HtmlLinkDestination;
pub use crate::internal::attributes::{
    HtmlAttributeApplicability, HtmlAttributeAuthorState, HtmlAttributeConformanceError,
    HtmlAttributeContextAuthorState, HtmlAttributeContextConformanceError,
    HtmlAttributeObservation, HtmlAttributeRecognition, HtmlAttributeReport, HtmlAutocapitalize,
    HtmlAutocapitalizeState, HtmlAutocorrect, HtmlBooleanAttribute, HtmlButtonSubmitterContext,
    HtmlButtonType, HtmlButtonTypeState, HtmlClosedBy, HtmlClosedByState, HtmlContentEditable,
    HtmlContentEditableState, HtmlCrossOrigin, HtmlCrossOriginState,
    HtmlCustomControlAttributeContext, HtmlDecoding, HtmlDir, HtmlDirState, HtmlDraggable,
    HtmlDraggableState, HtmlEnterKeyHint, HtmlFetchPriority, HtmlFormAutocomplete, HtmlFormEnctype,
    HtmlFormMethod, HtmlHidden, HtmlHiddenState, HtmlHyperlinkAttributeContext,
    HtmlImageControlsAttributeContext, HtmlImageMapAttributeContext, HtmlInputMode, HtmlInputType,
    HtmlLoading, HtmlOlType, HtmlPopover, HtmlPopoverState, HtmlPreload, HtmlPreloadState,
    HtmlReferrerPolicy, HtmlScriptAttributeContext, HtmlSpellcheck, HtmlSpellcheckState,
    HtmlStylesheetLinkAttributeContext, HtmlTrackKind, HtmlTranslate, HtmlTranslateState,
    HtmlWritingSuggestions, HtmlWritingSuggestionsState,
};
