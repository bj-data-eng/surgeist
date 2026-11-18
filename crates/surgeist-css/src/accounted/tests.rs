//! What the accounted serialization fronts charge.
//!
//! The independent figure for each count is the smallest limit under which
//! the plain front still succeeds, with the other two limits unbounded: the
//! limits count exactly what the context admits, so the smallest admitting
//! limit is the work a successful call does. The byte count is also the
//! length of the text.

use surgeist_ledger::{Limits, Operation, ProfileMode, Scope, ScopeReport};

use crate::CssParser;
use crate::accounting::{
    CssChange, CssFront, CssHostScope, CssLedger, CssLimits, CssResource, CssResourceTable,
    CssWork, CssWorkFields, CssWorkTable,
};
use crate::rule::{CssRule, CssSheet, CssStyleAncestor};
use crate::selector::CssNamespaceContext;
use crate::serialization::{CssRuleGraphInput, CssRuleSerializationLimits, CssSerializationLimits};

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

/// The input nodes, projection nodes and bytes the root holds.
fn charged(ledger: &CssLedger) -> [u64; 3] {
    let usage = ledger.usage(ledger.root()).unwrap();
    [
        usage.work(CssWork::SerializedInputNode),
        usage.work(CssWork::SerializedProjectionNode),
        usage.work(CssWork::SerializedByte),
    ]
}

fn report(ledger: &CssLedger, scope: Scope) -> ScopeReport<CssWork, CssResource> {
    let snapshot = ledger.snapshot().unwrap().unwrap();
    snapshot
        .scopes
        .into_iter()
        .find(|entry| entry.scope == scope)
        .unwrap()
}

const SEARCH: usize = 1 << 24;

fn limits(input: usize, projection: usize, bytes: usize) -> CssSerializationLimits {
    CssSerializationLimits::new(input, projection, bytes)
}

/// The smallest of each limit under which `admitted` holds, the others
/// unbounded.
fn smallest_limits(admitted: impl Fn(CssSerializationLimits) -> bool) -> [u64; 3] {
    let smallest = |make: &dyn Fn(usize) -> CssSerializationLimits| {
        let (mut low, mut high) = (0_usize, SEARCH);
        assert!(admitted(make(high)), "admitted under the search ceiling");
        while low < high {
            let middle = low + (high - low) / 2;
            if admitted(make(middle)) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        u64::try_from(low).unwrap()
    };
    [
        smallest(&|n| limits(n, usize::MAX, usize::MAX)),
        smallest(&|n| limits(usize::MAX, n, usize::MAX)),
        smallest(&|n| limits(usize::MAX, usize::MAX, n)),
    ]
}

fn unbounded() -> CssSerializationLimits {
    limits(usize::MAX, usize::MAX, usize::MAX)
}

fn sheet(source: &str) -> CssSheet {
    CssParser::default().parse_sheet(source).into_parts().0
}

fn rules(count: usize) -> String {
    (0..count)
        .map(|rule| format!(".c{rule} {{ color: red; margin: {rule}px }}\n@media screen {{ a {{ width: 1px }} }}\n"))
        .collect()
}

const SOURCES: [&str; 4] = [
    "",
    "a { color: red }",
    "@media (width > 1px) { a { margin: 1px 2px 3px } } b { transition: all 1s }",
    // A `@layer` block has no literal CSSOM format, so none is nested here.
    "@supports (display: grid) { .a, .b > .c { grid-template-columns: repeat(2, 1fr) } }",
];

#[test]
fn a_sheet_charges_what_its_limits_admit_for_specified_text() {
    for source in SOURCES {
        let sheet = sheet(source);
        let expected = smallest_limits(|limits| sheet.serialize_specified(limits).is_ok());
        let (ledger, host) = open_host();
        let text = sheet
            .accounted(&host)
            .serialize_specified(unbounded())
            .unwrap()
            .unwrap();
        assert_eq!(text, sheet.serialize_specified(unbounded()).unwrap());
        assert_eq!(charged(&ledger), expected, "{source:?}");
        assert_eq!(expected[2], u64::try_from(text.len()).unwrap());
    }
}

#[test]
fn a_sheet_charges_what_its_limits_admit_for_cssom_text() {
    for source in SOURCES {
        let sheet = sheet(source);
        let expected = smallest_limits(|limits| sheet.serialize_cssom(limits).is_ok());
        let (ledger, host) = open_host();
        let text = sheet
            .accounted(&host)
            .serialize_cssom(unbounded())
            .unwrap()
            .unwrap();
        assert_eq!(text, sheet.serialize_cssom(unbounded()).unwrap());
        assert_eq!(charged(&ledger), expected, "{source:?}");
    }
}

fn first_rule(source: &str) -> CssRule {
    sheet(source).rules()[0].clone()
}

#[test]
fn a_rule_charges_what_its_limits_admit() {
    for source in &SOURCES[1..] {
        let rule = first_rule(source);
        let specified = smallest_limits(|limits| rule.serialize_specified(limits).is_ok());
        let (ledger, host) = open_host();
        let text = rule
            .accounted(&host)
            .serialize_specified(unbounded())
            .unwrap()
            .unwrap();
        assert_eq!(text, rule.serialize_specified(unbounded()).unwrap());
        assert_eq!(charged(&ledger), specified, "{source:?}");

        let cssom = smallest_limits(|limits| rule.serialize_cssom(limits).is_ok());
        let (ledger, host) = open_host();
        let text = rule
            .accounted(&host)
            .serialize_cssom(unbounded())
            .unwrap()
            .unwrap();
        assert_eq!(text, rule.serialize_cssom(unbounded()).unwrap());
        assert_eq!(charged(&ledger), cssom, "{source:?}");
    }
}

fn graph_limits(values: CssSerializationLimits) -> CssRuleSerializationLimits {
    CssRuleSerializationLimits {
        values,
        max_depth: 256,
    }
}

#[test]
fn a_rule_graph_charges_what_its_limits_admit() {
    let namespaces = CssNamespaceContext::default();
    for source in &SOURCES[1..] {
        let rule = first_rule(source);
        let graph = CssRuleGraphInput::from_rule(&rule);

        let specified =
            smallest_limits(|limits| graph.serialize_specified(graph_limits(limits)).is_ok());
        let (ledger, host) = open_host();
        let text = graph
            .accounted(&host)
            .serialize_specified(graph_limits(unbounded()))
            .unwrap()
            .unwrap();
        assert_eq!(
            text,
            graph
                .serialize_specified(graph_limits(unbounded()))
                .unwrap()
        );
        assert_eq!(charged(&ledger), specified, "{source:?}");

        let serialize_cssom = |limits| {
            graph.serialize_cssom(&namespaces, CssStyleAncestor::Absent, graph_limits(limits))
        };
        let cssom = smallest_limits(|limits| serialize_cssom(limits).is_ok());
        let (ledger, host) = open_host();
        let text = graph
            .accounted(&host)
            .serialize_cssom(
                &namespaces,
                CssStyleAncestor::Absent,
                graph_limits(unbounded()),
            )
            .unwrap()
            .unwrap();
        assert_eq!(text, serialize_cssom(unbounded()).unwrap());
        assert_eq!(charged(&ledger), cssom, "{source:?}");
    }
}

#[test]
fn a_call_makes_the_same_ledger_calls_whatever_the_size_of_the_sheet() {
    for count in [1, 100] {
        let sheet = sheet(&rules(count));
        let mode = ProfileMode::Summary { timing: false };
        let (ledger, host) = host(Limits::unlimited(), mode);
        sheet
            .accounted(&host)
            .serialize_cssom(unbounded())
            .unwrap()
            .unwrap();
        for front in CssFront::ALL {
            let scope = report(&ledger, host.scope(front));
            let expected = if front == CssFront::Serialization {
                (1, 3, 1)
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
fn a_refused_serialization_is_returned_inside_ok_and_charged_what_was_admitted() {
    let sheet = sheet(SOURCES[2]);
    let full = smallest_limits(|limits| sheet.serialize_cssom(limits).is_ok());
    let bytes = usize::try_from(full[2]).unwrap();
    for short in [0, 1, bytes / 2, bytes - 1] {
        let limited = limits(usize::MAX, usize::MAX, short);
        let (ledger, host) = open_host();
        let refused = sheet
            .accounted(&host)
            .serialize_cssom(limited)
            .unwrap()
            .unwrap_err();
        assert!(refused.is_resource_limit());
        assert_eq!(
            refused.rule_path(),
            sheet.serialize_cssom(limited).unwrap_err().rule_path()
        );
        let [input, projection, charged_bytes] = charged(&ledger);
        // A refused charge is not counted, so the call stops at or below
        // the limit and below what the whole sheet needs.
        assert!(charged_bytes <= u64::try_from(short).unwrap());
        assert!(input <= full[0] && projection <= full[1]);
        assert!(charged_bytes < full[2]);
    }
}

/// A refused call is charged the work admitted before the refusal: the
/// sheet's root node is admitted before any byte, and a larger limit admits
/// at least as much.
#[test]
fn a_refused_serialization_is_charged_the_work_admitted_before_it() {
    let sheet = sheet(SOURCES[2]);
    let full = smallest_limits(|limits| sheet.serialize_cssom(limits).is_ok());
    let bytes = usize::try_from(full[2]).unwrap();
    let mut previous = [0; 3];
    for short in [0, 1, bytes / 2, bytes - 1] {
        let (ledger, host) = open_host();
        let refused = sheet
            .accounted(&host)
            .serialize_cssom(limits(usize::MAX, usize::MAX, short))
            .unwrap();
        assert!(refused.is_err());
        let now = charged(&ledger);
        assert!(now[0] >= 1, "the root node is admitted: {now:?}");
        for (dimension, (now, before)) in now.iter().zip(previous).enumerate() {
            assert!(
                *now >= before,
                "dimension {dimension} at {short}: {now} < {before}"
            );
        }
        previous = now;
    }
    // One byte short, everything but the last appended text is admitted.
    assert!(previous[2] > 0);

    // Refused on input nodes, one short of what the sheet needs.
    let input = usize::try_from(full[0]).unwrap();
    let (ledger, host) = open_host();
    let refused = sheet
        .accounted(&host)
        .serialize_cssom(limits(input - 1, usize::MAX, usize::MAX))
        .unwrap()
        .unwrap_err();
    assert!(refused.is_resource_limit());
    let [charged_input, _, charged_bytes] = charged(&ledger);
    assert!(
        charged_input >= 1 && charged_input < full[0],
        "{charged_input}"
    );
    assert!(charged_bytes > 0);
}

#[test]
fn a_host_cap_on_serialized_bytes_replaces_the_result_and_charges_nothing() {
    let sheet = sheet("a { color: red }");
    let full = smallest_limits(|limits| sheet.serialize_specified(limits).is_ok());
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            SerializedByte: full[2] - 1,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (ledger, host) = host(caps, ProfileMode::Off);
    let stop = sheet
        .accounted(&host)
        .serialize_specified(unbounded())
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(
        refusal.change(),
        CssChange::Work(CssWork::SerializedByte, full[2])
    );
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (full[2] - 1, 0, full[2])
    );
    assert_eq!(charged(&ledger), [0, 0, 0]);
}

#[test]
fn a_plain_serialization_after_an_accounted_one_charges_nothing() {
    let sheet = sheet(SOURCES[2]);
    let (ledger, host) = open_host();
    sheet
        .accounted(&host)
        .serialize_cssom(unbounded())
        .unwrap()
        .unwrap();
    let after = charged(&ledger);
    let _ = sheet.serialize_cssom(unbounded());
    let _ = sheet.serialize_specified(unbounded());
    assert_eq!(charged(&ledger), after);
}
