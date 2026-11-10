#![forbid(unsafe_code)]
//! Raw component boundaries through the normalized syntax arena.
//! Syntax 3 CRD20261001 §§5.4.9, 5.5.7–10 define the independent raw oracles.
//! Exact/comma tests call the real adopted production selectors, never a test splitter.
use super::*;
use surgeist_css_parser::document::Fault;

fn source(text: &str) -> (CssSourceSnapshot, Document<'_>) {
    let snapshot = CssSourceSnapshot::new(text);
    let document = source_document(text, &snapshot, 0).unwrap();
    (snapshot, document)
}

fn next(cursor: &mut SyntaxCursor<'_>, expected: TokenKind<'_>) -> NodeId {
    let CursorItem::Node(node) = cursor.consume() else {
        panic!("literal component before bounded EOF")
    };
    assert_eq!(cursor.document.kind(node), expected);
    node
}

fn site(boundary: &SyntaxBoundary, offset: usize) {
    assert_eq!(
        boundary.offset,
        Some(offset),
        "actual original bounded source endpoint"
    );
}

fn end(cursor: &mut SyntaxCursor<'_>) -> SyntaxBoundary {
    let CursorItem::EndOfInput(boundary) = cursor.consume() else {
        panic!("conceptual local EOF")
    };
    assert_eq!(cursor.consume(), CursorItem::EndOfInput(boundary.clone()));
    assert_eq!(cursor.consume(), CursorItem::EndOfInput(boundary.clone()));
    boundary
}

fn occurrence(
    document: &Document<'_>,
    node: NodeId,
    snapshot: &CssSourceSnapshot,
    spelling: &str,
    range: Range<usize>,
) {
    assert_eq!(document.spelling(node), Some(spelling));
    let CssValueOrigin::Parsed(origin) = document.origin(node) else {
        panic!("original token occurrence")
    };
    assert!(origin.source().same_snapshot(snapshot));
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

fn extent(document: &Document<'_>, node: NodeId) -> Option<Range<usize>> {
    document
        .range(node)
        .map(|range| range.start.value()..range.end.value())
}

fn eof_offset(document: &Document<'_>, group: NodeId) -> Option<usize> {
    match document.group_end(group) {
        Some(GroupEnd::Eof { at }) => at.map(Offset::value),
        _ => panic!("EOF returns each still-open object with its implicit-end fault"),
    }
}

#[test]
fn all_component_cursor_preserves_trivia_errors_and_later_tokens_around_complete_nested_groups() {
    let text = " f([)] x) url(a b) \"bad\nz ;]";
    let (snapshot, document) = source(text);
    let before = document.metrics();
    let mut root = document.cursor(document.root_list());
    let leading = next(&mut root, TokenKind::Whitespace);
    occurrence(&document, leading, &snapshot, " ", 0..1);
    let function = next(&mut root, TokenKind::Function("f"));
    occurrence(&document, function, &snapshot, "f(", 1..3);
    assert_eq!(extent(&document, function), Some(1..9));
    let parent_position = root.position();
    let mut arguments = document.cursor(document.children(function).unwrap());
    let square = next(
        &mut arguments,
        TokenKind::Opening(CssBlockKind::SquareBracket),
    );
    occurrence(&document, square, &snapshot, "[", 3..4);
    assert_eq!(extent(&document, square), Some(3..6));
    let mut square_contents = document.cursor(document.children(square).unwrap());
    let mismatch = next(
        &mut square_contents,
        TokenKind::Closing(CssBlockKind::Parenthesis),
    );
    assert_eq!(
        document.component(mismatch).fault(),
        Some(Fault::UnexpectedCloser)
    );
    occurrence(&document, mismatch, &snapshot, ")", 4..5);
    site(&end(&mut square_contents), 5);
    assert!(matches!(
        document.group_end(square),
        Some(GroupEnd::Closed { .. })
    ));
    assert_eq!(
        document.closing_site(square).and_then(Site::token),
        Some(Token::CloseSquare)
    );
    next(&mut arguments, TokenKind::Whitespace);
    let x = next(&mut arguments, TokenKind::Ident("x"));
    occurrence(&document, x, &snapshot, "x", 7..8);
    site(&end(&mut arguments), 8);
    assert!(matches!(
        document.group_end(function),
        Some(GroupEnd::Closed { .. })
    ));
    assert_eq!(
        document.closing_site(function).and_then(Site::token),
        Some(Token::CloseParen)
    );
    assert_eq!(
        root.position(),
        parent_position,
        "child consumption cannot advance its parent"
    );

    next(&mut root, TokenKind::Whitespace);
    let bad_url = next(&mut root, TokenKind::BadUrl);
    assert_eq!(document.component(bad_url).fault(), Some(Fault::BadUrl));
    occurrence(&document, bad_url, &snapshot, "url(a b)", 10..18);
    next(&mut root, TokenKind::Whitespace);
    let bad_string = next(&mut root, TokenKind::BadString);
    assert_eq!(
        document.component(bad_string).fault(),
        Some(Fault::BadString)
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
    assert_eq!(
        document.component(stray).fault(),
        Some(Fault::UnexpectedCloser)
    );
    occurrence(&document, stray, &snapshot, "]", 27..28);
    site(&end(&mut root), 28);
    assert_eq!(
        document.metrics(),
        before,
        "cursor traversal does not readmit or recharge components"
    );
}

#[test]
fn mismatched_closer_stays_inside_its_group_while_all_open_ancestors_close_at_actual_eof() {
    let text = " f({a[)b";
    let (snapshot, document) = source(text);
    let mut root = document.cursor(document.root_list());
    next(&mut root, TokenKind::Whitespace);
    let function = next(&mut root, TokenKind::Function("f"));
    let mut function_contents = document.cursor(document.children(function).unwrap());
    let curly = next(
        &mut function_contents,
        TokenKind::Opening(CssBlockKind::CurlyBracket),
    );
    let mut curly_contents = document.cursor(document.children(curly).unwrap());
    next(&mut curly_contents, TokenKind::Ident("a"));
    let square = next(
        &mut curly_contents,
        TokenKind::Opening(CssBlockKind::SquareBracket),
    );
    let mut square_contents = document.cursor(document.children(square).unwrap());
    let mismatch = next(
        &mut square_contents,
        TokenKind::Closing(CssBlockKind::Parenthesis),
    );
    assert_eq!(
        document.component(mismatch).fault(),
        Some(Fault::UnexpectedCloser)
    );
    occurrence(&document, mismatch, &snapshot, ")", 6..7);
    let b = next(&mut square_contents, TokenKind::Ident("b"));
    occurrence(&document, b, &snapshot, "b", 7..8);
    for (group, expected_range, opening_range) in [
        (function, 1..8, 1..3),
        (curly, 3..8, 3..4),
        (square, 5..8, 5..6),
    ] {
        assert_eq!(extent(&document, group), Some(expected_range));
        assert_eq!(eof_offset(&document, group), Some(8));
        let CssValueOrigin::Parsed(opening) = document.origin(group) else {
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
        assert_eq!(
            document.list_end(document.children(group).unwrap()),
            Some(Offset::new(8))
        );
    }
    site(&end(&mut square_contents), 8);
    site(&end(&mut curly_contents), 8);
    site(&end(&mut function_contents), 8);
    site(&end(&mut root), 8);
}

#[test]
fn bounded_source_window_uses_its_own_eof_instead_of_a_matching_closer_outside_the_window() {
    let text = "prefix f(a] b) outside";
    let snapshot = CssSourceSnapshot::new(text);
    let document =
        build_source(&text[7..13], 7, &snapshot, SyntaxInputLimits::default(), 0).unwrap();
    let mut root = document.cursor(document.root_list());
    let function = next(&mut root, TokenKind::Function("f"));
    assert_eq!(extent(&document, function), Some(7..13));
    let mut child = document.cursor(document.children(function).unwrap());
    next(&mut child, TokenKind::Ident("a"));
    let stray = next(&mut child, TokenKind::Closing(CssBlockKind::SquareBracket));
    occurrence(&document, stray, &snapshot, "]", 10..11);
    assert_eq!(
        document.component(stray).fault(),
        Some(Fault::UnexpectedCloser)
    );
    next(&mut child, TokenKind::Whitespace);
    next(&mut child, TokenKind::Ident("b"));
    site(&end(&mut child), 13);
    site(&end(&mut root), 13);
    assert_eq!(eof_offset(&document, function), Some(13));
    assert_eq!(&snapshot.as_str()[13..], ") outside");
}

#[test]
fn checked_implicit_groups_keep_original_component_occurrences_and_child_eof_without_fabricating_root_source()
 {
    let text = " f([x";
    let values = crate::component::values::parse_component_values(text).unwrap();
    let CssComponentValueRef::Function(original_function) = values.items()[1].view() else {
        panic!("actual checked function")
    };
    let original_square = &original_function.values().items()[0];
    let document = checked_document(values.items(), SyntaxInputLimits::default(), 0).unwrap();
    let mut root = document.cursor(document.root_list());
    next(&mut root, TokenKind::Whitespace);
    let function = next(&mut root, TokenKind::Function("f"));
    let Some(Attached::Component(component)) = document.payload(function) else {
        panic!("actual supplied function occurrence")
    };
    assert!(std::ptr::eq(*component, &values.items()[1]));
    let CssValueOrigin::Parsed(opening) = component.origin() else {
        panic!("original checked source")
    };
    let snapshot = opening.source();
    let mut child = document.cursor(document.children(function).unwrap());
    let square = next(&mut child, TokenKind::Opening(CssBlockKind::SquareBracket));
    let Some(Attached::Component(component)) = document.payload(square) else {
        panic!("actual supplied block occurrence")
    };
    assert!(std::ptr::eq(*component, original_square));
    let mut square_contents = document.cursor(document.children(square).unwrap());
    let x = next(&mut square_contents, TokenKind::Ident("x"));
    let CssValueOrigin::Parsed(x_origin) = document.origin(x) else {
        panic!("original checked source")
    };
    assert!(x_origin.source().same_snapshot(snapshot));
    assert_eq!(x_origin.span().start().byte_offset().value(), 4);
    assert_eq!(x_origin.span().end().byte_offset().value(), 5);
    for group in [square, function] {
        assert_eq!(eof_offset(&document, group), Some(5));
    }
    site(&end(&mut square_contents), 5);
    site(&end(&mut child), 5);
    assert!(
        end(&mut root).offset.is_none(),
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
        let metrics = document.metrics();
        let mut cursor = document.cursor(document.root_list());
        let node = consume_one_component(&mut cursor).unwrap();
        assert_eq!(document.kind(node), expected);
        if matches!(expected, TokenKind::Closing(_)) {
            assert_eq!(
                document.component(node).fault(),
                Some(Fault::UnexpectedCloser)
            );
            occurrence(&document, node, &snapshot, "]", 6..7);
        } else {
            assert_eq!(eof_offset(&document, node), Some(text.len()));
        }
        site(&end(&mut cursor), text.len());
        assert_eq!(document.metrics(), metrics);
    }
    let (_, document) = source(" /**/ ");
    let fault = consume_one_component(&mut document.cursor(document.root_list())).unwrap_err();
    assert_eq!(fault.kind, GenericFaultKind::EmptyInput);
    site(&fault.at, 6);
    assert_eq!(fault.range.list, document.root_list());
    assert_eq!(fault.range.start, fault.range.end);
    let (_, document) = source(" a /**/ f(x) ");
    let fault = consume_one_component(&mut document.cursor(document.root_list())).unwrap_err();
    assert_eq!(fault.kind, GenericFaultKind::TrailingInput);
    site(&fault.at, 8);
    assert_eq!(
        document.kind(document.list(fault.at.list)[fault.at.index]),
        TokenKind::Function("f")
    );
}

#[test]
fn comma_ranges_preserve_all_nested_kinds_trivia_and_final_empty_members() {
    let (_, document) = source("a,f(x,y),[x,y],(x,y),{x,y}, /*c*/ ,,");
    let metrics = document.metrics();
    let ranges = consume_comma_separated_components(&mut document.cursor(document.root_list()));
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
        assert_eq!(range.list, document.root_list());
    }
    for (range, kind) in ranges[1..5].iter().zip([
        TokenKind::Function("f"),
        TokenKind::Opening(CssBlockKind::SquareBracket),
        TokenKind::Opening(CssBlockKind::Parenthesis),
        TokenKind::Opening(CssBlockKind::CurlyBracket),
    ]) {
        let node = document.list(range.list)[range.start];
        assert_eq!(document.kind(node), kind);
        assert_eq!(document.list(document.children(node).unwrap()).len(), 3);
    }
    let trivia = &document.list(document.root_list())[10..13];
    assert_eq!(
        trivia.iter().map(|n| document.kind(*n)).collect::<Vec<_>>(),
        [
            TokenKind::Whitespace,
            TokenKind::Comment,
            TokenKind::Whitespace
        ]
    );
    site(&end(&mut document.cursor_range(&ranges[7])), 36);
    assert_eq!(document.metrics(), metrics);
    let (_, empty) = source("");
    assert_eq!(
        consume_comma_separated_components(&mut empty.cursor(empty.root_list())),
        [SyntaxRange {
            list: empty.root_list(),
            start: 0,
            end: 0
        }]
    );
}

#[test]
fn raw_comma_selector_uses_actual_checked_kinds_and_keeps_mixed_origins() {
    let first = crate::component::values::parse_component_values("A").unwrap();
    let comma = crate::component::values::parse_component_values(",").unwrap();
    let last = crate::component::values::parse_component_values("B").unwrap();
    let programmatic = |spelling| CssComponentValue::try_token(spelling).unwrap();
    let values = [
        first.items()[0].clone(),
        programmatic(","),
        CssComponentValue::try_string("payload,]").unwrap(),
        comma.items()[0].clone(),
        CssComponentValue::try_ident("literal,ident").unwrap(),
        programmatic(","),
        last.items()[0].clone(),
        programmatic(","),
    ];
    let document = checked_document(&values, SyntaxInputLimits::default(), 0).unwrap();
    let ranges = consume_comma_separated_components(&mut document.cursor(document.root_list()));
    assert_eq!(
        ranges.iter().map(|r| (r.start, r.end)).collect::<Vec<_>>(),
        [(0, 1), (2, 3), (4, 5), (6, 7), (8, 8)]
    );
    let ids = document.list(document.root_list());
    assert_eq!(
        consume_one_component(&mut document.cursor_range(&ranges[1])).unwrap(),
        ids[2],
        "one raw string remains a component, without checked promotion"
    );
    assert_eq!(document.token(ids[2]), Token::String("payload,]"));
    assert_eq!(document.kind(ids[4]), TokenKind::Ident("literal,ident"));
    for (index, original) in [(0, &values[0]), (6, &values[6])] {
        let Some(Attached::Component(component)) = document.payload(ids[index]) else {
            panic!("borrowed original checked occurrence")
        };
        assert!(std::ptr::eq(*component, original));
        assert_eq!(document.origin(ids[index]), *original.origin());
    }
    let CssValueOrigin::Parsed(a) = first.items()[0].origin() else {
        panic!("A source")
    };
    let CssValueOrigin::Parsed(b) = last.items()[0].origin() else {
        panic!("B source")
    };
    assert!(!a.source().same_snapshot(b.source()));
    assert!(document.source().is_none());
    assert!(end(&mut document.cursor_range(&ranges[4])).offset.is_none());
}

#[test]
fn selectors_respect_child_and_candidate_eof_without_consuming_parent_or_excluded_comma() {
    let (snapshot, document) = source("f(a,,)tail");
    let mut parent = document.cursor(document.root_list());
    let function = next(&mut parent, TokenKind::Function("f"));
    let parent_position = parent.position();
    let child = document.children(function).unwrap();
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
    assert_eq!(node, document.list(child)[0]);
    occurrence(&document, node, &snapshot, "a", 2..3);
    site(&end(&mut document.cursor_range(&ranges[2])), 5);
    let empty = consume_one_component(&mut document.cursor_range(&ranges[2])).unwrap_err();
    assert_eq!(empty.kind, GenericFaultKind::EmptyInput);
    site(&empty.at, 5);
    let mut candidate = document.cursor_range(&ranges[0]);
    consume_one_component(&mut candidate).unwrap();
    site(&end(&mut candidate), 3);
    assert_eq!(parent.position(), parent_position);
    let tail = next(&mut parent, TokenKind::Ident("tail"));
    occurrence(&document, tail, &snapshot, "tail", 6..10);
}
