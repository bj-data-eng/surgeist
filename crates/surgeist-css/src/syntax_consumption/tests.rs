use super::*;
use surgeist_css_parser::document::Fault;

fn source(text: &str) -> Document<'_> {
    source_document(text, &CssSourceSnapshot::new(text), 0).unwrap()
}

#[test]
fn candidate_cursor_eof_is_the_excluded_semicolon_without_recharging_input() {
    let document = source("name: f(a;b) ! IMPORTANT ; next: yes");
    let metrics = document.metrics();
    let mut root = document.cursor(document.root_list());
    let range = consume_declaration_candidate(&mut root);
    let semicolon = root.peek();
    let declaration = consume_declaration(&mut document.cursor_range(&range)).unwrap();
    assert!(declaration.important);
    assert_eq!(declaration.value.len(), 1);
    assert!(matches!(
        document.component(declaration.value[0]),
        Component::Function { .. }
    ));
    let mut candidate = document.cursor_range(&range);
    while matches!(candidate.consume(), CursorItem::Node(_)) {}
    let CursorItem::EndOfInput(at) = candidate.peek() else {
        panic!("candidate EOF");
    };
    assert_eq!(at.offset, Some("name: f(a;b) ! IMPORTANT ".len()));
    assert!(matches!(candidate.consume(), CursorItem::EndOfInput(_)));
    assert_eq!(root.peek(), semicolon);
    assert_eq!(document.metrics(), metrics);
}

#[test]
fn admitted_node_promotion_preserves_provider_payloads_and_implicit_group_owner() {
    let document = source("-0 #id #123 f(/*c*/A");
    let values = crate::component::values::parse::promote_nodes(
        &document,
        document.list(document.root_list()),
        crate::component::CssComponentValueLimits::default(),
    )
    .unwrap();
    let expected =
        crate::component::values::parse_component_values("-0 #id #123 f(/*c*/A").unwrap();
    assert_eq!(values, expected);
    let original = document.snapshot().unwrap();
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
            errors.list(errors.root_list()),
            crate::component::CssComponentValueLimits::default()
        )
        .unwrap_err()
        .kind(),
        CssComponentValueErrorKind::BadUrl
    );
}

#[test]
fn bad_tokens_and_mismatched_closers_remain_in_exact_order() {
    let document = source("f(\"b\n]url(a b))after");
    let [group, after] = document.list(document.root_list()) else {
        panic!("the group and the identifier after it")
    };
    let children = document.children(*group).unwrap();
    let faults: Vec<_> = document
        .list(children)
        .iter()
        .map(|node| document.component(*node).fault())
        .collect();
    assert_eq!(
        faults,
        [
            Some(Fault::BadString),
            None, // the newline the bad string left to be reconsumed
            Some(Fault::UnexpectedCloser),
            Some(Fault::BadUrl)
        ]
    );
    assert!(matches!(
        document.group_end(*group),
        Some(GroupEnd::Closed { .. })
    ));
    // group, bad string, newline, closer, bad url, sibling; the closer of the
    // group counts zero
    assert_eq!(document.metrics().components(), 6);
    assert_eq!(document.kind(*after), TokenKind::Ident("after"));
}

#[test]
fn child_eof_repeats_and_does_not_touch_parent_or_budget() {
    let document = source("f(a) b");
    let mut parent = document.cursor(document.root_list());
    let CursorItem::Node(group) = parent.consume() else {
        panic!()
    };
    let mut child = document.cursor(document.children(group).unwrap());
    assert!(matches!(child.consume(), CursorItem::Node(_)));
    let eof = child.consume();
    assert_eq!(child.consume(), eof);
    assert_eq!(child.consume(), eof);
    assert!(matches!(eof, CursorItem::EndOfInput(_)));
    assert!(matches!(parent.consume(), CursorItem::Node(_))); // whitespace sibling
    assert_eq!(document.metrics().components(), 4);
    assert_eq!(document.metrics().spelling_bytes(), 6);
}

#[test]
fn eof_at_rule_is_returned_but_eof_qualified_rule_is_not() {
    let document = source("@x f(a");
    let GenericRule::At(rule) =
        consume_one_rule(&mut document.cursor(document.root_list())).unwrap()
    else {
        panic!()
    };
    assert!(matches!(rule.termination, RuleTermination::EndOfInput));
    assert_eq!(rule.fault.unwrap().kind, GenericFaultKind::AtRuleEndOfInput);
    let document = source("a ; @x f(a");
    assert_eq!(
        consume_one_rule(&mut document.cursor(document.root_list()))
            .unwrap_err()
            .kind,
        GenericFaultKind::QualifiedRuleEndOfInput
    );
}

#[test]
fn exact_one_reports_empty_and_trailing_input() {
    let empty = source(" /*comment*/ ");
    assert_eq!(
        consume_one_rule(&mut empty.cursor(empty.root_list()))
            .unwrap_err()
            .kind,
        GenericFaultKind::EmptyInput
    );
    let two = source("a{} @x;");
    assert_eq!(
        consume_one_rule(&mut two.cursor(two.root_list()))
            .unwrap_err()
            .kind,
        GenericFaultKind::TrailingInput
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
    let document = checked_document(&values, SyntaxInputLimits::default(), 0).unwrap();
    for (node, original) in document.list(document.root_list()).iter().zip(&values) {
        assert_eq!(document.origin(*node), *original.origin());
    }
    assert!(document.boundary(document.root_list(), 3).offset.is_none());
    let original = source("@x {a(b)}");
    let GenericRule::At(rule) =
        consume_one_rule(&mut original.cursor(original.root_list())).unwrap()
    else {
        panic!()
    };
    assert!(rule.block.is_some());
}

#[test]
fn declaration_importance_is_generic_and_case_insensitive() {
    let document = source("unknown: value ! /*c*/ ImPoRtAnT  ");
    let declaration = consume_declaration(&mut document.cursor(document.root_list())).unwrap();
    assert!(declaration.important);
    assert_eq!(document.kind(declaration.name), TokenKind::Ident("unknown"));
    assert!(
        declaration
            .value
            .iter()
            .all(|node| document.kind(*node) != TokenKind::Delim('!'))
    );
    let invalid = source("name value");
    assert_eq!(
        consume_declaration(&mut invalid.cursor(invalid.root_list()))
            .unwrap_err()
            .kind,
        GenericFaultKind::InvalidDeclaration
    );
}

#[test]
fn aggregate_limits_charge_errors_repeated_occurrences_and_known_closers_once() {
    // Two bad strings, each followed by the newline it left to be reconsumed.
    let text = "\"x\n\"x\n";
    let snapshot = CssSourceSnapshot::new(text);
    let limits = SyntaxInputLimits {
        max_depth: 256,
        max_components: 4,
        max_known_spelling_bytes: 6,
    };
    let pass = build_source(text, 0, &snapshot, limits, 0).unwrap();
    assert_eq!(pass.metrics().components(), 4);
    assert_eq!(pass.metrics().spelling_bytes(), 6);
    assert_eq!(
        build_source(
            text,
            0,
            &snapshot,
            SyntaxInputLimits {
                max_components: 3,
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
        build_source(
            text,
            0,
            &snapshot,
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
        build_source(text, 0, &snapshot, limits, 0)
            .unwrap()
            .metrics(),
        pass.metrics()
    );
    let document = source("f(a");
    assert_eq!(document.metrics().spelling_bytes(), 3); // implicit ')' is not input
}

#[test]
fn structural_depth_exact_boundary_uses_heap_frames() {
    let text = format!("{}x{}", "(".repeat(256), ")".repeat(256));
    let document = source(&text);
    assert_eq!(document.metrics().maximum_depth(), 256);
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
