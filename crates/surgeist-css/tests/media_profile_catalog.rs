//! The selected MQ5 edition supersedes MQ4; public support metadata identifies
//! the effective definitions while retaining historical feature lookup IDs.
use surgeist_css::{
    CssFeatureKind, CssMediaConditionKind, CssMediaQuery, CssNormalizedItem, CssRuleContextKindRef,
    CssSupportStatus, feature_metadata, normalize_sheet, parse_component_values, parse_media_query,
    parse_sheet, validate_sheet,
};

const MQ5: &str = "https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/";

fn known(source: &str, name: &str) {
    let report = parse_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("condition query: {source}");
    };
    let CssMediaConditionKind::Feature(feature) = condition.kind() else {
        panic!("known feature: {source}");
    };
    assert_eq!(feature.name(), name);
}

fn effective(id: &str) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing {id}"));
    assert_eq!(metadata.kind(), CssFeatureKind::MediaQuery, "{id}");
    assert_eq!(metadata.source().url(), Some(MQ5), "{id}");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
    assert!(metadata.supported_subset().is_none(), "{id}");
    assert!(metadata.unsupported_remainder().is_none(), "{id}");
}

#[test]
fn existing_feature_identities_resolve_to_the_effective_published_grammar() {
    for name in [
        "width",
        "height",
        "device-width",
        "device-height",
        "aspect-ratio",
        "device-aspect-ratio",
        "resolution",
        "color",
        "color-index",
        "monochrome",
        "orientation",
        "scan",
        "grid",
    ] {
        known(&format!("({name})"), name);
        effective(&format!("official.media.feature.{name}"));
    }
    for name in [
        "hover",
        "any-hover",
        "pointer",
        "any-pointer",
        "prefers-color-scheme",
        "prefers-reduced-motion",
        "prefers-reduced-transparency",
        "prefers-contrast",
        "forced-colors",
    ] {
        known(&format!("({name})"), name);
        effective(&format!("ext.media.{name}"));
    }
}

#[test]
fn every_additional_published_feature_has_atomic_support_metadata() {
    let mut missing = Vec::new();
    for name in [
        "horizontal-viewport-segments",
        "vertical-viewport-segments",
        "update",
        "overflow-block",
        "overflow-inline",
        "color-gamut",
        "video-color-gamut",
        "dynamic-range",
        "video-dynamic-range",
        "environment-blending",
        "inverted-colors",
        "nav-controls",
        "scripting",
        "prefers-reduced-data",
    ] {
        known(&format!("({name})"), name);
        let id = format!("ext.media.{name}");
        if feature_metadata(&id).is_none() {
            missing.push(id);
        } else {
            effective(&id);
        }
    }
    assert!(missing.is_empty(), "missing authored support: {missing:?}");
}

#[test]
fn complete_signed_and_chained_ranges_are_not_reported_as_partial_subsets() {
    for (name, source) in [
        ("width", "(-2px < width <= calc(1em + 1px))"),
        ("height", "(20px >= height > -1px)"),
        ("resolution", "(-1dppx < resolution <= infinite)"),
        ("color", "(-1 < color <= calc(8 / 2))"),
        ("monochrome", "(8 >= monochrome > -1)"),
    ] {
        known(source, name);
        let id = format!("ext.media.range.{name}");
        let metadata = feature_metadata(&id).unwrap();
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
        effective(&id);
    }
}

#[test]
fn display_mode_cites_its_published_descriptor_instead_of_repository_baseline() {
    known("(display-mode: picture-in-picture)", "display-mode");
    effective("ext.media.display-mode");
    assert_eq!(
        feature_metadata("ext.media.display-mode")
            .unwrap()
            .production(),
        "#descdef-media-display-mode"
    );
}

#[test]
fn query_grammar_rows_cite_the_selected_effective_edition() {
    for source in ["screen", "not future()", "(color) or (hover)"] {
        assert!(parse_media_query(source).is_clean());
    }
    for id in [
        "baseline.media.type",
        "official.media.query-list-core",
        "ext.media.condition-syntax",
        "ext.media.malformed-member-never",
        "ext.media.resolution.dppx",
    ] {
        effective(id);
    }
}

#[test]
fn normalization_preserves_import_alternatives_and_symbolic_media_origins() {
    let source = concat!(
        "@import 'x' layer(theme) and (color);",
        "@media (-2px < width <= calc(1em + 1px)) {}"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean());
    assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let [
        CssNormalizedItem::Rule(import),
        CssNormalizedItem::Rule(media),
    ] = normalized.items()
    else {
        panic!("import and empty media retained in order");
    };
    let CssRuleContextKindRef::Import(import) = import.kind() else {
        panic!("import payload");
    };
    assert!(import.layer().is_none());
    assert!(import.supports().is_none());
    assert_eq!(
        import.serialize().unwrap().as_css(),
        "@import 'x' layer(theme) and (color);"
    );
    let CssRuleContextKindRef::Media(media) = media.kind() else {
        panic!("media context");
    };
    assert_eq!(
        media.serialize().unwrap().as_css(),
        "(-2px < width <= calc(1em + 1px))"
    );
    let constructed = CssMediaQuery::try_from_components(
        parse_component_values("(-2px < width <= calc(1em + 1px))").unwrap(),
    )
    .unwrap();
    let CssMediaQuery::Condition(condition) = &constructed else {
        panic!("checked condition");
    };
    assert!(
        matches!(condition.kind(), CssMediaConditionKind::Feature(feature) if feature.name() == "width")
    );
    assert_eq!(
        constructed.serialize().unwrap().as_css(),
        "(-2px < width <= calc(1em + 1px))"
    );
    let [query] = media.queries() else {
        panic!("one query");
    };
    assert_eq!(
        query.position().unwrap().byte_offset().value(),
        source.find("(-2px").unwrap()
    );
}
