//! Run this target as its own process. Its single test owns every allocation
//! region; fixture construction, assertions and reporting occur outside regions.
//! Allocation bytes measure allocator requests, not physical memory-copy traffic.

#![forbid(unsafe_code)]

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::{alloc::System, fmt::Write, hint::black_box};
use surgeist_cssom::*;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

// One sheet and no independently supplied global inputs remain fixed while its
// unrelated authored rules, live rules, blocks and lists grow. Batch bookkeeping
// permits two small owning tokens, but cannot justify copying that retained tree.
const EMPTY_OR_NOOP_BYTE_LIMIT: usize = 4096;
// The changed sheet index has one entry. Its record, summary and fixed-size input
// manifests have generous space here; its unrelated authored tree does not.
const ONE_FIELD_BYTE_LIMIT: usize = 16 * 1024;

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
                    identity: "allocation-fixture".into(),
                    revision: 1,
                },
                "allocation-fixture".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    assert_eq!(
        store.snapshot().sheet_rules(&sheet).unwrap().len(),
        rule_count
    );
    (store, sheet)
}

struct Row {
    rules: usize,
    empty: Stats,
    disabled_noop: Stats,
    rule_noop: Stats,
    one_field: Stats,
}

// This target deliberately has one test, so the default parallel test harness
// cannot run another test body while its process-global allocator is sampled.
#[test]
fn staging_allocations_ignore_unrelated_rule_payloads() {
    let mut rows = Vec::with_capacity(3);
    for rule_count in [1, 128, 1024] {
        let (mut store, sheet) = fixture(rule_count);
        let before = store.snapshot();
        let first_rule = before.sheet_rules(&sheet).unwrap()[0].clone();

        // Retain both an old snapshot and a competing staged edit: sharing must
        // work when the live owner cannot have unique ownership of its base.
        let mut competing = store.batch(guard(&store)).unwrap();
        assert!(competing.set_disabled(&sheet, true).unwrap());
        assert!(!store.snapshot().sheet(&sheet).unwrap().inputs().disabled);

        let empty_guard = guard(&store);
        let (empty_batch, empty) = measured(|| store.batch(empty_guard).unwrap());
        drop(empty_batch);

        let noop_guard = guard(&store);
        let ((changed, noop_commit), disabled_noop) = measured(|| {
            let mut batch = store.batch(noop_guard).unwrap();
            let changed = batch.set_disabled(&sheet, false).unwrap();
            (changed, store.commit(batch).unwrap())
        });
        assert!(!changed);
        assert!(matches!(
            noop_commit.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(store.snapshot().revision(), before.revision());
        drop(noop_commit);

        let rule_guard = guard(&store);
        let (rule_commit, rule_noop) = measured(|| {
            let mut batch = store.batch(rule_guard).unwrap();
            batch
                .set_rule_css_text(&first_rule, "not a parsed replacement")
                .unwrap();
            store.commit(batch).unwrap()
        });
        assert!(matches!(
            rule_commit.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(store.snapshot().revision(), before.revision());
        drop(rule_commit);

        let changed_guard = guard(&store);
        let ((changed, commit), one_field) = measured(|| {
            let mut batch = store.batch(changed_guard).unwrap();
            let changed = batch.set_disabled(&sheet, true).unwrap();
            (changed, store.commit(batch).unwrap())
        });
        assert!(changed);
        let after = store.snapshot();
        let CssomPublication::Changed(summary) = commit.publication() else {
            panic!("the sheet flag must publish exactly one coherent change")
        };
        assert_eq!(after.revision().value(), before.revision().value() + 1);
        assert_eq!(summary.from(), before.revision());
        assert_eq!(summary.to(), after.revision());
        assert!(summary.categories().contains(&CssomChange::Disabled));
        assert!(after.sheet(&sheet).unwrap().inputs().disabled);
        assert!(!before.sheet(&sheet).unwrap().inputs().disabled);
        assert_eq!(
            before.sheet_rules(&sheet).unwrap(),
            after.sheet_rules(&sheet).unwrap()
        );
        assert!(matches!(
            store.commit(competing),
            Err(CssomError::StaleRevision)
        ));
        assert_eq!(store.snapshot().revision(), after.revision());
        drop(store);
        assert!(!before.sheet(&sheet).unwrap().inputs().disabled);
        assert!(after.sheet(&sheet).unwrap().inputs().disabled);
        assert_eq!(before.sheet_rules(&sheet).unwrap().len(), rule_count);
        assert_eq!(after.sheet_rules(&sheet).unwrap().len(), rule_count);

        rows.push(Row {
            rules: rule_count,
            empty,
            disabled_noop,
            rule_noop,
            one_field,
        });
    }

    let mut failures = Vec::new();
    for row in rows {
        for (boundary, stats, maximum) in [
            ("empty_batch_open", row.empty, EMPTY_OR_NOOP_BYTE_LIMIT),
            (
                "checked_disabled_noop_with_commit",
                row.disabled_noop,
                EMPTY_OR_NOOP_BYTE_LIMIT,
            ),
            (
                "checked_rule_noop_with_commit",
                row.rule_noop,
                EMPTY_OR_NOOP_BYTE_LIMIT,
            ),
            (
                "one_sheet_flag_with_commit",
                row.one_field,
                ONE_FIELD_BYTE_LIMIT,
            ),
        ] {
            println!(
                "rules={} boundary={boundary} allocations={} allocated_bytes={} reallocations={} reallocated_bytes={} deallocations={} deallocated_bytes={} limit={maximum}",
                row.rules,
                stats.allocations,
                stats.bytes_allocated,
                stats.reallocations,
                stats.bytes_reallocated,
                stats.deallocations,
                stats.bytes_deallocated,
            );
            if stats.bytes_allocated > maximum {
                failures.push(format!(
                    "{boundary} with {} unrelated rules allocated {} bytes; maximum {maximum}",
                    row.rules, stats.bytes_allocated,
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
