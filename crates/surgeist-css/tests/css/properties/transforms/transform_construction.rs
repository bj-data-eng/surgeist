#![forbid(unsafe_code)]
//! Public authored transform construction and checked operand domains.
use surgeist_css::*;

fn number(value: &str) -> CssSpecifiedNumber {
    checked_number(value)
}
fn percentage(value: &str) -> CssSpecifiedPercentage {
    CssSpecifiedPercentage::try_from_component(
        CssComponentValue::try_token(&format!("{value}%")).unwrap(),
    )
    .unwrap()
}
fn zero_angle() -> CssAngleOrZero {
    CssAngleOrZero::Zero(
        CssZeroLiteral::try_from_component(CssComponentValue::try_number("0").unwrap()).unwrap(),
    )
}

fn angle(value: f32) -> CssAngleOrZero {
    CssAngleOrZero::Angle(CssAngleValue::from_literal(
        CssAngleLiteral::try_new(&value.to_string(), CssAngleUnit::Degrees).unwrap(),
    ))
}
fn number_calculation(text: &str) -> CssNumberCalculation {
    CssNumberCalculation::try_from_components(parse_component_values(text).unwrap()).unwrap()
}
#[test]
fn matrix_constructors_preserve_order_and_symbolic_number_operand() {
    let symbolic = number_calculation("calc(2 * 3)");
    let matrix = CssTransformMatrix::new([
        number("1"),
        CssSpecifiedNumber::try_from_calculation(symbolic.clone()).unwrap(),
        number("0"),
        number("1"),
        number("12"),
        number("-8"),
    ]);
    assert!(exact_number(
        matrix.components()[0].literal_component(),
        "1"
    ));
    assert!(matrix.components()[1].calculation() == Some(&symbolic));
    assert!(exact_number(
        matrix.components()[4].literal_component(),
        "12"
    ));
    assert!(exact_number(
        matrix.components()[5].literal_component(),
        "-8"
    ));

    let values = std::array::from_fn(|i| number(&i.to_string()));
    let matrix3d = CssTransformMatrix3d::new(values);
    assert_eq!(matrix3d.components().len(), 16);
    for (index, component) in matrix3d.components().iter().enumerate() {
        assert!(exact_number(
            component.literal_component(),
            &index.to_string()
        ));
    }
}

#[test]
fn rotate3d_scale_and_skew_constructors_preserve_distinct_operands() {
    let rotate = CssTransformRotate3d::new(number("1"), number("0"), number("-1"), angle(45.0));
    assert!(exact_number((rotate.x()).literal_component(), "1"));
    assert!(exact_number((rotate.y()).literal_component(), "0"));
    assert!(exact_number((rotate.z()).literal_component(), "-1"));
    assert!(
        matches!(rotate.angle(), CssAngleOrZero::Angle(v) if v.literal().is_some_and(|literal| literal.numeric().representation() == "45" && literal.unit() == CssAngleUnit::Degrees))
    );

    let one = CssTransformScale::new(CssTransformScaleComponent::Number(number("2")), None);
    assert!(
        matches!(one.x(), CssTransformScaleComponent::Number(v) if exact_number(v.literal_component(), "2"))
    );
    assert!(one.y().is_none());
    let two = CssTransformScale::new(
        CssTransformScaleComponent::Number(number("2")),
        Some(CssTransformScaleComponent::Number(
            CssSpecifiedNumber::try_from_calculation(number_calculation("calc(3 / 2)")).unwrap(),
        )),
    );
    assert!(
        matches!(two.y().unwrap(), CssTransformScaleComponent::Number(n) if n.calculation().is_some_and(|v| v
            .components()
            .serialize()
            .unwrap()
            .as_css()
            == "calc(3 / 2)"))
    );

    let scale3d = CssTransformScale3d::new(
        CssTransformScaleComponent::Number(number("1")),
        CssTransformScaleComponent::Percentage(percentage("30")),
        CssTransformScaleComponent::Number(number("2")),
    );
    assert!(
        matches!(scale3d.x(), CssTransformScaleComponent::Number(v) if exact_number(v.literal_component(), "1"))
    );
    assert!(
        matches!(scale3d.y(), CssTransformScaleComponent::Percentage(v) if exact_percentage(v.literal_component(), "30"))
    );
    assert!(
        matches!(scale3d.z(), CssTransformScaleComponent::Number(v) if exact_number(v.literal_component(), "2"))
    );

    let one = CssTransformSkew::new(zero_angle(), None);
    assert!(matches!(one.x(), CssAngleOrZero::Zero(_)));
    assert!(one.y().is_none());
    let two = CssTransformSkew::new(angle(10.0), Some(angle(-20.0)));
    assert!(
        matches!(two.x(), CssAngleOrZero::Angle(v) if v.literal().is_some_and(|literal| literal.numeric().representation() == "10"))
    );
    assert!(
        matches!(two.y(), Some(CssAngleOrZero::Angle(v)) if v.literal().is_some_and(|literal| literal.numeric().representation() == "-20"))
    );
}

#[test]
fn translate_constructors_keep_xy_percentages_and_pure_length_z() {
    let one = CssTransformTranslate::new(signed_length_percentage("25%"), None);
    assert!(exact_percentage(one.x().literal_component(), "25"));
    assert!(one.y().is_none());
    let two = CssTransformTranslate::new(
        signed_length_percentage("12px"),
        Some(signed_length_percentage("calc(10px + 20%)")),
    );
    assert!(exact_dimension(two.x().literal_component(), "12", "px"));
    assert!(
        two.y().unwrap().calculation().is_some_and(|v| v
            .components()
            .serialize()
            .unwrap()
            .as_css()
            == "calc(10px + 20%)")
    );

    let three = CssTransformTranslate3d::new(
        signed_length_percentage("10%"),
        signed_length_percentage("20%"),
        signed_length("calc(2px + 3px)"),
    );
    assert!(exact_percentage(three.x().literal_component(), "10"));
    assert!(exact_percentage(three.y().literal_component(), "20"));
    assert!(
        three
            .z()
            .calculation()
            .is_some_and(|v| v.components().serialize().unwrap().as_css() == "calc(2px + 3px)")
    );
}

#[test]
fn checked_length_leaves_reject_foreign_domains_but_retain_pure_math() {
    assert!(CssFiniteNumber::try_new(f32::NAN).is_none());
    assert!(CssFiniteNumber::try_new(f32::INFINITY).is_none());
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_ident("auto").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token("5%").unwrap()
        )
        .is_ok()
    );
    assert!(
        CssSpecifiedLength::try_from_component(CssComponentValue::try_token("5%").unwrap())
            .is_err()
    );
    assert!(
        CssLengthCalculation::try_from_components(
            parse_component_values("calc(2px + 5%)").unwrap()
        )
        .is_err()
    );
    assert!(
        CssLengthCalculation::try_from_components(
            parse_component_values("calc(10% / 10% * 1px)").unwrap()
        )
        .is_ok()
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("-1", "px").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_token("1%").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("0", "px").unwrap()
        )
        .is_ok()
    );
    let symbolic = nonnegative_length("calc(1px - 2px)");
    assert_eq!(
        symbolic
            .calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(1px - 2px)"
    );
}

#[test]
fn perspective_and_function_list_preserve_none_order_and_nonempty_invariant() {
    let perspective = CssTransformPerspective::Length(nonnegative_length("8px"));
    let functions = CssTransformFunctionList::try_new(vec![
        CssTransformFunction::Scale(CssTransformScale::new(
            CssTransformScaleComponent::Number(number("2")),
            None,
        )),
        CssTransformFunction::Perspective(perspective),
        CssTransformFunction::Translate3d(CssTransformTranslate3d::new(
            signed_length_percentage("10%"),
            signed_length_percentage("4px"),
            signed_length("6px"),
        )),
        CssTransformFunction::Perspective(CssTransformPerspective::None),
    ])
    .unwrap();
    let CssTransform::Functions(list) = CssTransform::Functions(functions) else {
        panic!("function list")
    };
    assert_eq!(
        list.functions()
            .iter()
            .map(CssTransformFunction::kind)
            .collect::<Vec<_>>(),
        [
            CssTransformFunctionKind::Scale,
            CssTransformFunctionKind::Perspective,
            CssTransformFunctionKind::Translate3d,
            CssTransformFunctionKind::Perspective,
        ]
    );
    assert!(
        matches!(&list.functions()[1], CssTransformFunction::Perspective(CssTransformPerspective::Length(v)) if exact_dimension(v.literal_component(), "8", "px"))
    );
    assert!(matches!(
        &list.functions()[3],
        CssTransformFunction::Perspective(CssTransformPerspective::None)
    ));
    assert!(CssTransformFunctionList::try_new(vec![]).is_none());
    assert!(matches!(CssTransform::None, CssTransform::None));
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}

fn signed_length(css: &str) -> surgeist_css::CssSpecifiedLength {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLength::try_from_calculation(
            surgeist_css::CssLengthCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLength::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}
fn nonnegative_length(css: &str) -> surgeist_css::CssSpecifiedNonNegativeLength {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedNonNegativeLength::try_from_calculation(
            surgeist_css::CssLengthCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedNonNegativeLength::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}
fn signed_length_percentage(css: &str) -> surgeist_css::CssSpecifiedLengthPercentage {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

fn exact_number(component: Option<&surgeist_css::CssComponentValue>, representation: &str) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Number(number))) if number.representation() == representation)
}
fn checked_number(representation: &str) -> surgeist_css::CssSpecifiedNumber {
    surgeist_css::CssSpecifiedNumber::try_from_component(
        surgeist_css::CssComponentValue::try_number(representation).unwrap(),
    )
    .unwrap()
}
