use super::*;

const SOURCE: &str = FLEX_COMPOSITION_SOURCE;
const REPORT: &str = "xml/generation-reports/all.json";
const RECEIPT: &[u8] =
    include_bytes!("../../tests/layout/browser_parity/flex_composition_focused_report.json");
const OWNERSHIP: &[u8] =
    include_bytes!("../../tests/layout/browser_parity/flex_composition_focused_ownership.json");

fn settings() -> browser::BrowserSettings {
    let bytes = INPUTS
        .iter()
        .find(|(name, _)| *name == "browser-owner.toml")
        .unwrap()
        .1;
    let declaration: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(bytes).unwrap(),
        "browser-owner.toml",
    )
    .unwrap();
    declaration.browser.settings().unwrap()
}

struct ReceiptRoot(Invocation);
impl Drop for ReceiptRoot {
    fn drop(&mut self) {
        self.0.check_terminal().unwrap();
        fs::remove_dir_all(&self.0.root).unwrap();
    }
}

fn prepared(
    root: &Path,
    bytes: &[u8],
    source: &str,
    mode: QualificationMode,
) -> Result<(Inventory, BrowserCorpus)> {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = DeclarationInput::focused(bytes, source, mode)?;
    let mode = input.execution_mode(mode);
    let inventory = inventory_with_retained_helper(
        root,
        owner,
        &input,
        mode,
        InputAccess::Prepare,
        Some(include_bytes!(
            "../../tests/layout/browser_parity/captured/scriptless-order/scripts/gentest/test_helper.js"
        )),
    )?;
    let corpus = corpus_for_inventory(
        root,
        owner,
        settings(),
        &inventory,
        true,
        input.declaration.expected()?,
    )?;
    Ok((inventory, corpus))
}

fn flex_receipt() -> (ReceiptRoot, Inventory, BrowserCorpus) {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = ReceiptRoot(Invocation::new(owner).unwrap());
    let (inventory, corpus) = prepared(
        &root.0.root,
        DECLARATION,
        SOURCE,
        QualificationMode::Regular,
    )
    .unwrap();
    for case in &inventory.cases {
        let output = case.case.output().as_str();
        let destination = root.0.root.join(output);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(
            owner
                .join("tests/layout/browser_parity/captured/flex-composition-differences")
                .join(output),
            destination,
        )
        .unwrap();
    }
    fs::create_dir_all(root.0.root.join("xml/generation-reports")).unwrap();
    fs::write(root.0.root.join(REPORT), RECEIPT).unwrap();
    fs::write(root.0.root.join("xml/generation-ownership.json"), OWNERSHIP).unwrap();
    (root, inventory, corpus)
}

#[test]
fn focused_contract_actual_flex_receipt_accepts_unrelated_append_and_checks_complete_semantics() {
    let (root, inventory, before) = flex_receipt();
    let mode = QualificationMode::FlexCompositionCapture;
    let adapter = QualificationAdapter::for_mode(mode);
    let report = browser::check_corpus(&before, &adapter).unwrap();
    assert_eq!(report.to_bytes().unwrap(), RECEIPT);
    let appended = [
        DECLARATION,
        br#"
[[records]]
source = "html/qualification/unrelated-receipt.html"
source_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
source_kind = "authored"
[records.expected]
kind = "unsupported"
reason = "Root context: AuthoredBodyOrHtmlStyle"
"#,
    ]
    .concat();
    let (_, after) = prepared(&root.0.root, &appended, SOURCE, QualificationMode::Regular).unwrap();
    assert_eq!(
        browser::check_corpus(&after, &adapter)
            .unwrap()
            .to_bytes()
            .unwrap(),
        RECEIPT
    );
    let mut raw = Vec::new();
    validate_corpus(
        &root.0.root,
        &inventory,
        &before,
        *report.summary(),
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
    )
    .unwrap();
    assert_eq!(raw, fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity/captured/flex-composition-differences/raw-comparisons.jsonl")).unwrap());
}

#[test]
fn focused_contract_actual_receipt_refuses_selected_resource_request_policy_browser_and_output_changes()
 {
    let (root, mut inventory, corpus) = flex_receipt();
    let mode = QualificationMode::FlexCompositionCapture;
    let adapter = QualificationAdapter::for_mode(mode);
    let changed = super::focused_declaration_tests::edit_record(
        DECLARATION,
        SOURCE,
        "inline_offset = 0",
        "inline_offset = 1",
    );
    let (_, selected) = prepared(&root.0.root, &changed, SOURCE, mode).unwrap();
    assert!(browser::check_corpus(&selected, &adapter).is_err());
    prepared(&root.0.root, DECLARATION, SOURCE, mode).unwrap();
    for path in [
        "scripts/gentest/test_base_style.css",
        "observation-request.json",
    ]
    .into_iter()
    .chain(
        flex_composition_qualification::input_pins()
            .iter()
            .map(|(path, _)| *path),
    ) {
        let file = root.0.root.join(path);
        let original = fs::read(&file).unwrap();
        fs::write(&file, b"changed input\n").unwrap();
        assert!(browser::check_corpus(&corpus, &adapter).is_err(), "{path}");
        fs::write(&file, original).unwrap();
        assert!(
            browser::check_corpus(&corpus, &adapter).is_ok(),
            "restored {path}"
        );
    }
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut browser = serde_json::to_value(settings()).unwrap();
    browser["launch"]["batch_size"] = serde_json::json!(1);
    let changed_browser = corpus_for_inventory(
        &root.0.root,
        owner,
        serde_json::from_value(browser).unwrap(),
        &inventory,
        true,
        BrowserReportSummary::new(4, 0, 0, 0, 0).unwrap(),
    )
    .unwrap();
    assert!(browser::check_corpus(&changed_browser, &adapter).is_err());
    let fixture = &inventory.fixtures[0];
    inventory.fixtures[0] = FixtureSpec::new(
        fixture.name().to_owned(),
        fixture.source().clone(),
        fixture.cases()[..3].to_vec(),
        fixture.status().clone(),
    )
    .unwrap();
    let missing_output = corpus_for_inventory(
        &root.0.root,
        owner,
        settings(),
        &inventory,
        true,
        BrowserReportSummary::new(3, 0, 0, 0, 0).unwrap(),
    )
    .unwrap();
    assert!(browser::check_corpus(&missing_output, &adapter).is_err());
}
