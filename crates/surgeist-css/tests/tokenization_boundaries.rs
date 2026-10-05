#![forbid(unsafe_code)]
//! Literal expectations follow the pinned CSS Syntax 3 CRD (2021-12-24):
//! §§3.3, 4.3.1, 4.3.3–4.3.11 and 4.3.14. Token values and hash flags are
//! public component contracts; lexical parse errors must also make an authored
//! recovery report non-clean. In particular, an escaped EOF returns U+FFFD
//! *and* a parse error (§4.3.7), whereas backslash-newline returns a delimiter
//! and a parse error (§4.3.1).
//! <https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/>

use surgeist_css::{
    CssComponentValueErrorKind, CssComponentValueRef, CssDeclarationList, CssHashFlag,
    CssValueOrigin, CssValueTokenRef, parse_component_values, parse_style_attribute,
    validate_style_attribute,
};

#[derive(Debug)]
enum ExpectedToken<'a> {
    Ident(&'a str),
    AtKeyword(&'a str),
    Hash(&'a str, CssHashFlag),
    String(&'a str),
    Url(&'a str),
    Delim(char),
    Number(&'a str),
    Dimension(&'a str, &'a str),
    Whitespace(&'a str),
    Cdc,
}

fn assert_tokens(source: &str, expected: &[ExpectedToken<'_>]) {
    let values = parse_component_values(source).expect("valid component token sequence");
    assert_eq!(values.items().len(), expected.len(), "{source:?}");
    for (component, expected) in values.items().iter().zip(expected) {
        let CssComponentValueRef::Token(actual) = component.view() else {
            panic!(
                "expected {expected:?}, got {:?}: {source:?}",
                component.view()
            );
        };
        match (actual, expected) {
            (CssValueTokenRef::Ident(actual), ExpectedToken::Ident(expected))
            | (CssValueTokenRef::AtKeyword(actual), ExpectedToken::AtKeyword(expected))
            | (CssValueTokenRef::String(actual), ExpectedToken::String(expected))
            | (CssValueTokenRef::Url(actual), ExpectedToken::Url(expected))
            | (CssValueTokenRef::Whitespace(actual), ExpectedToken::Whitespace(expected)) => {
                assert_eq!(actual, *expected, "{source:?}");
            }
            (
                CssValueTokenRef::Hash { value, flag },
                ExpectedToken::Hash(expected, expected_flag),
            ) => {
                assert_eq!(value, *expected, "{source:?}");
                assert_eq!(flag, *expected_flag, "{source:?}");
            }
            (CssValueTokenRef::Delim(actual), ExpectedToken::Delim(expected)) => {
                assert_eq!(actual, *expected, "{source:?}");
            }
            (CssValueTokenRef::Number(actual), ExpectedToken::Number(expected)) => {
                assert_eq!(actual.representation(), *expected, "{source:?}");
            }
            (
                CssValueTokenRef::Dimension { number, unit },
                ExpectedToken::Dimension(expected, expected_unit),
            ) => {
                assert_eq!(number.representation(), *expected, "{source:?}");
                assert_eq!(unit, *expected_unit, "{source:?}");
            }
            (CssValueTokenRef::Cdc, ExpectedToken::Cdc) => {}
            (actual, expected) => panic!("expected {expected:?}, got {actual:?}: {source:?}"),
        }
    }
}

fn assert_custom_values(declarations: &CssDeclarationList, expected: &[(&str, &str)]) {
    assert_eq!(declarations.len(), expected.len());
    for (declaration, (name, value)) in declarations.iter().zip(expected) {
        let custom = declaration.custom().expect("retained custom declaration");
        assert_eq!(custom.name().as_str(), *name);
        assert_eq!(
            custom
                .value()
                .value()
                .expect("ordinary custom value")
                .as_css(),
            *value
        );
    }
}

fn assert_clean_custom(source: &str, expected: &[(&str, &str)]) {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
    assert_custom_values(report.syntax(), expected);
    assert_eq!(
        validate_style_attribute(source),
        Ok(report.syntax().clone())
    );
}

fn assert_retained_lexical_error(source: &str, expected: &[(&str, &str)]) {
    let report = parse_style_attribute(source);
    assert_custom_values(report.syntax(), expected);
    assert!(
        !report.diagnostics().is_empty(),
        "a required lexical parse error must remain observable: {source:?}"
    );
    assert!(!report.is_clean());
    let failure = validate_style_attribute(source).expect_err("lexical recovery is not clean CSS");
    assert_eq!(failure.diagnostics(), report.diagnostics());
}

#[test]
fn backslash_newline_custom_values_retain_siblings_but_require_diagnostics() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let source = format!("--x:\\{newline};--after:kept");
        assert_retained_lexical_error(&source, &[("--x", "\\"), ("--after", "kept")]);
    }
}

#[test]
fn identifier_eof_escape_retains_replacement_but_requires_a_diagnostic() {
    assert_tokens("a\\", &[ExpectedToken::Ident("a\u{fffd}")]);
    assert_retained_lexical_error("--x:a\\", &[("--x", "a\\")]);
}

#[test]
fn at_keyword_eof_escape_retains_replacement_but_requires_a_diagnostic() {
    assert_tokens("@a\\", &[ExpectedToken::AtKeyword("a\u{fffd}")]);
    assert_retained_lexical_error("--x:@a\\", &[("--x", "@a\\")]);
}

#[test]
fn hash_eof_escape_retains_replacement_but_requires_a_diagnostic() {
    assert_tokens("#a\\", &[ExpectedToken::Hash("a\u{fffd}", CssHashFlag::Id)]);
    assert_retained_lexical_error("--x:#a\\", &[("--x", "#a\\")]);
}

#[test]
fn dimension_eof_escape_retains_replacement_but_requires_a_diagnostic() {
    assert_tokens("1p\\", &[ExpectedToken::Dimension("1", "p\u{fffd}")]);
    assert_retained_lexical_error("--x:1p\\", &[("--x", "1p\\")]);
}

#[test]
fn leading_eof_escape_is_an_identifier_replacement_with_a_diagnostic() {
    assert_tokens("\\", &[ExpectedToken::Ident("\u{fffd}")]);
    assert_retained_lexical_error("--x:\\", &[("--x", "\\")]);
}

#[test]
fn hyphen_before_invalid_escape_is_a_delimiter_and_preserves_newline() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        assert_tokens(
            &format!("-\\{newline}"),
            &[
                ExpectedToken::Delim('-'),
                ExpectedToken::Delim('\\'),
                ExpectedToken::Whitespace(newline),
            ],
        );
    }
}

#[test]
fn hash_before_invalid_escape_is_unrestricted_and_preserves_newline() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        assert_tokens(
            &format!("#-\\{newline}"),
            &[
                ExpectedToken::Hash("-", CssHashFlag::Unrestricted),
                ExpectedToken::Delim('\\'),
                ExpectedToken::Whitespace(newline),
            ],
        );
    }
}

#[test]
fn hash_id_flag_uses_authored_start_instead_of_decoded_leading_digit() {
    for (source, value, flag) in [
        ("#1", "1", CssHashFlag::Unrestricted),
        (r"#\31", "1", CssHashFlag::Id),
        ("#abc", "abc", CssHashFlag::Id),
        ("#--", "--", CssHashFlag::Id),
        ("#-", "-", CssHashFlag::Unrestricted),
        (r"#-\31", "-1", CssHashFlag::Id),
    ] {
        assert_tokens(source, &[ExpectedToken::Hash(value, flag)]);
    }
}

#[test]
fn identifier_lookahead_consumes_maximal_names_and_reconsumes_nonmembers() {
    for (source, expected) in [
        ("--", "--"),
        ("-a", "-a"),
        (r"-\61", "-a"),
        (r"\31 abc9_😀", "1abc9_😀"),
    ] {
        assert_tokens(source, &[ExpectedToken::Ident(expected)]);
    }
    assert_tokens(
        r"-\61 bc9_😀!tail",
        &[
            ExpectedToken::Ident("-abc9_😀"),
            ExpectedToken::Delim('!'),
            ExpectedToken::Ident("tail"),
        ],
    );
    assert_tokens("-->", &[ExpectedToken::Cdc]);
    assert_tokens("--x", &[ExpectedToken::Ident("--x")]);
    assert_tokens("-", &[ExpectedToken::Delim('-')]);
}

#[test]
fn numeric_lookahead_keeps_signed_dot_numbers_distinct_from_delimiters() {
    for source in [".5", "+.5", "-.5", "5", "+5", "-5"] {
        assert_tokens(source, &[ExpectedToken::Number(source)]);
    }
    assert_tokens(
        "+.",
        &[ExpectedToken::Delim('+'), ExpectedToken::Delim('.')],
    );
    assert_tokens(
        "-.",
        &[ExpectedToken::Delim('-'), ExpectedToken::Delim('.')],
    );
    assert_tokens(
        ".a",
        &[ExpectedToken::Delim('.'), ExpectedToken::Ident("a")],
    );
}

#[test]
fn complete_escapes_and_plain_eof_names_are_clean_authored_values() {
    for value in [
        "plain",
        "@plain",
        "#plain",
        "1px",
        r"a\!",
        r"a\\",
        r"\61 ",
        r"\000061 ",
    ] {
        // The final space belongs to the hex escape. A following delimiter keeps
        // it inside the retained authored value rather than trailing trivia.
        let value = format!("{value},");
        let source = format!("--x:{value};--after:kept");
        assert_clean_custom(&source, &[("--x", &value), ("--after", "kept")]);
    }
    assert_clean_custom("--x:plain", &[("--x", "plain")]);
    assert_tokens(r"a\!", &[ExpectedToken::Ident("a!")]);
    assert_tokens(r"a\\", &[ExpectedToken::Ident("a\\")]);
}

#[test]
fn hex_escapes_consume_one_to_six_digits_and_replace_invalid_scalars() {
    for (source, expected) in [
        (r"\6 z", "\u{6}z"),
        (r"\61 z", "az"),
        (r"\061 z", "az"),
        (r"\0061 z", "az"),
        (r"\00061 z", "az"),
        (r"\000061z", "az"),
        (r"\0000612", "a2"),
        (r"\0 ", "\u{fffd}"),
        (r"\D800 ", "\u{fffd}"),
        (r"\DFFF ", "\u{fffd}"),
        (r"\110000 ", "\u{fffd}"),
        (r"\10FFFF ", "\u{10ffff}"),
        (r"\!", "!"),
    ] {
        assert_tokens(source, &[ExpectedToken::Ident(expected)]);
    }
    assert_tokens(
        r"\000061  b",
        &[
            ExpectedToken::Ident("a"),
            ExpectedToken::Whitespace(" "),
            ExpectedToken::Ident("b"),
        ],
    );
    for whitespace in [" ", "\t", "\n", "\r", "\r\n", "\u{c}"] {
        assert_tokens(&format!("\\61{whitespace}b"), &[ExpectedToken::Ident("ab")]);
    }
}

#[test]
fn quoted_url_dispatch_preserves_function_name_and_string_arguments() {
    for (source, name, whitespace) in [
        ("url(\"a\")", "url", None),
        ("URL('a')", "URL", None),
        ("uRl( \"a\")", "uRl", Some(" ")),
        (r"\75 rl( 'a')", "url", Some(" ")),
    ] {
        let values = parse_component_values(source).expect("quoted URL is a function");
        let [component] = values.items() else {
            panic!("one function: {source:?}");
        };
        let CssComponentValueRef::Function(function) = component.view() else {
            panic!("quoted URL function: {source:?}");
        };
        assert_eq!(function.name(), name);
        let arguments = function.values().items();
        match whitespace {
            None => assert_eq!(arguments.len(), 1, "{source:?}"),
            Some(expected) => {
                assert_eq!(arguments.len(), 2, "{source:?}");
                assert!(
                    matches!(arguments[0].view(), CssComponentValueRef::Token(CssValueTokenRef::Whitespace(actual)) if actual == expected)
                );
            }
        }
        assert!(matches!(
            arguments.last().unwrap().view(),
            CssComponentValueRef::Token(CssValueTokenRef::String("a"))
        ));
    }
}

#[test]
fn unquoted_urls_consume_payload_whitespace_escapes_and_eof() {
    for (source, expected) in [
        ("URL(  a  )", "a"),
        (r"url(a\)b)", "a)b"),
        (r"url(a\29 b)", "a)b"),
        ("url([a]{b}/*c*/)", "[a]{b}/*c*/"),
        ("url(a", "a"),
        ("url(a\\", "a\u{fffd}"),
    ] {
        assert_tokens(source, &[ExpectedToken::Url(expected)]);
    }
    assert_clean_custom(
        "--x:url(a);--after:kept",
        &[("--x", "url(a)"), ("--after", "kept")],
    );
}

#[test]
fn bad_url_remnants_skip_escaped_closers_and_preserve_following_declaration() {
    for bad_url in [r"url(a b\)still)", r"url(a b\29 still)"] {
        let source = format!("--bad:{bad_url};--after:kept");
        let report = parse_style_attribute(&source);
        assert_custom_values(report.syntax(), &[("--after", "kept")]);
        assert!(!report.is_clean());
        assert!(validate_style_attribute(&source).is_err());
        let failure =
            parse_component_values(bad_url).expect_err("post-whitespace URL payload is bad");
        assert_eq!(failure.kind(), CssComponentValueErrorKind::BadUrl);
        let CssValueOrigin::Parsed(origin) = failure.origin() else {
            panic!("authored bad URL origin");
        };
        assert_eq!(origin.source().as_str(), bad_url);
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), bad_url.len());
    }
    for bad_url in [
        "url(a\"b)",
        "url(a(b)",
        "url(a\u{7}b)",
        "url(a\\\nb)",
        "url(a b",
    ] {
        assert_eq!(
            parse_component_values(bad_url).unwrap_err().kind(),
            CssComponentValueErrorKind::BadUrl,
            "{bad_url:?}"
        );
    }
}

#[test]
fn raw_string_newlines_are_bad_strings_and_leave_a_surviving_sibling() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let value = format!("\"a{newline}");
        let failure =
            parse_component_values(&value).expect_err("raw newline terminates a bad string");
        assert_eq!(failure.kind(), CssComponentValueErrorKind::BadString);
        let CssValueOrigin::Parsed(origin) = failure.origin() else {
            panic!("authored bad string origin");
        };
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            2,
            "newline is reconsumed"
        );
        let source = format!("--bad:{value};--after:kept");
        let report = parse_style_attribute(&source);
        assert_custom_values(report.syntax(), &[("--after", "kept")]);
        assert!(!report.is_clean());
        assert!(validate_style_attribute(&source).is_err());
    }
}

#[test]
fn string_backslash_newlines_are_clean_continuations_and_eof_keeps_payload() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let value = format!("\"a\\{newline}b\"");
        assert_tokens(&value, &[ExpectedToken::String("ab")]);
        let source = format!("--x:{value};--after:kept");
        assert_clean_custom(&source, &[("--x", &value), ("--after", "kept")]);
    }
    assert_tokens(r#""a\61 b""#, &[ExpectedToken::String("aab")]);
    // Payload only: the accepted EOF-string diagnostic correction is absent
    // from this historical baseline, so this test makes no report assertion.
    assert_tokens("\"a", &[ExpectedToken::String("a")]);
    assert_tokens("\"a\\", &[ExpectedToken::String("a")]);
}
