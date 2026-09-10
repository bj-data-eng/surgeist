use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

mod support;

use support::csstree::{
    expected_payload_relation_holds, validate_csstree_expected_classes_contract,
    validate_shared_support_surface,
};

const REPORT_SHA256: &str = "ed8c70539c10effc254a8c2c4faa750396c7f3647996f5120e9e311bbe96d366";

#[test]
fn explicit_expected_classes_bind_every_neutral_case_without_a_default() {
    assert!(validate_shared_support_surface());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(root.join("tests/csstree/expected-classes.json"))
        .expect("read committed expected-class registry");
    let document: Value =
        serde_json::from_slice(&bytes).expect("expected-class registry should be JSON");
    let object = exact_object(
        &document,
        &["schema_version", "generation_report_sha256", "records"],
    );
    assert_eq!(object["schema_version"], 1);
    assert_eq!(object["generation_report_sha256"], REPORT_SHA256);

    let neutral_ids = neutral_case_ids(root);
    assert_eq!(neutral_ids.len(), 935);
    let records = object["records"]
        .as_array()
        .expect("expected-class records should be an array");
    assert_eq!(records.len(), 935);

    let mut bound_ids = BTreeSet::new();
    let mut previous_id: Option<&str> = None;
    for record in records {
        let record = exact_object(record, &["id", "expectation_sha256", "class"]);
        let id = record["id"].as_str().expect("record ID should be a string");
        assert!(
            bound_ids.insert(id.to_owned()),
            "duplicate expected-class ID"
        );
        assert!(previous_id.is_none_or(|previous| previous < id));
        previous_id = Some(id);
        assert_complete_class(&record["class"]);
    }

    assert_eq!(bound_ids, neutral_ids);

    let summary = validate_csstree_expected_classes_contract(&bytes)
        .expect("typed expected-class registry contract should validate");
    assert_eq!(
        summary
            .class_counts()
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>(),
        935
    );
    assert_eq!(
        summary
            .policy_counts()
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>(),
        summary.class_counts()[3].1
    );
    assert_eq!(summary.digest().len(), 64);
}

#[test]
fn expected_class_registry_rejects_malformed_bindings_and_schema_escapes() {
    let committed = include_bytes!("csstree/expected-classes.json");
    let edits: [fn(&mut Value); 10] = [
        |document| document["schema_version"] = Value::from(2),
        |document| document["generation_report_sha256"] = Value::from("0".repeat(64)),
        |document| document["records"][0]["class"]["kind"] = Value::from("default"),
        |document| document["records"][0]["class"] = Value::Null,
        |document| {
            document["records"][0]["class"]["retained_syntax"]["predicate"]["relation"] =
                Value::from("any")
        },
        |document| {
            document["records"][0]["class"]["retained_syntax"]["extractor"]["kind"] =
                Value::from("wrapper_rules")
        },
        |document| {
            document["records"].as_array_mut().expect("records").pop();
        },
        |document| {
            let records = document["records"].as_array_mut().expect("records");
            records[1]["id"] = records[0]["id"].clone();
        },
        |document| {
            document["records"]
                .as_array_mut()
                .expect("records")
                .swap(0, 1);
        },
        |document| document["unknown"] = Value::Bool(true),
    ];
    for edit in edits {
        let mut document: Value = serde_json::from_slice(committed).expect("committed registry");
        edit(&mut document);
        let mut bytes = serde_json::to_string_pretty(&document)
            .expect("serialize malformed registry")
            .into_bytes();
        bytes.push(b'\n');
        assert!(
            validate_csstree_expected_classes_contract(&bytes).is_err(),
            "malformed registry should be rejected"
        );
    }

    let compact: Value = serde_json::from_slice(committed).expect("committed registry");
    let compact = serde_json::to_vec(&compact).expect("compact registry");
    assert!(validate_csstree_expected_classes_contract(&compact).is_err());
}

#[test]
fn payload_relations_use_exclusive_nonempty_intervals_and_exact_payload_end() {
    assert!(expected_payload_relation_holds(
        "intersects",
        10..20,
        10,
        9,
        11
    ));
    assert!(!expected_payload_relation_holds(
        "intersects",
        10..20,
        20,
        19,
        21
    ));
    assert!(!expected_payload_relation_holds(
        "intersects",
        10..20,
        15,
        20,
        21
    ));
    assert!(!expected_payload_relation_holds(
        "intersects",
        10..20,
        15,
        15,
        15
    ));
    assert!(!expected_payload_relation_holds(
        "intersects",
        10..10,
        10,
        9,
        11
    ));
    assert!(expected_payload_relation_holds(
        "ends_at",
        10..20,
        20,
        20,
        20
    ));
    assert!(expected_payload_relation_holds(
        "ends_at",
        10..10,
        10,
        10,
        10
    ));
    assert!(!expected_payload_relation_holds(
        "ends_at",
        10..20,
        20,
        19,
        20
    ));
    assert!(!expected_payload_relation_holds(
        "unknown",
        10..20,
        15,
        14,
        16
    ));
}

#[test]
fn recovery_end_relation_requires_nonempty_payload_overlap_and_exact_end() {
    assert!(expected_payload_relation_holds(
        "recovery_ends_at",
        10..20,
        20,
        10,
        20
    ));
    assert!(expected_payload_relation_holds(
        "recovery_ends_at",
        10..20,
        20,
        9,
        20
    ));
    for (payload, offset, start, end) in [
        (10..20, 20, 20, 20),
        (10..10, 10, 9, 10),
        (10..20, 19, 10, 20),
        (10..20, 20, 10, 21),
        (10..20, 21, 10, 21),
        (10..20, 20, 21, 20),
    ] {
        assert!(
            !expected_payload_relation_holds("recovery_ends_at", payload, offset, start, end),
            "a boundary relation must not accept empty overlap or wrapper diagnostics"
        );
    }
}

fn neutral_case_ids(root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let expectations = root.join("tests/corpus/csstree/expectations");
    for entry in fs::read_dir(&expectations).expect("read neutral expectation root") {
        collect_neutral_ids(
            &entry.expect("read neutral expectation entry").path(),
            &mut ids,
        );
    }
    ids
}

fn collect_neutral_ids(path: &Path, ids: &mut BTreeSet<String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read neutral expectation directory") {
            collect_neutral_ids(&entry.expect("read neutral expectation entry").path(), ids);
        }
        return;
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("json")
        || path.to_string_lossy().contains("generation-reports")
    {
        return;
    }
    let bytes = fs::read(path).expect("read neutral expectation");
    let document: Value =
        serde_json::from_slice(&bytes).expect("neutral expectation should be JSON");
    let cases = document["cases"]
        .as_array()
        .expect("neutral expectation should contain cases");
    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("neutral case ID should be a string");
        assert!(ids.insert(id.to_owned()), "duplicate neutral case ID");
    }
}

fn assert_complete_class(value: &Value) {
    let object = value
        .as_object()
        .expect("expected class should be a tagged object");
    match object.get("kind").and_then(Value::as_str) {
        Some("clean") => {
            exact_keys(object, &["kind", "retained_syntax"]);
            assert_syntax_predicate(&object["retained_syntax"]);
        }
        Some("recovered" | "strict_rejected") => {
            exact_keys(object, &["kind", "retained_syntax", "diagnostics"]);
            assert_syntax_predicate(&object["retained_syntax"]);
            assert_diagnostic_predicates(&object["diagnostics"], false);
        }
        Some("unsupported") => {
            exact_keys(object, &["kind", "reason", "policy"]);
            assert!(matches!(
                object["reason"].as_str(),
                Some(
                    "outside_selected_profile"
                        | "vendor_or_host_specific_syntax"
                        | "upstream_parser_option_without_public_css_equivalent"
                        | "generic_fragment_without_truthful_supported_property_or_descriptor"
                        | "ast_construction_without_public_stylesheet_meaning"
                )
            ));
            assert_unsupported_policy(&object["policy"]);
        }
        _ => panic!("every record requires one reviewed closed expected class"),
    }
}

fn assert_unsupported_policy(value: &Value) {
    let object = value
        .as_object()
        .expect("unsupported policy should be a tagged object");
    match object.get("kind").and_then(Value::as_str) {
        Some("panic_freedom_only") => exact_keys(object, &["kind"]),
        Some("full_observation") => {
            exact_keys(
                object,
                &["kind", "retained_syntax", "is_clean", "diagnostics"],
            );
            assert_syntax_predicate(&object["retained_syntax"]);
            assert!(object["is_clean"].is_boolean());
            assert_diagnostic_predicates(&object["diagnostics"], true);
        }
        _ => panic!("unsupported records require one finite execution policy"),
    }
}

fn assert_syntax_predicate(value: &Value) {
    let object = value
        .as_object()
        .expect("retained syntax should be an object");
    exact_keys(object, &["extractor", "predicate"]);
    assert!(object["extractor"].is_object());
    let predicate = object["predicate"]
        .as_object()
        .expect("syntax predicate should be an object");
    match predicate.get("relation").and_then(Value::as_str) {
        Some("exact") => {
            exact_keys(predicate, &["relation", "value"]);
            assert!(predicate["value"].as_u64().is_some());
        }
        Some("nonempty" | "empty") => exact_keys(predicate, &["relation"]),
        _ => panic!("retained syntax requires an exact, nonempty, or empty predicate"),
    }
}

fn assert_diagnostic_predicates(value: &Value, allow_empty: bool) {
    let diagnostics = value
        .as_array()
        .expect("diagnostic predicates should be an array");
    assert!(allow_empty || !diagnostics.is_empty());
    for diagnostic in diagnostics {
        let diagnostic = exact_object(diagnostic, &["code", "action", "payload_relation"]);
        assert!(diagnostic["code"].is_string());
        assert!(diagnostic["action"].is_string());
        assert!(matches!(
            diagnostic["payload_relation"].as_str(),
            Some("intersects" | "ends_at" | "recovery_ends_at")
        ));
    }
}

fn exact_object<'a>(value: &'a Value, keys: &[&str]) -> &'a Map<String, Value> {
    let object = value.as_object().expect("value should be an object");
    exact_keys(object, keys);
    object
}

fn exact_keys(object: &Map<String, Value>, keys: &[&str]) {
    assert_eq!(
        object.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        keys.iter().copied().collect::<BTreeSet<_>>()
    );
}
