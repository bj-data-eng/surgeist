//! Independently derived MQ5 and Syntax expectations for original CSSTree inputs.

use serde_json::Value;
use surgeist_css::{
    CssMediaConditionKind, CssMediaQuery, CssUnknownMediaFeatureReason, CssUnknownMediaFeatureRef,
    parse_media_query,
};

use crate::css::support::csstree::adapters::{EntryPoint, Extractor};
use crate::css::support::csstree::reconciliation::{
    ObservationRoute, PredictedObservation, Reconciliation, RowPaths, text,
};

fn reconciliation() -> Reconciliation {
    Reconciliation::load("media-query-reconciliation.json", RowPaths::Tests)
}

/// The audited retained node and the clean-validation contract for one row.
fn typed_node_reconciles(row: &Value) -> Result<(), String> {
    let id = text(row, "id");
    let input = text(row, "input");
    let report = parse_media_query(input);
    assert_eq!(
        report.clone().into_validation_result().is_ok(),
        row["clean"].as_bool().unwrap(),
        "{id}: clean validation contract"
    );
    match (text(row, "node"), report.syntax()) {
        ("Never", CssMediaQuery::Never(_)) => {}
        (expected, CssMediaQuery::Condition(condition)) => match (expected, condition.kind()) {
            ("Condition.UnknownFeature", CssMediaConditionKind::UnknownFeature(feature)) => {
                assert_eq!(feature.name(), text(row, "feature_name"), "{id}");
                let reason = match text(row, "feature_reason") {
                    "UnknownName" => CssUnknownMediaFeatureReason::UnknownName,
                    "InvalidValue" => CssUnknownMediaFeatureReason::InvalidValue,
                    other => panic!("{id}: unaudited reason {other}"),
                };
                assert_eq!(feature.reason(), reason, "{id}");
                assert_eq!(feature.authored(), Some(input), "{id}");
                assert_eq!(
                    matches!(feature.view(), CssUnknownMediaFeatureRef::Boolean),
                    row["boolean_feature"].as_bool().unwrap(),
                    "{id}: feature form"
                );
            }
            ("Condition.GeneralEnclosed", CssMediaConditionKind::GeneralEnclosed(enclosed)) => {
                assert_eq!(enclosed.authored(), Some(input), "{id}");
            }
            ("Condition.And", CssMediaConditionKind::And(operands)) => {
                let expected = row["enclosed_operands"].as_array().unwrap();
                assert_eq!(operands.conditions().len(), expected.len(), "{id}");
                for (operand, authored) in operands.conditions().iter().zip(expected) {
                    let CssMediaConditionKind::GeneralEnclosed(enclosed) = operand.kind() else {
                        panic!("{id}: expected opaque operand");
                    };
                    assert_eq!(enclosed.authored(), authored.as_str(), "{id}");
                }
            }
            _ => panic!("{id}: unexpected condition {condition:?}"),
        },
        _ => panic!("{id}: unexpected query {:?}", report.syntax()),
    }
    Ok(())
}

#[test]
fn original_media_queries_retain_audited_nodes_and_ordered_diagnostics() {
    // A media query always retains one query: an invalid one becomes `not all`.
    let seen = reconciliation().reconcile_observations(
        ObservationRoute {
            entry_point: EntryPoint::MediaQuery,
            extractor: Some(Extractor::MediaQuery),
        },
        |_| true,
        |row| PredictedObservation {
            syntax_count: 1,
            is_clean: row["clean"].as_bool().unwrap(),
            diagnostics: row["diagnostics"].as_array().unwrap().clone(),
        },
        Some(&typed_node_reconciles),
    );
    assert_eq!(seen, 49);
}

#[test]
fn original_media_identity_bytes_options_and_digests_are_bound() {
    let reconciliation = reconciliation();
    let rows = reconciliation.rows();
    assert_eq!(rows.len(), 49);
    assert_eq!(
        reconciliation.assert_original_bindings(rows, "mediaQuery"),
        49
    );
}

#[test]
fn registry_classes_match_independent_media_query_contracts() {
    reconciliation().assert_registry_classes();
}
