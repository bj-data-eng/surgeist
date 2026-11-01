//! Literal source oracles for Syntax 3 (2026-10-01) §§3.3, 4 and 5.3. The name
//! classes follow the 2021-12-24 §4.2 non-ASCII code point; the 2026 non-ASCII
//! ident code point restriction is a deferred tokenizer difference
//! (docs/reference.md#css-syntax-3-edition-differences).
//! Matching closers are group ends; bad tokens remain ordered error entries.
use super::*;
use surgeist_css_parser::document::Fault;
use surgeist_css_parser::token::{HashKind, Number};

fn source(text: &str) -> (CssSourceSnapshot, Document<'_>) {
    let snapshot = CssSourceSnapshot::new(text);
    let document = source_document(text, &snapshot, 0).unwrap();
    assert!(document.snapshot().unwrap().same_snapshot(&snapshot));
    (snapshot, document)
}

fn representation(
    document: &Document<'_>,
    node: NodeId,
    snapshot: &CssSourceSnapshot,
    spelling: &str,
    range: Range<usize>,
) {
    assert_eq!(document.spelling(node), Some(spelling));
    let CssValueOrigin::Parsed(origin) = document.origin(node) else {
        panic!("source occurrence must retain its parsed origin")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

fn closing_representation(
    document: &Document<'_>,
    group: NodeId,
    snapshot: &CssSourceSnapshot,
    spelling: &str,
    range: Range<usize>,
) {
    let closing = document.closing_site(group).unwrap();
    assert_eq!(closing.spelling(), Some(spelling));
    let CssValueOrigin::Parsed(origin) = document.closing_origin(group) else {
        panic!("actual authored matching closer")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

fn native(document: &Document<'_>, node: NodeId, expected: Token<'_>) {
    assert_eq!(document.token(node), expected);
}

fn number(representation: &str) -> Number<'_> {
    Number::parse(representation).expect("a literal number")
}

fn only_root(document: &Document<'_>) -> NodeId {
    let [node] = document.list(document.root_list()) else {
        panic!("one literal source occurrence")
    };
    *node
}

fn eof(document: &Document<'_>, list: ListId, offset: usize) {
    let mut cursor = document.cursor(list);
    for _ in document.list(list) {
        assert!(matches!(cursor.consume(), CursorItem::Node(_)));
    }
    let end = cursor.consume();
    let CursorItem::EndOfInput(boundary) = &end else {
        panic!("conceptual EOF")
    };
    assert_eq!(boundary.offset, Some(offset));
    assert_eq!(cursor.peek(), end);
    assert_eq!(cursor.consume(), end);
    assert_eq!(cursor.consume(), end);
}

fn single_native(text: &str, expected: Token<'_>) {
    let (snapshot, document) = source(text);
    let node = only_root(&document);
    native(&document, node, expected);
    representation(&document, node, &snapshot, text, 0..text.len());
    assert_eq!(
        document
            .range(node)
            .map(|range| range.start.value()..range.end.value()),
        Some(0..text.len())
    );
    eof(&document, document.root_list(), text.len());
}

fn extent(document: &Document<'_>, node: NodeId) -> Option<Range<usize>> {
    document
        .range(node)
        .map(|range| range.start.value()..range.end.value())
}

#[test]
fn source_leaf_inventory_preserves_decoded_values_and_original_representation() {
    for (text, expected) in [
        ("a", Token::Ident("a")),
        (r"\61", Token::Ident("a")),
        ("@a", Token::AtKeyword("a")),
        (r"@\61", Token::AtKeyword("a")),
        (
            "#12",
            Token::Hash {
                value: "12",
                kind: HashKind::Unrestricted,
            },
        ),
        (
            "#a",
            Token::Hash {
                value: "a",
                kind: HashKind::Id,
            },
        ),
        (
            r"#\31 x",
            Token::Hash {
                value: "1x",
                kind: HashKind::Id,
            },
        ),
        ("\"a\"", Token::String("a")),
        (r#""\61""#, Token::String("a")),
        ("url(a)", Token::Url("a")),
        (r"url(a\29 b)", Token::Url("a)b")),
        ("!", Token::Delim('!')),
        ("<!--", Token::Cdo),
        ("-->", Token::Cdc),
        (":", Token::Colon),
        (";", Token::Semicolon),
        (",", Token::Comma),
    ] {
        single_native(text, expected);
    }
    let (snapshot, document) = source("a/*x*/b");
    let [a, comment, b] = document.list(document.root_list()) else {
        panic!("comment retained between the two actual tokens")
    };
    native(&document, *a, Token::Ident("a"));
    native(&document, *b, Token::Ident("b"));
    native(&document, *comment, Token::Comment);
    representation(&document, *a, &snapshot, "a", 0..1);
    representation(&document, *comment, &snapshot, "/*x*/", 1..6);
    representation(&document, *b, &snapshot, "b", 6..7);
    eof(&document, document.root_list(), 7);
}

#[test]
fn source_groups_preserve_all_opening_and_matching_closing_payloads() {
    for (text, opening, opening_end) in [
        ("f()", Token::Function("f"), 2),
        (r"f\6f o()", Token::Function("foo"), 7),
        ("()", Token::OpenParen, 1),
        ("[]", Token::OpenSquare, 1),
        ("{}", Token::OpenCurly, 1),
    ] {
        let (snapshot, document) = source(text);
        let group = only_root(&document);
        native(&document, group, opening);
        representation(
            &document,
            group,
            &snapshot,
            &text[..opening_end],
            0..opening_end,
        );
        assert!(matches!(
            document.group_end(group),
            Some(GroupEnd::Closed { .. })
        ));
        closing_representation(
            &document,
            group,
            &snapshot,
            &text[opening_end..],
            opening_end..text.len(),
        );
        let children = document.children(group).unwrap();
        assert!(document.list(children).is_empty());
        assert_eq!(extent(&document, group), Some(0..text.len()));
        eof(&document, children, opening_end);
        eof(&document, document.root_list(), text.len());
    }
}

#[test]
fn source_unmatched_closers_remain_ordered_error_entries() {
    let (snapshot, document) = source(")]}");
    assert_eq!(document.list(document.root_list()).len(), 3);
    for (index, (expected, spelling)) in [
        (Token::CloseParen, ")"),
        (Token::CloseSquare, "]"),
        (Token::CloseCurly, "}"),
    ]
    .into_iter()
    .enumerate()
    {
        let node = document.list(document.root_list())[index];
        assert_eq!(
            document.component(node).fault(),
            Some(Fault::UnexpectedCloser)
        );
        native(&document, node, expected);
        representation(&document, node, &snapshot, spelling, index..index + 1);
        assert_eq!(extent(&document, node), Some(index..index + 1));
    }
    eof(&document, document.root_list(), 3);
}

#[test]
fn source_bad_strings_reconsume_each_preprocessed_newline() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let text = format!("\"bad{newline}x");
        let (snapshot, document) = source(&text);
        let [bad, whitespace, x] = document.list(document.root_list()) else {
            panic!("bad string, pending newline, and following identifier")
        };
        assert_eq!(document.component(*bad).fault(), Some(Fault::BadString));
        native(&document, *bad, Token::BadString);
        representation(&document, *bad, &snapshot, "\"bad", 0..4);
        native(&document, *whitespace, Token::Whitespace);
        representation(
            &document,
            *whitespace,
            &snapshot,
            newline,
            4..4 + newline.len(),
        );
        native(&document, *x, Token::Ident("x"));
        representation(
            &document,
            *x,
            &snapshot,
            "x",
            4 + newline.len()..5 + newline.len(),
        );
        eof(&document, document.root_list(), text.len());
        single_native(&format!("\"a\\{newline}b\""), Token::String("ab"));
    }
}

#[test]
fn source_bad_urls_preserve_remnants_and_real_following_boundary() {
    for (text, spelling, end) in [
        ("url(a b)tail", "url(a b)", 8),
        (r"url(a \)still)tail", r"url(a \)still)", 14),
    ] {
        let (snapshot, document) = source(text);
        let [bad, tail] = document.list(document.root_list()) else {
            panic!("one bad URL followed by the real tail token")
        };
        assert_eq!(document.component(*bad).fault(), Some(Fault::BadUrl));
        native(&document, *bad, Token::BadUrl);
        representation(&document, *bad, &snapshot, spelling, 0..end);
        assert_eq!(extent(&document, *bad), Some(0..end));
        native(&document, *tail, Token::Ident("tail"));
        representation(&document, *tail, &snapshot, "tail", end..end + 4);
        eof(&document, document.root_list(), end + 4);
    }
}

#[test]
fn source_numeric_payloads_preserve_sign_kind_units_and_negative_zero_bits() {
    for (text, sign, value, integer) in [
        ("+001", true, 1.0_f64, Some(1)),
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
        let Token::Number(actual) = document.token(node) else {
            panic!("numeric token")
        };
        assert_eq!(actual.representation(), text);
        assert_eq!(
            (
                actual.sign().is_some(),
                actual.to_f64().to_bits(),
                actual.integer()
            ),
            (sign, value.to_bits(), integer)
        );
        representation(&document, node, &snapshot, text, 0..text.len());
        eof(&document, document.root_list(), text.len());
    }
    single_native("25%", Token::Percentage(number("25")));
    for (text, representation, unit) in [
        ("+1.25PX", "+1.25", "PX"),
        ("12px", "12", "px"),
        (r"1P\58 ", "1", "PX"),
    ] {
        single_native(
            text,
            Token::Dimension {
                number: number(representation),
                unit,
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
        single_native(text, Token::Ident(text));
    }
    for (text, expected) in [
        ("/", Token::Delim('/')),
        ("@", Token::Delim('@')),
        ("`", Token::Delim('`')),
        ("\u{7f}", Token::Delim('\u{7f}')),
        ("\0", Token::Ident("\u{fffd}")),
        ("@\0", Token::AtKeyword("\u{fffd}")),
        (
            "#\0",
            Token::Hash {
                value: "\u{fffd}",
                kind: HashKind::Id,
            },
        ),
        (
            "1\0",
            Token::Dimension {
                number: number("1"),
                unit: "\u{fffd}",
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
        assert_eq!(document.component(bad).fault(), Some(Fault::BadUrl));
        native(&document, bad, Token::BadUrl);
        representation(&document, bad, &snapshot, &text, 0..text.len());
        eof(&document, document.root_list(), text.len());
    }
    for whitespace in ["\t", "\n", "\r", "\u{c}", " ", "\r\n"] {
        single_native(&format!("url({whitespace}a)"), Token::Url("a"));
    }
    for (text, value) in [
        ("url(\0a)", "\u{fffd}a"),
        ("url(\u{80}a)", "\u{80}a"),
        ("url(!a)", "!a"),
    ] {
        single_native(text, Token::Url(value));
    }
    let (snapshot, document) = source("a\r\nb");
    let [a, whitespace, b] = document.list(document.root_list()) else {
        panic!("CRLF stays one original whitespace occurrence")
    };
    native(&document, *a, Token::Ident("a"));
    native(&document, *whitespace, Token::Whitespace);
    native(&document, *b, Token::Ident("b"));
    representation(&document, *a, &snapshot, "a", 0..1);
    representation(&document, *whitespace, &snapshot, "\r\n", 1..3);
    representation(&document, *b, &snapshot, "b", 3..4);
    eof(&document, document.root_list(), 4);
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
        single_native(text, Token::Ident(value));
    }
    let (snapshot, document) = source(r"\61  b");
    let [a, whitespace, b] = document.list(document.root_list()) else {
        panic!("only one optional escape terminator is consumed")
    };
    native(&document, *a, Token::Ident("a"));
    representation(&document, *a, &snapshot, r"\61 ", 0..4);
    native(&document, *whitespace, Token::Whitespace);
    representation(&document, *whitespace, &snapshot, " ", 4..5);
    native(&document, *b, Token::Ident("b"));
    representation(&document, *b, &snapshot, "b", 5..6);
    eof(&document, document.root_list(), 6);
}

#[test]
fn source_window_and_empty_input_keep_bounded_original_eof_sites() {
    let text = "😀\r\n(a)tail";
    let snapshot = CssSourceSnapshot::new(text);
    let document = build_source(
        &text[6..9],
        6,
        &snapshot,
        SyntaxInputLimits {
            max_depth: 1,
            max_components: 2,
            max_known_spelling_bytes: 3,
        },
        0,
    )
    .unwrap();
    let before = document.metrics();
    assert_eq!(
        (
            before.components(),
            before.maximum_depth(),
            before.spelling_bytes()
        ),
        (2, 1, 3)
    );
    let group = only_root(&document);
    native(&document, group, Token::OpenParen);
    representation(&document, group, &snapshot, "(", 6..7);
    assert_eq!(extent(&document, group), Some(6..9));
    let children = document.children(group).unwrap();
    let [a] = document.list(children) else {
        panic!("bounded argument")
    };
    native(&document, *a, Token::Ident("a"));
    representation(&document, *a, &snapshot, "a", 7..8);
    assert!(matches!(
        document.group_end(group),
        Some(GroupEnd::Closed { .. })
    ));
    closing_representation(&document, group, &snapshot, ")", 8..9);
    for (origin, start_column, end_column) in [
        (document.origin(group), 0, 1),
        (document.origin(*a), 1, 2),
        (document.closing_origin(group), 2, 3),
    ] {
        let CssValueOrigin::Parsed(origin) = origin else {
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
    eof(&document, children, 8);
    eof(&document, document.root_list(), 9);
    assert_eq!(document.metrics(), before);
    let (_empty_snapshot, empty) = source("");
    assert_eq!(empty.node_count(), 0);
    assert!(empty.list(empty.root_list()).is_empty());
    assert_eq!(empty.metrics(), Default::default());
    eof(&empty, empty.root_list(), 0);
}

#[test]
fn borrowed_source_documents_read_the_callers_text_without_copying_it() {
    // The tree reads the text it was given; there is no working copy to move or
    // to copy again, however large the text is.
    for length in [4096, 131_072] {
        let original = format!("f({})g()", "x".repeat(length));
        let (snapshot, document) = source(&original);
        let retained = document.source().unwrap();
        assert_eq!(retained.as_ptr(), original.as_ptr());
        assert_eq!(retained.len(), original.len());
        let function = document
            .list(document.root_list())
            .iter()
            .copied()
            .find(|node| document.kind(*node) == TokenKind::Function("f"))
            .unwrap();
        let children = document.children(function).unwrap();
        eof(&document, children, length + 2);
        eof(&document, document.root_list(), original.len());
        assert!(document.snapshot().unwrap().same_snapshot(&snapshot));
    }
}

#[test]
fn recovery_document_owns_its_text_denies_curly_nesting_and_keeps_explicit_closing_sites() {
    // A recovery document outlives the call that built it, so it owns the text
    // it is given, moved and not copied; a `{}` block nested past the structural
    // ceiling stays one opaque denied boundary and the rule after it is
    // independent.
    let length = 65_536;
    let original = format!("({{{}}})f(x)tail", "x".repeat(length));
    let snapshot = CssSourceSnapshot::new(&original);
    let bytes = original.len();
    // One level of room is left under the ceiling: the parenthesis takes it.
    let working = original.clone();
    let allocation = working.as_ptr();
    let document = recovery_source_document(working, &snapshot, 255).unwrap();
    let [group, function, tail] = document.list(document.root_list()) else {
        panic!("one block, one real function and its following identifier")
    };
    let [denied] = document.list(document.children(*group).unwrap()) else {
        panic!("one denied boundary inside the block")
    };
    assert!(matches!(
        document.component(*denied),
        Component::Denied { .. }
    ));
    assert!(document.denied_error(*denied).is_some());
    assert_eq!(extent(&document, *denied), Some(1..length + 3));
    assert_eq!(document.kind(*tail), TokenKind::Ident("tail"));
    let children = document.children(*function).unwrap();
    eof(&document, children, length + 7);
    eof(&document, document.root_list(), bytes);
    assert_eq!(document.source(), Some(original.as_str()));
    assert_eq!(
        document.source().unwrap().as_ptr(),
        allocation,
        "the document reads the allocation it was given"
    );
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
        let [left, right] = document.list(document.root_list()) else {
            panic!("two source delimiters for {text}")
        };
        native(&document, *left, Token::Delim(first));
        native(&document, *right, Token::Delim('='));
        representation(&document, *left, &snapshot, &text[..1], 0..1);
        representation(&document, *right, &snapshot, "=", 1..2);
        assert_eq!(extent(&document, *left), Some(0..1));
        assert_eq!(extent(&document, *right), Some(1..2));
        let metrics = document.metrics();
        assert_eq!(
            (
                metrics.components(),
                metrics.maximum_depth(),
                metrics.spelling_bytes()
            ),
            (2, 0, 2)
        );
        eof(&document, document.root_list(), 2);
    }
}
