//! The free-running `Lexer` against CSS Syntax Module Level 3 (pinned edition,
//! 1 October 2026), section 4.
//!
//! Every expectation below is written by hand from the clause named beside it,
//! for inputs chosen from that clause's prose branches and worked examples. None
//! is read from the lexer's output or from another implementation. Tokens are
//! written in the notation of `tests/common/mod.rs`, for example `Ident("a")`.
//!
//! Positions are original UTF-8 byte offsets: section 3.3 is applied as the
//! lexer reads, so a carriage return and line feed pair is one scalar that
//! covers two bytes.

mod common;

use common::{assert_tiles, event, events, lex_all, lex_with, tokens, tokens_with_ranges};
use surgeist_css_parser::Offset;
use surgeist_css_parser::lexer::Options;

fn check(cases: &[(&str, &[&str])]) {
    for (text, expected) in cases {
        assert_eq!(tokens(text), *expected, "tokens of {text:?}");
        assert_tiles(text);
    }
}

// ---------------------------------------------------------------- 4.3.1

#[test]
fn the_end_of_the_text_is_the_eof_token_and_produces_no_lexeme() {
    check(&[("", &[])]);
}

#[test]
fn whitespace_is_one_token_for_the_longest_run() {
    // "Consume as much whitespace as possible. Return a <whitespace-token>."
    // Whitespace is a newline, a tab or a space (4.2).
    check(&[
        (" ", &["Whitespace"]),
        ("a \t\n b", &["Ident(\"a\")", "Whitespace", "Ident(\"b\")"]),
    ]);
    let (items, _) = lex_all("a \t\n b");
    assert_eq!(items[1].range, 1..5);
}

#[test]
fn single_code_point_tokens_are_their_own_tokens() {
    check(&[
        ("(", &["OpenParen"]),
        (")", &["CloseParen"]),
        ("[", &["OpenSquare"]),
        ("]", &["CloseSquare"]),
        ("{", &["OpenCurly"]),
        ("}", &["CloseCurly"]),
        (",", &["Comma"]),
        (":", &["Colon"]),
        (";", &["Semicolon"]),
        (
            "{(,)}",
            &[
                "OpenCurly",
                "OpenParen",
                "Comma",
                "CloseParen",
                "CloseCurly",
            ],
        ),
    ]);
}

#[test]
fn any_other_code_point_is_a_delim_token_with_that_code_point() {
    // "anything else: Return a <delim-token> with its value set to the current
    // input code point."
    for point in "!%&*/=>?^`|~$".chars() {
        let text = point.to_string();
        assert_eq!(tokens(&text), [format!("Delim({point:?})")], "{text:?}");
    }
    // A lone sign, full stop, less-than sign, number sign and commercial at
    // each fail their branch's condition and are delims.
    for point in "+-.<#@".chars() {
        let text = point.to_string();
        assert_eq!(tokens(&text), [format!("Delim({point:?})")], "{text:?}");
    }
    // The attribute-selector operators are two tokens, not one.
    check(&[
        ("~=", &["Delim('~')", "Delim('=')"]),
        ("|=", &["Delim('|')", "Delim('=')"]),
        ("^=", &["Delim('^')", "Delim('=')"]),
        ("$=", &["Delim('$')", "Delim('=')"]),
        ("*=", &["Delim('*')", "Delim('=')"]),
    ]);
}

#[test]
fn a_number_sign_starts_a_hash_only_before_an_ident_code_point_or_a_valid_escape() {
    // 4.3.1: the type flag is "id" when the next three code points would start
    // an ident sequence (4.3.9), "unrestricted" otherwise.
    check(&[
        ("#abc", &["Hash(\"abc\",id)"]),
        ("#1abc", &["Hash(\"1abc\",unrestricted)"]),
        ("#-a", &["Hash(\"-a\",id)"]),
        ("#--", &["Hash(\"--\",id)"]),
        ("#-1", &["Hash(\"-1\",unrestricted)"]),
        ("#_", &["Hash(\"_\",id)"]),
        ("#\\31 x", &["Hash(\"1x\",id)"]),
        ("#\\41", &["Hash(\"A\",id)"]),
        ("# a", &["Delim('#')", "Whitespace", "Ident(\"a\")"]),
        ("#(", &["Delim('#')", "OpenParen"]),
        ("##a", &["Delim('#')", "Hash(\"a\",id)"]),
    ]);
}

#[test]
fn a_commercial_at_starts_an_at_keyword_only_before_an_ident_sequence() {
    check(&[
        ("@media", &["AtKeyword(\"media\")"]),
        ("@-x", &["AtKeyword(\"-x\")"]),
        ("@--", &["AtKeyword(\"--\")"]),
        ("@_a1", &["AtKeyword(\"_a1\")"]),
        ("@\\6d edia", &["AtKeyword(\"media\")"]),
        ("@1", &["Delim('@')", "Number(\"1\",int)"]),
        ("@-1", &["Delim('@')", "Number(\"-1\",int)"]),
        ("@ a", &["Delim('@')", "Whitespace", "Ident(\"a\")"]),
        ("@@a", &["Delim('@')", "AtKeyword(\"a\")"]),
    ]);
}

#[test]
fn a_less_than_sign_is_a_cdo_only_before_exclamation_mark_and_two_hyphens() {
    check(&[
        ("<!--", &["Cdo"]),
        ("<!---", &["Cdo", "Delim('-')"]),
        ("<!-", &["Delim('<')", "Delim('!')", "Delim('-')"]),
        ("<!", &["Delim('<')", "Delim('!')"]),
        ("<a", &["Delim('<')", "Ident(\"a\")"]),
        ("<<!--", &["Delim('<')", "Cdo"]),
    ]);
}

#[test]
fn a_hyphen_minus_is_tried_as_a_number_then_a_cdc_then_an_ident_then_a_delim() {
    check(&[
        ("-1", &["Number(\"-1\",int)"]),
        ("-.5", &["Number(\"-.5\",num)"]),
        ("-->", &["Cdc"]),
        ("--x", &["Ident(\"--x\")"]),
        ("--x>", &["Ident(\"--x\")", "Delim('>')"]),
        ("-a", &["Ident(\"-a\")"]),
        ("-a1", &["Ident(\"-a1\")"]),
        ("--1", &["Ident(\"--1\")"]),
        ("-\\41", &["Ident(\"-A\")"]),
        ("-", &["Delim('-')"]),
        ("- ", &["Delim('-')", "Whitespace"]),
        ("-+1", &["Delim('-')", "Number(\"+1\",int)"]),
        ("--", &["Ident(\"--\")"]),
        ("---", &["Ident(\"---\")"]),
        ("-->-", &["Cdc", "Delim('-')"]),
    ]);
}

#[test]
fn a_plus_sign_and_a_full_stop_start_a_number_only_when_one_would_start() {
    check(&[
        ("+1", &["Number(\"+1\",int)"]),
        ("+.5", &["Number(\"+.5\",num)"]),
        (".5", &["Number(\".5\",num)"]),
        ("+a", &["Delim('+')", "Ident(\"a\")"]),
        ("+.", &["Delim('+')", "Delim('.')"]),
        ("+.a", &["Delim('+')", "Delim('.')", "Ident(\"a\")"]),
        (".a", &["Delim('.')", "Ident(\"a\")"]),
        (". 5", &["Delim('.')", "Whitespace", "Number(\"5\",int)"]),
    ]);
}

#[test]
fn a_reverse_solidus_starts_an_ident_like_token_only_with_a_valid_escape() {
    // 4.3.8: a backslash then anything but a newline, EOF included, is valid.
    check(&[
        ("\\g", &["Ident(\"g\")"]),
        ("\\ ", &["Ident(\" \")"]),
        ("\\\\", &["Ident(\"\\\\\")"]),
        ("\\{", &["Ident(\"{\")"]),
        ("\\\n", &["Delim('\\\\')", "Whitespace"]),
        (
            "a\\\nb",
            &[
                "Ident(\"a\")",
                "Delim('\\\\')",
                "Whitespace",
                "Ident(\"b\")",
            ],
        ),
    ]);
}

#[test]
fn an_ident_start_code_point_starts_an_ident_like_token() {
    check(&[
        ("a", &["Ident(\"a\")"]),
        ("Z", &["Ident(\"Z\")"]),
        ("_", &["Ident(\"_\")"]),
        ("a-b_c9", &["Ident(\"a-b_c9\")"]),
        ("a.b", &["Ident(\"a\")", "Delim('.')", "Ident(\"b\")"]),
        ("\u{e9}", &["Ident(\"\u{e9}\")"]),
        ("a\u{1f600}b", &["Ident(\"a\u{1f600}b\")"]),
    ]);
    let (items, _) = lex_all("\u{e9}1\u{1f600}");
    // One ident of three scalars covering 2 + 1 + 4 original bytes.
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].range, 0..7);
}

#[test]
fn the_non_ascii_ident_code_point_is_the_2021_rule_until_the_pinned_edition_is_adopted() {
    // 4.2 of the pinned edition lists ranges; the lexer implements the 2021
    // rule (every code point from U+0080), which the coverage page records as a
    // deliberate difference. Code points the pinned list excludes are idents
    // for now; adopting the list changes exactly this test.
    check(&[
        ("a\u{a0}b", &["Ident(\"a\u{a0}b\")"]),
        ("a\u{2603}b", &["Ident(\"a\u{2603}b\")"]),
    ]);
}

// ---------------------------------------------------------------- 4.3.2

#[test]
fn a_comment_runs_to_the_first_asterisk_solidus() {
    let (items, errors) = lex_all("/**/ /* a * / b */");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        ["Comment", "Whitespace", "Comment"]
    );
    assert_eq!(items[0].range, 0..4);
    assert_eq!(items[2].range, 5..18);
    assert!(items.iter().all(|item| !item.unterminated));
    assert!(errors.is_empty());
}

#[test]
fn a_comment_is_recognized_between_tokens_and_inside_none() {
    check(&[
        ("a/**/b", &["Ident(\"a\")", "Comment", "Ident(\"b\")"]),
        ("/**//**/", &["Comment", "Comment"]),
        ("/* \"x */y", &["Comment", "Ident(\"y\")"]),
        (
            "a/ *",
            &["Ident(\"a\")", "Delim('/')", "Whitespace", "Delim('*')"],
        ),
        ("/", &["Delim('/')"]),
        // Comments are not recognized inside a string or a url.
        ("\"/**/\"", &["String(\"/**/\")"]),
        ("url(a/**/b)", &["Url(\"a/**/b\")"]),
        ("url(/**/a)", &["Url(\"/**/a\")"]),
    ]);
}

#[test]
fn a_comment_that_reaches_eof_is_unterminated_and_a_parse_error() {
    // 4.3.2: "If the preceding paragraph ended by consuming an EOF code point,
    // this is a parse error." The asterisk of the opening cannot also close it.
    for text in ["/*", "/* a", "/*/", "/* a *", "/**"] {
        let (items, errors) = lex_all(text);
        assert_eq!(items.len(), 1, "{text:?}");
        assert!(items[0].unterminated, "{text:?}");
        assert_eq!(items[0].range, 0..text.len(), "{text:?}");
        assert_eq!(
            errors,
            [event("unterminated_comment", 0..text.len())],
            "{text:?}"
        );
    }
}

// ---------------------------------------------------------------- 4.3.3 / 4.3.13

#[test]
fn a_number_is_a_sign_digits_a_fraction_and_an_exponent() {
    check(&[
        ("12", &["Number(\"12\",int)"]),
        ("+12", &["Number(\"+12\",int)"]),
        ("-12", &["Number(\"-12\",int)"]),
        ("1.5", &["Number(\"1.5\",num)"]),
        ("-.5", &["Number(\"-.5\",num)"]),
        ("+.5e-3", &["Number(\"+.5e-3\",num)"]),
        ("1e3", &["Number(\"1e3\",num)"]),
        ("1E3", &["Number(\"1E3\",num)"]),
        ("1e+3", &["Number(\"1e+3\",num)"]),
        ("1e-3", &["Number(\"1e-3\",num)"]),
        ("0.50", &["Number(\"0.50\",num)"]),
        ("007", &["Number(\"007\",int)"]),
    ]);
}

#[test]
fn a_number_ends_where_no_further_part_applies() {
    check(&[
        // A full stop needs a digit after it to be a fraction.
        ("1.", &["Number(\"1\",int)", "Delim('.')"]),
        (
            "1..5",
            &["Number(\"1\",int)", "Delim('.')", "Number(\".5\",num)"],
        ),
        // A second full stop starts a new number: 4.3.13 step 4 consumes one
        // fraction only.
        ("0.0.2", &["Number(\"0.0\",num)", "Number(\".2\",num)"]),
        ("1.5.5", &["Number(\"1.5\",num)", "Number(\".5\",num)"]),
        // An exponent needs a digit after the marker and its optional sign.
        ("1e", &["Dimension(\"1\",int,\"e\")"]),
        ("1e+", &["Dimension(\"1\",int,\"e\")", "Delim('+')"]),
        ("1e-", &["Dimension(\"1\",int,\"e-\")"]),
        (
            "1e+x",
            &["Dimension(\"1\",int,\"e\")", "Delim('+')", "Ident(\"x\")"],
        ),
        ("1e-x", &["Dimension(\"1\",int,\"e-x\")"]),
        ("1e3e3", &["Dimension(\"1e3\",num,\"e3\")"]),
        // A sign after the number is not part of it.
        ("1-2", &["Number(\"1\",int)", "Number(\"-2\",int)"]),
        ("1+2", &["Number(\"1\",int)", "Number(\"+2\",int)"]),
        ("1-", &["Number(\"1\",int)", "Delim('-')"]),
    ]);
}

#[test]
fn a_numeric_token_is_a_dimension_before_an_ident_sequence_and_a_percentage_before_a_percent_sign()
{
    check(&[
        ("12px", &["Dimension(\"12\",int,\"px\")"]),
        ("1.5em", &["Dimension(\"1.5\",num,\"em\")"]),
        ("-1.5E3Q", &["Dimension(\"-1.5E3\",num,\"Q\")"]),
        ("12%", &["Percentage(\"12\",int)"]),
        ("12.5%", &["Percentage(\"12.5\",num)"]),
        ("1_", &["Dimension(\"1\",int,\"_\")"]),
        ("1--x", &["Dimension(\"1\",int,\"--x\")"]),
        ("1-x", &["Dimension(\"1\",int,\"-x\")"]),
        ("1\\70 x", &["Dimension(\"1\",int,\"px\")"]),
        ("1\\31 ", &["Dimension(\"1\",int,\"1\")"]),
        ("1%%", &["Percentage(\"1\",int)", "Delim('%')"]),
        (
            "1 px",
            &["Number(\"1\",int)", "Whitespace", "Ident(\"px\")"],
        ),
        // Percent is tested after the ident check, and neither applies to
        // a following digit.
        (
            "1 2",
            &["Number(\"1\",int)", "Whitespace", "Number(\"2\",int)"],
        ),
    ]);
    let (items, _) = lex_all("12px");
    assert_eq!(items[0].range, 0..4);
}

#[test]
fn the_number_representation_excludes_the_unit_and_percent_sign_and_keeps_the_exact_spelling() {
    let (items, _) = lex_all("+007.500E-02px 50%");
    assert_eq!(items[0].token, "Dimension(\"+007.500E-02\",num,\"px\")");
    assert_eq!(items[2].token, "Percentage(\"50\",int)");
}

// ---------------------------------------------------------------- 4.3.4

#[test]
fn an_ident_followed_by_a_left_parenthesis_is_a_function_token_that_includes_it() {
    let (items, _) = lex_all("f(a)");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        ["Function(\"f\")", "Ident(\"a\")", "CloseParen"]
    );
    assert_eq!(items[0].range, 0..2);
    check(&[
        ("-f(", &["Function(\"-f\")"]),
        ("\\66(", &["Function(\"f\")"]),
        ("f (", &["Ident(\"f\")", "Whitespace", "OpenParen"]),
        ("f()", &["Function(\"f\")", "CloseParen"]),
    ]);
}

#[test]
fn url_followed_by_a_parenthesis_and_no_quote_is_a_url_token() {
    // "ASCII case-insensitive match for url", including when written with
    // escapes (the ident sequence is decoded first).
    for text in [
        "url(a)",
        "URL(a)",
        "uRl(a)",
        "u\\72l(a)",
        "\\75rl(a)",
        "ur\\6c(a)",
    ] {
        let (items, errors) = lex_all(text);
        assert_eq!(
            items
                .iter()
                .map(|item| item.token.as_str())
                .collect::<Vec<_>>(),
            ["Url(\"a\")"],
            "{text:?}"
        );
        assert_eq!(items[0].range, 0..text.len(), "{text:?}");
        assert!(errors.is_empty(), "{text:?}");
    }
}

#[test]
fn url_followed_by_a_quote_is_a_function_token_and_the_string_is_separate() {
    // 4.3.4: "If the next one or two input code points are a quote, or whitespace
    // followed by a quote, create a <function-token>". The whitespace the loop
    // consumes is retained as its own whitespace token with its own interval.
    let (items, _) = lex_all("url(\"a\")");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        ["Function(\"url\")", "String(\"a\")", "CloseParen"]
    );
    assert_eq!(items[0].range, 0..4);
    assert_eq!(items[1].range, 4..7);

    let (items, _) = lex_all("url( 'a')");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        [
            "Function(\"url\")",
            "Whitespace",
            "String(\"a\")",
            "CloseParen"
        ]
    );
    assert_eq!(items[0].range, 0..4);
    assert_eq!(items[1].range, 4..5);

    // Two spaces: the loop consumes the first and the second is the whitespace
    // token, so one whitespace token follows the function token.
    let (items, _) = lex_all("url(  \"a\")");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        [
            "Function(\"url\")",
            "Whitespace",
            "String(\"a\")",
            "CloseParen"
        ]
    );
    assert_eq!(items[0].range, 0..4);
    assert_eq!(items[1].range, 4..6);
    assert_eq!(items[2].range, 6..9);
    assert_tiles("url(  \"a\")");
}

#[test]
fn only_url_exactly_selects_url_consumption() {
    check(&[
        (
            "xurl(a)",
            &["Function(\"xurl\")", "Ident(\"a\")", "CloseParen"],
        ),
        (
            "url-x(a)",
            &["Function(\"url-x\")", "Ident(\"a\")", "CloseParen"],
        ),
        (
            "urls(a)",
            &["Function(\"urls\")", "Ident(\"a\")", "CloseParen"],
        ),
        (
            "url (a)",
            &[
                "Ident(\"url\")",
                "Whitespace",
                "OpenParen",
                "Ident(\"a\")",
                "CloseParen",
            ],
        ),
        ("url", &["Ident(\"url\")"]),
        (
            "@url(a)",
            &[
                "AtKeyword(\"url\")",
                "OpenParen",
                "Ident(\"a\")",
                "CloseParen",
            ],
        ),
        (
            "1url(a)",
            &[
                "Dimension(\"1\",int,\"url\")",
                "OpenParen",
                "Ident(\"a\")",
                "CloseParen",
            ],
        ),
        (
            "#url(a)",
            &[
                "Hash(\"url\",id)",
                "OpenParen",
                "Ident(\"a\")",
                "CloseParen",
            ],
        ),
    ]);
}

// ---------------------------------------------------------------- 4.3.5, 4.3.7

#[test]
fn a_string_runs_to_its_own_quote() {
    let (items, errors) = lex_all("\"abc\" 'a\"b'");
    assert_eq!(items[0].token, "String(\"abc\")");
    assert_eq!(items[0].range, 0..5);
    assert_eq!(items[2].token, "String(\"a\\\"b\")");
    assert_eq!(items[2].range, 6..11);
    assert!(items.iter().all(|item| !item.unterminated));
    assert!(errors.is_empty());
    check(&[
        ("\"\"", &["String(\"\")"]),
        ("''", &["String(\"\")"]),
        ("'a\"b'", &["String(\"a\\\"b\")"]),
        ("\"a'b\"", &["String(\"a'b\")"]),
        ("\"a b\"", &["String(\"a b\")"]),
    ]);
}

#[test]
fn a_string_that_reaches_eof_is_unterminated_and_a_parse_error() {
    for (text, value) in [("\"abc", "abc"), ("'", ""), ("\"a\\41", "aA")] {
        let (items, errors) = lex_all(text);
        assert_eq!(items.len(), 1, "{text:?}");
        assert_eq!(items[0].token, format!("String({value:?})"), "{text:?}");
        assert!(items[0].unterminated, "{text:?}");
        assert_eq!(
            errors,
            [event("unterminated_string", 0..text.len())],
            "{text:?}"
        );
    }
}

#[test]
fn a_newline_in_a_string_makes_a_bad_string_and_is_reconsumed() {
    // 4.3.5: "newline: This is a parse error. Reconsume the current input code
    // point, create a <bad-string-token>, and return it." The newline then
    // starts a whitespace token, and the next quote a new string.
    let (items, errors) = lex_all("\"ab\ncd\"");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        ["BadString", "Whitespace", "Ident(\"cd\")", "String(\"\")"]
    );
    assert_eq!(items[0].range, 0..3);
    assert!(!items[0].unterminated);
    assert_eq!(items[1].range, 3..4);
    assert_eq!(items[3].range, 6..7);
    assert!(items[3].unterminated);
    assert_eq!(
        errors,
        [
            event("bad_string", 0..3),
            event("unterminated_string", 6..7)
        ]
    );
}

#[test]
fn a_backslash_in_a_string_continues_a_line_ends_at_eof_or_escapes() {
    // 4.3.5: backslash then EOF: do nothing. Backslash then a newline: consume
    // it. Otherwise consume an escaped code point (4.3.7).
    check(&[
        ("\"a\\\nb\"", &["String(\"ab\")"]),
        ("\"a\\\"b\"", &["String(\"a\\\"b\")"]),
        ("\"\\41 b\"", &["String(\"Ab\")"]),
        ("\"\\41\"", &["String(\"A\")"]),
        ("\"\\000041b\"", &["String(\"Ab\")"]),
        ("\"\\g\"", &["String(\"g\")"]),
        ("\"\\\\\"", &["String(\"\\\\\")"]),
    ]);
    // Backslash at EOF inside a string is not an escape-at-EOF error; only the
    // missing quote is.
    let (items, errors) = lex_all("\"a\\");
    assert_eq!(items[0].token, "String(\"a\")");
    assert_eq!(errors, [event("unterminated_string", 0..3)]);
}

#[test]
fn an_escaped_code_point_is_one_to_six_hex_digits_and_one_optional_whitespace() {
    // 4.3.7: "Consume as many hex digits as possible, but no more than 5" after
    // the first. If whitespace follows, consume it as well. Zero, a surrogate
    // and a value above U+10FFFF give U+FFFD.
    let ident = |text: &str| tokens(text);
    assert_eq!(ident("\\26 B"), ["Ident(\"&B\")"]);
    assert_eq!(ident("\\000026B"), ["Ident(\"&B\")"]);
    assert_eq!(ident("\\41"), ["Ident(\"A\")"]);
    assert_eq!(ident("\\41 "), ["Ident(\"A\")"]);
    assert_eq!(ident("\\41  "), ["Ident(\"A\")", "Whitespace"]);
    assert_eq!(ident("\\41\t"), ["Ident(\"A\")"]);
    assert_eq!(ident("\\41\n"), ["Ident(\"A\")"]);
    assert_eq!(ident("\\10ffff "), ["Ident(\"\u{10ffff}\")"]);
    assert_eq!(ident("\\0 "), ["Ident(\"\u{fffd}\")"]);
    assert_eq!(ident("\\000000 "), ["Ident(\"\u{fffd}\")"]);
    assert_eq!(ident("\\d800 "), ["Ident(\"\u{fffd}\")"]);
    assert_eq!(ident("\\dfff "), ["Ident(\"\u{fffd}\")"]);
    assert_eq!(ident("\\110000 "), ["Ident(\"\u{fffd}\")"]);
    assert_eq!(ident("\\ffffff "), ["Ident(\"\u{fffd}\")"]);
    // Seven digits: the seventh is an ordinary ident code point.
    assert_eq!(ident("\\1234567"), ["Ident(\"\u{fffd}7\")"]);
    assert_eq!(ident("\\31 23"), ["Ident(\"123\")"]);
    assert_eq!(
        ident("\\41\\42 \\43  x"),
        ["Ident(\"ABC\")", "Whitespace", "Ident(\"x\")"]
    );
    // A code point that is not a hex digit stands for itself.
    assert_eq!(ident("a\\ b"), ["Ident(\"a b\")"]);
    assert_eq!(ident("\\g"), ["Ident(\"g\")"]);
    assert_eq!(ident("\\{\\}"), ["Ident(\"{}\")"]);
    assert_eq!(ident("\\\u{e9}"), ["Ident(\"\u{e9}\")"]);
    let (items, _) = lex_all("\\41\\42 \\43  x");
    assert_eq!(items[0].range, 0..11);
    assert_eq!(items[1].range, 11..12);
}

#[test]
fn an_ident_sequence_consumes_ident_code_points_and_valid_escapes() {
    // 4.3.12.
    check(&[
        ("a\\62 c", &["Ident(\"abc\")"]),
        ("a\\\\b", &["Ident(\"a\\\\b\")"]),
        ("a\\.b", &["Ident(\"a.b\")"]),
        ("--a-\\2d-b", &["Ident(\"--a---b\")"]),
        ("\\31 0", &["Ident(\"10\")"]),
        // An escape stops the name at a backslash-newline, which is not a
        // valid escape.
        ("a\\\n", &["Ident(\"a\")", "Delim('\\\\')", "Whitespace"]),
    ]);
}

// ---------------------------------------------------------------- 4.3.6, 4.3.15

#[test]
fn an_unquoted_url_is_whitespace_trimmed_text_up_to_a_right_parenthesis() {
    let cases: &[(&str, &str)] = &[
        ("url(a)", "a"),
        ("url( a )", "a"),
        ("url(\t\na\n\t)", "a"),
        ("url()", ""),
        ("url(  )", ""),
        ("url(a/b.png?x=1#y)", "a/b.png?x=1#y"),
        ("url(a\\)b)", "a)b"),
        ("url(\\41 b)", "Ab"),
        ("url(a\\ b)", "a b"),
        ("url(\u{e9})", "\u{e9}"),
    ];
    for (text, value) in cases {
        let (items, errors) = lex_all(text);
        assert_eq!(items.len(), 1, "{text:?}");
        assert_eq!(items[0].token, format!("Url({value:?})"), "{text:?}");
        assert_eq!(items[0].range, 0..text.len(), "{text:?}");
        assert!(!items[0].unterminated, "{text:?}");
        assert!(errors.is_empty(), "{text:?}");
    }
    check(&[("url(a)url(b)", &["Url(\"a\")", "Url(\"b\")"])]);
}

#[test]
fn a_url_that_reaches_eof_is_unterminated_and_a_parse_error() {
    // 4.3.6: EOF is a parse error that returns the url token, also after
    // whitespace, and an escape at EOF gives U+FFFD (4.3.7).
    for (text, value) in [
        ("url(", ""),
        ("url(  ", ""),
        ("url(a", "a"),
        ("url(a \t", "a"),
        ("url(\\0", "\u{fffd}"),
        ("url(\\d800", "\u{fffd}"),
    ] {
        let (items, errors) = lex_all(text);
        assert_eq!(items.len(), 1, "{text:?}");
        assert_eq!(items[0].token, format!("Url({value:?})"), "{text:?}");
        assert!(items[0].unterminated, "{text:?}");
        assert_eq!(
            errors,
            [event("unterminated_url", 0..text.len())],
            "{text:?}"
        );
    }
    let (items, errors) = lex_all("url(a\\");
    assert_eq!(items[0].token, "Url(\"a\u{fffd}\")");
    assert!(items[0].unterminated);
    assert_eq!(
        errors,
        [event("escaped_eof", 5..6), event("unterminated_url", 0..6)]
    );
}

#[test]
fn a_quote_a_parenthesis_a_non_printable_or_an_invalid_escape_makes_a_bad_url() {
    // 4.3.6: each of these is a parse error, then the remnants (4.3.15) are
    // consumed up to the first unescaped right parenthesis or EOF.
    for text in [
        "url(a\"b)",
        "url(a'b)",
        "url(a(b)",
        "url(a\u{1}b)",
        "url(a\u{8}b)",
        "url(a\u{b}b)",
        "url(a\u{e}b)",
        "url(a\u{1f}b)",
        "url(a\u{7f}b)",
        "url(a\\\nb)",
        "url(\"a\" b)",
    ] {
        let (items, errors) = lex_all(text);
        if text == "url(\"a\" b)" {
            // This one is a function token and a string, not a url at all.
            assert_eq!(items[0].token, "Function(\"url\")");
            continue;
        }
        assert_eq!(items.len(), 1, "{text:?}");
        assert_eq!(items[0].token, "BadUrl", "{text:?}");
        assert_eq!(items[0].range, 0..text.len(), "{text:?}");
        assert!(!items[0].unterminated, "{text:?}");
        assert_eq!(errors, [event("bad_url", 0..text.len())], "{text:?}");
    }
    // A non-printable code point first in the url is the same.
    let (items, errors) = lex_all("url(\u{1}a)");
    assert_eq!(items[0].token, "BadUrl");
    assert_eq!(errors, [event("bad_url", 0..7)]);
}

#[test]
fn whitespace_followed_by_anything_but_a_right_parenthesis_is_a_bad_url_without_a_parse_error() {
    // 4.3.6 words this branch without "this is a parse error": "otherwise,
    // consume the remnants of a bad url, create a <bad-url-token>".
    for text in ["url(a b)", "url(a \tb)", "url(a b c)", "url(a\nb)"] {
        let (items, errors) = lex_all(text);
        assert_eq!(items.len(), 1, "{text:?}");
        assert_eq!(items[0].token, "BadUrl", "{text:?}");
        assert!(errors.is_empty(), "{text:?}");
    }
    // EOF while consuming the remnants is not a parse error either (4.3.15),
    // but the token reaches EOF.
    let (items, errors) = lex_all("url(a b");
    assert_eq!(items[0].token, "BadUrl");
    assert!(items[0].unterminated);
    assert!(errors.is_empty());
}

#[test]
fn the_remnants_of_a_bad_url_stop_at_an_unescaped_right_parenthesis_only() {
    // 4.3.15: "starts with a valid escape: consume an escaped code point. This
    // allows an escaped right parenthesis to be encountered without ending the
    // bad url."
    let (items, _) = lex_all("url(a b\\)c)d");
    assert_eq!(items[0].token, "BadUrl");
    assert_eq!(items[0].range, 0..11);
    assert_eq!(items[1].token, "Ident(\"d\")");
    let (items, _) = lex_all("url(a b\\29 c)d");
    assert_eq!(items[0].range, 0..13);
    assert_eq!(items[1].token, "Ident(\"d\")");
    // A hex escape consumes its optional terminator, so the space is not a
    // second thing; a backslash-newline is not a valid escape and is ordinary.
    let (items, _) = lex_all("url(a b\\\n)d");
    assert_eq!(items[0].range, 0..10);
    assert_eq!(items[1].token, "Ident(\"d\")");
    // A backslash at EOF is a valid escape; its code point is EOF, a parse
    // error (4.3.7), and the bad url reaches EOF.
    let (items, errors) = lex_all("url(a b\\");
    assert_eq!(items[0].token, "BadUrl");
    assert!(items[0].unterminated);
    assert_eq!(errors, [event("escaped_eof", 7..8)]);
}

#[test]
fn the_remnants_of_a_bad_url_examine_the_first_code_point_after_the_whitespace() {
    // 4.3.6: after whitespace that is followed by neither ")" nor EOF, "consume
    // the remnants of a bad url". 4.3.15 repeatedly consumes the next code
    // point and, "the input stream starts with a valid escape" (4.3.8: the
    // current code point, the backslash, and the next one), consumes an escaped
    // code point. So the backslash that comes first after the whitespace
    // escapes the code point after it, and a ")" so escaped does not end the
    // bad url. Neither case is a parse error (4.3.6 words the whitespace
    // branch without one).
    //
    // url(a \)b)c : the bad url is "url(a \)b)", bytes 0..10.
    let (items, errors) = lex_all("url(a \\)b)c");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        ["BadUrl", "Ident(\"c\")"]
    );
    assert_eq!(items[0].range, 0..10);
    assert!(!items[0].unterminated);
    assert!(errors.is_empty());
    // url(a \\)b)c : "\\" escapes the second backslash, so the next ")" ends the
    // bad url at byte 9; "b", ")" and "c" are ordinary tokens.
    let (items, errors) = lex_all("url(a \\\\)b)c");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        ["BadUrl", "Ident(\"b\")", "CloseParen", "Ident(\"c\")"]
    );
    assert_eq!(items[0].range, 0..9);
    assert!(errors.is_empty());
    // Tab and newline are whitespace too, and a second escaped parenthesis later
    // in the remnants is skipped the same way.
    let (items, _) = lex_all("url(a\t\\)b\\)c)d");
    assert_eq!(items[0].token, "BadUrl");
    assert_eq!(items[0].range, 0..13);
    assert_eq!(items[1].token, "Ident(\"d\")");
}

#[test]
fn a_hyphen_minus_before_a_backslash_and_a_newline_starts_no_ident_sequence() {
    // 4.3.9: for a first hyphen-minus the check is true only if the second code
    // point is an ident-start code point or a hyphen-minus, or the second and
    // third are a valid escape. A backslash followed by a newline is not one
    // (4.3.8), so it is false. Each consumer of the check then takes its other
    // branch: 4.3.1 makes a delim of "-" (and, in turn, of the backslash, a
    // parse error), an at-keyword needs the check (4.3.1), a dimension's unit
    // needs it (4.3.3), and a hash's type flag is "id" only if it holds (4.3.1).
    let backslash = |start: usize| event("escaped_newline", start..start + 1);
    let cases: &[(&str, &[&str], usize)] = &[
        ("-\\\n", &["Delim('-')", "Delim('\\\\')", "Whitespace"], 1),
        (
            "@-\\\n",
            &["Delim('@')", "Delim('-')", "Delim('\\\\')", "Whitespace"],
            2,
        ),
        (
            "1-\\\n",
            &[
                "Number(\"1\",int)",
                "Delim('-')",
                "Delim('\\\\')",
                "Whitespace",
            ],
            2,
        ),
        (
            "#-\\\n",
            &["Hash(\"-\",unrestricted)", "Delim('\\\\')", "Whitespace"],
            2,
        ),
    ];
    for (text, expected, at) in cases {
        assert_eq!(tokens(text), *expected, "{text:?}");
        assert_eq!(events(text), [backslash(*at)], "{text:?}");
        assert_tiles(text);
    }
    // With a carriage return or a form feed as the newline (3.3 makes it one).
    assert_eq!(
        tokens("-\\\r\n"),
        ["Delim('-')", "Delim('\\\\')", "Whitespace"]
    );
    // A real escape after the hyphen-minus does start an ident sequence.
    assert_eq!(tokens("-\\41"), ["Ident(\"-A\")"]);
    assert_eq!(tokens("#-\\41"), ["Hash(\"-A\",id)"]);
}

// ---------------------------------------------------------------- 3.3 and 4.3.7 EOF

#[test]
fn a_backslash_at_eof_is_a_valid_escape_for_the_ident_that_stands_for_u_fffd() {
    // 4.3.8: EOF is not a newline, so the escape is valid. 4.3.7: "EOF: This is
    // a parse error. Return U+FFFD".
    for (text, token, offset) in [
        ("\\", "Ident(\"\u{fffd}\")", 0),
        ("a\\", "Ident(\"a\u{fffd}\")", 1),
        ("#\\", "Hash(\"\u{fffd}\",id)", 1),
        ("@\\", "AtKeyword(\"\u{fffd}\")", 1),
        ("1\\", "Dimension(\"1\",int,\"\u{fffd}\")", 1),
        ("\\g\\", "Ident(\"g\u{fffd}\")", 2),
    ] {
        let (items, errors) = lex_all(text);
        assert_eq!(items.len(), 1, "{text:?}");
        assert_eq!(items[0].token, token, "{text:?}");
        assert_eq!(
            errors,
            [event("escaped_eof", offset..offset + 1)],
            "{text:?}"
        );
    }
    // A backslash then hex digits then EOF is an escape with a code point, so
    // there is no error.
    assert_eq!(events("\\41"), []);
    assert_eq!(events("\\0"), []);
}

#[test]
fn a_backslash_before_a_newline_outside_a_string_is_a_delim_and_a_parse_error() {
    // 4.3.1: "Otherwise, this is a parse error. Return a <delim-token>".
    for (text, offset) in [
        ("\\\n", 0),
        ("a\\\n", 1),
        ("#\\\n", 1),
        ("1\\\n", 1),
        (" \\\n", 1),
    ] {
        let (_, errors) = lex_all(text);
        assert_eq!(
            errors,
            [event("escaped_newline", offset..offset + 1)],
            "{text:?}"
        );
    }
    assert_eq!(
        tokens("#\\\n"),
        ["Delim('#')", "Delim('\\\\')", "Whitespace"]
    );
}

#[test]
fn preprocessing_turns_carriage_returns_and_form_feeds_into_line_feeds_and_keeps_original_bytes() {
    // 3.3: CR, FF and CR LF each become one LF. A newline is whitespace (4.2)
    // and ends a string (4.3.5); the interval keeps every original byte.
    let (items, _) = lex_all("a\rb\r\nc\u{c}d");
    assert_eq!(
        items
            .iter()
            .map(|item| item.token.as_str())
            .collect::<Vec<_>>(),
        [
            "Ident(\"a\")",
            "Whitespace",
            "Ident(\"b\")",
            "Whitespace",
            "Ident(\"c\")",
            "Whitespace",
            "Ident(\"d\")"
        ]
    );
    assert_eq!(items[1].range, 1..2);
    assert_eq!(items[3].range, 3..5);
    assert_eq!(items[5].range, 6..7);
    // Consecutive carriage returns are separate line feeds, in one run.
    let (items, _) = lex_all("\r\r\n\r");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].range, 0..4);

    // A CR LF pair is one newline inside a string too, so it ends the string.
    let (items, errors) = lex_all("'a\r\nb'");
    assert_eq!(items[0].token, "BadString");
    assert_eq!(items[0].range, 0..2);
    assert_eq!(items[1].token, "Whitespace");
    assert_eq!(items[1].range, 2..4);
    assert_eq!(errors[0], event("bad_string", 0..2));

    // And a backslash before CR LF continues the string, consuming both bytes.
    let (items, errors) = lex_all("'a\\\r\nb'");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].token, "String(\"ab\")");
    assert_eq!(items[0].range, 0..7);
    assert!(errors.is_empty());

    // A backslash before CR is not a valid escape.
    let (items, errors) = lex_all("\\\r");
    assert_eq!(items[0].token, "Delim('\\\\')");
    assert_eq!(items[1].range, 1..2);
    assert_eq!(errors, [event("escaped_newline", 0..1)]);

    // FF and CR are whitespace in a url.
    let (items, _) = lex_all("url(\ra\u{c})");
    assert_eq!(items[0].token, "Url(\"a\")");
    let (items, _) = lex_all("url(a\rb)");
    assert_eq!(items[0].token, "BadUrl");
}

#[test]
fn preprocessing_turns_null_into_the_replacement_character_which_is_an_ident_code_point() {
    // 3.3: NULL becomes U+FFFD. U+FFFD is non-ASCII, so it is an ident code
    // point (4.2); it is not a non-printable code point in a url.
    let (items, _) = lex_all("a\0b");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].token, "Ident(\"a\u{fffd}b\")");
    assert_eq!(items[0].range, 0..3);
    check(&[
        ("\0", &["Ident(\"\u{fffd}\")"]),
        ("#\0", &["Hash(\"\u{fffd}\",id)"]),
        ("\"a\0b\"", &["String(\"a\u{fffd}b\")"]),
        ("url(a\0b)", &["Url(\"a\u{fffd}b\")"]),
        ("/*\0*/", &["Comment"]),
        ("1\0", &["Dimension(\"1\",int,\"\u{fffd}\")"]),
    ]);
    // A NULL after a backslash is the escaped code point.
    assert_eq!(tokens("\\\0"), ["Ident(\"\u{fffd}\")"]);
}

// ---------------------------------------------------------------- 4.3.1 and 4.3.14: unicode ranges

#[test]
fn a_unicode_range_is_produced_only_when_allowed_and_the_input_starts_one() {
    // 4.3.1 and 4.3.11.
    assert_eq!(tokens_with_ranges("U+26"), ["UnicodeRange(0x26,0x26)"]);
    assert_eq!(tokens_with_ranges("u+26"), ["UnicodeRange(0x26,0x26)"]);
    assert_eq!(tokens_with_ranges("u+a"), ["UnicodeRange(0xa,0xa)"]);
    // Not allowed (the default), the same text is an ident and a number.
    assert_eq!(tokens("U+26"), ["Ident(\"U\")", "Number(\"+26\",int)"]);
    assert_eq!(
        tokens("u+a"),
        ["Ident(\"u\")", "Delim('+')", "Ident(\"a\")"]
    );
    // Allowed, an input that does not start a unicode-range is unchanged.
    for text in ["url", "unicode", "U", "U+", "U+g", "Ux+1", "u-1"] {
        assert_eq!(tokens_with_ranges(text), tokens(text), "{text:?}");
    }
    assert_eq!(tokens_with_ranges("U+"), ["Ident(\"U\")", "Delim('+')"]);
    assert_eq!(
        tokens_with_ranges("U+g"),
        ["Ident(\"U\")", "Delim('+')", "Ident(\"g\")"]
    );
}

#[test]
fn a_unicode_range_is_up_to_six_hex_digits_question_marks_and_an_optional_end() {
    // 4.3.14.
    let cases: &[(&str, &[&str])] = &[
        ("U+0-7F", &["UnicodeRange(0x0,0x7f)"]),
        ("U+0025-00FF", &["UnicodeRange(0x25,0xff)"]),
        ("U+4??", &["UnicodeRange(0x400,0x4ff)"]),
        ("U+???", &["UnicodeRange(0x0,0xfff)"]),
        ("U+??????", &["UnicodeRange(0x0,0xffffff)"]),
        ("U+1??", &["UnicodeRange(0x100,0x1ff)"]),
        ("U+aBc", &["UnicodeRange(0xabc,0xabc)"]),
        ("U+110000", &["UnicodeRange(0x110000,0x110000)"]),
        ("U+110000-110001", &["UnicodeRange(0x110000,0x110001)"]),
        // The end may be below the start: the token represents an empty range.
        ("U+20-10", &["UnicodeRange(0x20,0x10)"]),
        // At most six digits: the seventh is a separate number token.
        (
            "U+1234567",
            &["UnicodeRange(0x123456,0x123456)", "Number(\"7\",int)"],
        ),
        // Question marks fill up to six places and nothing more.
        (
            "U+12345??",
            &["UnicodeRange(0x123450,0x12345f)", "Delim('?')"],
        ),
        // A first segment with a question mark ends the token: no end part.
        ("U+4?-5", &["UnicodeRange(0x40,0x4f)", "Number(\"-5\",int)"]),
        // A hyphen needs a hex digit after it.
        ("U+1-", &["UnicodeRange(0x1,0x1)", "Delim('-')"]),
        ("U+1-g", &["UnicodeRange(0x1,0x1)", "Ident(\"-g\")"]),
        ("U+1-2-3", &["UnicodeRange(0x1,0x2)", "Number(\"-3\",int)"]),
        // Digits after a question mark are not part of the first segment.
        ("U+1?2", &["UnicodeRange(0x10,0x1f)", "Number(\"2\",int)"]),
        ("U+4??x", &["UnicodeRange(0x400,0x4ff)", "Ident(\"x\")"]),
        (
            "U+26 U+27",
            &[
                "UnicodeRange(0x26,0x26)",
                "Whitespace",
                "UnicodeRange(0x27,0x27)",
            ],
        ),
        (
            "a,U+26",
            &["Ident(\"a\")", "Comma", "UnicodeRange(0x26,0x26)"],
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(tokens_with_ranges(text), *expected, "{text:?}");
    }
    let (items, _) = lex_with("x U+4??-7", &Options::default().unicode_ranges(true));
    assert_eq!(items[2].range, 2..7);
    assert_eq!(items[3].token, "Number(\"-7\",int)");
}

// ---------------------------------------------------------------- positions and options

#[test]
fn the_origin_rebases_every_offset_and_error_range() {
    let options = Options::default().origin(Offset::new(40));
    assert_eq!(options.origin_offset(), Offset::new(40));
    let (items, errors) = lex_with("a b /* c", &options);
    assert_eq!(items[0].range, 40..41);
    assert_eq!(items[1].range, 41..42);
    assert_eq!(items[2].range, 42..43);
    assert_eq!(items[3].range, 43..44);
    assert_eq!(items[4].range, 44..48);
    assert_eq!(errors, [event("unterminated_comment", 44..48)]);
    // A sub-range re-lexed alone with unicode ranges allowed reports the
    // positions it had in the whole text (5.5.6 step 8).
    let whole = "src: U+26; unicode-range: U+4??;";
    let start = whole.find("U+4").unwrap();
    let value = &whole[start..whole.len() - 1];
    let (items, _) = lex_with(
        value,
        &Options::default()
            .unicode_ranges(true)
            .origin(Offset::new(start)),
    );
    assert_eq!(items[0].token, "UnicodeRange(0x400,0x4ff)");
    assert_eq!(items[0].range, start..start + 5);
}

#[test]
fn the_lexemes_of_any_text_tile_it() {
    for text in [
        "a{b:c;d:e(f,g)}",
        "@media (min-width:1px){a{color:#fff}}",
        "/* x */ a /* y",
        "url( a ) url(\"b\") url(c d) url(",
        "'a\\\nb' \"c\nd\" 'e",
        "\\ \\\n \\",
        "1.5e3 -.5 +1 1e 1e+ 1px 5% U+26",
        "<!-- --> <!- -- -",
        "a\u{0}b\r\nc\u{c}d",
        "\u{e9}\u{1f600}\u{fffd}",
        "#\\31 23 @\\ x",
    ] {
        assert_tiles(text);
    }
}
