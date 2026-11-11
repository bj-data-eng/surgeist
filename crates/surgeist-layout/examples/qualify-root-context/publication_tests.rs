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
        assert!(create_edition(&parent.0, Edition::RootContextV4).is_err());
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
                create_edition(&parent, Edition::RootContextV4).is_ok()
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
    assert!(create_edition(&parent.0, Edition::RootContextV4).is_err());
    assert_eq!(fs::read(diagnostic).unwrap(), b"retain winner");
}

#[test]
fn failed_stage_retains_unregistered_candidate_and_refuses_automatic_retry() {
    let parent = Directory::new();
    let error = with_new_edition(&parent.0, Edition::RootContextV4, |root| {
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
        with_new_edition(&parent.0, Edition::RootContextV4, |_| {
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
    assert!(create_edition(&parent.0, Edition::RootContextV4).is_err());
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
    check_input_census(&root.0, &expected, None).unwrap();
    fs::create_dir(root.0.join("unregistered")).unwrap();
    assert!(check_input_census(&root.0, &expected, None).is_err());
    fs::remove_dir(root.0.join("unregistered")).unwrap();
    fs::write(root.0.join("html/extra.html"), b"unowned").unwrap();
    assert!(check_input_census(&root.0, &expected, None).is_err());
    fs::remove_file(root.0.join("html/extra.html")).unwrap();
    fs::remove_file(root.0.join(path.as_str())).unwrap();
    assert!(check_input_census(&root.0, &expected, None).is_err());
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
    Origin::read(owner, &declaration.records, Edition::RootContextV4)
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
    check_input_census(&root.0, &snapshot.expected_inputs, None).unwrap();
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
    check_input_census(&root.0, &snapshot.expected_inputs, None).unwrap();
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
    let mut bytes = prefix.bytes().to_vec();
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
fn focused_geometry_requires_only_its_selected_predecessors() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let empty_editions = Directory::new();
    let mode = QualificationMode::Regular;
    for (source, requires_predecessors) in [
        ("html/flex/fri07_collapsed_strut_single_line.html", false),
        ("html/grid/grid_absolute_resolved_insets.html", true),
        ("html/flex/fri07_collapsed_strut_wrapping.html", true),
    ] {
        let root = Directory::new();
        let mut declaration = DeclarationInput::parse(DECLARATION).unwrap();
        select_records(&mut declaration.declaration, Some(source), mode).unwrap();
        let inventory =
            inventory(&root.0, owner, &declaration, mode, InputAccess::Prepare).unwrap();
        assert!(inventory.cases.iter().all(|case| case.semantic.is_some()));
        let selected = focused_geometry_predecessors(&empty_editions.0, &inventory.cases, mode);
        if requires_predecessors {
            assert!(selected.is_err());
            assert_eq!(
                focused_geometry_predecessors(owner, &inventory.cases, mode)
                    .unwrap()
                    .len(),
                4
            );
        } else {
            assert!(selected.unwrap().is_empty());
        }
    }
}

#[test]
fn appended_semantic_publication_preserves_sealed_source_and_rejects_shadows() {
    let source = "html/grid/grid_absolute_resolved_insets.html";
    let digest = "a".repeat(64);
    let retained = edition_input(DECLARATION).unwrap();
    let prefix = retained.bytes();
    let mut annotation = format!("\n[[semantic_publications]]\nsource = {source:?}\n");
    for (variant, _) in fixture_cases() {
        annotation.push_str(&format!(
            "[semantic_publications.specifications.{variant}]\npath = \"semantic/absolute-insets-{variant}.json\"\nsha256 = {digest:?}\n"
        ));
    }
    let grown = [prefix, annotation.as_bytes()].concat();
    let live = DeclarationInput::parse(&grown).unwrap();
    let sealed = edition_input(&grown).unwrap();
    let record = |input: &DeclarationInput<'_>| {
        input
            .declaration
            .records
            .iter()
            .position(|record| record.source.as_str() == source)
            .unwrap()
    };
    let live = &live.declaration.records[record(&live)];
    let original = &sealed.declaration.records[record(&sealed)];
    assert!(original.semantic_specifications.is_none());
    assert_eq!(
        semantic_record_digest(live).unwrap(),
        semantic_record_digest(original).unwrap()
    );
    let references = live.semantic_specifications.as_ref().unwrap();
    assert_eq!(references.len(), 4);
    for (variant, _) in fixture_cases() {
        assert_eq!(
            references[variant].path.as_str(),
            format!("semantic/absolute-insets-{variant}.json")
        );
    }

    assert!(DeclarationInput::parse(&[grown.as_slice(), annotation.as_bytes()].concat()).is_err());
    let wrong_source = annotation.replace(source, "html/grid/undeclared.html");
    assert!(DeclarationInput::parse(&[prefix, wrong_source.as_bytes()].concat()).is_err());
    let incomplete = annotation.lines().take(5).collect::<Vec<_>>().join("\n");
    assert!(DeclarationInput::parse(&[prefix, incomplete.as_bytes()].concat()).is_err());
    let duplicate_path = annotation.replace(
        "absolute-insets-border_box_rtl.json",
        "absolute-insets-border_box_ltr.json",
    );
    assert!(DeclarationInput::parse(&[prefix, duplicate_path.as_bytes()].concat()).is_err());
}

#[test]
fn sealed_edition_reconstructs_same_inputs_and_cases_after_unique_append() {
    let current = edition_input(DECLARATION).unwrap();
    assert_eq!(
        Sha256Digest::from_bytes(current.bytes()).as_str(),
        DECLARATION_PREFIX_SHA256
    );
    let grown = declaration_with_appended_original();
    let edition = edition_input(&grown).unwrap();
    assert_eq!(edition.bytes(), current.bytes());
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
    assert_eq!(fs::read(&manifest).unwrap(), current.bytes());
    assert_eq!(
        fs::metadata(&manifest).unwrap().modified().unwrap(),
        modified
    );
    check_input_census(&root.0, &checked.expected_inputs, None).unwrap();
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
    let prefix = source.bytes();
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
    let shadow = [prefix.bytes(), shadow_suffix.as_bytes()].concat();
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

#[test]
fn fixed_editions_create_distinct_exclusive_targets() {
    let parent = Directory::new();
    let old = create_edition(&parent.0, Edition::RootContextV4).unwrap();
    let current =
        create_edition(&parent.0, Edition::Exact(PublishedEdition::CanonicalBoxes)).unwrap();
    let measured =
        create_edition(&parent.0, Edition::Exact(PublishedEdition::MeasuredGrid)).unwrap();
    assert_eq!(measured, parent.0.join(authority::MEASURED_GRID_EDITION));
    assert!(create_edition(&parent.0, Edition::Exact(PublishedEdition::MeasuredGrid)).is_err());
    assert_eq!(old, parent.0.join("root-context-v4"));
    assert_eq!(current, parent.0.join(authority::EDITION));
    assert!(create_edition(&parent.0, Edition::Exact(PublishedEdition::CanonicalBoxes)).is_err());
}
#[test]
fn retained_publication_selects_only_its_own_specifications() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let retained = owner.join(authority::ARTIFACT_ROOT);
    let mut declaration = fs::read(retained.join("corpus.toml")).unwrap();
    declaration.extend_from_slice(
        br#"

[[records]]
source = "html/qualification/unretained-publication-test.html"
source_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
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
[records.semantic_specifications.border_box_ltr]
path = "semantic/unretained-publication-test.json"
sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#,
    );
    let input = canonical_input(
        &retained,
        &declaration,
        PublishedEdition::CanonicalBoxes,
        InputAccess::ReadOnly,
    )
    .unwrap();
    assert_eq!(input.bytes(), declaration);
    assert_eq!(input.declaration.records.len(), 1);
    assert_eq!(
        input.declaration.records[0].source.as_str(),
        "html/flex/fri07_collapsed_strut_wrapping.html"
    );
    assert!(
        canonical_input(
            owner,
            &declaration,
            PublishedEdition::CanonicalBoxes,
            InputAccess::Prepare,
        )
        .is_err()
    );
}

#[test]
fn terminal_output_has_explicit_exact_byte_custody_outside_the_input_map() {
    let root = Directory::new();
    let expected = BTreeMap::new();
    let bytes = b"derived terminal bytes\n";
    fs::write(root.0.join(authority::TERMINAL), bytes).unwrap();
    assert!(check_input_census(&root.0, &expected, None).is_err());
    check_input_census(&root.0, &expected, Some((authority::TERMINAL, bytes))).unwrap();
    assert!(
        check_input_census(&root.0, &expected, Some((authority::TERMINAL, b"changed"))).is_err()
    );
    assert_eq!(fs::read(root.0.join(authority::TERMINAL)).unwrap(), bytes);
}
#[test]
fn official_browser_settings_witness_keeps_focused_profile_serialization_exact() {
    let owner: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(owner_bytes().unwrap()).unwrap(),
        "browser-owner.toml",
    )
    .unwrap();
    let settings = owner.browser.settings().unwrap();
    let witness = authority::BrowserSettingsWitness::from_official_json(
        &serde_json::to_vec(&settings).unwrap(),
    )
    .unwrap();
    let old = serde_json::to_vec(&(
        "canonical-no-text-boxes",
        &settings,
        "record",
        "variant",
        "geometry",
        Option::<Sha256Digest>::None,
    ))
    .unwrap();
    let shared = serde_json::to_vec(&(
        "canonical-no-text-boxes",
        &witness,
        "record",
        "variant",
        "geometry",
        Option::<&str>::None,
    ))
    .unwrap();
    assert_eq!(
        old, shared,
        "the official typed serializer remains the profile authority"
    );
}

#[test]
fn late_terminal_census_failure_removes_only_the_owned_terminal_output() {
    let root = Directory::new();
    fs::write(root.0.join("unexpected-input"), b"late input").unwrap();
    assert!(
        emit_terminal(
            &root.0,
            &BTreeMap::new(),
            authority::TERMINAL,
            b"terminal bytes"
        )
        .is_err()
    );
    assert!(!root.0.join(authority::TERMINAL).exists());
    assert_eq!(
        fs::read(root.0.join("unexpected-input")).unwrap(),
        b"late input"
    );
    fs::write(root.0.join(authority::TERMINAL), b"preexisting authority").unwrap();
    assert!(
        emit_terminal(
            &root.0,
            &BTreeMap::new(),
            authority::TERMINAL,
            b"replacement"
        )
        .is_err()
    );
    assert_eq!(
        fs::read(root.0.join(authority::TERMINAL)).unwrap(),
        b"preexisting authority"
    );
}

#[test]
fn matching_cli_dispatches_the_named_cohort_to_the_publication_owner() {
    for (command, expected_access) in [
        ("--publish-matching-geometry", InputAccess::Prepare),
        ("--check-published-matching-geometry", InputAccess::ReadOnly),
    ] {
        let arguments = vec![command.to_owned(), "box-controls".to_owned()];
        let (fixture, mode) = crate::parse_arguments(&arguments).unwrap();
        assert_eq!(fixture, None);
        let (edition, access) = publication_request(mode).unwrap();
        assert_eq!(edition, Edition::Matching("box-controls"));
        assert!(matches!(
            (access, expected_access),
            (InputAccess::Prepare, InputAccess::Prepare)
                | (InputAccess::ReadOnly, InputAccess::ReadOnly)
        ));
        let selected = matching_input(DECLARATION, edition.name()).unwrap();
        assert_eq!(
            selected
                .declaration
                .records
                .iter()
                .map(|r| r.source.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "html/block/block_basic.html",
                "html/block/block_calc_width_margin.html",
                "html/flex/flex_calc_basis_margin_gap.html",
                "html/grid/grid_calc_track_and_item_margin.html",
            ])
        );
        let parent = Directory::new();
        let created = create_edition(&parent.0, edition).unwrap();
        assert_eq!(created, parent.0.join("box-controls"));
        assert!(create_edition(&parent.0, edition).is_err());
    }
    assert!(matching_input(DECLARATION, "unknown-controls").is_err());
}

#[test]
fn generic_derivation_dispatches_every_stored_terminal_category_and_request() {
    use PublishedEdition as Exact;
    use SpecializedProfile as Specialized;
    let cases = [
        (
            "canonical-box-differences",
            Edition::Exact(Exact::CanonicalBoxes),
            BoxObservationRequest::Geometry,
            4,
        ),
        (
            "measured-grid-differences",
            Edition::Exact(Exact::MeasuredGrid),
            BoxObservationRequest::MeasuredLeaf,
            28,
        ),
        (
            "flex-text-differences",
            Edition::Exact(Exact::FlexText),
            BoxObservationRequest::FlexContentText,
            4,
        ),
        (
            "grid-static-position-differences",
            Edition::Exact(Exact::GridStaticPosition),
            BoxObservationRequest::Geometry,
            8,
        ),
        (
            "relative-inset-differences",
            Edition::Exact(Exact::RelativeInsets),
            BoxObservationRequest::Geometry,
            4,
        ),
        (
            "wrapping-world-corner-differences",
            Edition::Exact(Exact::WrappingWorldCorners),
            BoxObservationRequest::Geometry,
            4,
        ),
        (
            "prepared-range-qualification",
            Edition::Specialized(Specialized::PreparedRange),
            BoxObservationRequest::PreparedRanges,
            4,
        ),
        (
            "flex-composition-qualification",
            Edition::Specialized(Specialized::FlexComposition),
            BoxObservationRequest::FlexComposition,
            4,
        ),
        (
            "original-subgrid-body-qualification",
            Edition::Specialized(Specialized::OriginalSubgridBody),
            BoxObservationRequest::OriginalSubgridBody,
            4,
        ),
        (
            "prepared-atomic-percentage-qualification",
            Edition::Specialized(Specialized::PreparedAtomicPercentage),
            BoxObservationRequest::PreparedAtomicPercentage,
            4,
        ),
        (
            "horizontal-br-font-band-qualification",
            Edition::Specialized(Specialized::HorizontalBrFontBand),
            BoxObservationRequest::HorizontalBrFontBand,
            4,
        ),
        (
            "vertical-br-font-band-qualification",
            Edition::Specialized(Specialized::VerticalBrFontBand),
            BoxObservationRequest::VerticalBrFontBand,
            12,
        ),
        (
            "empty-line-br-font-band-qualification",
            Edition::Specialized(Specialized::EmptyLineBrFontBand),
            BoxObservationRequest::EmptyLineBrFontBand,
            8,
        ),
        (
            "box-controls",
            Edition::Matching("box-controls"),
            BoxObservationRequest::Geometry,
            16,
        ),
    ];
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (name, expected, request, count) in cases {
        let arguments = vec![
            "--derive-published-qualification".to_owned(),
            name.to_owned(),
        ];
        let (fixture, mode) = crate::parse_arguments(&arguments).unwrap();
        assert_eq!(fixture, None);
        assert!(mode.derives_terminal());
        assert_eq!(mode.box_observations(), request);
        let (edition, access) = publication_request(mode).unwrap();
        assert_eq!(edition, expected);
        assert!(matches!(access, InputAccess::ReadOnly));
        let root = owner.join(PARENT).join(name);
        let retained = fs::read(root.join("corpus.toml")).unwrap();
        let input = match edition {
            Edition::Exact(exact) => canonical_input(&root, &retained, exact, access).unwrap(),
            Edition::Specialized(profile) => specialized_input(&retained, profile).unwrap(),
            Edition::Matching(edition) => matching_input(&retained, edition).unwrap(),
            Edition::GridLanesDiagnostics => {
                unreachable!("the diagnostic bundle is separately checked")
            }
            Edition::RootContextV4 => unreachable!("root-context-v4 has no terminal"),
        };
        assert_eq!(input.declaration.expected().unwrap().generated(), count);
        assert_eq!(fs::read(root.join("corpus.toml")).unwrap(), retained);
    }
    for arguments in [
        vec!["--derive-published-qualification"],
        vec!["--derive-published-qualification", "unknown-edition"],
        vec!["--derive-published-qualification", "../box-controls"],
        vec!["--derive-published-qualification", "root-context-v4"],
        vec![
            "--derive-published-qualification",
            "box-controls",
            "candidate.json",
        ],
        vec![
            "--derive-published-qualification",
            "box-controls",
            "--fixture",
            "html/other.html",
        ],
    ] {
        let arguments: Vec<_> = arguments.into_iter().map(str::to_owned).collect();
        assert!(crate::parse_arguments(&arguments).is_err());
    }
}

#[test]
fn generic_derivation_emits_exact_candidate_without_installing_or_changing_capture_inputs() {
    let root = Directory::new();
    let source = RelativePath::new("original.html").unwrap();
    let original = b"unchanged producing source bytes";
    let stored = b"previous terminal authority bytes";
    let candidate = b"new fully derived terminal authority bytes\n";
    fs::write(root.0.join(source.as_str()), original).unwrap();
    fs::write(root.0.join(authority::TERMINAL), stored).unwrap();
    let expected = BTreeMap::from([(source.clone(), Sha256Digest::from_bytes(original))]);
    let mut output = Vec::new();
    publish_terminal(
        &root.0,
        &expected,
        authority::TERMINAL,
        TerminalPublication::Derive(stored),
        candidate,
        &mut output,
    )
    .unwrap();
    assert_eq!(output, candidate);
    assert_eq!(fs::read(root.0.join(authority::TERMINAL)).unwrap(), stored);
    assert_eq!(fs::read(root.0.join(source.as_str())).unwrap(), original);
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 2);

    output.clear();
    assert!(
        publish_terminal(
            &root.0,
            &expected,
            authority::TERMINAL,
            TerminalPublication::Check(stored),
            candidate,
            &mut output
        )
        .is_err()
    );
    assert!(output.is_empty());
    fs::write(
        root.0.join(authority::TERMINAL),
        b"changed retained authority",
    )
    .unwrap();
    assert!(
        publish_terminal(
            &root.0,
            &expected,
            authority::TERMINAL,
            TerminalPublication::Derive(stored),
            candidate,
            &mut output
        )
        .is_err()
    );
    assert!(output.is_empty());
    fs::write(root.0.join(authority::TERMINAL), stored).unwrap();
    fs::remove_file(root.0.join(source.as_str())).unwrap();
    assert!(
        publish_terminal(
            &root.0,
            &expected,
            authority::TERMINAL,
            TerminalPublication::Derive(stored),
            candidate,
            &mut output
        )
        .is_err()
    );
    assert!(output.is_empty());
}

#[test]
fn generic_derivation_output_failure_preserves_existing_terminal() {
    struct BrokenOutput;
    impl Write for BrokenOutput {
        fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("candidate output unavailable"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let root = Directory::new();
    let stored = b"previous terminal authority bytes";
    fs::write(root.0.join(authority::TERMINAL), stored).unwrap();
    assert!(
        publish_terminal(
            &root.0,
            &BTreeMap::new(),
            authority::TERMINAL,
            TerminalPublication::Derive(stored),
            b"candidate",
            &mut BrokenOutput
        )
        .is_err()
    );
    assert_eq!(fs::read(root.0.join(authority::TERMINAL)).unwrap(), stored);
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
}

#[test]
fn superseded_canonical_terminal_refuses_current_live_record_authority() {
    const SOURCE: &str = "html/flex/fri07_collapsed_strut_wrapping.html";
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = owner.join(authority::ARTIFACT_ROOT);
    let retained = fs::read(root.join("corpus.toml")).unwrap();
    let input = canonical_input(
        &root,
        &retained,
        PublishedEdition::CanonicalBoxes,
        InputAccess::ReadOnly,
    )
    .unwrap();
    let predecessor = input
        .declaration
        .records
        .iter()
        .find(|record| record.source.as_str() == SOURCE)
        .unwrap();
    let current = Declaration::parse(DECLARATION).unwrap();
    let live = current
        .records
        .iter()
        .find(|record| record.source.as_str() == SOURCE)
        .unwrap();
    assert_eq!(predecessor.source_sha256, live.source_sha256);
    assert_eq!(predecessor.historical_sha256, live.historical_sha256);
    assert_ne!(
        serde_json::to_value(&predecessor.semantic_specifications).unwrap(),
        serde_json::to_value(&live.semantic_specifications).unwrap()
    );
    let selected = BTreeMap::from([(
        SOURCE.to_owned(),
        serde_json::to_value(predecessor).unwrap(),
    )]);
    let refused = current.check_live_records(&selected).unwrap_err();
    assert!(refused.to_string().contains("selected live record changed"));
    assert!(refused.to_string().contains(SOURCE));
    assert_eq!(fs::read(root.join("corpus.toml")).unwrap(), retained);
}
