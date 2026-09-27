#![forbid(unsafe_code)]

//! Typed CSS 2.1 line-height and shared font slash contracts.

use surgeist_css::{
    CssComponentValue, CssComponentValues, CssContributionValueRef, CssContributions, CssExpansion,
    CssExpansionErrorKind, CssFontValue, CssGlobalKeyword, CssImportance, CssInitialValueRef,
    CssKnownProperty, CssKnownPropertyValueRef, CssLengthPercentageCalculation, CssLineHeight,
    CssLonghandValueRef, CssNumberCalculation, CssNumericConstructionErrorKind, CssPropertyGrammar,
    CssPropertyKindRef, CssPropertyValueErrorKind, CssSerializedOrigin,
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedNonNegativeNumber,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    expand_declaration, parse_component_values, parse_property_value_for_grammar,
    parse_style_attribute,
};

fn property(text: &str) -> CssLineHeight {
    let report = parse_style_attribute(&format!("line-height:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::LineHeight(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("line height")
    };
    value.line_height().clone()
}

fn checked(
    components: CssComponentValues,
) -> Result<surgeist_css::CssDeclaration, surgeist_css::CssPropertyValueParseError> {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name("line-height").unwrap(),
        components,
        CssImportance::Normal,
    )
}

#[test]
fn authored_branches_keep_number_zero_distinct_and_math_symbolic() {
    assert_eq!(property("normal"), CssLineHeight::Normal);
    for (text, canonical) in [
        ("0", "0"),
        ("-0", "0"),
        ("1e999", &format!("1{}", "0".repeat(999))),
    ] {
        let CssLineHeight::Number(value) = property(text) else {
            panic!("number {text}")
        };
        assert!(value.literal_component().is_some());
        assert_eq!(value.serialize_specified().unwrap(), canonical);
    }
    for text in ["0px", "0%", "120%", "1em"] {
        let CssLineHeight::LengthPercentage(value) = property(text) else {
            panic!("length/percentage {text}")
        };
        assert!(value.literal_component().is_some());
    }
    assert_ne!(property("0"), property("0px"));
    assert_ne!(property("0"), property("0%"));
    for (text, number) in [("calc(0)", true), ("calc(0px)", false), ("calc(0%)", false)] {
        match property(text) {
            CssLineHeight::Number(value) if number => assert!(value.calculation().is_some()),
            CssLineHeight::LengthPercentage(value) if !number => {
                assert!(value.calculation().is_some())
            }
            _ => panic!("wrong math domain: {text}"),
        }
    }
    let CssLineHeight::Number(negative_math) = property("calc(-1)") else {
        panic!("number math")
    };
    assert!(negative_math.calculation().is_some());
    assert_eq!(negative_math.serialize_specified().unwrap(), "calc(-1)");
    let CssLineHeight::LengthPercentage(mixed) = property("calc(2px + 3%)") else {
        panic!("mixed")
    };
    assert_eq!(mixed.serialize_specified().unwrap(), "calc(3% + 2px)");
    for invalid in ["calc(1 + 2%)", "-1e-999", "-1e-999px", "-1e-999%"] {
        let report = parse_style_attribute(&format!("color:red;line-height:{invalid};color:blue"));
        assert_eq!(report.syntax().len(), 2, "{invalid}");
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
    }
}

#[test]
fn parsed_and_programmatic_values_have_distinct_origins_but_semantic_equality() {
    let parsed = property("1.250");
    let CssLineHeight::Number(parsed_scalar) = &parsed else {
        panic!("parsed number")
    };
    let CssValueOrigin::Parsed(origin) = parsed_scalar.origin() else {
        panic!("source origin")
    };
    assert_eq!(origin.source().as_str(), "line-height:1.250");
    assert_eq!(origin.span().start().byte_offset().value(), 12);
    let programmatic_scalar = CssSpecifiedNonNegativeNumber::try_from_component(
        CssComponentValue::try_number("1.250").unwrap(),
    )
    .unwrap();
    assert_ne!(parsed_scalar, &programmatic_scalar);
    assert_eq!(parsed, CssLineHeight::Number(programmatic_scalar));
    assert_ne!(parsed, property("1.25"));
    assert_eq!(
        parsed.serialize_specified().unwrap(),
        property("1.25").serialize_specified().unwrap()
    );

    let length = property("1.250px");
    let CssLineHeight::LengthPercentage(parsed_length) = &length else {
        panic!("parsed length")
    };
    let programmatic_length = CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_token("1.250px").unwrap(),
    )
    .unwrap();
    assert_ne!(parsed_length, &programmatic_length);
    assert_eq!(length, CssLineHeight::LengthPercentage(programmatic_length));
    let checked_value = checked(parse_component_values("1.250").unwrap()).unwrap();
    let CssKnownPropertyValueRef::LineHeight(value) =
        checked_value.known().unwrap().property_value().unwrap()
    else {
        panic!("checked")
    };
    assert_eq!(value.line_height(), &parsed);
    let invalid = checked(
        CssComponentValues::try_new(vec![CssComponentValue::try_number("-1e-999").unwrap()])
            .unwrap(),
    )
    .unwrap_err();
    assert!(matches!(
        invalid.kind(),
        CssPropertyValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        invalid.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
    let invalid = checked(parse_component_values("  -1e-999").unwrap()).unwrap_err();
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = invalid.origin() else {
        panic!("parsed invalid token origin")
    };
    assert_eq!(origin.source().as_str(), "  -1e-999");
    assert_eq!(origin.span().start().byte_offset().value(), 2);
}

#[test]
fn inherited_initial_current_wrapper_and_global_contributions_are_exact() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::LineHeight.metadata().unwrap().kind()
    else {
        panic!("longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("initial")
    };
    assert_eq!(
        initial.view(),
        CssLonghandValueRef::LineHeight(&CssLineHeight::Normal)
    );
    let report = parse_style_attribute("line-height:1.250!important");
    let source = &report.syntax()[0];
    let CssKnownPropertyValueRef::LineHeight(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("current")
    };
    assert_eq!(value.line_height(), &property("1.250"));
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("contribution")
    };
    let [item] = values.items() else {
        panic!("one contribution")
    };
    assert_eq!(item.property(), CssKnownProperty::LineHeight);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(
        item.ordinary_value().unwrap().view(),
        CssLonghandValueRef::LineHeight(value.line_height())
    );
    for (text, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let report = parse_style_attribute(&format!("line-height:{text}"));
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("global")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(
            values.items()[0].value(),
            CssContributionValueRef::Global(keyword)
        );
    }
}

#[test]
fn pending_reentry_and_shorthand_share_strict_authored_grammar() {
    let report = parse_style_attribute("line-height:env(lh)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("pending")
    };
    let replacement = parse_component_values("1.250 ").unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("ordinary")
    };
    let [item] = values.items() else {
        panic!("one replacement")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssLonghandValueRef::LineHeight(CssLineHeight::Number(value)) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("reentered number")
    };
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("replacement origin")
    };
    assert_eq!(origin.source().as_str(), "1.250 ");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    let programmatic =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("2.50").unwrap()]).unwrap();
    let CssContributions::Longhands(values) = handle.reenter(programmatic).unwrap() else {
        panic!("programmatic replacement")
    };
    let CssLonghandValueRef::LineHeight(CssLineHeight::Number(value)) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("programmatic number")
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(value.serialize_specified().unwrap(), "2.5");
    let invalid =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("-1e-999").unwrap()])
            .unwrap();
    let error = handle.reenter(invalid).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(mapped) = error.kind() else {
        panic!("mapped invalid")
    };
    assert!(matches!(
        mapped.kind(),
        CssPropertyValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        mapped.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
    for tokens in [("1", "0px"), ("1", ".2")] {
        let adjacent = CssComponentValues::try_new(vec![
            CssComponentValue::try_token(tokens.0).unwrap(),
            CssComponentValue::try_token(tokens.1).unwrap(),
        ])
        .unwrap();
        assert!(handle.reenter(adjacent).is_err(), "adjacent {tokens:?}");
    }
    for invalid in ["-1e-999", "var(--still)", "env(still)", "calc(1 + 2%)"] {
        assert!(
            handle
                .reenter(parse_component_values(invalid).unwrap())
                .is_err(),
            "{invalid}"
        );
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--still)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for (text, expected_number) in [
        ("16px/0 serif", true),
        ("16px/0px serif", false),
        ("16px/120% serif", false),
        ("16px/calc(-1) serif", true),
    ] {
        let report = parse_style_attribute(&format!("font:{text}"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::Font(font) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("font")
        };
        let CssFontValue::Explicit(font) = font.font() else {
            panic!("explicit")
        };
        assert_eq!(font.families().families()[0].as_str(), "serif");
        match font.line_height().unwrap() {
            CssLineHeight::Number(_) if expected_number => {}
            CssLineHeight::LengthPercentage(_) if !expected_number => {}
            _ => panic!("shorthand line-height branch: {text}"),
        }
    }
    for text in [
        "16px/-1e-999 serif",
        "16px/calc(1 + 2%) serif",
        "16px/math serif",
    ] {
        let report = parse_style_attribute(&format!("font:{text}"));
        assert!(!report.is_clean(), "{text}");
    }
}

#[test]
fn recovered_math_is_retained_by_raw_parse_but_rejected_by_strict_construction_and_reentry() {
    for (text, number) in [("calc(1", true), ("calc(1px", false)] {
        let source = format!("line-height:{text}");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{text}");
        assert!(!report.is_clean(), "{text}");
        let CssKnownPropertyValueRef::LineHeight(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("retained")
        };
        match value.line_height() {
            CssLineHeight::Number(admitted) if number => {
                let calculation = admitted.calculation().expect("number math");
                assert_eq!(
                    CssNumberCalculation::try_from_components(calculation.components().clone())
                        .unwrap_err()
                        .kind(),
                    &CssNumericConstructionErrorKind::RecoveredComponent
                );
                let transferred =
                    CssSpecifiedNonNegativeNumber::try_from_calculation(calculation.clone())
                        .unwrap();
                assert_eq!(transferred.origin(), admitted.origin());
                assert_eq!(transferred.serialize_specified().unwrap(), "calc(1)");
            }
            CssLineHeight::LengthPercentage(admitted) if !number => {
                let calculation = admitted.calculation().expect("length math");
                assert_eq!(
                    CssLengthPercentageCalculation::try_from_components(
                        calculation.components().clone()
                    )
                    .unwrap_err()
                    .kind(),
                    &CssNumericConstructionErrorKind::RecoveredComponent
                );
                let transferred = CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                    calculation.clone(),
                )
                .unwrap();
                assert_eq!(transferred.origin(), admitted.origin());
                assert_eq!(transferred.serialize_specified().unwrap(), "calc(1px)");
            }
            _ => panic!("retained math domain {text}"),
        }
        let pending = parse_style_attribute("line-height:env(lh)");
        let CssExpansion::Pending(handle) = expand_declaration(&pending.syntax()[0]).unwrap()
        else {
            panic!("pending")
        };
        assert!(
            handle
                .reenter(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
    }
}

#[test]
fn bounded_serialization_fails_atomically_for_both_branches() {
    let number = property("12.5");
    let length = property("12.5px");
    for value in [&number, &length] {
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 10, 20),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(10, 0, 20),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(10, 10, 3),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                expected
            );
        }
    }
    assert_eq!(number.serialize_specified().unwrap(), "12.5");
    assert_eq!(length.serialize_specified().unwrap(), "12.5px");
    let error = CssSpecifiedNonNegativeNumber::try_from_component(
        CssComponentValue::try_number("-1e-999").unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
}
