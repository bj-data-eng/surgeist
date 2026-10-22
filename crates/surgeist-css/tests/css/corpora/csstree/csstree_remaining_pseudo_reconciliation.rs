//! Independently specified original remaining pseudo grammar and source-traced
//! diagnostic contracts. Expected values never come from the runtime or registry.

use serde_json::{Value, json};
use surgeist_css::{CssPseudoClass, CssSelector, parse_selector};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, catalog, catalog_module,
    language_range_token_kind, namespace_context, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("remaining-pseudo-reconciliation.json", RowPaths::Crate)
}

fn compound_argument(argument: &surgeist_css::CssCompoundSelectorArgument) -> Value {
    let compound = argument.compound();
    assert!(compound.ids().is_empty());
    assert!(compound.attributes().is_empty());
    assert!(compound.pseudo_classes().is_empty());
    assert!(compound.pseudo_elements().is_none());
    assert!(!compound.has_scope_anchor());
    assert_eq!(compound.nesting_selectors(), 0);
    let mut members = Vec::new();
    if let Some(name) = compound.type_selector() {
        members.push(json!({"type_selector": name.local_name().unwrap(),
            "namespace": format!("{:?}", name.namespace())}));
    }
    members.extend(compound.classes().iter().map(|name| json!({"class": name})));
    json!(members)
}

fn typed_meaning(selector: &CssSelector) -> Value {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::Dir(direction)) => {
            json!({"pseudo_class": "Dir", "identifier": direction.as_str()})
        }
        CssSelector::PseudoClass(CssPseudoClass::Lang(languages)) => {
            let ranges: Vec<_> = languages.ranges().iter().map(|range| {
                json!({"token_kind": language_range_token_kind(range.kind()), "value": range.as_str()})
            }).collect();
            json!({"pseudo_class": "Lang", "ranges": ranges})
        }
        CssSelector::PseudoClass(CssPseudoClass::Has(list)) => {
            let selectors: Vec<_> = list
                .selectors()
                .iter()
                .map(|relative| {
                    let CssSelector::Class(name) = relative.selector() else {
                        panic!("expected original single-class relative selector: {relative:?}");
                    };
                    json!({"leading_combinator": format!("{:?}", relative.combinator()),
                    "compound": [{"class": name}]})
                })
                .collect();
            json!({"pseudo_class": "Has", "relative_selectors": selectors})
        }
        CssSelector::PseudoClass(CssPseudoClass::Host) => {
            json!({"pseudo_class": "host", "functional": false, "compound_argument": null})
        }
        CssSelector::PseudoClass(CssPseudoClass::HostFunction(argument)) => {
            json!({"pseudo_class": "host", "functional": true, "compound_argument": compound_argument(argument)})
        }
        CssSelector::PseudoClass(CssPseudoClass::HostContext(argument)) => {
            json!({"pseudo_class": "host-context", "functional": true, "compound_argument": compound_argument(argument)})
        }
        CssSelector::Compound(compound) => {
            assert!(compound.type_selector().is_none());
            assert!(compound.ids().is_empty());
            assert!(compound.classes().is_empty());
            assert!(compound.attributes().is_empty());
            assert!(compound.pseudo_classes().is_empty());
            assert!(!compound.has_scope_anchor());
            assert_eq!(compound.nesting_selectors(), 0);
            let [
                surgeist_css::CssPseudoElementSegment::PseudoElement(
                    surgeist_css::CssPseudoElement::Slotted(argument),
                ),
            ] = compound.pseudo_elements().unwrap().segments()
            else {
                panic!("expected sole slotted pseudo-element");
            };
            json!({"pseudo_element": "slotted", "functional": true, "compound_argument": compound_argument(argument)})
        }
        other => panic!("unexpected retained original pseudo: {other:?}"),
    }
}

/// Public selector presence and typed meaning for one row.
fn typed_pseudo_reconciles(row: &Value) -> Result<(), String> {
    let report = parse_selector(text(row, "input"), &namespace_context());
    let expected = &row["expected"];
    let actual = json!({"present": report.syntax().is_some(),
        "typed_expectation": report.syntax().as_ref().map(typed_meaning)});
    let predicted = json!({"present": expected["semantic_acceptance"],
        "typed_expectation": expected["typed_expectation"]});
    if actual == predicted {
        Ok(())
    } else {
        Err(format!("expected meaning {predicted}, actual {actual}"))
    }
}

#[test]
fn original_remaining_pseudos_preserve_typed_meaning_and_exact_recovery() {
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
        Some(&typed_pseudo_reconciles),
    );
    assert_eq!(seen, 75);
}

#[test]
fn remaining_registry_matches_independent_complete_original_classes() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_remaining_identity_options_and_selected_publication_remain_bound() {
    let reconciliation = reconciliation();
    let data = reconciliation.data();
    let publication = &catalog_module(&json!("selectors-4"))["publication"];
    // The saved whole-catalog digest is historical provenance. Bind this test
    // to its actual normative edition, allowing unrelated catalog maintenance.
    for key in ["url", "sha256", "bytes", "date", "kind"] {
        assert_eq!(
            publication[key], data["diagnostic_audits"]["nonshadow"]["normative_source"][key],
            "selected publication {key}"
        );
    }
    let catalog = catalog();
    for (name, source) in data["diagnostic_audits"]["shadow"]["normative_sources"]
        .as_object()
        .unwrap()
    {
        let actual = if name == "selectors" {
            publication
        } else {
            catalog["normative_definitions"]["sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|actual| actual["id"] == source["id"])
                .unwrap()
        };
        for key in ["url", "sha256", "bytes", "date", "kind", "revision", "path"] {
            if let Some(value) = source.get(key) {
                assert_eq!(&actual[key], value, "{name} source {key}");
            }
        }
    }
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 75);
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "selector"),
        75
    );
}
