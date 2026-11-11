//! Strict Layout corpus declarations and their source-import policy owner.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;
use surgeist_generator::browser::{
    self, BrowserCorpus, BrowserLocation, CaseSpec, FixtureSpec, FixtureStatus, ReportScope,
    SourceImportSpec,
};
use surgeist_generator::{PinnedSource, RelativePath, SourceRevision};

use crate::adapter::fixture_cases;
use crate::browser_profile::BrowserManifest;

const SOURCE_ATTESTATION: &str = "html/.surgeist-source.json";

type Result<T> = std::result::Result<T, String>;

fn relative(value: impl AsRef<str>) -> Result<RelativePath> {
    RelativePath::new(value).map_err(message)
}
fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    schema_version: u32,
    browser: BrowserManifest,
    generation_reports: Reports,
    source_roots: SourceRoots,
    imports: Imports,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reports {
    full: FullReport,
    scoped: Vec<ScopedReport>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FullReport {
    file: String,
    generated: usize,
    unsupported: usize,
    expected_fail: usize,
    quarantined: usize,
    failed_to_generate: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopedReport {
    filter: String,
    file: String,
    generated: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRoots {
    taffy: SourceRoot,
    surgeist: SourceRoot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRoot {
    kind: String,
    path: String,
    upstream_commit: Option<String>,
    description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Imports {
    taffy: TaffyImport,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaffyImport {
    repo: String,
    commit: String,
    source_dir: String,
    destination: String,
    expected_count: usize,
    excluded_destination_dirs: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source_root: CaseRoot,
    source: String,
    generator: CaseGenerator,
    status: CaseStatus,
    reason: Option<String>,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum CaseRoot {
    Taffy,
    Surgeist,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CaseGenerator {
    ConstrainedHtml,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CaseStatus {
    Active,
    ExpectedFail,
    Unsupported,
    Quarantined,
}

impl Manifest {
    pub(super) fn validate(&self) -> Result<()> {
        if self.schema_version != 2 {
            return Err("corpus manifest schema_version must be 2".to_string());
        }
        let taffy = &self.source_roots.taffy;
        let surgeist = &self.source_roots.surgeist;
        if taffy.kind != "taffy"
            || taffy.path != "html"
            || taffy.upstream_commit.as_deref() != Some(self.imports.taffy.commit.as_str())
            || taffy.description.trim().is_empty()
            || surgeist.kind != "surgeist"
            || surgeist.path != "html"
            || surgeist.upstream_commit.is_some()
            || surgeist.description.trim().is_empty()
            || self.imports.taffy.destination != "html"
        {
            return Err(
                "manifest source roots disagree with the layout corpus declaration".to_string(),
            );
        }
        self.browser.validate()?;
        let mut ids = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for case in &self.cases {
            if case.id.trim().is_empty() || !ids.insert(&case.id) {
                return Err("empty or duplicate corpus case identity".to_string());
            }
            let source = relative(&case.source)?;
            if !sources.insert(source.clone()) {
                return Err(format!("duplicate corpus case source {}", source.as_str()));
            }
            match case.generator {
                CaseGenerator::ConstrainedHtml => {}
            }
            if Path::new(source.as_str())
                .extension()
                .is_none_or(|extension| extension != "html")
            {
                return Err(format!("case {} must reference an HTML fixture", case.id));
            }
        }
        Ok(())
    }

    pub(super) fn source_import(&self) -> Result<SourceImportSpec> {
        let import = &self.imports.taffy;
        let pin = PinnedSource::new(
            "taffy",
            &import.repo,
            SourceRevision::new(&import.commit).map_err(message)?,
            relative(&import.source_dir)?,
        )
        .map_err(message)?;
        SourceImportSpec::new(
            pin,
            relative(&import.destination)?,
            relative(".surgeist-source.json")?,
            "html".to_string(),
            import.expected_count,
            import
                .excluded_destination_dirs
                .iter()
                .map(relative)
                .collect::<Result<_>>()?,
            self.cases
                .iter()
                .filter(|case| case.source_root == CaseRoot::Surgeist)
                .map(|case| relative(&case.source))
                .collect::<Result<_>>()?,
        )
        .map_err(message)
    }

    pub(super) fn corpus(
        &self,
        browser_location: BrowserLocation,
        files: Vec<RelativePath>,
    ) -> Result<BrowserCorpus> {
        let declarations = self
            .cases
            .iter()
            .filter(|case| case.source_root == CaseRoot::Surgeist)
            .map(|case| (case.source.as_str(), case))
            .collect::<BTreeMap<_, _>>();
        for source in declarations.keys() {
            if !files.iter().any(|file| file.as_str() == *source) {
                return Err(format!("missing declared layout fixture {source}"));
            }
        }
        let fixtures = files
            .into_iter()
            .map(|file| {
                let declaration = declarations.get(file.as_str()).copied();
                let path = Path::new(file.as_str());
                let stem = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| format!("fixture {} has no UTF-8 stem", file.as_str()))?;
                let name = declaration.map_or_else(|| stem.to_string(), |case| case.id.clone());
                let status = declaration.map_or(FixtureStatus::Active, |case| match case.status {
                    CaseStatus::Active => FixtureStatus::Active,
                    CaseStatus::ExpectedFail => FixtureStatus::ExpectedFail {
                        reason: case
                            .reason
                            .clone()
                            .unwrap_or_else(|| "manifest marks case expected-fail".to_string()),
                    },
                    CaseStatus::Unsupported => FixtureStatus::Unsupported {
                        reason: case
                            .reason
                            .clone()
                            .unwrap_or_else(|| "manifest marks case unsupported".to_string()),
                    },
                    CaseStatus::Quarantined => FixtureStatus::Quarantined {
                        reason: case
                            .reason
                            .clone()
                            .unwrap_or_else(|| "manifest marks case quarantined".to_string()),
                    },
                });
                let cases = fixture_cases()
                    .into_iter()
                    .map(|(variant, _)| {
                        let id = format!("{stem}__{variant}");
                        let output = Path::new("xml")
                            .join(path.parent().unwrap_or_else(|| Path::new("")))
                            .join(format!("{id}.xml"));
                        CaseSpec::new(id, variant.to_string(), relative(output.to_string_lossy())?)
                            .map_err(message)
                    })
                    .collect::<Result<Vec<_>>>()?;
                FixtureSpec::new(
                    name,
                    relative(format!("html/{}", file.as_str()))?,
                    cases,
                    status,
                )
                .map_err(message)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut reports = vec![
            ReportScope::new(
                relative(format!(
                    "generation-reports/{}",
                    self.generation_reports.full.file
                ))?,
                None,
            )
            .map_err(message)?,
        ];
        for scope in &self.generation_reports.scoped {
            reports.push(
                ReportScope::new(
                    relative(format!("generation-reports/{}", scope.file))?,
                    Some(relative(format!("html/{}", scope.filter))?),
                )
                .map_err(message)?
                .with_expected_generated(scope.generated)
                .map_err(message)?,
            );
        }
        let corpus = BrowserCorpus::new(
            browser_location,
            relative("corpus.toml")?,
            "surgeist-layout-generate".to_string(),
            relative("xml")?,
            reports,
            self.browser.settings()?,
            fixtures,
            vec![
                relative("scripts/gentest/test_helper.js")?,
                relative("scripts/gentest/test_base_style.css")?,
                relative("fonts/Ahem.ttf")?,
                relative("fonts/PROVENANCE.md")?,
            ],
            vec![relative(SOURCE_ATTESTATION)?],
        )
        .map_err(message)?
        .with_expected_counts(self.expected_counts()?)
        .with_exact_input_roots(vec![relative("scripts/gentest")?, relative("fonts")?])
        .map_err(message)?;
        Ok(corpus)
    }

    fn expected_counts(&self) -> Result<browser::BrowserReportSummary> {
        let expected = &self.generation_reports.full;
        browser::BrowserReportSummary::new(
            expected.generated,
            expected.unsupported,
            expected.expected_fail,
            expected.quarantined,
            expected.failed_to_generate,
        )
        .map_err(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_corpus_keeps_fixture_variants_in_its_output_root() {
        let mut manifest: Manifest = surgeist_generator::parse_manifest(
            include_str!("../../layout/browser_parity/corpus.toml"),
            "corpus.toml",
        )
        .unwrap();
        manifest.cases.clear();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let corpus = manifest
            .corpus(
                BrowserLocation::new(root, root).unwrap(),
                vec![RelativePath::new("grid/example.html").unwrap()],
            )
            .unwrap();
        assert_eq!(corpus.output_root().as_str(), "live-xml");
        let fixture = &corpus.fixtures()[0];
        assert_eq!(fixture.source().as_str(), "html/grid/example.html");
        let cases = fixture
            .cases()
            .iter()
            .map(|case| (case.id(), case.variant(), case.output().as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            cases,
            [
                (
                    "example__border_box_ltr",
                    "border_box_ltr",
                    "live-xml/grid/example__border_box_ltr.xml"
                ),
                (
                    "example__content_box_ltr",
                    "content_box_ltr",
                    "live-xml/grid/example__content_box_ltr.xml"
                ),
                (
                    "example__border_box_rtl",
                    "border_box_rtl",
                    "live-xml/grid/example__border_box_rtl.xml"
                ),
                (
                    "example__content_box_rtl",
                    "content_box_rtl",
                    "live-xml/grid/example__content_box_rtl.xml"
                ),
            ]
        );
    }
}
