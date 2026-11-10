//! Admission and execution of an immutable, registered current corpus edition.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use serde::Deserialize;
use serde_json::Value;

use super::support::{
    self, original_subgrid_body, prepared_atomic_percentage,
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
use support::{ObservedPlacement, capture_authority};

#[path = "qualification_catalogue.rs"]
mod qualification_catalogue;
use qualification_catalogue::{Declaration, Record};

const OWNER_DECLARATION: &str = "examples/qualify-root-context/corpus.toml";

type Result<T> = std::result::Result<T, QualificationError>;
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
    pins: BTreeMap<String, String>,
    owner_pins: BTreeMap<String, String>,
    selected_records: BTreeMap<String, Value>,
}

impl Freshness {
    fn verify_files(&self) -> Result<()> {
        regular_file(
            &self.owner,
            &format!("{}/{}", self.artifact_root, authority::TERMINAL),
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

fn selected_records(root: &Path, terminal: &Qualification) -> Result<BTreeMap<String, Value>> {
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
    MatchingGeometry,
    Exact {
        edition: PublishedEdition,
        spec: Box<ExactDifferenceSpec>,
        identity: Box<VerifiedIdentity>,
    },
    PreparedRange {
        variant: String,
    },
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
            self.freshness.verify(),
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
                AdmittedComparison::MatchingGeometry => {
                    support::exact_differences::compare_matching_geometry(&case.golden)
                }
                AdmittedComparison::PreparedRange { variant } => {
                    compare_prepared_range(&case.golden, variant)
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
            .try_for_each(|edition| edition.freshness.verify_files())
            .and_then(|()| verify_live(&self.owner, &self.selected_records));
        Ok(EditionRun::finish(cases, declared_cases, freshness))
    }
}

// This seal admits only the publisher's complete terminal artifact. Retained
// projections and directory existence do not establish capture authority.
const REGISTERED_WRAPPING_WORLD_TERMINAL_SHA256: &str =
    "801abc2424a4bd97f5d0fa4d1518fbe9fe060fa74135849c57ad01429f11f1d0";
const REGISTERED_MEASURED_GRID_TERMINAL_SHA256: &str =
    "b49a365a7d2cae81852ebc3d61086119ac94b04b9363583fdbbd80b9837c3c3b";

const REGISTERED_FLEX_TEXT_TERMINAL_SHA256: &str =
    "dc30448e413b58865816a7e04c52acc8a2e518d04d27c6e9b26d45afa24ef43e";

const REGISTERED_PREPARED_RANGE_TERMINAL_SHA256: &str =
    "2604d4f621d508f4466c01a6d652152729bb7384e1a3981591d48e8add55e89e";

const REGISTERED_FLEX_COMPOSITION_TERMINAL_SHA256: &str =
    "a62a47ffdd66a267579819bf03208d0e13ae3b0268e75ac9f71a92d84cc54149";

const REGISTERED_GRID_STATIC_POSITION_TERMINAL_SHA256: &str =
    "996f40a0fb869a7af9b2a530f1ed8eb35b5882920e9220df381af20ce8dbd87a";

const REGISTERED_RELATIVE_INSET_TERMINAL_SHA256: &str =
    "ab5e855ac082238860e925b6b85fd92a99b2a6e49664adcc8daae1042e33c267";

const REGISTERED_ORIGINAL_SUBGRID_BODY_TERMINAL_SHA256: &str =
    "a566dc1542149d992ff31142c46966b772089efa69ff3a518b6380a495126397";
const REGISTERED_PREPARED_ATOMIC_PERCENTAGE_TERMINAL_SHA256: &str =
    "b76c52ed64f9b63201681259db84ad3d8f3e2ef3d2e47da87c9337a054c9eb9f";

const REGISTERED_HORIZONTAL_BR_FONT_BAND_TERMINAL_SHA256: &str =
    "3bfc85f72d277f94a3af430cfdced7d32d2accd57139bd0abd85dc0625c69394";

const REGISTERED_VERTICAL_BR_FONT_BAND_TERMINAL_SHA256: &str =
    "183019eea4508e1eefb999d7d66dbd257b1b10c0112ad0d07dbcd66a0d6ea731";

const REGISTERED_EMPTY_LINE_BR_FONT_BAND_TERMINAL_SHA256: &str =
    "0957cf7d3a81b133579619f1bf8832c13508d9cd9aa4e3d86df388f97f43350a";

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
const REGISTERED_MATCHING: &[(&str, &str)] = &[
    (
        "grid-order",
        "f527352997769a0c8373662f925d89104e0ddc7b2382e19958adabcf431d77e8",
    ),
    (
        "scriptless-order",
        "9b6957c7042f1732e57035d6e7ceb9bbb78cbe9de94189a8b8cd1313e882fa9a",
    ),
    (
        "box-controls",
        "eb95dfdcacc6352d990230e39bc133744f7a316dc918ae0a42eb5a2d74bbb69c",
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
    if authority::current_owner_bindings(&terminal.retained_inputs, &semantic_paths, sidecar)?
        != terminal.current_owner_inputs
    {
        return Err(changed("complete current-owner input census"));
    }
    let report_bytes = read_pinned(&root, &terminal.report)?;
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
    let ownership: Ownership = parse_json(&read_pinned(&root, &terminal.ownership)?)?;
    let mut expected_outputs =
        BTreeMap::from([(terminal.report.path.clone(), terminal.report.sha256.clone())]);
    let mut artifacts = BTreeMap::new();
    for artifact in &report.generated {
        if artifacts.insert(artifact.name.as_str(), artifact).is_some()
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
            let cases = admit_matching(&root, &terminal, &report, &artifacts)?;
            freshness.verify()?;
            return Ok(AdmittedEdition { freshness, cases });
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
            return Ok(AdmittedEdition { freshness, cases });
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
    Ok(AdmittedEdition { freshness, cases })
}

fn admit_matching(
    root: &Path,
    terminal: &Qualification,
    report: &Report,
    artifacts: &BTreeMap<&str, &CapturedArtifact>,
) -> Result<Vec<AdmittedCase>> {
    let declaration = Declaration::parse(&read_pinned(root, &terminal.declaration)?)
        .map_err(|error| malformed(error.to_string()))?;
    let records: BTreeMap<_, _> = declaration
        .records
        .iter()
        .filter(|r| r.matching_publication.as_deref() == Some(terminal.edition.as_str()))
        .map(|r| (r.source.as_str(), r))
        .collect();
    if records.len() != terminal.records.len()
        || records.len() * crate::fixture_variants::fixture_cases().len() != terminal.cases.len()
    {
        return Err(changed("complete matching edition source census"));
    }
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
        let historical = read_pinned(root, &case.historical)?;
        let historical =
            std::str::from_utf8(&historical).map_err(|error| malformed(error.to_string()))?;
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
                historical_sha256: Some(&case.historical.sha256),
                browser_settings: &terminal.browser_settings,
                observation_request: "geometry",
                observation_input_sha256: None,
            })?;
        if identity != case.identity {
            return Err(changed("matching capture binding projection"));
        }
        let golden = support::Golden::parse(xml).map_err(|error| changed(error.to_string()))?;
        // Validate supported complete geometry without accepting a mismatch.
        let (raw, qualified) = support::exact_differences::compare_matching_geometry(&golden)?;
        if !raw.is_empty() || qualified.is_err() {
            return Err(changed(format!(
                "matching edition contains raw geometry differences: {raw:?}"
            )));
        }
        cases.push(AdmittedCase {
            output: descriptor.output.path.clone(),
            comparison: AdmittedComparison::MatchingGeometry,
            golden,
        });
    }
    if consumed.len() != artifacts.len() {
        return Err(changed("unconsumed matching generated artifact"));
    }
    Ok(cases)
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
