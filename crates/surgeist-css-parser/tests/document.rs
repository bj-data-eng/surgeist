//! Tests for the document: the token tree [`Document::parse`] builds from text
//! and a [`TreeBuilder`] builds from components, its limits and depth policy,
//! and its navigation.
//!
//! Every expectation is written by hand from CSS Syntax Module Level 3 of the
//! pinned edition (sections 3.3, 4.3 and 5.2, and the consume algorithms of
//! 5.5.7 to 5.5.10), never from running the code under test. The tokens of a
//! literal text are the ones section 4.3 gives; the tree around them is the one
//! 5.5.7 to 5.5.10 give: a closing token ends the group it matches, a closing
//! token that matches no open group is a preserved token, and the end of the
//! input ends every group still open.
//!
//! Section 4.2's non-ASCII ident code points are the 2021 rule in this crate
//! for now, so these tests use only non-ASCII code points that both editions
//! admit.

mod common;

use common::tree::{outline, outline_of, parse, span};
use common::{render, tokens};
use surgeist_css_parser::document::{
    BlockKind, BuildError, Component, Fault, GroupEnd, GroupKind, ListId, NodeId, TreeBuilder,
};
use surgeist_css_parser::lexer::Options;
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::token::{NumberKind, Sign, TokenKind};
use surgeist_css_parser::{Document, Offset, Token};

fn parse_with<'a>(
    text: &'a str,
    options: &Options,
    limits: Limits,
    excess: DepthExcess,
) -> Result<Document<'a>, BuildError> {
    Document::parse(text, options, limits, excess)
}

fn only(document: &Document<'_>) -> NodeId {
    let [node] = document.list(document.root_list()) else {
        panic!("one node in the root list of {:?}", document.source());
    };
    *node
}

fn root_nodes(document: &Document<'_>) -> Vec<NodeId> {
    document.list(document.root_list()).to_vec()
}

fn token_of(document: &Document<'_>, node: NodeId) -> String {
    render(document.token(node))
}

/// The end of the root list, read through an empty-handed stream: the end site
/// of the root stream.
fn root_end(document: &Document<'_>) -> Option<usize> {
    document.root().end_site().offset().map(Offset::value)
}

/// Asserts the node holds `expected`, was written as `spelling`, and covers
/// `range`.
fn assert_node(
    document: &Document<'_>,
    node: NodeId,
    expected: &str,
    spelling: &str,
    range: std::ops::Range<usize>,
) {
    assert_eq!(token_of(document, node), expected);
    assert_eq!(document.spelling(node), Some(spelling));
    assert_eq!(document.range(node).map(span), Some(range));
}

// ----------------------------------------------------------------- tokens as nodes

#[test]
fn every_token_of_the_inventory_is_one_node_with_its_decoded_value_and_written_spelling() {
    // Section 4.3: the decoded value of a name or string, the type flag of a
    // hash (decided from the code points after `#`, not from the decoded
    // value), and the spelling that is always the source text.
    for (text, expected) in [
        ("a", "Ident(\"a\")"),
        (r"\61", "Ident(\"a\")"),
        ("@a", "AtKeyword(\"a\")"),
        (r"@\61", "AtKeyword(\"a\")"),
        ("#12", "Hash(\"12\",unrestricted)"),
        ("#a", "Hash(\"a\",id)"),
        (r"#\31 x", "Hash(\"1x\",id)"),
        ("\"a\"", "String(\"a\")"),
        (r#""\61""#, "String(\"a\")"),
        ("url(a)", "Url(\"a\")"),
        (r"url(a\29 b)", "Url(\"a)b\")"),
        ("!", "Delim('!')"),
        ("<!--", "Cdo"),
        ("-->", "Cdc"),
        (":", "Colon"),
        (";", "Semicolon"),
        (",", "Comma"),
    ] {
        let document = parse(text);
        let node = only(&document);
        assert_node(&document, node, expected, text, 0..text.len());
        assert_eq!(root_end(&document), Some(text.len()), "{text:?}");
    }
}

#[test]
fn comments_are_nodes_between_the_tokens_they_separate() {
    let document = parse("a/*x*/b");
    let [a, comment, b] = root_nodes(&document)[..] else {
        panic!("three nodes");
    };
    assert_node(&document, a, "Ident(\"a\")", "a", 0..1);
    assert_node(&document, comment, "Comment", "/*x*/", 1..6);
    assert_node(&document, b, "Ident(\"b\")", "b", 6..7);
    assert_eq!(root_end(&document), Some(7));
}

#[test]
fn a_group_is_one_node_with_its_opening_spelling_and_a_range_through_its_closing_token() {
    // Section 5.5.9 and 5.5.10: the closing token that matches ends the group.
    // A function token includes its opening parenthesis (4.3.4), and an escaped
    // name keeps the escape in the spelling.
    for (text, opening, kind, opening_end) in [
        ("f()", "Function(\"f\")", GroupKind::Function, 2),
        (r"f\6f o()", "Function(\"foo\")", GroupKind::Function, 7),
        ("()", "OpenParen", GroupKind::Block(BlockKind::Paren), 1),
        ("[]", "OpenSquare", GroupKind::Block(BlockKind::Square), 1),
        ("{}", "OpenCurly", GroupKind::Block(BlockKind::Curly), 1),
    ] {
        let document = parse(text);
        let group = only(&document);
        assert_node(
            &document,
            group,
            opening,
            &text[..opening_end],
            0..text.len(),
        );
        assert_eq!(document.group_kind(group), Some(kind), "{text:?}");
        let Some(GroupEnd::Closed {
            range: Some(closer),
            payload: None,
        }) = document.group_end(group)
        else {
            panic!("{text:?} ends at its closing token");
        };
        assert_eq!(span(closer.clone()), opening_end..text.len(), "{text:?}");
        let children = document.children(group).expect("a group owns a list");
        assert!(document.list(children).is_empty());
        assert_eq!(document.list_end(children), Some(Offset::new(opening_end)));
        assert_eq!(root_end(&document), Some(text.len()));
        // The closing boundary reads as the matching closing token.
        let at_closer = document.node_at_offset(Offset::new(opening_end)).unwrap();
        assert!(at_closer.is_closing());
        assert_eq!(at_closer.spelling(), Some(&text[opening_end..]));
        let closing = at_closer.token().unwrap();
        assert_eq!(
            closing.kind(),
            match kind {
                GroupKind::Block(BlockKind::Square) => TokenKind::CloseSquare,
                GroupKind::Block(BlockKind::Curly) => TokenKind::CloseCurly,
                _ => TokenKind::CloseParen,
            }
        );
    }
}

#[test]
fn closing_tokens_that_close_nothing_are_preserved_in_order_as_faults() {
    // Section 5.2: `)`, `]` and `}` are always parse errors, and are preserved.
    let document = parse(")]}");
    let nodes = root_nodes(&document);
    assert_eq!(nodes.len(), 3);
    for (index, (node, expected)) in nodes
        .iter()
        .zip(["CloseParen", "CloseSquare", "CloseCurly"])
        .enumerate()
    {
        assert_node(
            &document,
            *node,
            expected,
            &")]}"[index..=index],
            index..index + 1,
        );
        assert_eq!(
            document.component(*node).fault(),
            Some(Fault::UnexpectedCloser)
        );
    }
}

#[test]
fn a_newline_ends_a_bad_string_and_is_the_whitespace_that_follows() {
    // Section 4.3.5: a newline is a parse error, the newline is reconsumed, and
    // a bad-string token is returned. Section 3.3 first makes CR, FF and CR LF
    // each one LF, which is whitespace.
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let text = format!("\"bad{newline}x");
        let document = parse(&text);
        let [bad, space, x] = root_nodes(&document)[..] else {
            panic!("a bad string, the newline, an identifier");
        };
        assert_node(&document, bad, "BadString", "\"bad", 0..4);
        assert_eq!(document.component(bad).fault(), Some(Fault::BadString));
        assert_node(
            &document,
            space,
            "Whitespace",
            newline,
            4..4 + newline.len(),
        );
        assert_node(
            &document,
            x,
            "Ident(\"x\")",
            "x",
            4 + newline.len()..5 + newline.len(),
        );
        // A backslash before a newline inside a string continues it instead.
        let continued = format!("\"a\\{newline}b\"");
        let document = parse(&continued);
        assert_node(
            &document,
            only(&document),
            "String(\"ab\")",
            &continued,
            0..continued.len(),
        );
    }
}

#[test]
fn a_bad_url_spans_its_remnants_and_the_next_token_starts_after_the_closing_parenthesis() {
    // Sections 4.3.6 and 4.3.15: whitespace followed by something other than `)`
    // makes a bad url, whose remnants end at the first unescaped `)`; an escaped
    // `)` is consumed as an escape.
    for (text, spelling, end) in [
        ("url(a b)tail", "url(a b)", 8),
        (r"url(a \)still)tail", r"url(a \)still)", 14),
    ] {
        let document = parse(text);
        let [bad, tail] = root_nodes(&document)[..] else {
            panic!("a bad url and the tail");
        };
        assert_node(&document, bad, "BadUrl", spelling, 0..end);
        assert_eq!(document.component(bad).fault(), Some(Fault::BadUrl));
        assert_node(&document, tail, "Ident(\"tail\")", "tail", end..end + 4);
        assert_eq!(root_end(&document), Some(end + 4));
    }
}

#[test]
fn a_numeric_token_keeps_its_exact_spelling_sign_type_flag_and_unit() {
    // Section 4.3.3 and 4.3.13: the type flag is "integer" unless a fraction or
    // an exponent is present; a dimension's unit and a percentage's percent
    // sign are not part of the number.
    for (text, sign, integer, value) in [
        ("+001", Some(Sign::Plus), true, 1.0_f64),
        ("1.0", None, false, 1.0),
        ("1e0", None, false, 1.0),
        ("1.25", None, false, 1.25),
        ("125e-2", None, false, 1.25),
        ("-0", Some(Sign::Minus), true, -0.0),
        ("-0.0", Some(Sign::Minus), false, -0.0),
        ("0", None, true, 0.0),
        ("9", None, true, 9.0),
    ] {
        let document = parse(text);
        let Component::Token {
            token: Token::Number(number),
            ..
        } = document.component(only(&document))
        else {
            panic!("{text:?} is a number");
        };
        assert_eq!(number.representation(), text);
        assert_eq!(number.sign(), sign, "{text:?}");
        assert_eq!(
            number.kind(),
            if integer {
                NumberKind::Integer
            } else {
                NumberKind::Number
            },
            "{text:?}"
        );
        assert_eq!(number.to_f64().to_bits(), value.to_bits(), "{text:?}");
        assert_eq!(root_end(&document), Some(text.len()));
    }
    let document = parse("25%");
    let Component::Token {
        token: Token::Percentage(number),
        ..
    } = document.component(only(&document))
    else {
        panic!("a percentage");
    };
    assert_eq!(number.representation(), "25");
    assert_eq!(document.spelling(only(&document)), Some("25%"));
    for (text, representation, unit) in [
        ("+1.25PX", "+1.25", "PX"),
        ("12px", "12", "px"),
        (r"1P\58 ", "1", "PX"),
    ] {
        let document = parse(text);
        let Component::Token {
            token:
                Token::Dimension {
                    number,
                    unit: actual,
                },
            ..
        } = document.component(only(&document))
        else {
            panic!("{text:?} is a dimension");
        };
        assert_eq!(number.representation(), representation);
        assert_eq!(actual, unit);
        assert_eq!(document.spelling(only(&document)), Some(text));
    }
}

#[test]
fn name_classes_and_null_preprocessing_decide_idents_delims_and_replacement() {
    // Section 4.2 (ASCII classes, and the non-ASCII code points both editions
    // admit) and section 3.3 (NUL is U+FFFD, an ident-start code point).
    for text in [
        "A",
        "Z",
        "a",
        "z",
        "_",
        "-a",
        "a-09",
        "\u{e9}",
        "\u{660}",
        "\u{10ffff}",
    ] {
        let document = parse(text);
        assert_node(
            &document,
            only(&document),
            &render(Token::Ident(text)),
            text,
            0..text.len(),
        );
    }
    for (text, expected) in [
        ("/", "Delim('/')"),
        ("@", "Delim('@')"),
        ("`", "Delim('`')"),
        ("\u{7f}", "Delim('\\u{7f}')"),
        ("\0", "Ident(\"\u{fffd}\")"),
        ("@\0", "AtKeyword(\"\u{fffd}\")"),
        ("#\0", "Hash(\"\u{fffd}\",id)"),
        ("1\0", "Dimension(\"1\",int,\"\u{fffd}\")"),
    ] {
        let document = parse(text);
        assert_node(&document, only(&document), expected, text, 0..text.len());
    }
}

#[test]
fn url_contents_are_decided_by_their_first_code_points() {
    // Section 4.3.6: a non-printable code point in a url makes a bad url; leading
    // whitespace is skipped; any other code point, NUL's replacement included,
    // is part of the value.
    for point in ['\u{1}', '\u{8}', '\u{b}', '\u{e}', '\u{1f}', '\u{7f}'] {
        let text = format!("url({point}a)");
        let document = parse(&text);
        assert_node(&document, only(&document), "BadUrl", &text, 0..text.len());
    }
    for whitespace in ["\t", "\n", "\r", "\u{c}", " ", "\r\n"] {
        let text = format!("url({whitespace}a)");
        let document = parse(&text);
        assert_node(
            &document,
            only(&document),
            "Url(\"a\")",
            &text,
            0..text.len(),
        );
    }
    for (text, value) in [
        ("url(\0a)", "\u{fffd}a"),
        ("url(\u{80}a)", "\u{80}a"),
        ("url(!a)", "!a"),
    ] {
        let document = parse(text);
        assert_node(
            &document,
            only(&document),
            &render(Token::Url(value)),
            text,
            0..text.len(),
        );
    }
    let document = parse("a\r\nb");
    let [a, space, b] = root_nodes(&document)[..] else {
        panic!("three nodes");
    };
    assert_node(&document, a, "Ident(\"a\")", "a", 0..1);
    assert_node(&document, space, "Whitespace", "\r\n", 1..3);
    assert_node(&document, b, "Ident(\"b\")", "b", 3..4);
}

#[test]
fn hex_escapes_decode_to_their_code_point_and_swallow_one_terminating_whitespace() {
    // Section 4.3.7: one to six hex digits and one optional whitespace; zero, a
    // surrogate and a value above the maximum allowed code point are U+FFFD.
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
        let document = parse(text);
        assert_node(
            &document,
            only(&document),
            &render(Token::Ident(value)),
            text,
            0..text.len(),
        );
    }
    // Only one whitespace belongs to the escape.
    let document = parse(r"\61  b");
    let [a, space, b] = root_nodes(&document)[..] else {
        panic!("three nodes");
    };
    assert_node(&document, a, "Ident(\"a\")", r"\61 ", 0..4);
    assert_node(&document, space, "Whitespace", " ", 4..5);
    assert_node(&document, b, "Ident(\"b\")", "b", 5..6);
}

#[test]
fn the_attribute_selector_operators_are_two_adjacent_delims() {
    // Section 4.3.1 has no `~=` token: `~` is a delim and `=` another.
    for (text, first) in [
        ("~=", '~'),
        ("|=", '|'),
        ("^=", '^'),
        ("$=", '$'),
        ("*=", '*'),
    ] {
        let document = parse(text);
        let [left, right] = root_nodes(&document)[..] else {
            panic!("two delims for {text}");
        };
        assert_node(
            &document,
            left,
            &format!("Delim({first:?})"),
            &text[..1],
            0..1,
        );
        assert_node(&document, right, "Delim('=')", "=", 1..2);
        let metrics = document.metrics();
        assert_eq!(
            (
                metrics.components(),
                metrics.maximum_depth(),
                metrics.spelling_bytes()
            ),
            (2, 0, 2)
        );
    }
}

#[test]
fn an_empty_text_is_an_empty_root_list_with_nothing_counted() {
    let document = parse("");
    assert_eq!(document.node_count(), 0);
    assert_eq!(document.list_count(), 1);
    assert!(document.list(document.root_list()).is_empty());
    assert_eq!(document.metrics().components(), 0);
    assert_eq!(document.metrics().maximum_depth(), 0);
    assert_eq!(document.metrics().spelling_bytes(), 0);
    assert_eq!(root_end(&document), Some(0));
    assert!(document.root().is_empty());
    assert!(document.lexical_errors().is_empty());
}

// ----------------------------------------------------------------- structure

#[test]
fn trivia_errors_and_later_tokens_stay_in_order_around_complete_nested_groups() {
    // Sections 5.5.7 to 5.5.10 over a text that mixes whitespace, a function that
    // holds a block, a stray `)`, a bad url, a bad string and a stray `]`.
    let text = " f([)] x) url(a b) \"bad\nz ;]";
    let document = parse(text);
    let nodes = root_nodes(&document);
    assert_eq!(nodes.len(), 11);
    let function = nodes[1];
    assert_node(&document, function, "Function(\"f\")", "f(", 1..9);
    // `range` covers the whole function; its spelling is the opening alone.
    assert_eq!(document.spelling(function), Some("f("));
    let arguments = document.children(function).unwrap();
    let [square, space, x] = document.list(arguments)[..] else {
        panic!("a block, whitespace, an identifier");
    };
    assert_node(&document, square, "OpenSquare", "[", 3..6);
    let inside = document.children(square).unwrap();
    let [stray] = document.list(inside)[..] else {
        panic!("the stray `)` stays inside the block it appeared in");
    };
    assert_node(&document, stray, "CloseParen", ")", 4..5);
    assert_eq!(document.list_end(inside), Some(Offset::new(5)));
    assert_eq!(token_of(&document, space), "Whitespace");
    assert_node(&document, x, "Ident(\"x\")", "x", 7..8);
    assert_eq!(document.list_end(arguments), Some(Offset::new(8)));
    for (index, expected) in [
        (2, "Whitespace"),
        (3, "BadUrl"),
        (4, "Whitespace"),
        (5, "BadString"),
        (6, "Whitespace"),
        (7, "Ident(\"z\")"),
        (8, "Whitespace"),
        (9, "Semicolon"),
        (10, "CloseSquare"),
    ] {
        assert_eq!(token_of(&document, nodes[index]), expected, "node {index}");
    }
    assert_eq!(document.spelling(nodes[3]), Some("url(a b)"));
    assert_eq!(document.range(nodes[3]).map(span), Some(10..18));
    assert_eq!(document.spelling(nodes[5]), Some("\"bad"));
    assert_eq!(document.range(nodes[5]).map(span), Some(19..23));
    assert_eq!(document.spelling(nodes[10]), Some("]"));
    assert_eq!(document.range(nodes[10]).map(span), Some(27..28));
    assert_eq!(root_end(&document), Some(28));
}

#[test]
fn the_end_of_the_text_closes_every_group_still_open_and_a_mismatch_stays_where_it_was() {
    // Section 5.5.9 and 5.5.10: the end of the input ends a simple block or a
    // function (a parse error); a closing token that is not the matching one is
    // consumed as a component value of the innermost group.
    let text = " f({a[)b";
    let document = parse(text);
    assert_eq!(
        outline(&document, document.root_list()),
        "Whitespace Function(\"f\")<OpenCurly<Ident(\"a\") OpenSquare<CloseParen Ident(\"b\")…>…>…>"
    );
    let nodes = root_nodes(&document);
    let function = nodes[1];
    let curly = document.list(document.children(function).unwrap())[0];
    let square = document.list(document.children(curly).unwrap())[1];
    for (group, range, opening) in [
        (function, 1..8, "f("),
        (curly, 3..8, "{"),
        (square, 5..8, "["),
    ] {
        assert_eq!(document.range(group).map(span), Some(range));
        assert_eq!(document.spelling(group), Some(opening));
        assert!(matches!(
            document.group_end(group),
            Some(GroupEnd::Eof { at: Some(at) }) if at.value() == 8
        ));
        let children = document.children(group).unwrap();
        assert_eq!(document.list_end(children), Some(Offset::new(8)));
    }
    let mismatch = document.list(document.children(square).unwrap())[0];
    assert_node(&document, mismatch, "CloseParen", ")", 6..7);
    assert_eq!(root_end(&document), Some(8));
}

#[test]
fn a_text_with_a_window_origin_ends_at_its_own_end_not_at_a_closer_outside() {
    // The text `f(a] b` lexed as the bytes 7..13 of a larger text, where a `)`
    // follows: the sub-text ends before that closer, so the function is closed by
    // the end of the sub-text, and every offset is the larger text's.
    let outer = "prefix f(a] b) outside";
    let document = parse_with(
        &outer[7..13],
        &Options::default().origin(Offset::new(7)),
        Limits::default(),
        DepthExcess::Fail,
    )
    .unwrap();
    let function = only(&document);
    assert_eq!(document.range(function).map(span), Some(7..13));
    let inside = document.children(function).unwrap();
    assert_eq!(
        outline(&document, inside),
        "Ident(\"a\") CloseSquare Whitespace Ident(\"b\")"
    );
    let stray = document.list(inside)[1];
    assert_eq!(document.range(stray).map(span), Some(10..11));
    assert_eq!(document.spelling(stray), Some("]"));
    assert_eq!(document.list_end(inside), Some(Offset::new(13)));
    assert_eq!(root_end(&document), Some(13));
    assert!(matches!(
        document.group_end(function),
        Some(GroupEnd::Eof { at: Some(at) }) if at.value() == 13
    ));
    // Text between two boundaries is taken from the sub-text with the origin
    // removed.
    let stream = document.root();
    assert_eq!(
        stream.text_between(stream.start_site(), stream.end_site()),
        Some("f(a] b")
    );
}

#[test]
fn a_mismatched_closer_cannot_pop_an_ancestor_or_reorder_later_siblings() {
    // `(` `[` `)` `]` `}` `x` `)` `y`: the `)` is inside the `[` block, which the
    // `]` then closes; the `}` is inside the `(` block, which the second `)`
    // closes; `y` follows at the top.
    let document = parse("([)]}x)y");
    assert_eq!(
        outline(&document, document.root_list()),
        "OpenParen<OpenSquare<CloseParen> CloseCurly Ident(\"x\")> Ident(\"y\")"
    );
    let metrics = document.metrics();
    assert_eq!(
        (
            metrics.components(),
            metrics.maximum_depth(),
            metrics.spelling_bytes()
        ),
        (6, 2, 8)
    );
    let paren = root_nodes(&document)[0];
    let paren_children = document.children(paren).unwrap();
    let square = document.list(paren_children)[0];
    let square_children = document.children(square).unwrap();
    assert_eq!(document.list_end(square_children), Some(Offset::new(3)));
    assert_eq!(document.list_end(paren_children), Some(Offset::new(6)));
    for (group, closer) in [(square, 3..4), (paren, 6..7)] {
        let Some(GroupEnd::Closed {
            range: Some(range), ..
        }) = document.group_end(group)
        else {
            panic!("closed by its own closing token");
        };
        assert_eq!(span(range.clone()), closer);
    }
    let bad = document.list(paren_children)[1];
    assert_node(&document, bad, "CloseCurly", "}", 4..5);
}

#[test]
fn tokens_inside_a_group_are_not_siblings_of_the_group_in_the_parent_list() {
    assert_eq!(
        outline_of("a f(b, c) [d]"),
        "Ident(\"a\") Whitespace Function(\"f\")<Ident(\"b\") Comma Whitespace Ident(\"c\")> Whitespace OpenSquare<Ident(\"d\")>"
    );
}

// ----------------------------------------------------------------- lexical errors

#[test]
fn the_lexical_errors_of_the_text_are_kept_in_the_order_the_lexer_met_them() {
    let document = parse("/* a\n\"b\n");
    let errors = document.lexical_errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].is_unterminated_comment());
    let document = parse("\"b\n url(c d)");
    let kinds: Vec<_> = document
        .lexical_errors()
        .iter()
        .map(|error| (error.is_bad_string(), error.is_bad_url()))
        .collect();
    // A bad string is a parse error; the bad url made by whitespace and then
    // `d` is a bad-url token without one (4.3.6).
    assert_eq!(kinds, [(true, false)]);
    assert!(parse("a b").lexical_errors().is_empty());
}

// ----------------------------------------------------------------- the depth policy

#[test]
fn nesting_at_the_depth_limit_builds_and_one_deeper_fails() {
    let text = format!("{}x{}", "(".repeat(256), ")".repeat(256));
    let document = parse(&text);
    assert_eq!(document.maximum_depth(), 256);
    assert_eq!(document.metrics().maximum_depth(), 256);
    let deeper = format!("({text})");
    let error = parse_with(
        &deeper,
        &Options::default(),
        Limits::default(),
        DepthExcess::Fail,
    )
    .unwrap_err();
    assert!(error.is_depth_limit());
    // The refused opening parenthesis is the 257th, at byte 256.
    assert_eq!(error.offset(), Some(Offset::new(256)));
}

#[test]
fn a_very_deep_text_builds_without_recursion() {
    // Building is iterative: a hundred thousand open blocks need no stack.
    let depth = 100_000;
    let text = format!("{}x", "(".repeat(depth));
    let limits = Limits::new(u32::try_from(depth).unwrap(), usize::MAX, usize::MAX);
    let document = parse_with(&text, &Options::default(), limits, DepthExcess::Fail).unwrap();
    assert_eq!(document.maximum_depth(), u32::try_from(depth).unwrap());
    assert_eq!(document.list_count(), depth + 1);
}

#[test]
fn deny_block_keeps_a_curly_block_past_the_limit_as_one_opaque_node_and_carries_on() {
    // Past the depth limit a `{}`-block is denied: one node spanning the block,
    // through its matching closer, with the text after it parsed as usual. The
    // groups inside the denied block nest and match among themselves; none of
    // its tokens are nodes.
    let limits = Limits::new(1, usize::MAX, usize::MAX);
    let text = "a ({b(}c)}d)e";
    // depth: `(` is 1 (allowed), `{` is 2 > 1: denied through its `}` at 9, the
    // `}` inside `b(...)` closing nothing it matches (the open group is `(`).
    let document = parse_with(text, &Options::default(), limits, DepthExcess::DenyBlock).unwrap();
    let nodes = root_nodes(&document);
    assert_eq!(nodes.len(), 4); // `a`, whitespace, the paren block, `e`
    let group = document.children(nodes[2]).unwrap();
    let [denied, d] = document.list(group)[..] else {
        panic!("the denied block and `d`");
    };
    assert!(matches!(
        document.component(denied),
        Component::Denied { .. }
    ));
    assert_eq!(document.range(denied).map(span), Some(3..10));
    assert_eq!(token_of(&document, d), "Ident(\"d\")");
    assert_eq!(token_of(&document, nodes[3]), "Ident(\"e\")");
    assert_eq!(document.children(denied), None);
    assert_eq!(document.group_kind(denied), None);
    // Metrics: a, the whitespace, `(`, the denied block, d, e are components; the
    // denied block is charged at its parent's depth with its whole spelling.
    let metrics = document.metrics();
    assert_eq!(metrics.components(), 6);
    assert_eq!(metrics.maximum_depth(), 1);
    assert_eq!(metrics.spelling_bytes(), text.len());
}

#[test]
fn deny_block_denies_only_curly_blocks_and_a_denied_block_without_a_closer_ends_at_the_text() {
    let limits = Limits::new(1, usize::MAX, usize::MAX);
    // A paren block, a square block or a function past the limit still fails.
    for text in ["((a))", "([a])", "(f(a))"] {
        let error =
            parse_with(text, &Options::default(), limits, DepthExcess::DenyBlock).unwrap_err();
        assert!(error.is_depth_limit(), "{text:?}");
    }
    // A curly block with no closing token is denied through the end.
    let document = parse_with(
        "({a (b",
        &Options::default(),
        limits,
        DepthExcess::DenyBlock,
    )
    .unwrap();
    let group = document.children(only(&document)).unwrap();
    let [denied] = document.list(group)[..] else {
        panic!("one denied block");
    };
    assert_eq!(document.range(denied).map(span), Some(1..6));
    // Fail refuses the same text.
    assert!(
        parse_with("({a", &Options::default(), limits, DepthExcess::Fail)
            .unwrap_err()
            .is_depth_limit()
    );
    // Within the limit the policy changes nothing.
    let document = parse_with("{a}", &Options::default(), limits, DepthExcess::DenyBlock).unwrap();
    assert_eq!(
        outline(&document, document.root_list()),
        "OpenCurly<Ident(\"a\")>"
    );
}

#[test]
fn limits_on_components_and_bytes_refuse_exactly_one_past_them() {
    let text = "@x;/*c*/@y;"; // five tokens, eleven bytes
    let limits = Limits::new(0, 5, 11);
    let document = parse_with(text, &Options::default(), limits, DepthExcess::Fail).unwrap();
    let metrics = document.metrics();
    assert_eq!(
        (
            metrics.components(),
            metrics.maximum_depth(),
            metrics.spelling_bytes()
        ),
        (5, 0, 11)
    );
    let error = parse_with(
        text,
        &Options::default(),
        Limits::new(0, 4, 11),
        DepthExcess::Fail,
    )
    .unwrap_err();
    assert!(error.is_node_limit());
    let error = parse_with(
        text,
        &Options::default(),
        Limits::new(0, 5, 10),
        DepthExcess::Fail,
    )
    .unwrap_err();
    assert!(error.is_byte_limit());
    assert!(error.offset().is_some());
    // The same text, failed and then built again, gives the same document.
    let again = parse_with(text, &Options::default(), limits, DepthExcess::Fail).unwrap();
    assert_eq!(again.metrics(), metrics);
}

#[test]
fn a_group_the_input_ended_costs_no_closing_bytes() {
    // `f(a` has three bytes; the closing parenthesis it never had adds none.
    let document = parse("f(a");
    assert_eq!(document.metrics().spelling_bytes(), 3);
    assert_eq!(document.metrics().components(), 2);
    let document = parse("f(a)");
    assert_eq!(document.metrics().spelling_bytes(), 4);
    assert_eq!(document.metrics().components(), 2);
}

// ----------------------------------------------------------------- navigation

#[test]
fn lists_nodes_and_groups_find_each_other() {
    let document = parse("a f(b [c]) d");
    let nodes = root_nodes(&document);
    assert_eq!(document.node_count(), 9);
    assert_eq!(document.list_count(), 3);
    // Nodes are numbered in text order, the group before what it holds.
    let function = nodes[2];
    assert_eq!(function.index(), 2);
    let arguments = document.children(function).unwrap();
    let square = document.list(arguments)[2];
    let inner = document.children(square).unwrap();
    assert_eq!(arguments.index(), 1);
    assert_eq!(inner.index(), 2);
    assert!(function < document.list(arguments)[0]);
    assert!(square < document.list(inner)[0]);
    // A leaf owns no list.
    assert_eq!(document.children(nodes[0]), None);
    // The parent of a list is the group that owns it, and of the root none.
    assert_eq!(document.parent_group(arguments), Some(function));
    assert_eq!(document.parent_group(inner), Some(square));
    assert_eq!(document.parent_group(document.root_list()), None);
    // A node's location is its list and its place in it.
    assert_eq!(document.location(function), (document.root_list(), 2));
    assert_eq!(document.location(square), (arguments, 2));
    assert_eq!(document.location(document.list(inner)[0]), (inner, 0));
    // Every node of a list reports that list.
    let lists = all_lists(&document);
    assert_eq!(lists.len(), document.list_count());
    for list in lists {
        for (index, node) in document.list(list).iter().enumerate() {
            assert_eq!(document.location(*node), (list, index));
        }
    }
}

/// Every list of the document, found by walking from the root list.
fn all_lists(document: &Document<'_>) -> Vec<ListId> {
    let mut lists = vec![document.root_list()];
    let mut next = 0;
    while next < lists.len() {
        for node in document.list(lists[next]) {
            if let Some(children) = document.children(*node) {
                lists.push(children);
            }
        }
        next += 1;
    }
    lists
}

/// Every node of the document, found by walking its lists.
fn all_nodes(document: &Document<'_>) -> Vec<NodeId> {
    all_lists(document)
        .into_iter()
        .flat_map(|list| document.list(list).to_vec())
        .collect()
}

#[test]
fn the_end_of_a_list_is_its_closing_token_or_the_end_of_the_text() {
    let document = parse("a(b) c(d");
    let nodes = root_nodes(&document);
    let closed = document.children(nodes[0]).unwrap();
    let open = document.children(nodes[2]).unwrap();
    assert_eq!(
        document.list_end(document.root_list()),
        Some(Offset::new(8))
    );
    assert_eq!(document.list_end(closed), Some(Offset::new(3)));
    assert_eq!(document.list_end(open), Some(Offset::new(8)));
}

#[test]
fn a_node_is_found_from_the_offset_where_its_token_or_closing_token_starts() {
    let text = "a (b c) d";
    let document = parse(text);
    let nodes = root_nodes(&document);
    // Every node is found at the start of its range, as the node at its place.
    let everything = all_nodes(&document);
    assert_eq!(everything.len(), document.node_count());
    for node in everything {
        let start = document.range(node).unwrap().start;
        let site = document.node_at_offset(start).unwrap();
        assert!(!site.is_closing());
        assert_eq!(site.node(), Some(node));
        assert_eq!((site.list(), site.index()), document.location(node));
        assert_eq!(site.offset(), Some(start));
    }
    // The closing token is found as the closing boundary of its group.
    let closing = document.node_at_offset(Offset::new(6)).unwrap();
    assert!(closing.is_closing());
    assert_eq!(closing.node(), Some(nodes[2]));
    // An offset past the text, or inside a token, finds nothing.
    assert!(document.node_at_offset(Offset::new(text.len())).is_none());
    let longer = parse("ident");
    assert!(longer.node_at_offset(Offset::new(2)).is_none());
}

#[test]
fn an_empty_group_is_found_at_its_closing_token() {
    let document = parse("a()");
    let closing = document.node_at_offset(Offset::new(2)).unwrap();
    assert!(closing.is_closing());
    assert_eq!(closing.node(), Some(root_nodes(&document)[0]));
}

#[test]
fn a_component_reports_its_node_children_and_fault() {
    let document = parse("f( ) [ ] { } a");
    let nodes = root_nodes(&document);
    let Component::Function {
        node,
        name,
        children,
        payload,
    } = document.component(nodes[0])
    else {
        panic!("a function");
    };
    assert_eq!((node, name, payload), (nodes[0], "f", None));
    assert_eq!(document.children(node), Some(children));
    for (index, kind) in [(2, BlockKind::Square), (4, BlockKind::Curly)] {
        let Component::Block {
            kind: actual, node, ..
        } = document.component(nodes[index])
        else {
            panic!("a block");
        };
        assert_eq!((actual, node), (kind, nodes[index]));
    }
    let leaf = document.component(nodes[6]);
    assert_eq!(leaf.children(), None);
    assert_eq!(leaf.fault(), None);
    assert_eq!(leaf.node(), nodes[6]);
    assert!(leaf.payload().is_none());
}

// ----------------------------------------------------------------- building from components

#[test]
fn a_builder_attaches_payloads_to_nodes_and_to_closing_tokens() {
    let mut builder = Document::<&str>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (function, inside) = builder
        .open(root, Token::Function("f"), None, Some("opening"))
        .unwrap();
    let argument = builder
        .leaf(inside, Token::Ident("a"), None, Some("argument"))
        .unwrap();
    let plain = builder.leaf(inside, Token::Comma, None, None).unwrap();
    builder
        .close(
            function,
            GroupEnd::Closed {
                range: None,
                payload: Some("closing"),
            },
        )
        .unwrap();
    let after = builder
        .leaf(root, Token::Ident("b"), None, Some("after"))
        .unwrap();
    let document = builder.finish(None);
    assert_eq!(document.payload(function), Some(&"opening"));
    assert_eq!(document.payload(argument), Some(&"argument"));
    assert_eq!(document.payload(plain), None);
    assert_eq!(document.payload(after), Some(&"after"));
    let Some(GroupEnd::Closed { payload, .. }) = document.group_end(function) else {
        panic!("closed");
    };
    assert_eq!(*payload, Some("closing"));
    // A site reads the payload of the token it is at: the opening token at the
    // boundary before it, the closing token at the closing boundary.
    let mut stream = document.root();
    let before = stream.site();
    stream.consume_function_name().unwrap();
    assert_eq!(before.payload(), Some(&"opening"));
    // Components carry the payload by reference, so a consumer sees the very
    // value that was attached, not a copy.
    let Component::Token { payload, .. } = document.component(argument) else {
        panic!("a token");
    };
    assert!(std::ptr::eq(
        payload.unwrap(),
        document.payload(argument).unwrap()
    ));
}

#[test]
fn a_payload_is_attached_without_any_trait_bound_on_its_type() {
    struct Opaque(
        #[expect(
            dead_code,
            reason = "never read: the point is that Opaque implements nothing"
        )]
        u32,
    );
    let mut builder = Document::<Opaque>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let node = builder
        .leaf(root, Token::Ident("a"), None, Some(Opaque(7)))
        .unwrap();
    let document = builder.finish(None);
    assert!(document.payload(node).is_some());
    // A component prints without printing its payload.
    assert!(format!("{:?}", document.component(node)).contains("has_payload: true"));
    let stream = document.root();
    let site = stream.site();
    let copy = site;
    assert_eq!(site, copy);
    assert!(format!("{site:?}").contains("Site"));
    let mark = stream.mark();
    let again = mark;
    assert_eq!(again.site(), mark.site());
    assert!(format!("{stream:?}").contains("Stream"));
}

#[test]
fn separately_added_adjacent_identifiers_are_not_merged_into_one_token() {
    // Section 5.4, "normalize into a token stream": a list of tokens is a stream
    // as it is; two identifiers added one after the other stay two tokens.
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    builder.leaf(root, Token::Ident("a"), None, None).unwrap();
    builder.leaf(root, Token::Ident("b"), None, None).unwrap();
    let document = builder.finish(None);
    let mut stream = document.root();
    assert_eq!(stream.consume_ident().ok(), Some("a"));
    assert_eq!(stream.consume_ident().ok(), Some("b"));
    assert!(stream.is_empty());
    // Nothing was spelled and nothing has a place in a text.
    assert_eq!(document.source(), None);
    assert_eq!(document.range(document.list(root)[0]), None);
    assert_eq!(document.spelling(document.list(root)[0]), None);
    assert_eq!(document.metrics().spelling_bytes(), 0);
    assert_eq!(document.metrics().components(), 2);
    assert!(document.node_at_offset(Offset::new(0)).is_none());
    assert_eq!(document.list_end(root), None);
}

#[test]
fn faults_and_mismatched_closers_added_to_a_group_stay_in_order() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (function, inside) = builder
        .open(root, Token::Function("f"), None, None)
        .unwrap();
    builder.leaf(inside, Token::BadString, None, None).unwrap();
    builder
        .leaf(inside, Token::CloseSquare, None, None)
        .unwrap();
    builder.leaf(inside, Token::BadUrl, None, None).unwrap();
    builder
        .close(
            function,
            GroupEnd::Closed {
                range: None,
                payload: None,
            },
        )
        .unwrap();
    builder
        .leaf(root, Token::Ident("after"), None, None)
        .unwrap();
    let document = builder.finish(None);
    assert_eq!(
        outline(&document, document.root_list()),
        "Function(\"f\")<BadString CloseSquare BadUrl> Ident(\"after\")"
    );
    let faults: Vec<_> = document
        .list(inside)
        .iter()
        .map(|node| document.component(*node).fault())
        .collect();
    assert_eq!(
        faults,
        [
            Some(Fault::BadString),
            Some(Fault::UnexpectedCloser),
            Some(Fault::BadUrl)
        ]
    );
    // Five components: the function, three faults and the sibling. The closing
    // token counts none.
    assert_eq!(document.metrics().components(), 5);
}

#[test]
fn a_builder_counts_tokens_without_an_interval_as_zero_bytes_unless_told_otherwise() {
    // A zero-byte budget admits tokens that have no interval; one that has an
    // interval is counted by its length; the caller may report bytes it knows.
    let limits = Limits::new(0, 2, 0);
    let mut builder = Document::<()>::builder(limits, 0, None).unwrap();
    let root = builder.root();
    builder.leaf(root, Token::Ident("é"), None, None).unwrap();
    builder.leaf(root, Token::Ident("é"), None, None).unwrap();
    assert!(
        builder
            .leaf(root, Token::Ident("é"), None, None)
            .unwrap_err()
            .is_node_limit()
    );
    let mut builder = Document::<()>::builder(Limits::new(0, 9, 3), 0, None).unwrap();
    let root = builder.root();
    let range = Offset::new(0)..Offset::new(2);
    builder
        .leaf(root, Token::Ident("ab"), Some(range), None)
        .unwrap();
    builder.charge_spelling(1).unwrap();
    assert!(builder.charge_spelling(1).unwrap_err().is_byte_limit());
}

#[test]
fn the_base_depth_counts_against_the_depth_limit_and_is_in_the_maximum_depth() {
    // `f()` is one deep. Started at base depth 255 it reaches the limit of 256;
    // started at 256 it exceeds it.
    let limits = Limits::new(256, 1, 3);
    let build = |base: u32| -> Result<Document<'static>, BuildError> {
        let mut builder = Document::<()>::builder(limits, base, None)?;
        let root = builder.root();
        let (group, _) = builder.open(
            root,
            Token::Function("f"),
            Some(Offset::new(0)..Offset::new(2)),
            None,
        )?;
        builder.close(
            group,
            GroupEnd::Closed {
                range: Some(Offset::new(2)..Offset::new(3)),
                payload: None,
            },
        )?;
        Ok(builder.finish(Some("f()")))
    };
    let document = build(255).unwrap();
    assert_eq!(document.maximum_depth(), 256);
    assert_eq!(document.metrics().spelling_bytes(), 3);
    assert!(build(256).unwrap_err().is_depth_limit());
    // A base depth that overflows the counter is a capacity error, not a wrap.
    assert!(build(u32::MAX).unwrap_err().is_capacity_overflow());
}

#[test]
fn a_failed_build_leaves_no_usable_document_and_the_same_input_builds_again() {
    let limits = Limits::new(1, 3, 8);
    let attempt = |limits: Limits| -> Result<Document<'static>, BuildError> {
        let mut builder = Document::<()>::builder(limits, 0, None)?;
        let root = builder.root();
        for _ in 0..2 {
            let (group, inside) = builder.open(root, Token::OpenCurly, None, None)?;
            builder.leaf(inside, Token::Ident("é"), None, None)?;
            builder.charge_spelling(4)?;
            builder.close(
                group,
                GroupEnd::Closed {
                    range: None,
                    payload: None,
                },
            )?;
        }
        Ok(builder.finish(None))
    };
    // Two groups of two components each is four components; the limit is three.
    assert!(attempt(limits).unwrap_err().is_node_limit());
    let wide = Limits::new(1, 4, 8);
    assert_eq!(attempt(wide).unwrap().metrics().components(), 4);
    assert!(attempt(Limits::new(1, 4, 7)).unwrap_err().is_byte_limit());
    assert!(attempt(Limits::new(0, 4, 8)).unwrap_err().is_depth_limit());
    // After the failures the same input still builds.
    assert!(attempt(wide).is_ok());
}

#[test]
#[should_panic(expected = "opens a group")]
fn a_leaf_cannot_be_a_token_that_opens_a_group() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let _ = builder.leaf(root, Token::OpenParen, None, None);
}

#[test]
#[should_panic(expected = "opens a function or a simple block")]
fn a_group_is_opened_only_by_the_token_that_opens_one() {
    let mut builder: TreeBuilder<'_, ()> = Document::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let _ = builder.open(root, Token::Ident("a"), None, None);
}

// ----------------------------------------------------------------- tokens and tree agree

#[test]
fn the_tree_holds_exactly_the_tokens_of_the_text_in_order() {
    // Walking the tree, an opening token, its contents and its closing token,
    // reproduces the lexer's tokens (the closing token only where there is one).
    for text in [
        "a b",
        "f(a, b) [c] {d:e}",
        "(a [b {c} d] e)",
        "f(a",
        "a) b] c}",
        "url(a b) \"c\n d /* e",
    ] {
        let document = parse(text);
        let mut walked = Vec::new();
        walk(&document, document.root_list(), &mut walked);
        assert_eq!(walked, tokens(text), "{text:?}");
    }
}

fn walk(document: &Document<'_>, list: ListId, out: &mut Vec<String>) {
    for &node in document.list(list) {
        out.push(token_of(document, node));
        if let Some(children) = document.children(node) {
            walk(document, children, out);
            if let Some(GroupEnd::Closed { .. }) = document.group_end(node) {
                let closing = match document.group_kind(node).unwrap() {
                    GroupKind::Block(BlockKind::Square) => "CloseSquare",
                    GroupKind::Block(BlockKind::Curly) => "CloseCurly",
                    _ => "CloseParen",
                };
                out.push(closing.to_owned());
            }
        }
    }
}

// ----------------------------------------------------------------- tokens added by hand

/// Adds the contents of `list` of `from` to `into` of `builder`, token by token,
/// with no interval and no payload.
fn copy_list<'d>(
    from: &'d Document<'d>,
    list: ListId,
    builder: &mut TreeBuilder<'d, ()>,
    into: ListId,
) {
    for &node in from.list(list) {
        match from.component(node) {
            Component::Token { token, .. } => {
                builder.leaf(into, token, None, None).unwrap();
            }
            Component::Function { children, .. } | Component::Block { children, .. } => {
                let (group, inside) = builder.open(into, from.token(node), None, None).unwrap();
                copy_list(from, children, builder, inside);
                let end = match from.group_end(node) {
                    Some(GroupEnd::Eof { .. }) => GroupEnd::Eof { at: None },
                    _ => GroupEnd::Closed {
                        range: None,
                        payload: None,
                    },
                };
                builder.close(group, end).unwrap();
            }
            _ => unreachable!("a parsed text with the default depth has no denied block"),
        }
    }
}

#[test]
fn tokens_of_every_kind_added_to_a_builder_read_back_as_they_were() {
    // Decoded names, strings and urls, the type flag of a hash, a number's exact
    // spelling and sign, a unit, a percentage, a unicode range, the faults and an
    // unclosed group all survive being added token by token.
    let text = "a f(#b #1 1.5e3px +5% \"s\\41 \" url(u) -->) [@k U+4?? ! ; , : /**/ \\61 ] \
                {\"bad\n} url(a b) ) } ] <!-- 007 -0.0e+2";
    let source = parse_with(
        text,
        &Options::default().unicode_ranges(true),
        Limits::default(),
        DepthExcess::Fail,
    )
    .unwrap();
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    copy_list(&source, source.root_list(), &mut builder, root);
    let copy = builder.finish(None);
    assert_eq!(
        outline(&copy, copy.root_list()),
        outline(&source, source.root_list())
    );
    assert_eq!(copy.node_count(), source.node_count());
    assert_eq!(copy.metrics().components(), source.metrics().components());
    assert_eq!(copy.maximum_depth(), source.maximum_depth());
    // The copy was never spelled, so it has no text of its own.
    assert_eq!(copy.metrics().spelling_bytes(), 0);
    assert_eq!(copy.source(), None);
}

#[test]
fn a_large_denied_block_is_one_node_charged_for_all_its_bytes() {
    // The block is denied without reading its contents into nodes; its whole
    // spelling is charged once and the tokens around it are ordinary.
    let inner = "x".repeat(65_536);
    let text = format!("a {{{inner}}} b");
    let limits = Limits::new(0, usize::MAX, usize::MAX);
    let document = parse_with(&text, &Options::default(), limits, DepthExcess::DenyBlock).unwrap();
    assert_eq!(
        outline(&document, document.root_list()),
        "Ident(\"a\") Whitespace Denied Whitespace Ident(\"b\")"
    );
    let denied = root_nodes(&document)[2];
    assert_eq!(document.range(denied).map(span), Some(2..65_540));
    assert_eq!(document.node_count(), 5);
    assert_eq!(document.metrics().components(), 5);
    assert_eq!(document.metrics().spelling_bytes(), text.len());
    assert_eq!(document.maximum_depth(), 0);
}

#[test]
#[should_panic(expected = "a group is closed once")]
fn a_group_cannot_be_closed_twice() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (group, _) = builder.open(root, Token::OpenParen, None, None).unwrap();
    builder.close(group, GroupEnd::Eof { at: None }).unwrap();
    let _ = builder.close(group, GroupEnd::Eof { at: None });
}

#[test]
#[should_panic(expected = "nothing is added to a group after it is closed")]
fn a_token_cannot_be_added_to_a_group_after_it_is_closed() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (group, inside) = builder.open(root, Token::OpenParen, None, None).unwrap();
    builder.close(group, GroupEnd::Eof { at: None }).unwrap();
    let _ = builder.leaf(inside, Token::Comma, None, None);
}

#[test]
#[should_panic(expected = "nothing is added to a group after it is closed")]
fn a_group_cannot_be_opened_inside_a_group_after_it_is_closed() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (group, inside) = builder
        .open(root, Token::Function("f"), None, None)
        .unwrap();
    builder.close(group, GroupEnd::Eof { at: None }).unwrap();
    let _ = builder.open(inside, Token::OpenSquare, None, None);
}

#[test]
fn a_closed_group_does_not_stop_its_siblings_from_growing() {
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (group, _) = builder.open(root, Token::OpenParen, None, None).unwrap();
    builder.close(group, GroupEnd::Eof { at: None }).unwrap();
    builder.leaf(root, Token::Comma, None, None).unwrap();
    let document = builder.finish(None);
    assert_eq!(document.list(document.root_list()).len(), 2);
}

/// The interval of a group and the end of its list are what the caller said
/// about the group's end, and only that: a closing token with an interval ends
/// both at its start and end, one the caller holds without a position ends
/// neither, a group the input ended is ended at the position given, and a group
/// that was never ended keeps the interval of its opening token and has no end.
#[test]
fn a_builder_group_ends_its_interval_and_its_list_where_the_caller_says() {
    let range = |from, to| Some(Offset::new(from)..Offset::new(to));
    let mut builder = Document::<()>::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    let (positioned, positioned_list) = builder
        .open(root, Token::OpenParen, range(0, 1), None)
        .unwrap();
    builder
        .leaf(positioned_list, Token::Comma, range(1, 2), None)
        .unwrap();
    builder
        .close(
            positioned,
            GroupEnd::Closed {
                range: range(2, 3),
                payload: None,
            },
        )
        .unwrap();
    let (by_eof, by_eof_list) = builder
        .open(root, Token::OpenSquare, range(4, 5), None)
        .unwrap();
    builder
        .close(
            by_eof,
            GroupEnd::Eof {
                at: Some(Offset::new(9)),
            },
        )
        .unwrap();
    let (unpositioned, unpositioned_list) = builder
        .open(root, Token::OpenCurly, range(10, 11), None)
        .unwrap();
    builder
        .close(
            unpositioned,
            GroupEnd::Closed {
                range: None,
                payload: None,
            },
        )
        .unwrap();
    let (without_interval, without_interval_list) =
        builder.open(root, Token::OpenParen, None, None).unwrap();
    builder
        .close(
            without_interval,
            GroupEnd::Closed {
                range: range(20, 21),
                payload: None,
            },
        )
        .unwrap();
    let (never_ended, never_ended_list) = builder
        .open(root, Token::OpenParen, range(30, 31), None)
        .unwrap();
    let document = builder.finish(None);

    assert_eq!(document.range(positioned), range(0, 3));
    assert_eq!(
        document.list_end(positioned_list),
        Some(Offset::new(2)),
        "a list ends where its closing token starts"
    );
    assert_eq!(document.range(by_eof), range(4, 9));
    assert_eq!(document.list_end(by_eof_list), Some(Offset::new(9)));
    assert_eq!(document.range(unpositioned), range(10, 11));
    assert_eq!(document.list_end(unpositioned_list), None);
    assert_eq!(document.range(without_interval), None);
    assert_eq!(
        document.list_end(without_interval_list),
        Some(Offset::new(20))
    );
    assert_eq!(document.range(never_ended), range(30, 31));
    assert_eq!(document.list_end(never_ended_list), None);
    assert!(matches!(
        document.group_end(never_ended),
        Some(GroupEnd::Eof { at: None })
    ));
    assert_eq!(document.parent_group(never_ended_list), Some(never_ended));
    assert_eq!(document.parent_group(document.root_list()), None);
    assert_eq!(
        document.group_kind(by_eof),
        Some(GroupKind::Block(BlockKind::Square))
    );
}
