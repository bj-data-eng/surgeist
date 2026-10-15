//! Local HTML lexical operations shared with qualified semantic consumers.
//!
//! Callers admit the actual next read/write before advancing a cursor. These
//! operations own their grammar and local state; they infer no tree, registry,
//! host facts, work allowance or cancellation policy.

pub use crate::custom_names::{
    CustomNameScanAction as HtmlCustomNameScanAction, validate_custom_element_name,
    validate_custom_element_name_with,
};
pub use crate::nonce::{
    NONCE_INSPECTION_BOUND as HTML_NONCE_INSPECTION_BOUND, nonce_element as is_nonce_element,
};
pub use crate::pi_pseudo_attributes::{
    Action as HtmlPiPseudoAttributeAction, Cursor as HtmlPiPseudoAttributeCursor,
    Error as HtmlPiPseudoAttributeError, Field as HtmlPiPseudoAttributeField,
    Name as HtmlPiPseudoAttributeName, Reply as HtmlPiPseudoAttributeReply, is_xml_name_code_point,
    is_xml_name_start_code_point,
};
pub use crate::template_markers::{
    First as HtmlTemplateMarkerFirst, HtmlTemplateMarkerNestingOverflow,
    Siblings as HtmlTemplateMarkerSiblings, Target as HtmlTemplateMarkerTarget,
    first as template_marker_first,
};
pub use crate::tokens::{
    SpaceTokenCursor as HtmlSpaceTokenCursor, SpaceTokenStep as HtmlSpaceTokenStep,
    ascii_whitespace as is_ascii_whitespace,
};

pub use crate::script_type::{HtmlScriptTypeCursor, HtmlScriptTypeWork};
