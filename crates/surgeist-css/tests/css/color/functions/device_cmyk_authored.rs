#![forbid(unsafe_code)]
//! Authored grammar: Color 5 WD 2026-09-08 §6 (#device-cmyk).
//! Direct alpha: Color 4 CRD 2026-09-08 §4.2 (#alpha-syntax).
//! Declared text uses the selected ordinary Standalone/Mix/Origin contract:
//! ink percentages scale exactly by 1/100 outside Origin, ink stays unbounded,
//! calculations stay symbolic, and ordinary direct alpha clamps and rounds.
//! This evidence makes no computed conversion, profile, or CSSOM claim.
//! Resource oracles use the shared cumulative specified-value contract. Missing
//! components need one input and projection each, without scalar arithmetic.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration: {css}")
    };
    value.clone()
}
fn color(css: &str) -> CssColor {
    let source = declaration(&format!("color:{css}"));
    let CssKnownPropertyValueRef::Color(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color")
    };
    value.value().clone()
}
fn images(css: &str) -> CssImageValueList {
    let source = declaration(&format!("background-image:{css}"));
    let CssKnownPropertyValueRef::BackgroundImage(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed images")
    };
    value.images().clone()
}
fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed expansion")
    };
    for value in values.items() {
        assert!(value.source().same_occurrence(source));
        assert_eq!(value.source().importance(), source.importance());
    }
    values
}
fn assert_text(source: &str, expected: &str) {
    let supplied = parse_component_values(source).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        supplied.clone(),
        CssImportance::Important,
    );
    // Exercise both admission routes before either assertion can fail at RED.
    let source_attribute = format!("color:{source}!important;opacity:.5");
    let report = parse_style_attribute(&source_attribute);
    let checked = checked.unwrap_or_else(|error| {
        panic!(
            "checked admission {source}: {error:?}; parsed diagnostics: {:?}",
            report.diagnostics()
        )
    });
    assert_eq!(checked.value_components(), &supplied);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&source_attribute).is_ok());
    assert_eq!(report.syntax().len(), 2);
    let parsed = &report.syntax()[0];
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(parsed.importance(), CssImportance::Important);
    assert_eq!(
        parsed.value_components().serialize().unwrap().as_css(),
        source
    );
    for value in [&checked, parsed] {
        let before = value.clone();
        let CssKnownPropertyValueRef::Color(payload) =
            value.known().unwrap().property_value().unwrap()
        else {
            panic!("typed color")
        };
        for _ in 0..3 {
            assert_eq!(
                payload.value().to_specified_css().unwrap(),
                expected,
                "{source}"
            );
        }
        assert_eq!(value, &before);
    }
    assert_eq!(color(expected).to_specified_css().unwrap(), expected);
}
macro_rules! text_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_text($source, $expected);
        }
    };
}

text_case!(
    legacy_four_numbers_have_modern_declared_text,
    "DeViCe-CmYk(0, .81, .81, .3)",
    "device-cmyk(0 0.81 0.81 0.3)"
);
text_case!(
    modern_four_percentages_scale_exactly,
    "device-cmyk(0% 81% 81% 30%)",
    "device-cmyk(0 0.81 0.81 0.3)"
);
text_case!(
    mixed_missing_and_unbounded_channels_are_retained,
    "device-cmyk(-.2 140% none 2)",
    "device-cmyk(-0.2 1.4 none 2)"
);
text_case!(
    missing_alpha_is_distinct_from_omission,
    "device-cmyk(none none none none / none)",
    "device-cmyk(none none none none / none)"
);
text_case!(
    ordinary_direct_alpha_clamps_high_and_omits_unity,
    "device-cmyk(0 0 0 1 / 150%)",
    "device-cmyk(0 0 0 1)"
);
text_case!(
    ordinary_direct_alpha_clamps_negative,
    "device-cmyk(0 0 0 1 / -.2)",
    "device-cmyk(0 0 0 1 / 0)"
);
text_case!(
    ordinary_direct_alpha_uses_six_places,
    "device-cmyk(0 0 0 1 / .123456789)",
    "device-cmyk(0 0 0 1 / 0.123457)"
);
text_case!(
    calculated_legacy_numbers_remain_symbolic,
    "device-cmyk(calc(1 / 2), 0, 0, 1)",
    "device-cmyk(calc(0.5) 0 0 1)"
);
text_case!(
    calculated_modern_percentages_remain_scaled_symbolic,
    "device-cmyk(calc(50%) 0 none 1 / calc(25%))",
    "device-cmyk(calc(0.5) 0 none 1 / calc(0.25))"
);
text_case!(
    contextual_number_calculation_remains_unresolved,
    "device-cmyk(calc(1em / 1px) 0 0 1)",
    "device-cmyk(calc(1 * 1em / 1px) 0 0 1)"
);
text_case!(
    exact_small_number_keeps_more_than_six_decimal_places,
    "device-cmyk(1e-20 0 0 1)",
    "device-cmyk(0.00000000000000000001 0 0 1)"
);
text_case!(
    exact_percentage_keeps_decimal_tail,
    "device-cmyk(100.000000000000000000001% 0 0 1)",
    "device-cmyk(1.00000000000000000000001 0 0 1)"
);
text_case!(
    origin_keeps_direct_domains_and_explicit_alpha,
    "alpha(from device-cmyk(-20% 140% none 2 / 100%))",
    "alpha(from device-cmyk(-20% 140% none 2 / 100%))"
);
text_case!(
    relative_origin_keeps_direct_percentage_domains,
    "rgb(from device-cmyk(0 81% 81% 30% / 100%) r g b)",
    "rgb(from device-cmyk(0 81% 81% 30% / 100%) r g b)"
);
text_case!(
    mix_child_uses_declared_number_domains,
    "color-mix(device-cmyk(0 81% 81% 30%), blue)",
    "color-mix(device-cmyk(0 0.81 0.81 0.3), blue)"
);
text_case!(
    contrast_child_uses_standalone_inside_origin,
    "alpha(from contrast-color(device-cmyk(0 81% 81% 30%)))",
    "alpha(from contrast-color(device-cmyk(0 0.81 0.81 0.3)))"
);
text_case!(
    light_dark_children_use_standalone_inside_origin,
    "alpha(from light-dark(device-cmyk(0 81% 81% 30%), blue))",
    "alpha(from light-dark(device-cmyk(0 0.81 0.81 0.3), blue))"
);

#[test]
fn device_identity_and_contextual_exclusion_survive_ancestors() {
    for source in [
        "device-cmyk(0 0 0 1)",
        "alpha(from device-cmyk(0 0 0 1))",
        "rgb(from device-cmyk(0 0 0 1) r g b)",
        "color-mix(red, device-cmyk(0 0 0 1))",
    ] {
        let value = color(source);
        assert_eq!(
            value.absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::DeviceCmyk)
        );
        assert_ne!(value, color("black"));
        assert_ne!(value, color("white"));
    }
    assert_eq!(color("device-cmyk(0 0 0 1)").kind_name(), "device-cmyk");
    assert_ne!(color("device-cmyk(0 0 0 1)"), color("device-cmyk(1 0 0 0)"));
    for (source, reason) in [
        (
            "contrast-color(device-cmyk(0 0 0 1))",
            CssAbsoluteColorExclusion::ContrastColor,
        ),
        (
            "light-dark(device-cmyk(0 0 0 1), blue)",
            CssAbsoluteColorExclusion::LightDark,
        ),
    ] {
        assert_eq!(
            color(source).absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(reason)
        );
    }
}

#[test]
fn admitted_device_colors_are_rejected_by_absolute_only_palette_boundaries() {
    let index =
        CssFontPaletteIndex::try_new(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(0)))
            .unwrap();
    for source in [
        "device-cmyk(0 0 0 1)",
        "alpha(from device-cmyk(0 0 0 1))",
        "color-mix(red, device-cmyk(0 0 0 1))",
    ] {
        assert_eq!(
            CssFontPaletteOverride::try_new(index.clone(), color(source)).unwrap_err(),
            CssFontPaletteConstructionError::ContextualColor(CssAbsoluteColorExclusion::DeviceCmyk)
        );
        let report = parse_sheet(&format!(
            "@font-palette-values --p {{ font-family: Demo; override-colors: 0 {source}; base-palette: light; }}"
        ));
        let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
            panic!("retained palette")
        };
        assert_eq!(rule.descriptors().len(), 2);
        let [diagnostic] = report.diagnostics() else {
            panic!("one palette diagnostic")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
    }
}

#[test]
fn malformed_grammar_drops_only_its_declaration_and_validation_rejects_it() {
    for invalid in [
        "device-cmyk()",
        "device-cmyk(0 0 1)",
        "device-cmyk(0 0 0 0 1)",
        "device-cmyk(0, 0, 1)",
        "device-cmyk(0, 0, 0, 0, 1)",
        "device-cmyk(0%, 0, 0, 1)",
        "device-cmyk(none, 0, 0, 1)",
        "device-cmyk(calc(10%), 0, 0, 1)",
        "device-cmyk(0, 0, 0, 1 / .5)",
        "device-cmyk(0 0, 0 1)",
        "device-cmyk(0, 0 0, 1)",
        "device-cmyk(0, 0, 0, 1,)",
        "device-cmyk(0 0 0 1 /)",
        "device-cmyk(0 0 0 1 / .5 .2)",
        "device-cmyk(0 0 0 1 / .5 / .2)",
        "device-cmyk(0 0 0 1, red)",
        "device-cmyk(0, 0, 0, 1, red)",
        "device-cmyk(0 0 0 1 red)",
        "device-cmyk(initial 0 0 1)",
        "device-cmyk(0 inherit 0 1)",
        "device-cmyk(0 0 unset 1)",
        "device-cmyk(0 0 0 revert)",
        "device-cmyk(0 0 0 1 / revert-layer)",
        "device-cmyk(calc(1px) 0 0 1)",
        "device-cmyk(calc(1 + 10%) 0 0 1)",
        "device-cmyk(0 0 0 1 / calc(1deg))",
    ] {
        let source = format!("color:{invalid};opacity:.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic: {source}")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidColorSyntax,
            "{source}"
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(validate_style_attribute(&source).is_err());
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Color),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

#[test]
fn unrelated_existing_color_controls_remain_clean_and_typed() {
    for (source, expected) in [
        (
            "color(display-p3 -20% 120% 50%)",
            "color(display-p3 -0.2 1.2 0.5)",
        ),
        (
            "alpha(from color(--P 50% 0% / 100%))",
            "alpha(from color(--P 50% 0% / 100%))",
        ),
        ("lab(calc(125) 0 0 / none)", "lab(calc(125) 0 0 / none)"),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn noncolor_properties_reject_device_color_and_retain_siblings() {
    for property in [
        "background-image",
        "mask-image",
        "border-image-source",
        "list-style-image",
        "opacity",
    ] {
        let report = parse_style_attribute(&format!("{property}:device-cmyk(0 0 0 1);color:green"));
        assert_eq!(report.syntax().len(), 1, "{property}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
    }
}

#[test]
fn registered_color_shorthand_gradient_shadow_and_filter_consumers_admit_device_color() {
    for property in [
        "color",
        "background-color",
        "border-color",
        "border-top-color",
        "outline-color",
        "text-decoration-color",
        "column-rule-color",
    ] {
        let css = format!("{property}:device-cmyk(0 0 0 1)!important");
        let source = declaration(&css);
        assert_eq!(source.importance(), CssImportance::Important);
        assert!(source.known().unwrap().property_value().is_some());
        assert!(validate_style_attribute(&css).is_ok());
    }
    for css in [
        "border:1px solid device-cmyk(0 0 0 1)",
        "box-shadow:0 0 device-cmyk(0 0 0 1)",
        "filter:drop-shadow(0 0 device-cmyk(0 0 0 1))",
        "background-image:linear-gradient(device-cmyk(0 0 0 1), blue)",
        "mask-image:radial-gradient(device-cmyk(0 0 0 1), blue)",
    ] {
        assert!(
            declaration(css).known().unwrap().property_value().is_some(),
            "{css}"
        );
        assert!(validate_style_attribute(css).is_ok(), "{css}");
    }
}

#[test]
fn background_color_alternative_expands_with_image_and_important_occurrence() {
    let source =
        declaration("background:light-dark(none, url(a.svg)) device-cmyk(0 81% 81% 30%)!important");
    let values = expanded(&source);
    let image = values
        .items()
        .iter()
        .find(|v| v.property() == CssKnownProperty::BackgroundImage)
        .unwrap();
    let CssLonghandValueRef::BackgroundImage(image) = image.ordinary_value().unwrap().view() else {
        panic!("image")
    };
    assert_eq!(
        image.serialize_specified().unwrap(),
        "light-dark(none, url(\"a.svg\"))"
    );
    let value = values
        .items()
        .iter()
        .find(|v| v.property() == CssKnownProperty::BackgroundColor)
        .unwrap();
    let CssLonghandValueRef::BackgroundColor(value) = value.ordinary_value().unwrap().view() else {
        panic!("color")
    };
    assert_eq!(
        value.to_specified_css().unwrap(),
        "device-cmyk(0 0.81 0.81 0.3)"
    );
    let source = declaration("background:device-cmyk(0 0 0 1)");
    let CssKnownPropertyValueRef::Background(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("background")
    };
    assert!(value.background().layers()[0].image().is_none());
    assert_eq!(
        value.background().layers()[0].color().unwrap().kind_name(),
        "device-cmyk"
    );
}

#[test]
fn duplicate_or_nonfinal_background_device_colors_drop_only_background() {
    for invalid in [
        "device-cmyk(0 0 0 1), none",
        "device-cmyk(0 0 0 1) blue",
        "device-cmyk(0 0 0 1) device-cmyk(0 0 1 0)",
    ] {
        let report = parse_style_attribute(&format!("background:{invalid};color:green"));
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
    }
}
#[test]
fn parsed_programmatic_and_mixed_components_keep_origins_importance_and_raw_identity() {
    let parsed = declaration("color:DeViCe-CmYk(0 0 0 1)!important");
    assert!(matches!(
        parsed.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let mixed_child = parse_component_values("0 0 0 1").unwrap();
    let mixed_origin = mixed_child.items()[0].origin().clone();
    for child in [
        CssComponentValues::try_new(vec![
            CssComponentValue::try_number("0").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            CssComponentValue::try_number("0").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            CssComponentValue::try_number("0").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            CssComponentValue::try_number("1").unwrap(),
        ])
        .unwrap(),
        mixed_child,
    ] {
        let graph = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("device-cmyk", child.clone()).unwrap(),
        ])
        .unwrap();
        let source = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            graph.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert!(source.position().is_none());
        assert_eq!(source.value_components(), &graph);
        assert_eq!(
            source.value_components().items()[0].origin(),
            &CssValueOrigin::Programmatic
        );
        let CssComponentValueRef::Function(function) = source.value_components().items()[0].view()
        else {
            panic!("retained function")
        };
        assert_eq!(function.values(), &child);
        if matches!(child.items()[0].origin(), CssValueOrigin::Parsed(_)) {
            assert_eq!(child.items()[0].origin(), &mixed_origin);
        }
        let before = source.clone();
        let values = expanded(&source);
        let CssLonghandValueRef::Color(value) = values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("color contribution")
        };
        assert_eq!(value.to_specified_css().unwrap(), "device-cmyk(0 0 0 1)");
        assert_eq!(source, before);
        assert_eq!(source.importance(), CssImportance::Important);
    }
    let before = parsed.clone();
    assert_eq!(expanded(&parsed).items().len(), 1);
    assert_eq!(parsed, before);
}

#[test]
fn pending_reentry_is_strict_repeatable_and_preserves_replacement_provenance() {
    for (property, replacement) in [
        ("color", "device-cmyk(0 0 0 1)"),
        ("background-color", "device-cmyk(none 0 0 1)"),
        (
            "background",
            "light-dark(none, url(a.svg)) device-cmyk(0 0 0 1)",
        ),
        (
            "background-image",
            "linear-gradient(device-cmyk(0 0 0 1), blue)",
        ),
    ] {
        let source = declaration(&format!("{property}:var(--choice)!important"));
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for invalid in ["device-cmyk()", "device-cmyk(0, 0, 0)", "device-cmyk(none)"] {
            assert!(matches!(
                pending
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        assert_eq!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values(replacement).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed replacement")
            };
            assert!(!values.items().is_empty());
            for value in values.items() {
                assert!(value.source().same_occurrence(&source));
                assert_eq!(value.source().importance(), CssImportance::Important);
                assert_eq!(value.replacement_components(), Some(&replacement));
                assert!(value.ordinary_value().is_some());
            }
        }
    }
}

#[test]
fn normalization_preserves_device_cmyk_order_values_and_source_occurrences() {
    let report = parse_sheet(
        ".a{color:device-cmyk(0 0 0 1)!important;background-color:device-cmyk(0 0 1 0);background-image:linear-gradient(device-cmyk(0 1 0 0), blue)}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|value| match value {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 3);
    for (index, expected) in [
        "device-cmyk(0 0 0 1)",
        "device-cmyk(0 0 1 0)",
        "linear-gradient(device-cmyk(0 1 0 0), blue)",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(values[index].order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            values[index].expansion()
        else {
            panic!("normalized terminal")
        };
        assert_eq!(items.items().len(), 1);
        assert!(
            items.items()[0]
                .source()
                .same_occurrence(values[index].source())
        );
        let output = match items.items()[0].ordinary_value().unwrap().view() {
            CssLonghandValueRef::Color(value) | CssLonghandValueRef::BackgroundColor(value) => {
                value.to_specified_css().unwrap()
            }
            CssLonghandValueRef::BackgroundImage(value) => value.serialize_specified().unwrap(),
            _ => panic!("selected normalized value"),
        };
        assert_eq!(output, expected);
    }
    assert_eq!(values[0].source().importance(), CssImportance::Important);
}

fn assert_limit_error(value: &CssColor, limits: Limits, expected: Kind) {
    let before = value.clone();
    for _ in 0..3 {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
        assert_eq!(value, &before);
    }
}

#[test]
fn all_four_missing_channels_and_alpha_share_exact_logical_budgets_with_wrappers() {
    // One color root plus one visit/projection per missing component. A
    // LightDark or Contrast root adds one. No direct scalar arithmetic occurs.
    for (source, expected, nodes) in [
        (
            "device-cmyk(none none none none)",
            "device-cmyk(none none none none)",
            5,
        ),
        (
            "device-cmyk(none none none none / none)",
            "device-cmyk(none none none none / none)",
            6,
        ),
        (
            "contrast-color(device-cmyk(none none none none / none))",
            "contrast-color(device-cmyk(none none none none / none))",
            7,
        ),
        (
            "light-dark(device-cmyk(none none none none / none), device-cmyk(none none none none / none))",
            "light-dark(device-cmyk(none none none none / none), device-cmyk(none none none none / none))",
            13,
        ),
    ] {
        let value = color(source);
        let before = value.clone();
        for _ in 0..3 {
            assert_eq!(
                value
                    .to_specified_css_with_limits(Limits::new(nodes, nodes, expected.len()))
                    .unwrap(),
                expected
            );
            assert_eq!(value, before);
        }
        assert_limit_error(
            &value,
            Limits::new(nodes - 1, nodes, expected.len()),
            Kind::InputNodeLimit,
        );
        assert_limit_error(
            &value,
            Limits::new(nodes, nodes - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        );
        assert_limit_error(
            &value,
            Limits::new(nodes, nodes, expected.len() - 1),
            Kind::ByteLimit,
        );
    }
}

#[test]
fn scalar_channels_and_omitted_mix_weights_exhaust_the_same_typed_resources() {
    for source in [
        "device-cmyk(.1 20% .3 .4 / .5)",
        "color-mix(device-cmyk(none none none none / none), blue)",
    ] {
        let value = color(source);
        let before = value.clone();
        assert_limit_error(
            &value,
            Limits::new(1, usize::MAX, usize::MAX),
            Kind::InputNodeLimit,
        );
        assert_limit_error(
            &value,
            Limits::new(usize::MAX, 1, usize::MAX),
            Kind::ProjectionNodeLimit,
        );
        assert_eq!(value, before);
    }
    // Omitted Mix weights perform additional exact arithmetic. Eight input
    // visits cannot be treated as an eight-node projection allowance.
    let value = color("color-mix(device-cmyk(none none none none / none), blue)");
    let expected = "color-mix(device-cmyk(none none none none / none), blue)";
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(8, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
    assert_limit_error(
        &value,
        Limits::new(7, usize::MAX, expected.len()),
        Kind::InputNodeLimit,
    );
    assert_limit_error(
        &value,
        Limits::new(8, 8, expected.len()),
        Kind::ProjectionNodeLimit,
    );
}

#[test]
fn utf8_sibling_and_caller_prefix_share_exact_bytes_with_device_alpha() {
    let pair = CssBorderColorPair::new(
        color("color(--é none)"),
        Some(color("device-cmyk(none none none none / none)")),
    );
    let expected = "color(--é none) device-cmyk(none none none none / none)";
    let before = pair.clone();
    for _ in 0..3 {
        assert_eq!(
            pair.serialize_specified_with_limits(Limits::new(
                usize::MAX,
                usize::MAX,
                expected.len()
            ))
            .unwrap(),
            expected
        );
        assert_eq!(
            pair.serialize_specified_with_limits(Limits::new(
                usize::MAX,
                usize::MAX,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
            Kind::ByteLimit
        );
        assert_eq!(pair, before);
    }
    let source = declaration("background:url(\"café.svg\") device-cmyk(0 81% 81% 30% / none)");
    let CssKnownPropertyValueRef::Background(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("background")
    };
    let value = value.background();
    let before = value.clone();
    let expected = "url(\"café.svg\") device-cmyk(0 0.81 0.81 0.3 / none)";
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(
                usize::MAX,
                usize::MAX,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, &before);
}

#[test]
fn image_wrappers_and_gradient_siblings_share_one_complete_byte_budget() {
    let value = images(
        "light-dark(linear-gradient(device-cmyk(0 81% 81% 30%), blue), none), linear-gradient(device-cmyk(none 0 0 1 / none), blue)",
    );
    let expected = "light-dark(linear-gradient(device-cmyk(0 0.81 0.81 0.3), blue), none), linear-gradient(device-cmyk(none 0 0 1 / none), blue)";
    let before = value.clone();
    for _ in 0..3 {
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(value, before);
    }
}

#[test]
fn numeric_depth_in_every_ink_channel_and_alpha_counts_the_device_function() {
    // Shared structural ceiling: 255 numeric function levels + CMYK = 256.
    // A 256-level numeric child plus CMYK must be rejected (257), regardless
    // of which of the four ink slots or alpha contains it.
    for slot in 0..5 {
        let calculation = format!("{}0.5{}", "calc(".repeat(255), ")".repeat(255));
        let mut channels = [
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "1".to_owned(),
        ];
        let mut alpha = "none".to_owned();
        if slot == 4 {
            alpha = calculation;
        } else {
            channels[slot] = calculation;
        }
        let text = format!("device-cmyk({} / {alpha})", channels.join(" "));
        assert_eq!(parse_component_values(&text).unwrap().nesting_depth(), 256);
        let value = color(&text);
        let before = value.clone();
        let mut expected_channels = ["0", "0", "0", "1"];
        let expected_alpha = if slot == 4 {
            "calc(0.5)"
        } else {
            expected_channels[slot] = "calc(0.5)";
            "none"
        };
        let expected = format!(
            "device-cmyk({} / {expected_alpha})",
            expected_channels.join(" ")
        );
        assert_eq!(value.to_specified_css().unwrap(), expected);
        assert_eq!(value, before);
        for invalid in [
            format!("contrast-color({text})"),
            format!("linear-gradient({text}, blue)"),
        ] {
            let property = if invalid.starts_with("contrast") {
                "color"
            } else {
                "background-image"
            };
            let report = parse_style_attribute(&format!("{property}:{invalid};opacity:.5"));
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.error().code() == CssErrorCode::NestingLimit),
                "{property} slot {slot}"
            );
            assert_eq!(report.syntax().len(), 1);
        }
        let deeper = text
            .replacen("calc(", "calc(calc(", 1)
            .replacen("0.5", "0.5)", 1);
        let report = parse_style_attribute(&format!("color:{deeper};opacity:.5"));
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.error().code() == CssErrorCode::NestingLimit),
            "slot {slot}"
        );
        assert_eq!(report.syntax().len(), 1);
    }
}
