//! Disposition of the original observations retained by the canonical report.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::portable_qualification::{
    AdmittedRegistry, HistoricalLink, HistoricalLinkKind, regular_file,
};
use super::support::capture_authority::{CapturedArtifact, sha256, unique_json};

const ACCOUNTING: &str = "historical-observations.json";
const MANIFEST: &str = "corpus.toml";
const REPORT: &str = "xml/generation-reports/all.json";
const PREPARED_TEXT: &str = "prepared-text-accounting.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DispositionDeclaration {
    schema_version: u8,
    manifest_sha256: String,
    report_sha256: String,
    retired: Retirement,
    superseded: Vec<Superseded>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Retirement {
    reason: RetirementReason,
    recapture_issue: u32,
    sources: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RetirementReason {
    MissingRootContextAuthority,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Superseded {
    source: String,
    edition: String,
    kind: HistoricalLinkKind,
}

#[derive(Deserialize)]
struct HistoricalReport {
    metadata: ReportMetadata,
    filter: Option<String>,
    summary: ReportSummary,
    generated: Vec<CapturedArtifact>,
}

#[derive(Deserialize)]
struct ReportMetadata {
    schema_version: u8,
    inputs: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct ReportSummary {
    generated: usize,
}

#[derive(Deserialize)]
struct PreparedTextAccounting {
    variants: Vec<PreparedObservation>,
}

#[derive(Deserialize)]
struct PreparedObservation {
    source: String,
    xml: String,
    xml_sha256: String,
    successor: Option<PreparedSuccessor>,
}

#[derive(Deserialize)]
struct PreparedSuccessor {
    xml: String,
    xml_sha256: String,
    source_sha256: String,
}

#[derive(Debug)]
enum Disposition {
    Retired,
    Superseded {
        edition: String,
        kind: HistoricalLinkKind,
    },
}

#[derive(Debug, Default)]
pub(super) struct HistoricalCounts {
    pub(super) retired: usize,
    pub(super) unchanged_source: usize,
    pub(super) source_equivalent: usize,
}

pub(super) struct HistoricalAccounting {
    artifacts: BTreeMap<String, CapturedArtifact>,
    dispositions: BTreeMap<String, Disposition>,
    seen: BTreeSet<String>,
    counts: HistoricalCounts,
}

fn read_file(root: &Path, name: &str) -> Result<Vec<u8>, String> {
    let path = regular_file(root, name).map_err(|error| error.to_string())?;
    fs::read(path).map_err(|error| format!("{name}: {error}"))
}

impl HistoricalAccounting {
    pub(super) fn load(root: &Path, registry: &AdmittedRegistry) -> Result<Self, String> {
        let declaration = read_file(root, ACCOUNTING)?;
        let declaration = std::str::from_utf8(&declaration).map_err(|error| error.to_string())?;
        let declaration: DispositionDeclaration =
            serde_json::from_value(unique_json(declaration).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        let manifest = read_file(root, MANIFEST)?;
        let report = read_file(root, REPORT)?;
        if declaration.schema_version != 1
            || sha256(&manifest) != declaration.manifest_sha256
            || sha256(&report) != declaration.report_sha256
        {
            return Err("historical accounting manifest or report identity changed".into());
        }
        let report: HistoricalReport =
            serde_json::from_slice(&report).map_err(|error| error.to_string())?;
        if report.metadata.schema_version != 4
            || report.filter.is_some()
            || report.summary.generated != report.generated.len()
            || report.metadata.inputs.get(MANIFEST) != Some(&declaration.manifest_sha256)
        {
            return Err("historical report census or manifest binding changed".into());
        }

        let variants: BTreeSet<_> = crate::fixture_variants::fixture_cases()
            .into_iter()
            .map(|(variant, _)| variant)
            .collect();
        let mut artifacts = BTreeMap::new();
        let mut families = BTreeMap::<String, BTreeSet<String>>::new();
        for artifact in report.generated {
            let Some(source) = artifact.source.strip_prefix("html/") else {
                return Err(format!(
                    "historical source outside html: {}",
                    artifact.source
                ));
            };
            let Some(stem) = source.strip_suffix(".html") else {
                return Err(format!(
                    "historical source is not HTML: {}",
                    artifact.source
                ));
            };
            let expected = format!("xml/{stem}__{}.xml", artifact.variant);
            if artifact.output != expected
                || report.metadata.inputs.get(&artifact.source) != Some(&artifact.source_sha256)
            {
                return Err(format!(
                    "historical source or output binding changed: {expected}"
                ));
            }
            let family = families.entry(artifact.source.clone()).or_default();
            if family.is_empty()
                && sha256(&read_file(root, &artifact.source)?) != artifact.source_sha256
            {
                return Err(format!("historical source changed: {}", artifact.source));
            }
            if !family.insert(artifact.variant.clone())
                || artifacts
                    .insert(artifact.output.clone(), artifact)
                    .is_some()
            {
                return Err(format!("duplicate historical observation: {expected}"));
            }
        }
        for (source, actual) in &families {
            if actual.iter().map(String::as_str).collect::<BTreeSet<_>>() != variants {
                return Err(format!("incomplete historical variants for {source}"));
            }
        }

        let Retirement {
            reason: RetirementReason::MissingRootContextAuthority,
            recapture_issue,
            sources,
        } = declaration.retired;
        if recapture_issue != 2021 {
            return Err("historical retirement lacks the owning recapture issue".into());
        }
        let mut dispositions = BTreeMap::new();
        for source in sources {
            Self::declare(&mut dispositions, source, Disposition::Retired)?;
        }
        for entry in declaration.superseded {
            Self::declare(
                &mut dispositions,
                entry.source,
                Disposition::Superseded {
                    edition: entry.edition,
                    kind: entry.kind,
                },
            )?;
        }
        if let Some(missing) = families
            .keys()
            .find(|source| !dispositions.contains_key(*source))
        {
            return Err(format!("unaccounted historical family: {missing}"));
        }
        if let Some(extra) = dispositions
            .keys()
            .find(|source| !families.contains_key(*source))
        {
            return Err(format!("extra historical disposition: {extra}"));
        }
        let accounting = Self {
            artifacts,
            dispositions,
            seen: BTreeSet::new(),
            counts: HistoricalCounts::default(),
        };
        accounting.verify_successors(registry.historical_links())?;
        accounting.verify_prepared_text(&read_file(root, PREPARED_TEXT)?)?;
        Ok(accounting)
    }

    fn declare(
        dispositions: &mut BTreeMap<String, Disposition>,
        source: String,
        disposition: Disposition,
    ) -> Result<(), String> {
        if dispositions.insert(source.clone(), disposition).is_some() {
            return Err(format!("duplicate historical disposition: {source}"));
        }
        Ok(())
    }

    fn verify_successors<'a>(
        &self,
        links: impl IntoIterator<Item = (&'a str, &'a HistoricalLink)>,
    ) -> Result<(), String> {
        let mut matched = BTreeSet::new();
        for (edition, link) in links {
            let relative = link
                .predecessor
                .path
                .strip_prefix("historical/")
                .ok_or_else(|| {
                    format!("invalid historical predecessor: {}", link.predecessor.path)
                })?;
            let path = format!("xml/{relative}");
            let Some(artifact) = self.artifacts.get(&path) else {
                // Some registered editions bind an older observation outside this report.
                continue;
            };
            if artifact.source != link.source
                || artifact.source_sha256 != link.source_sha256
                || artifact.variant != link.variant
                || artifact.output_sha256 != link.predecessor.sha256
                || !matches!(
                    &self.dispositions[&artifact.source],
                    Disposition::Superseded { edition: expected, kind }
                        if expected == edition && *kind == link.kind
                )
                || !matched.insert(artifact.output.as_str())
            {
                return Err(format!("historical successor binding changed: {path}"));
            }
        }
        for (path, artifact) in &self.artifacts {
            if matches!(
                &self.dispositions[&artifact.source],
                Disposition::Superseded { .. }
            ) && !matched.contains(path.as_str())
            {
                return Err(format!("missing authenticated successor: {path}"));
            }
        }
        Ok(())
    }

    fn verify_prepared_text(&self, bytes: &[u8]) -> Result<(), String> {
        let accounting: PreparedTextAccounting =
            serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        for observation in accounting.variants {
            if let Some(artifact) = self.artifacts.get(&observation.xml)
                && artifact.output_sha256 == observation.xml_sha256
                && artifact.source != observation.source
            {
                return Err(format!(
                    "prepared Text historical source changed: {}",
                    observation.xml
                ));
            }
            if let Some(successor) = observation.successor
                && successor.xml.starts_with("xml/")
                && !self.artifacts.get(&successor.xml).is_some_and(|artifact| {
                    artifact.source == observation.source
                        && artifact.source_sha256 == successor.source_sha256
                        && artifact.output_sha256 == successor.xml_sha256
                })
            {
                return Err(format!(
                    "prepared Text observation identity changed: {}",
                    successor.xml
                ));
            }
        }
        Ok(())
    }

    pub(super) fn observe(
        &mut self,
        root: &Path,
        path: &Path,
        xml: Option<&str>,
        missing_root_authority: bool,
    ) -> Result<(), String> {
        let relative = path
            .strip_prefix(root)
            .or_else(|_| path.strip_prefix("./tests/layout/browser_parity"))
            .or_else(|_| path.strip_prefix("crates/surgeist-layout/tests/layout/browser_parity"))
            .ok()
            .and_then(Path::to_str)
            .ok_or_else(|| format!("unaccounted historical path: {}", path.display()))?;
        let artifact = self
            .artifacts
            .get(relative)
            .ok_or_else(|| format!("unaccounted historical observation: {relative}"))?;
        if !self.seen.insert(relative.to_owned()) {
            return Err(format!("historical observation visited twice: {relative}"));
        }
        regular_file(root, relative).map_err(|error| error.to_string())?;
        if !xml.is_some_and(|xml| sha256(xml.as_bytes()) == artifact.output_sha256) {
            return Err(format!(
                "historical observation identity changed: {relative}"
            ));
        }
        if !missing_root_authority {
            return Err(format!("historical capture contract changed: {relative}"));
        }
        match &self.dispositions[&artifact.source] {
            Disposition::Retired => self.counts.retired += 1,
            Disposition::Superseded {
                kind: HistoricalLinkKind::SameSource,
                ..
            } => self.counts.unchanged_source += 1,
            Disposition::Superseded {
                kind: HistoricalLinkKind::SourceEquivalent,
                ..
            } => self.counts.source_equivalent += 1,
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Result<HistoricalCounts, String> {
        if let Some(missing) = self
            .artifacts
            .keys()
            .find(|path| !self.seen.contains(*path))
        {
            return Err(format!("unvisited historical observation: {missing}"));
        }
        Ok(self.counts)
    }
}

#[cfg(test)]
mod tests {
    use super::super::support::capture_authority::PathDigest;
    use super::*;

    const SOURCE: &str = "html/block/block_basic.html";
    const OUTPUT: &str = "xml/block/block_basic__border_box_ltr.xml";
    const HISTORY: &str = "historical/block/block_basic__border_box_ltr.xml";
    const XML: &str = include_str!("xml/block/block_basic__border_box_ltr.xml");

    fn sample(disposition: Disposition) -> HistoricalAccounting {
        let artifact = serde_json::from_value(serde_json::json!({
            "name": "block_basic__border_box_ltr",
            "source": SOURCE,
            "output": OUTPUT,
            "variant": "border_box_ltr",
            "source_sha256": "historical-source",
            "linked_resources": {},
            "job_sha256": "historical-job",
            "output_sha256": sha256(XML.as_bytes()),
        }))
        .unwrap();
        HistoricalAccounting {
            artifacts: BTreeMap::from([(OUTPUT.into(), artifact)]),
            dispositions: BTreeMap::from([(SOURCE.into(), disposition)]),
            seen: BTreeSet::new(),
            counts: HistoricalCounts::default(),
        }
    }

    fn owner() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity")
    }

    #[test]
    fn historical_disposition_rejects_unaccounted_and_unvisited_xml() {
        let root = owner();
        let mut accounting = sample(Disposition::Retired);
        let unexpected = root.join("xml/block/block_basic__border_box_rtl.xml");
        assert_eq!(
            accounting
                .observe(&root, &unexpected, Some(XML), true)
                .unwrap_err(),
            "unaccounted historical observation: xml/block/block_basic__border_box_rtl.xml"
        );
        assert_eq!(
            accounting.finish().unwrap_err(),
            format!("unvisited historical observation: {OUTPUT}")
        );
    }

    #[test]
    fn historical_disposition_refuses_altered_xml_and_unexpected_raw_outcome() {
        let root = owner();
        let path = root.join(OUTPUT);
        let changed = format!("{XML}\n");
        let mut accounting = sample(Disposition::Retired);
        assert_eq!(
            accounting
                .observe(&root, &path, Some(&changed), true)
                .unwrap_err(),
            format!("historical observation identity changed: {OUTPUT}")
        );
        let mut accounting = sample(Disposition::Retired);
        assert_eq!(
            accounting
                .observe(&root, &path, Some(XML), false)
                .unwrap_err(),
            format!("historical capture contract changed: {OUTPUT}")
        );
        let mut accounting = sample(Disposition::Retired);
        accounting.observe(&root, &path, Some(XML), true).unwrap();
        assert_eq!(accounting.finish().unwrap().retired, 1);
    }

    #[test]
    fn corpus_authority_refuses_empty_prepared_text_ledger() {
        let accounting = sample(Disposition::Retired);
        assert_eq!(
            accounting
                .verify_prepared_text(br#"{"variants":[]}"#)
                .unwrap_err(),
            "prepared Text accounting identity changed"
        );
    }

    #[test]
    fn historical_disposition_requires_exact_authenticated_successor() {
        let mut link = HistoricalLink {
            source: SOURCE.into(),
            source_sha256: "historical-source".into(),
            variant: "border_box_ltr".into(),
            predecessor: PathDigest {
                path: HISTORY.into(),
                sha256: sha256(XML.as_bytes()),
            },
            kind: HistoricalLinkKind::SameSource,
        };
        let accounting = sample(Disposition::Superseded {
            edition: "current".into(),
            kind: HistoricalLinkKind::SameSource,
        });
        assert_eq!(
            accounting.verify_successors([]).unwrap_err(),
            format!("missing authenticated successor: {OUTPUT}")
        );
        assert!(accounting.verify_successors([("old", &link)]).is_err());
        assert!(
            sample(Disposition::Retired)
                .verify_successors([("current", &link)])
                .is_err()
        );
        accounting.verify_successors([("current", &link)]).unwrap();
        link.predecessor.sha256 = "changed".into();
        assert_eq!(
            accounting
                .verify_successors([("current", &link)])
                .unwrap_err(),
            format!("historical successor binding changed: {OUTPUT}")
        );
    }
}
