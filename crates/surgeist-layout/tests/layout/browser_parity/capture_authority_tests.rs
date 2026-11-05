use super::*;
use serde_json::json;

const SETTINGS: &str = r#"{"source":"test","version":"1","version_output":"1","cache_root":"tmp/browser","provenance_format":"test/{version}","launch":{"batch_size":1,"navigation_timeout_ms":10,"dom_poll_interval_ms":1,"arguments":[]}}"#;
struct Fixture {
    metadata: CaptureMetadata,
    artifact: CapturedArtifact,
    case: CaseDescriptor,
    inputs: BTreeMap<String, String>,
    excluded: BTreeSet<String>,
    settings: BrowserSettingsWitness,
    xml: String,
}
impl Fixture {
    fn new(record: &str) -> Self {
        let hash = "a".repeat(64);
        let policy = "b".repeat(64);
        let xml = sha256(b"independent XML bytes");
        let inputs = BTreeMap::from([
            ("html/fixed.html".to_owned(), hash.clone()),
            ("policy.md".to_owned(), policy),
            ("base.css".to_owned(), hash.clone()),
            ("corpus.toml".to_owned(), "c".repeat(64)),
            ("semantic/one.json".to_owned(), "d".repeat(64)),
            ("semantic/two.json".to_owned(), "e".repeat(64)),
        ]);
        let metadata=serde_json::from_value(json!({"schema_version":4,"generator":"test","engine_version":"1",
            "host_executable_sha256":hash,"browser_executable":"tmp/browser/chrome","browser_executable_sha256":hash,
            "browser_provenance":"test/1","contract_sha256":hash,"inputs":inputs,"source_imports":[]})).unwrap();
        let artifact=serde_json::from_value(json!({"name":"fixed","source":"html/fixed.html","output":"xml/fixed.xml","variant":"content_box_ltr",
            "source_sha256":hash,"linked_resources":{},"job_sha256":hash,"output_sha256":xml})).unwrap();
        let case = CaseDescriptor {
            name: "fixed".into(),
            source: "html/fixed.html".into(),
            variant: "content_box_ltr".into(),
            output: PathDigest {
                path: "xml/fixed.xml".into(),
                sha256: xml.clone(),
            },
            record_sha256: record.to_owned(),
        };
        Self {
            metadata,
            artifact,
            case,
            inputs,
            excluded: BTreeSet::from(["semantic/one.json".into(), "semantic/two.json".into()]),
            settings: BrowserSettingsWitness::from_official_json(SETTINGS.as_bytes()).unwrap(),
            xml,
        }
    }
    fn verified(&self) -> Result<VerifiedIdentity> {
        identity_from_verified_inputs(IdentityInputs {
            metadata: &self.metadata,
            artifact: &self.artifact,
            case: &self.case,
            full_inputs: &self.inputs,
            excluded_specs: &self.excluded,
            actual_xml_sha256: &self.xml,
            historical_sha256: None,
            policy_sha256: &self.inputs["policy.md"],
            browser_settings: &self.settings,
            observation_request: "geometry",
            observation_input_sha256: None,
        })
    }
}
#[test]
fn lossless_settings_witness_preserves_the_existing_profile_tuple_bytes() {
    let fixture = Fixture::new(&"a".repeat(64));
    let verified = fixture.verified().unwrap();
    let expected = format!(
        r#"["canonical-no-text-boxes",{SETTINGS},"{}","content_box_ltr","geometry",null]"#,
        "a".repeat(64)
    );
    assert_eq!(
        verified.projection().profile_sha256,
        sha256(expected.as_bytes())
    );
    assert_eq!(serde_json::to_string(&fixture.settings).unwrap(), SETTINGS);
    assert!(!verified.projection().inputs.contains_key("corpus.toml"));
    assert!(
        !verified
            .projection()
            .inputs
            .contains_key("semantic/one.json")
    );
    assert!(
        !verified
            .projection()
            .inputs
            .contains_key("semantic/two.json")
    );
    assert_eq!(
        verified.projection().inputs.get("base.css"),
        Some(&"a".repeat(64))
    );
}
#[test]
fn pure_identity_requires_the_complete_actual_report_input_and_output_mapping() {
    let mut fixture = Fixture::new(&"a".repeat(64));
    fixture
        .inputs
        .insert("undeclared.css".into(), "a".repeat(64));
    assert!(
        matches!(fixture.verified(),Err(QualificationError::IdentityChanged {component}) if component=="capture metadata input census")
    );
    let mut fixture = Fixture::new(&"a".repeat(64));
    fixture.xml = sha256(b"changed XML");
    assert!(matches!(
        fixture.verified(),
        Err(QualificationError::IdentityChanged { .. })
    ));
    let mut fixture = Fixture::new(&"a".repeat(64));
    fixture.case.source = "html/foreign.html".into();
    assert!(matches!(
        fixture.verified(),
        Err(QualificationError::IdentityChanged { .. })
    ));
}
#[test]
fn shared_record_digest_excludes_only_spec_registration_and_retains_full_projection() {
    let original = json!({"source":"html/fixed.html","source_sha256":"a".repeat(64),"source_kind":"original","semantic_specifications":{"content_box_ltr":{"path":"semantic/one.json","sha256":"d".repeat(64)}}});
    let digest = semantic_record_digest(&original).unwrap();
    let mut changed = original.clone();
    changed["semantic_specifications"]["content_box_ltr"]["sha256"] = json!("e".repeat(64));
    assert_eq!(semantic_record_digest(&changed).unwrap(), digest);
    changed["source_sha256"] = json!("b".repeat(64));
    assert_ne!(semantic_record_digest(&changed).unwrap(), digest);
    assert!(original.get("semantic_specifications").is_some());
}
#[test]
fn settings_witness_rejects_duplicate_unknown_and_incomplete_wire_shapes() {
    for raw in [
        SETTINGS.replacen("\"source\":", "\"source\":\"duplicate\",\"source\":", 1),
        SETTINGS.replacen("\"version\":", "\"unknown\":", 1),
        SETTINGS.replace("\"batch_size\":1,", ""),
    ] {
        assert!(BrowserSettingsWitness::from_official_json(raw.as_bytes()).is_err());
    }
}
