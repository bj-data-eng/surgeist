use super::*;

const CAPTURE_REPORT: &str = "xml/generation-reports/all.json";
const CAPTURE_REPORT_SHA256: &str =
    "2e8f353f2661e99cf48f0caab02c4149dd307f5438988d12baf51fe08d9985ca";
const CAPTURE_DECLARATION_LENGTH: usize = 64333;
const CAPTURE_DECLARATION_SHA256: &str =
    "c2102dfe54a3c9ff9a0a93f696e889e01654a9bcd0e695146f0b007e40c3d174";

#[test]
fn prepared_range_qualification_accepts_only_the_declared_rtl_pdi_differences() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let result = qualify_retained_prepared_range(owner, &invocation.root, DECLARATION, |_| Ok(()));
    invocation.check_terminal().unwrap();
    fs::remove_dir_all(&invocation.root).unwrap();
    result.expect("the two adopted RTL PDI differences qualify; the two LTR controls match");
}

#[test]
fn prepared_range_receipt_replays_after_unrelated_catalogue_append() {
    let mut current = DECLARATION.to_vec();
    current.extend_from_slice(
        br#"

[[records]]
source = "html/qualification/unrelated-prepared-range-replay-stimulus.html"
source_sha256 = "e8d4898ce6c699e0f6264585e6f95f481ad35d1e34e66c458ada2117ebc615c1"
source_kind = "authored"
[records.expected]
kind = "unsupported"
reason = "Root context: AuthoredBodyOrHtmlStyle"
"#,
    );
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let result = qualify_retained_prepared_range(owner, &invocation.root, &current, |_| Ok(()));
    invocation.check_terminal().unwrap();
    fs::remove_dir_all(&invocation.root).unwrap();
    result.expect("unrelated records leave the prepared Range receipt valid");
}

#[test]
fn prepared_range_replay_refuses_a_resealed_capture_with_changed_root_width() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let result = qualify_retained_prepared_range(owner, &invocation.root, DECLARATION, |root| {
        let xml_path = root.join("xml/block/prepared_text_continuity_bidi__border_box_ltr.xml");
        let xml = fs::read_to_string(&xml_path)?;
        let changed = xml
            .replace("containing-width=\"800px\"", "containing-width=\"801px\"")
            .replace(
                "fallback-icb-width=\"800px\"",
                "fallback-icb-width=\"801px\"",
            );
        assert_ne!(xml, changed);
        fs::write(xml_path, &changed)?;
        let xml_hash = Sha256Digest::from_bytes(xml.as_bytes());
        let changed_xml_hash = Sha256Digest::from_bytes(changed.as_bytes());
        let report_path = root.join(CAPTURE_REPORT);
        let report = fs::read_to_string(&report_path)?;
        let changed_report = report.replace(xml_hash.as_str(), changed_xml_hash.as_str());
        assert_ne!(report, changed_report);
        fs::write(report_path, &changed_report)?;
        let ownership_path = root.join("xml/generation-ownership.json");
        let ownership = fs::read_to_string(&ownership_path)?;
        let report_hash = Sha256Digest::from_bytes(report.as_bytes());
        let changed_report_hash = Sha256Digest::from_bytes(changed_report.as_bytes());
        fs::write(
            ownership_path,
            ownership
                .replace(xml_hash.as_str(), changed_xml_hash.as_str())
                .replace(report_hash.as_str(), changed_report_hash.as_str()),
        )?;
        Ok(())
    });
    invocation.check_terminal().unwrap();
    fs::remove_dir_all(&invocation.root).unwrap();
    let error = result.expect_err("a self-consistent replacement cannot change the captured root");
    assert!(
        error
            .to_string()
            .contains("retained prepared Range capture report changed")
    );
}

fn qualify_retained_prepared_range(
    owner: &Path,
    root: &Path,
    current: &[u8],
    alter_outputs: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let source = crate::parity_support::prepared_range_qualification::SOURCE;
    let mode = QualificationMode::PreparedRangeCapture;
    let receipt = owner.join("tests/layout/browser_parity/captured/prepared-range-differences");
    let mut input = DeclarationInput::sealed_prefix(
        current,
        CAPTURE_DECLARATION_LENGTH,
        CAPTURE_DECLARATION_SHA256,
    )?;
    select_records(&mut input.declaration, Some(source), mode)?;
    let expected = input.declaration.expected()?;
    let inventory = inventory_with_retained_helper(
        root,
        owner,
        &input,
        mode,
        InputAccess::Prepare,
        Some(include_bytes!(
            "../../tests/layout/browser_parity/captured/prepared-range-differences/scripts/gentest/test_helper.js"
        )),
    )?;

    for relative in inventory
        .cases
        .iter()
        .map(|case| case.case.output().as_str())
        .chain([CAPTURE_REPORT, "xml/generation-ownership.json"])
    {
        let destination = root.join(relative);
        fs::create_dir_all(destination.parent().expect("output has a parent"))?;
        fs::copy(receipt.join(relative), destination)?;
    }
    alter_outputs(root)?;
    let report_bytes = fs::read(root.join(CAPTURE_REPORT))?;
    if Sha256Digest::from_bytes(&report_bytes).as_str() != CAPTURE_REPORT_SHA256 {
        return Err("retained prepared Range capture report changed".into());
    }
    let browser_declaration: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        &fs::read_to_string(root.join("browser-owner.toml"))?,
        "browser-owner.toml",
    )?;
    let corpus = corpus_for_inventory(
        root,
        owner,
        browser_declaration.browser.settings()?,
        &inventory,
        true,
        expected,
    )?;
    let report = browser::check_corpus(&corpus, &QualificationAdapter::for_mode(mode))?;
    let mut raw = Vec::new();
    validate_corpus(
        root,
        &inventory,
        &corpus,
        expected,
        &report,
        Validation {
            mode,
            filesystem_tampering: false,
            lineage: SemanticLineage::Focused,
            requirements: None,
            terminal_cases: None,
            diagnostic_raw: None,
            geometry_predecessors: BTreeMap::new(),
            policy_raw: &mut raw,
        },
    )?;
    if raw != fs::read(receipt.join("raw-comparisons.jsonl"))? {
        return Err("prepared Range replay differs from the retained raw comparisons".into());
    }
    Ok(())
}

#[test]
fn specialized_sources_are_selected_only_for_their_capture_profiles() {
    let profiles = [
        (
            crate::parity_support::prepared_range_qualification::SOURCE,
            QualificationMode::PreparedRangeCapture,
        ),
        (
            FLEX_COMPOSITION_SOURCE,
            QualificationMode::FlexCompositionCapture,
        ),
    ];
    let mut ordinary = DeclarationInput::parse(DECLARATION).unwrap();
    select_records(&mut ordinary.declaration, None, QualificationMode::Regular).unwrap();

    for (source, mode) in profiles {
        assert!(
            ordinary
                .declaration
                .records
                .iter()
                .all(|record| record.source.as_str() != source)
        );

        let mut selected = DeclarationInput::parse(DECLARATION).unwrap();
        select_records(&mut selected.declaration, Some(source), mode).unwrap();
        assert_eq!(selected.declaration.records.len(), 1);
        assert_eq!(selected.declaration.records[0].source.as_str(), source);

        let mut wrong_profile = DeclarationInput::parse(DECLARATION).unwrap();
        assert!(
            select_records(
                &mut wrong_profile.declaration,
                Some(source),
                QualificationMode::Regular,
            )
            .is_err()
        );
        assert!(select_records(&mut wrong_profile.declaration, None, mode).is_err());
    }
}
