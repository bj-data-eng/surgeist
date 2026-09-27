#![forbid(unsafe_code)]

//! Exact authored Fonts 4 font-size and the shared explicit font size component.

use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssContributionValueRef,
    CssContributions, CssExpansion, CssExpansionErrorKind, CssFontSize, CssFontValue,
    CssGlobalKeyword, CssImportance, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLengthPercentageCalculation, CssLonghandValueRef,
    CssNumericConstructionErrorKind, CssPropertyGrammar, CssPropertyKindRef,
    CssPropertyValueErrorKind, CssRecoveryAction, CssSerializedOrigin,
    CssSpecifiedNonNegativeLengthPercentage as SizeLengthPercentage,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    CssValueTokenRef, expand_declaration, parse_component_values, parse_property_value_for_grammar,
    parse_style_attribute,
};

fn scalar(text: &str) -> SizeLengthPercentage {
    SizeLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap()).unwrap()
}

fn property(text: &str) -> CssFontSize {
    let report = parse_style_attribute(&format!("font-size:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::FontSize(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font-size wrapper")
    };
    value.size().clone()
}

fn checked(
    components: CssComponentValues,
) -> Result<surgeist_css::CssDeclaration, surgeist_css::CssPropertyValueParseError> {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name("font-size").unwrap(),
        components,
        CssImportance::Normal,
    )
}

#[test]
fn absolute_relative_and_math_keywords_keep_distinct_typed_values() {
    for (text, expected) in [
        ("xx-small", CssFontSize::XxSmall),
        ("x-small", CssFontSize::XSmall),
        ("small", CssFontSize::Small),
        ("medium", CssFontSize::Medium),
        ("large", CssFontSize::Large),
        ("x-large", CssFontSize::XLarge),
        ("xx-large", CssFontSize::XxLarge),
        ("xxx-large", CssFontSize::XxxLarge),
        ("larger", CssFontSize::Larger),
        ("smaller", CssFontSize::Smaller),
        ("math", CssFontSize::Math),
    ] {
        assert_eq!(property(text), expected, "{text}");
        assert_eq!(expected.serialize_specified().unwrap(), text, "{text}");
    }
    assert_ne!(CssFontSize::Math, property("calc(1px)"));
}

#[test]
fn shared_checked_payload_accepts_all_length_categories_and_exact_nonnegative_values() {
    for (authored, expected) in [
        ("0", "0"),
        ("-0", "0"),
        ("-0px", "0px"),
        ("-0%", "0%"),
        ("1.25px", "1.25px"),
        ("1em", "1em"),
        ("1rem", "1rem"),
        ("1ex", "1ex"),
        ("1ch", "1ch"),
        ("1lh", "1lh"),
        ("1vw", "1vw"),
        ("1vi", "1vi"),
        ("1cqw", "1cqw"),
        ("1cm", "1cm"),
        ("12.5%", "12.5%"),
    ] {
        let value = scalar(authored);
        assert!(value.literal_component().is_some(), "{authored}");
        assert!(value.calculation().is_none(), "{authored}");
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic, "{authored}");
        assert_eq!(value.serialize_specified().unwrap(), expected, "{authored}");
        assert_eq!(
            property(authored),
            CssFontSize::LengthPercentage(value),
            "{authored}"
        );
    }
    let huge = scalar("1e999px");
    let expected = format!("1{}px", "0".repeat(999));
    assert_eq!(huge.serialize_specified().unwrap(), expected);
    assert_eq!(property("1e999px"), CssFontSize::LengthPercentage(huge));
    assert_eq!(
        scalar("1e-999px").serialize_specified().unwrap(),
        format!("0.{}1px", "0".repeat(998))
    );
    assert_eq!(
        scalar("1e-999%").serialize_specified().unwrap(),
        format!("0.{}1%", "0".repeat(998))
    );
    let component = scalar("1e-999px");
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        component.literal_component().unwrap().view()
    else {
        panic!("exact dimension")
    };
    assert_eq!(number.representation(), "1e-999");
    assert_eq!(unit, "px");
}

#[test]
fn tiny_negatives_and_wrong_numeric_domains_fail_without_dropping_neighbors() {
    for text in [
        "-1e-999px",
        "-1e-999%",
        "-0.0000000000000000000001em",
        "-1cm",
    ] {
        let error =
            SizeLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::OutOfRange,
            "{text}"
        );
    }
    for text in ["1", "1e-999", "1deg", "1s", "1frobs"] {
        let error =
            SizeLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch,
            "{text}"
        );
    }
    for text in [
        "-1e-999px",
        "-1e-999%",
        "1",
        "1deg",
        "calc(1deg)",
        "math 16px",
    ] {
        let report = parse_style_attribute(&format!("color:red;font-size:{text};color:blue"));
        assert_eq!(report.syntax().len(), 2, "{text}");
        assert_eq!(report.diagnostics().len(), 1, "{text}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{text}"
        );
    }
}

#[test]
fn bare_calculation_roots_reenter_literal_admission_while_functions_stay_symbolic() {
    for (text, expected) in [("0px", "0px"), ("0", "0")] {
        let calculation = CssLengthPercentageCalculation::try_from_components(
            parse_component_values(text).unwrap(),
        )
        .unwrap();
        let value = SizeLengthPercentage::try_from_calculation(calculation).unwrap();
        assert!(value.literal_component().is_some(), "{text}");
        assert!(value.calculation().is_none(), "{text}");
        assert_eq!(value.serialize_specified().unwrap(), expected, "{text}");
    }
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("1e999%").unwrap(),
    )
    .unwrap();
    let value = SizeLengthPercentage::try_from_calculation(calculation).unwrap();
    assert!(value.literal_component().is_some());
    assert_eq!(
        value.serialize_specified().unwrap(),
        format!("1{}%", "0".repeat(999))
    );
    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("-1e-999px").unwrap(),
    )
    .unwrap();
    assert_eq!(
        SizeLengthPercentage::try_from_calculation(calculation)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange
    );
    for (text, canonical) in [
        ("calc(-1px)", "calc(-1px)"),
        ("calc(2px + 3%)", "calc(3% + 2px)"),
    ] {
        let calculation = CssLengthPercentageCalculation::try_from_components(
            parse_component_values(text).unwrap(),
        )
        .unwrap();
        let value = SizeLengthPercentage::try_from_calculation(calculation).unwrap();
        assert!(value.literal_component().is_none(), "{text}");
        assert!(value.calculation().is_some(), "{text}");
        assert_eq!(value.serialize_specified().unwrap(), canonical, "{text}");
        assert_eq!(
            property(text),
            CssFontSize::LengthPercentage(value),
            "{text}"
        );
    }
}

#[test]
fn current_wrapper_initial_and_importance_contributions_are_typed() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::FontSize.metadata().unwrap().kind()
    else {
        panic!("size longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("ordinary initial")
    };
    assert_eq!(
        initial.view(),
        CssLonghandValueRef::FontSize(&CssFontSize::Medium)
    );
    let report = parse_style_attribute("font-size:xxx-large!important");
    let source = &report.syntax()[0];
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one contribution")
    };
    let [item] = values.items() else {
        panic!("one longhand")
    };
    assert_eq!(item.property(), CssKnownProperty::FontSize);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(
        item.ordinary_value().unwrap().view(),
        CssLonghandValueRef::FontSize(&CssFontSize::XxxLarge)
    );
    for (text, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let report = parse_style_attribute(&format!("font-size:{text}"));
        assert!(report.is_clean(), "{text}");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&report.syntax()[0]).unwrap()
        else {
            panic!("global contribution")
        };
        assert_eq!(
            values.items()[0].value(),
            CssContributionValueRef::Global(keyword)
        );
    }
}

#[test]
fn checked_and_pending_reentry_preserve_original_and_programmatic_origins() {
    let parsed = checked(parse_component_values("1.250px ").unwrap()).unwrap();
    let CssKnownPropertyValueRef::FontSize(value) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("checked size")
    };
    let CssFontSize::LengthPercentage(number) = value.size() else {
        panic!("number")
    };
    let CssValueOrigin::Parsed(origin) = number.origin() else {
        panic!("original parsed origin")
    };
    assert_eq!(origin.source().as_str(), "1.250px ");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 7);
    assert_eq!(number.serialize_specified().unwrap(), "1.25px");

    let programmatic = CssComponentValue::try_dimension("1.25", "PX").unwrap();
    let checked_value = checked(CssComponentValues::try_new(vec![programmatic]).unwrap()).unwrap();
    let CssKnownPropertyValueRef::FontSize(value) =
        checked_value.known().unwrap().property_value().unwrap()
    else {
        panic!("programmatic size")
    };
    let CssFontSize::LengthPercentage(number) = value.size() else {
        panic!("number")
    };
    assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(number.serialize_specified().unwrap(), "1.25px");
    let invalid = checked(
        CssComponentValues::try_new(vec![CssComponentValue::try_token("-1e-999px").unwrap()])
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
    let adjacent = CssComponentValues::try_new(vec![
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_token("0px").unwrap(),
    ])
    .unwrap();
    assert!(checked(adjacent).is_err());

    let report = parse_style_attribute("font-size:env(size)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("pending size")
    };
    let replacement = parse_component_values("1.250px ").unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("ordinary reentry")
    };
    let [item] = values.items() else {
        panic!("one replacement")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssLonghandValueRef::FontSize(CssFontSize::LengthPercentage(number)) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("replacement size")
    };
    let CssValueOrigin::Parsed(origin) = number.origin() else {
        panic!("parsed replacement")
    };
    assert_eq!(origin.source().as_str(), "1.250px ");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 7);
    let replacement =
        CssComponentValues::try_new(vec![CssComponentValue::try_token("2em").unwrap()]).unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement).unwrap() else {
        panic!("programmatic reentry")
    };
    let CssLonghandValueRef::FontSize(CssFontSize::LengthPercentage(number)) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("programmatic size")
    };
    assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    let error = handle
        .reenter(
            CssComponentValues::try_new(vec![CssComponentValue::try_token("-1e-999px").unwrap()])
                .unwrap(),
        )
        .unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(mapped) = error.kind() else {
        panic!("invalid reentry")
    };
    assert!(matches!(
        mapped.kind(),
        CssPropertyValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        mapped.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--still-pending)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
}

#[test]
fn recovered_math_requires_parser_admission_before_typed_transfer() {
    let report = parse_style_attribute("font-size:calc(1px");
    assert_eq!(report.syntax().len(), 1);
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::FontSize(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("retained size")
    };
    let CssFontSize::LengthPercentage(admitted) = value.size() else {
        panic!("retained calculation")
    };
    let calculation = admitted.calculation().expect("typed recovered math");
    let error =
        CssLengthPercentageCalculation::try_from_components(calculation.components().clone())
            .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    let transferred = SizeLengthPercentage::try_from_calculation(calculation.clone()).unwrap();
    assert_eq!(transferred.origin(), admitted.origin());
    assert_eq!(transferred.serialize_specified().unwrap(), "calc(1px)");
}

#[test]
fn shorthand_preserves_size_math_line_height_and_family_boundaries() {
    for (source, expected_size, expected_family, has_line_height) in [
        (
            "font:xxx-large serif",
            CssFontSize::XxxLarge,
            "serif",
            false,
        ),
        ("font:math serif", CssFontSize::Math, "serif", false),
        (
            "font:16px math",
            CssFontSize::LengthPercentage(scalar("16px")),
            "math",
            false,
        ),
        (
            "font:oblique calc(10deg) calc(350 + 50) calc(16px + 2px)/1.2 serif",
            CssFontSize::LengthPercentage(
                SizeLengthPercentage::try_from_calculation(
                    CssLengthPercentageCalculation::try_from_components(
                        parse_component_values("calc(16px + 2px)").unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
            "serif",
            true,
        ),
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("font")
        };
        let CssFontValue::Explicit(font) = value.font() else {
            panic!("explicit font")
        };
        assert_eq!(font.size(), &expected_size, "{source}");
        let [family] = font.families().families() else {
            panic!("one family")
        };
        assert_eq!(family.as_str(), expected_family, "{source}");
        assert_eq!(font.line_height().is_some(), has_line_height, "{source}");
    }
    for source in [
        "font:math",
        "font:xxx-large",
        "font:small-caps small-caps 16px serif",
    ] {
        assert!(!parse_style_attribute(source).is_clean(), "{source}");
    }
}

#[test]
fn specified_serialization_uses_one_bounded_budget_and_remains_atomic() {
    let value = property("1e999px");
    let expected = format!("1{}px", "0".repeat(999));
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                1,
                1,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, expected_kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected_kind
        );
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(
        CssFontSize::Math
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(CssFontSize::Math.serialize_specified().unwrap(), "math");
}
