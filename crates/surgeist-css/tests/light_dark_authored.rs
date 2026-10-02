#![forbid(unsafe_code)]
//! Color 5 WD20260908 §7 supplies exactly two colors or two image/none branches.
//! Color 4 CRD20260908 serialization and the existing specified-value budgets
//! supply the independent canonical and resource expectations below. LightDark
//! children use ordinary Standalone serialization, including under Mix/Origin:
//! frozen WebKit 73aa6c89 CSSLightDarkColor.cpp:66–72 serializes each stored color.
//! Masking 1 CRD20210805 §7.1/§7.9 admits images in mask layers.

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
        panic!("typed background images")
    };
    value.images().clone()
}

fn background(css: &str) -> CssBackground {
    let source = declaration(&format!("background:{css}"));
    let CssKnownPropertyValueRef::Background(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed background")
    };
    value.background().clone()
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
fn two_colors_preserve_order_case_normalization_and_symbolic_branches() {
    for (source, expected) in [
        ("LiGhT-DaRk(ReD, BLUE)", "light-dark(red, blue)"),
        (
            "light-dark(currentcolor, CanvasText)",
            "light-dark(currentcolor, canvastext)",
        ),
        (
            "light-dark(transparent, color(--P 50% 0%))",
            "light-dark(transparent, color(--P 0.5 0))",
        ),
        (
            "light-dark(light-dark(red, blue), green)",
            "light-dark(light-dark(red, blue), green)",
        ),
    ] {
        let value = color(source);
        assert_eq!(value.to_specified_css().unwrap(), expected);
        assert_ne!(value, color("red"));
        assert_eq!(color(expected).to_specified_css().unwrap(), expected);
    }
    assert_ne!(
        color("light-dark(red, blue)"),
        color("light-dark(blue, red)")
    );
}

#[test]
fn ordinary_child_rgb_hsl_and_calculated_alpha_have_explicit_canonical_values() {
    for (source, expected) in [
        (
            "light-dark(rgb(300 0 -5), hsl(0 100% 50%))",
            "light-dark(rgb(255, 0, 0), rgb(255, 0, 0))",
        ),
        (
            "light-dark(rgb(0 0 0 / calc(50%)), hsl(120 100% 50%))",
            "light-dark(rgba(0, 0, 0, 0.5), rgb(0, 255, 0))",
        ),
        (
            "light-dark(rgb(none 0 255), alpha(from red / calc(50%)))",
            "light-dark(color(srgb none 0 1), alpha(from red / calc(0.5)))",
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
fn light_dark_children_use_standalone_serialization_under_origin_and_mix_roles() {
    for (source, expected) in [
        (
            "alpha(from light-dark(hsl(0 100% 50%), rgb(300 0 0)) / 50%)",
            "alpha(from light-dark(rgb(255, 0, 0), rgb(255, 0, 0)) / 0.5)",
        ),
        (
            "rgb(from light-dark(hsl(120 100% 50%), blue) r g b)",
            "rgb(from light-dark(rgb(0, 255, 0), blue) r g b)",
        ),
        (
            "color-mix(light-dark(hsl(0 100% 50%), rgb(300 0 0)), blue)",
            "color-mix(light-dark(rgb(255, 0, 0), rgb(255, 0, 0)), blue)",
        ),
        (
            "light-dark(alpha(from rgb(300 0 0) / 50%), color-mix(red, blue))",
            "light-dark(alpha(from rgb(300 0 0) / 0.5), color-mix(red, blue))",
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
fn light_dark_contextual_exclusion_propagates_through_existing_color_wrappers() {
    for source in [
        "light-dark(red, blue)",
        "alpha(from light-dark(red, blue))",
        "rgb(from light-dark(red, blue) r g b)",
        "color-mix(red, light-dark(blue, green))",
    ] {
        assert_eq!(
            color(source).absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::LightDark),
            "{source}"
        );
    }
}

#[test]
fn two_image_branches_preserve_none_url_src_gradients_and_nested_pairs() {
    for (source, expected) in [
        ("LiGhT-DaRk(NONE, NONE)", "light-dark(none, none)"),
        (
            "light-dark(url(a.svg), none)",
            "light-dark(url(\"a.svg\"), none)",
        ),
        (
            "light-dark(none, src(\"b.svg\"))",
            "light-dark(none, src(\"b.svg\"))",
        ),
        (
            "light-dark(linear-gradient(red, blue), radial-gradient(red, blue))",
            "light-dark(linear-gradient(red, blue), radial-gradient(red, blue))",
        ),
        (
            "light-dark(light-dark(none, url(a.svg)), none)",
            "light-dark(light-dark(none, url(\"a.svg\")), none)",
        ),
    ] {
        let value = images(source);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let image = &value.images()[0];
        assert_ne!(image, &CssImageValue::None);
        let checked =
            CssImage::try_new(image.clone()).expect("LightDark none/none is a real image");
        assert_eq!(checked.serialize_specified().unwrap(), expected);
        assert_eq!(images(expected).serialize_specified().unwrap(), expected);
    }
    assert_ne!(
        images("light-dark(none, url(a.svg))"),
        images("light-dark(url(a.svg), none)")
    );
    assert!(CssImage::try_new(CssImageValue::None).is_none());
}

#[test]
fn exact_cardinality_commas_exhaustion_and_same_kind_are_required() {
    for property in ["color", "background-image"] {
        for invalid in [
            "light-dark()",
            "light-dark(red)",
            "light-dark(red blue)",
            "light-dark(red, blue, green)",
            "light-dark(red,)",
            "light-dark(,blue)",
            "light-dark(red, blue junk)",
            "light-dark(red, url(a.svg))",
            "light-dark(url(a.svg), red)",
            "light-dark(none, red)",
            "light-dark(red, none)",
            "light-dark(none, none, none)",
            "light-dark(none none)",
            "light-dark(none, url(a.svg) junk)",
            "light-dark(light-dark(red, url(a.svg)), blue)",
        ] {
            let source = format!("{property}:{invalid};opacity:.5");
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
                if property == "color" {
                    CssErrorCode::InvalidColorSyntax
                } else {
                    CssErrorCode::InvalidPropertyValue
                },
                "{source}"
            );
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert!(validate_style_attribute(&source).is_err());
        }
    }
}

#[test]
fn property_contexts_reject_the_other_form_and_keep_unfinished_color_guards() {
    for source in [
        "color:light-dark(none, none)",
        "color:light-dark(url(a.svg), url(b.svg))",
        "background-image:light-dark(red, blue)",
        "mask-image:light-dark(red, blue)",
        "border-image-source:light-dark(red, blue)",
        "list-style-image:light-dark(red, blue)",
        "color:contrast-color(red)",
        "color:device-cmyk(0 0 0 1)",
    ] {
        let report = parse_style_attribute(&format!("{source};opacity:.5"));
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
    }
}

#[test]
fn selected_color_consumers_admit_light_dark_without_selecting_a_branch() {
    for property in [
        "color",
        "background-color",
        "border-color",
        "border-top-color",
        "outline-color",
        "text-decoration-color",
        "column-rule-color",
    ] {
        let source = format!("{property}:light-dark(red, blue)!important");
        let value = declaration(&source);
        assert!(value.known().unwrap().property_value().is_some());
        assert_eq!(value.importance(), CssImportance::Important);
        assert!(validate_style_attribute(&source).is_ok());
    }
    for source in [
        "border:1px solid light-dark(red, blue)",
        "box-shadow:0 0 light-dark(red, blue)",
        "background-image:linear-gradient(light-dark(red, blue), green)",
        "mask-image:radial-gradient(light-dark(red, blue), green)",
    ] {
        assert!(
            declaration(source)
                .known()
                .unwrap()
                .property_value()
                .is_some(),
            "{source}"
        );
    }
    assert_eq!(
        images("linear-gradient(light-dark(red, blue), green)")
            .serialize_specified()
            .unwrap(),
        "linear-gradient(light-dark(red, blue), green)"
    );
}

#[test]
fn selected_image_consumers_admit_light_dark_image_and_none_branches() {
    for source in [
        "background-image:light-dark(none, url(a.svg))",
        "mask-image:light-dark(none, url(a.svg))",
        "border-image-source:light-dark(none, url(a.svg))",
        "list-style-image:light-dark(none, url(a.svg))",
        "content:light-dark(none, url(a.svg))",
        "list-style-type:symbols(light-dark(none, url(a.svg)))",
        "border-image:light-dark(none, url(a.svg)) 30",
        "list-style:inside light-dark(none, url(a.svg))",
    ] {
        let value = declaration(source);
        assert!(
            value.known().unwrap().property_value().is_some(),
            "{source}"
        );
        assert!(validate_style_attribute(source).is_ok(), "{source}");
    }
}

#[test]
fn background_disambiguates_the_two_forms_and_projects_both_actual_values() {
    for source in [
        "background:light-dark(red, blue)!important",
        "background:light-dark(none, url(a.svg)) light-dark(red, blue)!important",
        "background:light-dark(none, url(a.svg)), light-dark(red, blue)!important",
    ] {
        let declaration = declaration(source);
        let values = expanded(&declaration);
        let image = values
            .items()
            .iter()
            .find(|v| v.property() == CssKnownProperty::BackgroundImage)
            .unwrap();
        let CssLonghandValueRef::BackgroundImage(image) = image.ordinary_value().unwrap().view()
        else {
            panic!("images")
        };
        let expected = match source {
            "background:light-dark(red, blue)!important" => "none",
            "background:light-dark(none, url(a.svg)) light-dark(red, blue)!important" => {
                "light-dark(none, url(\"a.svg\"))"
            }
            _ => "light-dark(none, url(\"a.svg\")), none",
        };
        assert_eq!(image.serialize_specified().unwrap(), expected);
        let color = values
            .items()
            .iter()
            .find(|v| v.property() == CssKnownProperty::BackgroundColor)
            .unwrap();
        let CssLonghandValueRef::BackgroundColor(color) = color.ordinary_value().unwrap().view()
        else {
            panic!("color")
        };
        assert_eq!(color.to_specified_css().unwrap(), "light-dark(red, blue)");
    }
    let value = background("light-dark(red, blue)");
    assert!(value.layers()[0].image().is_none());
    assert_eq!(
        value.layers()[0].color().unwrap(),
        &color("light-dark(red, blue)")
    );
    let value = background("light-dark(none, none)");
    assert!(value.layers()[0].color().is_none());
    assert_eq!(
        value.layers()[0].image().unwrap(),
        &images("light-dark(none, none)").images()[0]
    );
}

#[test]
fn background_keeps_final_color_duplicate_and_layer_recovery_contracts() {
    for invalid in [
        "light-dark(red, blue), none",
        "light-dark(red, blue) red",
        "light-dark(none, none) url(a.svg)",
        "light-dark(red, url(a.svg))",
        "light-dark(none, none) / cover",
    ] {
        let report = parse_style_attribute(&format!("background:{invalid};color:green"));
        assert_eq!(report.syntax().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one background diagnostic")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    }
}

#[test]
fn mask_shorthand_admits_plain_and_light_dark_gradients_with_checked_layer_parity() {
    for css in [
        "linear-gradient(red, blue)",
        "radial-gradient(red, blue)",
        "light-dark(linear-gradient(red, blue), none)",
        "light-dark(none, radial-gradient(red, blue))",
    ] {
        let image = images(css).images()[0].clone();
        let checked = CssMaskList::try_new(vec![
            CssMaskLayer::try_new(Some(image), None, None, None).expect("Masking admits images"),
        ])
        .unwrap();
        let source = declaration(&format!("mask:{css}"));
        let CssKnownPropertyValueRef::Mask(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("mask")
        };
        assert_eq!(value.value(), &checked, "{css}");
        let source = declaration(&format!("mask:{css} left top / contain no-repeat"));
        let CssKnownPropertyValueRef::Mask(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("mask with existing components")
        };
        assert_eq!(value.value().layers().len(), 1);
        assert!(value.value().layers()[0].position().is_some());
    }
    assert!(CssMaskLayer::try_new(None, None, None, None).is_none());
}

#[test]
fn parsed_and_programmatic_components_keep_origin_importance_and_raw_identity() {
    let parsed = declaration("color:LiGhT-DaRk(RED, blue)!important");
    assert!(matches!(
        parsed.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let graph = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "light-dark",
            CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("red").unwrap(),
                CssComponentValue::try_token(",").unwrap(),
                CssComponentValue::try_ident("blue").unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let programmatic = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        graph.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert!(programmatic.position().is_none());
    assert_eq!(programmatic.value_components(), &graph);
    assert_eq!(
        programmatic.value_components().items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    for source in [parsed, programmatic] {
        let before = source.clone();
        let values = expanded(&source);
        let [value] = values.items() else {
            panic!("one color")
        };
        let CssLonghandValueRef::Color(value) = value.ordinary_value().unwrap().view() else {
            panic!("color terminal")
        };
        assert_eq!(value.to_specified_css().unwrap(), "light-dark(red, blue)");
        assert_eq!(source, before);
        assert_eq!(source.importance(), CssImportance::Important);
    }
}

#[test]
fn pending_reentry_preserves_color_image_and_background_replacement_provenance() {
    for (property, replacement) in [
        ("color", "light-dark(red, blue)"),
        ("background-image", "light-dark(none, url(a.svg)), none"),
        ("mask-image", "light-dark(none, linear-gradient(red, blue))"),
        (
            "background",
            "light-dark(none, url(a.svg)) light-dark(red, blue)",
        ),
    ] {
        let source = declaration(&format!("{property}:var(--choice)!important"));
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending {property}")
        };
        for invalid in [
            "light-dark(red, url(a.svg))",
            "light-dark(none, red)",
            "light-dark(red, blue, green)",
        ] {
            assert!(
                matches!(
                    pending
                        .reenter(parse_component_values(invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ),
                "{property}: {invalid}"
            );
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
fn normalization_retains_color_and_image_order_values_and_sources() {
    let report = parse_sheet(
        ".a{color:light-dark(red, blue)!important;background-image:light-dark(none, url(a.svg));mask-image:light-dark(none, none)}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 3);
    for (index, property) in [
        CssKnownProperty::Color,
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::MaskImage,
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
        assert_eq!(items.items()[0].property(), property);
        assert!(
            items.items()[0]
                .source()
                .same_occurrence(values[index].source())
        );
        match items.items()[0].ordinary_value().unwrap().view() {
            CssLonghandValueRef::Color(v) => {
                assert_eq!(v.to_specified_css().unwrap(), "light-dark(red, blue)")
            }
            CssLonghandValueRef::BackgroundImage(v) => assert_eq!(
                v.serialize_specified().unwrap(),
                "light-dark(none, url(\"a.svg\"))"
            ),
            CssLonghandValueRef::MaskImage(v) => {
                assert_eq!(v.serialize_specified().unwrap(), "light-dark(none, none)")
            }
            _ => panic!("expected normalized value"),
        }
    }
    assert_eq!(values[0].source().importance(), CssImportance::Important);
}

#[test]
fn color_pair_charges_wrapper_and_both_siblings_in_one_resource_context() {
    let value = color("light-dark(red, blue)");
    let expected = "light-dark(red, blue)";
    assert_eq!(
        value
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                3,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
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

#[test]
fn image_pairs_and_list_siblings_share_input_projection_and_punctuation_budgets() {
    let value = images("light-dark(none, none), light-dark(none, none)");
    let expected = "light-dark(none, none), light-dark(none, none)";
    // list + two wrappers + four none leaves = seven visited nodes.
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                7,
                7,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 7, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
    }
}

#[test]
fn utf8_image_bytes_and_background_prefixes_have_exact_atomic_limits() {
    let value = images("light-dark(url(\"café.svg\"), none)");
    let expected = "light-dark(url(\"café.svg\"), none)";
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
    let value =
        background("light-dark(none, none) left top / cover no-repeat light-dark(red, blue)");
    let expected = "light-dark(none, none) left top / cover no-repeat light-dark(red, blue)";
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
fn nested_color_and_image_grammar_obeys_the_shared_256_level_ceiling() {
    for (property, leaf) in [
        (CssKnownProperty::Color, "red"),
        (CssKnownProperty::BackgroundImage, "none"),
    ] {
        let mut text = leaf.to_owned();
        for _ in 0..256 {
            text = format!("light-dark({text}, {leaf})");
        }
        let components = parse_component_values(&text).expect("exact shared component ceiling");
        let source = parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Normal,
        )
        .expect("LightDark accepts the full structural ceiling");
        match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::Color(v) => {
                assert_eq!(v.value().to_specified_css().unwrap(), text)
            }
            CssKnownPropertyValueRef::BackgroundImage(v) => {
                assert_eq!(v.images().serialize_specified().unwrap(), text)
            }
            _ => panic!("selected deep value"),
        }
        let report = parse_style_attribute(&format!(
            "{}:light-dark({text}, {leaf})",
            property.canonical_name()
        ));
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|v| v.error().code() == CssErrorCode::NestingLimit)
        );
    }
}

#[test]
fn full_image_subtree_nesting_errors_survive_ambiguous_consumer_retries() {
    // The raw unquoted URL is a single token, not a component function.
    // Its typed image is nevertheless a URL function at depth one. Therefore
    // 256 LightDark functions fit the raw ceiling but exceed the image ceiling.
    let mut text = "url(a.svg)".to_owned();
    for _ in 0..256 {
        text = format!("light-dark({text}, none)");
    }
    let components = parse_component_values(&text).expect("raw graph fits its ceiling");
    assert_eq!(components.nesting_depth(), 256);
    assert_eq!(components.serialize().unwrap().as_css(), text);
    let mut leaf = &components.items()[0];
    for _ in 0..256 {
        let CssComponentValueRef::Function(function) = leaf.view() else {
            panic!("raw LightDark function")
        };
        leaf = &function.values().items()[0];
    }
    assert!(matches!(
        leaf.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Url("a.svg"))
    ));
    for property in [
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::MaskImage,
        CssKnownProperty::BorderImageSource,
        CssKnownProperty::ListStyleImage,
        CssKnownProperty::Background,
        CssKnownProperty::Content,
        CssKnownProperty::ListStyle,
        CssKnownProperty::BorderImage,
    ] {
        let error = parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Normal,
        )
        .expect_err("whole image subtree exceeds the checked ceiling");
        assert!(
            matches!(
                error.kind(),
                CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
            ),
            "{} must preserve the resource failure: {error:?}",
            property.canonical_name()
        );
        let source = declaration(&format!("{}:var(--image)", property.canonical_name()));
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending image consumer")
        };
        let error = pending.reenter(components.clone()).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
            panic!("typed invalid replacement")
        };
        assert!(
            matches!(
                error.kind(),
                CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
            ),
            "{} reentry must preserve the resource failure: {error:?}",
            property.canonical_name()
        );
    }
}

#[test]
fn repeated_serialization_preserves_authored_graphs_including_nested_math_and_images() {
    let value = images(
        "light-dark(linear-gradient(light-dark(red, blue) calc(1px + 5%), transparent), light-dark(src(\"#id\"), none))",
    );
    let expected = "light-dark(linear-gradient(light-dark(red, blue) calc(5% + 1px), transparent), light-dark(src(\"#id\"), none))";
    let before = value.clone();
    for _ in 0..3 {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
    }
    assert_eq!(images(expected).serialize_specified().unwrap(), expected);
}
