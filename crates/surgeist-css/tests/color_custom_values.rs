#![forbid(unsafe_code)]
//! Color 5 custom-profile contracts, checked without serialization as an oracle.
use surgeist_css::*;

fn components(text: &str) -> CssComponentValues {
    parse_component_values(text).unwrap()
}

fn expression(text: &str) -> CssProfileColorExpression {
    CssProfileColorExpression::try_from_components(components(text)).unwrap()
}

fn color(text: &str) -> CssAuthoredColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components(text),
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

fn profile() -> CssColorProfileName {
    CssColorProfileName::try_new("--P").unwrap()
}

fn unwrapped(mut value: CssCalculationExpressionRef<'_>) -> CssCalculationExpressionRef<'_> {
    loop {
        match value {
            CssCalculationExpressionRef::NestedCalc(v) | CssCalculationExpressionRef::Group(v) => {
                value = v.operand()
            }
            _ => return value,
        }
    }
}

#[test]
fn names_use_ident_rules_and_preserve_decoded_case() {
    for name in [
        "alpha", "default", "initial", "inherit", "--", "pi", "a b", "Cyan", "cyan",
    ] {
        assert_eq!(
            CssColorProfileComponentName::try_new(name)
                .unwrap()
                .as_str(),
            name
        );
    }
    for name in ["", "none", "NONE", "nOnE"] {
        assert!(CssColorProfileComponentName::try_new(name).is_none());
    }
    let escaped = expression("C\\79 an");
    let CssProfileColorExpressionRef::Reference(name) = escaped.view() else {
        panic!("reference")
    };
    assert_eq!(name.as_str(), "Cyan");
    assert_eq!(
        name,
        &CssColorProfileComponentName::try_new("Cyan").unwrap()
    );
    assert_ne!(
        name,
        &CssColorProfileComponentName::try_new("cyan").unwrap()
    );
    assert!(matches!(
        expression("NONE").view(),
        CssProfileColorExpressionRef::Literal(CssAuthoredColorComponent::None)
    ));
}

#[test]
fn direct_constants_are_references_but_math_constants_take_precedence() {
    for (name, constant) in [
        ("pi", CssNumericConstant::Pi),
        ("e", CssNumericConstant::E),
        ("infinity", CssNumericConstant::Infinity),
        ("-infinity", CssNumericConstant::NegativeInfinity),
        ("NaN", CssNumericConstant::NaN),
    ] {
        let direct = expression(name);
        let CssProfileColorExpressionRef::Reference(reference) = direct.view() else {
            panic!("direct reference")
        };
        assert_eq!(reference.as_str(), name);
        let math = expression(&format!("calc({name})"));
        let CssProfileColorExpressionRef::Calculation(calculation) = math.view() else {
            panic!("math")
        };
        assert!(calculation.references().is_empty());
        let CssCalculationExpressionRef::Constant(value) = unwrapped(calculation.expression())
        else {
            panic!("constant")
        };
        assert_eq!(value.value(), constant);
    }
    let math = expression("calc(pi + cyan + cyan + Magenta)");
    let CssProfileColorExpressionRef::Calculation(calculation) = math.view() else {
        panic!("math")
    };
    assert_eq!(
        calculation
            .references()
            .iter()
            .map(CssColorProfileComponentName::as_str)
            .collect::<Vec<_>>(),
        ["cyan", "cyan", "Magenta"]
    );
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
}

#[test]
fn profile_math_is_scoped_and_has_no_percentage_hint() {
    for invalid in [
        "",
        "cyan magenta",
        "calc(none)",
        "calc(cyan + 1%)",
        "calc(1px)",
        "calc(1deg)",
        "var(--x)",
        "env(x)",
    ] {
        assert!(
            CssProfileColorExpression::try_from_components(components(invalid)).is_err(),
            "{invalid}"
        );
    }
    for valid in ["calc(cyan * 2)", "min(cyan, magenta)", "calc(20% / 2)"] {
        expression(valid);
    }
    assert!(CssNumberCalculation::try_from_components(components("calc(cyan)")).is_err());
    assert!(CssPercentageCalculation::try_from_components(components("calc(cyan)")).is_err());
}

#[test]
fn programmatic_math_retains_parsed_profile_leaf_origin() {
    let leaf = components("Cyan").items()[0].clone();
    let graph = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "calc",
            CssComponentValues::try_new(vec![leaf.clone()]).unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let value = CssProfileColorExpression::try_from_components(graph.clone()).unwrap();
    assert_eq!(value.components(), &graph);
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    let CssProfileColorExpressionRef::Calculation(calculation) = value.view() else {
        panic!("math")
    };
    assert_eq!(calculation.components(), &graph);
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    let CssCalculationExpressionRef::ProfileChannel(channel) = unwrapped(calculation.expression())
    else {
        panic!("profile leaf")
    };
    assert_eq!(channel.name().as_str(), "Cyan");
    assert_eq!(channel.origin(), leaf.origin());
}

#[test]
fn checked_custom_lists_retain_exact_values_order_and_omission() {
    let literal =
        CssColorNumberLiteral::try_from_component(components("1e100").items()[0].clone()).unwrap();
    let channels = vec![
        CssAuthoredColorComponent::ExactNumber(literal.clone()),
        CssAuthoredColorComponent::None,
        CssAuthoredColorComponent::Percentage(CssFiniteNumber::try_new(-20.0).unwrap()),
    ];
    let custom = CssAuthoredCustomColor::try_new(profile(), channels.clone(), None).unwrap();
    assert_eq!(custom.profile(), &profile());
    assert_eq!(custom.channels(), channels);
    assert!(custom.alpha().is_none());
    assert_eq!(literal.numeric().representation(), "1e100");
    let current = CssAuthoredColor::from_custom(custom.clone());
    assert_eq!(current.custom_value(), Some(&custom));
    assert_eq!(
        current.absolute_eligibility(),
        CssAbsoluteColorEligibility::ProfileDependent
    );
    assert_eq!(
        CssAuthoredCustomColor::try_new(profile(), vec![], None).unwrap_err(),
        CssAuthoredColorConstructionError::EmptyComponents
    );
    let explicit_none =
        CssAuthoredCustomColor::try_new(profile(), channels, Some(CssAuthoredColorComponent::None))
            .unwrap();
    assert_ne!(custom, explicit_none);
}

#[test]
fn relative_profile_alpha_is_an_unbound_name_and_not_transparency() {
    let channels = vec![
        expression("alpha"),
        expression("initial"),
        expression("none"),
        expression("pi"),
    ];
    let custom = CssAuthoredRelativeCustomColor::try_new(
        color("red"),
        profile(),
        channels.clone(),
        Some(expression("alpha")),
    )
    .unwrap();
    assert_eq!(custom.source(), &color("red"));
    assert_eq!(custom.profile(), &profile());
    assert_eq!(custom.channels(), channels);
    let CssProfileColorExpressionRef::Reference(alpha) = custom.alpha().unwrap().view() else {
        panic!("unbound alpha")
    };
    assert_eq!(alpha.as_str(), "alpha");
    let current = CssAuthoredColor::from_relative_custom(custom.clone());
    assert_eq!(current.relative_custom_value(), Some(&custom));
    assert!(current.relative_value().is_none());
    assert_eq!(
        CssAuthoredRelativeCustomColor::try_new(color("red"), profile(), vec![], None).unwrap_err(),
        CssAuthoredColorConstructionError::EmptyComponents
    );
}

#[test]
fn custom_and_relative_math_share_the_composed_depth_limit() {
    for depth in [254, 255, 256] {
        let text = format!("{}1{}", "calc(".repeat(depth), ")".repeat(depth));
        let calculation = CssNumberCalculation::try_from_components(components(&text)).unwrap();
        let custom = CssAuthoredCustomColor::try_new(
            profile(),
            vec![CssAuthoredColorComponent::NumberCalculation(calculation)],
            None,
        );
        let relative = CssAuthoredRelativeCustomColor::try_new(
            color("red"),
            profile(),
            vec![expression(&text)],
            None,
        );
        if depth < 256 {
            assert!(custom.is_ok());
            assert!(relative.is_ok());
        } else {
            assert_eq!(
                custom.unwrap_err(),
                CssAuthoredColorConstructionError::NestingLimit
            );
            assert_eq!(
                relative.unwrap_err(),
                CssAuthoredColorConstructionError::NestingLimit
            );
        }
    }
}

#[test]
fn wide_custom_lists_keep_cumulative_token_and_byte_budgets() {
    let width = 128;
    let source = format!("color(--P {})", vec!["1"; width].join(" "));
    // Function + profile identifier + one whitespace and number per channel.
    let count = 2 + 2 * width;
    let limits = CssComponentValueLimits::try_new(256, count, source.len()).unwrap();
    let graph = parse_component_values_with_limits(&source, limits).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        graph.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    assert_eq!(
        value.current().custom_value().unwrap().channels().len(),
        width
    );
    for (tokens, bytes, expected) in [
        (
            count - 1,
            source.len(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            count,
            source.len() - 1,
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let limits = CssComponentValueLimits::try_new(256, tokens, bytes).unwrap();
        assert_eq!(
            parse_component_values_with_limits(&source, limits)
                .unwrap_err()
                .kind(),
            expected
        );
        assert_eq!(
            CssComponentValues::try_new_with_limits(graph.items().to_vec(), limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
}
