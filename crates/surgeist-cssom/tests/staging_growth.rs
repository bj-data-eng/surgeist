//! Isolated allocator process for the remaining table-index/touched-payload costs.
#![forbid(unsafe_code)]
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::{alloc::System, fmt::Write, hint::black_box};
use surgeist_css::*;
use surgeist_cssom::*;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn measure<T>(operation: impl FnOnce() -> T) -> (T, Stats) {
    let region = Region::new(GLOBAL);
    let result = black_box(operation());
    (result, region.change())
}
fn report(axis: &str, size: usize, boundary: &str, stats: Stats) {
    println!(
        "axis={axis} size={size} boundary={boundary} allocations={} allocated_bytes={} reallocations={} reallocated_bytes={}",
        stats.allocations, stats.bytes_allocated, stats.reallocations, stats.bytes_reallocated
    );
}
fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "growth".into(),
            revision: 1,
        },
        "growth".into(),
        None,
    )
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, source: &str) -> CssomSheetId {
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet(source, inputs()).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn block(snapshot: &CssomSnapshot, sheet: &CssomSheetId, index: usize) -> CssomBlockId {
    let rule = &snapshot.sheet_rules(sheet).unwrap()[index];
    snapshot.rule(rule).unwrap().data().block().unwrap().clone()
}

// One case per executable keeps the ordinary test harness from running another
// test body inside these global allocation regions.
#[test]
fn index_and_touched_payload_costs_are_distinct_from_unrelated_payloads() {
    for count in [1, 16, 128] {
        let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let mut setup = batch(&store);
        let mut tickets = Vec::with_capacity(count);
        for _ in 0..count {
            tickets.push(
                setup
                    .create_parsed_sheet("a { width: 1px; }", inputs())
                    .unwrap(),
            );
        }
        let committed = store.commit(setup).unwrap();
        let sheet = committed.sheet(&tickets[0]).unwrap().clone();
        let before = store.snapshot();
        let (mut edit, opened) = measure(|| batch(&store));
        let (changed, first) = measure(|| edit.set_disabled(&sheet, true).unwrap());
        let (restored, repeated) = measure(|| edit.set_disabled(&sheet, false).unwrap());
        let commit = store.commit(edit).unwrap();
        assert!(changed && restored);
        assert!(matches!(
            commit.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert!(opened.bytes_allocated <= 4096);
        assert!(repeated.bytes_allocated <= 4096);
        report("sheet_index", count, "open", opened);
        // This first mutation legitimately copies the whole touched sheet index.
        report("sheet_index", count, "first_flag_stage", first);
        report("sheet_index", count, "restore_exclusive_record", repeated);
    }

    for declarations in [1, 64, 256] {
        let mut source = String::from(".target { width: 1px; } .unrelated {");
        for index in 0..declarations {
            write!(source, "--p{index}: {}px;", index + 1).unwrap();
        }
        source.push('}');
        let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let sheet = create(&mut store, &source);
        let before = store.snapshot();
        let target = block(&before, &sheet, 0);
        let untouched = block(&before, &sheet, 1);
        let mut edit = batch(&store);
        let (removed, stats) = measure(|| {
            edit.remove_selected_terminal(
                &target,
                CssPropertyNameRef::Known(CssKnownProperty::Width),
            )
            .unwrap()
        });
        assert!(removed);
        let commit = store.commit(edit).unwrap();
        assert!(matches!(commit.publication(), CssomPublication::Changed(_)));
        let after = store.snapshot();
        let CssomBlockData::Properties {
            selected: CssomProjection::Available(selected),
            ..
        } = after.block(&untouched).unwrap().data()
        else {
            panic!("property block")
        };
        assert_eq!(selected.entries().len(), declarations);
        assert!(stats.bytes_allocated <= 32 * 1024);
        report(
            "unrelated_block_declarations",
            declarations,
            "remove_target_width_stage",
            stats,
        );
    }

    for declarations in [1, 64, 256] {
        let mut source = String::from(".target {");
        for index in 0..declarations {
            write!(source, "--p{index}: {}px;", index + 1).unwrap();
        }
        source.push('}');
        let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let sheet = create(&mut store, &source);
        let before = store.snapshot();
        let target = block(&before, &sheet, 0);
        let name = CssCustomPropertyName::try_from_decoded("--p0").unwrap();
        let mut edit = batch(&store);
        let (removed, stats) = measure(|| {
            edit.remove_selected_terminal(&target, CssPropertyNameRef::Custom(&name))
                .unwrap()
        });
        assert!(removed);
        store.commit(edit).unwrap();
        let after = store.snapshot();
        let CssomBlockData::Properties {
            selected: CssomProjection::Available(selected),
            ..
        } = after.block(&target).unwrap().data()
        else {
            panic!("property block")
        };
        assert_eq!(selected.entries().len(), declarations - 1);
        // The changed public owned enum still copies its own authored/selected
        // Vec payload. Report this real remaining cost without a constant claim.
        report(
            "touched_block_declarations",
            declarations,
            "remove_target_custom_stage",
            stats,
        );
    }

    for declarations in [1, 64, 256] {
        let mut source = String::from(
            "@font-feature-values Demo { @styleset { a: 1; } } @media screen { a {} } .target { width: 1px; } .unrelated {",
        );
        for index in 0..declarations {
            write!(source, "--p{index}: {}px;", index + 1).unwrap();
        }
        source.push('}');
        let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let sheet = create(&mut store, &source);
        let before = store.snapshot();
        let roots = before.sheet_rules(&sheet).unwrap();
        let feature = before.inspect_font_feature_values(&roots[0]).unwrap();
        let present = feature.map(CssFontFeatureValueKind::Styleset).clone();
        let empty = feature.map(CssFontFeatureValueKind::Swash).clone();
        let target = block(&before, &sheet, 2);
        let (results, stats) = measure(|| {
            let mut edit = batch(&store);
            let results = (
                edit.set_feature(&present, "a", &[1]).unwrap(),
                edit.delete_feature(&present, "absent").unwrap(),
                edit.clear_feature_map(&empty).unwrap(),
                edit.remove_selected_terminal(
                    &target,
                    CssPropertyNameRef::Known(CssKnownProperty::Color),
                )
                .unwrap(),
            );
            (results, store.commit(edit).unwrap())
        });
        assert_eq!(results.0, (false, false, false, false));
        assert!(matches!(
            results.1.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert!(stats.bytes_allocated <= 4096);
        report(
            "all_six_tables_unrelated_declarations",
            declarations,
            "checked_map_and_property_noops",
            stats,
        );
    }

    for inputs_count in [1, 64, 256] {
        let context = CssomContext {
            inputs: (0..inputs_count)
                .map(|index| CssomInput {
                    version: CssomInputVersion {
                        role: CssomInputRole::Base,
                        identity: format!("base{index}"),
                        revision: 1,
                    },
                    data: CssomInputData::Base {
                        url: Some("x".repeat(512)),
                        location: None,
                    },
                })
                .collect(),
            linked: Vec::new(),
        };
        let mut store = CssomStore::new(Default::default(), context).unwrap();
        let sheet = create(&mut store, "a {}");
        let before = store.snapshot();
        let (commit, stats) = measure(|| {
            let mut edit = batch(&store);
            edit.set_disabled(&sheet, false).unwrap();
            store.commit(edit).unwrap()
        });
        assert!(matches!(
            commit.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert!(stats.bytes_allocated <= 4096);
        report(
            "supplied_context_facts",
            inputs_count,
            "guard_open_checked_noop_commit",
            stats,
        );
    }

    for declarations in [1, 64, 256] {
        let mut source = String::from(".unrelated {");
        for index in 0..declarations {
            write!(source, "--p{index}: {}px;", index + 1).unwrap();
        }
        source.push('}');
        let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
        create(&mut store, &source);
        let mut setup = batch(&store);
        let ticket = setup
            .create_declarations(
                CssDeclarationList::try_new(Vec::new(), Default::default()).unwrap(),
                false,
                false,
                Some(CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "element".into(),
                    revision: 1,
                }),
            )
            .unwrap();
        let target = store.commit(setup).unwrap().block(&ticket).unwrap().clone();
        let mut initial = batch(&store);
        let preparation = initial
            .prepare_css_text(&target, "", Default::default())
            .unwrap();
        initial.apply_declaration(preparation, &[]).unwrap();
        drop(store.commit(initial).unwrap());
        let before = store.snapshot();
        let (commit, stats) = measure(|| {
            let mut edit = batch(&store);
            let preparation = edit
                .prepare_css_text(&target, "", Default::default())
                .unwrap();
            edit.apply_declaration(preparation, &[]).unwrap();
            store.commit(edit).unwrap()
        });
        assert!(matches!(
            commit.publication(),
            CssomPublication::Unchanged { .. }
        ));
        assert_eq!(commit.owner_effects().len(), 1);
        assert_eq!(store.snapshot().revision(), before.revision());
        assert!(stats.bytes_allocated <= 4096);
        report(
            "unrelated_declarations_with_required_owner_effect",
            declarations,
            "repeat_empty_css_text_commit",
            stats,
        );
    }
}
