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
    source_kind: SourceKind,
    expected: ExpectedDisposition,
    #[serde(default)]
    historical_sha256: BTreeMap<String, Sha256Digest>,
    relation: Option<PairedObservation>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SourceKind {
    Original,
    Authored,
}
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum ExpectedDisposition {
    Generated { metadata: ExpectedMetadata },
    Unsupported { reason: String },
}
#[derive(Clone, Deserialize)]
#[serde(tag = "context", rename_all = "kebab-case", deny_unknown_fields)]
enum ExpectedMetadata {
    AbsoluteLocal {
        hypothetical_display: String,
        containing_direction: String,
        hypothetical_direction: String,
        root_direction: String,
        root_writing_mode: String,
        inline_offset: f64,
        root_size: Option<ExpectedRootSize>,
    },
    BlockFlow {
        parent_direction: String,
        root_direction: String,
        root_writing_mode: String,
        root_position: String,
    },
}
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum ExpectedRootSize {
    Fixed { width: f64, height: f64 },
    FlexPair { item_width: f64, item_height: f64 },
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PairedObservation {
    peer: String,
    world_phase_change: bool,
}
struct ExpectedCase {
    source: RelativePath,
    case: CaseSpec,
    historical: Option<RelativePath>,
    expected: ExpectedDisposition,
    relation: Option<PairedObservation>,
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
        let mut generated = 0usize;
        let mut unsupported = 0usize;
        for record in &self.records {
            let count = match &record.expected {
                ExpectedDisposition::Generated { .. } => &mut generated,
                ExpectedDisposition::Unsupported { .. } => &mut unsupported,
            };
            *count = count
                .checked_add(fixture_cases().len())
                .ok_or("qualification case count overflow")?;
        }
        Ok(BrowserReportSummary::new(generated, unsupported, 0, 0, 0)?)
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

pub(super) fn run(fixture: Option<&str>) -> Result<()> {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut declaration: Declaration =
        surgeist_generator::parse_manifest(std::str::from_utf8(DECLARATION)?, "corpus.toml")?;
    assert_eq!(declaration.schema_version, 3);
    if let Some(source) = fixture {
        declaration
            .records
            .retain(|record| record.source.as_str() == source);
        if declaration.records.len() != 1 {
            return Err("focused qualification requires one declared source".into());
        }
        if declaration.records[0].relation.is_some() {
            return Err("paired observations require the complete inventory".into());
        }
        eprintln!(
            "qualify-root-context: focused declared source {source}; not a full inventory qualification"
        );
    }
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
    let result = qualify(
        &invocation.root,
        owner,
        settings,
        browser_path,
        declaration,
        fixture.is_some(),
    );
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
    focused: bool,
) -> Result<()> {
    let expected = declaration.expected()?;
    let inventory = prepare_inventory(root, owner, &declaration.records)?;
    let mut exact_roots = vec![
        RelativePath::new("scripts/gentest")?,
        RelativePath::new("fonts")?,
    ];
    if inventory.cases.iter().any(|case| case.historical.is_some()) {
        exact_roots.push(RelativePath::new("historical")?);
    }
    let corpus = BrowserCorpus::new(
        BrowserLocation::new(owner, root)?,
        RelativePath::new("corpus.toml")?,
        if focused {
            "surgeist-layout-root-context-focused".into()
        } else {
            "surgeist-layout-root-context-qualification".into()
        },
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
    .with_exact_input_roots(exact_roots)?
    .with_expected_counts(expected);
    let request = GenerationRequest::new(corpus.clone(), browser_path, None, None)?;
    let report = browser::generate(request, QualificationAdapter)?;
    assert_eq!(*report.summary(), expected);
    assert!(report.filter().is_none());
    assert_eq!(
        report.generated().len(),
        inventory
            .cases
            .iter()
            .filter(|case| matches!(&case.expected, ExpectedDisposition::Generated { .. }))
            .count()
    );
    qualify_dispositions(&report.to_bytes()?, &inventory.cases)?;
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
        if let Some(historical) = &case.historical {
            let historical = fs::read_to_string(root.join(historical.as_str()))?;
            qualify_historical_geometry(std::str::from_utf8(&bytes)?, &historical, case.case.id())?;
        }
    }
    eprintln!(
        "qualify-root-context: all {} source-bound historical expectation trees preserved",
        inventory
            .cases
            .iter()
            .filter(|case| case.historical.is_some())
            .count()
    );
    for artifact in report.generated() {
        let case = inventory
            .cases
            .iter()
            .find(|case| case.case.id() == artifact.name())
            .expect("declared generated case");
        let xml = fs::read_to_string(root.join(artifact.output().as_str()))?;
        qualify_xml(&xml, case)?;
        let golden = crate::parity_support::Golden::parse(&xml)?;
        crate::parity_support::assert_surgeist_matches(&golden)?;
    }
    qualify_pairs(root, &inventory.cases)?;
    let tamper_count = if let Some(first) = inventory
        .cases
        .iter()
        .find(|case| matches!(case.expected, ExpectedDisposition::Generated { .. }))
    {
        qualify_tampering(root, &corpus, &first.source, first.case.output())?;
        3
    } else {
        0
    };
    assert_eq!(
        browser::check_corpus(&corpus, &LayoutAdapter)?.to_bytes()?,
        report.to_bytes()?
    );
    eprintln!(
        "qualify-root-context: {} Generated and {} Unsupported V4 cases, declared pairs, historical geometry and {tamper_count} filesystem tamper refusals qualified",
        report.summary().generated(),
        report.summary().unsupported()
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
    let authored = owner.join("examples/qualify-root-context");
    for record in records {
        let source_owner = match record.source_kind {
            SourceKind::Original => &original,
            SourceKind::Authored => &authored,
        };
        let source = fs::read(source_owner.join(record.source.as_str()))?;
        assert_eq!(
            Sha256Digest::from_bytes(&source),
            record.source_sha256,
            "declared source changed: {}",
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
            .ok_or("qualification source must be an html/*.html path")?;
        let stem = family.rsplit('/').next().ok_or("missing source stem")?;
        match record.source_kind {
            SourceKind::Original => {
                assert!(matches!(
                    &record.expected,
                    ExpectedDisposition::Generated { .. }
                ));
                assert_eq!(record.historical_sha256.len(), fixture_cases().len());
            }
            SourceKind::Authored => assert!(record.historical_sha256.is_empty()),
        }
        if record.relation.is_some() {
            assert!(matches!(
                &record.expected,
                ExpectedDisposition::Generated { .. }
            ));
        }
        let mut cases = Vec::new();
        for (variant, _) in fixture_cases() {
            let output = RelativePath::new(format!("xml/{family}__{variant}.xml"))?;
            let historical = if matches!(record.source_kind, SourceKind::Original) {
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
                Some(historical)
            } else {
                None
            };
            let case = CaseSpec::new(format!("{stem}__{variant}"), variant.into(), output)?;
            inventory.cases.push(ExpectedCase {
                source: record.source.clone(),
                case: case.clone(),
                historical,
                expected: record.expected.clone(),
                relation: record.relation.clone(),
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
    assert_eq!(test.attribute("measurement-version"), Some("4"));
    let viewport = test
        .children()
        .find(|node| node.has_tag_name("viewport"))
        .ok_or("missing viewport")?;
    let ExpectedDisposition::Generated { metadata } = &case.expected else {
        return Err("Unsupported case unexpectedly emitted XML".into());
    };
    let (root_direction, root_writing_mode, root_position) = match metadata {
        ExpectedMetadata::AbsoluteLocal {
            hypothetical_display,
            containing_direction,
            hypothetical_direction,
            root_direction,
            root_writing_mode,
            inline_offset,
            root_size,
        } => {
            assert_eq!(
                test.attribute("observation-frame"),
                Some("root-border-local")
            );
            assert_eq!(
                viewport.attribute("root-context"),
                Some("absolute-root-local")
            );
            assert_eq!(
                viewport.attribute("containing-direction"),
                Some(containing_direction.as_str())
            );
            assert_eq!(
                viewport.attribute("hypothetical-direction"),
                Some(hypothetical_direction.as_str())
            );
            assert_eq!(
                viewport.attribute("hypothetical-display"),
                Some(hypothetical_display.as_str())
            );
            assert_eq!(
                viewport.attribute("hypothetical-anchor"),
                Some("inline-start")
            );
            assert_eq!(
                px_attribute(viewport, "hypothetical-inline-offset")?,
                *inline_offset
            );
            for (containing, fallback) in [
                ("containing-width", "fallback-icb-width"),
                ("containing-height", "fallback-icb-height"),
            ] {
                assert_eq!(
                    px_attribute(viewport, containing)?,
                    px_attribute(viewport, fallback)?
                );
            }
            for field in ["parent-inline-extent", "parent-direction"] {
                assert_eq!(
                    viewport.attribute(field),
                    None,
                    "absolute context has no BlockFlow field"
                );
            }
            qualify_root_size(
                test,
                viewport,
                root_size.as_ref(),
                *inline_offset,
                case.case.id(),
            )?;
            (
                root_direction.as_str(),
                root_writing_mode.as_str(),
                "absolute",
            )
        }
        ExpectedMetadata::BlockFlow {
            parent_direction,
            root_direction,
            root_writing_mode,
            root_position,
        } => {
            // The zero-edge Htb Auto body owns only its actual inline extent.
            // No used/root height is promoted into a body percentage basis or
            // a hypothetical static-position rectangle. The reader independently
            // validates the actual Native body-origin placement.
            assert_eq!(test.attribute("observation-frame"), Some("root-border"));
            assert_eq!(viewport.attribute("root-context"), Some("block-flow-auto"));
            assert_eq!(
                viewport.attribute("parent-direction"),
                Some(parent_direction.as_str())
            );
            assert_eq!(
                px_attribute(viewport, "parent-inline-extent")?,
                px_attribute(viewport, "fallback-icb-width")?
            );
            for field in [
                "containing-width",
                "containing-height",
                "containing-direction",
                "hypothetical-direction",
                "hypothetical-display",
                "hypothetical-anchor",
                "hypothetical-inline-offset",
                "parent-block-extent",
            ] {
                assert_eq!(
                    viewport.attribute(field),
                    None,
                    "Auto body has no definite block/hypothetical authority"
                );
            }
            (
                root_direction.as_str(),
                root_writing_mode.as_str(),
                root_position.as_str(),
            )
        }
    };
    assert_eq!(
        viewport.attribute("fallback-nearest-scrollport"),
        Some("none")
    );
    for field in [
        "fallback-containing-width-limit",
        "fallback-containing-height-limit",
    ] {
        assert_eq!(viewport.attribute(field), Some("unbounded"));
    }
    for field in ["fallback-icb-width", "fallback-icb-height"] {
        assert!(px_attribute(viewport, field)? >= 0.0);
    }
    let input = test
        .children()
        .find(|node| node.has_tag_name("input"))
        .ok_or("missing root input")?;
    let root = input
        .children()
        .find(roxmltree::Node::is_element)
        .ok_or("missing root node")?;
    // Relative and HorizontalTb are the protocol's explicit elision defaults.
    assert_eq!(
        root.attribute("position").unwrap_or("relative"),
        root_position
    );
    assert_eq!(
        root.attribute("writing-mode").unwrap_or("horizontal-tb"),
        root_writing_mode
    );
    let root_direction = if root_direction == "variant" {
        if case.case.variant().ends_with("_rtl") {
            "rtl"
        } else {
            "ltr"
        }
    } else {
        root_direction
    };
    assert_eq!(root.attribute("direction"), Some(root_direction));
    Ok(())
}

fn qualify_dispositions(bytes: &[u8], cases: &[ExpectedCase]) -> Result<()> {
    let report: serde_json::Value = serde_json::from_slice(bytes)?;
    let unsupported = report
        .get("unsupported")
        .and_then(serde_json::Value::as_array)
        .ok_or("browser report omitted Unsupported dispositions")?;
    for case in cases {
        if let ExpectedDisposition::Unsupported { reason } = &case.expected {
            let actual = unsupported
                .iter()
                .find(|entry| {
                    entry.get("name").and_then(serde_json::Value::as_str) == Some(case.case.id())
                })
                .ok_or_else(|| {
                    format!("{}: missing declared Unsupported refusal", case.case.id())
                })?;
            assert_eq!(
                actual.get("source").and_then(serde_json::Value::as_str),
                Some(case.source.as_str())
            );
            assert_eq!(
                actual.get("variant").and_then(serde_json::Value::as_str),
                Some(case.case.variant())
            );
            assert_eq!(
                actual.get("reason").and_then(serde_json::Value::as_str),
                Some(reason.as_str()),
                "{}: semantic/provenance refusal",
                case.case.id()
            );
        }
    }
    Ok(())
}
fn number_attribute(node: roxmltree::Node<'_, '_>, name: &str) -> Result<f64> {
    let value: f64 = node
        .attribute(name)
        .ok_or_else(|| format!("missing {name}"))?
        .parse()?;
    if !value.is_finite() {
        return Err(format!("nonfinite {name}").into());
    }
    Ok(value)
}
fn px_attribute(node: roxmltree::Node<'_, '_>, name: &str) -> Result<f64> {
    let raw = node
        .attribute(name)
        .and_then(|value| value.strip_suffix("px"))
        .ok_or_else(|| format!("missing pixel {name}"))?;
    let value: f64 = raw.parse()?;
    if !value.is_finite() {
        return Err(format!("nonfinite {name}").into());
    }
    Ok(value)
}
fn qualify_root_size(
    test: roxmltree::Node<'_, '_>,
    viewport: roxmltree::Node<'_, '_>,
    root_size: Option<&ExpectedRootSize>,
    inline_offset: f64,
    case_id: &str,
) -> Result<()> {
    let Some(expected) = root_size else {
        return Ok(());
    };
    assert_eq!(test.attribute("use-rounding"), Some("true"));
    let (width, height) = match expected {
        ExpectedRootSize::Fixed { width, height } => (*width, *height),
        ExpectedRootSize::FlexPair {
            item_width,
            item_height,
        } => {
            // Two genuine unshrinkable native children: MC=itemWidth,
            // MAX=2*itemWidth. The source-authenticated CB and indent are
            // operands; the eventual observed root rectangle is not.
            let available = (px_attribute(viewport, "containing-width")? - inline_offset).max(0.0);
            let width = available.max(*item_width).min(2.0 * item_width);
            (
                width,
                if width < 2.0 * item_width {
                    2.0 * item_height
                } else {
                    *item_height
                },
            )
        }
    };
    let expectations = test
        .children()
        .find(|node| node.has_tag_name("expectations"))
        .ok_or("missing expectations")?;
    let root = expectations
        .children()
        .find(roxmltree::Node::is_element)
        .ok_or("missing root observation")?;
    assert_eq!(number_attribute(root, "x")?, 0.0, "{case_id}: local root x");
    assert_eq!(number_attribute(root, "y")?, 0.0, "{case_id}: local root y");
    assert_eq!(
        number_attribute(root, "width")?,
        (width + 0.5).floor(),
        "{case_id}: authored/shrink width"
    );
    assert_eq!(
        number_attribute(root, "height")?,
        (height + 0.5).floor(),
        "{case_id}: actual native wrap height"
    );
    Ok(())
}
fn observation_section<'doc, 'input>(
    document: &'doc roxmltree::Document<'input>,
    name: &str,
) -> Result<roxmltree::Node<'doc, 'input>> {
    document
        .root_element()
        .children()
        .find(|node| node.has_tag_name(name))
        .ok_or_else(|| format!("missing {name}").into())
}
fn world_observations<'doc, 'input>(
    document: &'doc roxmltree::Document<'input>,
) -> Result<BTreeMap<String, roxmltree::Node<'doc, 'input>>> {
    let mut rows = BTreeMap::new();
    for node in observation_section(document, "diagnostics")?
        .children()
        .filter(roxmltree::Node::is_element)
    {
        assert!(node.has_tag_name("world-rounded-layout"));
        let path = node
            .attribute("node-path")
            .ok_or("world node path")?
            .to_owned();
        assert!(rows.insert(path, node).is_none(), "duplicate world path");
    }
    Ok(rows)
}
fn qualify_pairs(root: &Path, cases: &[ExpectedCase]) -> Result<()> {
    for case in cases {
        let Some(relation) = &case.relation else {
            continue;
        };
        let peer_name = format!("{}__{}", relation.peer, case.case.variant());
        let peer = cases
            .iter()
            .find(|other| other.case.id() == peer_name)
            .ok_or("undeclared paired source/variant")?;
        assert!(matches!(
            &peer.expected,
            ExpectedDisposition::Generated { .. }
        ));
        let left_xml = fs::read_to_string(root.join(case.case.output().as_str()))?;
        let right_xml = fs::read_to_string(root.join(peer.case.output().as_str()))?;
        let left = roxmltree::Document::parse(&left_xml)?;
        let right = roxmltree::Document::parse(&right_xml)?;
        // Every generated tree has already passed the maintained strict Native
        // comparison. Pair equality includes the complete rounded local tree,
        // not just root dimensions or a subset of scroll observations.
        compare_observation_tree(
            observation_section(&left, "expectations")?,
            observation_section(&right, "expectations")?,
            &format!("{}~{}", case.case.id(), peer.case.id()),
        );
        if relation.world_phase_change {
            let left_world = world_observations(&left)?;
            let right_world = world_observations(&right)?;
            assert_eq!(
                left_world.keys().collect::<Vec<_>>(),
                right_world.keys().collect::<Vec<_>>()
            );
            // Both actual root and child have authored10.5px height at the same
            // world phase. A half-pixel translation changes their world-corner
            // rounded heights together while their local tree stays identical.
            for path in ["0", "0.0"] {
                let l = *left_world.get(path).ok_or("missing shared-phase box")?;
                let r = *right_world
                    .get(path)
                    .ok_or("missing paired shared-phase box")?;
                assert_ne!(
                    number_attribute(l, "height")?,
                    number_attribute(r, "height")?,
                    "{path}: missing real world phase change"
                );
            }
            assert_eq!(
                number_attribute(left_world["0"], "height")?,
                number_attribute(left_world["0.0"], "height")?
            );
            assert_eq!(
                number_attribute(right_world["0"], "height")?,
                number_attribute(right_world["0.0"], "height")?
            );
        }
    }
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
