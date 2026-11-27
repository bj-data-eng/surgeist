//! The source walk steps of a store whose rules were each inserted from their
//! own text, so that every rule retains its own source snapshot.
//!
//! Two figures describe the steps. What a check charges a host as `SourceVisit` is
//! the work of measuring the records it touches: the host-charged steps of a
//! `set_disabled` operation are those of the one sheet record it changes, however
//! many rules the store retains. The size of the walk that reaches the state's
//! sources is what `max_source_traversal_steps` admits: the smallest value at which
//! the store can be built. Building a state creates its tables anew, and a table's
//! iteration order differs from one creation to the next within a process, so
//! figures that depended on the order of the tables would differ for each build of the
//! same state.

use std::sync::atomic::AtomicBool;
use surgeist_cssom::accounting::{CssFront, CssHostScope, CssLedger, CssLimits, CssWork};
use surgeist_cssom::{
    CssomContext, CssomEditGuard, CssomError, CssomInputRole, CssomInputVersion, CssomLimits,
    CssomSheetId, CssomSheetInputs, CssomStore,
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
    built(rules, CssomLimits::default(), Some(host)).unwrap()
}

/// The store of `store_of` under `limits`, or the refusal that stops its building.
fn built(
    rules: usize,
    limits: CssomLimits,
    host: Option<CssHostScope>,
) -> Result<(CssomStore, CssomSheetId), CssomError> {
    let mut store = match host {
        Some(host) => CssomStore::with_host(limits, CssomContext::default(), host)?,
        None => CssomStore::new(limits, CssomContext::default())?,
    };
    let mut edit = store.batch(guard(&store))?;
    let inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".into(),
            revision: 1,
        },
        "document".into(),
        None,
    );
    let ticket = edit.create_parsed_sheet("", inputs)?;
    let sheet = store.commit(edit)?.sheet(&ticket)?.clone();
    let mut edit = store.batch(guard(&store))?;
    let cancel = AtomicBool::new(false);
    for index in 0..rules {
        let text = rule_text(index);
        edit.insert_sheet_rule(&sheet, text.as_str(), index, &cancel)?;
    }
    store.commit(edit)?;
    Ok((store, sheet))
}

/// The steps one `set_disabled` on a freshly built store of `rules` rules charges.
fn census_steps(rules: usize) -> u64 {
    let (ledger, host) = host();
    let (store, sheet) = store_of(rules, host);
    let mut edit = store.batch(guard(&store)).unwrap();
    let before = visits(&ledger);
    edit.set_disabled(&sheet, true).unwrap();
    visits(&ledger) - before
}

/// The smallest `max_source_traversal_steps` under which a store of `rules` rules is built.
fn admitted_steps(rules: usize) -> usize {
    let admits = |max_source_traversal_steps| {
        built(
            rules,
            CssomLimits {
                max_source_traversal_steps,
                ..CssomLimits::default()
            },
            None,
        )
        .is_ok()
    };
    let (mut low, mut high) = (0, 1 << 24);
    assert!(admits(high));
    while low < high {
        let middle = low + (high - low) / 2;
        if admits(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

#[test]
fn the_same_state_built_repeatedly_charges_the_same_steps() {
    let figures = (0..8).map(|_| census_steps(40)).collect::<Vec<_>>();
    assert!(
        figures.iter().all(|steps| *steps == figures[0]),
        "steps charged by eight builds of one state: {figures:?}"
    );
    let admitted = (0..4).map(|_| admitted_steps(40)).collect::<Vec<_>>();
    assert!(
        admitted.iter().all(|steps| *steps == admitted[0]),
        "steps admitted for four builds of one state: {admitted:?}"
    );
}

#[test]
fn a_check_charges_the_steps_of_the_record_it_changes_whatever_the_rule_count() {
    let base = census_steps(25);
    assert!(base > 0, "{base}");
    for rules in [50, 100] {
        assert_eq!(
            census_steps(rules),
            base,
            "steps charged by one set_disabled at {rules} rules and at 25"
        );
    }
}

#[test]
fn the_steps_a_store_is_admitted_under_grow_at_most_linearly() {
    let base = admitted_steps(25);
    assert!(base > 25, "{base}");
    for factor in [2, 4] {
        let steps = admitted_steps(25 * factor);
        assert!(
            steps <= base * factor,
            "steps admitted for {} rules: {steps}, for 25 rules: {base}",
            25 * factor
        );
    }
}
