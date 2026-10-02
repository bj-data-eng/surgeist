#![forbid(unsafe_code)]
//! Color5 WD 2026-09-08 §11.1 omits the default shorter hue method.
//! Expectations come from that serialization clause, independently of the writer.

use surgeist_css::*;

fn parsed(source: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(source).unwrap(),
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

fn constructed(space: CssColorInterpolationSpace, hue: CssHueInterpolationMethod) -> CssColor {
    CssColor::from_color_mix(
        CssColorMix::try_new(
            Some(
                CssColorInterpolation::try_predefined(CssColorInterpolationMethod::new(
                    space,
                    Some(hue),
                ))
                .unwrap(),
            ),
            ["red", "blue"]
                .into_iter()
                .map(|name| {
                    CssColorMixComponent::new(
                        CssColor::from_named(CssNamedColor::try_new(name).unwrap()),
                        None,
                    )
                })
                .collect(),
        )
        .unwrap(),
    )
}

fn assert_text(value: &CssColor, expected: &str) {
    let before = value.clone();
    assert_eq!(value.to_specified_css().unwrap(), expected);
    assert_eq!(value, &before);
    assert_eq!(value.to_specified_css().unwrap(), expected);
    assert_eq!(parsed(expected).to_specified_css().unwrap(), expected);
}

macro_rules! shorter {
    ($name:ident, $space:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            for value in [
                parsed($source),
                constructed(
                    CssColorInterpolationSpace::$space,
                    CssHueInterpolationMethod::Shorter,
                ),
            ] {
                let interpolation = value
                    .color_mix_value()
                    .unwrap()
                    .interpolation()
                    .unwrap()
                    .predefined()
                    .unwrap();
                assert_eq!(interpolation.space(), CssColorInterpolationSpace::$space);
                assert_eq!(
                    interpolation.hue(),
                    Some(CssHueInterpolationMethod::Shorter)
                );
                assert_text(&value, $expected);
                assert_eq!(
                    value
                        .color_mix_value()
                        .unwrap()
                        .interpolation()
                        .unwrap()
                        .predefined()
                        .unwrap()
                        .hue(),
                    Some(CssHueInterpolationMethod::Shorter)
                );
            }
        }
    };
}

shorter!(
    hsl_shorter_is_omitted,
    Hsl,
    "color-mix(in hsl shorter hue, red, blue)",
    "color-mix(in hsl, red, blue)"
);
shorter!(
    hwb_shorter_is_omitted,
    Hwb,
    "color-mix(in hwb shorter hue, red, blue)",
    "color-mix(in hwb, red, blue)"
);
shorter!(
    lch_shorter_is_omitted,
    Lch,
    "color-mix(in lch shorter hue, red, blue)",
    "color-mix(in lch, red, blue)"
);
shorter!(
    oklch_shorter_is_omitted,
    Oklch,
    "color-mix(in oklch shorter hue, red, blue)",
    "color-mix(in oklch, red, blue)"
);

#[test]
fn nondefault_hue_methods_remain_explicit() {
    for (space, name) in [
        (CssColorInterpolationSpace::Hsl, "hsl"),
        (CssColorInterpolationSpace::Hwb, "hwb"),
        (CssColorInterpolationSpace::Lch, "lch"),
        (CssColorInterpolationSpace::Oklch, "oklch"),
    ] {
        for (hue, text) in [
            (CssHueInterpolationMethod::Longer, "longer hue"),
            (CssHueInterpolationMethod::Increasing, "increasing hue"),
            (CssHueInterpolationMethod::Decreasing, "decreasing hue"),
        ] {
            let expected = format!("color-mix(in {name} {text}, red, blue)");
            for value in [parsed(&expected), constructed(space, hue)] {
                assert_text(&value, &expected);
            }
        }
    }
}

#[test]
fn absent_hue_and_other_interpolation_forms_keep_their_spelling() {
    for expected in [
        "color-mix(in hsl, red, blue)",
        "color-mix(in hwb, red, blue)",
        "color-mix(in lch, red, blue)",
        "color-mix(in oklch, red, blue)",
        "color-mix(in --Profile, red, blue)",
        "color-mix(red, blue)",
    ] {
        assert_text(&parsed(expected), expected);
    }
    assert_text(
        &parsed("color-mix(in oklab, red, blue)"),
        "color-mix(red, blue)",
    );
}

#[test]
fn nested_and_origin_mixes_omit_only_default_hue_methods() {
    for (source, expected) in [
        (
            "color-mix(in lch shorter hue, red, color-mix(in hsl longer hue, red, blue))",
            "color-mix(in lch, red, color-mix(in hsl longer hue, red, blue))",
        ),
        (
            "color-mix(in hwb longer hue, color-mix(in oklch shorter hue, red, blue), blue)",
            "color-mix(in hwb longer hue, color-mix(in oklch, red, blue), blue)",
        ),
        (
            "alpha(from color-mix(in lch shorter hue, red, blue))",
            "alpha(from color-mix(in lch, red, blue))",
        ),
    ] {
        assert_text(&parsed(source), expected);
    }
}

#[test]
fn shorter_omission_preserves_style_importance_and_authored_tokens() {
    let source = "color-mix(in lch shorter hue, red, blue)";
    let style = format!("color: {source} !important; opacity: .5");
    let authored = format!(" {source} ");
    let report = parse_style_attribute(&style);
    assert!(report.diagnostics().is_empty());
    validate_style_attribute(&style).unwrap();
    assert_eq!(report.syntax().len(), 2);
    let declaration = &report.syntax()[0];
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        authored
    );
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    assert_text(value.value(), "color-mix(in lch, red, blue)");
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        authored
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
}

#[test]
fn omitted_hue_uses_final_byte_length_and_retains_traversal_costs() {
    let value = parsed("color-mix(in lch shorter hue, red, blue)");
    let before = value.clone();
    // Mix + explicit interpolation + two named colors cost I4/P4.
    // Filling/equality-checking the two omitted weights costs another P20.
    let expected = "color-mix(in lch, red, blue)";
    for (limits, error) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 24, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 23, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 24, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        for _ in 0..2 {
            assert_eq!(
                value
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                error
            );
            assert_eq!(value, before);
        }
    }
    assert_eq!(
        value
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                4,
                24,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(value, before);
}
