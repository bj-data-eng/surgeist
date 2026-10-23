//! Allocation evidence at the real component collector, excluding its source owner.

use super::*;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region};

fn retained_whitespace_bytes(nested: bool, length: usize) -> usize {
    let whitespace = " ".repeat(length);
    let text = if nested {
        format!("f({whitespace})")
    } else {
        whitespace.clone()
    };
    let snapshot = CssSourceSnapshot::new(&text);
    let mut input = ParserInput::new(&text);
    let mut parser = Parser::new(&mut input);

    // The crate test allocator is installed by numeric::borrowed_admission_tests.
    // This process runs only this case; fixture/snapshot construction is excluded.
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let values = collect(&mut parser, &snapshot, CssComponentValueLimits::default()).unwrap();
    let allocations = region.change();

    assert!(parser.is_exhausted());
    assert_eq!(
        values
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        text
    );
    let leaves = if nested {
        let CssComponentValueRef::Function(function) = values.items()[0].view() else {
            panic!("function containing whitespace");
        };
        assert_eq!(function.name(), "f");
        function.values().items()
    } else {
        values.items()
    };
    assert_eq!(leaves.len(), 1);
    let leaf = &leaves[0];
    let CssComponentValueRef::Token(CssValueTokenRef::Whitespace(token)) = leaf.view() else {
        panic!("one retained whitespace token");
    };
    assert_eq!(token, whitespace);
    let CssValueOrigin::Parsed(origin) = leaf.origin() else {
        panic!("original parsed leaf");
    };
    assert!(origin.source().same_snapshot(&snapshot));
    let start = if nested { 2 } else { 0 };
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), start + length);
    allocations.bytes_allocated + allocations.bytes_reallocated.max(0) as usize
}

#[test]
fn flat_and_nested_leaves_allocate_only_the_retained_spelling() {
    const TEST: &str = "component_values::parse::spelling_allocation_tests::flat_and_nested_leaves_allocate_only_the_retained_spelling";
    const CHILD: &str = "SURGEIST_COMPONENT_SPELLING_ALLOCATION_CHILD";
    const COMPLETED: &str = "spelling allocation bound checked";
    crate::test_support::run_in_isolated_child(TEST, CHILD, COMPLETED, check_spelling_growth);
}

fn check_spelling_growth() {
    let small = 128;
    let large = 65_536;
    // Only one retained leaf spelling grows; component counts and group shape
    // stay fixed. Permit a fixed metadata margin, never another payload copy.
    let bound = large - small + 1_024;
    let growth = [false, true].map(|nested| {
        let small_bytes = retained_whitespace_bytes(nested, small);
        let large_bytes = retained_whitespace_bytes(nested, large);
        let growth = large_bytes.saturating_sub(small_bytes);
        eprintln!(
            "nested={nested}: requested bytes {small_bytes} -> {large_bytes}; growth={growth}, retained spelling bound={bound}; exact text and source origins passed"
        );
        growth
    });
    assert!(
        growth.iter().all(|actual| *actual <= bound),
        "flat and nested leaves must allocate their retained spelling once: {growth:?} > {bound}"
    );
}
