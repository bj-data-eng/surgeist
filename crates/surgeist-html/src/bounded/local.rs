//! Local HTML lexical operations shared with qualified semantic consumers.
//!
//! Callers admit the actual next read/write before advancing a cursor. These
//! operations own their grammar and local state; they infer no tree, registry,
//! host facts, work allowance or cancellation policy.

pub use crate::internal::elements::template_markers::{
    First as HtmlTemplateMarkerFirst, HtmlTemplateMarkerNestingOverflow,
    Siblings as HtmlTemplateMarkerSiblings, Target as HtmlTemplateMarkerTarget,
    first as template_marker_first,
};
pub use crate::internal::microsyntax::pi_pseudo_attributes::{
    Action as HtmlPiPseudoAttributeAction, Cursor as HtmlPiPseudoAttributeCursor,
    Error as HtmlPiPseudoAttributeError, Field as HtmlPiPseudoAttributeField,
    Name as HtmlPiPseudoAttributeName, Reply as HtmlPiPseudoAttributeReply, is_xml_name_code_point,
    is_xml_name_start_code_point,
};
pub use crate::internal::microsyntax::tokens::{
    SpaceTokenCursor as HtmlSpaceTokenCursor, SpaceTokenStep as HtmlSpaceTokenStep,
    ascii_case_insensitive_unit, ascii_lowercase, ascii_whitespace as is_ascii_whitespace,
};
pub use crate::internal::security::nonce::{
    NONCE_INSPECTION_BOUND as HTML_NONCE_INSPECTION_BOUND, nonce_element as is_nonce_element,
};
pub use crate::internal::values::custom_names::{
    CustomNameScanAction as HtmlCustomNameScanAction, validate_custom_element_name,
    validate_custom_element_name_with,
};

pub use crate::internal::elements::script_type::{HtmlScriptTypeCursor, HtmlScriptTypeWork};
