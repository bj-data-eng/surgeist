use super::*;

#[test]
fn prepared_range_qualification_accepts_only_the_declared_rtl_pdi_differences() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let result = qualify_retained_prepared_range(owner, &invocation.root, DECLARATION);
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
    let result = qualify_retained_prepared_range(owner, &invocation.root, &current);
    invocation.check_terminal().unwrap();
    fs::remove_dir_all(&invocation.root).unwrap();
    result.expect("unrelated records leave the prepared Range receipt valid");
}

fn qualify_retained_prepared_range(owner: &Path, root: &Path, current: &[u8]) -> Result<()> {
    let source = "html/block/prepared_text_continuity_bidi.html";
    let mode = QualificationMode::PreparedRangeCapture;
    let receipt = owner.join("tests/layout/browser_parity/captured/prepared-range-differences");
    let mut input = DeclarationInput::parse(current)?;
    select_records(&mut input.declaration, Some(source), mode)?;
    let expected = input.declaration.expected()?;
    let inventory = prepare_inventory(root, owner, &input, mode)?;

    for relative in inventory
        .cases
        .iter()
        .map(|case| case.case.output().as_str())
        .chain([
            "xml/generation-reports/all.json",
            "xml/generation-ownership.json",
        ])
    {
        let destination = root.join(relative);
        fs::create_dir_all(destination.parent().expect("output has a parent"))?;
        fs::copy(receipt.join(relative), destination)?;
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
        },
    )
}
