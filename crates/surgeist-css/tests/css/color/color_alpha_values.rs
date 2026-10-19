#![forbid(unsafe_code)]
//! Color 5 alpha and authored absolute-eligibility contracts.
use surgeist_css::*;

fn color(text: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.value().clone()
}

fn alpha(text: &str) -> CssRelativeColorExpression {
    CssRelativeColorExpression::try_from_components(
        parse_component_values(text).unwrap(),
        CssRelativeColorEnvironment::Alpha,
        CssRelativeColorResultDomain::Alpha,
    )
    .unwrap()
}

#[test]
fn checked_alpha_keeps_omission_missing_and_channel_distinct() {
    let source = color("red");
    let omitted = CssAlphaColor::try_new(source.clone(), None).unwrap();
    assert_eq!(omitted.source(), &source);
    assert!(omitted.alpha().is_none());
    let missing = CssAlphaColor::try_new(source.clone(), Some(alpha("none"))).unwrap();
    let channel = CssAlphaColor::try_new(source, Some(alpha("alpha"))).unwrap();
    assert_ne!(omitted, missing);
    assert_ne!(missing, channel);
    let expression = channel.alpha().unwrap();
    assert_eq!(expression.environment(), CssRelativeColorEnvironment::Alpha);
    assert_eq!(
        expression.result_domain(),
        CssRelativeColorResultDomain::Alpha
    );
    assert_eq!(
        expression.value(),
        &CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::Alpha)
    );
    let current = CssColor::from_alpha(channel.clone());
    assert_eq!(current.alpha_value(), Some(&channel));
    assert!(current.relative_value().is_none());
}

#[test]
fn alpha_math_keeps_transparency_references_and_rejects_foreign_environments() {
    let expression = alpha("calc(alpha / 2)");
    let CssRelativeColorExpressionValue::Calculation(calculation) = expression.value() else {
        panic!("math")
    };
    assert_eq!(calculation.references(), &[CssRelativeColorChannel::Alpha]);
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    for text in [
        "r",
        "cyan",
        "calc(r)",
        "calc(cyan)",
        "1px",
        "1deg",
        "calc(none)",
        "alpha alpha",
        "var(--x)",
    ] {
        assert!(
            CssRelativeColorExpression::try_from_components(
                parse_component_values(text).unwrap(),
                CssRelativeColorEnvironment::Alpha,
                CssRelativeColorResultDomain::Alpha,
            )
            .is_err(),
            "{text}"
        );
    }
    let rgb = color("rgb(from red r g b / alpha)");
    let foreign = rgb.relative_value().unwrap().alpha().unwrap().clone();
    assert_eq!(foreign.result_domain(), CssRelativeColorResultDomain::Alpha);
    assert_eq!(
        CssAlphaColor::try_new(color("red"), Some(foreign)).unwrap_err(),
        CssColorConstructionError::InvalidExpressionEnvironment
    );
}

#[test]
fn alpha_exact_literals_remain_unclamped() {
    for text in ["1e100", "1e-100"] {
        let expression = alpha(text);
        let CssRelativeColorExpressionValue::Number(value) = expression.value() else {
            panic!("exact number")
        };
        assert_eq!(value.numeric().representation(), text);
    }
    assert!(
        matches!(alpha("-2").value(), CssRelativeColorExpressionValue::Number(v)
        if v.numeric().representation() == "-2")
    );
    assert!(
        matches!(alpha("200%").value(), CssRelativeColorExpressionValue::Percentage(v)
        if v.numeric().representation() == "200")
    );
}

#[test]
fn eligibility_walks_every_child_and_context_dominates_profile_dependence() {
    for text in [
        "red",
        "rgb(from red r g b)",
        "alpha(from red)",
        "color-mix(red, blue, green)",
    ] {
        assert_eq!(
            color(text).absolute_eligibility(),
            CssAbsoluteColorEligibility::Eligible,
            "{text}"
        );
    }
    for text in [
        "color(--Unknown 1)",
        "color(from red --Unknown cyan)",
        "color-mix(in --Unknown, red)",
        "alpha(from color(--Unknown 1))",
    ] {
        assert_eq!(
            color(text).absolute_eligibility(),
            CssAbsoluteColorEligibility::ProfileDependent,
            "{text}"
        );
    }
    for text in [
        "currentcolor",
        "alpha(from currentcolor)",
        "color(from currentcolor --P cyan)",
        "color-mix(color(--P 1), red, currentcolor)",
        "color-mix(in --P, currentcolor)",
    ] {
        assert_eq!(
            color(text).absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::CurrentColor),
            "{text}"
        );
    }
    for (text, exclusion) in [
        ("CanvasText", CssAbsoluteColorExclusion::SystemColor),
        (
            "color-mix(CanvasText, currentcolor)",
            CssAbsoluteColorExclusion::SystemColor,
        ),
        (
            "color-mix(currentcolor, CanvasText)",
            CssAbsoluteColorExclusion::CurrentColor,
        ),
    ] {
        assert_eq!(
            color(text).absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(exclusion),
            "{text}"
        );
    }
}

#[test]
fn checked_alpha_graph_enforces_depth_without_discarding_children() {
    let mut current = color("red");
    for _ in 0..256 {
        current = CssColor::from_alpha(CssAlphaColor::try_new(current, None).unwrap());
    }
    assert_eq!(current.clone(), current);
    assert_eq!(
        current.absolute_eligibility(),
        CssAbsoluteColorEligibility::Eligible
    );
    assert_eq!(
        CssAlphaColor::try_new(current, None).unwrap_err(),
        CssColorConstructionError::NestingLimit
    );
    for depth in [254, 255, 256] {
        let text = format!("{}alpha{}", "calc(".repeat(depth), ")".repeat(depth));
        let result = CssAlphaColor::try_new(color("red"), Some(alpha(&text)));
        if depth < 256 {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err(), CssColorConstructionError::NestingLimit);
        }
    }
}

#[test]
fn nested_alpha_custom_and_mix_branches_remain_typed() {
    assert!(color("alpha(from red)").alpha_value().is_some());
    assert_eq!(
        color("color(--P 1)")
            .custom_value()
            .unwrap()
            .channels()
            .len(),
        1
    );
    assert_eq!(
        color("color(from red --P cyan)")
            .relative_custom_value()
            .unwrap()
            .channels()
            .len(),
        1
    );
    let relative = color("rgb(from alpha(from red) r g b)");
    assert!(
        relative
            .relative_value()
            .unwrap()
            .source()
            .alpha_value()
            .is_some()
    );
    let mix = color("color-mix(in srgb, red, color(--P 1))");
    assert_eq!(mix.color_mix_value().unwrap().components().len(), 2);
    assert!(
        mix.color_mix_value().unwrap().components()[1]
            .color()
            .custom_value()
            .is_some()
    );
}

#[test]
fn alpha_math_retains_parsed_leaf_in_a_programmatic_function() {
    let leaf = parse_component_values("alpha").unwrap().items()[0].clone();
    let graph = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "calc",
            CssComponentValues::try_new(vec![leaf.clone()]).unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let value = CssRelativeColorExpression::try_from_components(
        graph,
        CssRelativeColorEnvironment::Alpha,
        CssRelativeColorResultDomain::Alpha,
    )
    .unwrap();
    let CssRelativeColorExpressionValue::Calculation(calculation) = value.value() else {
        panic!("math")
    };
    let mut expression = calculation.expression();
    assert_eq!(expression.origin(), &CssValueOrigin::Programmatic);
    loop {
        match expression {
            CssCalculationExpressionRef::NestedCalc(v) | CssCalculationExpressionRef::Group(v) => {
                expression = v.operand()
            }
            CssCalculationExpressionRef::Variable(v) => {
                assert_eq!(v.channel(), CssRelativeColorChannel::Alpha);
                assert_eq!(v.origin(), leaf.origin());
                break;
            }
            _ => panic!("transparency leaf"),
        }
    }
}
