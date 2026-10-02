#![forbid(unsafe_code)]
//! Intrinsic parsed alpha from Color 4 CRD 2026-09-08 §4.2, separately retaining
//! authored components and calculations under the selected family/role phases.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#alpha-syntax
//! SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! Expected classes derive from exact decimal comparison with 0, 1 or 100;
//! approximate observations never supply the endpoint-classification oracle.

use surgeist_css::*;

const FAMILIES: [&str; 8] = ["rgb", "hsl", "hwb", "lab", "lch", "oklab", "oklch", "srgb"];

#[derive(Clone, Copy, Debug)]
enum Class {
    Zero,
    One,
    Interior,
}

fn component(text: &str, programmatic: bool) -> CssColorComponent {
    if text == "none" {
        return CssColorComponent::None;
    }
    if text.starts_with("calc(") {
        let values = if programmatic {
            let argument = text
                .strip_prefix("calc(")
                .unwrap()
                .strip_suffix(')')
                .unwrap();
            CssComponentValues::try_new(vec![
                CssComponentValue::try_function(
                    "calc",
                    CssComponentValues::try_new(vec![
                        CssComponentValue::try_token(argument).unwrap(),
                    ])
                    .unwrap(),
                )
                .unwrap(),
            ])
            .unwrap()
        } else {
            parse_component_values(text).unwrap()
        };
        return if text.contains('%') {
            CssColorComponent::PercentageCalculation(
                CssPercentageCalculation::try_from_components(values).unwrap(),
            )
        } else {
            CssColorComponent::NumberCalculation(
                CssNumberCalculation::try_from_components(values).unwrap(),
            )
        };
    }
    let token = if programmatic {
        CssComponentValue::try_token(text).unwrap()
    } else {
        parse_component_values(text).unwrap().items()[0].clone()
    };
    if text.ends_with('%') {
        CssColorComponent::Percentage(CssColorPercentageLiteral::try_from_component(token).unwrap())
    } else {
        CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token).unwrap())
    }
}

fn constructed(family: &str, alpha: Option<&str>, programmatic: bool) -> CssColor {
    let number = || component("0", programmatic);
    let hue = || match number() {
        CssColorComponent::Number(value) => CssColorHue::Number(value),
        _ => unreachable!(),
    };
    let alpha = alpha.map(|text| component(text, programmatic));
    match family {
        "rgb" => CssColor::from_rgb(
            CssRgbColor::try_new(
                CssColorSyntax::Modern,
                [number(), number(), number()],
                alpha,
            )
            .unwrap(),
        ),
        "hsl" => CssColor::from_hsl(
            CssHslColor::try_new(CssColorSyntax::Modern, hue(), number(), number(), alpha).unwrap(),
        ),
        "hwb" => {
            CssColor::from_hwb(CssHwbColor::try_new(hue(), number(), number(), alpha).unwrap())
        }
        "lab" | "oklab" => {
            let value = CssLabColor::try_new(number(), number(), number(), alpha).unwrap();
            if family == "lab" {
                CssColor::from_lab(value)
            } else {
                CssColor::from_oklab(value)
            }
        }
        "lch" | "oklch" => {
            let value = CssLchColor::try_new(number(), number(), hue(), alpha).unwrap();
            if family == "lch" {
                CssColor::from_lch(value)
            } else {
                CssColor::from_oklch(value)
            }
        }
        "srgb" => CssColor::from_predefined(
            CssPredefinedColor::try_new(
                CssPredefinedColorSpace::Srgb,
                [number(), number(), number()],
                alpha,
            )
            .unwrap(),
        ),
        _ => panic!("test family"),
    }
}

fn spelling(family: &str, alpha: Option<&str>) -> String {
    let start = if family == "srgb" {
        "color(srgb".to_owned()
    } else {
        format!("{family}(")
    };
    format!(
        "{start}{}0 0 0{})",
        if family == "srgb" { " " } else { "" },
        alpha.map_or(String::new(), |a| format!(" / {a}"))
    )
}
fn declaration(family: &str, alpha: Option<&str>) -> CssDeclaration {
    let source = format!("color:{}", spelling(family, alpha));
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(validate_style_attribute(&source).unwrap(), *report.syntax());
    report.syntax()[0].clone()
}
fn color_of(declaration: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color payload")
    };
    value.value()
}
fn authored<'a>(color: &'a CssColor, family: &str) -> Option<&'a CssColorComponent> {
    match family {
        "rgb" => color.rgb_value().unwrap().alpha(),
        "hsl" => color.hsl_value().unwrap().alpha(),
        "hwb" => color.hwb_value().unwrap().alpha(),
        "lab" => color.lab_value().unwrap().alpha(),
        "oklab" => color.oklab_value().unwrap().alpha(),
        "lch" => color.lch_value().unwrap().alpha(),
        "oklch" => color.oklch_value().unwrap().alpha(),
        "srgb" => color.predefined_value().unwrap().alpha(),
        _ => unreachable!(),
    }
}
fn view<'a>(color: &'a CssColor, family: &str) -> CssParsedColorAlphaRef<'a> {
    match family {
        "rgb" => color.rgb_value().unwrap().parsed_alpha(),
        "hsl" => color.hsl_value().unwrap().parsed_alpha(),
        "hwb" => color.hwb_value().unwrap().parsed_alpha(),
        "lab" => color.lab_value().unwrap().parsed_alpha(),
        "oklab" => color.oklab_value().unwrap().parsed_alpha(),
        "lch" => color.lch_value().unwrap().parsed_alpha(),
        "oklch" => color.oklch_value().unwrap().parsed_alpha(),
        "srgb" => color.predefined_value().unwrap().parsed_alpha(),
        _ => unreachable!(),
    }
}
fn scalar<'a>(color: &'a CssColor, family: &str) -> CssColorAlphaScalarRef<'a> {
    let CssParsedColorAlphaRef::Scalar(value) = view(color, family) else {
        panic!("direct scalar")
    };
    value
}
fn assert_scalar(
    color: &CssColor,
    family: &str,
    spelling: &str,
    class: Class,
    approximation: Option<f64>,
    programmatic: bool,
) {
    let before = color.clone();
    let scalar = scalar(color, family);
    assert_eq!(
        scalar.is_zero(),
        matches!(class, Class::Zero),
        "{family}: {spelling}"
    );
    assert_eq!(
        scalar.is_one(),
        matches!(class, Class::One),
        "{family}: {spelling}"
    );
    assert_eq!(
        scalar.is_interior(),
        matches!(class, Class::Interior),
        "{family}: {spelling}"
    );
    assert!(std::ptr::eq(
        scalar.authored_component(),
        authored(color, family).unwrap()
    ));
    let (numeric, origin) = match scalar.authored_component() {
        CssColorComponent::Number(value) => {
            assert!(!spelling.ends_with('%'));
            (value.numeric(), value.origin())
        }
        CssColorComponent::Percentage(value) => {
            assert!(spelling.ends_with('%'));
            (value.numeric(), value.origin())
        }
        _ => panic!("direct authored domain"),
    };
    assert_eq!(
        numeric.representation(),
        spelling.strip_suffix('%').unwrap_or(spelling)
    );
    assert!(std::ptr::eq(scalar.origin(), origin));
    assert_eq!(matches!(origin, CssValueOrigin::Programmatic), programmatic);
    if let CssValueOrigin::Parsed(origin) = origin {
        let start = origin.span().start().byte_offset().value();
        assert_eq!(
            &origin.source().as_str()[start..start + spelling.len()],
            spelling
        );
    }
    let observed = scalar.as_unit_f64();
    assert!(observed.is_finite() && (0.0..=1.0).contains(&observed));
    if let Some(expected) = approximation {
        assert_eq!(
            observed.to_bits(),
            expected.to_bits(),
            "{family}: {spelling}"
        );
    }
    assert_eq!(color, &before);
}
fn both_scalar_front_doors(family: &str, text: &str, class: Class, approximation: Option<f64>) {
    let parsed = declaration(family, Some(text));
    assert_scalar(color_of(&parsed), family, text, class, approximation, false);
    assert_scalar(
        &constructed(family, Some(text), true),
        family,
        text,
        class,
        approximation,
        true,
    );
}

#[test]
fn direct_number_and_percentage_alpha_classify_exact_endpoints_in_every_payload() {
    for family in FAMILIES {
        for (text, class, approximation) in [
            ("-2", Class::Zero, 0.0),
            ("-25%", Class::Zero, 0.0),
            ("0", Class::Zero, 0.0),
            ("-0", Class::Zero, 0.0),
            ("-0%", Class::Zero, 0.0),
            (".25", Class::Interior, 0.25),
            ("25%", Class::Interior, 0.25),
            ("1", Class::One, 1.0),
            ("100%", Class::One, 1.0),
            ("2", Class::One, 1.0),
            ("150%", Class::One, 1.0),
        ] {
            both_scalar_front_doors(family, text, class, Some(approximation));
        }
    }
}

#[test]
fn exact_endpoint_neighbors_remain_interior_even_when_f64_rounds_to_one_or_zero() {
    for family in FAMILIES {
        for (text, class, approximation) in [
            (
                "0.99999999999999999999999999999999999999",
                Class::Interior,
                1.0,
            ),
            (
                "99.999999999999999999999999999999999999%",
                Class::Interior,
                1.0,
            ),
            ("1.00000000000000000000000000000000000001", Class::One, 1.0),
            ("100.000000000000000000000000000000000001%", Class::One, 1.0),
            ("1e-400", Class::Interior, 0.0),
            ("1e-400%", Class::Interior, 0.0),
            ("-1e-400", Class::Zero, 0.0),
            ("-1e-400%", Class::Zero, 0.0),
        ] {
            both_scalar_front_doors(family, text, class, Some(approximation));
        }
    }
}

#[test]
fn arbitrary_exponents_and_redundant_digits_do_not_replace_exact_classification() {
    let exponent = "999999999999999999999999999999999999999999999999999999999999";
    let long_one = format!("1{}e-600", "0".repeat(600));
    let long_hundred = format!("1{}e-598%", "0".repeat(600));
    let long_interior = format!("{}e-600", "9".repeat(600));
    let long_percent_interior = format!("{}e-598%", "9".repeat(600));
    for family in FAMILIES {
        for (text, class, approximation) in [
            (format!("1e{exponent}"), Class::One, 1.0),
            (format!("1e{exponent}%"), Class::One, 1.0),
            (format!("1e-{exponent}"), Class::Interior, 0.0),
            (format!("1e-{exponent}%"), Class::Interior, 0.0),
            (format!("-1e{exponent}"), Class::Zero, 0.0),
            (format!("-0e{exponent}%"), Class::Zero, 0.0),
            (long_one.clone(), Class::One, 1.0),
            (long_hundred.clone(), Class::One, 1.0),
            (long_interior.clone(), Class::Interior, 1.0),
            (long_percent_interior.clone(), Class::Interior, 1.0),
            ("+00025.000e+0000%".into(), Class::Interior, 0.25),
        ] {
            both_scalar_front_doors(family, &text, class, Some(approximation));
        }
    }
}

#[test]
fn equivalent_unit_alphas_share_observation_without_changing_authored_domains() {
    for family in FAMILIES {
        let number = constructed(family, Some(".125"), true);
        let percentage = constructed(family, Some("12.5%"), true);
        let a = scalar(&number, family);
        let b = scalar(&percentage, family);
        assert!(a.is_interior() && b.is_interior());
        assert_eq!(a.as_unit_f64(), 0.125);
        assert_eq!(b.as_unit_f64(), 0.125);
        assert!(matches!(
            a.authored_component(),
            CssColorComponent::Number(_)
        ));
        assert!(matches!(
            b.authored_component(),
            CssColorComponent::Percentage(_)
        ));
    }
}

#[test]
fn omitted_missing_and_calculated_alphas_keep_distinct_unresolved_branches() {
    for family in FAMILIES {
        for programmatic in [false, true] {
            for text in [
                None,
                Some("none"),
                Some("calc(-2)"),
                Some("calc(1)"),
                Some("calc(2)"),
                Some("calc(infinity)"),
                Some("calc(-25%)"),
                Some("calc(100%)"),
                Some("calc(200%)"),
            ] {
                let owner = if programmatic {
                    constructed(family, text, true)
                } else {
                    color_of(&declaration(family, text)).clone()
                };
                let before = owner.clone();
                match (view(&owner, family), authored(&owner, family), text) {
                    (CssParsedColorAlphaRef::Omitted, None, None) => {}
                    (
                        CssParsedColorAlphaRef::Missing,
                        Some(CssColorComponent::None),
                        Some("none"),
                    ) => {}
                    (
                        CssParsedColorAlphaRef::NumberCalculation(view),
                        Some(CssColorComponent::NumberCalculation(raw)),
                        Some(text),
                    ) => {
                        assert!(std::ptr::eq(view, raw));
                        assert_eq!(view.components().serialize().unwrap().as_css(), text);
                        assert_eq!(view.result_type(), CssCalculationType::Number);
                        assert_eq!(
                            matches!(view.origin(), CssValueOrigin::Programmatic),
                            programmatic
                        );
                    }
                    (
                        CssParsedColorAlphaRef::PercentageCalculation(view),
                        Some(CssColorComponent::PercentageCalculation(raw)),
                        Some(text),
                    ) => {
                        assert!(std::ptr::eq(view, raw));
                        assert_eq!(view.components().serialize().unwrap().as_css(), text);
                        assert_eq!(view.result_type(), CssCalculationType::Percentage);
                        assert_eq!(
                            matches!(view.origin(), CssValueOrigin::Programmatic),
                            programmatic
                        );
                    }
                    other => panic!("parsed alpha branch: {other:?}"),
                }
                assert_eq!(owner, before);
            }
        }
    }
}

#[test]
fn legacy_rgb_and_hsl_payloads_expose_the_same_intrinsic_direct_alpha() {
    for (source, checked, family, text) in [
        (
            "rgb(0,0,0,2)",
            CssColor::from_rgb(
                CssRgbColor::try_new(
                    CssColorSyntax::Legacy,
                    [
                        component("0", true),
                        component("0", true),
                        component("0", true),
                    ],
                    Some(component("2", true)),
                )
                .unwrap(),
            ),
            "rgb",
            "2",
        ),
        (
            "hsl(0,0%,0%,150%)",
            CssColor::from_hsl(
                CssHslColor::try_new(
                    CssColorSyntax::Legacy,
                    CssColorHue::Number(
                        CssColorNumberLiteral::try_from_component(
                            CssComponentValue::try_number("0").unwrap(),
                        )
                        .unwrap(),
                    ),
                    component("0%", true),
                    component("0%", true),
                    Some(component("150%", true)),
                )
                .unwrap(),
            ),
            "hsl",
            "150%",
        ),
    ] {
        let report = parse_style_attribute(&format!("color:{source}"));
        assert!(report.is_clean());
        assert_scalar(
            color_of(&report.syntax()[0]),
            family,
            text,
            Class::One,
            Some(1.0),
            false,
        );
        assert_scalar(&checked, family, text, Class::One, Some(1.0), true);
    }
}

fn contribution_color(contribution: &CssLonghandContribution) -> &CssColor {
    let CssLonghandValueRef::Color(color) = contribution.ordinary_value().unwrap().view() else {
        panic!("color contribution")
    };
    color
}
fn assert_expansion(expansion: &CssExpansion, family: &str, text: &str, source: &CssDeclaration) {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = expansion else {
        panic!("terminal color")
    };
    assert_eq!(values.items().len(), 1);
    assert!(values.items()[0].source().same_occurrence(source));
    let color = contribution_color(&values.items()[0]);
    assert_scalar(color, family, text, Class::One, Some(1.0), false);
    assert_eq!(color, color_of(source));
}

#[test]
fn expanded_and_normalized_transport_preserves_raw_alpha_and_exact_parsed_view() {
    for family in FAMILIES {
        let declaration = declaration(family, Some("150%"));
        assert_expansion(
            &expand_declaration(&declaration).unwrap(),
            family,
            "150%",
            &declaration,
        );
        let source = format!(".a{{color:{}}}", spelling(family, Some("150%")));
        let report = parse_sheet(&source);
        assert!(report.is_clean());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let items: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(items.len(), 1);
        assert_expansion(items[0].expansion(), family, "150%", items[0].source());
    }
}

#[test]
fn reading_parsed_alpha_keeps_existing_specified_text_and_raw_graph_unchanged() {
    for (family, text, expected) in [
        ("rgb", "150%", "rgb(0, 0, 0)"),
        ("hsl", "-2", "rgba(0, 0, 0, 0)"),
        ("hwb", "25%", "rgba(255, 0, 0, 0.25)"),
        ("lab", "150%", "lab(0 0 0)"),
        ("lch", "-2", "lch(0 0 0 / 0)"),
        ("oklab", "25%", "oklab(0 0 0 / 0.25)"),
        ("oklch", "150%", "oklch(0 0 0)"),
        ("srgb", "-2", "color(srgb 0 0 0 / 0)"),
    ] {
        let owner = color_of(&declaration(family, Some(text))).clone();
        let before = owner.clone();
        let output = owner.to_specified_css().unwrap();
        assert_eq!(output, expected);
        let _ = view(&owner, family);
        assert_eq!(owner.to_specified_css().unwrap(), output);
        assert_eq!(owner, before);
    }
}
