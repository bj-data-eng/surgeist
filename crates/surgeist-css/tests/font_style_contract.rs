#![forbid(unsafe_code)]

//! Authored Fonts 4 property, shared angle, and shorthand contracts.

use surgeist_css::{
    CssAngleCalculation, CssComponentValue, CssComponentValueRef, CssComponentValues,
    CssContributions, CssExpansion, CssExpansionErrorKind, CssFontObliqueAngle, CssFontStyle,
    CssFontStyleKeyword, CssFontValue, CssImportance, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandValueRef, CssNumericConstructionErrorKind,
    CssPropertyGrammar, CssPropertyKindRef, CssPropertyValueErrorKind, CssRecoveryAction,
    CssSerializedOrigin, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef, expand_declaration,
    parse_component_values, parse_property_value_for_grammar, parse_style_attribute,
};

fn angle(text: &str) -> CssFontObliqueAngle {
    CssFontObliqueAngle::try_from_component(CssComponentValue::try_token(text).unwrap()).unwrap()
}

fn property(text: &str) -> CssFontStyle {
    let report = parse_style_attribute(&format!("font-style:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::FontStyle(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font-style wrapper")
    };
    value.current().clone()
}

fn checked(
    components: CssComponentValues,
) -> Result<surgeist_css::CssDeclaration, surgeist_css::CssPropertyValueParseError> {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name("font-style").unwrap(),
        components,
        CssImportance::Normal,
    )
}

#[test]
fn keywords_and_optional_oblique_angle_preserve_authored_distinctions() {
    for (text, expected) in [
        ("normal", CssFontStyleKeyword::Normal),
        ("italic", CssFontStyleKeyword::Italic),
        ("left", CssFontStyleKeyword::Left),
        ("right", CssFontStyleKeyword::Right),
    ] {
        assert_eq!(property(text), CssFontStyle::Keyword(expected));
        assert_eq!(
            CssFontStyle::Keyword(expected)
                .serialize_specified()
                .unwrap(),
            text
        );
    }
    assert_eq!(property("oblique"), CssFontStyle::Oblique { angle: None });
    assert_eq!(
        property("oblique").serialize_specified().unwrap(),
        "oblique"
    );
    let explicit = property("oblique 14deg");
    assert_eq!(
        explicit,
        CssFontStyle::Oblique {
            angle: Some(angle("14deg"))
        }
    );
    assert_ne!(explicit, property("oblique"));
    assert_eq!(explicit.serialize_specified().unwrap(), "oblique 14deg");
}

#[test]
fn exact_lexical_bounds_and_radian_precision_retain_original_components() {
    for text in [
        "90deg",
        "-90deg",
        "+90deg",
        "100grad",
        "-100grad",
        ".25turn",
        "-.25turn",
        "1e-999deg",
        "-1e-999grad",
        "1e-999rad",
        "0e999999999999999999999999rad",
        "-0deg",
        "+0turn",
        "1.5707963267948966rad",
        "-1.5707963267948966rad",
    ] {
        let value = angle(text);
        assert!(value.literal_component().is_some(), "{text}");
        assert!(value.calculation().is_none(), "{text}");
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic, "{text}");
        assert!(
            property(&format!("oblique {text}"))
                .serialize_specified()
                .unwrap()
                .starts_with("oblique "),
            "{text}"
        );
    }
    for text in [
        "90.0000000000000000000000000000001deg",
        "-90.0000000000000000000000000000001deg",
        "100.0000000000000000000000000000001grad",
        ".2500000000000000000000000000001turn",
        "1e999deg",
        "1e999rad",
        "1.5707963267948968rad",
        "-1.5707963267948968rad",
    ] {
        let error =
            CssFontObliqueAngle::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::OutOfRange,
            "{text}"
        );
    }
    let rad = angle("1.5707963267948966rad");
    let component = rad.literal_component().unwrap();
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        component.view()
    else {
        panic!("retained dimension")
    };
    assert_eq!(number.representation(), "1.5707963267948966");
    assert_eq!(unit, "rad");
    assert_eq!(rad.serialize_specified().unwrap(), "1.5707963267948966rad");
    let same_binary64 = angle("1.5707963267948965rad");
    assert_eq!(
        "1.5707963267948965".parse::<f64>().unwrap(),
        "1.5707963267948966".parse::<f64>().unwrap(),
    );
    assert_ne!(rad, same_binary64);
    assert_eq!(
        same_binary64.serialize_specified().unwrap(),
        "1.5707963267948965rad"
    );
    let long = angle("0.0000000000000000000000000000000000000001turn");
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        long.literal_component().unwrap().view()
    else {
        panic!("retained tiny dimension")
    };
    assert_eq!(
        number.representation(),
        "0.0000000000000000000000000000000000000001"
    );
    assert_eq!(unit, "turn");
}

#[test]
fn wrong_domains_and_bare_math_roots_cannot_enter_the_angle_model() {
    for text in ["0", "10", "10%", "10px", "10frobs"] {
        let error =
            CssFontObliqueAngle::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch,
            "{text}"
        );
    }
    for text in ["0deg", "91deg"] {
        let calculation =
            CssAngleCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let result = CssFontObliqueAngle::try_from_calculation(calculation);
        assert_eq!(result.is_ok(), text == "0deg", "{text}");
    }
    for text in ["calc(91deg)", "calc(-100deg)"] {
        let calculation =
            CssAngleCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let value = CssFontObliqueAngle::try_from_calculation(calculation).unwrap();
        assert!(value.calculation().is_some(), "{text}");
        assert!(value.literal_component().is_none(), "{text}");
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            property(&format!("oblique {text}")),
            CssFontStyle::Oblique { angle: Some(value) }
        );
    }
    for text in [
        "oblique 0",
        "oblique 1px",
        "oblique 1%",
        "oblique calc(10px)",
        "oblique 10deg 20deg",
    ] {
        let report = parse_style_attribute(&format!("font-style:{text}"));
        assert!(report.syntax().is_empty(), "{text}");
        assert_eq!(report.diagnostics().len(), 1, "{text}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{text}"
        );
    }
}

#[test]
fn recovered_math_requires_parser_admission_before_typed_transfer() {
    let report = parse_style_attribute("font-style:oblique calc(10deg");
    assert_eq!(report.syntax().len(), 1);
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::FontStyle(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("retained style")
    };
    let CssFontStyle::Oblique {
        angle: Some(admitted),
    } = value.current()
    else {
        panic!("retained oblique math")
    };
    let calculation = admitted.calculation().expect("typed recovered calculation");
    let error =
        CssAngleCalculation::try_from_components(calculation.components().clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    let transferred = CssFontObliqueAngle::try_from_calculation(calculation.clone()).unwrap();
    assert_eq!(transferred.origin(), admitted.origin());
    assert_eq!(transferred.serialize_specified().unwrap(), "calc(10deg)");
}

#[test]
fn checked_values_and_pending_reentry_preserve_real_origins_and_importance() {
    let parsed = checked(parse_component_values("oblique 12.50deg ").unwrap()).unwrap();
    let CssKnownPropertyValueRef::FontStyle(value) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("style")
    };
    let CssFontStyle::Oblique { angle: Some(value) } = value.current() else {
        panic!("angle")
    };
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed token")
    };
    assert_eq!(origin.source().as_str(), "oblique 12.50deg ");
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(origin.span().end().byte_offset().value(), 16);

    let programmatic = checked(
        CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("oblique").unwrap(),
            CssComponentValue::try_token("12deg").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let CssKnownPropertyValueRef::FontStyle(value) =
        programmatic.known().unwrap().property_value().unwrap()
    else {
        panic!("style")
    };
    let CssFontStyle::Oblique { angle: Some(value) } = value.current() else {
        panic!("angle")
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);

    let invalid = checked(
        CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("oblique").unwrap(),
            CssComponentValue::try_token("91deg").unwrap(),
        ])
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
        CssComponentValue::try_ident("oblique").unwrap(),
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_token("0deg").unwrap(),
    ])
    .unwrap();
    assert!(checked(adjacent).is_err());

    let report = parse_style_attribute("font-style:env(style)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("pending property")
    };
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("oblique -20grad").unwrap())
        .unwrap()
    else {
        panic!("reentry")
    };
    let [item] = values.items() else {
        panic!("one contribution")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    let CssLonghandValueRef::FontStyle(CssFontStyle::Oblique { angle: Some(value) }) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("typed contribution")
    };
    assert_eq!(value.serialize_specified().unwrap(), "-20grad");
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed replacement angle")
    };
    assert_eq!(origin.source().as_str(), "oblique -20grad");
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(origin.span().end().byte_offset().value(), 15);
    let error = handle
        .reenter(
            CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("oblique").unwrap(),
                CssComponentValue::try_token("91deg").unwrap(),
            ])
            .unwrap(),
        )
        .unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(mapped) = error.kind() else {
        panic!("invalid angle replacement")
    };
    assert!(matches!(
        mapped.kind(),
        CssPropertyValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        mapped.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

#[test]
fn initial_metadata_css_wide_and_shorthand_ambiguity_are_typed() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::FontStyle.metadata().unwrap().kind()
    else {
        panic!("longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("ordinary initial")
    };
    assert_eq!(
        initial.view(),
        CssLonghandValueRef::FontStyle(&CssFontStyle::Keyword(CssFontStyleKeyword::Normal))
    );
    for source in ["font-style:inherit", "font-style:unset!important"] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1);
    }
    for (source, expected_angle) in [
        ("font:oblique 10deg 16px serif", Some("10deg")),
        ("font:oblique calc(10deg) 16px serif", Some("calc(10deg)")),
        ("font:oblique calc(16px) serif", None),
        ("font:oblique calc(700) calc(16px) serif", None),
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
        let Some(CssFontStyle::Oblique { angle }) = font.style() else {
            panic!("oblique style")
        };
        assert_eq!(
            angle
                .as_ref()
                .map(|angle| angle.serialize_specified().unwrap())
                .as_deref(),
            expected_angle
        );
    }
    for source in [
        "font:italic oblique 16px serif",
        "font:oblique 10deg italic 16px serif",
    ] {
        assert!(!parse_style_attribute(source).is_clean(), "{source}");
    }
}

#[test]
fn scalar_and_property_serialization_charge_budgets_atomically() {
    let value = property("oblique 10deg");
    assert_eq!(value.serialize_specified().unwrap(), "oblique 10deg");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(8, 8, 4))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value.serialize_specified().unwrap(), "oblique 10deg");
    let value = angle("-10grad");
    assert_eq!(value.serialize_specified().unwrap(), "-10grad");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(8, 8, 3))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
