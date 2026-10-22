//! Support-catalog contracts.
//!
//! Structural invariants hold for every catalog record: the status decides
//! which boundary fields are present, every cited source is the registered
//! source with that identity, every schema property owns exactly one row, and
//! every production anchor exists in the retained reference copy of its
//! source. Named records are checked against [`expected::EXPECTED`] and its
//! public parser vectors; the remaining tests pair records with the authored
//! grammar they claim.

use std::collections::{BTreeMap, BTreeSet};

use surgeist_css::CssTimeLiteral;
use surgeist_css::*;

mod expected;

use expected::{EXPECTED, Input};

#[test]
fn every_record_reports_exactly_the_boundary_fields_of_its_status() {
    for record in feature_catalog() {
        let id = record.id().as_str();
        let non_empty = |text: Option<&str>| text.is_some_and(|text| !text.trim().is_empty());
        match record.status() {
            CssSupportStatus::Complete => {
                assert_eq!(record.supported_subset(), None, "{id} subset");
                assert_eq!(record.unsupported_remainder(), None, "{id} remainder");
                assert_eq!(record.recognized_unsupported_code(), None, "{id} code");
            }
            CssSupportStatus::Partial => {
                assert!(non_empty(record.supported_subset()), "{id} subset");
                assert!(non_empty(record.unsupported_remainder()), "{id} remainder");
                assert_eq!(record.recognized_unsupported_code(), None, "{id} code");
            }
            CssSupportStatus::RecognizedUnsupported => {
                assert_eq!(record.supported_subset(), None, "{id} subset");
                assert_eq!(record.unsupported_remainder(), None, "{id} remainder");
                assert!(record.recognized_unsupported_code().is_some(), "{id} code");
            }
        }
    }
}

#[test]
fn catalog_identities_are_unique_and_exact_lookup_returns_each_record() {
    let mut seen = BTreeSet::new();
    for record in feature_catalog() {
        let id = record.id().as_str();
        assert!(seen.insert(id), "duplicate catalog identity {id}");
        let found = feature_metadata(id).unwrap_or_else(|| panic!("lookup of {id}"));
        assert!(
            std::ptr::eq(found, record),
            "{id} lookup returns its record"
        );
        assert!(!record.spelling().is_empty(), "{id} spelling");
        assert!(!record.production().is_empty(), "{id} production");
    }
}

#[test]
fn every_record_cites_the_registered_source_with_its_identity() {
    let mut registry = BTreeMap::new();
    for source in specification_sources() {
        assert!(
            registry.insert(source.id().as_str(), source).is_none(),
            "duplicate source identity {}",
            source.id().as_str()
        );
    }
    for record in feature_catalog() {
        let source = record.source();
        let registered = registry
            .get(source.id().as_str())
            .unwrap_or_else(|| panic!("{} cites an unregistered source", record.id().as_str()));
        assert_eq!(**registered, source, "{} source", record.id().as_str());
    }
    for exclusion in conformance_exclusions() {
        let source = exclusion.source();
        let registered = registry
            .get(source.id().as_str())
            .unwrap_or_else(|| panic!("{} cites an unregistered source", exclusion.id().as_str()));
        assert_eq!(**registered, source, "{} source", exclusion.id().as_str());
    }
}

#[test]
fn every_schema_property_owns_exactly_one_catalog_row() {
    let properties = CssKnownProperty::all();
    let schema_rows = feature_catalog()
        .iter()
        .filter(|record| {
            record.kind() == CssFeatureKind::Property
                && property_support_metadata(record.spelling())
                    .is_some_and(|support| std::ptr::eq(support.feature(), *record))
        })
        .count();
    assert_eq!(schema_rows, properties.len());
    for &property in properties {
        let name = property.canonical_name();
        let support = property_support_metadata(name).unwrap_or_else(|| panic!("{name}"));
        let record = support.feature();
        assert_eq!(support.property(), property, "{name} owner");
        assert_eq!(support.canonical_name(), name, "{name} canonical name");
        assert_eq!(support.aliases(), property.aliases(), "{name} aliases");
        assert_eq!(
            record.id().as_str(),
            property.stable_id(),
            "{name} identity"
        );
        assert_eq!(record.kind(), CssFeatureKind::Property, "{name} kind");
        assert_eq!(record.spelling(), name, "{name} spelling");
        assert!(
            std::ptr::eq(feature_metadata(property.stable_id()).unwrap(), record),
            "{name} stable identity lookup"
        );
        assert!(
            std::ptr::eq(
                property_support_metadata(&name.to_ascii_uppercase())
                    .unwrap()
                    .feature(),
                record
            ),
            "{name} ASCII case-insensitive lookup"
        );
        for alias in property.aliases() {
            let aliased = property_support_metadata(alias).unwrap_or_else(|| panic!("{alias}"));
            assert!(
                std::ptr::eq(aliased.feature(), record),
                "{alias} alias owner"
            );
        }
    }
}

/// Retained reference copies keyed by the exact source snapshot they convert.
fn reference_copies() -> BTreeMap<String, String> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../references");
    let mut copies = BTreeMap::new();
    for entry in std::fs::read_dir(&directory).expect("retained references directory") {
        let path = entry.expect("reference entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("UTF-8 reference copy");
        let snapshot = text.lines().find_map(|line| {
            line.strip_prefix("Source snapshot: ")
                .or_else(|| line.strip_prefix("Source: "))
                .map(|value| {
                    value
                        .trim()
                        .trim_start_matches('<')
                        .trim_end_matches('>')
                        .to_owned()
                })
        });
        if let Some(snapshot) = snapshot {
            copies.insert(snapshot, text);
        }
    }
    copies
}

// Records whose cited anchor is absent from the retained copy of their source.
// Each is a catalog data defect reported for correction; the anchor invariant
// still holds for every other record.
const UNRESOLVED_PRODUCTION_ANCHORS: &[(&str, &str)] = &[
    // Pseudo 4 (2025-06-27) does not define ::backdrop; Position 4 section 3.2
    // owns `#selectordef-backdrop`, and the source registry has no Position 4
    // source to cite.
    ("ext.pseudo-element.backdrop", "#selectordef-backdrop"),
];

// The WebKit CSSProperties witness is retained as metadata only.
const METADATA_ONLY_SOURCES: &[&str] = &["I-WEBKIT-SVG-GLYPH"];

#[test]
fn every_production_anchor_exists_in_the_cited_reference_copy() {
    let copies = reference_copies();
    let mut unresolved = Vec::new();
    for record in feature_catalog() {
        let id = record.id().as_str();
        let source = record.source();
        if METADATA_ONLY_SOURCES.contains(&source.id().as_str()) {
            assert!(source.url().is_some(), "{id} metadata-only source URL");
            continue;
        }
        for anchor in record.production().split(',') {
            let (page, fragment) = anchor
                .split_once('#')
                .unwrap_or_else(|| panic!("{id} production `{anchor}` is not an anchor"));
            assert!(!fragment.is_empty(), "{id} empty anchor");
            let copy = match (source.url(), source.repository_provenance()) {
                (Some(url), None) => copies.get(&format!("{url}{page}")),
                (None, Some(provenance)) => {
                    // Repository provenance is `revision:path`; the anchor's page
                    // names that same path.
                    let (revision, path) = provenance.split_once(':').expect("revision:path");
                    assert!(page.is_empty() || page == path, "{id} page `{page}`");
                    copies
                        .iter()
                        .find(|(snapshot, _)| {
                            snapshot.contains(revision) && snapshot.ends_with(path)
                        })
                        .map(|(_, text)| text)
                }
                _ => panic!("{id} source has exactly one provenance form"),
            };
            let copy = copy.unwrap_or_else(|| panic!("{id} has no retained copy for `{anchor}`"));
            if !copy.contains(&format!("id=\"{fragment}\"")) {
                unresolved.push((id, anchor));
            }
        }
    }
    assert_eq!(unresolved, UNRESOLVED_PRODUCTION_ANCHORS);
}

#[test]
fn named_conformance_records_expose_declared_metadata() {
    for expected in EXPECTED {
        let actual = feature_metadata(expected.id).expect("expected catalog record");
        assert_eq!(actual.id().as_str(), expected.id);
        assert_eq!(actual.kind(), expected.kind, "{} kind", expected.id);
        if let Some(spelling) = expected.spelling {
            assert_eq!(actual.spelling(), spelling, "{} spelling", expected.id);
        }
        assert_eq!(
            actual.source().id().as_str(),
            expected.source,
            "{} source",
            expected.id
        );
        assert_eq!(
            actual.production(),
            expected.production,
            "{} production",
            expected.id
        );
        assert_eq!(actual.status(), expected.status, "{} status", expected.id);
    }

    assert!(
        feature_metadata("BASELINE.RULE.IMPORT").is_none(),
        "lookup is exact"
    );
    assert!(
        feature_metadata("baseline.rule.import ").is_none(),
        "lookup does not trim"
    );
    assert!(feature_metadata("baseline.property.display").is_some());
    assert!(feature_metadata("").is_none());
}

#[test]
fn conformance_catalog_vectors_cover_each_supported_and_unsupported_boundary() {
    for expected in EXPECTED {
        match expected.status {
            CssSupportStatus::Complete | CssSupportStatus::Partial => {
                let input = expected
                    .positive
                    .expect("supported record needs a positive vector");
                let diagnostics = diagnostics(input);
                assert!(
                    diagnostics.is_empty(),
                    "{} positive vector produced {diagnostics:?}",
                    expected.id
                );
            }
            CssSupportStatus::RecognizedUnsupported => {
                assert!(
                    expected.positive.is_none(),
                    "recognized unsupported has no positive vector"
                );
            }
        }

        if matches!(
            expected.status,
            CssSupportStatus::Partial | CssSupportStatus::RecognizedUnsupported
        ) {
            assert!(
                expected.negative.is_some(),
                "{} unsupported boundary needs a vector",
                expected.id
            );
        }
        if let Some((input, code)) = expected.negative {
            let diagnostics = diagnostics(input);
            assert!(
                !diagnostics.is_empty(),
                "{} negative vector was accepted",
                expected.id
            );
            assert_eq!(
                diagnostics[0].0, code,
                "{} negative vector root",
                expected.id
            );
            if expected.status == CssSupportStatus::RecognizedUnsupported {
                assert_eq!(
                    diagnostics[0].1,
                    Some(expected.id),
                    "{} diagnostic feature identity",
                    expected.id
                );
            }
        }
    }
}

fn diagnostics(input: Input) -> Vec<(CssErrorCode, Option<&'static str>)> {
    let diagnostics = match input {
        Input::Sheet(source) => parse_sheet(source).diagnostics().to_vec(),
        Input::Style(source) => parse_style_attribute(source).diagnostics().to_vec(),
    };
    diagnostics
        .iter()
        .map(|diagnostic| {
            let feature = match diagnostic.error().kind() {
                ErrorKind::UnsupportedAtRule(detail) => Some(detail.feature().as_str()),
                _ => None,
            };
            (diagnostic.error().code(), feature)
        })
        .collect()
}

/// Asserts that one authored declaration is retained cleanly with the given identity.
fn assert_single_known_declaration(source: &str, id: &str) {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("{source}: expected one retained declaration");
    };
    let known = declaration.known().expect("known declaration");
    assert_eq!(known.property().stable_id(), id, "{source} identity");
}

/// Asserts that an authored declaration outside its property grammar is dropped.
fn assert_dropped_declaration(source: &str) {
    let report = parse_style_attribute(source);
    assert!(report.syntax().is_empty(), "{source}");
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{source}: expected one diagnostic: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue,
        "{source}"
    );
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{source}"
    );
}

#[test]
fn removed_track_properties_have_no_current_catalogue_metadata() {
    for id in [
        "baseline.property.align-tracks",
        "baseline.property.justify-tracks",
    ] {
        assert_eq!(feature_metadata(id), None);
        assert!(
            feature_catalog()
                .iter()
                .all(|feature| feature.id().as_str() != id)
        );
    }
    for id in [
        "baseline.property.align-content",
        "baseline.property.justify-content",
    ] {
        let metadata = feature_metadata(id).unwrap();
        assert_eq!(metadata.source().id().as_str(), "S-ALIGN3");
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
    }
}

#[test]
fn normalization_retains_current_alignment_contributions_after_obsolete_property_recovery() {
    let report = parse_sheet(
        ".x { align-tracks: initial; align-content: center !important; justify-tracks: var(--old); justify-content: space-between; }",
    );
    assert_eq!(report.diagnostics().len(), 2);
    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let [
        CssNormalizedItem::Rule(_),
        CssNormalizedItem::Declaration(align),
        CssNormalizedItem::Declaration(justify),
    ] = normalized.syntax().items()
    else {
        panic!("one rule and two current declarations")
    };
    for (declaration, property, importance, order) in [
        (
            align,
            CssKnownProperty::AlignContent,
            CssImportance::Important,
            0,
        ),
        (
            justify,
            CssKnownProperty::JustifyContent,
            CssImportance::Normal,
            1,
        ),
    ] {
        assert_eq!(declaration.source().known().unwrap().property(), property);
        assert_eq!(declaration.source().importance(), importance);
        assert_eq!(declaration.order(), order);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("current typed longhand contribution")
        };
        let [value] = values.items() else {
            panic!("one longhand")
        };
        assert_eq!(value.property(), property);
        assert!(value.source().same_occurrence(declaration.source()));
    }
}

#[test]
fn display_sources_separate_base_grammar_from_grid_lanes_additions() {
    let base = feature_metadata("baseline.property.display").unwrap();
    assert_eq!(base.source().id().as_str(), "S-DISPLAY3");
    assert_eq!(base.production(), "#propdef-display");
    assert_eq!(base.status(), CssSupportStatus::Complete);
    let extension = feature_metadata("ext.value.grid-lanes-display").unwrap();
    assert_eq!(extension.source().id().as_str(), "X-GRID3-20260121");
    assert_eq!(extension.production(), "#grid-lanes-containers");
    assert_eq!(extension.kind(), CssFeatureKind::Value);
    assert_eq!(extension.status(), CssSupportStatus::Complete);
    for text in [
        "inline",
        "ruby block",
        "list-item flow-root inline",
        "grid-lanes",
        "inline-grid-lanes",
    ] {
        let report = parse_style_attribute(&format!("display:{text}"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    }
    for text in ["inline grid-lanes", "grid-lanes block", "list-item flex"] {
        let report = parse_style_attribute(&format!("display:{text}"));
        assert_eq!(report.diagnostics().len(), 1);
        assert!(report.syntax().is_empty());
    }
}

#[test]
fn selected_font_family_and_shorthand_metadata_match_their_grammar_boundaries() {
    let family = feature_metadata("baseline.property.font-family").unwrap();
    assert_eq!(family.kind(), CssFeatureKind::Property);
    assert_eq!(family.spelling(), "font-family");
    assert_eq!(family.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(family.production(), "#propdef-font-family");
    assert_eq!(family.status(), CssSupportStatus::Complete);

    let font = feature_metadata("baseline.property.font").unwrap();
    assert_eq!(font.kind(), CssFeatureKind::Property);
    assert_eq!(font.spelling(), "font");
    assert_eq!(font.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(font.production(), "#propdef-font");
    assert_eq!(font.status(), CssSupportStatus::Complete);

    for authored in [
        "font-family: \"Avenir Next\", sans-serif",
        "font-family: \"\", menu, generic(kai), ui-rounded",
        "font: menu",
        "font: large menu",
        "font: 16px generic(fangsong)",
        "font: 450.5 16px serif",
    ] {
        let report = parse_style_attribute(authored);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{authored}");
    }

    for authored in ["font: xxx-large serif", "font: math serif"] {
        let report = parse_style_attribute(authored);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{authored}");
    }
}

#[test]
fn font_properties_and_font_face_descriptors_parse_their_selected_grammar() {
    for (property, authored) in [
        ("font-feature-settings", "\"kern\" on, \"liga\" 0"),
        ("font-kerning", "normal"),
        ("font-size-adjust", "0.5"),
        ("font-synthesis", "weight style"),
        ("font-palette", "palette-mix(light, dark)"),
        ("font-variant", "small-caps oldstyle-nums"),
        ("font-variant-alternates", "styleset(Alpha, beta)"),
        ("font-variant-caps", "all-small-caps"),
        ("font-variant-east-asian", "jis04 ruby"),
        ("font-variant-emoji", "emoji"),
        (
            "font-variant-ligatures",
            "common-ligatures no-discretionary-ligatures",
        ),
        (
            "font-variant-numeric",
            "lining-nums tabular-nums slashed-zero",
        ),
        ("font-variant-position", "super"),
    ] {
        let report = parse_style_attribute(&format!("{property}: {authored}"));
        assert!(report.is_clean(), "{property}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{property}");
    }

    let font_face = parse_sheet(concat!(
        "@font-face { font-family: Demo Sans; ",
        "src: local(\"Demo Sans\"), url(demo.woff2) format(\"woff2\"); ",
        "font-style: oblique; font-weight: 700; font-width: condensed; ",
        "unicode-range: U+0-7F; font-feature-settings: \"kern\" on; }"
    ));
    assert!(font_face.is_clean(), "{:?}", font_face.diagnostics());
    assert_eq!(font_face.syntax().rules().len(), 1);

    let display =
        parse_sheet("@font-face { font-family: Demo; src: url(demo.woff2); font-display: swap; }");
    assert!(display.is_clean(), "{:?}", display.diagnostics());

    let feature_values = parse_sheet("@font-feature-values Demo { @styleset { nice: 1; } }");
    assert!(feature_values.is_clean());
    assert!(matches!(
        feature_values.syntax().rules(),
        [CssRule::FontFeatureValues(_)]
    ));

    let numeric_weight = parse_style_attribute("font-weight: 725");
    assert!(
        numeric_weight.is_clean(),
        "{:?}",
        numeric_weight.diagnostics()
    );
    let invalid_numeric_weight = parse_style_attribute("font-weight: 1001");
    assert!(invalid_numeric_weight.syntax().is_empty());
    assert_eq!(invalid_numeric_weight.diagnostics().len(), 1);
    assert_eq!(
        invalid_numeric_weight.diagnostics()[0].error().code(),
        CssErrorCode::InvalidPropertyValue
    );

    // Fonts 4 oblique angles lie in [-90deg, 90deg].
    let (id, authored, rejected) = (
        "ext.descriptor.font-style-oblique-range",
        "font-style: oblique -10deg 20deg",
        "font-style: oblique 91deg",
    );
    let report = parse_sheet(&format!(
        "@font-face {{ font-family: Demo; src: url(demo.woff2); {authored}; }}"
    ));
    assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());

    let rejected_report = parse_sheet(&format!(
        "@font-face {{ font-family: Demo; src: url(demo.woff2); {rejected}; }}"
    ));
    assert_eq!(rejected_report.diagnostics().len(), 1, "{id}");
    assert_eq!(
        rejected_report.diagnostics()[0].error().code(),
        CssErrorCode::InvalidDescriptorValue,
        "{id}"
    );

    let modern_hints = parse_sheet(concat!(
        "@font-face { font-family: Demo; ",
        "src: url(demo.woff2) format(woff2) tech(variations, color-colrv1); }"
    ));
    assert!(modern_hints.is_clean(), "{:?}", modern_hints.diagnostics());
    let unknown_hint = parse_sheet(
        "@font-face { font-family: Demo; src: url(demo.woff2) format(woff3) tech(unknown); }",
    );
    let [CssRule::FontFace(face)] = unknown_hint.syntax().rules() else {
        panic!("invalid source hints must preserve the authored font face");
    };
    let family = face
        .descriptors()
        .effective(CssFontFaceDescriptorKind::FontFamily)
        .expect("retained family");
    assert!(matches!(
        family.value(),
        CssAuthoredFontFaceDescriptorValue::Ordinary(
            CssFontFaceDescriptorValue::FontFamily(value)
        ) if value.as_str() == "Demo"
    ));
    assert!(
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::Src)
            .is_none()
    );
    assert_eq!(unknown_hint.diagnostics().len(), 1);
    assert_eq!(
        unknown_hint.diagnostics()[0].error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(
        unknown_hint.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
}

#[test]
fn selected_font_width_records_keep_historical_ids_and_canonical_names() {
    let property = feature_metadata("baseline.property.font-stretch").unwrap();
    assert_eq!(property.kind(), CssFeatureKind::Property);
    assert_eq!(property.spelling(), "font-width");
    assert_eq!(property.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(property.production(), "#propdef-font-width");
    assert_eq!(property.status(), CssSupportStatus::Complete);
    let support = property_support_metadata("font-stretch").expect("legacy property name");
    assert_eq!(support.property(), CssKnownProperty::FontWidth);
    assert_eq!(support.canonical_name(), "font-width");
    assert_eq!(support.aliases(), &["font-stretch"]);

    let descriptor = feature_metadata("baseline.descriptor.font-stretch").unwrap();
    assert_eq!(descriptor.kind(), CssFeatureKind::Descriptor);
    assert_eq!(descriptor.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(descriptor.production(), "#font-prop-desc");
    assert_eq!(descriptor.status(), CssSupportStatus::Complete);

    let range = feature_metadata("ext.descriptor.font-stretch-range").unwrap();
    assert_eq!(range.kind(), CssFeatureKind::Descriptor);
    assert_eq!(range.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(range.production(), "#font-prop-desc");
    assert_eq!(range.status(), CssSupportStatus::Complete);

    for name in ["font-width", "font-stretch"] {
        let report = parse_sheet(&format!(
            "@font-face {{ font-family: Demo; src: url(demo.woff2); {name}: 125% 75%; }}"
        ));
        assert!(report.is_clean(), "{name}: {:?}", report.diagnostics());
        let [CssRule::FontFace(face)] = report.syntax().rules() else {
            panic!("one font face with {name}")
        };
        assert!(
            face.descriptors()
                .effective(CssFontFaceDescriptorKind::FontWidth)
                .is_some(),
            "{name}"
        );
        assert!(matches!(
            face.descriptors()
                .occurrences()
                .last()
                .map(|item| item.value()),
            Some(CssAuthoredFontFaceDescriptorValue::Ordinary(
                CssFontFaceDescriptorValue::FontWidth(_)
            ))
        ));
    }
}

#[test]
fn selected_font_weight_records_keep_historical_ids_and_complete_authored_grammar() {
    let property = parse_style_attribute("font-weight: 725.25");
    assert!(property.is_clean(), "{:?}", property.diagnostics());
    let descriptor = parse_sheet("@font-face { font-weight: bold 300; }");
    assert!(descriptor.is_clean(), "{:?}", descriptor.diagnostics());
}

#[test]
fn selected_font_style_records_keep_historical_ids_and_complete_authored_grammar() {
    assert!(parse_style_attribute("font-style: oblique 10deg").is_clean());
    assert!(parse_sheet("@font-face { font-style: oblique 20deg -10deg; }").is_clean());
    assert!(parse_style_attribute("font: oblique calc(10deg) 16px serif").is_clean());
}

#[test]
fn selected_font_size_record_keeps_historical_id_with_current_fonts_source() {
    let record = feature_metadata("baseline.property.font-size").expect("font-size metadata");
    assert_eq!(record.kind(), CssFeatureKind::Property);
    assert_eq!(record.spelling(), "font-size");
    assert_eq!(record.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(record.production(), "#propdef-font-size");
    assert_eq!(record.status(), CssSupportStatus::Complete);
    for input in [
        "font-size: xxx-large",
        "font-size: math",
        "font-size: 1e-999px",
        "font-size: calc(-1px)",
    ] {
        let report = parse_style_attribute(input);
        assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    }
    let rejected = parse_style_attribute("font-size: -1e-999px");
    assert!(rejected.syntax().is_empty());
    assert_eq!(rejected.diagnostics().len(), 1);
}

#[test]
fn selected_line_height_record_keeps_css2_identity_and_complete_authored_grammar() {
    let record = feature_metadata("baseline.property.line-height").expect("line-height metadata");
    assert_eq!(record.kind(), CssFeatureKind::Property);
    assert_eq!(record.spelling(), "line-height");
    assert_eq!(record.source().id().as_str(), "O-CSS2");
    assert_eq!(record.production(), "visudet.html#propdef-line-height");
    assert_eq!(record.status(), CssSupportStatus::Complete);
    for input in [
        "line-height: normal",
        "line-height: 1e999",
        "line-height: 120%",
        "line-height: calc(-1)",
    ] {
        let report = parse_style_attribute(input);
        assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    }
    for input in [
        "line-height: -1e-999",
        "line-height: -1e-999px",
        "line-height: -1e-999%",
    ] {
        let report = parse_style_attribute(input);
        assert!(report.syntax().is_empty(), "{input}");
        assert_eq!(report.diagnostics().len(), 1, "{input}");
    }
}

#[test]
fn environment_metadata_names_selected_descriptor_consumers_without_claiming_execution() {
    let record = feature_metadata("required.value.environment-substitution").unwrap();
    assert_eq!(record.source().id().as_str(), "D-ENV1");
    assert_eq!(record.status(), CssSupportStatus::Partial);
}

#[test]
fn attribute_substitution_metadata_matches_pending_known_property_behavior() {
    let record = feature_metadata("required.value.attribute-substitution").unwrap();
    assert_eq!(record.kind(), CssFeatureKind::Value);
    assert_eq!(record.source().id().as_str(), "D-VALUES5-ATTR");
    assert_eq!(record.source().tier(), CssSpecificationTier::LaterStandard);
    assert_eq!(record.status(), CssSupportStatus::Partial);
    assert_eq!(
        record.production(),
        "#component-function-commas,#typedef-syntax,#attr-notation,#attr-substitution,#attr-security"
    );
    assert!(record.supported_subset().is_some());
    assert!(record.unsupported_remainder().is_some());

    let source = specification_source("D-VALUES5-ATTR").unwrap();
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2024/WD-css-values-5-20241111/")
    );
    for authored in [
        "width: attr(data-width <length>, 1px)",
        "content: attr(data-label)",
    ] {
        let report = parse_style_attribute(authored);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{authored}");
        let pending = report.syntax()[0]
            .known()
            .unwrap()
            .substitution_dependent()
            .expect("valid attr() is pending");
        assert!(pending.as_css().contains("attr("), "{authored}");
    }
}

fn assert_clean_color(authored: &str) {
    let source = format!("color: {authored}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color),
        "{source}",
    );
}

fn assert_rejected_color(authored: &str) {
    let source = format!("color: {authored}");
    let report = parse_style_attribute(&source);
    assert!(report.syntax().is_empty(), "{source}");
    assert_eq!(report.diagnostics().len(), 1, "{source}");
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::InvalidColorSyntax,
        "{source}",
    );
}

#[test]
fn color4_value_and_property_metadata_match_public_authored_behavior() {
    assert_clean_color("red");
    assert_clean_color("rgb(0 0 0 / 150%)");
    assert_clean_color("hsl(calc(1turn - 90deg) 50% 50%)");
    assert_clean_color("rgb(calc(1 + 2) 20% none / 150%)");
    assert_clean_color("#00ff88cc");
    assert_clean_color("rebeccapurple");
    assert_clean_color("CanvasText");
    assert_clean_color("ActiveBorder");
    assert_clean_color("transparent");
    assert_clean_color("currentColor");
    assert_clean_color("hsl(30deg 120 -20% / none)");
    assert_clean_color("hwb(none 20 120% / -10%)");
    assert_clean_color("lab(50% 20 -30 / 120%)");
    assert_clean_color("lch(50% 20 30deg / none)");
    assert_clean_color("oklab(50% 0.1 -0.1)");
    assert_clean_color("oklch(50% 0.2 30deg)");
    assert_clean_color("color(display-p3 1 0 0 / 120%)");

    let color = property_support_metadata("color").expect("color metadata");
    assert_eq!(color.feature().status(), CssSupportStatus::Complete);
    assert_eq!(color.feature().source().id().as_str(), "O-COLOR4");
    assert_eq!(color.feature().production(), "#propdef-color");
    assert_eq!(color.feature().supported_subset(), None);
    assert_eq!(color.feature().unsupported_remainder(), None);

    let opacity_report = parse_style_attribute("opacity: 150%");
    assert!(
        opacity_report.is_clean(),
        "{:?}",
        opacity_report.diagnostics()
    );
    let opacity = property_support_metadata("opacity").expect("opacity metadata");
    assert_eq!(opacity.feature().status(), CssSupportStatus::Complete);
    assert_eq!(opacity.feature().source().id().as_str(), "O-COLOR4");
    assert_eq!(opacity.feature().production(), "#propdef-opacity");
    assert_eq!(opacity.feature().supported_subset(), None);
    assert_eq!(opacity.feature().unsupported_remainder(), None);
}

#[test]
fn relative_color_selected_families_and_deferred_remainder_are_distinct() {
    assert_clean_color("rgb(from red r g b / alpha)");
    assert_rejected_color("alpha(from red r g b)");
}

#[test]
fn relative_rgb_channels_reject_foreign_identifiers_and_dimensions() {
    assert_clean_color("rgb(from red r g b / alpha)");
    assert_rejected_color("rgb(from red h g b)");
}

#[test]
fn relative_hsl_channels_keep_hue_and_percentage_domains_distinct() {
    assert_clean_color("hsl(from red h s l / alpha)");
    assert_rejected_color("hsl(from red 10% s l)");
}

#[test]
fn relative_hwb_channels_use_only_hwb_environment() {
    assert_clean_color("hwb(from red h w b / alpha)");
    assert_rejected_color("hwb(from red h s b)");
}

#[test]
fn relative_lab_channels_use_only_lab_environment() {
    assert_clean_color("lab(from red l a b / alpha)");
    assert_rejected_color("lab(from red l c b)");
}

#[test]
fn relative_oklab_channels_use_only_oklab_environment() {
    assert_clean_color("oklab(from red l a b / alpha)");
    assert_rejected_color("oklab(from red l c b)");
}

#[test]
fn relative_lch_channels_keep_hue_domain_distinct() {
    assert_clean_color("lch(from red l c h / alpha)");
    assert_rejected_color("lch(from red l c calc(h + 10%))");
}

#[test]
fn relative_oklch_channels_keep_hue_domain_distinct() {
    assert_clean_color("oklch(from red l c h / alpha)");
    assert_rejected_color("oklch(from red l c calc(h + 10%))");
}

#[test]
fn relative_predefined_color_channels_follow_space_environment() {
    assert_clean_color("color(from red display-p3 r g b / alpha)");
    assert_rejected_color("color(from red srgb x g b)");
}

#[test]
fn color_mix_preserved_subset_rejects_cross_space_hue_methods() {
    assert_clean_color("color-mix(in oklch longer hue, red 25%, blue 75%)");
    assert_rejected_color("color-mix(in srgb longer hue, red, blue)");
}

#[test]
fn grid_metadata_matches_preserved_boundaries() {
    let grid = parse_style_attribute(concat!(
        "grid-template-columns: repeat(auto-fit, 10px); ",
        "grid-auto-rows: minmax(10px, auto)",
    ));
    assert!(grid.is_clean(), "{:?}", grid.diagnostics());
    assert_eq!(grid.syntax().len(), 2);
    assert_eq!(
        grid.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::GridTemplateColumns),
    );
    assert_eq!(
        grid.syntax()[1].known().map(|known| known.property()),
        Some(CssKnownProperty::GridAutoRows),
    );

    let invalid_grid = parse_style_attribute(
        "grid-template-columns: repeat(auto-fit, minmax(1fr, 10px)); color: red",
    );
    assert_eq!(invalid_grid.syntax().len(), 1);
    assert_eq!(
        invalid_grid.syntax()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::Color),
    );
    assert_eq!(
        invalid_grid.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );

    let keyframes = parse_sheet(concat!(
        "@keyframes fade { ",
        "from, 0%, from { } ",
        "from { opacity: 0; } ",
        "0% { opacity: 1; } ",
        "} ",
        "@keyframes empty {}",
    ));
    assert!(keyframes.is_clean(), "{:?}", keyframes.diagnostics());
    let [CssRule::Keyframes(fade), CssRule::Keyframes(empty)] = keyframes.syntax().rules() else {
        panic!("expected authored duplicate and empty keyframe rules");
    };
    assert_eq!(fade.blocks().len(), 3);
    assert!(fade.blocks()[0].declarations().is_empty());
    assert!(empty.blocks().is_empty());
}
#[test]
fn selected_alignment_grammars_cite_the_superseding_align3_source() {
    for name in [
        "align-content",
        "justify-content",
        "align-items",
        "justify-items",
        "align-self",
        "justify-self",
        "place-content",
        "place-items",
        "place-self",
    ] {
        let property = CssKnownProperty::from_name(name).unwrap();
        let feature = feature_metadata(property.stable_id()).unwrap();
        assert_eq!(feature.source().id().as_str(), "S-ALIGN3", "{name}");
        assert_eq!(feature.status(), CssSupportStatus::Complete, "{name}");
    }
}

#[test]
fn numeric_value_types_construct_checked_literals_and_drop_cross_type_declarations() {
    let dimension = CssSpecifiedLength::try_from_component(
        CssComponentValue::try_dimension("-1.5", "cqw").unwrap(),
    )
    .unwrap();
    assert!(exact_dimension(
        dimension.literal_component(),
        "-1.5",
        "cqw"
    ));
    assert!(CssComponentValue::try_dimension("infinity", "px").is_err());

    let angle =
        CssAngleCalculation::try_literal("-0.5", CssAngleUnit::Turns).expect("finite angle");
    let percentage = CssPercentageCalculation::try_literal(25.0).expect("finite percentage");
    let time =
        CssTimeCalculation::try_literal("-250", CssTimeUnit::Milliseconds).expect("finite time");
    let frequency = CssFrequencyCalculation::try_literal("1.5", CssFrequencyUnit::Kilohertz)
        .expect("finite frequency");
    assert_eq!(angle.result_type(), CssCalculationType::Angle);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    assert_eq!(time.result_type(), CssCalculationType::Time);
    assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
    assert!(CssAngleCalculation::try_literal("NaN", CssAngleUnit::Degrees).is_err());
    assert!(
        CssTimeCalculation::try_literal(&f32::INFINITY.to_string(), CssTimeUnit::Seconds).is_err()
    );
    assert!(CssFrequencyCalculation::try_literal("-inf", CssFrequencyUnit::Hertz).is_err());

    let parsed = parse_style_attribute(concat!(
        "width: calc((1cqw + 2%) * 3); ",
        "transform: rotate(calc((1turn + 180deg) / 2)); ",
        "transition-delay: calc((-1s + 250ms) * 2); ",
        "filter: hue-rotate(calc((1turn - 90deg) / 3))",
    ));
    assert!(parsed.is_clean(), "{:?}", parsed.diagnostics());
    assert_eq!(
        parsed
            .syntax()
            .iter()
            .map(|declaration| declaration.known().expect("known value").property())
            .collect::<Vec<_>>(),
        [
            CssKnownProperty::Width,
            CssKnownProperty::Transform,
            CssKnownProperty::TransitionDelay,
            CssKnownProperty::Filter,
        ],
    );

    let recovered = parse_style_attribute(concat!(
        "width: 1deg; ",
        "rotate: 1hz; ",
        "transition-delay: 1px; ",
        "color: red",
    ));
    assert_eq!(recovered.syntax().len(), 1);
    assert_eq!(
        recovered.syntax()[0]
            .known()
            .expect("retained sibling")
            .property(),
        CssKnownProperty::Color,
    );
    assert_eq!(recovered.diagnostics().len(), 3);
    assert!(
        recovered
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
}

#[test]
fn partial_color_and_selector_extensions_accept_their_supported_subset() {
    for (id, kind, input) in [
        (
            "ext.value.relative-color",
            CssFeatureKind::Value,
            Input::Style("color: rgb(from red r g b / alpha)"),
        ),
        (
            "ext.value.color-mix",
            CssFeatureKind::Value,
            Input::Style("color: color-mix(in oklch longer hue, red 25%, blue 75%)"),
        ),
        (
            "ext.supports.selector",
            CssFeatureKind::Selector,
            Input::Sheet("@supports selector(.card:is(.primary, .secondary)) {}"),
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.status(), CssSupportStatus::Partial, "{id} status");
        assert!(metadata.supported_subset().is_some(), "{id} subset");
        assert!(metadata.unsupported_remainder().is_some(), "{id} remainder");
        assert!(diagnostics(input).is_empty(), "{id} public positive vector");
    }

    let report = parse_sheet(concat!(
        "@font-feature-values Demo { @styleset { nice: 1; } } ",
        ".tail { color: red; }",
    ));
    assert!(report.is_clean());
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::FontFeatureValues(_), CssRule::Style(_)]
    ));
    let metadata =
        feature_metadata("later.rule.font-feature-values").expect("font-feature-values metadata");
    assert_eq!(metadata.kind(), CssFeatureKind::Rule);
    assert_eq!(metadata.spelling(), "@font-feature-values");
    assert_eq!(metadata.source().id().as_str(), "I-FONTS4-20260907");
    assert_eq!(metadata.production(), "#font-feature-values-syntax");
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
}

#[test]
fn official_position_metadata_matches_generic_position_behavior() {
    let report = parse_style_attribute("object-position: right 5% bottom 2px");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ObjectPosition(value) = report.syntax()[0]
        .known()
        .expect("known object-position")
        .property_value()
        .expect("ordinary object-position")
    else {
        panic!("expected object-position value");
    };
    assert!(matches!(
        value.position().horizontal(),
        CssHorizontalPosition::RightOffset(offset)
            if exact_percentage(offset.literal_component(), "5")
    ));
    assert!(matches!(
        value.position().vertical(),
        CssVerticalPosition::BottomOffset(offset)
            if exact_dimension(offset.literal_component(), "2", "px")
    ));
}

#[test]
fn official_background_position_metadata_matches_three_component_layer_behavior() {
    let report = parse_style_attribute("background-position: left 10px top");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundPosition(value) = report.syntax()[0]
        .known()
        .expect("known background-position")
        .property_value()
        .expect("ordinary background-position")
    else {
        panic!("expected background-position value");
    };
    let [layer] = value.positions().positions() else {
        panic!("expected one background-position layer");
    };
    assert!(matches!(
        layer.horizontal(),
        CssHorizontalPosition::LeftOffset(offset)
            if exact_dimension(offset.literal_component(), "10", "px")
    ));
    assert!(matches!(layer.vertical(), CssVerticalPosition::Top));
}

#[test]
fn background_position_property_metadata_matches_layer_list_behavior() {
    let report = parse_style_attribute("background-position: left top, right 4px bottom 8%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0]
        .known()
        .expect("known background-position");
    assert_eq!(declaration.property(), CssKnownProperty::BackgroundPosition);
    let CssKnownPropertyValueRef::BackgroundPosition(value) = declaration
        .property_value()
        .expect("ordinary background-position")
    else {
        panic!("expected background-position value");
    };
    assert_eq!(value.positions().positions().len(), 2);
}

#[test]
fn object_position_property_metadata_matches_semantic_accessor_behavior() {
    let report = parse_style_attribute("object-position: center 25%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0].known().expect("known object-position");
    assert_eq!(declaration.property(), CssKnownProperty::ObjectPosition);
    let CssKnownPropertyValueRef::ObjectPosition(value) = declaration
        .property_value()
        .expect("ordinary object-position")
    else {
        panic!("expected object-position value");
    };
    assert!(matches!(
        value.position().horizontal(),
        CssHorizontalPosition::Center
    ));
    assert!(matches!(
        value.position().vertical(),
        CssVerticalPosition::Offset(offset)
            if exact_percentage(offset.literal_component(), "25")
    ));
}

#[test]
fn transform_origin_property_metadata_matches_directed_z_behavior() {
    let report = parse_style_attribute("transform-origin: left top 50px");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0].known().expect("known transform-origin");
    assert_eq!(declaration.property(), CssKnownProperty::TransformOrigin);
    let CssKnownPropertyValueRef::TransformOrigin(value) = declaration
        .property_value()
        .expect("ordinary transform-origin")
    else {
        panic!("expected transform-origin value");
    };
    assert!(matches!(
        value.origin().horizontal(),
        CssHorizontalPosition::Left
    ));
    assert!(matches!(
        value.origin().vertical(),
        CssVerticalPosition::Top
    ));
    assert!(
        value
            .origin()
            .z()
            .is_some_and(|z| exact_dimension(z.literal_component(), "50", "px"))
    );
}

#[test]
fn mask_position_property_metadata_matches_generic_layer_behavior() {
    let report = parse_style_attribute("mask-position: left 10px bottom 20%, center 25%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0].known().expect("known mask-position");
    assert_eq!(declaration.property(), CssKnownProperty::MaskPosition);
    let CssKnownPropertyValueRef::MaskPosition(value) = declaration
        .property_value()
        .expect("ordinary mask-position")
    else {
        panic!("expected mask-position value");
    };
    let [first, second] = value.positions().positions() else {
        panic!("expected two mask-position layers");
    };
    assert!(matches!(
        first.horizontal(),
        CssHorizontalPosition::LeftOffset(offset)
            if exact_dimension(offset.literal_component(), "10", "px")
    ));
    assert!(matches!(second.horizontal(), CssHorizontalPosition::Center));
}

// Each Values and Units value type claimed Complete is exercised through a
// property whose grammar consumes it: the authored value is retained, and a
// value outside the type is dropped with InvalidPropertyValue (CSS Syntax 3
// section 8.1 and Values 3 section 2: an invalid declaration is ignored).

#[test]
fn official_integer_metadata_matches_checked_integer_behavior() {
    let value = CssIntegerCalculation::literal(-3);
    assert_eq!(value.result_type(), CssCalculationType::Number);
    assert!(!value.requires_rounding());
    // Values 3 section 5.1; z-index is `auto | <integer>`.
    assert_single_known_declaration("z-index: -3", "baseline.property.z-index");
    assert_dropped_declaration("z-index: 1.5");
}

#[test]
fn official_number_metadata_matches_checked_number_behavior() {
    let value = CssNumberCalculation::try_literal(-3.5).expect("finite number");
    assert_eq!(value.result_type(), CssCalculationType::Number);
    // Values 3 section 5.2; flex-grow is `<number [0,∞]>`.
    assert_single_known_declaration("flex-grow: 1.5", "baseline.property.flex-grow");
    assert_dropped_declaration("flex-grow: 1px");
}

#[test]
fn official_dimension_metadata_matches_checked_dimension_behavior() {
    let value = CssSpecifiedLength::try_from_component(
        CssComponentValue::try_dimension("1.5", "rem").unwrap(),
    )
    .unwrap();
    assert!(exact_dimension(value.literal_component(), "1.5", "rem"));
    // Values 3 section 5.4: a dimension carries a unit; an unknown unit is
    // not a length.
    assert_single_known_declaration("width: 1.5rem", "baseline.property.width");
    assert_dropped_declaration("width: 1.5qux");
}

#[test]
fn official_percentage_metadata_matches_checked_percentage_behavior() {
    let value = CssPercentageCalculation::try_literal(-12.5).expect("finite percentage");
    assert_eq!(value.result_type(), CssCalculationType::Percentage);
    // Values 3 section 5.5: `%` must immediately follow the number.
    assert_single_known_declaration("text-indent: -12.5%", "baseline.property.text-indent");
    assert_dropped_declaration("text-indent: 12.5 %");
}

#[test]
fn official_length_metadata_matches_checked_length_behavior() {
    let value = CssLengthCalculation::try_dimension(2.0, CssLengthUnit::Cqw)
        .expect("finite length dimension");
    assert_eq!(value.result_type(), CssCalculationType::Length);
    assert_single_known_declaration("width: 2cqw", "baseline.property.width");
    assert_dropped_declaration("width: 2deg");
}

#[test]
fn official_length_percentage_metadata_matches_mixed_length_parser_behavior() {
    let report = parse_style_attribute("width: calc((1px + 2%) * 3)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Width
    );
    // Values 4 section 10.10: length and angle do not add.
    assert_dropped_declaration("width: calc(1px + 2deg)");
}

#[test]
fn official_angle_metadata_matches_checked_angle_behavior() {
    let value =
        CssAngleCalculation::try_literal("-0.5", CssAngleUnit::Turns).expect("finite angle");
    assert_eq!(value.result_type(), CssCalculationType::Angle);
    assert_single_known_declaration("rotate: -0.5turn", "baseline.property.rotate");
    assert_dropped_declaration("rotate: 1hz");
}

#[test]
fn official_angle_percentage_metadata_matches_checked_mixed_models() {
    let angle =
        CssAngleCalculation::try_literal("45", CssAngleUnit::Degrees).expect("finite angle");
    let percentage = CssPercentageCalculation::try_literal(25.0).expect("finite percentage");
    assert_eq!(angle.result_type(), CssCalculationType::Angle);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    // Images 4 conic color stops are `<angle-percentage>`; a length is not.
    assert_single_known_declaration(
        "background-image: conic-gradient(red 10%, blue 90deg)",
        "baseline.property.background-image",
    );
    assert_dropped_declaration("background-image: conic-gradient(red 10px, blue)");
}

#[test]
fn official_time_metadata_matches_checked_time_behavior() {
    let value =
        CssTimeLiteral::try_new("-250", CssTimeUnit::Milliseconds).expect("finite signed time");
    assert_eq!(value.numeric().representation(), "-250");
    assert_eq!(value.unit(), CssTimeUnit::Milliseconds);
    assert_single_known_declaration(
        "transition-delay: -250ms",
        "baseline.property.transition-delay",
    );
    assert_dropped_declaration("transition-delay: 1px");
}

#[test]
fn official_time_percentage_metadata_matches_checked_mixed_models() {
    let time = CssTimeCalculation::try_literal("-1", CssTimeUnit::Seconds).expect("finite time");
    let percentage = CssPercentageCalculation::try_literal(50.0).expect("finite percentage");
    assert_eq!(time.result_type(), CssCalculationType::Time);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    // No selected property consumes <time-percentage>; a pure <time> context
    // keeps rejecting percentages (Values 4 section 10.10).
    assert_dropped_declaration("transition-delay: calc(1s + 10%)");
}

#[test]
fn official_frequency_metadata_matches_checked_frequency_behavior() {
    let value = CssFrequencyCalculation::try_literal("440", CssFrequencyUnit::Hertz)
        .expect("finite frequency");
    assert_eq!(value.result_type(), CssCalculationType::Frequency);
    // Speech 1 voice-pitch admits `<frequency [0Hz,∞]> && absolute`.
    assert_single_known_declaration(
        "voice-pitch: 200Hz absolute",
        "official.property.voice-pitch",
    );
    assert_dropped_declaration("voice-pitch: 200px absolute");
}

#[test]
fn official_frequency_percentage_metadata_matches_checked_mixed_models() {
    let frequency = CssFrequencyCalculation::try_literal("1.5", CssFrequencyUnit::Kilohertz)
        .expect("finite frequency");
    let percentage = CssPercentageCalculation::try_literal(75.0).expect("finite percentage");
    assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    assert_single_known_declaration(
        "voice-pitch: calc(100Hz + 10%)",
        "official.property.voice-pitch",
    );
    assert_dropped_declaration("voice-pitch: calc(100Hz + 10deg)");
}

#[test]
fn official_resolution_metadata_matches_checked_resolution_behavior() {
    let value =
        CssResolutionLiteral::try_new("2", CssResolutionUnit::Dppx).expect("exact resolution");
    assert_eq!(value.numeric().representation(), "2");
    assert_eq!(value.unit(), CssResolutionUnit::Dppx);
    // The resolution media feature is the selected <resolution> consumer. A
    // mistyped feature value is an unknown feature, not a syntax error
    // (Media Queries 5 section 3.2), so no authored rejection exists here.
    for source in [
        "@media (resolution: 2dppx) { .x { color: red; } }",
        "@media (min-resolution: 192dpi) { .x { color: red; } }",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    }
}

#[test]
fn official_calc_metadata_matches_typed_calculation_parser_behavior() {
    let report = parse_style_attribute("width: calc((1px + 2%) * 3)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Width
    );
    // Values 4 section 10.1: a calc() sum needs an operand after `+`.
    assert_dropped_declaration("width: calc(1px +)");
}

#[test]
fn transition_duration_metadata_matches_typed_duration_behavior() {
    assert_single_known_declaration(
        "transition-duration: calc((1s + 250ms) * 2)",
        "baseline.property.transition-duration",
    );
}

#[test]
fn transition_delay_metadata_matches_signed_delay_behavior() {
    assert_single_known_declaration(
        "transition-delay: -250ms",
        "baseline.property.transition-delay",
    );
}

#[test]
fn animation_duration_metadata_matches_typed_duration_behavior() {
    assert_single_known_declaration(
        "animation-duration: calc(1s + 250ms)",
        "baseline.property.animation-duration",
    );
}

#[test]
fn animation_delay_metadata_matches_signed_delay_behavior() {
    assert_single_known_declaration("animation-delay: -1s", "baseline.property.animation-delay");
}

#[test]
fn animation_iteration_count_metadata_matches_typed_iteration_behavior() {
    assert_single_known_declaration(
        "animation-iteration-count: calc((1 + 2) * 3)",
        "baseline.property.animation-iteration-count",
    );
}

#[test]
fn transition_metadata_matches_the_selected_complete_shorthand_grammar() {
    assert_single_known_declaration(
        "transition: opacity calc(1s + 250ms) linear -200ms",
        "baseline.property.transition",
    );
}

#[test]
fn animation_metadata_matches_the_selected_complete_shorthand_grammar() {
    assert_single_known_declaration(
        "animation: fade calc(1s + 250ms) linear calc((1 + 2) * 3) -200ms both running",
        "baseline.property.animation",
    );
}

#[test]
fn complete_media_range_metadata_preserves_original_comparison_evidence() {
    for (name, css) in [
        ("width", "(width >= 1px)"),
        ("height", "(height < 100vh)"),
        ("resolution", "(resolution >= 2dppx)"),
        ("color", "(color > 0)"),
        ("monochrome", "(monochrome = 1)"),
    ] {
        let id = format!("ext.media.range.{name}");
        let metadata = feature_metadata(&id).expect("range metadata");
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.source().id().as_str(), "X-MEDIA5");
        assert!(parse_sheet(&format!("@media {css} {{}}")).is_clean());
    }
}

#[test]
fn media_supports_and_import_preludes_parse_cleanly() {
    for source in [
        "@media speech and (device-width: 800px) and (resolution: 2dpcm) { .x { color: red; } }",
        "@supports (display: grid) and future-layout(mode) { .x { color: red; } }",
        "@supports selector(.card > .item:hover) { .x { color: red; } }",
        "@layer reset; @import url(theme.css) layer(theme) supports(display: grid) print;",
    ] {
        let report = parse_sheet(source);
        assert!(
            report.is_clean(),
            "paired grammar behavior must be clean for {source:?}: {:?}",
            report.diagnostics()
        );
    }

    let general_enclosed_source =
        specification_source("X-VALUES4").expect("immutable general-enclosed source");
    assert_eq!(
        general_enclosed_source.tier(),
        CssSpecificationTier::SurgeistExtension
    );
    assert_eq!(general_enclosed_source.url(), None);
    assert_eq!(
        general_enclosed_source.repository_provenance(),
        Some("720ea2863696971ea6a6744e0f23acbb3e6936bd:css-values-4/Overview.bs")
    );
    let selector =
        feature_metadata("ext.supports.selector").expect("Conditional 4 selector-test metadata");
    assert_eq!(selector.kind(), CssFeatureKind::Selector);
    assert_eq!(selector.source().id().as_str(), "R-CONDITIONAL4");
    assert_eq!(selector.production(), "#typedef-supports-selector-fn");
    assert_eq!(selector.status(), CssSupportStatus::Partial);
}

#[test]
fn selectors3_and_namespace_selectors_parse_into_typed_supports_conditions() {
    let selectors = parse_sheet(concat!(
        ":root,:link,:visited,:target,:lang(en),:hover,:active,:focus,",
        ":enabled,:disabled,:checked,:indeterminate,:first-child,:last-child,",
        ":only-child,:empty,:nth-child(2n+1),:nth-last-child(2),",
        ":first-of-type,:last-of-type,:only-of-type,:nth-of-type(odd),",
        ":nth-last-of-type(even),:not(.excluded) { color: red; }",
        "article#first#second.card[data-ready][lang|=\"en\"][title^=\"lead\"] ",
        "> section + a ~ p { color: red; }",
        ".line::first-line { color: red; }",
        ".letter:first-letter { color: red; }",
        ".generated:before { color: red; }",
    ));
    assert!(
        selectors.is_clean(),
        "paired Selectors 3 grammar behavior: {:?}",
        selectors.diagnostics()
    );

    let qualified = parse_sheet(concat!(
        "@namespace svg \"urn:svg\";",
        "svg|a,svg|*,*|a,|a,[svg|href],[*|title],[|lang],[plain] { color: red; }",
        "@supports selector(svg|a) {}",
    ));
    assert!(
        qualified.is_clean(),
        "paired namespace grammar behavior: {:?}",
        qualified.diagnostics()
    );
    assert!(matches!(
        qualified.syntax().rules().first(),
        Some(CssRule::Namespace(_))
    ));
    let Some(CssRule::Supports(typed)) = qualified.syntax().rules().last() else {
        panic!("expected a final typed supports rule")
    };
    assert!(matches!(
        typed.condition().kind(),
        CssSupportsConditionKind::Selector(_)
    ));

    let general_enclosed = parse_sheet("@supports selector(.x ||) {}");
    assert!(
        general_enclosed.is_clean(),
        "balanced selector remainder: {:?}",
        general_enclosed.diagnostics()
    );
    let [CssRule::Supports(general_enclosed)] = general_enclosed.syntax().rules() else {
        panic!("expected general-enclosed supports rule")
    };
    assert!(matches!(
        general_enclosed.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == Some("selector(.x ||)")
    ));

    let supports_selector =
        feature_metadata("ext.supports.selector").expect("selector() extension metadata");
    assert_eq!(supports_selector.id().as_str(), "ext.supports.selector");
    assert_eq!(supports_selector.kind(), CssFeatureKind::Selector);
    assert_eq!(supports_selector.source().id().as_str(), "R-CONDITIONAL4");
    assert_eq!(
        supports_selector.production(),
        "#typedef-supports-selector-fn"
    );
    assert_eq!(supports_selector.status(), CssSupportStatus::Partial);
}

#[test]
fn residual_css2_and_level_three_properties_parse_with_their_catalog_owner() {
    let properties = [
        (
            "official.property.border-collapse",
            "border-collapse",
            "O-CSS2",
            "tables.html#propdef-border-collapse",
            "collapse",
        ),
        (
            "official.property.border-spacing",
            "border-spacing",
            "O-CSS2",
            "tables.html#propdef-border-spacing",
            "2px 3px",
        ),
        (
            "official.property.caption-side",
            "caption-side",
            "O-CSS2",
            "tables.html#propdef-caption-side",
            "bottom",
        ),
        (
            "official.property.clip",
            "clip",
            "S-MASKING1",
            "#propdef-clip",
            "rect(auto, 10px, 20px, -1px)",
        ),
        (
            "official.property.empty-cells",
            "empty-cells",
            "O-CSS2",
            "tables.html#propdef-empty-cells",
            "hide",
        ),
        (
            "official.property.orphans",
            "orphans",
            "S-BREAK3",
            "#propdef-orphans",
            "3",
        ),
        (
            "official.property.break-after",
            "break-after",
            "S-BREAK3",
            "#propdef-break-after",
            "right",
        ),
        (
            "official.property.break-before",
            "break-before",
            "S-BREAK3",
            "#propdef-break-before",
            "page",
        ),
        (
            "official.property.break-inside",
            "break-inside",
            "S-BREAK3",
            "#propdef-break-inside",
            "avoid",
        ),
        (
            "official.property.quotes",
            "quotes",
            "X-CONTENT3",
            "#propdef-quotes",
            "\"open\" \"close\"",
        ),
        (
            "official.property.table-layout",
            "table-layout",
            "O-CSS2",
            "tables.html#propdef-table-layout",
            "fixed",
        ),
        (
            "official.property.widows",
            "widows",
            "S-BREAK3",
            "#propdef-widows",
            "4",
        ),
        (
            "official.property.word-spacing",
            "word-spacing",
            "X-TEXT4",
            "#propdef-word-spacing",
            "-0.25em",
        ),
        (
            "official.property.text-combine-upright",
            "text-combine-upright",
            "S-WRITING4",
            "#propdef-text-combine-upright",
            "all",
        ),
        (
            "official.property.text-orientation",
            "text-orientation",
            "O-WRITING3",
            "#propdef-text-orientation",
            "sideways",
        ),
        (
            "official.property.unicode-bidi",
            "unicode-bidi",
            "O-WRITING3",
            "#propdef-unicode-bidi",
            "isolate-override",
        ),
        (
            "official.property.caret-color",
            "caret-color",
            "X-UI4",
            "#propdef-caret-color",
            "rebeccapurple",
        ),
        (
            "official.property.outline-offset",
            "outline-offset",
            "X-UI4",
            "#propdef-outline-offset",
            "-2px",
        ),
        (
            "official.property.resize",
            "resize",
            "X-UI4",
            "#propdef-resize",
            "horizontal",
        ),
        (
            "official.property.contain",
            "contain",
            "I-CONTAIN2",
            "#propdef-contain",
            "paint size",
        ),
        (
            "official.property.transform-box",
            "transform-box",
            "O-TRANSFORMS1",
            "#propdef-transform-box",
            "view-box",
        ),
        (
            "official.property.background-blend-mode",
            "background-blend-mode",
            "O-COMPOSITING1",
            "#propdef-background-blend-mode",
            "multiply, luminosity",
        ),
        (
            "official.property.isolation",
            "isolation",
            "O-COMPOSITING1",
            "#propdef-isolation",
            "isolate",
        ),
        (
            "official.property.mix-blend-mode",
            "mix-blend-mode",
            "O-COMPOSITING1",
            "#propdef-mix-blend-mode",
            "soft-light",
        ),
    ];

    for (id, name, source, production, authored_value) in properties {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), CssFeatureKind::Property, "{id} kind");
        assert_eq!(metadata.spelling(), name, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");

        let property =
            property_support_metadata(name).unwrap_or_else(|| panic!("missing property {name}"));
        assert_eq!(property.feature(), metadata, "{id} property metadata");
        assert_eq!(property.canonical_name(), name, "{id} canonical name");

        let source = format!("{name}: {authored_value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("{id}: expected one retained declaration");
        };
        assert_eq!(
            declaration
                .known()
                .expect("known C12 property")
                .property()
                .stable_id(),
            id,
            "{id} parser vector",
        );
    }

    let alias = feature_metadata("official.property-alias.glyph-orientation-vertical")
        .expect("legacy glyph-orientation-vertical metadata");
    assert_eq!(alias.kind(), CssFeatureKind::PropertyAlias);
    assert_eq!(alias.spelling(), "glyph-orientation-vertical");
    assert_eq!(alias.source().id().as_str(), "O-WRITING3");
    assert_eq!(alias.production(), "#propdef-glyph-orientation-vertical");
    assert_eq!(alias.status(), CssSupportStatus::Partial);
    assert!(alias.supported_subset().is_some());
    assert!(alias.unsupported_remainder().is_some());
    let alias_report = parse_style_attribute("glyph-orientation-vertical: 90");
    assert!(alias_report.is_clean(), "{:?}", alias_report.diagnostics());
    assert_eq!(
        alias_report.syntax()[0]
            .known()
            .expect("known legacy alias")
            .property(),
        CssKnownProperty::TextOrientation,
    );
    assert!(CssKnownProperty::TextOrientation.aliases().is_empty());
    assert_eq!(
        CssKnownProperty::from_name("glyph-orientation-vertical"),
        None
    );

    for (id, spelling, source, production, vector) in [
        (
            "official.value.box-edge-keywords",
            "content-box|padding-box|border-box|margin-box|fill-box|stroke-box|view-box",
            "O-BOX3",
            "#keywords",
            "transform-box: view-box",
        ),
        (
            "official.value.blend-mode",
            "normal|multiply|screen|overlay|darken|lighten|color-dodge|color-burn|hard-light|soft-light|difference|exclusion|hue|saturation|color|luminosity",
            "O-COMPOSITING1",
            "#blending,#blendingseparable,#blendingnonseparable",
            "background-blend-mode: multiply, luminosity",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        let report = parse_style_attribute(vector);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{id} parser vector");
    }

    for keyword in [
        "content-box",
        "padding-box",
        "border-box",
        "margin-box",
        "fill-box",
        "stroke-box",
        "view-box",
    ] {
        assert!(
            CssBoxEdgeKeyword::from_keyword(keyword).is_some(),
            "{keyword}"
        );
    }
    assert_eq!(CssBoxEdgeKeyword::from_keyword("content"), None);

    for keyword in [
        "normal",
        "multiply",
        "screen",
        "overlay",
        "darken",
        "lighten",
        "color-dodge",
        "color-burn",
        "hard-light",
        "soft-light",
        "difference",
        "exclusion",
        "hue",
        "saturation",
        "color",
        "luminosity",
    ] {
        assert!(CssBlendMode::from_keyword(keyword).is_some(), "{keyword}");
    }
    assert_eq!(CssBlendMode::from_keyword("plus-lighter"), None);
}

#[test]
fn font_source_complete_grammar_retains_legacy_controls_and_rejects_malformed_src() {
    // Fonts 4 src accepts local names and legacy format labels; Values 4 src()
    // still requires a quoted string payload.
    let malformed = parse_sheet("@font-face { src: src(foo); }");
    assert!(!malformed.is_clean());
    assert_eq!(malformed.diagnostics().len(), 1);
    assert_eq!(
        malformed.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
    let [CssRule::FontFace(face)] = malformed.syntax().rules() else {
        panic!("expected retained font face");
    };
    assert!(
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::Src)
            .is_none()
    );
    let report = parse_sheet("@font-face { src: local(system-ui); }");
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor,
    );
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("expected the retained font face");
    };
    assert!(
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::Src)
            .is_none()
    );

    for (name, decoded) in [("\"system-ui\"", "system-ui"), ("menu", "menu")] {
        let report = parse_sheet(&format!("@font-face {{ src: local({name}); }}"));
        assert!(report.is_clean());
        let [CssRule::FontFace(face)] = report.syntax().rules() else {
            panic!("expected a retained font face");
        };
        let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(
            source,
        ))) = face
            .descriptors()
            .effective(CssFontFaceDescriptorKind::Src)
            .map(|record| record.value())
        else {
            panic!("expected ordinary source descriptor");
        };
        let [surgeist_css::CssFontFaceSource::Local(local)] = source.sources() else {
            panic!("expected the local name");
        };
        assert_eq!(local.as_str(), decoded);
    }
    let report = parse_sheet("@font-face { src: url(demo.woff2) format(\"woff2-variations\"); }");
    assert!(report.is_clean());
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("expected a retained font face");
    };
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(value))) =
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::Src)
            .map(|record| record.value())
    else {
        panic!("expected ordinary source descriptor");
    };
    let [surgeist_css::CssFontFaceSource::Url(source)] = value.sources() else {
        panic!("expected a retained URL source");
    };
    assert_eq!(
        match source.format().unwrap() {
            surgeist_css::CssFontFormat::String(value) => value.as_str(),
            _ => panic!("string format"),
        },
        "woff2-variations"
    );
    assert_eq!(
        source
            .format()
            .and_then(surgeist_css::CssFontFormat::recognized_format),
        Some(surgeist_css::CssFontFormatHint::Woff2)
    );
    assert!(source.tech().is_empty());
    assert_eq!(
        source.required_technologies().collect::<Vec<_>>(),
        [surgeist_css::CssFontTechHint::Variations]
    );
}

#[test]
fn source_registry_lookups_are_exact_and_preserve_provenance_xor() {
    // The selected Snapshot edition is the 2026 REC; the 2018 REC remains a
    // separate retained reference witness, not this live registry source.
    let ui3 = specification_source("O-UI3").expect("selected UI3 source");
    assert_eq!(ui3.tier(), CssSpecificationTier::Snapshot2026Official);
    assert_eq!(ui3.level(), "3");
    assert_eq!(
        ui3.url(),
        Some("https://www.w3.org/TR/2026/REC-css-ui-3-20260407/")
    );
    assert_eq!(ui3.repository_provenance(), None);

    let url = specification_source("I-VALUES4-20240312").expect("dated URL source");
    assert_eq!(url.tier(), CssSpecificationTier::SurgeistExtension);
    assert_eq!(url.level(), "4");
    assert_eq!(
        url.url(),
        Some("https://www.w3.org/TR/2024/WD-css-values-4-20240312/")
    );
    assert_eq!(url.repository_provenance(), None);

    let filter = specification_source("X-BACKDROP-FILTER").expect("named backdrop source");
    assert_eq!(filter.module(), "Filter Effects");
    assert_eq!(filter.level(), "2 named property exception");
    assert_eq!(filter.tier(), CssSpecificationTier::SurgeistExtension);
    assert_eq!(
        filter.url(),
        Some(
            "https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs"
        )
    );
    assert_eq!(filter.repository_provenance(), None);

    for source in specification_sources() {
        assert_ne!(
            source.url().is_some(),
            source.repository_provenance().is_some()
        );
    }
    assert!(specification_source("o-color4").is_none());
    assert!(specification_source(" O-COLOR4").is_none());
    assert!(specification_source("O-COLOR4 ").is_none());
    assert!(specification_source("").is_none());
}

#[test]
fn source_registry_exposes_scrollbars_level_one_reliable_provenance() {
    let source = specification_source("R-SCROLLBARS1").expect("Scrollbars 1 source");
    assert_eq!(source.id().as_str(), "R-SCROLLBARS1");
    assert_eq!(source.module(), "CSS Scrollbars Styling");
    assert_eq!(source.level(), "1");
    assert_eq!(source.tier(), CssSpecificationTier::Snapshot2026Reliable);
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/")
    );
    assert_eq!(source.repository_provenance(), None);

    assert!(specification_source("r-scrollbars1").is_none());
    assert!(specification_source("R-SCROLLBARS1 ").is_none());
}

#[test]
fn source_registry_exposes_containment_level_three_extension_provenance() {
    let source = specification_source("X-CONTAIN3").expect("Containment 3 source");
    assert_eq!(source.id().as_str(), "X-CONTAIN3");
    assert_eq!(source.module(), "CSS Containment");
    assert_eq!(source.level(), "3");
    assert_eq!(source.tier(), CssSpecificationTier::SurgeistExtension);
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2022/WD-css-contain-3-20220818/")
    );
    assert_eq!(source.repository_provenance(), None);

    assert!(specification_source("x-contain3").is_none());
    assert!(specification_source(" X-CONTAIN3").is_none());
}

#[test]
fn container_scrollbar_and_generated_records_cite_their_dated_sources() {
    let scrollbar =
        feature_metadata("baseline.property.scrollbar-width").expect("scrollbar-width record");
    assert_eq!(scrollbar.source().id().as_str(), "R-SCROLLBARS1");
    assert_eq!(
        scrollbar.source().tier(),
        CssSpecificationTier::Snapshot2026Reliable
    );

    for id in [
        "baseline.rule.container",
        "baseline.container.condition",
        "baseline.container.size-feature",
    ] {
        let feature = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}`"));
        assert_eq!(feature.source().id().as_str(), "X-CONDITIONAL5", "{id}");
        assert_eq!(
            feature.source().tier(),
            CssSpecificationTier::SurgeistExtension
        );
    }

    let generated =
        feature_metadata("official.selector.generated").expect("atomic generated pseudo-element");
    assert_eq!(generated.source().id().as_str(), "O-SELECTORS3");

    let report = parse_sheet(".note::before { color: red; }");
    assert!(
        report.is_clean(),
        "atomic parser case: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().rules().len(), 1);
}

#[test]
fn every_pseudo_element_and_media_feature_record_has_public_parser_evidence() {
    // The supported subset and its boundary for pseudo-element suffixes and
    // media-query lists, ranges and discrete features.
    for source in [
        ".item::before { content: \"x\"; }",
        "@media screen and (min-width: 1px), print { .x { color: red; } }",
        "@media (orientation: landscape) { .x { color: red; } }",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    }
    for (source, code) in [
        (
            ".item::part() { color: red; }",
            CssErrorCode::InvalidSelector,
        ),
        // Unknown features stay clean; an incomplete outer conjunction is invalid.
        (
            "@media (width >= 1px) and { .x { color: red; } }",
            CssErrorCode::InvalidMediaQuery,
        ),
        // Scripting is admitted, but typed queries prohibit an unwrapped outer `or`.
        (
            "@media screen and (scripting: enabled) or (color) { .x { color: red; } }",
            CssErrorCode::InvalidMediaQuery,
        ),
    ] {
        let report = parse_sheet(source);
        assert_eq!(
            report.diagnostics().first().map(|d| d.error().code()),
            Some(code),
            "{source}"
        );
    }

    let clean_cases = [
        (
            "official.selector.generated",
            "O-SELECTORS3",
            ".x::after { color: red; }",
        ),
        (
            "ext.pseudo-element.marker",
            "X-PSEUDO4",
            "li::marker { color: red; }",
        ),
        (
            "ext.pseudo-element.selection",
            "X-PSEUDO4",
            "::selection { color: red; }",
        ),
        (
            "ext.pseudo-element.backdrop",
            "X-PSEUDO4",
            ".dialog::backdrop { color: red; }",
        ),
        (
            "ext.pseudo-element.generated-marker",
            "X-PSEUDO4",
            ".item::before::marker { color: red; }",
        ),
        (
            "ext.pseudo-element.prefix",
            "X-PSEUDO4",
            "::prefix { color: red; }",
        ),
        (
            "ext.pseudo-element.suffix",
            "X-PSEUDO4",
            "::suffix { color: red; }",
        ),
        (
            "ext.pseudo-element.search-text",
            "X-PSEUDO4",
            ".x::SEARCH-TEXT { color: red; }",
        ),
        (
            "ext.pseudo-element.target-text",
            "X-PSEUDO4",
            ".x::target-text { color: red; }",
        ),
        (
            "ext.pseudo-element.spelling-error",
            "X-PSEUDO4",
            ".x::spelling-error { color: red; }",
        ),
        (
            "ext.pseudo-element.grammar-error",
            "X-PSEUDO4",
            ".x::grammar-error { color: red; }",
        ),
        (
            "ext.pseudo-element.highlight",
            "X-PSEUDO4",
            ".x::highlight(none) { color: red; }",
        ),
        (
            "ext.pseudo-element.placeholder",
            "X-PSEUDO4",
            "::slotted(.item)::placeholder { color: red; }",
        ),
        (
            "ext.pseudo-element.file-selector-button",
            "X-PSEUDO4",
            "::slotted(.item)::file-selector-button { color: red; }",
        ),
        (
            "ext.pseudo-element.details-content",
            "X-PSEUDO4",
            "details::details-content::part(label) { color: red; }",
        ),
        (
            "ext.pseudo-element.search-text-current",
            "X-PSEUDO4",
            ".x::search-text:not(:current) { color: red; }",
        ),
        (
            "ext.pseudo-element.unknown-webkit",
            "I-SELECTORS4",
            ".x::-WebKit-ASDF { color: red; }",
        ),
        (
            "ext.selector.autofill",
            "I-SELECTORS4",
            "input:autofill { color: red; }",
        ),
        (
            "ext.selector.autofill",
            "I-SELECTORS4",
            "input:-webkit-autofill { color: red; }",
        ),
        (
            "official.media.query-list-core",
            "X-MEDIA5",
            "@media screen, print { .x { color: red; } }",
        ),
        (
            "ext.media.condition-syntax",
            "X-MEDIA5",
            "@media not screen and (width: 1px) { .x { color: red; } }",
        ),
        (
            "official.media.feature.width",
            "X-MEDIA5",
            "@media (min-width: 1px) { .x { color: red; } }",
        ),
        (
            "official.media.feature.height",
            "X-MEDIA5",
            "@media (max-height: 2px) { .x { color: red; } }",
        ),
        (
            "official.media.feature.resolution",
            "X-MEDIA5",
            "@media (resolution: 192dpi) { .x { color: red; } }",
        ),
        (
            "ext.media.resolution.dppx",
            "X-MEDIA5",
            "@media (resolution: 2dppx) { .x { color: red; } }",
        ),
        (
            "official.media.feature.color",
            "X-MEDIA5",
            "@media (color: 8) { .x { color: red; } }",
        ),
        (
            "official.media.feature.monochrome",
            "X-MEDIA5",
            "@media (monochrome: 1) { .x { color: red; } }",
        ),
        (
            "ext.media.range.width",
            "X-MEDIA5",
            "@media (width >= 1px) { .x { color: red; } }",
        ),
        (
            "ext.media.range.height",
            "X-MEDIA5",
            "@media (height < 2px) { .x { color: red; } }",
        ),
        (
            "ext.media.range.resolution",
            "X-MEDIA5",
            "@media (resolution > 1dppx) { .x { color: red; } }",
        ),
        (
            "ext.media.range.color",
            "X-MEDIA5",
            "@media (color >= 8) { .x { color: red; } }",
        ),
        (
            "ext.media.range.monochrome",
            "X-MEDIA5",
            "@media (monochrome = 1) { .x { color: red; } }",
        ),
        (
            "official.media.feature.orientation",
            "X-MEDIA5",
            "@media (orientation: portrait) { .x { color: red; } }",
        ),
        (
            "ext.media.hover",
            "X-MEDIA5",
            "@media (hover: hover) { .x { color: red; } }",
        ),
        (
            "ext.media.any-hover",
            "X-MEDIA5",
            "@media (any-hover: none) { .x { color: red; } }",
        ),
        (
            "ext.media.pointer",
            "X-MEDIA5",
            "@media (pointer: fine) { .x { color: red; } }",
        ),
        (
            "ext.media.any-pointer",
            "X-MEDIA5",
            "@media (any-pointer: coarse) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-color-scheme",
            "X-MEDIA5",
            "@media (prefers-color-scheme: dark) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-reduced-motion",
            "X-MEDIA5",
            "@media (prefers-reduced-motion: reduce) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-reduced-transparency",
            "X-MEDIA5",
            "@media (prefers-reduced-transparency: reduce) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-contrast",
            "X-MEDIA5",
            "@media (prefers-contrast: more) { .x { color: red; } }",
        ),
        (
            "ext.media.forced-colors",
            "X-MEDIA5",
            "@media (forced-colors: active) { .x { color: red; } }",
        ),
        (
            "ext.media.display-mode",
            "X-MEDIA5",
            "@media (display-mode: picture-in-picture) { .x { color: red; } }",
        ),
    ];
    for (id, source_id, css) in clean_cases {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}`"));
        assert_eq!(metadata.source().id().as_str(), source_id, "{id}");
        let report = parse_sheet(css);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 1, "{id}");
    }

    let malformed = feature_metadata("ext.media.malformed-member-never")
        .expect("malformed-member recovery target");
    assert_eq!(malformed.source().id().as_str(), "X-MEDIA5");
    let source = "@media screen, ??? { .x { color: red; } }";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one malformed-member diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("???").expect("responsible malformed member")
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find(" ???").expect("malformed recovery unit")
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find('{').expect("end of media query list")
    );
}

#[test]
fn official_two_dimensional_transform_metadata_matches_typed_functions() {
    let report = parse_style_attribute(concat!(
        "transform: matrix(1, 0, 0, 1, 10, 20) translate(1px, 2%) ",
        "translateX(3px) translateY(4%) scale(1.5, 2) scaleX(.5) scaleY(2) ",
        "rotate(45deg) skew(10deg, 20deg) skewX(5deg) skewY(6deg)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .expect("known transform")
        .property_value()
        .expect("ordinary transform")
    else {
        panic!("expected transform value");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected typed transform functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssTransformFunction::Matrix(_),
            CssTransformFunction::Translate(_),
            CssTransformFunction::TranslateX(_),
            CssTransformFunction::TranslateY(_),
            CssTransformFunction::Scale(_),
            CssTransformFunction::ScaleX(_),
            CssTransformFunction::ScaleY(_),
            CssTransformFunction::Rotate(_),
            CssTransformFunction::Skew(_),
            CssTransformFunction::SkewX(_),
            CssTransformFunction::SkewY(_),
        ]
    ));
}

#[test]
fn transform_matrix3d_exposes_sixteen_finite_components() {
    let report = parse_style_attribute(
        "transform: matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected transform functions");
    };
    let CssTransformFunction::Matrix3d(matrix) = &functions.functions()[0] else {
        panic!("expected matrix3d");
    };
    assert_eq!(matrix.components().len(), 16);
}

#[test]
fn transform_perspective_accepts_none_and_zero_and_rejects_invalid_dimensions() {
    let report = parse_style_attribute("transform: perspective(none) perspective(0)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected transform functions");
    };
    assert!(matches!(
        functions.functions()[0],
        CssTransformFunction::Perspective(CssTransformPerspective::None)
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssTransformFunction::Perspective(CssTransformPerspective::Length(length))
            if matches!(length.literal_component().map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Number(number))) if number.representation() == "0")
    ));
    assert!(!parse_style_attribute("transform: perspective(10%)").is_clean());
    assert!(!parse_style_attribute("transform: perspective(-1px)").is_clean());
}

#[test]
fn transform_three_dimensional_rotations_are_typed() {
    let report = parse_style_attribute(
        "transform: rotate3d(1, 0, -1, 45deg) rotateX(10deg) rotateY(20deg) rotateZ(30deg)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected transform functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssTransformFunction::Rotate3d(_),
            CssTransformFunction::RotateX(_),
            CssTransformFunction::RotateY(_),
            CssTransformFunction::RotateZ(_),
        ]
    ));
}

#[test]
fn transform_three_dimensional_scales_preserve_number_and_percentage_operands() {
    let report = parse_style_attribute("transform: scale3d(1, 50%, 2) scaleZ(125%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected transform functions");
    };
    let CssTransformFunction::Scale3d(scale) = &functions.functions()[0] else {
        panic!("expected scale3d");
    };
    assert!(matches!(scale.x(), CssTransformScaleComponent::Number(_)));
    assert!(matches!(
        scale.y(),
        CssTransformScaleComponent::Percentage(_)
    ));
    assert!(matches!(
        functions.functions()[1],
        CssTransformFunction::ScaleZ(CssTransformScaleComponent::Percentage(_))
    ));
}

#[test]
fn transform_three_dimensional_translations_keep_z_length_only() {
    let report = parse_style_attribute("transform: translate3d(10%, 2px, 4em) translateZ(3px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected transform functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssTransformFunction::Translate3d(_),
            CssTransformFunction::TranslateZ(_),
        ]
    ));
    assert!(!parse_style_attribute("transform: translate3d(1px, 2px, 3%)").is_clean());
}

#[test]
fn official_easing_metadata_matches_typed_functions() {
    let report = parse_style_attribute(
        "transition-timing-function: ease, cubic-bezier(.25, 0, .75, 1), steps(2, jump-none), linear(0, 1)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) = report.syntax()[0]
        .known()
        .expect("known transition timing")
        .property_value()
        .expect("ordinary transition timing")
    else {
        panic!("expected transition timing value");
    };
    assert!(matches!(
        value.timing_functions().values(),
        [
            CssEasing::Keyword(_),
            CssEasing::CubicBezier(_),
            CssEasing::Steps(_),
            CssEasing::Linear(_),
        ]
    ));
    let source = feature_metadata("official.value.linear-easing")
        .unwrap()
        .source();
    assert_eq!(source.tier(), CssSpecificationTier::LaterStandard);
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2024/WD-css-easing-2-20240829/")
    );
}

#[test]
fn official_shadow_metadata_matches_typed_box_shadow() {
    let report = parse_style_attribute("box-shadow: red inset -1px 2px 3px -4px");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BoxShadow(value) = report.syntax()[0]
        .known()
        .expect("known box shadow")
        .property_value()
        .expect("ordinary box shadow")
    else {
        panic!("expected box-shadow value");
    };
    let CssBoxShadow::Shadows(shadows) = value.value() else {
        panic!("expected typed shadows");
    };
    assert!(shadows.shadows()[0].inset());
    assert!(shadows.shadows()[0].color().is_some());
}

#[test]
fn filter_function_list_preserves_typed_authored_order() {
    let report = parse_style_attribute(
        "filter: url(\"filters.svg#rough\") blur(2px) drop-shadow(1px 2px red) opacity(50%)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .expect("known filter")
        .property_value()
        .expect("ordinary filter")
    else {
        panic!("expected filter value");
    };
    let CssFilter::Functions(functions) = value.value() else {
        panic!("expected typed filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssFilterFunction::Url(_),
            CssFilterFunction::Blur(_),
            CssFilterFunction::DropShadow(_),
            CssFilterFunction::Opacity(_),
        ]
    ));
}

#[test]
fn every_filter_amount_function_has_exact_typed_domain() {
    let report = parse_style_attribute(concat!(
        "filter: blur(2px) brightness() contrast(25%) grayscale(.5) ",
        "hue-rotate(45deg) invert(10%) opacity(.75) saturate(2) sepia(30%)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter value");
    };
    let CssFilter::Functions(functions) = value.value() else {
        panic!("expected typed filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssFilterFunction::Blur(_),
            CssFilterFunction::Brightness(_),
            CssFilterFunction::Contrast(_),
            CssFilterFunction::Grayscale(_),
            CssFilterFunction::HueRotate(_),
            CssFilterFunction::Invert(_),
            CssFilterFunction::Opacity(_),
            CssFilterFunction::Saturate(_),
            CssFilterFunction::Sepia(_),
        ]
    ));
}

#[test]
fn drop_shadow_rejects_box_shadow_only_components() {
    let report = parse_style_attribute("filter: drop-shadow(red 1px 2px 3px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter value");
    };
    let CssFilter::Functions(functions) = value.value() else {
        panic!("expected typed filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [CssFilterFunction::DropShadow(_)]
    ));
    assert!(!parse_style_attribute("filter: drop-shadow(inset 1px 2px)").is_clean());
    assert!(!parse_style_attribute("filter: drop-shadow(1px 2px 3px 4px)").is_clean());
}

#[test]
fn all_basic_shape_functions_have_complete_authored_metadata() {
    for value in [
        "path('M 0 0 L 1 1')",
        "rect(0 1px 1px 0)",
        "xywh(0 0 1px 1px)",
        "inset(1px)",
        "circle(10px)",
        "ellipse(10px 20%)",
        "polygon(0 0, 100% 0)",
        "shape(from 0 0, line to 1px 1px)",
    ] {
        let report = parse_style_attribute(&format!("clip-path: {value}"));
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    }
}
#[test]
fn every_selected_basic_shape_has_typed_public_components() {
    let report = parse_style_attribute(concat!(
        "clip-path: inset(1px round 2px); ",
        "clip-path: circle(10px at center); ",
        "clip-path: ellipse(10px 20% at left top); ",
        "clip-path: polygon(evenodd round 2px, 0 0, 100% 0)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let expected = ["inset", "circle", "ellipse", "polygon"];
    for (declaration, expected) in report.syntax().iter().zip(expected) {
        let CssKnownPropertyValueRef::ClipPath(value) = declaration
            .known()
            .expect("known clip path")
            .property_value()
            .expect("ordinary clip path")
        else {
            panic!("expected clip-path value");
        };
        let CssClipPath::BasicShape(shape) = value.value() else {
            panic!("expected current {expected} shape");
        };
        let shape = shape.shape();
        assert!(
            matches!(
                (expected, shape),
                ("inset", CssBasicShape::Inset(_))
                    | ("circle", CssBasicShape::Circle(_))
                    | ("ellipse", CssBasicShape::Ellipse(_))
                    | ("polygon", CssBasicShape::Polygon(_))
            ),
            "expected {expected}"
        );
        if let CssBasicShape::Polygon(polygon) = shape {
            assert!(polygon.round().is_some(), "polygon round component");
        }
    }
}

#[test]
fn backdrop_filter_preserves_typed_authored_grammar_with_the_named_source() {
    let report = parse_style_attribute("backdrop-filter: blur(2px) opacity(50%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackdropFilter(value) = report.syntax()[0]
        .known()
        .expect("known backdrop filter")
        .property_value()
        .expect("ordinary backdrop filter")
    else {
        panic!("expected backdrop-filter value");
    };
    assert!(matches!(
        value.value(),
        CssFilter::Functions(functions)
            if matches!(
                functions.functions(),
                [CssFilterFunction::Blur(_), CssFilterFunction::Opacity(_)]
            )
    ));
}

#[test]
fn clip_path_complete_metadata_matches_all_authored_alternatives() {
    for value in [
        "path('M 0 0 L 1 1')",
        "rect(0 1px 1px 0)",
        "xywh(0 0 1px 1px)",
        "none",
        "url(\"shapes.svg#clip\")",
        "inset(1px)",
        "circle(10px)",
        "ellipse(10px 20%)",
        "polygon(round 2px, 0 0, 100% 0)",
        "border-box",
        "border-box circle(10px)",
        "ellipse() view-box",
        "shape(from 0 0, line to 1px 1px)",
        "circle(at x-start y-end)",
        "ellipse(at block-start inline-end)",
    ] {
        let report = parse_style_attribute(&format!("clip-path: {value}"));
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    }
}

#[test]
fn completed_function_property_metadata_matches_public_current_accessors() {
    let report = parse_style_attribute(concat!(
        "transform: translate(1px, 2%); ",
        "box-shadow: 1px 2px 3px red; ",
        "filter: blur(2px); ",
        "transition-timing-function: cubic-bezier(.25, 0, .75, 1); ",
        "animation-timing-function: steps(2, jump-none)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::Transform(value)
            if matches!(value.value(), CssTransform::Functions(_))
    ));
    assert!(matches!(
        report.syntax()[1]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::BoxShadow(value)
            if matches!(value.value(), CssBoxShadow::Shadows(_))
    ));
    assert!(matches!(
        report.syntax()[2]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::Filter(value)
            if matches!(value.value(), CssFilter::Functions(_))
    ));
    assert!(matches!(
        report.syntax()[3]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::TransitionTimingFunction(value)
            if matches!(value.timing_functions().values(), [CssEasing::CubicBezier(_)])
    ));
    assert!(matches!(
        report.syntax()[4]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::AnimationTimingFunction(value)
            if matches!(value.timing_functions().values(), [CssEasing::Steps(_)])
    ));
}

#[test]
fn exclusion_registry_exposes_named_official_audit_facts() {
    // UI3 REC 2026 §6.3.1 and UI4 WD 2026 §5.3.2 obsolete ime-mode;
    // neither defines a current authored value production.
    let ui3 = specification_source("O-UI3").unwrap();
    let ime = conformance_exclusion("excluded.O-UI3.property.ime-mode").unwrap();
    assert_eq!(ime.source(), *ui3);
    assert_eq!(ime.production(), "#input-method-editor");
    assert_eq!(
        ime.reason(),
        CssExclusionReason::SupersededWithoutCurrentProduction
    );
    assert_eq!(
        ime.superseding_ids().map(|ids| ids[0].as_str()),
        Some("X-UI4")
    );
    let ui4_ime = conformance_exclusion("excluded.X-UI4.property.ime-mode").unwrap();
    assert_eq!(ui4_ime.source(), *specification_source("X-UI4").unwrap());
    assert_eq!(ui4_ime.production(), "#input-method-editor");
    assert_eq!(ui4_ime.reason(), ime.reason());
    assert!(CssKnownProperty::from_name("ime-mode").is_none());

    // These clauses exclude execution, not their admitted property grammars.
    let ui3_behavior = conformance_exclusion("excluded.O-UI3.behavior").unwrap();
    assert_eq!(ui3_behavior.source(), *ui3);
    assert_eq!(
        ui3_behavior.production(),
        "#resize; #text-overflow; #ellipsing-details; #ellipsis-interaction; #ellipsis-scrolling; #cursor; #canvas_cursor; #caret-color (rendering and user interaction only)"
    );
    assert_eq!(
        ui3_behavior.reason(),
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    );
    assert!(ui3_behavior.superseding_ids().is_none());
    let ui3_audit = conformance_exclusion("excluded.O-UI3.informative-audit").unwrap();
    assert_eq!(ui3_audit.source(), *ui3);
    assert_eq!(ui3_audit.reason(), CssExclusionReason::InformativeOnly);

    let predecessor = conformance_exclusion("excluded.O-CSS2.property.margin")
        .expect("superseded CSS2 margin definition");
    assert_eq!(predecessor.source().id().as_str(), "O-CSS2");
    assert_eq!(predecessor.production(), "box.html#propdef-margin");
    assert_eq!(
        predecessor.reason(),
        CssExclusionReason::SupersededWithoutCurrentProduction
    );
    assert_eq!(
        predecessor.superseding_ids().map(|ids| ids[0].as_str()),
        Some("baseline.property.margin")
    );

    let informative = conformance_exclusion("excluded.O-CSS2.informative-property.azimuth")
        .expect("informative CSS2 Appendix A property row");
    assert_eq!(informative.production(), "aural.html#propdef-azimuth");
    assert_eq!(informative.reason(), CssExclusionReason::InformativeOnly);
    assert_eq!(informative.superseding_ids(), None);

    let processing = conformance_exclusion("excluded.O-IMAGES3.processing")
        .expect("out-of-boundary Images processing row");
    assert_eq!(
        processing.reason(),
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    );

    let audit = conformance_exclusion("excluded.O-COLOR4.informative-audit")
        .expect("Color 4 informative source audit row");
    assert_eq!(audit.source().id().as_str(), "O-COLOR4");
    assert_eq!(audit.reason(), CssExclusionReason::InformativeOnly);
    assert!(audit.superseding_ids().is_none());

    assert!(
        conformance_exclusions()
            .iter()
            .any(|row| row.id() == audit.id())
    );
    assert!(conformance_exclusion("EXCLUDED.O-COLOR4.INFORMATIVE-AUDIT").is_none());
    assert!(conformance_exclusion(" excluded.O-COLOR4.informative-audit").is_none());
    assert!(conformance_exclusion("excluded.O-COLOR4.informative-audit ").is_none());
}

#[test]
fn images_three_properties_are_publicly_recognized() {
    let object = parse_style_attribute("object-position: left top");
    assert!(object.is_clean(), "{:?}", object.diagnostics());
    assert_eq!(
        object.syntax()[0]
            .known()
            .expect("known object position")
            .property(),
        CssKnownProperty::ObjectPosition,
    );

    for (property, value) in [
        ("object-fit", "cover"),
        ("image-rendering", "pixelated"),
        ("image-orientation", "from-image"),
    ] {
        let source = format!("{property}: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained Images 3 declaration")
                .property()
                .canonical_name(),
            property,
            "{source}",
        );
        assert_eq!(
            report.syntax()[1]
                .known()
                .expect("retained color declaration")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );
    }
}

#[test]
fn images_and_backgrounds_records_parse_their_authored_vectors() {
    let image_and_background_records = [
        (
            "official.property.border-image",
            CssFeatureKind::Property,
            "border-image",
            "O-BACKGROUNDS3",
            "#propdef-border-image",
            "border-image: url(frame.png) 10 fill / 2 / 1 round",
        ),
        (
            "official.property.border-image-outset",
            CssFeatureKind::Property,
            "border-image-outset",
            "O-BACKGROUNDS3",
            "#propdef-border-image-outset",
            "border-image-outset: 1 2px 3 4px",
        ),
        (
            "official.property.border-image-repeat",
            CssFeatureKind::Property,
            "border-image-repeat",
            "O-BACKGROUNDS3",
            "#propdef-border-image-repeat",
            "border-image-repeat: round space",
        ),
        (
            "official.property.border-image-slice",
            CssFeatureKind::Property,
            "border-image-slice",
            "O-BACKGROUNDS3",
            "#propdef-border-image-slice",
            "border-image-slice: 10 fill",
        ),
        (
            "official.property.border-image-source",
            CssFeatureKind::Property,
            "border-image-source",
            "O-BACKGROUNDS3",
            "#propdef-border-image-source",
            "border-image-source: linear-gradient(red, blue)",
        ),
        (
            "official.property.border-image-width",
            CssFeatureKind::Property,
            "border-image-width",
            "O-BACKGROUNDS3",
            "#propdef-border-image-width",
            "border-image-width: 1 auto 25% 4px",
        ),
        (
            "official.property.image-orientation",
            CssFeatureKind::Property,
            "image-orientation",
            "O-IMAGES3",
            "#propdef-image-orientation",
            "image-orientation: 90deg flip",
        ),
        (
            "official.property.image-rendering",
            CssFeatureKind::Property,
            "image-rendering",
            "O-IMAGES3",
            "#propdef-image-rendering",
            "image-rendering: crisp-edges",
        ),
        (
            "official.property.object-fit",
            CssFeatureKind::Property,
            "object-fit",
            "O-IMAGES3",
            "#propdef-object-fit",
            "object-fit: scale-down",
        ),
        (
            "official.value.background-layer",
            CssFeatureKind::Value,
            "<bg-layer>",
            "O-BACKGROUNDS3",
            "#layering",
            "background: url(hero.png) left top / cover no-repeat fixed border-box",
        ),
        (
            "official.value.background-image",
            CssFeatureKind::Value,
            "<bg-image>",
            "O-BACKGROUNDS3",
            "#background-image",
            "background-image: url(hero.png), none",
        ),
        (
            "official.value.repeat-style",
            CssFeatureKind::Value,
            "<repeat-style>",
            "O-BACKGROUNDS3",
            "#background-repeat",
            "background-repeat: repeat-x, no-repeat round",
        ),
        (
            "official.value.background-attachment",
            CssFeatureKind::Value,
            "<attachment>",
            "O-BACKGROUNDS3",
            "#background-attachment",
            "background-attachment: fixed, local",
        ),
        (
            "official.value.background-size",
            CssFeatureKind::Value,
            "<bg-size>",
            "O-BACKGROUNDS3",
            "#background-size",
            "background-size: cover, 10px auto",
        ),
        (
            "official.value.line-style",
            CssFeatureKind::Value,
            "<line-style>",
            "O-BACKGROUNDS3",
            "#border-style",
            "border-style: none hidden dotted dashed",
        ),
        (
            "official.value.line-width",
            CssFeatureKind::Value,
            "<line-width>",
            "O-BACKGROUNDS3",
            "#border-width",
            "border-width: thin 2px medium thick",
        ),
        (
            "official.value.image",
            CssFeatureKind::Value,
            "<image>",
            "O-IMAGES3",
            "#image-values",
            "background-image: url(hero.png)",
        ),
        (
            "official.value.gradient",
            CssFeatureKind::Value,
            "<gradient>",
            "O-IMAGES3",
            "#gradients",
            "background-image: linear-gradient(red, blue)",
        ),
        (
            "official.value.linear-gradient",
            CssFeatureKind::Value,
            "linear-gradient()",
            "O-IMAGES3",
            "#linear-gradients",
            "background-image: linear-gradient(to right, red, blue)",
        ),
        (
            "official.value.radial-gradient",
            CssFeatureKind::Value,
            "radial-gradient()",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(circle, red, blue)",
        ),
        (
            "official.value.repeating-linear-gradient",
            CssFeatureKind::Value,
            "repeating-linear-gradient()",
            "O-IMAGES3",
            "#repeating-gradients",
            "background-image: repeating-linear-gradient(45deg, #000 10px, #fff 30px)",
        ),
        (
            "official.value.repeating-radial-gradient",
            CssFeatureKind::Value,
            "repeating-radial-gradient()",
            "O-IMAGES3",
            "#repeating-gradients",
            "background-image: repeating-radial-gradient(ellipse 20% 30% at center, red, blue 40px)",
        ),
        (
            "official.value.color-stop-list",
            CssFeatureKind::Value,
            "<color-stop-list>",
            "O-IMAGES3",
            "#color-stop-syntax",
            "background-image: linear-gradient(red 10%, 20%, blue 90%)",
        ),
        (
            "official.value.side-or-corner",
            CssFeatureKind::Value,
            "<side-or-corner>",
            "O-IMAGES3",
            "#linear-gradients",
            "background-image: linear-gradient(to right bottom, red, blue)",
        ),
        (
            "official.value.radial-shape",
            CssFeatureKind::Value,
            "<radial-shape>",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(circle, red, blue)",
        ),
        (
            "official.value.radial-size",
            CssFeatureKind::Value,
            "<radial-size>",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(ellipse 20% 30%, red, blue)",
        ),
        (
            "official.value.radial-extent",
            CssFeatureKind::Value,
            "<radial-extent>",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(closest-side, red, blue)",
        ),
    ];
    assert_eq!(image_and_background_records.len(), 27);

    for (id, kind, spelling, source, production, vector) in image_and_background_records {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");

        if kind == CssFeatureKind::Property {
            let property = property_support_metadata(spelling)
                .unwrap_or_else(|| panic!("missing property metadata for {spelling}"));
            assert!(std::ptr::eq(metadata, property.feature()), "{id} owner");
        }

        let report = parse_style_attribute(vector);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{id} parser vector");
    }

    let promoted_properties = [
        "background",
        "background-color",
        "background-image",
        "background-size",
        "background-repeat",
        "background-origin",
        "background-clip",
        "background-attachment",
        "border",
        "border-top",
        "border-right",
        "border-bottom",
        "border-left",
        "border-width",
        "border-top-width",
        "border-right-width",
        "border-bottom-width",
        "border-left-width",
        "border-color",
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
        "border-style",
        "border-top-style",
        "border-right-style",
        "border-bottom-style",
        "border-left-style",
        "border-radius",
        "border-top-left-radius",
        "border-top-right-radius",
        "border-bottom-right-radius",
        "border-bottom-left-radius",
    ];
    assert_eq!(promoted_properties.len(), 33);
    for name in promoted_properties {
        let metadata = property_support_metadata(name)
            .unwrap_or_else(|| panic!("missing promoted property metadata for {name}"))
            .feature();
        assert_eq!(metadata.source().id().as_str(), "O-BACKGROUNDS3", "{name}");
        assert_eq!(metadata.production(), format!("#propdef-{name}"), "{name}");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{name}");
    }

    for (id, source) in [
        ("baseline.property.background-position", "O-BACKGROUNDS3"),
        ("official.property.object-position", "O-IMAGES3"),
        ("baseline.property.box-shadow", "O-BACKGROUNDS3"),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing preserved {id}"));
        assert_eq!(metadata.source().id().as_str(), source, "{id}");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
    }
}

#[test]
fn linguistic_selector_metadata_matches_effective_authored_grammar() {
    for (id, spelling, production) in [
        ("official.selector.lang", ":lang()", "#the-lang-pseudo"),
        ("ext.selector.dir", ":dir()", "#the-dir-pseudo"),
    ] {
        let metadata = feature_metadata(id).expect("linguistic selector metadata");
        assert_eq!(metadata.kind(), CssFeatureKind::Selector);
        assert_eq!(metadata.spelling(), spelling);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.source().id().as_str(), "I-SELECTORS4");
        assert_eq!(metadata.production(), production);
    }
    for selector in [
        ":dir(ltr)",
        ":dir(future)",
        ":lang(en)",
        ":lang('')",
        ":lang(en, 'de-DE')",
    ] {
        let source = format!("{selector} {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().rules().len(), 1);
    }
    for selector in [":dir(1)", ":dir('ltr')", ":lang(en en)", ":lang(en,)"] {
        let source = format!("{selector} {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(report.syntax().rules().is_empty(), "{source}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidSelector
        );
    }
}

#[test]
fn required_shadow_definitions_supersede_prior_negative_catalog_examples() {
    for source in [
        ".host:host { color: red; }",
        ".item::part(label) { color: red; }",
    ] {
        let report = surgeist_css::parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(report.syntax().rules().len(), 1);
    }
}

#[test]
fn container_property_metadata_cites_each_selected_conditional_definition() {
    for (name, id, production, value) in [
        (
            "container-type",
            "official.property.container-type",
            "#propdef-container-type",
            "size scroll-state",
        ),
        (
            "container-name",
            "official.property.container-name",
            "#propdef-container-name",
            "Pane Pane",
        ),
        (
            "container",
            "official.property.container",
            "#propdef-container",
            "Pane / inline-size",
        ),
    ] {
        let metadata = feature_metadata(id).expect("container property metadata");
        assert_eq!(metadata.kind(), CssFeatureKind::Property);
        assert_eq!(metadata.spelling(), name);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.source().id().as_str(), "X-CONDITIONAL5");
        assert_eq!(
            metadata.source().url(),
            Some("https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/")
        );
        assert_eq!(metadata.production(), production);
        assert!(parse_style_attribute(&format!("{name}:{value}")).is_clean());
    }
}

#[test]
fn visibility_and_overflow_use_their_selected_definitions() {
    let visibility = feature_metadata("baseline.property.visibility").unwrap();
    assert_eq!(visibility.source().id().as_str(), "S-DISPLAY3");
    assert_eq!(visibility.production(), "#propdef-visibility");
    assert_eq!(visibility.status(), CssSupportStatus::Complete);
    let overflow = feature_metadata("baseline.property.overflow").unwrap();
    assert_eq!(overflow.source().id().as_str(), "X-OVERFLOW3");
    assert_eq!(overflow.production(), "#propdef-overflow");
}

#[test]
fn canonical_writing_modes_sources_include_selected_level_four_sideways_values() {
    for (feature, source, production) in [
        (
            "baseline.property.direction",
            "O-WRITING3",
            "#propdef-direction",
        ),
        (
            "official.property.unicode-bidi",
            "O-WRITING3",
            "#propdef-unicode-bidi",
        ),
        (
            "baseline.property.writing-mode",
            "S-WRITING4",
            "#propdef-writing-mode",
        ),
        (
            "official.property.text-orientation",
            "O-WRITING3",
            "#propdef-text-orientation",
        ),
    ] {
        let metadata = feature_metadata(feature).unwrap();
        assert_eq!(metadata.source().id().as_str(), source);
        assert_eq!(metadata.production(), production);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
    }
    for text in ["sideways-rl", "sideways-lr"] {
        assert!(parse_style_attribute(&format!("writing-mode:{text}")).is_clean());
    }
}

#[test]
fn text_combine_upright_uses_selected_level_four_digits_grammar() {
    let metadata = feature_metadata("official.property.text-combine-upright").unwrap();
    assert_eq!(metadata.source().id().as_str(), "S-WRITING4");
    assert_eq!(metadata.production(), "#propdef-text-combine-upright");
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    for text in ["digits", "digits 2", "digits 4", "digits calc(2.5)"] {
        assert!(parse_style_attribute(&format!("text-combine-upright:{text}")).is_clean());
    }
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}

#[test]
fn complete_z_index_catalogue_matches_the_typed_authored_lifecycle() {
    for (text, expected) in [
        ("AUTO", "auto"),
        ("+0002147483648", "2147483648"),
        ("calc(1.5)", "calc(1.5)"),
    ] {
        let report = parse_style_attribute(&format!("z-index:{text}!important"));
        assert!(report.is_clean());
        let source = &report.syntax()[0];
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(source).unwrap()
        else {
            panic!("one longhand")
        };
        let [item] = values.items() else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::ZIndex(value) = item.ordinary_value().unwrap().view() else {
            panic!("z-index payload")
        };
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn quirky_length_metadata_matches_scoped_pixel_admission() {
    let metadata = feature_metadata("official.value.quirky-length").unwrap();
    assert_eq!(metadata.kind(), CssFeatureKind::Value);
    assert_eq!(metadata.source().id().as_str(), "I-VALUES4-20240312");
    assert_eq!(metadata.production(), "#deprecated-quirky-length");
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    let context = CssParserContext::new(CssParserMode::Quirks);
    let report = context.parse_declaration("margin-top:7");
    assert!(report.is_clean());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "margin-top: 7px;"
    );
    assert!(!parse_declaration("margin-top:7").is_clean());
    assert!(!context.parse_declaration("background:7 8").is_clean());
}
