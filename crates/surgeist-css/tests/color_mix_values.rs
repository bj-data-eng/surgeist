#![forbid(unsafe_code)]
//! Public authored contracts from Color5 color-mix and Values4 calc-range.
use surgeist_css::*;

fn parsed(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("color:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn checked_components(values: CssComponentValues) -> CssDeclaration {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &values);
    declaration
}

fn checked(text: &str) -> CssDeclaration {
    checked_components(parse_component_values(text).unwrap())
}

fn wrapper(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color value")
    };
    value
}

fn color(text: &str) -> CssColor {
    wrapper(&checked(text)).value().clone()
}

fn component(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    assert_eq!(values.items().len(), 1);
    values.items()[0].clone()
}

#[test]
fn ordered_nonempty_lists_retain_duplicates_weights_and_method_omission() {
    for declaration in [
        parsed("color-mix(25% red, green 50%, red)"),
        checked("color-mix(25% red, green 50%, red)"),
    ] {
        let mix = wrapper(&declaration).value().color_mix_value().unwrap();
        assert!(mix.interpolation().is_none());
        assert_eq!(mix.components().len(), 3);
        for (item, expected) in mix.components().iter().zip(["red", "green", "red"]) {
            assert_eq!(item.color().named().unwrap().name(), expected);
        }
        for (item, expected) in mix.components()[..2].iter().zip([25.0, 50.0]) {
            let weight = item.weight().unwrap();
            assert_eq!(
                weight
                    .literal_value()
                    .unwrap()
                    .literal()
                    .numeric()
                    .representation(),
                if expected == 25.0 { "25" } else { "50" }
            );
            assert!(weight.calculation().is_none());
        }
        assert!(mix.components()[2].weight().is_none());
    }
    let singleton = checked("color-mix(red)");
    assert_eq!(
        wrapper(&singleton)
            .value()
            .color_mix_value()
            .unwrap()
            .components()
            .len(),
        1
    );
    let explicit = checked("color-mix(in oklab, red)");
    let mix = wrapper(&explicit).value().color_mix_value().unwrap();
    assert_eq!(
        mix.interpolation().unwrap().predefined().unwrap().space(),
        CssColorInterpolationSpace::Oklab
    );
    assert_ne!(wrapper(&singleton).value(), wrapper(&explicit).value());
}

#[test]
fn custom_profile_identity_decodes_escapes_without_folding_case() {
    for (spelling, expected) in [
        ("--Profile", "--Profile"),
        ("--profile", "--profile"),
        ("--Pr\\6f file", "--Profile"),
        ("--a\\ b", "--a b"),
        ("--", "--"),
    ] {
        for declaration in [
            parsed(&format!("color-mix(in {spelling}, red)")),
            checked(&format!("color-mix(in {spelling}, red)")),
        ] {
            let method = wrapper(&declaration)
                .value()
                .color_mix_value()
                .unwrap()
                .interpolation()
                .unwrap();
            assert!(method.predefined().is_none());
            assert_eq!(method.custom_profile().unwrap().as_str(), expected);
            assert_eq!(
                method.custom_profile(),
                Some(&CssColorProfileName::try_new(expected).unwrap())
            );
        }
    }
    assert_ne!(
        CssColorProfileName::try_new("--Profile"),
        CssColorProfileName::try_new("--profile")
    );
    for invalid in ["", "profile", "-profile", "--\0"] {
        assert!(CssColorProfileName::try_new(invalid).is_none(), "{invalid}");
    }
    let custom = CssColorInterpolation::custom(CssColorProfileName::try_new("--").unwrap());
    assert_eq!(custom.custom_profile().unwrap().as_str(), "--");
}

#[test]
fn bare_percentage_roots_cannot_masquerade_as_calculated_mix_weights() {
    for coefficient in [-5.0, 120.0, 25.0] {
        for root in [
            CssPercentageCalculation::try_literal(coefficient).unwrap(),
            CssPercentageCalculation::try_from_components(
                parse_component_values(&format!("{coefficient}%")).unwrap(),
            )
            .unwrap(),
        ] {
            let origin = root.components().items()[0].origin().clone();
            let error = CssColorMixWeight::try_calculation(root).unwrap_err();
            assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
            assert_eq!(error.origin(), &origin);
        }
    }
    for text in ["-5%", "120%", "100.0000000000000000000001%"] {
        let supplied = component(text);
        let error = CssColorMixPercentage::try_from_component(supplied.clone()).unwrap_err();
        assert_eq!(error.kind(), CssColorScalarErrorKind::OutOfRange);
        assert_eq!(error.origin(), supplied.origin());
    }
    let literal = CssColorMixWeight::literal(
        CssColorMixPercentage::try_from_component(component("25%")).unwrap(),
    );
    assert_eq!(
        literal
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "25"
    );
    assert!(literal.calculation().is_none());
}

#[test]
fn genuine_calculations_preserve_the_graph_origins_and_specified_range() {
    for coefficient in ["-5", "120", "25"] {
        let source = format!("calc({coefficient}%)");
        let parsed_graph = parse_component_values(&source).unwrap();
        let programmatic_graph = CssComponentValues::try_new(vec![
            CssComponentValue::try_function(
                "calc",
                CssComponentValues::try_new(vec![
                    CssComponentValue::try_token(&format!("{coefficient}%")).unwrap(),
                ])
                .unwrap(),
            )
            .unwrap(),
        ])
        .unwrap();
        for graph in [parsed_graph, programmatic_graph] {
            let root = CssPercentageCalculation::try_from_components(graph.clone()).unwrap();
            let origin = root.origin().clone();
            let weight = CssColorMixWeight::try_calculation(root.clone()).unwrap();
            assert!(weight.literal_value().is_none());
            let retained = weight.calculation().unwrap();
            assert_eq!(retained, &root);
            assert_eq!(retained.components(), &graph);
            assert_eq!(retained.origin(), &origin);
            assert_eq!(retained.result_type(), CssCalculationType::Percentage);
        }
        for text in [
            format!("color-mix(red {source})"),
            format!("color-mix({source} red)"),
        ] {
            for declaration in [parsed(&text), checked(&text)] {
                let mix = wrapper(&declaration).value().color_mix_value().unwrap();
                let weight = mix.components()[0].weight().unwrap();
                assert!(weight.literal_value().is_none());
                let calculation = weight.calculation().unwrap();
                assert_eq!(
                    calculation.components().serialize().unwrap().as_css(),
                    source
                );
                let CssValueOrigin::Parsed(origin) = calculation.origin() else {
                    panic!("original math source")
                };
                assert!(origin.source().as_str().contains(&text));
            }
        }
    }
}

#[test]
fn checked_list_and_interpolation_constructors_enforce_their_domains() {
    assert_eq!(
        CssColorMix::try_new(None, vec![]).unwrap_err(),
        CssColorMixConstructionError::EmptyComponents
    );
    for space in [
        CssColorInterpolationSpace::Oklab,
        CssColorInterpolationSpace::Lab,
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
    ] {
        assert_eq!(
            CssColorInterpolationMethod::try_new(space, Some(CssHueInterpolationMethod::Longer)),
            Err(CssColorInterpolationMethodConstructionError::HueRequiresPolarSpace)
        );
    }
    for space in [
        CssColorInterpolationSpace::Hsl,
        CssColorInterpolationSpace::Hwb,
        CssColorInterpolationSpace::Lch,
        CssColorInterpolationSpace::Oklch,
    ] {
        let method =
            CssColorInterpolationMethod::try_new(space, Some(CssHueInterpolationMethod::Longer))
                .unwrap();
        let interpolation = CssColorInterpolation::from_predefined(method);
        assert_eq!(interpolation.predefined(), Some(method));
        assert!(interpolation.custom_profile().is_none());
    }
    let exact = CssColorMixWeight::literal(
        CssColorMixPercentage::try_from_component(component("1e-100%")).unwrap(),
    );
    let mix = CssColorMix::try_new(
        None,
        vec![CssColorMixComponent::new(color("red"), Some(exact))],
    )
    .unwrap();
    let exact = mix.components()[0]
        .weight()
        .unwrap()
        .literal_value()
        .unwrap()
        .literal();
    assert_eq!(exact.numeric().representation(), "1e-100");
}

#[test]
fn checked_mixed_origin_weights_keep_the_actual_leaf_source() {
    let leaf = component("-5%");
    let calculation = CssComponentValue::try_function(
        "calc",
        CssComponentValues::try_new(vec![leaf.clone()]).unwrap(),
    )
    .unwrap();
    let children = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("red").unwrap(),
        calculation,
    ])
    .unwrap();
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("color-mix", children).unwrap(),
    ])
    .unwrap();
    let declaration = checked_components(values);
    let mix = wrapper(&declaration).value().color_mix_value().unwrap();
    let calculation = mix.components()[0].weight().unwrap().calculation().unwrap();
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    let mut expression = calculation.expression();
    loop {
        match expression {
            CssCalculationExpressionRef::NestedCalc(value)
            | CssCalculationExpressionRef::Group(value) => expression = value.operand(),
            CssCalculationExpressionRef::Value(value) => {
                assert_eq!(value.literal().representation(), "-5");
                assert_eq!(value.literal().origin(), leaf.origin());
                break;
            }
            _ => panic!("one percentage leaf"),
        }
    }
}

#[test]
fn enclosing_mix_counts_calculation_depth_in_its_composed_graph() {
    for (depth, succeeds) in [(255, true), (256, false)] {
        let source = format!("{}25%{}", "calc(".repeat(depth), ")".repeat(depth));
        let calculation =
            CssPercentageCalculation::try_from_components(parse_component_values(&source).unwrap())
                .unwrap();
        let weight = CssColorMixWeight::try_calculation(calculation).unwrap();
        let result = CssColorMix::try_new(
            None,
            vec![CssColorMixComponent::new(color("red"), Some(weight))],
        );
        if succeeds {
            assert_eq!(result.unwrap().components().len(), 1);
        } else {
            assert_eq!(
                result.unwrap_err(),
                CssColorMixConstructionError::NestingLimit
            );
        }
    }
}

#[test]
fn nested_mix_retains_each_list_child_and_weight_kind() {
    for (inner, expected_count, expected_weight) in [
        ("color-mix(in srgb, red 25%, blue 50%)", 2, Some("25")),
        ("color-mix(in srgb, red, green, blue)", 3, None),
        ("color-mix(in srgb, red 0.1%, blue)", 2, Some("0.1")),
        ("color-mix(in srgb, red calc(25%), blue)", 2, None),
    ] {
        let source = format!("color-mix(in srgb, red, rgb(from {inner} r g b))");
        for declaration in [parsed(&source), checked(&source)] {
            let nested = wrapper(&declaration)
                .value()
                .color_mix_value()
                .unwrap()
                .components()[1]
                .color()
                .relative_value()
                .unwrap()
                .source()
                .color_mix_value()
                .unwrap();
            assert_eq!(nested.components().len(), expected_count);
            let weight = nested.components()[0].weight();
            match expected_weight {
                Some(coefficient) => assert_eq!(
                    weight
                        .unwrap()
                        .literal_value()
                        .unwrap()
                        .literal()
                        .numeric()
                        .representation(),
                    coefficient
                ),
                None if inner.contains("calc") => assert!(weight.unwrap().calculation().is_some()),
                None => assert!(weight.is_none()),
            }
        }
    }
}

fn nested_list(depth: usize) -> String {
    let mut text = "rgb(1e100 0 0)".to_owned();
    for level in 1..depth {
        text = if level % 2 == 0 {
            format!("rgb(from {text} r g b)")
        } else {
            format!("color-mix(in srgb, red, green, {text})")
        };
    }
    text
}

#[test]
fn mixed_nary_relative_graphs_obey_the_shared_boundary_and_drop_normally() {
    for depth in [255, 256] {
        let text = nested_list(depth);
        for declaration in [parsed(&text), checked(&text)] {
            let value = wrapper(&declaration);
            let mut current = value.value();
            let mut visited = 1;
            loop {
                if let Some(mix) = current.color_mix_value() {
                    assert_eq!(mix.components().len(), 3);
                    current = mix.components()[2].color();
                } else if let Some(relative) = current.relative_value() {
                    current = relative.source();
                } else {
                    break;
                }
                visited += 1;
            }
            assert_eq!(visited, depth);
            let CssColorComponent::Number(leaf) = &current.rgb_value().unwrap().channels()[0]
            else {
                panic!("exact deepest leaf")
            };
            assert_eq!(leaf.numeric().representation(), "1e100");
        }
    }
    let source = nested_list(257);
    assert_eq!(
        parse_component_values(&source).unwrap_err().kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    let report = parse_style_attribute(&format!("color:{source};opacity:.5"));
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::NestingLimit
    );
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let child = color(&nested_list(256));
    assert_eq!(
        CssColorMix::try_new(None, vec![CssColorMixComponent::new(child, None)]).unwrap_err(),
        CssColorMixConstructionError::NestingLimit
    );
}

#[test]
fn moderately_wide_lists_charge_component_and_byte_budgets_cumulatively() {
    let width = 128;
    let source = format!("color-mix({})", vec!["red"; width].join(","));
    // One function, 128 identifiers and 127 commas: exactly 256 components.
    let count = 2 * width;
    let limits = CssComponentValueLimits::try_new(256, count, source.len()).unwrap();
    let graph = parse_component_values_with_limits(&source, limits).unwrap();
    let declaration = checked_components(graph.clone());
    let mix = wrapper(&declaration).value().color_mix_value().unwrap();
    assert_eq!(mix.components().len(), width);
    assert!(
        mix.components()
            .iter()
            .all(|item| item.color().named().unwrap().name() == "red" && item.weight().is_none())
    );
    for (components, bytes, expected) in [
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
        let limits = CssComponentValueLimits::try_new(256, components, bytes).unwrap();
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

#[test]
fn enclosing_mix_counts_absolute_and_relative_channel_math_depth() {
    for (math_depth, succeeds) in [(254, true), (255, false)] {
        let math = format!("{}1{}", "calc(".repeat(math_depth), ")".repeat(math_depth));
        for text in [
            format!("rgb({math} 0 0)"),
            format!("rgb(from red {math} g b)"),
        ] {
            // The channel math is nested below the rgb function. The new mix
            // adds one more function: 254 + 1 + 1 = 256, 255 + 1 + 1 = 257.
            for declaration in [parsed(&text), checked(&text)] {
                let child = wrapper(&declaration).value().clone();
                let result =
                    CssColorMix::try_new(None, vec![CssColorMixComponent::new(child, None)]);
                if succeeds {
                    assert_eq!(result.unwrap().components().len(), 1);
                } else {
                    assert_eq!(
                        result.unwrap_err(),
                        CssColorMixConstructionError::NestingLimit
                    );
                }
            }
        }
    }
}
