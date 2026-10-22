//! Independently specified original strict selector-list grammar and source-traced
//! diagnostic contracts. Expected values never come from the runtime or registry.

use serde_json::{Value, json};
use surgeist_css::{CssSelector, CssStyleSelector, parse_selector_list};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, assert_selected_publications,
    namespace_context, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load(
        "selector-list-original-reconciliation.json",
        RowPaths::Crate,
    )
}

fn compound_meaning(compound: &surgeist_css::CssCompoundSelector) -> Value {
    let name = compound.type_selector().expect("original type selector");
    json!({"type": {"local_name": name.local_name(), "namespace": format!("{:?}", name.namespace())},
        "ids": compound.ids(), "classes": compound.classes(),
        "attributes": compound.attributes().iter().map(|item| format!("{item:?}")).collect::<Vec<_>>(),
        "pseudo_classes": compound.pseudo_classes().iter().map(|item| format!("{item:?}")).collect::<Vec<_>>(),
        "pseudo_elements": compound.pseudo_elements().map(|item| format!("{item:?}")),
        "nesting_selectors": compound.nesting_selectors(), "scope_anchor": compound.has_scope_anchor()})
}

fn typed_meaning(selector: &CssSelector) -> Value {
    match selector {
        CssSelector::Tag(name) => json!({"kind": "Tag", "name": name, "namespace": "Any"}),
        CssSelector::Complex(complex) => {
            let rest: Vec<_> = complex
                .rest()
                .iter()
                .map(|part| {
                    json!({
                        "combinator": format!("{:?}", part.combinator()),
                        "selector": compound_meaning(part.selector())
                    })
                })
                .collect();
            json!({"kind": "Complex", "first": compound_meaning(complex.first()), "rest": rest})
        }
        other => panic!("unexpected original selector: {other:?}"),
    }
}

fn list_meaning(list: &surgeist_css::CssStyleSelectorList) -> Value {
    let selectors: Vec<_> = list
        .selectors()
        .iter()
        .map(|item| match item {
            CssStyleSelector::Selector(selector) => {
                json!({"kind": "Selector", "selector": typed_meaning(selector)})
            }
            other => panic!("ordinary list retained a non-ordinary member: {other:?}"),
        })
        .collect();
    json!({"kind": "CssStyleSelectorList", "selectors": selectors})
}

/// Public selector-list presence, strict validation and typed meaning for one row.
fn typed_selector_list_reconciles(row: &Value) -> Result<(), String> {
    let report = parse_selector_list(text(row, "input"), &namespace_context());
    let expected = &row["expected"];
    let actual = json!({"present": report.syntax().is_some(),
        "validation_accepts": report.clone().into_validation_result().is_ok(),
        "typed_expectation": report.syntax().as_ref().map(list_meaning)});
    let predicted = json!({"present": expected["semantic_acceptance"],
        "validation_accepts": expected["validation_accepts"],
        "typed_expectation": expected["typed_expectation"]});
    if actual == predicted {
        Ok(())
    } else {
        Err(format!("expected meaning {predicted}, actual {actual}"))
    }
}

#[test]
fn original_selector_lists_preserve_typed_meaning_and_exact_recovery() {
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::SelectorList,
            extractor: Some(Extractor::SelectorList),
        },
        |_| true,
        |row| PredictedObservation {
            syntax_count: row["expected"]["outer_count"].as_u64().unwrap(),
            is_clean: row["expected"]["is_clean"].as_bool().unwrap(),
            diagnostics: row["expected_raw_diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_selector_list_reconciles),
    );
    assert_eq!(seen, 10);
}

#[test]
fn original_selector_list_registry_matches_independent_complete_original_classes() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_selector_list_identity_options_and_selected_publication_remain_bound() {
    let reconciliation = reconciliation();
    assert_selected_publications(&reconciliation.data()["normative_sources"]);
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 10);
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "selectorList"),
        10
    );
}
