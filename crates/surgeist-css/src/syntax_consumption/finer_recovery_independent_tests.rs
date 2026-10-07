#![forbid(unsafe_code)]
use super::*;
use crate::{CssComponentValueLimits, CssHashFlag, CssNumericTokenKind};

fn origin_identity(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    assert_eq!(actual, expected);
    match (actual, expected) {
        (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) => {
            assert!(a.source().same_snapshot(b.source()));
        }
        (
            CssValueOrigin::ImplicitClosure {
                opening: a,
                at: a_end,
            },
            CssValueOrigin::ImplicitClosure {
                opening: b,
                at: b_end,
            },
        ) => {
            assert!(a.source().same_snapshot(b.source()));
            assert!(a_end.source().same_snapshot(b_end.source()));
        }
        _ => {}
    }
}

fn site(boundary: &SyntaxBoundary, snapshot: &CssSourceSnapshot, offset: usize) {
    let site = boundary
        .source
        .as_ref()
        .expect("actual supplied source boundary");
    assert!(site.snapshot.same_snapshot(snapshot));
    assert_eq!(site.offset, offset);
}

#[test]
fn candidate_ranges_stop_before_semicolon_and_keep_whole_document_admission() {
    let text = "outer{a:x;bad;y:z;}after";
    let snapshot = CssSourceSnapshot::new(text);
    let window = SourceWindow {
        text: Cow::Borrowed(text),
        range: 0..24,
        original: snapshot.clone(),
    };
    let limits = SyntaxInputLimits {
        max_depth: 1,
        max_components: 13,
        max_known_spelling_bytes: 24,
    };
    let expected = SyntaxInputMetrics {
        components: 13,
        maximum_depth: 1,
        known_spelling_bytes: 24,
        unspelled_tokens: 0,
    };
    let document = normalize(SyntaxInput::Source(window.clone()), limits, 0).unwrap();
    assert_eq!(document.metrics, expected);
    let [outer, group, after] = document.lists[document.root].as_slice() else {
        panic!("outer ident, actual Curly group, untouched root sibling")
    };
    assert_eq!(
        document.nodes[*outer].token().kind(),
        TokenKind::Ident("outer")
    );
    assert_eq!(
        document.nodes[*after].token().kind(),
        TokenKind::Ident("after")
    );
    let child = document.nodes[*group].children().unwrap();
    let mut selection = document.cursor(child);
    let range = consume_declaration_candidate(&mut selection);
    assert_eq!(
        range,
        SyntaxRange {
            list: child,
            start: 0,
            end: 3
        }
    );
    let CursorItem::Node(semicolon) = selection.peek() else {
        panic!("unconsumed delimiter")
    };
    assert_eq!(
        document.nodes[semicolon].token().kind(),
        TokenKind::Semicolon
    );
    let mut candidate = document.cursor_range(&range);
    let declaration = consume_declaration(&mut candidate).unwrap();
    assert_eq!(
        document.nodes[declaration.name].token().kind(),
        TokenKind::Ident("a")
    );
    assert_eq!(declaration.range, range);
    assert_eq!(declaration.colon, 1);
    assert_eq!(
        declaration.value_range,
        SyntaxRange {
            list: child,
            start: 2,
            end: 3
        }
    );
    assert!(!declaration.important);
    assert!(declaration.importance_range.is_none());
    assert_eq!(declaration.value.len(), 1);
    assert_eq!(
        document.nodes[declaration.value[0]].token().kind(),
        TokenKind::Ident("x")
    );
    let eof = candidate.consume();
    let CursorItem::EndOfInput(boundary) = &eof else {
        panic!("candidate-local EOF")
    };
    site(boundary, &snapshot, 9);
    assert_eq!(candidate.consume(), eof);
    candidate.reconsume_current();
    assert_eq!(candidate.consume(), eof);
    assert_eq!(candidate.consume(), eof);
    assert_eq!(selection.peek(), CursorItem::Node(semicolon));

    let units = consume_declaration_list(&mut document.cursor(child));
    assert_eq!(units.len(), 3);
    for (index, (unit, end)) in units.iter().zip([9, 13, 17]).enumerate() {
        let GenericDeclarationListItem::Declaration { range, parsed } = unit else {
            panic!("raw declaration attempt")
        };
        let mut bounded = document.cursor_range(range);
        while matches!(bounded.consume(), CursorItem::Node(_)) {}
        let CursorItem::EndOfInput(boundary) = bounded.consume() else {
            panic!("local EOF")
        };
        site(&boundary, &snapshot, end);
        match index {
            0 | 2 => {
                let parsed = parsed.as_ref().unwrap();
                assert_eq!(
                    document.nodes[parsed.name].token().kind(),
                    TokenKind::Ident(if index == 0 { "a" } else { "y" })
                );
                let [value] = parsed.value.as_slice() else {
                    panic!("one admitted value")
                };
                assert_eq!(
                    document.nodes[*value].token().kind(),
                    TokenKind::Ident(if index == 0 { "x" } else { "z" })
                );
            }
            1 => assert_eq!(
                parsed.as_ref().unwrap_err().kind,
                GenericFaultKind::InvalidDeclaration
            ),
            _ => unreachable!("three independently authored attempts"),
        }
    }
    site(&document.boundary(child, 10), &snapshot, 18);
    site(&document.boundary(document.root, 3), &snapshot, 24);
    assert_eq!(document.metrics, expected);
    for (too_small, kind) in [
        (
            SyntaxInputLimits {
                max_components: 12,
                ..limits
            },
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            SyntaxInputLimits {
                max_known_spelling_bytes: 23,
                ..limits
            },
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let failure = normalize(SyntaxInput::Source(window.clone()), too_small, 0)
            .err()
            .unwrap();
        assert_eq!(failure.kind(), kind);
        assert_eq!(
            normalize(SyntaxInput::Source(window.clone()), limits, 0)
                .unwrap()
                .metrics,
            expected
        );
    }
}

#[test]
fn direct_checked_and_native_promotion_preserves_payload_groups_and_origin_identity() {
    let first = crate::parse_component_values("é").unwrap();
    let second = crate::parse_component_values("é").unwrap();
    let number = crate::parse_component_values("+01").unwrap();
    let hash = crate::parse_component_values("#12").unwrap();
    let function = crate::parse_component_values("f(a/**/b)").unwrap();
    let block = crate::parse_component_values("{x}").unwrap();
    let implicit_function = crate::parse_component_values("g(z").unwrap();
    let values = [
        first.items()[0].clone(),
        second.items()[0].clone(),
        CssComponentValue::try_ident("constructed").unwrap(),
        number.items()[0].clone(),
        hash.items()[0].clone(),
        function.items()[0].clone(),
        block.items()[0].clone(),
        first.items()[0].clone(),
        implicit_function.items()[0].clone(),
    ];
    let document = normalize(
        SyntaxInput::CheckedComponents(&values),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let promoted = crate::component_values::promote_nodes(
        &document,
        &document.lists[document.root],
        CssComponentValueLimits::default(),
    )
    .unwrap();
    assert_eq!(promoted.items(), &values);
    for (actual, expected) in promoted.items().iter().zip(&values) {
        origin_identity(actual.origin(), expected.origin());
    }
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) =
        (promoted.items()[0].origin(), promoted.items()[1].origin())
    else {
        panic!("two parsed occurrences")
    };
    assert!(!a.source().same_snapshot(b.source()));
    assert!(matches!(
        promoted.items()[2].origin(),
        CssValueOrigin::Programmatic
    ));
    let CssComponentValueRef::Token(CssValueTokenRef::Number(value)) = promoted.items()[3].view()
    else {
        panic!("checked numeric carrier")
    };
    assert_eq!(value.representation(), "+01");
    assert_eq!(value.kind(), CssNumericTokenKind::Integer);
    assert!(value.has_sign());
    assert!(matches!(
        promoted.items()[4].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Hash {
            value: "12",
            flag: CssHashFlag::Unrestricted
        })
    ));
    let CssComponentValueRef::Function(actual) = promoted.items()[5].view() else {
        panic!("one function")
    };
    let CssComponentValueRef::Function(original) = function.items()[0].view() else {
        panic!("original function")
    };
    assert_eq!(actual.name(), "f");
    let [a, comment, b] = actual.values().items() else {
        panic!("ordered function children")
    };
    assert!(matches!(
        a.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("a"))
    ));
    assert!(matches!(comment.view(), CssComponentValueRef::Comment("")));
    assert!(matches!(
        b.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("b"))
    ));
    for (actual, expected) in actual
        .values()
        .items()
        .iter()
        .zip(original.values().items())
    {
        origin_identity(actual.origin(), expected.origin());
    }
    origin_identity(actual.closing_origin(), original.closing_origin());
    let CssComponentValueRef::Block(actual) = promoted.items()[6].view() else {
        panic!("one Curly group")
    };
    let CssComponentValueRef::Block(original) = block.items()[0].view() else {
        panic!("original Curly group")
    };
    assert_eq!(actual.kind(), CssBlockKind::CurlyBracket);
    assert_eq!(actual.values().items(), original.values().items());
    origin_identity(
        actual.values().items()[0].origin(),
        original.values().items()[0].origin(),
    );
    origin_identity(actual.closing_origin(), original.closing_origin());
    origin_identity(promoted.items()[7].origin(), first.items()[0].origin());
    let CssComponentValueRef::Function(actual) = promoted.items()[8].view() else {
        panic!("checked EOF-closed function")
    };
    let CssComponentValueRef::Function(original) = implicit_function.items()[0].view() else {
        panic!("original EOF-closed function")
    };
    assert_eq!(actual.name(), "g");
    assert!(matches!(
        actual.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    assert_eq!(actual.values().items(), original.values().items());
    origin_identity(
        actual.values().items()[0].origin(),
        original.values().items()[0].origin(),
    );
    origin_identity(actual.closing_origin(), original.closing_origin());

    // Real Native source nodes exercise reconstruction, not the checked clone arm.
    for text in ["f(a/**/b){x}", "f(a"] {
        let snapshot = CssSourceSnapshot::new(text);
        let document = source_document(text, &snapshot, 0).unwrap();
        let promoted = crate::component_values::promote_nodes(
            &document,
            &document.lists[document.root],
            CssComponentValueLimits::default(),
        )
        .unwrap();
        let CssComponentValueRef::Function(function) = promoted.items()[0].view() else {
            panic!("native function")
        };
        assert_eq!(function.name(), "f");
        let CssValueOrigin::Parsed(origin) = promoted.items()[0].origin() else {
            panic!("native source origin")
        };
        assert!(origin.source().same_snapshot(&snapshot));
        if text == "f(a" {
            assert_eq!(promoted.items().len(), 1);
            let [a] = function.values().items() else {
                panic!("one native argument")
            };
            assert!(matches!(
                a.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("a"))
            ));
            let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
                panic!("original implicit closing identity")
            };
            assert!(opening.source().same_snapshot(&snapshot));
            assert!(at.source().same_snapshot(&snapshot));
            assert_eq!(opening.span().start().byte_offset().value(), 0);
            assert_eq!(opening.span().end().byte_offset().value(), 2);
            assert_eq!(at.span().start().byte_offset().value(), 3);
            assert_eq!(at.span().end().byte_offset().value(), 3);
        } else {
            assert_eq!(promoted.items().len(), 2);
            let [a, comment, b] = function.values().items() else {
                panic!("native child order")
            };
            assert!(matches!(
                a.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("a"))
            ));
            assert!(matches!(comment.view(), CssComponentValueRef::Comment("")));
            assert!(matches!(
                b.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("b"))
            ));
            let CssComponentValueRef::Block(block) = promoted.items()[1].view() else {
                panic!("native block")
            };
            assert_eq!(block.kind(), CssBlockKind::CurlyBracket);
            let [x] = block.values().items() else {
                panic!("one native block child")
            };
            assert!(matches!(
                x.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("x"))
            ));
            for closing in [function.closing_origin(), block.closing_origin()] {
                let CssValueOrigin::Parsed(origin) = closing else {
                    panic!("actual native closer")
                };
                assert!(origin.source().same_snapshot(&snapshot));
            }
        }
    }
    let snapshot = CssSourceSnapshot::new("\"broken\n");
    let document = source_document(snapshot.as_str(), &snapshot, 0).unwrap();
    let error_node = document
        .nodes
        .iter()
        .position(|node| {
            matches!(
                node,
                SyntaxNode::Error {
                    cause: SyntaxTokenFault::BadString,
                    ..
                }
            )
        })
        .unwrap();
    let failure = crate::component_values::promote_nodes(
        &document,
        &[error_node],
        CssComponentValueLimits::default(),
    )
    .unwrap_err();
    origin_identity(
        failure.origin(),
        document.nodes[error_node].token().origin.as_ref(),
    );
}
