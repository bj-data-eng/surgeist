//! What a charged walk charges.
//!
//! The independent figure is the smallest step limit under which the same
//! walk succeeds on a fresh collector: the limit counts every step the
//! collector takes.

use surgeist_ledger::{Limits, Operation, ProfileMode};

use super::{CssSourceRetention, CssSourceRetentionAction, CssSourceRetentionResource};
use crate::CssParser;
use crate::accounting::{
    CssChange, CssFront, CssHostScope, CssLedger, CssLimits, CssResourceTable, CssWork,
    CssWorkFields, CssWorkTable,
};
use crate::rule::CssSheet;

fn host(root: CssLimits, mode: ProfileMode) -> (CssLedger, CssHostScope) {
    let operation = Operation::with_profile("host", root, 1 + CssFront::ALL.len(), mode)
        .unwrap()
        .share();
    let host = CssHostScope::open(operation.clone(), operation.root()).unwrap();
    (operation, host)
}

fn open_host() -> (CssLedger, CssHostScope) {
    host(Limits::unlimited(), ProfileMode::Off)
}

fn visits(ledger: &CssLedger) -> u64 {
    ledger
        .usage(ledger.root())
        .unwrap()
        .work(CssWork::SourceVisit)
}

fn sheet(source: &str) -> CssSheet {
    CssParser::default().parse_sheet(source).into_parts().0
}

fn rules(count: usize) -> String {
    (0..count)
        .map(|rule| format!(".c{rule} {{ color: red; margin: {rule}px }} @media screen {{ a {{ width: 1px }} }}\n"))
        .collect()
}

/// The smallest step limit under which a fresh collector walks `sheet`.
fn smallest_steps(sheet: &CssSheet) -> u64 {
    let admitted = |limit| {
        CssSourceRetention::new(usize::MAX, limit, 256)
            .visit(sheet)
            .is_ok()
    };
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

#[test]
fn a_walk_charges_the_steps_it_took() {
    for source in ["", "a { color: red }", rules(30).as_str()] {
        let sheet = sheet(source);
        let (ledger, host) = open_host();
        let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 256);
        sources
            .charge_walk(&host, |sources| sources.visit(&sheet))
            .unwrap()
            .unwrap();
        assert_eq!(visits(&ledger), smallest_steps(&sheet), "{source:?}");
        assert_eq!(visits(&ledger), u64::try_from(sources.steps()).unwrap());
    }
}

/// A collector keeps counting across walks, and each walk is charged what the
/// collector counted during it. A snapshot an earlier walk recorded costs the
/// same step as a new one, so what a walk is charged does not depend on what
/// came before it.
#[test]
fn a_walk_charges_only_the_steps_taken_during_it() {
    let first = sheet("a { color: red }");
    let second = sheet(&rules(5));
    let (ledger, host) = open_host();
    let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 256);
    sources.visit(&first).unwrap();
    let before = sources.steps();
    assert!(before > 0);
    sources
        .charge_walk(&host, |sources| sources.visit(&second))
        .unwrap()
        .unwrap();
    let after_first_walk = sources.steps();
    assert_eq!(
        visits(&ledger),
        u64::try_from(after_first_walk - before).unwrap()
    );
    assert_eq!(visits(&ledger), smallest_steps(&second));
    sources
        .charge_walk(&host, |sources| sources.visit(&first))
        .unwrap()
        .unwrap();
    assert_eq!(
        visits(&ledger),
        u64::try_from(sources.steps() - before).unwrap()
    );
    assert_eq!(
        visits(&ledger),
        smallest_steps(&second) + smallest_steps(&first)
    );
    assert!(sources.steps() > after_first_walk);
}

#[test]
fn a_refused_walk_is_returned_inside_ok_and_charged_the_steps_before_it() {
    let sheet = sheet(&rules(10));
    let needed = usize::try_from(smallest_steps(&sheet)).unwrap();
    for limit in [0, 1, needed / 2, needed - 1] {
        let (ledger, host) = open_host();
        let mut sources = CssSourceRetention::new(usize::MAX, limit, 256);
        let refused = sources
            .charge_walk(&host, |sources| sources.visit(&sheet))
            .unwrap()
            .unwrap_err();
        assert_eq!(
            refused.exhausted_limit(),
            Some(CssSourceRetentionResource::Visits)
        );
        // Every step under the limit was taken; the refused one is not counted.
        assert_eq!(visits(&ledger), u64::try_from(limit).unwrap());
    }
}

#[test]
fn a_walk_makes_one_batch_and_one_phase_whatever_its_size() {
    for count in [1, 100] {
        let sheet = sheet(&rules(count));
        let mode = ProfileMode::Summary { timing: false };
        let (ledger, host) = host(Limits::unlimited(), mode);
        let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 256);
        sources
            .charge_walk(&host, |sources| sources.visit(&sheet))
            .unwrap()
            .unwrap();
        let snapshot = ledger.snapshot().unwrap().unwrap();
        for front in CssFront::ALL {
            let scope = snapshot
                .scopes
                .iter()
                .find(|entry| entry.scope == host.scope(front))
                .unwrap();
            let expected = if front == CssFront::SourceRetention {
                (1, 1, 1)
            } else {
                (0, 0, 0)
            };
            assert_eq!(
                (scope.accepted_batches, scope.accepted_items, scope.phases),
                expected,
                "{front:?} with {count} rules"
            );
        }
    }
}

#[test]
fn a_host_cap_on_source_visits_replaces_the_result() {
    let sheet = sheet("a { color: red }");
    let needed = smallest_steps(&sheet);
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            SourceVisit: needed - 1,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (ledger, host) = host(caps, ProfileMode::Off);
    let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 256);
    let stop = sources
        .charge_walk(&host, |sources| sources.visit(&sheet))
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(
        refusal.change(),
        CssChange::Work(CssWork::SourceVisit, needed)
    );
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (needed - 1, 0, needed)
    );
    assert_eq!(visits(&ledger), 0);
    // The collector itself took its steps; only the charge was refused.
    assert_eq!(u64::try_from(sources.steps()).unwrap(), needed);
}

/// A walk refused for depth or by the caller's admission is charged the
/// steps the collector counted before the refusal.
#[test]
fn a_walk_refused_for_depth_or_by_admission_is_charged_its_counted_steps() {
    let sheet = sheet(&rules(5));

    // Depth zero refuses the sheet itself, after its first step.
    let (ledger, host) = open_host();
    let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 0);
    let refused = sources
        .charge_walk(&host, |sources| sources.visit(&sheet))
        .unwrap()
        .unwrap_err();
    assert_eq!(
        refused.exhausted_limit(),
        Some(CssSourceRetentionResource::Depth)
    );
    assert_eq!(visits(&ledger), 1);

    // The caller admits ten steps, then refuses.
    let needed = smallest_steps(&sheet);
    let mut steps = 0_u64;
    let mut admit = |action| {
        if matches!(action, CssSourceRetentionAction::Step) {
            steps += 1;
            steps <= 10
        } else {
            true
        }
    };
    let (ledger, host) = open_host();
    let mut sources =
        CssSourceRetention::with_admission(usize::MAX, usize::MAX, 256, &mut admit).unwrap();
    let refused = sources
        .charge_walk(&host, |sources| sources.visit(&sheet))
        .unwrap();
    assert!(refused.is_err());
    assert!(needed > 10);
    assert_eq!(visits(&ledger), 10);
}
