//! Text alignment, justification, wrapping, indentation, hyphenation limits and line-break input.

pub use crate::hyphenation::{HyphenateLimitLast, HyphenateLimitLines};
pub use crate::node_input::{
    InlineBreakOpportunity, LineBoxDirectionPolicy, LineBreakDisplay, LineBreakInput,
    LineBreakInputOf, TextAlign, TextAlignLast, TextGroupAlign, TextIndent, TextIndentOf,
    TextJustification, TextWrapStyle,
};
