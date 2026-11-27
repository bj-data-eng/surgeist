//! Run this target as its own process. Its single test owns every allocation
//! region; fixture construction, assertions and reporting occur outside regions.
//! Allocation bytes measure allocator requests, not physical memory-copy traffic.
//!
//! Creating a sheet adopts its rules one after another, and each adopted rule
//! appends to the rule list of the sheet. The state is checked after every rule,
//! so a list that the check kept a second handle on would be copied for each rule
//! appended and creation would allocate in proportion to the square of the rules.
//! Both tests below hold the allocation of creating N rules in proportion to N, and
//! the allocation of further appends to a large list to that of further appends to a
//! small one: only the first touch of the list in a batch copies it.

#![forbid(unsafe_code)]

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::{alloc::System, fmt::Write, hint::black_box, sync::atomic::AtomicBool};
use surgeist_cssom::*;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn measured<T>(operation: impl FnOnce() -> T) -> (T, Stats) {
    let region = Region::new(GLOBAL);
    let value = black_box(operation());
    let stats = region.change();
    (value, stats)
}

/// The bytes the allocator was asked for, new and grown.
fn requested(stats: &Stats) -> i64 {
    i64::try_from(stats.bytes_allocated).unwrap() + i64::try_from(stats.bytes_reallocated).unwrap()
}

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "creation-allocations".into(),
            revision: 1,
        },
        true,
    )
}

fn text(rules: usize) -> String {
    let mut source = String::new();
    for index in 0..rules {
        writeln!(source, ".r{index} {{ margin: 0 }}").unwrap();
    }
    source
}

/// The allocation of creating a sheet of `rules` rules in one batch.
fn creation(rules: usize) -> Stats {
    let source = text(rules);
    let store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store.batch(guard(&store)).unwrap();
    let (ticket, stats) = measured(|| batch.create_parsed_sheet(&source, inputs()).unwrap());
    drop(ticket);
    stats
}

/// The allocation of `appends` appends to the rule list of a sheet of `rules` rules, in one batch.
fn appends(rules: usize, appends: usize) -> Stats {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store.batch(guard(&store)).unwrap();
    let ticket = batch.create_parsed_sheet(&text(rules), inputs()).unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    let mut batch = store.batch(guard(&store)).unwrap();
    let cancel = AtomicBool::new(false);
    let (_, stats) = measured(|| {
        for index in 0..appends {
            batch
                .insert_sheet_rule(&sheet, ".added { margin: 1px }", rules + index, &cancel)
                .unwrap();
        }
    });
    stats
}

// This target deliberately has one test, so the default parallel test harness
// cannot run another test body while its process-global allocator is sampled.
#[test]
fn creating_a_sheet_and_appending_to_its_list_allocate_in_proportion_to_the_rules_added() {
    let sizes = [1000, 2000, 4000];
    let created = sizes.map(|rules| requested(&creation(rules)));
    for (rules, bytes) in sizes.iter().zip(created) {
        println!("creation rules={rules} requested_bytes={bytes}");
    }
    // Doubling the rules at most a little more than doubles the allocation; a copy of
    // the rule list for each rule appended makes it four times as large.
    for pair in created.windows(2) {
        assert!(
            pair[1] * 5 <= pair[0] * 12,
            "creating 2N rules requested {} bytes, creating N rules {} ({created:?} at {sizes:?})",
            pair[1],
            pair[0]
        );
    }

    // K appends cost what the first one costs plus a constant for each further one, however
    // long the list is: only the first touch of the list in the batch copies it.
    let (small, large) = (100, 2000);
    let extra = |rules: usize| requested(&appends(rules, 9)) - requested(&appends(rules, 1));
    let (small_extra, large_extra) = (extra(small), extra(large));
    println!(
        "appends: 8 further appends request {small_extra} bytes at {small} rules, {large_extra} at {large}"
    );
    assert!(
        large_extra <= small_extra + 8192,
        "8 further appends requested {large_extra} bytes at {large} rules and {small_extra} at {small}"
    );
}
