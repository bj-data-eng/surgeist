//! Text alignment, justification, wrapping, indentation, hyphenation limits and line-break input.
//!
//! [`TextAlign`], [`TextAlignLast`], [`TextGroupAlign`] and [`TextJustification`] are the normalized alignment and
//! expansion inputs, [`TextWrapStyle`] the preference for selecting legal line breaks, and [`TextIndentOf`] the
//! computed signed indentation. [`HyphenateLimitLines`] and [`HyphenateLimitLast`] are the computed hyphenation
//! limits supplied by the owning block, and [`LineBoxDirectionPolicy`] the direction used by line-box alignment and
//! indentation. [`LineBreakInputOf`] is the input of a line break ([`LineBreakDisplay`]), and
//! [`InlineBreakOpportunity`] the normalized break permission of a box-only inline participant.

pub use crate::hyphenation::{HyphenateLimitLast, HyphenateLimitLines};
pub use crate::node_input::{
    InlineBreakOpportunity, LineBoxDirectionPolicy, LineBreakDisplay, LineBreakInput,
    LineBreakInputOf, TextAlign, TextAlignLast, TextGroupAlign, TextIndent, TextIndentOf,
    TextJustification, TextWrapStyle,
};
