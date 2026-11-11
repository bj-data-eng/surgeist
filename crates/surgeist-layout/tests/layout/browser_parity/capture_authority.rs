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
use crate::original_flex_parent_profile::{
    self as original_flex_parent, percentage as percentage_flex_parent,
};
use crate::prepared_atomic_percentage_profile as atomic_percentage;
use crate::{
    original_block_axes_profile as block_axes, original_flex_axes_profile as flex_axes,
    original_grid_axes_profile as grid_axes, original_subgrid_axes_profile as subgrid_axes,
};

pub(crate) const EDITION: &str = "canonical-box-differences";
pub(crate) const MEASURED_GRID_EDITION: &str = "measured-grid-differences";
pub(crate) const ARTIFACT_ROOT: &str =
    "tests/layout/browser_parity/captured/canonical-box-differences";
use crate::{
    empty_line_br_font_band_profile as empty_line_br, vertical_br_font_band_profile as vertical_br,
};
const FLEX_TEXT_EDITION: &str = "flex-text-differences";
const GRID_STATIC_POSITION_EDITION: &str = "grid-static-position-differences";
const RELATIVE_INSET_EDITION: &str = "relative-inset-differences";
const WRAPPING_WORLD_CORNER_EDITION: &str = "wrapping-world-corner-differences";
pub(crate) const FIXED_GENERATOR: &str = "surgeist-layout-root-context-qualification";
pub(crate) const TERMINAL: &str = "qualification.json";
pub(crate) const TERMINAL_SCHEMA: u8 = 3;
pub(crate) const ORIGINAL_DIAGNOSTIC_EDITION: &str = "grid-lanes-axis-original-diagnostics";
pub(crate) const ORIGINAL_DIAGNOSTIC_RECEIPT: &str = "diagnostics.json";
pub(crate) const GRID_LANES_PAIRED_EDITION: &str = "grid-lanes-axis-controls";
pub(crate) const FIXED_CAPTURE_SHARED_INPUTS: [&str; 7] = [
    "browser-owner.toml",
    "fonts/Ahem.ttf",
    "fonts/PROVENANCE.md",
    "origin/source-provenance.json",
    "origin/source-receipt.json",
    "scripts/gentest/test_base_style.css",
    ROOT_HELPER_INPUT,
];
pub(crate) const GRID_LANES_AXIS_ORIGINAL_SOURCES: [&str; 9] = [
    "html/grid-lanes/grid_lanes_axes_horizontal_tb_parallel.html",
    "html/grid-lanes/grid_lanes_axes_horizontal_parent_orthogonal_child.html",
    "html/grid-lanes/grid_lanes_axes_sideways_lr_parallel.html",
    "html/grid-lanes/grid_lanes_axes_sideways_opposing.html",
    "html/grid-lanes/grid_lanes_axes_sideways_rl_parallel.html",
    "html/grid-lanes/grid_lanes_axes_vertical_lr_parallel.html",
    "html/grid-lanes/grid_lanes_axes_vertical_opposing.html",
    "html/grid-lanes/grid_lanes_axes_vertical_parent_orthogonal_child.html",
    "html/grid-lanes/grid_lanes_axes_vertical_rl_parallel.html",
];

pub(crate) fn grid_lanes_control_source(original: &str) -> Option<String> {
    let source = GRID_LANES_AXIS_ORIGINAL_SOURCES
        .iter()
        .find(|source| **source == original)?;
    let stem = source
        .strip_prefix("html/grid-lanes/")?
        .strip_suffix(".html")?;
    Some(format!("html/qualification/{stem}_control.html"))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DiagnosticReference {
    pub(crate) edition: String,
    pub(crate) sha256: String,
}

impl DiagnosticReference {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.edition != ORIGINAL_DIAGNOSTIC_EDITION {
            return Err(malformed("wrong paired original diagnostic edition"));
        }
        digest(&self.sha256)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceEquivalentMatching {
    pub(crate) edition: String,
    pub(crate) original: String,
    pub(crate) diagnostics: DiagnosticReference,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum MatchingMembership {
    Independent(String),
    SourceEquivalent(SourceEquivalentMatching),
}

impl MatchingMembership {
    pub(crate) fn edition(&self) -> &str {
        match self {
            Self::Independent(edition) => edition,
            Self::SourceEquivalent(pair) => &pair.edition,
        }
    }

    pub(crate) fn validate(&self, source: &str, kind: SourceKind) -> Result<()> {
        validate_matching_edition(self.edition())?;
        match self {
            Self::Independent(edition) if edition != GRID_LANES_PAIRED_EDITION => Ok(()),
            Self::SourceEquivalent(pair)
                if pair.edition == GRID_LANES_PAIRED_EDITION
                    && kind == SourceKind::Authored
                    && grid_lanes_control_source(&pair.original).as_deref() == Some(source) =>
            {
                pair.diagnostics.validate()
            }
            _ => Err(malformed(
                "matching membership requires its declared source-equivalent association",
            )),
        }
    }
}
pub(crate) const ROOT_HELPER_INPUT: &str = "scripts/gentest/test_helper.js";
pub(crate) const ROOT_HELPER_OWNER: &str =
    "tests/layout/browser_parity/scripts/gentest/test_helper.js";
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
    #[serde(rename = "root-context-script-custody-v1")]
    RootContextScriptCustodyV1,
    #[serde(rename = "root-context-fixed-orthogonal-v1")]
    RootContextFixedOrthogonalV1,
    #[serde(rename = "root-context-computed-margin-v1")]
    RootContextComputedMarginV1,
}

const PREPARED_RANGE_HELPER_SHA256: &str =
    "bd1b351e488f846b2e73947746ced8d8b290f49b108e220c228b2c4d94c069c0";
use crate::capture_helper_identity::{
    COMPUTED_MARGIN_HELPER_SHA256, FIXED_ORTHOGONAL_HELPER_SHA256, SCRIPT_CUSTODY_HELPER_SHA256,
};
const CURRENT_ROOT_VALIDATOR: RootCaptureProfile = RootCaptureProfile::RootContextComputedMarginV1;

pub(crate) fn root_helper_profile(digest: &str) -> Result<RootCaptureProfile> {
    match digest {
        ROOT_CONTEXT_V4_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextV4),
        PREPARED_RANGE_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextPreparedRangeV1),
        flex_composition::RETAINED_HELPER_SHA256 => {
            Ok(RootCaptureProfile::RootContextFlexCompositionV1)
        }
        SCRIPT_CUSTODY_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextScriptCustodyV1),
        FIXED_ORTHOGONAL_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextFixedOrthogonalV1),
        COMPUTED_MARGIN_HELPER_SHA256 => Ok(RootCaptureProfile::RootContextComputedMarginV1),
        _ => Err(changed("unsupported retained root proof helper")),
    }
}

fn verify_grid_lanes_producing_helper(inputs: &BTreeMap<String, String>) -> Result<()> {
    if inputs.get(ROOT_HELPER_INPUT).map(String::as_str) != Some(FIXED_ORTHOGONAL_HELPER_SHA256) {
        return Err(changed("source-equivalent Grid Lanes producing helper"));
    }
    Ok(())
}

pub(crate) fn verify_grid_lanes_shared_inputs(
    original: &BTreeMap<String, String>,
    control: &BTreeMap<String, String>,
) -> Result<()> {
    for path in FIXED_CAPTURE_SHARED_INPUTS {
        if original
            .get(path)
            .is_none_or(|digest| control.get(path) != Some(digest))
        {
            return Err(changed(format!("Grid Lanes pair shared input: {path}")));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureValidation {
    producer_profile: RootCaptureProfile,
    producer_helper_sha256: String,
    pub(crate) validator_profile: RootCaptureProfile,
    pub(crate) validator_helper_sha256: String,
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

    fn validate_recorded(&self, retained: &BTreeMap<String, String>) -> Result<()> {
        if retained.get(ROOT_HELPER_INPUT) != Some(&self.producer_helper_sha256)
            || root_helper_profile(&self.producer_helper_sha256)? != self.producer_profile
            || root_helper_profile(&self.validator_helper_sha256)? != self.validator_profile
        {
            return Err(malformed("original diagnostic recorded capture validation"));
        }
        Ok(())
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
    sources: &BTreeMap<&str, SourceKind>,
    semantic_paths: &BTreeSet<&str>,
    receipt_sidecar: &str,
) -> Result<BTreeMap<String, String>> {
    const ORIGINAL: &str = "tests/layout/browser_parity";
    path(receipt_sidecar)?;
    if sources.keys().any(|source| !retained.contains_key(*source)) {
        return Err(malformed("declared source is missing from retained inputs"));
    }
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
                    "prepared_atomic_percentage_request.json",
                    atomic_percentage::REQUEST,
                ),
                (
                    "original_flex_parent_request.json",
                    original_flex_parent::REQUEST,
                ),
                (
                    "original_percentage_flex_parent_request.json",
                    percentage_flex_parent::REQUEST,
                ),
                ("original_block_axes_request.json", block_axes::REQUEST),
                ("original_flex_axes_request.json", flex_axes::REQUEST),
                ("original_grid_axes_request.json", grid_axes::REQUEST),
                ("original_subgrid_axes_request.json", subgrid_axes::REQUEST),
                (
                    "original_subgrid_body_request.json",
                    original_subgrid_body::REQUEST,
                ),
                (
                    "horizontal_br_font_band_request.json",
                    super::br_font_band::REQUEST,
                ),
                ("vertical_br_font_band_request.json", vertical_br::REQUEST),
                (
                    "empty_line_br_font_band_request.json",
                    empty_line_br::REQUEST,
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
        } else if name == original_flex_parent::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_flex_parent.js".to_owned()
        } else if name == percentage_flex_parent::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_percentage_flex_parent.js"
                .to_owned()
        } else if name == block_axes::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_block_axes.js".to_owned()
        } else if name == flex_axes::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_flex_axes.js".to_owned()
        } else if name == grid_axes::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_grid_axes.js".to_owned()
        } else if name == subgrid_axes::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_subgrid_axes.js".to_owned()
        } else if name == original_subgrid_body::PROOF_SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/original_subgrid_body.js".to_owned()
        } else if name == atomic_percentage::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/prepared_atomic_percentage.js"
                .to_owned()
        } else if name == super::br_font_band::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/horizontal_br_font_band.js".to_owned()
        } else if name == vertical_br::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/vertical_br_font_band.js".to_owned()
        } else if name == empty_line_br::SCRIPT_PATH {
            "tests/bin/surgeist-layout-generate/measurement/empty_line_br_font_band.js".to_owned()
        } else if name == MEASURED_LEAF_SCRIPT {
            "tests/bin/surgeist-layout-generate/measurement/measured_leaf.js".to_owned()
        } else if let Some(tail) = name.strip_prefix("historical/") {
            if retained.get(OBSERVATION_REQUEST) == Some(&sha256(atomic_percentage::REQUEST)) {
                format!("{ORIGINAL}/historical/prepared-text-v1/{tail}")
            } else {
                format!("{ORIGINAL}/xml/{tail}")
            }
        } else if name.starts_with("html/") {
            let kind = sources
                .get(name.as_str())
                .ok_or_else(|| malformed("retained HTML has no declared source owner"))?;
            kind.owner_path(name)
        } else if name.starts_with("scripts/") || name.starts_with("fonts/") {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policy_source_sha256: Option<String>,
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
/// Policy-free source, artifact, report, resource and browser-profile binding.
pub(crate) struct CaptureBindingInputs<'a> {
    pub(crate) metadata: &'a CaptureMetadata,
    pub(crate) artifact: &'a CapturedArtifact,
    pub(crate) case: &'a CaseDescriptor,
    pub(crate) full_inputs: &'a BTreeMap<String, String>,
    pub(crate) excluded_specs: &'a BTreeSet<String>,
    pub(crate) actual_xml_sha256: &'a str,
    pub(crate) historical_sha256: Option<&'a str>,
    pub(crate) browser_settings: &'a BrowserSettingsWitness,
    pub(crate) observation_request: &'a str,
    pub(crate) observation_input_sha256: Option<&'a str>,
}
pub(crate) fn capture_binding_from_verified_inputs(
    input: CaptureBindingInputs<'_>,
) -> Result<IdentityProjection> {
    let CaptureBindingInputs {
        metadata,
        artifact,
        case,
        full_inputs,
        excluded_specs,
        actual_xml_sha256,
        historical_sha256,
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
    if observation_request != SpecializedProfile::OriginalFlexParent.observation_request()
        && full_inputs.contains_key(original_flex_parent::SCRIPT_PATH)
    {
        return Err(changed(
            "original Flex parent proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::OriginalPercentageFlexParent.observation_request()
        && full_inputs.contains_key(percentage_flex_parent::SCRIPT_PATH)
    {
        return Err(changed(
            "original percentage Flex parent proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::OriginalBlockAxes.observation_request()
        && full_inputs.contains_key(block_axes::SCRIPT_PATH)
    {
        return Err(changed(
            "original Block axes proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::OriginalFlexAxes.observation_request()
        && full_inputs.contains_key(flex_axes::SCRIPT_PATH)
    {
        return Err(changed(
            "original Flex axes proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::OriginalGridAxes.observation_request()
        && full_inputs.contains_key(grid_axes::SCRIPT_PATH)
    {
        return Err(changed(
            "original Grid axes proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::OriginalSubgridAxes.observation_request()
        && full_inputs.contains_key(subgrid_axes::SCRIPT_PATH)
    {
        return Err(changed(
            "original Subgrid axes proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::OriginalSubgridBody.observation_request()
        && full_inputs.contains_key(original_subgrid_body::PROOF_SCRIPT_PATH)
    {
        return Err(changed(
            "original subgrid body proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::PreparedAtomicPercentage.observation_request()
        && full_inputs.contains_key(atomic_percentage::SCRIPT_PATH)
    {
        return Err(changed(
            "prepared atomic percentage proof requires its observation request",
        ));
    }
    use super::br_font_band as horizontal_br;
    if observation_request != SpecializedProfile::HorizontalBrFontBand.observation_request()
        && full_inputs.contains_key(horizontal_br::SCRIPT_PATH)
    {
        return Err(changed(
            "horizontal BR proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::VerticalBrFontBand.observation_request()
        && full_inputs.contains_key(vertical_br::SCRIPT_PATH)
    {
        return Err(changed(
            "vertical BR proof requires its observation request",
        ));
    }
    if observation_request != SpecializedProfile::EmptyLineBrFontBand.observation_request()
        && full_inputs.contains_key(empty_line_br::SCRIPT_PATH)
    {
        return Err(changed(
            "empty-line break proof requires its observation request",
        ));
    }
    let (profile, font_demand) = match (observation_request, observation_input_sha256) {
        (request, None)
            if request == MatchingCategory::Fri06Subgrid.observation_request()
                && fri06_subgrid_profile(&case.source).is_some()
                && !full_inputs.contains_key(OBSERVATION_REQUEST)
                && artifact.linked_resources.is_empty() =>
        {
            verify_grid_lanes_producing_helper(full_inputs)?;
            ("block-flow-atomic-subgrid", "ahem-10px-atomic-paragraphs")
        }
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
        ("original-flex-parent" | "original-percentage-flex-parent", Some(request_sha256)) => {
            let (source_digest, request, script_path, script, profile, font_demand) =
                if observation_request == "original-percentage-flex-parent" {
                    if full_inputs.contains_key("fonts/Ahem.ttf") {
                        return Err(changed(
                            "original percentage Flex parent has no font demand",
                        ));
                    }
                    (
                        (case.source == percentage_flex_parent::SOURCE)
                            .then_some(percentage_flex_parent::SOURCE_SHA256),
                        percentage_flex_parent::REQUEST,
                        percentage_flex_parent::SCRIPT_PATH,
                        percentage_flex_parent::SCRIPT,
                        "canonical-original-percentage-flex-parent",
                        "none",
                    )
                } else {
                    (
                        original_flex_parent::source_digest(&case.source),
                        original_flex_parent::REQUEST,
                        original_flex_parent::SCRIPT_PATH,
                        original_flex_parent::SCRIPT,
                        "canonical-original-flex-parent",
                        "ahem-10px-zwsp",
                    )
                };
            let proof_sha256 = sha256(script.as_bytes());
            if source_digest != Some(&artifact.source_sha256)
                || request_sha256 != sha256(request)
                || !artifact.linked_resources.is_empty()
                || !full_inputs
                    .get(ROOT_HELPER_INPUT)
                    .is_some_and(|digest| original_flex_parent::accepts_helper(digest))
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed("original Flex parent capture profile"));
            }
            for (path, expected) in [
                (case.source.as_str(), artifact.source_sha256.as_str()),
                (OBSERVATION_REQUEST, request_sha256),
                (script_path, proof_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("original Flex parent input: {path}")));
                }
            }
            (profile, font_demand)
        }
        (
            "original-block-axes"
            | "original-flex-axes"
            | "original-grid-axes"
            | "original-subgrid-axes",
            Some(request_sha256),
        ) => {
            let helper_digest = full_inputs.get(ROOT_HELPER_INPUT).map(String::as_str);
            let (profile, source_digest, request, script_path, script, helper_valid) =
                match observation_request {
                    "original-block-axes" => (
                        "canonical-original-block-axes",
                        block_axes::source_digest(&case.source),
                        block_axes::REQUEST,
                        block_axes::SCRIPT_PATH,
                        block_axes::SCRIPT,
                        helper_digest.is_some_and(block_axes::accepts_helper),
                    ),
                    "original-flex-axes" => (
                        "canonical-original-flex-axes",
                        flex_axes::source_digest(&case.source),
                        flex_axes::REQUEST,
                        flex_axes::SCRIPT_PATH,
                        flex_axes::SCRIPT,
                        helper_digest.is_some_and(flex_axes::accepts_helper),
                    ),
                    "original-grid-axes" => (
                        "canonical-original-grid-axes",
                        grid_axes::source_digest(&case.source),
                        grid_axes::REQUEST,
                        grid_axes::SCRIPT_PATH,
                        grid_axes::SCRIPT,
                        helper_digest.is_some_and(grid_axes::accepts_helper),
                    ),
                    "original-subgrid-axes" => (
                        "canonical-original-subgrid-axes",
                        subgrid_axes::source_digest(&case.source),
                        subgrid_axes::REQUEST,
                        subgrid_axes::SCRIPT_PATH,
                        subgrid_axes::SCRIPT,
                        helper_digest.is_some_and(subgrid_axes::accepts_helper),
                    ),
                    _ => unreachable!("only original axis capture profiles enter this arm"),
                };
            let proof_sha256 = sha256(script.as_bytes());
            if source_digest != Some(artifact.source_sha256.as_str())
                || request_sha256 != sha256(request)
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key("fonts/Ahem.ttf")
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
                || !helper_valid
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed("original axis capture profile"));
            }
            for (path, expected) in [
                (case.source.as_str(), artifact.source_sha256.as_str()),
                (OBSERVATION_REQUEST, request_sha256),
                (script_path, proof_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("original axis input: {path}")));
                }
            }
            (profile, "none")
        }
        ("original-subgrid-body", Some(request_sha256)) => {
            let proof_sha256 = sha256(original_subgrid_body::PROOF_SCRIPT);
            if case.source != original_subgrid_body::SOURCE
                || artifact.source_sha256 != original_subgrid_body::SOURCE_SHA256
                || request_sha256 != sha256(original_subgrid_body::REQUEST)
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
            {
                return Err(changed("original subgrid body capture profile"));
            }
            if !full_inputs
                .get(ROOT_HELPER_INPUT)
                .is_some_and(|digest| crate::original_subgrid_body_profile::accepts_helper(digest))
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed(
                    "original subgrid body input: scripts/gentest/test_helper.js",
                ));
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
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("original subgrid body input: {path}")));
                }
            }
            ("canonical-original-subgrid-body", "none")
        }
        ("prepared-atomic-percentage", Some(request_sha256)) => {
            let proof_sha256 = sha256(atomic_percentage::SCRIPT.as_bytes());
            if case.source != atomic_percentage::SOURCE
                || artifact.source_sha256 != atomic_percentage::SOURCE_SHA256
                || request_sha256 != sha256(atomic_percentage::REQUEST)
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
            {
                return Err(changed("prepared atomic percentage capture profile"));
            }
            if !full_inputs
                .get(ROOT_HELPER_INPUT)
                .is_some_and(|digest| atomic_percentage::accepts_helper(digest))
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed(
                    "prepared atomic percentage input: scripts/gentest/test_helper.js",
                ));
            }
            for (path, expected) in [
                (atomic_percentage::SOURCE, atomic_percentage::SOURCE_SHA256),
                (OBSERVATION_REQUEST, request_sha256),
                (atomic_percentage::SCRIPT_PATH, proof_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
                ("fonts/Ahem.ttf", pinned_font::SHA256),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("prepared atomic percentage input: {path}")));
                }
            }
            (
                "canonical-prepared-atomic-percentage",
                "ahem-16px-prepared-ranges",
            )
        }
        ("horizontal-br-font-band", Some(request_sha256)) => {
            let proof_sha256 = sha256(horizontal_br::SCRIPT.as_bytes());
            if case.source != horizontal_br::SOURCE
                || artifact.source_sha256 != horizontal_br::SOURCE_SHA256
                || request_sha256 != sha256(horizontal_br::REQUEST)
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
            {
                return Err(changed("horizontal BR font-band capture profile"));
            }
            if !full_inputs.get(ROOT_HELPER_INPUT).is_some_and(|digest| {
                crate::horizontal_br_font_band_profile::accepts_helper(digest)
            }) || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed(
                    "horizontal BR font-band input: scripts/gentest/test_helper.js",
                ));
            }
            for (path, expected) in [
                (horizontal_br::SOURCE, horizontal_br::SOURCE_SHA256),
                (OBSERVATION_REQUEST, request_sha256),
                (horizontal_br::SCRIPT_PATH, proof_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
                ("fonts/Ahem.ttf", pinned_font::SHA256),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("horizontal BR font-band input: {path}")));
                }
            }
            ("canonical-horizontal-br-font-band", "ahem-20px-prepared")
        }
        ("vertical-br-font-band", Some(request_sha256)) => {
            let proof_sha256 = sha256(vertical_br::SCRIPT.as_bytes());
            let source_sha256 = vertical_br::source_digest(&case.source)
                .ok_or_else(|| changed("vertical BR source identity"))?;
            if artifact.source_sha256 != source_sha256
                || request_sha256 != sha256(vertical_br::REQUEST)
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
            {
                return Err(changed("vertical BR font-band capture profile"));
            }
            if !full_inputs
                .get(ROOT_HELPER_INPUT)
                .is_some_and(|digest| vertical_br::accepts_helper(digest))
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed(
                    "vertical BR font-band input: scripts/gentest/test_helper.js",
                ));
            }
            for (path, expected) in [
                (case.source.as_str(), source_sha256),
                (OBSERVATION_REQUEST, request_sha256),
                (vertical_br::SCRIPT_PATH, proof_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
                ("fonts/Ahem.ttf", pinned_font::SHA256),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("vertical BR font-band input: {path}")));
                }
            }
            ("canonical-vertical-br-font-band", "ahem-20px-prepared")
        }
        ("empty-line-br-font-band", Some(request_sha256)) => {
            let proof_sha256 = sha256(empty_line_br::SCRIPT.as_bytes());
            let source_sha256 = empty_line_br::source_digest(&case.source)
                .ok_or_else(|| changed("empty-line BR source identity"))?;
            if artifact.source_sha256 != source_sha256
                || request_sha256 != sha256(empty_line_br::REQUEST)
                || !artifact.linked_resources.is_empty()
                || full_inputs.contains_key(MEASURED_LEAF_SCRIPT)
                || full_inputs.contains_key(flex_content_text::SCRIPT_PATH)
                || full_inputs.contains_key("fonts/Ahem.ttf")
                || sha256(pinned_font::AHEM_STYLESHEET_WOFF2)
                    != pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
            {
                return Err(changed("empty-line BR font-band capture profile"));
            }
            if !full_inputs
                .get(ROOT_HELPER_INPUT)
                .is_some_and(|digest| empty_line_br::accepts_helper(digest))
                || excluded_specs.contains(ROOT_HELPER_INPUT)
            {
                return Err(changed(
                    "empty-line BR font-band input: scripts/gentest/test_helper.js",
                ));
            }
            for (path, expected) in [
                (case.source.as_str(), source_sha256),
                (OBSERVATION_REQUEST, request_sha256),
                (empty_line_br::SCRIPT_PATH, proof_sha256.as_str()),
                (
                    "scripts/gentest/test_base_style.css",
                    pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256,
                ),
            ] {
                if full_inputs.get(path).map(String::as_str) != Some(expected)
                    || excluded_specs.contains(path)
                {
                    return Err(changed(format!("empty-line BR font-band input: {path}")));
                }
            }
            (
                "canonical-empty-line-br-font-band",
                "ahem-20px-stylesheet-woff2",
            )
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
            let helper_matches = full_inputs.get(ROOT_HELPER_INPUT).is_some_and(|digest| {
                flex_composition::accepts_helper(digest)
                    || ranges && digest == PREPARED_RANGE_HELPER_SHA256
            });
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
    for hash in [
        &artifact.source_sha256,
        actual_xml_sha256,
        &case.record_sha256,
        &metadata.browser_executable_sha256,
    ] {
        digest(hash)?;
    }
    if let Some(hash) = historical_sha256 {
        digest(hash)?;
    }
    for map in [full_inputs, &artifact.linked_resources] {
        for (name, hash) in map {
            path(name)?;
            digest(hash)?;
        }
    }
    for name in [
        &case.source,
        &case.output.path,
        &metadata.browser_executable,
    ] {
        path(name)?;
    }
    if case.name.is_empty() || metadata.generator.is_empty() || metadata.engine_version.is_empty() {
        return Err(malformed("empty capture binding identity"));
    }
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
        policy_source_sha256: None,
    };
    Ok(projection)
}
/// Exact and specialized policy identity is layered over the common capture binding.
pub(crate) fn identity_from_verified_inputs(input: IdentityInputs<'_>) -> Result<VerifiedIdentity> {
    let policy = input.policy_sha256;
    digest(policy)?;
    let required = match input.observation_request {
        "original-flex-parent" => Some(sha256(original_flex_parent::SCRIPT.as_bytes())),
        "original-percentage-flex-parent" => {
            Some(sha256(percentage_flex_parent::SCRIPT.as_bytes()))
        }
        "original-block-axes" => Some(sha256(block_axes::SCRIPT.as_bytes())),
        "original-flex-axes" => Some(sha256(flex_axes::SCRIPT.as_bytes())),
        "original-grid-axes" => Some(sha256(grid_axes::SCRIPT.as_bytes())),
        "original-subgrid-axes" => Some(sha256(subgrid_axes::SCRIPT.as_bytes())),
        "original-subgrid-body" => Some(sha256(original_subgrid_body::PROOF_SCRIPT)),
        "prepared-atomic-percentage" => Some(sha256(atomic_percentage::SCRIPT.as_bytes())),
        "horizontal-br-font-band" => Some(sha256(super::br_font_band::SCRIPT.as_bytes())),
        "vertical-br-font-band" => Some(sha256(vertical_br::SCRIPT.as_bytes())),
        "empty-line-br-font-band" => Some(sha256(empty_line_br::SCRIPT.as_bytes())),
        _ => None,
    };
    if required
        .as_deref()
        .is_some_and(|expected| expected != policy)
    {
        return Err(changed("specialized proof policy identity"));
    }
    let mut projection = capture_binding_from_verified_inputs(CaptureBindingInputs {
        metadata: input.metadata,
        artifact: input.artifact,
        case: input.case,
        full_inputs: input.full_inputs,
        excluded_specs: input.excluded_specs,
        actual_xml_sha256: input.actual_xml_sha256,
        historical_sha256: input.historical_sha256,
        browser_settings: input.browser_settings,
        observation_request: input.observation_request,
        observation_input_sha256: input.observation_input_sha256,
    })?;
    projection.policy_source_sha256 = Some(policy.to_owned());
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceKind {
    Original,
    Authored,
}

impl SourceKind {
    pub(crate) const fn owner_directory(self) -> &'static str {
        match self {
            Self::Original => "tests/layout/browser_parity",
            Self::Authored => "examples/qualify-root-context",
        }
    }

    fn owner_path(self, source: &str) -> String {
        format!("{}/{source}", self.owner_directory())
    }
}

impl RecordProjection {
    pub(crate) fn source_kind(&self) -> Result<SourceKind> {
        serde_json::from_value(self.value["source_kind"].clone())
            .map_err(|_| malformed("invalid retained source kind"))
    }
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
    OriginalFlexParent,
    OriginalPercentageFlexParent,
    OriginalBlockAxes,
    OriginalFlexAxes,
    OriginalGridAxes,
    OriginalSubgridAxes,
    OriginalSubgridBody,
    PreparedAtomicPercentage,
    HorizontalBrFontBand,
    VerticalBrFontBand,
    EmptyLineBrFontBand,
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
            Self::OriginalFlexParent => {
                "tests/layout/browser_parity/captured/original-flex-parent-qualification"
            }
            Self::OriginalPercentageFlexParent => {
                "tests/layout/browser_parity/captured/original-percentage-flex-parent-qualification"
            }
            Self::OriginalBlockAxes => {
                "tests/layout/browser_parity/captured/original-block-axes-qualification"
            }
            Self::OriginalFlexAxes => {
                "tests/layout/browser_parity/captured/original-flex-axes-qualification"
            }
            Self::OriginalGridAxes => {
                "tests/layout/browser_parity/captured/original-grid-axes-qualification"
            }
            Self::OriginalSubgridAxes => {
                "tests/layout/browser_parity/captured/original-subgrid-axes-qualification"
            }
            Self::OriginalSubgridBody => {
                "tests/layout/browser_parity/captured/original-subgrid-body-qualification"
            }
            Self::PreparedAtomicPercentage => {
                "tests/layout/browser_parity/captured/prepared-atomic-percentage-qualification"
            }
            Self::HorizontalBrFontBand => {
                "tests/layout/browser_parity/captured/horizontal-br-font-band-qualification"
            }
            Self::VerticalBrFontBand => {
                "tests/layout/browser_parity/captured/vertical-br-font-band-qualification"
            }
            Self::EmptyLineBrFontBand => {
                "tests/layout/browser_parity/captured/empty-line-br-font-band-qualification"
            }
        }
    }

    pub(crate) fn observation_request(self) -> &'static str {
        match self {
            Self::PreparedRange => "prepared-ranges",
            Self::FlexComposition => "flex-composition",
            Self::OriginalFlexParent => "original-flex-parent",
            Self::OriginalPercentageFlexParent => "original-percentage-flex-parent",
            Self::OriginalBlockAxes => "original-block-axes",
            Self::OriginalFlexAxes => "original-flex-axes",
            Self::OriginalGridAxes => "original-grid-axes",
            Self::OriginalSubgridAxes => "original-subgrid-axes",
            Self::OriginalSubgridBody => "original-subgrid-body",
            Self::PreparedAtomicPercentage => "prepared-atomic-percentage",
            Self::HorizontalBrFontBand => "horizontal-br-font-band",
            Self::VerticalBrFontBand => "vertical-br-font-band",
            Self::EmptyLineBrFontBand => "empty-line-br-font-band",
        }
    }

    pub(crate) fn sources(self) -> &'static [&'static str] {
        match self {
            Self::PreparedRange => &[prepared_range_qualification::SOURCE],
            Self::FlexComposition => &[flex_composition::SOURCE],
            Self::OriginalFlexParent => original_flex_parent::SOURCES,
            Self::OriginalPercentageFlexParent => &[percentage_flex_parent::SOURCE],
            Self::OriginalBlockAxes => block_axes::SOURCES,
            Self::OriginalFlexAxes => flex_axes::SOURCES,
            Self::OriginalGridAxes => grid_axes::SOURCES,
            Self::OriginalSubgridAxes => subgrid_axes::SOURCES,
            Self::OriginalSubgridBody => &[original_subgrid_body::SOURCE],
            Self::PreparedAtomicPercentage => &[atomic_percentage::SOURCE],
            Self::HorizontalBrFontBand => &[super::br_font_band::SOURCE],
            Self::VerticalBrFontBand => vertical_br::SOURCES,
            Self::EmptyLineBrFontBand => empty_line_br::SOURCES,
        }
    }

    pub(crate) fn from_edition(edition: &str) -> Option<Self> {
        [
            Self::PreparedRange,
            Self::FlexComposition,
            Self::OriginalFlexParent,
            Self::OriginalPercentageFlexParent,
            Self::OriginalBlockAxes,
            Self::OriginalFlexAxes,
            Self::OriginalGridAxes,
            Self::OriginalSubgridAxes,
            Self::OriginalSubgridBody,
            Self::PreparedAtomicPercentage,
            Self::HorizontalBrFontBand,
            Self::VerticalBrFontBand,
            Self::EmptyLineBrFontBand,
        ]
        .into_iter()
        .find(|profile| profile.edition() == edition)
    }
}

pub(crate) fn original_flex_parent_evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalFlexParent {
        proof_script: PathDigest {
            path: original_flex_parent::SCRIPT_PATH.to_owned(),
            sha256: sha256(original_flex_parent::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn original_percentage_flex_parent_evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalPercentageFlexParent {
        proof_script: PathDigest {
            path: percentage_flex_parent::SCRIPT_PATH.to_owned(),
            sha256: sha256(percentage_flex_parent::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn original_block_axes_evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalBlockAxes {
        proof_script: PathDigest {
            path: block_axes::SCRIPT_PATH.to_owned(),
            sha256: sha256(block_axes::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn original_flex_axes_evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalFlexAxes {
        proof_script: PathDigest {
            path: flex_axes::SCRIPT_PATH.to_owned(),
            sha256: sha256(flex_axes::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn original_grid_axes_evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalGridAxes {
        proof_script: PathDigest {
            path: grid_axes::SCRIPT_PATH.to_owned(),
            sha256: sha256(grid_axes::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn original_subgrid_axes_evidence() -> SpecializedEvidence {
    SpecializedEvidence::OriginalSubgridAxes {
        proof_script: PathDigest {
            path: subgrid_axes::SCRIPT_PATH.to_owned(),
            sha256: sha256(subgrid_axes::SCRIPT.as_bytes()),
        },
    }
}

pub(crate) fn prepared_atomic_percentage_evidence() -> SpecializedEvidence {
    SpecializedEvidence::PreparedAtomicPercentage {
        proof_script: PathDigest {
            path: atomic_percentage::SCRIPT_PATH.to_owned(),
            sha256: sha256(atomic_percentage::SCRIPT.as_bytes()),
        },
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
    OriginalFlexParent {
        proof_script: PathDigest,
    },
    OriginalPercentageFlexParent {
        proof_script: PathDigest,
    },
    OriginalBlockAxes {
        proof_script: PathDigest,
    },
    OriginalFlexAxes {
        proof_script: PathDigest,
    },
    OriginalGridAxes {
        proof_script: PathDigest,
    },
    OriginalSubgridAxes {
        proof_script: PathDigest,
    },
    OriginalSubgridBody {
        proof_script: PathDigest,
    },
    PreparedAtomicPercentage {
        proof_script: PathDigest,
    },
    HorizontalBrFontBand {
        proof_script: PathDigest,
    },
    VerticalBrFontBand {
        proof_script: PathDigest,
    },
    EmptyLineBrFontBand {
        proof_script: PathDigest,
    },
}

impl SpecializedEvidence {
    fn profile(&self) -> SpecializedProfile {
        match self {
            Self::PreparedRange { .. } => SpecializedProfile::PreparedRange,
            Self::FlexComposition { .. } => SpecializedProfile::FlexComposition,
            Self::OriginalFlexParent { .. } => SpecializedProfile::OriginalFlexParent,
            Self::OriginalPercentageFlexParent { .. } => {
                SpecializedProfile::OriginalPercentageFlexParent
            }
            Self::OriginalBlockAxes { .. } => SpecializedProfile::OriginalBlockAxes,
            Self::OriginalFlexAxes { .. } => SpecializedProfile::OriginalFlexAxes,
            Self::OriginalGridAxes { .. } => SpecializedProfile::OriginalGridAxes,
            Self::OriginalSubgridAxes { .. } => SpecializedProfile::OriginalSubgridAxes,
            Self::OriginalSubgridBody { .. } => SpecializedProfile::OriginalSubgridBody,
            Self::PreparedAtomicPercentage { .. } => SpecializedProfile::PreparedAtomicPercentage,
            Self::HorizontalBrFontBand { .. } => SpecializedProfile::HorizontalBrFontBand,
            Self::VerticalBrFontBand { .. } => SpecializedProfile::VerticalBrFontBand,
            Self::EmptyLineBrFontBand { .. } => SpecializedProfile::EmptyLineBrFontBand,
        }
    }

    pub(crate) fn policy_digest(&self) -> String {
        match self {
            Self::PreparedRange { policy } => policy.sha256.clone(),
            Self::OriginalFlexParent { proof_script }
            | Self::OriginalPercentageFlexParent { proof_script }
            | Self::OriginalBlockAxes { proof_script }
            | Self::OriginalFlexAxes { proof_script }
            | Self::OriginalGridAxes { proof_script }
            | Self::OriginalSubgridAxes { proof_script }
            | Self::OriginalSubgridBody { proof_script }
            | Self::PreparedAtomicPercentage { proof_script }
            | Self::HorizontalBrFontBand { proof_script }
            | Self::VerticalBrFontBand { proof_script }
            | Self::EmptyLineBrFontBand { proof_script } => proof_script.sha256.clone(),
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
            Self::OriginalFlexParent { proof_script }
            | Self::OriginalPercentageFlexParent { proof_script }
            | Self::OriginalBlockAxes { proof_script }
            | Self::OriginalFlexAxes { proof_script }
            | Self::OriginalGridAxes { proof_script }
            | Self::OriginalSubgridAxes { proof_script }
            | Self::OriginalSubgridBody { proof_script }
            | Self::PreparedAtomicPercentage { proof_script }
            | Self::HorizontalBrFontBand { proof_script }
            | Self::VerticalBrFontBand { proof_script }
            | Self::EmptyLineBrFontBand { proof_script } => vec![proof_script],
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

pub(crate) fn matching_metadata_is_valid(
    category: MatchingCategory,
    source: &str,
    kind: SourceKind,
    metadata: &Value,
) -> bool {
    match category {
        MatchingCategory::Geometry => {
            metadata["context"] == "absolute-local" && metadata["root_size"].is_null()
        }
        MatchingCategory::Fri06Subgrid => {
            let Some((mode, owners)) = fri06_subgrid_profile(source) else {
                return false;
            };
            kind == SourceKind::Original
                && metadata["context"] == "block-flow"
                && metadata["parent_direction"] == "ltr"
                && metadata["root_direction"] == "variant"
                && metadata["root_writing_mode"] == mode
                && metadata["root_position"] == "relative"
                && metadata["atomic_paragraphs"] == 6
                && metadata["grid_reference_owners"]
                    .as_array()
                    .is_some_and(|actual| {
                        actual
                            .iter()
                            .map(Value::as_str)
                            .eq(owners.iter().copied().map(Some))
                    })
        }
    }
}

pub(crate) const FRI06_SUBGRID_EDITION: &str = "fri06-subgrid-representatives";
pub(crate) const FRI06_SUBGRID_SOURCES: [&str; 3] = [
    "html/subgrid/subgrid_baseline_inline_column_inner_col1_first.html",
    "html/subgrid/subgrid_baseline_vertical_auto_rows_inner_row1_first.html",
    "html/subgrid/subgrid_baseline_vertical_nested_inner_row1_first.html",
];

pub(crate) fn fri06_subgrid_profile(
    source: &str,
) -> Option<(&'static str, &'static [&'static str])> {
    match FRI06_SUBGRID_SOURCES
        .iter()
        .position(|&known| known == source)?
    {
        0 => Some(("horizontal-tb", &[])),
        _ => Some(("vertical-rl", &["/", "/0", "/0/0"])),
    }
}

pub(crate) fn validate_matching_edition(edition: &str) -> Result<()> {
    if edition.is_empty()
        || edition.starts_with('-')
        || edition.ends_with('-')
        || !edition
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || SpecializedProfile::from_edition(edition).is_some()
        || matches!(
            edition,
            "root-context-v4"
                | ORIGINAL_DIAGNOSTIC_EDITION
                | EDITION
                | MEASURED_GRID_EDITION
                | FLEX_TEXT_EDITION
                | GRID_STATIC_POSITION_EDITION
                | RELATIVE_INSET_EDITION
                | WRAPPING_WORLD_CORNER_EDITION
        )
    {
        return Err(malformed("invalid matching edition identifier"));
    }
    Ok(())
}

#[derive(Clone, Copy, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CaseNaming {
    #[default]
    Basename,
    SourcePath,
}

impl CaseNaming {
    pub(crate) fn source_names(self, source: &str) -> Result<(&str, &str)> {
        let family = source
            .strip_prefix("html/")
            .and_then(|path| path.strip_suffix(".html"))
            .ok_or_else(|| malformed("qualification source must be an html/*.html path"))?;
        let basename = family
            .rsplit('/')
            .next()
            .filter(|stem| !stem.is_empty())
            .ok_or_else(|| malformed("missing source stem"))?;
        let stem = match self {
            Self::Basename => basename,
            Self::SourcePath => family,
        };
        Ok((family, stem))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MatchingCategory {
    Geometry,
    Fri06Subgrid,
}

impl MatchingCategory {
    pub(crate) fn for_edition(edition: &str) -> Self {
        if edition == FRI06_SUBGRID_EDITION {
            Self::Fri06Subgrid
        } else {
            Self::Geometry
        }
    }

    pub(crate) const fn observation_request(self) -> &'static str {
        match self {
            Self::Geometry => "geometry",
            Self::Fri06Subgrid => "fri06-subgrid-representatives",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MatchingProvenance {
    Original { historical: PathDigest },
    Authored,
}

impl MatchingProvenance {
    fn bind(kind: SourceKind, historical: Option<PathDigest>) -> Result<Self> {
        match (kind, historical) {
            (SourceKind::Original, Some(historical)) => Ok(Self::Original { historical }),
            (SourceKind::Authored, None) => Ok(Self::Authored),
            _ => Err(malformed(
                "matching history disagrees with the declared source kind",
            )),
        }
    }

    pub(crate) const fn historical(&self) -> Option<&PathDigest> {
        match self {
            Self::Original { historical } => Some(historical),
            Self::Authored => None,
        }
    }

    fn serialize_history<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        self.historical().serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct MatchingCase {
    pub(crate) case: CaseDescriptor,
    #[serde(
        rename = "historical",
        serialize_with = "MatchingProvenance::serialize_history"
    )]
    provenance: MatchingProvenance,
    matching: MatchingCategory,
    identity: IdentityProjection,
}

impl MatchingCase {
    pub(crate) fn new(
        case: CaseDescriptor,
        provenance: MatchingProvenance,
        matching: MatchingCategory,
        identity: IdentityProjection,
    ) -> Result<Self> {
        if identity.historical_sha256.as_deref()
            != provenance.historical().map(|entry| entry.sha256.as_str())
        {
            return Err(malformed(
                "matching predecessor identity disagrees with source provenance",
            ));
        }
        Ok(Self {
            case,
            provenance,
            matching,
            identity,
        })
    }

    pub(crate) const fn category(&self) -> MatchingCategory {
        self.matching
    }

    pub(crate) const fn provenance(&self) -> &MatchingProvenance {
        &self.provenance
    }

    pub(crate) const fn identity(&self) -> &IdentityProjection {
        &self.identity
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchingCaseWire {
    case: CaseDescriptor,
    #[serde(deserialize_with = "super::exact_differences::explicit_nullable")]
    historical: Option<PathDigest>,
    matching: MatchingCategory,
    identity: IdentityProjection,
}

impl MatchingCaseWire {
    fn bind(self, kind: SourceKind) -> Result<MatchingCase> {
        MatchingCase::new(
            self.case,
            MatchingProvenance::bind(kind, self.historical)?,
            self.matching,
            self.identity,
        )
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum QualifiedCaseWire {
    Exact(ExactCase),
    Specialized(SpecializedCase),
    Matching(MatchingCaseWire),
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum QualifiedCase {
    Exact(ExactCase),
    Specialized(SpecializedCase),
    Matching(MatchingCase),
}

impl QualifiedCase {
    pub(crate) fn descriptor(&self) -> &CaseDescriptor {
        match self {
            Self::Exact(case) => &case.case,
            Self::Specialized(case) => &case.case,
            Self::Matching(case) => &case.case,
        }
    }

    fn identity(&self) -> &IdentityProjection {
        match self {
            Self::Exact(case) => &case.identity,
            Self::Specialized(case) => &case.identity,
            Self::Matching(case) => case.identity(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct QualificationData<Case> {
    pub(crate) schema_version: u8,
    pub(crate) edition: String,
    pub(crate) declaration: PathDigest,
    pub(crate) records: Vec<RecordProjection>,
    pub(crate) browser_settings: BrowserSettingsWitness,
    pub(crate) report: PathDigest,
    pub(crate) ownership: PathDigest,
    pub(crate) cases: Vec<Case>,
    pub(crate) retained_inputs: BTreeMap<String, String>,
    pub(crate) current_owner_inputs: BTreeMap<String, String>,
    pub(crate) capture_validation: CaptureValidation,
}
pub(crate) type Qualification = QualificationData<QualifiedCase>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalDiagnosticDifference {
    path: String,
    field: super::NumericComparisonField,
    browser: super::Scalar,
    native: super::Scalar,
}

pub(crate) fn original_diagnostic_differences(
    golden: &super::Golden,
) -> Result<Vec<OriginalDiagnosticDifference>> {
    fn plain_boxes(node: &super::Node, expected: &super::Expectation) -> bool {
        node.kind == super::NodeKind::Div
            && node.text.is_none()
            && [expected.x, expected.y, expected.width, expected.height]
                .into_iter()
                .all(|value| value.is_some_and(|value| value.is_finite()))
            && node.children.len() == expected.children.len()
            && node
                .children
                .iter()
                .zip(&expected.children)
                .all(|(child, expected)| plain_boxes(child, expected))
    }
    if golden.element_scroll_sizes.is_some()
        || golden.measured_leaf.is_some()
        || golden.has_prepared_range_frame()
        || golden.has_flex_content_text()
        || golden.root.children.len() != 2
        || golden.root.children.iter().any(|container| {
            container.children.len() != 2
                || container
                    .children
                    .iter()
                    .any(|leaf| !leaf.children.is_empty())
        })
        || !plain_boxes(&golden.root, &golden.expectations)
    {
        return Err(malformed(
            "original diagnostics require complete finite plain-box geometry",
        ));
    }
    super::collect_surgeist_numeric_mismatches(golden)
        .map_err(|error| changed(error.to_string()))?
        .into_iter()
        .map(|mismatch| match mismatch {
            super::NumericMismatch::Layout(value) => Ok(OriginalDiagnosticDifference {
                path: value.path().to_owned(),
                field: value.field(),
                browser: value.browser_value(),
                native: value.native_value(),
            }),
            super::NumericMismatch::ElementScrollSize(_) => {
                Err(malformed("unexpected original diagnostic observation"))
            }
        })
        .collect()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalDiagnosticCase {
    pub(crate) case: CaseDescriptor,
    pub(crate) historical: PathDigest,
    pub(crate) identity: IdentityProjection,
    pub(crate) raw_differences: Vec<OriginalDiagnosticDifference>,
}

pub(crate) type OriginalDiagnosticReceipt = QualificationData<OriginalDiagnosticCase>;

impl OriginalDiagnosticReceipt {
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        unique_json(raw)?;
        let receipt: Self =
            serde_json::from_str(raw).map_err(|error| malformed(error.to_string()))?;
        receipt.validate_original_diagnostics()?;
        Ok(receipt)
    }

    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
        self.validate_original_diagnostics()?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|e| malformed(e.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    fn validate_original_diagnostics(&self) -> Result<()> {
        let sources: BTreeSet<_> = GRID_LANES_AXIS_ORIGINAL_SOURCES.into_iter().collect();
        let variants = fixture_cases();
        if self.schema_version != 1
            || self.edition != ORIGINAL_DIAGNOSTIC_EDITION
            || self.declaration.path != "corpus.toml"
            || self.report.path != "xml/generation-reports/all.json"
            || self.ownership.path != "xml/generation-ownership.json"
            || self.retained_inputs.get(&self.declaration.path) != Some(&self.declaration.sha256)
            || self.current_owner_inputs.get(ROOT_HELPER_OWNER)
                != Some(&self.capture_validation.validator_helper_sha256)
            || self.records.len() != sources.len()
            || self.cases.len() != sources.len() * variants.len()
        {
            return Err(malformed("original diagnostic receipt identity or census"));
        }
        self.capture_validation
            .validate_recorded(&self.retained_inputs)?;
        verify_grid_lanes_producing_helper(&self.retained_inputs)?;
        for binding in [&self.declaration, &self.report, &self.ownership] {
            digest(&binding.sha256)?;
        }
        let mut records = BTreeMap::new();
        for record in &self.records {
            if !sources.contains(record.source.as_str())
                || record.source_kind()? != SourceKind::Original
                || record.value["source"].as_str() != Some(record.source.as_str())
                || semantic_record_digest(&record.value)? != record.sha256
                || records.insert(record.source.as_str(), record).is_some()
            {
                return Err(malformed("original diagnostic source projection"));
            }
        }
        let mut seen = BTreeSet::new();
        let mut historical_paths = BTreeSet::new();
        for diagnostic in &self.cases {
            let case = &diagnostic.case;
            let record = records
                .get(case.source.as_str())
                .ok_or_else(|| malformed("original diagnostic case source"))?;
            let family = case
                .source
                .strip_prefix("html/")
                .and_then(|s| s.strip_suffix(".html"))
                .ok_or_else(|| malformed("original diagnostic source path"))?;
            let stem = family
                .rsplit('/')
                .next()
                .ok_or_else(|| malformed("original diagnostic stem"))?;
            let variant = case.variant.as_str();
            let identity = &diagnostic.identity;
            if !variants.iter().any(|(known, _)| *known == variant)
                || !seen.insert((case.source.as_str(), variant))
                || case.name != format!("{stem}__{variant}")
                || case.output.path != format!("xml/{family}__{variant}.xml")
                || diagnostic.historical.path != format!("historical/{family}__{variant}.xml")
                || record.value["historical_sha256"][variant].as_str()
                    != Some(diagnostic.historical.sha256.as_str())
                || self.retained_inputs.get(&diagnostic.historical.path)
                    != Some(&diagnostic.historical.sha256)
                || identity.historical_sha256.as_ref() != Some(&diagnostic.historical.sha256)
                || identity.policy_source_sha256.is_some()
                || identity.source != case.source
                || identity.variant != case.variant
                || identity.capture_sha256 != case.output.sha256
                || identity.record_sha256 != case.record_sha256
                || record.sha256 != case.record_sha256
                || record.value["source_sha256"].as_str() != Some(identity.source_sha256.as_str())
                || self.retained_inputs.get(&case.source) != Some(&identity.source_sha256)
            {
                return Err(malformed("original diagnostic case binding"));
            }
            digest(&case.output.sha256)?;
            digest(&diagnostic.historical.sha256)?;
            historical_paths.insert(diagnostic.historical.path.as_str());
            if diagnostic
                .raw_differences
                .iter()
                .any(|entry| !entry.browser.is_finite() || !entry.native.is_finite())
            {
                return Err(malformed("nonfinite original diagnostic difference"));
            }
        }
        if self
            .retained_inputs
            .keys()
            .filter(|path| path.starts_with("historical/"))
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != historical_paths
        {
            return Err(malformed("original diagnostic historical input census"));
        }
        Ok(())
    }
}

impl QualificationData<QualifiedCaseWire> {
    fn bind_source_provenance(self) -> Result<Qualification> {
        let sources: BTreeMap<_, _> = self
            .records
            .iter()
            .map(|record| Ok((record.source.as_str(), record.source_kind()?)))
            .collect::<Result<_>>()?;
        let cases = self
            .cases
            .into_iter()
            .map(|case| match case {
                QualifiedCaseWire::Exact(case) => Ok(QualifiedCase::Exact(case)),
                QualifiedCaseWire::Specialized(case) => Ok(QualifiedCase::Specialized(case)),
                QualifiedCaseWire::Matching(wire) => {
                    let kind = sources
                        .get(wire.case.source.as_str())
                        .ok_or_else(|| malformed("matching source is missing"))?;
                    Ok(QualifiedCase::Matching(wire.bind(*kind)?))
                }
            })
            .collect::<Result<_>>()?;
        Ok(Qualification {
            schema_version: self.schema_version,
            edition: self.edition,
            declaration: self.declaration,
            records: self.records,
            browser_settings: self.browser_settings,
            report: self.report,
            ownership: self.ownership,
            cases,
            retained_inputs: self.retained_inputs,
            current_owner_inputs: self.current_owner_inputs,
            capture_validation: self.capture_validation,
        })
    }
}

impl Qualification {
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|e| malformed(e.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        let wire = unique_json(raw)?;
        if wire
            .get("cases")
            .and_then(Value::as_array)
            .is_some_and(|cases| {
                cases.iter().any(|case| {
                    case.get("matching").is_some()
                        && case
                            .get("identity")
                            .is_some_and(|identity| identity.get("policy_source_sha256").is_some())
                })
            })
        {
            return Err(malformed(
                "matching capture binding cannot contain policy identity",
            ));
        }
        let wire: QualificationData<QualifiedCaseWire> =
            serde_json::from_str(raw).map_err(|e| malformed(e.to_string()))?;
        let value = wire.bind_source_provenance()?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        let specialized = SpecializedProfile::from_edition(&self.edition);
        let matching = self
            .cases
            .iter()
            .any(|case| matches!(case, QualifiedCase::Matching(_)));
        if matching {
            validate_matching_edition(&self.edition)?;
        }

        if self.schema_version != TERMINAL_SCHEMA
            || (!matching
                && specialized.is_none()
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
        if let Some(profile) = specialized
            && (self.records.len() != profile.sources().len()
                || self.cases.len() != profile.sources().len() * fixture_cases().len())
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
        let mut matching_history = BTreeSet::new();
        let mut paired_diagnostics = None;
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
                (QualifiedCase::Exact(case), None) if !matching => {
                    if case.identity.policy_source_sha256.is_none() {
                        return Err(malformed("missing exact policy identity"));
                    }
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
                    if variants
                        .insert((descriptor.source.as_str(), descriptor.variant.as_str())) =>
                {
                    self.validate_specialized(case, profile, record)?;
                }
                (QualifiedCase::Matching(case), None)
                    if matching
                        && variants
                            .insert((descriptor.source.as_str(), descriptor.variant.as_str())) =>
                {
                    let naming: CaseNaming = record
                        .value
                        .get("case_identity")
                        .map(|value| serde_json::from_value(value.clone()))
                        .transpose()
                        .map_err(|_| malformed("invalid matching case identity"))?
                        .unwrap_or_default();
                    let (family, stem) = naming.source_names(&descriptor.source)?;
                    let variant = descriptor.variant.as_str();
                    let kind = record.source_kind()?;
                    let membership: MatchingMembership =
                        serde_json::from_value(record.value["matching_publication"].clone())
                            .map_err(|_| malformed("missing or invalid matching membership"))?;
                    membership.validate(&descriptor.source, kind)?;
                    if let MatchingMembership::SourceEquivalent(pair) = &membership {
                        if paired_diagnostics
                            .as_ref()
                            .is_some_and(|reference| reference != &pair.diagnostics)
                        {
                            return Err(malformed("paired matching diagnostic references differ"));
                        }
                        paired_diagnostics = Some(pair.diagnostics.clone());
                    }
                    if self
                        .current_owner_inputs
                        .get(&kind.owner_path(&descriptor.source))
                        != Some(&case.identity.source_sha256)
                    {
                        return Err(changed("matching current source owner"));
                    }
                    if membership.edition() != self.edition
                        || !record.value["semantic_specifications"].is_null()
                        || !record.value["specialized_publication"].is_null()
                        || !record.value["relation"].is_null()
                        || record.value["expected"]["kind"] != "generated"
                        || case.category() != MatchingCategory::for_edition(&self.edition)
                        || !matching_metadata_is_valid(
                            case.category(),
                            &descriptor.source,
                            kind,
                            &record.value["expected"]["metadata"],
                        )
                        || !fixture_cases().iter().any(|(v, _)| *v == variant)
                        || descriptor.name != format!("{stem}__{variant}")
                        || descriptor.output.path != format!("xml/{family}__{variant}.xml")
                        || record.value["source_sha256"] != case.identity.source_sha256
                        || self.retained_inputs.get(&descriptor.source)
                            != Some(&case.identity.source_sha256)
                        || case.identity.policy_source_sha256.is_some()
                    {
                        return Err(malformed("matching terminal source or category changed"));
                    }
                    let history = record.value["historical_sha256"]
                        .as_object()
                        .ok_or_else(|| malformed("missing matching record historical pins"))?;
                    match (kind, case.provenance()) {
                        (SourceKind::Original, MatchingProvenance::Original { historical })
                            if history.len() == fixture_cases().len()
                                && historical.path
                                    == format!("historical/{family}__{variant}.xml")
                                && history.get(variant).and_then(Value::as_str)
                                    == Some(historical.sha256.as_str())
                                && case.identity.historical_sha256.as_ref()
                                    == Some(&historical.sha256) =>
                        {
                            self.require_retained(historical)?;
                            matching_history.insert(historical.path.as_str());
                        }
                        (SourceKind::Authored, MatchingProvenance::Authored)
                            if history.is_empty() && case.identity.historical_sha256.is_none() => {}
                        _ => return Err(malformed("matching terminal predecessor changed")),
                    }
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
        if matching
            && self
                .retained_inputs
                .keys()
                .filter(|path| path.starts_with("historical/"))
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                != matching_history
        {
            return Err(malformed("matching historical input census changed"));
        }
        if self.edition == FRI06_SUBGRID_EDITION {
            if self.records.len() != FRI06_SUBGRID_SOURCES.len() {
                return Err(malformed("incomplete FRI06 Subgrid matching cohort"));
            }
            verify_grid_lanes_producing_helper(&self.retained_inputs)?;
        }
        if self.edition == GRID_LANES_PAIRED_EDITION {
            if paired_diagnostics.is_none()
                || self.records.len() != GRID_LANES_AXIS_ORIGINAL_SOURCES.len()
            {
                return Err(malformed("incomplete source-equivalent matching cohort"));
            }
            verify_grid_lanes_producing_helper(&self.retained_inputs)?;
        }
        if matching
            && (self.cases.len() != self.records.len() * fixture_cases().len()
                || self.records.iter().any(|record| {
                    fixture_cases()
                        .iter()
                        .any(|(variant, _)| !variants.contains(&(record.source.as_str(), *variant)))
                }))
        {
            return Err(malformed("incomplete matching terminal census"));
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
        let source = case.case.source.as_str();
        if !profile.sources().contains(&source) {
            return Err(malformed(
                "specialized terminal source is outside its profile",
            ));
        }
        let family = source
            .strip_prefix("html/")
            .and_then(|path| path.strip_suffix(".html"))
            .expect("fixed original source path");
        let stem = family.rsplit('/').next().expect("fixed source name");
        let variant = &case.case.variant;
        if case.evidence.profile() != profile
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
            SpecializedEvidence::OriginalFlexParent { .. } => {
                original_flex_parent_evidence() == case.evidence
            }
            SpecializedEvidence::OriginalPercentageFlexParent { .. } => {
                original_percentage_flex_parent_evidence() == case.evidence
            }
            SpecializedEvidence::OriginalBlockAxes { .. } => {
                original_block_axes_evidence() == case.evidence
            }
            SpecializedEvidence::OriginalFlexAxes { .. } => {
                original_flex_axes_evidence() == case.evidence
            }
            SpecializedEvidence::OriginalGridAxes { .. } => {
                original_grid_axes_evidence() == case.evidence
            }
            SpecializedEvidence::OriginalSubgridAxes { .. } => {
                original_subgrid_axes_evidence() == case.evidence
            }
            SpecializedEvidence::OriginalSubgridBody { .. } => {
                original_subgrid_body::evidence() == case.evidence
            }
            SpecializedEvidence::PreparedAtomicPercentage { .. } => {
                prepared_atomic_percentage_evidence() == case.evidence
            }
            SpecializedEvidence::HorizontalBrFontBand { .. } => {
                super::br_font_band::horizontal_evidence() == case.evidence
            }
            SpecializedEvidence::VerticalBrFontBand { .. } => {
                super::br_font_band::vertical_evidence() == case.evidence
            }
            SpecializedEvidence::EmptyLineBrFontBand { .. } => {
                super::br_font_band::empty_line_evidence() == case.evidence
            }
        };
        if !evidence_matches
            || case.identity.policy_source_sha256.as_deref()
                != Some(case.evidence.policy_digest().as_str())
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
