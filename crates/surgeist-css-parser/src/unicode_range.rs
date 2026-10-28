//! The `unicode-range` token of CSS Syntax Module Level 3.
//!
//! Section 4.3.14 consumes a `<unicode-range-token>` only when the tokenizer is
//! told unicode ranges are allowed (section 4.3.1); the `unicode-range`
//! descriptor of `@font-face` is the one place that does. This module holds the
//! token's value, [`CodePointRange`]. The tokenizer does not judge whether the
//! range makes sense: a bound above U+10FFFF, or an end before the start, is a
//! value of the token, and the descriptor's grammar decides what to do with it.

/// The starting and ending code point of a `<unicode-range-token>`.
///
/// The token represents the inclusive range from [`start`](Self::start) to
/// [`end`](Self::end); when the end is before the start it represents an empty
/// range. A bound is what up to six hexadecimal digits spell, so it can exceed
/// U+10FFFF, the maximum allowed code point.
///
/// ```
/// use surgeist_css_parser::unicode_range::CodePointRange;
///
/// // `U+4??` spells the range U+400 to U+4FF.
/// let range = CodePointRange::new(0x400, 0x4ff);
/// assert_eq!((range.start(), range.end()), (0x400, 0x4ff));
/// assert!(!range.is_empty());
/// assert!(CodePointRange::new(0x20, 0x10).is_empty());
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodePointRange {
    start: u32,
    end: u32,
}

impl CodePointRange {
    /// Creates the range from `start` to `end`, both inclusive.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Returns the first code point of the range.
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Returns the last code point of the range.
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }

    /// Returns whether the range holds no code point, because its end is before
    /// its start.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.end < self.start
    }
}
