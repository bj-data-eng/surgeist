//! Bounded Input attribute recognition, value conversion and range sanitization.
//!
//! Callers admit conversion work and apply the result. Range sanitization
//! also leaves its arithmetic buffers and strings with the caller.

pub use crate::internal::elements::input::input_colorspace_with;
pub use crate::internal::elements::input_range::{
    HtmlInputRangeNumber, HtmlInputRangeReplacement, HtmlInputRangeSanitizationAction,
    HtmlInputRangeSanitizationCursor, HtmlInputRangeSanitizationReply,
    HtmlInputRangeSanitizationReplyError, HtmlInputRangeValue,
};
pub use crate::internal::elements::input_value_conversion::{
    HtmlInputDateMilliseconds, HtmlInputDateRead, HtmlInputValueAssignment,
    HtmlInputValueConversionError, input_value_as_date, input_value_as_number,
    input_value_from_date, input_value_from_number,
};
