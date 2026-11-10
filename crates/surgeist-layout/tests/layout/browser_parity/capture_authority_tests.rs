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
    policy: String,
}
impl Fixture {
    fn new(record: &str) -> Self {
        let hash = "a".repeat(64);
        let policy = "b".repeat(64);
        let xml = sha256(b"independent XML bytes");
        let inputs = BTreeMap::from([
            ("html/fixed.html".to_owned(), hash.clone()),
            ("policy.md".to_owned(), policy.clone()),
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
            policy,
        }
    }
    fn verified(&self) -> Result<VerifiedIdentity> {
        self.with_observation("geometry", None)
    }
    fn with_observation(
        &self,
        request: &str,
        request_sha: Option<&str>,
    ) -> Result<VerifiedIdentity> {
        identity_from_verified_inputs(IdentityInputs {
            metadata: &self.metadata,
            artifact: &self.artifact,
            case: &self.case,
            full_inputs: &self.inputs,
            excluded_specs: &self.excluded,
            actual_xml_sha256: &self.xml,
            historical_sha256: None,
            policy_sha256: &self.policy,
            browser_settings: &self.settings,
            observation_request: request,
            observation_input_sha256: request_sha,
        })
    }
}
#[test]
fn measured_identity_requires_the_exact_request_capture_script_and_stylesheet() {
    let request = sha256(b"{\"schema_version\":1,\"element_scroll_sizes\":\"horizontal-zero-edge-divs\",\"measured_leaf\":\"ahem-10px-zwsp\"}\n");
    let script = sha256(include_bytes!(
        "../../bin/surgeist-layout-generate/measurement/measured_leaf.js"
    ));
    let stylesheet = "5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6";
    let expected = [
        ("observation-request.json", request.as_str()),
        ("scripts/gentest/measured_leaf.js", script.as_str()),
        ("scripts/gentest/test_base_style.css", stylesheet),
    ];
    let fixture = |changed: Option<(&str, Option<&str>)>| {
        let mut fixture = Fixture::new(&"a".repeat(64));
        for (path, digest) in expected {
            if let Some((key, replacement)) = changed
                && path == key
            {
                if let Some(replacement) = replacement {
                    fixture
                        .inputs
                        .insert(path.to_owned(), replacement.to_owned());
                }
            } else {
                fixture.inputs.insert(path.to_owned(), digest.to_owned());
            }
        }
        fixture.metadata.inputs = fixture.inputs.clone();
        fixture
    };
    let valid = fixture(None);
    let verified = valid
        .with_observation("measured-leaf", Some(&request))
        .unwrap_or_else(|error| {
            panic!("measured request and its exact resources must bind: {error}")
        });
    assert_eq!(verified.projection().font_demand, "ahem-10px-zwsp");
    let profile = format!(
        r#"["canonical-measured-grid",{SETTINGS},"{}","content_box_ltr","measured-leaf","{request}"]"#,
        "a".repeat(64)
    );
    assert_eq!(
        verified.projection().profile_sha256,
        sha256(profile.as_bytes())
    );
    assert!(
        valid
            .with_observation("measured-leaf", Some(stylesheet))
            .is_err()
    );
    assert!(valid.with_observation("geometry", None).is_err());
    let mut excluded = fixture(None);
    excluded
        .excluded
        .insert("scripts/gentest/measured_leaf.js".into());
    assert!(
        excluded
            .with_observation("measured-leaf", Some(&request))
            .is_err()
    );
    for (path, digest) in expected {
        assert_eq!(verified.projection().inputs[path], digest);
        for replacement in [
            None,
            Some("0000000000000000000000000000000000000000000000000000000000000000"),
        ] {
            assert!(
                fixture(Some((path, replacement)))
                    .with_observation("measured-leaf", Some(&request))
                    .is_err(),
                "{path} must remain bound even when the report matches the changed inputs"
            );
        }
    }
}

#[test]
fn flex_text_identity_binds_the_request_script_and_font_stylesheet() {
    const REQUEST: &str = "1148009cb8ef089475a09c1a9cda8ab19ac8693c0f4af141f0f2586e173742e5";
    let mut fixture = Fixture::new(&"a".repeat(64));
    let inputs = [
        ("observation-request.json", REQUEST),
        (
            "scripts/gentest/flex_content_text.js",
            "dbc5513b959c33231ea177c294458723e0fafeb154c6af4a391406f73708f189",
        ),
        (
            "scripts/gentest/test_base_style.css",
            "5d00a3f3c55322b7002b065eacc6b4f3f14ecad83f757c79679b6ec6dee4fec6",
        ),
        (
            "scripts/gentest/test_helper.js",
            "af0f458c290f0261106810fa3e42442afe7a5599872774130ac3244548cb74ee",
        ),
    ];
    fixture
        .inputs
        .extend(inputs.map(|(path, hash)| (path.to_owned(), hash.to_owned())));
    fixture.metadata.inputs = fixture.inputs.clone();
    let verified = fixture
        .with_observation("flex-content-text", Some(REQUEST))
        .expect("direct text evidence binds to its own capture profile");
    assert_eq!(
        verified.projection().font_demand,
        "ahem-10px-flex-direct-text"
    );
    let profile = format!(
        r#"["canonical-flex-content-text",{SETTINGS},"{}","content_box_ltr","flex-content-text","{REQUEST}"]"#,
        "a".repeat(64)
    );
    assert_eq!(
        verified.projection().profile_sha256,
        sha256(profile.as_bytes())
    );
    assert!(fixture.with_observation("geometry", None).is_err());
    assert!(fixture.with_observation("flex-content-text", None).is_err());
    assert!(
        fixture
            .with_observation("flex-content-text", Some(&"b".repeat(64)))
            .is_err()
    );

    for (path, hash) in inputs {
        assert_eq!(verified.projection().inputs[path], hash);
        fixture.excluded.insert(path.to_owned());
        assert!(
            fixture
                .with_observation("flex-content-text", Some(REQUEST))
                .is_err()
        );
        fixture.excluded.remove(path);

        fixture.inputs.remove(path);
        fixture.metadata.inputs = fixture.inputs.clone();
        assert!(
            fixture
                .with_observation("flex-content-text", Some(REQUEST))
                .is_err()
        );

        fixture.inputs.insert(path.to_owned(), "b".repeat(64));
        fixture.metadata.inputs = fixture.inputs.clone();
        assert!(
            fixture
                .with_observation("flex-content-text", Some(REQUEST))
                .is_err()
        );
        fixture.inputs.insert(path.to_owned(), hash.to_owned());
        fixture.metadata.inputs = fixture.inputs.clone();
    }
}

#[test]
fn specialized_identity_binds_exact_observation_authority() {
    for (request, font, report, xml) in [
        (
            "prepared-ranges",
            "ahem-16px-prepared-ranges",
            include_str!("captured/prepared-range-differences/xml/generation-reports/all.json"),
            include_bytes!("captured/prepared-range-differences/xml/block/prepared_text_continuity_bidi__border_box_ltr.xml").as_slice(),
        ),
        (
            "flex-composition",
            "none",
            include_str!("captured/flex-composition-differences/xml/generation-reports/all.json"),
            include_bytes!("captured/flex-composition-differences/xml/flex/fri07_flex_composition__border_box_ltr.xml").as_slice(),
        ),
    ] {
        let report = unique_json(report).unwrap();
        let mut fixture = Fixture::new(&"a".repeat(64));
        fixture.metadata = serde_json::from_value(report["metadata"].clone()).unwrap();
        fixture.artifact = serde_json::from_value(report["generated"][0].clone()).unwrap();
        fixture.case.name.clone_from(&fixture.artifact.name);
        fixture.case.source.clone_from(&fixture.artifact.source);
        fixture.case.variant.clone_from(&fixture.artifact.variant);
        fixture.case.output.path.clone_from(&fixture.artifact.output);
        fixture.case.output.sha256.clone_from(&fixture.artifact.output_sha256);
        fixture.xml = sha256(xml);
        fixture.inputs = fixture.metadata.inputs.clone();
        let request_sha = fixture.inputs[OBSERVATION_REQUEST].clone();
        let verified = fixture
            .with_observation(request, Some(&request_sha))
            .unwrap_or_else(|error| panic!("{request}: exact captured identity refused: {error}"));
        assert_eq!(verified.projection().font_demand, font);
        assert_eq!(verified.projection().inputs[OBSERVATION_REQUEST], request_sha);
        assert!(fixture.with_observation(request, None).is_err());
        assert!(
            fixture
                .with_observation(request, Some(&sha256(b"other request")))
                .is_err()
        );
        assert!(fixture.with_observation("geometry", None).is_err());
        assert!(
            fixture
                .with_observation("element-scroll-sizes", Some(&request_sha))
                .is_err()
        );

        for key in [
            OBSERVATION_REQUEST,
            ROOT_HELPER_INPUT,
            "scripts/gentest/test_base_style.css",
            "fonts/Ahem.ttf",
        ] {
            fixture.excluded.insert(key.to_owned());
            assert!(
                fixture.with_observation(request, Some(&request_sha)).is_err(),
                "excluded {request} identity input {key}"
            );
            fixture.excluded.remove(key);
            let original = fixture.inputs.insert(key.to_owned(), sha256(b"changed")).unwrap();
            fixture.metadata.inputs = fixture.inputs.clone();
            assert!(
                fixture.with_observation(request, Some(&request_sha)).is_err(),
                "changed {request} identity input {key}"
            );
            fixture.inputs.insert(key.to_owned(), original);
            fixture.metadata.inputs = fixture.inputs.clone();
        }
        fixture.artifact.linked_resources.insert("fonts/Ahem.ttf".into(), sha256(b"changed"));
        assert!(fixture.with_observation(request, Some(&request_sha)).is_err());
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
    let expected_identity = CaseIdentity::from_verified_projection(json!({
        "source":"html/fixed.html", "source_sha256":"a".repeat(64),
        "variant":"content_box_ltr", "historical_sha256":null,
        "capture_sha256":sha256(b"independent XML bytes"), "record_sha256":"a".repeat(64),
        "generator":"test", "engine_version":"1", "browser_executable":"tmp/browser/chrome",
        "browser_executable_sha256":"a".repeat(64), "browser_provenance":"test/1",
        "profile_sha256":sha256(expected.as_bytes()), "source_imports_sha256":sha256(b"[]"),
        "inputs":{"html/fixed.html":"a".repeat(64), "policy.md":"b".repeat(64), "base.css":"a".repeat(64)},
        "linked_resources":{}, "font_demand":"none", "policy_source_sha256":"b".repeat(64)
    })).unwrap();
    assert_eq!(verified.identity(), &expected_identity);
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

#[test]
fn terminal_wire_preserves_publisher_bytes_and_rejects_broken_associations() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(ARTIFACT_ROOT)
        .join(TERMINAL);
    let raw = std::fs::read_to_string(file).expect("published terminal fixture");
    let terminal = Qualification::parse(&raw).expect("valid emitted terminal");
    assert_eq!(terminal.edition, EDITION);
    assert_eq!(terminal.to_bytes().unwrap(), raw.as_bytes());
    for (pointer, value) in [
        ("/cases/0/identity/capture_sha256", json!("a".repeat(64))),
        ("/cases/0/requirements/output", json!("xml/foreign.xml")),
        ("/records/0/sha256", json!("a".repeat(64))),
        ("/declaration/sha256", json!("a".repeat(64))),
    ] {
        let mut changed: Value = serde_json::from_str(&raw).unwrap();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            Qualification::parse(&changed.to_string()).is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn specialized_terminal_requires_complete_current_and_historical_bindings() {
    const SOURCE: &str = "html/block/prepared_text_continuity_bidi.html";
    const POLICY: &str = "semantic/prepared-range-policy.md";
    const POLICY_SHA: &str = "ef6219d28d15d9131a45d0c48edef12436333a1e1f2868f54d9ec8847d09ada0";
    let report: Value = serde_json::from_str(include_str!(
        "captured/prepared-range-differences/xml/generation-reports/all.json"
    ))
    .unwrap();
    let mut retained: BTreeMap<String, String> =
        serde_json::from_value(report["metadata"]["inputs"].clone()).unwrap();
    retained.insert(POLICY.into(), POLICY_SHA.into());
    let generated = report["generated"].as_array().unwrap();
    let historical = generated
        .iter()
        .map(|artifact| {
            let output = artifact["output"].as_str().unwrap();
            let predecessor = output.replacen("xml/", "historical/", 1);
            (
                artifact["variant"].as_str().unwrap().to_owned(),
                retained[&predecessor].clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let projection = json!({
        "source": SOURCE,
        "source_kind": "original",
        "source_sha256": retained[SOURCE],
        "historical_sha256": historical,
        "specialized_publication": "prepared-range"
    });
    let record_sha = semantic_record_digest(&projection).unwrap();
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(ARTIFACT_ROOT)
        .join(TERMINAL);
    let mut terminal: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    let mut template = terminal["cases"][0]["identity"].clone();
    template["source"] = json!(SOURCE);
    template["source_sha256"] = json!(retained[SOURCE]);
    template["record_sha256"] = json!(record_sha);
    template["policy_source_sha256"] = json!(POLICY_SHA);
    template["font_demand"] = json!("ahem-16px-prepared-ranges");
    terminal["edition"] = json!("prepared-range-qualification");
    terminal["records"] = json!([{"source": SOURCE, "sha256": record_sha, "value": projection}]);
    terminal["declaration"]["sha256"] = json!(retained["corpus.toml"]);
    terminal["capture_validation"] = json!(CaptureValidation::from_retained(&retained).unwrap());
    terminal["cases"] = json!(generated.iter().map(|artifact| {
        let mut identity = template.clone();
        let variant = artifact["variant"].as_str().unwrap();
        let predecessor = artifact["output"].as_str().unwrap().replacen("xml/", "historical/", 1);
        identity["variant"] = json!(variant);
        identity["capture_sha256"] = artifact["output_sha256"].clone();
        identity["historical_sha256"] = json!(retained[&predecessor]);
        json!({
            "case": {
                "name": artifact["name"], "source": SOURCE, "variant": variant,
                "output": {"path": artifact["output"], "sha256": artifact["output_sha256"]},
                "record_sha256": record_sha
            },
            "historical": {"path": predecessor, "sha256": retained[&predecessor]},
            "evidence": {"profile": "prepared-range", "policy": {"path": POLICY, "sha256": POLICY_SHA}},
            "identity": identity
        })
    }).collect::<Vec<_>>());
    terminal["retained_inputs"] = json!(retained);
    let valid = Qualification::parse(&terminal.to_string())
        .expect("all four specialized variants bind to their captured predecessors");
    let serialized = valid.to_bytes().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&serialized).unwrap(),
        terminal
    );

    for (pointer, replacement) in [
        ("/edition", json!("flex-composition-qualification")),
        ("/cases/0/historical", Value::Null),
        (
            "/cases/0/historical/path",
            terminal["cases"][1]["historical"]["path"].clone(),
        ),
        ("/cases/0/historical/sha256", json!("0".repeat(64))),
        ("/cases/0/identity/historical_sha256", Value::Null),
        ("/cases/0/evidence/policy/sha256", json!("0".repeat(64))),
    ] {
        let mut changed = terminal.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            Qualification::parse(&changed.to_string()).is_err(),
            "{pointer}"
        );
    }
    let mut incomplete = terminal.clone();
    incomplete["cases"].as_array_mut().unwrap().pop();
    assert!(Qualification::parse(&incomplete.to_string()).is_err());
    terminal["cases"][0]["spec"] = json!({"path": POLICY, "sha256": POLICY_SHA});
    assert!(Qualification::parse(&terminal.to_string()).is_err());
}

#[test]
fn current_terminal_requires_distinct_producer_and_validator_authority() {
    const VALIDATOR: &str = "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
    const HELPER: &str = "af0f458c290f0261106810fa3e42442afe7a5599872774130ac3244548cb74ee";
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(ARTIFACT_ROOT)
        .join(TERMINAL);
    let mut terminal: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    terminal["schema_version"] = json!(3);
    terminal["capture_validation"] = json!({
        "producer_profile": "root-context-v4",
        "producer_helper_sha256": HELPER,
        "validator_profile": "root-context-flex-composition-v1",
        "validator_helper_sha256": VALIDATOR,
    });
    terminal["current_owner_inputs"]["tests/layout/browser_parity/scripts/gentest/test_helper.js"] =
        json!(VALIDATOR);
    Qualification::parse(&terminal.to_string()).expect("explicit capture and validation authority");

    for (pointer, value) in [
        ("/capture_validation/producer_profile", json!("unknown")),
        (
            "/capture_validation/producer_helper_sha256",
            json!("e".repeat(64)),
        ),
        ("/capture_validation/validator_profile", json!("unknown")),
        (
            "/capture_validation/validator_profile",
            json!("root-context-v4"),
        ),
        ("/capture_validation/validator_helper_sha256", json!(HELPER)),
        (
            "/capture_validation/validator_helper_sha256",
            json!("e".repeat(64)),
        ),
        (
            "/retained_inputs/scripts~1gentest~1test_helper.js",
            json!("e".repeat(64)),
        ),
        (
            "/current_owner_inputs/tests~1layout~1browser_parity~1scripts~1gentest~1test_helper.js",
            json!("e".repeat(64)),
        ),
    ] {
        let mut altered = terminal.clone();
        *altered.pointer_mut(pointer).unwrap() = value;
        assert!(
            Qualification::parse(&altered.to_string()).is_err(),
            "{pointer}"
        );
    }
    terminal["schema_version"] = json!(2);
    terminal
        .as_object_mut()
        .unwrap()
        .remove("capture_validation");
    assert!(Qualification::parse(&terminal.to_string()).is_err());
}

#[test]
fn live_owner_bindings_cover_all_roles_and_refuse_unknown_or_overlapping_paths() {
    let roles = [
        (
            "observation-request.json",
            "tests/bin/surgeist-layout-generate/measurement/measured_leaf_request.json",
        ),
        (
            "scripts/gentest/measured_leaf.js",
            "tests/bin/surgeist-layout-generate/measurement/measured_leaf.js",
        ),
        (
            "browser-owner.toml",
            "tests/layout/browser_parity/corpus.toml",
        ),
        (
            "html/fixed.html",
            "tests/layout/browser_parity/html/fixed.html",
        ),
        (
            "scripts/gentest/base.css",
            "tests/layout/browser_parity/scripts/gentest/base.css",
        ),
        (
            "fonts/font.ttf",
            "tests/layout/browser_parity/fonts/font.ttf",
        ),
        (
            "historical/fixed.xml",
            "tests/layout/browser_parity/xml/fixed.xml",
        ),
        ("semantic/one.json", "semantic/one.json"),
        ("policy.md", "policy.md"),
    ];
    let mut retained = BTreeMap::new();
    let mut expected = BTreeMap::new();
    for (index, (input, owner)) in roles.into_iter().enumerate() {
        let hash = if input == OBSERVATION_REQUEST {
            sha256(MEASURED_LEAF_REQUEST_BYTES)
        } else {
            sha256(format!("independent input {index}").as_bytes())
        };
        retained.insert(input.to_owned(), hash.clone());
        expected.insert(owner.to_owned(), hash);
    }
    for name in [
        "corpus.toml",
        "origin/source-receipt.json",
        "origin/source-provenance.json",
        "origin/selected-originals.json",
    ] {
        retained.insert(name.into(), sha256(name.as_bytes()));
    }
    expected.insert(
        "tests/layout/browser_parity/html/.surgeist-source.json".into(),
        sha256(b"origin/source-receipt.json"),
    );
    let semantic = BTreeSet::from(["semantic/one.json", "policy.md"]);
    assert_eq!(
        current_owner_bindings(&retained, &semantic, "html/.surgeist-source.json").unwrap(),
        expected
    );
    for sidecar in ["../foreign.json", "html/fixed.html", "/absolute.json"] {
        assert!(current_owner_bindings(&retained, &semantic, sidecar).is_err());
    }
    retained.insert("unknown.bin".into(), "a".repeat(64));
    assert!(current_owner_bindings(&retained, &semantic, "html/.surgeist-source.json").is_err());
}

#[test]
fn live_owner_bindings_require_the_exact_specialized_request() {
    const REQUESTS: [(&str, &[u8]); 2] = [
        (
            "prepared_range_request.json",
            include_bytes!(
                "../../bin/surgeist-layout-generate/measurement/prepared_range_request.json"
            ),
        ),
        (
            "flex_composition_request.json",
            include_bytes!("captured/flex-composition-differences/observation-request.json"),
        ),
    ];
    let mut retained = BTreeMap::from([(
        "origin/source-receipt.json".into(),
        sha256(b"independently authored source receipt"),
    )]);
    for (file, bytes) in REQUESTS {
        let digest = sha256(bytes);
        retained.insert(OBSERVATION_REQUEST.into(), digest.clone());
        let owners =
            current_owner_bindings(&retained, &BTreeSet::new(), "html/.surgeist-source.json")
                .unwrap();
        assert_eq!(
            owners.get(&format!(
                "tests/bin/surgeist-layout-generate/measurement/{file}"
            )),
            Some(&digest)
        );
    }
    retained.insert(OBSERVATION_REQUEST.into(), sha256(b"unknown request"));
    assert!(
        current_owner_bindings(&retained, &BTreeSet::new(), "html/.surgeist-source.json").is_err()
    );
}

#[test]
fn matching_terminal_wire_requires_policy_free_complete_original_bindings() {
    // This exercises the terminal wire consumer only. Actual capture bytes,
    // observations and comparison are authenticated again during admission.
    let raw = include_str!("captured/canonical-box-differences/qualification.json");
    let original = Qualification::parse(raw).unwrap();
    let mut terminal: Value = serde_json::from_str(raw).unwrap();
    terminal["edition"] = json!("box-controls");
    let record = &mut terminal["records"][0]["value"];
    record["semantic_specifications"] = Value::Null;
    record["matching_publication"] = json!("box-controls");
    let record_sha = semantic_record_digest(record).unwrap();
    terminal["records"][0]["sha256"] = json!(record_sha);
    for field in ["retained_inputs", "current_owner_inputs"] {
        terminal[field]
            .as_object_mut()
            .unwrap()
            .retain(|path, _| !path.starts_with("semantic/"));
    }
    for case in terminal["cases"].as_array_mut().unwrap() {
        let object = case.as_object_mut().unwrap();
        object.remove("spec");
        object.remove("policy");
        object.remove("requirements");
        object.insert("matching".into(), json!("geometry"));
        case["case"]["record_sha256"] = json!(record_sha);
        let variant = case["case"]["variant"].as_str().unwrap().to_owned();
        let identity = case["identity"].as_object_mut().unwrap();
        identity.remove("policy_source_sha256");
        identity.insert("record_sha256".into(), json!(record_sha));
        identity
            .get_mut("inputs")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .retain(|path, _| !path.starts_with("semantic/"));
        identity.insert(
            "profile_sha256".into(),
            json!(sha256(
                &serde_json::to_vec(&(
                    "canonical-no-text-boxes",
                    &original.browser_settings,
                    record_sha.as_str(),
                    variant.as_str(),
                    "geometry",
                    Option::<&str>::None,
                ))
                .unwrap()
            )),
        );
    }
    let valid = Qualification::parse(&terminal.to_string())
        .expect("complete original matching wire must admit a policy-free capture binding");
    assert_eq!(
        serde_json::from_slice::<Value>(&valid.to_bytes().unwrap()).unwrap(),
        terminal
    );
    for (pointer, replacement) in [
        ("/edition", json!("other-controls")),
        ("/cases/0/matching", json!("unknown")),
        ("/cases/0/historical", Value::Null),
        ("/cases/0/historical/sha256", json!("0".repeat(64))),
        ("/cases/0/identity/historical_sha256", Value::Null),
        ("/cases/0/identity/source_sha256", json!("0".repeat(64))),
        ("/cases/0/identity/record_sha256", json!("0".repeat(64))),
        ("/cases/0/identity/capture_sha256", json!("0".repeat(64))),
    ] {
        let mut changed = terminal.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            Qualification::parse(&changed.to_string()).is_err(),
            "{pointer}"
        );
    }
    let mut incomplete = terminal.clone();
    incomplete["cases"].as_array_mut().unwrap().pop();
    assert!(Qualification::parse(&incomplete.to_string()).is_err());
    let mut duplicate = terminal.clone();
    duplicate["cases"][1] = duplicate["cases"][0].clone();
    assert!(Qualification::parse(&duplicate.to_string()).is_err());
    let mut missing_history = terminal.clone();
    missing_history["records"][0]["value"]["historical_sha256"]
        .as_object_mut()
        .unwrap()
        .remove("border_box_ltr");
    let hash = semantic_record_digest(&missing_history["records"][0]["value"]).unwrap();
    missing_history["records"][0]["sha256"] = json!(hash);
    for case in missing_history["cases"].as_array_mut().unwrap() {
        case["case"]["record_sha256"] = json!(hash);
        case["identity"]["record_sha256"] = json!(hash);
    }
    assert!(Qualification::parse(&missing_history.to_string()).is_err());
    for (pointer, value) in [
        (
            "/cases/0/identity",
            ("policy_source_sha256", json!("0".repeat(64))),
        ),
        (
            "/cases/0",
            (
                "policy",
                json!({"path":"semantic/forbidden-policy.md","sha256":"0".repeat(64)}),
            ),
        ),
    ] {
        let mut changed = terminal.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(value.0.into(), value.1);
        assert!(
            Qualification::parse(&changed.to_string()).is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn matching_capture_binding_authenticates_inputs_without_a_policy_identity() {
    let mut fixture = Fixture::new(&"a".repeat(64));
    fixture.inputs.remove("policy.md");
    fixture.metadata.inputs = fixture.inputs.clone();
    let bind = |fixture: &Fixture| {
        capture_binding_from_verified_inputs(CaptureBindingInputs {
            metadata: &fixture.metadata,
            artifact: &fixture.artifact,
            case: &fixture.case,
            full_inputs: &fixture.inputs,
            excluded_specs: &fixture.excluded,
            actual_xml_sha256: &fixture.xml,
            historical_sha256: None,
            browser_settings: &fixture.settings,
            observation_request: "geometry",
            observation_input_sha256: None,
        })
    };
    let projection = bind(&fixture).unwrap();
    assert!(
        serde_json::to_value(&projection)
            .unwrap()
            .get("policy_source_sha256")
            .is_none()
    );
    fixture.inputs.remove("html/fixed.html");
    assert!(bind(&fixture).is_err());
    fixture
        .inputs
        .insert("html/fixed.html".into(), "0".repeat(64));
    fixture.metadata.inputs = fixture.inputs.clone();
    assert!(bind(&fixture).is_err());
}

include!("capture_authority_script_custody_tests.rs");
