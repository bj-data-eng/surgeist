//! Additive owning-module functional evidence for the adopted private API.
//! Install as a child module of syntax_consumption, alongside its existing tests.
use super::*;
use crate::component::CssComponentValues;
use crate::parser::property_input::SiteExt;
use surgeist_css_parser::document::Fault;

fn independent_source(text: &str) -> Document<'_> {
    source_document(text, &CssSourceSnapshot::new(text), 0).unwrap()
}

fn independent_price(document: &Document<'_>, components: usize, depth: u32, bytes: usize) {
    let metrics = document.metrics();
    assert_eq!(
        (
            metrics.components(),
            metrics.maximum_depth(),
            metrics.spelling_bytes()
        ),
        (components, depth, bytes)
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

fn independent_site(boundary: &SyntaxBoundary, offset: usize) {
    assert_eq!(
        boundary.offset,
        Some(offset),
        "actual supplied source boundary"
    );
}

#[test]
fn independent_checked_numeric_payload_survives_direct_input() {
    let checked = crate::component::values::parse_component_values("+01").unwrap();
    let checked_document =
        checked_document(checked.items(), SyntaxInputLimits::default(), 0).unwrap();
    independent_price(&checked_document, 1, 0, 3);
    let [checked_number] = checked_document.list(checked_document.root_list()) else {
        panic!("one checked lexical integer")
    };
    let Some(Attached::Component(component)) = checked_document.payload(*checked_number) else {
        panic!("actual checked number carrier")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        panic!("lexical numeric payload")
    };
    assert_eq!(number.representation(), "+01");
    assert_eq!(
        number.kind(),
        crate::component::CssNumericTokenKind::Integer
    );
    assert!(number.has_sign());
    let Token::Number(literal) = checked_document.token(*checked_number) else {
        panic!("the tree's token is the checked number")
    };
    assert_eq!(literal.representation(), "+01");
    assert_eq!(literal.integer(), Some(1));
    assert_eq!(
        checked_document.origin(*checked_number),
        *checked.items()[0].origin()
    );
    // The component's spelling is its lexeme; the tree holds no text.
    assert!(checked_document.spelling(*checked_number).is_none());
    assert_eq!(
        checked_document
            .site(checked_document.root_list(), 0)
            .and_then(SiteExt::lexeme),
        Some("+01")
    );
}

#[test]
fn independent_mixed_equal_text_snapshots_have_no_invented_source() {
    let first = crate::component::values::parse_component_values("é").unwrap();
    let second = crate::component::values::parse_component_values("é").unwrap();
    let before_first = first.clone();
    let before_second = second.clone();
    let CssValueOrigin::Parsed(first_origin) = first.items()[0].origin() else {
        panic!("first source")
    };
    let CssValueOrigin::Parsed(second_origin) = second.items()[0].origin() else {
        panic!("second source")
    };
    assert!(!first_origin.source().same_snapshot(second_origin.source()));
    let values = [
        first.items()[0].clone(),
        second.items()[0].clone(),
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_block(
            CssBlockKind::CurlyBracket,
            CssComponentValues::try_new(Vec::new(), Default::default()).unwrap(),
        )
        .unwrap(),
    ];
    let limits = SyntaxInputLimits {
        max_depth: 1,
        max_components: 4,
        max_known_spelling_bytes: 7,
    };
    let document = checked_document(&values, limits, 0).unwrap();
    independent_price(&document, 4, 1, 7);
    let rule = consume_qualified_rule(&mut document.cursor(document.root_list())).unwrap();
    let [a, b, number] = rule.prelude.as_slice() else {
        panic!("three direct prelude items, not merged text")
    };
    assert_eq!(document.kind(*a), TokenKind::Ident("é"));
    assert_eq!(document.kind(*b), TokenKind::Ident("é"));
    independent_origin(&document.origin(*a), first_origin.source(), 0, 2);
    independent_origin(&document.origin(*b), second_origin.source(), 0, 2);
    assert!(
        matches!(document.token(*number), Token::Number(literal) if literal.integer() == Some(1))
    );
    assert_eq!(document.origin(*number), CssValueOrigin::Programmatic);
    let child = document.children(rule.block).unwrap();
    assert!(
        matches!(document.cursor(child).peek(), CursorItem::EndOfInput(boundary) if boundary.offset.is_none())
    );
    assert!(
        document
            .boundary(
                document.root_list(),
                document.list(document.root_list()).len()
            )
            .offset
            .is_none()
    );
    assert_eq!(
        checked_document(
            &values,
            SyntaxInputLimits {
                max_known_spelling_bytes: 6,
                ..limits
            },
            0
        )
        .err()
        .unwrap()
        .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    let retry = checked_document(&values, limits, 0).unwrap();
    independent_price(&retry, 4, 1, 7);
    assert_eq!(first, before_first);
    assert_eq!(second, before_second);
}

#[test]
fn independent_bounded_source_and_child_eof_reconsume_keep_their_actual_end_sites() {
    let text = "/*😀*/f(a)tail";
    let snapshot = CssSourceSnapshot::new(text);
    let document = build_source(
        &text[8..12],
        8,
        &snapshot,
        SyntaxInputLimits {
            max_depth: 1,
            max_components: 2,
            max_known_spelling_bytes: 4,
        },
        0,
    )
    .unwrap();
    independent_price(&document, 2, 1, 4);
    let mut parent = document.cursor(document.root_list());
    let first = parent.peek();
    assert_eq!(parent.peek(), first);
    let CursorItem::Node(group) = parent.consume() else {
        panic!("actual bounded function")
    };
    independent_origin(&document.origin(group), &snapshot, 8, 10);
    let mut child = document.cursor(document.children(group).unwrap());
    let CursorItem::Node(argument) = child.consume() else {
        panic!("argument a")
    };
    assert_eq!(document.kind(argument), TokenKind::Ident("a"));
    independent_origin(&document.origin(argument), &snapshot, 10, 11);
    let child_eof = child.consume();
    let CursorItem::EndOfInput(boundary) = &child_eof else {
        panic!("bounded child EOF")
    };
    independent_site(boundary, 11);
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
    independent_site(boundary, 12);
    parent.reconsume_current();
    assert_eq!(
        parent.consume(),
        parent_eof,
        "reconsuming EOF cannot resurrect the function"
    );
    independent_price(&document, 2, 1, 4);
    assert_eq!(snapshot.as_str(), text);
}

#[test]
fn independent_child_list_eof_and_at_rule_fault_belong_to_their_own_child() {
    let original = independent_source("outer{f(@x");
    let snapshot = original.snapshot().unwrap().clone();
    let [_, outer] = original.list(original.root_list()) else {
        panic!("outer ident and curly group")
    };
    let outer_children = original.children(*outer).unwrap();
    independent_price(&original, 4, 2, 10);
    let [function] = original.list(outer_children) else {
        panic!("the function, no outer wrapper")
    };
    let children = original.children(*function).unwrap();
    assert_eq!(
        original
            .group_end(*function)
            .map(|end| matches!(end, GroupEnd::Eof { at: Some(at) } if at.value() == 10)),
        Some(true)
    );
    independent_origin(&original.origin(*function), &snapshot, 6, 8);
    let at = original.boundary(children, 1);
    assert_eq!(at.list, children, "boundary belongs to the child list");
    assert_eq!(at.index, 1);
    independent_site(&at, 10);
    let mut child = original.cursor(children);
    let rule = consume_at_rule(&mut child);
    assert_eq!(original.kind(rule.name), TokenKind::AtKeyword("x"));
    independent_origin(&original.origin(rule.name), &snapshot, 8, 10);
    let RuleTermination::EndOfInput(end) = &rule.termination else {
        panic!("child at-rule EOF")
    };
    assert_eq!(end, &at);
    let fault = rule.fault.unwrap();
    assert_eq!(fault.kind, GenericFaultKind::AtRuleEndOfInput);
    assert_eq!(&fault.at, &at);
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
}

#[test]
fn independent_repeated_checked_groups_charge_each_delimiter_once_and_retry_atomically() {
    let parsed = crate::component::values::parse_component_values("{é}").unwrap();
    let values = [parsed.items()[0].clone(), parsed.items()[0].clone()];
    let before = values.clone();
    let limits = SyntaxInputLimits {
        max_depth: 1,
        max_components: 4,
        max_known_spelling_bytes: 8,
    };
    let document = checked_document(&values, limits, 0).unwrap();
    independent_price(&document, 4, 1, 8); // 2 × (group+ident), 2 × (1+2+1 bytes)
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
        let failure = checked_document(&values, limit, 0).expect_err("no usable partial document");
        assert_eq!(failure.kind(), kind);
        assert_eq!(failure.origin(), origin);
        assert_eq!(values, before);
        let retry = checked_document(&values, limits, 0).unwrap();
        independent_price(&retry, 4, 1, 8);
        for node in retry.list(retry.root_list()) {
            assert_eq!(retry.origin(*node), *parsed.items()[0].origin());
            assert!(matches!(
                retry.group_end(*node),
                Some(GroupEnd::Closed { .. })
            ));
            assert_eq!(
                retry.closing_origin(*node),
                *original_block.closing_origin()
            );
            assert_eq!(
                retry.closing_site(*node).and_then(SiteExt::lexeme),
                Some("}")
            );
            let child = retry.children(*node).unwrap();
            let [ident] = retry.list(child) else {
                panic!("one repeated child")
            };
            assert_eq!(retry.kind(*ident), TokenKind::Ident("é"));
            assert_eq!(
                retry.origin(*ident),
                *original_block.values().items()[0].origin()
            );
        }
    }
}

#[test]
fn independent_base_depth_ceiling_is_distinct_from_the_nesting_limit() {
    let snapshot = CssSourceSnapshot::new("f()");
    let limits = SyntaxInputLimits {
        max_depth: 256,
        max_components: 1,
        max_known_spelling_bytes: 3,
    };
    // The tree counts depth from its own root: one level below the base.
    let at_ceiling = build_source("f()", 0, &snapshot, limits, 255).unwrap();
    independent_price(&at_ceiling, 1, 1, 3);
    assert_eq!(
        build_source("f()", 0, &snapshot, limits, 256)
            .err()
            .unwrap()
            .kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    assert_eq!(
        build_source(
            "f()",
            0,
            &snapshot,
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
        &build_source("f()", 0, &snapshot, limits, 255).unwrap(),
        1,
        1,
        3,
    );
    // A checked build counts the base depth in the depth it reports.
    let parsed = crate::component::values::parse_component_values("f()").unwrap();
    independent_price(
        &checked_document(parsed.items(), limits, 255).unwrap(),
        1,
        256,
        3,
    );
    assert_eq!(
        checked_document(parsed.items(), limits, 256)
            .err()
            .unwrap()
            .kind(),
        CssComponentValueErrorKind::NestingLimit
    );
}

#[test]
fn independent_exact_one_rejects_without_consuming_second_unit_but_charges_all_input() {
    let text = "@x;/*c*/@y;";
    let snapshot = CssSourceSnapshot::new(text);
    let limits = SyntaxInputLimits {
        max_depth: 0,
        max_components: 5,
        max_known_spelling_bytes: 11,
    };
    let document = build_source(text, 0, &snapshot, limits, 0).unwrap();
    independent_price(&document, 5, 0, 11);
    let mut cursor = document.cursor(document.root_list());
    let fault = consume_one_rule(&mut cursor).unwrap_err();
    assert_eq!(fault.kind, GenericFaultKind::TrailingInput);
    independent_site(&fault.at, 8);
    let CursorItem::Node(second) = cursor.peek() else {
        panic!("unconsumed second at-rule name")
    };
    assert_eq!(document.kind(second), TokenKind::AtKeyword("y"));
    assert_eq!(cursor.consume(), CursorItem::Node(second));
    let CursorItem::Node(semicolon) = cursor.peek() else {
        panic!("second unit was not consumed to reject it")
    };
    assert_eq!(document.kind(semicolon), TokenKind::Semicolon);
    assert_eq!(
        build_source(
            text,
            0,
            &snapshot,
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
        build_source(
            text,
            0,
            &snapshot,
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
        &build_source(text, 0, &snapshot, limits, 0).unwrap(),
        5,
        0,
        11,
    );
    let terminated = independent_source("@x;/*c*/");
    independent_price(&terminated, 3, 0, 8);
    let GenericRule::At(rule) =
        consume_one_rule(&mut terminated.cursor(terminated.root_list())).unwrap()
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
    independent_price(&document, 6, 2, 8);
    let snapshot = document.snapshot().unwrap().clone();
    let [paren, y] = document.list(document.root_list()) else {
        panic!("parenthesis group and later root sibling")
    };
    assert_eq!(document.kind(*y), TokenKind::Ident("y"));
    independent_origin(&document.origin(*y), &snapshot, 7, 8);
    let parent_children = document.children(*paren).unwrap();
    let [square, bad_curly, x] = document.list(parent_children) else {
        panic!("nested square, wrong closer, x")
    };
    let square_children = document.children(*square).unwrap();
    let [bad_paren] = document.list(square_children) else {
        panic!("mismatched paren stays inside active square")
    };
    for (node, kind, spelling, start) in [
        (*bad_paren, CssBlockKind::Parenthesis, ")", 2),
        (*bad_curly, CssBlockKind::CurlyBracket, "}", 4),
    ] {
        assert_eq!(
            document.component(node).fault(),
            Some(Fault::UnexpectedCloser)
        );
        assert_eq!(document.kind(node), TokenKind::Closing(kind));
        assert_eq!(document.spelling(node), Some(spelling));
        independent_origin(&document.origin(node), &snapshot, start, start + 1);
    }
    assert_eq!(document.kind(*x), TokenKind::Ident("x"));
    independent_site(&document.boundary(square_children, 1), 3);
    independent_site(&document.boundary(parent_children, 3), 6);
    assert!(matches!(
        document.group_end(*square),
        Some(GroupEnd::Closed { .. })
    ));
    independent_origin(&document.closing_origin(*square), &snapshot, 3, 4);
    assert!(matches!(
        document.group_end(*paren),
        Some(GroupEnd::Closed { .. })
    ));
    independent_origin(&document.closing_origin(*paren), &snapshot, 6, 7);
}
