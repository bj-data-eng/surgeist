//! Tests for the `accounting` module: the dimensions of the CSS schema, the
//! front that charges each of them, the scopes a host handle opens, and what
//! a charge does with and without a host.
//!
//! The expected classification is the table recorded for the schema: each
//! dimension, its unit and the fronts that charge it. The expected usage and
//! refusal figures are derived by hand from the charges each test makes.

use surgeist_css_parser::accounting::{
    CssAccount, CssAccountingError, CssAccountingFault, CssChange, CssFront, CssHostScope,
    CssLedger, CssLimits, CssResource, CssResourceTable, CssWork, CssWorkFields, CssWorkTable,
};
use surgeist_ledger::{Dimension, Limits, Operation, ProfileMode, Scope, ScopeReport, Tariffs};

use CssFront::{Cssom, Input, Normalization, Parse, Serialization, SourceRetention};

/// The fronts recorded as charging each work dimension.
const WORK: [(CssWork, &[CssFront]); 18] = [
    (CssWork::InputUnit, &[Input]),
    (CssWork::PreparationStep, &[Input]),
    (CssWork::Token, &[Parse]),
    (CssWork::Component, &[Parse]),
    (CssWork::SpellingByte, &[Parse]),
    (CssWork::SerializedInputNode, &[Serialization]),
    (CssWork::SerializedProjectionNode, &[Serialization]),
    (CssWork::SerializedByte, &[Serialization]),
    (CssWork::NormalizedRule, &[Normalization]),
    (CssWork::NormalizedDeclaration, &[Normalization]),
    (CssWork::NormalizedContribution, &[Normalization]),
    (CssWork::SourceVisit, &[SourceRetention, Cssom]),
    (CssWork::ColorInputUnit, &[CssFront::Color]),
    (CssWork::ColorAction, &[CssFront::Color]),
    (CssWork::CensusRun, &[Cssom]),
    (CssWork::CensusObject, &[Cssom]),
    (CssWork::CensusEntry, &[Cssom]),
    (CssWork::RuleFormatNode, &[Cssom]),
];

/// The fronts recorded as holding each resource dimension.
const RESOURCES: [(CssResource, &[CssFront]); 3] = [
    (CssResource::RetainedObject, &[Cssom]),
    (CssResource::RetainedEntry, &[Cssom]),
    (CssResource::RetainedStringByte, &[Cssom]),
];

/// The recorded name of each front's scope.
const NAMES: [(CssFront, &str); 7] = [
    (Input, "css-input"),
    (Parse, "css-parse"),
    (Serialization, "css-serialize"),
    (Normalization, "css-normalize"),
    (SourceRetention, "css-source"),
    (CssFront::Color, "css-color"),
    (Cssom, "cssom"),
];

fn no_caps() -> CssLimits {
    Limits::unlimited()
}

/// A host operation with a `document` scope below its root, and room for
/// `handles` handles below that scope.
fn host(root: CssLimits, document: CssLimits, handles: usize) -> (CssLedger, Scope) {
    host_with(root, document, handles, ProfileMode::Off)
}

fn host_with(
    root: CssLimits,
    document: CssLimits,
    handles: usize,
    mode: ProfileMode,
) -> (CssLedger, Scope) {
    let scopes = 2 + handles * CssFront::ALL.len();
    let mut operation = Operation::with_profile("host", root, scopes, mode).unwrap();
    let document = operation
        .child(operation.root(), "document", document, Tariffs::identity())
        .unwrap();
    (operation.share(), document)
}

fn token_cap(limit: u64) -> CssLimits {
    CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            Token: limit,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    )
}

fn work(ledger: &CssLedger, scope: Scope, dimension: CssWork) -> u64 {
    ledger.usage(scope).unwrap().work(dimension)
}

fn report(ledger: &CssLedger, scope: Scope) -> ScopeReport<CssWork, CssResource> {
    let snapshot = ledger.snapshot().unwrap().unwrap();
    snapshot
        .scopes
        .into_iter()
        .find(|entry| entry.scope == scope)
        .unwrap()
}

#[test]
fn every_dimension_is_charged_by_the_fronts_recorded_for_it() {
    assert_eq!(
        WORK.map(|(dimension, _)| dimension).as_slice(),
        CssWork::ALL,
        "the recorded table lists every work dimension once, in declaration order"
    );
    assert_eq!(
        RESOURCES.map(|(dimension, _)| dimension).as_slice(),
        CssResource::ALL
    );
    for (dimension, expected) in WORK {
        let charging: Vec<CssFront> = CssFront::ALL
            .into_iter()
            .filter(|front| front.work().contains(&dimension))
            .collect();
        assert_eq!(charging, expected, "{dimension:?}");
    }
    for (dimension, expected) in RESOURCES {
        let holding: Vec<CssFront> = CssFront::ALL
            .into_iter()
            .filter(|front| front.resources().contains(&dimension))
            .collect();
        assert_eq!(holding, expected, "{dimension:?}");
    }
}

#[test]
fn a_front_names_each_of_its_dimensions_once_and_fits_one_charge() {
    for front in CssFront::ALL {
        let work = front.work();
        let resources = front.resources();
        for (position, dimension) in work.iter().enumerate() {
            assert!(!work[..position].contains(dimension), "{front:?}");
        }
        for (position, dimension) in resources.iter().enumerate() {
            assert!(!resources[..position].contains(dimension), "{front:?}");
        }
        assert!(
            work.len() + resources.len() <= CssFront::CHARGE_CHANGES,
            "{front:?} charges {} dimensions",
            work.len() + resources.len()
        );
    }
    assert_eq!(CssFront::CHARGE_CHANGES, 8);
}

#[test]
fn a_handle_opens_one_named_scope_per_front_without_caps() {
    let (ledger, document) = host(no_caps(), no_caps(), 1);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    assert_eq!(handle.parent(), document);
    assert_eq!(NAMES.map(|(front, _)| front), CssFront::ALL);
    for (front, name) in NAMES {
        let scope = handle.scope(front);
        assert_eq!(front.scope_name(), name);
        assert_eq!(ledger.scope_name(scope).unwrap(), name);
        assert_eq!(ledger.limits(scope).unwrap(), no_caps());
        assert_ne!(scope, document);
    }
    for (position, front) in CssFront::ALL.into_iter().enumerate() {
        for other in &CssFront::ALL[..position] {
            assert_ne!(handle.scope(front), handle.scope(*other));
        }
    }
}

#[test]
fn a_charge_reaches_its_front_scope_and_every_scope_above_it_in_the_same_units() {
    let (ledger, document) = host(no_caps(), no_caps(), 1);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    account
        .charge(
            Parse,
            &[
                CssChange::Work(CssWork::Token, 7),
                CssChange::Work(CssWork::Component, 3),
                CssChange::Work(CssWork::SpellingByte, 40),
            ],
        )
        .unwrap();
    account
        .charge(
            Serialization,
            &[CssChange::Work(CssWork::SerializedByte, 11)],
        )
        .unwrap();
    for scope in [handle.scope(Parse), document, ledger.root()] {
        assert_eq!(work(&ledger, scope, CssWork::Token), 7);
        assert_eq!(work(&ledger, scope, CssWork::Component), 3);
        assert_eq!(work(&ledger, scope, CssWork::SpellingByte), 40);
    }
    assert_eq!(
        work(&ledger, handle.scope(Parse), CssWork::SerializedByte),
        0
    );
    for scope in [handle.scope(Serialization), document, ledger.root()] {
        assert_eq!(work(&ledger, scope, CssWork::SerializedByte), 11);
    }
}

#[test]
fn a_charge_is_one_ledger_batch_and_an_empty_charge_is_none() {
    let mode = ProfileMode::Summary { timing: false };
    let (ledger, document) = host_with(no_caps(), no_caps(), 1, mode);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    let full = [
        CssChange::Work(CssWork::SourceVisit, 1),
        CssChange::Work(CssWork::CensusRun, 2),
        CssChange::Work(CssWork::CensusObject, 3),
        CssChange::Work(CssWork::CensusEntry, 4),
        CssChange::Work(CssWork::RuleFormatNode, 5),
        CssChange::Acquire(CssResource::RetainedObject, 6),
        CssChange::Acquire(CssResource::RetainedEntry, 7),
        CssChange::Acquire(CssResource::RetainedStringByte, 8),
    ];
    assert_eq!(full.len(), CssFront::CHARGE_CHANGES);
    account.charge(Cssom, &full).unwrap();
    account.charge(Cssom, &[]).unwrap();
    let cssom = report(&ledger, handle.scope(Cssom));
    assert_eq!(cssom.accepted_batches, 1);
    assert_eq!(cssom.accepted_items, 8);
    assert_eq!(cssom.refused_batches, 0);
    assert_eq!(cssom.usage.work(CssWork::CensusEntry), 4);
    assert_eq!(cssom.usage.live(CssResource::RetainedStringByte), 8);
}

#[test]
fn a_second_handle_opens_its_own_scopes_and_a_clone_shares_them() {
    let (ledger, document) = host(no_caps(), no_caps(), 2);
    let first = CssHostScope::open(ledger.clone(), document).unwrap();
    let second = CssHostScope::open(ledger.clone(), document).unwrap();
    let shared = first.clone();
    for front in CssFront::ALL {
        assert_ne!(first.scope(front), second.scope(front));
        assert_eq!(first.scope(front), shared.scope(front));
        assert_eq!(
            ledger.scope_name(second.scope(front)).unwrap(),
            front.scope_name()
        );
    }
    let token = |amount| [CssChange::Work(CssWork::Token, amount)];
    CssAccount::host(&first).charge(Parse, &token(5)).unwrap();
    CssAccount::host(&second).charge(Parse, &token(9)).unwrap();
    CssAccount::host(&shared).charge(Parse, &token(1)).unwrap();
    assert_eq!(work(&ledger, first.scope(Parse), CssWork::Token), 6);
    assert_eq!(work(&ledger, second.scope(Parse), CssWork::Token), 9);
    assert_eq!(work(&ledger, document, CssWork::Token), 15);
}

#[test]
fn a_spent_scope_bound_is_a_typed_fault() {
    // Room for the root, the document scope and one handle, plus three scopes.
    let scopes = 2 + CssFront::ALL.len() + 3;
    let mut operation = Operation::new("host", no_caps(), scopes).unwrap();
    let document = operation
        .child(operation.root(), "document", no_caps(), Tariffs::identity())
        .unwrap();
    let ledger = operation.share();
    CssHostScope::open(ledger.clone(), document).unwrap();
    assert_eq!(
        CssHostScope::open(ledger, document).unwrap_err(),
        CssAccountingFault::ScopeLimit { limit: scopes }
    );
}

#[test]
fn a_scope_of_another_operation_is_a_typed_fault() {
    let (ledger, _) = host(no_caps(), no_caps(), 1);
    let (_, foreign) = host(no_caps(), no_caps(), 1);
    assert_eq!(
        CssHostScope::open(ledger, foreign).unwrap_err(),
        CssAccountingFault::ForeignScope
    );
}

#[test]
fn a_cap_of_the_supplied_scope_refuses_with_its_limit_total_and_amount() {
    let (ledger, document) = host(no_caps(), token_cap(100), 1);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    account
        .charge(Parse, &[CssChange::Work(CssWork::Token, 60)])
        .unwrap();
    let refused = CssChange::Work(CssWork::Token, 41);
    let stop = account
        .charge(Parse, &[CssChange::Work(CssWork::Component, 5), refused])
        .unwrap_err();
    let refusal = *stop.host_refusal().unwrap();
    assert_eq!(stop, CssAccountingError::Host(refusal));
    assert_eq!(refusal.scope(), document);
    assert_eq!(refusal.change(), refused);
    assert_eq!(refusal.limit(), 100);
    assert_eq!(refusal.total(), 60);
    assert_eq!(refusal.amount(), 41);
    // The batch is admitted whole or not at all.
    for scope in [handle.scope(Parse), document, ledger.root()] {
        assert_eq!(work(&ledger, scope, CssWork::Token), 60);
        assert_eq!(work(&ledger, scope, CssWork::Component), 0);
    }
    // The charge that reaches the cap exactly is admitted.
    account
        .charge(Parse, &[CssChange::Work(CssWork::Token, 40)])
        .unwrap();
    assert_eq!(work(&ledger, document, CssWork::Token), 100);
}

#[test]
fn a_cap_of_a_scope_above_the_supplied_scope_refuses_as_that_scope() {
    let (ledger, document) = host(token_cap(10), no_caps(), 1);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let stop = CssAccount::host(&handle)
        .charge(Parse, &[CssChange::Work(CssWork::Token, 11)])
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(refusal.scope(), ledger.root());
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (10, 0, 11)
    );
}

#[test]
fn a_change_outside_the_front_is_a_fault_that_reaches_no_ledger_call() {
    let mode = ProfileMode::Summary { timing: false };
    let (ledger, document) = host_with(no_caps(), no_caps(), 1, mode);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    let outside = [
        [
            CssChange::Work(CssWork::Token, 1),
            CssChange::Work(CssWork::ColorAction, 1),
        ],
        [
            CssChange::Work(CssWork::Token, 1),
            CssChange::Acquire(CssResource::RetainedObject, 1),
        ],
        [
            CssChange::Work(CssWork::Token, 1),
            CssChange::Release(CssResource::RetainedEntry, 1),
        ],
    ];
    for changes in outside {
        let stop = account.charge(Parse, &changes).unwrap_err();
        assert_eq!(
            stop,
            CssAccountingError::Fault(CssAccountingFault::UnclassifiedChange { front: Parse })
        );
        assert_eq!(stop.host_refusal(), None);
    }
    let parse = report(&ledger, handle.scope(Parse));
    assert_eq!((parse.accepted_batches, parse.refused_batches), (0, 0));
    assert_eq!(parse.usage.work(CssWork::Token), 0);
}

#[test]
fn a_charge_longer_than_the_bound_is_a_fault_that_reaches_no_ledger_call() {
    let mode = ProfileMode::Summary { timing: false };
    let (ledger, document) = host_with(no_caps(), no_caps(), 1, mode);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let changes = [CssChange::Work(CssWork::Token, 1); 9];
    let stop = CssAccount::host(&handle)
        .charge(Parse, &changes)
        .unwrap_err();
    assert_eq!(
        stop,
        CssAccountingError::Fault(CssAccountingFault::ChargeLength {
            front: Parse,
            length: 9
        })
    );
    let parse = report(&ledger, handle.scope(Parse));
    assert_eq!((parse.accepted_batches, parse.refused_batches), (0, 0));
}

#[test]
fn a_gauge_follows_acquisitions_and_releases_and_refuses_an_excess_release() {
    let (ledger, document) = host(no_caps(), no_caps(), 1);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    let entries = CssResource::RetainedEntry;
    account
        .charge(Cssom, &[CssChange::Acquire(entries, 12)])
        .unwrap();
    account
        .charge(Cssom, &[CssChange::Release(entries, 5)])
        .unwrap();
    let usage = ledger.usage(document).unwrap();
    assert_eq!((usage.live(entries), usage.peak(entries)), (7, 12));
    let stop = account
        .charge(Cssom, &[CssChange::Release(entries, 8)])
        .unwrap_err();
    assert_eq!(
        stop,
        CssAccountingError::Fault(CssAccountingFault::ReleaseUnderflow)
    );
    assert_eq!(ledger.usage(document).unwrap().live(entries), 7);
}

#[test]
fn a_counter_that_would_pass_its_range_is_a_fault() {
    let (ledger, document) = host(no_caps(), no_caps(), 1);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    account
        .charge(Parse, &[CssChange::Work(CssWork::Token, u64::MAX)])
        .unwrap();
    let stop = account
        .charge(Parse, &[CssChange::Work(CssWork::Token, 1)])
        .unwrap_err();
    assert_eq!(
        stop,
        CssAccountingError::Fault(CssAccountingFault::CounterOverflow)
    );
    assert_eq!(stop.host_refusal(), None);
    assert_eq!(work(&ledger, document, CssWork::Token), u64::MAX);
}

#[test]
fn a_timed_front_records_one_phase_of_its_scope_and_returns_the_result() {
    let mode = ProfileMode::Summary { timing: false };
    let (ledger, document) = host_with(no_caps(), no_caps(), 1, mode);
    let handle = CssHostScope::open(ledger.clone(), document).unwrap();
    let account = CssAccount::host(&handle);
    let mut runs = 0;
    let result = account.timed(Normalization, || {
        runs += 1;
        "normalized"
    });
    assert_eq!(result, Ok("normalized"));
    assert_eq!(runs, 1);
    assert_eq!(report(&ledger, handle.scope(Normalization)).phases, 1);
    assert_eq!(report(&ledger, handle.scope(Parse)).phases, 0);
    assert_eq!(report(&ledger, document).phases, 0);
}

#[test]
fn an_account_with_no_host_admits_every_charge_and_only_runs_a_timed_front() {
    let account = CssAccount::NONE;
    assert!(account.observer().is_none());
    // No operation exists, so nothing a ledger would refuse or fault is seen.
    let outside = [CssChange::Work(CssWork::ColorAction, u64::MAX); 20];
    for front in CssFront::ALL {
        assert_eq!(account.charge(front, &outside), Ok(()));
        assert_eq!(account.charge(front, &[]), Ok(()));
    }
    let mut runs = 0;
    assert_eq!(account.timed(Parse, || runs += 1), Ok(()));
    assert_eq!(runs, 1);
}

#[test]
fn an_account_with_a_host_names_it() {
    let (ledger, document) = host(no_caps(), no_caps(), 1);
    let handle = CssHostScope::open(ledger, document).unwrap();
    let observer = CssAccount::host(&handle).observer().unwrap();
    assert_eq!(observer.parent(), document);
    assert_eq!(observer.scope(Input), handle.scope(Input));
}
