#![forbid(unsafe_code)]
//! Generic declaration boundaries through the existing syntax owner.
//! Oracles: selected Syntax 3 CRD20211224 §§5.3.6, 5.3.8, 5.4.5, 5.4.6.
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
        // The real caller owns leading-trivia handling; §5.4.6 starts at ident.
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
        assert!(crate::parse_declaration(text).syntax().is_none(), "{text}");
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
    let first = crate::parse_component_values("same").unwrap();
    let second = crate::parse_component_values("same").unwrap();
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

#[test]
fn normalized_mixed_lists_retain_at_candidates_and_whole_semicolon_bounded_declarations() {
    let text =
        " ; a:f(x;y); @future f(a;b); @block {hidden:one; deeper{lost:two}} kept:y; last:z; @eof";
    let (_, original) = source(text);
    let checked = crate::parse_component_values(text).unwrap();
    let documents = [
        normalize(
            SyntaxInput::Components {
                document: &original,
                list: original.root,
            },
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap(),
        normalize(
            SyntaxInput::CheckedComponents(checked.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap(),
    ];
    for document in std::iter::once(&original).chain(documents.iter()) {
        let mut cursor = document.cursor(document.root);
        let items = consume_declaration_list(&mut cursor);
        let [
            GenericDeclarationListItem::Declaration {
                parsed: Ok(a),
                range: a_range,
            },
            GenericDeclarationListItem::At(statement),
            GenericDeclarationListItem::At(block),
            GenericDeclarationListItem::Declaration {
                parsed: Ok(kept), ..
            },
            GenericDeclarationListItem::Declaration {
                parsed: Ok(last), ..
            },
            GenericDeclarationListItem::At(at_eof),
        ] = items.as_slice()
        else {
            panic!("three declarations and three at candidates")
        };
        assert_eq!(document.nodes[a.name].token().kind(), TokenKind::Ident("a"));
        assert_eq!(a.range, *a_range);
        assert_eq!(a.value.len(), 1);
        let nested = document.nodes[a.value[0]].children().unwrap();
        assert_eq!(
            spellings(document, &document.lists[nested]),
            ["x", ";", "y"]
        );
        assert_eq!(
            document.nodes[statement.name].token().kind(),
            TokenKind::AtKeyword("future")
        );
        assert!(matches!(
            statement.termination,
            RuleTermination::Semicolon(_)
        ));
        assert!(statement.fault.is_none());
        assert_eq!(
            document.nodes[block.name].token().kind(),
            TokenKind::AtKeyword("block")
        );
        assert!(matches!(block.termination, RuleTermination::Block));
        assert!(block.block.is_some());
        assert_eq!(
            document.nodes[kept.name].token().kind(),
            TokenKind::Ident("kept")
        );
        assert_eq!(
            document.nodes[last.name].token().kind(),
            TokenKind::Ident("last")
        );
        assert_eq!(
            document.nodes[at_eof.name].token().kind(),
            TokenKind::AtKeyword("eof")
        );
        assert!(matches!(at_eof.termination, RuleTermination::EndOfInput(_)));
        assert_eq!(
            at_eof.fault.as_ref().unwrap().kind,
            GenericFaultKind::AtRuleEndOfInput
        );
        assert!(matches!(cursor.peek(), CursorItem::EndOfInput(_)));
    }
}

#[test]
fn malformed_list_units_do_not_promote_grouped_declarations_or_consume_later_siblings() {
    let text = "a:x; ,f(inner;hidden) {lost:yes;} ignored:value; kept:y; missing colon; last:z";
    let (_, document) = source(text);
    let items = consume_declaration_list(&mut document.cursor(document.root));
    let [
        GenericDeclarationListItem::Declaration { parsed: Ok(a), .. },
        GenericDeclarationListItem::Declaration {
            parsed: Err(nonident),
            range: invalid_range,
        },
        GenericDeclarationListItem::Declaration {
            parsed: Ok(kept), ..
        },
        GenericDeclarationListItem::Declaration {
            parsed: Err(missing_colon),
            range: missing_range,
        },
        GenericDeclarationListItem::Declaration {
            parsed: Ok(last), ..
        },
    ] = items.as_slice()
    else {
        panic!("one failed candidate per malformed semicolon-bounded unit")
    };
    assert_eq!(document.nodes[a.name].token().kind(), TokenKind::Ident("a"));
    assert_eq!(nonident.kind, GenericFaultKind::InvalidDeclaration);
    assert_eq!(missing_colon.kind, GenericFaultKind::InvalidDeclaration);
    assert_eq!(
        range_spelling(&document, invalid_range),
        [",", "f(", " ", "{", " ", "ignored", ":", "value"]
    );
    assert_eq!(
        range_spelling(&document, missing_range),
        ["missing", " ", "colon"]
    );
    assert_eq!(
        document.nodes[kept.name].token().kind(),
        TokenKind::Ident("kept")
    );
    assert_eq!(
        document.nodes[last.name].token().kind(),
        TokenKind::Ident("last")
    );
}

#[test]
fn child_list_eof_bounds_a_final_at_candidate_without_exposing_parent_siblings() {
    let text = "before{;a:f(x;y); @pending body} after:z";
    let (snapshot, document) = source(text);
    let [before, group, _, after, _, _] = document.lists[document.root].as_slice() else {
        panic!("actual root body and following declaration")
    };
    assert_eq!(
        document.nodes[*before].token().kind(),
        TokenKind::Ident("before")
    );
    assert_eq!(
        document.nodes[*after].token().kind(),
        TokenKind::Ident("after")
    );
    let child = document.nodes[*group].children().unwrap();
    let mut cursor = document.cursor(child);
    let items = consume_declaration_list(&mut cursor);
    let [
        GenericDeclarationListItem::Declaration { parsed: Ok(a), .. },
        GenericDeclarationListItem::At(at),
    ] = items.as_slice()
    else {
        panic!("only child declaration and EOF at candidate")
    };
    assert_eq!(document.nodes[a.name].token().kind(), TokenKind::Ident("a"));
    assert_eq!(
        document.nodes[at.name].token().kind(),
        TokenKind::AtKeyword("pending")
    );
    assert_eq!(
        at.fault.as_ref().unwrap().kind,
        GenericFaultKind::AtRuleEndOfInput
    );
    eof(&mut cursor, &snapshot, text.find('}').unwrap());
    assert_eq!(
        document.nodes[*after].token().kind(),
        TokenKind::Ident("after")
    );
}
