use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

const RECEIPT_SHA256: &str = "a393498f143c6cbd7ebe7503c16fefb410d0495e586253bb1bc31b9354418d0e";
static NEXT_BUNDLE: AtomicU64 = AtomicU64::new(0);

fn artifact_root(owner: &Path) -> PathBuf {
    owner
        .join("tests/layout/browser_parity/captured")
        .join(authority::ORIGINAL_DIAGNOSTIC_EDITION)
}

fn owner() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn copy_file(from: &Path, to: &Path, relative: &str) {
    let target = to.join(relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(from.join(relative), target).unwrap();
}

#[test]
fn original_diagnostics_replay_all_nine_families_and_four_variants() {
    let replayed = admit_original_diagnostics(owner(), RECEIPT_SHA256).unwrap();
    assert_eq!(replayed.cases.len(), 36);
    for source in authority::GRID_LANES_AXIS_ORIGINAL_SOURCES {
        let stem = Path::new(source).file_stem().unwrap().to_str().unwrap();
        for (variant, _) in crate::fixture_variants::fixture_cases() {
            let golden = replayed.golden(source, variant).unwrap();
            assert_eq!(golden.name, format!("{stem}__{variant}"));
        }
    }
    assert!(
        replayed
            .golden("html/unknown.html", "border_box_ltr")
            .is_err()
    );
    assert!(authority::validate_matching_edition(authority::ORIGINAL_DIAGNOSTIC_EDITION).is_err());
}

struct CopiedBundle {
    owner: PathBuf,
    root: PathBuf,
}

impl CopiedBundle {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temporary_owner = std::env::temp_dir().join(format!(
            "surgeist-original-diagnostic-{}-{nonce}-{}",
            std::process::id(),
            NEXT_BUNDLE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&temporary_owner).unwrap();
        let bundle = Self {
            root: artifact_root(&temporary_owner),
            owner: temporary_owner,
        };
        let original = artifact_root(owner());
        let receipt =
            fs::read_to_string(original.join(authority::ORIGINAL_DIAGNOSTIC_RECEIPT)).unwrap();
        let receipt = authority::OriginalDiagnosticReceipt::parse(&receipt).unwrap();
        for file in static_files(&original).unwrap() {
            copy_file(&original, &bundle.root, &file);
        }
        for file in receipt
            .current_owner_inputs
            .keys()
            .map(String::as_str)
            .chain([OWNER_DECLARATION])
        {
            copy_file(owner(), &bundle.owner, file);
        }
        bundle
    }

    fn copy_controls(&self) {
        let relative = Edition::Matching(authority::GRID_LANES_PAIRED_EDITION).artifact_root();
        let from = owner().join(&relative);
        let to = self.owner.join(&relative);
        let terminal =
            Qualification::parse(&fs::read_to_string(from.join(authority::TERMINAL)).unwrap())
                .unwrap();
        for file in static_files(&from).unwrap() {
            copy_file(&from, &to, &file);
        }
        for file in terminal.current_owner_inputs.keys() {
            copy_file(owner(), &self.owner, file);
        }
    }

    fn read(&self, path: &str) -> Value {
        parse_json(&fs::read(self.root.join(path)).unwrap()).unwrap()
    }

    fn receipt(&self) -> Value {
        self.read(authority::ORIGINAL_DIAGNOSTIC_RECEIPT)
    }

    fn write(&self, path: &str, bytes: &[u8]) -> String {
        fs::write(self.root.join(path), bytes).unwrap();
        sha256(bytes)
    }

    fn write_json(&self, path: &str, value: &Value) -> String {
        self.write(path, &serde_json::to_vec_pretty(value).unwrap())
    }

    fn browser_settings(&self) -> authority::BrowserSettingsWitness {
        let current =
            fs::read_to_string(self.root.join(authority::ORIGINAL_DIAGNOSTIC_RECEIPT)).unwrap();
        authority::OriginalDiagnosticReceipt::parse(&current)
            .unwrap()
            .browser_settings
    }

    fn rewrite_receipt(&self, value: Value) -> String {
        self.write_receipt(value, self.browser_settings())
    }

    fn write_receipt(&self, value: Value, witness: authority::BrowserSettingsWitness) -> String {
        let mut receipt: authority::OriginalDiagnosticReceipt =
            serde_json::from_value(value).unwrap();
        receipt.browser_settings = witness;
        // Reissue the edited wire so validation happens at the reader boundary.
        let mut bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        bytes.push(b'\n');
        self.write(authority::ORIGINAL_DIAGNOSTIC_RECEIPT, &bytes)
    }

    fn repin_report(
        &self,
        mut receipt: Value,
        mut report: Value,
        settings: authority::BrowserSettingsWitness,
    ) -> String {
        report["metadata"]["inputs"] = receipt["retained_inputs"].clone();
        let metadata = serde_json::from_value(report["metadata"].clone()).unwrap();
        let inputs = serde_json::from_value(receipt["retained_inputs"].clone()).unwrap();
        for row in receipt["cases"].as_array_mut().unwrap() {
            let case: authority::CaseDescriptor =
                serde_json::from_value(row["case"].clone()).unwrap();
            let artifact = report["generated"]
                .as_array()
                .unwrap()
                .iter()
                .find(|artifact| artifact["name"] == case.name)
                .unwrap();
            let artifact = serde_json::from_value(artifact.clone()).unwrap();
            let xml = fs::read(self.root.join(&case.output.path)).unwrap();
            let identity =
                authority::capture_binding_from_verified_inputs(authority::CaptureBindingInputs {
                    metadata: &metadata,
                    artifact: &artifact,
                    case: &case,
                    full_inputs: &inputs,
                    excluded_specs: &BTreeSet::new(),
                    actual_xml_sha256: &sha256(&xml),
                    historical_sha256: row["historical"]["sha256"].as_str(),
                    browser_settings: &settings,
                    observation_request: "geometry",
                    observation_input_sha256: None,
                })
                .unwrap();
            row["identity"] = serde_json::to_value(identity).unwrap();
        }
        let report_path = receipt["report"]["path"].as_str().unwrap().to_owned();
        let report_hash = self.write_json(&report_path, &report);
        receipt["report"]["sha256"] = Value::from(report_hash.as_str());
        let ownership_path = receipt["ownership"]["path"].as_str().unwrap().to_owned();
        let mut ownership = self.read(&ownership_path);
        ownership["artifacts"][&report_path] = Value::from(report_hash);
        receipt["ownership"]["sha256"] = Value::from(self.write_json(&ownership_path, &ownership));
        self.write_receipt(receipt, settings)
    }

    fn expect_refusal(&self, pin: &str, reason: &str) {
        let error = admit_original_diagnostics(&self.owner, pin)
            .err()
            .expect("tampered evidence must refuse");
        assert!(
            error.to_string().contains(reason),
            "expected {reason:?}, got {error}"
        );
    }
}

impl Drop for CopiedBundle {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.owner).unwrap();
    }
}

#[test]
fn paired_matching_refuses_missing_altered_or_stale_original_diagnostics() {
    let bundle = CopiedBundle::new();
    bundle.copy_controls();
    let admitted = admit_grid_lanes_axis_controls(&bundle.owner).unwrap();
    let selected = admitted.outputs().map(str::to_owned).collect();
    assert!(
        admitted
            .execute_selected(&selected)
            .unwrap()
            .registered_coverage_complete()
    );

    let path = bundle.root.join(authority::ORIGINAL_DIAGNOSTIC_RECEIPT);
    let receipt = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    let error = admit_grid_lanes_axis_controls(&bundle.owner).err().unwrap();
    assert!(
        error
            .to_string()
            .contains(authority::ORIGINAL_DIAGNOSTIC_RECEIPT),
        "{error}"
    );

    let mut changed = receipt.clone();
    changed.push(b'\n');
    fs::write(&path, changed).unwrap();
    let error = admit_grid_lanes_axis_controls(&bundle.owner).err().unwrap();
    assert!(
        error
            .to_string()
            .contains(authority::ORIGINAL_DIAGNOSTIC_RECEIPT),
        "{error}"
    );
    fs::write(path, receipt).unwrap();

    let report = bundle.receipt()["report"]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    let stale = admitted
        .execute_selected_before_freshness(&selected, || {
            let mut bytes = fs::read(bundle.root.join(&report)).unwrap();
            bytes.push(b'\n');
            bundle.write(&report, &bytes);
        })
        .unwrap();
    assert!(stale.freshness_error.unwrap().contains(&report));
    assert_eq!(stale.cases.len(), 36);
    assert!(
        stale
            .cases
            .iter()
            .all(|case| case.raw.as_ref().unwrap().is_empty() && case.qualified.is_err())
    );
}

#[test]
fn paired_matching_refuses_an_adoptable_older_producing_helper() {
    let terminal_path = owner()
        .join(Edition::Matching(authority::GRID_LANES_PAIRED_EDITION).artifact_root())
        .join(authority::TERMINAL);
    let bytes = fs::read(&terminal_path).unwrap();
    let mut wire: Value = parse_json(&bytes).unwrap();
    let mut inputs: BTreeMap<String, String> =
        serde_json::from_value(wire["retained_inputs"].clone()).unwrap();
    inputs.insert(
        authority::ROOT_HELPER_INPUT.to_owned(),
        crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256.to_owned(),
    );
    wire["capture_validation"] =
        serde_json::to_value(authority::CaptureValidation::from_retained(&inputs).unwrap())
            .unwrap();
    wire["retained_inputs"] = serde_json::to_value(inputs).unwrap();
    let text = serde_json::to_string(&wire).unwrap();
    let error = Qualification::parse(&text).err().unwrap();
    assert!(
        error
            .to_string()
            .contains("source-equivalent Grid Lanes producing helper"),
        "{error}"
    );
}

#[test]
fn original_diagnostics_refuse_a_repinned_older_producing_helper() {
    let bundle = CopiedBundle::new();
    assert_eq!(
        admit_original_diagnostics(&bundle.owner, RECEIPT_SHA256)
            .unwrap()
            .cases
            .len(),
        36
    );
    let mut receipt = bundle.receipt();
    let report = bundle.read(receipt["report"]["path"].as_str().unwrap());
    let helper = authority::ROOT_HELPER_INPUT;
    let older = fs::read(
        owner()
            .join("tests/layout/browser_parity/captured/scriptless-order")
            .join(helper),
    )
    .unwrap();
    let older_digest = bundle.write(helper, &older);
    assert_eq!(
        older_digest,
        crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256
    );
    assert_ne!(receipt["retained_inputs"][helper], older_digest);
    receipt["retained_inputs"][helper] = Value::from(older_digest);
    let inputs = serde_json::from_value(receipt["retained_inputs"].clone()).unwrap();
    receipt["capture_validation"] =
        serde_json::to_value(authority::CaptureValidation::from_retained(&inputs).unwrap())
            .unwrap();
    let pin = bundle.repin_report(receipt, report, bundle.browser_settings());
    assert_ne!(pin, RECEIPT_SHA256);
    bundle.expect_refusal(&pin, "Grid Lanes producing helper");
}

#[test]
fn original_diagnostics_refuse_receipt_changes_and_replay_empty_inventories() {
    let bundle = CopiedBundle::new();
    admit_original_diagnostics(&bundle.owner, RECEIPT_SHA256).unwrap();
    let mut receipt = bundle.receipt();
    assert!(
        !receipt["cases"][0]["raw_differences"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    receipt["cases"][0]["raw_differences"] = serde_json::json!([]);
    let changed_pin = bundle.rewrite_receipt(receipt);
    bundle.expect_refusal(RECEIPT_SHA256, authority::ORIGINAL_DIAGNOSTIC_RECEIPT);
    bundle.expect_refusal(&changed_pin, "original diagnostic numeric replay");
}

#[test]
fn original_diagnostics_refuse_coherently_repinned_context_changes() {
    let bundle = CopiedBundle::new();
    let mut receipt = bundle.receipt();
    let output = receipt["cases"][0]["case"]["output"]["path"]
        .as_str()
        .unwrap()
        .to_owned();
    let xml = fs::read_to_string(bundle.root.join(&output)).unwrap();
    let changed_xml = xml.replace(
        "hypothetical-inline-offset=\"0px\"",
        "hypothetical-inline-offset=\"1px\"",
    );
    assert_ne!(changed_xml, xml);
    let xml_hash = bundle.write(&output, changed_xml.as_bytes());
    receipt["cases"][0]["case"]["output"]["sha256"] = Value::from(xml_hash.as_str());
    receipt["cases"][0]["identity"]["capture_sha256"] = Value::from(xml_hash.as_str());
    let report_path = receipt["report"]["path"].as_str().unwrap().to_owned();
    let mut report = bundle.read(&report_path);
    let artifact = report["generated"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["output"].as_str() == Some(output.as_str()))
        .unwrap();
    artifact["output_sha256"] = Value::from(xml_hash.as_str());
    let report_hash = bundle.write_json(&report_path, &report);
    receipt["report"]["sha256"] = Value::from(report_hash.as_str());
    let ownership_path = receipt["ownership"]["path"].as_str().unwrap().to_owned();
    let mut ownership = bundle.read(&ownership_path);
    ownership["artifacts"][&output] = Value::from(xml_hash);
    ownership["artifacts"][&report_path] = Value::from(report_hash);
    receipt["ownership"]["sha256"] = Value::from(bundle.write_json(&ownership_path, &ownership));
    let pin = bundle.rewrite_receipt(receipt);
    bundle.expect_refusal(&pin, "root inline offset changed");
}

#[test]
fn original_diagnostics_refuse_a_false_original_ownership_label() {
    let bundle = CopiedBundle::new();
    let mut receipt = bundle.receipt();
    let path = "origin/selected-originals.json";
    let mut selected = bundle.read(path);
    assert_eq!(selected[0]["ownership"], "repository-authored");
    selected[0]["ownership"] = Value::from("imported");
    receipt["retained_inputs"][path] = Value::from(bundle.write_json(path, &selected));
    let pin = bundle.rewrite_receipt(receipt);
    bundle.expect_refusal(&pin, "selected original diagnostic import partition");
}

#[test]
fn original_diagnostics_refuse_repinned_browser_provenance() {
    let bundle = CopiedBundle::new();
    let receipt = bundle.receipt();
    let mut report = bundle.read(receipt["report"]["path"].as_str().unwrap());
    report["metadata"]["browser_provenance"] = Value::from("unrelated-browser/149");
    let pin = bundle.repin_report(receipt, report, bundle.browser_settings());
    bundle.expect_refusal(&pin, "Grid Lanes browser executable or provenance");
}

#[test]
fn original_diagnostics_refuse_repinned_browser_outside_owner_cache() {
    let bundle = CopiedBundle::new();
    let receipt = bundle.receipt();
    let mut report = bundle.read(receipt["report"]["path"].as_str().unwrap());
    let executable = "tmp/unrelated-browser/chrome";
    let settings = &receipt["browser_settings"];
    let provenance = settings["provenance_format"]
        .as_str()
        .unwrap()
        .replace("{version}", settings["version"].as_str().unwrap())
        .replace("{repository_relative_executable}", executable);
    report["metadata"]["browser_executable"] = Value::from(executable);
    report["metadata"]["browser_provenance"] = Value::from(provenance);
    let pin = bundle.repin_report(receipt, report, bundle.browser_settings());
    bundle.expect_refusal(&pin, "Grid Lanes browser executable or provenance");
}

#[test]
fn original_diagnostics_refuse_repinned_browser_launch_settings() {
    let bundle = CopiedBundle::new();
    let receipt = bundle.receipt();
    let report = bundle.read(receipt["report"]["path"].as_str().unwrap());
    let mut settings = receipt["browser_settings"].clone();
    assert_ne!(settings["launch"]["batch_size"], 1);
    settings["launch"]["batch_size"] = Value::from(1);
    let settings = authority::BrowserSettingsWitness::from_official_json(
        &serde_json::to_vec(&settings).unwrap(),
    )
    .unwrap();
    let pin = bundle.repin_report(receipt, report, settings);
    bundle.expect_refusal(
        &pin,
        "Grid Lanes browser settings differ from the owner manifest",
    );
}

#[test]
fn original_diagnostics_refuse_repinned_missing_global_input() {
    let bundle = CopiedBundle::new();
    let mut receipt = bundle.receipt();
    let report = bundle.read(receipt["report"]["path"].as_str().unwrap());
    let path = "fonts/PROVENANCE.md";
    assert!(
        receipt["retained_inputs"]
            .as_object_mut()
            .unwrap()
            .remove(path)
            .is_some()
    );
    assert!(
        receipt["current_owner_inputs"]
            .as_object_mut()
            .unwrap()
            .remove(&format!("tests/layout/browser_parity/{path}"))
            .is_some()
    );
    fs::remove_file(bundle.root.join(path)).unwrap();
    let pin = bundle.repin_report(receipt, report, bundle.browser_settings());
    bundle.expect_refusal(&pin, "complete Grid Lanes capture input census");
}

#[test]
fn original_diagnostics_refuse_repinned_invented_linked_resource() {
    let bundle = CopiedBundle::new();
    let receipt = bundle.receipt();
    let mut report = bundle.read(receipt["report"]["path"].as_str().unwrap());
    let resource = "fonts/Ahem.ttf";
    assert_eq!(
        report["generated"][0]["linked_resources"],
        serde_json::json!({})
    );
    report["generated"][0]["linked_resources"][resource] =
        receipt["retained_inputs"][resource].clone();
    let pin = bundle.repin_report(receipt, report, bundle.browser_settings());
    bundle.expect_refusal(&pin, "Grid Lanes plain-box capture has linked resources");
}
