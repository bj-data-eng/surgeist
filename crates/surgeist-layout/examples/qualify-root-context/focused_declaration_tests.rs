use super::*;

const BASIC: &str = "html/block/block_basic.html";
const CHAIN: &str = "html/qualification/collapsed-leading-whitespace.html";
const UNRELATED: &str = r#"

[[records]]
source = "html/qualification/unrelated-focused-declaration.html"
source_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
source_kind = "authored"
[records.expected]
kind = "unsupported"
reason = "Root context: AuthoredBodyOrHtmlStyle"
"#;

struct TemporaryCorpus(PathBuf);
impl Drop for TemporaryCorpus {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[derive(Debug, PartialEq)]
struct Prepared {
    inputs: BTreeMap<RelativePath, Sha256Digest>,
    global_inputs: Vec<RelativePath>,
    jobs: Vec<FixtureSpec>,
}

fn prepared(bytes: &[u8], source: &str, mode: QualificationMode) -> Result<Prepared> {
    let input = DeclarationInput::focused(bytes, source, mode)?;
    let mode = input.execution_mode(mode);
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner)?;
    let root = TemporaryCorpus(invocation.root.clone());
    let inventory = prepare_inventory(&root.0, owner, &input, mode)?;
    invocation.check_terminal()?;
    Ok(Prepared {
        inputs: inventory.expected_inputs,
        global_inputs: inventory.global_inputs,
        jobs: inventory.fixtures,
    })
}

fn appended() -> Vec<u8> {
    [DECLARATION, UNRELATED.as_bytes()].concat()
}

pub(super) fn edit_record(bytes: &[u8], source: &str, from: &str, to: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).unwrap();
    let start = text.find(&format!("source = \"{source}\"")).unwrap();
    let end = text[start..]
        .find("\n[[")
        .map_or(text.len(), |offset| start + offset);
    let record = &text[start..end];
    assert_eq!(record.matches(from).count(), 1);
    [&text[..start], &record.replace(from, to), &text[end..]]
        .concat()
        .into_bytes()
}

fn append_preserves(source: &str, mode: QualificationMode, selected_sources: &[&str]) {
    let before = prepared(DECLARATION, source, mode).unwrap();
    let after = prepared(&appended(), source, mode).unwrap();
    assert_eq!(before.jobs, after.jobs);
    assert_eq!(before.global_inputs, after.global_inputs);
    assert_eq!(
        before
            .jobs
            .iter()
            .map(|job| job.source().as_str())
            .collect::<BTreeSet<_>>(),
        selected_sources.iter().copied().collect::<BTreeSet<_>>(),
    );
    assert_eq!(
        before.inputs, after.inputs,
        "unselected records cannot change focused input identity"
    );
}

#[test]
fn focused_declaration_regular_inputs_and_jobs_survive_unrelated_append() {
    append_preserves(BASIC, QualificationMode::Regular, &[BASIC]);
}

#[test]
fn focused_declaration_flex_profile_inputs_and_jobs_survive_unrelated_append() {
    append_preserves(
        FLEX_COMPOSITION_SOURCE,
        QualificationMode::FlexCompositionCapture,
        &[FLEX_COMPOSITION_SOURCE],
    );
}

#[test]
fn focused_declaration_diagnostic_inputs_keep_transitive_peers_without_unrelated_records() {
    append_preserves(
        CHAIN,
        QualificationMode::ElementScrollSizeDiagnostics,
        &[
            "html/qualification/fixed-grid.html",
            "html/qualification/fixed-inline-grid.html",
            CHAIN,
        ],
    );
}

#[test]
fn focused_declaration_validation_mode_is_bound_even_when_measurement_jobs_match() {
    let regular = prepared(DECLARATION, BASIC, QualificationMode::Regular).unwrap();
    let numeric = prepared(DECLARATION, BASIC, QualificationMode::NumericDiagnostics).unwrap();
    assert_eq!(regular.jobs, numeric.jobs);
    assert_ne!(
        regular.inputs, numeric.inputs,
        "numeric diagnostics and strict qualification have distinct identity"
    );
}

#[test]
fn focused_declaration_selected_metadata_changes_input_identity() {
    let changed = edit_record(DECLARATION, BASIC, "inline_offset = 0", "inline_offset = 1");
    let before = prepared(DECLARATION, BASIC, QualificationMode::Regular).unwrap();
    let after = prepared(&changed, BASIC, QualificationMode::Regular).unwrap();
    assert_eq!(before.jobs, after.jobs);
    assert_ne!(before.inputs, after.inputs);
}

#[test]
fn focused_declaration_required_peer_metadata_changes_input_identity() {
    let changed = edit_record(
        DECLARATION,
        "html/qualification/fixed-grid.html",
        "inline_offset = 0",
        "inline_offset = 1",
    );
    let before = prepared(
        DECLARATION,
        CHAIN,
        QualificationMode::ElementScrollSizeDiagnostics,
    )
    .unwrap();
    let after = prepared(
        &changed,
        CHAIN,
        QualificationMode::ElementScrollSizeDiagnostics,
    )
    .unwrap();
    assert_eq!(before.jobs, after.jobs);
    assert_ne!(before.inputs, after.inputs);
}

#[test]
fn focused_declaration_validates_complete_catalogue_before_selecting_known_source() {
    for suffix in [
        UNRELATED.replace(
            "html/qualification/unrelated-focused-declaration.html",
            BASIC,
        ),
        UNRELATED.replace("unrelated-focused-declaration.html", "block_basic.html"),
        UNRELATED.replace(
            "source_kind = \"authored\"",
            "source_kind = \"authored\"\nunknown_field = true",
        ),
    ] {
        let bytes = [DECLARATION, suffix.as_bytes()].concat();
        assert!(prepared(&bytes, BASIC, QualificationMode::Regular).is_err());
    }
    assert!(
        prepared(
            DECLARATION,
            "html/qualification/undeclared-focus.html",
            QualificationMode::Regular
        )
        .is_err()
    );
}

#[test]
fn focused_declaration_readonly_inventory_refuses_changed_resource_without_repair() {
    let input = DeclarationInput::focused(DECLARATION, BASIC, QualificationMode::Regular).unwrap();
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let root = TemporaryCorpus(invocation.root.clone());
    let before = prepare_inventory(&root.0, owner, &input, QualificationMode::Regular).unwrap();
    let checked = inventory(
        &root.0,
        owner,
        &input,
        QualificationMode::Regular,
        InputAccess::ReadOnly,
    )
    .unwrap();
    assert_eq!(before.expected_inputs, checked.expected_inputs);
    let resource = root.0.join("scripts/gentest/test_base_style.css");
    fs::write(&resource, b"div { width: 1px; }\n").unwrap();
    assert!(
        inventory(
            &root.0,
            owner,
            &input,
            QualificationMode::Regular,
            InputAccess::ReadOnly
        )
        .is_err()
    );
    assert_eq!(fs::read(resource).unwrap(), b"div { width: 1px; }\n");
    invocation.check_terminal().unwrap();
}

#[test]
fn focused_declaration_refuses_lossy_numbers_before_selection() {
    for value in ["nan", "inf", "-inf"] {
        for replacement in [
            format!("inline_offset = {value}"),
            format!(
                "inline_offset = 0\nroot_size = {{ kind = \"fixed\", width = {value}, height = 20 }}"
            ),
            format!(
                "inline_offset = 0\nroot_size = {{ kind = \"fixed\", width = 20, height = {value} }}"
            ),
            format!(
                "inline_offset = 0\nroot_size = {{ kind = \"flex-pair\", item_width = {value}, item_height = 20 }}"
            ),
            format!(
                "inline_offset = 0\nroot_size = {{ kind = \"flex-pair\", item_width = 20, item_height = {value} }}"
            ),
        ] {
            let changed = edit_record(DECLARATION, BASIC, "inline_offset = 0", &replacement);
            for source in [BASIC, FLEX_COMPOSITION_SOURCE] {
                assert!(
                    DeclarationInput::focused(&changed, source, QualificationMode::Regular)
                        .is_err(),
                    "{replacement} must refuse before selecting {source}"
                );
            }
        }
    }
}

#[test]
fn focused_declaration_refuses_selected_and_unselected_optional_nonfinite_offsets() {
    for value in ["nan", "inf", "-inf"] {
        let changed = edit_record(
            DECLARATION,
            BASIC,
            "inline_offset = 0",
            &format!("inline_offset = 0\nblock_offset = {value}"),
        );
        for source in [BASIC, FLEX_COMPOSITION_SOURCE] {
            assert!(
                DeclarationInput::focused(&changed, source, QualificationMode::Regular).is_err(),
                "{value} must refuse before selecting {source}"
            );
        }
    }
}

#[test]
fn focused_declaration_equal_cyclic_closures_keep_requested_roots_distinct() {
    let grid = "html/qualification/fixed-grid.html";
    let inline = "html/qualification/fixed-inline-grid.html";
    let cyclic = edit_record(
        DECLARATION,
        grid,
        "[records.expected]",
        "[records.relation]\npeer = \"fixed-inline-grid\"\nworld_phase_change = false\n[records.expected]",
    );
    let mode = QualificationMode::ElementScrollSizeDiagnostics;
    let first = DeclarationInput::focused(&cyclic, grid, mode).unwrap();
    let second = DeclarationInput::focused(&cyclic, inline, mode).unwrap();
    assert_eq!(
        serde_json::to_value(&first.declaration.records).unwrap(),
        serde_json::to_value(&second.declaration.records).unwrap()
    );
    assert_eq!(first.declaration.records.len(), 2);
    assert_ne!(first.bytes(), second.bytes());
    let a: serde_json::Value = serde_json::from_slice(first.bytes()).unwrap();
    let b: serde_json::Value = serde_json::from_slice(second.bytes()).unwrap();
    assert_eq!(a["requested_source"], grid);
    assert_eq!(b["requested_source"], inline);
}

#[test]
fn focused_declaration_aliases_normalize_before_binding_profile_and_bytes() {
    let implicit = DeclarationInput::focused(
        DECLARATION,
        FLEX_COMPOSITION_SOURCE,
        QualificationMode::Regular,
    )
    .unwrap();
    let explicit = DeclarationInput::focused(
        DECLARATION,
        FLEX_COMPOSITION_SOURCE,
        QualificationMode::FlexCompositionCapture,
    )
    .unwrap();
    assert_eq!(implicit.bytes(), explicit.bytes());
    assert_eq!(
        implicit.execution_mode(QualificationMode::Regular),
        QualificationMode::FlexCompositionCapture
    );
    let value: serde_json::Value = serde_json::from_slice(implicit.bytes()).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["mode"], "flex-composition-capture");
    assert_eq!(value["requested_source"], FLEX_COMPOSITION_SOURCE);
    assert_eq!(
        value["records"],
        serde_json::to_value(&implicit.declaration.records).unwrap()
    );
    assert_eq!(implicit.declaration_path(), FOCUSED_DECLARATION);
}

#[test]
fn focused_declaration_execution_refuses_mode_or_typed_record_identity_mismatch() {
    let mut input =
        DeclarationInput::focused(DECLARATION, BASIC, QualificationMode::Regular).unwrap();
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let root = TemporaryCorpus(invocation.root.clone());
    let error = prepare_inventory(
        &root.0,
        owner,
        &input,
        QualificationMode::NumericDiagnostics,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("mode differs from execution"));
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 0);
    input.declaration.records[0].source_sha256 = Sha256Digest::from_bytes(b"changed record");
    let error = prepare_inventory(&root.0, owner, &input, QualificationMode::Regular)
        .err()
        .unwrap();
    assert!(
        error
            .to_string()
            .contains("records differ from their identity")
    );
    assert_eq!(fs::read_dir(&root.0).unwrap().count(), 0);
    invocation.check_terminal().unwrap();
}
