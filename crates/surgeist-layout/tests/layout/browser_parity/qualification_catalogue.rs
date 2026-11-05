//! Typed source catalogue shared by capture publication and portable validation.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use serde::{Deserialize, Serialize};
use surgeist_generator::{RelativePath, Sha256Digest};

use super::fixture_variants::fixture_cases;

pub(crate) struct Declaration {
    pub(crate) records: Vec<Record>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalogue {
    schema_version: u8,
    records: Vec<Record>,
    #[serde(default)]
    specialized_publications: Vec<SpecializedPublication>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpecializedPublication {
    source: RelativePath,
    profile: SpecializedProfile,
}

#[derive(Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SpecializedProfile {
    PreparedRange,
    FlexComposition,
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
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) specialized_publication: Option<SpecializedProfile>,
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
        let mut catalogue: Catalogue =
            surgeist_generator::parse_manifest(std::str::from_utf8(bytes)?, "corpus.toml")?;
        if catalogue.schema_version != 3 {
            return Err("qualification declaration schema_version must be 3".into());
        }
        let variants = fixture_cases().map(|(variant, _)| variant);
        let mut sources = BTreeSet::new();
        let mut stems = BTreeMap::new();
        let mut semantic_paths = BTreeSet::new();
        for record in &catalogue.records {
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
        for record in &catalogue.records {
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
        if catalogue.records.is_empty() {
            return Err("root-context qualification requires source records".into());
        }
        let mut profiles = BTreeSet::new();
        for SpecializedPublication { source, profile } in catalogue.specialized_publications {
            let record = catalogue
                .records
                .iter_mut()
                .find(|record| record.source == source)
                .ok_or_else(|| {
                    format!(
                        "specialized publication requires a declared source: {}",
                        source.as_str()
                    )
                })?;
            if !matches!(record.source_kind, SourceKind::Original)
                || record.semantic_specifications.is_some()
                || record.relation.is_some()
                || record.specialized_publication.is_some()
                || !profiles.insert(profile)
            {
                return Err(format!(
                    "specialized publication requires a unique original source and profile: {}",
                    source.as_str()
                )
                .into());
            }
            record.specialized_publication = Some(profile);
        }
        Ok(Self {
            records: catalogue.records,
        })
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
            } else if record.specialized_publication.is_some() {
                uncovered
                    .extend(fixture_cases().map(|(variant, _)| format!("{source} ({variant})")));
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

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE: &[u8] = include_bytes!("../../../examples/qualify-root-context/corpus.toml");
    const SPECIALIZED: &str = r#"

[[specialized_publications]]
source = "html/block/prepared_text_continuity_bidi.html"
profile = "prepared-range"

[[specialized_publications]]
source = "html/flex/fri07_flex_composition.html"
profile = "flex-composition"
"#;

    #[test]
    fn appended_publications_require_all_specialized_variants() {
        let prior = Declaration::parse(LIVE).unwrap();
        let appended = Declaration::parse(&[LIVE, SPECIALIZED.as_bytes()].concat()).unwrap();
        let mut selected = BTreeMap::new();
        for record in prior
            .records
            .iter()
            .filter(|record| record.semantic_specifications.is_some())
        {
            selected.insert(
                record.source.as_str().to_owned(),
                serde_json::to_value(record).unwrap(),
            );
        }

        let expected: BTreeSet<_> = [
            "html/block/prepared_text_continuity_bidi.html",
            "html/flex/fri07_flex_composition.html",
        ]
        .into_iter()
        .flat_map(|source| fixture_cases().map(move |(variant, _)| format!("{source} ({variant})")))
        .collect();
        let uncovered = appended.check_live_records(&selected).unwrap();
        assert_eq!(uncovered.into_iter().collect::<BTreeSet<_>>(), expected);

        let record = appended
            .records
            .iter()
            .find(|record| {
                record.source.as_str() == "html/block/prepared_text_continuity_bidi.html"
            })
            .unwrap();
        let projection = serde_json::to_value(record).unwrap();
        assert_eq!(projection["specialized_publication"], "prepared-range");
        selected.insert(record.source.as_str().to_owned(), projection);
        let uncovered = appended.check_live_records(&selected).unwrap();
        assert_eq!(uncovered.len(), 4);
        assert!(
            uncovered
                .iter()
                .all(|case| case.contains("fri07_flex_composition"))
        );
        assert!(prior.check_live_records(&selected).is_err());
    }

    #[test]
    fn specialized_publications_cannot_shadow_or_reuse_authority() {
        let catalogue = Declaration::parse(LIVE).unwrap();
        let legacy = catalogue
            .records
            .iter()
            .find(|record| record.semantic_specifications.is_some())
            .unwrap()
            .source
            .as_str();
        let inline = std::str::from_utf8(LIVE).unwrap().replacen(
            "[[records]]\n",
            "[[records]]\nspecialized_publication = \"prepared-range\"\n",
            1,
        );
        assert!(Declaration::parse(inline.as_bytes()).is_err());
        for appendix in [
            SPECIALIZED.repeat(2),
            SPECIALIZED.replace("flex-composition", "prepared-range"),
            SPECIALIZED.replace("html/flex/fri07_flex_composition.html", "html/missing.html"),
            SPECIALIZED.replace("html/flex/fri07_flex_composition.html", legacy),
            SPECIALIZED.replace(
                "html/flex/fri07_flex_composition.html",
                "html/qualification/fixed-grid.html",
            ),
        ] {
            assert!(Declaration::parse(&[LIVE, appendix.as_bytes()].concat()).is_err());
        }
    }
}
