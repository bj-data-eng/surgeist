//! What the operations of a store with a host charge.
//!
//! The independent figures are the smallest limits under which the same work
//! succeeds without a host: `max_entries` admits exactly the entries a census
//! counts, `max_source_traversal_steps` the steps of its walk, `max_objects`
//! the objects it examines and the state itself, and `max_nodes` the rule
//! nodes a formatting prepares. Where an operation runs several censuses, the
//! test-only census probe gives the counts.

use surgeist_ledger::{Limits, Operation, ProfileMode, ScopeReport};

use crate::accounting::{
    CssChange, CssFront, CssHostScope, CssLedger, CssLimits, CssResource, CssResourceTable,
    CssWork, CssWorkFields, CssWorkTable,
};
use crate::state_limits::probe;
use crate::*;

const SOURCE: &str = "@media screen { .a { color: red } .b { margin: 1px 2px } } .c { color: green } .d { width: 1px }";

fn host(caps: CssLimits, mode: ProfileMode) -> (CssLedger, CssHostScope) {
    let operation = Operation::with_profile("host", caps, 1 + CssFront::ALL.len(), mode)
        .unwrap()
        .share();
    let host = CssHostScope::open(operation.clone(), operation.root()).unwrap();
    (operation, host)
}

fn open_host() -> (CssLedger, CssHostScope) {
    host(Limits::unlimited(), ProfileMode::Off)
}

/// The census runs, objects, entries, walk steps and rule nodes the root holds.
fn charged(ledger: &CssLedger) -> [u64; 5] {
    let usage = ledger.usage(ledger.root()).unwrap();
    [
        CssWork::CensusRun,
        CssWork::CensusObject,
        CssWork::CensusEntry,
        CssWork::SourceVisit,
        CssWork::RuleFormatNode,
    ]
    .map(|dimension| usage.work(dimension))
}

fn since(before: [u64; 5], after: [u64; 5]) -> [u64; 5] {
    std::array::from_fn(|index| after[index] - before[index])
}

fn report(ledger: &CssLedger, host: &CssHostScope) -> ScopeReport<CssWork, CssResource> {
    let scope = host.scope(CssFront::Cssom);
    ledger
        .snapshot()
        .unwrap()
        .unwrap()
        .scopes
        .into_iter()
        .find(|entry| entry.scope == scope)
        .unwrap()
}

fn smallest(admitted: impl Fn(usize) -> bool) -> u64 {
    let (mut low, mut high) = (0_usize, 1 << 24);
    assert!(admitted(high));
    while low < high {
        let middle = low + (high - low) / 2;
        if admitted(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    u64::try_from(low).unwrap()
}

fn version(name: &str) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Document,
        identity: name.into(),
        revision: 1,
    }
}

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

fn make(limits: CssomLimits, host: Option<CssHostScope>) -> Result<CssomStore, CssomError> {
    match host {
        Some(host) => CssomStore::with_host(limits, CssomContext::default(), host),
        None => CssomStore::new(limits, CssomContext::default()),
    }
}

/// A store holding one constructed sheet of `SOURCE`, and that sheet.
fn with_sheet(
    limits: CssomLimits,
    host: Option<CssHostScope>,
) -> Result<(CssomStore, CssomSheetId), CssomError> {
    let mut store = make(limits, host)?;
    let mut edit = store.batch(guard(&store))?;
    let inputs = CssomSheetInputs::constructed(version("document"), "document".into(), None);
    let ticket = edit.create_parsed_sheet(SOURCE, inputs)?;
    let sheet = store.commit(edit)?.sheet(&ticket)?.clone();
    Ok((store, sheet))
}

/// One census of the store `with_sheet` builds, run by disabling its sheet.
fn disabled_sheet(limits: CssomLimits) -> bool {
    with_sheet(limits, None)
        .and_then(|(store, sheet)| store.batch(guard(&store))?.set_disabled(&sheet, true))
        .is_ok()
}

#[test]
fn a_store_with_a_host_charges_its_first_census() {
    let (ledger, host) = open_host();
    CssomStore::with_host(CssomLimits::default(), CssomContext::default(), host).unwrap();
    let new = |limits| CssomStore::new(limits, CssomContext::default()).is_ok();
    let objects = smallest(|max_objects| {
        new(CssomLimits {
            max_objects,
            ..CssomLimits::default()
        })
    });
    let steps = smallest(|max_source_traversal_steps| {
        new(CssomLimits {
            max_source_traversal_steps,
            ..CssomLimits::default()
        })
    });
    assert_eq!(charged(&ledger), [1, objects - 1, 0, steps, 0]);
}

#[test]
fn a_batch_operation_charges_the_census_of_the_state_it_leaves() {
    let (ledger, host) = open_host();
    let (store, sheet) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let before = charged(&ledger);
    edit.set_disabled(&sheet, true).unwrap();
    let objects = smallest(|max_objects| {
        disabled_sheet(CssomLimits {
            max_objects,
            ..CssomLimits::default()
        })
    });
    let entries = smallest(|max_entries| {
        disabled_sheet(CssomLimits {
            max_entries,
            ..CssomLimits::default()
        })
    });
    let steps = smallest(|max_source_traversal_steps| {
        disabled_sheet(CssomLimits {
            max_source_traversal_steps,
            ..CssomLimits::default()
        })
    });
    assert!(objects > 4 && entries > 4 && steps > 4);
    assert_eq!(
        since(before, charged(&ledger)),
        [1, objects - 1, entries, steps, 0]
    );
}

#[test]
fn a_sheet_creation_charges_every_census_run_its_adoption_makes() {
    probe::set_cross_check(false);
    let (ledger, host) = open_host();
    let store = make(CssomLimits::default(), Some(host)).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let before = charged(&ledger);
    probe::reset();
    edit.create_parsed_sheet(SOURCE, CssomSheetInputs::external(version("sheet"), true))
        .unwrap();
    let work = probe::read();
    let runs = probe::whole_runs() + probe::extensions() + 1;
    let charged = since(before, charged(&ledger));
    assert!(probe::extensions() > 2);
    assert_eq!(
        [charged[0], charged[1], charged[3], charged[4]],
        [runs, work.examined, work.steps, 0].map(|count| u64::try_from(count).unwrap())
    );
    assert!(charged[2] > 0);
}

#[test]
fn a_replacement_is_charged_once_before_it_publishes() {
    let mode = ProfileMode::Summary { timing: false };
    let (ledger, host) = host(Limits::unlimited(), mode);
    let (mut store, sheet) = with_sheet(CssomLimits::default(), Some(host.clone())).unwrap();
    let before = report(&ledger, &host);
    store
        .begin_replace_sheet(guard(&store), &sheet, ".e { color: blue }")
        .unwrap();
    let after = report(&ledger, &host);
    // The census of the request, of the batch operation and of the commit.
    assert_eq!(after.accepted_batches - before.accepted_batches, 1);
    assert_eq!(after.phases - before.phases, 1);
    assert_eq!(
        after.usage.work(CssWork::CensusRun) - before.usage.work(CssWork::CensusRun),
        3
    );
}

#[test]
fn a_host_refusal_at_commit_publishes_nothing() {
    // The constructor, the creation's runs and the commit's census.
    let (ledger, host) = open_host();
    let (store, _) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let needed = charged(&ledger)[0];
    drop(store);
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            CensusRun: needed - 1,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (ledger, host) = self::host(caps, ProfileMode::Off);
    let mut store = make(CssomLimits::default(), Some(host)).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let inputs = CssomSheetInputs::constructed(version("document"), "document".into(), None);
    edit.create_parsed_sheet(SOURCE, inputs).unwrap();
    let before = store.snapshot();
    let ran = charged(&ledger)[0];
    let Err(CssomError::Accounting(stop)) = store.commit(edit) else {
        panic!("the host refuses the commit's charge");
    };
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(refusal.change(), CssChange::Work(CssWork::CensusRun, 1));
    assert_eq!((refusal.limit(), refusal.total()), (needed - 1, ran));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert!(store.snapshot().sheets().is_empty());
}

#[test]
fn a_host_refusal_poisons_the_batch() {
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            CensusRun: 1,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (ledger, host) = host(caps, ProfileMode::Off);
    let store = make(CssomLimits::default(), Some(host)).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let inputs = CssomSheetInputs::constructed(version("document"), "document".into(), None);
    let first = edit.create_parsed_sheet(SOURCE, inputs.clone());
    assert!(matches!(first, Err(CssomError::Accounting(_))));
    assert!(matches!(
        edit.create_parsed_sheet(SOURCE, inputs),
        Err(CssomError::BatchAborted)
    ));
    assert_eq!(charged(&ledger)[0], 1);
}

/// A context linking a snapshot, which adds entries and walk steps to the
/// census of an empty state.
fn linked_context() -> CssomContext {
    let mut donor = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = donor.batch(guard(&donor)).unwrap();
    let ticket = edit
        .create_parsed_sheet(
            "a { color: red }",
            CssomSheetInputs::external(version("donor"), true),
        )
        .unwrap();
    let sheet = donor.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    CssomContext {
        inputs: Vec::new(),
        linked: vec![CssomLinkedInput {
            version: version("link"),
            snapshot: donor.snapshot(),
            sheet,
        }],
    }
}

#[test]
fn a_limit_refusal_is_charged_the_work_before_it() {
    let context = linked_context();
    let admitted = |limits| CssomStore::new(limits, context.clone()).is_ok();
    let entries = smallest(|max_entries| {
        admitted(CssomLimits {
            max_entries,
            ..CssomLimits::default()
        })
    });
    let steps = smallest(|max_source_traversal_steps| {
        admitted(CssomLimits {
            max_source_traversal_steps,
            ..CssomLimits::default()
        })
    });
    let strings = smallest(|max_string_bytes| {
        admitted(CssomLimits {
            max_string_bytes,
            ..CssomLimits::default()
        })
    });
    assert!(entries > 0 && steps > 0 && strings > 0);
    let (ledger, host) = open_host();
    let limits = CssomLimits {
        max_string_bytes: usize::try_from(strings).unwrap() - 1,
        ..CssomLimits::default()
    };
    let refused = CssomStore::with_host(limits, context, host);
    assert!(
        matches!(
            refused,
            Err(CssomError::Limit {
                resource: "host and live string bytes",
                ..
            })
        ),
        "{:?}",
        refused.err()
    );
    // The string bytes are checked last, after the totals and the walk.
    assert_eq!(charged(&ledger), [1, 0, entries, steps, 0]);
}

#[test]
fn a_refused_walk_is_charged_the_steps_it_took() {
    let context = linked_context();
    let admitted = |limits| CssomStore::new(limits, context.clone()).is_ok();
    let entries = smallest(|max_entries| {
        admitted(CssomLimits {
            max_entries,
            ..CssomLimits::default()
        })
    });
    let steps = smallest(|max_source_traversal_steps| {
        admitted(CssomLimits {
            max_source_traversal_steps,
            ..CssomLimits::default()
        })
    });
    assert!(steps > 1);
    let (ledger, host) = open_host();
    let limits = CssomLimits {
        max_source_traversal_steps: usize::try_from(steps).unwrap() - 1,
        ..CssomLimits::default()
    };
    let refused = CssomStore::with_host(limits, context, host);
    assert!(matches!(refused, Err(CssomError::Limit { .. })));
    // The step the limit refused is not taken.
    assert_eq!(charged(&ledger), [1, 0, entries, steps - 1, 0]);
}

#[test]
fn a_host_refusal_inside_a_replacement_publishes_nothing() {
    // The runs before the replacement, and the replacement's three.
    let (ledger, host) = open_host();
    with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let before = charged(&ledger)[0];
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            CensusRun: before + 2,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (ledger, host) = self::host(caps, ProfileMode::Off);
    let (mut store, sheet) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let published = store.snapshot();
    let refused = store.begin_replace_sheet(guard(&store), &sheet, ".e { color: blue }");
    let Err(CssomError::Accounting(stop)) = refused else {
        panic!("the host refuses the replacement's charge");
    };
    assert_eq!(
        stop.host_refusal().unwrap().change(),
        CssChange::Work(CssWork::CensusRun, 3)
    );
    assert_eq!(store.snapshot().revision(), published.revision());
    assert_eq!(store.pending_sheet_replacement(&sheet).unwrap(), None);
    assert_eq!(charged(&ledger)[0], before);
}

#[test]
fn formatting_a_rule_charges_the_nodes_it_prepared() {
    let (ledger, host) = open_host();
    let (store, _) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let group = store
        .state
        .rules
        .iter()
        .find(|(_, rule)| {
            matches!(rule.parent, CssomParent::Sheet(_))
                && matches!(rule.data(), CssomRuleData::Group { .. })
        })
        .map(|(id, _)| id.clone())
        .unwrap();
    let edit = store.batch(guard(&store)).unwrap();
    let nodes = smallest(|max_nodes| {
        edit.rule_css_text(
            &group,
            &CssomRuleFormatLimits {
                max_nodes,
                ..CssomRuleFormatLimits::default()
            },
        )
        .is_ok()
    });
    // The group and its two style rules.
    assert_eq!(nodes, 3);
    let before = charged(&ledger);
    let text = edit
        .rule_css_text(&group, &CssomRuleFormatLimits::default())
        .unwrap();
    assert!(text.starts_with("@media screen"), "{text}");
    assert_eq!(since(before, charged(&ledger)), [0, 0, 0, 0, nodes]);
}

/// The accepted batches and phases of the `cssom` scope.
fn batches_and_phases(ledger: &CssLedger, host: &CssHostScope) -> (u64, u64) {
    let scope = report(ledger, host);
    (scope.accepted_batches, scope.phases)
}

fn summary_host() -> (CssLedger, CssHostScope) {
    host(Limits::unlimited(), ProfileMode::Summary { timing: false })
}

#[test]
fn formatting_a_rule_through_its_handle_charges_the_store_host() {
    let (ledger, host) = summary_host();
    let (store, sheet) = with_sheet(CssomLimits::default(), Some(host.clone())).unwrap();
    let rule = CssStyleSheet::from_state(&store.state, &sheet)
        .unwrap()
        .css_rules(&store)
        .unwrap()
        .item(&store, 0)
        .unwrap()
        .unwrap();
    let (before, calls) = (charged(&ledger), batches_and_phases(&ledger, &host));
    let text = rule
        .css_text(&store, &CssomRuleFormatLimits::default())
        .unwrap();
    assert!(text.starts_with("@media screen"), "{text}");
    // The group and its two style rules, as the batch front formats them.
    assert_eq!(since(before, charged(&ledger)), [0, 0, 0, 0, 3]);
    let after = batches_and_phases(&ledger, &host);
    assert_eq!((after.0 - calls.0, after.1 - calls.1), (1, 1));
}

#[test]
fn a_store_object_operation_is_charged_in_one_batch_and_one_phase() {
    let (ledger, host) = summary_host();
    let (mut store, sheet) = with_sheet(CssomLimits::default(), Some(host.clone())).unwrap();
    let handle = CssStyleSheet::from_state(&store.state, &sheet).unwrap();
    let before = batches_and_phases(&ledger, &host);
    let runs = charged(&ledger)[0];
    let _ = handle
        .insert_rule(
            &mut store,
            ".e { color: blue }",
            Some(0),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
    let after = batches_and_phases(&ledger, &host);
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1));
    // Its staging, adoption and commit census runs all land in that batch.
    assert!(charged(&ledger)[0] - runs >= 2);
}

#[test]
fn a_batch_method_of_two_steps_is_one_operation() {
    let (ledger, host) = summary_host();
    let (store, sheet) = with_sheet(CssomLimits::default(), Some(host.clone())).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let before = batches_and_phases(&ledger, &host);
    let runs = charged(&ledger)[0];
    edit.add_sheet_rule(
        &sheet,
        Some(".e"),
        Some("color: blue"),
        None,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    let after = batches_and_phases(&ledger, &host);
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1));
    assert!(charged(&ledger)[0] - runs >= 2);
}

#[test]
fn finishing_and_cancelling_a_replacement_are_charged_once_each() {
    let (ledger, host) = summary_host();
    let (mut store, sheet) = with_sheet(CssomLimits::default(), Some(host.clone())).unwrap();
    let job = store
        .begin_replace_sheet(guard(&store), &sheet, ".e { color: blue }")
        .unwrap();
    let before = batches_and_phases(&ledger, &host);
    let runs = charged(&ledger)[0];
    let done = store
        .finish_replace_sheet(guard(&store), job.parse())
        .unwrap();
    assert!(matches!(done.outcome(), CssomReplaceOutcome::Replaced));
    let after = batches_and_phases(&ledger, &host);
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1));
    assert!(charged(&ledger)[0] - runs >= 2);

    let job = store
        .begin_replace_sheet(guard(&store), &sheet, ".f { color: blue }")
        .unwrap();
    let before = batches_and_phases(&ledger, &host);
    store.cancel_replace_sheet(job.token()).unwrap();
    let after = batches_and_phases(&ledger, &host);
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1));
    assert_eq!(store.pending_sheet_replacement(&sheet).unwrap(), None);
}
