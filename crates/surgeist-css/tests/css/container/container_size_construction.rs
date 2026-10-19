#![forbid(unsafe_code)]
//! Typed authored contracts for Conditional 5 size features and scoped Values5 §9.
//! Existing callable-boundary recognition evidence is in container_size_grammar.
use surgeist_css::*;

fn condition(source: &str) -> CssContainerCondition {
    CssContainerCondition::try_from_components(parse_component_values(source).unwrap()).unwrap()
}
fn feature(condition: &CssContainerCondition) -> &CssContainerFeatureQuery {
    let CssContainerConditionKind::Feature(feature) = condition.kind() else {
        panic!("recognized feature: {condition:?}")
    };
    feature
}
fn length(value: &CssContainerLength) -> &CssLengthCalculation {
    let CssContainerLengthRef::Numeric(value) = value.view() else {
        panic!("typed length")
    };
    value
}
fn ratio(value: &CssContainerRatio) -> &CssMediaRatio {
    let CssContainerRatioRef::Numeric(value) = value.view() else {
        panic!("typed ratio")
    };
    value
}
fn number(value: &CssNumberCalculation) -> &str {
    let CssCalculationExpressionRef::Value(value) = value.expression() else {
        panic!("number literal")
    };
    value.literal().representation()
}

#[test]
fn boolean_identity_does_not_invent_a_comparison_or_scalar() {
    for (source, expected) in [
        ("(WIDTH)", CssContainerSizeFeatureKind::Width),
        ("(height)", CssContainerSizeFeatureKind::Height),
        ("(inline-size)", CssContainerSizeFeatureKind::InlineSize),
        ("(block-size)", CssContainerSizeFeatureKind::BlockSize),
        ("(aspect-ratio)", CssContainerSizeFeatureKind::AspectRatio),
        ("(orientation)", CssContainerSizeFeatureKind::Orientation),
    ] {
        assert!(
            matches!(feature(&condition(source)), CssContainerFeatureQuery::Boolean(actual) if *actual == expected)
        );
    }
}

#[test]
fn range_views_preserve_plain_prefixes_operand_order_and_inclusivity() {
    for (source, form) in [
        ("(width: 1px)", "plain"),
        ("(min-width: 1px)", "min"),
        ("(max-width: 1px)", "max"),
    ] {
        let condition = condition(source);
        let CssContainerFeatureQuery::Width(range) = feature(&condition) else {
            panic!("width")
        };
        let value = match (form, range.view()) {
            ("plain", CssMediaRangeRef::Plain { value })
            | ("min", CssMediaRangeRef::Min { value })
            | ("max", CssMediaRangeRef::Max { value }) => value,
            _ => panic!("authored range spelling {source}"),
        };
        assert_eq!(length(value).result_type(), CssCalculationType::Length);
    }
    let condition = condition("(2px >= inline-size > 10px)");
    let CssContainerFeatureQuery::InlineSize(range) = feature(&condition) else {
        panic!("inline-size")
    };
    let CssMediaRangeRef::Descending {
        left,
        left_inclusive,
        right,
        right_inclusive,
    } = range.view()
    else {
        panic!("descending")
    };
    assert!(left_inclusive);
    assert!(!right_inclusive);
    assert_eq!(left.components().serialize().unwrap().as_css(), "2px");
    assert_eq!(right.components().serialize().unwrap().as_css(), "10px");
    let condition = self::condition("(1px < height <= 2px)");
    let CssContainerFeatureQuery::Height(range) = feature(&condition) else {
        panic!("height")
    };
    let CssMediaRangeRef::Ascending {
        left_inclusive,
        right_inclusive,
        ..
    } = range.view()
    else {
        panic!("ascending")
    };
    assert!(!left_inclusive);
    assert!(right_inclusive);
    let condition = self::condition("(1px = block-size)");
    let CssContainerFeatureQuery::BlockSize(range) = feature(&condition) else {
        panic!("block-size")
    };
    assert!(matches!(
        range.view(),
        CssMediaRangeRef::ValueFirst {
            comparison: CssQueryComparison::Equal,
            ..
        }
    ));
}

#[test]
fn exact_numeric_views_retain_magnitudes_and_unitless_zero_without_f32() {
    for (source, representation) in [
        ("(width: 1e999px)", "1e999"),
        ("(width: -1e-999px)", "-1e-999"),
        (
            "(width: 12345678901234567890.123456789px)",
            "12345678901234567890.123456789",
        ),
        ("(width: -0)", "-0"),
    ] {
        let condition = condition(source);
        let CssContainerFeatureQuery::Width(range) = feature(&condition) else {
            panic!("width")
        };
        let CssMediaRangeRef::Plain { value } = range.view() else {
            panic!("plain")
        };
        let CssCalculationExpressionRef::Value(value) = length(value).expression() else {
            panic!("literal")
        };
        assert_eq!(value.literal().representation(), representation);
    }
    for (source, numerator, denominator, omitted) in [
        ("(aspect-ratio: 1e999 / 1e-999)", "1e999", "1e-999", false),
        ("(aspect-ratio: -0 / 0)", "-0", "0", false),
        ("(aspect-ratio: 1.5)", "1.5", "1", true),
    ] {
        let condition = condition(source);
        let CssContainerFeatureQuery::AspectRatio(range) = feature(&condition) else {
            panic!("ratio")
        };
        let CssMediaRangeRef::Plain { value } = range.view() else {
            panic!("plain")
        };
        let value = ratio(value);
        assert_eq!(number(value.numerator()), numerator);
        assert_eq!(number(value.denominator()), denominator);
        assert_eq!(value.denominator_is_omitted(), omitted);
        if omitted {
            assert_eq!(value.denominator().origin(), &CssValueOrigin::Programmatic);
        }
    }
    for source in [
        "(aspect-ratio: -1e-999 / 2)",
        "(aspect-ratio: 2 / -1e-999)",
        "(aspect-ratio: / 2)",
        "(aspect-ratio: 1 /)",
    ] {
        assert!(matches!(
            condition(source).kind(),
            CssContainerConditionKind::GeneralEnclosed(_)
        ));
    }
}

#[test]
fn pending_values_keep_whole_operand_and_required_domain_without_typing_fallbacks() {
    for (source, expected, domain) in [
        (
            "(width: calc(var(--n) * 1px))",
            "calc(var(--n) * 1px)",
            CssContainerValueDomain::Length,
        ),
        (
            "(aspect-ratio: var(--pair, 0 / 0))",
            "var(--pair, 0 / 0)",
            CssContainerValueDomain::Ratio,
        ),
        (
            "(aspect-ratio: var(--n) / 2)",
            "var(--n) / 2",
            CssContainerValueDomain::Ratio,
        ),
        (
            "(orientation: var(--Mode, red))",
            "var(--Mode, red)",
            CssContainerValueDomain::Orientation,
        ),
    ] {
        let condition = condition(source);
        let pending = match feature(&condition) {
            CssContainerFeatureQuery::Width(range) => {
                let CssMediaRangeRef::Plain { value } = range.view() else {
                    panic!("plain")
                };
                let CssContainerLengthRef::Pending(value) = value.view() else {
                    panic!("pending")
                };
                value
            }
            CssContainerFeatureQuery::AspectRatio(range) => {
                let CssMediaRangeRef::Plain { value } = range.view() else {
                    panic!("plain")
                };
                let CssContainerRatioRef::Pending(value) = value.view() else {
                    panic!("pending")
                };
                value
            }
            CssContainerFeatureQuery::Orientation(value) => {
                let CssContainerOrientationRef::Pending(value) = value.view() else {
                    panic!("pending")
                };
                value
            }
            _ => panic!("selected domain"),
        };
        assert_eq!(pending.domain(), domain);
        assert_eq!(pending.serialize().unwrap().as_css(), expected);
        assert_eq!(pending.components().serialize().unwrap().as_css(), expected);
        assert!(pending.serialize_with_limit(1).is_err());
        let CssValueOrigin::Parsed(origin) = pending.components().items()[0].origin() else {
            panic!("original operand")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(expected).unwrap()
        );
    }
}

#[test]
fn tree_counting_is_inspectable_and_remains_symbolic_in_container_context() {
    for (source, expected) in [
        (
            "(aspect-ratio: sibling-count())",
            CssTreeCountingFunction::SiblingCount,
        ),
        (
            "(aspect-ratio: sibling-index())",
            CssTreeCountingFunction::SiblingIndex,
        ),
    ] {
        let condition = condition(source);
        let CssContainerFeatureQuery::AspectRatio(range) = feature(&condition) else {
            panic!("ratio")
        };
        let CssMediaRangeRef::Plain { value } = range.view() else {
            panic!("plain")
        };
        let CssCalculationExpressionRef::TreeCounting(value) =
            ratio(value).numerator().expression()
        else {
            panic!("tree counting")
        };
        assert_eq!(value.function(), expected);
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("tree origin")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find("sibling-").unwrap()
        );
    }
    assert!(
        CssNumberCalculation::try_from_components(
            parse_component_values("sibling-count()").unwrap()
        )
        .is_err()
    );
    let condition = condition("(width: calc(sibling-index() * 1px))");
    let CssContainerFeatureQuery::Width(range) = feature(&condition) else {
        panic!("width")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain")
    };
    assert_eq!(length(value).result_type(), CssCalculationType::Length);
}

#[test]
fn mixed_origin_pending_components_preserve_token_origins_and_bounded_construction() {
    let var = parse_component_values("var(--limit,)").unwrap().items()[0].clone();
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("width").unwrap(),
        CssComponentValue::try_token(":").unwrap(),
        var,
    ])
    .unwrap();
    let enclosed = CssGeneralEnclosed::try_parenthesized(values).unwrap();
    let condition = CssContainerCondition::try_from_enclosed(enclosed).unwrap();
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    let CssContainerFeatureQuery::Width(range) = feature(&condition) else {
        panic!("width")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain")
    };
    let CssContainerLengthRef::Pending(value) = value.view() else {
        panic!("pending")
    };
    let output = value.serialize().unwrap();
    assert!(matches!(
        output.origin_at(0),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(_)))
    ));
    let components = CssComponentValues::try_new(condition.components().to_vec()).unwrap();
    let error = CssContainerCondition::try_from_components_with_limits(
        components,
        CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
    )
    .unwrap_err();
    assert!(matches!(error, CssContainerConstructionError::Component(_)));
}

#[test]
fn recovered_numeric_closures_remain_recognized_and_serialize_explicitly() {
    let components = parse_component_values("(width: calc(1px + 2px").unwrap();
    let condition = CssContainerCondition::try_from_components(components).unwrap();
    assert!(matches!(
        feature(&condition),
        CssContainerFeatureQuery::Width(_)
    ));
    assert_eq!(
        condition.serialize().unwrap().as_css(),
        "(width: calc(1px + 2px))"
    );
}
