//! Media Queries 4/5 lists, features, ranges and `@media` rules through the
//! public media-query and stylesheet parsers.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn media_query_list_constructor_accepts_empty_and_nonempty_lists() {
    assert!(CssMediaQueryList::new(Vec::new()).queries().is_empty());
    assert!(
        !CssMediaQueryList::new(vec![
            surgeist_css::parse_media_query("screen")
                .into_validation_result()
                .unwrap()
        ])
        .queries()
        .is_empty()
    );
}

#[test]
fn media_condition_list_constructor_requires_at_least_two_conditions() {
    let parsed = parse_media_query_list("(min-width: 600px)").expect("clean parse report");
    let [CssMediaQuery::Condition(width)] = parsed.queries() else {
        panic!("expected checked media width condition");
    };
    let width = width.clone();
    assert_eq!(CssMediaConditionList::try_new(Vec::new()), None);
    assert_eq!(CssMediaConditionList::try_new(vec![width.clone()]), None);
    assert!(CssMediaConditionList::try_new(vec![width.clone(), width]).is_some());
}

#[test]
fn query_numeric_constructors_preserve_finite_nonnegative_values_and_authored_units() {
    assert_eq!(
        CssQueryLength::try_new(12.0, CssLengthUnit::Rem)
            .unwrap()
            .value()
            .value(),
        12.0
    );
    assert_eq!(
        CssQueryLength::try_new(12.0, CssLengthUnit::Rem)
            .unwrap()
            .unit(),
        Some(CssLengthUnit::Rem)
    );
    assert_eq!(CssQueryLength::try_new(-0.1, CssLengthUnit::Px), None);
    assert_eq!(CssQueryLength::try_new(f32::NAN, CssLengthUnit::Px), None);
    assert_eq!(CssQueryLength::unitless_zero().value().value(), 0.0);
    assert_eq!(CssQueryLength::unitless_zero().unit(), None);

    assert_eq!(
        CssRatio::try_new(16.0, 9.0).unwrap().numerator().value(),
        16.0
    );
    assert_eq!(
        CssRatio::try_new(16.0, 9.0).unwrap().denominator().value(),
        9.0
    );
    assert_eq!(CssRatio::try_new(-1.0, 1.0), None);
    assert_eq!(CssRatio::try_new(1.0, 0.0), None);
    assert_eq!(CssRatio::try_new(f32::INFINITY, 1.0), None);

    assert_eq!(
        CssNumericLiteral::<CssResolution>::try_new("2", CssResolutionUnit::Dppx)
            .unwrap()
            .unit(),
        CssResolutionUnit::Dppx
    );
    assert!(CssNumericLiteral::<CssResolution>::try_new("0", CssResolutionUnit::Dpi).is_ok());
    assert!(CssNumericLiteral::<CssResolution>::try_new("NaN", CssResolutionUnit::Dpi).is_err());
}

#[test]
fn media_feature_names_are_canonical() {
    let parsed = parse_media_query_list("(width: 1px)").expect("clean parse report");
    let [CssMediaQuery::Condition(condition)] = parsed.queries() else {
        panic!("expected checked width condition");
    };
    let CssMediaConditionKind::Feature(feature) = condition.kind() else {
        panic!("expected numeric feature");
    };
    assert_eq!(feature.name(), "width");
    assert_eq!(
        CssMediaFeatureQuery::PrefersColorScheme(CssColorSchemePreference::Dark).name(),
        "prefers-color-scheme"
    );
    assert_eq!(
        CssMediaFeatureQuery::AnyPointer(CssPointerCapability::Fine).name(),
        "any-pointer"
    );
}

#[test]
fn media_query_parser_accepts_supported_types_ranges_and_conditions() {
    for css in [
        "screen",
        "print",
        "speech",
        "tv",
        "screen and (min-width: 600px)",
        "(device-width: 800px)",
        "(aspect-ratio: 16/9)",
        "(color-index)",
        "(scan: progressive)",
        "(grid: 1)",
        "(width >= 600px)",
        "(orientation: landscape)",
        "(prefers-color-scheme: dark)",
        "(hover: hover) and (pointer: fine)",
        "not screen and (max-width: 400px)",
        "screen, print",
    ] {
        parse_media_query_list(css).unwrap_or_else(|error| {
            panic!("{css} should parse as a media query list: {error}");
        });
    }
}

#[test]
fn media_query_parser_preserves_unknown_and_rejects_malformed_conditions() {
    for css in [
        "future-screen",
        "(unknown-feature: yes)",
        "(width: auto)",
        "(width: min-content)",
    ] {
        let query = parse_media_query_list(css)
            .unwrap_or_else(|error| panic!("{css} should be valid defined-false syntax: {error}"));
        assert!(!matches!(query.queries(), [CssMediaQuery::Never(_)]));
    }

    let enclosed = parse_media_query_list("(width >= )").expect("clean parse report");
    assert!(
        matches!(enclosed.queries(), [CssMediaQuery::Condition(condition)] if matches!(condition.kind(), CssMediaConditionKind::GeneralEnclosed(value) if value.authored() == Some("(width >= )")))
    );
    // Media Queries 4 §3: `<media-type> and <media-condition-without-or>`, so
    // `and` needs a following condition and a media type takes no `or`. No
    // token is responsible for a truncated query; its diagnostic names the
    // query's first token (an implementation-defined choice).
    for (css, encountered) in [("screen and", Some("and")), ("screen or print", Some("or"))] {
        let error = parse_media_query_list(css).expect_err("invalid media query list");
        assert_rejection_matches(
            css,
            &error,
            Rejection::new(CssErrorCode::InvalidMediaQuery, None, encountered),
        );
    }
}

#[test]
fn media_query_parser_preserves_typed_query_structure() {
    let query_list =
        parse_media_query_list("not screen and (max-width: 400px)").expect("clean parse report");
    let [CssMediaQuery::Typed(query)] = query_list.queries() else {
        panic!("expected one typed media query");
    };

    assert_eq!(query.modifier(), Some(CssMediaQueryModifier::Not));
    assert_eq!(query.media_type(), CssMediaType::Screen);
    let Some(condition) = query.condition() else {
        panic!("expected width condition");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(width)) = condition.kind()
    else {
        panic!("expected width condition");
    };
    let CssMediaRangeRef::Max { value } = width.view() else {
        panic!("expected authored media range form");
    };
    assert_media_length_literal(value, "400", "px");
}

#[test]
fn media_query_parser_preserves_condition_only_range_structure() {
    let query_list = parse_media_query_list("(width >= 600px)").expect("clean parse report");
    let [CssMediaQuery::Condition(condition)] = query_list.queries() else {
        panic!("expected one condition-only width query");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(width)) = condition.kind()
    else {
        panic!("expected one condition-only width query");
    };

    let CssMediaRangeRef::FeatureFirst {
        comparison: CssQueryComparison::GreaterThanOrEqual,
        value,
    } = width.view()
    else {
        panic!("expected authored media range form");
    };
    assert_media_length_literal(value, "600", "px");
}

#[test]
fn media_query_parser_preserves_discrete_and_condition_list_structure() {
    let query_list =
        parse_media_query_list("(hover: hover) and (pointer: fine)").expect("clean parse report");
    let [CssMediaQuery::Condition(condition)] = query_list.queries() else {
        panic!("expected one condition-only and query");
    };
    let CssMediaConditionKind::And(list) = condition.kind() else {
        panic!("expected one condition-only and query");
    };
    let [hover, pointer] = list.conditions() else {
        panic!("expected two conditions");
    };

    assert!(matches!(
        hover.kind(),
        CssMediaConditionKind::Feature(CssMediaFeatureQuery::Hover(CssHoverCapability::Hover))
    ));
    assert!(matches!(
        pointer.kind(),
        CssMediaConditionKind::Feature(CssMediaFeatureQuery::Pointer(CssPointerCapability::Fine))
    ));
}

#[test]
fn media_query_parser_preserves_comma_separated_queries() {
    let query_list = parse_media_query_list("screen, print").expect("clean parse report");
    let [screen, print] = query_list.queries() else {
        panic!("expected two media queries");
    };

    let CssMediaQuery::Typed(screen) = screen else {
        panic!("expected screen query")
    };
    let CssMediaQuery::Typed(print) = print else {
        panic!("expected print query")
    };
    assert_eq!(screen.media_type(), CssMediaType::Screen);
    assert_eq!(print.media_type(), CssMediaType::Print);
}

#[test]
fn media_rule_parser_accepts_style_rule_body() {
    let sheet = parse_sheet("@media screen and (min-width: 600px) { .panel { color: black; } }")
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one media rule");
    };
    let rule = media_rule(rule);

    let [CssMediaQuery::Typed(query)] = rule.query().queries() else {
        panic!("expected typed media query");
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert!(query.condition().is_some());
    assert_eq!(rule.position().unwrap().byte_offset().value(), 0);
    assert_eq!(rule.position().unwrap().line().value(), 0);
    assert_eq!(rule.position().unwrap().column().value(), 0);

    let [nested] = rule.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(nested).selectors().selectors()[0].selector(),
        &CssSelector::Class("panel".to_owned())
    );
}

#[test]
fn media_rule_parser_accepts_nested_media_rule() {
    let sheet =
        parse_sheet("@media screen { @media (min-width: 600px) { .panel { color: black; } } }")
            .expect("clean parse report");
    let [outer] = sheet.rules() else {
        panic!("expected one outer media rule");
    };
    let outer = media_rule(outer);
    let [inner] = outer.rules() else {
        panic!("expected one inner media rule");
    };
    let inner = media_rule(inner);
    let [nested] = inner.rules() else {
        panic!("expected one nested style rule");
    };

    assert_eq!(
        style_rule(nested).selectors().selectors()[0].selector(),
        &CssSelector::Class("panel".to_owned())
    );
}

#[test]
fn media_rule_parser_retains_unknown_features_and_rejects_invalid_bodies() {
    let sheet = parse_sheet("@media (unknown: yes) { .panel { color: black; } }")
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected retained media rule")
    };
    assert!(matches!(
        media_rule(rule).query().queries(),
        [CssMediaQuery::Condition(condition)]
            if matches!(condition.kind(), CssMediaConditionKind::UnknownFeature(_))
    ));
    assert_rejection(
        "@media screen { .panel { made-up: value; } }",
        Rejection::unknown_property("made-up"),
    );
}
