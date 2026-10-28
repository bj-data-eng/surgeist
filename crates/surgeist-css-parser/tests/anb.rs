//! Tests for the `An+B` microsyntax of CSS Syntax Module Level 3 section 6: the
//! recognition and interpretation of section 6.2 (every clause of the
//! production, its footnote about whitespace, and the examples of 6.1 and the
//! introduction) and the serialization of section 9.1.
//!
//! Expectations are written by hand from those sections. The notation of a case
//! is the text a grammar would hand over; the result is `A` and `B`, or why not.

mod common;

use common::grammar::show;
use common::tree::parse;
use surgeist_css_parser::anb::{self, AnPlusB};

/// What `anb::parse` made of `text`: `A` and `B`, or the kind of failure, and
/// what was left in the stream.
fn read(text: &str) -> (Result<(i32, i32), String>, String) {
    let document = parse(text);
    let mut stream = document.root();
    let result = anb::parse(&mut stream)
        .map(|value| (value.a(), value.b()))
        .map_err(|error| {
            if error.is_overflow() {
                "overflow".to_owned()
            } else if error.is_end_of_stream() {
                "end".to_owned()
            } else {
                format!(
                    "unexpected {}",
                    common::render(error.unexpected_token().unwrap())
                )
            }
        });
    (result, show(&stream))
}

/// The value of `text` when it is a notation by itself.
fn value(text: &str) -> (i32, i32) {
    let (result, rest) = read(text);
    assert_eq!(rest, "", "{text:?} leaves {rest:?}");
    result.unwrap_or_else(|why| panic!("{text:?}: {why}"))
}

fn rejected(text: &str) -> String {
    read(text).0.unwrap_err()
}

// ------------------------------------------------------------- 6.2 the clauses

#[test]
fn odd_and_even_are_2n_plus_1_and_2n_in_any_ascii_case() {
    assert_eq!(value("odd"), (2, 1));
    assert_eq!(value("even"), (2, 0));
    assert_eq!(value("ODD"), (2, 1));
    assert_eq!(value("EvEn"), (2, 0));
}

#[test]
fn an_integer_is_b_with_a_zero() {
    assert_eq!(value("5"), (0, 5));
    assert_eq!(value("+5"), (0, 5));
    assert_eq!(value("-5"), (0, -5));
    assert_eq!(value("0"), (0, 0));
    assert_eq!(value("007"), (0, 7));
}

#[test]
fn an_n_dimension_is_a_with_b_zero() {
    assert_eq!(value("2n"), (2, 0));
    assert_eq!(value("-2n"), (-2, 0));
    assert_eq!(value("+3N"), (3, 0));
    assert_eq!(value("0n"), (0, 0));
    assert_eq!(value("12n"), (12, 0));
}

#[test]
fn n_with_an_optional_plus_and_minus_n_are_one_and_minus_one() {
    assert_eq!(value("n"), (1, 0));
    assert_eq!(value("N"), (1, 0));
    assert_eq!(value("+n"), (1, 0));
    assert_eq!(value("-n"), (-1, 0));
    assert_eq!(value("-N"), (-1, 0));
}

#[test]
fn an_ndashdigit_dimension_or_ident_has_a_negative_b_from_its_digits() {
    // <ndashdigit-dimension>: A is the value; B is the unit without its first
    // code point, read as a base-10 number, negative.
    assert_eq!(value("2n-1"), (2, -1));
    assert_eq!(value("-3n-10"), (-3, -10));
    assert_eq!(value("0n-0"), (0, 0));
    assert_eq!(value("5N-007"), (5, -7));
    // '+'? <ndashdigit-ident>: A is 1.
    assert_eq!(value("n-1"), (1, -1));
    assert_eq!(value("+n-12"), (1, -12));
    assert_eq!(value("N-3"), (1, -3));
    // <dashndashdigit-ident>: A is -1; B is the ident without its first two
    // code points.
    assert_eq!(value("-n-1"), (-1, -1));
    assert_eq!(value("-N-25"), (-1, -25));
}

#[test]
fn a_signed_integer_after_n_is_b() {
    // <n-dimension> <signed-integer> and the forms with n.
    assert_eq!(value("2n+3"), (2, 3));
    assert_eq!(value("2n +3"), (2, 3));
    assert_eq!(value("2n -3"), (2, -3));
    assert_eq!(value("n+3"), (1, 3));
    assert_eq!(value("+n -3"), (1, -3));
    assert_eq!(value("+n+3"), (1, 3));
    assert_eq!(value("-n +6"), (-1, 6));
    assert_eq!(value("-n+6"), (-1, 6));
    assert_eq!(value("-n-6"), (-1, -6)); // the ident `-n-6`, and the same value
}

#[test]
fn a_signless_integer_after_a_dash_is_the_negation_of_b() {
    // <ndash-dimension> <signless-integer>, '+'? n- <signless-integer>, -n-
    // <signless-integer>.
    assert_eq!(value("2n- 3"), (2, -3));
    assert_eq!(value("n- 3"), (1, -3));
    assert_eq!(value("+n- 3"), (1, -3));
    assert_eq!(value("-n- 3"), (-1, -3));
    assert_eq!(value("3N-   0"), (3, 0));
}

#[test]
fn a_plus_or_minus_then_a_signless_integer_after_n_is_b_with_that_sign() {
    // <n-dimension> ['+' | '-'] <signless-integer> and the forms with n.
    assert_eq!(value("3n + 1"), (3, 1));
    assert_eq!(value("3n - 1"), (3, -1));
    assert_eq!(value("3n+ 1"), (3, 1));
    assert_eq!(value("3n- 1"), (3, -1)); // the dimension `3n-` form
    assert_eq!(value("n + 1"), (1, 1));
    assert_eq!(value("+n - 1"), (1, -1));
    assert_eq!(value("-n + 6"), (-1, 6));
    assert_eq!(value("-n - 6"), (-1, -6));
    assert_eq!(value("-n+ 6"), (-1, 6));
}

// ------------------------------------------------------------- 6 and 6.1 examples

#[test]
fn the_examples_of_the_introduction_have_the_values_the_text_gives_them() {
    assert_eq!(value("2n+0"), (2, 0)); // all of the even elements
    assert_eq!(value("even"), (2, 0)); // same
    assert_eq!(value("4n+1"), (4, 1));
    assert_eq!(value("-1n+6"), (-1, 6)); // the first 6 elements
    assert_eq!(value("-4n+10"), (-4, 10));
}

#[test]
fn the_examples_of_the_informal_description_have_the_values_the_text_gives_them() {
    // When A is 0 the An part may be omitted.
    assert_eq!(value("0n+5"), (0, 5));
    assert_eq!(value("5"), (0, 5));
    // When A is 1 or -1 the 1 may be omitted.
    assert_eq!(value("1n+0"), (1, 0));
    assert_eq!(value("n+0"), (1, 0));
    assert_eq!(value("n"), (1, 0));
    // When B is 0 the +B part may be omitted.
    assert_eq!(value("2n+0"), (2, 0));
    assert_eq!(value("2n"), (2, 0));
    // When B is negative its minus sign replaces the plus sign.
    assert_eq!(value("3n-6"), (3, -6));
    // Whitespace on either side of the + or - that separates An and B.
    assert_eq!(value("3n + 1"), (3, 1));
    assert_eq!(value("+3n - 2"), (3, -2));
    assert_eq!(value("-n+ 6"), (-1, 6));
    assert_eq!(value("+6"), (0, 6));
}

#[test]
fn the_invalid_examples_do_not_match_as_a_whole() {
    // `3n + -6`: the signed integer after the plus is not signless. The match is
    // `3n`, and the rest is left for the caller, which then fails to be empty.
    assert_eq!(
        read("3n + -6"),
        (
            Ok((3, 0)),
            "Whitespace Delim('+') Whitespace Number(\"-6\",int)".to_owned()
        )
    );
    // `3 n`: the match is the integer `3`.
    assert_eq!(
        read("3 n"),
        (Ok((0, 3)), "Whitespace Ident(\"n\")".to_owned())
    );
    // `+ 2n` and `+ 2`: whitespace between the plus and what follows.
    assert_eq!(rejected("+ 2n"), "unexpected Whitespace");
    assert_eq!(rejected("+ 2"), "unexpected Whitespace");
}

// ------------------------------------------------------------- the footnote and the longest match

#[test]
fn no_whitespace_may_separate_a_plus_sign_from_the_n_it_precedes() {
    assert_eq!(rejected("+ n"), "unexpected Whitespace");
    assert_eq!(rejected("+ n-1"), "unexpected Whitespace");
    // A comment is not a token of the specification, so it does not separate
    // them.
    assert_eq!(value("+/**/n"), (1, 0));
    // But a plus before anything that does not start with n is not a notation.
    assert_eq!(rejected("+x"), "unexpected Ident(\"x\")");
    assert_eq!(rejected("+-n"), "unexpected Ident(\"-n\")");
}

#[test]
fn whitespace_is_valid_and_ignored_between_every_other_pair_of_tokens() {
    assert_eq!(value("  2n  +  1"), (2, 1));
    assert_eq!(value("\n2n\t-\n1"), (2, -1));
    assert_eq!(value("n /*c*/ + /*d*/ 1"), (1, 1));
    // Leading whitespace is consumed; trailing whitespace is not.
    assert_eq!(read("  5  "), (Ok((0, 5)), "Whitespace".to_owned()));
}

#[test]
fn the_match_is_the_longest_the_production_allows_and_the_rest_is_left() {
    assert_eq!(
        read("2n + 1 foo"),
        (Ok((2, 1)), "Whitespace Ident(\"foo\")".to_owned())
    );
    assert_eq!(
        read("2n + x"),
        (
            Ok((2, 0)),
            "Whitespace Delim('+') Whitespace Ident(\"x\")".to_owned()
        )
    );
    assert_eq!(
        read("2n -"),
        (Ok((2, 0)), "Whitespace Delim('-')".to_owned())
    );
    assert_eq!(
        read("2n 3"),
        (Ok((2, 0)), "Whitespace Number(\"3\",int)".to_owned())
    );
    assert_eq!(
        read("2n+ x"),
        (Ok((2, 0)), "Delim('+') Whitespace Ident(\"x\")".to_owned())
    );
    // `+3` after `n` is a signed integer; a second plus is not part of it.
    assert_eq!(
        read("n +3 +4"),
        (Ok((1, 3)), "Whitespace Number(\"+4\",int)".to_owned())
    );
    // `2n+-1`: the plus is followed by a signed number, which is not signless.
    assert_eq!(
        read("2n+-1"),
        (Ok((2, 0)), "Delim('+') Number(\"-1\",int)".to_owned())
    );
}

// ------------------------------------------------------------- what does not match

#[test]
fn anything_the_production_does_not_describe_is_not_a_notation() {
    // A non-integer number or dimension.
    assert_eq!(rejected("1.5n"), "unexpected Dimension(\"1.5\",num,\"n\")");
    assert_eq!(rejected("2.0"), "unexpected Number(\"2.0\",num)");
    assert_eq!(rejected("2e1"), "unexpected Number(\"2e1\",num)");
    assert_eq!(rejected("1e1n"), "unexpected Dimension(\"1e1\",num,\"n\")");
    // A unit that is not n, n- or n- and digits.
    assert_eq!(rejected("2m"), "unexpected Dimension(\"2\",int,\"m\")");
    assert_eq!(rejected("2nn"), "unexpected Dimension(\"2\",int,\"nn\")");
    assert_eq!(rejected("2n-a"), "unexpected Dimension(\"2\",int,\"n-a\")");
    assert_eq!(
        rejected("2n--1"),
        "unexpected Dimension(\"2\",int,\"n--1\")"
    );
    // An ident that is not n, -n, odd, even or an n-form.
    assert_eq!(rejected("nn"), "unexpected Ident(\"nn\")");
    assert_eq!(rejected("n-a"), "unexpected Ident(\"n-a\")");
    assert_eq!(rejected("-n1"), "unexpected Ident(\"-n1\")");
    assert_eq!(rejected("--n"), "unexpected Ident(\"--n\")");
    assert_eq!(rejected("odd1"), "unexpected Ident(\"odd1\")");
    // A token of another kind, a percentage included.
    assert_eq!(rejected("2%"), "unexpected Percentage(\"2\",int)");
    assert_eq!(rejected("a"), "unexpected Ident(\"a\")");
    assert_eq!(rejected("\"2\""), "unexpected String(\"2\")");
    assert_eq!(rejected(","), "unexpected Comma");
    assert_eq!(rejected("f(2n)"), "unexpected Function(\"f\")");
    // The minus alone, and a dash form without its integer.
    assert_eq!(rejected("-"), "unexpected Delim('-')");
    assert_eq!(rejected("n-"), "end");
    assert_eq!(rejected("n- x"), "unexpected Ident(\"x\")");
    assert_eq!(rejected("-n-"), "end");
    assert_eq!(rejected("2n-"), "end");
    // A dash form takes a signless integer only.
    assert_eq!(rejected("n- -3"), "unexpected Number(\"-3\",int)");
    assert_eq!(rejected("n- +3"), "unexpected Number(\"+3\",int)");
    assert_eq!(rejected("n- 3.5"), "unexpected Number(\"3.5\",num)");
    // The end.
    assert_eq!(rejected(""), "end");
    assert_eq!(rejected("  "), "end");
}

#[test]
fn a_failed_parse_leaves_the_stream_where_it_was() {
    let document = parse("  n- x");
    let mut stream = document.root();
    let before = stream.site();
    assert!(anb::parse(&mut stream).is_err());
    assert_eq!(stream.site(), before);
    // A partial success restores to just after what matched.
    let document = parse("2n + x");
    let mut stream = document.root();
    assert_eq!(anb::parse(&mut stream).unwrap(), AnPlusB::new(2, 0));
    assert_eq!(
        show(&stream),
        "Whitespace Delim('+') Whitespace Ident(\"x\")"
    );
}

// ------------------------------------------------------------- the range of A and B

#[test]
fn values_at_the_edges_of_i32_are_values() {
    assert_eq!(value("2147483647"), (0, i32::MAX));
    assert_eq!(value("-2147483648"), (0, i32::MIN));
    assert_eq!(value("2147483647n"), (i32::MAX, 0));
    assert_eq!(value("-2147483648n"), (i32::MIN, 0));
    assert_eq!(value("n+2147483647"), (1, i32::MAX));
    assert_eq!(value("n-2147483648"), (1, i32::MIN));
    assert_eq!(value("n- 2147483648"), (1, i32::MIN));
    assert_eq!(value("-n-2147483648"), (-1, i32::MIN));
    assert_eq!(value("n -2147483648"), (1, i32::MIN));
    assert_eq!(value("2n - 2147483648"), (2, i32::MIN));
}

#[test]
fn a_value_outside_i32_is_a_typed_overflow_and_not_a_saturated_number() {
    for text in [
        "2147483648",
        "-2147483649",
        "2147483648n",
        "-2147483649n",
        "n+2147483648",
        "n-2147483649",
        "n- 2147483649",
        "2n+2147483648",
        "n -2147483649",
        "n - 2147483649",
        "n + 2147483648",
        "-n-2147483649",
        "+n-2147483649",
        "99999999999999999999",
        "99999999999999999999n",
        "n-99999999999999999999",
        "-n-99999999999999999999",
        "n+99999999999999999999",
        "n- 99999999999999999999",
    ] {
        assert_eq!(rejected(text), "overflow", "{text:?}");
    }
}

#[test]
fn an_overflow_leaves_the_stream_where_it_was_and_names_the_boundary() {
    let document = parse("n + 2147483648");
    let mut stream = document.root();
    let before = stream.site();
    let error = anb::parse(&mut stream).unwrap_err();
    assert!(error.is_overflow());
    assert!(!error.is_unexpected_token() && !error.is_end_of_stream() && !error.is_refused());
    assert!(error.stream_error().is_none());
    assert_eq!(stream.site(), before);
    // The overflowing integer is the boundary the error is about.
    assert_eq!(error.at().spelling(), Some("2147483648"));
    assert!(error.to_string().contains("i32"));
    // The error does not convert into a stream error.
    assert!(error.clone().into_stream_error().is_err());
}

#[test]
fn a_stream_error_converts_back_into_the_stream_error() {
    let document = parse("x");
    let error = anb::parse(&mut document.root()).unwrap_err();
    assert!(error.is_unexpected_token());
    let stream_error = error.clone().into_stream_error().unwrap();
    assert!(stream_error.is_unexpected_token());
    assert_eq!(error.stream_error(), Some(&stream_error));
}

// ------------------------------------------------------------- section 9.1

fn written(a: i32, b: i32) -> String {
    AnPlusB::new(a, b).to_string()
}

#[test]
fn an_a_of_zero_serializes_as_b_alone() {
    // Step 1.
    assert_eq!(written(0, 5), "5");
    assert_eq!(written(0, 0), "0");
    assert_eq!(written(0, -5), "-5");
    assert_eq!(written(0, i32::MIN), "-2147483648");
    assert_eq!(written(0, i32::MAX), "2147483647");
}

#[test]
fn a_of_one_and_minus_one_serialize_as_n_and_minus_n() {
    // Step 3.
    assert_eq!(written(1, 0), "n");
    assert_eq!(written(-1, 0), "-n");
    assert_eq!(written(1, 3), "n+3");
    assert_eq!(written(-1, 6), "-n+6");
    assert_eq!(written(1, -3), "n-3");
    assert_eq!(written(-1, -3), "-n-3");
}

#[test]
fn any_other_a_serializes_as_its_integer_and_n_and_b_follows_by_its_sign() {
    assert_eq!(written(2, 0), "2n");
    assert_eq!(written(2, 1), "2n+1");
    assert_eq!(written(-4, 10), "-4n+10");
    assert_eq!(written(3, -6), "3n-6");
    assert_eq!(written(i32::MIN, i32::MIN), "-2147483648n-2147483648");
    assert_eq!(written(i32::MAX, i32::MAX), "2147483647n+2147483647");
}

#[test]
fn a_serialized_notation_reads_back_as_the_same_a_and_b() {
    for a in [-5, -2, -1, 0, 1, 2, 3, 100, i32::MIN, i32::MAX] {
        for b in [-100, -3, -1, 0, 1, 2, 7, i32::MIN, i32::MAX] {
            let text = written(a, b);
            assert_eq!(value(&text), (a, b), "{text}");
        }
    }
}

#[test]
fn the_introduction_examples_serialize_to_their_shortest_form() {
    assert_eq!(AnPlusB::new(2, 0).to_string(), "2n");
    assert_eq!(AnPlusB::new(2, 1).to_string(), "2n+1"); // odd
    assert_eq!(AnPlusB::new(4, 1).to_string(), "4n+1");
    assert_eq!(AnPlusB::new(-1, 6).to_string(), "-n+6");
    assert_eq!(AnPlusB::new(0, 5).to_string(), "5");
}

// ------------------------------------------------------------- the meter

#[test]
fn a_refusal_is_reported_and_never_read_as_the_end_of_the_notation() {
    use common::tree::{Recording, metered_document};

    // `odd`: the first call examines the ident while whitespace is discarded,
    // the second consumes it.
    let meter = Recording::admitting();
    let document = metered_document(&meter, "odd");
    meter.refuse_from(1);
    let error = anb::parse(&mut document.root()).unwrap_err();
    assert!(error.is_refused());
    assert!(!error.is_end_of_stream() && !error.is_overflow());

    // `n`: after the ident the parse looks for a B, and finding the end of the
    // stream is a call of the meter. Refused, it is not the end: the notation
    // is not read as `n` with no B.
    let meter = Recording::admitting();
    let document = metered_document(&meter, "n");
    meter.refuse_from(2);
    let mut stream = document.root();
    let before = stream.site();
    let error = anb::parse(&mut stream).unwrap_err();
    assert!(error.is_refused());
    assert_eq!(stream.site(), before, "a failed parse leaves the stream");

    // One more call is admitted and the same text is `n`.
    let meter = Recording::admitting();
    let document = metered_document(&meter, "n");
    meter.refuse_from(3);
    let value = anb::parse(&mut document.root()).unwrap();
    assert_eq!((value.a(), value.b()), (1, 0));
}
