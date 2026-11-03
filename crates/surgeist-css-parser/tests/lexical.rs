//! Specification-derived tests for the lexical definitions of CSS Syntax
//! Module Level 3 (pinned edition, 1 October 2026): the code-point classes of
//! section 4.2 and the checks of sections 4.3.8 to 4.3.11.
//!
//! Every expectation is written from the clause text. Where a test counts the
//! code points of a class over all of Unicode, the count is derived from the
//! ranges the clause lists, not from the implementation.

mod common;

use common::ranges::{ADMITTED, admitted};
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

/// How many scalar values the ranges of section 4.2 cover, counted by walking
/// them (none of the ranges holds a surrogate).
fn admitted_count() -> usize {
    ADMITTED
        .iter()
        .map(|&(low, high)| (low..=high).filter_map(char::from_u32).count())
        .sum()
}

#[test]
fn non_ascii_ident_code_point_is_exactly_the_ranges_of_section_4_2() {
    // The pinned edition's list: U+00B7; U+00C0 to U+00D6; U+00D8 to U+00F6;
    // U+00F8 to U+037D; U+037F to U+1FFF; U+200C; U+200D; U+203F; U+2040;
    // U+2070 to U+218F; U+2C00 to U+2FEF; U+3001 to U+D7FF; U+F900 to U+FDCF;
    // U+FDF0 to U+FFFD; and U+10000 and above, all inclusive. Adding the
    // lengths of those fifteen bullets gives 1,102,639 code points, and no
    // other scalar value is a member.
    assert_eq!(admitted_count(), 1_102_639);
    assert_eq!(count_where(is_non_ascii_ident_code_point), 1_102_639);
    for code in 0..=0x10_ffff_u32 {
        let Some(point) = char::from_u32(code) else {
            continue;
        };
        assert_eq!(
            is_non_ascii_ident_code_point(point),
            code >= 0x80 && admitted(point),
            "U+{code:04X}"
        );
    }
}

#[test]
fn non_ascii_ident_code_point_holds_at_both_edges_of_every_range_and_not_beside_them() {
    for &(low, high) in &ADMITTED {
        for edge in [low, high] {
            let point = char::from_u32(edge).unwrap();
            assert!(is_non_ascii_ident_code_point(point), "U+{edge:04X}");
        }
        // The scalar value on either side is a member only when another range
        // holds it (U+200C and U+200D, U+203F and U+2040, touch each other).
        let below = (0..low).rev().find_map(char::from_u32).unwrap();
        let held_below = ADMITTED
            .iter()
            .any(|&(l, h)| (l..=h).contains(&u32::from(below)));
        assert_eq!(
            is_non_ascii_ident_code_point(below),
            held_below && u32::from(below) >= 0x80,
            "U+{:04X}, below U+{low:04X}",
            u32::from(below)
        );
        if let Some(above) = (high + 1..=0x10_ffff).find_map(char::from_u32) {
            let held_above = ADMITTED
                .iter()
                .any(|&(l, h)| (l..=h).contains(&u32::from(above)));
            assert_eq!(
                is_non_ascii_ident_code_point(above),
                held_above,
                "U+{:04X}, above U+{high:04X}",
                u32::from(above)
            );
        }
    }
    for excluded in [
        '\u{80}', '\u{a0}', '\u{b6}', '\u{b8}', '\u{bf}', '\u{d7}', '\u{f7}', '\u{37e}',
        '\u{2000}', '\u{200b}', '\u{200e}', '\u{2028}', '\u{2029}', '\u{203e}', '\u{2041}',
        '\u{206f}', '\u{2190}', '\u{2603}', '\u{2bff}', '\u{2ff0}', '\u{3000}', '\u{e000}',
        '\u{f8ff}', '\u{fdd0}', '\u{fdef}', '\u{fffe}', '\u{ffff}', '\u{7f}', 'a',
    ] {
        assert!(
            !is_non_ascii_ident_code_point(excluded),
            "U+{:04X}",
            excluded as u32
        );
    }
    for member in [
        '\u{b7}',
        '\u{c0}',
        '\u{d6}',
        '\u{d8}',
        '\u{f6}',
        '\u{f8}',
        '\u{37d}',
        '\u{37f}',
        '\u{1fff}',
        '\u{200c}',
        '\u{200d}',
        '\u{203f}',
        '\u{2040}',
        '\u{2070}',
        '\u{218f}',
        '\u{2c00}',
        '\u{2fef}',
        '\u{3001}',
        '\u{d7ff}',
        '\u{f900}',
        '\u{fdcf}',
        '\u{fdf0}',
        '\u{fffd}',
        '\u{10000}',
        '\u{1f600}',
        '\u{10ffff}',
    ] {
        assert!(
            is_non_ascii_ident_code_point(member),
            "U+{:04X}",
            member as u32
        );
    }
}

#[test]
fn ident_start_is_a_letter_a_non_ascii_ident_code_point_or_low_line() {
    // 52 letters, the low line, and the admitted non-ASCII scalar values.
    assert_eq!(count_where(is_ident_start), 52 + 1 + 1_102_639);
    for member in ['a', 'Z', '_', '\u{b7}', 'é', '\u{10000}'] {
        assert!(is_ident_start(member), "{member:?}");
    }
    for other in [
        '-', '0', '9', ' ', '\\', '\n', '\0', '.', '#', '@', '\u{80}', '\u{a0}', '\u{d7}',
        '\u{2603}', '\u{fffe}',
    ] {
        assert!(!is_ident_start(other), "{other:?}");
    }
}

#[test]
fn ident_code_point_is_an_ident_start_a_digit_or_hyphen_minus() {
    assert_eq!(
        count_where(is_ident_code_point),
        52 + 1 + 10 + 1 + 1_102_639
    );
    for member in ['a', '_', '-', '0', '9', 'é', '\u{b7}'] {
        assert!(is_ident_code_point(member), "{member:?}");
    }
    for other in [
        ' ', '\\', '\n', '\0', '.', '#', '@', '+', '(', '"', '\u{80}', '\u{a0}', '\u{d7}',
        '\u{2603}', '\u{fffe}',
    ] {
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
    for second in ['a', 'Z', '_', 'é', '\u{b7}', '\u{10000}'] {
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
    for first in ['a', 'z', 'A', 'Z', '_', 'é', '\u{b7}', '\u{1f600}'] {
        assert!(would_start_ident(Some(first), None, None), "{first:?}");
        assert!(
            would_start_ident(Some(first), Some('\n'), Some(' ')),
            "{first:?}"
        );
    }
}

#[test]
fn ident_does_not_start_with_a_non_ascii_code_point_that_section_4_2_leaves_out() {
    for point in ['\u{80}', '\u{a0}', '\u{d7}', '\u{2603}', '\u{fffe}'] {
        assert!(
            !would_start_ident(Some(point), Some('a'), Some('a')),
            "{point:?}"
        );
        assert!(
            !would_start_ident(Some('-'), Some(point), Some('a')),
            "after a hyphen-minus: {point:?}"
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
