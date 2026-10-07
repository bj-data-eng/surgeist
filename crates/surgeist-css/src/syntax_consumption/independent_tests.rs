//! Additive owning-module functional evidence for the adopted private API.
//! Install as a child module of syntax_consumption, alongside its existing tests.
use super::*;

fn independent_source(text: &str) -> SyntaxDocument<'static> {
    source_document(text, &CssSourceSnapshot::new(text), 0).unwrap()
}

fn independent_raw(token: Token<'static>, spelling: Option<&'static str>) -> SyntaxToken<'static> {
    SyntaxToken {
        payload: TokenPayload::Native(token),
        spelling: spelling.map(Cow::Borrowed),
        origin: Cow::Owned(CssValueOrigin::Programmatic),
    }
}

fn independent_price(
    document: &SyntaxDocument<'_>,
    components: usize,
    depth: u32,
    bytes: usize,
    unspelled: usize,
) {
    assert_eq!(
        document.metrics,
        SyntaxInputMetrics {
            components,
            maximum_depth: depth,
            known_spelling_bytes: bytes,
            unspelled_tokens: unspelled
        }
    );
}

fn independent_origin(
    origin: &CssValueOrigin,
    snapshot: &CssSourceSnapshot,
    start: usize,
    end: usize,
) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed token origin")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

fn independent_site(boundary: &SyntaxBoundary, snapshot: &CssSourceSnapshot, offset: usize) {
    let site = boundary
        .source
        .as_ref()
        .expect("actual supplied source boundary");
    assert!(site.snapshot.same_snapshot(snapshot));
    assert_eq!(site.offset, offset);
}

#[test]
fn independent_native_numeric_hash_string_and_bad_payloads_survive_direct_input() {
    let tokens = [
        independent_raw(
            Token::Number {
                has_sign: true,
                value: -0.0,
                int_value: None,
            },
            Some("-0.0"),
        ),
        independent_raw(Token::Hash("Id".into()), Some("#Id")),
        independent_raw(Token::IDHash("Id".into()), Some("#Id")),
        independent_raw(Token::QuotedString(";{}".into()), Some("\";{}\"")),
        independent_raw(
            Token::Dimension {
                has_sign: true,
                value: 1.25,
                int_value: None,
                unit: "PX".into(),
            },
            Some("+1.25PX"),
        ),
        independent_raw(
            Token::Percentage {
                has_sign: false,
                unit_value: 0.25,
                int_value: Some(25),
            },
            Some("25%"),
        ),
        independent_raw(Token::BadString("bad".into()), Some("\"bad")),
        independent_raw(Token::BadUrl("url payload".into()), Some("url(a b)")),
    ];
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    independent_price(&document, 8, 0, 37, 0);
    let nodes = &document.lists[document.root];
    let [
        number,
        hash,
        id_hash,
        string,
        dimension,
        percentage,
        bad_string,
        bad_url,
    ] = nodes.as_slice()
    else {
        panic!("eight ordered input occurrences")
    };
    let TokenPayload::Native(Token::Number {
        has_sign,
        value,
        int_value,
    }) = &document.nodes[*number].token().payload
    else {
        panic!("native number")
    };
    assert!(*has_sign);
    assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
    assert_eq!(*int_value, None);
    assert!(
        matches!(&document.nodes[*hash].token().payload, TokenPayload::Native(Token::Hash(name)) if name.as_ref() == "Id")
    );
    assert!(
        matches!(&document.nodes[*id_hash].token().payload, TokenPayload::Native(Token::IDHash(name)) if name.as_ref() == "Id")
    );
    assert!(
        matches!(&document.nodes[*string].token().payload, TokenPayload::Native(Token::QuotedString(value)) if value.as_ref() == ";{}")
    );
    let TokenPayload::Native(Token::Dimension {
        has_sign,
        value,
        int_value,
        unit,
    }) = &document.nodes[*dimension].token().payload
    else {
        panic!("native dimension")
    };
    assert!(*has_sign);
    assert_eq!((*value, *int_value, unit.as_ref()), (1.25, None, "PX"));
    let TokenPayload::Native(Token::Percentage {
        has_sign,
        unit_value,
        int_value,
    }) = &document.nodes[*percentage].token().payload
    else {
        panic!("native percentage")
    };
    assert!(!*has_sign);
    assert_eq!((*unit_value, *int_value), (0.25, Some(25)));
    assert!(
        matches!(&document.nodes[*bad_string], SyntaxNode::Error { token: SyntaxToken { payload: TokenPayload::Native(Token::BadString(value)), .. }, cause: SyntaxTokenFault::BadString } if value.as_ref() == "bad")
    );
    assert!(
        matches!(&document.nodes[*bad_url], SyntaxNode::Error { token: SyntaxToken { payload: TokenPayload::Native(Token::BadUrl(value)), .. }, cause: SyntaxTokenFault::BadUrl } if value.as_ref() == "url payload")
    );
    for (node, original) in nodes.iter().zip(&tokens) {
        assert_eq!(
            document.nodes[*node].token().spelling.as_deref(),
            original.spelling.as_deref()
        );
        assert_eq!(
            document.nodes[*node].token().origin.as_ref(),
            &CssValueOrigin::Programmatic
        );
    }
    assert!(
        document
            .boundary(document.root, nodes.len())
            .source
            .is_none()
    );

    let checked = crate::parse_component_values("+01").unwrap();
    let checked_document = normalize(
        SyntaxInput::CheckedComponents(checked.items()),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    independent_price(&checked_document, 1, 0, 3, 0);
    let [checked_number] = checked_document.lists[checked_document.root].as_slice() else {
        panic!("one checked lexical integer")
    };
    let retained = checked_document.nodes[*checked_number].token();
    let TokenPayload::Checked {
        component,
        lexeme: CheckedLexeme::Leaf,
    } = &retained.payload
    else {
        panic!("actual checked number carrier")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        panic!("lexical numeric payload")
    };
    assert_eq!(number.representation(), "+01");
    assert_eq!(number.kind(), crate::CssNumericTokenKind::Integer);
    assert!(number.has_sign());
    assert_eq!(retained.spelling.as_deref(), Some("+01"));
    assert_eq!(retained.origin.as_ref(), checked.items()[0].origin());
}

#[test]
fn independent_mixed_equal_text_snapshots_and_unspelled_payload_have_no_invented_source() {
    let first = crate::parse_component_values("é").unwrap();
    let second = crate::parse_component_values("é").unwrap();
    let before_first = first.clone();
    let before_second = second.clone();
    let CssValueOrigin::Parsed(first_origin) = first.items()[0].origin() else {
        panic!("first source")
    };
    let CssValueOrigin::Parsed(second_origin) = second.items()[0].origin() else {
        panic!("second source")
    };
    assert!(!first_origin.source().same_snapshot(second_origin.source()));
    let tokens = [
        SyntaxToken::checked(&first.items()[0], CheckedLexeme::Leaf),
        SyntaxToken::checked(&second.items()[0], CheckedLexeme::Leaf),
        independent_raw(
            Token::Number {
                has_sign: false,
                value: 1.0,
                int_value: Some(1),
            },
            None,
        ),
        independent_raw(Token::CurlyBracketBlock, Some("{")),
        independent_raw(Token::CloseCurlyBracket, Some("}")),
    ];
    let limits = SyntaxInputLimits {
        max_depth: 1,
        max_components: 4,
        max_known_spelling_bytes: 6,
    };
    let document = normalize(SyntaxInput::Tokens(&tokens), limits, 0).unwrap();
    independent_price(&document, 4, 1, 6, 1);
    let rule = consume_qualified_rule(&mut document.cursor(document.root)).unwrap();
    let [a, b, number] = rule.prelude.as_slice() else {
        panic!("three direct prelude items, not merged text")
    };
    assert_eq!(document.nodes[*a].token().kind(), TokenKind::Ident("é"));
    assert_eq!(document.nodes[*b].token().kind(), TokenKind::Ident("é"));
    independent_origin(
        document.nodes[*a].token().origin.as_ref(),
        first_origin.source(),
        0,
        2,
    );
    independent_origin(
        document.nodes[*b].token().origin.as_ref(),
        second_origin.source(),
        0,
        2,
    );
    assert!(
        matches!(&document.nodes[*number].token().payload, TokenPayload::Native(Token::Number { value, int_value: Some(1), .. }) if *value == 1.0)
    );
    assert_eq!(
        document.nodes[*number].token().origin.as_ref(),
        &CssValueOrigin::Programmatic
    );
    let child = document.nodes[rule.block].children().unwrap();
    assert!(
        matches!(document.cursor(child).peek(), CursorItem::EndOfInput(boundary) if boundary.source.is_none())
    );
    assert!(
        document
            .boundary(document.root, document.lists[document.root].len())
            .source
            .is_none()
    );
    assert_eq!(
        normalize(
            SyntaxInput::Tokens(&tokens),
            SyntaxInputLimits {
                max_known_spelling_bytes: 5,
                ..limits
            },
            0
        )
        .err()
        .unwrap()
        .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    let retry = normalize(SyntaxInput::Tokens(&tokens), limits, 0).unwrap();
    independent_price(&retry, 4, 1, 6, 1);
    assert_eq!(first, before_first);
    assert_eq!(second, before_second);
}

#[test]
fn independent_bounded_source_and_child_eof_reconsume_keep_their_actual_end_sites() {
    let text = "/*😀*/f(a)tail";
    let snapshot = CssSourceSnapshot::new(text);
    let input = SourceWindow {
        text: Cow::Borrowed(text),
        range: 8..12,
        original: snapshot.clone(),
    };
    let document = normalize(
        SyntaxInput::Source(input),
        SyntaxInputLimits {
            max_depth: 1,
            max_components: 2,
            max_known_spelling_bytes: 4,
        },
        0,
    )
    .unwrap();
    independent_price(&document, 2, 1, 4, 0);
    let mut parent = document.cursor(document.root);
    let first = parent.peek();
    assert_eq!(parent.peek(), first);
    let CursorItem::Node(group) = parent.consume() else {
        panic!("actual bounded function")
    };
    independent_origin(
        document.nodes[group].token().origin.as_ref(),
        &snapshot,
        8,
        10,
    );
    let mut child = document.cursor(document.nodes[group].children().unwrap());
    let CursorItem::Node(argument) = child.consume() else {
        panic!("argument a")
    };
    assert_eq!(
        document.nodes[argument].token().kind(),
        TokenKind::Ident("a")
    );
    independent_origin(
        document.nodes[argument].token().origin.as_ref(),
        &snapshot,
        10,
        11,
    );
    let child_eof = child.consume();
    let CursorItem::EndOfInput(boundary) = &child_eof else {
        panic!("bounded child EOF")
    };
    independent_site(boundary, &snapshot, 11);
    child.reconsume_current();
    assert_eq!(
        child.consume(),
        child_eof,
        "reconsuming conceptual EOF cannot resurrect argument a"
    );
    assert_eq!(child.consume(), child_eof);
    let parent_eof = parent.consume();
    let CursorItem::EndOfInput(boundary) = &parent_eof else {
        panic!("bounded root EOF")
    };
    independent_site(boundary, &snapshot, 12);
    parent.reconsume_current();
    assert_eq!(
        parent.consume(),
        parent_eof,
        "reconsuming EOF cannot resurrect the function"
    );
    independent_price(&document, 2, 1, 4, 0);
    assert_eq!(snapshot.as_str(), text);
}

#[test]
fn independent_grouped_sublist_copy_remaps_implicit_end_and_at_rule_fault_to_its_own_child() {
    let original = independent_source("outer{f(@x");
    let [_, outer] = original.lists[original.root].as_slice() else {
        panic!("outer ident and curly group")
    };
    let outer_children = original.nodes[*outer].children().unwrap();
    let copied = normalize(
        SyntaxInput::Components {
            document: &original,
            list: outer_children,
        },
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    independent_price(&original, 4, 2, 10, 0);
    independent_price(&copied, 2, 1, 4, 0);
    let [function] = copied.lists[copied.root].as_slice() else {
        panic!("copied supplied function, no outer wrapper")
    };
    let children = copied.nodes[*function].children().unwrap();
    let GroupEnd::Implicit { opening, at } = copied.nodes[*function].end().unwrap() else {
        panic!("original implicit function end")
    };
    let snapshot = &original.source.as_ref().unwrap().original;
    independent_origin(opening.as_ref(), snapshot, 6, 8);
    assert_eq!(at.list, children, "boundary belongs to copied child list");
    assert_eq!(at.index, 1);
    independent_site(at, snapshot, 10);
    let mut child = copied.cursor(children);
    let rule = consume_at_rule(&mut child);
    assert_eq!(
        copied.nodes[rule.name].token().kind(),
        TokenKind::AtKeyword("x")
    );
    independent_origin(
        copied.nodes[rule.name].token().origin.as_ref(),
        snapshot,
        8,
        10,
    );
    let RuleTermination::EndOfInput(end) = &rule.termination else {
        panic!("child at-rule EOF")
    };
    assert_eq!(end, at);
    let fault = rule.fault.unwrap();
    assert_eq!(fault.kind, GenericFaultKind::AtRuleEndOfInput);
    assert_eq!(&fault.at, at);
    assert_eq!(
        fault.range,
        SyntaxRange {
            list: children,
            start: 0,
            end: 1
        }
    );
    assert_eq!(child.consume(), CursorItem::EndOfInput(at.clone()));
    child.reconsume_current();
    assert_eq!(child.consume(), CursorItem::EndOfInput(at.clone()));
    independent_site(&copied.boundary(copied.root, 1), snapshot, 10);
}

#[test]
fn independent_repeated_checked_groups_charge_each_delimiter_once_and_retry_atomically() {
    let parsed = crate::parse_component_values("{é}").unwrap();
    let values = [parsed.items()[0].clone(), parsed.items()[0].clone()];
    let before = values.clone();
    let limits = SyntaxInputLimits {
        max_depth: 1,
        max_components: 4,
        max_known_spelling_bytes: 8,
    };
    let document = normalize(SyntaxInput::CheckedComponents(&values), limits, 0).unwrap();
    independent_price(&document, 4, 1, 8, 0); // 2 × (group+ident), 2 × (1+2+1 bytes)
    let CssComponentValueRef::Block(original_block) = parsed.items()[0].view() else {
        panic!("source block")
    };
    for (limit, kind, origin) in [
        (
            SyntaxInputLimits {
                max_components: 3,
                ..limits
            },
            CssComponentValueErrorKind::ComponentLimit,
            original_block.values().items()[0].origin(),
        ),
        (
            SyntaxInputLimits {
                max_known_spelling_bytes: 7,
                ..limits
            },
            CssComponentValueErrorKind::ByteLimit,
            original_block.closing_origin(),
        ),
        (
            SyntaxInputLimits {
                max_depth: 0,
                ..limits
            },
            CssComponentValueErrorKind::NestingLimit,
            parsed.items()[0].origin(),
        ),
    ] {
        let failure = normalize(SyntaxInput::CheckedComponents(&values), limit, 0)
            .err()
            .expect("no usable partial document");
        assert_eq!(failure.kind(), kind);
        assert_eq!(failure.origin(), origin);
        assert_eq!(values, before);
        let retry = normalize(SyntaxInput::CheckedComponents(&values), limits, 0).unwrap();
        independent_price(&retry, 4, 1, 8, 0);
        for node in &retry.lists[retry.root] {
            assert_eq!(
                retry.nodes[*node].token().origin.as_ref(),
                parsed.items()[0].origin()
            );
            let GroupEnd::Explicit(closing) = retry.nodes[*node].end().unwrap() else {
                panic!("actual matching closer")
            };
            assert_eq!(closing.origin.as_ref(), original_block.closing_origin());
            assert_eq!(closing.spelling.as_deref(), Some("}"));
            let child = retry.nodes[*node].children().unwrap();
            let [ident] = retry.lists[child].as_slice() else {
                panic!("one repeated child")
            };
            assert_eq!(retry.nodes[*ident].token().kind(), TokenKind::Ident("é"));
            assert_eq!(
                retry.nodes[*ident].token().origin.as_ref(),
                original_block.values().items()[0].origin()
            );
        }
    }
}

#[test]
fn independent_unspelled_zero_byte_budget_and_base_depth_ceiling_are_distinct() {
    let token = independent_raw(Token::Ident("é".into()), None);
    let tokens = [token.clone(), token];
    let limits = SyntaxInputLimits {
        max_depth: 0,
        max_components: 2,
        max_known_spelling_bytes: 0,
    };
    let document = normalize(SyntaxInput::Tokens(&tokens), limits, 0).unwrap();
    independent_price(&document, 2, 0, 0, 2);
    let failure = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits {
            max_components: 1,
            ..limits
        },
        0,
    )
    .err()
    .unwrap();
    assert_eq!(failure.kind(), CssComponentValueErrorKind::ComponentLimit);
    assert_eq!(failure.origin(), &CssValueOrigin::Programmatic);
    independent_price(
        &normalize(SyntaxInput::Tokens(&tokens), limits, 0).unwrap(),
        2,
        0,
        0,
        2,
    );
    assert!(document.boundary(document.root, 2).source.is_none());
    let snapshot = CssSourceSnapshot::new("f()");
    let window = SourceWindow {
        text: Cow::Borrowed("f()"),
        range: 0..3,
        original: snapshot,
    };
    let limits = SyntaxInputLimits {
        max_depth: 256,
        max_components: 1,
        max_known_spelling_bytes: 3,
    };
    let at_ceiling = normalize(SyntaxInput::Source(window.clone()), limits, 255).unwrap();
    independent_price(&at_ceiling, 1, 256, 3, 0);
    assert_eq!(
        normalize(SyntaxInput::Source(window.clone()), limits, 256)
            .err()
            .unwrap()
            .kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    assert_eq!(
        normalize(
            SyntaxInput::Source(window.clone()),
            SyntaxInputLimits {
                max_depth: u32::MAX,
                ..limits
            },
            256
        )
        .err()
        .unwrap()
        .kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    independent_price(
        &normalize(SyntaxInput::Source(window), limits, 255).unwrap(),
        1,
        256,
        3,
        0,
    );
}

#[test]
fn independent_exact_one_rejects_without_consuming_second_unit_but_charges_all_input() {
    let text = "@x;/*c*/@y;";
    let snapshot = CssSourceSnapshot::new(text);
    let window = SourceWindow {
        text: Cow::Borrowed(text),
        range: 0..11,
        original: snapshot.clone(),
    };
    let limits = SyntaxInputLimits {
        max_depth: 0,
        max_components: 5,
        max_known_spelling_bytes: 11,
    };
    let document = normalize(SyntaxInput::Source(window.clone()), limits, 0).unwrap();
    independent_price(&document, 5, 0, 11, 0);
    let mut cursor = document.cursor(document.root);
    let fault = consume_one_rule(&mut cursor).unwrap_err();
    assert_eq!(fault.kind, GenericFaultKind::TrailingInput);
    independent_site(&fault.at, &snapshot, 8);
    let CursorItem::Node(second) = cursor.peek() else {
        panic!("unconsumed second at-rule name")
    };
    assert_eq!(
        document.nodes[second].token().kind(),
        TokenKind::AtKeyword("y")
    );
    assert_eq!(cursor.consume(), CursorItem::Node(second));
    let CursorItem::Node(semicolon) = cursor.peek() else {
        panic!("second unit was not consumed to reject it")
    };
    assert_eq!(
        document.nodes[semicolon].token().kind(),
        TokenKind::Semicolon
    );
    assert_eq!(
        normalize(
            SyntaxInput::Source(window.clone()),
            SyntaxInputLimits {
                max_components: 4,
                ..limits
            },
            0
        )
        .err()
        .unwrap()
        .kind(),
        CssComponentValueErrorKind::ComponentLimit
    );
    assert_eq!(
        normalize(
            SyntaxInput::Source(window.clone()),
            SyntaxInputLimits {
                max_known_spelling_bytes: 10,
                ..limits
            },
            0
        )
        .err()
        .unwrap()
        .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    independent_price(
        &normalize(SyntaxInput::Source(window), limits, 0).unwrap(),
        5,
        0,
        11,
        0,
    );
    let terminated = independent_source("@x;/*c*/");
    independent_price(&terminated, 3, 0, 8, 0);
    let GenericRule::At(rule) = consume_one_rule(&mut terminated.cursor(terminated.root)).unwrap()
    else {
        panic!("one terminated rule")
    };
    assert!(matches!(rule.termination, RuleTermination::Semicolon(_)));
    assert!(
        rule.fault.is_none(),
        "trailing comment plus final EOF cannot change the earlier semicolon termination"
    );
}

#[test]
fn independent_mismatched_closers_cannot_pop_an_ancestor_or_reorder_later_siblings() {
    let document = independent_source("([)]}x)y");
    independent_price(&document, 6, 2, 8, 0);
    let snapshot = &document.source.as_ref().unwrap().original;
    let [paren, y] = document.lists[document.root].as_slice() else {
        panic!("parenthesis group and later root sibling")
    };
    assert_eq!(document.nodes[*y].token().kind(), TokenKind::Ident("y"));
    independent_origin(document.nodes[*y].token().origin.as_ref(), snapshot, 7, 8);
    let parent_children = document.nodes[*paren].children().unwrap();
    let [square, bad_curly, x] = document.lists[parent_children].as_slice() else {
        panic!("nested square, wrong closer, x")
    };
    let square_children = document.nodes[*square].children().unwrap();
    let [bad_paren] = document.lists[square_children].as_slice() else {
        panic!("mismatched paren stays inside active square")
    };
    for (node, kind, spelling, start) in [
        (*bad_paren, CssBlockKind::Parenthesis, ")", 2),
        (*bad_curly, CssBlockKind::CurlyBracket, "}", 4),
    ] {
        assert!(matches!(
            document.nodes[node],
            SyntaxNode::Error {
                cause: SyntaxTokenFault::UnexpectedCloser,
                ..
            }
        ));
        assert_eq!(
            document.nodes[node].token().kind(),
            TokenKind::Closing(kind)
        );
        assert_eq!(
            document.nodes[node].token().spelling.as_deref(),
            Some(spelling)
        );
        independent_origin(
            document.nodes[node].token().origin.as_ref(),
            snapshot,
            start,
            start + 1,
        );
    }
    assert_eq!(document.nodes[*x].token().kind(), TokenKind::Ident("x"));
    independent_site(&document.boundary(square_children, 1), snapshot, 3);
    independent_site(&document.boundary(parent_children, 3), snapshot, 6);
    let GroupEnd::Explicit(square_end) = document.nodes[*square].end().unwrap() else {
        panic!("matching square closer")
    };
    independent_origin(square_end.origin.as_ref(), snapshot, 3, 4);
    let GroupEnd::Explicit(paren_end) = document.nodes[*paren].end().unwrap() else {
        panic!("matching parenthesis closer")
    };
    independent_origin(paren_end.origin.as_ref(), snapshot, 6, 7);
}
