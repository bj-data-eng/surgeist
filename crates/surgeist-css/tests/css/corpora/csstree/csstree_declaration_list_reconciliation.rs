//! Independently specified original declaration-list grammar and source-traced
//! diagnostic contracts. Expected values never come from the runtime or registry.

use serde_json::{Value, json};
use surgeist_css::CssParser;
use surgeist_css::declaration::{CssDeclarationList, CssImportance};
use surgeist_css::property::CssKnownPropertyValueRef;

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, assert_selected_publications,
    text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("declaration-list-reconciliation.json", RowPaths::Crate)
}

fn expected_named_colors(id: &str) -> &'static [&'static str] {
    match id {
        "declarationList/nesting.json#/basic" | "declarationList/nesting.json#/nested @media" => {
            &["blue"]
        }
        "declarationList/tolerant.json#/no colon after a property name" => &["red"],
        _ => &[],
    }
}

fn importance(importance: CssImportance) -> &'static str {
    match importance {
        CssImportance::Normal => "normal",
        CssImportance::Important => "important",
    }
}

fn declarations(list: &CssDeclarationList, expected_names: &[&str]) -> Value {
    assert_eq!(list.len(), expected_names.len());
    let declarations: Vec<_> = list
        .iter()
        .zip(expected_names)
        .map(|(declaration, expected_name)| {
            let Some(CssKnownPropertyValueRef::Color(value)) =
                declaration.known().and_then(|known| known.property_value())
            else {
                panic!("unexpected retained declaration: {declaration:?}");
            };
            assert_eq!(
                value.value().named().expect("named color").name(),
                *expected_name
            );
            json!({"property": "color", "importance": importance(declaration.importance())})
        })
        .collect();
    json!(declarations)
}

/// Public retained declarations, their typed named colors and importance, and
/// strict validation for one row.
fn typed_declarations_reconcile(row: &Value) -> Result<(), String> {
    let input = text(row, "input");
    let report = CssParser::default().parse_block_contents(input);
    let actual = json!({
        "declarations": declarations(report.syntax(), expected_named_colors(text(row, "id"))),
        "validation_accepts": CssParser::default().parse_block_contents(input).into_validation_result().is_ok()});
    // The pinned oracle's RGBA fields record the retired color projection.
    // Keep its property/order/importance contract and assert the authored
    // named branch directly against independent per-case expectations above.
    let expected_declarations: Vec<_> = row["expected"]["ordered_declarations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| json!({"property": item["property"], "importance": item["importance"]}))
        .collect();
    let predicted = json!({"declarations": expected_declarations,
        "validation_accepts": row["expected"]["validation_accepts"]});
    if actual == predicted {
        Ok(())
    } else {
        Err(format!("expected {predicted}, actual {actual}"))
    }
}

#[test]
fn original_declaration_lists_preserve_raw_grammar_typed_values_and_exact_recovery() {
    // The original declaration-list grammar requires the raw style-attribute
    // adapter: no wrapper, and the whole retained list as the extracted syntax.
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::StyleAttribute,
            extractor: Some(Extractor::DeclarationList),
        },
        |_| true,
        |row| PredictedObservation {
            syntax_count: row["expected"]["declaration_count"].as_u64().unwrap(),
            is_clean: row["expected"]["is_clean"].as_bool().unwrap(),
            diagnostics: row["expected_raw_diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_declarations_reconcile),
    );
    assert_eq!(seen, 19);
}

#[test]
fn declaration_list_registry_matches_independent_complete_original_classes() {
    reconciliation().assert_registry_classes();
}

#[test]
fn original_declaration_list_identity_options_and_selected_publications_remain_bound() {
    let reconciliation = reconciliation();
    // Keep the original whole-catalog digest as provenance; unrelated catalog
    // additions do not alter these selected normative editions.
    assert_selected_publications(&reconciliation.data()["normative_sources"]);
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 19);
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "declarationList"),
        19
    );
}
