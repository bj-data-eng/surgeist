//! Layout-private portable data and one identity calculation. This module uses
//! no Generator types. Callers authenticate bytes/censuses before calculation.
use super::exact_differences::{CaseIdentity, QualificationError, flex_composition_qualification};
use super::{
    flex_composition, flex_content_text, original_subgrid_body, pinned_font,
    prepared_range_qualification,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, value::RawValue};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};

use crate::fixture_variants::fixture_cases;

pub(crate) const EDITION: &str = "canonical-box-differences";
pub(crate) const MEASURED_GRID_EDITION: &str = "measured-grid-differences";
pub(crate) const ARTIFACT_ROOT: &str =
    "tests/layout/browser_parity/captured/canonical-box-differences";
const FLEX_TEXT_EDITION: &str = "flex-text-differences";
const GRID_STATIC_POSITION_EDITION: &str = "grid-static-position-differences";
const RELATIVE_INSET_EDITION: &str = "relative-inset-differences";
const WRAPPING_WORLD_CORNER_EDITION: &str = "wrapping-world-corner-differences";
pub(crate) const FIXED_GENERATOR: &str = "surgeist-layout-root-context-qualification";
pub(crate) const TERMINAL: &str = "qualification.json";
pub(crate) const TERMINAL_SCHEMA: u8 = 3;
pub(crate) const ROOT_HELPER_INPUT: &str = "scripts/gentest/test_helper.js";
const ROOT_HELPER_OWNER: &str = "tests/layout/browser_parity/scripts/gentest/test_helper.js";
const ROOT_HELPER: &[u8] = include_bytes!("scripts/gentest/test_helper.js");
const ROOT_CONTEXT_V4_HELPER_SHA256: &str =
    "af0f458c290f0261106810fa3e42442afe7a5599872774130ac3244548cb74ee";
const OBSERVATION_REQUEST: &str = "observation-request.json";
const MEASURED_LEAF_SCRIPT: &str = "scripts/gentest/measured_leaf.js";
const ELEMENT_SCROLL_SIZES_REQUEST_BYTES: &[u8] = include_bytes!(
    "../../bin/surgeist-layout-generate/measurement/element_scroll_sizes_request.json"
);
const MEASURED_LEAF_REQUEST_BYTES: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/measured_leaf_request.json");
const PREPARED_RANGE_REQUEST_BYTES: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/prepared_range_request.json");
pub(super) const FLEX_COMPOSITION_REQUEST_BYTES: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/flex_composition_request.json");
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
    #[serde(rename = "root-context-prepared-range-v1")]
    RootContextPreparedRangeV1,
    #[serde(rename = "root-context-flex-composition-v1")]
    RootContextFlexCompositionV1,
}

const PREPARED_RANGE_HELPER_SHA256: &str =
    "bd1b351e488f846b2e73947746ced8d8b290f49b108e220c228b2c4d94c069c0";
const FLEX_COMPOSITION_HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";
const CURRENT_ROOT_VALIDATOR: RootCaptureProfile = RootCaptureProfile::RootContextFlexCompositionV1;

pub(crate) fn root_helper_profile(digest: &str) -> Result<RootCaptureProfile> {
    match digest {
        ROOT_CONTEXT_V4_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextV4),
        PREPARED_RANGE_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextPreparedRangeV1),
        FLEX_COMPOSITION_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextFlexCompositionV1),
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
            let file = [
                ("measured_leaf_request.json", MEASURED_LEAF_REQUEST_BYTES),
                ("flex_content_text_request.json", flex_content_text::REQUEST),
                ("prepared_range_request.json", PREPARED_RANGE_REQUEST_BYTES),
                (
                    "original_subgrid_body_request.json",
                    original_subgrid_body::REQUEST,
                ),
                (
                    "flex_composition_request.json",
                    FLEX_COMPOSITION_REQUEST_BYTES,
                ),
            ]
            .into_iter()
            .find_map(|(file, bytes)| (hash == &sha256(bytes)).then_some(file))
            .ok_or_else(|| changed("unregistered observation request identity"))?;
            format!("tests/bin/surgeist-layout-generate/measurement/{file}")
        } else if name == flex_content_text::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/flex_content_text.js".to_owned()
        } else if name == original_subgrid_body::PROOF_SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_subgrid_body.js".to_owned()
        } else if name == MEASURED_LEAF_SCRIPT {
            "tests/bin/surgeist-layout-generate/measurement/measured_leaf.js".to_owned()
        } else if let Some(tail) = name.strip_prefix("historical/") {
            format!("{ORIGINAL}/xml/{tail}")
        } else if name.starts_with("scripts/")
            || name.starts_with("fonts/")
            || name.starts_with("html/")
        {
            format!("{ORIGINAL}/{name}")
        } else if name == prepared_range_qualification::RETAINED_POLICY {
            if hash != prepared_range_qualification::POLICY_SHA256 {
                return Err(changed("prepared Range policy section identity"));
            }
            prepared_range_qualification::POLICY_SOURCE.to_owned()
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
    if observation_request != SpecializedProfile::OriginalSubgridBody.observation_request()
        && full_inputs.contains_key(original_subgrid_body::PROOF_SCRIPT_PATH)
    {
        return Err(changed(
            "original subgrid body proof requires its observation request",
        ));
    }
    let (profile, font_demand) = match (observation_request, observation_input_sha256) {
        ("geometry", None)
            if !full_inputs.contains_key(OBSERVATION_REQUEST)
                && !full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                && !full_inputs.contains_key(flex_content_text::SCRIPT_PATH) =>
        {
            ("canonical-no-text-boxes", "none")
        }
        ("element-scroll-sizes", Some(request_sha256))
            if request_sha256 == sha256(ELEMENT_SCROLL_SIZES_REQUEST_BYTES)
                && full_inputs.get(OBSERVATION_REQUEST).map(String::as_str)
                    == Some(request_sha256)
                && !excluded_specs.contains(OBSERVATION_REQUEST)
                && !full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
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
        ("original-subgrid-body", Some(request_sha256)) => {
            let proof_sha256 = sha256(original_subgrid_body::PROOF_SCRIPT);
            if case.source != original_subgrid_body::SOURCE
                || artifact.source_sha256 != original_subgrid_body::SOURCE_SHA256
                || request_sha256 != sha256(original_subgrid_body::REQUEST)
                || policy_sha256 != proof_sha256
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
            {
                return Err(changed("original subgrid body capture profile"));
            }
            for (path, expected) in [
                (
                    original_subgrid_body::SOURCE,
                    original_subgrid_body::SOURCE_SHA256,
                ),
                (OBSERVATION_REQUEST, request_sha256),
                (
                    original_subgrid_body::PROOF_SCRIPT_PATH,
                    proof_sha256.as_str(),
                ),
                (ROOT_HELPER_INPUT, original_subgrid_body::HELPER_SHA256),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("original subgrid body input: {path}")));
                }
            }
            ("canonical-original-subgrid-body", "none")
        }
        ("prepared-ranges" | "flex-composition", Some(request_sha256))
            if !full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                && !full_inputs.contains_key(flex_content_text::SCRIPT_PATH) =>
        {
            let ranges =
                observation_request == SpecializedProfile::PreparedRange.observation_request();
            let (source, request, profile, font) = if ranges {
                (
                    prepared_range_qualification::SOURCE,
                    PREPARED_RANGE_REQUEST_BYTES,
                    "canonical-prepared-ranges",
                    "ahem-16px-prepared-ranges",
                )
            } else {
                (
                    flex_composition::SOURCE,
                    FLEX_COMPOSITION_REQUEST_BYTES,
                    "canonical-flex-composition",
                    "none",
                )
            };
            const FONT_PATH: &str = "fonts/Ahem.ttf";
            let resources_match = if ranges {
                artifact.linked_resources.len() == 1
                    && artifact.linked_resources.get(FONT_PATH).map(String::as_str)
                        == Some(pinned_font::SHA256)
            } else {
                artifact.linked_resources.is_empty()
            };
            let helper = full_inputs
                .get(ROOT_HELPER_INPUT)
                .and_then(|hash| root_helper_profile(hash).ok());
            let helper_matches = match helper {
                Some(RootCaptureProfile::RootContextFlexCompositionV1) => true,
                Some(RootCaptureProfile::RootContextPreparedRangeV1) => ranges,
                _ => false,
            };
            if case.source != source
                || request_sha256 != sha256(request)
                || !resources_match
                || !helper_matches
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed("specialized capture profile"));
            }
            for (path, digest) in [
                (OBSERVATION_REQUEST, request_sha256),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
                (FONT_PATH, pinned_font::SHA256),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(digest)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("specialized observation input: {path}")));
                }
            }
            (profile, font)
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
    WorldCornerHeight,
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
pub(crate) struct ExactCase {
    pub(crate) case: CaseDescriptor,
    pub(crate) spec: PathDigest,
    pub(crate) policy: PathDigest,
    #[serde(deserialize_with = "super::exact_differences::explicit_nullable")]
    pub(crate) historical: Option<PathDigest>,
    pub(crate) requirements: CurrentRequirements,
    pub(crate) identity: IdentityProjection,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SpecializedProfile {
    PreparedRange,
    FlexComposition,
    OriginalSubgridBody,
}

impl SpecializedProfile {
    pub(crate) fn edition(self) -> &'static str {
        self.artifact_root()
            .rsplit('/')
            .next()
            .expect("fixed edition path")
    }

    pub(crate) fn artifact_root(self) -> &'static str {
        match self {
            Self::PreparedRange => {
                "tests/layout/browser_parity/captured/prepared-range-qualification"
            }
            Self::FlexComposition => {
                "tests/layout/browser_parity/captured/flex-composition-qualification"
            }
            Self::OriginalSubgridBody => {
                "tests/layout/browser_parity/captured/original-subgrid-body-qualification"
            }
        }
    }

    pub(crate) fn observation_request(self) -> &'static str {
        match self {
            Self::PreparedRange => "prepared-ranges",
            Self::FlexComposition => "flex-composition",
            Self::OriginalSubgridBody => "original-subgrid-body",
        }
    }

    pub(crate) fn source(self) -> &'static str {
        match self {
            Self::PreparedRange => prepared_range_qualification::SOURCE,
            Self::FlexComposition => flex_composition::SOURCE,
            Self::OriginalSubgridBody => original_subgrid_body::SOURCE,
        }
    }

    fn from_edition(edition: &str) -> Option<Self> {
        [
            Self::PreparedRange,
            Self::FlexComposition,
            Self::OriginalSubgridBody,
        ]
        .into_iter()
        .find(|profile| profile.edition() == edition)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "profile", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum SpecializedEvidence {
    PreparedRange {
        policy: PathDigest,
    },
    FlexComposition {
        collapse_policy: PathDigest,
        scrollbar_policy: PathDigest,
        native_oracle: PathDigest,
        gutter_control: PathDigest,
    },
    OriginalSubgridBody {
        proof_script: PathDigest,
    },
}

impl SpecializedEvidence {
    fn profile(&self) -> SpecializedProfile {
        match self {
            Self::PreparedRange { .. } => SpecializedProfile::PreparedRange,
            Self::FlexComposition { .. } => SpecializedProfile::FlexComposition,
            Self::OriginalSubgridBody { .. } => SpecializedProfile::OriginalSubgridBody,
        }
    }

    pub(crate) fn policy_digest(&self) -> String {
        match self {
            Self::PreparedRange { policy } => policy.sha256.clone(),
            Self::OriginalSubgridBody { proof_script } => proof_script.sha256.clone(),
            Self::FlexComposition {
                collapse_policy,
                scrollbar_policy,
                ..
            } => {
                sha256(format!("{}{}", collapse_policy.sha256, scrollbar_policy.sha256).as_bytes())
            }
        }
    }

    pub(crate) fn bindings(&self) -> Vec<&PathDigest> {
        match self {
            Self::PreparedRange { policy } => vec![policy],
            Self::OriginalSubgridBody { proof_script } => vec![proof_script],
            Self::FlexComposition {
                collapse_policy,
                scrollbar_policy,
                native_oracle,
                gutter_control,
            } => vec![
                collapse_policy,
                scrollbar_policy,
                native_oracle,
                gutter_control,
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpecializedCase {
    pub(crate) case: CaseDescriptor,
    pub(crate) historical: PathDigest,
    pub(crate) evidence: SpecializedEvidence,
    pub(crate) identity: IdentityProjection,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum QualifiedCase {
    Exact(ExactCase),
    Specialized(SpecializedCase),
}

impl QualifiedCase {
    pub(crate) fn descriptor(&self) -> &CaseDescriptor {
        match self {
            Self::Exact(case) => &case.case,
            Self::Specialized(case) => &case.case,
        }
    }

    fn identity(&self) -> &IdentityProjection {
        match self {
            Self::Exact(case) => &case.identity,
            Self::Specialized(case) => &case.identity,
        }
    }
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
        let specialized = SpecializedProfile::from_edition(&self.edition);
        if self.schema_version != TERMINAL_SCHEMA
            || (specialized.is_none()
                && !matches!(
                    self.edition.as_str(),
                    EDITION
                        | MEASURED_GRID_EDITION
                        | FLEX_TEXT_EDITION
                        | GRID_STATIC_POSITION_EDITION
                        | RELATIVE_INSET_EDITION
                        | WRAPPING_WORLD_CORNER_EDITION
                ))
            || self.declaration.path != "corpus.toml"
            || self.report.path != "xml/generation-reports/all.json"
            || self.ownership.path != "xml/generation-ownership.json"
            || self.records.is_empty()
            || self.cases.is_empty()
        {
            return Err(malformed("invalid terminal edition/census"));
        }
        if specialized.is_some()
            && (self.records.len() != 1 || self.cases.len() != fixture_cases().len())
        {
            return Err(malformed("incomplete specialized terminal census"));
        }
        if self.capture_validation != CaptureValidation::from_retained(&self.retained_inputs)?
            || self.current_owner_inputs.get(ROOT_HELPER_OWNER)
                != Some(&self.capture_validation.validator_helper_sha256)
        {
            return Err(changed("terminal producer or current validation authority"));
        }
        for entry in [&self.declaration, &self.report, &self.ownership] {
            path(&entry.path)?;
            digest(&entry.sha256)?;
        }
        let mut records = BTreeMap::new();
        for record in &self.records {
            path(&record.source)?;
            digest(&record.sha256)?;
            if semantic_record_digest(&record.value)? != record.sha256
                || record.value.get("source").and_then(Value::as_str)
                    != Some(record.source.as_str())
                || records.insert(record.source.as_str(), record).is_some()
            {
                return Err(malformed("terminal semantic record projection changed"));
            }
        }
        if self.retained_inputs.get("corpus.toml") != Some(&self.declaration.sha256) {
            return Err(malformed("terminal declaration input seal changed"));
        }

        let mut names = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        let mut variants = BTreeSet::new();
        for case in &self.cases {
            let descriptor = case.descriptor();
            path(&descriptor.output.path)?;
            digest(&descriptor.output.sha256)?;
            digest(&descriptor.record_sha256)?;
            if !names.insert(&descriptor.name) || !outputs.insert(&descriptor.output.path) {
                return Err(malformed("duplicate terminal case/output"));
            }
            let Some(record) = records.get(descriptor.source.as_str()) else {
                return Err(malformed("terminal case source is missing"));
            };
            if record.sha256 != descriptor.record_sha256 {
                return Err(malformed("terminal case record association changed"));
            }
            match (case, specialized) {
                (QualifiedCase::Exact(case), None) => {
                    if case.requirements.case_id != descriptor.name
                        || case.requirements.output != descriptor.output.path
                    {
                        return Err(malformed("terminal requirement mapping changed"));
                    }
                    for entry in [&case.spec, &case.policy] {
                        self.require_retained(entry)?;
                    }
                    if let Some(historical) = &case.historical {
                        path(&historical.path)?;
                        digest(&historical.sha256)?;
                    }
                }
                (QualifiedCase::Specialized(case), Some(profile))
                    if variants.insert(descriptor.variant.as_str()) =>
                {
                    self.validate_specialized(case, profile, record)?;
                }
                _ => return Err(malformed("terminal edition or variant association changed")),
            }
            let identity = case.identity();
            if identity.source != descriptor.source
                || identity.variant != descriptor.variant
                || identity.capture_sha256 != descriptor.output.sha256
                || identity.record_sha256 != descriptor.record_sha256
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

    fn require_retained(&self, entry: &PathDigest) -> Result<()> {
        path(&entry.path)?;
        digest(&entry.sha256)?;
        if self.retained_inputs.get(&entry.path) != Some(&entry.sha256) {
            return Err(malformed("terminal evidence differs from retained input"));
        }
        Ok(())
    }

    fn validate_specialized(
        &self,
        case: &SpecializedCase,
        profile: SpecializedProfile,
        record: &RecordProjection,
    ) -> Result<()> {
        let source = profile.source();
        let family = source
            .strip_prefix("html/")
            .and_then(|path| path.strip_suffix(".html"))
            .expect("fixed original source path");
        let stem = family.rsplit('/').next().expect("fixed source name");
        let variant = &case.case.variant;
        if case.evidence.profile() != profile
            || case.case.source != source
            || record.value["specialized_publication"]
                != serde_json::to_value(profile).map_err(|error| malformed(error.to_string()))?
            || record.value["source_kind"] != "original"
            || record.value["source_sha256"] != case.identity.source_sha256
            || self.retained_inputs.get(source) != Some(&case.identity.source_sha256)
            || !fixture_cases().iter().any(|(key, _)| *key == variant)
            || case.case.name != format!("{stem}__{variant}")
            || case.case.output.path != format!("xml/{family}__{variant}.xml")
            || case.historical.path != format!("historical/{family}__{variant}.xml")
            || record.value["historical_sha256"][variant] != case.historical.sha256
            || case.identity.historical_sha256.as_ref() != Some(&case.historical.sha256)
        {
            return Err(malformed(
                "specialized terminal source or predecessor changed",
            ));
        }
        let evidence_matches = match &case.evidence {
            SpecializedEvidence::PreparedRange { policy } => {
                policy.path == prepared_range_qualification::RETAINED_POLICY
                    && policy.sha256 == prepared_range_qualification::POLICY_SHA256
            }
            SpecializedEvidence::FlexComposition { .. } => {
                flex_composition_qualification::evidence(variant).as_ref() == Some(&case.evidence)
            }
            SpecializedEvidence::OriginalSubgridBody { .. } => {
                original_subgrid_body::evidence() == case.evidence
            }
        };
        if !evidence_matches || case.identity.policy_source_sha256 != case.evidence.policy_digest()
        {
            return Err(malformed("specialized terminal policy identity changed"));
        }
        let mut evidence_paths = BTreeSet::from([case.historical.path.as_str()]);
        self.require_retained(&case.historical)?;
        for entry in case.evidence.bindings() {
            if !evidence_paths.insert(entry.path.as_str()) {
                return Err(malformed("duplicate specialized evidence path"));
            }
            self.require_retained(entry)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "capture_authority_tests.rs"]
mod tests;
