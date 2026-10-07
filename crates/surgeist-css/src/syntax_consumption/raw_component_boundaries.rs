#![forbid(unsafe_code)]
//! Raw component boundaries through the normalized syntax arena.
//! Syntax 3 CRD20211224 §§5.3.10, 5.4.7–9 define the independent raw oracles.
//! Exact/comma tests call the real adopted production selectors, never a test splitter.
use super::*;

fn source(text: &str) -> (CssSourceSnapshot, SyntaxDocument<'static>) {
    let snapshot = CssSourceSnapshot::new(text);
    let document = source_document(text, &snapshot, 0).unwrap();
    (snapshot, document)
}

fn next(cursor: &mut SyntaxCursor<'_>, expected: TokenKind<'_>) -> NodeId {
    let CursorItem::Node(node) = cursor.consume() else {
        panic!("literal component before bounded EOF")
    };
    assert_eq!(cursor.document.nodes[node].token().kind(), expected);
    node
}

fn site(boundary: &SyntaxBoundary, snapshot: &CssSourceSnapshot, offset: usize) {
    let source = boundary
        .source
        .as_ref()
        .expect("actual original bounded source endpoint");
    assert!(source.snapshot.same_snapshot(snapshot));
    assert_eq!(source.offset, offset);
}

fn end(cursor: &mut SyntaxCursor<'_>) -> SyntaxBoundary {
    let CursorItem::EndOfInput(boundary) = cursor.consume() else {
        panic!("conceptual local EOF")
    };
    cursor.reconsume_current();
    assert_eq!(cursor.consume(), CursorItem::EndOfInput(boundary.clone()));
    assert_eq!(cursor.consume(), CursorItem::EndOfInput(boundary.clone()));
    boundary
}

fn occurrence(
    document: &SyntaxDocument<'_>,
    node: NodeId,
    snapshot: &CssSourceSnapshot,
    spelling: &str,
    range: Range<usize>,
) {
    let token = document.nodes[node].token();
    assert_eq!(token.spelling.as_deref(), Some(spelling));
    let CssValueOrigin::Parsed(origin) = token.origin.as_ref() else {
        panic!("original token occurrence")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

#[test]
fn all_component_cursor_preserves_trivia_errors_and_later_tokens_around_complete_nested_groups() {
    let text = " f([)] x) url(a b) \"bad\nz ;]";
    let (snapshot, document) = source(text);
    let before = document.metrics;
    let mut root = document.cursor(document.root);
    let leading = next(&mut root, TokenKind::Whitespace);
    occurrence(&document, leading, &snapshot, " ", 0..1);
    let function = next(&mut root, TokenKind::Function("f"));
    occurrence(&document, function, &snapshot, "f(", 1..3);
    assert_eq!(document.range(function), Some(1..9));
    let parent_position = root.position();
    let mut arguments = document.cursor(document.nodes[function].children().unwrap());
    let square = next(
        &mut arguments,
        TokenKind::Opening(CssBlockKind::SquareBracket),
    );
    occurrence(&document, square, &snapshot, "[", 3..4);
    assert_eq!(document.range(square), Some(3..6));
    let mut square_contents = document.cursor(document.nodes[square].children().unwrap());
    let mismatch = next(
        &mut square_contents,
        TokenKind::Closing(CssBlockKind::Parenthesis),
    );
    assert!(matches!(
        document.nodes[mismatch],
        SyntaxNode::Error {
            cause: SyntaxTokenFault::UnexpectedCloser,
            ..
        }
    ));
    occurrence(&document, mismatch, &snapshot, ")", 4..5);
    site(&end(&mut square_contents), &snapshot, 5);
    let GroupEnd::Explicit(close_square) = document.nodes[square].end().unwrap() else {
        panic!("square's own matching closer")
    };
    assert_eq!(
        close_square.kind(),
        TokenKind::Closing(CssBlockKind::SquareBracket)
    );
    next(&mut arguments, TokenKind::Whitespace);
    let x = next(&mut arguments, TokenKind::Ident("x"));
    occurrence(&document, x, &snapshot, "x", 7..8);
    site(&end(&mut arguments), &snapshot, 8);
    let GroupEnd::Explicit(close_function) = document.nodes[function].end().unwrap() else {
        panic!("function's own matching closer")
    };
    assert_eq!(
        close_function.kind(),
        TokenKind::Closing(CssBlockKind::Parenthesis)
    );
    assert_eq!(
        root.position(),
        parent_position,
        "child consumption cannot advance its parent"
    );

    next(&mut root, TokenKind::Whitespace);
    let bad_url = next(&mut root, TokenKind::BadUrl);
    assert!(matches!(
        document.nodes[bad_url],
        SyntaxNode::Error {
            cause: SyntaxTokenFault::BadUrl,
            ..
        }
    ));
    assert!(
        matches!(&document.nodes[bad_url].token().payload, TokenPayload::Native(Token::BadUrl(value)) if value.as_ref() == "a b")
    );
    occurrence(&document, bad_url, &snapshot, "url(a b)", 10..18);
    next(&mut root, TokenKind::Whitespace);
    let bad_string = next(&mut root, TokenKind::BadString);
    assert!(matches!(
        document.nodes[bad_string],
        SyntaxNode::Error {
            cause: SyntaxTokenFault::BadString,
            ..
        }
    ));
    assert!(
        matches!(&document.nodes[bad_string].token().payload, TokenPayload::Native(Token::BadString(value)) if value.as_ref() == "bad")
    );
    occurrence(&document, bad_string, &snapshot, "\"bad", 19..23);
    let newline = next(&mut root, TokenKind::Whitespace);
    occurrence(&document, newline, &snapshot, "\n", 23..24);
    let z = next(&mut root, TokenKind::Ident("z"));
    occurrence(&document, z, &snapshot, "z", 24..25);
    next(&mut root, TokenKind::Whitespace);
    let semicolon = next(&mut root, TokenKind::Semicolon);
    occurrence(&document, semicolon, &snapshot, ";", 26..27);
    let stray = next(&mut root, TokenKind::Closing(CssBlockKind::SquareBracket));
    assert!(matches!(
        document.nodes[stray],
        SyntaxNode::Error {
            cause: SyntaxTokenFault::UnexpectedCloser,
            ..
        }
    ));
    occurrence(&document, stray, &snapshot, "]", 27..28);
    site(&end(&mut root), &snapshot, 28);
    assert_eq!(
        document.metrics, before,
        "cursor traversal does not readmit or recharge components"
    );
}

#[test]
fn mismatched_closer_stays_inside_its_group_while_all_open_ancestors_close_at_actual_eof() {
    let text = " f({a[)b";
    let (snapshot, original) = source(text);
    let copied = normalize(
        SyntaxInput::Components {
            document: &original,
            list: original.root,
        },
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    for (document, source_input) in [(&original, true), (&copied, false)] {
        let mut root = document.cursor(document.root);
        next(&mut root, TokenKind::Whitespace);
        let function = next(&mut root, TokenKind::Function("f"));
        let mut function_contents = document.cursor(document.nodes[function].children().unwrap());
        let curly = next(
            &mut function_contents,
            TokenKind::Opening(CssBlockKind::CurlyBracket),
        );
        let mut curly_contents = document.cursor(document.nodes[curly].children().unwrap());
        next(&mut curly_contents, TokenKind::Ident("a"));
        let square = next(
            &mut curly_contents,
            TokenKind::Opening(CssBlockKind::SquareBracket),
        );
        let mut square_contents = document.cursor(document.nodes[square].children().unwrap());
        let mismatch = next(
            &mut square_contents,
            TokenKind::Closing(CssBlockKind::Parenthesis),
        );
        assert!(matches!(
            document.nodes[mismatch],
            SyntaxNode::Error {
                cause: SyntaxTokenFault::UnexpectedCloser,
                ..
            }
        ));
        occurrence(document, mismatch, &snapshot, ")", 6..7);
        let b = next(&mut square_contents, TokenKind::Ident("b"));
        occurrence(document, b, &snapshot, "b", 7..8);
        for (group, expected_range, opening_range) in [
            (function, 1..8, 1..3),
            (curly, 3..8, 3..4),
            (square, 5..8, 5..6),
        ] {
            if source_input {
                assert_eq!(document.range(group), Some(expected_range));
            }
            let GroupEnd::Implicit { opening, at } = document.nodes[group].end().unwrap() else {
                panic!("EOF returns each still-open object with its implicit-end fault")
            };
            let CssValueOrigin::Parsed(opening) = opening.as_ref() else {
                panic!("actual source opener")
            };
            assert!(opening.source().same_snapshot(&snapshot));
            assert_eq!(
                opening.span().start().byte_offset().value(),
                opening_range.start
            );
            assert_eq!(
                opening.span().end().byte_offset().value(),
                opening_range.end
            );
            assert_eq!(at.list, document.nodes[group].children().unwrap());
            site(at, &snapshot, 8);
        }
        site(&end(&mut square_contents), &snapshot, 8);
        site(&end(&mut curly_contents), &snapshot, 8);
        site(&end(&mut function_contents), &snapshot, 8);
        site(&end(&mut root), &snapshot, 8);
    }
}

#[test]
fn bounded_source_window_uses_its_own_eof_instead_of_a_matching_closer_outside_the_window() {
    let text = "prefix f(a] b) outside";
    let snapshot = CssSourceSnapshot::new(text);
    let document = normalize(
        SyntaxInput::Source(SourceWindow {
            text: Cow::Borrowed(text),
            range: 7..13,
            original: snapshot.clone(),
        }),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let mut root = document.cursor(document.root);
    let function = next(&mut root, TokenKind::Function("f"));
    assert_eq!(document.range(function), Some(7..13));
    let mut child = document.cursor(document.nodes[function].children().unwrap());
    next(&mut child, TokenKind::Ident("a"));
    let stray = next(&mut child, TokenKind::Closing(CssBlockKind::SquareBracket));
    occurrence(&document, stray, &snapshot, "]", 10..11);
    assert!(matches!(
        document.nodes[stray],
        SyntaxNode::Error {
            cause: SyntaxTokenFault::UnexpectedCloser,
            ..
        }
    ));
    next(&mut child, TokenKind::Whitespace);
    next(&mut child, TokenKind::Ident("b"));
    site(&end(&mut child), &snapshot, 13);
    site(&end(&mut root), &snapshot, 13);
    let GroupEnd::Implicit { at, .. } = document.nodes[function].end().unwrap() else {
        panic!("outside ')' cannot close bounded input")
    };
    site(at, &snapshot, 13);
    assert_eq!(&snapshot.as_str()[13..], ") outside");
}

#[test]
fn unspelled_native_error_payloads_keep_component_order_and_do_not_invent_source_or_relex_punctuation()
 {
    let native = |token| SyntaxToken {
        payload: TokenPayload::Native(token),
        spelling: None,
        origin: Cow::Owned(CssValueOrigin::Programmatic),
    };
    let tokens = [
        native(Token::Function("f".into())),
        native(Token::BadString("payload;]".into())),
        native(Token::WhiteSpace(" \n")),
        native(Token::CloseSquareBracket),
        native(Token::Ident("inner".into())),
        native(Token::CloseParenthesis),
        native(Token::BadUrl("url payload;(".into())),
        native(Token::Ident("later".into())),
    ];
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let mut root = document.cursor(document.root);
    let function = next(&mut root, TokenKind::Function("f"));
    let mut child = document.cursor(document.nodes[function].children().unwrap());
    let bad_string = next(&mut child, TokenKind::BadString);
    assert!(
        matches!(&document.nodes[bad_string], SyntaxNode::Error { token: SyntaxToken { payload: TokenPayload::Native(Token::BadString(value)), .. }, cause: SyntaxTokenFault::BadString } if value.as_ref() == "payload;]")
    );
    next(&mut child, TokenKind::Whitespace);
    let mismatch = next(&mut child, TokenKind::Closing(CssBlockKind::SquareBracket));
    assert!(matches!(
        document.nodes[mismatch],
        SyntaxNode::Error {
            cause: SyntaxTokenFault::UnexpectedCloser,
            ..
        }
    ));
    next(&mut child, TokenKind::Ident("inner"));
    assert!(end(&mut child).source.is_none());
    let GroupEnd::Explicit(close) = document.nodes[function].end().unwrap() else {
        panic!("supplied actual matching close")
    };
    assert_eq!(close.kind(), TokenKind::Closing(CssBlockKind::Parenthesis));
    assert!(close.spelling.is_none());
    let bad_url = next(&mut root, TokenKind::BadUrl);
    assert!(
        matches!(&document.nodes[bad_url], SyntaxNode::Error { token: SyntaxToken { payload: TokenPayload::Native(Token::BadUrl(value)), .. }, cause: SyntaxTokenFault::BadUrl } if value.as_ref() == "url payload;(")
    );
    next(&mut root, TokenKind::Ident("later"));
    assert!(end(&mut root).source.is_none());
    assert!(document.source.is_none());
    for node in &document.nodes {
        assert!(node.token().spelling.is_none());
        assert_eq!(node.token().origin.as_ref(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn checked_implicit_groups_keep_original_component_occurrences_and_child_eof_without_fabricating_root_source()
 {
    let text = " f([x";
    let values = crate::parse_component_values(text).unwrap();
    let CssComponentValueRef::Function(original_function) = values.items()[1].view() else {
        panic!("actual checked function")
    };
    let original_square = &original_function.values().items()[0];
    let document = normalize(
        SyntaxInput::CheckedComponents(values.items()),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let mut root = document.cursor(document.root);
    next(&mut root, TokenKind::Whitespace);
    let function = next(&mut root, TokenKind::Function("f"));
    let TokenPayload::Checked {
        component,
        lexeme: CheckedLexeme::Opening,
    } = &document.nodes[function].token().payload
    else {
        panic!("actual supplied function occurrence")
    };
    assert!(std::ptr::eq(*component, &values.items()[1]));
    let CssValueOrigin::Parsed(opening) = component.origin() else {
        panic!("original checked source")
    };
    let snapshot = opening.source();
    let mut child = document.cursor(document.nodes[function].children().unwrap());
    let square = next(&mut child, TokenKind::Opening(CssBlockKind::SquareBracket));
    let TokenPayload::Checked {
        component,
        lexeme: CheckedLexeme::Opening,
    } = &document.nodes[square].token().payload
    else {
        panic!("actual supplied block occurrence")
    };
    assert!(std::ptr::eq(*component, original_square));
    let mut square_contents = document.cursor(document.nodes[square].children().unwrap());
    let x = next(&mut square_contents, TokenKind::Ident("x"));
    occurrence(&document, x, snapshot, "x", 4..5);
    for group in [square, function] {
        let GroupEnd::Implicit { at, .. } = document.nodes[group].end().unwrap() else {
            panic!("original EOF-implied delimiter")
        };
        assert_eq!(at.list, document.nodes[group].children().unwrap());
        site(at, snapshot, 5);
    }
    site(&end(&mut square_contents), snapshot, 5);
    site(&end(&mut child), snapshot, 5);
    assert!(
        end(&mut root).source.is_none(),
        "supplied list has no authored root end beyond its actual retained child origins"
    );
}

#[test]
fn exact_raw_selector_ignores_outer_trivia_but_retains_one_error_or_implicit_group() {
    for (text, expected) in [
        (
            " /**/ ] /*tail*/ ",
            TokenKind::Closing(CssBlockKind::SquareBracket),
        ),
        (" /**/ f([x", TokenKind::Function("f")),
    ] {
        let (snapshot, document) = source(text);
        let metrics = document.metrics;
        let mut cursor = document.cursor(document.root);
        let node = consume_one_component(&mut cursor).unwrap();
        assert_eq!(document.nodes[node].token().kind(), expected);
        if matches!(expected, TokenKind::Closing(_)) {
            assert!(matches!(
                document.nodes[node],
                SyntaxNode::Error {
                    cause: SyntaxTokenFault::UnexpectedCloser,
                    ..
                }
            ));
            occurrence(&document, node, &snapshot, "]", 6..7);
        } else {
            let GroupEnd::Implicit { at, .. } = document.nodes[node].end().unwrap() else {
                panic!("raw exact success retains implicit-end fault")
            };
            site(at, &snapshot, text.len());
        }
        site(&end(&mut cursor), &snapshot, text.len());
        assert_eq!(document.metrics, metrics);
    }
    let (snapshot, document) = source(" /**/ ");
    let fault = consume_one_component(&mut document.cursor(document.root)).unwrap_err();
    assert_eq!(fault.kind, GenericFaultKind::EmptyInput);
    site(&fault.at, &snapshot, 6);
    assert_eq!(fault.range.list, document.root);
    assert_eq!(fault.range.start, fault.range.end);
    let (snapshot, document) = source(" a /**/ f(x) ");
    let fault = consume_one_component(&mut document.cursor(document.root)).unwrap_err();
    assert_eq!(fault.kind, GenericFaultKind::TrailingInput);
    site(&fault.at, &snapshot, 8);
    assert_eq!(
        document.nodes[document.lists[fault.at.list][fault.at.index]]
            .token()
            .kind(),
        TokenKind::Function("f")
    );
}

#[test]
fn comma_ranges_preserve_all_nested_kinds_trivia_and_final_empty_members() {
    let (snapshot, document) = source("a,f(x,y),[x,y],(x,y),{x,y}, /*c*/ ,,");
    let metrics = document.metrics;
    let ranges = consume_comma_separated_components(&mut document.cursor(document.root));
    assert_eq!(
        ranges.iter().map(|r| (r.start, r.end)).collect::<Vec<_>>(),
        [
            (0, 1),
            (2, 3),
            (4, 5),
            (6, 7),
            (8, 9),
            (10, 13),
            (14, 14),
            (15, 15)
        ]
    );
    for range in &ranges {
        assert_eq!(range.list, document.root);
    }
    for (range, kind) in ranges[1..5].iter().zip([
        TokenKind::Function("f"),
        TokenKind::Opening(CssBlockKind::SquareBracket),
        TokenKind::Opening(CssBlockKind::Parenthesis),
        TokenKind::Opening(CssBlockKind::CurlyBracket),
    ]) {
        let node = document.lists[range.list][range.start];
        assert_eq!(document.nodes[node].token().kind(), kind);
        assert_eq!(
            document.lists[document.nodes[node].children().unwrap()].len(),
            3
        );
    }
    let trivia = &document.lists[document.root][10..13];
    assert_eq!(
        trivia
            .iter()
            .map(|n| document.nodes[*n].token().kind())
            .collect::<Vec<_>>(),
        [
            TokenKind::Whitespace,
            TokenKind::Comment,
            TokenKind::Whitespace
        ]
    );
    site(&end(&mut document.cursor_range(&ranges[7])), &snapshot, 36);
    assert_eq!(document.metrics, metrics);
    let (_, empty) = source("");
    assert_eq!(
        consume_comma_separated_components(&mut empty.cursor(empty.root)),
        [SyntaxRange {
            list: empty.root,
            start: 0,
            end: 0
        }]
    );
}

#[test]
fn raw_comma_selector_uses_actual_native_and_checked_kinds_and_keeps_mixed_origins() {
    let first = crate::parse_component_values("A").unwrap();
    let comma = crate::parse_component_values(",").unwrap();
    let last = crate::parse_component_values("B").unwrap();
    let native = |token| SyntaxToken {
        payload: TokenPayload::Native(token),
        spelling: None,
        origin: Cow::Owned(CssValueOrigin::Programmatic),
    };
    let tokens = [
        SyntaxToken::checked(&first.items()[0], CheckedLexeme::Leaf),
        native(Token::Comma),
        native(Token::BadString("payload,]".into())),
        SyntaxToken::checked(&comma.items()[0], CheckedLexeme::Leaf),
        native(Token::Ident("literal,ident".into())),
        native(Token::Comma),
        SyntaxToken::checked(&last.items()[0], CheckedLexeme::Leaf),
        native(Token::Comma),
    ];
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let ranges = consume_comma_separated_components(&mut document.cursor(document.root));
    assert_eq!(
        ranges.iter().map(|r| (r.start, r.end)).collect::<Vec<_>>(),
        [(0, 1), (2, 3), (4, 5), (6, 7), (8, 8)]
    );
    let ids = &document.lists[document.root];
    assert_eq!(
        consume_one_component(&mut document.cursor_range(&ranges[1])).unwrap(),
        ids[2],
        "one raw bad token remains a component, without checked promotion"
    );
    assert!(
        matches!(&document.nodes[ids[2]], SyntaxNode::Error { token: SyntaxToken { payload: TokenPayload::Native(Token::BadString(value)), spelling: None, .. }, cause: SyntaxTokenFault::BadString } if value.as_ref() == "payload,]")
    );
    assert_eq!(
        document.nodes[ids[4]].token().kind(),
        TokenKind::Ident("literal,ident")
    );
    for (index, original) in [(0, &first.items()[0]), (6, &last.items()[0])] {
        let TokenPayload::Checked { component, .. } = &document.nodes[ids[index]].token().payload
        else {
            panic!("borrowed original checked occurrence")
        };
        assert!(std::ptr::eq(*component, original));
        assert_eq!(
            document.nodes[ids[index]].token().origin.as_ref(),
            original.origin()
        );
    }
    let CssValueOrigin::Parsed(a) = first.items()[0].origin() else {
        panic!("A source")
    };
    let CssValueOrigin::Parsed(b) = last.items()[0].origin() else {
        panic!("B source")
    };
    assert!(!a.source().same_snapshot(b.source()));
    assert!(document.source.is_none());
    assert!(end(&mut document.cursor_range(&ranges[4])).source.is_none());
}

#[test]
fn selectors_respect_child_and_candidate_eof_without_consuming_parent_or_excluded_comma() {
    let (snapshot, document) = source("f(a,,)tail");
    let mut parent = document.cursor(document.root);
    let function = next(&mut parent, TokenKind::Function("f"));
    let parent_position = parent.position();
    let child = document.nodes[function].children().unwrap();
    let ranges = consume_comma_separated_components(&mut document.cursor(child));
    assert_eq!(
        ranges,
        [
            SyntaxRange {
                list: child,
                start: 0,
                end: 1
            },
            SyntaxRange {
                list: child,
                start: 2,
                end: 2
            },
            SyntaxRange {
                list: child,
                start: 3,
                end: 3
            }
        ]
    );
    let node = consume_one_component(&mut document.cursor_range(&ranges[0])).unwrap();
    assert_eq!(node, document.lists[child][0]);
    occurrence(&document, node, &snapshot, "a", 2..3);
    site(&end(&mut document.cursor_range(&ranges[2])), &snapshot, 5);
    let empty = consume_one_component(&mut document.cursor_range(&ranges[2])).unwrap_err();
    assert_eq!(empty.kind, GenericFaultKind::EmptyInput);
    site(&empty.at, &snapshot, 5);
    let mut candidate = document.cursor_range(&ranges[0]);
    consume_one_component(&mut candidate).unwrap();
    site(&end(&mut candidate), &snapshot, 3);
    assert_eq!(parent.position(), parent_position);
    let tail = next(&mut parent, TokenKind::Ident("tail"));
    occurrence(&document, tail, &snapshot, "tail", 6..10);
}
