//! Invocation-owned corpus and assertions; no historical corpus is rewritten.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use surgeist_generator::browser::{
    self, AcquisitionMode, BrowserCorpus, BrowserCorpusAdapter, BrowserLocation,
    BrowserReportSummary, CaseOutcome, CaseSpec, FixtureInput, FixtureSpec, FixtureStatus,
    GenerationError, GenerationRequest, PreparedFixture, ReportScope,
};
use surgeist_generator::{GeneratorErrorKind, RelativePath, Sha256Digest};

use crate::adapter::{AdapterError, LayoutAdapter, fixture_cases};
use crate::browser_profile::BrowserManifest;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const DECLARATION: &[u8] = include_bytes!("corpus.toml");
const INPUTS: &[(&str, &[u8])] = &[
    (
        "scripts/gentest/test_helper.js",
        include_bytes!("../../tests/layout/browser_parity/scripts/gentest/test_helper.js"),
    ),
    (
        "scripts/gentest/test_base_style.css",
        include_bytes!("../../tests/layout/browser_parity/scripts/gentest/test_base_style.css"),
    ),
    (
        "fonts/Ahem.ttf",
        include_bytes!("../../tests/layout/browser_parity/fonts/Ahem.ttf"),
    ),
    (
        "browser-owner.toml",
        include_bytes!("../../tests/layout/browser_parity/corpus.toml"),
    ),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    schema_version: u8,
    records: Vec<Record>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    source: RelativePath,
    source_sha256: Sha256Digest,
    expected_hypothetical_display: String,
    historical_sha256: BTreeMap<String, Sha256Digest>,
}

struct ExpectedCase {
    source: RelativePath,
    case: CaseSpec,
    historical: RelativePath,
    hypothetical_display: String,
}

struct Inventory {
    fixtures: Vec<FixtureSpec>,
    cases: Vec<ExpectedCase>,
    global_inputs: Vec<RelativePath>,
    expected_inputs: BTreeMap<RelativePath, Sha256Digest>,
}

impl Declaration {
    fn expected(&self) -> Result<BrowserReportSummary> {
        if self.records.is_empty() {
            return Err("root-context qualification requires source records".into());
        }
        let generated = self
            .records
            .len()
            .checked_mul(fixture_cases().len())
            .ok_or("qualification case count overflow")?;
        Ok(BrowserReportSummary::new(generated, 0, 0, 0, 0)?)
    }
}

// This intentionally projects only the browser declaration. It does not claim
// to validate the owning corpus's Taffy import or complete fixture inventory.
#[derive(Deserialize)]
struct OwnerBrowserDeclaration {
    browser: BrowserManifest,
}

struct QualificationAdapter;
impl BrowserCorpusAdapter for QualificationAdapter {
    type Error = AdapterError;

    fn prepare(
        &self,
        input: FixtureInput<'_>,
    ) -> std::result::Result<PreparedFixture, Self::Error> {
        LayoutAdapter.prepare(input)
    }
    fn lower(
        &self,
        fixture: &FixtureSpec,
        measurement: serde_json::Value,
    ) -> std::result::Result<Vec<CaseOutcome>, Self::Error> {
        let outcomes = LayoutAdapter.lower(fixture, measurement)?;
        for outcome in &outcomes {
            if let CaseOutcome::Unsupported { case_id, reason } = outcome {
                eprintln!("qualify-root-context: {case_id}: Unsupported: {reason}");
            }
        }
        Ok(outcomes)
    }
}

pub(super) fn run() -> Result<()> {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let declaration: Declaration =
        surgeist_generator::parse_manifest(std::str::from_utf8(DECLARATION)?, "corpus.toml")?;
    assert_eq!(declaration.schema_version, 2);
    let _ = declaration.expected()?;
    let owner_bytes = INPUTS
        .iter()
        .find(|(name, _)| *name == "browser-owner.toml")
        .expect("browser declaration")
        .1;
    let owner_declaration: OwnerBrowserDeclaration = surgeist_generator::parse_manifest(
        std::str::from_utf8(owner_bytes)?,
        "browser-owner.toml",
    )?;
    owner_declaration.browser.validate()?;
    let settings = owner_declaration.browser.settings()?;
    let browser_path = browser::acquire_browser(owner, &settings, AcquisitionMode::ExistingOnly)?;
    let invocation = Invocation::new(owner)?;
    eprintln!(
        "qualify-root-context: invocation {}",
        invocation.root.display()
    );
    let result = qualify(&invocation.root, owner, settings, browser_path, declaration);
    // Generation owns and joins its supervisor. Never remove outstanding profile
    // journals, even on an assertion/generation failure: preserve them for recovery.
    let terminal = invocation.check_terminal();
    if let Err(error) = terminal {
        eprintln!(
            "qualify-root-context: retained {}: {error}",
            invocation.root.display()
        );
        return Err(error);
    }
    match result {
        Ok(()) => fs::remove_dir_all(&invocation.root).map_err(Into::into),
        Err(error) => {
            eprintln!(
                "qualify-root-context: failed corpus retained at {}",
                invocation.root.display()
            );
            Err(error)
        }
    }
}

struct Invocation {
    root: PathBuf,
}
impl Invocation {
    fn new(owner: &Path) -> Result<Self> {
        let parent = owner.join("tmp/qualify-root-context");
        fs::create_dir_all(&parent)?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = parent.join(format!("{}-{nonce}", std::process::id()));
        fs::create_dir(&root)?;
        Ok(Self { root })
    }
    fn check_terminal(&self) -> Result<()> {
        let profiles = self.root.join(".surgeist-generator/profiles/browser");
        match fs::read_dir(&profiles) {
            Ok(mut entries) => {
                if entries.next().transpose()?.is_some() {
                    return Err(format!(
                        "outstanding owned browser profiles at {}",
                        profiles.display()
                    )
                    .into());
                }
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

fn qualify(
    root: &Path,
    owner: &Path,
    settings: browser::BrowserSettings,
    browser_path: RelativePath,
    declaration: Declaration,
) -> Result<()> {
    let expected = declaration.expected()?;
    let inventory = prepare_inventory(root, owner, &declaration.records)?;
    let corpus = BrowserCorpus::new(
        BrowserLocation::new(owner, root)?,
        RelativePath::new("corpus.toml")?,
        "surgeist-layout-root-context-qualification".into(),
        RelativePath::new("xml")?,
        vec![ReportScope::new(
            RelativePath::new("generation-reports/all.json")?,
            None,
        )?],
        settings,
        inventory.fixtures,
        inventory.global_inputs,
        Vec::new(),
    )?
    .with_expected_inputs(inventory.expected_inputs)?
    .with_exact_input_roots(vec![
        RelativePath::new("scripts/gentest")?,
        RelativePath::new("fonts")?,
        RelativePath::new("historical")?,
    ])?
    .with_expected_counts(expected);
    let request = GenerationRequest::new(corpus.clone(), browser_path, None, None)?;
    let report = browser::generate(request, QualificationAdapter)?;
    assert_eq!(*report.summary(), expected);
    assert!(report.filter().is_none());
    assert_eq!(report.generated().len(), inventory.cases.len());
    assert_eq!(
        fs::read(root.join("xml/generation-reports/all.json"))?,
        report.to_bytes()?
    );
    assert_eq!(
        browser::check_corpus(&corpus, &LayoutAdapter)?.to_bytes()?,
        report.to_bytes()?
    );

    // Qualify every retained observation before requiring the new certificate.
    // Historical fields are assertion inputs only; none enter Layout or capture.
    for artifact in report.generated() {
        let case = inventory
            .cases
            .iter()
            .find(|case| case.case.id() == artifact.name())
            .expect("declared generated case");
        assert_eq!(artifact.source(), &case.source);
        assert_eq!(artifact.output(), case.case.output());
        assert_eq!(artifact.variant(), case.case.variant());
        let bytes = fs::read(root.join(artifact.output().as_str()))?;
        assert_eq!(Sha256Digest::from_bytes(&bytes), *artifact.output_sha256());
        let historical = fs::read_to_string(root.join(case.historical.as_str()))?;
        qualify_historical_geometry(std::str::from_utf8(&bytes)?, &historical, case.case.id())?;
    }
    eprintln!(
        "qualify-root-context: all {} source-bound historical expectation trees preserved",
        inventory.cases.len()
    );
    for artifact in report.generated() {
        let case = inventory
            .cases
            .iter()
            .find(|case| case.case.id() == artifact.name())
            .expect("declared generated case");
        let xml = fs::read_to_string(root.join(artifact.output().as_str()))?;
        qualify_xml(&xml, case)?;
    }
    let first = inventory
        .cases
        .first()
        .expect("nonempty qualified inventory");
    qualify_tampering(root, &corpus, &first.source, first.case.output())?;
    assert_eq!(
        browser::check_corpus(&corpus, &LayoutAdapter)?.to_bytes()?,
        report.to_bytes()?
    );
    eprintln!(
        "qualify-root-context: {} Generated V4 local cases, historical geometry and three filesystem tamper refusals qualified",
        inventory.cases.len()
    );
    Ok(())
}

fn write_input(
    root: &Path,
    path: &RelativePath,
    bytes: &[u8],
    expected: &mut BTreeMap<RelativePath, Sha256Digest>,
) -> Result<()> {
    let destination = root.join(path.as_str());
    fs::create_dir_all(destination.parent().expect("input parent"))?;
    fs::write(destination, bytes)?;
    assert!(
        expected
            .insert(path.clone(), Sha256Digest::from_bytes(bytes))
            .is_none(),
        "duplicate qualification input"
    );
    Ok(())
}

fn prepare_inventory(root: &Path, owner: &Path, records: &[Record]) -> Result<Inventory> {
    let mut inventory = Inventory {
        fixtures: Vec::new(),
        cases: Vec::new(),
        global_inputs: Vec::new(),
        expected_inputs: BTreeMap::new(),
    };
    for (name, bytes) in INPUTS
        .iter()
        .copied()
        .chain(std::iter::once(("corpus.toml", DECLARATION)))
    {
        let path = RelativePath::new(name)?;
        write_input(root, &path, bytes, &mut inventory.expected_inputs)?;
        if name != "corpus.toml" {
            inventory.global_inputs.push(path);
        }
    }
    let original = owner.join("tests/layout/browser_parity");
    for record in records {
        let source = fs::read(original.join(record.source.as_str()))?;
        assert_eq!(
            Sha256Digest::from_bytes(&source),
            record.source_sha256,
            "original source changed: {}",
            record.source.as_str()
        );
        write_input(
            root,
            &record.source,
            &source,
            &mut inventory.expected_inputs,
        )?;
        let family = record
            .source
            .as_str()
            .strip_prefix("html/")
            .and_then(|path| path.strip_suffix(".html"))
            .ok_or("qualification source must be an original html/*.html path")?;
        let stem = family.rsplit('/').next().ok_or("missing source stem")?;
        assert_eq!(record.historical_sha256.len(), fixture_cases().len());
        let mut cases = Vec::new();
        for (variant, _) in fixture_cases() {
            let output = RelativePath::new(format!("xml/{family}__{variant}.xml"))?;
            let historical = RelativePath::new(format!("historical/{family}__{variant}.xml"))?;
            let bytes = fs::read(original.join(output.as_str()))?;
            let digest = record
                .historical_sha256
                .get(variant)
                .ok_or("missing historical variant digest")?;
            assert_eq!(
                &Sha256Digest::from_bytes(&bytes),
                digest,
                "historical observation changed: {}",
                output.as_str()
            );
            write_input(root, &historical, &bytes, &mut inventory.expected_inputs)?;
            inventory.global_inputs.push(historical.clone());
            let case = CaseSpec::new(format!("{stem}__{variant}"), variant.into(), output)?;
            inventory.cases.push(ExpectedCase {
                source: record.source.clone(),
                case: case.clone(),
                historical,
                hypothetical_display: record.expected_hypothetical_display.clone(),
            });
            cases.push(case);
        }
        inventory.fixtures.push(FixtureSpec::new(
            stem.into(),
            record.source.clone(),
            cases,
            FixtureStatus::Active,
        )?);
    }
    Ok(inventory)
}

fn qualify_historical_geometry(current: &str, historical: &str, case_id: &str) -> Result<()> {
    let current = roxmltree::Document::parse(current)?;
    let historical = roxmltree::Document::parse(historical)?;
    assert_eq!(current.root_element().attribute("name"), Some(case_id));
    assert_eq!(historical.root_element().attribute("name"), Some(case_id));
    assert_eq!(
        current.root_element().attribute("use-rounding"),
        historical.root_element().attribute("use-rounding")
    );
    let current = current
        .root_element()
        .children()
        .find(|node| node.has_tag_name("expectations"))
        .expect("current expectations");
    let historical = historical
        .root_element()
        .children()
        .find(|node| node.has_tag_name("expectations"))
        .expect("historical expectations");
    compare_observation_tree(current, historical, case_id);
    Ok(())
}

fn compare_observation_tree(
    current: roxmltree::Node<'_, '_>,
    historical: roxmltree::Node<'_, '_>,
    path: &str,
) {
    assert_eq!(
        current.tag_name().name(),
        historical.tag_name().name(),
        "{path}: observation category"
    );
    assert_eq!(
        current.tag_name().namespace(),
        historical.tag_name().namespace(),
        "{path}: observation namespace"
    );
    let attributes = |node: roxmltree::Node<'_, '_>| {
        node.attributes()
            .map(|attribute| (attribute.name().to_owned(), attribute.value().to_owned()))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(
        attributes(current),
        attributes(historical),
        "{path}: exact historical observation fields"
    );
    for node in [current, historical] {
        assert!(
            node.children()
                .filter(roxmltree::Node::is_text)
                .all(|text| text.text().is_none_or(|value| value.trim().is_empty())),
            "{path}: unexpected observation text"
        );
    }
    let current = current
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    let historical = historical
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    assert_eq!(
        current.len(),
        historical.len(),
        "{path}: observation cardinality"
    );
    for (index, (current, historical)) in current.into_iter().zip(historical).enumerate() {
        compare_observation_tree(current, historical, &format!("{path}/{index}"));
    }
}

fn qualify_xml(xml: &str, case: &ExpectedCase) -> Result<()> {
    let document = roxmltree::Document::parse(xml)?;
    let test = document.root_element();
    assert!(test.has_tag_name("test"));
    assert_eq!(test.attribute("name"), Some(case.case.id()));
    assert_eq!(
        test.attribute("measurement-version"),
        Some("4"),
        "{}: missing V4 source-qualified local capture",
        case.case.id()
    );
    assert_eq!(
        test.attribute("observation-frame"),
        Some("root-border-local")
    );
    let viewport = test
        .children()
        .find(|node| node.has_tag_name("viewport"))
        .expect("viewport");
    assert_eq!(
        viewport.attribute("root-context"),
        Some("absolute-root-local")
    );
    assert_eq!(viewport.attribute("containing-direction"), Some("ltr"));
    assert_eq!(viewport.attribute("hypothetical-direction"), Some("ltr"));
    assert_eq!(
        viewport.attribute("hypothetical-display"),
        Some(case.hypothetical_display.as_str())
    );
    assert_eq!(
        viewport.attribute("hypothetical-anchor"),
        Some("inline-start")
    );
    assert_eq!(
        viewport.attribute("hypothetical-inline-offset"),
        Some("0px")
    );
    assert_eq!(
        viewport.attribute("fallback-nearest-scrollport"),
        Some("none")
    );
    for attribute in [
        "containing-width",
        "containing-height",
        "fallback-icb-width",
        "fallback-icb-height",
    ] {
        assert!(
            viewport.attribute(attribute).is_some(),
            "missing {attribute}"
        );
    }
    let input = test
        .children()
        .find(|node| node.has_tag_name("input"))
        .expect("input");
    let root = input
        .children()
        .find(roxmltree::Node::is_element)
        .expect("root input");
    assert_eq!(root.attribute("position"), Some("absolute"));
    Ok(())
}

fn qualify_tampering(
    root: &Path,
    corpus: &BrowserCorpus,
    source: &RelativePath,
    output: &RelativePath,
) -> Result<()> {
    for path in [source, output] {
        let path = root.join(path.as_str());
        let original = fs::read(&path)?;
        let mut modified = original.clone();
        modified.push(b'\n');
        fs::write(&path, modified)?;
        let result = browser::check_corpus(corpus, &LayoutAdapter);
        fs::write(&path, original)?;
        assert_invalid_inventory(result);
    }
    let undeclared = root.join("scripts/gentest/undeclared.js");
    fs::write(&undeclared, b"// undeclared input\n")?;
    let result = browser::check_corpus(corpus, &LayoutAdapter);
    fs::remove_file(undeclared)?;
    assert_invalid_inventory(result);
    Ok(())
}

fn assert_invalid_inventory(
    result: std::result::Result<browser::BrowserGenerationReport, GenerationError<AdapterError>>,
) {
    match result {
        Err(GenerationError::Generator(error)) => assert_eq!(
            error.kind(),
            GeneratorErrorKind::InvalidInventory,
            "{error}"
        ),
        other => panic!("expected typed input/output inventory refusal, received {other:?}"),
    }
}
