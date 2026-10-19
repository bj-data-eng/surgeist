#![forbid(unsafe_code)]

//! Exact authored font-width contracts from CSS Fonts 4 WD 2026-09-07 §§2.3, 2.7.

use surgeist_css::*;

fn scalar(text: &str) -> CssSpecifiedNonNegativePercentage {
    let values = parse_component_values(text).unwrap();
    let [component] = values.items() else {
        panic!("one percentage: {text}")
    };
    CssSpecifiedNonNegativePercentage::try_from_component(component.clone()).unwrap()
}

fn ordinary_percentage_spelling(value: &CssFontWidth) -> &str {
    let CssFontWidth::Percentage(value) = value else {
        panic!("percentage width")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = value
        .literal_component()
        .expect("ordinary percentage")
        .view()
    else {
        panic!("percentage token")
    };
    number.representation()
}

fn width(name: &str, value: &str, checked: bool) -> CssFontWidth {
    let declaration = if checked {
        parse_property_value_for_grammar(
            CssPropertyGrammar::from_name(name).unwrap(),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap()
    } else {
        let report = parse_style_attribute(&format!("{name}:{value}"));
        assert!(
            report.is_clean(),
            "{name}:{value}: {:?}",
            report.diagnostics()
        );
        report.syntax()[0].clone()
    };
    let CssKnownPropertyValueRef::FontWidth(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("font-width typed wrapper")
    };
    wrapper.value().clone()
}

#[test]
fn nine_keywords_have_the_fonts_four_exact_percentage_mapping() {
    use CssFontWidthKeyword::*;
    for (keyword, spelling, percentage) in [
        (UltraCondensed, "ultra-condensed", "50%"),
        (ExtraCondensed, "extra-condensed", "62.5%"),
        (Condensed, "condensed", "75%"),
        (SemiCondensed, "semi-condensed", "87.5%"),
        (Normal, "normal", "100%"),
        (SemiExpanded, "semi-expanded", "112.5%"),
        (Expanded, "expanded", "125%"),
        (ExtraExpanded, "extra-expanded", "150%"),
        (UltraExpanded, "ultra-expanded", "200%"),
    ] {
        assert_eq!(
            keyword.percentage().serialize_specified().unwrap(),
            percentage
        );
        let direct = CssFontWidth::Keyword(keyword);
        assert_eq!(direct.serialize_specified().unwrap(), spelling);
        for name in ["font-width", "font-stretch"] {
            for checked in [false, true] {
                assert_eq!(width(name, spelling, checked), direct);
            }
        }
        assert_ne!(direct, CssFontWidth::Percentage(scalar(percentage)));
    }
}

#[test]
fn percentage_constructor_preserves_exact_literals_and_rejects_invalid_roots() {
    for (text, canonical) in [
        ("0%", "0%"),
        ("-0%", "0%"),
        ("62.500000000000000000001%", "62.5%"),
        ("999999999999999999999999%", "999999999999999999999999%"),
    ] {
        let direct = CssFontWidth::Percentage(scalar(text));
        assert_eq!(direct.serialize_specified().unwrap(), canonical);
        for name in ["font-width", "font-stretch"] {
            for checked in [false, true] {
                let actual = width(name, text, checked);
                assert_eq!(actual, direct, "{name}:{text}");
                assert_eq!(
                    ordinary_percentage_spelling(&actual),
                    text.trim_end_matches('%')
                );
                assert_eq!(
                    actual.serialize_specified().unwrap(),
                    canonical,
                    "{name}:{text}"
                );
            }
        }
    }
    assert_ne!(scalar("62.500000000000000000001%"), scalar("62.5%"));
    assert_ne!(
        scalar("999999999999999999999999%"),
        scalar("999999999999999999999998%")
    );
    for text in ["0", "1", "1px", "-1%", "-1e-999%"] {
        let values = parse_component_values(text).unwrap();
        let [component] = values.items() else {
            panic!("one invalid scalar")
        };
        assert!(
            CssSpecifiedNonNegativePercentage::try_from_component(component.clone()).is_err(),
            "{text}"
        );
    }
    for text in ["1e999%", "1e-999%"] {
        let direct = CssFontWidth::Percentage(scalar(text));
        assert_eq!(width("font-width", text, false), direct);
        assert_eq!(width("font-stretch", text, true), direct);
        assert_eq!(
            ordinary_percentage_spelling(&width("font-width", text, false)),
            text.trim_end_matches('%')
        );
        assert_eq!(
            ordinary_percentage_spelling(&width("font-stretch", text, true)),
            text.trim_end_matches('%')
        );
    }
}

#[test]
fn bare_percentage_calculation_roots_reenter_exact_range_admission() {
    for text in ["-1%", "-1e-999%"] {
        let typed =
            CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
                .expect("negative percentage is in the percentage calculation domain");
        assert!(
            CssSpecifiedNonNegativePercentage::try_from_calculation(typed).is_err(),
            "{text}"
        );
    }
    let positive = CssSpecifiedNonNegativePercentage::try_from_calculation(
        CssPercentageCalculation::try_from_components(parse_component_values("125%").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(positive.literal_component().is_some());
    assert!(positive.calculation().is_none());
    assert_eq!(positive.serialize_specified().unwrap(), "125%");
}

#[test]
fn parsed_percentages_keep_source_origin_and_symbolic_math_unresolved() {
    let report = parse_style_attribute("FoNt-StReTcH:62.500000000000000000001%");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::FontWidth(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font-width wrapper for legacy name")
    };
    let CssFontWidth::Percentage(parsed) = wrapper.value() else {
        panic!("exact parsed percentage")
    };
    assert!(matches!(parsed.origin(), CssValueOrigin::Parsed(_)));
    assert!(matches!(
        scalar("62.500000000000000000001%").origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(parsed.serialize_specified().unwrap(), "62.5%");

    assert!(matches!(
        parsed.literal_component().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "62.500000000000000000001"
    ));
    for text in ["calc(25% + 50%)", "calc(25% - 50%)"] {
        let calculation =
            CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let exact = CssSpecifiedNonNegativePercentage::try_from_calculation(calculation).unwrap();
        assert!(exact.calculation().is_some());
        assert!(exact.literal_component().is_none());
        let value = CssFontWidth::Percentage(exact);
        assert!(value.serialize_specified().is_ok());
        for checked in [false, true] {
            let parsed = width("font-width", text, checked);
            assert_eq!(parsed, value);
            let CssFontWidth::Percentage(parsed) = parsed else {
                panic!("symbolic percentage width")
            };
            assert!(parsed.calculation().is_some());
            assert_eq!(
                parsed.serialize_specified().unwrap(),
                value.serialize_specified().unwrap()
            );
        }
    }
}

#[test]
fn scalar_and_width_serialization_enforce_input_projection_and_byte_budgets() {
    let value = CssFontWidth::Percentage(scalar("75%"));
    assert_eq!(value.serialize_specified().unwrap(), "75%");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "75%"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 3),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 3),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 2),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let scalar = scalar("75%");
    assert_eq!(
        scalar
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "75%"
    );
    assert_eq!(
        scalar
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 2))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
