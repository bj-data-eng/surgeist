#![forbid(unsafe_code)]
//! Public authored-color construction contracts from CSS Color 4 and the
//! 2026-09-08 CSS Color 5 relative-color and color-mix definitions.

use surgeist_css::*;

fn token(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    assert_eq!(values.items().len(), 1, "{text}");
    values.items()[0].clone()
}

fn number(text: &str) -> CssColorComponent {
    CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token(text)).unwrap())
}

fn percentage(text: &str) -> CssColorComponent {
    CssColorComponent::Percentage(
        CssColorPercentageLiteral::try_from_component(token(text)).unwrap(),
    )
}

fn hue(text: &str) -> CssColorHue {
    CssColorHue::Number(CssColorNumberLiteral::try_from_component(token(text)).unwrap())
}

fn named(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}

fn relative_expression(
    text: &str,
    environment: CssRelativeColorEnvironment,
    domain: CssRelativeColorResultDomain,
) -> CssRelativeColorExpression {
    CssRelativeColorExpression::try_from_components(
        parse_component_values(text).unwrap(),
        environment,
        domain,
    )
    .unwrap()
}

#[test]
fn hex_named_and_symbolic_keywords_have_checked_distinct_branches() {
    for digits in ["aBc", "ABcd", "123aBc", "123aBcDe"] {
        let hex = CssHexColor::try_new(digits).unwrap();
        assert_eq!(
            CssColor::from_hex(hex).hex_value().unwrap().digits(),
            digits
        );
    }
    for digits in ["", "ab", "abcde", "abcdefg", "abcdefghi", "abz", "ébc"] {
        assert!(CssHexColor::try_new(digits).is_none(), "{digits}");
    }
    let named = named("ReBeccAPurple");
    assert_eq!(named.named().unwrap().name(), "rebeccapurple");
    assert_eq!(named.to_specified_css().unwrap(), "rebeccapurple");
    for name in ["", "not-a-css-color", "currentcolor", "transparent"] {
        assert!(CssNamedColor::try_new(name).is_none(), "{name}");
    }
    let current = CssColor::current_color();
    assert!(current.is_current_color());
    assert_eq!(current.to_specified_css().unwrap(), "currentcolor");
    let transparent = CssColor::transparent();
    assert!(transparent.is_transparent());
    assert_eq!(transparent.to_specified_css().unwrap(), "transparent");
    let system = CssColor::from_system(CssSystemColor::CanvasText);
    assert_eq!(system.system(), Some(CssSystemColor::CanvasText));
    assert_eq!(system.to_specified_css().unwrap(), "canvastext");
}

#[test]
fn rgb_constructor_checks_legacy_domains_and_retains_modern_missing_components() {
    let legacy = CssRgbColor::try_new(
        CssColorSyntax::Legacy,
        [number("1"), number("2"), number("3")],
        Some(percentage("120%")),
    )
    .unwrap();
    assert_eq!(legacy.syntax(), CssColorSyntax::Legacy);
    assert!(
        matches!(legacy.channels()[1], CssColorComponent::Number(ref value)
        if value.numeric().representation() == "2"
            && matches!(value.origin(), CssValueOrigin::Parsed(_)))
    );
    assert!(
        matches!(legacy.alpha(), Some(CssColorComponent::Percentage(value))
        if value.numeric().representation() == "120")
    );
    assert!(CssColor::from_rgb(legacy).rgb_value().is_some());
    for channels in [
        [number("1"), percentage("2%"), number("3")],
        [number("1"), CssColorComponent::None, number("3")],
    ] {
        assert_eq!(
            CssRgbColor::try_new(CssColorSyntax::Legacy, channels, None).unwrap_err(),
            CssColorConstructionError::InvalidSyntax
        );
    }
    assert_eq!(
        CssRgbColor::try_new(
            CssColorSyntax::Legacy,
            [percentage("1%"), percentage("2%"), percentage("3%")],
            Some(CssColorComponent::None),
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidSyntax
    );
    let modern = CssRgbColor::try_new(
        CssColorSyntax::Modern,
        [number("1e100"), percentage("0.1%"), CssColorComponent::None],
        Some(CssColorComponent::None),
    )
    .unwrap();
    assert_eq!(modern.syntax(), CssColorSyntax::Modern);
    assert!(
        matches!(modern.channels()[0], CssColorComponent::Number(ref value)
        if value.numeric().representation() == "1e100")
    );
    assert!(matches!(modern.channels()[2], CssColorComponent::None));
    assert!(matches!(modern.alpha(), Some(CssColorComponent::None)));
}

#[test]
fn hsl_legacy_rules_and_other_absolute_payloads_preserve_authored_fields() {
    let legacy = CssHslColor::try_new(
        CssColorSyntax::Legacy,
        hue("450"),
        percentage("120%"),
        percentage("-10%"),
        None,
    )
    .unwrap();
    assert_eq!(legacy.syntax(), CssColorSyntax::Legacy);
    assert!(
        matches!(legacy.saturation(), CssColorComponent::Percentage(value)
        if value.numeric().representation() == "120")
    );
    assert!(legacy.alpha().is_none());
    assert!(CssColor::from_hsl(legacy).hsl_value().is_some());
    assert_eq!(
        CssHslColor::try_new(
            CssColorSyntax::Legacy,
            hue("1"),
            number("20"),
            percentage("30%"),
            None,
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidSyntax
    );
    assert_eq!(
        CssHslColor::try_new(
            CssColorSyntax::Legacy,
            CssColorHue::None,
            percentage("20%"),
            percentage("30%"),
            None,
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidSyntax
    );
    assert_eq!(
        CssHslColor::try_new(
            CssColorSyntax::Legacy,
            hue("1"),
            percentage("20%"),
            percentage("30%"),
            Some(CssColorComponent::None),
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidSyntax
    );
    let modern = CssHslColor::try_new(
        CssColorSyntax::Modern,
        CssColorHue::None,
        number("20"),
        CssColorComponent::None,
        Some(CssColorComponent::None),
    )
    .unwrap();
    assert!(matches!(modern.hue(), CssColorHue::None));
    assert!(matches!(modern.saturation(), CssColorComponent::Number(_)));
    assert!(matches!(modern.lightness(), CssColorComponent::None));
    assert!(CssColor::from_hsl(modern).hsl_value().is_some());

    let hwb = CssHwbColor::try_new(hue("720"), percentage("120%"), number("-5"), None).unwrap();
    assert!(matches!(hwb.hue(), CssColorHue::Number(value)
        if value.numeric().representation() == "720"));
    assert!(
        matches!(hwb.whiteness(), CssColorComponent::Percentage(value)
        if value.numeric().representation() == "120")
    );
    assert!(hwb.alpha().is_none());
    assert!(CssColor::from_hwb(hwb).hwb_value().is_some());

    let lab = CssLabColor::try_new(number("120"), number("-10"), number("3"), None).unwrap();
    assert!(matches!(lab.lightness(), CssColorComponent::Number(value)
        if value.numeric().representation() == "120"));
    assert!(lab.alpha().is_none());
    assert!(CssColor::from_lab(lab.clone()).lab_value().is_some());
    assert!(CssColor::from_oklab(lab).oklab_value().is_some());
    let lch = CssLchColor::try_new(number("120"), number("-3"), hue("450"), None).unwrap();
    assert!(matches!(lch.chroma(), CssColorComponent::Number(value)
        if value.numeric().representation() == "-3"));
    assert!(matches!(lch.hue(), CssColorHue::Number(value)
        if value.numeric().representation() == "450"));
    assert!(CssColor::from_lch(lch.clone()).lch_value().is_some());
    assert!(CssColor::from_oklch(lch).oklch_value().is_some());

    let predefined = CssPredefinedColor::try_new(
        CssPredefinedColorSpace::DisplayP3,
        [number("1"), percentage("50%"), CssColorComponent::None],
        None,
    )
    .unwrap();
    assert_eq!(predefined.color_space(), CssPredefinedColorSpace::DisplayP3);
    assert!(
        matches!(&predefined.channels()[1], CssColorComponent::Percentage(value)
        if value.numeric().representation() == "50")
    );
    assert!(predefined.alpha().is_none());
    assert!(
        CssColor::from_predefined(predefined)
            .predefined_value()
            .is_some()
    );
}

#[test]
fn relative_expression_construction_preserves_origin_and_precise_errors() {
    let programmatic = CssRelativeColorExpression::try_from_components(
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("h").unwrap()]).unwrap(),
        CssRelativeColorEnvironment::Hsl,
        CssRelativeColorResultDomain::Hue,
    )
    .unwrap();
    assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
    assert!(matches!(
        programmatic.value(),
        CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::H)
    ));
    let parsed = relative_expression(
        "calc(h + 180)",
        CssRelativeColorEnvironment::Hsl,
        CssRelativeColorResultDomain::Hue,
    );
    assert!(matches!(parsed.origin(), CssValueOrigin::Parsed(_)));
    let CssRelativeColorExpressionValue::Calculation(calculation) = parsed.value() else {
        panic!("expected retained relative calculation");
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_eq!(calculation.references(), &[CssRelativeColorChannel::H]);

    let bad = parse_component_values(" /* gap */ 1px").unwrap();
    let bad_index = bad.items().len() - 1;
    let bad_origin = bad.items()[bad_index].origin().clone();
    let error = CssRelativeColorExpression::try_from_components(
        bad,
        CssRelativeColorEnvironment::Hsl,
        CssRelativeColorResultDomain::Hue,
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::Component(CssComponentValueErrorKind::InvalidToken)
    );
    assert_eq!(error.origin(), Some(&bad_origin));
    assert_eq!(error.path(), Some(&[bad_index][..]));
    for (text, environment, domain, kind) in [
        (
            "h",
            CssRelativeColorEnvironment::Alpha,
            CssRelativeColorResultDomain::Hue,
            CssNumericConstructionErrorKind::InvalidArgumentType,
        ),
        (
            "10%",
            CssRelativeColorEnvironment::Hsl,
            CssRelativeColorResultDomain::Hue,
            CssNumericConstructionErrorKind::RootDomainMismatch,
        ),
    ] {
        let error = CssRelativeColorExpression::try_from_components(
            parse_component_values(text).unwrap(),
            environment,
            domain,
        )
        .unwrap_err();
        assert_eq!(error.kind(), &kind, "{text}");
        assert_eq!(error.path(), Some(&[0][..]));
    }
    let extra_values = parse_component_values("h s").unwrap();
    let extra_origin = extra_values.items()[2].origin().clone();
    let extra = CssRelativeColorExpression::try_from_components(
        extra_values,
        CssRelativeColorEnvironment::Hsl,
        CssRelativeColorResultDomain::Hue,
    )
    .unwrap_err();
    assert_eq!(
        extra.kind(),
        &CssNumericConstructionErrorKind::MultipleValues
    );
    assert_eq!(extra.origin(), Some(&extra_origin));
    assert_eq!(extra.path(), Some(&[2][..]));
    let nested_values = parse_component_values("calc(h + 1px)").unwrap();
    let CssComponentValueRef::Function(function) = nested_values.items()[0].view() else {
        panic!("expected calc function");
    };
    let nested_origin = function.values().items()[4].origin().clone();
    let nested = CssRelativeColorExpression::try_from_components(
        nested_values,
        CssRelativeColorEnvironment::Hsl,
        CssRelativeColorResultDomain::Hue,
    )
    .unwrap_err();
    assert_eq!(
        nested.kind(),
        &CssNumericConstructionErrorKind::IncompatibleTypes
    );
    assert_eq!(nested.origin(), Some(&nested_origin));
    assert_eq!(nested.path(), Some(&[0, 4][..]));
    let CssValueOrigin::Parsed(origin) = nested_origin else {
        panic!("expected original parsed offending dimension");
    };
    assert_eq!(origin.source().as_str(), "calc(h + 1px)");
    assert_eq!(origin.span().start().byte_offset().value(), 9);
    assert_eq!(origin.span().end().byte_offset().value(), 12);
    for environment in [
        CssRelativeColorEnvironment::PredefinedRgb(CssPredefinedColorSpace::XyzD50),
        CssRelativeColorEnvironment::Xyz(CssPredefinedColorSpace::Srgb),
    ] {
        let error = CssRelativeColorExpression::try_from_components(
            parse_component_values("r").unwrap(),
            environment,
            CssRelativeColorResultDomain::NumberPercentage,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::InvalidArgumentType
        );
        assert_eq!(error.path(), Some(&[0][..]));
    }
}

#[test]
fn relative_color_constructor_derives_signature_and_checks_each_slot() {
    use CssRelativeColorEnvironment as E;
    use CssRelativeColorResultDomain as D;
    let channels = [
        relative_expression("h", E::Hsl, D::Hue),
        relative_expression("s", E::Hsl, D::NumberPercentage),
        relative_expression("l", E::Hsl, D::NumberPercentage),
    ];
    let alpha = relative_expression("alpha", E::Hsl, D::Alpha);
    let relative = CssRelativeColor::try_new(
        CssRelativeColorFunction::Hsl,
        named("red"),
        channels.clone(),
        Some(alpha),
    )
    .unwrap();
    assert_eq!(relative.environment(), E::Hsl);
    assert!(matches!(relative.function(), CssRelativeColorFunction::Hsl));
    assert_eq!(relative.source().named().unwrap().name(), "red");
    assert_eq!(relative.channels()[0].result_domain(), D::Hue);
    assert_eq!(relative.alpha().unwrap().result_domain(), D::Alpha);
    let color = CssColor::from_relative(relative);
    assert_eq!(
        color.to_specified_css().unwrap(),
        "hsl(from red h s l / alpha)"
    );

    let wrong_environment = [
        relative_expression("h", E::Hwb, D::Hue),
        channels[1].clone(),
        channels[2].clone(),
    ];
    assert_eq!(
        CssRelativeColor::try_new(
            CssRelativeColorFunction::Hsl,
            named("red"),
            wrong_environment,
            None
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidExpressionEnvironment
    );
    let wrong_slot = [
        relative_expression("h", E::Hsl, D::NumberPercentage),
        channels[1].clone(),
        channels[2].clone(),
    ];
    assert_eq!(
        CssRelativeColor::try_new(
            CssRelativeColorFunction::Hsl,
            named("red"),
            wrong_slot,
            None
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidExpressionEnvironment
    );
    assert_eq!(
        CssRelativeColor::try_new(
            CssRelativeColorFunction::Rgb,
            named("red"),
            channels.clone(),
            None
        )
        .unwrap_err(),
        CssColorConstructionError::InvalidExpressionEnvironment
    );
    for alpha in [
        relative_expression("alpha", E::Hwb, D::Alpha),
        relative_expression("alpha", E::Hsl, D::NumberPercentage),
    ] {
        assert_eq!(
            CssRelativeColor::try_new(
                CssRelativeColorFunction::Hsl,
                named("red"),
                channels.clone(),
                Some(alpha),
            )
            .unwrap_err(),
            CssColorConstructionError::InvalidExpressionEnvironment
        );
    }
}

#[test]
fn mix_constructor_retains_order_weights_and_symbolic_calculation() {
    let tiny = "0.0000000000000000000001%";
    let tiny_weight =
        CssColorMixWeight::literal(CssColorMixPercentage::try_from_component(token(tiny)).unwrap());
    let symbolic = CssColorMixWeight::try_calculation(
        CssPercentageCalculation::try_from_components(
            parse_component_values("calc(25% + 10%)").unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let interpolation = CssColorInterpolation::try_predefined(CssColorInterpolationMethod::new(
        CssColorInterpolationSpace::Lch,
        Some(CssHueInterpolationMethod::Longer),
    ))
    .unwrap();
    let mix = CssColorMix::try_new(
        Some(interpolation),
        vec![
            CssColorMixComponent::new(named("red"), Some(tiny_weight)),
            CssColorMixComponent::new(named("blue"), Some(symbolic)),
            CssColorMixComponent::new(named("green"), None),
        ],
    )
    .unwrap();
    assert_eq!(mix.components().len(), 3);
    assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
    assert_eq!(mix.components()[1].color().named().unwrap().name(), "blue");
    assert_eq!(mix.components()[2].color().named().unwrap().name(), "green");
    assert_eq!(
        mix.components()[0]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "0.0000000000000000000001"
    );
    let calculation = mix.components()[1].weight().unwrap().calculation().unwrap();
    assert_eq!(calculation.result_type(), CssCalculationType::Percentage);
    assert_eq!(
        calculation.components().serialize().unwrap().as_css(),
        "calc(25% + 10%)"
    );
    assert!(mix.components()[2].weight().is_none());
    assert_eq!(
        mix.interpolation().unwrap().predefined().unwrap().hue(),
        Some(CssHueInterpolationMethod::Longer)
    );
    assert!(CssColor::from_color_mix(mix).color_mix_value().is_some());
    assert_eq!(
        CssColorMix::try_new(None, vec![]).unwrap_err(),
        CssColorMixConstructionError::EmptyComponents
    );
    assert!(
        CssColorInterpolation::try_predefined(CssColorInterpolationMethod::new(
            CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
            Some(CssHueInterpolationMethod::Longer),
        ))
        .is_none()
    );
    for text in ["-0.0000000000000000000001%", "100.0000000000000000000001%"] {
        let error = CssColorMixPercentage::try_from_component(token(text)).unwrap_err();
        assert_eq!(error.kind(), CssColorScalarErrorKind::OutOfRange);
    }
    let precise =
        CssColorMixPercentage::try_from_component(token("99.9999999999999999999999%")).unwrap();
    assert_eq!(
        precise.literal().numeric().representation(),
        "99.9999999999999999999999"
    );
    let default_mix = CssColor::from_color_mix(
        CssColorMix::try_new(
            None,
            vec![
                CssColorMixComponent::new(named("red"), None),
                CssColorMixComponent::new(named("blue"), None),
            ],
        )
        .unwrap(),
    );
    assert_eq!(
        default_mix.to_specified_css().unwrap(),
        "color-mix(red, blue)"
    );
}

#[test]
fn construction_depth_and_serialization_budget_have_checked_boundaries() {
    let mut color = named("red");
    for _ in 0..256 {
        color = CssColor::from_alpha(CssAlphaColor::try_new(color, None).unwrap());
    }
    assert_eq!(
        CssAlphaColor::try_new(color, None).unwrap_err(),
        CssColorConstructionError::NestingLimit
    );
    let purple = named("purple");
    assert_eq!(
        purple
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "purple"
    );
    assert_eq!(
        purple
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 5))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
