#![forbid(unsafe_code)]
//! Generic declaration boundaries through the existing syntax owner.
//! Oracles: selected Syntax 3 CRD20261001 §§5.4.7 and 5.5.6; the declaration-list
//! cases follow CRD20211224 §§5.3.8 and 5.4.5 for the deferred raw
//! declaration-list front (docs/reference.md#css-syntax-3-edition-differences).
//! Node identity and ranges exercise the retained-input contract, not a relex.
use super::*;

fn source(text: &str) -> (CssSourceSnapshot, SyntaxDocument<'static>) {
    let snapshot = CssSourceSnapshot::new(text);
    let document = source_document(text, &snapshot, 0).unwrap();
    (snapshot, document)
}

fn spellings<'a>(document: &'a SyntaxDocument<'_>, nodes: &[NodeId]) -> Vec<&'a str> {
    nodes
        .iter()
        .map(|node| document.nodes[*node].token().spelling.as_deref().unwrap())
        .collect()
}

fn range_spelling<'a>(document: &'a SyntaxDocument<'_>, range: &SyntaxRange) -> Vec<&'a str> {
    spellings(
        document,
        &document.lists[range.list][range.start..range.end],
    )
}

fn parsed_origin(token: &SyntaxToken<'_>, snapshot: &CssSourceSnapshot, bytes: Range<usize>) {
    let CssValueOrigin::Parsed(origin) = token.origin.as_ref() else {
        panic!("original source occurrence")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), bytes.start);
    assert_eq!(origin.span().end().byte_offset().value(), bytes.end);
}

fn eof(cursor: &mut SyntaxCursor<'_>, snapshot: &CssSourceSnapshot, offset: usize) {
    let CursorItem::EndOfInput(boundary) = cursor.consume() else {
        panic!("bounded EOF")
    };
    let site = boundary.source.as_ref().unwrap();
    assert!(site.snapshot.same_snapshot(snapshot));
    assert_eq!(site.offset, offset);
    assert_eq!(cursor.peek(), CursorItem::EndOfInput(boundary));
}

#[test]
fn unknown_and_empty_generic_declarations_reach_exact_eof_before_property_admission() {
    for (text, name, expected_value) in [
        (" \t/**/ unknown : x ", "unknown", vec!["x"]),
        (" /**/ unknown: \t/**/", "unknown", vec![]),
        (" \t width: arbitrary", "width", vec!["arbitrary"]),
    ] {
        let (snapshot, document) = source(text);
        let mut cursor = document.cursor(document.root);
        // The real caller owns leading-trivia handling; §5.5.6 starts at ident.
        cursor.skip_trivia();
        let declaration = consume_declaration(&mut cursor).unwrap();
        assert_eq!(
            document.nodes[declaration.name].token().kind(),
            TokenKind::Ident(name)
        );
        assert_eq!(spellings(&document, &declaration.value), expected_value);
        assert!(!declaration.important);
        assert!(declaration.importance_range.is_none());
        eof(&mut cursor, &snapshot, text.len());
        // Ordinary public admission is deliberately stronger than structure.
        assert!(
            crate::CssDeclaration::parse(&crate::CssParser::default(), text)
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
        let fault = consume_declaration(&mut document.cursor(document.root)).unwrap_err();
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
        let (snapshot, document) = source(text);
        let mut cursor = document.cursor(document.root);
        let declaration = consume_declaration(&mut cursor).unwrap();
        assert!(declaration.important, "{text}");
        assert_eq!(spellings(&document, &declaration.value), ["a"], "{text}");
        let importance = declaration.importance_range.as_ref().unwrap();
        assert_eq!(
            document.nodes[document.lists[importance.list][importance.start]]
                .token()
                .kind(),
            TokenKind::Delim('!')
        );
        assert_eq!(importance.end, declaration.range.end);
        eof(&mut cursor, &snapshot, text.len());
    }
}

#[test]
fn stripping_the_final_priority_pair_retains_an_earlier_bang_and_its_origin() {
    let text = "name: a !old !IMPORTANT ";
    let (snapshot, document) = source(text);
    let declaration = consume_declaration(&mut document.cursor(document.root)).unwrap();
    assert!(declaration.important);
    assert_eq!(
        spellings(&document, &declaration.value),
        ["a", " ", "!", "old"]
    );
    let earlier_bang = declaration.value[2];
    parsed_origin(document.nodes[earlier_bang].token(), &snapshot, 8..9);
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
        let (_, original) = source(text);
        let document = normalize(
            SyntaxInput::Components {
                document: &original,
                list: original.root,
            },
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        let declaration = consume_declaration(&mut document.cursor(document.root)).unwrap();
        assert!(!declaration.important, "{text}");
        assert!(declaration.importance_range.is_none(), "{text}");
        assert_eq!(spellings(&document, &declaration.value), expected, "{text}");
        if let Some(children) = document.nodes[declaration.value[0]].children() {
            assert_eq!(
                spellings(&document, &document.lists[children]),
                ["!", "important"]
            );
        }
    }
}

#[test]
fn trimmed_components_keep_internal_trivia_and_original_post_colon_range() {
    let text = "name: /**/ a /*mid*/ b /**/  ";
    let (snapshot, document) = source(text);
    let declaration = consume_declaration(&mut document.cursor(document.root)).unwrap();
    assert_eq!(
        spellings(&document, &declaration.value),
        ["a", " ", "/*mid*/", " ", "b"]
    );
    assert_eq!(
        range_spelling(&document, &declaration.value_range),
        [" ", "/**/", " ", "a", " ", "/*mid*/", " ", "b"]
    );
    assert_eq!(declaration.value_range.start, declaration.colon + 1);
    assert_eq!(declaration.range.end, document.lists[document.root].len());
    assert_eq!(document.range(declaration.value[0]), Some(11..12));
    assert_eq!(document.range(declaration.value[4]), Some(21..22));
    for (node, bytes) in declaration
        .value
        .iter()
        .zip([11..12, 12..13, 13..20, 20..21, 21..22])
    {
        parsed_origin(document.nodes[*node].token(), &snapshot, bytes);
    }
}

#[test]
fn supplied_unspelled_payload_and_distinct_source_origins_survive_priority_extraction() {
    let first = crate::component_values::parse_component_values("same").unwrap();
    let second = crate::component_values::parse_component_values("same").unwrap();
    let native = |token| SyntaxToken {
        payload: TokenPayload::Native(token),
        spelling: None,
        origin: Cow::Owned(CssValueOrigin::Programmatic),
    };
    let tokens = [
        native(Token::Ident("unknown".into())),
        native(Token::Colon),
        SyntaxToken::checked(&first.items()[0], CheckedLexeme::Leaf),
        native(Token::Number {
            has_sign: true,
            value: -0.0,
            int_value: None,
        }),
        SyntaxToken::checked(&second.items()[0], CheckedLexeme::Leaf),
        native(Token::Delim('!')),
        native(Token::Ident("IMPORTANT".into())),
    ];
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let mut cursor = document.cursor(document.root);
    let declaration = consume_declaration(&mut cursor).unwrap();
    assert!(declaration.important);
    assert_eq!(declaration.value, document.lists[document.root][2..5]);
    let [first_node, number, second_node] = declaration.value.as_slice() else {
        panic!("three direct values")
    };
    let TokenPayload::Checked {
        component: first_component,
        lexeme: CheckedLexeme::Leaf,
    } = &document.nodes[*first_node].token().payload
    else {
        panic!("first checked occurrence")
    };
    let TokenPayload::Checked {
        component: second_component,
        lexeme: CheckedLexeme::Leaf,
    } = &document.nodes[*second_node].token().payload
    else {
        panic!("second checked occurrence")
    };
    assert!(std::ptr::eq(*first_component, &first.items()[0]));
    assert!(std::ptr::eq(*second_component, &second.items()[0]));
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (
        document.nodes[*first_node].token().origin.as_ref(),
        document.nodes[*second_node].token().origin.as_ref(),
    ) else {
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
    let TokenPayload::Native(Token::Number {
        has_sign,
        value,
        int_value,
    }) = &document.nodes[*number].token().payload
    else {
        panic!("native unspelled numeric payload")
    };
    assert!(*has_sign);
    assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(*int_value, None);
    assert!(document.nodes[*number].token().spelling.is_none());
    assert_eq!(
        document.nodes[*number].token().origin.as_ref(),
        &CssValueOrigin::Programmatic
    );
    let CursorItem::EndOfInput(boundary) = cursor.peek() else {
        panic!("supplied EOF")
    };
    assert!(boundary.source.is_none());
    assert_eq!(
        declaration.value_range,
        SyntaxRange {
            list: document.root,
            start: 2,
            end: 5
        }
    );
    assert_eq!(
        declaration.importance_range,
        Some(SyntaxRange {
            list: document.root,
            start: 5,
            end: 7
        })
    );
}
