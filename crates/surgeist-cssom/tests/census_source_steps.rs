//! The source walk steps of a census of a store whose rules were each inserted
//! from their own text, so that every rule retains its own source snapshot.
//!
//! The figure is what one census of the state charges a host as
//! `SourceVisit`: the host-charged steps of a `set_disabled` operation, which
//! runs one census of the state it leaves. Building a state creates its tables
//! anew, and a table's iteration order differs from one creation to the next
//! within a process, so a census whose steps depended on the order it walks
//! the tables in would charge a different figure for each build of the same
//! state.

use std::sync::atomic::AtomicBool;
use surgeist_cssom::accounting::{CssFront, CssHostScope, CssLedger, CssLimits, CssWork};
use surgeist_cssom::{
    CssomContext, CssomEditGuard, CssomInputRole, CssomInputVersion, CssomLimits, CssomSheetId,
    CssomSheetInputs, CssomStore,
};

fn host() -> (CssLedger, CssHostScope) {
    let caps: CssLimits = surgeist_ledger::Limits::unlimited();
    let operation = surgeist_ledger::Operation::new("census", caps, 1 + CssFront::ALL.len())
        .unwrap()
        .share();
    let host = CssHostScope::open(operation.clone(), operation.root()).unwrap();
    (operation, host)
}

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

fn visits(ledger: &CssLedger) -> u64 {
    ledger
        .usage(ledger.root())
        .unwrap()
        .work(CssWork::SourceVisit)
}

/// The text of the rule at `index`: its own selector, and a number of
/// declarations that depends on the index, so that rules retain their snapshots
/// a different number of times.
fn rule_text(index: usize) -> String {
    let declarations = (0..=index % 5)
        .map(|which| format!("margin-top: {}px", 1 + which))
        .collect::<Vec<_>>()
        .join("; ");
    format!(".r{index} {{ color: red; {declarations} }}")
}

/// A store holding one constructed sheet that has `rules` style rules, each
/// inserted from its own text.
fn store_of(rules: usize, host: CssHostScope) -> (CssomStore, CssomSheetId) {
    let mut store =
        CssomStore::with_host(CssomLimits::default(), CssomContext::default(), host).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".into(),
            revision: 1,
        },
        "document".into(),
        None,
    );
    let ticket = edit.create_parsed_sheet("", inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let mut edit = store.batch(guard(&store)).unwrap();
    let cancel = AtomicBool::new(false);
    for index in 0..rules {
        let text = rule_text(index);
        edit.insert_sheet_rule(&sheet, text.as_str(), index, &cancel)
            .unwrap();
    }
    store.commit(edit).unwrap();
    (store, sheet)
}

/// The steps one census of a freshly built store of `rules` rules charges.
fn census_steps(rules: usize) -> u64 {
    let (ledger, host) = host();
    let (store, sheet) = store_of(rules, host);
    let mut edit = store.batch(guard(&store)).unwrap();
    let before = visits(&ledger);
    edit.set_disabled(&sheet, true).unwrap();
    visits(&ledger) - before
}

#[test]
fn the_same_state_built_repeatedly_charges_the_same_census_steps() {
    let figures = (0..8).map(|_| census_steps(40)).collect::<Vec<_>>();
    assert!(
        figures.iter().all(|steps| *steps == figures[0]),
        "census steps of eight builds of one state: {figures:?}"
    );
}

#[test]
fn the_census_steps_of_individually_inserted_rules_grow_at_most_linearly() {
    let base = census_steps(25);
    assert!(base > 25, "{base}");
    for factor in [2, 4] {
        let steps = census_steps(25 * factor as usize);
        assert!(
            steps <= base * factor,
            "census steps of {} rules: {steps}, of 25 rules: {base}",
            25 * factor
        );
    }
}
