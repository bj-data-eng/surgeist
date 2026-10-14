//! An isolated child keeps global allocation counters separate from other tests.

#![forbid(unsafe_code)]

use super::NumericInputContext;
use crate::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssParsedOrigin,
    CssSourceSnapshot, CssValueOrigin, CssValueTokenRef, parse_component_values,
};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn first_number(root: &CssComponentValue) -> &CssComponentValue {
    let CssComponentValueRef::Function(rgb) = root.view() else {
        panic!("RGB root");
    };
    let CssComponentValueRef::Function(calc) = rgb.values().items()[0].view() else {
        panic!("nested calc operand");
    };
    &calc.values().items()[0]
}

fn parsed_origin(component: &CssComponentValue) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("original parsed occurrence");
    };
    origin
}

fn component_css(component: &CssComponentValue) -> String {
    CssComponentValues::try_new(vec![component.clone()])
        .unwrap()
        .serialize()
        .unwrap()
        .as_css()
        .to_owned()
}

#[test]
fn mapped_collection_lends_root_without_transport_allocation() {
    const TEST: &str = "numeric::borrowed_admission_tests::mapped_collection_lends_root_without_transport_allocation";
    const CHILD: &str = "SURGEIST_CSS_COMPONENT_BORROW_ALLOCATION_CHILD";
    if std::env::var(CHILD).as_deref() != Ok(TEST) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, TEST)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated allocation witness: {}\n{}\n{}",
            output.status,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        );
        return;
    }

    const CHILDREN: &str = "calc(1 + max(2, min(3,4))) 5 6";
    const ROOT: &str = "rgb(calc(1 + max(2, min(3,4))) 5 6)";
    const SOURCE: &str = "rgb(calc(1 + max(2, min(3,4))) 5 6) blue";

    // The source route actually collects an owned graph. Its allocations are
    // legitimate source admission and are outside the mapped-transport region.
    let source = CssSourceSnapshot::new(SOURCE);
    let mut source_input = cssparser::ParserInput::new(SOURCE);
    let mut source_cursor = cssparser::Parser::new(&mut source_input);
    let source_root = NumericInputContext::Parsed(&source)
        .collect(&mut source_cursor)
        .unwrap();
    assert_eq!(source_cursor.expect_ident().unwrap().as_ref(), "blue");
    assert!(source_cursor.is_exhausted());
    let source_leaf = first_number(&source_root);
    let origin = parsed_origin(source_leaf);
    assert!(origin.source().same_snapshot(&source));
    assert_eq!(origin.span().start().byte_offset().value(), 9);
    assert_eq!(origin.span().end().byte_offset().value(), 10);

    // A programmatic root with actual parsed children exercises mixed origins.
    // All fixture construction, validation, serialization and parser setup stay
    // outside the measured call. Ordinary ASCII names need no escape decoding.
    let children = parse_component_values(CHILDREN).unwrap();
    let root = CssComponentValue::try_function("rgb", children).unwrap();
    let expected_leaf_origin = parsed_origin(first_number(&root)).clone();
    let values =
        CssComponentValues::try_new(vec![root, CssComponentValue::try_ident("blue").unwrap()])
            .unwrap();
    let serialized = values.serialize().unwrap();
    let context = NumericInputContext::Components(&values, &serialized);
    let mut input = cssparser::ParserInput::new(serialized.as_css());
    let mut cursor = cssparser::Parser::new(&mut input);

    let region = Region::new(GLOBAL);
    let collected = context.collect(&mut cursor).unwrap();
    let allocations = region.change();

    // This coercion works for the existing owned component and for a borrowed
    // ownership carrier. The witness never requires a future production API.
    let observed: &CssComponentValue = &collected;
    assert_eq!(observed.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(observed, &values.items()[0]);
    let leaf = first_number(observed);
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = leaf.view() else {
        panic!("exact number token");
    };
    assert_eq!(number.representation(), "1");
    let origin = parsed_origin(leaf);
    assert!(origin.source().same_snapshot(expected_leaf_origin.source()));
    assert_eq!(origin.source().as_str(), CHILDREN);
    assert_eq!(origin.span(), expected_leaf_origin.span());
    assert_eq!(origin.span().start().byte_offset().value(), 5);
    assert_eq!(origin.span().end().byte_offset().value(), 6);
    assert_eq!(cursor.expect_ident().unwrap().as_ref(), "blue");
    assert!(cursor.is_exhausted());
    assert_eq!(component_css(&source_root), ROOT);
    assert_eq!(component_css(observed), ROOT);

    let same_root = std::ptr::eq(observed, &values.items()[0]);
    eprintln!(
        "mapped nested component collection: allocations={}, reallocations={}, bytes_allocated={}, bytes_reallocated={}, original_root={same_root}; source cursor, mapped cursor, exact spelling and mixed snapshot origins passed",
        allocations.allocations,
        allocations.reallocations,
        allocations.bytes_allocated,
        allocations.bytes_reallocated,
    );
    assert_eq!(
        (
            allocations.allocations,
            allocations.reallocations,
            allocations.bytes_allocated,
            allocations.bytes_reallocated,
        ),
        (0, 0, 0, 0),
        "mapped collection must lend its checked subtree without allocating a transport copy",
    );
    assert!(
        same_root,
        "mapped collection must inspect the original node"
    );
}
