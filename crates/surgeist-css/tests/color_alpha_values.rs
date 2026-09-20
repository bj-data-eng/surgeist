#![forbid(unsafe_code)]
//! Color 5 alpha and authored absolute-eligibility contracts.
use surgeist_css::*;

fn color(text: &str) -> CssAuthoredColor {
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
    value.current().clone()
}

fn alpha(text: &str) -> CssTypedRelativeColorExpression {
    CssTypedRelativeColorExpression::try_alpha_from_components(
        parse_component_values(text).unwrap(),
    )
    .unwrap()
}

#[test]
fn checked_alpha_keeps_omission_missing_and_channel_distinct() {
    let source = color("red");
    let omitted = CssAuthoredAlphaColor::try_new(source.clone(), None).unwrap();
    assert_eq!(omitted.source(), &source);
    assert!(omitted.alpha().is_none());
    let missing = CssAuthoredAlphaColor::try_new(source.clone(), Some(alpha("none"))).unwrap();
    let channel = CssAuthoredAlphaColor::try_new(source, Some(alpha("alpha"))).unwrap();
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
    let current = CssAuthoredColor::from_alpha(channel.clone());
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
            CssTypedRelativeColorExpression::try_alpha_from_components(
                parse_component_values(text).unwrap()
            )
            .is_err(),
            "{text}"
        );
    }
    let rgb = color("rgb(from red r g b / alpha)");
    let foreign = rgb.relative_value().unwrap().alpha().unwrap().clone();
    assert_eq!(foreign.result_domain(), CssRelativeColorResultDomain::Alpha);
    assert_eq!(
        CssAuthoredAlphaColor::try_new(color("red"), Some(foreign)).unwrap_err(),
        CssAuthoredColorConstructionError::InvalidExpressionEnvironment
    );
}

#[test]
fn alpha_exact_literals_remain_unclamped() {
    for text in ["1e100", "1e-100"] {
        let expression = alpha(text);
        let CssRelativeColorExpressionValue::ExactNumber(value) = expression.value() else {
            panic!("exact number")
        };
        assert_eq!(value.numeric().representation(), text);
    }
    assert_eq!(
        alpha("-2").value(),
        &CssRelativeColorExpressionValue::Number(CssFiniteNumber::try_new(-2.0).unwrap())
    );
    assert_eq!(
        alpha("200%").value(),
        &CssRelativeColorExpressionValue::Percentage(CssFiniteNumber::try_new(200.0).unwrap())
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
        current =
            CssAuthoredColor::from_alpha(CssAuthoredAlphaColor::try_new(current, None).unwrap());
    }
    assert_eq!(current.clone(), current);
    assert_eq!(
        current.absolute_eligibility(),
        CssAbsoluteColorEligibility::Eligible
    );
    assert_eq!(
        CssAuthoredAlphaColor::try_new(current, None).unwrap_err(),
        CssAuthoredColorConstructionError::NestingLimit
    );
    for depth in [254, 255, 256] {
        let text = format!("{}alpha{}", "calc(".repeat(depth), ")".repeat(depth));
        let result = CssAuthoredAlphaColor::try_new(color("red"), Some(alpha(&text)));
        if depth < 256 {
            assert!(result.is_ok());
        } else {
            assert_eq!(
                result.unwrap_err(),
                CssAuthoredColorConstructionError::NestingLimit
            );
        }
    }
}

#[test]
fn new_branches_have_no_frozen_projection_even_inside_old_branches() {
    for text in [
        "alpha(from red)",
        "color(--P 1)",
        "color(from red --P cyan)",
        "rgb(from alpha(from red) r g b)",
        "color-mix(in srgb, red, color(--P 1))",
    ] {
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
        assert!(value.i01_subset().is_none(), "{text}");
    }
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
    let value = CssTypedRelativeColorExpression::try_alpha_from_components(graph).unwrap();
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
