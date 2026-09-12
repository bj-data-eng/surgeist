use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::digest::sha256_hex;

#[path = "csstree_adapters.rs"]
mod adapters;

use adapters::{
    Adapter, EntryPoint, Extractor as RegistryExtractor, OptionsProfile, PropertyOrDescriptor,
    REGISTRY, RegistryEntry, UnsupportedPolicy as RegistryUnsupportedPolicy,
};
use surgeist_css::{
    CssErrorCode, CssImportance, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssPropertyNameRef, CssRecoveryAction, CssRecoveryDiagnostic, parse_declaration,
    parse_font_face_descriptor_value, parse_media_query, parse_media_query_list,
    parse_property_value_text, parse_rule, parse_selector, parse_selector_list, parse_sheet,
    parse_style_attribute, parse_style_block,
};
use surgeist_css::{validate_sheet, validate_style_attribute};

const CORPUS_ROOT: &str = "tests/corpus/csstree";
const EXPECTED_CLASSES_PATH: &str = "tests/csstree/expected-classes.json";
const ORACLE_PATH: &str = "tests/csstree/oracle.json";
const REPORT_PATH: &str = "expectations/generation-reports/all.json";
const SOURCE_SIDECAR_PATH: &str = "source/.surgeist-source.json";
const EXPECTED_MANIFEST_DIGEST: &str =
    "dc61d7c2cd387418e1a9b403e2aa71266abfdf291f95c7598688d48452cec96e";
const EXPECTED_SOURCE_REPOSITORY: &str = "https://github.com/csstree/csstree.git";
const EXPECTED_SOURCE_REVISION: &str = "88e3d965c0b1628642a30a841745b410d6835052";
const EXPECTED_SOURCE_TREE: &str = "bfadc7a7a8d93dce59a27fa7df3bb0f6f6a623d8";
const EXPECTED_IMPORT_PROVENANCE: &str =
    "f4f2957f38bf42d052a2593b94f853b6f17255eadd2ac26928c8bebe6bb3423c";
const EXPECTED_GENERATOR: &str = "surgeist-css-generate";
const EXPECTED_ARTIFACTS: usize = 74;
const EXPECTED_CASES: usize = 935;
const EXPECTED_PARSED: usize = 721;
const EXPECTED_REJECTED: usize = 214;
const EXPECTED_CONTEXT_COUNTS: [(Context, usize); 11] = [
    (Context::Atrule, 130),
    (Context::AtrulePrelude, 2),
    (Context::Block, 29),
    (Context::Declaration, 77),
    (Context::DeclarationList, 19),
    (Context::MediaQuery, 49),
    (Context::Rule, 33),
    (Context::Selector, 317),
    (Context::SelectorList, 10),
    (Context::Stylesheet, 76),
    (Context::Value, 193),
];

static NEUTRAL_INVENTORY: OnceLock<Result<NeutralInventory, String>> = OnceLock::new();
static CORPUS: OnceLock<Result<ValidatedCorpus, String>> = OnceLock::new();

pub(crate) fn validate_shared_support_surface() -> bool {
    let _ = load_csstree_corpus as fn() -> Result<&'static ValidatedCorpus, &'static str>;
    let _ = load_csstree_oracle as fn() -> Result<&'static Oracle, &'static str>;
    let _ = validate_csstree_oracle_contract as fn(&[u8]) -> Result<(), Vec<OracleContractFailure>>;
    let _ = regenerate_csstree_oracle as fn() -> Result<Vec<u8>, Vec<BaselineFailure>>;
    let _ = capture_csstree_oracle_from_public_parser as fn() -> Result<(), String>;
    let _ = validate_csstree_expected_classes_contract
        as fn(&[u8]) -> Result<ExpectedClassSummary, String>;
    // Each integration-test binary compiles this shared module independently.
    let _ = ExpectedClassSummary::digest as fn(&ExpectedClassSummary) -> &str;
    let _ = ExpectedClassSummary::class_counts
        as fn(&ExpectedClassSummary) -> [(&'static str, usize); 4];
    let _ = ExpectedClassSummary::policy_counts
        as fn(&ExpectedClassSummary) -> [(&'static str, usize); 2];
    let _ = expected_payload_relation_holds
        as fn(&str, std::ops::Range<usize>, usize, usize, usize) -> bool;
    let _ = ValidatedCorpus::artifact_count as fn(&ValidatedCorpus) -> usize;
    let _ = ValidatedCorpus::case_count as fn(&ValidatedCorpus) -> usize;
    let _ = ValidatedCorpus::parsed_count as fn(&ValidatedCorpus) -> usize;
    let _ = ValidatedCorpus::rejected_count as fn(&ValidatedCorpus) -> usize;
    let _ = ValidatedCorpus::context_counts as fn(&ValidatedCorpus) -> [(&'static str, usize); 11];
    true
}

const _: fn() -> bool = validate_shared_support_surface;

pub(crate) fn load_csstree_corpus() -> Result<&'static ValidatedCorpus, &'static str> {
    let corpus = load_validated_csstree()?;
    let oracle = load_csstree_oracle()?;
    debug_assert!(std::ptr::eq(&corpus.oracle, oracle));
    debug_assert_eq!(corpus.inventory.cases.len(), oracle.id_count());
    debug_assert_eq!(oracle.generation_report_sha256().len(), 64);
    debug_assert_eq!(
        corpus.inventory.cases.len(),
        oracle
            .outcome_counts()
            .iter()
            .map(|(_, count)| count)
            .sum::<usize>()
    );
    Ok(corpus)
}

fn load_validated_csstree() -> Result<&'static ValidatedCorpus, &'static str> {
    match CORPUS.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
        read_artifact_set(&root).and_then(validate_artifact_set)
    }) {
        Ok(corpus) => Ok(corpus),
        Err(error) => Err(error.as_str()),
    }
}

fn load_neutral_csstree_inventory() -> Result<&'static NeutralInventory, &'static str> {
    match NEUTRAL_INVENTORY.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
        read_neutral_artifact_set(&root).and_then(validate_neutral_artifact_set)
    }) {
        Ok(inventory) => Ok(inventory),
        Err(error) => Err(error.as_str()),
    }
}

pub(crate) fn load_csstree_oracle() -> Result<&'static Oracle, &'static str> {
    load_validated_csstree().map(|corpus| &corpus.oracle)
}

#[derive(Debug)]
pub(crate) struct ValidatedCorpus {
    inventory: NeutralInventory,
    oracle: Oracle,
}

#[derive(Debug)]
struct NeutralInventory {
    artifact_count: usize,
    cases: Vec<ValidatedCase>,
    parsed_count: usize,
    rejected_count: usize,
    context_counts: [usize; 11],
    report_digest: String,
    expected_classes_digest: String,
}

#[derive(Debug)]
pub(crate) struct Oracle {
    generation_report_sha256: String,
    id_count: usize,
    outcome_counts: [usize; 4],
}

impl Oracle {
    pub(crate) fn generation_report_sha256(&self) -> &str {
        &self.generation_report_sha256
    }

    pub(crate) fn id_count(&self) -> usize {
        self.id_count
    }

    pub(crate) fn outcome_counts(&self) -> [(&'static str, usize); 4] {
        let names = ["clean", "recovered", "strict_rejected", "unsupported"];
        std::array::from_fn(|index| (names[index], self.outcome_counts[index]))
    }
}

impl ValidatedCorpus {
    pub(crate) fn artifact_count(&self) -> usize {
        self.inventory.artifact_count
    }

    pub(crate) fn case_count(&self) -> usize {
        debug_assert_eq!(
            self.inventory.cases.len(),
            self.oracle.outcome_counts.iter().sum::<usize>()
        );
        self.inventory.cases.len()
    }

    pub(crate) fn parsed_count(&self) -> usize {
        self.inventory.parsed_count
    }

    pub(crate) fn rejected_count(&self) -> usize {
        self.inventory.rejected_count
    }

    pub(crate) fn context_counts(&self) -> [(&'static str, usize); 11] {
        std::array::from_fn(|index| {
            (
                EXPECTED_CONTEXT_COUNTS[index].0.as_str(),
                self.inventory.context_counts[index],
            )
        })
    }
}

#[derive(Debug)]
struct ValidatedCase {
    id: String,
    expectation_path: String,
    expectation_sha256: String,
    source: String,
    context: Context,
    _label: Option<String>,
    input: String,
    _upstream_outcome: UpstreamOutcome,
    _canonical_css: Option<String>,
    options: Options,
    expected_class: Option<ExpectedClass>,
}

#[derive(Debug)]
struct ArtifactSet {
    report: Vec<u8>,
    expected_classes: Vec<u8>,
    oracle: Vec<u8>,
    expectations: BTreeMap<String, Vec<u8>>,
    sources: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug)]
struct NeutralArtifactSet {
    report: Vec<u8>,
    expected_classes: Vec<u8>,
    expectations: BTreeMap<String, Vec<u8>>,
    sources: BTreeMap<String, Vec<u8>>,
}

fn read_artifact_set(root: &Path) -> Result<ArtifactSet, String> {
    let report_path = root.join(REPORT_PATH);

    Ok(ArtifactSet {
        report: read_file(&report_path)?,
        expected_classes: read_file(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED_CLASSES_PATH),
        )?,
        oracle: read_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join(ORACLE_PATH))?,
        expectations: read_json_artifacts(root, "expectations", Some(REPORT_PATH))?,
        sources: read_json_artifacts(root, "source", Some(SOURCE_SIDECAR_PATH))?,
    })
}

fn read_neutral_artifact_set(root: &Path) -> Result<NeutralArtifactSet, String> {
    let report_path = root.join(REPORT_PATH);

    Ok(NeutralArtifactSet {
        report: read_file(&report_path)?,
        expected_classes: read_file(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED_CLASSES_PATH),
        )?,
        expectations: read_json_artifacts(root, "expectations", Some(REPORT_PATH))?,
        sources: read_json_artifacts(root, "source", Some(SOURCE_SIDECAR_PATH))?,
    })
}

fn read_json_artifacts(
    root: &Path,
    relative_root: &str,
    excluded_path: Option<&str>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut pending = vec![root.join(relative_root)];
    let mut artifacts = BTreeMap::new();

    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!("failed to read entry in {}: {error}", directory.display())
            })?;
            let file_type = entry.file_type().map_err(|error| {
                format!("failed to inspect {}: {error}", entry.path().display())
            })?;
            let path = entry.path();
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("json")
            {
                continue;
            }

            let relative = relative_path(root, &path)?;
            if excluded_path == Some(relative.as_str()) {
                continue;
            }
            let bytes = read_file(&path)?;
            if artifacts.insert(relative.clone(), bytes).is_some() {
                return Err(format!("duplicate artifact path {relative}"));
            }
        }
    }

    Ok(artifacts)
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        format!(
            "{} is not beneath corpus root {}: {error}",
            path.display(),
            root.display()
        )
    })?;
    let relative = relative
        .to_str()
        .ok_or_else(|| format!("artifact path is not UTF-8: {}", path.display()))?
        .replace(std::path::MAIN_SEPARATOR, "/");
    if !is_canonical_relative_path(&relative) {
        return Err(format!("artifact path is not canonical: {relative}"));
    }
    Ok(relative)
}

fn validate_artifact_set(artifacts: ArtifactSet) -> Result<ValidatedCorpus, String> {
    let ArtifactSet {
        report,
        expected_classes,
        oracle,
        expectations,
        sources,
    } = artifacts;
    let inventory = validate_neutral_artifact_set(NeutralArtifactSet {
        report,
        expected_classes,
        expectations,
        sources,
    })?;
    let validated_oracle = validate_oracle(
        &oracle,
        &inventory.report_digest,
        &inventory.expected_classes_digest,
        &inventory.cases,
    )?;
    Ok(ValidatedCorpus {
        inventory,
        oracle: validated_oracle,
    })
}

fn validate_neutral_artifact_set(
    artifacts: NeutralArtifactSet,
) -> Result<NeutralInventory, String> {
    let expected_classes_digest = sha256_hex(&artifacts.expected_classes);
    let report_digest = sha256_hex(&artifacts.report);
    let report: RawReport = serde_json::from_slice(&artifacts.report)
        .map_err(|error| format!("failed to deserialize corpus report: {error}"))?;
    validate_report_header(&report)?;

    if report.artifacts.len() != EXPECTED_ARTIFACTS {
        return Err(format!(
            "report artifact census mismatch: expected {EXPECTED_ARTIFACTS}, found {}",
            report.artifacts.len()
        ));
    }

    let report_expectations = report
        .artifacts
        .iter()
        .map(|artifact| artifact.output_path.clone())
        .collect::<BTreeSet<_>>();
    let report_sources = report
        .artifacts
        .iter()
        .map(|artifact| artifact.provenance.source_path.clone())
        .collect::<BTreeSet<_>>();
    let disk_expectations = artifacts.expectations.keys().cloned().collect();
    let disk_sources = artifacts.sources.keys().cloned().collect();
    if report_expectations != disk_expectations {
        return Err(set_mismatch(
            "expectation artifact inventory",
            &report_expectations,
            &disk_expectations,
        ));
    }
    if report_sources != disk_sources {
        return Err(set_mismatch(
            "source artifact inventory",
            &report_sources,
            &disk_sources,
        ));
    }

    let mut cases = Vec::with_capacity(EXPECTED_CASES);
    let mut seen_ids = BTreeSet::new();
    let mut previous_id: Option<String> = None;
    let mut previous_output_path: Option<&str> = None;
    let mut parsed_count = 0;
    let mut rejected_count = 0;
    let mut context_counts = [0; 11];

    for artifact in &report.artifacts {
        if previous_output_path.is_some_and(|previous| previous >= artifact.output_path.as_str()) {
            return Err(format!(
                "report artifacts are not in canonical order at {}",
                artifact.output_path
            ));
        }
        previous_output_path = Some(&artifact.output_path);

        validate_report_artifact(artifact)?;
        let expectation_bytes = artifacts
            .expectations
            .get(&artifact.output_path)
            .ok_or_else(|| format!("missing expectation artifact {}", artifact.output_path))?;
        let source_bytes = artifacts
            .sources
            .get(&artifact.provenance.source_path)
            .ok_or_else(|| {
                format!(
                    "missing source artifact {}",
                    artifact.provenance.source_path
                )
            })?;
        let expectation: RawExpectation =
            serde_json::from_slice(expectation_bytes).map_err(|error| {
                format!(
                    "failed to deserialize expectation {}: {error}",
                    artifact.output_path
                )
            })?;
        validate_expectation_header(&expectation, artifact)?;

        if expectation.cases.len() != artifact.case_count {
            return Err(format!(
                "case census mismatch for {}: report {}, expectation {}",
                artifact.output_path,
                artifact.case_count,
                expectation.cases.len()
            ));
        }

        let expected_output_digest = sha256_hex(expectation_bytes);
        if artifact.output_digest != expected_output_digest {
            return Err(format!(
                "output digest mismatch for {}: report {}, actual {expected_output_digest}",
                artifact.output_path, artifact.output_digest
            ));
        }
        let expected_source_digest = sha256_hex(source_bytes);
        if artifact.provenance.source_digest != expected_source_digest {
            return Err(format!(
                "source digest mismatch for {}: report {}, actual {expected_source_digest}",
                artifact.provenance.source_path, artifact.provenance.source_digest
            ));
        }
        if expectation.source_sha256 != expected_source_digest {
            return Err(format!(
                "expectation source digest mismatch for {}: expectation {}, actual {expected_source_digest}",
                artifact.output_path, expectation.source_sha256
            ));
        }

        let source_suffix = artifact
            .provenance
            .source_path
            .strip_prefix("source/")
            .ok_or_else(|| {
                format!(
                    "source path lacks canonical prefix: {}",
                    artifact.provenance.source_path
                )
            })?;
        let expected_id_prefix = format!("{source_suffix}#/");

        for raw_case in expectation.cases {
            if !raw_case.id.starts_with(&expected_id_prefix) {
                return Err(format!(
                    "case ID {} is not bound to source {}",
                    raw_case.id, artifact.provenance.source_path
                ));
            }
            if !seen_ids.insert(raw_case.id.clone()) {
                return Err(format!("duplicate case ID {}", raw_case.id));
            }
            if previous_id
                .as_deref()
                .is_some_and(|previous| previous >= raw_case.id.as_str())
            {
                return Err(format!(
                    "case IDs are not in canonical order at {}",
                    raw_case.id
                ));
            }
            previous_id = Some(raw_case.id.clone());

            if raw_case.status != Disposition::Active {
                return Err(format!("case {} has non-active disposition", raw_case.id));
            }
            if raw_case.reason.is_some() {
                return Err(format!(
                    "active case {} carries a disposition reason",
                    raw_case.id
                ));
            }

            match raw_case.upstream_outcome {
                UpstreamOutcome::Parsed => parsed_count += 1,
                UpstreamOutcome::Rejected => rejected_count += 1,
            }
            context_counts[raw_case.context.index()] += 1;
            cases.push(ValidatedCase {
                id: raw_case.id,
                expectation_path: artifact.output_path.clone(),
                expectation_sha256: expected_output_digest.clone(),
                source: artifact.provenance.source_path.clone(),
                context: raw_case.context,
                _label: raw_case.label,
                input: raw_case.input,
                _upstream_outcome: raw_case.upstream_outcome,
                _canonical_css: raw_case.canonical_css,
                options: raw_case.options.unwrap_or_default(),
                expected_class: None,
            });
        }
    }

    validate_census(
        &report,
        &cases,
        parsed_count,
        rejected_count,
        context_counts,
    )?;

    validate_registry(&cases)?;
    validate_expected_classes(&artifacts.expected_classes, &report_digest, &mut cases)?;

    Ok(NeutralInventory {
        artifact_count: report.artifacts.len(),
        cases,
        parsed_count,
        rejected_count,
        context_counts,
        report_digest,
        expected_classes_digest,
    })
}

fn validate_report_header(report: &RawReport) -> Result<(), String> {
    if report.manifest_digest != EXPECTED_MANIFEST_DIGEST {
        return Err(format!(
            "report manifest digest mismatch: expected {EXPECTED_MANIFEST_DIGEST}, report {}",
            report.manifest_digest
        ));
    }
    if report.source_repository != EXPECTED_SOURCE_REPOSITORY {
        return Err(format!(
            "unexpected source repository {}",
            report.source_repository
        ));
    }
    if report.source_revision != EXPECTED_SOURCE_REVISION {
        return Err(format!(
            "unexpected source revision {}",
            report.source_revision
        ));
    }
    if report.counts
        != (ReportCounts {
            active: EXPECTED_CASES,
            expected_fail: 0,
            unsupported: 0,
            quarantined: 0,
            failed_to_generate: 0,
        })
    {
        return Err(format!(
            "unexpected report disposition counts: {:?}",
            report.counts
        ));
    }
    Ok(())
}

fn validate_report_artifact(artifact: &ReportArtifact) -> Result<(), String> {
    if !is_canonical_relative_path(&artifact.output_path)
        || !artifact.output_path.starts_with("expectations/")
        || !artifact.output_path.ends_with(".json")
    {
        return Err(format!(
            "non-canonical expectation path {}",
            artifact.output_path
        ));
    }
    if !is_canonical_relative_path(&artifact.provenance.source_path)
        || !artifact.provenance.source_path.starts_with("source/")
        || !artifact.provenance.source_path.ends_with(".json")
    {
        return Err(format!(
            "non-canonical source path {}",
            artifact.provenance.source_path
        ));
    }
    let output_suffix = artifact
        .output_path
        .strip_prefix("expectations/")
        .expect("validated expectation prefix");
    let source_suffix = artifact
        .provenance
        .source_path
        .strip_prefix("source/")
        .expect("validated source prefix");
    if output_suffix != source_suffix {
        return Err(format!(
            "report path pair does not share a suffix: {} and {}",
            artifact.output_path, artifact.provenance.source_path
        ));
    }
    if artifact.provenance.generator != EXPECTED_GENERATOR {
        return Err(format!(
            "unexpected generator {} for {}",
            artifact.provenance.generator, artifact.output_path
        ));
    }
    if artifact.provenance.schema_version != 1 {
        return Err(format!(
            "unsupported report schema version {} for {}",
            artifact.provenance.schema_version, artifact.output_path
        ));
    }
    if artifact.provenance.domain_provenance.csstree_import != EXPECTED_IMPORT_PROVENANCE {
        return Err(format!(
            "unexpected import provenance for {}",
            artifact.output_path
        ));
    }
    validate_digest("report output", &artifact.output_digest)?;
    validate_digest("report source", &artifact.provenance.source_digest)?;
    Ok(())
}

fn validate_expectation_header(
    expectation: &RawExpectation,
    artifact: &ReportArtifact,
) -> Result<(), String> {
    if expectation.schema_version != 1 {
        return Err(format!(
            "unsupported expectation schema version {} for {}",
            expectation.schema_version, artifact.output_path
        ));
    }
    if expectation.generator != EXPECTED_GENERATOR {
        return Err(format!(
            "unexpected expectation generator {} for {}",
            expectation.generator, artifact.output_path
        ));
    }
    if expectation.source != artifact.provenance.source_path {
        return Err(format!(
            "expectation source mismatch for {}: expectation {}, report {}",
            artifact.output_path, expectation.source, artifact.provenance.source_path
        ));
    }
    if expectation.source_sha256 != artifact.provenance.source_digest {
        return Err(format!(
            "expectation/report source digest mismatch for {}",
            artifact.output_path
        ));
    }
    if expectation.source_revision != EXPECTED_SOURCE_REVISION {
        return Err(format!(
            "unexpected expectation source revision {} for {}",
            expectation.source_revision, artifact.output_path
        ));
    }
    if expectation.import_provenance_sha256 != EXPECTED_IMPORT_PROVENANCE
        || expectation.import_provenance_sha256
            != artifact.provenance.domain_provenance.csstree_import
    {
        return Err(format!(
            "expectation import provenance mismatch for {}",
            artifact.output_path
        ));
    }
    validate_digest("expectation source", &expectation.source_sha256)?;
    Ok(())
}

fn validate_census(
    report: &RawReport,
    cases: &[ValidatedCase],
    parsed_count: usize,
    rejected_count: usize,
    context_counts: [usize; 11],
) -> Result<(), String> {
    if cases.len() != EXPECTED_CASES {
        return Err(format!(
            "case census mismatch: expected {EXPECTED_CASES}, found {}",
            cases.len()
        ));
    }
    if parsed_count != EXPECTED_PARSED || rejected_count != EXPECTED_REJECTED {
        return Err(format!(
            "outcome census mismatch: expected {EXPECTED_PARSED}/{EXPECTED_REJECTED} parsed/rejected, found {parsed_count}/{rejected_count}"
        ));
    }
    if context_counts != EXPECTED_CONTEXT_COUNTS.map(|(_, expected_count)| expected_count) {
        return Err(format!("context census mismatch: {context_counts:?}"));
    }
    if report.counts.active != cases.len() {
        return Err(format!(
            "report active census mismatch: report {}, loaded {}",
            report.counts.active,
            cases.len()
        ));
    }
    Ok(())
}

fn validate_expected_classes(
    bytes: &[u8],
    report_digest: &str,
    cases: &mut [ValidatedCase],
) -> Result<(), String> {
    let raw: RawExpectedClasses = serde_json::from_slice(bytes)
        .map_err(|error| format!("failed to deserialize expected classes: {error}"))?;
    let canonical = format!(
        "{}\n",
        serde_json::to_string_pretty(&raw)
            .map_err(|error| format!("failed to canonicalize expected classes: {error}"))?
    );
    if bytes != canonical.as_bytes() {
        let offset = bytes
            .iter()
            .zip(canonical.as_bytes())
            .position(|(actual, expected)| actual != expected)
            .unwrap_or_else(|| bytes.len().min(canonical.len()));
        return Err(format!(
            "expected classes are not canonical pretty JSON with one final LF at byte {offset}: actual length {}, canonical length {}",
            bytes.len(),
            canonical.len()
        ));
    }
    if raw.schema_version != 1 {
        return Err(format!(
            "unsupported expected-class schema version {}",
            raw.schema_version
        ));
    }
    validate_digest(
        "expected-class generation report",
        &raw.generation_report_sha256,
    )?;
    if raw.generation_report_sha256 != report_digest {
        return Err(format!(
            "expected-class generation report digest mismatch: registry {}, actual {report_digest}",
            raw.generation_report_sha256
        ));
    }
    if raw.records.len() != EXPECTED_CASES {
        return Err(format!(
            "expected-class case census mismatch: expected {EXPECTED_CASES}, found {}",
            raw.records.len()
        ));
    }

    let case_indexes = cases
        .iter()
        .enumerate()
        .map(|(index, case)| (case.id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut previous_id: Option<&str> = None;
    for record in &raw.records {
        if !seen.insert(record.id.as_str()) {
            return Err(format!("duplicate expected-class case ID {}", record.id));
        }
        if previous_id.is_some_and(|previous| previous >= record.id.as_str()) {
            return Err(format!(
                "expected-class records are not in canonical order at {}",
                record.id
            ));
        }
        previous_id = Some(&record.id);
        let Some(index) = case_indexes.get(record.id.as_str()).copied() else {
            return Err(format!("extra expected-class case ID {}", record.id));
        };
        let case = &mut cases[index];
        validate_digest(
            "expected-class neutral expectation",
            &record.expectation_sha256,
        )?;
        if record.expectation_sha256 != case.expectation_sha256 {
            return Err(format!(
                "expected-class expectation digest mismatch for {}: registry {}, neutral {}",
                record.id, record.expectation_sha256, case.expectation_sha256
            ));
        }
        validate_expected_class(case, &record.expected_class)?;
        case.expected_class = Some(record.expected_class.clone());
    }

    if seen.len() != cases.len() {
        let expected = cases.iter().map(|case| case.id.clone()).collect();
        let actual = seen.into_iter().map(str::to_owned).collect();
        return Err(set_mismatch(
            "expected-class case ID inventory",
            &expected,
            &actual,
        ));
    }
    if cases.iter().any(|case| case.expected_class.is_none()) {
        return Err("expected-class registry left an unclassified neutral case".into());
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExpectedClassSummary {
    digest: String,
    class_counts: [usize; 4],
    policy_counts: [usize; 2],
}

impl ExpectedClassSummary {
    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }

    pub(crate) fn class_counts(&self) -> [(&'static str, usize); 4] {
        [
            ("clean", self.class_counts[0]),
            ("recovered", self.class_counts[1]),
            ("strict_rejected", self.class_counts[2]),
            ("unsupported", self.class_counts[3]),
        ]
    }

    pub(crate) fn policy_counts(&self) -> [(&'static str, usize); 2] {
        [
            ("full_observation", self.policy_counts[0]),
            ("panic_freedom_only", self.policy_counts[1]),
        ]
    }
}

pub(crate) fn validate_csstree_expected_classes_contract(
    bytes: &[u8],
) -> Result<ExpectedClassSummary, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
    let mut artifacts = read_neutral_artifact_set(&root)?;
    artifacts.expected_classes = bytes.to_vec();
    let inventory = validate_neutral_artifact_set(artifacts)?;
    let mut class_counts = [0; 4];
    let mut policy_counts = [0; 2];
    for case in &inventory.cases {
        match case
            .expected_class
            .as_ref()
            .expect("validated inventory binds every expected class")
        {
            ExpectedClass::Clean { .. } => class_counts[0] += 1,
            ExpectedClass::Recovered { .. } => class_counts[1] += 1,
            ExpectedClass::StrictRejected { .. } => class_counts[2] += 1,
            ExpectedClass::Unsupported { policy, .. } => {
                class_counts[3] += 1;
                match policy {
                    ExpectedUnsupportedPolicy::FullObservation { .. } => policy_counts[0] += 1,
                    ExpectedUnsupportedPolicy::PanicFreedomOnly => policy_counts[1] += 1,
                }
            }
        }
    }
    Ok(ExpectedClassSummary {
        digest: sha256_hex(bytes),
        class_counts,
        policy_counts,
    })
}

fn resolve_case_registry(
    case: &ValidatedCase,
    registry: RegistryEntry,
) -> Result<RegistryEntry, String> {
    let profile = case.options.profile().map_err(str::to_owned)?;
    registry
        .resolve(profile)
        .map_err(|error| format!("registry options mismatch for {}: {error:?}", case.id))
}

fn validate_expected_class(case: &ValidatedCase, expected: &ExpectedClass) -> Result<(), String> {
    let registry = REGISTRY
        .iter()
        .find(|entry| entry.fixture_path() == case.expectation_path)
        .copied()
        .ok_or_else(|| format!("missing registry entry for expected class {}", case.id))?;
    let registry = resolve_case_registry(case, registry)?;
    match expected {
        ExpectedClass::Clean { retained_syntax } => {
            require_truthful_adapter(case, registry)?;
            validate_expected_retained_syntax(case, registry, retained_syntax)?;
        }
        ExpectedClass::Recovered {
            retained_syntax,
            diagnostics,
        } => {
            require_truthful_adapter(case, registry)?;
            validate_expected_retained_syntax(case, registry, retained_syntax)?;
            if !matches!(
                retained_syntax.predicate,
                SyntaxCount::Nonempty | SyntaxCount::Exact { value: 1.. }
            ) || diagnostics.is_empty()
            {
                return Err(format!(
                    "recovered expected class lacks retained payload syntax or diagnostics for {}",
                    case.id
                ));
            }
        }
        ExpectedClass::StrictRejected {
            retained_syntax,
            diagnostics,
        } => {
            require_truthful_adapter(case, registry)?;
            validate_expected_retained_syntax(case, registry, retained_syntax)?;
            if !matches!(
                retained_syntax.predicate,
                SyntaxCount::Empty | SyntaxCount::Exact { value: 0 }
            ) || diagnostics.is_empty()
            {
                return Err(format!(
                    "strict-rejected expected class must require empty payload syntax and diagnostics for {}",
                    case.id
                ));
            }
        }
        ExpectedClass::Unsupported { reason, policy } => match policy {
            ExpectedUnsupportedPolicy::FullObservation {
                retained_syntax,
                is_clean,
                diagnostics,
            } => {
                if !registry.has_truthful_complete_input()
                    || matches!(
                        reason,
                        UnsupportedReason::GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor
                    )
                {
                    return Err(format!(
                        "full-observation expected class is inconsistent with registry truthfulness for {}",
                        case.id
                    ));
                }
                validate_expected_retained_syntax(case, registry, retained_syntax)?;
                if *is_clean != diagnostics.is_empty() {
                    return Err(format!(
                        "full-observation clean predicate disagrees with diagnostics for {}",
                        case.id
                    ));
                }
            }
            ExpectedUnsupportedPolicy::PanicFreedomOnly => {
                if registry.has_truthful_complete_input()
                    || registry.unsupported_policy()
                        != Some(RegistryUnsupportedPolicy::PanicFreedomOnly)
                    || !matches!(
                        reason,
                        UnsupportedReason::GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor
                    )
                {
                    return Err(format!(
                        "panic-freedom expected class is inconsistent with adapterless registry facts for {}",
                        case.id
                    ));
                }
            }
        },
    }
    Ok(())
}

fn require_truthful_adapter(case: &ValidatedCase, registry: RegistryEntry) -> Result<(), String> {
    if registry.has_truthful_complete_input() {
        Ok(())
    } else {
        Err(format!(
            "supported expected class lacks a truthful complete input for {}",
            case.id
        ))
    }
}

fn validate_expected_retained_syntax(
    case: &ValidatedCase,
    registry: RegistryEntry,
    retained_syntax: &ExpectedRetainedSyntax,
) -> Result<(), String> {
    let expected_extractor = raw_extractor(registry.extractor());
    if retained_syntax.extractor != expected_extractor {
        return Err(format!(
            "expected-class extractor mismatch for {}: registry {:?}, expected class {:?}",
            case.id, expected_extractor, retained_syntax.extractor
        ));
    }
    Ok(())
}

fn validate_oracle(
    bytes: &[u8],
    report_digest: &str,
    expected_classes_digest: &str,
    cases: &[ValidatedCase],
) -> Result<Oracle, String> {
    let raw = load_csstree_oracle_schema(bytes)?;

    if raw.schema_version != 1 {
        return Err(format!(
            "unsupported CSS oracle schema version {}",
            raw.schema_version
        ));
    }
    if raw.provider_repository != EXPECTED_SOURCE_REPOSITORY {
        return Err(format!(
            "unexpected CSS oracle provider {}",
            raw.provider_repository
        ));
    }
    if raw.source_revision != EXPECTED_SOURCE_REVISION {
        return Err(format!(
            "unexpected CSS oracle source revision {}",
            raw.source_revision
        ));
    }
    if raw.source_tree != EXPECTED_SOURCE_TREE {
        return Err(format!(
            "unexpected CSS oracle source tree {}",
            raw.source_tree
        ));
    }
    if raw.expectation_schema_version != 1 {
        return Err(format!(
            "unsupported CSS oracle expectation schema version {}",
            raw.expectation_schema_version
        ));
    }
    validate_digest(
        "CSS oracle generation report",
        &raw.generation_report_sha256,
    )?;
    if raw.generation_report_sha256 != report_digest {
        return Err(format!(
            "CSS oracle generation report digest mismatch: oracle {}, actual {report_digest}",
            raw.generation_report_sha256
        ));
    }
    validate_digest(
        "CSS oracle expected-class registry",
        &raw.expected_class_registry_sha256,
    )?;
    if raw.expected_class_registry_sha256 != expected_classes_digest {
        return Err(format!(
            "CSS oracle expected-class registry digest mismatch: oracle {}, actual {expected_classes_digest}",
            raw.expected_class_registry_sha256
        ));
    }

    if raw.records.len() != EXPECTED_CASES {
        return Err(format!(
            "CSS oracle case census mismatch: expected {EXPECTED_CASES}, found {}",
            raw.records.len()
        ));
    }

    validate_registry(cases)?;

    let failures = collect_oracle_contract_failures(&raw, cases);
    if !failures.is_empty() {
        let detail = failures
            .iter()
            .map(OracleContractFailure::summary)
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!("CSS oracle contract failures:\n{detail}"));
    }

    let expected_ids = cases
        .iter()
        .map(|case| case.id.clone())
        .collect::<BTreeSet<_>>();
    let oracle_ids = raw
        .records
        .iter()
        .map(|record| record.id.clone())
        .collect::<BTreeSet<_>>();
    let mut outcome_counts = [0; 4];
    for record in &raw.records {
        outcome_counts[record.outcome.index()] += 1;
    }

    if oracle_ids != expected_ids {
        return Err(set_mismatch(
            "CSS oracle case ID inventory",
            &expected_ids,
            &oracle_ids,
        ));
    }
    if outcome_counts.iter().sum::<usize>() != EXPECTED_CASES {
        return Err(format!(
            "CSS oracle outcome partition mismatch: {outcome_counts:?}"
        ));
    }

    Ok(Oracle {
        generation_report_sha256: raw.generation_report_sha256,
        id_count: oracle_ids.len(),
        outcome_counts,
    })
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OracleContractFailureKind {
    DocumentContractMismatch,
    NonCanonicalRecordOrder,
    DuplicateCaseId,
    ExtraCaseId,
    ExpectationDigestMismatch,
    SourceMismatch,
    PathMismatch,
    NonCanonicalPath,
    ContextMismatch,
    InputMismatch,
    OptionsMismatch,
    RegistryMissing,
    RegistryContextMismatch,
    IllegalOptions,
    RegistryOptionsMismatch,
    ProbeMismatch,
    ProbeOutcomePolicyMismatch,
    UnsupportedPolicyObservationMismatch,
    OutcomeMismatch,
    ExpectedClassMismatch,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct OracleContractFailure {
    case_id: String,
    path: String,
    context: String,
    kind: OracleContractFailureKind,
    source: String,
    options: String,
    input: String,
    detail: String,
}

impl OracleContractFailure {
    fn for_record(
        record: &RawOracleRecord,
        kind: OracleContractFailureKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            case_id: record.id.clone(),
            path: record.path.clone(),
            context: record.context.as_str().to_owned(),
            kind,
            source: record.source.clone(),
            options: serde_json::to_string(&record.options)
                .expect("serializing closed CSS oracle options cannot fail"),
            input: record.input.clone(),
            detail: detail.into(),
        }
    }

    pub(crate) fn case_id(&self) -> &str {
        &self.case_id
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn context(&self) -> &str {
        &self.context
    }

    pub(crate) const fn kind(&self) -> OracleContractFailureKind {
        self.kind
    }

    fn summary(&self) -> String {
        format!(
            "{:?}: case_id={:?} path={:?} context={:?} source={:?} options={} input={:?}: {}",
            self.kind,
            self.case_id,
            self.path,
            self.context,
            self.source,
            self.options,
            self.input,
            self.detail
        )
    }
}

pub(crate) fn validate_csstree_oracle_contract(
    bytes: &[u8],
) -> Result<(), Vec<OracleContractFailure>> {
    let inventory = load_neutral_csstree_inventory().map_err(|error| {
        vec![oracle_document_failure(
            bytes,
            format!("committed neutral corpus is invalid: {error}"),
        )]
    })?;

    match validate_oracle(
        bytes,
        &inventory.report_digest,
        &inventory.expected_classes_digest,
        &inventory.cases,
    ) {
        Ok(_) => Ok(()),
        Err(error) => {
            if let Ok(raw) = load_csstree_oracle_schema(bytes) {
                let failures = collect_oracle_contract_failures(&raw, &inventory.cases);
                if !failures.is_empty() {
                    return Err(failures);
                }
            }
            Err(vec![oracle_document_failure(bytes, error)])
        }
    }
}

fn oracle_document_failure(bytes: &[u8], detail: String) -> OracleContractFailure {
    let document = serde_json::from_slice::<serde_json::Value>(bytes).ok();
    let record = document
        .as_ref()
        .and_then(|document| document.get("records"))
        .and_then(serde_json::Value::as_array)
        .and_then(|records| records.first());
    let field = |name: &str| {
        record
            .and_then(|record| record.get(name))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<oracle>")
            .to_owned()
    };
    let options = record
        .and_then(|record| record.get("options"))
        .map_or_else(|| "<oracle>".to_owned(), serde_json::Value::to_string);
    OracleContractFailure {
        case_id: field("id"),
        path: field("path"),
        context: field("context"),
        kind: OracleContractFailureKind::DocumentContractMismatch,
        source: field("source"),
        options,
        input: field("input"),
        detail,
    }
}

fn collect_oracle_contract_failures(
    raw: &RawOracle,
    cases: &[ValidatedCase],
) -> Vec<OracleContractFailure> {
    let expected = cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let mut failures = Vec::new();
    let mut seen = BTreeSet::new();
    let mut previous_id: Option<&str> = None;

    for record in &raw.records {
        let mut reject = |kind, detail: String| {
            failures.push(OracleContractFailure::for_record(record, kind, detail));
        };

        if !seen.insert(record.id.as_str()) {
            reject(
                OracleContractFailureKind::DuplicateCaseId,
                "duplicate CSS oracle case ID".into(),
            );
        }
        if previous_id.is_some_and(|previous| previous >= record.id.as_str()) {
            reject(
                OracleContractFailureKind::NonCanonicalRecordOrder,
                format!(
                    "CSS oracle records are not in canonical order; previous case ID was {:?}",
                    previous_id.unwrap_or_default()
                ),
            );
        }
        previous_id = Some(&record.id);

        let Some(case) = expected.get(record.id.as_str()).copied() else {
            reject(
                OracleContractFailureKind::ExtraCaseId,
                "extra CSS oracle case ID is absent from neutral expectations".into(),
            );
            continue;
        };

        if validate_digest("CSS oracle neutral expectation", &record.expectation_sha256).is_err()
            || record.expectation_sha256 != case.expectation_sha256
        {
            reject(
                OracleContractFailureKind::ExpectationDigestMismatch,
                format!(
                    "oracle digest {}, neutral digest {}",
                    record.expectation_sha256, case.expectation_sha256
                ),
            );
        }
        if record.source != case.source {
            reject(
                OracleContractFailureKind::SourceMismatch,
                format!(
                    "oracle source {:?}, neutral source {:?}",
                    record.source, case.source
                ),
            );
        }
        if record.path != case.expectation_path {
            reject(
                OracleContractFailureKind::PathMismatch,
                format!(
                    "CSS oracle path mismatch: oracle path {:?}, neutral path {:?}",
                    record.path, case.expectation_path
                ),
            );
        }
        if !is_canonical_relative_path(&record.path)
            || !record.path.starts_with("expectations/")
            || !record.path.ends_with(".json")
        {
            reject(
                OracleContractFailureKind::NonCanonicalPath,
                "expectation path is not canonical".into(),
            );
        }
        if record.context != case.context {
            reject(
                OracleContractFailureKind::ContextMismatch,
                format!(
                    "CSS oracle context mismatch: oracle context {:?}, neutral context {:?}",
                    record.context.as_str(),
                    case.context.as_str()
                ),
            );
        }
        if record.input != case.input {
            reject(
                OracleContractFailureKind::InputMismatch,
                "CSS oracle input mismatch: oracle input differs from neutral input".into(),
            );
        }
        if record.options != case.options {
            reject(
                OracleContractFailureKind::OptionsMismatch,
                "CSS oracle options mismatch: oracle options differ from neutral options".into(),
            );
        }

        let Some(registry) = REGISTRY
            .iter()
            .find(|entry| entry.fixture_path() == case.expectation_path)
            .copied()
        else {
            reject(
                OracleContractFailureKind::RegistryMissing,
                "neutral expectation path has no registry entry".into(),
            );
            continue;
        };
        if registry.context() != record.context.as_str() {
            reject(
                OracleContractFailureKind::RegistryContextMismatch,
                format!("registry context is {:?}", registry.context()),
            );
        }
        match record.options.profile() {
            Ok(profile) if !registry.accepts_options(profile) => reject(
                OracleContractFailureKind::RegistryOptionsMismatch,
                format!("registry rejects options profile {profile:?}"),
            ),
            Ok(_) => {}
            Err(error) => reject(OracleContractFailureKind::IllegalOptions, error.to_owned()),
        }
        let registry = match resolve_case_registry(case, registry) {
            Ok(registry) => registry,
            Err(error) => {
                reject(OracleContractFailureKind::RegistryOptionsMismatch, error);
                continue;
            }
        };
        if let Err(error) = validate_probe(record, registry) {
            reject(OracleContractFailureKind::ProbeMismatch, error);
        }
        if !probe_outcome_policy_is_legal(&record.probe, &record.outcome) {
            reject(
                OracleContractFailureKind::ProbeOutcomePolicyMismatch,
                "active and panic-freedom probe policies are inconsistent".into(),
            );
        }
        if let Err(error) = validate_outcome(record) {
            let kind = if matches!(&record.outcome, Outcome::Unsupported { .. }) {
                OracleContractFailureKind::UnsupportedPolicyObservationMismatch
            } else {
                OracleContractFailureKind::OutcomeMismatch
            };
            reject(kind, error);
        }
        if let Err(error) = validate_oracle_expected_class(record, case) {
            reject(OracleContractFailureKind::ExpectedClassMismatch, error);
        }
    }

    failures.sort();
    failures
}

fn validate_oracle_expected_class(
    record: &RawOracleRecord,
    case: &ValidatedCase,
) -> Result<(), String> {
    let expected = case.expected_class.as_ref().ok_or_else(|| {
        format!(
            "CSS oracle case {} has no prevalidated expected class",
            case.id
        )
    })?;
    let matches = match (expected, &record.outcome) {
        (ExpectedClass::Clean { retained_syntax }, Outcome::Clean) => {
            record.observation.0.as_ref().is_some_and(|observation| {
                observation_matches_expected(observation, retained_syntax, true, &[])
            })
        }
        (
            ExpectedClass::Recovered {
                retained_syntax,
                diagnostics,
            },
            Outcome::Recovered,
        )
        | (
            ExpectedClass::StrictRejected {
                retained_syntax,
                diagnostics,
            },
            Outcome::StrictRejected,
        ) => record.observation.0.as_ref().is_some_and(|observation| {
            observation_matches_expected(observation, retained_syntax, false, diagnostics)
        }),
        (
            ExpectedClass::Unsupported {
                reason: expected_reason,
                policy: expected_policy,
            },
            Outcome::Unsupported { reason, policy },
        ) => {
            expected_reason == reason
                && match (expected_policy, policy, record.observation.0.as_ref()) {
                    (
                        ExpectedUnsupportedPolicy::PanicFreedomOnly,
                        UnsupportedPolicy::PanicFreedomOnly,
                        None,
                    ) => true,
                    (
                        ExpectedUnsupportedPolicy::FullObservation {
                            retained_syntax,
                            is_clean,
                            diagnostics,
                        },
                        UnsupportedPolicy::FullObservation,
                        Some(observation),
                    ) => observation_matches_expected(
                        observation,
                        retained_syntax,
                        *is_clean,
                        diagnostics,
                    ),
                    _ => false,
                }
        }
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(format!(
            "CSS oracle outcome {:?} diverges from pre-bound expected class {:?} for {}",
            record.outcome, expected, case.id
        ))
    }
}

fn observation_matches_expected(
    observation: &Observation,
    retained_syntax: &ExpectedRetainedSyntax,
    is_clean: bool,
    diagnostics: &[ExpectedDiagnostic],
) -> bool {
    retained_syntax.predicate.matches(observation.syntax_count)
        && observation.extractor == retained_syntax.extractor
        && observation.is_clean == is_clean
        && observation.diagnostics.len() == diagnostics.len()
        && observation
            .diagnostics
            .iter()
            .zip(diagnostics)
            .all(|(actual, expected)| {
                actual.code == expected.code
                    && actual.action == expected.action
                    && actual.payload_relation == expected.payload_relation
            })
}

fn probe_outcome_policy_is_legal(probe: &Probe, outcome: &Outcome) -> bool {
    matches!(
        (probe, outcome),
        (
            Probe::PanicFreedom { .. },
            Outcome::Unsupported {
                policy: UnsupportedPolicy::PanicFreedomOnly,
                ..
            }
        ) | (
            Probe::Active { .. },
            Outcome::Clean | Outcome::Recovered | Outcome::StrictRejected
        ) | (
            Probe::Active { .. },
            Outcome::Unsupported {
                policy: UnsupportedPolicy::FullObservation,
                ..
            }
        )
    )
}

fn validate_registry(cases: &[ValidatedCase]) -> Result<(), String> {
    if !adapters::validate_closed_model() {
        return Err("CSS adapter/extractor closed model is internally inconsistent".into());
    }
    if REGISTRY.len() != EXPECTED_ARTIFACTS {
        return Err(format!(
            "CSS adapter registry census mismatch: expected {EXPECTED_ARTIFACTS}, found {}",
            REGISTRY.len()
        ));
    }
    let expected = cases
        .iter()
        .map(|case| case.expectation_path.clone())
        .collect::<BTreeSet<_>>();
    let actual = REGISTRY
        .iter()
        .map(|entry| entry.fixture_path().to_owned())
        .collect::<BTreeSet<_>>();
    if actual.len() != REGISTRY.len() {
        return Err("CSS adapter registry contains a duplicate fixture path".into());
    }
    if expected != actual {
        return Err(set_mismatch("CSS adapter registry", &expected, &actual));
    }
    if let Some(entry) = REGISTRY
        .iter()
        .find(|entry| !entry.has_legal_context_combination())
    {
        return Err(format!(
            "CSS adapter registry has an illegal context combination for {}",
            entry.fixture_path()
        ));
    }
    Ok(())
}

pub(crate) fn load_csstree_oracle_schema(bytes: &[u8]) -> Result<RawOracle, String> {
    let raw: RawOracle = serde_json::from_slice(bytes)
        .map_err(|error| format!("failed to deserialize CSS oracle: {error}"))?;
    let canonical = canonicalize_raw_oracle(&raw)?;
    if bytes != canonical.as_bytes() {
        return Err("CSS oracle is not canonical pretty JSON with one final LF".into());
    }
    Ok(raw)
}

pub(crate) fn canonicalize_csstree_oracle_schema(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let raw: RawOracle = serde_json::from_slice(bytes)
        .map_err(|error| format!("failed to deserialize CSS oracle: {error}"))?;
    canonicalize_raw_oracle(&raw).map(String::into_bytes)
}

fn canonicalize_raw_oracle(raw: &RawOracle) -> Result<String, String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(raw)
            .map_err(|error| format!("failed to canonicalize CSS oracle: {error}"))?
    ))
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BaselineFailureKind {
    RegistryMissing,
    AdapterMismatch,
    ParserPanicked,
    ExtractorMismatch,
    DiagnosticOutsidePayload,
    ExpectedClassMismatch,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct BaselineFailure {
    case_id: String,
    path: String,
    context: String,
    kind: BaselineFailureKind,
    detail: String,
}

impl BaselineFailure {
    fn new(case: &ValidatedCase, kind: BaselineFailureKind, detail: impl Into<String>) -> Self {
        Self {
            case_id: case.id.clone(),
            path: case.expectation_path.clone(),
            context: case.context.as_str().to_owned(),
            kind,
            detail: detail.into(),
        }
    }

    fn summary(&self) -> String {
        format!(
            "{:?}: case_id={:?} path={:?} context={:?}: {}",
            self.kind, self.case_id, self.path, self.context, self.detail
        )
    }
}

pub(crate) fn regenerate_csstree_oracle() -> Result<Vec<u8>, Vec<BaselineFailure>> {
    let corpus = load_csstree_corpus().map_err(|error| {
        vec![BaselineFailure {
            case_id: "<corpus>".into(),
            path: ORACLE_PATH.into(),
            context: "<corpus>".into(),
            kind: BaselineFailureKind::AdapterMismatch,
            detail: error.into(),
        }]
    })?;
    observe_csstree_oracle(&corpus.inventory)
}

pub(crate) fn capture_csstree_oracle_from_public_parser() -> Result<(), String> {
    if std::env::var_os("SURGEIST_CSSTREE_WRITE_ORACLE").as_deref()
        != Some(std::ffi::OsStr::new("1"))
    {
        return Err("refusing to write CSS oracle without SURGEIST_CSSTREE_WRITE_ORACLE=1".into());
    }

    let inventory = load_neutral_csstree_inventory().map_err(str::to_owned)?;
    require_expected_classes_digest(&inventory.expected_classes_digest)?;
    let bytes = observe_csstree_oracle(inventory).map_err(|failures| {
        failures
            .iter()
            .map(BaselineFailure::summary)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    require_expected_classes_digest(&inventory.expected_classes_digest)?;
    let replacement = load_csstree_oracle_schema(&bytes)?;
    validate_oracle(
        &bytes,
        &inventory.report_digest,
        &inventory.expected_classes_digest,
        &inventory.cases,
    )?;

    let oracle_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(ORACLE_PATH);
    let existing = read_file(&oracle_path)?;
    let existing = load_csstree_oracle_schema(&existing)?;
    validate_replacement_bindings(&existing, &replacement)?;
    atomic_write_oracle(&oracle_path, &bytes)?;
    require_expected_classes_digest(&inventory.expected_classes_digest)?;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
    read_artifact_set(&root)
        .and_then(validate_artifact_set)
        .map(|_| ())
        .map_err(|error| format!("reloading ValidatedCorpus after oracle capture failed: {error}"))
}

fn require_expected_classes_digest(expected: &str) -> Result<(), String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED_CLASSES_PATH);
    let actual = sha256_hex(&read_file(&path)?);
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "immutable expected-class registry changed during parser/capture command: expected {expected}, found {actual}"
        ))
    }
}

fn observe_csstree_oracle(inventory: &NeutralInventory) -> Result<Vec<u8>, Vec<BaselineFailure>> {
    let mut records = Vec::with_capacity(inventory.cases.len());
    let mut failures = Vec::new();

    for case in &inventory.cases {
        match observe_csstree_record(case) {
            Ok(record) => records.push(record),
            Err(failure) => failures.push(failure),
        }
    }
    failures.sort();
    if !failures.is_empty() {
        return Err(failures);
    }

    let oracle = RawOracle {
        schema_version: 1,
        provider_repository: EXPECTED_SOURCE_REPOSITORY.into(),
        source_revision: EXPECTED_SOURCE_REVISION.into(),
        source_tree: EXPECTED_SOURCE_TREE.into(),
        expectation_schema_version: 1,
        generation_report_sha256: inventory.report_digest.clone(),
        expected_class_registry_sha256: inventory.expected_classes_digest.clone(),
        records,
    };
    canonicalize_raw_oracle(&oracle)
        .map(String::into_bytes)
        .map_err(|error| {
            vec![BaselineFailure {
                case_id: "<oracle>".into(),
                path: ORACLE_PATH.into(),
                context: "<oracle>".into(),
                kind: BaselineFailureKind::AdapterMismatch,
                detail: error,
            }]
        })
}

fn observe_csstree_record(case: &ValidatedCase) -> Result<RawOracleRecord, BaselineFailure> {
    let registry = REGISTRY
        .iter()
        .find(|entry| entry.fixture_path() == case.expectation_path)
        .copied()
        .ok_or_else(|| {
            BaselineFailure::new(
                case,
                BaselineFailureKind::RegistryMissing,
                "neutral expectation path has no explicit registry entry",
            )
        })?;
    let registry = resolve_case_registry(case, registry)
        .map_err(|error| BaselineFailure::new(case, BaselineFailureKind::AdapterMismatch, error))?;
    let complete = registry
        .adapter()
        .wrap(&case.input, registry.property_or_descriptor())
        .map_err(|mismatch| {
            BaselineFailure::new(
                case,
                BaselineFailureKind::AdapterMismatch,
                format!("complete-input adapter failed: {mismatch:?}"),
            )
        })?;
    let span = complete.payload_span();
    let payload = Payload {
        prefix: complete.source()[..span.start].to_owned(),
        suffix: complete.source()[span.end..].to_owned(),
        input_byte_length: case.input.len(),
    };

    let expected_class = case.expected_class.as_ref().ok_or_else(|| {
        BaselineFailure::new(
            case,
            BaselineFailureKind::ExpectedClassMismatch,
            "neutral case reached observation without a prevalidated expected class",
        )
    })?;

    let (probe, outcome, observation) = if matches!(
        expected_class,
        ExpectedClass::Unsupported {
            policy: ExpectedUnsupportedPolicy::PanicFreedomOnly,
            ..
        }
    ) {
        let parsed = catch_unwind(AssertUnwindSafe(|| {
            let _report = parse_style_attribute(complete.source());
            validate_strict_parity(
                _report.is_clean(),
                _report.diagnostics(),
                validate_style_attribute(complete.source()),
            )?;
            Ok::<(), String>(())
        }));
        match parsed {
            Err(_) => {
                return Err(BaselineFailure::new(
                    case,
                    BaselineFailureKind::ParserPanicked,
                    "public parse_style_attribute unwound for adapterless panic-freedom probe",
                ));
            }
            Ok(Err(error)) => {
                return Err(BaselineFailure::new(
                    case,
                    BaselineFailureKind::ExpectedClassMismatch,
                    error,
                ));
            }
            Ok(Ok(())) => {}
        }
        let ExpectedClass::Unsupported { reason, .. } = expected_class else {
            unreachable!("matched panic-freedom expected class")
        };
        (
            Probe::PanicFreedom {
                entry_point: EntryPoint::StyleAttribute,
                adapter: Adapter::CustomPropertyContainment,
                payload,
            },
            Outcome::Unsupported {
                reason: *reason,
                policy: UnsupportedPolicy::PanicFreedomOnly,
            },
            NullableObservation(None),
        )
    } else {
        if !registry.has_truthful_complete_input() {
            return Err(BaselineFailure::new(
                case,
                BaselineFailureKind::AdapterMismatch,
                "active expected class lacks a truthful complete-input registry entry",
            ));
        }
        let observation = observe_public_parser(case, registry, &complete)?;
        let outcome = outcome_for_expected_class(case, expected_class, &observation)?;
        (
            Probe::Active {
                entry_point: registry.entry_point(),
                adapter: registry.adapter(),
                extractor: raw_extractor(registry.extractor()),
                property_or_descriptor: NullableString(
                    registry
                        .property_or_descriptor()
                        .map(PropertyOrDescriptor::name)
                        .map(str::to_owned),
                ),
                options: case.options.clone(),
                payload,
            },
            outcome,
            NullableObservation(Some(observation)),
        )
    };

    Ok(RawOracleRecord {
        id: case.id.clone(),
        path: case.expectation_path.clone(),
        expectation_sha256: case.expectation_sha256.clone(),
        source: case.source.clone(),
        context: case.context,
        input: case.input.clone(),
        options: case.options.clone(),
        probe,
        outcome,
        observation,
    })
}

fn outcome_for_expected_class(
    case: &ValidatedCase,
    expected: &ExpectedClass,
    observation: &Observation,
) -> Result<Outcome, BaselineFailure> {
    let mismatch = |detail: String| {
        BaselineFailure::new(case, BaselineFailureKind::ExpectedClassMismatch, detail)
    };
    match expected {
        ExpectedClass::Clean { retained_syntax } => {
            require_observation_predicates(case, observation, retained_syntax, true, &[])?;
            Ok(Outcome::Clean)
        }
        ExpectedClass::Recovered {
            retained_syntax,
            diagnostics,
        } => {
            require_observation_predicates(case, observation, retained_syntax, false, diagnostics)?;
            Ok(Outcome::Recovered)
        }
        ExpectedClass::StrictRejected {
            retained_syntax,
            diagnostics,
        } => {
            require_observation_predicates(case, observation, retained_syntax, false, diagnostics)?;
            Ok(Outcome::StrictRejected)
        }
        ExpectedClass::Unsupported { reason, policy } => {
            let ExpectedUnsupportedPolicy::FullObservation {
                retained_syntax,
                is_clean,
                diagnostics,
            } = policy
            else {
                return Err(mismatch(
                    "panic-freedom expected class reached full observation".into(),
                ));
            };
            require_observation_predicates(
                case,
                observation,
                retained_syntax,
                *is_clean,
                diagnostics,
            )?;
            Ok(Outcome::Unsupported {
                reason: *reason,
                policy: UnsupportedPolicy::FullObservation,
            })
        }
    }
}

fn require_observation_predicates(
    case: &ValidatedCase,
    observation: &Observation,
    retained_syntax: &ExpectedRetainedSyntax,
    is_clean: bool,
    diagnostics: &[ExpectedDiagnostic],
) -> Result<(), BaselineFailure> {
    if observation_matches_expected(observation, retained_syntax, is_clean, diagnostics) {
        return Ok(());
    }
    Err(BaselineFailure::new(
        case,
        BaselineFailureKind::ExpectedClassMismatch,
        format!(
            "expected retained_syntax={retained_syntax:?} is_clean={is_clean} diagnostics={diagnostics:?}; observed syntax_count={} is_clean={} diagnostics={:?}",
            observation.syntax_count, observation.is_clean, observation.diagnostics
        ),
    ))
}

fn observe_public_parser(
    case: &ValidatedCase,
    registry: RegistryEntry,
    complete: &adapters::CompleteInput,
) -> Result<Observation, BaselineFailure> {
    let observed = catch_unwind(AssertUnwindSafe(|| match registry.entry_point() {
        EntryPoint::Sheet => {
            let report = parse_sheet(complete.source());
            let count = registry
                .extractor()
                .extract_sheet(report.syntax())
                .map_err(|mismatch| format!("public sheet extractor failed: {mismatch:?}"))?;
            let observation = observation_from_report(
                raw_extractor(registry.extractor()),
                count,
                report.is_clean(),
                report.diagnostics(),
                complete,
            )?;
            validate_strict_parity(
                report.is_clean(),
                report.diagnostics(),
                validate_sheet(complete.source()),
            )?;
            Ok::<Observation, String>(observation)
        }
        EntryPoint::StyleBlock => {
            let report = parse_style_block(complete.source(), &corpus_namespace_context());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::StyleBlock,
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::Rule => {
            let report = parse_rule(complete.source(), &corpus_namespace_context());
            match registry.adapter() {
                Adapter::TopLevelAtRule => {
                    let count = registry
                        .extractor()
                        .extract_at_rule(report.syntax())
                        .map_err(|mismatch| {
                            format!("public at-rule extractor failed: {mismatch:?}")
                        })?;
                    fragment_observation(report, registry, registry.extractor(), complete, |_| {
                        count
                    })
                }
                Adapter::TopLevelRule => fragment_observation(
                    report,
                    registry,
                    RegistryExtractor::SheetRules,
                    complete,
                    |syntax| usize::from(syntax.is_some()),
                ),
                adapter => Err(format!("public rule adapter mismatch: {adapter:?}")),
            }
        }
        EntryPoint::Declaration => {
            let report = parse_declaration(complete.source());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::StyleDeclarations,
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::PropertyValueText => {
            let Some(PropertyOrDescriptor::Property(property)) = registry.property_or_descriptor()
            else {
                return Err("raw property value entry requires property context".into());
            };
            let report = parse_property_value_text(
                complete.source(),
                CssPropertyNameRef::Known(property),
                CssImportance::Normal,
            );
            if report
                .syntax()
                .as_ref()
                .is_some_and(|value| value.property_name() != CssPropertyNameRef::Known(property))
            {
                return Err("raw value does not match its property context".into());
            }
            fragment_observation(
                report,
                registry,
                RegistryExtractor::KnownDeclaration { index: 0, property },
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::Selector => {
            let report = parse_selector(complete.source(), &corpus_namespace_context());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::Selector,
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::SelectorList => {
            let report = parse_selector_list(complete.source(), &corpus_namespace_context());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::SelectorList,
                complete,
                |syntax| syntax.as_ref().map_or(0, |list| list.selectors().len()),
            )
        }
        EntryPoint::MediaQuery => {
            let report = parse_media_query(complete.source());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::MediaQuery,
                complete,
                |_| 1,
            )
        }
        EntryPoint::MediaQueryList => {
            let report = parse_media_query_list(complete.source());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::MediaQueries,
                complete,
                |syntax| syntax.queries().len(),
            )
        }
        EntryPoint::FontFaceDescriptorValue => {
            let Some(PropertyOrDescriptor::FontFaceDescriptor(kind)) =
                registry.property_or_descriptor()
            else {
                return Err("font-face value entry point requires descriptor context".into());
            };
            let report = parse_font_face_descriptor_value(complete.source(), kind.css_kind());
            if report
                .syntax()
                .as_ref()
                .is_some_and(|value| value.kind() != kind.css_kind())
            {
                return Err("font-face value does not match its descriptor context".into());
            }
            fragment_observation(
                report,
                registry,
                RegistryExtractor::FontFaceDescriptor(kind),
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::StyleAttribute => {
            let report = parse_style_attribute(complete.source());
            let count = registry
                .extractor()
                .extract_declaration_list(report.syntax())
                .map_err(|mismatch| {
                    format!("public style-attribute extractor failed: {mismatch:?}")
                })?;
            let observation = observation_from_report(
                raw_extractor(registry.extractor()),
                count,
                report.is_clean(),
                report.diagnostics(),
                complete,
            )?;
            validate_strict_parity(
                report.is_clean(),
                report.diagnostics(),
                validate_style_attribute(complete.source()),
            )?;
            Ok::<Observation, String>(observation)
        }
    }));

    match observed {
        Ok(Ok(observation)) => Ok(observation),
        Ok(Err(error)) => Err(BaselineFailure::new(
            case,
            if error.contains("payload relation") {
                BaselineFailureKind::DiagnosticOutsidePayload
            } else {
                BaselineFailureKind::ExtractorMismatch
            },
            error,
        )),
        Err(_) => Err(BaselineFailure::new(
            case,
            BaselineFailureKind::ParserPanicked,
            "public parser or extractor unwound",
        )),
    }
}

fn corpus_namespace_context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").expect("fixed corpus namespace prefix")),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )])
}

fn fragment_observation<T>(
    report: surgeist_css::CssParseReport<T>,
    registry: RegistryEntry,
    expected_extractor: RegistryExtractor,
    complete: &adapters::CompleteInput,
    count: impl FnOnce(&T) -> usize,
) -> Result<Observation, String> {
    if registry.extractor() != expected_extractor {
        return Err(format!(
            "public fragment extractor mismatch: {:?}",
            registry.extractor()
        ));
    }
    let observation = observation_from_report(
        raw_extractor(expected_extractor),
        count(report.syntax()),
        report.is_clean(),
        report.diagnostics(),
        complete,
    )?;
    let clean = report.is_clean();
    let diagnostics = report.diagnostics().to_vec();
    validate_strict_parity(clean, &diagnostics, report.into_validation_result())?;
    Ok(observation)
}

fn validate_strict_parity<T>(
    is_clean: bool,
    diagnostics: &[CssRecoveryDiagnostic],
    strict: Result<T, surgeist_css::CssValidationFailure>,
) -> Result<(), String> {
    match (is_clean, strict) {
        (true, Ok(_)) => Ok(()),
        (false, Err(failure)) if failure.diagnostics() == diagnostics => Ok(()),
        (true, Err(failure)) => Err(format!(
            "app-strict rejected a clean ordinary report with diagnostics {:?}",
            failure.diagnostics()
        )),
        (false, Ok(_)) => Err("app-strict accepted a recovered ordinary report".into()),
        (false, Err(failure)) => Err(format!(
            "app-strict diagnostics diverged from ordinary report: strict={:?} ordinary={diagnostics:?}",
            failure.diagnostics()
        )),
    }
}

fn observation_from_report(
    extractor: Extractor,
    syntax_count: usize,
    is_clean: bool,
    diagnostics: &[CssRecoveryDiagnostic],
    complete: &adapters::CompleteInput,
) -> Result<Observation, String> {
    let payload_span = complete.payload_span();
    let diagnostics = diagnostics
        .iter()
        .map(|diagnostic| {
            let byte_offset = diagnostic.error().position().byte_offset().value();
            let span_start = diagnostic.span().start().byte_offset().value();
            let span_end = diagnostic.span().end().byte_offset().value();
            let intersects = payload_relation_holds(
                PayloadRelation::Intersects,
                payload_span.clone(),
                byte_offset,
                span_start,
                span_end,
            );
            let ends_at = payload_relation_holds(
                PayloadRelation::EndsAt,
                payload_span.clone(),
                byte_offset,
                span_start,
                span_end,
            );
            let recovery_ends_at = payload_relation_holds(
                PayloadRelation::RecoveryEndsAt,
                payload_span.clone(),
                byte_offset,
                span_start,
                span_end,
            );
            let payload_relation = if intersects {
                PayloadRelation::Intersects
            } else if ends_at {
                PayloadRelation::EndsAt
            } else if recovery_ends_at {
                PayloadRelation::RecoveryEndsAt
            } else {
                return Err(format!(
                    "no closed payload relation holds for payload span {payload_span:?}: \
                     diagnostic offset={byte_offset} span={span_start}..{span_end}"
                ));
            };
            Ok::<RawDiagnostic, String>(RawDiagnostic {
                code: css_error_code_name(diagnostic.error().code())?,
                action: css_recovery_action_name(diagnostic.action())?,
                byte_offset,
                span_start,
                span_end,
                multiplicity: 1,
                payload_relation,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Observation {
        extractor,
        syntax_count,
        is_clean,
        diagnostics,
    })
}

fn raw_extractor(extractor: RegistryExtractor) -> Extractor {
    match extractor {
        RegistryExtractor::SheetRules => Extractor::SheetRules,
        RegistryExtractor::AtRule => Extractor::AtRule {},
        RegistryExtractor::TopLevelRuleKind(kind) => Extractor::TopLevelRuleKind {
            rule_kind: kind.name().into(),
        },
        RegistryExtractor::StyleBlock => Extractor::StyleBlock,
        RegistryExtractor::StyleDeclarations => Extractor::StyleDeclarations,
        RegistryExtractor::StyleSelector => Extractor::StyleSelector,
        RegistryExtractor::Selector => Extractor::Selector,
        RegistryExtractor::SelectorList => Extractor::SelectorList,
        RegistryExtractor::MediaQuery => Extractor::MediaQuery,
        RegistryExtractor::DeclarationList => Extractor::DeclarationList,
        RegistryExtractor::MediaQueries => Extractor::MediaQueries,
        RegistryExtractor::MediaChildren => Extractor::MediaChildren,
        RegistryExtractor::SupportsChildren => Extractor::SupportsChildren,
        RegistryExtractor::ContainerChildren => Extractor::ContainerChildren,
        RegistryExtractor::ScopeChildren => Extractor::ScopeChildren,
        RegistryExtractor::LayerBlockChildren => Extractor::LayerBlockChildren,
        RegistryExtractor::FontFaceDescriptor(descriptor) => Extractor::FontFaceDescriptor {
            descriptor: descriptor.name().into(),
        },
        RegistryExtractor::KnownDeclaration { index, property } => Extractor::KnownDeclaration {
            index,
            property: property.canonical_name().into(),
        },
    }
}

fn css_error_code_name(code: CssErrorCode) -> Result<CssErrorCodeName, String> {
    match code {
        CssErrorCode::UnexpectedEnd => Ok(CssErrorCodeName::UnexpectedEnd),
        CssErrorCode::UnexpectedToken => Ok(CssErrorCodeName::UnexpectedToken),
        CssErrorCode::InvalidEncodingDeclaration => {
            Ok(CssErrorCodeName::InvalidEncodingDeclaration)
        }
        CssErrorCode::InvalidAtRulePlacement => Ok(CssErrorCodeName::InvalidAtRulePlacement),
        CssErrorCode::InvalidAtRulePrelude => Ok(CssErrorCodeName::InvalidAtRulePrelude),
        CssErrorCode::InvalidAtRuleBody => Ok(CssErrorCodeName::InvalidAtRuleBody),
        CssErrorCode::UnknownAtRule => Ok(CssErrorCodeName::UnknownAtRule),
        CssErrorCode::UnsupportedAtRule => Ok(CssErrorCodeName::UnsupportedAtRule),
        CssErrorCode::InvalidQualifiedRule => Ok(CssErrorCodeName::InvalidQualifiedRule),
        CssErrorCode::InvalidSelector => Ok(CssErrorCodeName::InvalidSelector),
        CssErrorCode::InvalidMediaQuery => Ok(CssErrorCodeName::InvalidMediaQuery),
        CssErrorCode::UnknownProperty => Ok(CssErrorCodeName::UnknownProperty),
        CssErrorCode::UnsupportedProperty => Ok(CssErrorCodeName::UnsupportedProperty),
        CssErrorCode::InvalidPropertyValue => Ok(CssErrorCodeName::InvalidPropertyValue),
        CssErrorCode::InvalidDeclarationAnnotation => {
            Ok(CssErrorCodeName::InvalidDeclarationAnnotation)
        }
        CssErrorCode::UnknownDescriptor => Ok(CssErrorCodeName::UnknownDescriptor),
        CssErrorCode::UnsupportedDescriptor => Ok(CssErrorCodeName::UnsupportedDescriptor),
        CssErrorCode::InvalidDescriptorValue => Ok(CssErrorCodeName::InvalidDescriptorValue),
        CssErrorCode::InvalidDescriptorCombination => {
            Ok(CssErrorCodeName::InvalidDescriptorCombination)
        }
        CssErrorCode::InvalidColorSyntax => Ok(CssErrorCodeName::InvalidColorSyntax),
        CssErrorCode::NestingLimit => Ok(CssErrorCodeName::NestingLimit),
        _ => Err("public parser returned an unrecognized CssErrorCode variant".into()),
    }
}

fn css_recovery_action_name(action: CssRecoveryAction) -> Result<CssRecoveryActionName, String> {
    match action {
        CssRecoveryAction::RejectInput => Ok(CssRecoveryActionName::RejectInput),
        CssRecoveryAction::DropDeclaration => Ok(CssRecoveryActionName::DropDeclaration),
        CssRecoveryAction::DropDescriptor => Ok(CssRecoveryActionName::DropDescriptor),
        CssRecoveryAction::DropQualifiedRule => Ok(CssRecoveryActionName::DropQualifiedRule),
        CssRecoveryAction::DropAtRule => Ok(CssRecoveryActionName::DropAtRule),
        CssRecoveryAction::DropKeyframeBlock => Ok(CssRecoveryActionName::DropKeyframeBlock),
        CssRecoveryAction::DropSelectorListItem => Ok(CssRecoveryActionName::DropSelectorListItem),
        CssRecoveryAction::ReplaceMediaQueryWithNever => {
            Ok(CssRecoveryActionName::ReplaceMediaQueryWithNever)
        }
        CssRecoveryAction::RetainWithImplicitClosure => {
            Ok(CssRecoveryActionName::RetainWithImplicitClosure)
        }
        CssRecoveryAction::IgnoreUnterminatedComment => {
            Ok(CssRecoveryActionName::IgnoreUnterminatedComment)
        }
        CssRecoveryAction::IgnoreLegacyToken => Ok(CssRecoveryActionName::IgnoreLegacyToken),
        CssRecoveryAction::StopAtNestingLimit => Ok(CssRecoveryActionName::StopAtNestingLimit),
        _ => Err("public parser returned an unrecognized CssRecoveryAction variant".into()),
    }
}

fn validate_replacement_bindings(
    existing: &RawOracle,
    replacement: &RawOracle,
) -> Result<(), String> {
    if existing.schema_version != replacement.schema_version
        || existing.provider_repository != replacement.provider_repository
        || existing.source_revision != replacement.source_revision
        || existing.source_tree != replacement.source_tree
        || existing.expectation_schema_version != replacement.expectation_schema_version
        || existing.generation_report_sha256 != replacement.generation_report_sha256
        || existing.expected_class_registry_sha256 != replacement.expected_class_registry_sha256
    {
        return Err("refusing to replace CSS oracle with drifted metadata bindings".into());
    }
    if existing.records.len() != replacement.records.len() {
        return Err("refusing to replace CSS oracle with a different record census".into());
    }
    for (existing, replacement) in existing.records.iter().zip(&replacement.records) {
        if existing.id != replacement.id
            || existing.path != replacement.path
            || existing.expectation_sha256 != replacement.expectation_sha256
            || existing.source != replacement.source
            || existing.context != replacement.context
            || existing.options != replacement.options
            || existing.input != replacement.input
        {
            return Err(format!(
                "refusing to replace CSS oracle record with drifted neutral binding: {}",
                replacement.id
            ));
        }
    }
    Ok(())
}

fn atomic_write_oracle(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let directory = path
        .parent()
        .ok_or_else(|| format!("CSS oracle has no parent directory: {}", path.display()))?;
    let temporary: PathBuf = directory.join(".oracle.json.surgeist-capture.tmp");
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| {
                format!(
                    "failed to create same-directory CSS oracle temporary file {}: {error}",
                    temporary.display()
                )
            })?;
        file.write_all(bytes)
            .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("failed to sync {}: {error}", temporary.display()))?;
        fs::rename(&temporary, path).map_err(|error| {
            format!(
                "failed to atomically rename {} to {}: {error}",
                temporary.display(),
                path.display()
            )
        })
    })();
    if write_result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn validate_probe(record: &RawOracleRecord, registry: RegistryEntry) -> Result<(), String> {
    match &record.probe {
        Probe::Active {
            entry_point,
            adapter,
            extractor,
            property_or_descriptor,
            options,
            payload,
        } => {
            if *entry_point != registry.entry_point() || *adapter != registry.adapter() {
                return Err(format!(
                    "CSS oracle active probe does not match registry for {}",
                    record.id
                ));
            }
            validate_extractor(extractor, &record.id)?;
            if !extractor_matches_registry(extractor, registry.extractor()) {
                return Err(format!(
                    "CSS oracle extractor does not match registry for {}",
                    record.id
                ));
            }
            if property_or_descriptor
                .0
                .as_deref()
                .is_some_and(str::is_empty)
            {
                return Err(format!(
                    "CSS oracle property_or_descriptor is empty for {}",
                    record.id
                ));
            }
            if options != &record.options {
                return Err(format!(
                    "CSS oracle active probe options mismatch for {}",
                    record.id
                ));
            }
            let registry_owner = registry
                .property_or_descriptor()
                .map(PropertyOrDescriptor::name);
            if property_or_descriptor.0.as_deref() != registry_owner {
                return Err(format!(
                    "CSS oracle property_or_descriptor does not match registry for {}",
                    record.id
                ));
            }
            let expected = registry
                .adapter()
                .wrap(&record.input, registry.property_or_descriptor())
                .map_err(|mismatch| {
                    format!(
                        "CSS adapter wrapper mismatch for {}: {mismatch:?}",
                        record.id
                    )
                })?;
            validate_payload(payload, &record.input, &record.id, &expected)?;
        }
        Probe::PanicFreedom {
            entry_point,
            adapter,
            payload,
        } => {
            if *entry_point != EntryPoint::StyleAttribute
                || *adapter != Adapter::CustomPropertyContainment
                || registry.unsupported_policy()
                    != Some(RegistryUnsupportedPolicy::PanicFreedomOnly)
            {
                return Err(format!(
                    "CSS oracle panic-freedom probe is illegal for {}",
                    record.id
                ));
            }
            if payload.prefix != "--surgeist-corpus-probe:" || payload.suffix != ";" {
                return Err(format!(
                    "CSS oracle panic-freedom wrapper mismatch for {}",
                    record.id
                ));
            }
            let expected = Adapter::CustomPropertyContainment
                .wrap(&record.input, None)
                .map_err(|mismatch| {
                    format!(
                        "CSS panic-freedom wrapper mismatch for {}: {mismatch:?}",
                        record.id
                    )
                })?;
            validate_payload(payload, &record.input, &record.id, &expected)?;
        }
    }
    Ok(())
}

fn validate_payload(
    payload: &Payload,
    input: &str,
    id: &str,
    expected: &adapters::CompleteInput,
) -> Result<(), String> {
    if payload.input_byte_length != input.len() {
        return Err(format!(
            "CSS oracle input byte length mismatch for {id}: oracle {}, input {}",
            payload.input_byte_length,
            input.len()
        ));
    }
    let span = payload.prefix.len()..payload.prefix.len() + input.len();
    let mut source =
        String::with_capacity(payload.prefix.len() + input.len() + payload.suffix.len());
    source.push_str(&payload.prefix);
    source.push_str(input);
    source.push_str(&payload.suffix);
    if source.get(span.clone()) != Some(input)
        || span != expected.payload_span()
        || source != expected.source()
    {
        return Err(format!("CSS oracle payload wrapper mismatch for {id}"));
    }
    Ok(())
}

fn extractor_matches_registry(extractor: &Extractor, expected: RegistryExtractor) -> bool {
    match (extractor, expected) {
        (Extractor::SheetRules, RegistryExtractor::SheetRules)
        | (Extractor::AtRule {}, RegistryExtractor::AtRule)
        | (Extractor::StyleBlock, RegistryExtractor::StyleBlock)
        | (Extractor::StyleDeclarations, RegistryExtractor::StyleDeclarations)
        | (Extractor::StyleSelector, RegistryExtractor::StyleSelector)
        | (Extractor::Selector, RegistryExtractor::Selector)
        | (Extractor::SelectorList, RegistryExtractor::SelectorList)
        | (Extractor::MediaQuery, RegistryExtractor::MediaQuery)
        | (Extractor::DeclarationList, RegistryExtractor::DeclarationList)
        | (Extractor::MediaQueries, RegistryExtractor::MediaQueries)
        | (Extractor::MediaChildren, RegistryExtractor::MediaChildren)
        | (Extractor::SupportsChildren, RegistryExtractor::SupportsChildren)
        | (Extractor::ContainerChildren, RegistryExtractor::ContainerChildren)
        | (Extractor::ScopeChildren, RegistryExtractor::ScopeChildren)
        | (Extractor::LayerBlockChildren, RegistryExtractor::LayerBlockChildren) => true,
        (
            Extractor::TopLevelRuleKind { rule_kind },
            RegistryExtractor::TopLevelRuleKind(expected),
        ) => rule_kind == expected.name(),
        (
            Extractor::FontFaceDescriptor { descriptor },
            RegistryExtractor::FontFaceDescriptor(expected),
        ) => descriptor == expected.name(),
        (
            Extractor::KnownDeclaration { index, property },
            RegistryExtractor::KnownDeclaration {
                index: expected_index,
                property: expected_property,
            },
        ) => *index == expected_index && property == expected_property.canonical_name(),
        _ => false,
    }
}

fn validate_extractor(extractor: &Extractor, id: &str) -> Result<(), String> {
    match extractor {
        Extractor::SheetRules
        | Extractor::AtRule {}
        | Extractor::StyleBlock
        | Extractor::StyleDeclarations
        | Extractor::StyleSelector
        | Extractor::Selector
        | Extractor::SelectorList
        | Extractor::MediaQuery
        | Extractor::DeclarationList
        | Extractor::MediaQueries
        | Extractor::MediaChildren
        | Extractor::SupportsChildren
        | Extractor::ContainerChildren
        | Extractor::ScopeChildren
        | Extractor::LayerBlockChildren => {}
        Extractor::TopLevelRuleKind { rule_kind } => {
            if !is_canonical_tag(rule_kind) {
                return Err(format!("invalid top-level rule kind for {id}"));
            }
        }
        Extractor::FontFaceDescriptor { descriptor } => {
            if !is_canonical_tag(descriptor) {
                return Err(format!("invalid font-face descriptor for {id}"));
            }
        }
        Extractor::KnownDeclaration { index, property } => {
            let _ = index;
            if property.is_empty() {
                return Err(format!("empty known declaration property for {id}"));
            }
        }
    }
    Ok(())
}

fn validate_outcome(record: &RawOracleRecord) -> Result<(), String> {
    let observation = record.observation.0.as_ref();
    let payload = record.probe.payload();
    let payload_span = payload.prefix.len()..payload.prefix.len() + payload.input_byte_length;
    match &record.outcome {
        Outcome::Clean => {
            let observation = observation.ok_or_else(|| {
                format!(
                    "clean CSS oracle outcome lacks observation for {}",
                    record.id
                )
            })?;
            validate_observation(observation, &record.id, payload_span.clone())?;
            if !observation.is_clean || !observation.diagnostics.is_empty() {
                return Err(format!(
                    "clean CSS oracle observation is not clean for {}",
                    record.id
                ));
            }
        }
        Outcome::Recovered | Outcome::StrictRejected => {
            let observation = observation.ok_or_else(|| {
                format!(
                    "diagnostic CSS oracle outcome lacks observation for {}",
                    record.id
                )
            })?;
            validate_observation(observation, &record.id, payload_span.clone())?;
            if observation.is_clean || observation.diagnostics.is_empty() {
                return Err(format!(
                    "diagnostic CSS oracle observation is clean for {}",
                    record.id
                ));
            }
        }
        Outcome::Unsupported { reason, policy } => {
            let _ = reason;
            match (policy, observation) {
                (UnsupportedPolicy::FullObservation, Some(observation)) => {
                    validate_observation(observation, &record.id, payload_span.clone())?;
                }
                (UnsupportedPolicy::PanicFreedomOnly, None) => {}
                (UnsupportedPolicy::FullObservation, None) => {
                    return Err(format!(
                        "full-observation unsupported outcome lacks observation for {}",
                        record.id
                    ));
                }
                (UnsupportedPolicy::PanicFreedomOnly, Some(_)) => {
                    return Err(format!(
                        "panic-freedom-only outcome carries observation for {}",
                        record.id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_observation(
    observation: &Observation,
    id: &str,
    payload_span: std::ops::Range<usize>,
) -> Result<(), String> {
    validate_extractor(&observation.extractor, id)?;
    let _ = observation.syntax_count;
    for diagnostic in &observation.diagnostics {
        let _ = (diagnostic.code, diagnostic.action);
        if diagnostic.multiplicity != 1 {
            return Err(format!("invalid diagnostic multiplicity for {id}"));
        }
        if diagnostic.span_start > diagnostic.span_end {
            return Err(format!("invalid diagnostic span for {id}"));
        }
        let relation_holds = payload_relation_holds(
            diagnostic.payload_relation,
            payload_span.clone(),
            diagnostic.byte_offset,
            diagnostic.span_start,
            diagnostic.span_end,
        );
        if !relation_holds {
            return Err(format!(
                "diagnostic payload relation does not hold for {id}"
            ));
        }
    }
    Ok(())
}

fn payload_relation_holds(
    relation: PayloadRelation,
    payload_span: std::ops::Range<usize>,
    byte_offset: usize,
    span_start: usize,
    span_end: usize,
) -> bool {
    match relation {
        PayloadRelation::Intersects => {
            payload_span.start < payload_span.end
                && payload_span.start <= byte_offset
                && byte_offset < payload_span.end
                && span_start < span_end
                && span_start < payload_span.end
                && span_end > payload_span.start
        }
        PayloadRelation::EndsAt => {
            byte_offset == payload_span.end
                && span_start == payload_span.end
                && span_end == payload_span.end
        }
        PayloadRelation::RecoveryEndsAt => {
            payload_span.start < payload_span.end
                && byte_offset == payload_span.end
                && span_start < payload_span.end
                && span_end == payload_span.end
        }
    }
}

pub(crate) fn expected_payload_relation_holds(
    relation: &str,
    payload_span: std::ops::Range<usize>,
    byte_offset: usize,
    span_start: usize,
    span_end: usize,
) -> bool {
    let relation = match relation {
        "intersects" => PayloadRelation::Intersects,
        "ends_at" => PayloadRelation::EndsAt,
        "recovery_ends_at" => PayloadRelation::RecoveryEndsAt,
        _ => return false,
    };
    payload_relation_holds(relation, payload_span, byte_offset, span_start, span_end)
}

fn is_canonical_tag(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawExpectedClasses {
    schema_version: u64,
    generation_report_sha256: String,
    records: Vec<RawExpectedClassRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawExpectedClassRecord {
    id: String,
    expectation_sha256: String,
    #[serde(rename = "class")]
    expected_class: ExpectedClass,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ExpectedClass {
    Clean {
        retained_syntax: ExpectedRetainedSyntax,
    },
    Recovered {
        retained_syntax: ExpectedRetainedSyntax,
        diagnostics: Vec<ExpectedDiagnostic>,
    },
    StrictRejected {
        retained_syntax: ExpectedRetainedSyntax,
        diagnostics: Vec<ExpectedDiagnostic>,
    },
    Unsupported {
        reason: UnsupportedReason,
        policy: ExpectedUnsupportedPolicy,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedRetainedSyntax {
    extractor: Extractor,
    predicate: SyntaxCount,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDiagnostic {
    code: CssErrorCodeName,
    action: CssRecoveryActionName,
    payload_relation: PayloadRelation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ExpectedUnsupportedPolicy {
    FullObservation {
        retained_syntax: ExpectedRetainedSyntax,
        is_clean: bool,
        diagnostics: Vec<ExpectedDiagnostic>,
    },
    PanicFreedomOnly,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawOracle {
    schema_version: u64,
    provider_repository: String,
    source_revision: String,
    source_tree: String,
    expectation_schema_version: u64,
    generation_report_sha256: String,
    expected_class_registry_sha256: String,
    records: Vec<RawOracleRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawOracleRecord {
    id: String,
    path: String,
    expectation_sha256: String,
    source: String,
    context: Context,
    input: String,
    options: Options,
    probe: Probe,
    outcome: Outcome,
    observation: NullableObservation,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Probe {
    Active {
        entry_point: EntryPoint,
        adapter: Adapter,
        extractor: Extractor,
        property_or_descriptor: NullableString,
        options: Options,
        payload: Payload,
    },
    PanicFreedom {
        entry_point: EntryPoint,
        adapter: Adapter,
        payload: Payload,
    },
}

impl Probe {
    const fn payload(&self) -> &Payload {
        match self {
            Self::Active { payload, .. } | Self::PanicFreedom { payload, .. } => payload,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Extractor {
    SheetRules,
    AtRule {},
    TopLevelRuleKind { rule_kind: String },
    StyleBlock,
    StyleDeclarations,
    StyleSelector,
    Selector,
    SelectorList,
    MediaQuery,
    DeclarationList,
    MediaQueries,
    MediaChildren,
    SupportsChildren,
    ContainerChildren,
    ScopeChildren,
    LayerBlockChildren,
    FontFaceDescriptor { descriptor: String },
    KnownDeclaration { index: usize, property: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    prefix: String,
    suffix: String,
    input_byte_length: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Outcome {
    Clean,
    Recovered,
    StrictRejected,
    Unsupported {
        reason: UnsupportedReason,
        policy: UnsupportedPolicy,
    },
}

impl Outcome {
    const fn index(&self) -> usize {
        match self {
            Self::Clean => 0,
            Self::Recovered => 1,
            Self::StrictRejected => 2,
            Self::Unsupported { .. } => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum UnsupportedReason {
    OutsideSelectedProfile,
    VendorOrHostSpecificSyntax,
    UpstreamParserOptionWithoutPublicCssEquivalent,
    GenericFragmentWithoutTruthfulSupportedPropertyOrDescriptor,
    AstConstructionWithoutPublicStylesheetMeaning,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum UnsupportedPolicy {
    FullObservation,
    PanicFreedomOnly,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
struct NullableString(Option<String>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
struct NullableObservation(Option<Observation>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    extractor: Extractor,
    syntax_count: usize,
    is_clean: bool,
    diagnostics: Vec<RawDiagnostic>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "relation", rename_all = "snake_case", deny_unknown_fields)]
enum SyntaxCount {
    Exact { value: usize },
    Nonempty,
    Empty,
}

impl SyntaxCount {
    const fn matches(&self, actual: usize) -> bool {
        match self {
            Self::Exact { value } => actual == *value,
            Self::Nonempty => actual > 0,
            Self::Empty => actual == 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawDiagnostic {
    code: CssErrorCodeName,
    action: CssRecoveryActionName,
    byte_offset: usize,
    span_start: usize,
    span_end: usize,
    multiplicity: usize,
    payload_relation: PayloadRelation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CssErrorCodeName {
    UnexpectedEnd,
    UnexpectedToken,
    InvalidEncodingDeclaration,
    InvalidAtRulePlacement,
    InvalidAtRulePrelude,
    InvalidAtRuleBody,
    UnknownAtRule,
    UnsupportedAtRule,
    InvalidQualifiedRule,
    InvalidSelector,
    InvalidMediaQuery,
    UnknownProperty,
    UnsupportedProperty,
    InvalidPropertyValue,
    InvalidDeclarationAnnotation,
    UnknownDescriptor,
    UnsupportedDescriptor,
    InvalidDescriptorValue,
    InvalidDescriptorCombination,
    InvalidColorSyntax,
    NestingLimit,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CssRecoveryActionName {
    RejectInput,
    DropDeclaration,
    DropDescriptor,
    DropQualifiedRule,
    DropAtRule,
    DropKeyframeBlock,
    DropSelectorListItem,
    ReplaceMediaQueryWithNever,
    RetainWithImplicitClosure,
    IgnoreUnterminatedComment,
    IgnoreLegacyToken,
    StopAtNestingLimit,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PayloadRelation {
    Intersects,
    EndsAt,
    /// A nonempty recovery unit overlaps the payload and ends at its error,
    /// exactly at the payload end. No suffix bytes belong to the recovery unit.
    RecoveryEndsAt,
}

fn set_mismatch(label: &str, expected: &BTreeSet<String>, actual: &BTreeSet<String>) -> String {
    let missing = expected.difference(actual).cloned().collect::<Vec<_>>();
    let extra = actual.difference(expected).cloned().collect::<Vec<_>>();
    format!("{label} mismatch: missing {missing:?}, extra {extra:?}")
}

fn is_canonical_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn validate_digest(label: &str, digest: &str) -> Result<(), String> {
    if digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(format!(
            "{label} digest is not canonical lowercase SHA-256: {digest}"
        ))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReport {
    manifest_digest: String,
    source_repository: String,
    source_revision: String,
    counts: ReportCounts,
    artifacts: Vec<ReportArtifact>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReportCounts {
    active: usize,
    expected_fail: usize,
    unsupported: usize,
    quarantined: usize,
    failed_to_generate: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportArtifact {
    provenance: ReportProvenance,
    output_path: String,
    output_digest: String,
    case_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportProvenance {
    source_path: String,
    source_digest: String,
    generator: String,
    schema_version: u64,
    domain_provenance: DomainProvenance,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainProvenance {
    #[serde(rename = "csstree-import")]
    csstree_import: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExpectation {
    schema_version: u64,
    generator: String,
    source: String,
    source_sha256: String,
    source_revision: String,
    import_provenance_sha256: String,
    cases: Vec<RawCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCase {
    id: String,
    context: Context,
    #[serde(default)]
    label: Option<String>,
    input: String,
    upstream_outcome: UpstreamOutcome,
    #[serde(default)]
    canonical_css: Option<String>,
    #[serde(default)]
    options: Option<Options>,
    status: Disposition,
    #[serde(default, deserialize_with = "deserialize_present_optional_string")]
    reason: Option<Option<String>>,
}

fn deserialize_present_optional_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct Options {
    #[serde(default, rename = "atrule", skip_serializing_if = "Option::is_none")]
    _atrule: Option<String>,
    #[serde(
        default,
        rename = "parseAtrulePrelude",
        skip_serializing_if = "Option::is_none"
    )]
    _parse_atrule_prelude: Option<bool>,
    #[serde(
        default,
        rename = "parseCustomProperty",
        skip_serializing_if = "Option::is_none"
    )]
    _parse_custom_property: Option<bool>,
    #[serde(
        default,
        rename = "parseRulePrelude",
        skip_serializing_if = "Option::is_none"
    )]
    _parse_rule_prelude: Option<bool>,
    #[serde(
        default,
        rename = "parseValue",
        skip_serializing_if = "Option::is_none"
    )]
    _parse_value: Option<bool>,
    #[serde(default, rename = "property", skip_serializing_if = "Option::is_none")]
    _property: Option<String>,
}

impl Options {
    fn profile(&self) -> Result<OptionsProfile, &'static str> {
        match (
            self._atrule.as_deref(),
            self._parse_atrule_prelude,
            self._parse_custom_property,
            self._parse_rule_prelude,
            self._parse_value,
            self._property.as_deref(),
        ) {
            (None, None, None, None, None, None) => Ok(OptionsProfile::Empty),
            (Some("media"), None, None, None, None, None) => Ok(OptionsProfile::AtruleMedia),
            (None, Some(false), None, None, None, None) => {
                Ok(OptionsProfile::ParseAtrulePreludeFalse)
            }
            (None, None, Some(true), None, None, None) => {
                Ok(OptionsProfile::ParseCustomPropertyTrue)
            }
            (None, None, None, Some(false), None, None) => {
                Ok(OptionsProfile::ParseRulePreludeFalse)
            }
            (None, None, None, None, Some(false), None) => Ok(OptionsProfile::ParseValueFalse),
            (None, None, Some(true), None, None, Some("--var")) => {
                Ok(OptionsProfile::CustomPropertyVar)
            }
            _ => Err("unsupported option combination; no default is inferred"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
enum Context {
    #[serde(rename = "atrule")]
    Atrule,
    #[serde(rename = "atrulePrelude")]
    AtrulePrelude,
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "declaration")]
    Declaration,
    #[serde(rename = "declarationList")]
    DeclarationList,
    #[serde(rename = "mediaQuery")]
    MediaQuery,
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "selector")]
    Selector,
    #[serde(rename = "selectorList")]
    SelectorList,
    #[serde(rename = "stylesheet")]
    Stylesheet,
    #[serde(rename = "value")]
    Value,
}

impl Context {
    const fn index(self) -> usize {
        match self {
            Self::Atrule => 0,
            Self::AtrulePrelude => 1,
            Self::Block => 2,
            Self::Declaration => 3,
            Self::DeclarationList => 4,
            Self::MediaQuery => 5,
            Self::Rule => 6,
            Self::Selector => 7,
            Self::SelectorList => 8,
            Self::Stylesheet => 9,
            Self::Value => 10,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Atrule => "atrule",
            Self::AtrulePrelude => "atrulePrelude",
            Self::Block => "block",
            Self::Declaration => "declaration",
            Self::DeclarationList => "declarationList",
            Self::MediaQuery => "mediaQuery",
            Self::Rule => "rule",
            Self::Selector => "selector",
            Self::SelectorList => "selectorList",
            Self::Stylesheet => "stylesheet",
            Self::Value => "value",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum UpstreamOutcome {
    Parsed,
    Rejected,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Disposition {
    Active,
    ExpectedFail,
    Unsupported,
    Quarantined,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // These expectations follow exact raw-fragment admission, source-coordinate
    // preservation, and Media Queries/Syntax EOF recovery contracts. Fixture identity
    // is retained only to select the owning corpus adapter; no oracle is consulted.
    fn raw_fragment_observation(path: &str, context: Context, source: &str) -> serde_json::Value {
        let registry = *REGISTRY
            .iter()
            .find(|entry| entry.fixture_path() == path)
            .unwrap();
        let case = ValidatedCase {
            id: format!("{path}#raw-contract"),
            expectation_path: path.into(),
            expectation_sha256: String::new(),
            source: path.into(),
            context,
            _label: None,
            input: source.into(),
            _upstream_outcome: UpstreamOutcome::Parsed,
            _canonical_css: None,
            options: serde_json::from_str("{}").unwrap(),
            expected_class: None,
        };
        let complete = registry
            .adapter()
            .wrap(source, registry.property_or_descriptor())
            .unwrap();
        let observation = observe_public_parser(&case, registry, &complete)
            .expect("raw observation must retain payload-local diagnostics");
        serde_json::to_value(observation).unwrap()
    }

    fn assert_raw_at_rule_rejects_trailing_input(source: &str) {
        // Syntax 3 single-rule admission requires EOF after the at-rule;
        // retaining siblings through stylesheet recovery is not this contract.
        let value =
            raw_fragment_observation("expectations/atrule/block.json", Context::Atrule, source);
        assert_eq!(value["syntax_count"], 0, "{source}");
        assert_eq!(value["is_clean"], false, "{source}");
        let diagnostics = value["diagnostics"].as_array().unwrap();
        assert_eq!(diagnostics.len(), 1, "{source}");
        assert_eq!(diagnostics[0]["action"], "reject_input", "{source}");
        assert_eq!(diagnostics[0]["span_start"], 0, "{source}");
        assert_eq!(diagnostics[0]["span_end"], source.len(), "{source}");
    }

    #[test]
    fn raw_at_rule_observation_rejects_trailing_semicolon() {
        assert_raw_at_rule_rejects_trailing_input("@media{} ;");
    }

    #[test]
    fn raw_at_rule_observation_rejects_trailing_rule() {
        assert_raw_at_rule_rejects_trailing_input("@media{} a{}");
    }

    #[test]
    fn raw_at_rule_observation_retains_clean_empty_rule() {
        let value = raw_fragment_observation(
            "expectations/atrule/block.json",
            Context::Atrule,
            "@media{}",
        );
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], true);
        assert!(value["diagnostics"].as_array().unwrap().is_empty());
    }

    #[test]
    fn at_rule_extractor_schema_round_trips_and_rejects_extra_fields() {
        let value = serde_json::json!({ "kind": "at_rule" });
        let extractor: Extractor = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(extractor, Extractor::AtRule {});
        assert_eq!(serde_json::to_value(&extractor).unwrap(), value);
        assert!(extractor_matches_registry(
            &extractor,
            RegistryExtractor::AtRule
        ));
        assert!(!extractor_matches_registry(
            &extractor,
            RegistryExtractor::SheetRules
        ));
        assert!(
            serde_json::from_value::<Extractor>(serde_json::json!({
                "kind": "at_rule", "rule_kind": "media"
            }))
            .is_err()
        );
    }

    #[test]
    fn raw_at_rule_observation_preserves_inner_recovery_and_eof() {
        for (source, action) in [
            ("@media{a{unknown:x}}", "drop_declaration"),
            ("@media{@unknown x;}", "drop_at_rule"),
            ("@media{", "retain_with_implicit_closure"),
        ] {
            let value =
                raw_fragment_observation("expectations/atrule/block.json", Context::Atrule, source);
            assert_eq!(value["extractor"]["kind"], "at_rule");
            assert_eq!(value["syntax_count"], 1, "{source}");
            assert_eq!(value["is_clean"], false, "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert_eq!(diagnostics[0]["action"], action, "{source}");
            if source == "@media{" {
                assert_eq!(diagnostics[0]["byte_offset"], source.len());
                assert_eq!(diagnostics[0]["span_end"], source.len());
            }
        }
    }

    #[test]
    fn raw_at_rule_observation_rejects_outer_grammar_without_retained_closure() {
        for source in [
            "@unknown x;",
            "@unknown {",
            "@charset \"UTF-8\";",
            "@media{} @media{}",
            "@media{a{unknown:x}} ;",
        ] {
            assert_raw_at_rule_rejects_trailing_input(source);
        }
    }

    #[test]
    fn raw_at_rule_observation_retains_isolated_import_namespace_and_empty_font_face() {
        for source in [
            "@import \"a.css\";",
            "@namespace ns \"urn:test\";",
            "@font-face{}",
            "@media{ns|a{}}",
        ] {
            let value =
                raw_fragment_observation("expectations/atrule/block.json", Context::Atrule, source);
            assert_eq!(value["syntax_count"], 1, "{source}");
            assert_eq!(value["is_clean"], true, "{source}");
            assert_eq!(value["diagnostics"], serde_json::json!([]), "{source}");
        }
    }

    #[test]
    fn raw_at_rule_observation_preserves_original_unicode_crlf_coordinates() {
        let source = "/*😀*/\r\n/*é*/@media{a{unknown:x}}";
        let value =
            raw_fragment_observation("expectations/atrule/block.json", Context::Atrule, source);
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(
            value["diagnostics"][0]["byte_offset"],
            source.find("unknown").unwrap()
        );
        assert_eq!(
            value["diagnostics"][0]["span_start"],
            source.find("unknown").unwrap()
        );
        let report = parse_rule(source, &corpus_namespace_context());
        let Some(surgeist_css::CssRule::Media(media)) = report.syntax() else {
            panic!("expected retained media rule");
        };
        assert_eq!(
            media.position().byte_offset().value(),
            source.find('@').unwrap()
        );
        assert_eq!(media.position().line().value(), 1);
        assert_eq!(media.position().column().value(), 5);
        let [surgeist_css::CssRule::Style(style)] = media.rules() else {
            panic!("expected retained style child");
        };
        assert!(style.declarations().is_empty());
    }

    #[test]
    fn raw_style_block_observation_rejects_extra_outer_input() {
        // A real-brace style-block fragment consumes exactly one whole block.
        // A recovered stylesheet containing that block is a different contract.
        for source in ["{color:red};", "{color:red} a{color:blue}"] {
            let value =
                raw_fragment_observation("expectations/block/Block.json", Context::Block, source);
            assert_eq!(value["syntax_count"], 0, "{source}");
            assert_eq!(value["is_clean"], false, "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert_eq!(diagnostics[0]["action"], "reject_input", "{source}");
            assert_eq!(diagnostics[0]["span_start"], 0, "{source}");
            assert_eq!(diagnostics[0]["span_end"], source.len(), "{source}");
        }
    }

    #[test]
    fn raw_style_block_observation_distinguishes_empty_retention_and_eof() {
        for (source, action) in [
            ("{}", None),
            ("{color:red}", None),
            ("{unknown:x}", Some("drop_declaration")),
            ("{color:red", Some("retain_with_implicit_closure")),
        ] {
            let value =
                raw_fragment_observation("expectations/block/Block.json", Context::Block, source);
            // This context observes block presence, not declaration cardinality.
            assert_eq!(value["syntax_count"], 1, "{source}");
            assert_eq!(value["is_clean"], action.is_none(), "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            if let Some(action) = action {
                assert_eq!(diagnostics.len(), 1, "{source}");
                assert_eq!(diagnostics[0]["action"], action, "{source}");
                if action == "retain_with_implicit_closure" {
                    assert_eq!(diagnostics[0]["byte_offset"], source.len());
                    assert_eq!(diagnostics[0]["span_start"], source.len());
                    assert_eq!(diagnostics[0]["span_end"], source.len());
                }
            } else {
                assert!(diagnostics.is_empty());
            }
        }
    }

    #[test]
    fn raw_rule_observation_rejects_extra_outer_input() {
        // Pinned Syntax 3 section 5.3.5 requires EOF after one rule, even when
        // stylesheet recovery could retain exactly one surviving sibling.
        for source in [
            "a{} b{}",
            "a{} @unknown x;",
            "@unknown x; a{}",
            "a{} ;",
            "a{unknown:x} b{}",
        ] {
            let value =
                raw_fragment_observation("expectations/rule/Rule.json", Context::Rule, source);
            assert_eq!(value["syntax_count"], 0, "{source}");
            assert_eq!(value["is_clean"], false, "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            assert_eq!(diagnostics.len(), 1, "{source}");
            assert_eq!(diagnostics[0]["action"], "reject_input", "{source}");
            assert_eq!(diagnostics[0]["span_start"], 0, "{source}");
            assert_eq!(diagnostics[0]["span_end"], source.len(), "{source}");
        }
    }

    #[test]
    fn raw_rule_observation_retains_inner_recovery_and_actual_eof() {
        for (source, action) in [
            ("a{color:red}", None),
            ("a{unknown:x;color:red}", Some("drop_declaration")),
            ("a{color:red", Some("retain_with_implicit_closure")),
        ] {
            let value =
                raw_fragment_observation("expectations/rule/Rule.json", Context::Rule, source);
            assert_eq!(value["syntax_count"], 1, "{source}");
            assert_eq!(value["is_clean"], action.is_none(), "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            if let Some(action) = action {
                assert_eq!(diagnostics.len(), 1, "{source}");
                assert_eq!(diagnostics[0]["action"], action, "{source}");
            } else {
                assert!(diagnostics.is_empty());
            }
        }
    }

    #[test]
    fn raw_property_value_observation_rejects_authored_delimiters() {
        // Property values exclude declaration separators and annotations;
        // separate property identity and importance cannot excuse these tokens.
        let path = "expectations/value/Dimension.json";
        for source in ["10px;", "var(--width)!important"] {
            let value = raw_fragment_observation(path, Context::Value, source);
            assert_eq!(value["syntax_count"], 0, "{source}");
            assert_eq!(value["is_clean"], false, "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            assert!(!diagnostics.is_empty(), "{source}");
            for diagnostic in diagnostics {
                assert_eq!(diagnostic["action"], "reject_input", "{source}");
                assert_eq!(diagnostic["span_start"], 0, "{source}");
                assert_eq!(diagnostic["span_end"], source.len(), "{source}");
            }
        }
        let value = raw_fragment_observation(path, Context::Value, "10px");
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], true);
    }

    #[test]
    fn raw_property_value_observation_retains_valid_eof_closure() {
        // Syntax closes a retained function at the caller's actual EOF. No
        // synthetic declaration terminator may become part of its arguments.
        let source = "var(--width";
        let value = raw_fragment_observation(
            "expectations/value/function/var.json",
            Context::Value,
            source,
        );
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], false);
        let diagnostics = value["diagnostics"].as_array().unwrap();
        assert!(!diagnostics.is_empty());
        for diagnostic in diagnostics {
            assert_eq!(diagnostic["action"], "retain_with_implicit_closure");
            assert_eq!(diagnostic["span_end"], source.len());
        }
    }

    #[test]
    fn raw_declaration_observation_rejects_original_invalid_values_in_payload() {
        // Syntax 3's declaration-value grammar rejects unmatched nested closers;
        // Filter Effects 1 does not define the proprietary alpha() function.
        // Neither discarded declaration owns an implicit EOF closure.
        for (path, source) in [
            (
                "expectations/declaration/custom-property.json",
                "--var: ([)]",
            ),
            (
                "expectations/declaration/filter.json",
                "filter:alpha(opacity",
            ),
        ] {
            let value = raw_fragment_observation(path, Context::Declaration, source);
            assert_eq!(value["syntax_count"], 0, "{source}");
            assert_eq!(value["is_clean"], false, "{source}");
            let diagnostics = value["diagnostics"].as_array().unwrap();
            assert!(!diagnostics.is_empty(), "{source}");
            for diagnostic in diagnostics {
                assert_eq!(diagnostic["action"], "reject_input", "{source}");
                assert_eq!(diagnostic["span_start"], 0, "{source}");
                assert_eq!(diagnostic["span_end"], source.len(), "{source}");
            }
        }
    }

    #[test]
    fn raw_declaration_observation_requires_one_complete_declaration() {
        // Syntax 3's parse-a-declaration entry consumes to EOF. A top-level
        // semicolon belongs to declaration-list syntax, not declaration-value.
        let path = "expectations/declaration/custom-property.json";
        for source in ["--x:a;--y:b", "--x:a;", ";--x:a"] {
            let value = raw_fragment_observation(path, Context::Declaration, source);
            assert_eq!(value["syntax_count"], 0, "{source}");
            assert_eq!(value["is_clean"], false, "{source}");
            assert!(
                value["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|d| d["action"] == "reject_input"),
                "{source}"
            );
        }
        let value = raw_fragment_observation(path, Context::Declaration, "--x:{a:b;c:d}");
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], true);
        assert_eq!(value["diagnostics"], serde_json::json!([]));
    }

    #[test]
    fn raw_font_face_value_observation_keeps_incomplete_range_eof_in_payload() {
        // Original UnicodeRange fixtures: Syntax 3 section 7.1 requires a
        // continuation after u and +. Missing input belongs to the raw EOF,
        // not an injected descriptor terminator or surrounding font-face rule.
        let path = "expectations/value/UnicodeRange.json";
        for source in ["U+", "u"] {
            let value = raw_fragment_observation(path, Context::Value, source);
            assert_eq!(value["syntax_count"], 0);
            assert_eq!(value["is_clean"], false);
            assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
            let error = &value["diagnostics"][0];
            assert_eq!(error["code"], "invalid_descriptor_value");
            assert_eq!(error["action"], "reject_input");
            assert_eq!(error["byte_offset"], source.len());
            assert_eq!(error["span_start"], 0);
            assert_eq!(error["span_end"], source.len());
            assert_eq!(error["payload_relation"], "recovery_ends_at");
        }
        let value = raw_fragment_observation(path, Context::Value, "u+?");
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], true);
        assert_eq!(value["diagnostics"], serde_json::json!([]));
    }

    #[test]
    fn raw_selector_observation_rejects_a_second_root_at_the_authored_comma() {
        let value = raw_fragment_observation(
            "expectations/selector/Selector.json",
            Context::Selector,
            ".a,.b",
        );
        assert_eq!(value["syntax_count"], 0);
        assert_eq!(value["is_clean"], false);
        assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
        let error = &value["diagnostics"][0];
        assert_eq!(error["code"], "invalid_selector");
        assert_eq!(error["action"], "reject_input");
        assert_eq!(error["byte_offset"], 2);
        assert_eq!(error["span_start"], 0);
        assert_eq!(error["span_end"], 5);
        assert_eq!(error["payload_relation"], "intersects");
    }

    #[test]
    fn raw_selector_list_observation_counts_members_and_preserves_empty_tail_error() {
        let path = "expectations/selectorList/Selector.json";
        let value = raw_fragment_observation(path, Context::SelectorList, ".a,.b");
        assert_eq!(value["syntax_count"], 2);
        assert_eq!(value["is_clean"], true);
        assert_eq!(value["diagnostics"], serde_json::json!([]));
        let value = raw_fragment_observation(path, Context::SelectorList, ".a,");
        assert_eq!(value["syntax_count"], 0);
        assert_eq!(value["is_clean"], false);
        assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
        let error = &value["diagnostics"][0];
        assert_eq!(error["code"], "invalid_selector");
        assert_eq!(error["action"], "reject_input");
        assert_eq!(error["byte_offset"], 3);
        assert_eq!(error["span_start"], 0);
        assert_eq!(error["span_end"], 3);
        assert_eq!(error["payload_relation"], "recovery_ends_at");
    }

    #[test]
    fn raw_media_observation_retains_actual_eof_without_a_synthetic_rule_block() {
        let value = raw_fragment_observation(
            "expectations/mediaQuery/MediaQuery.json",
            Context::MediaQuery,
            "(fo",
        );
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], false);
        assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
        let closure = &value["diagnostics"][0];
        assert_eq!(closure["code"], "unexpected_end");
        assert_eq!(closure["action"], "retain_with_implicit_closure");
        assert_eq!(closure["byte_offset"], 3);
        assert_eq!(closure["span_start"], 3);
        assert_eq!(closure["span_end"], 3);
        assert_eq!(closure["payload_relation"], "ends_at");
    }

    #[test]
    fn raw_media_observation_preserves_unterminated_comment_diagnostic() {
        // Pinned Syntax 3 section 4.3.2 consumes the comment and reports EOF.
        let value = raw_fragment_observation(
            "expectations/mediaQuery/MediaQuery.json",
            Context::MediaQuery,
            "screen/*",
        );
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], false);
        assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
        let diagnostic = &value["diagnostics"][0];
        assert_eq!(diagnostic["code"], "unexpected_end");
        assert_eq!(diagnostic["action"], "ignore_unterminated_comment");
        assert_eq!(diagnostic["byte_offset"], 8);
        assert_eq!(diagnostic["span_start"], 6);
        assert_eq!(diagnostic["span_end"], 8);
        assert_eq!(diagnostic["payload_relation"], "recovery_ends_at");
    }

    #[test]
    fn raw_selector_observation_uses_a_named_namespace_without_injected_source() {
        let value = raw_fragment_observation(
            "expectations/selector/TypeSelector.json",
            Context::Selector,
            "ns|λ",
        );
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], true);
        assert_eq!(value["diagnostics"], serde_json::json!([]));
    }

    #[test]
    fn raw_single_media_observation_replaces_comma_separated_input_as_one_query() {
        let value = raw_fragment_observation(
            "expectations/mediaQuery/MediaQuery.json",
            Context::MediaQuery,
            "screen,print",
        );
        assert_eq!(value["syntax_count"], 1);
        assert_eq!(value["is_clean"], false);
        assert_eq!(value["diagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(value["diagnostics"][0]["code"], "invalid_media_query");
        assert_eq!(
            value["diagnostics"][0]["action"],
            "replace_media_query_with_never"
        );
    }

    type OracleMutation = (&'static str, OracleContractFailureKind, fn(&mut Value));

    #[test]
    fn payload_observation_retains_nonempty_recovery_ending_at_payload_end() {
        let complete = adapters::Adapter::Stylesheet
            .wrap("@media;", None)
            .expect("whole-sheet input needs no wrapper");
        let report = parse_sheet(complete.source());
        assert!(!report.is_clean());
        assert!(report.syntax().rules().is_empty());
        assert_eq!(report.diagnostics().len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.error().position().byte_offset().value(), 7);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), 7);

        // CssRecoveryDiagnostic explicitly permits an error at the exclusive
        // end of a nonempty recovery span. The whole source is the payload here.
        let observation = observation_from_report(
            Extractor::SheetRules,
            0,
            report.is_clean(),
            report.diagnostics(),
            &complete,
        )
        .expect("valid recovery wholly within the payload must remain observable");
        let observation = serde_json::to_value(observation).expect("serialized observation");
        assert_eq!(
            observation["diagnostics"][0]["payload_relation"],
            "recovery_ends_at"
        );
        assert_eq!(observation["diagnostics"][0]["byte_offset"], 7);
        assert_eq!(observation["diagnostics"][0]["span_start"], 0);
        assert_eq!(observation["diagnostics"][0]["span_end"], 7);
    }

    #[test]
    fn payload_observation_rejects_recovery_after_consuming_wrapper_suffix() {
        let complete = adapters::CompleteInput::from_parts(
            "@namespace ns \"surgeist-corpus-probe\";",
            ":has(.a{)",
            "{--surgeist-corpus-probe:0;}",
        )
        .expect("deliberately wrapped negative control preserves its payload");
        let report = parse_sheet(complete.source());
        assert!(report.diagnostics().iter().any(|diagnostic| {
            diagnostic.error().position().byte_offset().value() > complete.payload_span().end
        }));
        assert!(
            observation_from_report(
                Extractor::StyleSelector,
                0,
                report.is_clean(),
                report.diagnostics(),
                &complete,
            )
            .is_err(),
            "a wrapper diagnostic must not be relabeled as payload-local recovery"
        );
    }

    fn committed_artifacts() -> ArtifactSet {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
        read_artifact_set(&root).expect("committed artifacts should be readable")
    }

    fn oracle_record_test_artifacts(expected_classes: &[u8]) -> ArtifactSet {
        let mut artifacts = committed_artifacts();
        artifacts.expected_classes = expected_classes.to_vec();
        let digest = sha256_hex(expected_classes);
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["expected_class_registry_sha256"] = Value::from(digest);
        });
        artifacts
    }

    fn oracle_record_test_failures(artifacts: &ArtifactSet) -> Vec<OracleContractFailure> {
        let inventory = validate_neutral_artifact_set(NeutralArtifactSet {
            report: artifacts.report.clone(),
            expected_classes: artifacts.expected_classes.clone(),
            expectations: artifacts.expectations.clone(),
            sources: artifacts.sources.clone(),
        })
        .expect("record tests require a valid neutral artifact set");
        if let Err(error) = validate_oracle(
            &artifacts.oracle,
            &inventory.report_digest,
            &inventory.expected_classes_digest,
            &inventory.cases,
        ) {
            assert!(
                error.starts_with("CSS oracle contract failures:\n"),
                "record test setup must pass the document gates: {error}"
            );
        }
        let oracle = load_csstree_oracle_schema(&artifacts.oracle)
            .expect("record tests require a canonical closed-schema oracle");
        collect_oracle_contract_failures(&oracle, &inventory.cases)
    }

    fn assert_oracle_record_failure_absent(
        artifacts: &ArtifactSet,
        kind: OracleContractFailureKind,
    ) {
        // Other record semantics in the copied oracle remain unresolved. The
        // mutation's own failure must be absent before it is introduced.
        let failures = oracle_record_test_failures(artifacts);
        assert!(
            failures.iter().all(|failure| failure.kind() != kind),
            "record test setup already contains {kind:?}"
        );
    }

    fn expected_classes_with_one_media_rule(bytes: &[u8]) -> Vec<u8> {
        const ID: &str = "atrule/atrule/media.json#/single media type";
        let neutral: Value = serde_json::from_str(include_str!(
            "../corpus/csstree/expectations/atrule/atrule/media.json"
        ))
        .expect("pinned neutral media fixture");
        let case = neutral["cases"]
            .as_array()
            .expect("neutral media cases")
            .iter()
            .find(|case| case["id"] == ID)
            .expect("single media rule fixture");
        assert_eq!(case["input"], "@media screen{}");

        // Conditional 3's @media grammar retains this single authored rule,
        // including its empty rule list. The expected count is independent of
        // the copied oracle and the Supports/Scope extractor migration.
        let report = parse_sheet("@media screen{}");
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert!(matches!(
            report.syntax().rules(),
            [surgeist_css::CssRule::Media(_)]
        ));

        let mut expected: RawExpectedClasses =
            serde_json::from_slice(bytes).expect("typed expected-class registry");
        let record = expected
            .records
            .iter_mut()
            .find(|record| record.id == ID)
            .expect("single media rule expected class");
        let ExpectedClass::Clean { retained_syntax } = &mut record.expected_class else {
            panic!("the single media rule must have a clean expected class");
        };
        assert_eq!(
            retained_syntax.extractor,
            Extractor::TopLevelRuleKind {
                rule_kind: "media".into(),
            }
        );
        assert!(matches!(retained_syntax.predicate, SyntaxCount::Nonempty));
        retained_syntax.predicate = SyntaxCount::Exact { value: 1 };

        let variant = format!(
            "{}\n",
            serde_json::to_string_pretty(&expected).expect("canonical expected-class variant")
        )
        .into_bytes();
        assert_ne!(variant.as_slice(), bytes);
        validate_csstree_expected_classes_contract(&variant)
            .expect("the independently specified exact count retains the registry contract");
        variant
    }

    #[test]
    fn oracle_record_fixture_binds_supplied_expected_bytes_without_other_changes() {
        let original = committed_artifacts();
        let expected = expected_classes_with_one_media_rule(&original.expected_classes);
        let expected_before = expected.clone();
        let artifacts = oracle_record_test_artifacts(&expected);
        assert_eq!(
            expected, expected_before,
            "caller bytes must remain unchanged"
        );
        assert_eq!(artifacts.expected_classes, expected);
        assert_eq!(artifacts.report, original.report);
        assert_eq!(artifacts.expectations, original.expectations);
        assert_eq!(artifacts.sources, original.sources);
        load_csstree_oracle_schema(&artifacts.oracle)
            .expect("the copied oracle must retain canonical closed-schema bytes");

        let mut original_oracle: Value =
            serde_json::from_slice(&original.oracle).expect("original oracle JSON");
        let original_digest = original_oracle
            .as_object_mut()
            .expect("original oracle object")
            .remove("expected_class_registry_sha256")
            .expect("original expected-class digest");
        let mut copied_oracle: Value =
            serde_json::from_slice(&artifacts.oracle).expect("copied oracle JSON");
        let copied_digest = copied_oracle
            .as_object_mut()
            .expect("copied oracle object")
            .remove("expected_class_registry_sha256")
            .expect("copied expected-class digest");
        assert_eq!(copied_oracle, original_oracle, "all other oracle fields");

        let unchanged = committed_artifacts();
        assert_eq!(unchanged.oracle, original.oracle, "persisted oracle bytes");
        assert_eq!(
            unchanged.expected_classes, original.expected_classes,
            "persisted expected-class bytes"
        );
        assert_eq!(unchanged.report, original.report, "persisted report bytes");
        assert_eq!(
            unchanged.expectations, original.expectations,
            "persisted neutral expectation bytes"
        );
        assert_eq!(
            unchanged.sources, original.sources,
            "persisted source bytes"
        );
        let expected_digest = Value::from(sha256_hex(&expected));
        assert_ne!(
            original_digest, expected_digest,
            "independent digest stimulus"
        );
        assert_eq!(
            copied_digest, expected_digest,
            "the document fixture must bind the supplied expected-class bytes"
        );
    }

    #[test]
    fn oracle_record_fixture_stale_header_is_rejected_before_record_validation() {
        let original = committed_artifacts();
        let expected = expected_classes_with_one_media_rule(&original.expected_classes);
        let expected_digest = sha256_hex(&expected);
        let mut artifacts = oracle_record_test_artifacts(&expected);

        // Establish this guard's header precondition independently of the
        // fixture helper. The copied record semantics remain unresolved.
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["expected_class_registry_sha256"] = Value::from(expected_digest.clone());
        });
        let inventory = validate_neutral_artifact_set(NeutralArtifactSet {
            report: artifacts.report.clone(),
            expected_classes: artifacts.expected_classes.clone(),
            expectations: artifacts.expectations.clone(),
            sources: artifacts.sources.clone(),
        })
        .expect("the supplied neutral artifact set must validate");
        if let Err(error) = validate_oracle(
            &artifacts.oracle,
            &inventory.report_digest,
            &inventory.expected_classes_digest,
            &inventory.cases,
        ) {
            assert!(
                error.starts_with("CSS oracle contract failures:\n"),
                "a correctly bound header must reach record validation: {error}"
            );
        }

        let stale_digest = "0".repeat(64);
        assert_ne!(stale_digest, expected_digest);
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["expected_class_registry_sha256"] = Value::from(stale_digest.clone());
        });
        let error = validate_artifact_set(artifacts)
            .expect_err("a stale fixture header must fail the unchanged document guard");
        assert_eq!(
            error,
            format!(
                "CSS oracle expected-class registry digest mismatch: oracle {stale_digest}, actual {expected_digest}"
            )
        );
    }

    fn mutate_first_expectation(artifacts: &mut ArtifactSet, edit: impl FnOnce(&mut Value)) {
        let path = artifacts
            .expectations
            .keys()
            .next()
            .expect("expectation inventory")
            .clone();
        let mut document: Value =
            serde_json::from_slice(&artifacts.expectations[&path]).expect("expectation JSON");
        edit(&mut document);
        let bytes = serde_json::to_vec(&document).expect("serialize mutated expectation");
        let digest = sha256_hex(&bytes);
        artifacts.expectations.insert(path.clone(), bytes);
        mutate_report(artifacts, |report| {
            let artifact = report["artifacts"]
                .as_array_mut()
                .expect("artifact array")
                .iter_mut()
                .find(|artifact| artifact["output_path"] == path)
                .expect("report artifact");
            artifact["output_digest"] = Value::from(digest);
        });
    }

    fn mutate_report(artifacts: &mut ArtifactSet, edit: impl FnOnce(&mut Value)) {
        let mut report: Value =
            serde_json::from_slice(&artifacts.report).expect("report JSON should deserialize");
        edit(&mut report);
        artifacts.report = serde_json::to_vec(&report).expect("serialize mutated report");
    }

    fn mutate_oracle(artifacts: &mut ArtifactSet, edit: impl FnOnce(&mut Value)) {
        let mut oracle: Value =
            serde_json::from_slice(&artifacts.oracle).expect("oracle JSON should deserialize");
        edit(&mut oracle);
        let mut bytes =
            serde_json::to_string_pretty(&oracle).expect("serialize mutated oracle into JSON");
        bytes.push('\n');
        artifacts.oracle = bytes.into_bytes();
    }

    fn mutate_canonical_oracle(artifacts: &mut ArtifactSet, edit: impl FnOnce(&mut Value)) {
        let mut oracle: Value =
            serde_json::from_slice(&artifacts.oracle).expect("oracle JSON should deserialize");
        edit(&mut oracle);
        let oracle: RawOracle =
            serde_json::from_value(oracle).expect("mutated oracle should retain its schema");
        let mut bytes =
            serde_json::to_string_pretty(&oracle).expect("serialize canonical mutated oracle");
        bytes.push('\n');
        artifacts.oracle = bytes.into_bytes();
    }

    fn assert_rejected(artifacts: ArtifactSet, expected: &str) {
        let error = validate_artifact_set(artifacts).expect_err("artifact set should be rejected");
        assert!(
            error.contains(expected),
            "expected error containing {expected:?}, got {error:?}"
        );
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn oracle_contract_reports_typed_document_failures() {
        let failures = validate_csstree_oracle_contract(b"{not-json")
            .expect_err("malformed JSON should produce one typed failure");
        assert_eq!(failures.len(), 1);
        let failure = &failures[0];
        assert_eq!(
            failure.kind(),
            OracleContractFailureKind::DocumentContractMismatch
        );
        assert_eq!(failure.case_id(), "<oracle>");
        assert_eq!(failure.path(), "<oracle>");
        assert_eq!(failure.context(), "<oracle>");

        let canonical =
            canonicalize_csstree_oracle_schema(include_bytes!("../csstree/oracle.json"))
                .expect("committed oracle schema should canonicalize");
        assert_eq!(canonical, include_bytes!("../csstree/oracle.json"));
    }

    #[test]
    fn loader_rejects_malformed_schema() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["schema_version"] = Value::from(2);
        });
        assert_rejected(artifacts, "expectation schema version");
    }

    #[test]
    fn loader_rejects_malformed_report_schema() {
        let mut artifacts = committed_artifacts();
        mutate_report(&mut artifacts, |report| {
            report["artifacts"][0]["provenance"]["schema_version"] = Value::from(2);
        });
        assert_rejected(artifacts, "report schema version");
    }

    #[test]
    fn loader_rejects_duplicate_ids() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            let first_id = document["cases"][0]["id"].clone();
            document["cases"][1]["id"] = first_id;
        });
        assert_rejected(artifacts, "duplicate case ID");
    }

    #[test]
    fn loader_rejects_unknown_context() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["cases"][0]["context"] = Value::from("unknown");
        });
        assert_rejected(artifacts, "failed to deserialize expectation");
    }

    #[test]
    fn loader_rejects_unknown_outcome() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["cases"][0]["upstream_outcome"] = Value::from("unknown");
        });
        assert_rejected(artifacts, "failed to deserialize expectation");
    }

    #[test]
    fn loader_rejects_non_active_disposition() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["cases"][0]["status"] = Value::from("expected-fail");
            document["cases"][0]["reason"] = Value::from("known mismatch");
        });
        assert_rejected(artifacts, "non-active disposition");
    }

    #[test]
    fn loader_rejects_reason_on_active_case() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["cases"][0]["reason"] = Value::from("not neutral");
        });
        assert_rejected(artifacts, "carries a disposition reason");
    }

    #[test]
    fn loader_rejects_null_reason_on_active_case() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["cases"][0]["reason"] = Value::Null;
        });
        assert_rejected(artifacts, "carries a disposition reason");
    }

    #[test]
    fn loader_rejects_unknown_case_option() {
        let mut artifacts = committed_artifacts();
        mutate_first_expectation(&mut artifacts, |document| {
            document["cases"][0]["options"]["unknown"] = Value::Bool(true);
        });
        assert_rejected(artifacts, "failed to deserialize expectation");
    }

    #[test]
    fn loader_rejects_stale_report_output_digest() {
        let mut artifacts = committed_artifacts();
        mutate_report(&mut artifacts, |report| {
            report["artifacts"][0]["output_digest"] = Value::from("0".repeat(64));
        });
        assert_rejected(artifacts, "output digest mismatch");
    }

    #[test]
    fn loader_rejects_stale_report_manifest_digest() {
        let mut artifacts = committed_artifacts();
        mutate_report(&mut artifacts, |report| {
            report["manifest_digest"] = Value::from("0".repeat(64));
        });
        assert_rejected(artifacts, "report manifest digest mismatch");
    }

    #[test]
    fn loader_rejects_stale_source_digest() {
        let mut artifacts = committed_artifacts();
        mutate_report(&mut artifacts, |report| {
            report["artifacts"][0]["provenance"]["source_digest"] = Value::from("0".repeat(64));
        });
        assert_rejected(artifacts, "expectation/report source digest mismatch");
    }

    #[test]
    fn loader_rejects_missing_expectation_artifact() {
        let mut artifacts = committed_artifacts();
        let path = artifacts
            .expectations
            .keys()
            .next()
            .expect("expectation inventory")
            .clone();
        artifacts.expectations.remove(&path);
        assert_rejected(artifacts, "expectation artifact inventory mismatch");
    }

    #[test]
    fn loader_rejects_extra_expectation_artifact() {
        let mut artifacts = committed_artifacts();
        artifacts
            .expectations
            .insert("expectations/extra.json".into(), b"{}".to_vec());
        assert_rejected(artifacts, "expectation artifact inventory mismatch");
    }

    #[test]
    fn loader_rejects_noncanonical_report_order() {
        let mut artifacts = committed_artifacts();
        mutate_report(&mut artifacts, |report| {
            report["artifacts"]
                .as_array_mut()
                .expect("artifact array")
                .swap(0, 1);
        });
        assert_rejected(artifacts, "report artifacts are not in canonical order");
    }

    #[test]
    fn oracle_loader_rejects_malformed_schema() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["unknown"] = Value::Bool(true);
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_duplicate_ids() {
        let mut artifacts =
            oracle_record_test_artifacts(include_bytes!("../csstree/expected-classes.json"));
        assert_oracle_record_failure_absent(&artifacts, OracleContractFailureKind::DuplicateCaseId);
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["records"][1]["id"] = oracle["records"][0]["id"].clone();
        });
        assert_rejected(artifacts, "duplicate CSS oracle case ID");
    }

    #[test]
    fn oracle_loader_rejects_missing_ids() {
        let mut artifacts =
            oracle_record_test_artifacts(include_bytes!("../csstree/expected-classes.json"));
        let oracle = load_csstree_oracle_schema(&artifacts.oracle)
            .expect("canonical oracle before removing one record");
        assert_eq!(oracle.records.len(), EXPECTED_CASES);
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["records"]
                .as_array_mut()
                .expect("oracle record array")
                .pop();
        });
        assert_rejected(artifacts, "CSS oracle case census mismatch");
    }

    #[test]
    fn oracle_loader_rejects_extra_ids() {
        let mut artifacts =
            oracle_record_test_artifacts(include_bytes!("../csstree/expected-classes.json"));
        assert_oracle_record_failure_absent(&artifacts, OracleContractFailureKind::ExtraCaseId);
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            let records = oracle["records"]
                .as_array_mut()
                .expect("oracle record array");
            records.last_mut().expect("last record")["id"] = Value::from("zzzz-extra");
        });
        assert_rejected(artifacts, "extra CSS oracle case ID");
    }

    #[test]
    fn oracle_loader_rejects_stale_report_digest() {
        let mut artifacts = committed_artifacts();
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["generation_report_sha256"] = Value::from("0".repeat(64));
        });
        assert_rejected(artifacts, "generation report digest mismatch");
    }

    #[test]
    fn oracle_loader_rejects_noncanonical_record_order() {
        let mut artifacts =
            oracle_record_test_artifacts(include_bytes!("../csstree/expected-classes.json"));
        assert_oracle_record_failure_absent(
            &artifacts,
            OracleContractFailureKind::NonCanonicalRecordOrder,
        );
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            oracle["records"]
                .as_array_mut()
                .expect("oracle record array")
                .swap(0, 1);
        });
        assert_rejected(artifacts, "CSS oracle records are not in canonical order");
    }

    #[test]
    fn oracle_loader_rejects_noncanonical_json_bytes() {
        let mut artifacts = committed_artifacts();
        let oracle: Value =
            serde_json::from_slice(&artifacts.oracle).expect("oracle JSON should deserialize");
        artifacts.oracle = serde_json::to_vec(&oracle).expect("serialize compact oracle JSON");
        assert_rejected(artifacts, "CSS oracle is not canonical pretty JSON");
    }

    #[test]
    fn oracle_loader_rejects_unknown_probe_tag() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["probe"]["kind"] = Value::from("unknown");
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_active_adapter_outside_closed_registry() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            let record = &mut oracle["records"][0];
            let options = record["options"].clone();
            let payload = record["probe"]["payload"].clone();
            record["probe"] = serde_json::json!({
                "kind": "active",
                "entry_point": "sheet",
                "adapter": "deferred_to_t2",
                "extractor": { "kind": "sheet_rules" },
                "property_or_descriptor": null,
                "options": options,
                "payload": payload
            });
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_unknown_active_extractor_tag() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            let record = &mut oracle["records"][0];
            let options = record["options"].clone();
            let payload = record["probe"]["payload"].clone();
            record["probe"] = serde_json::json!({
                "kind": "active",
                "entry_point": "sheet",
                "adapter": "stylesheet",
                "extractor": { "kind": "unknown" },
                "property_or_descriptor": null,
                "options": options,
                "payload": payload
            });
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_unknown_outcome_tag() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["outcome"] = serde_json::json!({ "kind": "expected_fail" });
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_quarantined_outcome_tag() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["outcome"] = serde_json::json!({ "kind": "quarantined" });
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_unknown_options() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["options"]["unknown"] = Value::Bool(true);
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_unknown_unsupported_reason() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["outcome"]["reason"] = Value::from("not_implemented");
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_unknown_unsupported_policy() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["outcome"]["policy"] = Value::from("skip");
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_keeps_probe_and_outcome_phases_separate() {
        let mut artifacts = committed_artifacts();
        mutate_oracle(&mut artifacts, |oracle| {
            oracle["records"][0]["probe"]["outcome"] = Value::from("unsupported");
        });
        assert_rejected(artifacts, "failed to deserialize CSS oracle");
    }

    #[test]
    fn oracle_loader_rejects_stale_provider_revision_and_tree() {
        for field in ["provider_repository", "source_revision", "source_tree"] {
            let mut artifacts = committed_artifacts();
            mutate_canonical_oracle(&mut artifacts, |oracle| {
                oracle[field] = Value::from("stale");
            });
            assert_rejected(artifacts, "unexpected CSS oracle");
        }
    }

    #[test]
    fn oracle_loader_rejects_path_context_options_and_input_drift() {
        let mutations: [OracleMutation; 4] = [
            (
                "path mismatch",
                OracleContractFailureKind::PathMismatch,
                |oracle| {
                    oracle["records"][0]["path"] = Value::from("expectations/extra.json");
                },
            ),
            (
                "context mismatch",
                OracleContractFailureKind::ContextMismatch,
                |oracle| {
                    oracle["records"][0]["context"] = Value::from("value");
                },
            ),
            (
                "options mismatch",
                OracleContractFailureKind::OptionsMismatch,
                |oracle| {
                    oracle["records"][0]["options"]["parseValue"] = Value::Bool(false);
                },
            ),
            (
                "input mismatch",
                OracleContractFailureKind::InputMismatch,
                |oracle| {
                    oracle["records"][0]["input"] = Value::from("different");
                },
            ),
        ];
        for (expected, kind, mutation) in mutations {
            let mut artifacts =
                oracle_record_test_artifacts(include_bytes!("../csstree/expected-classes.json"));
            assert_oracle_record_failure_absent(&artifacts, kind);
            mutate_canonical_oracle(&mut artifacts, mutation);
            assert_rejected(artifacts, expected);
        }
    }

    #[test]
    fn oracle_loader_rejects_stale_payload_length() {
        let registry = REGISTRY
            .iter()
            .find(|entry| entry.fixture_path() == "expectations/stylesheet/StyleSheet.json")
            .copied()
            .expect("stylesheet registry entry");
        let mut record = RawOracleRecord {
            id: "stylesheet/StyleSheet.json#/payload-length".into(),
            path: "expectations/stylesheet/StyleSheet.json".into(),
            expectation_sha256: "0".repeat(64),
            source: "source/stylesheet/StyleSheet.json".into(),
            context: Context::Stylesheet,
            input: ".é{}".into(),
            options: Options::default(),
            probe: Probe::Active {
                entry_point: EntryPoint::Sheet,
                adapter: Adapter::Stylesheet,
                extractor: Extractor::SheetRules,
                property_or_descriptor: NullableString(None),
                options: Options::default(),
                payload: Payload {
                    prefix: String::new(),
                    suffix: String::new(),
                    input_byte_length: 5,
                },
            },
            outcome: Outcome::Clean,
            observation: NullableObservation(Some(Observation {
                extractor: Extractor::SheetRules,
                syntax_count: 1,
                is_clean: true,
                diagnostics: Vec::new(),
            })),
        };
        validate_probe(&record, registry).expect("the independently authored probe is valid");
        let Probe::Active { payload, .. } = &mut record.probe else {
            panic!("the fixture uses an active stylesheet probe");
        };
        payload.input_byte_length = 6;
        assert_eq!(
            validate_probe(&record, registry).expect_err("stale byte length must be rejected"),
            "CSS oracle input byte length mismatch for stylesheet/StyleSheet.json#/payload-length: oracle 6, input 5"
        );
    }

    #[test]
    fn oracle_loader_rejects_full_observation_without_observation() {
        const ID: &str = "atrule/block.json#/shouldn't create a raw node when no prelude and parseAtrulePrelude is false";
        let mut expected: Value =
            serde_json::from_slice(include_bytes!("../csstree/expected-classes.json")).unwrap();
        let expected_record = expected["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|record| record["id"] == ID)
            .unwrap();
        // The fixture tests the oracle loader independently of the unfinished
        // corpus audit. An unknown outer rule rejects the whole raw request.
        expected_record["class"]["policy"]["diagnostics"][0]["action"] =
            Value::String("reject_input".into());
        let expected: RawExpectedClasses = serde_json::from_value(expected).unwrap();
        let expected = format!("{}\n", serde_json::to_string_pretty(&expected).unwrap());
        let mut artifacts = oracle_record_test_artifacts(expected.as_bytes());
        // Independently specify the complete observation for an unknown @test
        // rule. This record's upstream parser option has no public equivalent,
        // so its existing expected class requires full observation.
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            let record = oracle["records"]
                .as_array_mut()
                .expect("oracle records")
                .iter_mut()
                .find(|record| record["id"] == ID)
                .expect("unknown at-rule option fixture");
            assert_eq!(record["input"], "@test {}");
            assert_eq!(
                record["options"],
                serde_json::json!({ "parseAtrulePrelude": false })
            );
            record["probe"] = serde_json::json!({
                "kind": "active",
                "entry_point": "rule",
                "adapter": "top_level_at_rule",
                "extractor": { "kind": "at_rule" },
                "property_or_descriptor": null,
                "options": { "parseAtrulePrelude": false },
                "payload": { "prefix": "", "suffix": "", "input_byte_length": 8 },
            });
            record["outcome"] = serde_json::json!({
                "kind": "unsupported",
                "reason": "upstream_parser_option_without_public_css_equivalent",
                "policy": "full_observation",
            });
            record["observation"] = serde_json::json!({
                "extractor": { "kind": "at_rule" },
                "syntax_count": 0,
                "is_clean": false,
                "diagnostics": [{
                    "code": "unknown_at_rule",
                    "action": "reject_input",
                    "byte_offset": 0,
                    "span_start": 0,
                    "span_end": 8,
                    "multiplicity": 1,
                    "payload_relation": "intersects",
                }],
            });
        });
        let oracle = load_csstree_oracle_schema(&artifacts.oracle)
            .expect("canonical full-observation fixture");
        let record = oracle
            .records
            .iter()
            .find(|record| record.id == ID)
            .expect("full-observation fixture record");
        let registry = REGISTRY
            .iter()
            .find(|entry| entry.fixture_path() == record.path)
            .copied()
            .expect("full-observation fixture registry");
        validate_probe(record, registry).expect("the independently specified probe is valid");
        validate_outcome(record)
            .expect("full observation is valid before removing its observation");
        let failures = oracle_record_test_failures(&artifacts);
        assert!(
            failures.iter().all(|failure| failure.case_id() != ID),
            "the full-observation fixture must satisfy every record contract before mutation"
        );
        mutate_canonical_oracle(&mut artifacts, |oracle| {
            let record = oracle["records"]
                .as_array_mut()
                .expect("oracle records")
                .iter_mut()
                .find(|record| record["id"] == ID)
                .expect("full-observation fixture");
            record["observation"] = Value::Null;
        });
        assert_rejected(artifacts, "lacks observation");
    }

    // Options establish a media grammar only for the explicit atrule: media case.
    fn prelude_contract_case(source: &str, options: Value, class: Value) -> ValidatedCase {
        ValidatedCase {
            id: "atrulePrelude/index.json#raw-contract".into(),
            expectation_path: "expectations/atrulePrelude/index.json".into(),
            expectation_sha256: String::new(),
            source: "source/atrulePrelude/index.json".into(),
            context: Context::AtrulePrelude,
            _label: None,
            input: source.into(),
            _upstream_outcome: UpstreamOutcome::Parsed,
            _canonical_css: None,
            options: serde_json::from_value(options).unwrap(),
            expected_class: Some(serde_json::from_value(class).unwrap()),
        }
    }

    fn prelude_clean_class(count: usize) -> Value {
        serde_json::json!({"kind":"clean", "retained_syntax":{
            "extractor":{"kind":"media_queries"},
            "predicate":{"relation":"exact", "value":count}
        }})
    }

    #[test]
    fn prelude_media_options_select_raw_list_admission_and_payload() {
        for (source, count) in [("screen", 1), ("screen,print", 2), ("", 0)] {
            let case = prelude_contract_case(
                source,
                serde_json::json!({"atrule":"media"}),
                prelude_clean_class(count),
            );
            validate_expected_class(&case, case.expected_class.as_ref().unwrap()).unwrap();
            let observed = observe_csstree_record(&case)
                .expect("explicit media prelude has a raw query-list adapter");
            let value = serde_json::to_value(observed).unwrap();
            assert_eq!(value["probe"]["entry_point"], "media_query_list");
            assert_eq!(value["probe"]["adapter"], "media_at_rule_prelude");
            assert_eq!(value["probe"]["payload"]["prefix"], "");
            assert_eq!(value["probe"]["payload"]["suffix"], "");
            assert_eq!(value["probe"]["payload"]["input_byte_length"], source.len());
            assert_eq!(value["observation"]["syntax_count"], count);
            assert_eq!(value["observation"]["is_clean"], true);
            assert_eq!(value["observation"]["diagnostics"], serde_json::json!([]));
        }
    }

    #[test]
    fn prelude_without_rule_name_has_only_generic_panic_freedom_evidence() {
        let class = serde_json::json!({"kind":"unsupported", "reason":"generic_fragment_without_truthful_supported_property_or_descriptor", "policy":{"kind":"panic_freedom_only"}});
        let case = prelude_contract_case("foo bar", serde_json::json!({}), class);
        validate_expected_class(&case, case.expected_class.as_ref().unwrap())
            .expect("a generic unnamed prelude cannot claim a media grammar");
        let value = serde_json::to_value(observe_csstree_record(&case).unwrap()).unwrap();
        assert_eq!(value["probe"]["kind"], "panic_freedom");
        assert_eq!(value["probe"]["entry_point"], "style_attribute");
        assert_eq!(value["probe"]["adapter"], "custom_property_containment");
        assert_eq!(
            value["probe"]["payload"]["prefix"],
            "--surgeist-corpus-probe:"
        );
        assert_eq!(value["probe"]["payload"]["suffix"], ";");
        assert_eq!(value["observation"], Value::Null);
        assert_eq!(value["outcome"]["kind"], "unsupported");
        assert_eq!(value["outcome"]["policy"], "panic_freedom_only");
    }

    #[test]
    fn prelude_unrecognized_options_cannot_select_the_default_media_route() {
        let case = prelude_contract_case(
            "screen",
            serde_json::json!({"parseValue":false}),
            prelude_clean_class(1),
        );
        assert!(validate_expected_class(&case, case.expected_class.as_ref().unwrap()).is_err());
        assert!(observe_csstree_record(&case).is_err());
    }
}
