//! Specification-derived tests for the lexical definitions of CSS Syntax
//! Module Level 3 (pinned edition, 1 October 2026): the code-point classes of
//! section 4.2 and the checks of sections 4.3.8 to 4.3.11.
//!
//! Every expectation is written from the clause text. Where a test counts the
//! code points of a class over all of Unicode, the count is derived from the
//! ranges the clause lists, not from the implementation.

use surgeist_css_parser::lexical::{
    is_digit, is_hex_digit, is_ident_code_point, is_ident_start, is_letter, is_lowercase_letter,
    is_newline, is_non_ascii_ident_code_point, is_non_printable, is_uppercase_letter,
    is_whitespace, valid_escape, would_start_ident, would_start_number, would_start_unicode_range,
};

/// How many Unicode scalar values (every `char`) satisfy `predicate`.
fn count_where(predicate: impl Fn(char) -> bool) -> usize {
    (0..=0x10_FFFFu32)
        .filter_map(char::from_u32)
        .filter(|&c| predicate(c))
        .count()
}

/// Every character of `members`, and no other scalar value, satisfies `predicate`.
fn assert_exactly(predicate: impl Fn(char) -> bool, members: &[char]) {
    for &member in members {
        assert!(
            predicate(member),
            "U+{:04X} should be a member",
            member as u32
        );
    }
    assert_eq!(
        count_where(&predicate),
        members.len(),
        "the class has members beyond the {} listed",
        members.len()
    );
}

// ---- 4.2 code-point classes ------------------------------------------------

#[test]
fn digit_is_exactly_u0030_to_u0039_inclusive() {
    assert_exactly(
        is_digit,
        &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
    );
    assert!(!is_digit('/'), "U+002F is just below U+0030");
    assert!(!is_digit(':'), "U+003A is just above U+0039");
}

#[test]
fn digit_excludes_non_ascii_decimal_digits() {
    assert!(!is_digit('\u{663}'), "ARABIC-INDIC DIGIT THREE");
    assert!(!is_digit('\u{ff13}'), "FULLWIDTH DIGIT THREE");
}

#[test]
fn hex_digit_is_digit_or_a_to_f_in_either_case() {
    let mut members: Vec<char> = ('0'..='9').collect();
    members.extend('A'..='F');
    members.extend('a'..='f');
    assert_exactly(is_hex_digit, &members);
    for edge in ['@', 'G', '`', 'g', '/', ':'] {
        assert!(!is_hex_digit(edge), "{edge:?} is adjacent to a range end");
    }
}

#[test]
fn uppercase_letter_is_exactly_u0041_to_u005a_inclusive() {
    let members: Vec<char> = ('A'..='Z').collect();
    assert_exactly(is_uppercase_letter, &members);
    assert!(!is_uppercase_letter('@') && !is_uppercase_letter('['));
}

#[test]
fn lowercase_letter_is_exactly_u0061_to_u007a_inclusive() {
    let members: Vec<char> = ('a'..='z').collect();
    assert_exactly(is_lowercase_letter, &members);
    assert!(!is_lowercase_letter('`') && !is_lowercase_letter('{'));
}

#[test]
fn letter_is_an_uppercase_or_a_lowercase_letter_and_nothing_else() {
    let members: Vec<char> = ('A'..='Z').chain('a'..='z').collect();
    assert_exactly(is_letter, &members);
}

#[test]
fn non_ascii_ident_code_point_is_every_code_point_from_u0080_under_the_2021_rule() {
    // 24 December 2021 CRD, section 4.2: a "non-ASCII code point" is a code
    // point with a value equal to or greater than U+0080. The count is the
    // number of Unicode scalar values from U+0080 up: 0x110000 code points,
    // less the 0x800 surrogates, less the 0x80 ASCII code points.
    assert_eq!(
        count_where(is_non_ascii_ident_code_point),
        0x11_0000 - 0x800 - 0x80
    );
    for member in [
        '\u{80}',
        '\u{9f}',
        '\u{a0}',
        '\u{d7}',
        '\u{2603}',
        '\u{fffe}',
        '\u{10ffff}',
    ] {
        assert!(
            is_non_ascii_ident_code_point(member),
            "U+{:04X}",
            member as u32
        );
    }
    assert!(!is_non_ascii_ident_code_point('\u{7f}'));
    assert!(!is_non_ascii_ident_code_point('a'));
}

#[test]
fn ident_start_is_a_letter_a_non_ascii_ident_code_point_or_low_line() {
    // 52 letters, the low line, and every non-ASCII scalar value.
    assert_eq!(
        count_where(is_ident_start),
        52 + 1 + (0x11_0000 - 0x800 - 0x80)
    );
    for member in ['a', 'Z', '_', '\u{80}', 'é', '\u{10000}'] {
        assert!(is_ident_start(member), "{member:?}");
    }
    for other in ['-', '0', '9', ' ', '\\', '\n', '\0', '.', '#', '@'] {
        assert!(!is_ident_start(other), "{other:?}");
    }
}

#[test]
fn ident_code_point_is_an_ident_start_a_digit_or_hyphen_minus() {
    assert_eq!(
        count_where(is_ident_code_point),
        52 + 1 + 10 + 1 + (0x11_0000 - 0x800 - 0x80)
    );
    for member in ['a', '_', '-', '0', '9', 'é'] {
        assert!(is_ident_code_point(member), "{member:?}");
    }
    for other in [' ', '\\', '\n', '\0', '.', '#', '@', '+', '(', '"'] {
        assert!(!is_ident_code_point(other), "{other:?}");
    }
}

#[test]
fn non_printable_is_u0000_to_u0008_u000b_u000e_to_u001f_and_u007f() {
    // Nine, one, eighteen and one code points.
    let mut members: Vec<char> = ('\u{0}'..='\u{8}').collect();
    members.push('\u{b}');
    members.extend('\u{e}'..='\u{1f}');
    members.push('\u{7f}');
    assert_eq!(members.len(), 9 + 1 + 18 + 1);
    assert_exactly(is_non_printable, &members);
}

#[test]
fn non_printable_excludes_the_control_code_points_that_are_whitespace_or_form_feed() {
    for other in ['\t', '\n', '\u{c}', '\r', ' ', '\u{80}', '\u{a0}'] {
        assert!(!is_non_printable(other), "U+{:04X}", other as u32);
    }
}

#[test]
fn newline_is_only_u000a_because_preprocessing_converts_cr_and_ff() {
    assert_exactly(is_newline, &['\n']);
    assert!(!is_newline('\r'));
    assert!(!is_newline('\u{c}'));
}

#[test]
fn whitespace_is_newline_tab_and_space() {
    assert_exactly(is_whitespace, &['\n', '\t', ' ']);
    for other in ['\r', '\u{c}', '\u{a0}', '\u{2003}', '\u{b}'] {
        assert!(!is_whitespace(other), "U+{:04X}", other as u32);
    }
}

// ---- 4.3.8 check if two code points are a valid escape ---------------------

#[test]
fn escape_is_valid_when_a_backslash_is_followed_by_a_non_newline() {
    for second in ['a', '0', ' ', '\\', '\t', '"', '\u{2603}', '\0', '\u{fffd}'] {
        assert!(valid_escape(Some('\\'), Some(second)), "second {second:?}");
    }
}

#[test]
fn escape_is_invalid_when_the_backslash_is_followed_by_a_newline() {
    assert!(!valid_escape(Some('\\'), Some('\n')));
}

#[test]
fn escape_is_valid_when_the_backslash_is_followed_by_eof() {
    // 4.3.8 returns true unless the second code point is a newline, and EOF
    // is not a newline. The "consume an escaped code point" algorithm
    // (4.3.7) has an EOF branch for exactly this case. The "Changes from the
    // 20 February 2014 Candidate Recommendation" entry says a final backslash
    // is "not a valid escape", which contradicts the normative text; the
    // normative text governs.
    assert!(valid_escape(Some('\\'), None));
}

#[test]
fn escape_is_invalid_when_the_first_code_point_is_not_a_backslash() {
    for first in [Some('a'), Some('/'), Some('\n'), Some(' '), Some('-'), None] {
        for second in [Some('a'), Some('\n'), Some('\\'), None] {
            assert!(!valid_escape(first, second), "{first:?} {second:?}");
        }
    }
}

#[test]
fn escape_treats_carriage_return_and_form_feed_as_ordinary_second_code_points() {
    // They are not newlines (4.2); preprocessing removes them before this
    // check can see them.
    assert!(valid_escape(Some('\\'), Some('\r')));
    assert!(valid_escape(Some('\\'), Some('\u{c}')));
}

// ---- 4.3.9 check if three code points would start an ident sequence --------

#[test]
fn ident_starts_after_hyphen_minus_when_the_second_is_an_ident_start() {
    for second in ['a', 'Z', '_', 'é', '\u{80}', '\u{10000}'] {
        assert!(
            would_start_ident(Some('-'), Some(second), None),
            "second {second:?}"
        );
        // The third code point is not consulted on this branch.
        assert!(would_start_ident(Some('-'), Some(second), Some('3')));
    }
}

#[test]
fn ident_starts_after_hyphen_minus_when_the_second_is_another_hyphen_minus() {
    assert!(would_start_ident(Some('-'), Some('-'), None));
    assert!(would_start_ident(Some('-'), Some('-'), Some('-')));
    assert!(would_start_ident(Some('-'), Some('-'), Some('3')));
}

#[test]
fn ident_starts_after_hyphen_minus_when_the_second_and_third_are_a_valid_escape() {
    assert!(would_start_ident(Some('-'), Some('\\'), Some('x')));
    assert!(would_start_ident(Some('-'), Some('\\'), Some('\\')));
    // The third is EOF: a trailing backslash is a valid escape (4.3.8).
    assert!(would_start_ident(Some('-'), Some('\\'), None));
}

#[test]
fn ident_does_not_start_after_hyphen_minus_with_a_backslash_and_a_newline() {
    assert!(!would_start_ident(Some('-'), Some('\\'), Some('\n')));
}

#[test]
fn ident_does_not_start_after_hyphen_minus_followed_by_anything_else() {
    for second in [
        Some('0'),
        Some('9'),
        Some('.'),
        Some('+'),
        Some(' '),
        Some('\n'),
        Some('('),
        None,
    ] {
        assert!(
            !would_start_ident(Some('-'), second, Some('a')),
            "second {second:?}"
        );
    }
}

#[test]
fn ident_starts_with_an_ident_start_code_point_whatever_follows() {
    for first in ['a', 'z', 'A', 'Z', '_', 'é', '\u{2603}'] {
        assert!(would_start_ident(Some(first), None, None), "{first:?}");
        assert!(
            would_start_ident(Some(first), Some('\n'), Some(' ')),
            "{first:?}"
        );
    }
}

#[test]
fn ident_starts_with_a_backslash_when_the_first_two_are_a_valid_escape() {
    assert!(would_start_ident(Some('\\'), Some('a'), None));
    assert!(would_start_ident(Some('\\'), Some('\\'), None));
    assert!(would_start_ident(Some('\\'), None, None));
    // The third code point is not consulted.
    assert!(would_start_ident(Some('\\'), Some('a'), Some('\n')));
}

#[test]
fn ident_does_not_start_with_a_backslash_before_a_newline() {
    assert!(!would_start_ident(Some('\\'), Some('\n'), Some('a')));
}

#[test]
fn ident_does_not_start_with_anything_else() {
    for first in [
        Some('0'),
        Some('9'),
        Some('.'),
        Some('+'),
        Some('#'),
        Some('@'),
        Some(' '),
        Some('\n'),
        Some('('),
        Some('"'),
        None,
    ] {
        assert!(
            !would_start_ident(first, Some('a'), Some('a')),
            "first {first:?}"
        );
    }
}

// ---- 4.3.10 check if three code points would start a number ----------------

#[test]
fn number_starts_with_a_sign_followed_by_a_digit() {
    for sign in ['+', '-'] {
        for digit in ['0', '5', '9'] {
            assert!(would_start_number(Some(sign), Some(digit), None));
            assert!(would_start_number(Some(sign), Some(digit), Some('x')));
        }
    }
}

#[test]
fn number_starts_with_a_sign_a_full_stop_and_a_digit() {
    for sign in ['+', '-'] {
        assert!(would_start_number(Some(sign), Some('.'), Some('0')));
        assert!(would_start_number(Some(sign), Some('.'), Some('9')));
    }
}

#[test]
fn number_does_not_start_with_a_sign_a_full_stop_and_a_non_digit() {
    for sign in ['+', '-'] {
        for third in [Some('a'), Some('.'), Some(' '), Some('+'), None] {
            assert!(
                !would_start_number(Some(sign), Some('.'), third),
                "sign {sign:?} third {third:?}"
            );
        }
    }
}

#[test]
fn number_does_not_start_with_a_sign_followed_by_anything_else() {
    for sign in ['+', '-'] {
        for second in [Some('a'), Some('+'), Some('-'), Some(' '), Some('e'), None] {
            assert!(
                !would_start_number(Some(sign), second, Some('5')),
                "sign {sign:?} second {second:?}"
            );
        }
    }
}

#[test]
fn number_starts_with_a_full_stop_followed_by_a_digit() {
    assert!(would_start_number(Some('.'), Some('0'), None));
    assert!(would_start_number(Some('.'), Some('9'), Some('x')));
}

#[test]
fn number_does_not_start_with_a_full_stop_followed_by_a_non_digit() {
    // The third code point is not consulted.
    for second in [Some('a'), Some('.'), Some('-'), Some(' '), None] {
        assert!(
            !would_start_number(Some('.'), second, Some('5')),
            "second {second:?}"
        );
    }
}

#[test]
fn number_starts_with_a_digit_whatever_follows() {
    for digit in ['0', '1', '9'] {
        assert!(would_start_number(Some(digit), None, None));
        assert!(would_start_number(Some(digit), Some('x'), Some('y')));
    }
}

#[test]
fn number_does_not_start_with_anything_else() {
    for first in [
        Some('a'),
        Some('e'),
        Some('#'),
        Some(' '),
        Some('\\'),
        Some('\u{663}'),
        Some('\u{ff13}'),
        None,
    ] {
        assert!(
            !would_start_number(first, Some('5'), Some('5')),
            "first {first:?}"
        );
    }
}

// ---- 4.3.11 check if three code points would start a unicode-range ---------

#[test]
fn unicode_range_starts_with_u_plus_and_a_question_mark_or_hex_digit() {
    for first in ['U', 'u'] {
        for third in ['?', '0', '9', 'A', 'F', 'a', 'f'] {
            assert!(
                would_start_unicode_range(Some(first), Some('+'), Some(third)),
                "{first:?}+{third:?}"
            );
        }
    }
}

#[test]
fn unicode_range_does_not_start_when_the_first_code_point_is_not_u() {
    for first in [Some('V'), Some('t'), Some('+'), Some('0'), Some(' '), None] {
        assert!(
            !would_start_unicode_range(first, Some('+'), Some('0')),
            "first {first:?}"
        );
    }
}

#[test]
fn unicode_range_does_not_start_when_the_second_code_point_is_not_plus_sign() {
    for second in [Some('-'), Some('0'), Some('u'), Some(' '), Some('?'), None] {
        assert!(
            !would_start_unicode_range(Some('u'), second, Some('0')),
            "second {second:?}"
        );
    }
}

#[test]
fn unicode_range_does_not_start_when_the_third_code_point_is_neither_question_mark_nor_hex() {
    for third in [
        Some('g'),
        Some('G'),
        Some('z'),
        Some('-'),
        Some('+'),
        Some(' '),
        None,
    ] {
        assert!(
            !would_start_unicode_range(Some('U'), Some('+'), third),
            "third {third:?}"
        );
    }
}

// ---- evaluation in constants -----------------------------------------------

#[test]
fn every_check_is_usable_in_a_constant_context() {
    const {
        assert!(is_digit('5'));
        assert!(is_hex_digit('f'));
        assert!(is_uppercase_letter('Q') && is_lowercase_letter('q') && is_letter('q'));
        assert!(is_non_ascii_ident_code_point('\u{e9}'));
        assert!(is_ident_start('_'));
        assert!(is_ident_code_point('-'));
        assert!(is_non_printable('\u{7f}'));
        assert!(is_newline('\n'));
        assert!(is_whitespace(' '));
        assert!(valid_escape(Some('\\'), None));
        assert!(would_start_ident(Some('-'), Some('-'), None));
        assert!(would_start_number(Some('.'), Some('5'), None));
        assert!(would_start_unicode_range(Some('u'), Some('+'), Some('?')));
    }
}
