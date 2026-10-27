//! Tests for `match_ignore_ascii_case!`: ASCII case-insensitive matching of a
//! string against lowercase literal arms. The expectations come from the
//! macro's documented contract, which is `str::eq_ignore_ascii_case` against
//! each arm in order.

use std::cell::Cell;

use surgeist_css_parser::match_ignore_ascii_case;

fn keyword(ident: &str) -> u32 {
    match_ignore_ascii_case! { ident,
        "repeat-x" => 1,
        "repeat-y" => 2,
        "no-repeat" => 3,
        _ => 0,
    }
}

#[test]
fn a_literal_matches_in_any_ascii_case() {
    for spelling in ["repeat-x", "REPEAT-X", "Repeat-X", "rEpEaT-x"] {
        assert_eq!(keyword(spelling), 1, "{spelling}");
    }
    assert_eq!(keyword("No-Repeat"), 3);
}

#[test]
fn a_scrutinee_matching_no_arm_takes_the_fallback() {
    assert_eq!(keyword("space"), 0);
    assert_eq!(keyword("repeat-"), 0, "a prefix of an arm is not the arm");
    assert_eq!(
        keyword("repeat-xx"),
        0,
        "an extension of an arm is not the arm"
    );
}

#[test]
fn alternatives_share_one_arm() {
    fn side(ident: &str) -> u32 {
        match_ignore_ascii_case! { ident,
            "left" | "start" => 1,
            "right" | "end" | "finish" => 2,
            _ => 0,
        }
    }
    assert_eq!(side("LEFT"), 1);
    assert_eq!(side("Start"), 1);
    assert_eq!(side("END"), 2);
    assert_eq!(side("Finish"), 2);
    assert_eq!(side("middle"), 0);
}

#[test]
fn a_guard_that_fails_lets_a_later_arm_match() {
    fn side(ident: &str, horizontal: Option<u8>) -> &'static str {
        match_ignore_ascii_case! { ident,
            "left" if horizontal.is_none() => "first left",
            "left" => "second left",
            _ => "other",
        }
    }
    assert_eq!(side("Left", None), "first left");
    assert_eq!(side("Left", Some(0)), "second left");
}

#[test]
fn a_guard_that_fails_with_no_later_arm_takes_the_fallback() {
    fn side(ident: &str, allowed: bool) -> u32 {
        match_ignore_ascii_case! { ident,
            "left" if allowed => 1,
            _ => 0,
        }
    }
    assert_eq!(side("left", true), 1);
    assert_eq!(side("left", false), 0);
}

#[test]
fn a_guard_applies_to_every_alternative_of_its_arm() {
    fn kind(ident: &str, wide: bool) -> u32 {
        match_ignore_ascii_case! { ident,
            "a" | "b" if wide => 1,
            "a" | "b" => 2,
            _ => 0,
        }
    }
    assert_eq!((kind("A", true), kind("B", true)), (1, 1));
    assert_eq!((kind("A", false), kind("B", false)), (2, 2));
}

#[test]
fn the_first_matching_arm_wins() {
    fn first(ident: &str) -> u32 {
        match_ignore_ascii_case! { ident,
            "dup" => 1,
            "dup" => 2,
            _ => 0,
        }
    }
    assert_eq!(first("DUP"), 1);
}

#[test]
fn a_scrutinee_longer_than_every_arm_takes_the_fallback() {
    assert_eq!(keyword(&"repeat-x".repeat(1000)), 0);
    let long = "a".repeat(1 << 20);
    assert_eq!(keyword(&long), 0);
}

#[test]
fn a_scrutinee_exactly_as_long_as_the_longest_arm_can_match() {
    // "no-repeat" has nine bytes, the longest arm.
    assert_eq!(keyword("NO-REPEAT"), 3);
    assert_eq!(keyword("NO-REPEATS"), 0);
}

#[test]
fn an_empty_scrutinee_matches_only_an_empty_arm() {
    assert_eq!(keyword(""), 0);
    let matched = match_ignore_ascii_case! { "",
        "" => "empty",
        "x" => "x",
        _ => "other",
    };
    assert_eq!(matched, "empty");
}

#[test]
fn a_non_ascii_scrutinee_never_matches_an_ascii_arm() {
    // U+017F (long s) and U+212A (Kelvin sign) lowercase to ASCII letters
    // under Unicode case mapping, but ASCII case folding leaves them alone.
    fn word(text: &str) -> u32 {
        match_ignore_ascii_case! { text,
            "ske" => 1,
            "kelvin" => 2,
            _ => 0,
        }
    }
    assert_eq!(word("\u{17f}ke"), 0);
    assert_eq!(word("\u{212a}elvin"), 0);
    assert_eq!(word("SKE"), 1);
    assert_eq!(word("K\u{e9}"), 0);
    assert_eq!(word("\u{1f600}"), 0);
}

#[test]
fn a_non_ascii_scrutinee_short_enough_to_be_compared_does_not_panic() {
    fn word(text: &str) -> u32 {
        match_ignore_ascii_case! { text,
            "abcdef" => 1,
            _ => 0,
        }
    }
    // Two-byte, three-byte and four-byte scalar values fit in the six-byte
    // buffer.
    assert_eq!(word("\u{e9}\u{e9}"), 0);
    assert_eq!(word("a\u{1f600}"), 0);
    assert_eq!(word("\u{2603}\u{2603}"), 0);
}

#[test]
fn a_non_ascii_arm_literal_matches_exactly() {
    fn word(text: &str) -> u32 {
        match_ignore_ascii_case! { text,
            "caf\u{e9}" => 1,
            _ => 0,
        }
    }
    assert_eq!(word("caf\u{e9}"), 1);
    assert_eq!(
        word("CAF\u{e9}"),
        1,
        "the ASCII letters fold, the accent is exact"
    );
    assert_eq!(
        word("CAF\u{c9}"),
        0,
        "U+00C9 is not ASCII, so it does not fold to U+00E9"
    );
}

#[test]
fn the_fallback_alone_is_a_valid_invocation() {
    let result = match_ignore_ascii_case! { "anything",
        _ => 7,
    };
    assert_eq!(result, 7);
}

#[test]
fn a_trailing_comma_after_the_fallback_is_optional() {
    let with = match_ignore_ascii_case! { "a",
        "a" => 1,
        _ => 0,
    };
    let without = match_ignore_ascii_case! { "a",
        "a" => 1,
        _ => 0
    };
    assert_eq!((with, without), (1, 1));
}

#[test]
fn the_scrutinee_is_evaluated_exactly_once() {
    let calls = Cell::new(0);
    let scrutinee = || {
        calls.set(calls.get() + 1);
        "Beta"
    };
    let value = match_ignore_ascii_case! { scrutinee(),
        "alpha" => 1,
        "beta" => 2,
        "gamma" => 3,
        _ => 0,
    };
    assert_eq!(value, 2);
    assert_eq!(calls.get(), 1);
    let longer = || {
        calls.set(calls.get() + 1);
        "a scrutinee longer than every arm"
    };
    let value = match_ignore_ascii_case! { longer(),
        "alpha" => 1,
        _ => 0,
    };
    assert_eq!(value, 0);
    assert_eq!(calls.get(), 2);
}

#[test]
fn only_the_matching_body_runs() {
    let ran = Cell::new(0u32);
    let value = match_ignore_ascii_case! { "b",
        "a" => { ran.set(ran.get() + 1); 1 },
        "b" => { ran.set(ran.get() + 10); 2 },
        _ => { ran.set(ran.get() + 100); 0 },
    };
    assert_eq!((value, ran.get()), (2, 10));
}

#[test]
fn a_body_may_return_from_the_enclosing_function() {
    fn parse(text: &str) -> Result<u32, String> {
        match_ignore_ascii_case! { text,
            "one" => return Ok(1),
            "two" => {},
            _ => return Err(format!("unknown {text}")),
        }
        Ok(2)
    }
    assert_eq!(parse("ONE"), Ok(1));
    assert_eq!(parse("Two"), Ok(2));
    assert_eq!(parse("three"), Err("unknown three".to_owned()));
}

#[test]
fn the_scrutinee_may_be_a_reference_to_an_owned_string_or_a_cow() {
    let owned = String::from("Auto");
    let from_string = match_ignore_ascii_case! { &owned,
        "auto" => 1,
        _ => 0,
    };
    let cow = std::borrow::Cow::Borrowed("AUTO");
    let from_cow = match_ignore_ascii_case! { &cow,
        "auto" => 1,
        _ => 0,
    };
    assert_eq!((from_string, from_cow), (1, 1));
}

#[test]
fn a_guard_and_a_body_see_the_callers_names_not_the_macros() {
    // The macro uses internal bindings named `input`, `buffer` and `lowered`.
    // The caller's bindings of the same names must stay visible to guards and
    // bodies.
    let input = 5;
    let buffer = 6;
    let lowered = 7;
    let value = match_ignore_ascii_case! { "x",
        "x" if input == 5 && buffer == 6 => input + buffer + lowered,
        _ => 0,
    };
    assert_eq!(value, 18);
}

// Caller items named exactly as anything the expansion might use internally.
const LONGEST: usize = 99;
const LOWERCASE: &str = "caller";
const BYTES: usize = 7;
const INDEX: usize = 8;
const LENGTHS: usize = 9;
const INPUT: usize = 10;
const BUFFER: usize = 11;
const LOWERED: usize = 12;

#[test]
fn caller_items_named_like_the_expansions_internals_are_visible_to_guards_and_bodies() {
    let value = match_ignore_ascii_case! { "x",
        "x" if LONGEST == 99 && LOWERCASE == "caller" && BYTES + INDEX + LENGTHS == 24 => {
            LONGEST + LOWERCASE.len() + INPUT + BUFFER + LOWERED
        },
        _ => 0,
    };
    assert_eq!(value, 99 + 6 + 10 + 11 + 12);
    // The fallback sees them too.
    let fallback = match_ignore_ascii_case! { "y",
        "x" => 1,
        _ => LONGEST,
    };
    assert_eq!(fallback, 99);
}

#[test]
fn caller_locals_named_like_the_expansions_internals_are_visible_to_guards_and_bodies() {
    let bytes = 1;
    let index = 2;
    let lengths = 3;
    let longest = 4;
    let value = match_ignore_ascii_case! { "x",
        "x" if bytes + index + lengths + longest == 10 => bytes * 1000 + index * 100 + lengths * 10 + longest,
        _ => 0,
    };
    assert_eq!(value, 1234);
}

#[test]
fn the_expansion_is_usable_where_an_expression_is_expected() {
    let sum = 1 + match_ignore_ascii_case! { "TWO", "two" => 2, _ => 0 } * 10;
    assert_eq!(sum, 21);
}
