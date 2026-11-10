//! The lexical definitions of CSS Syntax Module Level 3.
//!
//! This module holds, once, the code-point classes of section 4.2 and the
//! three-code-point checks of sections 4.3.8 to 4.3.11. Every tokenizer, the
//! serializer and every consumer that needs a lexical test without
//! tokenizing uses these functions, so no second definition can drift.
//!
//! All functions are `const fn`s over `char` and `Option<char>`. They describe
//! the code points the tokenizer sees *after* input preprocessing (section
//! 3.3): a carriage return, a form feed and a NUL never reach the tokenizer,
//! so [`is_newline`] is true only for U+000A LINE FEED. The checks take their
//! code points as `Option<char>`, where `None` is the specification's EOF code
//! point, the one that stands in for the end of the input stream.
//!
//! A caller that holds text before preprocessing preprocesses each code point
//! itself (U+0000 becomes U+FFFD; U+000D and U+000C become U+000A) before it
//! asks one of these questions. [`InputCursor`](crate::input::InputCursor)
//! does that as it decodes, and the stepwise tokenizer
//! ([`lexer::protocol`](crate::lexer::protocol)) assumes it. The definitions
//! exist once: a second predicate for unpreprocessed text would only be this one
//! behind a conversion.
//!
//! # The non-ASCII ident code point
//!
//! [`is_non_ascii_ident_code_point`] is the list of ranges in section 4.2 of the
//! pinned 1 October 2026 edition, which matches the non-ASCII code points that
//! HTML allows in valid custom element names. [`is_ident_start`],
//! [`is_ident_code_point`] and [`would_start_ident`] build on it, so a code
//! point outside the ranges (U+00A0, U+00D7, U+2603 or U+FFFE, for example) ends
//! an ident sequence, and the tokenizer reads it as a `<delim-token>`. The
//! serializer escapes the same code points.

/// Whether `code_point` is a digit: U+0030 DIGIT ZERO (0) to U+0039 DIGIT NINE
/// (9) inclusive (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_digit;
///
/// assert!(is_digit('7'));
/// assert!(!is_digit('a'));
/// ```
#[must_use]
pub const fn is_digit(code_point: char) -> bool {
    code_point.is_ascii_digit()
}

/// Whether `code_point` is a hex digit: a [digit](is_digit), U+0041 LATIN
/// CAPITAL LETTER A (A) to U+0046 (F) inclusive, or U+0061 LATIN SMALL LETTER A
/// (a) to U+0066 (f) inclusive (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_hex_digit;
///
/// assert!(is_hex_digit('F'));
/// assert!(!is_hex_digit('g'));
/// ```
#[must_use]
pub const fn is_hex_digit(code_point: char) -> bool {
    code_point.is_ascii_hexdigit()
}

/// The value, 0 to 9, of a [digit](is_digit), or `None` for any other code
/// point.
pub(crate) const fn digit_value(code_point: char) -> Option<u8> {
    if is_digit(code_point) {
        Some(code_point as u8 - b'0')
    } else {
        None
    }
}

/// The value, 0 to 15, of a [hex digit](is_hex_digit), or `None` for any other
/// code point.
pub(crate) const fn hex_digit_value(code_point: char) -> Option<u8> {
    if is_digit(code_point) {
        digit_value(code_point)
    } else if is_hex_digit(code_point) {
        Some((code_point as u8 | 0x20) - b'a' + 10)
    } else {
        None
    }
}

/// Whether `code_point` is an uppercase letter: U+0041 LATIN CAPITAL LETTER A
/// (A) to U+005A LATIN CAPITAL LETTER Z (Z) inclusive (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_uppercase_letter;
///
/// assert!(is_uppercase_letter('Q'));
/// assert!(!is_uppercase_letter('q'));
/// ```
#[must_use]
pub const fn is_uppercase_letter(code_point: char) -> bool {
    code_point.is_ascii_uppercase()
}

/// Whether `code_point` is a lowercase letter: U+0061 LATIN SMALL LETTER A (a)
/// to U+007A LATIN SMALL LETTER Z (z) inclusive (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_lowercase_letter;
///
/// assert!(is_lowercase_letter('q'));
/// assert!(!is_lowercase_letter('Q'));
/// ```
#[must_use]
pub const fn is_lowercase_letter(code_point: char) -> bool {
    code_point.is_ascii_lowercase()
}

/// Whether `code_point` is a letter: an [uppercase letter](is_uppercase_letter)
/// or a [lowercase letter](is_lowercase_letter) (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_letter;
///
/// assert!(is_letter('z'));
/// assert!(!is_letter('_'));
/// ```
#[must_use]
pub const fn is_letter(code_point: char) -> bool {
    is_uppercase_letter(code_point) || is_lowercase_letter(code_point)
}

/// Whether `code_point` is a non-ASCII ident code point (section 4.2): one of
/// U+00B7, U+00C0 to U+00D6, U+00D8 to U+00F6, U+00F8 to U+037D, U+037F to
/// U+1FFF, U+200C, U+200D, U+203F, U+2040, U+2070 to U+218F, U+2C00 to U+2FEF,
/// U+3001 to U+D7FF, U+F900 to U+FDCF, U+FDF0 to U+FFFD, or U+10000 and above.
/// All of the ranges are inclusive.
///
/// No ASCII code point is a member, and neither are the C1 controls, U+00A0
/// and every other code point outside those ranges, so none of them starts or
/// continues an ident sequence. See the
/// [module documentation](self#the-non-ascii-ident-code-point).
///
/// ```
/// use surgeist_css_parser::lexical::is_non_ascii_ident_code_point;
///
/// assert!(is_non_ascii_ident_code_point('é'));
/// assert!(is_non_ascii_ident_code_point('\u{b7}'));
/// assert!(is_non_ascii_ident_code_point('\u{1f600}'));
/// assert!(!is_non_ascii_ident_code_point('e'));
/// assert!(!is_non_ascii_ident_code_point('\u{a0}'));
/// assert!(!is_non_ascii_ident_code_point('\u{d7}'));
/// assert!(!is_non_ascii_ident_code_point('\u{2603}'));
/// assert!(!is_non_ascii_ident_code_point('\u{fffe}'));
/// ```
#[must_use]
pub const fn is_non_ascii_ident_code_point(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{b7}'
            | '\u{c0}'..='\u{d6}'
            | '\u{d8}'..='\u{f6}'
            | '\u{f8}'..='\u{37d}'
            | '\u{37f}'..='\u{1fff}'
            | '\u{200c}'
            | '\u{200d}'
            | '\u{203f}'
            | '\u{2040}'
            | '\u{2070}'..='\u{218f}'
            | '\u{2c00}'..='\u{2fef}'
            | '\u{3001}'..='\u{d7ff}'
            | '\u{f900}'..='\u{fdcf}'
            | '\u{fdf0}'..='\u{fffd}'
            | '\u{10000}'..='\u{10ffff}'
    )
}

/// Whether `code_point` is an ident-start code point: a [letter](is_letter),
/// a [non-ASCII ident code point](is_non_ascii_ident_code_point), or U+005F LOW
/// LINE (_) (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_ident_start;
///
/// assert!(is_ident_start('_'));
/// assert!(!is_ident_start('-'));
/// assert!(!is_ident_start('3'));
/// ```
#[must_use]
pub const fn is_ident_start(code_point: char) -> bool {
    is_letter(code_point) || is_non_ascii_ident_code_point(code_point) || code_point == '_'
}

/// Whether `code_point` is an ident code point: an
/// [ident-start code point](is_ident_start), a [digit](is_digit), or U+002D
/// HYPHEN-MINUS (-) (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_ident_code_point;
///
/// assert!(is_ident_code_point('-'));
/// assert!(is_ident_code_point('3'));
/// assert!(!is_ident_code_point(' '));
/// ```
#[must_use]
pub const fn is_ident_code_point(code_point: char) -> bool {
    is_ident_start(code_point) || is_digit(code_point) || code_point == '-'
}

/// Whether `code_point` is a non-printable code point: U+0000 NULL to U+0008
/// BACKSPACE inclusive, U+000B LINE TABULATION, U+000E SHIFT OUT to U+001F
/// INFORMATION SEPARATOR ONE inclusive, or U+007F DELETE (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_non_printable;
///
/// assert!(is_non_printable('\u{7f}'));
/// assert!(!is_non_printable('\t'));
/// ```
#[must_use]
pub const fn is_non_printable(code_point: char) -> bool {
    matches!(code_point, '\u{0}'..='\u{8}' | '\u{b}' | '\u{e}'..='\u{1f}' | '\u{7f}')
}

/// Whether `code_point` is a newline: U+000A LINE FEED (section 4.2).
///
/// U+000D CARRIAGE RETURN and U+000C FORM FEED are not newlines. Input
/// preprocessing converts both to U+000A before the tokenizer reads them.
///
/// ```
/// use surgeist_css_parser::lexical::is_newline;
///
/// assert!(is_newline('\n'));
/// assert!(!is_newline('\r'));
/// ```
#[must_use]
pub const fn is_newline(code_point: char) -> bool {
    code_point == '\n'
}

/// Whether `code_point` is whitespace: a [newline](is_newline), U+0009
/// CHARACTER TABULATION, or U+0020 SPACE (section 4.2).
///
/// ```
/// use surgeist_css_parser::lexical::is_whitespace;
///
/// assert!(is_whitespace('\t'));
/// assert!(!is_whitespace('\u{a0}'));
/// ```
#[must_use]
pub const fn is_whitespace(code_point: char) -> bool {
    is_newline(code_point) || code_point == '\t' || code_point == ' '
}

/// Checks whether two code points are a valid escape (section 4.3.8).
///
/// `first` and `second` are the two code points in stream order; `None` is the
/// EOF code point. The result is false when `first` is not U+005C REVERSE
/// SOLIDUS (\\) and false when `second` is a [newline](is_newline). It is true
/// otherwise, including when `second` is EOF: a backslash that ends the input
/// is a valid escape, which the "consume an escaped code point" algorithm
/// (section 4.3.7) answers with U+FFFD and a parse error.
///
/// ```
/// use surgeist_css_parser::lexical::valid_escape;
///
/// assert!(valid_escape(Some('\\'), Some('a')));
/// assert!(valid_escape(Some('\\'), None));
/// assert!(!valid_escape(Some('\\'), Some('\n')));
/// assert!(!valid_escape(Some('a'), Some('b')));
/// ```
#[must_use]
pub const fn valid_escape(first: Option<char>, second: Option<char>) -> bool {
    match (first, second) {
        (Some('\\'), Some(second)) => !is_newline(second),
        (Some('\\'), None) => true,
        _ => false,
    }
}

/// Checks whether three code points would start an ident sequence (section
/// 4.3.9).
///
/// The arguments are the three code points in stream order; `None` is the EOF
/// code point. The first one decides which rule applies:
///
/// - U+002D HYPHEN-MINUS (-): true when the second is an
///   [ident-start code point](is_ident_start) or another hyphen-minus, or when
///   the second and third are a [valid escape](valid_escape);
/// - an [ident-start code point](is_ident_start): true;
/// - U+005C REVERSE SOLIDUS (\\): true when the first and second are a
///   [valid escape](valid_escape);
/// - anything else, including EOF: false.
///
/// ```
/// use surgeist_css_parser::lexical::would_start_ident;
///
/// assert!(would_start_ident(Some('-'), Some('-'), None));
/// assert!(would_start_ident(Some('-'), Some('\\'), Some('x')));
/// assert!(would_start_ident(Some('a'), None, None));
/// assert!(!would_start_ident(Some('-'), Some('3'), None));
/// ```
#[must_use]
pub const fn would_start_ident(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> bool {
    match first {
        Some('-') => {
            let second_starts = match second {
                Some(second) => is_ident_start(second) || second == '-',
                None => false,
            };
            second_starts || valid_escape(second, third)
        }
        Some('\\') => valid_escape(first, second),
        Some(first) => is_ident_start(first),
        None => false,
    }
}

/// Checks whether three code points would start a number (section 4.3.10).
///
/// The arguments are the three code points in stream order; `None` is the EOF
/// code point. The first one decides which rule applies:
///
/// - U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-): true when the second is a
///   [digit](is_digit), or when the second is U+002E FULL STOP (.) and the third
///   is a digit;
/// - U+002E FULL STOP (.): true when the second is a digit;
/// - a [digit](is_digit): true;
/// - anything else, including EOF: false.
///
/// ```
/// use surgeist_css_parser::lexical::would_start_number;
///
/// assert!(would_start_number(Some('-'), Some('.'), Some('5')));
/// assert!(would_start_number(Some('7'), None, None));
/// assert!(!would_start_number(Some('+'), Some('.'), Some('x')));
/// ```
#[must_use]
pub const fn would_start_number(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> bool {
    match (first, second, third) {
        (Some('+' | '-'), Some(second), _) if is_digit(second) => true,
        (Some('+' | '-'), Some('.'), Some(third)) => is_digit(third),
        (Some('.'), Some(second), _) => is_digit(second),
        (Some(first), _, _) => is_digit(first),
        (None, _, _) => false,
    }
}

/// Checks whether three code points would start a unicode-range (section
/// 4.3.11).
///
/// The arguments are the three code points in stream order; `None` is the EOF
/// code point. The result is true when the first is U+0055 LATIN CAPITAL LETTER
/// U (U) or U+0075 LATIN SMALL LETTER U (u), the second is U+002B PLUS SIGN
/// (+), and the third is U+003F QUESTION MARK (?) or a
/// [hex digit](is_hex_digit).
///
/// ```
/// use surgeist_css_parser::lexical::would_start_unicode_range;
///
/// assert!(would_start_unicode_range(Some('U'), Some('+'), Some('?')));
/// assert!(would_start_unicode_range(Some('u'), Some('+'), Some('0')));
/// assert!(!would_start_unicode_range(Some('u'), Some('+'), Some('g')));
/// ```
#[must_use]
pub const fn would_start_unicode_range(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> bool {
    match (first, second, third) {
        (Some('U' | 'u'), Some('+'), Some(third)) => third == '?' || is_hex_digit(third),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values are those of the digits of section 4.2, which the standard
    /// library's radix conversion also gives; every Unicode scalar is asked.
    #[test]
    fn digit_values_agree_with_the_digit_classes_for_every_scalar() {
        for code in 0..=0x0010_FFFF_u32 {
            let Some(point) = char::from_u32(code) else {
                continue;
            };
            let decimal = point.to_digit(10).map(|value| value as u8);
            let hexadecimal = point.to_digit(16).map(|value| value as u8);
            assert_eq!(digit_value(point), decimal, "{point:?}");
            assert_eq!(hex_digit_value(point), hexadecimal, "{point:?}");
            assert_eq!(digit_value(point).is_some(), is_digit(point));
            assert_eq!(hex_digit_value(point).is_some(), is_hex_digit(point));
        }
    }
}
