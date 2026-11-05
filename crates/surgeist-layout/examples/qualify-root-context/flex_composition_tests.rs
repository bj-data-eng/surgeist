use super::*;

const REPORT: &str = "xml/generation-reports/all.json";
const REPORT_SHA256: &str = "65bb238d03aae80b366fa1f481be42137799466f74c1a35fafdb40d2460478a2";
const DECLARATION_LENGTH: usize = 65_209;
const DECLARATION_SHA256: &str = "fe930e781f5391a427c9142bdf2161368759febee658f0da6614fe5fb87a0848";

#[test]
fn retained_sideways_flex_replay_qualifies_all_four_variants() {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invocation = Invocation::new(owner).unwrap();
    let result = replay(owner, &invocation.root, DECLARATION);
    invocation.check_terminal().unwrap();
    fs::remove_dir_all(&invocation.root).unwrap();
    let raw = String::from_utf8(result.expect("the four declared variants qualify")).unwrap();
    assert_eq!(raw.lines().count(), 4);
}

fn replay(owner: &Path, root: &Path, current: &[u8]) -> Result<Vec<u8>> {
    let mode = QualificationMode::FlexCompositionCapture;
    let receipt = owner.join("tests/layout/browser_parity/captured/flex-composition-differences");
    let mut input =
        DeclarationInput::sealed_prefix(current, DECLARATION_LENGTH, DECLARATION_SHA256)?;
    select_records(&mut input.declaration, Some(FLEX_COMPOSITION_SOURCE), mode)?;
    let expected = input.declaration.expected()?;
    let inventory = prepare_inventory(root, owner, &input, mode)?;

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
            prepared_range_raw: &mut raw,
        },
    )?;
    Ok(raw)
}
