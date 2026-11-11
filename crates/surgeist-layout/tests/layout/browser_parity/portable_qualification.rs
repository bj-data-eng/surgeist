//! Admission and execution of an immutable, registered current corpus edition.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use serde::Deserialize;
use serde_json::Value;

use super::support::{
    self, original_flex_parent, original_subgrid_body, prepared_atomic_percentage,
    prepared_range_qualification as prepared_range,
};
use support::capture_authority::{
    self as authority, CaptureMetadata, CapturedArtifact, CaseSelection, IdentityInputs,
    PathDigest, Qualification, SpecializedProfile, VerifiedIdentity, identity_from_verified_inputs,
    semantic_record_digest, sha256,
};
use support::exact_differences::{
    CaptureAuthority, ExactDifferenceSpec, PublishedEdition, QualificationError,
    QualifiedComparison, QualifiedVerdict, RawVerdict, complete_captured_comparison,
    flex_composition_qualification as flex, verify_wrapping_world_predecessor,
};
use support::{ObservedPlacement, capture_authority, parse_root_context};

#[path = "qualification_catalogue.rs"]
mod qualification_catalogue;
use qualification_catalogue::{Declaration, Record};

const OWNER_DECLARATION: &str = "examples/qualify-root-context/corpus.toml";

type Result<T> = std::result::Result<T, QualificationError>;

#[cfg(test)]
#[path = "original_diagnostic_tests.rs"]
mod original_diagnostic_tests;

type PlacementObserver<'a> =
    dyn FnMut(&ObservedPlacement) -> std::result::Result<(), support::Error> + 'a;

#[derive(Clone, Copy)]
pub(super) enum Edition {
    Exact(PublishedEdition),
    Specialized(SpecializedProfile),
    Matching(&'static str),
}

impl Edition {
    fn name(self) -> &'static str {
        match self {
            Self::Exact(edition) => edition.name(),
            Self::Specialized(profile) => profile.edition(),
            Self::Matching(edition) => edition,
        }
    }

    fn artifact_root(self) -> String {
        match self {
            Self::Exact(edition) => edition.artifact_root().to_owned(),
            Self::Specialized(profile) => profile.artifact_root().to_owned(),
            Self::Matching(edition) => format!("tests/layout/browser_parity/captured/{edition}"),
        }
    }
}

pub(super) struct Registration<'a> {
    pub(super) edition: Edition,
    pub(super) terminal_sha256: &'a str,
}

fn changed(component: impl Into<String>) -> QualificationError {
    QualificationError::IdentityChanged {
        component: component.into(),
    }
}

fn malformed(reason: impl Into<String>) -> QualificationError {
    QualificationError::MalformedDeclaration {
        reason: reason.into(),
    }
}

fn regular_file(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.is_empty() || relative.contains(['\\', '\0']) {
        return Err(malformed("invalid artifact-relative path"));
    }
    let components: Vec<_> = Path::new(relative).components().collect();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
        || relative
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(malformed(format!(
            "invalid artifact-relative path: {relative}"
        )));
    }
    let mut path = root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        path.push(component.as_os_str());
        let kind = fs::symlink_metadata(&path)
            .map_err(|error| changed(format!("{relative}: {error}")))?
            .file_type();
        let expected_kind = if index + 1 == components.len() {
            kind.is_file()
        } else {
            kind.is_dir()
        };
        if kind.is_symlink() || !expected_kind {
            return Err(changed(format!("non-regular artifact path: {relative}")));
        }
    }
    Ok(path)
}

fn read_pinned(root: &Path, entry: &PathDigest) -> Result<Vec<u8>> {
    let bytes = fs::read(regular_file(root, &entry.path)?)
        .map_err(|error| changed(format!("{}: {error}", entry.path)))?;
    if sha256(&bytes) != entry.sha256 {
        return Err(changed(format!("artifact bytes: {}", entry.path)));
    }
    Ok(bytes)
}

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let raw = std::str::from_utf8(bytes).map_err(|error| malformed(error.to_string()))?;
    let value = authority::unique_json(raw)?;
    serde_json::from_value(value).map_err(|error| malformed(error.to_string()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportSummary {
    generated: usize,
    unsupported: usize,
    expected_fail: usize,
    quarantined: usize,
    failed_to_generate: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    metadata: CaptureMetadata,
    #[serde(deserialize_with = "support::exact_differences::explicit_nullable")]
    filter: Option<String>,
    summary: ReportSummary,
    generated: Vec<CapturedArtifact>,
    unsupported: Vec<Value>,
    expected_fail: Vec<Value>,
    quarantined: Vec<Value>,
    failed_to_generate: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ownership {
    schema_version: u8,
    artifacts: BTreeMap<String, String>,
}

fn static_files(root: &Path) -> Result<BTreeSet<String>> {
    fn walk(root: &Path, directory: &Path, files: &mut BTreeSet<String>) -> Result<()> {
        let entries = fs::read_dir(directory).map_err(|error| changed(error.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|error| changed(error.to_string()))?;
            let kind = entry
                .file_type()
                .map_err(|error| changed(error.to_string()))?;
            let path = entry.path();
            if kind.is_symlink() {
                return Err(changed(format!("linked artifact: {}", path.display())));
            }
            if directory == root && entry.file_name() == ".surgeist-generator" {
                if !kind.is_dir() {
                    return Err(changed("invalid runtime control namespace"));
                }
                continue;
            }
            if kind.is_dir() {
                walk(root, &path, files)?;
            } else if kind.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| malformed(error.to_string()))?;
                let relative = relative
                    .to_str()
                    .ok_or_else(|| malformed("non-UTF8 artifact path"))?;
                files.insert(relative.replace(std::path::MAIN_SEPARATOR, "/"));
            } else {
                return Err(changed("non-regular artifact entry"));
            }
        }
        Ok(())
    }
    let mut files = BTreeSet::new();
    walk(root, root, &mut files)?;
    Ok(files)
}

struct Freshness {
    owner: PathBuf,
    root: PathBuf,
    artifact_root: String,
    receipt_name: &'static str,
    pins: BTreeMap<String, String>,
    owner_pins: BTreeMap<String, String>,
    selected_records: BTreeMap<String, Value>,
}

impl Freshness {
    fn verify_files(&self) -> Result<()> {
        regular_file(
            &self.owner,
            &format!("{}/{}", self.artifact_root, self.receipt_name),
        )?;
        for (path, digest) in &self.pins {
            read_pinned(
                &self.root,
                &PathDigest {
                    path: path.clone(),
                    sha256: digest.clone(),
                },
            )?;
        }
        for (path, digest) in &self.owner_pins {
            if path == prepared_range::POLICY_SOURCE {
                let repository = self
                    .owner
                    .parent()
                    .and_then(Path::parent)
                    .ok_or_else(|| changed("missing repository policy owner"))?;
                let file = regular_file(repository, path)?;
                let contract =
                    fs::read_to_string(file).map_err(|error| changed(error.to_string()))?;
                let section = prepared_range::PolicyIdentity::verified_section(&contract)
                    .map_err(|error| changed(error.to_string()))?;
                if sha256(section.as_bytes()) != *digest {
                    return Err(changed("current prepared Range policy section"));
                }
            } else {
                read_pinned(
                    &self.owner,
                    &PathDigest {
                        path: path.clone(),
                        sha256: digest.clone(),
                    },
                )?;
            }
        }
        if static_files(&self.root)? != self.pins.keys().cloned().collect() {
            return Err(changed("complete registered artifact census"));
        }
        Ok(())
    }

    fn verify(&self) -> Result<Vec<String>> {
        self.verify_files()?;
        verify_live(&self.owner, &self.selected_records)
    }
}

fn verify_live(owner: &Path, selected_records: &BTreeMap<String, Value>) -> Result<Vec<String>> {
    let bytes = fs::read(regular_file(owner, OWNER_DECLARATION)?)
        .map_err(|error| changed(format!("live catalogue: {error}")))?;
    let live =
        Declaration::parse(&bytes).map_err(|error| changed(format!("live catalogue: {error}")))?;
    live.check_live_records(selected_records)
        .map_err(|error| changed(error.to_string()))
}

fn record_value(record: &Record) -> Result<Value> {
    serde_json::to_value(record).map_err(|error| malformed(error.to_string()))
}

fn selected_records<Case>(
    root: &Path,
    terminal: &authority::QualificationData<Case>,
) -> Result<BTreeMap<String, Value>> {
    let bytes = read_pinned(root, &terminal.declaration)?;
    let captured = Declaration::parse(&bytes)
        .map_err(|error| malformed(format!("retained catalogue: {error}")))?;
    let records: BTreeMap<_, _> = captured
        .records
        .iter()
        .map(|record| (record.source.as_str(), record))
        .collect();
    terminal
        .records
        .iter()
        .map(|projection| {
            let record = records
                .get(projection.source.as_str())
                .ok_or_else(|| changed("selected record missing from retained catalogue"))?;
            let value = record_value(record)?;
            if value != projection.value {
                return Err(changed(format!(
                    "retained record projection: {}",
                    projection.source
                )));
            }
            Ok((projection.source.clone(), value))
        })
        .collect()
}

struct AdmittedCase {
    output: String,
    comparison: AdmittedComparison,
    golden: support::Golden,
}

enum AdmittedComparison {
    MatchingGeometry {
        category: authority::MatchingCategory,
        source: String,
    },
    Exact {
        edition: PublishedEdition,
        spec: Box<ExactDifferenceSpec>,
        identity: Box<VerifiedIdentity>,
    },
    PreparedRange {
        variant: String,
    },
    OriginalFlexParent,
    OriginalSubgridBody {
        variant: String,
    },
    PreparedAtomicPercentage,
    BrFontBand(support::BrFontBandProfile),
    FlexComposition {
        variant: String,
        inputs: Rc<flex::Inputs>,
    },
}

pub struct AdmittedEdition {
    freshness: Freshness,
    diagnostic_freshness: Option<Freshness>,
    cases: Vec<AdmittedCase>,
}

pub(super) struct CaseOutcome {
    pub(super) output: String,
    pub(super) raw: std::result::Result<Vec<String>, String>,
    pub(super) qualified: std::result::Result<QualifiedComparison, String>,
}

pub struct EditionRun {
    pub(super) cases: Vec<CaseOutcome>,
    pub declared_cases: usize,
    pub(super) freshness_error: Option<String>,
    pub uncovered: Vec<String>,
}

impl EditionRun {
    fn finish(
        mut cases: Vec<CaseOutcome>,
        declared_cases: usize,
        freshness: Result<Vec<String>>,
    ) -> Self {
        let (uncovered, freshness_error) = match freshness {
            Ok(uncovered) => (uncovered, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        };
        if let Some(error) = &freshness_error {
            for case in &mut cases {
                case.qualified = Err(error.clone());
            }
        }
        Self {
            cases,
            declared_cases,
            freshness_error,
            uncovered,
        }
    }

    pub(super) fn selected_cases_passed(&self) -> bool {
        self.freshness_error.is_none() && self.cases.iter().all(|case| case.qualified.is_ok())
    }

    pub fn registered_coverage_complete(&self) -> bool {
        self.selected_cases_passed() && self.cases.len() == self.declared_cases
    }

    pub fn current_coverage_complete(&self) -> bool {
        self.registered_coverage_complete() && self.uncovered.is_empty()
    }
}

impl AdmittedEdition {
    fn verify_diagnostic_freshness(&self) -> Result<()> {
        if let Some(diagnostics) = &self.diagnostic_freshness {
            diagnostics.verify()?;
        }
        Ok(())
    }

    pub(super) fn outputs(&self) -> impl Iterator<Item = &str> {
        self.cases.iter().map(|case| case.output.as_str())
    }

    pub fn execute_selected(&self, selected: &BTreeSet<String>) -> Result<EditionRun> {
        self.execute_selected_before_freshness(selected, || {})
    }

    fn execute_selected_before_freshness(
        &self,
        selected: &BTreeSet<String>,
        before_freshness: impl FnOnce(),
    ) -> Result<EditionRun> {
        let cases = self.execute_cases(selected, &mut |_| Ok(()))?;
        before_freshness();
        Ok(EditionRun::finish(
            cases,
            self.cases.len(),
            self.freshness.verify().and_then(|uncovered| {
                self.verify_diagnostic_freshness()?;
                Ok(uncovered)
            }),
        ))
    }

    fn execute_cases(
        &self,
        selected: &BTreeSet<String>,
        observe_placement: &mut PlacementObserver<'_>,
    ) -> Result<Vec<CaseOutcome>> {
        let known: BTreeSet<_> = self.outputs().map(str::to_owned).collect();
        if !selected.is_subset(&known) {
            return Err(changed("selected output is outside the registered edition"));
        }
        let mut cases = Vec::new();
        for case in self
            .cases
            .iter()
            .filter(|case| selected.contains(&case.output))
        {
            let completed = match &case.comparison {
                AdmittedComparison::Exact {
                    edition,
                    spec,
                    identity,
                } => {
                    let authority = CaptureAuthority::Published {
                        edition: *edition,
                        identity: identity.identity(),
                    };
                    spec.resolve_against(authority, &case.golden)
                        .and_then(|resolved| {
                            complete_captured_comparison(&case.golden, &resolved).map(|completed| {
                                let raw = completed
                                    .raw_differences()
                                    .map(|field| format!("{field:?}"))
                                    .collect();
                                let qualified = resolved
                                    .qualify(&completed)
                                    .map_err(|error| error.to_string());
                                (raw, qualified)
                            })
                        })
                }
                AdmittedComparison::MatchingGeometry { category, source } => {
                    support::exact_differences::compare_matching_capture(
                        *category,
                        source,
                        &case.golden,
                    )
                }
                AdmittedComparison::PreparedRange { variant } => {
                    compare_prepared_range(&case.golden, variant)
                }
                AdmittedComparison::OriginalFlexParent => {
                    original_flex_parent::compare(&case.golden)
                }
                AdmittedComparison::OriginalSubgridBody { variant } => {
                    original_subgrid_body::compare(&case.golden, variant).map(
                        |(fields, qualified)| {
                            (fields, qualified.map_err(|error| error.to_string()))
                        },
                    )
                }
                AdmittedComparison::BrFontBand(profile) => {
                    support::br_font_band::compare_authenticated(&case.golden, *profile)
                        .map(|()| {
                            let matched = QualifiedComparison {
                                raw: RawVerdict::Matches,
                                qualified: QualifiedVerdict::MatchingControl,
                            };
                            (Vec::new(), Ok(matched))
                        })
                        .map_err(|error| QualificationError::ComparisonFailure {
                            reason: error.to_string(),
                        })
                }
                AdmittedComparison::PreparedAtomicPercentage => {
                    prepared_atomic_percentage::compare(&case.golden, &mut *observe_placement)
                        .map(|(fields, qualified)| {
                            (fields, qualified.map_err(|error| error.to_string()))
                        })
                        .map_err(|error| QualificationError::ComparisonFailure {
                            reason: error.to_string(),
                        })
                }
                AdmittedComparison::FlexComposition { variant, inputs } => {
                    flex::compare(&case.golden, variant, inputs).map(|comparison| {
                        let (fields, qualified) = comparison.into_outcomes();
                        (fields, qualified.map_err(|error| error.to_string()))
                    })
                }
            };
            let (raw, qualified) = match completed {
                Ok((raw, qualified)) => (Ok(raw), qualified),
                Err(error) => {
                    let reason = error.to_string();
                    (Err(reason.clone()), Err(reason))
                }
            };
            cases.push(CaseOutcome {
                output: case.output.clone(),
                raw,
                qualified,
            });
        }
        Ok(cases)
    }
}

fn compare_prepared_range(
    golden: &support::Golden,
    variant: &str,
) -> Result<(
    Vec<String>,
    std::result::Result<QualifiedComparison, String>,
)> {
    let comparison_error = |error: support::Error| QualificationError::ComparisonFailure {
        reason: error.to_string(),
    };
    let (fields, qualified) = prepared_range::compare(golden, prepared_range::SOURCE, variant)
        .and_then(prepared_range::PreparedComparison::into_outcomes)
        .map_err(comparison_error)?;
    Ok((fields, qualified.map_err(|error| error.to_string())))
}

pub struct AdmittedRegistry {
    editions: Vec<AdmittedEdition>,
    owner: PathBuf,
    selected_records: BTreeMap<String, Value>,
}

impl AdmittedRegistry {
    pub(super) fn outputs(&self) -> impl Iterator<Item = String> {
        self.editions.iter().flat_map(|edition| {
            let root = &edition.freshness.artifact_root;
            edition
                .outputs()
                .map(move |output| format!("{root}/{output}"))
        })
    }

    pub fn execute_selected(&self, selected: &BTreeSet<String>) -> Result<EditionRun> {
        self.execute_selected_with_placement(selected, &mut |_| Ok(()))
    }

    pub(super) fn execute_selected_with_placement(
        &self,
        selected: &BTreeSet<String>,
        observe_placement: &mut PlacementObserver<'_>,
    ) -> Result<EditionRun> {
        let known = self.outputs().collect::<BTreeSet<_>>();
        if !selected.is_subset(&known) {
            return Err(changed("selected output is outside the current registry"));
        }
        let mut cases = Vec::new();
        let mut declared_cases = 0;
        for edition in &self.editions {
            let prefix = format!("{}/", edition.freshness.artifact_root);
            let edition_selection = selected
                .iter()
                .filter_map(|output| output.strip_prefix(&prefix).map(str::to_owned))
                .collect();
            for mut outcome in edition.execute_cases(&edition_selection, observe_placement)? {
                outcome.output = format!("{prefix}{}", outcome.output);
                cases.push(outcome);
            }
            declared_cases += edition.cases.len();
        }
        let freshness = self
            .editions
            .iter()
            .try_for_each(|edition| {
                edition.freshness.verify_files()?;
                edition.verify_diagnostic_freshness()
            })
            .and_then(|()| verify_live(&self.owner, &self.selected_records));
        Ok(EditionRun::finish(cases, declared_cases, freshness))
    }
}

// This seal admits only the publisher's complete terminal artifact. Retained
// projections and directory existence do not establish capture authority.
const REGISTERED_WRAPPING_WORLD_TERMINAL_SHA256: &str =
    "38882c8082ff30ad035e0460e41d72e04a2142af9e0d4d2fb8ea650760429d03";
const REGISTERED_MEASURED_GRID_TERMINAL_SHA256: &str =
    "bbf4da5d75460a35cde9f97fd9f146077631688f3a72260c6d4ee962c5bbd810";

const REGISTERED_FLEX_TEXT_TERMINAL_SHA256: &str =
    "f4a466130c99a38029e14eb17d15fbe5355e9c38585bf76024b59fd70dc165cd";

const REGISTERED_PREPARED_RANGE_TERMINAL_SHA256: &str =
    "35d31dacd87c4acb96e8e0759834155ec19bb707d32ef80699b5756c6dad6981";

const REGISTERED_FLEX_COMPOSITION_TERMINAL_SHA256: &str =
    "da9ef04965e489c3ecfa19388afbad4eb1d37f67a2e024a85b4620a9aa3d2557";

const REGISTERED_GRID_STATIC_POSITION_TERMINAL_SHA256: &str =
    "c9c23df593a48d3dd9531e59e6cf5d259eb59cb655469835ab375bc523cb0f34";

const REGISTERED_RELATIVE_INSET_TERMINAL_SHA256: &str =
    "35b2c35dfe4b4023300719ad08c5b45d619b5031531c8b085ab2843520d19339";

const REGISTERED_ORIGINAL_FLEX_PARENT_TERMINAL_SHA256: &str =
    "157d78c22bb9ed255a975a31f0e2d071877326cbd9cb0b37526e044ec8c4e985";

const REGISTERED_ORIGINAL_FLEX_AXES_TERMINAL_SHA256: &str =
    "c8a2285df0982669f1eb36180438187c0bf1ae27d936db8555c8f050790345af";
const REGISTERED_ORIGINAL_GRID_AXES_TERMINAL_SHA256: &str =
    "542e82a38d4c16f1ba8d28d1ddcc452406738b3bba2e27c9626f40fbf1e12c69";
const REGISTERED_ORIGINAL_SUBGRID_AXES_TERMINAL_SHA256: &str =
    "d8dc92425e265b9fbf28cf59acdd323311f24a2a19e0b6c66f71193143a0914f";
const REGISTERED_ORIGINAL_BLOCK_AXES_TERMINAL_SHA256: &str =
    "a1f3349d560df61ac553681140608f509ed1e06b678a0f73fc44061e42633942";
const REGISTERED_ORIGINAL_PERCENTAGE_FLEX_PARENT_TERMINAL_SHA256: &str =
    "2cfa177fff59ccfdc4e5c62f0196a18ec474c9c2e6f323b198690443b5ea64f0";

const REGISTERED_ORIGINAL_SUBGRID_BODY_TERMINAL_SHA256: &str =
    "5f49ddb8e8a2479e9de8e415bd397a5f2e181a8dda7d42fb66651d67ac1e0387";
const REGISTERED_PREPARED_ATOMIC_PERCENTAGE_TERMINAL_SHA256: &str =
    "5f850f59dba596fbc1e05bd5e110e55c537f88b49763bd635d74a6d4753a2e46";

const REGISTERED_HORIZONTAL_BR_FONT_BAND_TERMINAL_SHA256: &str =
    "2fdfde36f06d9a0c501a68e9edc0b36854d86536fe390e2ba87d0836a2a2454f";

const REGISTERED_VERTICAL_BR_FONT_BAND_TERMINAL_SHA256: &str =
    "0452f816eee7167575693d4a21b49f924fc681b46908e263c6cecc6b0f951b22";

const REGISTERED_EMPTY_LINE_BR_FONT_BAND_TERMINAL_SHA256: &str =
    "ddcfea5bf7ea485d64c73aa4ea7425c312a6e2771809616563f29c363a29e657";

pub(super) fn admit_grid_static_positions(owner: &Path) -> Result<AdmittedEdition> {
    admit(
        owner,
        Registration {
            edition: Edition::Exact(PublishedEdition::GridStaticPosition),
            terminal_sha256: REGISTERED_GRID_STATIC_POSITION_TERMINAL_SHA256,
        },
    )
}

// Register only a complete publisher terminal after reviewed capture/check.
const REGISTERED_GRID_LANES_PAIRED_TERMINAL_SHA256: &str =
    "7e634f9608d3e362ec42670bbca337b389241b4e3c095fb4e216ccdd073d7f2b";

pub(super) fn admit_grid_lanes_axis_controls(owner: &Path) -> Result<AdmittedEdition> {
    admit(
        owner,
        Registration {
            edition: Edition::Matching(authority::GRID_LANES_PAIRED_EDITION),
            terminal_sha256: REGISTERED_GRID_LANES_PAIRED_TERMINAL_SHA256,
        },
    )
}

const REGISTERED_MATCHING: &[(&str, &str)] = &[
    (
        "fri05-computed-overflow",
        "e211f16cd17282ed95765ba578cfc09a3c9a3a1d09de8fede992a14b2ab594fe",
    ),
    (
        authority::FRI06_SUBGRID_EDITION,
        "d2441c9dd6c15eff6e5614ca6e8540676041a1b6d7e27a5b93cd363e5be79d32",
    ),
    (
        "fri07-ordinary-flex-boxes",
        "1c732137441ce14b8c7c9e59b4ce741c39708ae2543aebcefc6ab60c50c50b78",
    ),
    (
        authority::FRI07_INTRINSIC_FLEX_EDITION,
        "7b156a74615954eb2a6d90f210b9f2ef33bb5b878e2b435e5009b570b3df481f",
    ),
    (
        crate::original_inline_subgrid_profile::EDITION,
        "d8f83b8db359b6252b7c7f72cfa67f4865c16a83f8dcc47e4b0fe6dfc953f7cc",
    ),
    (
        authority::ORIGINAL_RELATIVE_RTL_SUBGRID_EDITION,
        "197161389553cca64302c8ff9971f5530bfc7cd84326c80f3b6b39a769e23be0",
    ),
    (
        "fri08-manifest-plain-box-predecessors",
        "db2f948cb18e3a6661e7297fc15164ebc121c8282e08be9d38b99ef2cc3c2ec9",
    ),
    (
        authority::GRID_LANES_PAIRED_EDITION,
        REGISTERED_GRID_LANES_PAIRED_TERMINAL_SHA256,
    ),
    (
        "baseline-math",
        "3958751ab93bbf1af1a6639f33c859a066a285885847797eaf02460b9236543d",
    ),
    (
        "grid-order",
        "84123b6cbeef3e3661d033e3bd035946e54ae0d20c694ae2038ab3daddec1652",
    ),
    (
        "scriptless-order",
        "217143a63d732823090482b5421b3f3271f0d0961e85cd3e79f5b3fb55430a50",
    ),
    (
        "box-controls",
        "047aa92fa87d200e5220ce9e707486187d452f0952475b5a453f68a5775d0d69",
    ),
];

pub(super) fn admit_registered(owner: &Path) -> Result<AdmittedRegistry> {
    let mut editions = vec![
        admit(
            owner,
            Registration {
                edition: Edition::Exact(PublishedEdition::WrappingWorldCorners),
                terminal_sha256: REGISTERED_WRAPPING_WORLD_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Exact(PublishedEdition::MeasuredGrid),
                terminal_sha256: REGISTERED_MEASURED_GRID_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Exact(PublishedEdition::FlexText),
                terminal_sha256: REGISTERED_FLEX_TEXT_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::PreparedRange),
                terminal_sha256: REGISTERED_PREPARED_RANGE_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::FlexComposition),
                terminal_sha256: REGISTERED_FLEX_COMPOSITION_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalFlexParent),
                terminal_sha256: REGISTERED_ORIGINAL_FLEX_PARENT_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalPercentageFlexParent),
                terminal_sha256: REGISTERED_ORIGINAL_PERCENTAGE_FLEX_PARENT_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalBlockAxes),
                terminal_sha256: REGISTERED_ORIGINAL_BLOCK_AXES_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalFlexAxes),
                terminal_sha256: REGISTERED_ORIGINAL_FLEX_AXES_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalGridAxes),
                terminal_sha256: REGISTERED_ORIGINAL_GRID_AXES_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalSubgridAxes),
                terminal_sha256: REGISTERED_ORIGINAL_SUBGRID_AXES_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::OriginalSubgridBody),
                terminal_sha256: REGISTERED_ORIGINAL_SUBGRID_BODY_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::PreparedAtomicPercentage),
                terminal_sha256: REGISTERED_PREPARED_ATOMIC_PERCENTAGE_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::HorizontalBrFontBand),
                terminal_sha256: REGISTERED_HORIZONTAL_BR_FONT_BAND_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::VerticalBrFontBand),
                terminal_sha256: REGISTERED_VERTICAL_BR_FONT_BAND_TERMINAL_SHA256,
            },
        )?,
        admit(
            owner,
            Registration {
                edition: Edition::Specialized(SpecializedProfile::EmptyLineBrFontBand),
                terminal_sha256: REGISTERED_EMPTY_LINE_BR_FONT_BAND_TERMINAL_SHA256,
            },
        )?,
        admit_grid_static_positions(owner)?,
        admit(
            owner,
            Registration {
                edition: Edition::Exact(PublishedEdition::RelativeInsets),
                terminal_sha256: REGISTERED_RELATIVE_INSET_TERMINAL_SHA256,
            },
        )?,
    ];
    for &(edition, terminal_sha256) in REGISTERED_MATCHING {
        editions.push(admit(
            owner,
            Registration {
                edition: Edition::Matching(edition),
                terminal_sha256,
            },
        )?);
    }
    let mut selected_records = BTreeMap::new();
    for edition in &editions {
        for (source, record) in &edition.freshness.selected_records {
            if selected_records
                .insert(source.clone(), record.clone())
                .is_some()
            {
                return Err(changed(format!(
                    "source is registered in more than one edition: {source}"
                )));
            }
        }
    }
    verify_live(owner, &selected_records)?;
    Ok(AdmittedRegistry {
        editions,
        owner: owner.to_owned(),
        selected_records,
    })
}

pub(super) fn admit(owner: &Path, registration: Registration<'_>) -> Result<AdmittedEdition> {
    let edition = registration.edition;
    if let Edition::Matching(name) = edition {
        authority::validate_matching_edition(name)?;
    }
    let artifact_root = edition.artifact_root();
    let root = owner.join(&artifact_root);
    regular_file(owner, &format!("{artifact_root}/{}", authority::TERMINAL))?;
    let terminal_pin = PathDigest {
        path: authority::TERMINAL.into(),
        sha256: registration.terminal_sha256.into(),
    };
    let terminal_bytes = read_pinned(&root, &terminal_pin)?;
    let terminal = Qualification::parse(
        std::str::from_utf8(&terminal_bytes).map_err(|error| malformed(error.to_string()))?,
    )?;
    if terminal.edition != edition.name() {
        return Err(changed("registered terminal edition association"));
    }
    if terminal.to_bytes()? != terminal_bytes {
        return Err(malformed("noncanonical terminal serialization"));
    }
    let mut pins = terminal.retained_inputs.clone();
    for entry in [&terminal.report, &terminal.ownership, &terminal_pin]
        .into_iter()
        .chain(terminal.cases.iter().map(|case| &case.descriptor().output))
    {
        if pins
            .insert(entry.path.clone(), entry.sha256.clone())
            .is_some()
        {
            return Err(malformed("input/output artifact paths overlap"));
        }
    }
    if terminal.current_owner_inputs.is_empty()
        || terminal.retained_inputs.get(&terminal.declaration.path)
            != Some(&terminal.declaration.sha256)
    {
        return Err(changed("canonical declaration freshness binding"));
    }
    let freshness = Freshness {
        owner: owner.into(),
        root: root.clone(),
        artifact_root,
        receipt_name: authority::TERMINAL,
        pins,
        owner_pins: terminal.current_owner_inputs.clone(),
        selected_records: selected_records(&root, &terminal)?,
    };
    freshness.verify()?;
    let provenance = terminal
        .retained_inputs
        .get("origin/source-provenance.json")
        .ok_or_else(|| changed("missing source-owner provenance"))?;
    let provenance: Value = parse_json(&read_pinned(
        &root,
        &PathDigest {
            path: "origin/source-provenance.json".into(),
            sha256: provenance.clone(),
        },
    )?)?;
    let sidecar = provenance
        .get("sidecar")
        .and_then(Value::as_str)
        .ok_or_else(|| malformed("missing source receipt owner"))?;
    let semantic_paths = terminal
        .cases
        .iter()
        .flat_map(|case| match case {
            authority::QualifiedCase::Exact(case) => vec![&case.spec, &case.policy],
            authority::QualifiedCase::Specialized(case) => case.evidence.bindings(),
            authority::QualifiedCase::Matching(_) => Vec::new(),
        })
        .map(|binding| binding.path.as_str())
        .collect();
    let source_kinds = terminal
        .records
        .iter()
        .map(|record| Ok((record.source.as_str(), record.source_kind()?)))
        .collect::<Result<_>>()?;
    if matches!(edition, Edition::Matching(_)) {
        verify_matching_original_selection(&root, &terminal, &source_kinds, &provenance)?;
    }
    if authority::current_owner_bindings(
        &terminal.retained_inputs,
        &source_kinds,
        &semantic_paths,
        sidecar,
    )? != terminal.current_owner_inputs
    {
        return Err(changed("complete current-owner input census"));
    }
    let (report, report_bytes) = complete_report(&root, &terminal)?;
    let artifacts: BTreeMap<_, _> = report
        .generated
        .iter()
        .map(|a| (a.name.as_str(), a))
        .collect();
    let mut records = BTreeMap::new();
    for record in &terminal.records {
        if semantic_record_digest(&record.value)? != record.sha256
            || record.value.get("source").and_then(Value::as_str) != Some(record.source.as_str())
            || records.insert(record.source.as_str(), record).is_some()
        {
            return Err(changed("sealed semantic record projection"));
        }
    }
    let edition = match edition {
        Edition::Exact(edition) => edition,
        Edition::Matching(_) => {
            let (cases, diagnostic_freshness) =
                admit_matching(owner, &root, &terminal, &report, &report_bytes, &artifacts)?;
            freshness.verify()?;
            if let Some(diagnostics) = &diagnostic_freshness {
                diagnostics.verify()?;
            }
            return Ok(AdmittedEdition {
                freshness,
                diagnostic_freshness,
                cases,
            });
        }
        Edition::Specialized(profile) => {
            let cases = admit_specialized(
                &root,
                &terminal,
                &report,
                &report_bytes,
                &artifacts,
                profile,
            )?;
            freshness.verify()?;
            return Ok(AdmittedEdition {
                freshness,
                diagnostic_freshness: None,
                cases,
            });
        }
    };
    let exact_cases = terminal
        .cases
        .iter()
        .map(|case| match case {
            authority::QualifiedCase::Exact(case) => Ok(case),
            authority::QualifiedCase::Specialized(_) | authority::QualifiedCase::Matching(_) => {
                Err(changed("expected an exact difference edition"))
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let excluded_specs = exact_cases
        .iter()
        .map(|case| case.spec.path.clone())
        .collect();
    let mut consumed = BTreeSet::new();
    let mut cases = Vec::new();
    let measured = edition == PublishedEdition::MeasuredGrid;
    let flex_text = edition == PublishedEdition::FlexText;
    let (observation_request, observation_input_sha256) = match edition {
        PublishedEdition::CanonicalBoxes
        | PublishedEdition::GridStaticPosition
        | PublishedEdition::RelativeInsets
        | PublishedEdition::WrappingWorldCorners => ("geometry", None),
        PublishedEdition::MeasuredGrid | PublishedEdition::FlexText => {
            let request = terminal
                .retained_inputs
                .get("observation-request.json")
                .ok_or_else(|| changed("missing retained observation request"))?;
            (
                if flex_text {
                    "flex-content-text"
                } else {
                    "measured-leaf"
                },
                Some(request.as_str()),
            )
        }
    };
    for case in exact_cases {
        let artifact = artifacts
            .get(case.case.name.as_str())
            .ok_or_else(|| changed("missing declared current output"))?;
        let record = records
            .get(case.case.source.as_str())
            .ok_or_else(|| changed("missing source record projection"))?;
        let reference = record
            .value
            .get("semantic_specifications")
            .and_then(|map| map.get(case.case.variant.as_str()))
            .ok_or_else(|| changed("missing canonical per-variant semantic reference"))?;
        let reference: PathDigest = serde_json::from_value(reference.clone())
            .map_err(|error| malformed(error.to_string()))?;
        if reference != case.spec
            || record.sha256 != case.case.record_sha256
            || !consumed.insert((record.source.as_str(), case.case.variant.as_str()))
        {
            return Err(changed(
                "current case differs from its canonical semantic reference",
            ));
        }
        let spec_bytes = read_pinned(&root, &case.spec)?;
        let spec = ExactDifferenceSpec::parse(
            std::str::from_utf8(&spec_bytes).map_err(|error| malformed(error.to_string()))?,
        )?;
        let (policy_path, policy_sha) = spec.policy_source();
        if policy_path != case.policy.path || policy_sha != case.policy.sha256 {
            return Err(changed("canonical policy binding"));
        }
        read_pinned(&root, &case.policy)?;
        let historical_bytes = case
            .historical
            .as_ref()
            .map(|entry| read_pinned(&root, entry))
            .transpose()?;
        let historical = historical_bytes.as_deref().map(sha256);
        if record.value.get("source_sha256").and_then(Value::as_str)
            != Some(artifact.source_sha256.as_str())
            || record
                .value
                .get("historical_sha256")
                .and_then(|map| map.get(case.case.variant.as_str()))
                .and_then(Value::as_str)
                != historical.as_deref()
        {
            return Err(changed("canonical source/history mapping"));
        }
        let xml = read_pinned(&root, &case.case.output)?;
        let xml_text = std::str::from_utf8(&xml).map_err(|error| malformed(error.to_string()))?;
        let golden = if flex_text {
            let source = read_pinned(
                &root,
                &PathDigest {
                    path: case.case.source.clone(),
                    sha256: artifact.source_sha256.clone(),
                },
            )?;
            support::Golden::parse_with_flex_content_text(
                xml_text,
                std::str::from_utf8(&report_bytes).map_err(|error| malformed(error.to_string()))?,
                &case.case.source,
                &source,
                &case.case.output.path,
            )
        } else {
            support::Golden::parse(xml_text)
        }
        .map_err(|error| malformed(error.to_string()))?;
        if golden.name != case.case.name
            || golden.has_element_scroll_sizes() != (measured || flex_text)
            || golden.measured_leaf.is_some() != measured
        {
            return Err(changed("canonical observation profile"));
        }
        if edition == PublishedEdition::WrappingWorldCorners {
            verify_wrapping_world_predecessor(
                &golden,
                historical_bytes
                    .as_deref()
                    .ok_or_else(|| changed("missing wrapping predecessor"))?,
            )?;
        }
        let selection = CaseSelection {
            name: case.case.name.clone(),
            source: case.case.source.clone(),
            source_sha256: artifact.source_sha256.clone(),
            variant: case.case.variant.clone(),
            output: case.case.output.path.clone(),
            record_sha256: record.sha256.clone(),
            historical_sha256: historical.clone(),
        };
        if spec.prebind_current(edition, &selection)? != case.requirements {
            return Err(changed("prebound current observable coverage"));
        }
        let xml_sha = sha256(&xml);
        let verified = identity_from_verified_inputs(IdentityInputs {
            metadata: &report.metadata,
            artifact,
            case: &case.case,
            full_inputs: &terminal.retained_inputs,
            excluded_specs: &excluded_specs,
            actual_xml_sha256: &xml_sha,
            historical_sha256: historical.as_deref(),
            policy_sha256: &case.policy.sha256,
            browser_settings: &terminal.browser_settings,
            observation_request,
            observation_input_sha256,
        })?;
        if verified.projection() != &case.identity {
            return Err(changed(
                "stored identity differs from authenticated actual inputs",
            ));
        }
        // Resolve every selected authority before any Native operation begins.
        spec.resolve_against(
            CaptureAuthority::Published {
                edition,
                identity: verified.identity(),
            },
            &golden,
        )?;
        cases.push(AdmittedCase {
            output: case.case.output.path.clone(),
            comparison: AdmittedComparison::Exact {
                edition,
                spec: Box::new(spec),
                identity: Box::new(verified),
            },
            golden,
        });
    }
    let expected_references: BTreeSet<_> = terminal
        .records
        .iter()
        .flat_map(|record| {
            record
                .value
                .get("semantic_specifications")
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(move |map| {
                    map.keys()
                        .map(move |variant| (record.source.as_str(), variant.as_str()))
                })
        })
        .collect();
    if consumed != expected_references || consumed.is_empty() {
        return Err(changed("complete selected canonical reference census"));
    }
    freshness.verify()?;
    Ok(AdmittedEdition {
        freshness,
        diagnostic_freshness: None,
        cases,
    })
}

pub(super) struct AuthenticatedOriginalDiagnostics {
    cases: BTreeMap<(String, String), support::Golden>,
    sources: BTreeMap<String, Vec<u8>>,
    freshness: Freshness,
}

impl AuthenticatedOriginalDiagnostics {
    pub(super) fn golden(&self, source: &str, variant: &str) -> Result<&support::Golden> {
        self.cases
            .get(&(source.to_owned(), variant.to_owned()))
            .ok_or_else(|| changed("missing authenticated original diagnostic case"))
    }

    fn source(&self, source: &str) -> Result<&[u8]> {
        self.sources
            .get(source)
            .map(Vec::as_slice)
            .ok_or_else(|| changed("missing authenticated original diagnostic source"))
    }
}

pub(super) fn admit_original_diagnostics(
    owner: &Path,
    receipt_sha256: &str,
) -> Result<AuthenticatedOriginalDiagnostics> {
    let artifact_root = format!(
        "tests/layout/browser_parity/captured/{}",
        authority::ORIGINAL_DIAGNOSTIC_EDITION
    );
    let root = owner.join(&artifact_root);
    let receipt_pin = PathDigest {
        path: authority::ORIGINAL_DIAGNOSTIC_RECEIPT.into(),
        sha256: receipt_sha256.into(),
    };
    regular_file(owner, &format!("{artifact_root}/{}", receipt_pin.path))?;
    let bytes = read_pinned(&root, &receipt_pin)?;
    let receipt = authority::OriginalDiagnosticReceipt::parse(
        std::str::from_utf8(&bytes).map_err(|error| malformed(error.to_string()))?,
    )?;
    if receipt.to_bytes()? != bytes {
        return Err(malformed("noncanonical original diagnostic receipt"));
    }
    let mut pins = receipt.retained_inputs.clone();
    for entry in [&receipt.report, &receipt.ownership, &receipt_pin]
        .into_iter()
        .chain(receipt.cases.iter().map(|case| &case.case.output))
    {
        if pins
            .insert(entry.path.clone(), entry.sha256.clone())
            .is_some()
        {
            return Err(malformed("original diagnostic input/output overlap"));
        }
    }
    let validator = authority::CaptureValidation::from_retained(&receipt.retained_inputs)?;
    let mut owner_pins = receipt.current_owner_inputs.clone();
    owner_pins.insert(
        authority::ROOT_HELPER_OWNER.to_owned(),
        validator.validator_helper_sha256,
    );
    let freshness = Freshness {
        owner: owner.into(),
        root: root.clone(),
        artifact_root,
        receipt_name: authority::ORIGINAL_DIAGNOSTIC_RECEIPT,
        pins,
        owner_pins,
        selected_records: selected_records(&root, &receipt)?,
    };
    freshness.verify()?;
    let mut declaration = Declaration::parse(&read_pinned(&root, &receipt.declaration)?)
        .map_err(|error| malformed(error.to_string()))?;
    declaration
        .select_original_diagnostics()
        .map_err(|error| malformed(error.to_string()))?;
    let records: BTreeMap<_, _> = declaration
        .records
        .iter()
        .map(|record| (record.source.as_str(), record))
        .collect();
    let source_kinds: BTreeMap<_, _> = records
        .iter()
        .map(|(source, record)| (*source, record.source_kind))
        .collect();
    verify_original_diagnostic_origin(&root, &receipt, &source_kinds, &freshness.owner_pins)?;
    let (report, _) = complete_report(&root, &receipt)?;
    if report.metadata.generator != authority::FIXED_GENERATOR
        || !report.metadata.source_imports.is_empty()
    {
        return Err(changed("original diagnostic report lineage"));
    }
    verify_fixed_capture_contract(
        &root,
        &receipt,
        &report,
        receipt
            .cases
            .iter()
            .map(|case| case.historical.path.as_str()),
        "Grid Lanes",
    )?;
    let artifacts: BTreeMap<_, _> = report
        .generated
        .iter()
        .map(|artifact| (artifact.name.as_str(), artifact))
        .collect();
    let mut cases = BTreeMap::new();
    for diagnostic in &receipt.cases {
        let case = &diagnostic.case;
        let record = records
            .get(case.source.as_str())
            .ok_or_else(|| changed("diagnostic source record"))?;
        let artifact = artifacts
            .get(case.name.as_str())
            .ok_or_else(|| changed("diagnostic generated artifact"))?;
        let xml = read_pinned(&root, &case.output)?;
        let xml = std::str::from_utf8(&xml).map_err(|error| malformed(error.to_string()))?;
        record
            .verify_original_diagnostic_capture(xml, &case.name, &case.variant)
            .map_err(|error| changed(error.to_string()))?;
        let identity =
            authority::capture_binding_from_verified_inputs(authority::CaptureBindingInputs {
                metadata: &report.metadata,
                artifact,
                case,
                full_inputs: &receipt.retained_inputs,
                excluded_specs: &BTreeSet::new(),
                actual_xml_sha256: &sha256(xml.as_bytes()),
                historical_sha256: Some(&diagnostic.historical.sha256),
                browser_settings: &receipt.browser_settings,
                observation_request: "geometry",
                observation_input_sha256: None,
            })?;
        if identity != diagnostic.identity {
            return Err(changed("original diagnostic capture binding"));
        }
        let golden = support::Golden::parse(xml).map_err(|error| changed(error.to_string()))?;
        if authority::original_diagnostic_differences(&golden)? != diagnostic.raw_differences {
            return Err(changed(format!(
                "original diagnostic numeric replay: {}",
                case.name
            )));
        }
        cases.insert((case.source.clone(), case.variant.clone()), golden);
    }
    if cases.len() != artifacts.len() {
        return Err(changed("complete original diagnostic replay census"));
    }
    let sources = records
        .into_iter()
        .map(|(source, record)| {
            let bytes = read_pinned(
                &root,
                &PathDigest {
                    path: source.to_owned(),
                    sha256: record.source_sha256.as_str().to_owned(),
                },
            )?;
            Ok((source.to_owned(), bytes))
        })
        .collect::<Result<_>>()?;
    freshness.verify()?;
    Ok(AuthenticatedOriginalDiagnostics {
        cases,
        sources,
        freshness,
    })
}

fn verify_fixed_capture_contract<'a, Case>(
    root: &Path,
    receipt: &'a authority::QualificationData<Case>,
    report: &Report,
    historical: impl Iterator<Item = &'a str>,
    cohort: &str,
) -> Result<()> {
    const BROWSER_OWNER: &str = "browser-owner.toml";
    let expected: BTreeSet<_> = authority::FIXED_CAPTURE_SHARED_INPUTS
        .into_iter()
        .chain(["corpus.toml", "origin/selected-originals.json"])
        .chain(receipt.records.iter().map(|record| record.source.as_str()))
        .chain(historical)
        .collect();
    if !expected
        .into_iter()
        .eq(receipt.retained_inputs.keys().map(String::as_str))
    {
        return Err(changed(format!("complete {cohort} capture input census")));
    }
    if report
        .generated
        .iter()
        .any(|artifact| !artifact.linked_resources.is_empty())
    {
        return Err(changed(format!(
            "{cohort} plain-box capture has linked resources"
        )));
    }

    // The generator owns full manifest validation and launch. The portable
    // reader compares only the owner fields that enter its settings witness.
    #[derive(Deserialize)]
    struct OwnerProjection {
        browser: BrowserProjection,
    }
    #[derive(Deserialize, serde::Serialize)]
    struct BrowserProjection {
        source: String,
        version: String,
        version_output: String,
        cache_root: surgeist_generator::RelativePath,
        provenance_format: String,
        launch: LaunchProjection,
    }
    #[derive(Deserialize, serde::Serialize)]
    struct LaunchProjection {
        batch_size: usize,
        navigation_timeout_ms: u64,
        dom_poll_interval_ms: u64,
        arguments: Vec<String>,
    }
    let manifest = read_pinned(
        root,
        &PathDigest {
            path: BROWSER_OWNER.into(),
            sha256: receipt.retained_inputs[BROWSER_OWNER].clone(),
        },
    )?;
    let manifest = std::str::from_utf8(&manifest).map_err(|error| malformed(error.to_string()))?;
    let owner: OwnerProjection = surgeist_generator::parse_manifest(manifest, BROWSER_OWNER)
        .map_err(|error| malformed(error.to_string()))?;
    let witness = serde_json::to_value(&receipt.browser_settings)
        .map_err(|error| malformed(error.to_string()))?;
    let declared =
        serde_json::to_value(&owner.browser).map_err(|error| malformed(error.to_string()))?;
    if witness != declared {
        return Err(changed(format!(
            "{cohort} browser settings differ from the owner manifest"
        )));
    }
    let executable = surgeist_generator::RelativePath::new(&report.metadata.browser_executable)
        .map_err(|error| malformed(error.to_string()))?;
    let prefix = format!("{}/", owner.browser.cache_root.as_str());
    let provenance = owner
        .browser
        .provenance_format
        .replace("{version}", &owner.browser.version)
        .replace("{repository_relative_executable}", executable.as_str());
    if !executable.as_str().starts_with(&prefix) || report.metadata.browser_provenance != provenance
    {
        return Err(changed(format!(
            "{cohort} browser executable or provenance"
        )));
    }
    Ok(())
}

fn verify_original_diagnostic_origin(
    root: &Path,
    receipt: &authority::OriginalDiagnosticReceipt,
    source_kinds: &BTreeMap<&str, authority::SourceKind>,
    current_owner_inputs: &BTreeMap<String, String>,
) -> Result<()> {
    let retained_json = |path: &str| -> Result<Value> {
        let hash = receipt
            .retained_inputs
            .get(path)
            .ok_or_else(|| changed(format!("missing original diagnostic origin input: {path}")))?;
        parse_json(&read_pinned(
            root,
            &PathDigest {
                path: path.into(),
                sha256: hash.clone(),
            },
        )?)
    };
    let source_receipt = retained_json("origin/source-receipt.json")?;
    let provenance = retained_json("origin/source-provenance.json")?;
    let sidecar = "html/.surgeist-source.json";
    if provenance["sidecar"].as_str() != Some(sidecar)
        || source_receipt["schema_version"].as_u64() != Some(1)
        || ["schema_version", "specification", "files"]
            .iter()
            .any(|key| source_receipt[key] != provenance[key])
    {
        return Err(changed("original diagnostic import receipt projection"));
    }
    if &authority::current_owner_bindings(
        &receipt.retained_inputs,
        source_kinds,
        &BTreeSet::new(),
        sidecar,
    )? != current_owner_inputs
    {
        return Err(changed("complete original diagnostic current-owner inputs"));
    }
    verify_matching_original_selection(root, receipt, source_kinds, &provenance)?;
    let imported = source_receipt["files"]
        .as_array()
        .ok_or_else(|| malformed("original import file inventory"))?;
    let authored = source_receipt["specification"]["authored_files"]
        .as_array()
        .ok_or_else(|| malformed("original authored source partition"))?;
    let selected = retained_json("origin/selected-originals.json")?;
    for entry in selected
        .as_array()
        .ok_or_else(|| malformed("selected original sources"))?
    {
        let source = entry["source"]
            .as_str()
            .and_then(|source| source.strip_prefix("html/"))
            .ok_or_else(|| malformed("selected original source path"))?;
        let admitted = match entry["ownership"].as_str() {
            Some("imported") => imported.iter().any(|file| {
                file["path"].as_str() == Some(source) && file["sha256"] == entry["sha256"]
            }),
            Some("repository-authored") => {
                authored.iter().any(|file| file.as_str() == Some(source))
                    && !imported
                        .iter()
                        .any(|file| file["path"].as_str() == Some(source))
            }
            _ => false,
        };
        if !admitted {
            return Err(changed("selected original diagnostic import partition"));
        }
    }
    Ok(())
}

fn complete_report<Case>(
    root: &Path,
    terminal: &authority::QualificationData<Case>,
) -> Result<(Report, Vec<u8>)> {
    let report_bytes = read_pinned(root, &terminal.report)?;
    let report: Report = parse_json(&report_bytes)?;
    if report.filter.is_some()
        || report.summary.generated != terminal.cases.len()
        || report.generated.len() != terminal.cases.len()
        || report.summary.unsupported != 0
        || !report.unsupported.is_empty()
        || report.summary.expected_fail != 0
        || !report.expected_fail.is_empty()
        || report.summary.quarantined != 0
        || !report.quarantined.is_empty()
        || report.summary.failed_to_generate != 0
        || !report.failed_to_generate.is_empty()
        || report.metadata.inputs != terminal.retained_inputs
    {
        return Err(changed("complete current generated report census"));
    }
    let ownership: Ownership = parse_json(&read_pinned(root, &terminal.ownership)?)?;
    let mut expected_outputs =
        BTreeMap::from([(terminal.report.path.clone(), terminal.report.sha256.clone())]);
    let mut names = BTreeSet::new();
    for artifact in &report.generated {
        if !names.insert(artifact.name.as_str())
            || expected_outputs
                .insert(artifact.output.clone(), artifact.output_sha256.clone())
                .is_some()
        {
            return Err(malformed("duplicate generated case or output"));
        }
    }
    if ownership.schema_version != 1 || ownership.artifacts != expected_outputs {
        return Err(changed("complete generated ownership census"));
    }
    Ok((report, report_bytes))
}

fn verify_matching_original_selection<Case>(
    root: &Path,
    terminal: &authority::QualificationData<Case>,
    source_kinds: &BTreeMap<&str, authority::SourceKind>,
    provenance: &Value,
) -> Result<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SelectedOriginal {
        source: String,
        ownership: String,
        sha256: String,
    }

    let receipt = terminal
        .retained_inputs
        .get("origin/source-receipt.json")
        .ok_or_else(|| changed("missing original source receipt"))?;
    if provenance["receipt_sha256"].as_str() != Some(receipt.as_str()) {
        return Err(changed("original source receipt/provenance association"));
    }
    let path = "origin/selected-originals.json";
    let sha256 = terminal
        .retained_inputs
        .get(path)
        .ok_or_else(|| changed("missing selected original source inventory"))?;
    let selected: Vec<SelectedOriginal> = parse_json(&read_pinned(
        root,
        &PathDigest {
            path: path.into(),
            sha256: sha256.clone(),
        },
    )?)?;
    let expected: BTreeSet<_> = source_kinds
        .iter()
        .filter_map(|(source, kind)| (*kind == authority::SourceKind::Original).then_some(*source))
        .collect();
    let mut actual = BTreeSet::new();
    for entry in &selected {
        let source = entry.source.as_str();
        if !actual.insert(source)
            || !expected.contains(source)
            || terminal.retained_inputs.get(source) != Some(&entry.sha256)
            || !matches!(entry.ownership.as_str(), "imported" | "repository-authored")
        {
            return Err(changed("selected original source ownership or digest"));
        }
    }
    if actual != expected {
        return Err(changed("complete selected original source census"));
    }
    Ok(())
}

fn admit_matching(
    owner: &Path,
    root: &Path,
    terminal: &Qualification,
    report: &Report,
    report_bytes: &[u8],
    artifacts: &BTreeMap<&str, &CapturedArtifact>,
) -> Result<(Vec<AdmittedCase>, Option<Freshness>)> {
    let declaration = Declaration::parse(&read_pinned(root, &terminal.declaration)?)
        .map_err(|error| malformed(error.to_string()))?;
    let records: BTreeMap<_, _> = declaration
        .records
        .iter()
        .filter(|r| r.matching_edition() == Some(terminal.edition.as_str()))
        .map(|r| (r.source.as_str(), r))
        .collect();
    if records.len() != terminal.records.len()
        || records.len() * crate::fixture_variants::fixture_cases().len() != terminal.cases.len()
    {
        return Err(changed("complete matching edition source census"));
    }
    if terminal.edition == authority::FRI06_SUBGRID_EDITION {
        verify_fixed_capture_contract(
            root,
            terminal,
            report,
            terminal.cases.iter().filter_map(|case| match case {
                authority::QualifiedCase::Matching(case) => case
                    .provenance()
                    .historical()
                    .map(|entry| entry.path.as_str()),
                _ => None,
            }),
            "FRI06 Subgrid",
        )?;
        if report.metadata.generator != authority::FIXED_GENERATOR
            || !report.metadata.source_imports.is_empty()
        {
            return Err(changed("FRI06 Subgrid report lineage"));
        }
    }
    let diagnostics = if terminal.edition == authority::GRID_LANES_PAIRED_EDITION {
        verify_fixed_capture_contract(root, terminal, report, std::iter::empty(), "Grid Lanes")?;
        if report.metadata.generator != authority::FIXED_GENERATOR
            || !report.metadata.source_imports.is_empty()
        {
            return Err(changed("paired Grid Lanes report lineage"));
        }
        let reference = records
            .values()
            .next()
            .and_then(|record| record.source_equivalent_matching())
            .ok_or_else(|| changed("missing paired diagnostic reference"))?;
        let originals = admit_original_diagnostics(owner, &reference.diagnostics.sha256)?;
        authority::verify_grid_lanes_shared_inputs(
            &originals.freshness.pins,
            &terminal.retained_inputs,
        )?;
        Some(originals)
    } else {
        None
    };
    let mut consumed = BTreeSet::new();
    let mut cases = Vec::new();
    for qualified in &terminal.cases {
        let authority::QualifiedCase::Matching(case) = qualified else {
            return Err(changed("expected matching geometry category"));
        };
        let descriptor = &case.case;
        let artifact = artifacts
            .get(descriptor.name.as_str())
            .ok_or_else(|| changed("missing matching generated artifact"))?;
        if !consumed.insert(descriptor.name.as_str()) {
            return Err(malformed("duplicate matching generated case"));
        }
        let record = records
            .get(descriptor.source.as_str())
            .ok_or_else(|| changed("missing matching source record"))?;
        let bytes = read_pinned(root, &descriptor.output)?;
        let xml = std::str::from_utf8(&bytes).map_err(|error| malformed(error.to_string()))?;
        if &record
            .matching_provenance(&descriptor.variant)
            .map_err(|error| changed(error.to_string()))?
            != case.provenance()
        {
            return Err(changed("matching declared source provenance"));
        }
        let historical = case
            .provenance()
            .historical()
            .map(|entry| read_pinned(root, entry))
            .transpose()?;
        let historical = historical
            .as_deref()
            .map(std::str::from_utf8)
            .transpose()
            .map_err(|error| malformed(error.to_string()))?;
        record
            .verify_matching_capture(xml, historical, &descriptor.name, &descriptor.variant)
            .map_err(|error| changed(error.to_string()))?;
        let identity =
            authority::capture_binding_from_verified_inputs(authority::CaptureBindingInputs {
                metadata: &report.metadata,
                artifact,
                case: descriptor,
                full_inputs: &terminal.retained_inputs,
                excluded_specs: &BTreeSet::new(),
                actual_xml_sha256: &sha256(&bytes),
                historical_sha256: case
                    .provenance()
                    .historical()
                    .map(|entry| entry.sha256.as_str()),
                browser_settings: &terminal.browser_settings,
                observation_request: case.category().observation_request(),
                observation_input_sha256: case
                    .category()
                    .observation_input_sha256(&terminal.retained_inputs),
            })?;
        if &identity != case.identity() {
            return Err(changed("matching capture binding projection"));
        }
        let golden = if case.category() == authority::MatchingCategory::Fri07IntrinsicFlexText {
            let source = read_pinned(
                root,
                &PathDigest {
                    path: descriptor.source.clone(),
                    sha256: artifact.source_sha256.clone(),
                },
            )?;
            support::Golden::parse_with_flex_content_text(
                xml,
                std::str::from_utf8(report_bytes).map_err(|error| malformed(error.to_string()))?,
                &descriptor.source,
                &source,
                &descriptor.output.path,
            )
        } else {
            support::Golden::parse(xml)
        }
        .map_err(|error| changed(error.to_string()))?;
        // Validate supported complete geometry without accepting a mismatch.
        let (raw, qualified) = support::exact_differences::compare_matching_capture(
            case.category(),
            &descriptor.source,
            &golden,
        )?;
        if !raw.is_empty() || qualified.is_err() {
            return Err(changed(format!(
                "matching edition contains raw geometry differences: {raw:?}"
            )));
        }
        if let Some(pair) = record.source_equivalent_matching() {
            let diagnostics = diagnostics
                .as_ref()
                .ok_or_else(|| changed("missing authenticated original diagnostics"))?;
            let control_source = read_pinned(
                root,
                &PathDigest {
                    path: descriptor.source.clone(),
                    sha256: record.source_sha256.as_str().to_owned(),
                },
            )?;
            support::grid_lanes_native_pair::verify_declared_pair(
                pair,
                &descriptor.variant,
                diagnostics.source(&pair.original)?,
                &control_source,
                diagnostics.golden(&pair.original, &descriptor.variant)?,
                &golden,
            )
            .map_err(|error| changed(format!("{}: {error}", descriptor.name)))?;
        }
        cases.push(AdmittedCase {
            output: descriptor.output.path.clone(),
            comparison: AdmittedComparison::MatchingGeometry {
                category: case.category(),
                source: descriptor.source.clone(),
            },
            golden,
        });
    }
    if consumed.len() != artifacts.len() {
        return Err(changed("unconsumed matching generated artifact"));
    }
    Ok((cases, diagnostics.map(|diagnostics| diagnostics.freshness)))
}

fn admit_specialized(
    root: &Path,
    terminal: &Qualification,
    report: &Report,
    report_bytes: &[u8],
    artifacts: &BTreeMap<&str, &CapturedArtifact>,
    profile: SpecializedProfile,
) -> Result<Vec<AdmittedCase>> {
    if report.metadata.generator != authority::FIXED_GENERATOR
        || !report.metadata.source_imports.is_empty()
    {
        return Err(changed("specialized publication report lineage"));
    }
    let report_text =
        std::str::from_utf8(report_bytes).map_err(|error| malformed(error.to_string()))?;
    let flex_inputs = match profile {
        SpecializedProfile::PreparedRange
        | SpecializedProfile::OriginalFlexParent
        | SpecializedProfile::OriginalPercentageFlexParent
        | SpecializedProfile::OriginalBlockAxes
        | SpecializedProfile::OriginalFlexAxes
        | SpecializedProfile::OriginalGridAxes
        | SpecializedProfile::OriginalSubgridAxes
        | SpecializedProfile::OriginalSubgridBody
        | SpecializedProfile::PreparedAtomicPercentage
        | SpecializedProfile::HorizontalBrFontBand
        | SpecializedProfile::VerticalBrFontBand
        | SpecializedProfile::EmptyLineBrFontBand => None,
        SpecializedProfile::FlexComposition => Some(Rc::new(flex::Inputs::load(root)?)),
    };
    terminal
        .cases
        .iter()
        .map(|entry| {
            let authority::QualifiedCase::Specialized(case) = entry else {
                return Err(changed("expected specialized case"));
            };
            let artifact = artifacts
                .get(case.case.name.as_str())
                .ok_or_else(|| changed("missing specialized output"))?;
            let historical = read_pinned(root, &case.historical)?;
            let xml = read_pinned(root, &case.case.output)?;
            let verified = identity_from_verified_inputs(IdentityInputs {
                metadata: &report.metadata,
                artifact,
                case: &case.case,
                full_inputs: &terminal.retained_inputs,
                excluded_specs: &BTreeSet::new(),
                actual_xml_sha256: &sha256(&xml),
                historical_sha256: Some(&sha256(&historical)),
                policy_sha256: &case.evidence.policy_digest(),
                browser_settings: &terminal.browser_settings,
                observation_request: profile.observation_request(),
                observation_input_sha256: terminal
                    .retained_inputs
                    .get("observation-request.json")
                    .map(String::as_str),
            })?;
            if verified.projection() != &case.identity {
                return Err(changed(
                    "stored specialized identity differs from authenticated inputs",
                ));
            }
            let xml = std::str::from_utf8(&xml).map_err(|error| malformed(error.to_string()))?;
            let variant = case.case.variant.clone();
            let (golden, comparison) = match profile {
                SpecializedProfile::PreparedRange => (
                    support::Golden::parse(xml),
                    AdmittedComparison::PreparedRange { variant },
                ),
                SpecializedProfile::OriginalFlexParent
                | SpecializedProfile::OriginalPercentageFlexParent => {
                    let current = support::Golden::parse(xml)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = std::str::from_utf8(&historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = support::Golden::parse(historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    original_flex_parent::verify_historical(&current, &historical)
                        .map_err(|error| changed(error.to_string()))?;
                    (Ok(current), AdmittedComparison::OriginalFlexParent)
                }
                SpecializedProfile::OriginalBlockAxes
                | SpecializedProfile::OriginalFlexAxes
                | SpecializedProfile::OriginalGridAxes
                | SpecializedProfile::OriginalSubgridAxes => {
                    let historical = std::str::from_utf8(&historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    qualification_catalogue::verify_historical_geometry(
                        xml,
                        historical,
                        &case.case.name,
                        &case.case.source,
                        &case.case.variant,
                    )
                    .map_err(|error| changed(error.to_string()))?;
                    let record = terminal
                        .records
                        .iter()
                        .find(|record| record.source == case.case.source)
                        .ok_or_else(|| changed("missing original axis record"))?;
                    let expected: qualification_catalogue::ExpectedDisposition =
                        serde_json::from_value(record.value["expected"].clone())
                            .map_err(|error| malformed(error.to_string()))?;
                    let qualification_catalogue::ExpectedDisposition::Generated { metadata } =
                        expected
                    else {
                        return Err(changed("original axis record requires generated geometry"));
                    };
                    qualification_catalogue::verify_absolute_metadata(
                        xml,
                        &metadata,
                        &case.case.name,
                        &variant,
                    )
                    .map_err(|error| changed(error.to_string()))?;
                    (
                        support::Golden::parse(xml),
                        AdmittedComparison::MatchingGeometry {
                            category: authority::MatchingCategory::Geometry,
                            source: case.case.source.clone(),
                        },
                    )
                }
                SpecializedProfile::OriginalSubgridBody => {
                    let current = support::Golden::parse(xml)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = std::str::from_utf8(&historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = support::Golden::parse(historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    original_subgrid_body::verify_historical(&current, &variant, &historical)?;
                    (
                        Ok(current),
                        AdmittedComparison::OriginalSubgridBody { variant },
                    )
                }
                SpecializedProfile::HorizontalBrFontBand
                | SpecializedProfile::VerticalBrFontBand
                | SpecializedProfile::EmptyLineBrFontBand => {
                    let current = support::Golden::parse(xml)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = std::str::from_utf8(&historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = support::Golden::parse(historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    support::br_font_band::verify_historical(&current, &historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    let projection = match profile {
                        SpecializedProfile::HorizontalBrFontBand => {
                            support::BrFontBandProfile::Horizontal
                        }
                        SpecializedProfile::VerticalBrFontBand => {
                            support::BrFontBandProfile::Vertical
                        }
                        SpecializedProfile::EmptyLineBrFontBand => {
                            support::BrFontBandProfile::EmptyLine
                        }
                        _ => unreachable!("selected BR font-band profile"),
                    };
                    (Ok(current), AdmittedComparison::BrFontBand(projection))
                }
                SpecializedProfile::PreparedAtomicPercentage => {
                    let source = read_pinned(
                        root,
                        &PathDigest {
                            path: artifact.source.clone(),
                            sha256: artifact.source_sha256.clone(),
                        },
                    )?;
                    let current = support::Golden::parse_with_prepared_atomic_percentage(
                        xml,
                        report_text,
                        &artifact.source,
                        &source,
                        &artifact.output,
                    )
                    .map_err(|error| malformed(error.to_string()))?;
                    let historical = std::str::from_utf8(&historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    let historical = support::Golden::parse(historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    prepared_atomic_percentage::verify_historical(&current, &historical)
                        .map_err(|error| malformed(error.to_string()))?;
                    (Ok(current), AdmittedComparison::PreparedAtomicPercentage)
                }
                SpecializedProfile::FlexComposition => {
                    let inputs = flex_inputs.as_ref().expect("Flex composition inputs");
                    let source = read_pinned(
                        root,
                        &PathDigest {
                            path: artifact.source.clone(),
                            sha256: artifact.source_sha256.clone(),
                        },
                    )?;
                    (
                        support::Golden::parse_with_published_flex_composition(
                            xml,
                            report_text,
                            &artifact.source,
                            &source,
                            &artifact.output,
                        ),
                        AdmittedComparison::FlexComposition {
                            variant,
                            inputs: Rc::clone(inputs),
                        },
                    )
                }
            };
            let golden = golden.map_err(|error| malformed(error.to_string()))?;
            if golden.name != case.case.name
                || (golden.has_prepared_range_frame()
                    != matches!(
                        profile,
                        SpecializedProfile::PreparedRange
                            | SpecializedProfile::PreparedAtomicPercentage
                    ))
            {
                return Err(changed("specialized observation frame"));
            }
            Ok(AdmittedCase {
                output: case.case.output.path.clone(),
                comparison,
                golden,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "portable_qualification_tests.rs"]
mod tests;

#[cfg(test)]
#[test]
fn original_inline_subgrid_alignment_capture_refuses_changed_source_authority() {
    let source = "html/subgrid/subgrid_alignment_002_center_start_orthogonal_item.html";
    let case = "subgrid_alignment_002_center_start_orthogonal_item__border_box_ltr";
    let original = include_str!(
        "xml/subgrid/subgrid_alignment_002_center_start_orthogonal_item__border_box_ltr.xml"
    );
    let current = include_str!(
        "captured/original-inline-subgrid-alignment/xml/subgrid/subgrid_alignment_002_center_start_orthogonal_item__border_box_ltr.xml"
    );
    let declaration = Declaration::parse(include_bytes!(
        "../../../examples/qualify-root-context/corpus.toml"
    ))
    .unwrap();
    let record = declaration
        .records
        .iter()
        .find(|record| record.source.as_str() == source)
        .unwrap();
    let verify =
        |xml: &str| record.verify_matching_capture(xml, Some(original), case, "border_box_ltr");
    verify(current).unwrap();
    support::Golden::parse(current).unwrap();

    for (field, changed) in [
        (
            "root-context=\"inline-grid\"",
            "root-context=\"block-flow-auto\"",
        ),
        ("position=\"static\"", "position=\"relative\""),
        ("display=\"inline-grid\"", "display=\"grid\""),
        (
            "writing-mode=\"vertical-lr\"",
            "writing-mode=\"horizontal-tb\"",
        ),
        ("grid-template-rows=\"50px\"", "grid-template-rows=\"51px\""),
    ] {
        assert!(current.contains(field));
        assert!(
            verify(&current.replacen(field, changed, 1)).is_err(),
            "{field} accepted as {changed}",
        );
    }
    for (field, changed) in [
        ("parent-direction=\"ltr\"", "parent-direction=\"rtl\""),
        (
            "parent-inline-extent=\"800px\"",
            "parent-inline-extent=\"50px\"",
        ),
        (
            "observation-frame=\"root-border\"",
            "observation-frame=\"root-border-local\"",
        ),
    ] {
        assert!(current.contains(field));
        assert!(
            support::Golden::parse(&current.replacen(field, changed, 1)).is_err(),
            "{field} accepted as {changed}",
        );
    }
}
