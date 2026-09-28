#![forbid(unsafe_code)]
//! Public authored transform construction and checked operand domains.
use surgeist_css::*;

fn finite(value: f32) -> CssFiniteNumber {
    CssFiniteNumber::try_new(value).unwrap()
}
fn number(value: f32) -> CssTransformNumber {
    CssTransformNumber::Literal(finite(value))
}
fn percentage(value: f32) -> CssTransformPercentage {
    CssTransformPercentage::Literal(finite(value))
}
fn angle(value: f32) -> CssTransformAngle {
    CssTransformAngle::Literal(CssAngleLiteral::try_new(value, CssAngleUnit::Degrees).unwrap())
}
fn length_percentage(value: CssLength) -> CssTransformLengthPercentage {
    CssTransformLengthPercentage::try_new(value).unwrap()
}
fn pure_length(value: CssLength) -> CssTransformLength {
    CssTransformLength::try_new(value).unwrap()
}
fn number_calculation(text: &str) -> CssNumberCalculation {
    CssNumberCalculation::try_from_components(parse_component_values(text).unwrap()).unwrap()
}
fn length_calculation(text: &str) -> CssLength {
    let calc =
        CssLengthPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap();
    CssLength::Calc(CssCalcLength::Typed(calc))
}

#[test]
fn matrix_constructors_preserve_order_and_symbolic_number_operand() {
    let symbolic = number_calculation("calc(2 * 3)");
    let matrix = CssTransformMatrix::new([
        number(1.0),
        CssTransformNumber::Calculation(symbolic.clone()),
        number(0.0),
        number(1.0),
        number(12.0),
        number(-8.0),
    ]);
    assert!(matches!(&matrix.components()[0], CssTransformNumber::Literal(v) if v.value() == 1.0));
    assert!(
        matches!(&matrix.components()[1], CssTransformNumber::Calculation(v) if v == &symbolic)
    );
    assert!(matches!(&matrix.components()[4], CssTransformNumber::Literal(v) if v.value() == 12.0));
    assert!(matches!(&matrix.components()[5], CssTransformNumber::Literal(v) if v.value() == -8.0));

    let values = std::array::from_fn(|i| number(i as f32));
    let matrix3d = CssTransformMatrix3d::new(values);
    assert_eq!(matrix3d.components().len(), 16);
    for (index, component) in matrix3d.components().iter().enumerate() {
        assert!(matches!(component, CssTransformNumber::Literal(v) if v.value() == index as f32));
    }
}

#[test]
fn rotate3d_scale_and_skew_constructors_preserve_distinct_operands() {
    let rotate = CssTransformRotate3d::new(number(1.0), number(0.0), number(-1.0), angle(45.0));
    assert!(matches!(rotate.x(), CssTransformNumber::Literal(v) if v.value() == 1.0));
    assert!(matches!(rotate.y(), CssTransformNumber::Literal(v) if v.value() == 0.0));
    assert!(matches!(rotate.z(), CssTransformNumber::Literal(v) if v.value() == -1.0));
    assert!(
        matches!(rotate.angle(), CssTransformAngle::Literal(v) if v.value() == 45.0 && v.unit() == CssAngleUnit::Degrees)
    );

    let one = CssTransformScale::new(number(2.0), None);
    assert!(matches!(one.x(), CssTransformNumber::Literal(v) if v.value() == 2.0));
    assert!(one.y().is_none());
    let two = CssTransformScale::new(
        number(2.0),
        Some(CssTransformNumber::Calculation(number_calculation(
            "calc(3 / 2)",
        ))),
    );
    assert!(
        matches!(two.y(), Some(CssTransformNumber::Calculation(v)) if v.components().serialize().unwrap().as_css() == "calc(3 / 2)")
    );

    let scale3d = CssTransformScale3d::new(
        CssTransformScaleComponent::Number(number(1.0)),
        CssTransformScaleComponent::Percentage(percentage(30.0)),
        CssTransformScaleComponent::Number(number(2.0)),
    );
    assert!(
        matches!(scale3d.x(), CssTransformScaleComponent::Number(CssTransformNumber::Literal(v)) if v.value() == 1.0)
    );
    assert!(
        matches!(scale3d.y(), CssTransformScaleComponent::Percentage(CssTransformPercentage::Literal(v)) if v.value() == 30.0)
    );
    assert!(
        matches!(scale3d.z(), CssTransformScaleComponent::Number(CssTransformNumber::Literal(v)) if v.value() == 2.0)
    );

    let one = CssTransformSkew::new(CssTransformAngle::Zero, None);
    assert!(matches!(one.x(), CssTransformAngle::Zero));
    assert!(one.y().is_none());
    let two = CssTransformSkew::new(angle(10.0), Some(angle(-20.0)));
    assert!(matches!(two.x(), CssTransformAngle::Literal(v) if v.value() == 10.0));
    assert!(matches!(two.y(), Some(CssTransformAngle::Literal(v)) if v.value() == -20.0));
}

#[test]
fn translate_constructors_keep_xy_percentages_and_pure_length_z() {
    let one = CssTransformTranslate::new(
        length_percentage(CssLength::try_percent(25.0).unwrap()),
        None,
    );
    assert!(matches!(one.x().value(), CssLength::Percent(v) if v.value() == 25.0));
    assert!(one.y().is_none());
    let two = CssTransformTranslate::new(
        length_percentage(CssLength::try_px(12.0).unwrap()),
        Some(length_percentage(length_calculation("calc(10px + 20%)"))),
    );
    assert!(matches!(two.x().value(), CssLength::Px(v) if v.value() == 12.0));
    assert!(
        matches!(two.y().unwrap().value(), CssLength::Calc(CssCalcLength::Typed(v)) if v.components().serialize().unwrap().as_css() == "calc(10px + 20%)")
    );

    let three = CssTransformTranslate3d::new(
        length_percentage(CssLength::try_percent(10.0).unwrap()),
        length_percentage(CssLength::try_percent(20.0).unwrap()),
        pure_length(length_calculation("calc(2px + 3px)")),
    );
    assert!(matches!(three.x().value(), CssLength::Percent(v) if v.value() == 10.0));
    assert!(matches!(three.y().value(), CssLength::Percent(v) if v.value() == 20.0));
    assert!(
        matches!(three.z().value(), CssLength::Calc(CssCalcLength::Typed(v)) if v.components().serialize().unwrap().as_css() == "calc(2px + 3px)")
    );
}

#[test]
fn checked_length_leaves_reject_foreign_domains_but_retain_pure_math() {
    assert!(CssFiniteNumber::try_new(f32::NAN).is_none());
    assert!(CssFiniteNumber::try_new(f32::INFINITY).is_none());
    assert!(CssTransformLengthPercentage::try_new(CssLength::Auto).is_none());
    assert!(CssTransformLengthPercentage::try_new(CssLength::try_percent(5.0).unwrap()).is_some());
    assert!(CssTransformLength::try_new(CssLength::try_percent(5.0).unwrap()).is_none());
    assert!(CssTransformLength::try_new(length_calculation("calc(2px + 5%)")).is_none());
    assert!(CssTransformLength::try_new(length_calculation("calc(10% / 10% * 1px)")).is_some());
    assert!(CssTransformNonNegativeLength::try_new(CssLength::try_px(-1.0).unwrap()).is_none());
    assert!(CssTransformNonNegativeLength::try_new(CssLength::try_percent(1.0).unwrap()).is_none());
    assert!(CssTransformNonNegativeLength::try_new(CssLength::try_px(0.0).unwrap()).is_some());
    let symbolic = length_calculation("calc(1px - 2px)");
    let positive = CssTransformNonNegativeLength::try_new(symbolic).unwrap();
    assert!(
        matches!(positive.value(), CssLength::Calc(CssCalcLength::Typed(v)) if v.components().serialize().unwrap().as_css() == "calc(1px - 2px)")
    );
}

#[test]
fn perspective_and_function_list_preserve_none_order_and_nonempty_invariant() {
    let perspective = CssTransformPerspective::Length(
        CssTransformNonNegativeLength::try_new(CssLength::try_px(8.0).unwrap()).unwrap(),
    );
    let functions = CssTransformFunctionList::try_new(vec![
        CssTransformFunction::Scale(CssTransformScale::new(number(2.0), None)),
        CssTransformFunction::Perspective(perspective),
        CssTransformFunction::Translate3d(CssTransformTranslate3d::new(
            length_percentage(CssLength::try_percent(10.0).unwrap()),
            length_percentage(CssLength::try_px(4.0).unwrap()),
            pure_length(CssLength::try_px(6.0).unwrap()),
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
        matches!(&list.functions()[1], CssTransformFunction::Perspective(CssTransformPerspective::Length(v)) if matches!(v.value(), CssLength::Px(n) if n.value() == 8.0))
    );
    assert!(matches!(
        &list.functions()[3],
        CssTransformFunction::Perspective(CssTransformPerspective::None)
    ));
    assert!(CssTransformFunctionList::try_new(vec![]).is_none());
    assert!(matches!(CssTransform::None, CssTransform::None));
}
