//! Source-ordered range sanitization decisions; arithmetic, strings and admission stay caller-owned.

pub use crate::internal::elements::input_range::{
    HtmlInputRangeNumber, HtmlInputRangeReplacement, HtmlInputRangeSanitizationAction,
    HtmlInputRangeSanitizationCursor, HtmlInputRangeSanitizationReply,
    HtmlInputRangeSanitizationReplyError, HtmlInputRangeValue,
};
