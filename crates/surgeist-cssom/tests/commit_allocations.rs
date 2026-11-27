//! Run this target as its own process. Its single test owns every allocation
//! region; fixture construction, assertions and reporting occur outside regions.
//! Allocation bytes measure allocator requests, not physical memory-copy traffic.
//!
//! The checks of a commit keep figures with the store's state, so a commit that
//! changes one sheet flag allocates for that change alone: its allocation at a
//! large store equals its allocation at a small one, up to a fixed slack.

#![forbid(unsafe_code)]

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::{alloc::System, fmt::Write, hint::black_box};
use surgeist_cssom::*;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// The rule counts compared.
const SMALL: usize = 64;
const LARGE: usize = 2048;
/// Allocator requests the large store may make beyond the small one's.
const SLACK_BYTES: usize = 1024;
const SLACK_ALLOCATIONS: usize = 8;

fn measured<T>(operation: impl FnOnce() -> T) -> (T, Stats) {
    let region = Region::new(GLOBAL);
    let value = black_box(operation());
    let stats = region.change();
    (value, stats)
}

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

fn fixture(rule_count: usize) -> (CssomStore, CssomSheetId) {
    let mut source = String::new();
    for index in 0..rule_count {
        writeln!(source, ".rule{index} {{ width: 1px; }}").unwrap();
    }
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store.batch(guard(&store)).unwrap();
    let ticket = batch
        .create_parsed_sheet(
            &source,
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "commit-allocation-fixture".into(),
                    revision: 1,
                },
                "commit-allocation-fixture".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    (store, sheet)
}

/// The allocation of opening a batch, disabling the sheet and committing it.
fn disabling(rule_count: usize) -> Stats {
    let (mut store, sheet) = fixture(rule_count);
    let before = store.snapshot();
    let batch_guard = guard(&store);
    let ((changed, commit), stats) = measured(|| {
        let mut batch = store.batch(batch_guard).unwrap();
        let changed = batch.set_disabled(&sheet, true).unwrap();
        (changed, store.commit(batch).unwrap())
    });
    assert!(changed);
    assert!(matches!(commit.publication(), CssomPublication::Changed(_)));
    assert!(store.snapshot().sheet(&sheet).unwrap().inputs().disabled);
    assert!(!before.sheet(&sheet).unwrap().inputs().disabled);
    stats
}

// This target deliberately has one test, so the default parallel test harness
// cannot run another test body while its process-global allocator is sampled.
#[test]
fn committing_one_sheet_flag_allocates_the_same_at_every_store_size() {
    let small = disabling(SMALL);
    let large = disabling(LARGE);
    for (name, stats) in [("small", &small), ("large", &large)] {
        println!(
            "{name} allocations={} allocated_bytes={} reallocations={} reallocated_bytes={}",
            stats.allocations, stats.bytes_allocated, stats.reallocations, stats.bytes_reallocated,
        );
    }
    assert!(
        large.bytes_allocated <= small.bytes_allocated + SLACK_BYTES,
        "committing one sheet flag allocated {} bytes at {SMALL} rules and {} bytes at {LARGE}",
        small.bytes_allocated,
        large.bytes_allocated,
    );
    assert!(
        large.allocations <= small.allocations + SLACK_ALLOCATIONS,
        "committing one sheet flag made {} allocations at {SMALL} rules and {} at {LARGE}",
        small.allocations,
        large.allocations,
    );
}
