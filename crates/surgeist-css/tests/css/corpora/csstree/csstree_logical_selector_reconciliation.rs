//! Independent Selectors 4 logical-pseudo expectations for the 45 original
//! CSSTree records. The fixture cites pinned grammar and diagnostic source
//! derivations; neither the current parser nor registry supplies its oracle.

use serde_json::Value;
use surgeist_css::{CssPseudoClass, CssPseudoSelectorListItem, CssSelector, parse_selector};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, namespace_context, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("logical-selector-reconciliation.json", RowPaths::Tests)
}

/// Public selector presence and, for recovered forgiving lists, the exact
/// surviving members for one row.
fn typed_members_reconcile(row: &Value) -> Result<(), String> {
    let report = parse_selector(text(row, "input"), &namespace_context());
    if report.syntax().is_some() != row["present"].as_bool().unwrap() {
        return Err(format!("presence differs: {:?}", report.syntax()));
    }
    let Some(members) = row["expected_inner_members_when_recovered"].as_array() else {
        return Ok(());
    };
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list))) =
        report.syntax()
    else {
        return Err(format!("retained forgiving pseudo: {:?}", report.syntax()));
    };
    let expected: Vec<_> = members
        .iter()
        .map(|member| {
            CssPseudoSelectorListItem::Selector(CssSelector::Class(
                member.as_str().unwrap().strip_prefix('.').unwrap().into(),
            ))
        })
        .collect();
    if list.items() == expected {
        Ok(())
    } else {
        Err(format!(
            "actual surviving members {:?}, expected {expected:?}",
            list.items()
        ))
    }
}

fn assert_group(present: bool, clean: bool, count: usize) {
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::Selector,
            extractor: Some(Extractor::Selector),
        },
        |row| row["present"] == present && row["clean"] == clean,
        |row| PredictedObservation {
            syntax_count: u64::from(row["present"].as_bool().unwrap()),
            is_clean: row["clean"].as_bool().unwrap(),
            diagnostics: row["diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_members_reconcile),
    );
    assert_eq!(seen, count);
}

#[test]
fn clean_original_logical_selectors() {
    assert_group(true, true, 19);
}

#[test]
fn recovered_original_logical_selectors_retain_meaningful_members() {
    assert_group(true, false, 8);
}

#[test]
fn invalid_original_logical_selectors_reject_whole_input() {
    assert_group(false, false, 18);
}

#[test]
fn original_identity_bytes_options_and_source_digests_are_bound() {
    let reconciliation = reconciliation();
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 45);
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "selector"),
        45
    );
}

#[test]
fn registry_classes_match_independent_logical_selector_contracts() {
    reconciliation().assert_registry_classes();
}
