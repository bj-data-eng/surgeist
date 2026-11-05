//! Layout-private portable data and one identity calculation. This module uses
//! no Generator types. Callers authenticate bytes/censuses before calculation.
use super::exact_differences::{CaseIdentity, QualificationError};
use super::flex_content_text;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, value::RawValue};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const EDITION: &str = "canonical-box-differences";
pub(crate) const MEASURED_GRID_EDITION: &str = "measured-grid-differences";
pub(crate) const ARTIFACT_ROOT: &str =
    "tests/layout/browser_parity/captured/canonical-box-differences";
const FLEX_TEXT_EDITION: &str = "flex-text-differences";
pub(crate) const TERMINAL: &str = "qualification.json";
pub(crate) const TERMINAL_SCHEMA: u8 = 3;
pub(crate) const ROOT_HELPER_INPUT: &str = "scripts/gentest/test_helper.js";
const ROOT_HELPER_OWNER: &str = "tests/layout/browser_parity/scripts/gentest/test_helper.js";
const ROOT_HELPER: &[u8] = include_bytes!("scripts/gentest/test_helper.js");
const ROOT_CONTEXT_V4_HELPER_SHA256: &str =
    "af0f458c290f0261106810fa3e42442afe7a5599872774130ac3244548cb74ee";
const OBSERVATION_REQUEST: &str = "observation-request.json";
const MEASURED_LEAF_SCRIPT: &str = "scripts/gentest/measured_leaf.js";
const MEASURED_LEAF_REQUEST_BYTES: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/measured_leaf_request.json");
const MEASURED_LEAF_SCRIPT_BYTES: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/measured_leaf.js");

type Result<T> = std::result::Result<T, QualificationError>;
fn malformed(reason: impl Into<String>) -> QualificationError {
    QualificationError::MalformedDeclaration {
        reason: reason.into(),
    }
}
fn changed(component: impl Into<String>) -> QualificationError {
    QualificationError::IdentityChanged {
        component: component.into(),
    }
}
pub(crate) fn unique_json(raw: &str) -> Result<Value> {
    super::prepared_source::unique_json(raw).map_err(|e| malformed(e.to_string()))
}
pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum RootCaptureProfile {
    #[serde(rename = "root-context-v4")]
    RootContextV4,
}

const CURRENT_ROOT_VALIDATOR: RootCaptureProfile = RootCaptureProfile::RootContextV4;

pub(crate) fn root_helper_profile(digest: &str) -> Result<RootCaptureProfile> {
    match digest {
        ROOT_CONTEXT_V4_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextV4),
        _ => Err(changed("unsupported retained root proof helper")),
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureValidation {
    producer_profile: RootCaptureProfile,
    producer_helper_sha256: String,
    validator_profile: RootCaptureProfile,
    validator_helper_sha256: String,
}

impl CaptureValidation {
    pub(crate) fn from_retained(inputs: &BTreeMap<String, String>) -> Result<Self> {
        let producer = inputs
            .get(ROOT_HELPER_INPUT)
            .ok_or_else(|| changed("missing retained root proof helper"))?;
        let validator = sha256(ROOT_HELPER);
        if root_helper_profile(&validator)? != CURRENT_ROOT_VALIDATOR {
            return Err(changed(
                "canonical root helper does not match the current proof profile",
            ));
        }
        Ok(Self {
            producer_profile: root_helper_profile(producer)?,
            producer_helper_sha256: producer.clone(),
            validator_profile: CURRENT_ROOT_VALIDATOR,
            validator_helper_sha256: validator,
        })
    }
}

/// Existing sorted-Value record identity, excluding only self-referential spec
/// registrations. The complete projection remains retained in the terminal.
pub(crate) fn semantic_record_digest(record: &Value) -> Result<String> {
    let mut semantic = record.clone();
    semantic
        .as_object_mut()
        .ok_or_else(|| malformed("record projection must be an object"))?
        .remove("semantic_specifications");
    Ok(sha256(
        &serde_json::to_vec(&semantic).map_err(|e| malformed(e.to_string()))?,
    ))
}

/// Bind every retained input to its current handwritten owner. Origin witnesses
/// are retained values; the verified source receipt itself has one live owner.
pub(crate) fn current_owner_bindings(
    retained: &BTreeMap<String, String>,
    semantic_paths: &BTreeSet<&str>,
    receipt_sidecar: &str,
) -> Result<BTreeMap<String, String>> {
    const ORIGINAL: &str = "tests/layout/browser_parity";
    path(receipt_sidecar)?;
    let mut bindings = BTreeMap::new();
    for (name, hash) in retained {
        let mut owner_hash = hash.clone();
        let owner = if name == ROOT_HELPER_INPUT {
            root_helper_profile(hash)?;
            owner_hash = sha256(ROOT_HELPER);
            ROOT_HELPER_OWNER.to_owned()
        } else if name == "browser-owner.toml" {
            format!("{ORIGINAL}/corpus.toml")
        } else if name == OBSERVATION_REQUEST {
            let file = if hash == &sha256(flex_content_text::REQUEST) {
                "flex_content_text_request.json"
            } else {
                "measured_leaf_request.json"
            };
            format!("tests/bin/surgeist-layout-generate/measurement/{file}")
        } else if name == flex_content_text::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/flex_content_text.js".to_owned()
        } else if name == MEASURED_LEAF_SCRIPT {
            "tests/bin/surgeist-layout-generate/measurement/measured_leaf.js".to_owned()
        } else if let Some(tail) = name.strip_prefix("historical/") {
            format!("{ORIGINAL}/xml/{tail}")
        } else if name.starts_with("scripts/")
            || name.starts_with("fonts/")
            || name.starts_with("html/")
        {
            format!("{ORIGINAL}/{name}")
        } else if semantic_paths.contains(name.as_str()) {
            name.clone()
        } else if matches!(
            name.as_str(),
            "corpus.toml"
                | "origin/source-receipt.json"
                | "origin/source-provenance.json"
                | "origin/selected-originals.json"
        ) {
            continue;
        } else {
            return Err(malformed(format!(
                "unrepresented current-owner input role: {name}"
            )));
        };
        path(&owner)?;
        if bindings.insert(owner, owner_hash).is_some() {
            return Err(malformed("duplicate current-owner input role"));
        }
    }
    let receipt = retained
        .get("origin/source-receipt.json")
        .ok_or_else(|| malformed("missing retained source receipt"))?;
    if bindings
        .insert(format!("{ORIGINAL}/{receipt_sidecar}"), receipt.clone())
        .is_some()
    {
        return Err(malformed("source receipt overlaps another owner role"));
    }
    Ok(bindings)
}

fn digest(value: &str) -> Result<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(malformed("expected SHA-256"))
    }
}
fn path(value: &str) -> Result<()> {
    if value.is_empty()
        || value.contains(['\\', '\0'])
        || value
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        Err(malformed("expected owned relative path"))
    } else {
        Ok(())
    }
}

/// Lossless data witness from serde_json::to_vec(actual BrowserSettings).
/// RawValue preserves the official serializer's field order in the existing
/// profile tuple. This owns wire shape, not browser-setting validation/logic.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct BrowserSettingsWitness(Box<RawValue>);
impl BrowserSettingsWitness {
    pub(crate) fn from_official_json(bytes: &[u8]) -> Result<Self> {
        let raw = std::str::from_utf8(bytes).map_err(|e| malformed(e.to_string()))?;
        let value = unique_json(raw)?;
        let object = value
            .as_object()
            .ok_or_else(|| malformed("browser settings witness must be an object"))?;
        let keys: BTreeSet<_> = object.keys().map(String::as_str).collect();
        if keys
            != BTreeSet::from([
                "source",
                "version",
                "version_output",
                "cache_root",
                "provenance_format",
                "launch",
            ])
        {
            return Err(malformed("browser settings witness fields changed"));
        }
        for key in [
            "source",
            "version",
            "version_output",
            "cache_root",
            "provenance_format",
        ] {
            if !object[key].is_string() {
                return Err(malformed("browser settings witness string changed"));
            }
        }
        let launch = object["launch"]
            .as_object()
            .ok_or_else(|| malformed("browser launch witness must be an object"))?;
        if launch.keys().map(String::as_str).collect::<BTreeSet<_>>()
            != BTreeSet::from([
                "batch_size",
                "navigation_timeout_ms",
                "dom_poll_interval_ms",
                "arguments",
            ])
        {
            return Err(malformed("browser launch witness fields changed"));
        }
        for key in [
            "batch_size",
            "navigation_timeout_ms",
            "dom_poll_interval_ms",
        ] {
            if launch[key].as_u64().is_none() {
                return Err(malformed("browser launch witness integer changed"));
            }
        }
        if !launch["arguments"]
            .as_array()
            .is_some_and(|values| values.iter().all(Value::is_string))
        {
            return Err(malformed("browser launch argument witness changed"));
        }
        Ok(Self(
            RawValue::from_string(raw.to_owned()).map_err(|e| malformed(e.to_string()))?,
        ))
    }
}
impl<'de> Deserialize<'de> for BrowserSettingsWitness {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(d)?;
        Self::from_official_json(raw.get().as_bytes()).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureMetadata {
    pub(crate) schema_version: u8,
    pub(crate) generator: String,
    pub(crate) engine_version: String,
    #[serde(rename = "host_executable_sha256")]
    _host_executable_sha256: String,
    pub(crate) browser_executable: String,
    pub(crate) browser_executable_sha256: String,
    pub(crate) browser_provenance: String,
    #[serde(rename = "contract_sha256")]
    _contract_sha256: String,
    pub(crate) inputs: BTreeMap<String, String>,
    pub(crate) source_imports: Vec<Value>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CapturedArtifact {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) output: String,
    pub(crate) variant: String,
    pub(crate) source_sha256: String,
    pub(crate) linked_resources: BTreeMap<String, String>,
    #[serde(rename = "job_sha256")]
    _job_sha256: String,
    pub(crate) output_sha256: String,
}

/// Stored projection is data only. It intentionally has no conversion into
/// verified authority; recompute from report/mapping/actual byte digests instead.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityProjection {
    source: String,
    source_sha256: String,
    variant: String,
    #[serde(deserialize_with = "super::exact_differences::explicit_nullable")]
    historical_sha256: Option<String>,
    capture_sha256: String,
    record_sha256: String,
    generator: String,
    engine_version: String,
    browser_executable: String,
    browser_executable_sha256: String,
    browser_provenance: String,
    profile_sha256: String,
    source_imports_sha256: String,
    inputs: BTreeMap<String, String>,
    linked_resources: BTreeMap<String, String>,
    font_demand: String,
    policy_source_sha256: String,
}
pub(crate) struct VerifiedIdentity {
    identity: CaseIdentity,
    projection: IdentityProjection,
}
impl VerifiedIdentity {
    pub(crate) fn identity(&self) -> &CaseIdentity {
        &self.identity
    }
    pub(crate) fn projection(&self) -> &IdentityProjection {
        &self.projection
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PathDigest {
    pub(crate) path: String,
    pub(crate) sha256: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaseDescriptor {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) variant: String,
    pub(crate) output: PathDigest,
    pub(crate) record_sha256: String,
}
#[derive(Clone, Debug)]
pub(crate) struct CaseSelection {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) source_sha256: String,
    pub(crate) variant: String,
    pub(crate) output: String,
    pub(crate) record_sha256: String,
    pub(crate) historical_sha256: Option<String>,
}
pub(crate) struct IdentityInputs<'a> {
    pub(crate) metadata: &'a CaptureMetadata,
    pub(crate) artifact: &'a CapturedArtifact,
    pub(crate) case: &'a CaseDescriptor,
    pub(crate) full_inputs: &'a BTreeMap<String, String>,
    pub(crate) excluded_specs: &'a BTreeSet<String>,
    pub(crate) actual_xml_sha256: &'a str,
    pub(crate) historical_sha256: Option<&'a str>,
    pub(crate) policy_sha256: &'a str,
    pub(crate) browser_settings: &'a BrowserSettingsWitness,
    pub(crate) observation_request: &'a str,
    pub(crate) observation_input_sha256: Option<&'a str>,
}
/// Pure calculation shared by issuer and portable consumer. The caller first
/// checks actual bytes, full report/input census, selected mapping and owner
/// freshness. No value from a stored IdentityProjection enters this function.
pub(crate) fn identity_from_verified_inputs(input: IdentityInputs<'_>) -> Result<VerifiedIdentity> {
    let IdentityInputs {
        metadata,
        artifact,
        case,
        full_inputs,
        excluded_specs,
        actual_xml_sha256,
        historical_sha256,
        policy_sha256,
        browser_settings,
        observation_request,
        observation_input_sha256,
    } = input;
    for hash in [
        &metadata._host_executable_sha256,
        &metadata._contract_sha256,
        &artifact._job_sha256,
    ] {
        digest(hash)?;
    }
    if metadata.schema_version != 4 || &metadata.inputs != full_inputs {
        return Err(changed("capture metadata input census"));
    }
    if artifact.name != case.name
        || artifact.source != case.source
        || artifact.variant != case.variant
        || artifact.output != case.output.path
        || artifact.output_sha256 != case.output.sha256
        || artifact.output_sha256 != actual_xml_sha256
        || full_inputs.get(&case.source) != Some(&artifact.source_sha256)
    {
        return Err(changed("source/variant/capture mapping"));
    }
    let (profile, font_demand) = match (observation_request, observation_input_sha256) {
        ("geometry", None) | ("element-scroll-sizes", Some(_))
            if !full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                && !full_inputs.contains_key(flex_content_text::SCRIPT_PATH) =>
        {
            ("canonical-no-text-boxes", "none")
        }
        ("measured-leaf", Some(request_sha256))
            if !full_inputs.contains_key(flex_content_text::SCRIPT_PATH) =>
        {
            if request_sha256 != sha256(MEASURED_LEAF_REQUEST_BYTES) {
                return Err(changed("measured leaf request identity"));
            }
            let script_sha256 = sha256(MEASURED_LEAF_SCRIPT_BYTES);
            for (name, expected) in [
                (OBSERVATION_REQUEST, request_sha256),
                (MEASURED_LEAF_SCRIPT, script_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    super::pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
            ] {
                if full_inputs.get(name).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(name)
                {
                    return Err(changed(format!("measured leaf input: {name}")));
                }
            }
            ("canonical-measured-grid", "ahem-10px-zwsp")
        }
        ("flex-content-text", Some(request_sha256))
            if !full_inputs.contains_key(MEASURED_LEAF_SCRIPT) =>
        {
            if request_sha256 != sha256(flex_content_text::REQUEST) {
                return Err(changed("Flex content text request identity"));
            }
            if excluded_specs.contains(ROOT_HELPER_INPUT)
                || !full_inputs
                    .get(ROOT_HELPER_INPUT)
                    .is_some_and(|hash| root_helper_profile(hash).is_ok())
            {
                return Err(changed("Flex content text root proof helper"));
            }
            for (name, bytes) in flex_content_text::CAPTURE_INPUTS {
                if full_inputs.get(name) != Some(&sha256(bytes)) || excluded_specs.contains(name) {
                    return Err(changed(format!("Flex content text input: {name}")));
                }
            }
            ("canonical-flex-content-text", flex_content_text::PROFILE)
        }
        _ => return Err(malformed("unsupported observation request/input binding")),
    };
    if let Some(hash) = observation_input_sha256 {
        digest(hash)?;
    }
    let inputs = full_inputs
        .iter()
        .filter(|(path, _)| path.as_str() != "corpus.toml" && !excluded_specs.contains(*path))
        .map(|(path, digest)| (path.clone(), digest.clone()))
        .collect();
    let profile_sha256 = sha256(
        &serde_json::to_vec(&(
            profile,
            browser_settings,
            case.record_sha256.as_str(),
            case.variant.as_str(),
            observation_request,
            observation_input_sha256,
        ))
        .map_err(|e| malformed(e.to_string()))?,
    );
    let projection = IdentityProjection {
        source: case.source.clone(),
        source_sha256: artifact.source_sha256.clone(),
        variant: case.variant.clone(),
        historical_sha256: historical_sha256.map(str::to_owned),
        capture_sha256: actual_xml_sha256.to_owned(),
        record_sha256: case.record_sha256.clone(),
        generator: metadata.generator.clone(),
        engine_version: metadata.engine_version.clone(),
        browser_executable: metadata.browser_executable.clone(),
        browser_executable_sha256: metadata.browser_executable_sha256.clone(),
        browser_provenance: metadata.browser_provenance.clone(),
        profile_sha256,
        source_imports_sha256: sha256(
            &serde_json::to_vec(&metadata.source_imports).map_err(|e| malformed(e.to_string()))?,
        ),
        inputs,
        linked_resources: artifact.linked_resources.clone(),
        font_demand: font_demand.to_owned(),
        policy_source_sha256: policy_sha256.to_owned(),
    };
    let identity = CaseIdentity::from_verified_projection(
        serde_json::to_value(&projection).map_err(|e| malformed(e.to_string()))?,
    )?;
    Ok(VerifiedIdentity {
        identity,
        projection,
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CanonicalField {
    X,
    Y,
    Width,
    Height,
    ClientWidth,
    ClientHeight,
    ScrollWidth,
    ScrollHeight,
    LayoutScrollWidth,
    LayoutScrollHeight,
}
#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selector {
    pub(crate) path: String,
    pub(crate) field: CanonicalField,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CurrentFrame {
    RootBorderLocal,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CurrentDomain {
    Layout,
    LayoutAndElementSizes,
}

impl CurrentDomain {
    pub(crate) fn fields(self) -> &'static [CanonicalField] {
        const ALL: [CanonicalField; 8] = [
            CanonicalField::X,
            CanonicalField::Y,
            CanonicalField::Width,
            CanonicalField::Height,
            CanonicalField::ClientWidth,
            CanonicalField::ClientHeight,
            CanonicalField::ScrollWidth,
            CanonicalField::ScrollHeight,
        ];
        match self {
            Self::Layout => &ALL[..4],
            Self::LayoutAndElementSizes => &ALL,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CurrentRequirements {
    pub(crate) case_id: String,
    pub(crate) output: String,
    pub(crate) frame: CurrentFrame,
    pub(crate) domain: CurrentDomain,
    pub(crate) selectors: Vec<Selector>,
    pub(crate) nodes: Vec<String>,
    // Both channels are required, not just the phase selected by the browser.
    pub(crate) unrounded_packets: Vec<String>,
    pub(crate) final_packets: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordProjection {
    pub(crate) source: String,
    pub(crate) sha256: String,
    pub(crate) value: Value,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct QualifiedCase {
    pub(crate) case: CaseDescriptor,
    pub(crate) spec: PathDigest,
    pub(crate) policy: PathDigest,
    #[serde(deserialize_with = "super::exact_differences::explicit_nullable")]
    pub(crate) historical: Option<PathDigest>,
    pub(crate) requirements: CurrentRequirements,
    pub(crate) identity: IdentityProjection,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Qualification {
    pub(crate) schema_version: u8,
    pub(crate) edition: String,
    pub(crate) declaration: PathDigest,
    pub(crate) records: Vec<RecordProjection>,
    pub(crate) browser_settings: BrowserSettingsWitness,
    pub(crate) report: PathDigest,
    pub(crate) ownership: PathDigest,
    pub(crate) cases: Vec<QualifiedCase>,
    pub(crate) retained_inputs: BTreeMap<String, String>,
    pub(crate) current_owner_inputs: BTreeMap<String, String>,
    pub(crate) capture_validation: CaptureValidation,
}
impl Qualification {
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|e| malformed(e.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        unique_json(raw)?;
        let value: Self = serde_json::from_str(raw).map_err(|e| malformed(e.to_string()))?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        if self.schema_version != TERMINAL_SCHEMA
            || !matches!(
                self.edition.as_str(),
                EDITION | MEASURED_GRID_EDITION | FLEX_TEXT_EDITION
            )
            || self.declaration.path != "corpus.toml"
            || self.report.path != "xml/generation-reports/all.json"
            || self.ownership.path != "xml/generation-ownership.json"
            || self.records.is_empty()
            || self.cases.is_empty()
        {
            return Err(malformed("invalid terminal edition/census"));
        }
        if self.capture_validation != CaptureValidation::from_retained(&self.retained_inputs)?
            || self.current_owner_inputs.get(ROOT_HELPER_OWNER)
                != Some(&self.capture_validation.validator_helper_sha256)
        {
            return Err(changed("terminal producer or current validation authority"));
        }
        for entry in [&self.declaration, &self.report, &self.ownership]
            .into_iter()
            .chain(self.cases.iter().flat_map(|c| {
                [&c.case.output, &c.spec, &c.policy]
                    .into_iter()
                    .chain(c.historical.iter())
            }))
        {
            path(&entry.path)?;
            digest(&entry.sha256)?;
        }
        let mut cases = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        for case in &self.cases {
            if !cases.insert(&case.case.name) || !outputs.insert(&case.case.output.path) {
                return Err(malformed("duplicate terminal case/output"));
            }
            if case.requirements.case_id != case.case.name
                || case.requirements.output != case.case.output.path
            {
                return Err(malformed("terminal requirement mapping changed"));
            }
            digest(&case.case.record_sha256)?;
        }
        let mut records = BTreeMap::new();
        for record in &self.records {
            path(&record.source)?;
            digest(&record.sha256)?;
            if semantic_record_digest(&record.value)? != record.sha256
                || record.value.get("source").and_then(Value::as_str)
                    != Some(record.source.as_str())
                || records
                    .insert(record.source.as_str(), record.sha256.as_str())
                    .is_some()
            {
                return Err(malformed("terminal semantic record projection changed"));
            }
        }
        if self.retained_inputs.get("corpus.toml") != Some(&self.declaration.sha256) {
            return Err(malformed("terminal declaration input seal changed"));
        }
        for case in &self.cases {
            if records.get(case.case.source.as_str()).copied()
                != Some(case.case.record_sha256.as_str())
                || self.retained_inputs.get(&case.spec.path) != Some(&case.spec.sha256)
                || self.retained_inputs.get(&case.policy.path) != Some(&case.policy.sha256)
            {
                return Err(malformed("terminal record/spec/policy association changed"));
            }
            if case.identity.source != case.case.source
                || case.identity.variant != case.case.variant
                || case.identity.capture_sha256 != case.case.output.sha256
                || case.identity.record_sha256 != case.case.record_sha256
            {
                return Err(malformed("stored projection/case association changed"));
            }
        }
        for map in [&self.retained_inputs, &self.current_owner_inputs] {
            for (name, hash) in map {
                path(name)?;
                digest(hash)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "capture_authority_tests.rs"]
mod tests;
