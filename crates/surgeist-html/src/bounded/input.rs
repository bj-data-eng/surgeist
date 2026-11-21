//! Bounded Input attribute recognition and source-ordered range sanitization.
//!
//! Arithmetic, strings and admission stay with the caller.

pub use crate::internal::elements::input::input_colorspace_with;
pub use crate::internal::elements::input_range::{
    HtmlInputRangeNumber, HtmlInputRangeReplacement, HtmlInputRangeSanitizationAction,
    HtmlInputRangeSanitizationCursor, HtmlInputRangeSanitizationReply,
    HtmlInputRangeSanitizationReplyError, HtmlInputRangeValue,
};
