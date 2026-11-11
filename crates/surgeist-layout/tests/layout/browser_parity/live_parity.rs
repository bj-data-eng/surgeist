//! Browser provenance and independent Native results for the live corpus.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::portable_qualification::regular_file;
use super::support::{
    self, EndpointUnobservableField, Golden, NumericMismatch, RootContext,
    capture_authority::{CapturedArtifact, sha256, unique_json},
};

const HISTORICAL_REPORT: &str = "xml/generation-reports/all.json";
const LIVE_OUTPUT: &str = "live-xml";
const LIVE_REPORT: &str = "live-xml/generation-reports/all.json";
const LIVE_OWNERSHIP: &str = "live-xml/generation-ownership.json";
const LIVE_DECLARATION: &str = "live-corpus.toml";
const CAPTURE_HELPER: &str = "scripts/gentest/test_helper.js";
const RECEIPT: &str = "live-native-observations.json";
const OWNERS: &str = "live-native-owners.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureReport {
    metadata: Metadata,
    filter: Option<String>,
    summary: Counts,
    generated: Vec<CapturedArtifact>,
    unsupported: Vec<Disposition>,
    expected_fail: Vec<Disposition>,
    quarantined: Vec<Disposition>,
    failed_to_generate: Vec<Disposition>,
}

#[derive(Deserialize)]
struct Metadata {
    schema_version: u8,
    inputs: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Disposition {
    source: String,
    variant: Option<String>,
    reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Counts {
    generated: usize,
    unsupported: usize,
    expected_fail: usize,
    quarantined: usize,
    failed_to_generate: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ownership {
    schema_version: u8,
    artifacts: BTreeMap<String, String>,
}

type Classifications<'a> = BTreeMap<(&'a str, Option<&'a str>), (&'static str, Option<&'a str>)>;

impl CaptureReport {
    fn classifications(&self) -> Result<Classifications<'_>, String> {
        if self.metadata.schema_version != 4
            || self.filter.is_some()
            || self.summary
                != (Counts {
                    generated: self.generated.len(),
                    unsupported: self.unsupported.len(),
                    expected_fail: self.expected_fail.len(),
                    quarantined: self.quarantined.len(),
                    failed_to_generate: self.failed_to_generate.len(),
                })
            || self.summary.generated == 0
            || self.summary.failed_to_generate != 0
        {
            return Err("invalid full browser report counts or metadata".into());
        }

        let mut cases = Classifications::new();
        for artifact in &self.generated {
            if cases
                .insert(
                    (artifact.source.as_str(), Some(artifact.variant.as_str())),
                    ("generated", None),
                )
                .is_some()
            {
                return Err(format!("duplicate captured case {}", artifact.output));
            }
        }
        for (kind, dispositions) in [
            ("unsupported", &self.unsupported),
            ("expected-fail", &self.expected_fail),
            ("quarantined", &self.quarantined),
        ] {
            for disposition in dispositions {
                if disposition.reason.trim().is_empty()
                    || cases
                        .insert(
                            (disposition.source.as_str(), disposition.variant.as_deref()),
                            (kind, Some(disposition.reason.as_str())),
                        )
                        .is_some()
                {
                    return Err(format!(
                        "invalid or duplicate {kind} disposition for {}",
                        disposition.source
                    ));
                }
            }
        }
        Ok(cases)
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClassificationSummary {
    sources: usize,
    historical: Counts,
    live: Counts,
    transitions: BTreeMap<String, usize>,
    newly_unsupported: BTreeMap<String, usize>,
    changed_unsupported_reasons: BTreeMap<String, usize>,
}

impl ClassificationSummary {
    fn compare(historical: CaptureReport, live: &CaptureReport) -> Result<Self, String> {
        let before = historical.classifications()?;
        let after = live.classifications()?;
        if !before.keys().eq(after.keys()) {
            return Err("historical and live source or variant census differs".into());
        }
        let mut sources = BTreeSet::new();
        let mut transitions = BTreeMap::new();
        let mut newly_unsupported = BTreeMap::new();
        let mut changed_unsupported_reasons = BTreeMap::new();
        for ((source, variant), (previous, old_reason)) in before {
            sources.insert(source);
            let (current, reason) = after[&(source, variant)];
            *transitions
                .entry(format!("{previous} to {current}"))
                .or_default() += 1;
            if let Some(reason) = reason.filter(|_| current == "unsupported") {
                if previous == "generated" {
                    *newly_unsupported.entry(reason.to_owned()).or_default() += 1;
                } else if previous == "unsupported" && old_reason != Some(reason) {
                    *changed_unsupported_reasons
                        .entry(reason.to_owned())
                        .or_default() += 1;
                }
            }
        }
        Ok(Self {
            sources: sources.len(),
            historical: historical.summary,
            live: live.summary,
            transitions,
            newly_unsupported,
            changed_unsupported_reasons,
        })
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum NativeOutcome {
    Differences {
        fields: Vec<NumericMismatch>,
    },
    Refused {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        numeric_refusal: Option<String>,
    },
    Unobservable {
        fields: Vec<EndpointUnobservableField>,
    },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeNonmatch {
    source: String,
    output: String,
    variant: String,
    outcome: NativeOutcome,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    historical_report_sha256: String,
    live_report_sha256: String,
    live_ownership_sha256: String,
    classifications: ClassificationSummary,
    native_matches: usize,
    native_nonmatches: Vec<NativeNonmatch>,
}

pub(super) struct LiveCorpusRun {
    receipt: Receipt,
    errors: Vec<String>,
}

pub(super) struct LiveCorpusCounts {
    pub(super) sources: usize,
    pub(super) unsupported: usize,
    pub(super) matches: usize,
    pub(super) differences: usize,
    pub(super) refusals: usize,
    pub(super) unobservable: usize,
}

impl LiveCorpusRun {
    pub(super) fn collect(root: &Path) -> Result<Self, String> {
        let historical_bytes = read_file(root, HISTORICAL_REPORT)?;
        let live_bytes = read_file(root, LIVE_REPORT)?;
        let historical: CaptureReport = parse_json(&historical_bytes)?;
        let live: CaptureReport = parse_json(&live_bytes)?;
        verify_shared_inputs(&historical.metadata.inputs, &live.metadata.inputs)?;
        let classifications = ClassificationSummary::compare(historical, &live)?;
        for (path, expected) in &live.metadata.inputs {
            read_pinned(root, path, expected)?;
        }

        let mut discovered = live_outputs(root)?;
        let live_report_sha256 = sha256(&live_bytes);
        let (live_ownership_sha256, mut errors) =
            match verify_ownership(root, &live_report_sha256, &live.generated) {
                Ok(digest) => (digest, Vec::new()),
                Err(error) => (String::new(), vec![error]),
            };
        let mut receipt = Receipt {
            schema_version: 1,
            historical_report_sha256: sha256(&historical_bytes),
            live_report_sha256,
            live_ownership_sha256,
            classifications,
            native_matches: 0,
            native_nonmatches: Vec::new(),
        };
        let mut visited = BTreeSet::new();
        for artifact in live.generated {
            if !visited.insert(artifact.output.clone()) {
                errors.push(format!("duplicate live output {}", artifact.output));
                continue;
            }
            discovered.remove(&artifact.output);
            match compare_capture(root, &live.metadata.inputs, &artifact) {
                Ok(None) => receipt.native_matches += 1,
                Ok(Some(outcome)) => receipt.native_nonmatches.push(NativeNonmatch {
                    source: artifact.source,
                    output: artifact.output,
                    variant: artifact.variant,
                    outcome,
                }),
                Err(error) => errors.push(format!("{}: {error}", artifact.output)),
            }
        }
        errors.extend(
            discovered
                .into_iter()
                .map(|path| format!("unreported live output {path}")),
        );
        for (path, expected) in [
            (HISTORICAL_REPORT, &receipt.historical_report_sha256),
            (LIVE_REPORT, &receipt.live_report_sha256),
            (LIVE_OWNERSHIP, &receipt.live_ownership_sha256),
        ] {
            if !expected.is_empty()
                && let Err(error) = read_pinned(root, path, expected)
            {
                errors.push(error);
            }
        }
        Ok(Self { receipt, errors })
    }

    pub(super) fn check_accounting(&self, root: &Path) -> Result<(), String> {
        self.require_complete()?;
        let expected: Receipt = parse_json(&read_file(root, RECEIPT)?)?;
        if expected != self.receipt {
            return Err("live corpus classifications or Native observations changed".into());
        }

        let owners: BTreeMap<String, Vec<u32>> = parse_json(&read_file(root, OWNERS)?)?;
        let unresolved: BTreeSet<_> = self
            .receipt
            .native_nonmatches
            .iter()
            .map(|row| row.source.as_str())
            .collect();
        if unresolved != owners.keys().map(String::as_str).collect()
            || owners.values().any(|issues| {
                issues.is_empty()
                    || issues.contains(&0)
                    || issues.iter().collect::<BTreeSet<_>>().len() != issues.len()
            })
        {
            return Err(
                "live Native issue ownership differs from the unresolved source census".into(),
            );
        }
        Ok(())
    }

    pub(super) fn check_report_identity(&self, root: &Path) -> Result<(), String> {
        let expected: Receipt = parse_json(&read_file(root, RECEIPT)?)?;
        if expected.schema_version != self.receipt.schema_version
            || expected.historical_report_sha256 != self.receipt.historical_report_sha256
            || expected.live_report_sha256 != self.receipt.live_report_sha256
            || expected.live_ownership_sha256 != self.receipt.live_ownership_sha256
        {
            return Err("live corpus report identity changed".into());
        }
        Ok(())
    }

    fn require_complete(&self) -> Result<(), String> {
        if !self.errors.is_empty() {
            return Err(self.errors.join("\n"));
        }
        if self.receipt.native_matches + self.receipt.native_nonmatches.len()
            != self.receipt.classifications.live.generated
        {
            return Err("live Native comparison census is incomplete".into());
        }
        Ok(())
    }

    pub(super) fn write_receipt(&self, path: &Path) -> Result<(), String> {
        self.require_complete()?;
        let mut bytes =
            serde_json::to_vec_pretty(&self.receipt).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        fs::File::create_new(path)
            .and_then(|mut file| file.write_all(&bytes))
            .map_err(|error| format!("{}: {error}", path.display()))
    }

    pub(super) fn counts(&self) -> LiveCorpusCounts {
        let mut counts = LiveCorpusCounts {
            sources: self.receipt.classifications.sources,
            unsupported: self.receipt.classifications.live.unsupported,
            matches: self.receipt.native_matches,
            differences: 0,
            refusals: 0,
            unobservable: 0,
        };
        for row in &self.receipt.native_nonmatches {
            match row.outcome {
                NativeOutcome::Differences { .. } => counts.differences += 1,
                NativeOutcome::Refused { .. } => counts.refusals += 1,
                NativeOutcome::Unobservable { .. } => counts.unobservable += 1,
            }
        }
        counts
    }

    pub(super) fn raw_failures(&self) -> Vec<String> {
        let mut failures = self.errors.clone();
        failures.extend(self.receipt.native_nonmatches.iter().map(|row| {
            let outcome = serde_json::to_string(&row.outcome).expect("finite Native comparison");
            format!("{}: {outcome}", row.output)
        }));
        failures
    }
}

fn verify_shared_inputs(
    historical: &BTreeMap<String, String>,
    live: &BTreeMap<String, String>,
) -> Result<(), String> {
    if live.len() != historical.len() + 1
        || !live.contains_key(LIVE_DECLARATION)
        || historical.iter().any(|(path, expected)| {
            live.get(path)
                .is_none_or(|digest| path != CAPTURE_HELPER && digest != expected)
        })
    {
        return Err("live and historical source input inventories disagree".into());
    }
    Ok(())
}

fn verify_ownership(
    root: &Path,
    report_sha256: &str,
    generated: &[CapturedArtifact],
) -> Result<String, String> {
    let bytes = read_file(root, LIVE_OWNERSHIP)?;
    let ownership: Ownership = parse_json(&bytes)?;
    let wanted: BTreeMap<&str, &str> = generated
        .iter()
        .map(|artifact| (artifact.output.as_str(), artifact.output_sha256.as_str()))
        .chain([(LIVE_REPORT, report_sha256)])
        .collect();
    if ownership.schema_version != 1
        || !ownership
            .artifacts
            .iter()
            .map(|(path, digest)| (path.as_str(), digest.as_str()))
            .eq(wanted)
    {
        return Err("live ownership does not cover the exact report and generated outputs".into());
    }
    Ok(sha256(&bytes))
}

fn compare_capture(
    root: &Path,
    inputs: &BTreeMap<String, String>,
    artifact: &CapturedArtifact,
) -> Result<Option<NativeOutcome>, String> {
    let source = artifact
        .source
        .strip_prefix("html/")
        .and_then(|path| path.strip_suffix(".html"))
        .ok_or_else(|| format!("invalid live capture source {}", artifact.source))?;
    if artifact.output != format!("{LIVE_OUTPUT}/{source}__{}.xml", artifact.variant)
        || inputs.get(&artifact.source) != Some(&artifact.source_sha256)
    {
        return Err("live capture source, variant, or output binding changed".into());
    }
    for (path, expected) in &artifact.linked_resources {
        read_pinned(root, path, expected)?;
    }
    let bytes = read_pinned(root, &artifact.output, &artifact.output_sha256)?;
    let xml = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
    let golden = Golden::parse(xml).map_err(|error| error.to_string())?;
    if golden.name != artifact.name
        || matches!(
            golden.viewport.root_context,
            RootContext::UnqualifiedCapture { .. }
        )
    {
        return Err("live capture name or root-context authority changed".into());
    }
    let outcome = match support::assert_surgeist_matches_with_endpoint_accounting(&golden) {
        Ok(result) if result.endpoint_unobservable_fields().is_empty() => None,
        Ok(result) => Some(NativeOutcome::Unobservable {
            fields: result.endpoint_unobservable_fields().to_vec(),
        }),
        Err(error) => Some(
            match support::collect_surgeist_numeric_mismatches(&golden) {
                Ok(fields) if !fields.is_empty() => NativeOutcome::Differences { fields },
                result => {
                    let reason = error.to_string();
                    let numeric_refusal = result
                        .err()
                        .map(|error| error.to_string())
                        .filter(|error| error != &reason);
                    NativeOutcome::Refused {
                        reason,
                        numeric_refusal,
                    }
                }
            },
        ),
    };
    Ok(outcome)
}

fn live_outputs(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut pending = vec![root.join(LIVE_OUTPUT)];
    let mut outputs = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_dir() {
                pending.push(path);
            } else if !kind.is_file() {
                return Err(format!("non-regular live output {}", path.display()));
            } else {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| error.to_string())?
                    .to_str()
                    .ok_or_else(|| format!("invalid live output path {}", path.display()))?;
                let relative = relative.replace('\\', "/");
                if relative != LIVE_REPORT && relative != LIVE_OWNERSHIP {
                    outputs.insert(relative);
                }
            }
        }
    }
    Ok(outputs)
}

fn read_file(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = regular_file(root, relative).map_err(|error| error.to_string())?;
    fs::read(path).map_err(|error| format!("{relative}: {error}"))
}

fn read_pinned(root: &Path, relative: &str, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = read_file(root, relative)?;
    if sha256(&bytes) != expected {
        return Err(format!("live corpus digest changed for {relative}"));
    }
    Ok(bytes)
}

fn parse_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    serde_json::from_value(unique_json(text).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_report(variants: &[&str]) -> CaptureReport {
        let generated = variants
            .iter()
            .map(|variant| {
                serde_json::from_value(serde_json::json!({
                    "name": format!("block_basic__{variant}"),
                    "source": "html/block/block_basic.html",
                    "output": format!("live-xml/block/block_basic__{variant}.xml"),
                    "variant": variant,
                    "source_sha256": "source",
                    "linked_resources": {},
                    "job_sha256": "job",
                    "output_sha256": "output",
                }))
                .unwrap()
            })
            .collect();
        CaptureReport {
            metadata: Metadata {
                schema_version: 4,
                inputs: BTreeMap::new(),
            },
            filter: None,
            summary: Counts {
                generated: variants.len(),
                unsupported: 0,
                expected_fail: 0,
                quarantined: 0,
                failed_to_generate: 0,
            },
            generated,
            unsupported: Vec::new(),
            expected_fail: Vec::new(),
            quarantined: Vec::new(),
            failed_to_generate: Vec::new(),
        }
    }

    #[test]
    fn classification_refuses_a_missing_live_variant() {
        let historical = capture_report(&["border_box_ltr", "border_box_rtl"]);
        let live = capture_report(&["border_box_ltr"]);
        assert_eq!(
            ClassificationSummary::compare(historical, &live).unwrap_err(),
            "historical and live source or variant census differs"
        );
    }

    #[test]
    fn live_input_inventory_refuses_dropped_or_rebound_source() {
        let historical = BTreeMap::from([
            (CAPTURE_HELPER.into(), "old-helper".into()),
            ("html/block/block_basic.html".into(), "source".into()),
        ]);
        let mut live = BTreeMap::from([
            (CAPTURE_HELPER.into(), "live-helper".into()),
            ("html/block/block_basic.html".into(), "source".into()),
            (LIVE_DECLARATION.into(), "declaration".into()),
        ]);
        verify_shared_inputs(&historical, &live).unwrap();
        live.insert("html/block/block_basic.html".into(), "changed".into());
        assert!(verify_shared_inputs(&historical, &live).is_err());
        live.remove("html/block/block_basic.html");
        assert!(verify_shared_inputs(&historical, &live).is_err());
    }

    #[test]
    fn receipt_refuses_an_uncompared_live_output() {
        let variants = &["border_box_ltr", "border_box_rtl"];
        let classifications =
            ClassificationSummary::compare(capture_report(variants), &capture_report(variants))
                .unwrap();
        let run = LiveCorpusRun {
            receipt: Receipt {
                schema_version: 1,
                historical_report_sha256: "historical".into(),
                live_report_sha256: "live".into(),
                live_ownership_sha256: "ownership".into(),
                classifications,
                native_matches: 1,
                native_nonmatches: Vec::new(),
            },
            errors: Vec::new(),
        };
        assert_eq!(
            run.require_complete().unwrap_err(),
            "live Native comparison census is incomplete"
        );
    }

    #[test]
    fn live_digest_refuses_altered_bytes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
        assert_eq!(
            read_pinned(&root, HISTORICAL_REPORT, "altered").unwrap_err(),
            format!("live corpus digest changed for {HISTORICAL_REPORT}")
        );
    }
}
