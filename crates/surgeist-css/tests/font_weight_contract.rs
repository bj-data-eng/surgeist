#![forbid(unsafe_code)]

//! Exact authored font-weight property and common scalar contracts (Fonts 4 §2.2).

use surgeist_css::{
    CssAbsoluteFontWeight as Absolute, CssComponentValue, CssComponentValues, CssContributions,
    CssExpansion, CssExpansionErrorKind, CssFontStyle, CssFontValue, CssFontWeight,
    CssFontWeightNumber, CssImportance, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandValueRef, CssNumberCalculation,
    CssNumericConstructionErrorKind, CssPropertyGrammar, CssPropertyKindRef,
    CssPropertyValueErrorKind, CssSerializedOrigin, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, expand_declaration,
    parse_component_values, parse_property_value_for_grammar, parse_style_attribute,
};

fn number(text: &str) -> CssFontWeightNumber {
    let component = CssComponentValue::try_number(text).unwrap();
    CssFontWeightNumber::try_from_component(component).unwrap()
}

fn property(text: &str) -> CssFontWeight {
    let report = parse_style_attribute(&format!("font-weight:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one weight declaration")
    };
    let CssKnownPropertyValueRef::FontWeight(weight) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed weight property")
    };
    weight.value().clone()
}

fn checked(
    components: CssComponentValues,
) -> Result<surgeist_css::CssDeclaration, surgeist_css::CssPropertyValueParseError> {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name("font-weight").unwrap(),
        components,
        CssImportance::Normal,
    )
}

fn checked_weight(components: CssComponentValues) -> CssFontWeight {
    let declaration = checked(components).unwrap();
    let CssKnownPropertyValueRef::FontWeight(weight) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed checked weight")
    };
    weight.value().clone()
}

#[test]
fn exact_fractional_and_exponent_bounds_ignore_float_rounding() {
    for (text, canonical) in [
        ("1", "1"),
        ("1000", "1000"),
        ("725.25", "725.25"),
        ("1e3", "1000"),
        ("10e-1", "1"),
    ] {
        let value = number(text);
        assert!(value.literal_component().is_some(), "{text}");
        assert!(value.calculation().is_none(), "{text}");
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic, "{text}");
        assert_eq!(value.serialize_specified().unwrap(), canonical, "{text}");
        assert_eq!(
            property(text),
            CssFontWeight::Absolute(Absolute::Number(value)),
            "{text}"
        );
    }
    for text in [
        "0",
        "1001",
        "0.999999999999999999999999",
        "1000.000000000000000000001",
        "1e-999",
        "1e999",
        "-1e999",
    ] {
        let component = CssComponentValue::try_number(text).unwrap();
        let error = CssFontWeightNumber::try_from_component(component).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::OutOfRange,
            "{text}"
        );
        let report = parse_style_attribute(&format!("font-weight:{text}"));
        assert!(!report.is_clean(), "{text}");
        assert!(report.syntax().is_empty(), "{text}");
    }
}

#[test]
fn wrong_domain_and_bare_math_roots_cannot_bypass_literal_bounds() {
    for text in ["10%", "10px"] {
        let error =
            CssFontWeightNumber::try_from_component(CssComponentValue::try_token(text).unwrap())
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(error.origin(), Some(&CssValueOrigin::Programmatic));
    }
    for text in ["0", "1001"] {
        let calculation =
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let error = CssFontWeightNumber::try_from_calculation(calculation).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::OutOfRange,
            "{text}"
        );
    }
    let calculation =
        CssNumberCalculation::try_from_components(parse_component_values("calc(0)").unwrap())
            .unwrap();
    let value = CssFontWeightNumber::try_from_calculation(calculation).unwrap();
    assert!(value.literal_component().is_none());
    assert!(value.calculation().is_some());
    assert_eq!(value.serialize_specified().unwrap(), "calc(0)");
    assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    for text in ["calc(10px)", "calc(10%)"] {
        assert!(
            checked(parse_component_values(text).unwrap()).is_err(),
            "{text}"
        );
        let report = parse_style_attribute(&format!("font-weight:{text}"));
        assert!(!report.is_clean(), "{text}");
    }
}

#[test]
fn property_keywords_initial_and_direct_expansion_remain_typed() {
    for (text, expected) in [
        ("normal", CssFontWeight::Absolute(Absolute::Normal)),
        ("bold", CssFontWeight::Absolute(Absolute::Bold)),
        ("bolder", CssFontWeight::Bolder),
        ("lighter", CssFontWeight::Lighter),
    ] {
        assert_eq!(property(text), expected);
        assert_eq!(expected.serialize_specified().unwrap(), text);
    }
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::FontWeight.metadata().unwrap().kind()
    else {
        panic!("weight longhand metadata")
    };
    assert!(metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("ordinary initial")
    };
    assert_eq!(
        value.view(),
        CssLonghandValueRef::FontWeight(&CssFontWeight::Absolute(Absolute::Normal))
    );
    let report = parse_style_attribute("font-weight:bolder!important");
    let source = &report.syntax()[0];
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("terminal font-weight expansion")
    };
    let [item] = values.items() else {
        panic!("single weight contribution")
    };
    assert_eq!(item.property(), CssKnownProperty::FontWeight);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(
        item.ordinary_value().unwrap().view(),
        CssLonghandValueRef::FontWeight(&CssFontWeight::Bolder)
    );
}

#[test]
fn checked_and_pending_property_numeric_origins_remain_exact() {
    let parsed = checked_weight(parse_component_values("725.250 ").unwrap());
    let CssFontWeight::Absolute(Absolute::Number(parsed_number)) = parsed else {
        panic!("parsed numeric weight")
    };
    let CssValueOrigin::Parsed(origin) = parsed_number.origin() else {
        panic!("numeric token keeps parsed origin")
    };
    assert_eq!(origin.source().as_str(), "725.250 ");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 7);

    let programmatic = CssComponentValue::try_number("725.25").unwrap();
    let weight = checked_weight(CssComponentValues::try_new(vec![programmatic]).unwrap());
    let CssFontWeight::Absolute(Absolute::Number(number)) = weight else {
        panic!("programmatic numeric weight")
    };
    assert_eq!(number.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(number.serialize_specified().unwrap(), "725.25");

    let invalid =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("1001").unwrap()]).unwrap();
    let error = checked(invalid).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );

    let report = parse_style_attribute("font-weight:env(weight)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("pending weight property")
    };
    let mut mixed = parse_component_values("725.250 ").unwrap().items().to_vec();
    let CssContributions::Longhands(values) = handle
        .reenter(CssComponentValues::try_new(mixed.clone()).unwrap())
        .unwrap()
    else {
        panic!("reentered weight")
    };
    let CssLonghandValueRef::FontWeight(CssFontWeight::Absolute(Absolute::Number(number))) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("reentered numeric weight")
    };
    let CssValueOrigin::Parsed(origin) = number.origin() else {
        panic!("original parsed origin")
    };
    assert_eq!(origin.source().as_str(), "725.250 ");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 7);
    mixed.clear();
    mixed.push(CssComponentValue::try_number("725.25").unwrap());
    let CssContributions::Longhands(values) = handle
        .reenter(CssComponentValues::try_new(mixed).unwrap())
        .unwrap()
    else {
        panic!("reentered programmatic weight")
    };
    let CssLonghandValueRef::FontWeight(CssFontWeight::Absolute(Absolute::Number(number))) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("programmatic replacement")
    };
    assert_eq!(number.origin(), &CssValueOrigin::Programmatic);

    let invalid =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("1001").unwrap()]).unwrap();
    let error = handle.reenter(invalid).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(mapped) = error.kind() else {
        panic!("invalid pending replacement")
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
fn adjacent_programmatic_number_components_do_not_merge_into_one_weight() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_number("00").unwrap(),
    ])
    .unwrap();
    assert!(checked(components.clone()).is_err());
    let report = parse_style_attribute("font-weight:env(weight)");
    let CssExpansion::Pending(handle) = expand_declaration(&report.syntax()[0]).unwrap() else {
        panic!("pending weight")
    };
    assert!(matches!(
        handle.reenter(components).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
}

#[test]
fn shorthand_borrows_fractional_and_symbolic_weight_without_claiming_other_grammar() {
    for (source, expected) in [
        ("font: 725.25 16px serif", "725.25"),
        ("font: calc(725.25) 16px serif", "calc(725.25)"),
        ("font: bolder 16px serif", "bolder"),
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::Font(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("font shorthand")
        };
        let CssFontValue::Explicit(explicit) = wrapper.font() else {
            panic!("explicit font")
        };
        assert_eq!(
            explicit.weight().unwrap().serialize_specified().unwrap(),
            expected
        );
    }
    let source = "font: oblique 10deg 16px serif";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Font(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font shorthand")
    };
    let CssFontValue::Explicit(explicit) = wrapper.font() else {
        panic!("explicit font")
    };
    let Some(CssFontStyle::Oblique { angle: Some(angle) }) = explicit.style() else {
        panic!("authored oblique angle")
    };
    assert_eq!(angle.serialize_specified().unwrap(), "10deg");
    assert!(explicit.weight().is_none());

    for source in [
        "font: bold 700 16px serif",
        "font: oblique 10deg italic 16px serif",
    ] {
        let report = parse_style_attribute(source);
        assert!(!report.is_clean(), "{source}");
    }
}

#[test]
fn specified_serialization_limits_are_atomic() {
    let value = CssFontWeight::Absolute(Absolute::Number(number("725.25")));
    let error = value
        .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(8, 8, 3))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value.serialize_specified().unwrap(), "725.25");
}
