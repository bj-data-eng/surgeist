//! Section 3.3 over text: the "filter code points" algorithm.

use std::borrow::Cow;

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
    if !text.bytes().any(|byte| matches!(byte, 0x00 | 0x0C | 0x0D)) {
        return Cow::Borrowed(text);
    }
    let mut filtered = String::with_capacity(text.len());
    let mut points = text.chars().peekable();
    while let Some(point) = points.next() {
        match point {
            '\r' => {
                if points.peek() == Some(&'\n') {
                    points.next();
                }
                filtered.push('\n');
            }
            '\u{c}' => filtered.push('\n'),
            '\0' => filtered.push('\u{fffd}'),
            point => filtered.push(point),
        }
    }
    Cow::Owned(filtered)
}
