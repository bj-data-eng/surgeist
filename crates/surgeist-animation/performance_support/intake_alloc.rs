//! Allocation contracts for the additive text intake APIs.

use std::hint::black_box;
use std::sync::Arc;

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats};
use surgeist_animation::{CompositeValue, DiscreteValue, PropertyKey, TransformValue};

pub fn report(filter: &str, iterations: usize) {
    println!(
        "case,iterations,allocations,reallocations,deallocations,bytes_allocated,bytes_reallocated"
    );
    let mut matched = 0;
    let shared: Arc<str> = Arc::from(" property ");
    let blank: Arc<str> = Arc::from("\u{2003}\u{a0}");
    macro_rules! check {
        ($name:literal, report, $operation:expr) => {
            measure(filter, $name, iterations, None, $operation);
        };
        ($name:literal, $expected:expr, $operation:expr) => {
            matched += measure(filter, $name, iterations, Some($expected), $operation);
        };
    }
    check!("intake/property/legacy", report, || PropertyKey::new(
        black_box(" property ")
    )
    .unwrap());
    check!("intake/property/text", 1, || PropertyKey::from_text(
        black_box(" property ")
    )
    .unwrap());
    check!("intake/property/shared", 0, || PropertyKey::from_shared(
        black_box(shared.clone())
    )
    .unwrap());
    check!("intake/discrete/legacy", report, || DiscreteValue::new(
        black_box(" property ")
    )
    .unwrap());
    check!("intake/discrete/text", 1, || DiscreteValue::from_text(
        black_box(" property ")
    )
    .unwrap());
    check!("intake/discrete/shared", 0, || DiscreteValue::from_shared(
        black_box(shared.clone())
    )
    .unwrap());
    check!("intake/transform/legacy", report, || {
        TransformValue::unsupported(black_box(" property "))
    });
    check!("intake/transform/text", 1, || {
        TransformValue::unsupported_from_text(black_box(" property "))
    });
    check!("intake/transform/shared", 0, || {
        TransformValue::unsupported_from_shared(black_box(shared.clone()))
    });
    check!("intake/composite/legacy", report, || {
        CompositeValue::unsupported(black_box(" property "))
    });
    check!("intake/composite/text", 1, || {
        CompositeValue::unsupported_from_text(black_box(" property "))
    });
    check!("intake/composite/shared", 0, || {
        CompositeValue::unsupported_from_shared(black_box(shared.clone()))
    });
    check!("intake/property/blank_text", 0, || PropertyKey::from_text(
        black_box("\u{2003}\u{a0}")
    ));
    check!("intake/property/blank_shared", 0, || {
        PropertyKey::from_shared(black_box(blank.clone()))
    });
    check!(
        "intake/discrete/blank_text",
        0,
        || DiscreteValue::from_text(black_box("\u{2003}\u{a0}"))
    );
    check!("intake/discrete/blank_shared", 0, || {
        DiscreteValue::from_shared(black_box(blank.clone()))
    });
    check!("intake/transform/blank_text", report, || {
        TransformValue::unsupported_from_text(black_box("\u{2003}\u{a0}"))
    });
    check!("intake/transform/blank_shared", report, || {
        TransformValue::unsupported_from_shared(black_box(blank.clone()))
    });
    check!("intake/composite/blank_text", report, || {
        CompositeValue::unsupported_from_text(black_box("\u{2003}\u{a0}"))
    });
    check!("intake/composite/blank_shared", report, || {
        CompositeValue::unsupported_from_shared(black_box(blank.clone()))
    });
    assert!(matched > 0, "filter matched no intake contracts");
}

fn measure<T>(
    filter: &str,
    name: &str,
    iterations: usize,
    allocations: Option<usize>,
    operation: impl Fn() -> T,
) -> usize {
    if !name.contains(filter) {
        return 0;
    }
    let mut total = Stats::default();
    for _ in 0..iterations {
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        let value = operation();
        let stats = region.change();
        black_box(&value);
        drop(value);
        total.allocations += stats.allocations;
        total.reallocations += stats.reallocations;
        total.deallocations += stats.deallocations;
        total.bytes_allocated += stats.bytes_allocated;
        total.bytes_reallocated += stats.bytes_reallocated;
    }
    println!(
        "{name},{iterations},{},{},{},{},{}",
        total.allocations,
        total.reallocations,
        total.deallocations,
        total.bytes_allocated,
        total.bytes_reallocated
    );
    if let Some(allocations) = allocations {
        assert_eq!(
            total.allocations,
            allocations * iterations,
            "{name}: constructor allocation contract"
        );
        assert_eq!(
            total.reallocations, 0,
            "{name}: constructor reallocation contract"
        );
    }
    1
}
