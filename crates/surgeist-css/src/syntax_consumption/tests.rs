use super::*;

fn source(text: &str) -> SyntaxDocument<'static> {
    source_document(text, &CssSourceSnapshot::new(text), 0).unwrap()
}
fn raw(token: Token<'static>, spelling: Option<&'static str>) -> SyntaxToken<'static> {
    SyntaxToken {
        payload: TokenPayload::Native(token),
        spelling: spelling.map(Cow::Borrowed),
        origin: Cow::Owned(CssValueOrigin::Programmatic),
    }
}

#[test]
fn candidate_cursor_eof_is_the_excluded_semicolon_without_recharging_input() {
    let document = source("name: f(a;b) ! IMPORTANT ; next: yes");
    let metrics = (
        document.metrics.components,
        document.metrics.known_spelling_bytes,
    );
    let mut root = document.cursor(document.root);
    let range = consume_declaration_candidate(&mut root);
    let semicolon = root.peek();
    let declaration = consume_declaration(&mut document.cursor_range(&range)).unwrap();
    assert!(declaration.important);
    assert_eq!(declaration.value.len(), 1);
    assert!(matches!(
        document.nodes[declaration.value[0]],
        SyntaxNode::Function { .. }
    ));
    let mut candidate = document.cursor_range(&range);
    while matches!(candidate.consume(), CursorItem::Node(_)) {}
    let CursorItem::EndOfInput(at) = candidate.peek() else {
        panic!("candidate EOF");
    };
    assert_eq!(at.source.unwrap().offset, "name: f(a;b) ! IMPORTANT ".len());
    candidate.reconsume_current();
    assert!(matches!(candidate.consume(), CursorItem::EndOfInput(_)));
    assert_eq!(root.peek(), semicolon);
    assert_eq!(
        (
            document.metrics.components,
            document.metrics.known_spelling_bytes
        ),
        metrics
    );
}

#[test]
fn admitted_node_promotion_preserves_provider_payloads_and_implicit_group_owner() {
    let document = source("-0 #id #123 f(/*c*/A");
    let values = crate::component::values::parse::promote_nodes(
        &document,
        &document.lists[document.root],
        crate::component::CssComponentValueLimits::default(),
    )
    .unwrap();
    let expected =
        crate::component::values::parse_component_values("-0 #id #123 f(/*c*/A").unwrap();
    assert_eq!(values, expected);
    let original = &document.source.as_ref().unwrap().original;
    let CssValueOrigin::ImplicitClosure { opening, at } = values.first_implicit_origin().unwrap()
    else {
        panic!("implicit group");
    };
    assert!(opening.source().same_snapshot(original));
    assert!(at.source().same_snapshot(original));
    let errors = source("A url(a b) B");
    assert_eq!(
        crate::component::values::parse::promote_nodes(
            &errors,
            &errors.lists[errors.root],
            crate::component::CssComponentValueLimits::default()
        )
        .unwrap_err()
        .kind(),
        CssComponentValueErrorKind::BadUrl
    );
}

#[test]
fn supplied_adjacent_identifiers_are_not_merged_or_relexed() {
    let tokens = [
        raw(Token::Ident("a".into()), None),
        raw(Token::Ident("b".into()), None),
    ];
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    assert_eq!(document.lists[0].len(), 2);
    assert_eq!(document.nodes[0].token().kind(), TokenKind::Ident("a"));
    assert_eq!(document.nodes[1].token().kind(), TokenKind::Ident("b"));
    assert_eq!(document.metrics.unspelled_tokens, 2);
    assert_eq!(document.metrics.known_spelling_bytes, 0);
    assert!(document.cursor(0).document.boundary(0, 2).source.is_none());
}

#[test]
fn bad_tokens_and_mismatched_closers_remain_in_exact_order() {
    let tokens = [
        raw(Token::Function("f".into()), Some("f(")),
        raw(Token::BadString("broken".into()), None),
        raw(Token::CloseSquareBracket, Some("]")),
        raw(Token::BadUrl("bad".into()), None),
        raw(Token::CloseParenthesis, Some(")")),
        raw(Token::Ident("after".into()), None),
    ];
    let document = normalize(
        SyntaxInput::Tokens(&tokens),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    let group = &document.nodes[document.lists[0][0]];
    let children = group.children().unwrap();
    let causes: Vec<_> = document.lists[children]
        .iter()
        .map(|node| match document.nodes[*node] {
            SyntaxNode::Error { cause, .. } => cause,
            _ => panic!("error token retained"),
        })
        .collect();
    assert_eq!(
        causes,
        [
            SyntaxTokenFault::BadString,
            SyntaxTokenFault::UnexpectedCloser,
            SyntaxTokenFault::BadUrl
        ]
    );
    assert!(matches!(group.end(), Some(GroupEnd::Explicit(_))));
    assert_eq!(document.metrics.components, 5); // group, three errors, sibling; closer counts zero
    assert_eq!(
        document.nodes[*document.lists[0].last().unwrap()]
            .token()
            .kind(),
        TokenKind::Ident("after")
    );
}

#[test]
fn child_eof_repeats_and_reconsume_does_not_touch_parent_or_budget() {
    let document = source("f(a) b");
    let mut parent = document.cursor(0);
    let CursorItem::Node(group) = parent.consume() else {
        panic!()
    };
    let mut child = document.cursor(document.nodes[group].children().unwrap());
    let first = child.consume();
    child.reconsume_current();
    assert_eq!(child.consume(), first);
    let eof = child.consume();
    assert_eq!(child.consume(), eof);
    child.reconsume_current();
    assert_eq!(child.consume(), eof);
    assert!(matches!(eof, CursorItem::EndOfInput(_)));
    assert!(matches!(parent.consume(), CursorItem::Node(_))); // whitespace sibling
    assert_eq!(document.metrics.components, 4);
    assert_eq!(document.metrics.known_spelling_bytes, 6);
}

#[test]
fn at_rule_nested_terminators_and_qualified_semicolons_use_generic_boundaries() {
    let document = source("@x f(;{}) [;] { a:b } ; @y z{} tail{}");
    let rules = consume_rules(&mut document.cursor(0), RuleListSyntax::Stylesheet);
    assert_eq!(rules.len(), 3);
    let GenericRule::At(at) = rules[0].as_ref().unwrap() else {
        panic!()
    };
    assert!(matches!(at.termination, RuleTermination::Block));
    assert!(at.fault.is_none());
    let GenericRule::Qualified(qualified) = rules[1].as_ref().unwrap() else {
        panic!()
    };
    assert_eq!(
        document.nodes[qualified.prelude[0]].token().kind(),
        TokenKind::Semicolon
    );
    assert!(
        qualified
            .prelude
            .iter()
            .any(|node| document.nodes[*node].token().kind() == TokenKind::AtKeyword("y"))
    );
}

#[test]
fn eof_at_rule_is_returned_but_eof_qualified_rule_is_not() {
    let document = source("@x f(a");
    let GenericRule::At(rule) = consume_one_rule(&mut document.cursor(0)).unwrap() else {
        panic!()
    };
    assert!(matches!(rule.termination, RuleTermination::EndOfInput(_)));
    assert_eq!(rule.fault.unwrap().kind, GenericFaultKind::AtRuleEndOfInput);
    let document = source("a ; @x f(a");
    assert_eq!(
        consume_one_rule(&mut document.cursor(0)).unwrap_err().kind,
        GenericFaultKind::QualifiedRuleEndOfInput
    );
}

#[test]
fn cdo_cdc_and_exact_one_obey_entry_flags() {
    let document = source("<!-- --> a{}");
    let rules = consume_rules(&mut document.cursor(0), RuleListSyntax::Stylesheet);
    let GenericRule::Qualified(rule) = rules[0].as_ref().unwrap() else {
        panic!()
    };
    assert_eq!(
        document.nodes[rule.prelude[0]].token().kind(),
        TokenKind::Ident("a")
    );
    let rules = consume_rules(&mut document.cursor(0), RuleListSyntax::BlockContents);
    let GenericRule::Qualified(rule) = rules[0].as_ref().unwrap() else {
        panic!()
    };
    assert_eq!(
        document.nodes[rule.prelude[0]].token().kind(),
        TokenKind::Cdo
    );
    let empty = source(" /*comment*/ ");
    assert_eq!(
        consume_one_rule(&mut empty.cursor(0)).unwrap_err().kind,
        GenericFaultKind::EmptyInput
    );
    let two = source("a{} @x;");
    assert_eq!(
        consume_one_rule(&mut two.cursor(0)).unwrap_err().kind,
        GenericFaultKind::TrailingInput
    );
}

// Syntax 3 (2026 CRD) §5.5.5 consume a block's contents in a rule-only grammar.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
#[test]
fn block_contents_discard_semicolons_and_stop_qualified_rules_at_them() {
    let text = "; a{} ;; b c ; @x y; --p: v {} d{} ; e{} f";
    let document = source(text);
    let offset = |list, index| {
        document
            .boundary(list, index)
            .source
            .expect("source boundary")
            .offset
    };
    let units: Vec<_> = consume_rules(&mut document.cursor(0), RuleListSyntax::BlockContents)
        .into_iter()
        .map(|unit| match unit {
            Ok(GenericRule::At(rule)) => {
                assert!(matches!(rule.termination, RuleTermination::Semicolon(_)));
                (
                    None,
                    offset(rule.range.list, rule.range.start)
                        ..offset(rule.range.list, rule.range.end),
                )
            }
            Ok(GenericRule::Qualified(rule)) => (
                None,
                offset(rule.range.list, rule.range.start)..offset(rule.range.list, rule.range.end),
            ),
            Err(fault) => (
                Some(fault.kind),
                offset(fault.range.list, fault.range.start)
                    ..offset(fault.range.list, fault.range.end),
            ),
        })
        .map(|(kind, range)| (kind, &text[range]))
        .collect();
    assert_eq!(
        units,
        [
            (None, "a{}"),
            (Some(GenericFaultKind::QualifiedRuleStopToken), "b c "),
            (None, "@x y;"),
            (Some(GenericFaultKind::CustomPropertyRule), "--p: v {} d{} "),
            (None, "e{}"),
            (Some(GenericFaultKind::QualifiedRuleEndOfInput), "f"),
        ]
    );
    // The stylesheet root keeps a `;` as qualified-rule prelude input.
    let rules = consume_rules(&mut document.cursor(0), RuleListSyntax::Stylesheet);
    let GenericRule::Qualified(rule) = rules[0].as_ref().unwrap() else {
        panic!("semicolon-prefixed qualified rule")
    };
    assert_eq!(
        document.nodes[rule.prelude[0]].token().kind(),
        TokenKind::Semicolon
    );
}

#[test]
fn grouped_inputs_preserve_structure_and_mixed_snapshot_identity() {
    let first = crate::component::values::parse_component_values("A").unwrap();
    let second = crate::component::values::parse_component_values("B").unwrap();
    let values = [
        first.items()[0].clone(),
        second.items()[0].clone(),
        CssComponentValue::try_ident("C").unwrap(),
    ];
    let document = normalize(
        SyntaxInput::CheckedComponents(&values),
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    for (node, original) in document.lists[0].iter().zip(&values) {
        assert_eq!(
            document.nodes[*node].token().origin.as_ref(),
            original.origin()
        );
    }
    assert!(document.boundary(0, 3).source.is_none());
    let original = source("@x {a(b)}");
    let grouped = normalize(
        SyntaxInput::Components {
            document: &original,
            list: 0,
        },
        SyntaxInputLimits::default(),
        0,
    )
    .unwrap();
    assert_eq!(original.metrics, grouped.metrics);
    let GenericRule::At(rule) = consume_one_rule(&mut grouped.cursor(0)).unwrap() else {
        panic!()
    };
    assert!(rule.block.is_some());
}

#[test]
fn declaration_importance_is_generic_and_case_insensitive() {
    let document = source("unknown: value ! /*c*/ ImPoRtAnT  ");
    let declaration = consume_declaration(&mut document.cursor(0)).unwrap();
    assert!(declaration.important);
    assert_eq!(
        document.nodes[declaration.name].token().kind(),
        TokenKind::Ident("unknown")
    );
    assert!(
        declaration
            .value
            .iter()
            .all(|node| document.nodes[*node].token().kind() != TokenKind::Delim('!'))
    );
    let invalid = source("name value");
    assert_eq!(
        consume_declaration(&mut invalid.cursor(0))
            .unwrap_err()
            .kind,
        GenericFaultKind::InvalidDeclaration
    );
}

#[test]
fn aggregate_limits_charge_errors_repeated_occurrences_and_known_closers_once() {
    let token = raw(Token::BadString("x".into()), Some("\"x\n"));
    let tokens = [token.clone(), token];
    let limits = SyntaxInputLimits {
        max_depth: 256,
        max_components: 2,
        max_known_spelling_bytes: 6,
    };
    let pass = normalize(SyntaxInput::Tokens(&tokens), limits, 0).unwrap();
    assert_eq!(pass.metrics.components, 2);
    assert_eq!(pass.metrics.known_spelling_bytes, 6);
    assert_eq!(
        normalize(
            SyntaxInput::Tokens(&tokens),
            SyntaxInputLimits {
                max_components: 1,
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
    assert_eq!(
        normalize(SyntaxInput::Tokens(&tokens), limits, 0)
            .unwrap()
            .metrics,
        pass.metrics
    );
    let document = source("f(a");
    assert_eq!(document.metrics.known_spelling_bytes, 3); // implicit ')' is not input
}

#[test]
fn structural_depth_exact_boundary_uses_heap_frames() {
    let text = format!("{}x{}", "(".repeat(256), ")".repeat(256));
    let document = source(&text);
    assert_eq!(document.metrics.maximum_depth, 256);
    let too_deep = format!("({text})");
    let snapshot = CssSourceSnapshot::new(&too_deep);
    assert_eq!(
        source_document(&too_deep, &snapshot, 0)
            .err()
            .unwrap()
            .kind(),
        CssComponentValueErrorKind::NestingLimit
    );
}
