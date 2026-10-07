//! Literal source oracles for Syntax 3 (2021-12-24) §§3.3, 4, 4.2, and 5.2.
//! Matching closers are group ends; bad tokens remain ordered error entries.
use super::*;

fn source(text: &str) -> (CssSourceSnapshot, SyntaxDocument<'static>) {
    let snapshot = CssSourceSnapshot::new(text);
    let document = source_document(text, &snapshot, 0).unwrap();
    assert!(
        document
            .source
            .as_ref()
            .unwrap()
            .original
            .same_snapshot(&snapshot)
    );
    (snapshot, document)
}

fn representation(
    token: &SyntaxToken<'_>,
    snapshot: &CssSourceSnapshot,
    spelling: &str,
    range: Range<usize>,
) {
    assert_eq!(token.spelling.as_deref(), Some(spelling));
    let CssValueOrigin::Parsed(origin) = token.origin.as_ref() else {
        panic!("source occurrence must retain its parsed origin")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

fn native(token: &SyntaxToken<'_>, expected: Token<'_>) {
    let TokenPayload::Native(actual) = &token.payload else {
        panic!("source must retain its native token payload")
    };
    assert_eq!(actual, &expected);
}

fn only_root(document: &SyntaxDocument<'_>) -> NodeId {
    let [node] = document.lists[document.root].as_slice() else {
        panic!("one literal source occurrence")
    };
    *node
}

fn eof(document: &SyntaxDocument<'_>, list: ListId, snapshot: &CssSourceSnapshot, offset: usize) {
    let mut cursor = document.cursor(list);
    for _ in &document.lists[list] {
        assert!(matches!(cursor.consume(), CursorItem::Node(_)));
    }
    let end = cursor.consume();
    let CursorItem::EndOfInput(boundary) = &end else {
        panic!("conceptual EOF")
    };
    let site = boundary.source.as_ref().expect("real bounded source site");
    assert!(site.snapshot.same_snapshot(snapshot));
    assert_eq!(site.offset, offset);
    assert_eq!(cursor.peek(), end);
    cursor.reconsume_current();
    assert_eq!(cursor.consume(), end);
    assert_eq!(cursor.consume(), end);
}

fn single_native(text: &str, expected: Token<'_>) {
    let (snapshot, document) = source(text);
    let node = only_root(&document);
    native(document.nodes[node].token(), expected);
    representation(document.nodes[node].token(), &snapshot, text, 0..text.len());
    assert_eq!(document.range(node), Some(0..text.len()));
    eof(&document, document.root, &snapshot, text.len());
}

#[test]
fn source_leaf_inventory_preserves_decoded_values_and_original_representation() {
    for (text, expected) in [
        ("a", Token::Ident("a".into())),
        (r"\61", Token::Ident("a".into())),
        ("@a", Token::AtKeyword("a".into())),
        (r"@\61", Token::AtKeyword("a".into())),
        ("#12", Token::Hash("12".into())),
        ("#a", Token::IDHash("a".into())),
        (r"#\31 x", Token::IDHash("1x".into())),
        ("\"a\"", Token::QuotedString("a".into())),
        (r#""\61""#, Token::QuotedString("a".into())),
        ("url(a)", Token::UnquotedUrl("a".into())),
        (r"url(a\29 b)", Token::UnquotedUrl("a)b".into())),
        ("!", Token::Delim('!')),
        ("<!--", Token::CDO),
        ("-->", Token::CDC),
        (":", Token::Colon),
        (";", Token::Semicolon),
        (",", Token::Comma),
    ] {
        single_native(text, expected);
    }
    let (snapshot, document) = source("a/*x*/b");
    let [a, comment, b] = document.lists[document.root].as_slice() else {
        panic!("comment retained between the two actual tokens")
    };
    native(document.nodes[*a].token(), Token::Ident("a".into()));
    native(document.nodes[*b].token(), Token::Ident("b".into()));
    assert!(
        matches!(&document.nodes[*comment], SyntaxNode::Comment(token)
        if matches!(&token.payload, TokenPayload::Comment(value) if value.as_ref() == "x"))
    );
    representation(document.nodes[*a].token(), &snapshot, "a", 0..1);
    representation(document.nodes[*comment].token(), &snapshot, "/*x*/", 1..6);
    representation(document.nodes[*b].token(), &snapshot, "b", 6..7);
    eof(&document, document.root, &snapshot, 7);
}

#[test]
fn source_groups_preserve_all_opening_and_matching_closing_payloads() {
    for (text, opening, closing, opening_end) in [
        (
            "f()",
            Token::Function("f".into()),
            Token::CloseParenthesis,
            2,
        ),
        (
            r"f\6f o()",
            Token::Function("foo".into()),
            Token::CloseParenthesis,
            7,
        ),
        ("()", Token::ParenthesisBlock, Token::CloseParenthesis, 1),
        (
            "[]",
            Token::SquareBracketBlock,
            Token::CloseSquareBracket,
            1,
        ),
        ("{}", Token::CurlyBracketBlock, Token::CloseCurlyBracket, 1),
    ] {
        let (snapshot, document) = source(text);
        let group = only_root(&document);
        native(document.nodes[group].token(), opening);
        representation(
            document.nodes[group].token(),
            &snapshot,
            &text[..opening_end],
            0..opening_end,
        );
        let GroupEnd::Explicit(end) = document.nodes[group].end().unwrap() else {
            panic!("actual authored matching closer")
        };
        native(end, closing);
        representation(
            end,
            &snapshot,
            &text[opening_end..],
            opening_end..text.len(),
        );
        let children = document.nodes[group].children().unwrap();
        assert!(document.lists[children].is_empty());
        assert_eq!(document.range(group), Some(0..text.len()));
        eof(&document, children, &snapshot, opening_end);
        eof(&document, document.root, &snapshot, text.len());
    }
}

#[test]
fn source_unmatched_closers_remain_ordered_error_entries() {
    let (snapshot, document) = source(")]}");
    assert_eq!(document.lists[document.root].len(), 3);
    for (index, (expected, spelling)) in [
        (Token::CloseParenthesis, ")"),
        (Token::CloseSquareBracket, "]"),
        (Token::CloseCurlyBracket, "}"),
    ]
    .into_iter()
    .enumerate()
    {
        let node = document.lists[document.root][index];
        assert!(matches!(
            &document.nodes[node],
            SyntaxNode::Error {
                cause: SyntaxTokenFault::UnexpectedCloser,
                ..
            }
        ));
        native(document.nodes[node].token(), expected);
        representation(
            document.nodes[node].token(),
            &snapshot,
            spelling,
            index..index + 1,
        );
        assert_eq!(document.range(node), Some(index..index + 1));
    }
    eof(&document, document.root, &snapshot, 3);
}

#[test]
fn source_bad_strings_reconsume_each_preprocessed_newline() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let text = format!("\"bad{newline}x");
        let (snapshot, document) = source(&text);
        let [bad, whitespace, x] = document.lists[document.root].as_slice() else {
            panic!("bad string, pending newline, and following identifier")
        };
        assert!(matches!(
            &document.nodes[*bad],
            SyntaxNode::Error {
                cause: SyntaxTokenFault::BadString,
                ..
            }
        ));
        native(document.nodes[*bad].token(), Token::BadString("bad".into()));
        representation(document.nodes[*bad].token(), &snapshot, "\"bad", 0..4);
        assert!(matches!(&document.nodes[*whitespace].token().payload,
            TokenPayload::Whitespace(value) if value.as_ref() == newline));
        representation(
            document.nodes[*whitespace].token(),
            &snapshot,
            newline,
            4..4 + newline.len(),
        );
        native(document.nodes[*x].token(), Token::Ident("x".into()));
        representation(
            document.nodes[*x].token(),
            &snapshot,
            "x",
            4 + newline.len()..5 + newline.len(),
        );
        eof(&document, document.root, &snapshot, text.len());
        single_native(
            &format!("\"a\\{newline}b\""),
            Token::QuotedString("ab".into()),
        );
    }
}

#[test]
fn source_bad_urls_preserve_remnants_and_real_following_boundary() {
    for (text, contents, spelling, end) in [
        ("url(a b)tail", "a b", "url(a b)", 8),
        (r"url(a \)still)tail", r"a \)still", r"url(a \)still)", 14),
    ] {
        let (snapshot, document) = source(text);
        let [bad, tail] = document.lists[document.root].as_slice() else {
            panic!("one bad URL followed by the real tail token")
        };
        assert!(matches!(
            &document.nodes[*bad],
            SyntaxNode::Error {
                cause: SyntaxTokenFault::BadUrl,
                ..
            }
        ));
        native(document.nodes[*bad].token(), Token::BadUrl(contents.into()));
        representation(document.nodes[*bad].token(), &snapshot, spelling, 0..end);
        assert_eq!(document.range(*bad), Some(0..end));
        native(document.nodes[*tail].token(), Token::Ident("tail".into()));
        representation(
            document.nodes[*tail].token(),
            &snapshot,
            "tail",
            end..end + 4,
        );
        eof(&document, document.root, &snapshot, end + 4);
    }
}

#[test]
fn source_numeric_payloads_preserve_sign_kind_units_and_negative_zero_bits() {
    for (text, sign, value, integer) in [
        ("+001", true, 1.0_f32, Some(1)),
        ("1.0", false, 1.0, None),
        ("1e0", false, 1.0, None),
        ("1.25", false, 1.25, None),
        ("125e-2", false, 1.25, None),
        ("-0", true, -0.0, Some(0)),
        ("-0.0", true, -0.0, None),
        ("0", false, 0.0, Some(0)),
        ("9", false, 9.0, Some(9)),
    ] {
        let (snapshot, document) = source(text);
        let node = only_root(&document);
        let TokenPayload::Native(Token::Number {
            has_sign,
            value: actual,
            int_value,
        }) = &document.nodes[node].token().payload
        else {
            panic!("native numeric token")
        };
        assert_eq!(
            (*has_sign, actual.to_bits(), *int_value),
            (sign, value.to_bits(), integer)
        );
        representation(document.nodes[node].token(), &snapshot, text, 0..text.len());
        eof(&document, document.root, &snapshot, text.len());
    }
    single_native(
        "25%",
        Token::Percentage {
            has_sign: false,
            unit_value: 0.25,
            int_value: Some(25),
        },
    );
    for (text, sign, value, integer, unit) in [
        ("+1.25PX", true, 1.25, None, "PX"),
        ("12px", false, 12.0, Some(12), "px"),
        (r"1P\58 ", false, 1.0, Some(1), "PX"),
    ] {
        single_native(
            text,
            Token::Dimension {
                has_sign: sign,
                value,
                int_value: integer,
                unit: unit.into(),
            },
        );
    }
}

#[test]
fn source_name_classes_and_nul_preprocessing_preserve_original_spelling() {
    for text in [
        "A",
        "Z",
        "a",
        "z",
        "_",
        "-a",
        "a-09",
        "\u{80}",
        "\u{9f}",
        "\u{a0}",
        "\u{660}",
        "\u{10ffff}",
    ] {
        single_native(text, Token::Ident(text.into()));
    }
    for (text, expected) in [
        ("/", Token::Delim('/')),
        ("@", Token::Delim('@')),
        ("`", Token::Delim('`')),
        ("\u{7f}", Token::Delim('\u{7f}')),
        ("\0", Token::Ident("\u{fffd}".into())),
        ("@\0", Token::AtKeyword("\u{fffd}".into())),
        ("#\0", Token::IDHash("\u{fffd}".into())),
        (
            "1\0",
            Token::Dimension {
                has_sign: false,
                value: 1.0,
                int_value: Some(1),
                unit: "\u{fffd}".into(),
            },
        ),
    ] {
        single_native(text, expected);
    }
}

#[test]
fn source_url_dispatch_distinguishes_nonprintables_whitespace_and_name_content() {
    for character in ['\u{1}', '\u{8}', '\u{b}', '\u{e}', '\u{1f}', '\u{7f}'] {
        let text = format!("url({character}a)");
        let (snapshot, document) = source(&text);
        let bad = only_root(&document);
        assert!(matches!(
            &document.nodes[bad],
            SyntaxNode::Error {
                cause: SyntaxTokenFault::BadUrl,
                ..
            }
        ));
        native(
            document.nodes[bad].token(),
            Token::BadUrl(format!("{character}a").into()),
        );
        representation(document.nodes[bad].token(), &snapshot, &text, 0..text.len());
        eof(&document, document.root, &snapshot, text.len());
    }
    for whitespace in ["\t", "\n", "\r", "\u{c}", " ", "\r\n"] {
        single_native(
            &format!("url({whitespace}a)"),
            Token::UnquotedUrl("a".into()),
        );
    }
    for (text, value) in [
        ("url(\0a)", "\u{fffd}a"),
        ("url(\u{80}a)", "\u{80}a"),
        ("url(!a)", "!a"),
    ] {
        single_native(text, Token::UnquotedUrl(value.into()));
    }
    let (snapshot, document) = source("a\r\nb");
    let [a, whitespace, b] = document.lists[document.root].as_slice() else {
        panic!("CRLF stays one original whitespace occurrence")
    };
    native(document.nodes[*a].token(), Token::Ident("a".into()));
    assert!(matches!(&document.nodes[*whitespace].token().payload,
        TokenPayload::Whitespace(value) if value.as_ref() == "\r\n"));
    native(document.nodes[*b].token(), Token::Ident("b".into()));
    representation(document.nodes[*a].token(), &snapshot, "a", 0..1);
    representation(document.nodes[*whitespace].token(), &snapshot, "\r\n", 1..3);
    representation(document.nodes[*b].token(), &snapshot, "b", 3..4);
    eof(&document, document.root, &snapshot, 4);
}

#[test]
fn source_hex_escapes_keep_consumed_spelling_and_scalar_boundaries() {
    for (text, value) in [
        (r"\41 ", "A"),
        (r"\041 ", "A"),
        (r"\0041 ", "A"),
        (r"\00041 ", "A"),
        (r"\000041 ", "A"),
        (r"\39 x", "9x"),
        (r"\3a x", ":x"),
        (r"\A ", "\n"),
        (r"\F ", "\u{f}"),
        (r"\f ", "\u{f}"),
        (r"\G", "G"),
        (r"\g", "g"),
        (r"\0000417", "A7"),
        (r"\0 ", "\u{fffd}"),
        (r"\D800 ", "\u{fffd}"),
        (r"\DFFF ", "\u{fffd}"),
        (r"\110000 ", "\u{fffd}"),
        (r"\10FFFF ", "\u{10ffff}"),
    ] {
        single_native(text, Token::Ident(value.into()));
    }
    let (snapshot, document) = source(r"\61  b");
    let [a, whitespace, b] = document.lists[document.root].as_slice() else {
        panic!("only one optional escape terminator is consumed")
    };
    native(document.nodes[*a].token(), Token::Ident("a".into()));
    representation(document.nodes[*a].token(), &snapshot, r"\61 ", 0..4);
    assert!(matches!(&document.nodes[*whitespace].token().payload,
        TokenPayload::Whitespace(value) if value.as_ref() == " "));
    representation(document.nodes[*whitespace].token(), &snapshot, " ", 4..5);
    native(document.nodes[*b].token(), Token::Ident("b".into()));
    representation(document.nodes[*b].token(), &snapshot, "b", 5..6);
    eof(&document, document.root, &snapshot, 6);
}

#[test]
fn source_window_and_empty_input_keep_bounded_original_eof_sites() {
    let text = "😀\r\n(a)tail";
    let snapshot = CssSourceSnapshot::new(text);
    let document = normalize(
        SyntaxInput::Source(SourceWindow {
            text: Cow::Borrowed(text),
            range: 6..9,
            original: snapshot.clone(),
        }),
        SyntaxInputLimits {
            max_depth: 1,
            max_components: 2,
            max_known_spelling_bytes: 3,
        },
        0,
    )
    .unwrap();
    let before = document.metrics;
    assert_eq!(
        before,
        SyntaxInputMetrics {
            components: 2,
            maximum_depth: 1,
            known_spelling_bytes: 3,
            unspelled_tokens: 0,
        }
    );
    let group = only_root(&document);
    native(document.nodes[group].token(), Token::ParenthesisBlock);
    representation(document.nodes[group].token(), &snapshot, "(", 6..7);
    assert_eq!(document.range(group), Some(6..9));
    let children = document.nodes[group].children().unwrap();
    let [a] = document.lists[children].as_slice() else {
        panic!("bounded argument")
    };
    native(document.nodes[*a].token(), Token::Ident("a".into()));
    representation(document.nodes[*a].token(), &snapshot, "a", 7..8);
    let GroupEnd::Explicit(close) = document.nodes[group].end().unwrap() else {
        panic!("real close")
    };
    native(close, Token::CloseParenthesis);
    representation(close, &snapshot, ")", 8..9);
    for (token, start_column, end_column) in [
        (document.nodes[group].token(), 0, 1),
        (document.nodes[*a].token(), 1, 2),
        (close, 2, 3),
    ] {
        let CssValueOrigin::Parsed(origin) = token.origin.as_ref() else {
            panic!("parsed coordinate")
        };
        assert_eq!(
            (
                origin.span().start().line().value(),
                origin.span().start().column().value()
            ),
            (1, start_column)
        );
        assert_eq!(
            (
                origin.span().end().line().value(),
                origin.span().end().column().value()
            ),
            (1, end_column)
        );
    }
    eof(&document, children, &snapshot, 8);
    eof(&document, document.root, &snapshot, 9);
    assert_eq!(document.metrics, before);
    let (empty_snapshot, empty) = source("");
    assert!(empty.nodes.is_empty());
    assert!(empty.lists[empty.root].is_empty());
    assert_eq!(empty.metrics, SyntaxInputMetrics::default());
    eof(&empty, empty.root, &empty_snapshot, 0);
}

#[test]
fn supplied_unspelled_provider_operators_are_not_retokenized() {
    for payload in [
        Token::IncludeMatch,
        Token::DashMatch,
        Token::PrefixMatch,
        Token::SuffixMatch,
        Token::SubstringMatch,
    ] {
        let tokens = [SyntaxToken {
            payload: TokenPayload::Native(payload.clone()),
            spelling: None,
            origin: Cow::Owned(CssValueOrigin::Programmatic),
        }];
        let document = normalize(
            SyntaxInput::Tokens(&tokens),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let node = only_root(&document);
        native(document.nodes[node].token(), payload);
        assert_eq!(document.nodes[node].token().spelling, None);
        assert_eq!(
            document.nodes[node].token().origin.as_ref(),
            &CssValueOrigin::Programmatic
        );
        assert_eq!(document.range(node), None);
        assert_eq!(
            document.metrics,
            SyntaxInputMetrics {
                components: 1,
                maximum_depth: 0,
                known_spelling_bytes: 0,
                unspelled_tokens: 1
            }
        );
        let mut cursor = document.cursor(document.root);
        assert_eq!(cursor.consume(), CursorItem::Node(node));
        let CursorItem::EndOfInput(boundary) = cursor.consume() else {
            panic!("supplied-list EOF")
        };
        assert!(boundary.source.is_none());
    }
}

#[test]
fn source_match_operators_are_two_delimiter_occurrences_with_real_ranges() {
    // Syntax 3 §4's vocabulary and §4.3.1's default dispatch give two delimiters.
    // Provider-combined match variants have no source-produced token identity.
    for (text, first) in [
        ("~=", '~'),
        ("|=", '|'),
        ("^=", '^'),
        ("$=", '$'),
        ("*=", '*'),
    ] {
        let (snapshot, document) = source(text);
        let [left, right] = document.lists[document.root].as_slice() else {
            panic!("two source delimiters for {text}")
        };
        native(document.nodes[*left].token(), Token::Delim(first));
        native(document.nodes[*right].token(), Token::Delim('='));
        representation(document.nodes[*left].token(), &snapshot, &text[..1], 0..1);
        representation(document.nodes[*right].token(), &snapshot, "=", 1..2);
        assert_eq!(document.range(*left), Some(0..1));
        assert_eq!(document.range(*right), Some(1..2));
        assert_eq!(
            document.metrics,
            SyntaxInputMetrics {
                components: 2,
                maximum_depth: 0,
                known_spelling_bytes: 2,
                unspelled_tokens: 0
            }
        );
        eof(&document, document.root, &snapshot, 2);
    }
}
