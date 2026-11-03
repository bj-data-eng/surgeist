//! Local HTML lexical operations shared with qualified semantic consumers.
//!
//! Callers admit the actual next read/write before advancing a cursor. These
//! operations own their grammar and local state; they infer no tree, registry,
//! host facts, work allowance or cancellation policy.

pub use crate::internal::microsyntax::pi_pseudo_attributes::{
    HtmlPiPseudoAttributeAction, HtmlPiPseudoAttributeCursor, HtmlPiPseudoAttributeError,
    HtmlPiPseudoAttributeField, HtmlPiPseudoAttributeName, HtmlPiPseudoAttributeReply,
    is_xml_name_code_point, is_xml_name_start_code_point,
};
pub use crate::internal::microsyntax::tokens::{
    HtmlSpaceTokenCursor, HtmlSpaceTokenStep, ascii_case_insensitive_unit, ascii_lowercase,
    is_ascii_whitespace,
};
pub use crate::internal::values::custom_names::{
    HtmlCustomNameScanAction, validate_custom_element_name, validate_custom_element_name_with,
};
