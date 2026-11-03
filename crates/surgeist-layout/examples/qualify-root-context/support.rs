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
        "html/grid/grid_fit_content_points_argument.html",
        include_bytes!(
            "../../tests/layout/browser_parity/html/grid/grid_fit_content_points_argument.html"
        ),
    ),
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
    name: String,
    source: RelativePath,
    source_sha256: Sha256Digest,
    expected: BrowserReportSummary,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    variant: String,
    output: RelativePath,
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
    assert_eq!(declaration.schema_version, 1);
    assert_eq!(
        declaration.expected,
        BrowserReportSummary::new(4, 0, 0, 0, 0)?
    );
    assert_eq!(declaration.cases.len(), fixture_cases().len());
    for (case, (variant, _)) in declaration.cases.iter().zip(fixture_cases()) {
        assert_eq!(case.variant, variant);
    }
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
    let mut expected_inputs = BTreeMap::new();
    for (name, bytes) in INPUTS
        .iter()
        .copied()
        .chain(std::iter::once(("corpus.toml", DECLARATION)))
    {
        let path = RelativePath::new(name)?;
        let destination = root.join(path.as_str());
        fs::create_dir_all(destination.parent().expect("input parent"))?;
        fs::write(destination, bytes)?;
        expected_inputs.insert(path, Sha256Digest::from_bytes(bytes));
    }
    assert_eq!(
        expected_inputs.get(&declaration.source),
        Some(&declaration.source_sha256)
    );
    let cases = declaration
        .cases
        .iter()
        .map(|case| CaseSpec::new(case.id.clone(), case.variant.clone(), case.output.clone()))
        .collect::<surgeist_generator::Result<Vec<_>>>()?;
    let fixture = FixtureSpec::new(
        declaration.name,
        declaration.source.clone(),
        cases,
        FixtureStatus::Active,
    )?;
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
        vec![fixture],
        INPUTS
            .iter()
            .filter(|(name, _)| *name != declaration.source.as_str())
            .map(|(name, _)| RelativePath::new(name))
            .collect::<surgeist_generator::Result<Vec<_>>>()?,
        Vec::new(),
    )?
    .with_expected_inputs(expected_inputs)?
    .with_exact_input_roots(vec![
        RelativePath::new("scripts/gentest")?,
        RelativePath::new("fonts")?,
    ])?
    .with_expected_counts(declaration.expected);
    let request = GenerationRequest::new(corpus.clone(), browser_path, None, None)?;
    let report = browser::generate(request, QualificationAdapter)?;
    assert_eq!(*report.summary(), declaration.expected);
    assert!(report.filter().is_none());
    assert_eq!(report.generated().len(), declaration.cases.len());
    assert_eq!(
        fs::read(root.join("xml/generation-reports/all.json"))?,
        report.to_bytes()?
    );
    for artifact in report.generated() {
        let case = declaration
            .cases
            .iter()
            .find(|case| case.id == artifact.name())
            .expect("declared generated case");
        assert_eq!(artifact.source(), &declaration.source);
        assert_eq!(artifact.output(), &case.output);
        assert_eq!(artifact.variant(), case.variant);
        let bytes = fs::read(root.join(artifact.output().as_str()))?;
        assert_eq!(Sha256Digest::from_bytes(&bytes), *artifact.output_sha256());
        qualify_xml(std::str::from_utf8(&bytes)?, &case.id)?;
    }
    assert_eq!(
        browser::check_corpus(&corpus, &LayoutAdapter)?.to_bytes()?,
        report.to_bytes()?
    );
    qualify_tampering(
        root,
        &corpus,
        &declaration.source,
        &declaration.cases[0].output,
    )?;
    assert_eq!(
        browser::check_corpus(&corpus, &LayoutAdapter)?.to_bytes()?,
        report.to_bytes()?
    );
    eprintln!(
        "qualify-root-context: four generated variants, complete provenance/XML and three tamper refusals qualified"
    );
    Ok(())
}

fn qualify_xml(xml: &str, case_id: &str) -> Result<()> {
    let document = roxmltree::Document::parse(xml)?;
    let test = document.root_element();
    assert!(test.has_tag_name("test"));
    assert_eq!(test.attribute("name"), Some(case_id));
    assert_eq!(test.attribute("measurement-version"), Some("3"));
    assert_eq!(test.attribute("observation-frame"), Some("root-border"));
    let viewport = test
        .children()
        .find(|node| node.has_tag_name("viewport"))
        .expect("viewport");
    assert_eq!(
        viewport.attribute("root-context"),
        Some("positioned-auto-insets")
    );
    assert_eq!(viewport.attribute("containing-direction"), Some("ltr"));
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
    assert!(
        test.children()
            .any(|node| node.has_tag_name("expectations"))
    );
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
