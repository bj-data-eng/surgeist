//! Authored range orientation, exact operands and provenance are public contracts.
use surgeist_css::{
    CssCalculationExpressionRef, CssMediaConditionKind, CssMediaFeatureQuery, CssMediaGridRef,
    CssMediaQuery, CssMediaRangeRef, CssMediaResolutionRef, CssQueryComparison, CssValueOrigin,
    parse_media_query, parse_media_query_list,
};

fn feature(source: &str) -> CssMediaFeatureQuery {
    let report = parse_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("expected condition");
    };
    let CssMediaConditionKind::Feature(feature) = condition.kind() else {
        panic!("expected known feature");
    };
    feature.clone()
}

fn literal(expression: CssCalculationExpressionRef<'_>, expected: &str) {
    let CssCalculationExpressionRef::Value(value) = expression else {
        panic!("expected exact literal");
    };
    assert_eq!(value.literal().representation(), expected);
}

#[test]
fn single_bound_ranges_keep_their_authored_form() {
    for (source, form) in [
        ("(width: -1px)", 0),
        ("(min-width: -1px)", 1),
        ("(max-width: -1px)", 2),
        ("(width <= -1px)", 3),
        ("(-1px >= width)", 4),
    ] {
        let CssMediaFeatureQuery::Width(range) = feature(source) else {
            panic!("expected width");
        };
        let value = match (form, range.view()) {
            (0, CssMediaRangeRef::Plain { value })
            | (1, CssMediaRangeRef::Min { value })
            | (2, CssMediaRangeRef::Max { value }) => value,
            (3, CssMediaRangeRef::FeatureFirst { comparison, value }) => {
                assert_eq!(comparison, CssQueryComparison::LessThanOrEqual);
                value
            }
            (4, CssMediaRangeRef::ValueFirst { value, comparison }) => {
                assert_eq!(comparison, CssQueryComparison::GreaterThanOrEqual);
                value
            }
            _ => panic!("wrong authored range form for {source}"),
        };
        literal(value.calculation().expression(), "-1");
    }
}

#[test]
fn chained_ranges_preserve_direction_inclusivity_and_empty_intervals() {
    for (source, ascending, first, second) in [
        ("(-100px < width <= 40em)", true, "-100", "40"),
        ("(80em >= width > -100px)", false, "80", "-100"),
        ("(100px < width <= 10px)", true, "100", "10"),
    ] {
        let CssMediaFeatureQuery::Width(range) = feature(source) else {
            panic!("expected width");
        };
        let (left, right) = match range.view() {
            CssMediaRangeRef::Ascending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => {
                assert!(ascending);
                assert!(!left_inclusive);
                assert!(right_inclusive);
                (left, right)
            }
            CssMediaRangeRef::Descending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => {
                assert!(!ascending);
                assert!(left_inclusive);
                assert!(!right_inclusive);
                (left, right)
            }
            _ => panic!("expected chained range"),
        };
        literal(left.calculation().expression(), first);
        literal(right.calculation().expression(), second);
    }
}

#[test]
fn omitted_ratio_denominator_is_one_with_programmatic_origin() {
    let CssMediaFeatureQuery::AspectRatio(range) = feature("(aspect-ratio: 2)") else {
        panic!("expected ratio");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("expected plain ratio");
    };
    assert!(value.denominator_is_omitted());
    literal(value.numerator().expression(), "2");
    literal(value.denominator().expression(), "1");
    assert!(matches!(
        value.denominator().origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(value.numerator().position().is_some());
}

#[test]
fn explicit_zero_ratio_denominator_retains_its_authored_origin() {
    let source = "(aspect-ratio: 1.5/0)";
    let CssMediaFeatureQuery::AspectRatio(range) = feature(source) else {
        panic!("expected ratio");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("expected plain ratio");
    };
    assert!(!value.denominator_is_omitted());
    literal(value.numerator().expression(), "1.5");
    literal(value.denominator().expression(), "0");
    assert_eq!(
        value
            .denominator()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.find('0').unwrap()
    );
}

#[test]
fn grid_calculations_keep_number_rounding_and_deferred_bounds() {
    for source in ["(grid: calc(0.5))", "(grid: calc(2))", "(grid: calc(-1))"] {
        let CssMediaFeatureQuery::Grid(grid) = feature(source) else {
            panic!("expected grid");
        };
        let CssMediaGridRef::Calculation(calculation) = grid.view() else {
            panic!("expected symbolic grid calculation");
        };
        assert!(calculation.requires_rounding());
        assert!(matches!(
            calculation.expression(),
            CssCalculationExpressionRef::NestedCalc(_)
        ));
    }
}

#[test]
fn resolution_keeps_infinite_distinct_from_exact_signed_units() {
    let CssMediaFeatureQuery::Resolution(range) = feature("(resolution: infinite)") else {
        panic!("expected resolution");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("expected plain resolution");
    };
    assert!(matches!(value.view(), CssMediaResolutionRef::Infinite(_)));

    let CssMediaFeatureQuery::Resolution(range) = feature("(resolution >= -2x)") else {
        panic!("expected resolution");
    };
    let CssMediaRangeRef::FeatureFirst { comparison, value } = range.view() else {
        panic!("expected feature-first resolution");
    };
    assert_eq!(comparison, CssQueryComparison::GreaterThanOrEqual);
    let CssMediaResolutionRef::Numeric(calculation) = value.view() else {
        panic!("expected numeric resolution");
    };
    literal(calculation.expression(), "-2");
    let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
        panic!("expected resolution literal");
    };
    assert_eq!(value.literal().unit(), Some("x"));
}

#[test]
fn infinite_resolution_can_precede_the_feature_name() {
    for source in [
        "(infinite > resolution)",
        "(infinite >= resolution > 0dppx)",
    ] {
        let CssMediaFeatureQuery::Resolution(range) = feature(source) else {
            panic!("expected resolution");
        };
        let left = match range.view() {
            CssMediaRangeRef::ValueFirst { value, comparison } => {
                assert_eq!(comparison, CssQueryComparison::GreaterThan);
                value
            }
            CssMediaRangeRef::Descending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => {
                assert!(left_inclusive);
                assert!(!right_inclusive);
                let CssMediaResolutionRef::Numeric(calculation) = right.view() else {
                    panic!("expected zero resolution bound");
                };
                literal(calculation.expression(), "0");
                left
            }
            _ => panic!("expected value-first resolution range"),
        };
        assert!(matches!(left.view(), CssMediaResolutionRef::Infinite(_)));
    }
}

#[test]
fn numeric_origins_use_the_complete_unicode_query_list_source() {
    let source = "/* 😀 */ print,\n(width: -1e999px)";
    let report = parse_media_query_list(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssMediaQuery::Condition(condition) = &report.syntax().queries()[1] else {
        panic!("expected second query condition");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(range)) = condition.kind()
    else {
        panic!("expected width");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("expected plain width");
    };
    literal(value.calculation().expression(), "-1e999");
    let position = value.calculation().position().unwrap();
    assert_eq!(
        position.byte_offset().value(),
        source.find("-1e999").unwrap()
    );
    assert_eq!(position.line().value(), 1);
    assert_eq!(position.column().value(), 8);
}

#[test]
fn discrete_feature_domains_accept_each_selected_keyword() {
    // MQ5 feature definitions supply these domains independently of parser lookup.
    for (name, keywords) in [
        ("update", &["none", "slow", "fast"][..]),
        ("overflow-block", &["none", "scroll", "paged"][..]),
        ("overflow-inline", &["none", "scroll"][..]),
        ("color-gamut", &["srgb", "p3", "rec2020"][..]),
        ("video-color-gamut", &["srgb", "p3", "rec2020"][..]),
        ("dynamic-range", &["standard", "high"][..]),
        ("video-dynamic-range", &["standard", "high"][..]),
        (
            "environment-blending",
            &["opaque", "additive", "subtractive"][..],
        ),
        ("inverted-colors", &["none", "inverted"][..]),
        ("nav-controls", &["none", "back"][..]),
        ("scripting", &["none", "initial-only", "enabled"][..]),
        ("prefers-reduced-data", &["no-preference", "reduce"][..]),
    ] {
        for keyword in keywords {
            let result = feature(&format!("({name}: {keyword})"));
            assert_eq!(result.name(), name);
        }
        // No discrete feature acquires a range domain through prefix spelling.
        let report = parse_media_query(&format!("(min-{name}: {})", keywords[0]));
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("expected retained unknown condition");
        };
        assert!(!matches!(
            condition.kind(),
            CssMediaConditionKind::Feature(_)
        ));
    }
}
