use super::*;

const RECEIPT: &str = "tests/layout/browser_parity/captured/flex-composition-differences";
const REPORT: &str = "xml/generation-reports/all.json";
const REPORT_SHA256: &str = "65bb238d03aae80b366fa1f481be42137799466f74c1a35fafdb40d2460478a2";
const DECLARATION_LENGTH: usize = 65_209;
const DECLARATION_SHA256: &str = "fe930e781f5391a427c9142bdf2161368759febee658f0da6614fe5fb87a0848";
// The focused receipt pins this producer helper but did not retain its own copy.
// The durable edition preserves the exact bytes bound by that report input.
const RETAINED_HELPER: &[u8] = include_bytes!(
    "../../tests/layout/browser_parity/captured/flex-composition-qualification/scripts/gentest/test_helper.js"
);

#[test]
fn retained_sideways_flex_replay_qualifies_all_four_variants() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let result = replay(owner, &invocation.root, DECLARATION);
    invocation.check_terminal().unwrap();
    fs::remove_dir_all(&invocation.root).unwrap();
    let raw = result.expect("the four declared variants qualify");
    assert_eq!(
        raw,
        fs::read(owner.join(RECEIPT).join("raw-comparisons.jsonl")).unwrap()
    );
}

fn replay(owner: &Path, root: &Path, current: &[u8]) -> Result<Vec<u8>> {
    let mode = QualificationMode::FlexCompositionCapture;
    let receipt = owner.join(RECEIPT);
    let mut input =
        DeclarationInput::sealed_prefix(current, DECLARATION_LENGTH, DECLARATION_SHA256)?;
    select_records(&mut input.declaration, Some(FLEX_COMPOSITION_SOURCE), mode)?;
    let expected = input.declaration.expected()?;
    let receipt_report = fs::read(receipt.join(REPORT))?;
    if Sha256Digest::from_bytes(&receipt_report).as_str() != REPORT_SHA256 {
        return Err("retained Flex composition capture report changed".into());
    }
    let report_value: serde_json::Value = serde_json::from_slice(&receipt_report)?;
    let report_inputs: BTreeMap<RelativePath, Sha256Digest> =
        serde_json::from_value(report_value["metadata"]["inputs"].clone())?;
    let helper = RelativePath::new(capture_authority::ROOT_HELPER_INPUT)?;
    let retained_digest = Sha256Digest::from_bytes(RETAINED_HELPER);
    if report_inputs.get(&helper) != Some(&retained_digest) {
        return Err("retained Flex composition helper differs from capture report".into());
    }
    let prepared = prepare_inventory(root, owner, &input, mode)?;
    assert_ne!(
        prepared.expected_inputs.get(&helper),
        Some(&retained_digest)
    );
    let mut expected_after_helper = prepared.expected_inputs.clone();
    expected_after_helper.insert(helper.clone(), retained_digest);
    assert_eq!(expected_after_helper, report_inputs);
    fs::write(root.join(helper.as_str()), RETAINED_HELPER)?;
    let inventory = inventory(root, owner, &input, mode, InputAccess::ReadOnly)?;
    assert_eq!(inventory.expected_inputs, report_inputs);

    for relative in inventory
        .cases
        .iter()
        .map(|case| case.case.output().as_str())
        .chain([REPORT, "xml/generation-ownership.json"])
    {
        let destination = root.join(relative);
        fs::create_dir_all(destination.parent().expect("output has a parent"))?;
        fs::copy(receipt.join(relative), destination)?;
    }
    let report_bytes = fs::read(root.join(REPORT))?;
    if Sha256Digest::from_bytes(&report_bytes).as_str() != REPORT_SHA256 {
        return Err("retained Flex composition capture report changed".into());
    }
    let owner_manifest: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        &fs::read_to_string(root.join("browser-owner.toml"))?,
        "browser-owner.toml",
    )?;
    let corpus = corpus_for_inventory(
        root,
        owner,
        owner_manifest.browser.settings()?,
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
            geometry_predecessors: BTreeMap::new(),
            policy_raw: &mut raw,
        },
    )?;
    Ok(raw)
}
