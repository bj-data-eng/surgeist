//! The `unicode-range` token of CSS Syntax Module Level 3.
//!
//! Section 4.3.14 consumes a `<unicode-range-token>` only when the tokenizer is
//! told unicode ranges are allowed (section 4.3.1); the `unicode-range`
//! descriptor of `@font-face` is the one place that does. This module holds the
//! token's value, [`CodePointRange`], and the algorithm of section 5.5.11,
//! [`consume_value`], which tokenizes the text of such a descriptor with
//! unicode ranges allowed. The tokenizer does not judge whether the range makes
//! sense: a bound above U+10FFFF, or an end before the start, is a value of the
//! token, and the descriptor's grammar decides what to do with it.

use crate::document::BuildError;
use crate::lexer::Options;
use crate::limits::{DepthExcess, Limits};
use crate::{Document, Offset};

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

/// Consumes the value of a `unicode-range` descriptor (section 5.5.11): the text
/// is tokenized with unicode ranges allowed, and the list of component values
/// of the result is the root list of the returned document.
///
/// Section 5.5.6 step 8 runs this over the original text of a
/// `unicode-range` declaration's value (see
/// [`Declaration::original_text`](crate::grammar::Declaration::original_text)),
/// where the ordinary tokenization of that text would read `U+26` as an ident
/// and a number. `origin` is the offset of `text` in the text it was cut from,
/// so the offsets the document reports are positions in that larger text. The
/// document is of the caller's payload type `P`; every node carries none.
///
/// The descriptor's grammar decides what a range is worth: this function only
/// tokenizes (section 4.3.14).
///
/// # Errors
///
/// Fails when `limits` do not admit the text, as [`Document::parse`] does.
///
/// ```
/// use surgeist_css_parser::limits::Limits;
/// use surgeist_css_parser::unicode_range::{CodePointRange, consume_value};
/// use surgeist_css_parser::{Document, Offset, Token};
///
/// // `U+4??, U+26` as the value of the descriptor, which starts at byte 40.
/// let document: Document = consume_value("U+4??, U+26", Offset::new(40), Limits::default())?;
/// let mut stream = document.root();
/// assert_eq!(
///     stream.consume_token().unwrap(),
///     Token::UnicodeRange(CodePointRange::new(0x400, 0x4ff))
/// );
/// assert_eq!(stream.offset(), Some(Offset::new(45)));
/// stream.consume_comma().unwrap();
/// assert_eq!(
///     stream.consume_after_whitespace().unwrap(),
///     Token::UnicodeRange(CodePointRange::new(0x26, 0x26))
/// );
/// # Ok::<(), surgeist_css_parser::document::BuildError>(())
/// ```
pub fn consume_value<'s, P>(
    text: &'s str,
    origin: Offset,
    limits: Limits,
) -> Result<Document<'s, P>, BuildError> {
    let options = Options::default().unicode_ranges(true).origin(origin);
    Document::parse(text, &options, limits, DepthExcess::Fail)
}
