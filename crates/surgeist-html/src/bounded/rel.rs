//! Retained local relationship interpretation and checked source-ordered refinement.
pub use crate::internal::elements::rel::controlled::{
    HtmlRelActionError, HtmlRelInputs, HtmlRelInterpretation, HtmlRelInterpretationError,
    HtmlRelMeter, HtmlRelRefinement, HtmlRelReservation, classify_rel, rel_ascii_keyword,
    rel_href_is_empty_or_ascii_whitespace_with,
};
