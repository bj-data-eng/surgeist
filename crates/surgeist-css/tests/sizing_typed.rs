#![forbid(unsafe_code)]

use surgeist_css::{
    CssBoxSize, CssCalcSize, CssKnownPropertyValueRef, CssMaxSizeValue, CssSizeValue,
    CssSpecifiedNonNegativeLengthPercentage, parse_component_values, parse_style_attribute,
};

fn component(text: &str) -> surgeist_css::CssComponentValue {
    parse_component_values(text).unwrap().items()[0].clone()
}

fn size(name: &str, text: &str) -> CssSizeValue {
    let source = format!("{name}:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let value = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap();
    match value {
        CssKnownPropertyValueRef::Width(value) => value.current().clone(),
        CssKnownPropertyValueRef::Height(value) => value.current().clone(),
        CssKnownPropertyValueRef::InlineSize(value) => value.current().clone(),
        CssKnownPropertyValueRef::BlockSize(value) => value.current().clone(),
        CssKnownPropertyValueRef::MinWidth(value) => value.current().clone(),
        CssKnownPropertyValueRef::MinHeight(value) => value.current().clone(),
        CssKnownPropertyValueRef::MinInlineSize(value) => value.current().clone(),
        CssKnownPropertyValueRef::MinBlockSize(value) => value.current().clone(),
        _ => panic!("preferred or minimum size"),
    }
}

#[test]
fn checked_literal_preserves_exact_lexeme_and_rejects_tiny_negative() {
    let literal =
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component("1e999px")).unwrap();
    let surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension {
        number,
        unit,
    }) = literal.literal_component().unwrap().view()
    else {
        panic!("exact authored dimension")
    };
    assert_eq!(number.representation(), "1e999");
    assert_eq!(unit, "px");
    assert_eq!(
        literal.literal_component().unwrap().origin(),
        literal.origin()
    );
    let box_size = CssBoxSize::LengthPercentage(literal);
    let canonical = box_size.serialize_specified().unwrap();
    assert!(canonical.ends_with("px"));
    assert_eq!(canonical.len(), 1002); // 1 followed by 999 zeroes, then px.
    assert!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component("-1e-999%")).is_err()
    );
}

#[test]
fn parsed_intrinsic_and_fit_content_values_have_distinct_typed_branches() {
    assert_eq!(size("inline-size", "auto"), CssSizeValue::Auto);
    assert_eq!(
        size("block-size", "contain"),
        CssSizeValue::BoxSize(CssBoxSize::Contain)
    );
    assert_eq!(
        size("min-width", "fit-content"),
        CssSizeValue::BoxSize(CssBoxSize::FitContent)
    );
    let CssSizeValue::BoxSize(CssBoxSize::FitContentFunction(argument)) =
        size("width", "fit-content(calc(1px + 2%))")
    else {
        panic!("typed fit-content argument")
    };
    assert!(argument.calculation().is_some());
    assert_eq!(argument.serialize_specified().unwrap(), "calc(2% + 1px)");
}

#[test]
fn programmatic_checked_values_preserve_their_typed_boundary() {
    let length = CssSpecifiedNonNegativeLengthPercentage::try_from_component(component("2.5%"))
        .expect("checked percentage");
    let preferred = CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length));
    assert_eq!(preferred.serialize_specified().unwrap(), "2.5%");
    let calc = CssCalcSize::try_from_component(component("calc-size(min-content, size + 1px)"))
        .expect("checked calc-size");
    let maximum = CssMaxSizeValue::try_box_size(CssBoxSize::CalcSize(calc.try_into().unwrap()))
        .expect("maximum permits intrinsic basis");
    assert_eq!(
        maximum.serialize_specified().unwrap(),
        "calc-size(min-content, 1px + size)"
    );
}

#[test]
fn maximum_constructor_rejects_auto_in_nested_calc_size_basis() {
    let nested =
        CssCalcSize::try_from_component(component("calc-size(calc-size(auto, 1px), size + 1px)"))
            .unwrap();
    let surgeist_css::CssCalcSizeBasisRef::Nested(child) = nested.basis() else {
        panic!("nested basis")
    };
    let auto_origin = child.basis_origin().clone();
    let error = CssMaxSizeValue::try_box_size(CssBoxSize::CalcSize(nested.try_into().unwrap()))
        .unwrap_err();
    assert_eq!(error.origin(), Some(&auto_origin));
    let max = CssMaxSizeValue::try_box_size(CssBoxSize::Contain).unwrap();
    assert_eq!(max.box_size(), Some(&CssBoxSize::Contain));
    assert_eq!(max.serialize_specified().unwrap(), "contain");
    assert!(CssMaxSizeValue::NONE.is_none());
}

#[test]
fn invalid_nested_basis_and_fit_argument_report_original_token_positions() {
    for (name, value, offender) in [
        ("max-width", "calc-size(calc-size(auto, 1px), 1px)", "auto"),
        ("inline-size", "fit-content(-1px)", "-1px"),
    ] {
        let source = format!("color:red;/*😀*/\n{name}:{value};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid sizing value: {source}")
        };
        assert_eq!(
            diagnostic.error().code(),
            surgeist_css::CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(offender).unwrap(),
            "original offending token: {source}"
        );
    }
}

#[test]
fn checked_maximum_reports_nested_auto_origin_after_programmatic_trivia() {
    let parsed = parse_style_attribute("/*😀*/\nwidth:calc-size(calc-size(auto, 1px), 1px)");
    assert!(parsed.is_clean());
    let width = &parsed.syntax()[0];
    let CssKnownPropertyValueRef::Width(value) = width.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed preferred size")
    };
    let CssSizeValue::BoxSize(CssBoxSize::CalcSize(calc)) = value.current() else {
        panic!("parsed nested calc-size")
    };
    let surgeist_css::CssCalcSizeBasisRef::Nested(child) = calc.as_calc_size().basis() else {
        panic!("nested calc-size basis")
    };
    let expected_origin = child.basis_origin().clone();
    let mut items = parse_component_values("/* programmatic */ ")
        .unwrap()
        .items()
        .to_vec();
    items.extend(width.value_components().items().iter().cloned());
    let components = surgeist_css::CssComponentValues::try_new(items).unwrap();
    let error = surgeist_css::parse_property_value_for_grammar(
        surgeist_css::CssPropertyGrammar::from_name("max-width").unwrap(),
        components,
        surgeist_css::CssImportance::Normal,
    )
    .unwrap_err();
    assert_eq!(
        error.origin(),
        &surgeist_css::CssSerializedOrigin::Token(expected_origin)
    );
}

#[test]
fn sizing_equality_preserves_structure_across_distinct_origins() {
    let parsed = size("width", "2px");
    let checked = CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_dimension("2", "px").unwrap(),
        )
        .unwrap(),
    ));
    assert_eq!(parsed, checked);
    let (
        Some(CssBoxSize::LengthPercentage(parsed_length)),
        Some(CssBoxSize::LengthPercentage(checked_length)),
    ) = (parsed.box_size(), checked.box_size())
    else {
        panic!("literal sizes")
    };
    assert_ne!(parsed_length.origin(), checked_length.origin());
    assert_eq!(
        checked_length.origin(),
        &surgeist_css::CssValueOrigin::Programmatic
    );

    let parsed_max = parse_style_attribute("max-width:2px");
    assert!(parsed_max.is_clean());
    let CssKnownPropertyValueRef::MaxWidth(parsed_max_value) = parsed_max.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("parsed maximum")
    };
    let checked_max = CssMaxSizeValue::try_box_size(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_dimension("2", "px").unwrap(),
        )
        .unwrap(),
    ))
    .unwrap();
    assert_eq!(parsed_max_value.current(), &checked_max);

    let authored_fit = size("height", "fit-content(calc(1px + 2%))");
    let checked_fit = CssSizeValue::BoxSize(CssBoxSize::FitContentFunction(
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(
                parse_component_values("calc(1px + 2%)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    ));
    assert_eq!(authored_fit, checked_fit);

    let authored_calc = size("inline-size", "calc-size(min-content, size + 1px)");
    let checked_calc = CssSizeValue::BoxSize(CssBoxSize::CalcSize(
        CssCalcSize::try_from_component(component("calc-size(min-content, size + 1px)"))
            .unwrap()
            .try_into()
            .unwrap(),
    ));
    assert_eq!(authored_calc, checked_calc);

    assert_ne!(parsed, size("width", "3px"));
    assert_ne!(parsed, size("width", "02px"));
    assert_ne!(parsed, size("width", "2.0px"));
    assert_ne!(parsed, CssSizeValue::Auto);
    assert_ne!(
        size("width", "calc(1px + 2%)"),
        size("width", "calc(2% + 1px)")
    );
    assert_ne!(
        size("width", "calc(1px + 2%)"),
        size("width", "calc(1px - 2%)")
    );
    assert_ne!(
        size("width", "calc(1px + 2%)"),
        size("width", "calc((1px + 2%))")
    );
    assert_ne!(
        authored_calc,
        size("inline-size", "calc-size(min-content, 1px + size)")
    );
    assert_eq!(
        authored_calc,
        size("inline-size", "calc-size(MIN-CONTENT, size + 1px)")
    );
    assert_ne!(CssBoxSize::FitContent, CssBoxSize::Contain);
    assert_ne!(
        CssMaxSizeValue::NONE,
        CssMaxSizeValue::try_box_size(CssBoxSize::Contain).unwrap()
    );
}

#[test]
fn coupled_declaration_equality_keeps_identity_importance_and_position_distinct() {
    use surgeist_css::{CssImportance, CssKnownProperty, CssPropertyNameRef, parse_property_value};

    let report = parse_style_attribute("width:2px");
    assert!(report.is_clean());
    let parsed = &report.syntax()[0];
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        parse_component_values("2px").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(parsed.body(), checked.body());
    assert_ne!(parsed, &checked); // Only the parsed declaration has a name position.
    assert!(!parsed.same_occurrence(&checked));
    let important = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        parse_component_values("2px").unwrap(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.body(), important.body());
    assert_ne!(checked, important);
    let height = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Height),
        parse_component_values("2px").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_ne!(checked.body(), height.body());
    let lower = parse_style_attribute("width:calc-size(min-content, size)");
    let upper = parse_style_attribute("width:calc-size(MIN-CONTENT, size)");
    assert!(lower.is_clean() && upper.is_clean());
    let (
        CssKnownPropertyValueRef::Width(lower_value),
        CssKnownPropertyValueRef::Width(upper_value),
    ) = (
        lower.syntax()[0].known().unwrap().property_value().unwrap(),
        upper.syntax()[0].known().unwrap().property_value().unwrap(),
    )
    else {
        panic!("checked calc-size wrappers")
    };
    assert_eq!(lower_value.current(), upper_value.current());
    assert_ne!(lower.syntax()[0].body(), upper.syntax()[0].body());
    assert_ne!(
        CssKnownProperty::Width.stable_id(),
        CssKnownProperty::Height.stable_id()
    );
}
