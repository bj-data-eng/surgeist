//! Requested and actual collector-bank admission precede use and retain old custody.
#![forbid(unsafe_code)]

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use std::{alloc::System, cell::Cell};
use surgeist_css::{
    CssSourceRetention, CssSourceRetentionAction, CssSourceRetentionError, parse_sheet,
};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[test]
fn refused_requested_or_actual_replacement_disposes_only_new_backing_before_return() {
    for stop in [1, 2] {
        let first = parse_sheet("a {}")
            .syntax()
            .parse_source()
            .unwrap()
            .source()
            .clone();
        let second = parse_sheet("b {}")
            .syntax()
            .parse_source()
            .unwrap()
            .source()
            .clone();
        let region = Region::new(GLOBAL);
        let watching = Cell::new(false);
        let scratch = Cell::new(0);
        let at_refusal_allocations = Cell::new(0);
        let at_refusal_deallocations = Cell::new(0);
        let mut admit = |action| {
            if watching.get() && matches!(action, CssSourceRetentionAction::Scratch { .. }) {
                scratch.set(scratch.get() + 1);
                if scratch.get() == stop {
                    let stats = region.change();
                    at_refusal_allocations.set(stats.allocations);
                    at_refusal_deallocations.set(stats.deallocations);
                    return false;
                }
            }
            true
        };
        let mut collector = CssSourceRetention::with_admission(4096, 10, 10, &mut admit).unwrap();
        collector.source(&first).unwrap();
        let original = collector.usage();
        let before = region.change();
        watching.set(true);
        let result = collector.source(&second);
        let after = region.change();
        assert_eq!(result, Err(CssSourceRetentionError::caller_admission()));
        assert_eq!(collector.usage(), original);
        assert_eq!(at_refusal_deallocations.get(), before.deallocations);
        assert_eq!(at_refusal_allocations.get() - before.allocations, stop - 1);
        assert_eq!(after.allocations - before.allocations, stop - 1);
        assert_eq!(after.deallocations - before.deallocations, stop - 1);
        assert_eq!(after.reallocations, before.reallocations);
        // The original bank remains until its real collector owner disposes it.
        drop(collector);
        assert_eq!(region.change().deallocations - after.deallocations, 1);
    }
}
