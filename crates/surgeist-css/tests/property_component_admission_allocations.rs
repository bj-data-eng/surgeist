#![forbid(unsafe_code)]

//! Standalone public admission allocation contract, without test-harness threads.
//! Supplied comments are already owned input, and neither a CSS-wide declaration
//! nor a root-delimiter error needs an authored value string in its result.
//! Keeping component count fixed while enlarging only that comment must therefore
//! not require a payload-sized grammar transport or diagnostic mapping copy.
//! The allocator's safe instrumentation follows animation's allocation_check.

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::alloc::System;
use std::hint::black_box;
use surgeist_css::{
    CssComponentValues, CssGlobalKeyword, CssImportance, CssKnownProperty, CssPropertyNameRef,
    CssPropertyValueErrorKind, CssSerializedOrigin, CssTokenKind, CssValueOrigin, ErrorKind,
    parse_component_values, parse_property_value,
};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const SMALL_COMMENT_BYTES: usize = 128;
const LARGE_COMMENT_BYTES: usize = 65_536;
// Fixed-size cursor, declaration and error metadata may allocate. This allowance
// concerns growth with comment bytes; it is not a zero-allocation requirement.
const MAX_ADMISSION_BYTE_GROWTH: usize = 1_024;

#[derive(Clone, Copy)]
enum Outcome {
    GlobalInitial,
    RootSemicolon,
}

impl Outcome {
    fn tail(self) -> &'static str {
        match self {
            Self::GlobalInitial => "initial",
            Self::RootSemicolon => ";",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::GlobalInitial => "global-initial",
            Self::RootSemicolon => "root-semicolon",
        }
    }
}

fn fixture(comment_bytes: usize, outcome: Outcome) -> CssComponentValues {
    let source = format!("/*{}*/{}", "x".repeat(comment_bytes), outcome.tail());
    let values = parse_component_values(&source).expect("complete comment and root token");
    assert_eq!(values.items().len(), 2);
    values
}

fn measure(values: &CssComponentValues, outcome: Outcome) -> Stats {
    // Input cloning and expected-origin capture deliberately precede the region.
    let input = values.clone();
    let expected_token_origin = input.items()[1].origin().clone();
    let region = Region::new(GLOBAL);
    let result = black_box(parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        black_box(input),
        CssImportance::Normal,
    ));
    let stats = region.change();

    // Public semantic expectations, diagnostics and destruction are unmeasured.
    match outcome {
        Outcome::GlobalInitial => {
            let declaration = result.expect("the whole CSS-wide initial value is valid");
            let known = declaration.known().expect("known width declaration");
            assert_eq!(known.property(), CssKnownProperty::Width);
            assert_eq!(known.global(), Some(CssGlobalKeyword::Initial));
            assert!(known.property_value().is_none());
            assert_eq!(declaration.importance(), CssImportance::Normal);
            assert_eq!(declaration.value_components(), values);
            assert!(declaration.position().is_none());
            assert!(declaration.parsed_name().is_none());
            assert!(declaration.parsed_value().is_none());
            assert_same_snapshot(
                declaration.value_components().items()[1].origin(),
                &expected_token_origin,
            );
            drop(declaration);
        }
        Outcome::RootSemicolon => {
            let error = result.expect_err("a root semicolon is not declaration-value data");
            let CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedToken(detail)) =
                error.kind()
            else {
                panic!("expected a root-token grammar rejection: {error:?}");
            };
            assert_eq!(detail.encountered().kind(), CssTokenKind::Semicolon);
            assert_eq!(detail.encountered().authored(), ";");
            let CssSerializedOrigin::Token(actual_origin) = error.origin() else {
                panic!("expected the original semicolon token: {error:?}");
            };
            assert_eq!(actual_origin, &expected_token_origin);
            assert_same_snapshot(actual_origin, &expected_token_origin);
            let CssValueOrigin::Parsed(original) = actual_origin else {
                panic!("fixture token has its actual parsed source");
            };
            let start = original.span().start().byte_offset().value();
            let end = original.span().end().byte_offset().value();
            assert_eq!(&original.source().as_str()[start..end], ";");
            assert_eq!(end, original.source().as_str().len());
            drop(error);
        }
    }
    stats
}

fn assert_same_snapshot(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected)
    else {
        panic!("fixture token origins retain parsed snapshots");
    };
    assert!(actual.source().same_snapshot(expected.source()));
}

fn requested_bytes(stats: Stats) -> usize {
    // stats_alloc records net reallocation growth separately from new allocations.
    // Retain those raw fields in the report; this sum is the budgeted metric.
    stats
        .bytes_allocated
        .checked_add(usize::try_from(stats.bytes_reallocated).unwrap_or(0))
        .expect("bounded fixture allocation arithmetic")
}

fn main() {
    let mut failures = Vec::new();
    for outcome in [Outcome::GlobalInitial, Outcome::RootSemicolon] {
        let small = fixture(SMALL_COMMENT_BYTES, outcome);
        let large = fixture(LARGE_COMMENT_BYTES, outcome);
        // Warm both public paths before comparing equally shaped retained inputs.
        let _ = measure(&small, outcome);
        let _ = measure(&large, outcome);
        let small_stats = measure(&small, outcome);
        let large_stats = measure(&large, outcome);
        let growth = requested_bytes(large_stats).saturating_sub(requested_bytes(small_stats));
        println!(
            "{}: comment bytes {} -> {}; small={small_stats:?}; large={large_stats:?}; admission byte growth={growth}; allowance={MAX_ADMISSION_BYTE_GROWTH}",
            outcome.name(),
            SMALL_COMMENT_BYTES,
            LARGE_COMMENT_BYTES,
        );
        if growth > MAX_ADMISSION_BYTE_GROWTH {
            failures.push(format!(
                "{} admission copied payload-sized input: byte growth {growth} exceeds {MAX_ADMISSION_BYTE_GROWTH}",
                outcome.name(),
            ));
        }
        drop(small);
        drop(large);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
