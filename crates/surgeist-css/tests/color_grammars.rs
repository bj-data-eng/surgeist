use surgeist_css::{
    CssColorComponent, CssColorHue, CssColorInterpolationSpace, CssColorSyntax,
    CssHueInterpolationMethod, CssKnownDeclaredValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssOpacityValue, CssPredefinedColorSpace,
    CssRelativeColorEnvironment, CssRelativeColorFunction, CssSystemColor, parse_style_attribute,
};

fn color_value(source: &str) -> surgeist_css::CssColorPropertyValue {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .expect("known color declaration")
        .property_value()
        .expect("ordinary color value")
    else {
        panic!("expected color wrapper");
    };
    value.clone()
}

#[test]
fn opacity_percentage_is_retained_with_its_valid_sibling() {
    let report = parse_style_attribute("opacity: 150%; color: red");

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
    );
    assert_eq!(
        report.syntax()[1].known().map(|known| known.property()),
        Some(CssKnownProperty::Color),
    );
}

#[test]
fn deprecated_system_color_is_retained_with_its_valid_sibling() {
    let report = parse_style_attribute("color: ActiveBorder; opacity: 0.5");

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color),
    );
    assert_eq!(
        report.syntax()[1].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
    );
}

#[test]
fn perceptual_color_with_typed_math_is_retained_with_its_valid_sibling() {
    let report =
        parse_style_attribute("color: lab(calc(50% + 10%) calc(20 + 5) -30 / 120%); opacity: 0.5");

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color),
    );
    assert_eq!(
        report.syntax()[1].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
    );
}

#[test]
fn relative_rgb_rejects_untyped_channel_identifiers_and_retains_its_valid_sibling() {
    let report = parse_style_attribute("color: rgb(from red bogus bogus bogus); opacity: 0.5");

    assert_eq!(report.syntax().len(), 1, "{:?}", report.diagnostics());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
    );
}

#[test]
fn color_mix_preserved_subset_rejects_cross_space_hue_methods() {
    let report =
        parse_style_attribute("color: color-mix(in srgb longer hue, red, blue); opacity: 0.5");

    assert_eq!(report.syntax().len(), 1, "{:?}", report.diagnostics());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity),
    );
}

#[test]
fn color_mix_preserved_subset_retains_space_hue_components_and_order() {
    let value = color_value(concat!(
        "color: color-mix(in oklch longer hue, ",
        "lab(calc(50% + 10%) 20 30) 25%, ",
        "rgb(from blue r g b / alpha) 75%)",
    ));
    let color_mix = value
        .value()
        .color_mix_value()
        .expect("checked current color-mix branch");
    assert_eq!(
        color_mix
            .interpolation()
            .unwrap()
            .predefined()
            .unwrap()
            .space(),
        CssColorInterpolationSpace::Oklch,
    );
    assert_eq!(
        color_mix
            .interpolation()
            .unwrap()
            .predefined()
            .unwrap()
            .hue(),
        Some(CssHueInterpolationMethod::Longer),
    );
    assert!(color_mix.components()[0].color().lab_value().is_some());
    assert_eq!(
        color_mix.components()[0]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "25"
    );
    assert!(color_mix.components()[1].color().relative_value().is_some());
    assert_eq!(
        color_mix.components()[1]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "75"
    );
}

#[test]
fn color_mix_preserved_subset_accepts_supported_spaces_and_polar_hue_methods() {
    for source in [
        "color: color-mix(in srgb, red, blue)",
        "color: color-mix(in srgb-linear, red 0%, blue 100%)",
        "color: color-mix(in display-p3, red, blue)",
        "color: color-mix(in a98-rgb, red, blue)",
        "color: color-mix(in prophoto-rgb, red, blue)",
        "color: color-mix(in rec2020, red, blue)",
        "color: color-mix(in xyz-d50, red, blue)",
        "color: color-mix(in xyz, red, blue)",
        "color: color-mix(in hsl shorter hue, red, blue)",
        "color: color-mix(in hwb increasing hue, red, blue)",
        "color: color-mix(in lab, red, blue)",
        "color: color-mix(in lch decreasing hue, red, blue)",
        "color: color-mix(in oklab, red, blue)",
        "color: color-mix(in oklch longer hue, red, blue)",
    ] {
        let value = color_value(source);
        assert!(value.value().color_mix_value().is_some(), "{source}");
    }

    let compatible = color_value("color: color-mix(in srgb, red 25%, blue)");
    assert_eq!(
        compatible.value().color_mix_value().unwrap().components()[0]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "25"
    );
}

#[test]
fn color_mix_rejects_malformed_components_and_invalid_nested_colors() {
    for invalid in [
        "color-mix(srgb, red, blue)",
        "color-mix(in srgb longer hue, red, blue)",
        "color-mix(in lab shorter hue, red, blue)",
        "color-mix(in srgb, red 101%, blue)",
        "color-mix(in srgb, red -1%, blue)",
        "color-mix(in srgb, red 25% 30%, blue)",
        "color-mix(in srgb, red,, blue)",
        "light-dark(red, blue, green)",
        "device-cmyk(0 0 1)",
    ] {
        let source = format!("color: {invalid}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            1,
            "{invalid}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Opacity),
            "{invalid}",
        );
    }
}

#[test]
fn color_mix_nesting_preserves_the_exact_parser_boundary() {
    for depth in [255_usize, 256] {
        let nested = format!(
            "{}red{}",
            "color-mix(in srgb, ".repeat(depth),
            ", blue)".repeat(depth),
        );
        let source = format!("color: {nested}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2, "depth {depth}");
    }

    let depth = 257_usize;
    let nested = format!(
        "{}red{}",
        "color-mix(in srgb, ".repeat(depth),
        ", blue)".repeat(depth),
    );
    let source = format!("color: {nested}; opacity: 0.5");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("depth {depth}: expected one diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        surgeist_css::CssErrorCode::NestingLimit,
    );
    assert_eq!(
        diagnostic.action(),
        surgeist_css::CssRecoveryAction::StopAtNestingLimit,
    );
}

#[test]
fn relative_color_families_expose_their_closed_current_environments_and_i01_projection() {
    for (source, expected_function, expected_environment) in [
        (
            "color: rgb(from red r g b / alpha)",
            CssRelativeColorFunction::Rgb,
            CssRelativeColorEnvironment::Rgb,
        ),
        (
            "color: rgba(from red r g b)",
            CssRelativeColorFunction::Rgb,
            CssRelativeColorEnvironment::Rgb,
        ),
        (
            "color: hsla(from red h s l / alpha)",
            CssRelativeColorFunction::Hsl,
            CssRelativeColorEnvironment::Hsl,
        ),
        (
            "color: hwb(from red h w b / alpha)",
            CssRelativeColorFunction::Hwb,
            CssRelativeColorEnvironment::Hwb,
        ),
        (
            "color: lab(from red l a b / alpha)",
            CssRelativeColorFunction::Lab,
            CssRelativeColorEnvironment::Lab,
        ),
        (
            "color: lch(from red l c h / alpha)",
            CssRelativeColorFunction::Lch,
            CssRelativeColorEnvironment::Lch,
        ),
        (
            "color: oklab(from red l a b / alpha)",
            CssRelativeColorFunction::Oklab,
            CssRelativeColorEnvironment::Oklab,
        ),
        (
            "color: oklch(from red l c h / alpha)",
            CssRelativeColorFunction::Oklch,
            CssRelativeColorEnvironment::Oklch,
        ),
        (
            "color: color(from red display-p3 r g b / alpha)",
            CssRelativeColorFunction::Color(CssPredefinedColorSpace::DisplayP3),
            CssRelativeColorEnvironment::PredefinedRgb(CssPredefinedColorSpace::DisplayP3),
        ),
        (
            "color: color(from red xyz x y z / alpha)",
            CssRelativeColorFunction::Color(CssPredefinedColorSpace::XyzD65),
            CssRelativeColorEnvironment::Xyz(CssPredefinedColorSpace::XyzD65),
        ),
    ] {
        let value = color_value(source);
        let relative = value
            .value()
            .relative_value()
            .expect("typed relative-color branch");
        assert_eq!(relative.function(), &expected_function, "{source}");
        assert_eq!(relative.environment(), expected_environment, "{source}");
        assert_eq!(relative.channels().len(), 3, "{source}");
    }
}

#[test]
fn relative_color_each_environment_accepts_its_channel_references_in_typed_math() {
    for source in [
        "color: rgb(from red calc(r + 1) calc(g + 1) calc(b + 1) / calc(alpha * 0.5))",
        "color: hsl(from red calc(h + 1) calc(s + 1) calc(l + 1) / calc(alpha + 0.1))",
        "color: hwb(from red calc(h + 1) calc(w + 1) calc(b + 1) / calc(alpha + 0.1))",
        "color: lab(from red calc(l + 1) calc(a + 1) calc(b + 1) / calc(alpha + 0.1))",
        "color: lch(from red calc(l + 1) calc(c + 1) calc(h + 1) / calc(alpha + 0.1))",
        "color: oklab(from red calc(l + 1) calc(a + 1) calc(b + 1) / calc(alpha + 0.1))",
        "color: oklch(from red calc(l + 1) calc(c + 1) calc(h + 1) / calc(alpha + 0.1))",
        "color: color(from red rec2020 calc(r + 1) calc(g + 1) calc(b + 1) / calc(alpha + 0.1))",
        "color: color(from red xyz-d50 calc(x + 1) calc(y + 1) calc(z + 1) / calc(alpha + 0.1))",
    ] {
        let value = color_value(source);
        assert!(value.value().relative_value().is_some(), "{source}");
    }
}

#[test]
fn relative_color_channels_reject_foreign_names_dimensions_and_malformed_grammar() {
    for invalid in [
        "rgb(from red h g b)",
        "rgb(from red r 1px b)",
        "rgb(from red r g b / 1deg)",
        "hsl(from red r s l)",
        "hsl(from red h 1deg l)",
        "hsl(from red 10% s l)",
        "hwb(from red h s b)",
        "lab(from red l c b)",
        "lch(from red l a h)",
        "oklab(from red l c b)",
        "oklch(from red l a h)",
        "color(from red srgb x g b)",
        "color(from red xyz r y z)",
        "rgb(from red calc(h + 1) g b)",
        "oklch(from red l c calc(h + 10%))",
        "rgb(from red r g)",
        "rgb(from red r g b extra)",
        "rgb(from red r g b /)",
        "rgb(from red r g b / alpha / alpha)",
        "alpha(from red r g b)",
    ] {
        let source = format!("color: {invalid}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            1,
            "{invalid}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Opacity),
            "{invalid}",
        );
    }
}

#[test]
fn relative_color_math_preserves_exact_values_without_float_range_evaluation() {
    use surgeist_css::{
        CssCalculationExpressionRef as Expr, CssCalculationProductOperator,
        CssCalculationSumOperator, CssCalculationType, CssRelativeColorExpressionValue,
        CssValueOrigin,
    };

    for (math, expected, product) in [
        ("calc(1e999 + 1)", ["1e999", "1"], false),
        ("calc(3e38 * 3e38)", ["3e38", "3e38"], true),
    ] {
        let source = format!("color: rgb(from red {math} g b); opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2);
        let value = color_value(&source);
        let relative = value.value().relative_value().unwrap();
        let CssRelativeColorExpressionValue::Calculation(calculation) =
            relative.channels()[0].value()
        else {
            panic!("expected exact relative-color calculation");
        };
        assert_eq!(calculation.result_type(), CssCalculationType::Number);
        assert!(calculation.references().is_empty());
        let Expr::NestedCalc(root) = calculation.expression() else {
            panic!("calc root")
        };
        let leaves = match root.operand() {
            Expr::Product(terms) if product => {
                assert_eq!(terms.len(), 2);
                assert_eq!(
                    terms.factor(1).unwrap().operator(),
                    Some(CssCalculationProductOperator::Multiply)
                );
                [
                    terms.factor(0).unwrap().expression(),
                    terms.factor(1).unwrap().expression(),
                ]
            }
            Expr::Sum(terms) if !product => {
                assert_eq!(terms.len(), 2);
                assert_eq!(
                    terms.term(1).unwrap().operator(),
                    Some(CssCalculationSumOperator::Add)
                );
                [
                    terms.term(0).unwrap().expression(),
                    terms.term(1).unwrap().expression(),
                ]
            }
            _ => panic!("expected authored operator"),
        };
        let CssValueOrigin::Parsed(root_origin) = calculation.expression().origin() else {
            panic!("parsed root")
        };
        assert_eq!(root_origin.source().as_str(), source);
        assert_eq!(
            root_origin.span().start().byte_offset().value(),
            source.find("calc(").unwrap()
        );
        for (leaf, spelling) in leaves.into_iter().zip(expected) {
            let Expr::Value(value) = leaf else {
                panic!("exact lexical leaf")
            };
            let literal = value.literal();
            assert_eq!(literal.representation(), spelling);
            let CssValueOrigin::Parsed(origin) = literal.origin() else {
                panic!("parsed leaf")
            };
            assert!(origin.source().same_snapshot(root_origin.source()));
            assert_eq!(
                &source[origin.span().start().byte_offset().value()
                    ..origin.span().end().byte_offset().value()],
                spelling
            );
        }
    }
}

#[test]
fn relative_color_origins_recurse_without_evaluation() {
    let value = color_value(concat!(
        "color: rgb(from oklch(from color(from red xyz x y z) l c h) ",
        "r g b / alpha)",
    ));
    let outer = value.value().relative_value().unwrap();
    let middle = outer.source().relative_value().unwrap();
    let inner = middle.source().relative_value().unwrap();
    assert_eq!(outer.environment(), CssRelativeColorEnvironment::Rgb);
    assert_eq!(middle.environment(), CssRelativeColorEnvironment::Oklch);
    assert_eq!(
        inner.environment(),
        CssRelativeColorEnvironment::Xyz(CssPredefinedColorSpace::XyzD65)
    );
}

#[test]
fn relative_color_nesting_preserves_the_exact_parser_boundary() {
    for depth in [255_usize, 256] {
        let nested = format!(
            "{}red{}",
            "rgb(from ".repeat(depth),
            " r g b)".repeat(depth),
        );
        let source = format!("color: {nested}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2);
    }

    {
        let depth = 257_usize;
        let nested = format!(
            "{}red{}",
            "rgb(from ".repeat(depth),
            " r g b)".repeat(depth),
        );
        let source = format!("color: {nested}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            surgeist_css::CssErrorCode::NestingLimit,
            "depth {depth}",
        );
        assert_eq!(
            diagnostic.action(),
            surgeist_css::CssRecoveryAction::StopAtNestingLimit,
            "depth {depth}",
        );
    }
}

#[test]
fn authored_keyword_and_hex_colors_preserve_their_current_branches() {
    let current = color_value("color: CurrentColor");
    assert!(current.value().is_current_color());

    let transparent = color_value("color: transparent");
    assert!(transparent.value().is_transparent());

    for (source, digits) in [
        ("color: #0f8", "0f8"),
        ("color: #0f8c", "0f8c"),
        ("color: #00ff88", "00ff88"),
        ("color: #00ff88cc", "00ff88cc"),
    ] {
        let value = color_value(source);
        assert_eq!(value.value().hex_value().unwrap().digits(), digits);
    }

    let named = color_value("color: ReBeccAPurple");
    assert_eq!(named.value().named().unwrap().name(), "rebeccapurple");
}

#[test]
fn authored_system_colors_distinguish_current_and_deprecated_sets() {
    let current = color_value("color: CanvasText");
    assert_eq!(current.value().system(), Some(CssSystemColor::CanvasText));

    let deprecated = color_value("color: ThreeDLightShadow");
    assert_eq!(
        deprecated.value().system(),
        Some(CssSystemColor::ThreeDLightShadow)
    );
}

#[test]
fn authored_rgb_keeps_legacy_and_modern_component_domains() {
    let legacy = color_value("color: rgba(300, -20, 40, 150%)");
    let rgb = legacy.value().rgb_value().unwrap();
    assert_eq!(rgb.syntax(), CssColorSyntax::Legacy);
    assert!(matches!(
        rgb.channels(),
        [
            CssColorComponent::Number(red),
            CssColorComponent::Number(green),
            CssColorComponent::Number(blue),
        ] if red.numeric().representation() == "300" && green.numeric().representation() == "-20" && blue.numeric().representation() == "40"
    ));
    assert!(matches!(
        rgb.alpha(),
        Some(CssColorComponent::Percentage(value)) if value.numeric().representation() == "150"
    ));

    let modern = color_value(concat!(
        "color: rgb(calc(1 + 2) calc(10% + 20%) none / ",
        "calc(50% + 10%))",
    ));
    let rgb = modern.value().rgb_value().unwrap();
    assert_eq!(rgb.syntax(), CssColorSyntax::Modern);
    assert!(matches!(
        rgb.channels(),
        [
            CssColorComponent::NumberCalculation(_),
            CssColorComponent::PercentageCalculation(_),
            CssColorComponent::None,
        ]
    ));
    assert!(matches!(
        rgb.alpha(),
        Some(CssColorComponent::PercentageCalculation(_))
    ));
}

#[test]
fn authored_hsl_and_hwb_keep_hue_and_percentage_domains() {
    let hsl = color_value("color: hsl(calc(1turn - 90deg) 120% -20% / none)");
    let hsl = hsl.value().hsl_value().unwrap();
    assert_eq!(hsl.syntax(), CssColorSyntax::Modern);
    assert!(matches!(hsl.hue(), CssColorHue::AngleCalculation(_)));
    assert!(matches!(
        hsl.saturation(),
        CssColorComponent::Percentage(value) if value.numeric().representation() == "120"
    ));
    assert!(matches!(hsl.alpha(), Some(CssColorComponent::None)));

    let legacy = color_value("color: hsla(-30, 120%, -10%, 2)");
    assert_eq!(
        legacy.value().hsl_value().unwrap().syntax(),
        CssColorSyntax::Legacy
    );

    let hwb = color_value("color: hwb(none calc(20% + 5%) 120% / -10%)");
    let hwb = hwb.value().hwb_value().unwrap();
    assert!(matches!(hwb.hue(), CssColorHue::None));
    assert!(matches!(
        hwb.whiteness(),
        CssColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(
        hwb.blackness(),
        CssColorComponent::Percentage(value) if value.numeric().representation() == "120"
    ));
}

#[test]
fn authored_perceptual_colors_preserve_channels_alpha_and_function_identity() {
    let lab = color_value("color: lab(calc(50% + 10%) calc(20 + 5) -30% / calc(120% - 5%))");
    let lab_value = lab.value().lab_value().expect("typed Lab branch");
    assert!(matches!(
        lab_value.lightness(),
        CssColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(
        lab_value.a(),
        CssColorComponent::NumberCalculation(_)
    ));
    assert!(matches!(
        lab_value.b(),
        CssColorComponent::Percentage(value)
            if value.numeric().representation() == "-30"
    ));
    assert!(matches!(
        lab_value.alpha(),
        Some(CssColorComponent::PercentageCalculation(_))
    ));

    let lch = color_value("color: lch(125% -20 calc(1turn - 90deg) / none)");
    let lch_value = lch.value().lch_value().expect("typed LCH branch");
    assert!(matches!(
        lch_value.lightness(),
        CssColorComponent::Percentage(value) if value.numeric().representation() == "125"
    ));
    assert!(matches!(
        lch_value.chroma(),
        CssColorComponent::Number(value) if value.numeric().representation() == "-20"
    ));
    assert!(matches!(lch_value.hue(), CssColorHue::AngleCalculation(_)));
    assert!(matches!(lch_value.alpha(), Some(CssColorComponent::None)));

    let oklab = color_value("color: oklab(none 150% -2 / 3)");
    let oklab_value = oklab.value().oklab_value().expect("typed Oklab branch");
    assert!(matches!(oklab_value.lightness(), CssColorComponent::None));
    assert!(matches!(
        oklab_value.a(),
        CssColorComponent::Percentage(value) if value.numeric().representation() == "150"
    ));
    assert!(matches!(
        oklab_value.b(),
        CssColorComponent::Number(value) if value.numeric().representation() == "-2"
    ));

    let oklch = color_value("color: oklch(-20 150% none)");
    let oklch_value = oklch.value().oklch_value().expect("typed Oklch branch");
    assert!(matches!(
        oklch_value.lightness(),
        CssColorComponent::Number(value) if value.numeric().representation() == "-20"
    ));
    assert!(matches!(
        oklch_value.chroma(),
        CssColorComponent::Percentage(value) if value.numeric().representation() == "150"
    ));
    assert!(matches!(oklch_value.hue(), CssColorHue::None));

    let compatible = color_value("color: lab(50% 20 30 / 50%)");
    assert!(matches!(compatible.value().lab_value().unwrap().alpha(),
        Some(CssColorComponent::Percentage(v)) if v.numeric().representation() == "50"));

    let out_of_range = color_value("color: lab(125% -20 30 / 150%)");
    assert!(matches!(out_of_range.value().lab_value().unwrap().alpha(),
        Some(CssColorComponent::Percentage(v)) if v.numeric().representation() == "150"));
}

#[test]
fn authored_predefined_colors_preserve_supported_space_and_channel_kinds() {
    for (name, expected) in [
        ("srgb", CssPredefinedColorSpace::Srgb),
        ("srgb-linear", CssPredefinedColorSpace::SrgbLinear),
        ("display-p3", CssPredefinedColorSpace::DisplayP3),
        (
            "display-p3-linear",
            CssPredefinedColorSpace::DisplayP3Linear,
        ),
        ("a98-rgb", CssPredefinedColorSpace::A98Rgb),
        ("prophoto-rgb", CssPredefinedColorSpace::ProphotoRgb),
        ("rec2020", CssPredefinedColorSpace::Rec2020),
        ("xyz", CssPredefinedColorSpace::XyzD65),
        ("xyz-d50", CssPredefinedColorSpace::XyzD50),
        ("xyz-d65", CssPredefinedColorSpace::XyzD65),
    ] {
        let value = color_value(&format!(
            "color: color({name} calc(1 + 2) 120% none / -25%)"
        ));
        let predefined = value
            .value()
            .predefined_value()
            .expect("typed predefined color branch");
        assert_eq!(predefined.color_space(), expected, "{name}");
        assert!(matches!(
            predefined.channels(),
            [
                CssColorComponent::NumberCalculation(_),
                CssColorComponent::Percentage(percentage),
                CssColorComponent::None,
        ] if percentage.numeric().representation() == "120"
        ));
        assert!(matches!(
            predefined.alpha(),
            Some(CssColorComponent::Percentage(value))
                if value.numeric().representation() == "-25"
        ));
    }
}

#[test]
fn display_p3_linear_color_preserves_exact_channels() {
    let value = color_value("color: color(display-p3-linear 1 0.5 0)");
    let current = value.value().predefined_value().unwrap();
    assert_eq!(
        current.color_space(),
        CssPredefinedColorSpace::DisplayP3Linear
    );
    for (channel, expected) in current.channels().iter().zip(["1", "0.5", "0"]) {
        assert!(
            matches!(channel, CssColorComponent::Number(value) if value.numeric().representation() == expected)
        );
    }
}

#[test]
fn out_of_range_predefined_alpha_has_no_lossy_compatibility_projection() {
    let value = color_value("color: color(srgb 1 0.5 0 / 150%)");
    assert!(value.value().predefined_value().is_some());
}

#[test]
fn predefined_inexact_literals_have_no_rounded_compatibility_projection() {
    let value = color_value("color: color(display-p3 0.8 0.2 0.1 / 90%)");
    assert_eq!(
        value.value().predefined_value().unwrap().color_space(),
        CssPredefinedColorSpace::DisplayP3,
    );
    for (channel, expected) in value
        .value()
        .predefined_value()
        .unwrap()
        .channels()
        .iter()
        .zip(["0.8", "0.2", "0.1"])
    {
        let CssColorComponent::Number(literal) = channel else {
            panic!("nonbinary32 authored coefficient must remain exact");
        };
        assert_eq!(literal.numeric().representation(), expected);
    }
}

#[test]
fn invalid_perceptual_and_predefined_color_forms_drop_only_the_declaration() {
    for invalid in [
        "lab(50% 20)",
        "lab(50%, 20, 30)",
        "lab(50% 20 30 40)",
        "lab(50% 1px 30)",
        "lch(50% 20 10%)",
        "lch(50% 20 10deg / 1 / 2)",
        "oklab(50% 20 30deg)",
        "oklch(50% 20 30deg, 1)",
        "color(srgb 1 2)",
        "color(srgb 1, 2, 3)",
        "color(srgb 1 2 3 4)",
        "color(srgb 1px 2 3)",
    ] {
        let source = format!("color: {invalid}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{invalid}");
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Opacity),
            "{invalid}",
        );
    }
}

#[test]
fn perceptual_color_calculations_preserve_the_exact_depth_boundary() {
    for depth in [254_usize, 255] {
        let source = format!(
            "color: oklab({}1{} 2 3); opacity: 0.5",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2);
    }

    for depth in [256_usize, 257] {
        let source = format!(
            "color: oklab({}1{} 2 3); opacity: 0.5",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source.match_indices("calc(").nth(255).unwrap().0;
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            surgeist_css::CssErrorCode::NestingLimit,
            "depth {depth}",
        );
        assert_eq!(
            diagnostic.action(),
            surgeist_css::CssRecoveryAction::StopAtNestingLimit,
            "depth {depth}",
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}",
        );
    }
}

#[test]
fn invalid_color_separators_units_and_arities_drop_only_the_declaration() {
    for invalid in [
        "rgb(1, 20%, 3)",
        "rgb(1, 2 3)",
        "rgb(1 2)",
        "rgb(1px 2 3)",
        "hsl(20, 30%, 40% / 50%)",
        "hwb(20, 30%, 40%)",
        "hwb(20deg 30% 40% 50%)",
    ] {
        let source = format!("color: {invalid}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{invalid}");
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Opacity),
            "{invalid}",
        );
    }
}

#[test]
fn authored_color_calculations_preserve_the_exact_depth_boundary() {
    for depth in [254_usize, 255] {
        let source = format!(
            "color: rgb({}1{} 2 3); opacity: 0.5",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2);
    }

    for depth in [256_usize, 257] {
        let source = format!(
            "color: rgb({}1{} 2 3); opacity: 0.5",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source.match_indices("calc(").nth(255).unwrap().0;
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            surgeist_css::CssErrorCode::NestingLimit,
            "depth {depth}",
        );
        assert_eq!(
            diagnostic.action(),
            surgeist_css::CssRecoveryAction::StopAtNestingLimit,
            "depth {depth}",
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}",
        );
    }
}

#[test]
fn opacity_preserves_finite_authored_number_and_percentage_branches() {
    let report = parse_style_attribute(concat!(
        "opacity: 0.5; ",
        "opacity: -0.5; ",
        "opacity: 1.5; ",
        "opacity: -25%; ",
        "opacity: 150%; ",
        "color: red",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 6);

    let mut opacity = report.syntax()[..5].iter().map(|declaration| {
        let CssKnownPropertyValueRef::Opacity(value) = declaration
            .known()
            .expect("known opacity declaration")
            .property_value()
            .expect("ordinary opacity value")
        else {
            panic!("expected opacity wrapper");
        };
        value
    });

    let value = opacity.next().unwrap();
    assert!(
        matches!(value.value(), CssOpacityValue::Scalar(value) if value.numeric().representation() == "0.5")
    );
    assert_eq!(value.value().serialize_specified().unwrap(), "0.5");

    for input in ["-0.5", "1.5"] {
        let value = opacity.next().unwrap();
        assert!(
            matches!(value.value(), CssOpacityValue::Scalar(value) if value.numeric().representation() == input)
        );
    }

    for input in ["-25%", "150%"] {
        let value = opacity.next().unwrap();
        assert!(
            matches!(value.value(), CssOpacityValue::Scalar(value) if value.numeric().representation() == input.trim_end_matches('%'))
        );
    }
}

#[test]
fn opacity_ordinary_global_and_substitution_values_remain_distinct() {
    let report =
        parse_style_attribute("opacity: 150%; opacity: inherit; opacity: var(--authored-opacity)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    assert!(matches!(
        report.syntax()[0]
            .known()
            .expect("ordinary opacity")
            .declared_value(),
        CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::Opacity(_))
    ));
    assert!(matches!(
        report.syntax()[1]
            .known()
            .expect("global opacity")
            .declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[2]
            .known()
            .expect("substitution-dependent opacity")
            .declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

#[test]
fn custom_profile_colors_are_retained_with_their_valid_sibling() {
    for color in ["color(--custom 1 2 3)", "color(from red --custom r g b)"] {
        let report = parse_style_attribute(&format!("color: {color}; opacity: 0.5"));
        assert!(report.is_clean(), "{color}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
    }
}
