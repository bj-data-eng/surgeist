//! Typed source catalogue shared by capture publication and portable validation.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use serde::{Deserialize, Serialize};
use surgeist_generator::{RelativePath, Sha256Digest};

use super::fixture_variants::fixture_cases;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Declaration {
    schema_version: u8,
    pub(crate) records: Vec<Record>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub(crate) source: RelativePath,
    pub(crate) source_sha256: Sha256Digest,
    pub(crate) source_kind: SourceKind,
    pub(crate) expected: ExpectedDisposition,
    #[serde(default)]
    pub(crate) historical_sha256: BTreeMap<String, Sha256Digest>,
    pub(crate) relation: Option<PairedObservation>,
    pub(crate) semantic_specifications: Option<BTreeMap<String, SemanticReference>>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SemanticReference {
    pub(crate) path: RelativePath,
    pub(crate) sha256: Sha256Digest,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceKind {
    Original,
    Authored,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ExpectedDisposition {
    Generated { metadata: ExpectedMetadata },
    Unsupported { reason: String },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "context", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ExpectedMetadata {
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
        atomic_paragraphs: Option<usize>,
        grid_reference_owners: Option<Vec<String>>,
    },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ExpectedRootSize {
    Fixed { width: f64, height: f64 },
    FlexPair { item_width: f64, item_height: f64 },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PairedObservation {
    pub(crate) peer: String,
    pub(crate) world_phase_change: bool,
}
impl Declaration {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let declaration: Self =
            surgeist_generator::parse_manifest(std::str::from_utf8(bytes)?, "corpus.toml")?;
        if declaration.schema_version != 3 {
            return Err("qualification declaration schema_version must be 3".into());
        }
        let variants = fixture_cases().map(|(variant, _)| variant);
        let mut sources = BTreeSet::new();
        let mut stems = BTreeMap::new();
        let mut semantic_paths = BTreeSet::new();
        for record in &declaration.records {
            if !sources.insert(record.source.clone()) {
                return Err(
                    format!("duplicate qualification source: {}", record.source.as_str()).into(),
                );
            }
            let (_, stem) = record.source_names()?;
            let generated = matches!(&record.expected, ExpectedDisposition::Generated { .. });
            if stems.insert(stem, generated).is_some() {
                return Err(format!("duplicate qualification case stem: {stem}").into());
            }
            match record.source_kind {
                SourceKind::Original => {
                    if !generated
                        || record.historical_sha256.len() != variants.len()
                        || variants
                            .iter()
                            .any(|variant| !record.historical_sha256.contains_key(*variant))
                    {
                        return Err(format!("original source requires a Generated disposition and every historical variant: {}", record.source.as_str()).into());
                    }
                }
                SourceKind::Authored if !record.historical_sha256.is_empty() => {
                    return Err(format!(
                        "authored source cannot declare historical variants: {}",
                        record.source.as_str()
                    )
                    .into());
                }
                SourceKind::Authored => {}
            }
            if record.relation.is_some() && !generated {
                return Err(format!(
                    "paired source requires a Generated disposition: {}",
                    record.source.as_str()
                )
                .into());
            }
            if let Some(specifications) = &record.semantic_specifications {
                if specifications.is_empty() || !generated {
                    return Err(
                        "semantic specifications require a nonempty Generated variant map".into(),
                    );
                }
                for (variant, reference) in specifications {
                    if !variants.contains(&variant.as_str())
                        || !reference.path.as_str().starts_with("semantic/")
                    {
                        return Err("semantic specification requires a registered variant and semantic/ input path".into());
                    }
                    if !semantic_paths.insert(reference.path.as_str()) {
                        return Err(format!(
                            "duplicate semantic specification path: {}",
                            reference.path.as_str()
                        )
                        .into());
                    }
                }
            }
        }
        for record in &declaration.records {
            if let Some(relation) = &record.relation
                && stems.get(relation.peer.as_str()) != Some(&true)
            {
                return Err(format!(
                    "paired source requires a declared Generated peer: {}",
                    relation.peer
                )
                .into());
            }
        }
        if declaration.records.is_empty() {
            return Err("root-context qualification requires source records".into());
        }
        Ok(declaration)
    }
    pub(crate) fn check_live_records(
        &self,
        selected: &BTreeMap<String, serde_json::Value>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut matched = 0;
        let mut uncovered = Vec::new();
        for record in &self.records {
            let source = record.source.as_str();
            if let Some(expected) = selected.get(source) {
                if &serde_json::to_value(record)? != expected {
                    return Err(format!("selected live record changed: {source}").into());
                }
                matched += 1;
            } else if let Some(references) = &record.semantic_specifications {
                uncovered.extend(
                    references
                        .keys()
                        .map(|variant| format!("{source} ({variant})")),
                );
            }
        }
        if matched != selected.len() {
            return Err("selected live record missing".into());
        }
        Ok(uncovered)
    }
}

impl Record {
    pub(crate) fn source_names(&self) -> Result<(&str, &str), Box<dyn Error + Send + Sync>> {
        let family = self
            .source
            .as_str()
            .strip_prefix("html/")
            .and_then(|path| path.strip_suffix(".html"))
            .ok_or("qualification source must be an html/*.html path")?;
        let stem = family
            .rsplit('/')
            .next()
            .filter(|stem| !stem.is_empty())
            .ok_or("missing source stem")?;
        Ok((family, stem))
    }
}
