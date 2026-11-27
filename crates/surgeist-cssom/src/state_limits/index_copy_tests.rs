//! The cost of the table index copy of one rule insert, as a measurement.
//!
//! The first mutation of a table after a capture copies its index (see
//! [`crate::shared_table`]), so an insert into a store of N rules copies the indices
//! of the rule, block and list tables, whose lengths grow with N. This test prints, for
//! stores of 1 000, 10 000 and 100 000 rules, the time of one rule insert with its
//! commit in turn (opening the batch, the insert, the commit), the time of the three
//! index copies and of dropping the indices they replace, which the commit does, the
//! copy of the sheet's rule list, the move of the new member in it, the lookup of every
//! member the insert makes, the rebuild of the sheet's namespaces, what is left of the insert's time, and the records and steps
//! one more insert measures, which do not grow with the store. It asserts nothing about
//! time and is ignored by default. Run it with
//!
//! ```sh
//! cargo test --offline --locked -j 1 -p surgeist-cssom --lib -- --ignored --nocapture index_copy
//! ```
//!
//! and add `--release` for the figures of an optimized build. The 100 000-rule store holds
//! about 1.7 GB in a release build.
use crate::*;
use std::hint::black_box;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

fn store_of(rules: usize) -> (CssomStore, CssomSheetId) {
    let text = (0..rules)
        .map(|index| format!(".r{index} {{ width: 1px }}\n"))
        .collect::<String>();
    let limits = CssomLimits {
        max_objects: 1 << 24,
        max_entries: 1 << 26,
        max_string_bytes: 1 << 28,
        max_source_traversal_steps: 1 << 30,
        ..CssomLimits::default()
    };
    let mut store = CssomStore::new(limits, CssomContext::default()).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "index-copy".into(),
            revision: 1,
        },
        "index-copy".into(),
        None,
    );
    let ticket = edit.create_parsed_sheet(&text, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (store, sheet)
}

/// The median of `rounds` timings of `run`.
fn median(rounds: usize, mut run: impl FnMut() -> Duration) -> Duration {
    let mut times = (0..rounds).map(|_| run()).collect::<Vec<_>>();
    times.sort();
    times[times.len() / 2]
}

#[test]
#[ignore = "a measurement, which prints and asserts nothing about time"]
fn index_copy_of_one_rule_insert() {
    // The comparison with the reference census of every record would dominate the insert.
    super::probe::set_cross_check(false);
    for rules in [1_000, 10_000, 100_000] {
        let (mut store, sheet) = store_of(rules);
        let rounds = if rules >= 100_000 { 5 } else { 15 };
        // The time of the batch, the insert and the commit, in turn.
        let mut phases = [Vec::new(), Vec::new(), Vec::new()];
        for _ in 0..rounds {
            let start = Instant::now();
            let mut edit = store.batch(guard(&store)).unwrap();
            let opened = start.elapsed();
            let cancel = AtomicBool::new(false);
            edit.insert_sheet_rule(&sheet, ".added { width: 2px }", 0, &cancel)
                .unwrap();
            let inserted = start.elapsed();
            store.commit(edit).unwrap();
            let committed = start.elapsed();
            phases[0].push(opened);
            phases[1].push(inserted - opened);
            phases[2].push(committed - inserted);
        }
        let [open, operation, commit] = phases.map(|mut times| {
            times.sort();
            times[times.len() / 2]
        });
        let insert = open + operation + commit;
        // The work one more insert measures: a fixed number of records and steps, since an
        // insert changes the records of its rule and the lists, and the sheet's authored
        // tree and the rules around it stay as they are, measured once.
        super::probe::reset();
        let mut edit = store.batch(guard(&store)).unwrap();
        edit.insert_sheet_rule(&sheet, ".more { width: 3px }", 0, &AtomicBool::new(false))
            .unwrap();
        store.commit(edit).unwrap();
        let work = super::probe::read();
        let state = store.state.clone();
        let (rule, block) = (
            state.rules.iter().next().unwrap().0.clone(),
            state.blocks.iter().next().unwrap().0.clone(),
        );
        let (list, members) = state
            .lists
            .iter()
            .max_by_key(|(_, members)| members.len())
            .map(|(id, members)| (id.clone(), members.clone()))
            .unwrap();
        // Each table is shared with the published state, so a mutation copies its index and
        // the commit later drops the index the copy replaces.
        let copies = median(rounds, || {
            let (mut rules_copy, mut blocks_copy, mut lists_copy) = (
                state.rules.clone(),
                state.blocks.clone(),
                state.lists.clone(),
            );
            let start = Instant::now();
            rules_copy.get_mut(&rule).unwrap();
            blocks_copy.get_mut(&block).unwrap();
            lists_copy.get_mut(&list).unwrap();
            let copied = start.elapsed();
            // The commit drops the indices the copies replace, as many entries to release.
            let start = Instant::now();
            drop((rules_copy, blocks_copy, lists_copy));
            copied + start.elapsed()
        });
        // The rule list of the sheet is copied by the first insert of a batch and the old one
        // dropped by the commit, and the new member is moved in.
        let list_copy = median(rounds, || {
            let start = Instant::now();
            drop(black_box(members.clone()));
            start.elapsed()
        });
        let memmove = median(rounds, || {
            let mut copy = members.clone();
            copy.reserve(1);
            let start = Instant::now();
            copy.insert(0, members[0].clone());
            black_box(&copy);
            let moved = start.elapsed();
            drop(copy);
            moved
        });
        // The insert looks up every member of the list for the order of the rules' phases.
        let lookups = median(rounds, || {
            let start = Instant::now();
            for id in &members {
                black_box(state.rules.get(id).map(CssomRule::authored));
            }
            start.elapsed()
        });
        // The insert rebuilds the namespaces of the sheet from its authored rules.
        let authored = state.sheet(&sheet).unwrap().authored();
        let namespaces = median(rounds, || {
            let start = Instant::now();
            black_box(surgeist_css::selector::CssNamespaceContext::from_sheet(
                authored,
            ));
            start.elapsed()
        });
        let rest = insert.saturating_sub(copies + list_copy + memmove + lookups + namespaces);
        println!(
            "rules={rules} tables: rules={} blocks={} lists={} members={} batch_us={} insert_us={} commit_us={} insert_total_us={} \
             split_us: three_index_copies_and_drops={} list_copy_and_drop={} list_memmove={} member_lookups={} namespaces={} unattributed={} \
             records_measured={} steps_measured={}",
            state.rules.len(),
            state.blocks.len(),
            state.lists.len(),
            members.len(),
            open.as_micros(),
            operation.as_micros(),
            commit.as_micros(),
            insert.as_micros(),
            copies.as_micros(),
            list_copy.as_micros(),
            memmove.as_micros(),
            lookups.as_micros(),
            namespaces.as_micros(),
            rest.as_micros(),
            work.examined,
            work.steps,
        );
    }
}
