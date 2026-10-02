#![forbid(unsafe_code)]
//! Color 5 WD20260908 §8 defines exactly one symbolic color input.
//! Ordinary child output follows Color 4 CRD20260908 and the selected Standalone
//! role, even under Mix/Origin. Frozen WebKit 73aa6c89 CSSContrastColor.cpp:59–64
//! independently serializes its stored color; contrast evaluation belongs later.
//! Resource expectations use the existing cumulative specified-value contract.
use surgeist_css::*;

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

#[test]
fn one_symbolic_color_is_retained_with_explicit_canonical_spelling() {
    for (source, expected) in [
        ("CoNtRaSt-CoLoR(ReD)", "contrast-color(red)"),
        (
            "contrast-color(currentColor)",
            "contrast-color(currentcolor)",
        ),
        ("contrast-color(CanvasText)", "contrast-color(canvastext)"),
        (
            "contrast-color(color(--P 50% 0%))",
            "contrast-color(color(--P 0.5 0))",
        ),
        ("contrast-color(transparent)", "contrast-color(transparent)"),
        (
            "contrast-color(contrast-color(red))",
            "contrast-color(contrast-color(red))",
        ),
        (
            "contrast-color(light-dark(red, blue))",
            "contrast-color(light-dark(red, blue))",
        ),
    ] {
        let value = color(source);
        assert_eq!(value.to_specified_css().unwrap(), expected);
        assert_ne!(value, color("white"));
        assert_ne!(value, color("black"));
        assert_eq!(color(expected).to_specified_css().unwrap(), expected);
    }
    assert_ne!(color("contrast-color(red)"), color("contrast-color(blue)"));
}

#[test]
fn standalone_child_rgb_hsl_and_calculated_alpha_have_independent_outputs() {
    for (source, expected) in [
        (
            "contrast-color(rgb(300 0 -5))",
            "contrast-color(rgb(255, 0, 0))",
        ),
        (
            "contrast-color(hsl(120 100% 50%))",
            "contrast-color(rgb(0, 255, 0))",
        ),
        (
            "contrast-color(rgb(calc(50%) 0 0))",
            "contrast-color(rgb(127.5, 0, 0))",
        ),
        (
            "contrast-color(rgb(0 0 0 / calc(50%)))",
            "contrast-color(rgba(0, 0, 0, 0.5))",
        ),
        (
            "contrast-color(rgb(none 0 255))",
            "contrast-color(color(srgb none 0 1))",
        ),
        (
            "contrast-color(hsl(none 50% 50%))",
            "contrast-color(hsl(none 50% 50%))",
        ),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn contrast_input_uses_standalone_even_when_its_wrapper_has_origin_or_mix_role() {
    for (source, expected) in [
        (
            "alpha(from contrast-color(hsl(0 100% 50%)) / 50%)",
            "alpha(from contrast-color(rgb(255, 0, 0)) / 0.5)",
        ),
        (
            "rgb(from contrast-color(rgb(300 0 0)) r g b)",
            "rgb(from contrast-color(rgb(255, 0, 0)) r g b)",
        ),
        (
            "color-mix(contrast-color(hsl(0 100% 50%)), blue)",
            "color-mix(contrast-color(rgb(255, 0, 0)), blue)",
        ),
        (
            "contrast-color(alpha(from rgb(300 0 0) / 50%))",
            "contrast-color(alpha(from rgb(300 0 0) / 0.5))",
        ),
        (
            "contrast-color(rgb(from red r g b / alpha))",
            "contrast-color(rgb(from red r g b / alpha))",
        ),
        (
            "contrast-color(color-mix(red, blue))",
            "contrast-color(color-mix(red, blue))",
        ),
        (
            "light-dark(contrast-color(hsl(0 100% 50%)), blue)",
            "light-dark(contrast-color(rgb(255, 0, 0)), blue)",
        ),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn contextual_contrast_exclusion_propagates_through_existing_wrappers() {
    for source in [
        "contrast-color(red)",
        "contrast-color(currentcolor)",
        "contrast-color(CanvasText)",
        "contrast-color(color(--P 0.5))",
        "alpha(from contrast-color(red))",
        "rgb(from contrast-color(red) r g b)",
        "color-mix(red, contrast-color(blue))",
    ] {
        assert_eq!(
            color(source).absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::ContrastColor),
            "{source}"
        );
    }
    assert_eq!(
        color("light-dark(contrast-color(red), blue)").absolute_eligibility(),
        CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::LightDark)
    );
}

#[test]
fn absolute_palette_construction_rejects_the_admitted_contextual_contrast_input() {
    let index =
        CssFontPaletteIndex::try_new(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(0)))
            .unwrap();
    for source in [
        "contrast-color(red)",
        "alpha(from contrast-color(red))",
        "color-mix(red, contrast-color(blue))",
    ] {
        assert_eq!(
            CssFontPaletteOverride::try_new(index.clone(), color(source)).unwrap_err(),
            CssFontPaletteConstructionError::ContextualColor(
                CssAbsoluteColorExclusion::ContrastColor
            )
        );
        let report = parse_sheet(&format!(
            "@font-palette-values --p {{ font-family: Demo; override-colors: 0 {source}; base-palette: light; }}"
        ));
        let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
            panic!("retained palette")
        };
        assert_eq!(rule.descriptors().len(), 2);
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDescriptor
        );
    }
}

#[test]
fn exact_single_color_cardinality_and_exhaustion_drop_only_the_invalid_declaration() {
    for invalid in [
        "contrast-color()",
        "contrast-color(red, blue)",
        "contrast-color(red blue)",
        "contrast-color(red,)",
        "contrast-color(,red)",
        "contrast-color(red / 50%)",
        "contrast-color(none)",
        "contrast-color(url(a.svg))",
        "contrast-color(linear-gradient(red, blue))",
        "contrast-color(light-dark(none, none))",
        "contrast-color(initial)",
        "contrast-color(inherit)",
        "contrast-color(unset)",
        "contrast-color(revert)",
        "contrast-color(revert-layer)",
        "contrast-color(red vs white black)",
        "color-contrast(red vs white black)",
        "contrast-color(red to 4.5)",
        "contrast-color",
        "device-cmyk(0 0 0 1)",
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
    }
}

#[test]
fn image_and_noncolor_property_contexts_reject_a_contrast_color() {
    for property in [
        "background-image",
        "mask-image",
        "border-image-source",
        "list-style-image",
        "opacity",
    ] {
        let report = parse_style_attribute(&format!("{property}:contrast-color(red);color:green"));
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
fn registered_color_consumers_admit_contrast_without_evaluating_it() {
    for property in [
        "color",
        "background-color",
        "border-color",
        "border-top-color",
        "outline-color",
        "text-decoration-color",
        "column-rule-color",
    ] {
        let css = format!("{property}:contrast-color(red)!important");
        let source = declaration(&css);
        assert_eq!(source.importance(), CssImportance::Important);
        assert!(source.known().unwrap().property_value().is_some());
        assert!(validate_style_attribute(&css).is_ok());
    }
    for css in [
        "border:1px solid contrast-color(red)",
        "box-shadow:0 0 contrast-color(red)",
        "filter:drop-shadow(0 0 contrast-color(red))",
        "background-image:linear-gradient(contrast-color(red), blue)",
        "mask-image:radial-gradient(contrast-color(red), blue)",
    ] {
        assert!(
            declaration(css).known().unwrap().property_value().is_some(),
            "{css}"
        );
    }
}

#[test]
fn background_projects_contrast_as_final_color_alongside_a_light_dark_image() {
    let source =
        declaration("background:light-dark(none, url(a.svg)) contrast-color(red)!important");
    let values = expanded(&source);
    let image = values
        .items()
        .iter()
        .find(|v| v.property() == CssKnownProperty::BackgroundImage)
        .unwrap();
    let CssLonghandValueRef::BackgroundImage(image) = image.ordinary_value().unwrap().view() else {
        panic!("image terminal")
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
        panic!("color terminal")
    };
    assert_eq!(value, &color("contrast-color(red)"));
    assert_eq!(value.to_specified_css().unwrap(), "contrast-color(red)");
    let source = declaration("background:contrast-color(red)");
    let CssKnownPropertyValueRef::Background(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("color-only background")
    };
    assert!(value.background().layers()[0].image().is_none());
    assert_eq!(
        value.background().layers()[0]
            .color()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "contrast-color(red)"
    );
}

#[test]
fn nonfinal_or_duplicate_background_contrast_colors_preserve_sibling_recovery() {
    for invalid in [
        "contrast-color(red), none",
        "contrast-color(red) blue",
        "contrast-color(red) contrast-color(blue)",
    ] {
        let report = parse_style_attribute(&format!("background:{invalid};color:green"));
        assert_eq!(report.syntax().len(), 1, "{invalid}");
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
    let parsed = declaration("color:CoNtRaSt-CoLoR(RED)!important");
    assert!(matches!(
        parsed.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let mixed_child = parse_component_values("red").unwrap();
    let mixed_origin = mixed_child.items()[0].origin().clone();
    for child in [
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("red").unwrap()]).unwrap(),
        mixed_child,
    ] {
        let graph = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("contrast-color", child.clone()).unwrap(),
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
        assert_eq!(value.to_specified_css().unwrap(), "contrast-color(red)");
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
        ("color", "contrast-color(red)"),
        ("background-color", "contrast-color(currentcolor)"),
        (
            "background",
            "light-dark(none, url(a.svg)) contrast-color(red)",
        ),
        (
            "background-image",
            "linear-gradient(contrast-color(red), blue)",
        ),
    ] {
        let source = declaration(&format!("{property}:var(--choice)!important"));
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for invalid in [
            "contrast-color()",
            "contrast-color(red, blue)",
            "contrast-color(none)",
        ] {
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
fn normalization_preserves_contrast_order_values_and_source_occurrences() {
    let report = parse_sheet(
        ".a{color:contrast-color(red)!important;background-color:contrast-color(blue);background-image:linear-gradient(contrast-color(green), blue)}",
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
        "contrast-color(red)",
        "contrast-color(blue)",
        "linear-gradient(contrast-color(green), blue)",
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

#[test]
fn wrapper_and_child_charge_exact_cumulative_node_projection_and_byte_budgets() {
    for (source, expected, nodes) in [
        ("contrast-color(red)", "contrast-color(red)", 2),
        (
            "light-dark(contrast-color(red), contrast-color(blue))",
            "light-dark(contrast-color(red), contrast-color(blue))",
            5,
        ),
        (
            "color-mix(contrast-color(red), blue)",
            "color-mix(contrast-color(red), blue)",
            4,
        ),
    ] {
        let value = color(source);
        assert_eq!(
            value
                .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                    nodes,
                    nodes,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let before = value.clone();
            assert_eq!(
                value
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, before);
        }
    }
}

#[test]
fn utf8_url_prefix_and_nested_image_children_share_the_complete_output_budget() {
    let source = declaration("background:url(\"café.svg\") contrast-color(red)");
    let CssKnownPropertyValueRef::Background(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("background")
    };
    let expected = "url(\"café.svg\") contrast-color(red)";
    let value = value.background();
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    let value = images(
        "light-dark(linear-gradient(contrast-color(red), blue), none), linear-gradient(contrast-color(green), blue)",
    );
    let expected = "light-dark(linear-gradient(contrast-color(red), blue), none), linear-gradient(contrast-color(green), blue)";
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                100,
                100,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn parsed_and_checked_contrast_chains_obey_the_complete_256_level_ceiling() {
    for leaf in ["red", "rgb(1 0 0)"] {
        let mut text = leaf.to_owned();
        let wrappers = if leaf == "red" { 256 } else { 255 };
        for _ in 0..wrappers {
            text = format!("contrast-color({text})");
        }
        let components = parse_component_values(&text).unwrap();
        assert_eq!(components.nesting_depth(), 256);
        let source = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            components,
            CssImportance::Normal,
        )
        .expect("complete admitted graph at ceiling");
        let CssKnownPropertyValueRef::Color(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("deep color")
        };
        let expected = if leaf == "red" {
            text.clone()
        } else {
            format!(
                "{}rgb(1, 0, 0){}",
                "contrast-color(".repeat(255),
                ")".repeat(255)
            )
        };
        assert_eq!(value.value().to_specified_css().unwrap(), expected);
        let report = parse_style_attribute(&format!("color:contrast-color({text})"));
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|value| value.error().code() == CssErrorCode::NestingLimit)
        );
    }
}

#[test]
fn complete_numeric_child_depth_is_counted_and_repeated_output_keeps_raw_graphs() {
    let calculation = format!("{}50%{}", "calc(".repeat(254), ")".repeat(254));
    let text = format!("contrast-color(rgb({calculation} 0 0))");
    let value = color(&text);
    let before = value.clone();
    for _ in 0..3 {
        assert_eq!(
            value.to_specified_css().unwrap(),
            "contrast-color(rgb(127.5, 0, 0))"
        );
        assert_eq!(value, before);
    }
    let report = parse_style_attribute(&format!("color:contrast-color({text})"));
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|value| value.error().code() == CssErrorCode::NestingLimit)
    );
}
