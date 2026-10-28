#![forbid(unsafe_code)]
//! Generic declaration boundaries through the existing syntax owner.
//! Oracles: selected Syntax 3 CRD20261001 §§5.4.7 and 5.5.6; the declaration-list
//! cases follow CRD20211224 §§5.3.8 and 5.4.5 for the deferred raw
//! declaration-list front (docs/reference.md#css-syntax-3-edition-differences).
//! Node identity and ranges exercise the retained-input contract, not a relex.
use super::*;

fn source(text: &str) -> (CssSourceSnapshot, Document<'_>) {
    let snapshot = CssSourceSnapshot::new(text);
    let document = source_document(text, &snapshot, 0).unwrap();
    (snapshot, document)
}

fn spellings<'a>(document: &'a Document<'_>, nodes: &[NodeId]) -> Vec<&'a str> {
    nodes
        .iter()
        .map(|node| document.spelling(*node).unwrap())
        .collect()
}

fn range_spelling<'a>(document: &'a Document<'_>, range: &SyntaxRange) -> Vec<&'a str> {
    spellings(document, &document.list(range.list)[range.start..range.end])
}

fn parsed_origin(
    document: &Document<'_>,
    node: NodeId,
    snapshot: &CssSourceSnapshot,
    bytes: Range<usize>,
) {
    let CssValueOrigin::Parsed(origin) = document.origin(node) else {
        panic!("original source occurrence")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), bytes.start);
    assert_eq!(origin.span().end().byte_offset().value(), bytes.end);
}

fn eof(cursor: &mut SyntaxCursor<'_>, offset: usize) {
    let CursorItem::EndOfInput(boundary) = cursor.consume() else {
        panic!("bounded EOF")
    };
    assert_eq!(boundary.offset, Some(offset));
    assert_eq!(cursor.peek(), CursorItem::EndOfInput(boundary));
}

#[test]
fn unknown_and_empty_generic_declarations_reach_exact_eof_before_property_admission() {
    for (text, name, expected_value) in [
        (" \t/**/ unknown : x ", "unknown", vec!["x"]),
        (" /**/ unknown: \t/**/", "unknown", vec![]),
        (" \t width: arbitrary", "width", vec!["arbitrary"]),
    ] {
        let (_, document) = source(text);
        let mut cursor = document.cursor(document.root_list());
        // The real caller owns leading-trivia handling; §5.5.6 starts at ident.
        cursor.skip_trivia();
        let declaration = consume_declaration(&mut cursor).unwrap();
        assert_eq!(document.kind(declaration.name), TokenKind::Ident(name));
        assert_eq!(spellings(&document, &declaration.value), expected_value);
        assert!(!declaration.important);
        assert!(declaration.importance_range.is_none());
        eof(&mut cursor, text.len());
        // Ordinary public admission is deliberately stronger than structure.
        assert!(
            crate::declaration::CssDeclaration::parse(&crate::CssParser::default(), text)
                .syntax()
                .is_none(),
            "{text}"
        );
    }
}

#[test]
fn nonident_and_missing_colon_return_no_generic_declaration() {
    for text in ["@name:x", ":x", "1:x", "name value", "name \t/**/", ""] {
        let (_, document) = source(text);
        let fault = consume_declaration(&mut document.cursor(document.root_list())).unwrap_err();
        assert_eq!(fault.kind, GenericFaultKind::InvalidDeclaration, "{text}");
    }
}

#[test]
fn terminal_priority_uses_decoded_ascii_case_insensitive_ident_and_ignores_trivia() {
    for text in [
        "name: a!important",
        "name: a !IMPORTANT \t",
        "name: a !/**/ ImPoRtAnT /**/ ",
        r"name: a !\69mportant",
        r"name: a !\49 MPORTANT",
    ] {
        let (_, document) = source(text);
        let mut cursor = document.cursor(document.root_list());
        let declaration = consume_declaration(&mut cursor).unwrap();
        assert!(declaration.important, "{text}");
        assert_eq!(spellings(&document, &declaration.value), ["a"], "{text}");
        let importance = declaration.importance_range.as_ref().unwrap();
        assert_eq!(
            document.kind(document.list(importance.list)[importance.start]),
            TokenKind::Delim('!')
        );
        assert_eq!(importance.end, declaration.range.end);
        eof(&mut cursor, text.len());
    }
}

#[test]
fn stripping_the_final_priority_pair_retains_an_earlier_bang_and_its_origin() {
    let text = "name: a !old !IMPORTANT ";
    let (snapshot, document) = source(text);
    let declaration = consume_declaration(&mut document.cursor(document.root_list())).unwrap();
    assert!(declaration.important);
    assert_eq!(
        spellings(&document, &declaration.value),
        ["a", " ", "!", "old"]
    );
    let earlier_bang = declaration.value[2];
    parsed_origin(&document, earlier_bang, &snapshot, 8..9);
    assert_eq!(
        range_spelling(&document, &declaration.value_range),
        [" ", "a", " ", "!", "old"]
    );
    assert_eq!(
        range_spelling(&document, declaration.importance_range.as_ref().unwrap()),
        ["!", "IMPORTANT", " "]
    );
}

#[test]
fn grouped_or_nonterminal_bang_important_pairs_remain_normal_value_components() {
    for (text, expected) in [
        ("name: f(!important)", vec!["f("]),
        ("name: [!important]", vec!["["]),
        ("name: {!important}", vec!["{"]),
        (
            "name: a !important tail",
            vec!["a", " ", "!", "important", " ", "tail"],
        ),
        ("name: a !important!", vec!["a", " ", "!", "important", "!"]),
        ("name: a !importantx", vec!["a", " ", "!", "importantx"]),
    ] {
        let (_, document) = source(text);
        let declaration = consume_declaration(&mut document.cursor(document.root_list())).unwrap();
        assert!(!declaration.important, "{text}");
        assert!(declaration.importance_range.is_none(), "{text}");
        assert_eq!(spellings(&document, &declaration.value), expected, "{text}");
        if let Some(children) = document.children(declaration.value[0]) {
            assert_eq!(
                spellings(&document, document.list(children)),
                ["!", "important"]
            );
        }
    }
}

#[test]
fn trimmed_components_keep_internal_trivia_and_original_post_colon_range() {
    let text = "name: /**/ a /*mid*/ b /**/  ";
    let (snapshot, document) = source(text);
    let declaration = consume_declaration(&mut document.cursor(document.root_list())).unwrap();
    assert_eq!(
        spellings(&document, &declaration.value),
        ["a", " ", "/*mid*/", " ", "b"]
    );
    assert_eq!(
        range_spelling(&document, &declaration.value_range),
        [" ", "/**/", " ", "a", " ", "/*mid*/", " ", "b"]
    );
    assert_eq!(declaration.value_range.start, declaration.colon + 1);
    assert_eq!(
        declaration.range.end,
        document.list(document.root_list()).len()
    );
    let extent = |node| {
        document
            .range(node)
            .map(|range| range.start.value()..range.end.value())
    };
    assert_eq!(extent(declaration.value[0]), Some(11..12));
    assert_eq!(extent(declaration.value[4]), Some(21..22));
    for (node, bytes) in declaration
        .value
        .iter()
        .zip([11..12, 12..13, 13..20, 20..21, 21..22])
    {
        parsed_origin(&document, *node, &snapshot, bytes);
    }
}

#[test]
fn checked_values_with_distinct_source_origins_survive_priority_extraction() {
    let first = crate::component::values::parse_component_values("same").unwrap();
    let second = crate::component::values::parse_component_values("same").unwrap();
    let programmatic = |spelling| CssComponentValue::try_token(spelling).unwrap();
    let values = [
        CssComponentValue::try_ident("unknown").unwrap(),
        programmatic(":"),
        first.items()[0].clone(),
        CssComponentValue::try_number("-0.0").unwrap(),
        second.items()[0].clone(),
        programmatic("!"),
        CssComponentValue::try_ident("IMPORTANT").unwrap(),
    ];
    let document = checked_document(&values, SyntaxInputLimits::default(), 0).unwrap();
    let mut cursor = document.cursor(document.root_list());
    let declaration = consume_declaration(&mut cursor).unwrap();
    assert!(declaration.important);
    assert_eq!(declaration.value, document.list(document.root_list())[2..5]);
    let [first_node, number, second_node] = declaration.value.as_slice() else {
        panic!("three direct values")
    };
    let Some(Attached::Component(first_component)) = document.payload(*first_node) else {
        panic!("first checked occurrence")
    };
    let Some(Attached::Component(second_component)) = document.payload(*second_node) else {
        panic!("second checked occurrence")
    };
    assert!(std::ptr::eq(*first_component, &values[2]));
    assert!(std::ptr::eq(*second_component, &values[4]));
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) =
        (document.origin(*first_node), document.origin(*second_node))
    else {
        panic!("distinct original source owners")
    };
    let (CssValueOrigin::Parsed(expected_a), CssValueOrigin::Parsed(expected_b)) =
        (first.items()[0].origin(), second.items()[0].origin())
    else {
        panic!("original origins")
    };
    assert!(a.source().same_snapshot(expected_a.source()));
    assert!(b.source().same_snapshot(expected_b.source()));
    assert!(!a.source().same_snapshot(b.source()));
    let Token::Number(literal) = document.token(*number) else {
        panic!("checked numeric payload")
    };
    assert_eq!(literal.representation(), "-0.0");
    assert!(literal.sign().is_some());
    assert_eq!(literal.to_f64().to_bits(), (-0.0_f64).to_bits());
    assert_eq!(literal.integer(), None);
    assert!(document.spelling(*number).is_none());
    assert_eq!(document.origin(*number), CssValueOrigin::Programmatic);
    let CursorItem::EndOfInput(boundary) = cursor.peek() else {
        panic!("supplied EOF")
    };
    assert!(boundary.offset.is_none());
    let root = document.root_list();
    assert_eq!(
        declaration.value_range,
        SyntaxRange {
            list: root,
            start: 2,
            end: 5
        }
    );
    assert_eq!(
        declaration.importance_range,
        Some(SyntaxRange {
            list: root,
            start: 5,
            end: 7
        })
    );
}
