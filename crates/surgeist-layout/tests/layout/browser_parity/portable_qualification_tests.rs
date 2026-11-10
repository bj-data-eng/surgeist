//! Functional admission tests with synthetic capture provenance. Neither the
//! provenance nor the authored browser observations claim a real browser run.
//! The complete Native oracle is fixed arithmetic, never a Layout snapshot.
use super::*;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use support::capture_authority::{BrowserSettingsWitness, CaseDescriptor};
use support::exact_differences::{QualifiedVerdict, RawVerdict};

const VARIANTS: [&str; 4] = [
    "border_box_ltr",
    "border_box_rtl",
    "content_box_ltr",
    "content_box_rtl",
];
const SOURCE: &str = "html/qualification/portable-empty.html";
const POLICY: &str = "docs/portable-fixed-box-policy.md";
const OWNER_SOURCE: &str = "examples/qualify-root-context/html/qualification/portable-empty.html";
const ORIGINAL_OWNER_SOURCE: &str =
    "tests/layout/browser_parity/html/qualification/portable-empty.html";
const BASE_CSS: &str = "scripts/gentest/test_base_style.css";
const OWNER_GLOBAL: &str = "tests/layout/browser_parity/scripts/gentest/test_base_style.css";
const PRODUCER_HELPER: &str = "scripts/gentest/test_helper.js";
const VALIDATOR_HELPER: &str = "tests/layout/browser_parity/scripts/gentest/test_helper.js";
const PRODUCER_HELPER_BYTES: &[u8] =
    include_bytes!("captured/canonical-box-differences/scripts/gentest/test_helper.js");
const VALIDATOR_HELPER_BYTES: &[u8] = include_bytes!("scripts/gentest/test_helper.js");
const OWNER_BROWSER: &str = "tests/layout/browser_parity/corpus.toml";
const RECEIPT: &str = "origin/source-receipt.json";
const PROVENANCE: &str = "origin/source-provenance.json";
const SELECTED_ORIGINALS: &str = "origin/selected-originals.json";
const RECEIPT_SIDECAR: &str = "html/.surgeist-source.json";
const OWNER_RECEIPT: &str = "tests/layout/browser_parity/html/.surgeist-source.json";
const REPORT: &str = "xml/generation-reports/all.json";
const OWNERSHIP: &str = "xml/generation-ownership.json";
// Exact official BrowserSettings field order/shape, deliberately synthetic
// values. The portable reader consumes the witness without Generator types.
const SETTINGS: &str = r#"{"source":"synthetic-portable-test","version":"1","version_output":"synthetic 1","cache_root":"tmp/synthetic-browser","provenance_format":"synthetic/{version}","launch":{"batch_size":1,"navigation_timeout_ms":100,"dom_poll_interval_ms":1,"arguments":[]}}"#;
const SOURCE_BYTES: &[u8] = b"<!doctype html><link rel=\"stylesheet\" href=\"../../scripts/gentest/test_base_style.css\"><div style=\"position:absolute;display:flex;width:20px;height:10px\"></div>\n";
const POLICY_BYTES: &[u8] = b"Synthetic policy: empty fixed box width 20, browser observation 21; exact one-field intentional difference.\n";
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

// Independent source arithmetic for one empty 20x10 Flex root: all physical
// edges are zero, overflow is Visible, no named-baseline item is eligible,
// scroll ranges and scrollbars are zero. Both complete packet channels are
// authored, including getters not requested by the browser geometry table.
fn authored_oracle(direction: &str) -> Value {
    let point = json!({"x":0,"y":0});
    let size = json!({"width":20,"height":10});
    let rect = json!({"origin":point,"size":size});
    let edges = json!({"top":0,"right":0,"bottom":0,"left":0});
    let flow = json!({"writing_mode":"horizontal-tb","direction":direction});
    let pair = json!({"first":{"state":"absent"},"last":{"state":"absent"}});
    let facts = json!({
        "source_index":0,"location":point,"size":size,"content_size":size,
        "border":edges,"padding":edges,"margin":edges,
        "named_baselines":{"alphabetic":pair,"central":pair},
        "scroll":{"state":"present","facts":{
            "flow_axes":flow,"used_overflow_x":"visible","used_overflow_y":"visible",
            "border_box":rect,"padding_box":rect,"content_box":rect,"scrollport":rect,
            "clip_x":null,"clip_y":null,"scrollable_overflow":rect,
            "range_x":{"minimum":0,"maximum":0},"range_y":{"minimum":0,"maximum":0},
            "gutter_top":null,"gutter_right":null,"gutter_bottom":null,"gutter_left":null,
            "scrollbar_size":{"width":0,"height":0},"resolved_scroll_padding":edges,
            "optimal_viewing_region":rect,"scroll_snap_type":{"state":"none"},
            "target":{"border_box":rect,"scroll_margin":edges,"flow_axes":flow,
                "block_align":"none","inline_align":"none","snap_stop":"normal"}
        }}
    });
    json!({"path":"/","role":"ordinary-box","children":[],
        "unrounded":{"state":"present","facts":facts},
        "final_output":{"state":"present","facts":facts}})
}
fn case_name(variant: &str) -> String {
    format!("portable-empty__{variant}")
}
fn output_path(variant: &str) -> String {
    format!("xml/qualification/{}.xml", case_name(variant))
}
fn spec_path(variant: &str) -> String {
    format!("semantic/portable-empty-{variant}.json")
}
fn authored_requirements(variant: &str) -> Value {
    json!({"case_id":case_name(variant),"output":output_path(variant),
        "frame":"root-border-local","domain":"layout",
        "selectors":[{"path":"/","field":"x"},{"path":"/","field":"y"},
            {"path":"/","field":"width"},{"path":"/","field":"height"}],
        "nodes":["/"],"unrounded_packets":["/"],"final_packets":["/"]})
}
fn authored_xml(variant: &str) -> String {
    let direction = if variant.ends_with("rtl") {
        "rtl"
    } else {
        "ltr"
    };
    let sizing = if variant.starts_with("border") {
        "border-box"
    } else {
        "content-box"
    };
    // Browser width 21 is authored independently of the input width 20 and the
    // Native oracle. No decoder or browser generated this fixture.
    format!(
        r#"<test name="{}" measurement-version="4" observation-frame="root-border-local" use-rounding="false">
<viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px" containing-direction="ltr" hypothetical-direction="ltr" hypothetical-display="flex" hypothetical-anchor="inline-start" hypothetical-inline-offset="0px" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div source-tag="div" grid-alignment-profile="{{&quot;profile&quot;:1,&quot;owners&quot;:[]}}" display="flex" position="absolute" direction="{direction}" box-sizing="{sizing}" width="20px" height="10px"/></input>
<expectations><node x="0" y="0" width="21" height="10"/></expectations>
<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="21" height="10"/></diagnostics></test>"#,
        case_name(variant)
    )
}
struct Draft {
    xml: Vec<String>,
    specs: Vec<Value>,
}
impl Draft {
    fn authored() -> Self {
        let xml = VARIANTS
            .iter()
            .map(|variant| authored_xml(variant))
            .collect();
        let specs = VARIANTS.iter().map(|variant| {
            let direction = if variant.ends_with("rtl") { "rtl" } else { "ltr" };
            json!({"profile":"canonical-no-text-boxes","identities":{},
                "representation":{"published_edition":"canonical-box-differences",
                    "current":authored_requirements(variant)},
                "policy":{"id":"portable-fixed-box","edition":"1","source":POLICY,
                    "source_sha256":sha256(POLICY_BYTES)},
                "reason":"Independently authored fixed-box portable admission mechanism",
                "native":{"source_derivation":"Empty 20px by 10px Flex root, zero edges, Visible overflow and named absence",
                    "nodes":[authored_oracle(direction)]},
                "differences":[{"domain":"layout","path":"/","field":"width","browser":21,"native":20}]})
        }).collect();
        Self { xml, specs }
    }
}
struct Fixture {
    owner: PathBuf,
    edition: Edition,
    terminal: Value,
    terminal_sha256: String,
}
fn bytes(value: &Value) -> Vec<u8> {
    let mut result = serde_json::to_vec_pretty(value).unwrap();
    result.push(b'\n');
    result
}
fn terminal_bytes(value: &Value) -> Vec<u8> {
    // Value's sorted map order is not the official BrowserSettings serializer
    // order. Preserve the exact witness used to derive profile_sha256. Invalid
    // settings in malformed-wire tests remain invalid, rather than repaired.
    if value.get("browser_settings") != Some(&serde_json::from_str::<Value>(SETTINGS).unwrap()) {
        return bytes(value);
    }
    let mut terminal = value.clone();
    terminal["browser_settings"] = json!("portable-settings-raw-witness");
    let raw = String::from_utf8(bytes(&terminal)).unwrap().replacen(
        "\"portable-settings-raw-witness\"",
        SETTINGS,
        1,
    );
    // Exact deterministic terminal struct order and pretty/LF wire belong to
    // the publisher's schema. Parsing/serialization here provides only wire
    // shape; verified identity was separately derived from actual byte maps.
    // Deliberately malformed terminal tests must stay malformed.
    match Qualification::parse(&raw).and_then(|qualification| qualification.to_bytes()) {
        Ok(canonical) => canonical,
        Err(_) => raw.into_bytes(),
    }
}
fn put(root: &Path, path: &str, contents: &[u8]) {
    let target = root.join(path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, contents).unwrap();
}
fn read_value(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}
fn metadata(inputs: &BTreeMap<String, String>) -> Value {
    json!({"schema_version":4,"generator":"synthetic-portable-fixture",
        "engine_version":"synthetic-1","host_executable_sha256":sha256(b"synthetic host witness"),
        "browser_executable":"tmp/synthetic-browser/chrome",
        "browser_executable_sha256":sha256(b"synthetic browser witness"),
        "browser_provenance":"synthetic/1","contract_sha256":sha256(b"synthetic contract witness"),
        "inputs":inputs,"source_imports":[]})
}
fn declaration_bytes(record: &Value) -> Vec<u8> {
    // Same schema-3 record and semantic-reference representation as the owning
    // canonical TOML declaration. This is retained input, not parsed by a fake.
    let mut result = format!(
        "schema_version = 3\n\n[[records]]\nsource = {SOURCE:?}\nsource_sha256 = {:?}\nsource_kind = \"authored\"\n[records.historical_sha256]\n[records.expected]\nkind = \"generated\"\n[records.expected.metadata]\ncontext = \"absolute-local\"\nroot_writing_mode = \"horizontal-tb\"\nhypothetical_display = \"flex\"\ncontaining_direction = \"ltr\"\nhypothetical_direction = \"ltr\"\nroot_direction = \"variant\"\ninline_offset = 0\n",
        record["source_sha256"].as_str().unwrap()
    );
    for variant in VARIANTS {
        let reference = &record["semantic_specifications"][variant];
        result.push_str(&format!(
            "\n[records.semantic_specifications.{variant}]\npath = {:?}\nsha256 = {:?}\n",
            reference["path"].as_str().unwrap(),
            reference["sha256"].as_str().unwrap()
        ));
    }
    result.into_bytes()
}
impl Fixture {
    fn new() -> Self {
        Self::issue(|_| {})
    }
    fn issue(edit: impl FnOnce(&mut Draft)) -> Self {
        let owner = std::env::temp_dir().join(format!(
            "surgeist-portable-authority-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&owner).unwrap();
        let mut fixture = Self {
            owner,
            edition: Edition::Exact(PublishedEdition::CanonicalBoxes),
            terminal: Value::Null,
            terminal_sha256: String::new(),
        };
        let mut draft = Draft::authored();
        edit(&mut draft);
        fixture.seal(draft);
        fixture
    }
    fn root(&self) -> PathBuf {
        self.owner.join(self.edition.artifact_root())
    }
    fn admit(&self) -> Result<AdmittedEdition> {
        admit(
            &self.owner,
            Registration {
                edition: self.edition,
                terminal_sha256: &self.terminal_sha256,
            },
        )
    }
    fn all(&self) -> BTreeSet<String> {
        VARIANTS
            .iter()
            .map(|variant| output_path(variant))
            .collect()
    }
    fn anchor_terminal_bytes(&mut self, raw: &[u8]) {
        put(&self.root(), authority::TERMINAL, raw);
        self.terminal_sha256 = sha256(raw);
    }
    fn rewrite_terminal(&mut self, edit: impl FnOnce(&mut Value)) {
        edit(&mut self.terminal);
        self.anchor_terminal_bytes(&terminal_bytes(&self.terminal));
    }
    fn rewrite_report_raw(&mut self, raw: &[u8]) {
        put(&self.root(), REPORT, raw);
        let digest = sha256(raw);
        let mut ownership = read_value(&self.root(), OWNERSHIP);
        ownership["artifacts"][REPORT] = json!(digest);
        let ownership_bytes = bytes(&ownership);
        put(&self.root(), OWNERSHIP, &ownership_bytes);
        self.rewrite_terminal(|terminal| {
            terminal["report"]["sha256"] = json!(digest);
            terminal["ownership"]["sha256"] = json!(sha256(&ownership_bytes));
        });
    }
    fn rewrite_report(&mut self, edit: impl FnOnce(&mut Value)) {
        let mut report = read_value(&self.root(), REPORT);
        edit(&mut report);
        self.rewrite_report_raw(&bytes(&report));
    }
    fn rewrite_ownership(&mut self, edit: impl FnOnce(&mut Value)) {
        let mut ownership = read_value(&self.root(), OWNERSHIP);
        edit(&mut ownership);
        let raw = bytes(&ownership);
        put(&self.root(), OWNERSHIP, &raw);
        self.rewrite_terminal(|terminal| terminal["ownership"]["sha256"] = json!(sha256(&raw)));
    }
    fn seal(&mut self, mut draft: Draft) {
        let root = self.root();
        let source_receipt = bytes(
            &json!({"schema_version":1,"source":"synthetic","files":{SOURCE:sha256(SOURCE_BYTES)}}),
        );
        let source_provenance = bytes(&json!({
            "sidecar": RECEIPT_SIDECAR,
            "source": "synthetic-portable-test",
            "receipt_sha256": sha256(&source_receipt),
        }));
        let base = BTreeMap::from([
            (SOURCE.to_owned(), SOURCE_BYTES.to_vec()),
            (POLICY.to_owned(), POLICY_BYTES.to_vec()),
            (PRODUCER_HELPER.to_owned(), PRODUCER_HELPER_BYTES.to_vec()),
            (
                BASE_CSS.to_owned(),
                b"div { box-sizing: var(--box-sizing); direction: var(--direction); }\n".to_vec(),
            ),
            (
                "browser-owner.toml".to_owned(),
                b"[browser]\nsource = \"synthetic-portable-test\"\nversion = \"1\"\n".to_vec(),
            ),
            (RECEIPT.to_owned(), source_receipt),
            (PROVENANCE.to_owned(), source_provenance),
            (SELECTED_ORIGINALS.to_owned(), b"[]".to_vec()),
        ]);
        for (path, contents) in &base {
            put(&root, path, contents);
        }
        let mut record = json!({"source":SOURCE,"source_sha256":sha256(SOURCE_BYTES),
            "source_kind":"authored","historical_sha256":{},"relation":null,
            "expected":{"kind":"generated","metadata":{"context":"absolute-local",
                "root_writing_mode":"horizontal-tb","hypothetical_display":"flex",
                "containing_direction":"ltr","hypothetical_direction":"ltr",
                "root_direction":"variant","inline_offset":0.0,"root_size":null}},"semantic_specifications":{}});
        let record_sha256 = semantic_record_digest(&record).unwrap();
        let settings = BrowserSettingsWitness::from_official_json(SETTINGS.as_bytes()).unwrap();
        let excluded: BTreeSet<_> = VARIANTS.iter().map(|variant| spec_path(variant)).collect();
        let mut inputs: BTreeMap<String, String> = base
            .iter()
            .map(|(path, data)| (path.clone(), sha256(data)))
            .collect();
        // Spec/declaration hashes are excluded from identity by the declared
        // self-reference rule. Final identity is recomputed below using actual
        // read-back byte hashes and the actual final report mappings.
        inputs.insert("corpus.toml".into(), sha256(b"unsealed declaration"));
        for path in &excluded {
            inputs.insert(path.clone(), sha256(b"unsealed spec"));
        }
        let mut artifacts = Vec::new();
        let mut descriptors = Vec::new();
        for (index, variant) in VARIANTS.iter().enumerate() {
            let output = output_path(variant);
            put(&root, &output, draft.xml[index].as_bytes());
            let digest = sha256(draft.xml[index].as_bytes());
            let descriptor = CaseDescriptor {
                name: case_name(variant),
                source: SOURCE.into(),
                variant: (*variant).into(),
                output: PathDigest {
                    path: output.clone(),
                    sha256: digest.clone(),
                },
                record_sha256: record_sha256.clone(),
            };
            let artifact = json!({"name":case_name(variant),"source":SOURCE,"output":output,"variant":variant,
                "source_sha256":sha256(SOURCE_BYTES),"linked_resources":{BASE_CSS:sha256(&base[BASE_CSS])},
                "job_sha256":sha256(format!("synthetic job {variant}").as_bytes()),"output_sha256":digest});
            let verified = identity_from_verified_inputs(IdentityInputs {
                metadata: &serde_json::from_value(metadata(&inputs)).unwrap(),
                artifact: &serde_json::from_value(artifact.clone()).unwrap(),
                case: &descriptor,
                full_inputs: &inputs,
                excluded_specs: &excluded,
                actual_xml_sha256: &digest,
                historical_sha256: None,
                policy_sha256: &inputs[POLICY],
                browser_settings: &settings,
                observation_request: "geometry",
                observation_input_sha256: None,
            })
            .unwrap();
            draft.specs[index]["identities"]["published"] =
                serde_json::to_value(verified.projection()).unwrap();
            let spec_raw = bytes(&draft.specs[index]);
            let spec = spec_path(variant);
            put(&root, &spec, &spec_raw);
            record["semantic_specifications"][*variant] =
                json!({"path":spec,"sha256":sha256(&spec_raw)});
            artifacts.push(artifact);
            descriptors.push(descriptor);
        }
        put(&root, "corpus.toml", &declaration_bytes(&record));
        let retained: BTreeMap<String, String> = inputs
            .keys()
            .map(|path| (path.clone(), sha256(&fs::read(root.join(path)).unwrap())))
            .collect();
        let report = json!({"metadata":metadata(&retained),"filter":null,
            "summary":{"generated":4,"unsupported":0,"expected_fail":0,"quarantined":0,"failed_to_generate":0},
            "generated":artifacts,"unsupported":[],"expected_fail":[],"quarantined":[],"failed_to_generate":[]});
        let report_raw = bytes(&report);
        put(&root, REPORT, &report_raw);
        let actual_report: Report = parse_json(&fs::read(root.join(REPORT)).unwrap()).unwrap();
        let mut cases = Vec::new();
        for (index, variant) in VARIANTS.iter().enumerate() {
            let xml_digest = sha256(&fs::read(root.join(output_path(variant))).unwrap());
            let verified = identity_from_verified_inputs(IdentityInputs {
                metadata: &actual_report.metadata,
                artifact: &actual_report.generated[index],
                case: &descriptors[index],
                full_inputs: &retained,
                excluded_specs: &excluded,
                actual_xml_sha256: &xml_digest,
                historical_sha256: None,
                policy_sha256: &retained[POLICY],
                browser_settings: &settings,
                observation_request: "geometry",
                observation_input_sha256: None,
            })
            .unwrap();
            let projection = serde_json::to_value(verified.projection()).unwrap();
            assert_eq!(
                projection, draft.specs[index]["identities"]["published"],
                "the fixture's final actual-byte identity must equal its separately sealed semantic declaration"
            );
            cases.push(json!({"case":descriptors[index],
                "spec":{"path":spec_path(variant),"sha256":retained[&spec_path(variant)]},
                "policy":{"path":POLICY,"sha256":retained[POLICY]},"historical":null,
                "requirements":authored_requirements(variant),"identity":projection}));
        }
        let mut owned_outputs = BTreeMap::from([(REPORT.to_owned(), sha256(&report_raw))]);
        for descriptor in &descriptors {
            owned_outputs.insert(
                descriptor.output.path.clone(),
                descriptor.output.sha256.clone(),
            );
        }
        let ownership_raw = bytes(&json!({"schema_version":1,"artifacts":owned_outputs}));
        put(&root, OWNERSHIP, &ownership_raw);
        // Independently authored owner mapping, rather than the production
        // current_owner_bindings helper. Every expected owner file exists and
        // its digest comes from the actual bytes written into this temp owner.
        let mut owner_files = BTreeMap::from([
            (OWNER_SOURCE.to_owned(), SOURCE_BYTES.to_vec()),
            (POLICY.to_owned(), POLICY_BYTES.to_vec()),
            (OWNER_GLOBAL.to_owned(), base[BASE_CSS].clone()),
            (VALIDATOR_HELPER.to_owned(), VALIDATOR_HELPER_BYTES.to_vec()),
            (OWNER_BROWSER.to_owned(), base["browser-owner.toml"].clone()),
            (OWNER_RECEIPT.to_owned(), base[RECEIPT].clone()),
        ]);
        for variant in VARIANTS {
            owner_files.insert(
                spec_path(variant),
                fs::read(root.join(spec_path(variant))).unwrap(),
            );
        }
        let owner_pins: BTreeMap<_, _> = owner_files
            .iter()
            .map(|(path, data)| {
                put(&self.owner, path, data);
                (
                    path.clone(),
                    sha256(&fs::read(self.owner.join(path)).unwrap()),
                )
            })
            .collect();
        put(
            &self.owner,
            OWNER_DECLARATION,
            &fs::read(root.join("corpus.toml")).unwrap(),
        );
        self.terminal = json!({"schema_version":3,"edition":"canonical-box-differences",
            "capture_validation":{
                "producer_profile":"root-context-v4",
                "producer_helper_sha256":sha256(PRODUCER_HELPER_BYTES),
                "validator_profile":"root-context-fixed-orthogonal-v1",
                "validator_helper_sha256":sha256(VALIDATOR_HELPER_BYTES),
            },
            "declaration":{"path":"corpus.toml","sha256":retained["corpus.toml"]},
            "records":[{"source":SOURCE,"sha256":record_sha256,"value":record}],
            "browser_settings":settings,"report":{"path":REPORT,"sha256":sha256(&report_raw)},
            "ownership":{"path":OWNERSHIP,"sha256":sha256(&ownership_raw)},"cases":cases,
            "retained_inputs":retained,"current_owner_inputs":owner_pins});
        self.anchor_terminal_bytes(&terminal_bytes(&self.terminal));
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.owner).expect("remove exactly this test's owned fixture");
    }
}
fn admission_error(fixture: &Fixture) -> QualificationError {
    match fixture.admit() {
        Ok(_) => panic!("tampered edition was admitted"),
        Err(error) => error,
    }
}
fn expect_changed(fixture: &Fixture) {
    assert!(matches!(
        admission_error(fixture),
        QualificationError::IdentityChanged { .. }
    ));
}
fn expect_malformed(fixture: &Fixture) {
    assert!(matches!(
        admission_error(fixture),
        QualificationError::MalformedDeclaration { .. }
    ));
}
fn assert_intentional(case: &CaseOutcome) {
    assert_eq!(
        case.raw.as_ref().unwrap().len(),
        1,
        "{} raw width mismatch must remain",
        case.output
    );
    assert_eq!(
        case.qualified.as_ref().unwrap(),
        &QualifiedComparison {
            raw: RawVerdict::Different { fields: 1 },
            qualified: QualifiedVerdict::Intentional {
                policy: "portable-fixed-box".into(),
                edition: "1".into()
            },
        }
    );
}

#[test]
fn complete_registered_edition_qualifies_all_four_variants_without_hiding_raw_differences() {
    let fixture = Fixture::new();
    let edition = fixture.admit().unwrap();
    assert_eq!(
        edition.outputs().collect::<BTreeSet<_>>(),
        fixture.all().iter().map(String::as_str).collect()
    );
    let run = edition.execute_selected(&fixture.all()).unwrap();
    assert_eq!(run.declared_cases, 4);
    assert_eq!(run.cases.len(), 4);
    assert!(run.freshness_error.is_none());
    for case in &run.cases {
        assert_intentional(case);
    }
    assert!(run.selected_cases_passed());
    assert!(run.current_coverage_complete());
}
#[test]
fn selected_subset_and_empty_selection_do_not_claim_full_current_coverage() {
    let fixture = Fixture::new();
    let edition = fixture.admit().unwrap();
    for selected in [BTreeSet::from([output_path(VARIANTS[2])]), BTreeSet::new()] {
        let run = edition.execute_selected(&selected).unwrap();
        assert_eq!(
            run.cases
                .iter()
                .map(|case| case.output.clone())
                .collect::<BTreeSet<_>>(),
            selected
        );
        assert_eq!(run.declared_cases, 4);
        assert!(run.selected_cases_passed());
        assert!(!run.current_coverage_complete());
        for case in &run.cases {
            assert_intentional(case);
        }
    }
    assert!(matches!(
        edition.execute_selected(&BTreeSet::from(["xml/unregistered.xml".into()])),
        Err(QualificationError::IdentityChanged { .. })
    ));
}
#[test]
fn required_edition_absence_and_missing_terminal_are_admission_failures() {
    for entire_edition in [false, true] {
        let fixture = Fixture::new();
        if entire_edition {
            fs::remove_dir_all(fixture.root()).unwrap();
        } else {
            fs::remove_file(fixture.root().join(authority::TERMINAL)).unwrap();
        }
        expect_changed(&fixture);
    }
}
#[test]
fn external_terminal_anchor_rejects_even_semantically_identical_changed_bytes() {
    let fixture = Fixture::new();
    let terminal = fs::read(fixture.root().join(authority::TERMINAL)).unwrap();
    let mut changed = terminal.clone();
    changed.push(b' ');
    put(&fixture.root(), authority::TERMINAL, &changed);
    expect_changed(&fixture);
    put(&fixture.root(), authority::TERMINAL, &terminal);
    assert!(matches!(
        admit(
            &fixture.owner,
            Registration {
                edition: Edition::Exact(PublishedEdition::CanonicalBoxes),
                terminal_sha256: &"0".repeat(64)
            }
        ),
        Err(QualificationError::IdentityChanged { .. })
    ));
}
#[test]
fn every_retained_output_and_control_artifact_is_byte_pinned_and_required() {
    let paths = [
        "corpus.toml".to_owned(),
        SOURCE.into(),
        POLICY.into(),
        BASE_CSS.into(),
        PRODUCER_HELPER.into(),
        "browser-owner.toml".into(),
        RECEIPT.into(),
        PROVENANCE.into(),
        SELECTED_ORIGINALS.into(),
        spec_path(VARIANTS[0]),
        output_path(VARIANTS[1]),
        REPORT.into(),
        OWNERSHIP.into(),
    ];
    for path in paths {
        for remove in [false, true] {
            let fixture = Fixture::new();
            if remove {
                fs::remove_file(fixture.root().join(&path)).unwrap();
            } else {
                let mut raw = fs::read(fixture.root().join(&path)).unwrap();
                raw.push(b' ');
                put(&fixture.root(), &path, &raw);
            }
            expect_changed(&fixture);
        }
    }
}
#[test]
fn adding_unrelated_grid_registrations_preserves_the_qualified_box_cohort() {
    let fixture = Fixture::new();
    let selected = fixture.all();
    let before = fixture
        .admit()
        .unwrap()
        .execute_selected(&selected)
        .unwrap();
    assert!(before.current_coverage_complete());

    let mut live = fs::read(fixture.owner.join(OWNER_DECLARATION)).unwrap();
    let mut grid = format!(
        r#"

[[records]]
source = "html/grid/added-grid.html"
source_sha256 = {:?}
source_kind = "authored"
[records.expected]
kind = "generated"
[records.expected.metadata]
context = "absolute-local"
root_writing_mode = "horizontal-tb"
hypothetical_display = "grid"
containing_direction = "ltr"
hypothetical_direction = "ltr"
root_direction = "variant"
inline_offset = 0
"#,
        sha256(b"unrelated Grid source")
    );
    for variant in VARIANTS {
        grid.push_str(&format!(
            "\n[records.semantic_specifications.{variant}]\npath = \"semantic/added-grid-{variant}.json\"\nsha256 = {:?}\n",
            sha256(variant.as_bytes())
        ));
    }
    live.extend_from_slice(grid.as_bytes());
    put(&fixture.owner, OWNER_DECLARATION, &live);

    let after = fixture
        .admit()
        .unwrap()
        .execute_selected(&selected)
        .unwrap();
    assert!(after.registered_coverage_complete());
    assert!(!after.current_coverage_complete());
    assert_eq!(after.cases.len(), before.cases.len());
    for (current, original) in after.cases.iter().zip(&before.cases) {
        assert_eq!(current.output, original.output);
        assert_eq!(current.raw, original.raw);
        assert_intentional(current);
    }
}

#[test]
fn valid_catalogue_edits_to_the_selected_record_or_spec_registration_refuse() {
    for (original, replacement) in [
        ("inline_offset = 0", "inline_offset = 1"),
        (
            "semantic/portable-empty-border_box_ltr.json",
            "semantic/moved-border_box_ltr.json",
        ),
    ] {
        let fixture = Fixture::new();
        let live = fs::read_to_string(fixture.owner.join(OWNER_DECLARATION)).unwrap();
        assert!(live.contains(original));
        put(
            &fixture.owner,
            OWNER_DECLARATION,
            live.replacen(original, replacement, 1).as_bytes(),
        );
        assert!(matches!(
            admission_error(&fixture),
            QualificationError::IdentityChanged { component }
                if component.contains("selected live record changed")
        ));
    }
}

#[test]
fn live_catalogue_rejects_unknown_fields_and_colliding_case_names() {
    let fixture = Fixture::new();
    let live = fs::read_to_string(fixture.owner.join(OWNER_DECLARATION)).unwrap();
    let other = live.split_once("[[records]]").unwrap().1.replacen(
        SOURCE,
        "html/grid/portable-empty.html",
        1,
    );
    for (candidate, message) in [
        (
            live.replacen("schema_version = 3", "schema_version = 3\nextra = 0", 1),
            "live catalogue",
        ),
        (
            format!("{live}\n[[records]]{other}"),
            "duplicate qualification case stem",
        ),
    ] {
        put(&fixture.owner, OWNER_DECLARATION, candidate.as_bytes());
        let error = admission_error(&fixture);
        assert!(
            matches!(&error, QualificationError::IdentityChanged { component } if component.contains(message)),
            "{error}"
        );
    }
}

#[test]
fn live_catalogue_rejects_invalid_history_specification_and_peer_bindings() {
    let fixture = Fixture::new();
    let live = fs::read_to_string(fixture.owner.join(OWNER_DECLARATION)).unwrap();
    let other = live
        .split_once("[[records]]")
        .unwrap()
        .1
        .replace(SOURCE, "html/grid/another-grid.html");
    let different_specs = other.replace("semantic/portable-empty-", "semantic/another-grid-");
    let history = different_specs.replacen(
        "[records.historical_sha256]",
        &format!(
            "[records.historical_sha256]\nbogus = {:?}",
            sha256(b"unrelated history")
        ),
        1,
    );
    let missing_peer = different_specs.replacen(
        "source_kind = \"authored\"",
        "source_kind = \"authored\"\nrelation = { peer = \"missing-peer\", world_phase_change = false }",
        1,
    );
    for (record, message) in [
        (
            history,
            "authored source cannot declare historical variants",
        ),
        (other, "duplicate semantic specification path"),
        (
            missing_peer,
            "paired source requires a declared Generated peer",
        ),
    ] {
        put(
            &fixture.owner,
            OWNER_DECLARATION,
            format!("{live}\n[[records]]{record}").as_bytes(),
        );
        let error = admission_error(&fixture);
        assert!(
            matches!(&error, QualificationError::IdentityChanged { component } if component.contains(message)),
            "{error}"
        );
    }
}

#[test]
fn terminal_registration_cannot_replace_the_registration_in_retained_bytes() {
    let mut fixture = Fixture::new();
    fixture.rewrite_terminal(|terminal| {
        terminal["records"][0]["value"]["semantic_specifications"][VARIANTS[0]]["path"] =
            json!("semantic/moved-border_box_ltr.json");
    });
    assert!(matches!(
        admission_error(&fixture),
        QualificationError::IdentityChanged { component }
            if component.contains("retained record projection")
    ));
}

#[test]
fn current_owner_byte_drift_or_missing_owner_refuses_unchanged_retained_artifacts() {
    for path in [
        OWNER_DECLARATION.to_owned(),
        OWNER_SOURCE.to_owned(),
        POLICY.to_owned(),
        OWNER_GLOBAL.to_owned(),
        VALIDATOR_HELPER.to_owned(),
        OWNER_BROWSER.to_owned(),
        OWNER_RECEIPT.to_owned(),
        spec_path(VARIANTS[0]),
    ] {
        for remove in [false, true] {
            let fixture = Fixture::new();
            if remove {
                fs::remove_file(fixture.owner.join(&path)).unwrap();
            } else {
                put(
                    &fixture.owner,
                    &path,
                    b"changed current owning source bytes",
                );
            }
            expect_changed(&fixture);
        }
    }
}
#[test]
fn artifact_census_rejects_extra_files_but_ignores_only_the_runtime_control_namespace() {
    for path in [
        "unregistered.txt",
        "xml/qualification/unregistered.xml",
        "semantic/unregistered.json",
        "nested/.surgeist-generator/unregistered",
    ] {
        let fixture = Fixture::new();
        put(&fixture.root(), path, b"extra static artifact");
        expect_changed(&fixture);
    }
    let fixture = Fixture::new();
    put(
        &fixture.root(),
        ".surgeist-generator/lock",
        b"owned runtime control",
    );
    assert!(fixture.admit().is_ok());
}
#[test]
fn closed_terminal_wire_rejects_duplicate_unknown_missing_and_wrong_nullable_fields() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new();
        let mut terminal = fixture.terminal.clone();
        match mutation {
            0 => {
                terminal["unregistered"] = json!(true);
            }
            1 => {
                terminal.as_object_mut().unwrap().remove("browser_settings");
            }
            2 => {
                terminal["cases"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("historical");
            }
            3 => {
                terminal["browser_settings"]["launch"]["arguments"] = json!([1]);
            }
            _ => {
                terminal["schema_version"] = json!(99);
            }
        }
        fixture.anchor_terminal_bytes(&bytes(&terminal));
        expect_malformed(&fixture);
    }
    let mut fixture = Fixture::new();
    let raw = String::from_utf8(bytes(&fixture.terminal))
        .unwrap()
        .replacen(
            "\"edition\":",
            "\"edition\":\"canonical-box-differences\",\"edition\":",
            1,
        );
    fixture.anchor_terminal_bytes(raw.as_bytes());
    expect_malformed(&fixture);
}
#[test]
fn closed_report_wire_rejects_duplicate_unknown_and_absent_explicit_filter() {
    for mutation in 0..3 {
        let mut fixture = Fixture::new();
        let mut report = read_value(&fixture.root(), REPORT);
        if mutation == 0 {
            report["unregistered"] = json!(0);
        }
        if mutation == 1 {
            report.as_object_mut().unwrap().remove("filter");
        }
        let raw = if mutation == 2 {
            String::from_utf8(bytes(&report))
                .unwrap()
                .replacen("\"filter\":", "\"filter\":null,\"filter\":", 1)
                .into_bytes()
        } else {
            bytes(&report)
        };
        fixture.rewrite_report_raw(&raw);
        expect_malformed(&fixture);
    }
}
#[test]
fn partial_or_non_generated_report_dispositions_cannot_admit_a_current_edition() {
    for mutation in 0..7 {
        let mut fixture = Fixture::new();
        fixture.rewrite_report(|report| match mutation {
            0 => report["filter"] = json!("portable"),
            1 => report["summary"]["generated"] = json!(3),
            2 => {
                report["generated"].as_array_mut().unwrap().pop();
            }
            3 => report["unsupported"] = json!([{"source":"html/foreign.html"}]),
            4 => report["summary"]["expected_fail"] = json!(1),
            5 => report["quarantined"] = json!([{}]),
            _ => report["failed_to_generate"] = json!([{}]),
        });
        expect_changed(&fixture);
    }
}
#[test]
fn report_input_and_generated_mapping_drift_refuse_identical_geometry() {
    for mutation in 0..6 {
        let mut fixture = Fixture::new();
        fixture.rewrite_report(|report| match mutation {
            0 => report["metadata"]["inputs"][BASE_CSS] = json!(sha256(b"other input")),
            1 => report["generated"][0]["source"] = json!("html/foreign.html"),
            2 => report["generated"][0]["variant"] = json!(VARIANTS[1]),
            3 => report["generated"][0]["source_sha256"] = json!(sha256(b"foreign source")),
            4 => report["generated"][0]["name"] = json!("foreign-case"),
            _ => report["generated"][0]["linked_resources"] = json!({}),
        });
        expect_changed(&fixture);
    }
}
#[test]
fn duplicate_report_cases_and_incomplete_or_extra_ownership_are_rejected() {
    let mut fixture = Fixture::new();
    fixture.rewrite_report(|report| {
        report["generated"][1] = report["generated"][0].clone();
    });
    expect_malformed(&fixture);
    for mutation in 0..4 {
        let mut fixture = Fixture::new();
        fixture.rewrite_ownership(|ownership| match mutation {
            0 => {
                ownership["artifacts"]
                    .as_object_mut()
                    .unwrap()
                    .remove(&output_path(VARIANTS[0]));
            }
            1 => ownership["artifacts"]["xml/unregistered.xml"] = json!(sha256(b"extra")),
            2 => ownership["schema_version"] = json!(2),
            _ => ownership["artifacts"][&output_path(VARIANTS[0])] = json!(sha256(b"wrong output")),
        });
        expect_changed(&fixture);
    }
    let mut fixture = Fixture::new();
    fixture.rewrite_ownership(|ownership| ownership["unregistered"] = json!(true));
    expect_malformed(&fixture);
}
#[test]
fn canonical_record_and_per_variant_reference_mappings_are_not_numeric_authority() {
    for mutation in 0..5 {
        let mut fixture = Fixture::new();
        fixture.rewrite_terminal(|terminal| match mutation {
            0 => {
                terminal["records"][0]["value"]["semantic_specifications"][VARIANTS[0]]["path"] =
                    json!(spec_path(VARIANTS[1]))
            }
            1 => {
                terminal["records"][0]["value"]["semantic_specifications"]
                    .as_object_mut()
                    .unwrap()
                    .remove(VARIANTS[0]);
            }
            2 => {
                terminal["records"][0]["value"]["semantic_specifications"]["unknown_variant"] =
                    terminal["records"][0]["value"]["semantic_specifications"][VARIANTS[0]].clone()
            }
            3 => {
                terminal["records"][0]["value"]["semantic_specifications"][VARIANTS[0]]["sha256"] =
                    json!(sha256(b"wrong semantic bytes"))
            }
            _ => terminal["cases"][0]["case"]["variant"] = json!(VARIANTS[1]),
        });
        assert!(fixture.admit().is_err(), "mapping mutation {mutation}");
    }
    let mut fixture = Fixture::new();
    fixture.rewrite_terminal(|terminal| {
        terminal["records"][0]["value"]["expected"]["metadata"]["inline_offset"] = json!(1)
    });
    expect_malformed(&fixture);
}
#[test]
fn stored_projection_tampering_cannot_replace_recomputed_actual_input_authority() {
    for field in [
        "generator",
        "source_sha256",
        "profile_sha256",
        "source_imports_sha256",
        "policy_source_sha256",
    ] {
        let mut fixture = Fixture::new();
        fixture.rewrite_terminal(|terminal| {
            terminal["cases"][0]["identity"][field] = json!(sha256(b"unverified stored identity"))
        });
        assert!(
            matches!(admission_error(&fixture),QualificationError::IdentityChanged {component} if component == "stored identity differs from authenticated actual inputs"),
            "{field}"
        );
    }
    let mut fixture = Fixture::new();
    fixture.rewrite_terminal(|terminal| {
        terminal["cases"][0]["identity"]["inputs"][BASE_CSS] = json!(sha256(b"wrong stored input"))
    });
    expect_changed(&fixture);
}
#[test]
fn reissued_wrong_or_missing_current_selector_and_packet_requirements_refuse_before_execution() {
    for key in ["selectors", "nodes", "unrounded_packets", "final_packets"] {
        let fixture =
            Fixture::issue(|draft| draft.specs[0]["representation"]["current"][key] = json!([]));
        assert!(
            matches!(
                admission_error(&fixture),
                QualificationError::IncompleteRepresentation { .. }
            ),
            "{key}"
        );
    }
    for key in ["case_id", "output"] {
        let fixture = Fixture::issue(|draft| {
            draft.specs[0]["representation"]["current"][key] = json!("xml/foreign.xml")
        });
        assert!(
            matches!(
                admission_error(&fixture),
                QualificationError::IncompleteRepresentation { .. }
            ),
            "{key}"
        );
    }
    let fixture = Fixture::issue(|draft| {
        draft.specs[0]["representation"]
            .as_object_mut()
            .unwrap()
            .remove("current");
    });
    assert!(matches!(
        admission_error(&fixture),
        QualificationError::IncompleteRepresentation { .. }
    ));
    for packet in ["unrounded", "final_output"] {
        let fixture = Fixture::issue(|draft| {
            draft.specs[0]["native"]["nodes"][0][packet] = json!({"state":"absent"})
        });
        assert!(
            matches!(
                admission_error(&fixture),
                QualificationError::IncompleteNativeOracle { .. }
            ),
            "{packet}"
        );
    }
}
#[test]
fn reissued_equal_numbers_in_the_wrong_frame_or_with_a_missing_requested_field_refuse_admission() {
    let fixture = Fixture::issue(|draft| {
        draft.xml[0] = draft.xml[0]
            .replace("observation-frame=\"root-border-local\"","observation-frame=\"root-border\"")
            .replace("root-context=\"absolute-root-local\" containing-width=\"800px\" containing-height=\"600px\" containing-direction=\"ltr\" hypothetical-direction=\"ltr\" hypothetical-display=\"flex\" hypothetical-anchor=\"inline-start\" hypothetical-inline-offset=\"0px\"", "root-context=\"block-flow-auto\" parent-inline-extent=\"800px\" parent-direction=\"ltr\"")
            .replace("position=\"absolute\"","position=\"relative\"");
    });
    assert!(matches!(
        admission_error(&fixture),
        QualificationError::IncompleteRepresentation { .. }
    ));
    let fixture = Fixture::issue(|draft| {
        draft.xml[0] = draft.xml[0].replace("<node x=\"0\" y=\"0\"", "<node x=\"0\"")
    });
    assert!(matches!(
        admission_error(&fixture),
        QualificationError::IncompleteRepresentation { .. }
    ));
}
#[test]
fn reissued_semantic_spec_rejects_unknown_duplicate_nodes_and_missing_nullable_native_facts() {
    let fixture = Fixture::issue(|draft| draft.specs[0]["unknown"] = json!(true));
    expect_malformed(&fixture);
    let fixture = Fixture::issue(|draft| {
        let node = draft.specs[0]["native"]["nodes"][0].clone();
        draft.specs[0]["native"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node);
    });
    assert!(matches!(
        admission_error(&fixture),
        QualificationError::DuplicateKey { .. }
    ));
    let fixture = Fixture::issue(|draft| {
        draft.specs[0]["native"]["nodes"][0]["final_output"]["facts"]["scroll"]["facts"]
            .as_object_mut()
            .unwrap()
            .remove("clip_y");
    });
    expect_malformed(&fixture);
}
#[test]
fn reissued_oracle_failure_keeps_raw_comparison_and_executes_every_later_case() {
    let fixture = Fixture::issue(|draft| {
        draft.specs[0]["native"]["nodes"][0]["unrounded"]["facts"]["content_size"]["width"] =
            json!(19)
    });
    let run = fixture
        .admit()
        .unwrap()
        .execute_selected(&fixture.all())
        .unwrap();
    assert_eq!(run.cases.len(), 4);
    assert_eq!(run.cases[0].raw.as_ref().unwrap().len(), 1);
    assert!(
        run.cases[0]
            .qualified
            .as_ref()
            .unwrap_err()
            .contains("content_size.width")
    );
    for case in &run.cases[1..] {
        assert_intentional(case);
    }
    assert!(!run.selected_cases_passed());
    assert!(!run.current_coverage_complete());
    assert!(
        run.freshness_error.is_none(),
        "failure must reach the semantic stage under reissued byte authority"
    );
}
#[test]
fn reissued_source_oracle_disagreement_and_unexpected_browser_pass_are_semantic_failures() {
    for unexpected_pass in [false, true] {
        let fixture = Fixture::issue(|draft| {
            if unexpected_pass {
                draft.xml[0] = draft.xml[0].replace("width=\"21\"", "width=\"20\"");
            } else {
                draft.xml[0] = draft.xml[0].replace("width=\"20px\"", "width=\"19px\"");
            }
        });
        let run = fixture
            .admit()
            .unwrap()
            .execute_selected(&fixture.all())
            .unwrap();
        assert_eq!(run.cases.len(), 4);
        assert!(run.freshness_error.is_none());
        assert_eq!(
            run.cases[0].raw.as_ref().unwrap().len(),
            usize::from(!unexpected_pass)
        );
        let semantic_error = run.cases[0].qualified.as_ref().unwrap_err();
        if unexpected_pass {
            assert_eq!(semantic_error, "UnexpectedPass");
        } else {
            assert!(
                semantic_error.starts_with("NativeOracleChanged"),
                "{semantic_error}"
            );
        }
        for case in &run.cases[1..] {
            assert_intentional(case);
        }
        assert!(!run.selected_cases_passed());
        assert!(!run.current_coverage_complete());
    }
}
#[test]
fn stale_after_admission_invalidates_every_qualified_outcome_without_erasing_raw_results() {
    let fixture = Fixture::new();
    let edition = fixture.admit().unwrap();
    put(
        &fixture.owner,
        OWNER_GLOBAL,
        b"owner changed after admission",
    );
    let run = edition.execute_selected(&fixture.all()).unwrap();
    assert_eq!(run.cases.len(), 4);
    assert!(run.freshness_error.is_some());
    for case in &run.cases {
        assert_eq!(case.raw.as_ref().unwrap().len(), 1);
        assert_eq!(
            case.qualified.as_ref().unwrap_err(),
            run.freshness_error.as_ref().unwrap()
        );
    }
    assert!(!run.selected_cases_passed());
    assert!(!run.current_coverage_complete());
}
#[test]
fn final_freshness_recheck_rejects_a_late_change_without_returning_a_qualified_prefix() {
    let fixture = Fixture::new();
    let edition = fixture.admit().unwrap();
    let run = edition
        .execute_selected_before_freshness(&fixture.all(), || {
            put(
                &fixture.root(),
                POLICY,
                b"late changed policy after real Native comparisons",
            );
        })
        .unwrap();
    assert_eq!(run.cases.len(), 4);
    let error = run.freshness_error.as_ref().unwrap();
    for case in &run.cases {
        assert_eq!(case.raw.as_ref().unwrap().len(), 1);
        assert_eq!(case.qualified.as_ref().unwrap_err(), error);
    }
    assert!(!run.selected_cases_passed());
    assert!(!run.current_coverage_complete());
}

#[test]
fn a_later_case_with_a_wrong_complete_requirement_refuses_the_entire_edition() {
    let fixture = Fixture::issue(|draft| {
        draft.specs[3]["representation"]["current"]["selectors"][0]["path"] = json!("/0");
    });
    assert!(matches!(
        admission_error(&fixture),
        QualificationError::IncompleteRepresentation { .. }
    ));
    for packet in ["unrounded_packets", "final_packets"] {
        let fixture = Fixture::issue(|draft| {
            draft.specs[2]["representation"]["current"][packet] = json!(["/0"])
        });
        assert!(matches!(
            admission_error(&fixture),
            QualificationError::IncompleteRepresentation { .. }
        ));
    }
}
#[test]
fn missing_frame_and_wrong_frame_declarations_are_malformed_despite_equal_geometry() {
    for missing in [false, true] {
        let fixture = Fixture::issue(|draft| {
            if missing {
                draft.specs[1]["representation"]["current"]
                    .as_object_mut()
                    .unwrap()
                    .remove("frame");
            } else {
                draft.specs[1]["representation"]["current"]["frame"] = json!("root-border");
            }
        });
        expect_malformed(&fixture);
    }
}
#[test]
fn missing_records_and_inconsistent_prebound_terminal_requirements_refuse_admission() {
    let mut fixture = Fixture::new();
    fixture.rewrite_terminal(|terminal| terminal["records"] = json!([]));
    expect_malformed(&fixture);
    let mut fixture = Fixture::new();
    fixture.rewrite_terminal(|terminal| {
        terminal["cases"][2]["requirements"]["final_packets"] = json!([])
    });
    expect_changed(&fixture);
}
#[test]
fn reissued_extra_or_changed_raw_difference_cannot_be_qualified_as_the_original_pair() {
    for extra_field in [false, true] {
        let fixture = Fixture::issue(|draft| {
            if extra_field {
                draft.xml[0] = draft.xml[0].replace(
                    "<node x=\"0\" y=\"0\" width=\"21\" height=\"10\"",
                    "<node x=\"0\" y=\"0\" width=\"21\" height=\"11\"",
                );
            } else {
                draft.specs[0]["differences"][0]["browser"] = json!(21.0625);
            }
        });
        let run = fixture
            .admit()
            .unwrap()
            .execute_selected(&fixture.all())
            .unwrap();
        assert!(run.freshness_error.is_none());
        assert_eq!(run.cases.len(), 4);
        assert_eq!(
            run.cases[0].raw.as_ref().unwrap().len(),
            if extra_field { 2 } else { 1 }
        );
        let semantic_error = run.cases[0].qualified.as_ref().unwrap_err();
        assert!(
            semantic_error.starts_with("DifferenceSetChanged"),
            "{semantic_error}"
        );
        assert!(semantic_error.contains(if extra_field {
            "layout:/:Height"
        } else {
            "layout:/:Width"
        }));
        for case in &run.cases[1..] {
            assert_intentional(case);
        }
        assert!(!run.current_coverage_complete());
    }
}

#[test]
fn reissued_omission_of_source_spec_policy_global_or_receipt_owner_pins_is_refused() {
    for path in [
        OWNER_SOURCE.to_owned(),
        spec_path(VARIANTS[1]),
        POLICY.to_owned(),
        OWNER_GLOBAL.to_owned(),
        OWNER_BROWSER.to_owned(),
        OWNER_RECEIPT.to_owned(),
    ] {
        let mut fixture = Fixture::new();
        fixture.rewrite_terminal(|terminal| {
            terminal["current_owner_inputs"]
                .as_object_mut()
                .unwrap()
                .remove(&path);
        });
        expect_changed(&fixture);
    }
}

#[test]
fn registered_relative_insets_preserve_all_four_exact_variants() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry = admit_registered(owner).unwrap();
    let prefix = "tests/layout/browser_parity/captured/relative-inset-differences/";
    let selected: BTreeSet<_> = registry
        .outputs()
        .filter(|path| path.starts_with(prefix))
        .collect();
    let expected = VARIANTS
        .map(|variant| format!("{prefix}xml/block/block_inset_percentage__{variant}.xml"))
        .into_iter()
        .collect();
    assert_eq!(selected, expected);
    let run = registry.execute_selected(&selected).unwrap();
    assert!(run.selected_cases_passed());
    assert_eq!(run.cases.len(), 4);
    for case in run.cases {
        let raw = case.raw.unwrap();
        assert_eq!(raw.len(), 3);
        for (index, (browser, native)) in [(0.0, 1.0), (10.0, 8.0), (20.0, 21.0)]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                raw[index],
                format!(
                    "ComparedField(Layout {{ path: \"/{index}\", field: Y, browser: Number({browser:?}), native: Some(Number({native:?})) }})"
                )
            );
        }
        let verdict = case.qualified.unwrap();
        assert_eq!(verdict.raw, RawVerdict::Different { fields: 3 });
        assert_eq!(
            verdict.qualified,
            QualifiedVerdict::Intentional {
                policy: "relative-insets-final-measured-basis".into(),
                edition: "css-position-3-20251007-native-1".into(),
            }
        );
    }
}

#[test]
fn registered_prepared_range_preserves_raw_differences_and_exact_controls() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry = admit_registered(owner).unwrap();
    let selected: BTreeSet<_> = registry
        .outputs()
        .filter(|path| path.contains("/prepared-range-qualification/"))
        .collect();
    assert_eq!(selected.len(), 4);
    let run = registry.execute_selected(&selected).unwrap();
    assert!(run.selected_cases_passed());
    for case in &run.cases {
        let fields = case.raw.as_ref().unwrap();
        if case.output.ends_with("_rtl.xml") {
            assert_eq!(fields.len(), 1);
            for detail in ["72.0", "24.0", "native_pieces", "aggregate"] {
                assert!(fields[0].contains(detail), "{}", fields[0]);
            }
            assert_eq!(
                case.qualified.as_ref().unwrap().raw,
                RawVerdict::Different { fields: 1 }
            );
        } else {
            assert!(fields.is_empty());
            assert!(case.qualified.is_ok());
        }
    }
}

#[test]
fn ordinary_prepared_range_keeps_a_late_unclassified_difference_in_raw_output() {
    let original = include_str!(
        "captured/prepared-range-qualification/xml/block/prepared_text_continuity_bidi__border_box_rtl.xml"
    );
    let changed = original.replace(
        "start=\"56\" advance=\"16\"",
        "start=\"56.015625\" advance=\"16\"",
    );
    assert_ne!(original, changed);
    let golden = support::Golden::parse(&changed).unwrap();
    let (raw, qualified) = compare_prepared_range(&golden, "border_box_rtl").unwrap();
    assert_eq!(raw.len(), 2);
    assert!(raw[0].contains("browser_start: 72.0"));
    assert!(raw[1].contains("browser_start: 56.015625"));
    assert!(
        qualified
            .unwrap_err()
            .contains("prepared Range comparison changed")
    );
}

#[test]
fn registered_sideways_flex_preserves_both_policies_and_complete_raw_fields() {
    let registry = admit_registered(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let selected = registry
        .outputs()
        .filter(|path| path.contains("/flex-composition-qualification/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(selected.len(), 4);
    let run = registry.execute_selected(&selected).unwrap();
    assert!(run.selected_cases_passed());
    for case in &run.cases {
        let rtl = case.output.ends_with("_rtl.xml");
        let fields = case.raw.as_ref().unwrap();
        assert_eq!(fields.len(), if rtl { 10 } else { 9 });
        assert_eq!(
            fields
                .iter()
                .filter(|field| field.contains("path: \"/3/0\"") && field.contains("field: Y,"))
                .count(),
            usize::from(rtl)
        );
        let QualifiedVerdict::IntentionalFields(assignments) =
            &case.qualified.as_ref().unwrap().qualified
        else {
            panic!("sideways Flex requires policy-specific assignments");
        };
        assert_eq!(assignments.len(), if rtl { 2 } else { 1 });
        assert_eq!(assignments[0].policy, "flexbox-collapse-two-round-struts");
        assert_eq!(assignments[0].fields.len(), 9);
        if rtl {
            assert_eq!(assignments[1].policy, "classic-scrollbar-logical-end");
            assert_eq!(assignments[1].fields.len(), 1);
            assert!(assignments[1].fields[0].contains("path: \"/3/0\""));
        }
    }
}

fn matching_fixture(observed_width: u32) -> Fixture {
    let mut fixture = Fixture::new();
    let old_root = fixture.root();
    let new_root = fixture
        .owner
        .join("tests/layout/browser_parity/captured/synthetic-box-controls");
    fs::rename(&old_root, &new_root).unwrap();
    let mut terminal = fixture.terminal.clone();
    let mut record = terminal["records"][0]["value"].clone();
    let mut declaration = String::from_utf8(declaration_bytes(&record))
        .unwrap()
        .split("\n[records.semantic_specifications.")
        .next()
        .unwrap()
        .to_owned();
    declaration = declaration.replace("source_kind = \"authored\"", "source_kind = \"original\"");
    let mut retained: BTreeMap<String, String> =
        serde_json::from_value(terminal["retained_inputs"].clone()).unwrap();
    let removed: Vec<_> = retained
        .keys()
        .filter(|p| p.starts_with("semantic/") || p.as_str() == POLICY)
        .cloned()
        .collect();
    for path in removed {
        retained.remove(&path);
        fs::remove_file(new_root.join(path)).unwrap();
    }
    let mut owner_pins: BTreeMap<String, String> =
        serde_json::from_value(terminal["current_owner_inputs"].clone()).unwrap();
    owner_pins.retain(|path, _| !path.starts_with("semantic/") && path != POLICY);
    let source = fs::read(fixture.owner.join(OWNER_SOURCE)).unwrap();
    let source_sha256 = sha256(&source);
    assert_eq!(owner_pins.remove(OWNER_SOURCE), Some(source_sha256.clone()));
    fs::remove_file(fixture.owner.join(OWNER_SOURCE)).unwrap();
    put(&fixture.owner, ORIGINAL_OWNER_SOURCE, &source);
    owner_pins.insert(ORIGINAL_OWNER_SOURCE.to_owned(), source_sha256.clone());
    let selected = bytes(&json!([{
        "source": SOURCE,
        "ownership": "imported",
        "sha256": source_sha256,
    }]));
    put(&new_root, SELECTED_ORIGINALS, &selected);
    retained.insert(SELECTED_ORIGINALS.to_owned(), sha256(&selected));
    let mut report = read_value(&new_root, REPORT);
    let mut historical = BTreeMap::new();
    let mut history_lines = String::new();
    for variant in VARIANTS {
        let output = output_path(variant);
        let xml =
            authored_xml(variant).replace("width=\"21\"", &format!("width=\"{observed_width}\""));
        put(&new_root, &output, xml.as_bytes());
        let predecessor = output.replacen("xml/", "historical/", 1);
        let hash = sha256(xml.as_bytes());
        put(&new_root, &predecessor, xml.as_bytes());
        retained.insert(predecessor, hash.clone());
        historical.insert(variant, hash.clone());
        history_lines.push_str(&format!("{variant} = {hash:?}\n"));
        let owner = format!("tests/layout/browser_parity/{output}");
        put(&fixture.owner, &owner, xml.as_bytes());
        owner_pins.insert(owner, hash.clone());
        let artifact = report["generated"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["variant"] == variant)
            .unwrap();
        artifact["output_sha256"] = json!(hash);
    }
    declaration = declaration.replace(
        "[records.historical_sha256]\n",
        &format!("[records.historical_sha256]\n{history_lines}"),
    );
    declaration.push_str(&format!("\n[[matching_publications]]\nedition = \"synthetic-box-controls\"\nsources = [{SOURCE:?}]\n"));
    put(&new_root, "corpus.toml", declaration.as_bytes());
    put(&fixture.owner, OWNER_DECLARATION, declaration.as_bytes());
    retained.insert("corpus.toml".into(), sha256(declaration.as_bytes()));
    record["source_kind"] = json!("original");
    record["historical_sha256"] = json!(historical);
    record["semantic_specifications"] = Value::Null;
    record["matching_publication"] = json!("synthetic-box-controls");
    let record_sha = semantic_record_digest(&record).unwrap();
    report["metadata"]["inputs"] = json!(retained);
    let report_raw = bytes(&report);
    put(&new_root, REPORT, &report_raw);
    let parsed_report: Report = parse_json(&report_raw).unwrap();
    let settings = BrowserSettingsWitness::from_official_json(SETTINGS.as_bytes()).unwrap();
    let mut outputs = BTreeMap::from([(REPORT.to_owned(), sha256(&report_raw))]);
    let cases: Vec<_> = parsed_report.generated.iter().map(|artifact| {
        outputs.insert(artifact.output.clone(),artifact.output_sha256.clone());
        let descriptor = CaseDescriptor { name: artifact.name.clone(), source: SOURCE.into(), variant: artifact.variant.clone(),
            output: PathDigest { path: artifact.output.clone(), sha256: artifact.output_sha256.clone() }, record_sha256: record_sha.clone() };
        let predecessor = artifact.output.replacen("xml/","historical/",1);
        let history = PathDigest { path: predecessor.clone(), sha256: retained[&predecessor].clone() };
        let identity = authority::capture_binding_from_verified_inputs(authority::CaptureBindingInputs {
            metadata: &parsed_report.metadata, artifact, case: &descriptor, full_inputs: &retained, excluded_specs: &BTreeSet::new(),
            actual_xml_sha256: &artifact.output_sha256, historical_sha256: Some(&history.sha256), browser_settings: &settings,
            observation_request: "geometry", observation_input_sha256: None,
        }).unwrap();
        json!({"case":descriptor,"historical":history,"matching":"geometry","identity":identity})
    }).collect();
    let ownership = bytes(&json!({"schema_version":1,"artifacts":outputs}));
    put(&new_root, OWNERSHIP, &ownership);
    terminal["edition"] = json!("synthetic-box-controls");
    terminal["records"] = json!([{"source":SOURCE,"sha256":record_sha,"value":record}]);
    terminal["cases"] = json!(cases);
    terminal["declaration"]["sha256"] = json!(retained["corpus.toml"]);
    terminal["retained_inputs"] = json!(retained);
    terminal["current_owner_inputs"] = json!(owner_pins);
    terminal["report"]["sha256"] = json!(sha256(&report_raw));
    terminal["ownership"]["sha256"] = json!(sha256(&ownership));
    fixture.edition = Edition::Matching("synthetic-box-controls");
    fixture.terminal = terminal;
    fixture.anchor_terminal_bytes(&terminal_bytes(&fixture.terminal));
    fixture
}

#[test]
fn matching_portable_admission_revalidates_complete_policy_free_capture_and_freshness() {
    let fixture = matching_fixture(20);
    let edition = fixture.admit().unwrap();
    let result = edition.execute_selected(&fixture.all()).unwrap();
    assert!(result.current_coverage_complete());
    assert_eq!(result.cases.len(), 4);
    assert!(
        result
            .cases
            .iter()
            .all(|case| case.raw.as_ref().unwrap().is_empty() && case.qualified.is_ok())
    );
    put(
        &fixture.owner,
        ORIGINAL_OWNER_SOURCE,
        b"changed original source",
    );
    assert!(
        edition
            .execute_selected(&fixture.all())
            .unwrap()
            .freshness_error
            .is_some()
    );
}

#[test]
fn matching_portable_admission_refuses_missing_changed_observations_and_raw_mismatches() {
    assert!(matching_fixture(21).admit().is_err());
    for changed in [
        "missing-input",
        "changed-history",
        "changed-context",
        "changed-packet",
        "missing-variant",
    ] {
        let mut fixture = matching_fixture(20);
        match changed {
            "missing-input" => fs::remove_file(fixture.root().join(SOURCE)).unwrap(),
            "changed-history" => {
                let history = fixture.terminal["cases"][0]["historical"]["path"]
                    .as_str()
                    .unwrap();
                put(&fixture.root(), history, b"changed observation");
            }
            "changed-context" | "changed-packet" => {
                let output = output_path(VARIANTS[0]);
                let xml = fs::read_to_string(fixture.root().join(&output)).unwrap();
                let xml = if changed == "changed-context" {
                    xml.replace(
                        "hypothetical-display=\"flex\"",
                        "hypothetical-display=\"grid\"",
                    )
                } else {
                    xml.replace("width=\"20\"", "width=\"21\"")
                };
                put(&fixture.root(), &output, xml.as_bytes());
            }
            "missing-variant" => fixture.rewrite_terminal(|terminal| {
                terminal["cases"].as_array_mut().unwrap().pop();
            }),
            _ => unreachable!(),
        }
        assert!(fixture.admit().is_err(), "{changed}");
    }
}
