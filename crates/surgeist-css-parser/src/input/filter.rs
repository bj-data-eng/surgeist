//! Section 3.3 over text: the "filter code points" algorithm.
//!
//! This module also holds the one definition of what section 3.3 does to a
//! code point, which [`InputCursor`](super::InputCursor), the free-running
//! lexer and [`filter_code_points`] each apply in their own way (as a stream,
//! over bytes of text and over a whole string).

use std::borrow::Cow;

/// What section 3.3 makes of `value` by itself: a U+000D CARRIAGE RETURN and a
/// U+000C FORM FEED become U+000A LINE FEED, and a U+0000 NULL becomes U+FFFD
/// REPLACEMENT CHARACTER. Every other code point is itself.
///
/// A carriage return that is followed by a line feed forms one line feed with
/// it; see [`absorbs_line_feed`]. The replacement of surrogate code points does
/// not apply to a `char`.
pub(crate) const fn filter_code_point(value: char) -> char {
    match value {
        '\0' => '\u{fffd}',
        '\r' | '\u{c}' => '\n',
        _ => value,
    }
}

/// Whether section 3.3 changes `value`, so that a text with such a code point
/// is not yet preprocessed.
pub(crate) const fn is_filtered(value: char) -> bool {
    filter_code_point(value) != value
}

/// Whether `value`, followed by a U+000A LINE FEED, forms one line feed with
/// it: a U+000D CARRIAGE RETURN does.
pub(crate) const fn absorbs_line_feed(value: char) -> bool {
    value == '\r'
}

/// Filters `text` as section 3.3 says: each U+000D CARRIAGE RETURN, each
/// U+000C FORM FEED and each U+000D followed by U+000A LINE FEED becomes one
/// U+000A, and each U+0000 NULL becomes U+FFFD REPLACEMENT CHARACTER.
///
/// The specification also replaces surrogate code points with U+FFFD, but a
/// Rust string cannot hold one: preprocess UTF-16 units that may contain a
/// lone surrogate with [`preprocess_utf16`](super::preprocess_utf16) instead.
/// Text that has nothing to filter is returned unchanged and borrowed.
///
/// ```
/// use surgeist_css_parser::input::filter_code_points;
///
/// assert_eq!(filter_code_points("a\r\nb\rc\x0cd\0"), "a\nb\nc\nd\u{fffd}");
/// assert!(matches!(filter_code_points("a\nb"), std::borrow::Cow::Borrowed("a\nb")));
/// ```
#[must_use]
pub fn filter_code_points(text: &str) -> Cow<'_, str> {
    // Only ASCII code points are filtered, and a byte of a multi-byte sequence
    // is never ASCII, so the bytes of the text decide.
    if !text.bytes().any(|byte| is_filtered(char::from(byte))) {
        return Cow::Borrowed(text);
    }
    let mut filtered = String::with_capacity(text.len());
    let mut points = text.chars().peekable();
    while let Some(point) = points.next() {
        if absorbs_line_feed(point) && points.peek() == Some(&'\n') {
            points.next();
        }
        filtered.push(filter_code_point(point));
    }
    Cow::Owned(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_per_code_point_rules_are_those_of_section_3_3() {
        assert_eq!(filter_code_point('\0'), '\u{fffd}');
        assert_eq!(filter_code_point('\r'), '\n');
        assert_eq!(filter_code_point('\u{c}'), '\n');
        for point in ['\n', 'a', ' ', '\u{fffd}', '\u{85}', '\u{2028}'] {
            assert_eq!(filter_code_point(point), point);
            assert!(!is_filtered(point));
        }
        assert!(is_filtered('\0') && is_filtered('\r') && is_filtered('\u{c}'));
        assert!(absorbs_line_feed('\r'));
        assert!(!absorbs_line_feed('\u{c}') && !absorbs_line_feed('\n'));
    }
}
