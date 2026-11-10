//! Typed source catalogue shared by capture publication and portable validation.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use serde::{Deserialize, Serialize};
use surgeist_generator::{RelativePath, Sha256Digest};

pub(crate) use super::capture_authority::SpecializedProfile;
use crate::fixture_variants::fixture_cases;

pub(crate) struct Declaration {
    pub(crate) records: Vec<Record>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalogue {
    schema_version: u8,
    records: Vec<Record>,
    #[serde(default)]
    semantic_publications: Vec<SemanticPublication>,
    #[serde(default)]
    specialized_publications: Vec<SpecializedPublication>,
    #[serde(default)]
    matching_publications: Vec<MatchingPublication>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SemanticPublication {
    source: RelativePath,
    replaces_sha256: Option<Sha256Digest>,
    specifications: BTreeMap<String, SemanticReference>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchingPublication {
    edition: String,
    sources: Vec<RelativePath>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpecializedPublication {
    source: RelativePath,
    profile: SpecializedProfile,
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
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub(crate) matching_publication: Option<String>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        block_offset: Option<f64>,
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
                validate_semantic_references(specifications, &variants, &mut semantic_paths)?;
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
        let mut annotated = BTreeSet::new();
        for SemanticPublication {
            source,
            replaces_sha256,
            specifications,
        } in catalogue.semantic_publications
        {
            let record = catalogue
                .records
                .iter_mut()
                .find(|record| record.source == source)
                .ok_or_else(|| {
                    format!(
                        "semantic publication requires a declared source: {}",
                        source.as_str()
                    )
                })?;
            if !annotated.insert(source)
                || !matches!(record.source_kind, SourceKind::Original)
                || record.relation.is_some()
                || specifications.len() != variants.len()
            {
                return Err(format!(
                    "semantic publication requires a unique unpaired original and all variants: {}",
                    record.source.as_str()
                )
                .into());
            }
            match (&record.semantic_specifications, replaces_sha256) {
                (None, None) => {}
                (Some(previous), Some(pin))
                    if previous.len() == variants.len()
                        && Sha256Digest::from_bytes(serde_json::to_vec(previous)?) == pin => {}
                _ => {
                    return Err(format!(
                        "semantic publication predecessor differs: {}",
                        record.source.as_str()
                    )
                    .into());
                }
            }
            validate_semantic_references(&specifications, &variants, &mut semantic_paths)?;
            record.semantic_specifications = Some(specifications);
        }
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
                || !profile.sources().contains(&record.source.as_str())
            {
                return Err(format!(
                    "specialized publication requires an original source within its declared profile: {}",
                    source.as_str()
                )
                .into());
            }
            record.specialized_publication = Some(profile);
        }
        let mut editions = BTreeSet::new();
        for publication in catalogue.matching_publications {
            super::capture_authority::validate_matching_edition(&publication.edition)?;
            if publication.sources.is_empty() || !editions.insert(publication.edition.clone()) {
                return Err("matching publication requires a unique nonempty edition".into());
            }
            for source in publication.sources {
                let record = catalogue
                    .records
                    .iter_mut()
                    .find(|record| record.source == source)
                    .ok_or("matching publication requires a declared source")?;
                if !matches!(record.source_kind, SourceKind::Original)
                    || !matches!(
                        record.expected,
                        ExpectedDisposition::Generated {
                            metadata: ExpectedMetadata::AbsoluteLocal {
                                root_size: None,
                                ..
                            }
                        }
                    )
                    || record.relation.is_some()
                    || record.semantic_specifications.is_some()
                    || record.specialized_publication.is_some()
                    || record.matching_publication.is_some()
                {
                    return Err("matching publication requires unique unpaired original box sources without other authority".into());
                }
                record.matching_publication = Some(publication.edition.clone());
            }
        }
        Ok(Self {
            records: catalogue.records,
        })
    }
    pub(crate) fn check_live_records(
        &self,
        selected: &BTreeMap<String, serde_json::Value>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let matching_editions: BTreeSet<_> = selected
            .values()
            .filter_map(|record| {
                record
                    .get("matching_publication")
                    .and_then(serde_json::Value::as_str)
            })
            .collect();
        let mut matched = 0;
        let mut uncovered = Vec::new();
        for record in &self.records {
            let source = record.source.as_str();
            if let Some(expected) = selected.get(source) {
                if &serde_json::to_value(record)? != expected {
                    return Err(format!("selected live record changed: {source}").into());
                }
                matched += 1;
            } else if record
                .matching_publication
                .as_deref()
                .is_some_and(|edition| matching_editions.contains(edition))
            {
                return Err("selected matching edition membership changed".into());
            } else if record.specialized_publication.is_some()
                || record.matching_publication.is_some()
            {
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

fn validate_semantic_references(
    specifications: &BTreeMap<String, SemanticReference>,
    variants: &[&str],
    paths: &mut BTreeSet<RelativePath>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for (variant, reference) in specifications {
        if !variants.contains(&variant.as_str())
            || !reference.path.as_str().starts_with("semantic/")
        {
            return Err(
                "semantic specification requires a registered variant and semantic/ input path"
                    .into(),
            );
        }
        if !paths.insert(reference.path.clone()) {
            return Err(format!(
                "duplicate semantic specification path: {}",
                reference.path.as_str()
            )
            .into());
        }
    }
    Ok(())
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

/// Shared issuer/portable validation of every retained historical observation.
pub(crate) fn verify_historical_geometry(
    current: &str,
    historical: &str,
    name: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let current = roxmltree::Document::parse(current)?;
    let historical = roxmltree::Document::parse(historical)?;
    for document in [&current, &historical] {
        if document.root_element().attribute("name") != Some(name) {
            return Err("historical case association changed".into());
        }
    }
    if current.root_element().attribute("use-rounding")
        != historical.root_element().attribute("use-rounding")
    {
        return Err("historical rounding changed".into());
    }
    compare_observation_tree(
        current
            .root_element()
            .children()
            .find(|n| n.has_tag_name("expectations"))
            .ok_or("missing current expectations")?,
        historical
            .root_element()
            .children()
            .find(|n| n.has_tag_name("expectations"))
            .ok_or("missing historical expectations")?,
    )
}

pub(crate) fn compare_observation_tree(
    a: roxmltree::Node<'_, '_>,
    b: roxmltree::Node<'_, '_>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let attributes = |node: roxmltree::Node<'_, '_>| {
        node.attributes()
            .map(|a| {
                (
                    (a.namespace().map(str::to_owned), a.name().to_owned()),
                    a.value().to_owned(),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };
    if a.tag_name() != b.tag_name()
        || attributes(a) != attributes(b)
        || [a, b].iter().any(|node| {
            node.children()
                .filter(roxmltree::Node::is_text)
                .any(|text| text.text().is_some_and(|s| !s.trim().is_empty()))
        })
    {
        return Err("historical observation fields changed".into());
    }
    let a: Vec<_> = a.children().filter(roxmltree::Node::is_element).collect();
    let b: Vec<_> = b.children().filter(roxmltree::Node::is_element).collect();
    if a.len() != b.len() {
        return Err("historical observation cardinality changed".into());
    }
    for (a, b) in a.into_iter().zip(b) {
        compare_observation_tree(a, b)?;
    }
    Ok(())
}

/// Shared declared absolute-root certificate checks. The Golden parser separately
/// validates the protocol shape and all finite fallback/root-context values.
pub(crate) fn verify_absolute_metadata(
    xml: &str,
    metadata: &ExpectedMetadata,
    name: &str,
    variant: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ExpectedMetadata::AbsoluteLocal {
        hypothetical_display,
        containing_direction,
        hypothetical_direction,
        root_direction,
        root_writing_mode,
        inline_offset,
        block_offset,
        ..
    } = metadata
    else {
        return Err("matching geometry requires absolute-local metadata".into());
    };
    let doc = roxmltree::Document::parse(xml)?;
    let test = doc.root_element();
    let viewport = test
        .children()
        .find(|n| n.has_tag_name("viewport"))
        .ok_or("missing viewport")?;
    let root = test
        .children()
        .find(|n| n.has_tag_name("input"))
        .and_then(|n| n.children().find(roxmltree::Node::is_element))
        .ok_or("missing root input")?;
    let direction = if root_direction == "variant" {
        if variant.ends_with("_rtl") {
            "rtl"
        } else {
            "ltr"
        }
    } else {
        root_direction.as_str()
    };
    if !test.has_tag_name("test")
        || test.attribute("name") != Some(name)
        || test.attribute("measurement-version") != Some("4")
        || test.attribute("observation-frame") != Some("root-border-local")
        || viewport.attribute("root-context") != Some("absolute-root-local")
        || viewport.attribute("hypothetical-display") != Some(hypothetical_display.as_str())
        || viewport.attribute("containing-direction") != Some(containing_direction.as_str())
        || viewport.attribute("hypothetical-direction") != Some(hypothetical_direction.as_str())
        || viewport.attribute("hypothetical-anchor") != Some("inline-start")
        || root.attribute("position").unwrap_or("relative") != "absolute"
        || root.attribute("direction") != Some(direction)
        || root.attribute("writing-mode").unwrap_or("horizontal-tb") != root_writing_mode
    {
        return Err("declared root capture metadata changed".into());
    }
    fn px(node: roxmltree::Node<'_, '_>, name: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let value: f64 = node
            .attribute(name)
            .and_then(|s| s.strip_suffix("px"))
            .ok_or("missing root pixel attribute")?
            .parse()?;
        if !value.is_finite() {
            return Err("nonfinite root pixel attribute".into());
        }
        Ok(value)
    }
    if !inline_offset.is_finite() || px(viewport, "hypothetical-inline-offset")? != *inline_offset {
        return Err("root inline offset changed".into());
    }
    match block_offset {
        Some(offset)
            if offset.is_finite() && px(viewport, "hypothetical-block-offset")? == *offset => {}
        None if viewport.attribute("hypothetical-block-offset").is_none() => {}
        _ => return Err("root block offset changed".into()),
    }
    for (containing, fallback) in [
        ("containing-width", "fallback-icb-width"),
        ("containing-height", "fallback-icb-height"),
    ] {
        if px(viewport, containing)? != px(viewport, fallback)? {
            return Err("root containing/fallback extent differs".into());
        }
    }
    Ok(())
}

impl Record {
    pub(crate) fn verify_matching_capture(
        &self,
        current: &str,
        historical: &str,
        name: &str,
        variant: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ExpectedDisposition::Generated { metadata } = &self.expected else {
            return Err("matching output must be Generated".into());
        };
        verify_historical_geometry(current, historical, name)?;
        verify_absolute_metadata(current, metadata, name, variant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unannotated_catalogue() -> &'static [u8] {
        let live = include_str!("../../../examples/qualify-root-context/corpus.toml");
        live.split("\n[[specialized_publications]]")
            .next()
            .expect("catalogue prefix")
            .as_bytes()
    }
    const SPECIALIZED: &str = r#"

[[specialized_publications]]
source = "html/block/prepared_text_continuity_bidi.html"
profile = "prepared-range"

[[specialized_publications]]
source = "html/flex/fri07_flex_composition.html"
profile = "flex-composition"
"#;

    #[test]
    fn semantic_publication_supersedes_only_its_exact_predecessor() {
        const SOURCE: &str = "html/flex/fri07_collapsed_strut_wrapping.html";
        let prefix = unannotated_catalogue();
        let prior = Declaration::parse(prefix).unwrap();
        let old = prior
            .records
            .iter()
            .find(|record| record.source.as_str() == SOURCE)
            .unwrap();
        let old_specs = old.semantic_specifications.as_ref().unwrap();
        let digest = Sha256Digest::from_bytes(serde_json::to_vec(old_specs).unwrap());
        let mut appendix = format!(
            "\n[[semantic_publications]]\nsource = {SOURCE:?}\nreplaces_sha256 = {:?}\n",
            digest.as_str(),
        );
        for (variant, _) in fixture_cases() {
            appendix.push_str(&format!(
                "[semantic_publications.specifications.{variant}]\npath = \"semantic/successor-{variant}.json\"\nsha256 = {:?}\n",
                old_specs[variant].sha256.as_str()
            ));
        }
        let parse = |suffix: &str| Declaration::parse(&[prefix, suffix.as_bytes()].concat());
        let active = parse(&appendix).unwrap();
        let successor = active
            .records
            .iter()
            .find(|record| record.source.as_str() == SOURCE)
            .unwrap();
        assert_eq!(successor.historical_sha256, old.historical_sha256);
        assert_eq!(successor.source_sha256, old.source_sha256);
        let successor = successor.semantic_specifications.as_ref().unwrap();
        assert_eq!(successor.len(), 4);
        assert_eq!(
            successor["border_box_ltr"].path.as_str(),
            "semantic/successor-border_box_ltr.json"
        );
        assert!(parse(&appendix.replace(digest.as_str(), &"0".repeat(64))).is_err());
        assert!(parse(&appendix.repeat(2)).is_err());
        assert!(
            parse(&appendix.replace(
                "semantic/successor-border_box_ltr.json",
                old_specs["border_box_ltr"].path.as_str()
            ))
            .is_err()
        );
    }

    #[test]
    fn appended_publications_require_all_specialized_variants() {
        let prior = Declaration::parse(unannotated_catalogue()).unwrap();
        let appended =
            Declaration::parse(&[unannotated_catalogue(), SPECIALIZED.as_bytes()].concat())
                .unwrap();
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
        let catalogue = Declaration::parse(unannotated_catalogue()).unwrap();
        let legacy = catalogue
            .records
            .iter()
            .find(|record| record.semantic_specifications.is_some())
            .unwrap()
            .source
            .as_str();
        let inline = std::str::from_utf8(unannotated_catalogue())
            .unwrap()
            .replacen(
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
            assert!(
                Declaration::parse(&[unannotated_catalogue(), appendix.as_bytes()].concat())
                    .is_err()
            );
        }
    }

    const MATCHING: &str = r#"

[[matching_publications]]
edition = "box-controls"
sources = [
  "html/grid/grid_fit_content_points_argument.html",
  "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
]
"#;

    #[test]
    fn matching_publications_bind_named_original_cohorts_and_report_uncovered_variants() {
        let prior = Declaration::parse(unannotated_catalogue()).unwrap();
        let appended = Declaration::parse(&[unannotated_catalogue(), MATCHING.as_bytes()].concat())
            .expect("a named matching cohort of complete original sources must be admitted");
        let mut selected = BTreeMap::new();
        for record in prior
            .records
            .iter()
            .filter(|record| record.semantic_specifications.is_some())
        {
            let projection = serde_json::to_value(record).unwrap();
            assert!(projection.get("matching_publication").is_none());
            selected.insert(record.source.as_str().to_owned(), projection);
        }
        let expected: BTreeSet<_> = [
            "html/grid/grid_fit_content_points_argument.html",
            "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
        ]
        .into_iter()
        .flat_map(|source| fixture_cases().map(move |(variant, _)| format!("{source} ({variant})")))
        .collect();
        assert_eq!(
            appended
                .check_live_records(&selected)
                .unwrap()
                .into_iter()
                .collect::<BTreeSet<_>>(),
            expected
        );
        for source in [
            "html/grid/grid_fit_content_points_argument.html",
            "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
        ] {
            let record = appended
                .records
                .iter()
                .find(|record| record.source.as_str() == source)
                .unwrap();
            let projection = serde_json::to_value(record).unwrap();
            assert_eq!(projection["matching_publication"], "box-controls");
            selected.insert(source.to_owned(), projection);
        }
        assert!(appended.check_live_records(&selected).unwrap().is_empty());
        assert!(prior.check_live_records(&selected).is_err());
        selected
            .get_mut("html/grid/grid_fit_content_points_argument.html")
            .unwrap()["matching_publication"] = serde_json::json!("other-controls");
        assert!(appended.check_live_records(&selected).is_err());
    }

    #[test]
    fn matching_publications_accept_authored_edition_names_without_fixed_owner_variants() {
        for edition in ["box-controls", "independent-box-controls"] {
            let input = [
                unannotated_catalogue(),
                MATCHING.replace("box-controls", edition).as_bytes(),
            ]
            .concat();
            let parsed =
                Declaration::parse(&input).expect("the catalogue owns matching edition identity");
            let record = parsed
                .records
                .iter()
                .find(|record| {
                    record.source.as_str() == "html/grid/grid_fit_content_points_argument.html"
                })
                .unwrap();
            assert_eq!(
                serde_json::to_value(record).unwrap()["matching_publication"],
                edition
            );
        }
    }

    #[test]
    fn matching_publications_reject_ambiguous_membership_and_other_capture_authority() {
        let prior = Declaration::parse(unannotated_catalogue()).unwrap();
        let semantic = prior
            .records
            .iter()
            .find(|record| record.semantic_specifications.is_some())
            .unwrap()
            .source
            .as_str();
        for appendix in [
            MATCHING.repeat(2),
            MATCHING.replace("box-controls", "../escaped"),
            MATCHING.replace("box-controls", ""),
            MATCHING.replace(
                "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
                "html/grid/grid_fit_content_points_argument.html",
            ),
            MATCHING.replace(
                "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
                "html/missing.html",
            ),
            MATCHING.replace(
                "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
                "html/qualification/fixed-grid.html",
            ),
            MATCHING.replace(
                "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
                semantic,
            ),
            format!(
                "{MATCHING}{}",
                MATCHING.replace("box-controls", "another-controls")
            ),
            format!(
                "{SPECIALIZED}{}",
                MATCHING.replace(
                    "html/subgrid/subgrid_standalone_axis_column_autoflow.html",
                    "html/block/prepared_text_continuity_bidi.html"
                )
            ),
        ] {
            assert!(
                Declaration::parse(&[unannotated_catalogue(), appendix.as_bytes()].concat())
                    .is_err(),
                "{appendix}"
            );
        }
    }
    #[test]
    fn matching_capture_revalidation_rejects_changed_history_and_declared_root_metadata() {
        const SOURCE: &str = "html/flex/fri07_collapsed_strut_wrapping.html";
        const NAME: &str = "fri07_collapsed_strut_wrapping__border_box_ltr";
        let catalogue = Declaration::parse(unannotated_catalogue()).unwrap();
        let record = catalogue
            .records
            .iter()
            .find(|r| r.source.as_str() == SOURCE)
            .unwrap();
        let current = include_str!(
            "captured/canonical-box-differences/xml/flex/fri07_collapsed_strut_wrapping__border_box_ltr.xml"
        );
        let historical =
            include_str!("xml/flex/fri07_collapsed_strut_wrapping__border_box_ltr.xml");
        record
            .verify_matching_capture(current, historical, NAME, "border_box_ltr")
            .unwrap();
        let changed_context = current.replace(
            "hypothetical-display=\"flex\"",
            "hypothetical-display=\"grid\"",
        );
        assert_ne!(changed_context, current);
        assert!(
            record
                .verify_matching_capture(&changed_context, historical, NAME, "border_box_ltr")
                .is_err()
        );
        let (prefix, observations) = historical.split_once("<expectations>").unwrap();
        let changed = format!(
            "{prefix}<expectations>{}",
            observations.replacen("<node ", "<node extra-observation=\"changed\" ", 1)
        );
        assert_ne!(changed, historical);
        assert!(
            record
                .verify_matching_capture(current, &changed, NAME, "border_box_ltr")
                .is_err()
        );
        let dropped = historical.replacen(" x=\"0\"", "", 1);
        assert_ne!(dropped, historical);
        assert!(
            record
                .verify_matching_capture(current, &dropped, NAME, "border_box_ltr")
                .is_err()
        );
        assert!(
            record
                .verify_matching_capture(current, historical, NAME, "border_box_rtl")
                .is_err()
        );
    }
    #[test]
    fn matching_live_membership_rejects_same_edition_growth_and_reports_new_editions() {
        const EXTRA: &str = "html/subgrid/subgrid_overflow_hidden_does_not_prohibit.html";
        let old =
            Declaration::parse(&[unannotated_catalogue(), MATCHING.as_bytes()].concat()).unwrap();
        let selected: BTreeMap<_, _> = old
            .records
            .iter()
            .filter(|record| {
                record.semantic_specifications.is_some() || record.matching_publication.is_some()
            })
            .map(|record| {
                (
                    record.source.as_str().to_owned(),
                    serde_json::to_value(record).unwrap(),
                )
            })
            .collect();
        assert!(old.check_live_records(&selected).unwrap().is_empty());
        let growth = MATCHING.replace("sources = [", &format!("sources = [\n  {EXTRA:?},"));
        let growth =
            Declaration::parse(&[unannotated_catalogue(), growth.as_bytes()].concat()).unwrap();
        assert!(growth.check_live_records(&selected).is_err());
        let other = format!(
            "{MATCHING}\n[[matching_publications]]\nedition = \"later-controls\"\nsources = [{EXTRA:?}]\n"
        );
        let other =
            Declaration::parse(&[unannotated_catalogue(), other.as_bytes()].concat()).unwrap();
        let uncovered = other
            .check_live_records(&selected)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            uncovered,
            fixture_cases()
                .map(|(variant, _)| format!("{EXTRA} ({variant})"))
                .into_iter()
                .collect()
        );
    }
}
