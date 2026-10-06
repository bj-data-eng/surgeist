#![forbid(unsafe_code)]
//! Shared authored numeric domains, exercised through functional public APIs.
//! Exact retention is the adopted Surgeist contract; ordinary cubic-bezier X
//! has the Easing 1 inclusive [0,1] range and genuine math remains deferred.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2023/CRD-css-easing-1-20230213/#cubic-bezier-easing-functions

use surgeist_css::*;

fn number(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}

fn percentage(text: &str) -> CssSpecifiedPercentage {
    CssSpecifiedPercentage::try_from_component(CssComponentValue::try_token(text).unwrap()).unwrap()
}

fn number_math(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values(text).unwrap()).unwrap(),
    )
    .unwrap()
}

fn percentage_math(text: &str) -> CssSpecifiedPercentage {
    CssSpecifiedPercentage::try_from_calculation(
        CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap(),
    )
    .unwrap()
}

fn parsed_token(text: &str) -> CssComponentValue {
    parse_component_values(text)
        .unwrap()
        .items()
        .iter()
        .find(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .unwrap()
        .clone()
}

fn number_spelling(value: &CssSpecifiedNumber) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
        value.literal_component().unwrap().view()
    else {
        panic!("ordinary number")
    };
    number.representation()
}

fn percentage_spelling(value: &CssSpecifiedPercentage) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) =
        value.literal_component().unwrap().view()
    else {
        panic!("ordinary percentage")
    };
    number.representation()
}

#[test]
fn signed_shared_scalars_preserve_exact_lexemes_and_programmatic_origins() {
    for text in [
        "1e999",
        "-1e999",
        "-1e-999",
        "+01.250",
        "-0e9999999999999999999999999999999999999999",
    ] {
        let value = number(text);
        assert_eq!(number_spelling(&value), text);
        assert!(value.calculation().is_none());
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        let value = percentage(&format!("{text}%"));
        assert_eq!(percentage_spelling(&value), text);
        assert!(value.calculation().is_none());
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn signed_percentage_checks_its_root_and_preserves_rejected_origin() {
    for text in ["1", "0", "1px", "1deg", "auto"] {
        let component = parsed_token(text);
        let origin = component.origin().clone();
        let error = CssSpecifiedPercentage::try_from_component(component).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(error.origin(), Some(&origin));
    }
    for text in ["-0%", "-1e-999%", "-1e999%"] {
        assert_eq!(
            percentage_spelling(&percentage(text)),
            text.strip_suffix('%').unwrap()
        );
    }
    for text in ["-1e-999%", "-1e999%"] {
        assert!(
            CssSpecifiedNonNegativePercentage::try_from_component(
                CssComponentValue::try_token(text).unwrap()
            )
            .is_err()
        );
    }
}

#[test]
fn parsed_shared_scalars_retain_coordinates_and_direct_origin_sensitive_equality() {
    let parsed = CssSpecifiedNumber::try_from_component(parsed_token("  -1e999")).unwrap();
    let CssValueOrigin::Parsed(origin) = parsed.origin() else {
        panic!("parsed number")
    };
    assert_eq!(origin.source().as_str(), "  -1e999");
    assert_eq!(origin.span().start().byte_offset().value(), 2);
    assert_ne!(parsed, number("-1e999"));
    let parsed = CssSpecifiedPercentage::try_from_component(parsed_token("  -1e999%")).unwrap();
    let CssValueOrigin::Parsed(origin) = parsed.origin() else {
        panic!("parsed percentage")
    };
    assert_eq!(origin.source().as_str(), "  -1e999%");
    assert_eq!(origin.span().start().byte_offset().value(), 2);
    assert_ne!(parsed, percentage("-1e999%"));
}

#[test]
fn bare_number_and_percentage_calculations_reenter_literal_admission() {
    for text in ["-1e999", "+01.250", "-0"] {
        let value = number_math(text);
        assert_eq!(number_spelling(&value), text);
        assert!(value.calculation().is_none());
    }
    for text in ["-1e999%", "+01.250%", "-0%"] {
        let value = percentage_math(text);
        assert_eq!(percentage_spelling(&value), text.strip_suffix('%').unwrap());
        assert!(value.calculation().is_none());
    }
    let number = number_math("calc(-1e999)");
    assert!(number.literal_component().is_none());
    assert_eq!(
        number.calculation().unwrap().result_type(),
        CssCalculationType::Number
    );
    let percentage = percentage_math("calc(-1e999%)");
    assert!(percentage.literal_component().is_none());
    assert_eq!(
        percentage.calculation().unwrap().result_type(),
        CssCalculationType::Percentage
    );
}

#[test]
fn cubic_x_checks_exact_sign_exponent_and_inclusive_boundaries() {
    for text in [
        "0",
        "-0",
        "+0e9999999999999999999999999999999999999999",
        "-0e-9999999999999999999999999999999999999999",
        "1",
        "1.0000000000000000000",
        "10e-1",
        "1e-999",
        "0.99999999999999999999999999",
    ] {
        let x = CssCubicBezierX::try_new(number(text)).unwrap();
        assert_eq!(number_spelling(x.value()), text);
        assert!(CssCubicBezierX::try_new(number_math(text)).is_some());
    }
    for text in [
        "-1e-999",
        "-0.00000000000000000001",
        "1.0000000000000000001",
        "10.000000000000000001e-1",
        "1e999",
    ] {
        assert!(CssCubicBezierX::try_new(number(text)).is_none(), "{text}");
        assert!(
            CssCubicBezierX::try_new(number_math(text)).is_none(),
            "bare root {text}"
        );
    }
    assert!(
        CssCubicBezierX::try_new(number("1e9999999999999999999999999999999999999999")).is_none()
    );
    for text in ["calc(-1)", "calc(2)", "calc(1 + 1)"] {
        assert!(
            CssCubicBezierX::try_new(number_math(text))
                .unwrap()
                .value()
                .calculation()
                .is_some()
        );
    }
}

#[test]
fn cubic_constructor_retains_axis_roles_and_unrestricted_exact_y() {
    let value = CssCubicBezier::try_new(
        number("0"),
        number("-1e999"),
        number("1"),
        number_math("calc(1e999)"),
    )
    .unwrap();
    assert_eq!(number_spelling(value.x1().value()), "0");
    assert_eq!(number_spelling(value.y1()), "-1e999");
    assert_eq!(number_spelling(value.x2().value()), "1");
    assert!(value.y2().calculation().is_some());
    assert!(
        CssCubicBezier::try_new(number("-1e-999"), number("0"), number("1"), number("1")).is_none()
    );
    assert!(
        CssCubicBezier::try_new(
            number("0"),
            number("0"),
            number("1.0000000000000001"),
            number("1")
        )
        .is_none()
    );
}

#[test]
fn matrices_preserve_all_ordered_exact_and_symbolic_members() {
    let matrix = CssTransformMatrix::new([
        number("1"),
        number_math("calc(2 + 3)"),
        number("0"),
        number("1"),
        number("1e999"),
        number("-1e999"),
    ]);
    assert_eq!(number_spelling(&matrix.components()[0]), "1");
    assert!(matrix.components()[1].calculation().is_some());
    assert_eq!(number_spelling(&matrix.components()[4]), "1e999");
    assert_eq!(number_spelling(&matrix.components()[5]), "-1e999");
    let matrix = CssTransformMatrix3d::new(std::array::from_fn(|index| number(&index.to_string())));
    assert_eq!(matrix.components().len(), 16);
    for (index, value) in matrix.components().iter().enumerate() {
        assert_eq!(number_spelling(value), index.to_string());
    }
}

#[test]
fn transform_scale_components_preserve_signed_percentage_identity_and_omission() {
    let x = CssTransformScaleComponent::Number(number("1e999"));
    let y = CssTransformScaleComponent::Percentage(percentage("-1e999%"));
    let z = CssTransformScaleComponent::Percentage(percentage_math("calc(-25%)"));
    let three = CssTransformScale3d::new(x.clone(), y.clone(), z.clone());
    assert!(
        matches!(three.x(), CssTransformScaleComponent::Number(value) if number_spelling(value) == "1e999")
    );
    assert!(
        matches!(three.y(), CssTransformScaleComponent::Percentage(value) if percentage_spelling(value) == "-1e999")
    );
    assert!(
        matches!(three.z(), CssTransformScaleComponent::Percentage(value) if value.calculation().is_some())
    );
    assert!(matches!(
        CssTransformFunction::ScaleZ(y),
        CssTransformFunction::ScaleZ(CssTransformScaleComponent::Percentage(_))
    ));
    let one = CssTransformScale::new(CssTransformScaleComponent::Number(number("-1e999")), None);
    assert_eq!(number_spelling(scale_number(one.x())), "-1e999");
    assert!(one.y().is_none());
    let two = CssTransformScale::new(
        CssTransformScaleComponent::Number(number("-1e999")),
        Some(CssTransformScaleComponent::Number(number_math("calc(2)"))),
    );
    assert!(scale_number(two.y().unwrap()).calculation().is_some());
    assert_ne!(one, two);
}

#[test]
fn independent_scale_checks_cardinality_and_retains_symbolic_number_factors() {
    assert!(CssScaleValues::try_new(vec![]).is_none());
    for count in 1..=3 {
        let values =
            CssScaleValues::try_new(vec![
                CssTransformScaleComponent::Number(number("-1e999"));
                count
            ])
            .unwrap();
        assert_eq!(values.values().len(), count);
        assert!(
            values
                .values()
                .iter()
                .all(|value| number_spelling(scale_number(value)) == "-1e999")
        );
    }
    assert!(
        CssScaleValues::try_new(vec![CssTransformScaleComponent::Number(number("1")); 4]).is_none()
    );
    assert!(
        CssScaleValues::try_new(vec![CssTransformScaleComponent::Number(number_math(
            "calc(1)"
        ))])
        .is_some()
    );
    assert!(
        CssScaleValues::try_new(vec![
            CssTransformScaleComponent::Number(number("1")),
            CssTransformScaleComponent::Number(number_math("calc(2)"))
        ])
        .is_some()
    );
    assert!(
        CssScaleValues::try_new(vec![CssTransformScaleComponent::Number(number_math("1"))])
            .is_some()
    );
}

#[test]
fn number_aggregates_ignore_only_origins_and_keep_order_omission_and_lexical_structure() {
    let parsed = CssSpecifiedNumber::try_from_component(parsed_token("  1")).unwrap();
    let built = number("1");
    assert_ne!(parsed, built);
    let a = CssTransformMatrix::new([
        parsed.clone(),
        number("2"),
        number("3"),
        number("4"),
        number("5"),
        number("6"),
    ]);
    let b = CssTransformMatrix::new([
        built.clone(),
        number("2"),
        number("3"),
        number("4"),
        number("5"),
        number("6"),
    ]);
    assert_eq!(a, b);
    assert_eq!(
        CssTransformFunction::ScaleX(CssTransformScaleComponent::Number(parsed.clone())),
        CssTransformFunction::ScaleX(CssTransformScaleComponent::Number(built.clone()))
    );
    assert_eq!(
        CssTransformFunction::ScaleY(CssTransformScaleComponent::Number(parsed.clone())),
        CssTransformFunction::ScaleY(CssTransformScaleComponent::Number(built.clone()))
    );
    assert_ne!(
        CssTransformFunction::ScaleX(CssTransformScaleComponent::Number(parsed.clone())),
        CssTransformFunction::ScaleY(CssTransformScaleComponent::Number(built.clone()))
    );
    assert_eq!(
        CssTransformScale::new(CssTransformScaleComponent::Number(parsed.clone()), None),
        CssTransformScale::new(CssTransformScaleComponent::Number(built.clone()), None)
    );
    assert_ne!(
        CssTransformScale::new(CssTransformScaleComponent::Number(parsed.clone()), None),
        CssTransformScale::new(
            CssTransformScaleComponent::Number(built.clone()),
            Some(CssTransformScaleComponent::Number(number("1")))
        )
    );
    assert_ne!(
        CssTransformScale::new(
            CssTransformScaleComponent::Number(number("1")),
            Some(CssTransformScaleComponent::Number(number("2")))
        ),
        CssTransformScale::new(
            CssTransformScaleComponent::Number(number("2")),
            Some(CssTransformScaleComponent::Number(number("1")))
        )
    );
    assert_ne!(
        CssTransformScale::new(CssTransformScaleComponent::Number(number("1")), None),
        CssTransformScale::new(CssTransformScaleComponent::Number(number("1.0")), None)
    );
    assert_eq!(
        CssScaleValues::try_new(vec![CssTransformScaleComponent::Number(parsed)]).unwrap(),
        CssScaleValues::try_new(vec![CssTransformScaleComponent::Number(built)]).unwrap()
    );
}

#[test]
fn percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure() {
    let parsed = CssSpecifiedPercentage::try_from_component(parsed_token("  25%")).unwrap();
    let built = percentage("25%");
    assert!(
        (parsed) != (built),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    assert!(
        (CssTransformScaleComponent::Percentage(parsed))
            == (CssTransformScaleComponent::Percentage(built)),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    assert!(
        (CssTransformScaleComponent::Percentage(percentage("25%")))
            != (CssTransformScaleComponent::Number(number("25"))),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    assert!(
        (CssTransformScaleComponent::Percentage(percentage("25%")))
            != (CssTransformScaleComponent::Percentage(percentage("25.0%"))),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    assert!(
        (CssTransformScaleComponent::Percentage(percentage("25%")))
            != (CssTransformScaleComponent::Percentage(percentage_math("calc(25%)"))),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    let a = percentage_math("calc(25% + 5%)");
    let b = percentage_math("  calc(25% + 5%)");
    assert!(
        (a) != (b),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    assert!(
        (a.calculation().unwrap()) != (b.calculation().unwrap()),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
    assert!(
        (CssTransformFunction::ScaleZ(CssTransformScaleComponent::Percentage(a)))
            == (CssTransformFunction::ScaleZ(CssTransformScaleComponent::Percentage(b))),
        "percentage_aggregate_equality_ignores_origins_and_keeps_branch_and_math_structure: direct equality contract"
    );
}

#[test]
fn cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles() {
    let a = number_math("calc(1 + 2)");
    let b = number_math("  calc(1 + 2)");
    assert!(
        (a) != (b),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
    assert!(
        (a.calculation().unwrap()) != (b.calculation().unwrap()),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
    assert!(
        (CssTransformScale::new(CssTransformScaleComponent::Number(a.clone()), None))
            == (CssTransformScale::new(CssTransformScaleComponent::Number(b.clone()), None)),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
    assert!(
        (CssTransformScale::new(CssTransformScaleComponent::Number(a.clone()), None))
            != (CssTransformScale::new(
                CssTransformScaleComponent::Number(number_math("calc(2 + 1)")),
                None
            )),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
    let make = |y| CssCubicBezier::try_new(number("0"), y, number("1"), number("0")).unwrap();
    assert!(
        (make(a)) == (make(b)),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
    assert!(
        (make(number("1"))) != (make(number("1.0"))),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
    assert!(
        (make(number("1")))
            != (CssCubicBezier::try_new(number("0"), number("0"), number("1"), number("1"))
                .unwrap()),
        "cubic_and_symbolic_number_aggregate_equality_keeps_ast_and_axis_roles: direct equality contract"
    );
}

#[test]
fn three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles() {
    let parsed = CssSpecifiedNumber::try_from_component(parsed_token("  1")).unwrap();
    let ordinary = number("1");
    assert!(
        (CssTransformMatrix3d::new(std::array::from_fn(|_| parsed.clone())))
            == (CssTransformMatrix3d::new(std::array::from_fn(|_| ordinary.clone()))),
        "three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles: direct equality contract"
    );
    let angle = CssAngleOrZero::Angle(CssAngleValue::from_literal(
        CssAngleLiteral::try_new("45", CssAngleUnit::Degrees).unwrap(),
    ));
    let rotate = |axis| CssTransformRotate3d::new(axis, number("0"), number("-1"), angle.clone());
    assert!(
        (rotate(parsed.clone())) == (rotate(ordinary.clone())),
        "three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles: direct equality contract"
    );
    assert!(
        (rotate(ordinary.clone()))
            != (CssTransformRotate3d::new(
                number("0"),
                ordinary.clone(),
                number("-1"),
                angle.clone()
            )),
        "three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles: direct equality contract"
    );
    assert!(
        (rotate(ordinary.clone()))
            != (CssTransformRotate3d::new(
                ordinary.clone(),
                number("0"),
                number("-1"),
                CssAngleOrZero::Zero(
                    CssZeroLiteral::try_from_component(CssComponentValue::try_number("0").unwrap())
                        .unwrap()
                )
            )),
        "three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles: direct equality contract"
    );
    let a = CssTransformScaleComponent::Number(parsed);
    let b = CssTransformScaleComponent::Number(ordinary);
    let pct = CssTransformScaleComponent::Percentage(percentage("25%"));
    assert!(
        (CssTransformScale3d::new(a, pct.clone(), b.clone()))
            == (CssTransformScale3d::new(b.clone(), pct.clone(), b.clone())),
        "three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles: direct equality contract"
    );
    assert!(
        (CssTransformScale3d::new(b.clone(), pct.clone(), b.clone()))
            != (CssTransformScale3d::new(pct, b.clone(), b)),
        "three_dimensional_aggregates_compare_shared_structure_and_preserve_other_roles: direct equality contract"
    );
}

#[test]
fn signed_scalar_projection_has_exact_independent_budgets_and_atomic_errors() {
    let number = number("-12.50");
    let percentage = percentage("-12.50%");
    assert_eq!(
        number
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 5))
            .unwrap(),
        "-12.5"
    );
    assert_eq!(
        percentage
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "-12.5%"
    );
    for (input, projection, bytes, kind) in [
        (
            0,
            1,
            10,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            1,
            0,
            10,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (1, 1, 4, CssSpecifiedValueSerializationErrorKind::ByteLimit),
    ] {
        assert_eq!(
            number
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input, projection, bytes
                ))
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            percentage
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input, projection, bytes
                ))
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        percentage
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 5))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(number_spelling(&number), "-12.50");
    assert_eq!(percentage_spelling(&percentage), "-12.50");
    assert_eq!(number.serialize_specified().unwrap(), "-12.5");
    assert_eq!(percentage.serialize_specified().unwrap(), "-12.5%");
}

#[test]
fn shared_signed_number_projection_keeps_cumulative_existing_variation_budgets() {
    let value = CssFontVariationSettings::Variations(
        CssFontVariationList::try_new(vec![
            CssFontVariation::new(CssOpenTypeTag::try_new("wght").unwrap(), number("-1")),
            CssFontVariation::new(CssOpenTypeTag::try_new("wdth").unwrap(), number("2")),
        ])
        .unwrap(),
    );
    let expected = "\"wght\" -1, \"wdth\" 2";
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                4,
                4,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (input, projection, bytes, kind) in [
        (
            3,
            4,
            expected.len(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            4,
            3,
            expected.len(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            4,
            4,
            expected.len() - 1,
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input, projection, bytes
                ))
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

fn scale_number(value: &CssTransformScaleComponent) -> &CssSpecifiedNumber {
    let CssTransformScaleComponent::Number(v) = value else {
        panic!("number factor")
    };
    v
}
