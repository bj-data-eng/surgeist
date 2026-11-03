use super::*;
use std::sync::{Arc, Barrier};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "surgeist-layout-edition-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn parent_admission_requires_existing_owned_directory_without_creating_ancestors() {
    let owner = Directory::new();
    assert!(edition_parent(&owner.0).is_err());
    assert_eq!(fs::read_dir(&owner.0).unwrap().count(), 0);
    fs::create_dir_all(owner.0.join(ORIGINAL)).unwrap();
    assert!(edition_parent(&owner.0).is_err());
    assert!(!owner.0.join(PARENT).exists());
    fs::create_dir(owner.0.join(PARENT)).unwrap();
    assert_eq!(edition_parent(&owner.0).unwrap(), owner.0.join(PARENT));
}

#[cfg(unix)]
#[test]
fn parent_admission_refuses_an_external_directory_alias() {
    use std::os::unix::fs::symlink;
    let owner = Directory::new();
    let foreign = Directory::new();
    fs::create_dir_all(owner.0.join(ORIGINAL)).unwrap();
    symlink(&foreign.0, owner.0.join(PARENT)).unwrap();
    assert!(edition_parent(&owner.0).is_err());
    assert_eq!(fs::read_dir(&foreign.0).unwrap().count(), 0);
}

#[test]
fn exclusive_creation_preserves_existing_directory_and_file_targets() {
    for directory in [false, true] {
        let parent = Directory::new();
        let target = parent.0.join(EDITION);
        if directory {
            fs::create_dir(&target).unwrap();
            fs::write(target.join("owned-diagnostic"), b"retain").unwrap();
        } else {
            fs::write(&target, b"retain").unwrap();
        }
        assert!(create_edition(&parent.0).is_err());
        let retained = if directory {
            target.join("owned-diagnostic")
        } else {
            target
        };
        assert_eq!(fs::read(retained).unwrap(), b"retain");
    }
}

#[test]
fn simultaneous_exclusive_creators_have_one_winner_without_replacing_its_files() {
    let parent = Directory::new();
    let barrier = Arc::new(Barrier::new(2));
    let handles = (0..2)
        .map(|_| {
            let parent = parent.0.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                create_edition(&parent).is_ok()
            })
        })
        .collect::<Vec<_>>();
    let winners = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(|won| *won)
        .count();
    assert_eq!(winners, 1);
    let diagnostic = parent.0.join(EDITION).join("owned-diagnostic");
    fs::write(&diagnostic, b"retain winner").unwrap();
    assert!(create_edition(&parent.0).is_err());
    assert_eq!(fs::read(diagnostic).unwrap(), b"retain winner");
}

#[test]
fn failed_stage_retains_unregistered_candidate_and_refuses_automatic_retry() {
    let parent = Directory::new();
    let error = with_new_edition(&parent.0, |root| {
        fs::write(root.join("preparation-diagnostic"), b"actual partial input")?;
        Err("stage refused".into())
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "stage refused");
    let target = parent.0.join(EDITION);
    assert_eq!(
        fs::read(target.join("preparation-diagnostic")).unwrap(),
        b"actual partial input"
    );
    let mut called = false;
    assert!(
        with_new_edition(&parent.0, |_| {
            called = true;
            Ok(())
        })
        .is_err()
    );
    assert!(!called);
    assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn linked_target_and_linked_input_refuse_without_touching_foreign_files() {
    use std::os::unix::fs::symlink;
    let parent = Directory::new();
    let foreign = Directory::new();
    fs::write(foreign.0.join("input"), b"foreign").unwrap();
    symlink(&foreign.0, parent.0.join(EDITION)).unwrap();
    assert!(create_edition(&parent.0).is_err());
    symlink(foreign.0.join("input"), parent.0.join("input")).unwrap();
    let path = RelativePath::new("input").unwrap();
    assert!(
        inventory_input(
            &parent.0,
            &path,
            b"foreign",
            &mut BTreeMap::new(),
            InputAccess::ReadOnly
        )
        .is_err()
    );
    assert_eq!(fs::read(foreign.0.join("input")).unwrap(), b"foreign");
}

#[test]
fn read_only_inputs_refuse_missing_changed_or_stale_witnesses_without_repair() {
    let root = Directory::new();
    let path = RelativePath::new("origin/receipt.json").unwrap();
    let compare = |bytes: &[u8]| {
        inventory_input(
            &root.0,
            &path,
            bytes,
            &mut BTreeMap::new(),
            InputAccess::ReadOnly,
        )
    };
    assert!(compare(b"current").is_err());
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 0);
    fs::create_dir(root.0.join("origin")).unwrap();
    fs::write(root.0.join(path.as_str()), b"current").unwrap();
    compare(b"current").unwrap();
    assert!(compare(b"new authoritative bytes").is_err());
    assert_eq!(fs::read(root.0.join(path.as_str())).unwrap(), b"current");
    fs::write(root.0.join(path.as_str()), b"tampered").unwrap();
    assert!(compare(b"current").is_err());
    assert_eq!(fs::read(root.0.join(path.as_str())).unwrap(), b"tampered");
}

#[test]
fn complete_input_census_refuses_extra_empty_directories_files_and_missing_inputs() {
    let root = Directory::new();
    let path = RelativePath::new("html/actual-source.html").unwrap();
    let mut expected = BTreeMap::new();
    inventory_input(
        &root.0,
        &path,
        b"source witness",
        &mut expected,
        InputAccess::Prepare,
    )
    .unwrap();
    check_input_census(&root.0, &expected).unwrap();
    fs::create_dir(root.0.join("unregistered")).unwrap();
    assert!(check_input_census(&root.0, &expected).is_err());
    fs::remove_dir(root.0.join("unregistered")).unwrap();
    fs::write(root.0.join("html/extra.html"), b"unowned").unwrap();
    assert!(check_input_census(&root.0, &expected).is_err());
    fs::remove_file(root.0.join("html/extra.html")).unwrap();
    fs::remove_file(root.0.join(path.as_str())).unwrap();
    assert!(check_input_census(&root.0, &expected).is_err());
}

#[test]
fn selected_origin_membership_uses_actual_receipt_and_authored_partition() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = owner.join(ORIGINAL);
    let bytes = fs::read(original.join("html/.surgeist-source.json")).unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // Form the public checked provenance from the REAL full receipt; this tests
    // its parser/member contract, not the verify_import execution preflight.
    let mut encoded = serde_json::json!({
        "schema_version": receipt["schema_version"],
        "specification": receipt["specification"],
        "files": receipt["files"],
        "sidecar": "html/.surgeist-source.json",
        "receipt_sha256": Sha256Digest::from_bytes(&bytes),
    });
    let proof: SourceImportProvenance = serde_json::from_value(encoded.clone()).unwrap();
    let imported = &proof.files()[0];
    assert_eq!(
        selected_ownership(imported.path(), imported.sha256(), &proof).unwrap(),
        "imported"
    );
    assert!(
        selected_ownership(
            imported.path(),
            &Sha256Digest::from_bytes(b"changed"),
            &proof
        )
        .is_err()
    );
    let authored = &proof.specification().authored_files()[0];
    let digest =
        Sha256Digest::from_bytes(fs::read(original.join("html").join(authored.as_str())).unwrap());
    assert_eq!(
        selected_ownership(authored, &digest, &proof).unwrap(),
        "repository-authored"
    );
    assert!(
        selected_ownership(
            &RelativePath::new("unowned/not-declared.html").unwrap(),
            &digest,
            &proof
        )
        .is_err()
    );
    encoded["receipt_sha256"] =
        serde_json::to_value(Sha256Digest::from_bytes(b"unrelated receipt")).unwrap();
    assert!(serde_json::from_value::<SourceImportProvenance>(encoded).is_err());
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn complete_durable_inventory_is_admitted_without_browser_or_output() {
    let root = Directory::new();
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = edition_input(DECLARATION).unwrap();
    let declaration = &input.declaration;
    let mut snapshot = inventory(
        &root.0,
        owner,
        &input,
        QualificationMode::Regular,
        InputAccess::Prepare,
    )
    .unwrap();
    Origin::read(owner, &declaration.records)
        .unwrap()
        .inputs(&root.0, &mut snapshot, InputAccess::Prepare)
        .unwrap();
    let browser: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(owner_bytes().unwrap()).unwrap(),
        "browser-owner.toml",
    )
    .unwrap();
    // Fixture HTML and global helper/origin inputs have different owners in
    // Generator's inventory contract. The complete Layout declaration must be
    // admissible before requesting any browser or generated artifact.
    let corpus = corpus_for_inventory(
        &root.0,
        owner,
        browser.browser.settings().unwrap(),
        &snapshot,
        false,
        declaration.expected().unwrap(),
    )
    .expect("the complete source-qualified inventory is admitted");
    assert_eq!(corpus.fixtures().len(), declaration.records.len());
    check_input_census(&root.0, &snapshot.expected_inputs).unwrap();
    assert!(!root.0.join("xml").exists());
    assert!(!root.0.join(".surgeist-generator").exists());
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn browser_free_check_of_incomplete_real_inventory_refuses_without_writing() {
    let root = Directory::new();
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut input = DeclarationInput::parse(DECLARATION).unwrap();
    let selected = input
        .declaration
        .records
        .iter()
        .find(|record| {
            record.relation.is_none()
                && matches!(&record.expected, ExpectedDisposition::Generated { .. })
        })
        .unwrap()
        .source
        .clone();
    input
        .declaration
        .records
        .retain(|record| record.source == selected);
    let snapshot = inventory(
        &root.0,
        owner,
        &input,
        QualificationMode::Regular,
        InputAccess::Prepare,
    )
    .unwrap();
    let browser: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(owner_bytes().unwrap()).unwrap(),
        "browser-owner.toml",
    )
    .unwrap();
    let corpus = corpus_for_inventory(
        &root.0,
        owner,
        browser.browser.settings().unwrap(),
        &snapshot,
        false,
        BrowserReportSummary::new(fixture_cases().len(), 0, 0, 0, 0).unwrap(),
    )
    .unwrap();
    match browser::check_corpus(&corpus, &LayoutAdapter) {
        Err(GenerationError::Generator(error)) => {
            assert_eq!(error.kind(), GeneratorErrorKind::Verification);
            assert!(error.to_string().contains("current full report is absent"));
        }
        other => panic!("expected missing-report refusal, received {other:?}"),
    }
    // No browser/cache acquisition, repair, output directory or machine journals.
    check_input_census(&root.0, &snapshot.expected_inputs).unwrap();
    assert!(!root.0.join("xml").exists());
    assert!(!root.0.join(".surgeist-generator").exists());
    let reports = root.0.join("xml/generation-reports");
    fs::create_dir_all(&reports).unwrap();
    let report = reports.join("all.json");
    fs::write(&report, b"not a generation report").unwrap();
    assert!(browser::check_corpus(&corpus, &LayoutAdapter).is_err());
    assert_eq!(fs::read(&report).unwrap(), b"not a generation report");
    assert_eq!(fs::read_dir(&reports).unwrap().count(), 1);
    assert!(!root.0.join(".surgeist-generator").exists());
}

// This is one independent append stimulus, not a second scheduling inventory.
// Its actual imported HTML/history supplies only input identity, never geometry.
const APPENDED_ORIGINAL: &str =
    "html/grid/grid_fit_content_percent_indefinite_max_content_hidden.html";

fn declaration_with_appended_original() -> Vec<u8> {
    let prefix = edition_input(DECLARATION).unwrap();
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).join(ORIGINAL);
    let source_hash = Sha256Digest::from_bytes(fs::read(owner.join(APPENDED_ORIGINAL)).unwrap());
    let mut bytes = prefix.bytes.to_vec();
    let mut suffix = format!(
        "\n[[records]]\nsource = {APPENDED_ORIGINAL:?}\nsource_sha256 = \"{}\"\n",
        source_hash.as_str()
    );
    suffix.push_str("source_kind = \"original\"\n[records.historical_sha256]\n");
    let family = APPENDED_ORIGINAL
        .strip_prefix("html/")
        .unwrap()
        .strip_suffix(".html")
        .unwrap();
    for (variant, _) in fixture_cases() {
        let path = format!("xml/{family}__{variant}.xml");
        let digest = Sha256Digest::from_bytes(fs::read(owner.join(path)).unwrap());
        suffix.push_str(&format!("{variant} = \"{}\"\n", digest.as_str()));
    }
    suffix.push_str(
        r#"[records.expected]
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
    );
    bytes.extend_from_slice(suffix.as_bytes());
    bytes
}

fn case_identities(snapshot: &Inventory) -> Vec<(String, String, String, String)> {
    snapshot
        .cases
        .iter()
        .map(|case| {
            (
                case.source.as_str().to_owned(),
                case.case.id().to_owned(),
                case.case.variant().to_owned(),
                case.case.output().as_str().to_owned(),
            )
        })
        .collect()
}

#[test]
fn sealed_edition_reconstructs_same_inputs_and_cases_after_unique_append() {
    let current = edition_input(DECLARATION).unwrap();
    assert_eq!(
        Sha256Digest::from_bytes(current.bytes).as_str(),
        DECLARATION_PREFIX_SHA256
    );
    let grown = declaration_with_appended_original();
    let edition = edition_input(&grown).unwrap();
    assert_eq!(edition.bytes, current.bytes);
    assert_eq!(
        edition.declaration.expected().unwrap(),
        current.declaration.expected().unwrap()
    );
    let root = Directory::new();
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let original = inventory(
        &root.0,
        owner,
        &current,
        QualificationMode::Regular,
        InputAccess::Prepare,
    )
    .unwrap();
    let manifest = root.0.join("corpus.toml");
    let modified = fs::metadata(&manifest).unwrap().modified().unwrap();
    let checked = inventory(
        &root.0,
        owner,
        &edition,
        QualificationMode::Regular,
        InputAccess::ReadOnly,
    )
    .unwrap();
    assert_eq!(checked.expected_inputs, original.expected_inputs);
    assert_eq!(case_identities(&checked), case_identities(&original));
    assert_eq!(fs::read(&manifest).unwrap(), current.bytes);
    assert_eq!(
        fs::metadata(&manifest).unwrap().modified().unwrap(),
        modified
    );
    check_input_census(&root.0, &checked.expected_inputs).unwrap();
    // Output bytes cannot select the source prefix or be repaired by checking.
    fs::write(&manifest, b"schema_version = 3\n").unwrap();
    assert!(
        inventory(
            &root.0,
            owner,
            &edition,
            QualificationMode::Regular,
            InputAccess::ReadOnly
        )
        .is_err()
    );
    assert_eq!(fs::read(&manifest).unwrap(), b"schema_version = 3\n");
    assert!(!root.0.join("xml").exists());
}

#[test]
fn sealed_edition_refuses_short_edited_reordered_or_missing_prefix() {
    let source = edition_input(DECLARATION).unwrap();
    let prefix = source.bytes;
    assert!(edition_input(b"schema_version = 3\n").is_err());
    let short = &prefix[..prefix.len() - 1];
    assert!(DeclarationInput::parse(short).is_ok());
    assert!(edition_input(short).is_err());
    let mut edited = prefix.to_vec();
    let comment = std::str::from_utf8(prefix)
        .unwrap()
        .find("# One source")
        .unwrap();
    edited[comment + 2] = b'T';
    assert!(DeclarationInput::parse(&edited).is_ok());
    assert!(edition_input(&edited).is_err());
    let text = std::str::from_utf8(prefix).unwrap();
    let first = text.find("\n[[records]]").unwrap();
    let second = first + 1 + text[first + 1..].find("\n[[records]]").unwrap();
    let third = second + 1 + text[second + 1..].find("\n[[records]]").unwrap();
    let reordered = format!(
        "{}{}{}{}",
        &text[..first],
        &text[second..third],
        &text[first..second],
        &text[third..]
    );
    assert!(DeclarationInput::parse(reordered.as_bytes()).is_ok());
    assert!(edition_input(reordered.as_bytes()).is_err());
}

#[test]
fn complete_current_declaration_refuses_appended_duplicate_and_prior_source_shadow() {
    let grown = declaration_with_appended_original();
    let mut duplicate = grown.clone();
    duplicate.extend_from_slice(&grown[DECLARATION_PREFIX_LENGTH..]);
    let error = edition_input(&duplicate).err().unwrap();
    assert!(error.to_string().contains("duplicate qualification source"));
    let prefix = edition_input(DECLARATION).unwrap();
    let shadow_suffix = std::str::from_utf8(&grown[DECLARATION_PREFIX_LENGTH..])
        .unwrap()
        .replace(
            APPENDED_ORIGINAL,
            prefix.declaration.records[0].source.as_str(),
        );
    let shadow = [prefix.bytes, shadow_suffix.as_bytes()].concat();
    assert!(DeclarationInput::parse(&shadow).is_err());
    assert!(
        edition_input(&shadow)
            .err()
            .unwrap()
            .to_string()
            .contains("duplicate qualification source")
    );
}

#[test]
fn full_qualification_and_numeric_focus_keep_appended_original_and_complete_bytes() {
    let grown = declaration_with_appended_original();
    let edition = edition_input(&grown).unwrap();
    let mut full = DeclarationInput::parse(&grown).unwrap();
    assert_eq!(
        full.declaration.records.len(),
        edition.declaration.records.len() + 1
    );
    let old_counts = edition.declaration.expected().unwrap();
    let new_counts = full.declaration.expected().unwrap();
    assert_eq!(
        new_counts.generated(),
        old_counts.generated() + fixture_cases().len()
    );
    assert_eq!(new_counts.unsupported(), old_counts.unsupported());
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = Directory::new();
    let snapshot = inventory(
        &root.0,
        owner,
        &full,
        QualificationMode::Regular,
        InputAccess::Prepare,
    )
    .unwrap();
    assert_eq!(snapshot.fixtures.len(), full.declaration.records.len());
    assert!(
        snapshot
            .cases
            .iter()
            .any(|case| case.source.as_str() == APPENDED_ORIGINAL)
    );
    assert_eq!(fs::read(root.0.join("corpus.toml")).unwrap(), grown);
    select_records(
        &mut full.declaration,
        Some(APPENDED_ORIGINAL),
        QualificationMode::NumericDiagnostics,
    )
    .unwrap();
    let focused = Directory::new();
    let snapshot = inventory(
        &focused.0,
        owner,
        &full,
        QualificationMode::NumericDiagnostics,
        InputAccess::Prepare,
    )
    .unwrap();
    assert_eq!(snapshot.fixtures.len(), 1);
    assert_eq!(snapshot.cases.len(), fixture_cases().len());
    assert!(
        snapshot
            .cases
            .iter()
            .all(|case| case.source.as_str() == APPENDED_ORIGINAL)
    );
    assert_eq!(fs::read(focused.0.join("corpus.toml")).unwrap(), grown);
}

#[test]
fn whole_current_declaration_and_shared_input_freshness_still_refuse_stale_binary() {
    let owner = Directory::new();
    for (name, bytes) in INPUTS {
        let name = if *name == "browser-owner.toml" {
            "corpus.toml"
        } else {
            *name
        };
        let path = owner.0.join(ORIGINAL).join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    let declaration = owner.0.join("examples/qualify-root-context/corpus.toml");
    fs::create_dir_all(declaration.parent().unwrap()).unwrap();
    fs::write(&declaration, DECLARATION).unwrap();
    require_current_inputs(&owner.0).unwrap();
    let grown = declaration_with_appended_original();
    fs::write(&declaration, &grown).unwrap();
    assert_eq!(
        require_current_inputs(&owner.0).unwrap_err().to_string(),
        "current qualification declaration differs from this executable"
    );
    assert_eq!(fs::read(&declaration).unwrap(), grown);
    fs::write(&declaration, DECLARATION).unwrap();
    let helper = owner
        .0
        .join(ORIGINAL)
        .join("scripts/gentest/test_helper.js");
    fs::write(&helper, b"changed current helper").unwrap();
    assert!(
        require_current_inputs(&owner.0)
            .unwrap_err()
            .to_string()
            .contains("current source owner differs from this executable")
    );
    assert_eq!(fs::read(helper).unwrap(), b"changed current helper");
}
