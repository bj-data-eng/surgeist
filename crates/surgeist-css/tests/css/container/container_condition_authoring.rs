//! Container queries (Conditional Rules 5) and `@container` rules through the
//! public stylesheet parser and constructors.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

/// The condition of a lone `@container <source> {}` prelude, parsed by the
/// public stylesheet parser (Conditional Rules 5 §3.1), or its first
/// diagnostic.
fn container_condition(source: &str) -> Result<CssContainerCondition, Error> {
    let report = parse_sheet(&format!("@container {source} {{}}"));
    if let Some(diagnostic) = report.diagnostics().first() {
        return Err(diagnostic.error().clone());
    }
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("{source}: one container rule");
    };
    let [entry] = rule.prelude().entries() else {
        panic!("{source}: one container prelude entry");
    };
    Ok(entry.query().expect("container query").clone())
}

fn assert_container_theme_value(condition: &CssContainerCondition) {
    let CssContainerConditionKind::Style(query) = condition.kind() else {
        panic!("style")
    };
    let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Plain {
        name: CssContainerStyleFeatureName::Custom(name),
        value,
    }) = query.kind()
    else {
        panic!("plain custom feature")
    };
    assert_eq!(name.as_str(), "--theme");
    assert_eq!(value.serialize().unwrap().as_css(), " dark");
}

#[test]
fn container_name_constructor_rejects_invalid_and_reserved_names() {
    assert_eq!(
        CssContainerName::try_new("sidebar").unwrap().as_str(),
        "sidebar"
    );
    assert_eq!(
        CssContainerName::try_new("layout-pane").unwrap().as_str(),
        "layout-pane"
    );

    for name in ["", " \t\n ", "two names", "1pane", "pane;", "none"] {
        assert_eq!(
            CssContainerName::try_new(name),
            None,
            "{name} should reject"
        );
    }
    for reserved in [
        "and",
        "or",
        "not",
        "default",
        "NoNe",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "InItIaL",
    ] {
        assert_eq!(
            CssContainerName::try_new(reserved),
            None,
            "{reserved} should reject"
        );
    }
}

#[test]
fn container_condition_list_constructor_requires_at_least_two_conditions() {
    let width = container_condition("(width > 600px)").unwrap();
    assert_eq!(CssContainerConditionList::try_new(Vec::new()), None);
    assert_eq!(
        CssContainerConditionList::try_new(vec![width.clone()]),
        None
    );
    assert!(CssContainerConditionList::try_new(vec![width.clone(), width]).is_some());
}

#[test]
fn container_condition_parser_accepts_supported_size_style_and_boolean_conditions() {
    for css in [
        "(width > 600px)",
        "(inline-size >= 30rem)",
        "(aspect-ratio > 1 / 1)",
        "(orientation: landscape)",
        "not (width < 300px)",
        "(width > 600px) and (orientation: landscape)",
        "(width > 600px) or (orientation: portrait)",
        "style(--theme)",
        "style(--theme: dark)",
    ] {
        container_condition(css).unwrap_or_else(|error| {
            panic!("{css} should parse as a container condition: {error}");
        });
    }
}

#[test]
fn container_condition_parser_retains_unrecognized_enclosures() {
    for css in [
        "(unknown > 1px)",
        "(width: auto)",
        "(width: min-content)",
        "(aspect-ratio: -1 / 1)",
        "style(future-property: red)",
        "scroll-state(future: top)",
        "(width > )",
    ] {
        let condition = container_condition(css).expect("valid opaque enclosure");
        let CssContainerConditionKind::GeneralEnclosed(value) = condition.kind() else {
            panic!("{css} should retain an opaque enclosure");
        };
        assert_eq!(value.serialize().unwrap().as_css(), css);
    }
}

#[test]
fn container_condition_parser_recognizes_scroll_state_keyword_features() {
    let condition = container_condition("scroll-state(stuck: top)").unwrap();
    let CssContainerConditionKind::ScrollState(query) = condition.kind() else {
        panic!("scroll-state")
    };
    let CssContainerScrollQueryKind::Feature(CssContainerScrollFeature::Stuck(value)) =
        query.kind()
    else {
        panic!("stuck")
    };
    assert!(matches!(
        value.view(),
        CssContainerStuckValueRef::Keyword(CssContainerStuckKeyword::Top)
    ));
}

#[test]
fn container_condition_parser_preserves_size_feature_structure() {
    let condition = container_condition("(inline-size >= 30rem)").unwrap();
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::InlineSize(inline_size)) =
        condition.kind()
    else {
        panic!("expected inline-size feature");
    };

    let CssMediaRangeRef::FeatureFirst { comparison, value } = inline_size.view() else {
        panic!("feature-first size range")
    };
    assert_eq!(comparison, CssQueryComparison::GreaterThanOrEqual);
    let CssContainerLengthRef::Numeric(value) = value.view() else {
        panic!("typed length")
    };
    assert_eq!(value.components().serialize().unwrap().as_css(), "30rem");
}

#[test]
fn container_condition_parser_preserves_ratio_and_logic_structure() {
    let condition =
        container_condition("(aspect-ratio > 1 / 1) and (orientation: landscape)").unwrap();
    let CssContainerConditionKind::And(list) = condition.kind() else {
        panic!("expected and condition list");
    };
    let [ratio, orientation] = list.conditions() else {
        panic!("expected two conditions");
    };

    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::AspectRatio(ratio)) =
        ratio.kind()
    else {
        panic!("expected aspect-ratio feature");
    };
    let CssMediaRangeRef::FeatureFirst { comparison, value } = ratio.view() else {
        panic!("ratio range")
    };
    assert_eq!(comparison, CssQueryComparison::GreaterThan);
    let CssContainerRatioRef::Numeric(value) = value.view() else {
        panic!("typed ratio")
    };
    assert_eq!(
        value
            .numerator()
            .components()
            .serialize()
            .unwrap()
            .as_css()
            .trim(),
        "1"
    );
    assert_eq!(
        value
            .denominator()
            .components()
            .serialize()
            .unwrap()
            .as_css()
            .trim(),
        "1"
    );
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::Orientation(value)) =
        orientation.kind()
    else {
        panic!("orientation")
    };
    assert!(matches!(
        value.view(),
        CssContainerOrientationRef::Keyword(CssOrientation::Landscape)
    ));
}

#[test]
fn container_style_query_preserves_custom_property_presence() {
    let condition = container_condition("style(--theme)").unwrap();
    let CssContainerConditionKind::Style(query) = condition.kind() else {
        panic!("style")
    };
    assert!(
        matches!(query.kind(), CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Boolean(CssContainerStyleFeatureName::Custom(name))) if name.as_str() == "--theme")
    );
}

#[test]
fn container_style_query_preserves_custom_property_authored_value() {
    let condition = container_condition("style(--theme: dark)").unwrap();
    assert_container_theme_value(&condition);
}

#[test]
fn container_rule_parser_accepts_unnamed_named_and_style_conditions() {
    let sheet = parse_sheet("@container (inline-size > 30rem) { .card { color: black; } }")
        .expect("unnamed container rule parses");
    let [rule] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let rule = container_rule(rule);
    assert_eq!(rule.prelude().entries()[0].name(), None);
    assert_eq!(rule.position().unwrap().byte_offset().value(), 0);
    assert_eq!(rule.position().unwrap().line().value(), 0);
    assert_eq!(rule.position().unwrap().column().value(), 0);
    assert!(matches!(
        rule.prelude().entries()[0].query().unwrap().kind(),
        CssContainerConditionKind::Feature(CssContainerFeatureQuery::InlineSize(_))
    ));
    let [nested] = rule.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(nested).selectors().selectors()[0].selector(),
        &CssSelector::Class("card".to_owned())
    );

    let sheet = parse_sheet("@container sidebar (width >= 300px) { .title { color: black; } }")
        .expect("named container rule parses");
    let [rule] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let rule = container_rule(rule);
    assert_eq!(
        rule.prelude().entries()[0].name(),
        Some(&CssContainerName::try_new("sidebar").unwrap())
    );
    assert!(matches!(
        rule.prelude().entries()[0].query().unwrap().kind(),
        CssContainerConditionKind::Feature(CssContainerFeatureQuery::Width(_))
    ));

    let sheet = parse_sheet("@container style(--theme: dark) { .title { color: black; } }")
        .expect("style query container rule parses");
    let [rule] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let rule = container_rule(rule);
    assert_eq!(rule.prelude().entries()[0].name(), None);
    assert_container_theme_value(rule.prelude().entries()[0].query().unwrap());
}

#[test]
fn nested_conditional_rules_allow_media_and_container_in_either_direction() {
    let sheet =
        parse_sheet("@media screen { @container (width > 300px) { .panel { color: black; } } }")
            .expect("media rule can contain a container rule");
    let [media] = sheet.rules() else {
        panic!("expected one media rule");
    };
    let media = media_rule(media);
    let [container] = media.rules() else {
        panic!("expected one nested container rule");
    };
    let container = container_rule(container);
    let [style] = container.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(style).selectors().selectors()[0].selector(),
        &CssSelector::Class("panel".to_owned())
    );

    let sheet =
        parse_sheet("@container (width > 300px) { @media screen { .panel { color: black; } } }")
            .expect("container rule can contain a media rule");
    let [container] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let container = container_rule(container);
    let [media] = container.rules() else {
        panic!("expected one nested media rule");
    };
    let media = media_rule(media);
    let [style] = media.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(style).selectors().selectors()[0].selector(),
        &CssSelector::Class("panel".to_owned())
    );
}

#[test]
fn container_rule_parser_retains_unknown_features_and_rejects_invalid_bodies() {
    let sheet = parse_sheet("@container (unknown > 1px) { .card { color: black; } }")
        .expect("clean parse report");
    assert!(matches!(
        container_rule(&sheet.rules()[0]).prelude().entries()[0]
            .query()
            .unwrap()
            .kind(),
        CssContainerConditionKind::GeneralEnclosed(_)
    ));
    assert_rejection(
        "@container (width > 300px) { @import \"x.css\"; }",
        Rejection::placement("import"),
    );
    assert_rejection(
        "@container (width > 300px) { .card { made-up: 1; } }",
        Rejection::unknown_property("made-up"),
    );
}
