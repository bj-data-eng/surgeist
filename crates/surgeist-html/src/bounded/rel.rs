//! Retained local relationship interpretation and checked source-ordered refinement.
pub use crate::internal::elements::rel::controlled::{
    HtmlRelActionError, HtmlRelInterpretation, HtmlRelInterpretationError, HtmlRelMeter,
    HtmlRelRefinement, HtmlRelReservation, RelInputs, classify_rel, rel_ascii_keyword,
    rel_href_is_empty_or_ascii_whitespace_with,
};
