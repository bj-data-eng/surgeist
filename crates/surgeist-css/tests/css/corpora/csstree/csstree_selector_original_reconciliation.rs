//! Independently specified original single-selector grammar and source-traced
//! diagnostic contracts. Expected values never come from the runtime or registry.

use serde_json::{Value, json};
use surgeist_css::{CssParser, CssPseudoClass, CssPseudoSelectorListItem, CssSelector};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, assert_selected_publications,
    language_range_token_kind, namespace_context, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("selector-original-reconciliation.json", RowPaths::Crate)
}

fn pseudo_meaning(pseudo: &CssPseudoClass) -> Value {
    match pseudo {
        CssPseudoClass::Not(list) => {
            let selectors: Vec<_> = list
                .items()
                .iter()
                .map(|item| match item {
                    CssPseudoSelectorListItem::Selector(selector) => typed_meaning(selector),
                    _ => panic!("strict admitted Not member"),
                })
                .collect();
            json!({"pseudo_class": "Not", "selectors": selectors})
        }
        CssPseudoClass::Lang(list) => {
            let ranges: Vec<_> = list
                .ranges()
                .iter()
                .map(|range| {
                    json!({
                        "token_kind": language_range_token_kind(range.kind()), "value": range.as_str()
                    })
                })
                .collect();
            json!({"pseudo_class": "Lang", "ranges": ranges})
        }
        other => panic!("unexpected original pseudo: {other:?}"),
    }
}

fn compound_meaning(compound: &surgeist_css::CssCompoundSelector) -> Value {
    assert!(compound.pseudo_elements().is_none());
    assert!(!compound.has_scope_anchor());
    assert_eq!(compound.nesting_selectors(), 0);
    let mut items = Vec::new();
    if let Some(name) = compound.type_selector() {
        items.push(
            json!({"type_selector": {"local_name": name.local_name().unwrap_or("*"),
            "namespace": format!("{:?}", name.namespace())}}),
        );
    }
    items.extend(compound.ids().iter().map(|id| json!({"id": id})));
    items.extend(
        compound
            .classes()
            .iter()
            .map(|class| json!({"class": class})),
    );
    items.extend(compound.attributes().iter().map(|attribute| {
        let (matcher, value) = match attribute.matcher() {
            surgeist_css::CssAttributeMatcher::Exists => ("Exists", None),
            surgeist_css::CssAttributeMatcher::Equals(value) => ("Equals", Some(value.as_str())),
            other => panic!("unexpected original attribute matcher: {other:?}"),
        };
        json!({"attribute": {"local_name": attribute.name().as_str(),
            "namespace": format!("{:?}", attribute.namespace()), "matcher": matcher,
            "value": value, "case_sensitivity": format!("{:?}", attribute.case_sensitivity())}})
    }));
    items.extend(compound.pseudo_classes().iter().map(pseudo_meaning));
    json!({"kind": "compound", "items": items})
}

fn typed_meaning(selector: &CssSelector) -> Value {
    // Normalize only the public compatibility variants. All retained semantic
    // fields are checked, regardless of the parser's equivalent enum projection.
    match selector {
        CssSelector::Tag(name) => json!({"kind": "compound", "items": [
            {"type_selector": {"local_name": name, "namespace": "Any"}}]}),
        CssSelector::Class(name) => json!({"kind": "compound", "items": [{"class": name}]}),
        CssSelector::Key(name) => json!({"kind": "compound", "items": [{"id": name}]}),
        CssSelector::PseudoClass(pseudo) => {
            json!({"kind": "compound", "items": [pseudo_meaning(pseudo)]})
        }
        CssSelector::Compound(compound) => compound_meaning(compound),
        CssSelector::Complex(complex) => {
            let rest: Vec<_> = complex.rest().iter().map(|part| json!({
                "combinator": format!("{:?}", part.combinator()), "selector": compound_meaning(part.selector())
            })).collect();
            json!({"kind": "complex", "first": compound_meaning(complex.first()), "rest": rest})
        }
        other => panic!("unexpected original selector: {other:?}"),
    }
}

/// Public selector presence, strict validation and typed meaning for one row.
fn typed_selector_reconciles(row: &Value) -> Result<(), String> {
    let report = CssSelector::parse(
        &CssParser::default(),
        text(row, "input"),
        &namespace_context(),
    );
    let expected = &row["expected"];
    let actual = json!({"present": report.syntax().is_some(),
        "validation_accepts": report.clone().into_validation_result().is_ok(),
        "typed_expectation": report.syntax().as_ref().map(typed_meaning)});
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
fn original_selectors_preserve_typed_meaning_and_exact_recovery() {
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::Selector,
            extractor: Some(Extractor::Selector),
        },
        |_| true,
        |row| PredictedObservation {
            syntax_count: row["expected"]["outer_count"].as_u64().unwrap(),
            is_clean: row["expected"]["is_clean"].as_bool().unwrap(),
            diagnostics: row["expected_raw_diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_selector_reconciles),
    );
    assert_eq!(seen, 14);
}

#[test]
fn original_selector_registry_matches_independent_complete_original_classes() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_selector_identity_options_and_selected_publication_remain_bound() {
    let reconciliation = reconciliation();
    assert_selected_publications(&reconciliation.data()["normative_sources"]);
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 14);
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "selector"),
        14
    );
}
