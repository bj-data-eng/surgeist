//! Checked media construction and canonical authored output share parsed semantics.
use surgeist_css::{
    CssBlockKind, CssComponentValue, CssComponentValueErrorKind, CssComponentValueLimits,
    CssComponentValues, CssMediaCondition, CssMediaConditionKind, CssMediaConstructionError,
    CssMediaFeatureQuery, CssMediaQuery, CssMediaRangeRef, CssMediaSerializationError,
    CssSerializedOrigin, CssValueOrigin, parse_component_values, parse_media_query,
    parse_media_query_list,
};

fn values(items: Vec<CssComponentValue>) -> CssComponentValues {
    CssComponentValues::try_new(items).unwrap()
}

fn plain_width(value: CssComponentValue) -> CssComponentValues {
    values(vec![
        CssComponentValue::try_block(
            CssBlockKind::Parenthesis,
            values(vec![
                CssComponentValue::try_ident("width").unwrap(),
                CssComponentValue::try_token(":").unwrap(),
                value,
            ]),
        )
        .unwrap(),
    ])
}

#[test]
fn programmatic_numeric_queries_have_checked_values_without_source_coordinates() {
    let components = plain_width(CssComponentValue::try_dimension("-1e999", "px").unwrap());
    let query = CssMediaQuery::try_from_components(components).unwrap();
    assert_eq!(query.position(), None);
    assert!(matches!(query.origin(), CssValueOrigin::Programmatic));
    let CssMediaQuery::Condition(condition) = &query else {
        panic!("condition-only query");
    };
    assert_eq!(condition.position(), None);
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(range)) = condition.kind()
    else {
        panic!("known width");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain value");
    };
    let surgeist_css::CssCalculationExpressionRef::Value(literal) =
        value.calculation().expression()
    else {
        panic!("exact length");
    };
    assert_eq!(literal.literal().representation(), "-1e999");
    assert_eq!(literal.literal().unit(), Some("px"));
    assert!(matches!(
        literal.literal().origin(),
        CssValueOrigin::Programmatic
    ));
    assert_eq!(query.serialize().unwrap().as_css(), "(width: -1e999px)");
}

#[test]
fn cloned_parsed_numeric_children_keep_their_original_source_in_programmatic_queries() {
    let parsed = parse_component_values("/* 😀 */\n-2em").unwrap();
    let operand = parsed.items().last().unwrap().clone();
    let expected = operand.origin().clone();
    let components = plain_width(operand);
    let first = CssMediaCondition::try_from_components(components.clone()).unwrap();
    let second = CssMediaCondition::try_from_components(components).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.position(), None);
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(range)) = first.kind() else {
        panic!("width");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain width");
    };
    assert_eq!(value.calculation().origin(), &expected);
    let serialized = first.serialize().unwrap();
    assert_eq!(serialized.as_css(), "(width: -2em)");
    assert!(
        matches!(serialized.origin_at(8), Some(CssSerializedOrigin::Token(origin)) if origin == &expected)
    );
}

#[test]
fn opaque_programmatic_functions_preserve_original_components_and_token_boundaries() {
    let args = values(vec![
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_ident("e2").unwrap(),
    ]);
    let original = CssComponentValue::try_function("Future", args).unwrap();
    let condition = CssMediaCondition::try_from_components(values(vec![original.clone()])).unwrap();
    let CssMediaConditionKind::GeneralEnclosed(enclosed) = condition.kind() else {
        panic!("opaque function");
    };
    assert_eq!(enclosed.component(), &original);
    assert_eq!(enclosed.position(), None);
    assert_eq!(condition.serialize().unwrap().as_css(), "Future(1/**/e2)");
}

#[test]
fn checked_construction_rejects_recovery_and_query_condition_context_mismatches() {
    let recovered = parse_component_values("future(").unwrap();
    assert!(matches!(
        CssMediaQuery::try_from_components(recovered),
        Err(CssMediaConstructionError::RecoveredInput { .. })
    ));
    assert!(matches!(
        CssMediaCondition::try_from_components(parse_component_values("screen").unwrap()),
        Err(CssMediaConstructionError::InvalidConditionGrammar { .. })
    ));
    assert!(matches!(
        CssMediaQuery::try_from_components(parse_component_values("screen, print").unwrap()),
        Err(CssMediaConstructionError::InvalidQueryGrammar { .. })
    ));
}

#[test]
fn grammar_errors_map_inserted_token_boundaries_back_to_original_input() {
    let original = parse_component_values("/* 😀 */\nprint").unwrap();
    let extra = original.items().last().unwrap().clone();
    let origin = extra.origin().clone();
    let error = CssMediaQuery::try_from_components(values(vec![
        CssComponentValue::try_ident("screen").unwrap(),
        extra,
    ]))
    .unwrap_err();
    assert!(matches!(
        error,
        CssMediaConstructionError::InvalidQueryGrammar { .. }
    ));
    assert_eq!(error.origin(), &origin);
}

#[test]
fn requested_component_limits_preserve_their_actual_failure_categories() {
    let components = values(vec![
        CssComponentValue::try_function("future", values(vec![])).unwrap(),
    ]);
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 7).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error =
            CssMediaQuery::try_from_components_with_limits(components.clone(), limits).unwrap_err();
        let CssMediaConstructionError::Component(error) = error else {
            panic!("typed component resource failure");
        };
        assert_eq!(error.kind(), expected);
        assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
    }
}

#[test]
fn construction_byte_budget_includes_canonical_ratio_expansion() {
    let source = "(aspect-ratio:2)";
    let canonical = "(aspect-ratio: 2 / 1)";
    let components = parse_component_values(source).unwrap();
    let tight = CssComponentValueLimits::try_new(256, usize::MAX, source.len()).unwrap();
    let CssMediaConstructionError::Component(error) =
        CssMediaQuery::try_from_components_with_limits(components.clone(), tight).unwrap_err()
    else {
        panic!("canonical output exceeds selected byte limit");
    };
    assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
    let exact = CssComponentValueLimits::try_new(256, usize::MAX, canonical.len()).unwrap();
    let query = CssMediaQuery::try_from_components_with_limits(components, exact).unwrap();
    assert_eq!(query.serialize().unwrap().as_css(), canonical);
}

#[test]
fn canonical_queries_preserve_symbolic_values_and_authored_order() {
    for (source, expected) in [
        ("(WIDTH>=+001.5PX)", "(width >= +001.5PX)"),
        ("(\\77 idth: 2px)", "(width: 2px)"),
        ("(ASPECT-RATIO:2)", "(aspect-ratio: 2 / 1)"),
        ("(80em>=width>-100px)", "(80em >= width > -100px)"),
        ("(width </**/= 10px)", "(width <= 10px)"),
        ("NOT SCREEN AND (COLOR)", "not screen and (color)"),
        ("future(1/**/e2)", "future(1/**/e2)"),
        ("(FuTuRe: ACTIVE)", "(FuTuRe: ACTIVE)"),
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let output = report.syntax().serialize().unwrap();
        assert_eq!(output.as_css(), expected);
        let reparsed = parse_media_query(output.as_css());
        assert!(reparsed.is_clean());
        assert_eq!(reparsed.syntax().serialize().unwrap().as_css(), expected);
    }
}

#[test]
fn a_recovered_member_prevents_partial_list_serialization() {
    let report = parse_media_query_list("screen and, print");
    assert!(!report.is_clean());
    let CssMediaSerializationError::RecoveredNever { origin } =
        report.syntax().serialize().unwrap_err()
    else {
        panic!("recovered member failure");
    };
    assert!(matches!(origin, CssValueOrigin::Parsed(_)));
    assert_eq!(
        parse_media_query("not all")
            .syntax()
            .serialize()
            .unwrap()
            .as_css(),
        "not all"
    );
    assert_eq!(
        parse_media_query_list("")
            .syntax()
            .serialize()
            .unwrap()
            .as_css(),
        ""
    );
}

#[test]
fn unknown_features_keep_their_domain_reason_and_original_component() {
    use surgeist_css::CssUnknownMediaFeatureReason;
    for (source, reason) in [
        ("(future: foo)", CssUnknownMediaFeatureReason::UnknownName),
        ("(min-width)", CssUnknownMediaFeatureReason::UnknownName),
        ("(grid: 2)", CssUnknownMediaFeatureReason::InvalidValue),
        (
            "(hover > none)",
            CssUnknownMediaFeatureReason::InvalidOperation,
        ),
    ] {
        let original = parse_component_values(source).unwrap();
        let condition = CssMediaCondition::try_from_components(original.clone()).unwrap();
        let CssMediaConditionKind::UnknownFeature(unknown) = condition.kind() else {
            panic!("unknown feature: {source}");
        };
        assert_eq!(unknown.reason(), reason);
        assert_eq!(unknown.component(), &original.items()[0]);
        assert_eq!(unknown.authored(), Some(source));
    }
    let report = parse_media_query("(aspect-ratio: -1/2)");
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("retained enclosure");
    };
    assert!(matches!(
        condition.kind(),
        CssMediaConditionKind::GeneralEnclosed(_)
    ));
}

#[test]
fn negation_preserves_unknown_conditions_and_separate_unknown_media_types() {
    for source in ["not (future: foo)", "not future()"] {
        let report = parse_media_query(source);
        assert!(report.is_clean());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition negation");
        };
        let CssMediaConditionKind::Not(operand) = condition.kind() else {
            panic!("retained negation");
        };
        assert!(matches!(
            operand.kind(),
            CssMediaConditionKind::UnknownFeature(_) | CssMediaConditionKind::GeneralEnclosed(_)
        ));
    }
    let report = parse_media_query("not future-type");
    let CssMediaQuery::Typed(query) = report.syntax() else {
        panic!("typed media query");
    };
    assert_eq!(
        query.modifier(),
        Some(surgeist_css::CssMediaQueryModifier::Not)
    );
    assert_eq!(query.media_type(), surgeist_css::CssMediaType::Unknown);
    assert!(query.unknown_media_type().is_some());
}

#[test]
fn generic_feature_math_uses_named_numeric_domains_without_admitting_percentages() {
    // MQ5's mf-value includes number and dimension, but not percentage;
    // Values 4 math substitution preserves the final named numeric type.
    for source in [
        "(future: calc(1px))",
        "(future: calc(1fr))",
        "(future: calc(1px / 1px))",
        "(future: calc(1% / 1%))",
        "(future: 3qu)",
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition");
        };
        let CssMediaConditionKind::UnknownFeature(unknown) = condition.kind() else {
            panic!("valid generic feature with unknown name: {source}");
        };
        assert_eq!(
            unknown.reason(),
            surgeist_css::CssUnknownMediaFeatureReason::UnknownName
        );
    }
    for source in ["(future: calc(1%))", "(future: calc(1px * 1px))"] {
        let report = parse_media_query(source);
        assert!(report.is_clean());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition");
        };
        assert!(
            matches!(condition.kind(), CssMediaConditionKind::GeneralEnclosed(_)),
            "{source}"
        );
    }
}

#[test]
fn repeated_identifiers_retain_the_middle_feature_names_own_origin() {
    let source = "(future < future < future)";
    let components = parse_component_values(source).unwrap();
    let surgeist_css::CssComponentValueRef::Block(block) = components.items()[0].view() else {
        panic!("parenthesized feature");
    };
    let expected = block.values().items()[4].origin().clone();
    let condition = CssMediaCondition::try_from_components(components).unwrap();
    let output = condition.serialize().unwrap();
    assert_eq!(output.as_css(), source);
    assert!(
        matches!(output.origin_at(10), Some(CssSerializedOrigin::Token(origin)) if origin == &expected)
    );
}
