mod support;

use support::csstree::{
    OracleContractFailureKind, canonicalize_csstree_oracle_schema, load_csstree_corpus,
    load_csstree_oracle, load_csstree_oracle_schema, validate_csstree_oracle_contract,
    validate_shared_support_surface,
};

#[test]
fn committed_csstree_oracle_has_validated_metadata_and_outcome_partition() {
    assert!(validate_shared_support_surface());
    let oracle = load_csstree_oracle().expect("committed CSS oracle should remain validated");
    let cached = load_csstree_oracle().expect("validated CSS oracle should remain cached");
    assert!(std::ptr::eq(oracle, cached));
    let corpus = load_csstree_corpus().expect("oracle corpus should remain validated");
    assert_eq!(corpus.artifact_count(), 74);
    assert_eq!(corpus.case_count(), 935);
    assert_eq!(corpus.parsed_count(), 721);
    assert_eq!(corpus.rejected_count(), 214);
    assert_eq!(
        corpus.context_counts(),
        [
            ("atrule", 130),
            ("atrulePrelude", 2),
            ("block", 29),
            ("declaration", 77),
            ("declarationList", 19),
            ("mediaQuery", 49),
            ("rule", 33),
            ("selector", 317),
            ("selectorList", 10),
            ("stylesheet", 76),
            ("value", 193),
        ]
    );
    assert_eq!(
        oracle.generation_report_sha256(),
        "ed8c70539c10effc254a8c2c4faa750396c7f3647996f5120e9e311bbe96d366"
    );
    assert_eq!(oracle.id_count(), 935);
    assert_eq!(
        oracle.outcome_counts(),
        [
            ("clean", 0),
            ("recovered", 0),
            ("strict_rejected", 0),
            ("unsupported", 935),
        ]
    );
}

#[test]
fn csstree_oracle_rejects_unknown_diagnostic_code() {
    let oracle = oracle_with_diagnostic_tags("not_a_css_error_code", "drop_declaration");
    let error = load_csstree_oracle_schema(oracle.as_bytes())
        .expect_err("unknown diagnostic code should be rejected");
    assert!(
        error.contains("failed to deserialize CSS oracle"),
        "unexpected loader error: {error}"
    );
}

#[test]
fn csstree_oracle_rejects_unknown_diagnostic_action() {
    let oracle = oracle_with_diagnostic_tags("unexpected_token", "not_a_css_recovery_action");
    let error = load_csstree_oracle_schema(oracle.as_bytes())
        .expect_err("unknown diagnostic action should be rejected");
    assert!(
        error.contains("failed to deserialize CSS oracle"),
        "unexpected loader error: {error}"
    );
}

#[test]
fn csstree_oracle_contract_rejects_illegal_records() {
    let mut oracle: serde_json::Value =
        serde_json::from_str(include_str!("csstree/oracle.json")).expect("committed oracle JSON");
    let records = oracle["records"]
        .as_array_mut()
        .expect("oracle records should be an array");

    records.swap(0, 1);
    records[0]["expectation_sha256"] = serde_json::Value::from("0".repeat(64));
    records[10]["path"] = records[0]["path"].clone();
    records[2]["outcome"] = serde_json::json!({ "kind": "clean" });
    records[3]["outcome"]["policy"] = serde_json::Value::from("full_observation");

    let bytes = serde_json::to_vec(&oracle).expect("serialize malformed oracle");
    let bytes = canonicalize_csstree_oracle_schema(&bytes)
        .expect("malformed records should retain the closed JSON schema");
    let failures = validate_csstree_oracle_contract(&bytes)
        .expect_err("independent malformed records should all be rejected");

    assert!(failures.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(failures.iter().all(|failure| {
        !failure.case_id().is_empty()
            && failure.path().starts_with("expectations/")
            && !failure.context().is_empty()
    }));
    assert!(
        failures.iter().any(|failure| {
            failure.kind() == OracleContractFailureKind::NonCanonicalRecordOrder
        })
    );
    assert!(
        failures.iter().any(|failure| {
            failure.kind() == OracleContractFailureKind::ExpectationDigestMismatch
        })
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.kind() == OracleContractFailureKind::PathMismatch)
    );
    assert!(failures.iter().any(|failure| {
        failure.kind() == OracleContractFailureKind::ProbeOutcomePolicyMismatch
    }));
    assert!(failures.iter().any(|failure| {
        failure.kind() == OracleContractFailureKind::UnsupportedPolicyObservationMismatch
    }));
}

#[test]
fn csstree_oracle_contract_binds_every_neutral_record_field() {
    let bytes = mutate_canonical_oracle(|oracle| {
        let records = oracle["records"].as_array_mut().expect("oracle records");
        records[10]["path"] = records[0]["path"].clone();
        records[11]["expectation_sha256"] = serde_json::Value::from("0".repeat(64));
        records[12]["source"] = serde_json::Value::from("source/other.json");
        records[13]["context"] = serde_json::Value::from("value");
        records[14]["input"] = serde_json::Value::from("different input");
        records[15]["options"] = serde_json::json!({ "parseValue": false });
    });
    let failures = validate_csstree_oracle_contract(&bytes)
        .expect_err("every neutral identity drift should be rejected together");
    let kinds = failures
        .iter()
        .map(|failure| failure.kind())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(kinds.contains(&OracleContractFailureKind::PathMismatch));
    assert!(kinds.contains(&OracleContractFailureKind::ExpectationDigestMismatch));
    assert!(kinds.contains(&OracleContractFailureKind::SourceMismatch));
    assert!(kinds.contains(&OracleContractFailureKind::ContextMismatch));
    assert!(kinds.contains(&OracleContractFailureKind::InputMismatch));
    assert!(kinds.contains(&OracleContractFailureKind::OptionsMismatch));
}

#[test]
fn csstree_oracle_contract_rejects_every_malformed_schema_class() {
    let schema_escapes: [fn(&mut serde_json::Value); 9] = [
        |oracle| oracle["unknown"] = serde_json::Value::Bool(true),
        |oracle| oracle["records"][0]["context"] = serde_json::Value::from("unknown"),
        |oracle| oracle["records"][0]["options"]["unknown"] = serde_json::Value::Bool(true),
        |oracle| oracle["records"][0]["probe"]["kind"] = serde_json::Value::from("unknown"),
        |oracle| oracle["records"][0]["probe"]["adapter"] = serde_json::Value::from("unknown"),
        |oracle| oracle["records"][0]["outcome"]["kind"] = serde_json::Value::from("expected_fail"),
        |oracle| oracle["records"][0]["outcome"]["kind"] = serde_json::Value::from("quarantined"),
        |oracle| {
            oracle["records"][0]["outcome"]["reason"] = serde_json::Value::from("not_implemented")
        },
        |oracle| oracle["records"][0]["outcome"]["policy"] = serde_json::Value::from("skip"),
    ];
    for edit in schema_escapes {
        let mut oracle = committed_oracle_value();
        edit(&mut oracle);
        let bytes = serde_json::to_vec(&oracle).expect("serialize schema escape");
        assert_typed_contract_rejection(&bytes);
    }

    let valid_schema_failures: [fn(&mut serde_json::Value); 8] = [
        |oracle| oracle["schema_version"] = serde_json::Value::from(2),
        |oracle| oracle["expectation_schema_version"] = serde_json::Value::from(2),
        |oracle| oracle["source_revision"] = serde_json::Value::from("stale"),
        |oracle| oracle["source_tree"] = serde_json::Value::from("stale"),
        |oracle| oracle["generation_report_sha256"] = serde_json::Value::from("0".repeat(64)),
        |oracle| {
            let records = oracle["records"].as_array_mut().expect("oracle records");
            records[1]["id"] = records[0]["id"].clone();
        },
        |oracle| {
            oracle["records"]
                .as_array_mut()
                .expect("oracle records")
                .pop();
        },
        |oracle| {
            oracle["records"]
                .as_array_mut()
                .expect("oracle records")
                .swap(0, 1);
        },
    ];
    for edit in valid_schema_failures {
        let bytes = mutate_canonical_oracle(edit);
        assert_typed_contract_rejection(&bytes);
    }

    assert_typed_contract_rejection(b"{not-json");
}

fn committed_oracle_value() -> serde_json::Value {
    serde_json::from_str(include_str!("csstree/oracle.json")).expect("committed oracle JSON")
}

fn mutate_canonical_oracle(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let mut oracle = committed_oracle_value();
    edit(&mut oracle);
    let bytes = serde_json::to_vec(&oracle).expect("serialize mutated oracle");
    canonicalize_csstree_oracle_schema(&bytes).expect("mutation should retain the closed schema")
}

fn assert_typed_contract_rejection(bytes: &[u8]) {
    let failures = validate_csstree_oracle_contract(bytes)
        .expect_err("malformed oracle class should be rejected");
    assert!(!failures.is_empty());
    assert!(failures.iter().all(|failure| {
        !failure.case_id().is_empty() && !failure.path().is_empty() && !failure.context().is_empty()
    }));
}

fn oracle_with_diagnostic_tags(code: &str, action: &str) -> String {
    let oracle = include_str!("csstree/oracle.json");
    let target = concat!(
        "        \"policy\": \"panic_freedom_only\"\n",
        "      },\n",
        "      \"observation\": null",
    );
    let replacement = format!(
        concat!(
            "        \"policy\": \"full_observation\"\n",
            "      }},\n",
            "      \"observation\": {{\n",
            "        \"syntax_count\": {{\n",
            "          \"relation\": \"exact\",\n",
            "          \"value\": 0\n",
            "        }},\n",
            "        \"is_clean\": false,\n",
            "        \"diagnostics\": [\n",
            "          {{\n",
            "            \"code\": {},\n",
            "            \"action\": {},\n",
            "            \"byte_offset\": 0,\n",
            "            \"span_start\": 0,\n",
            "            \"span_end\": 0,\n",
            "            \"multiplicity\": 1\n",
            "          }}\n",
            "        ],\n",
            "        \"payload_relation\": \"ends_at\"\n",
            "      }}",
        ),
        serde_json::to_string(code).expect("diagnostic code should serialize"),
        serde_json::to_string(action).expect("diagnostic action should serialize"),
    );
    assert!(
        oracle.contains(target),
        "CSS oracle should contain a panic-freedom record"
    );
    oracle.replacen(target, &replacement, 1)
}
