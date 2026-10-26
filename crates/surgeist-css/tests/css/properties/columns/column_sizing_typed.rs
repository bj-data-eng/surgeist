use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::value_assertions::positive_literal;
use surgeist_css::*;

fn exact_positive(text: &str) -> CssNumericLiteral<CssInteger> {
    positive_literal(
        CssNumericLiteral::<CssInteger>::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn positive_integer_constructor_rejects_zero_and_negative_without_narrowing() {
    for text in ["0", "+0000", "-0", "-1", "-99999999999999999999"] {
        let integer = CssNumericLiteral::<CssInteger>::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap();
        assert!(positive_literal(integer).is_none(), "{text}");
    }
    for text in ["1", "+0001", "2147483648", "9999999999999999999999"] {
        let exact = exact_positive(text);
        assert_eq!(exact.numeric().representation(), text);
        assert!(matches!(exact.origin(), CssValueOrigin::Programmatic));
    }
    assert_eq!(
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(exact_positive("+0002147483648"))
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "2147483648"
    );
    assert_eq!(
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(exact_positive(
            "9999999999999999999999"
        ))
        .unwrap()
        .serialize_specified(Default::default())
        .unwrap(),
        "9999999999999999999999"
    );
    assert_eq!(exact_positive("1").numeric().representation(), "1");
}

#[test]
fn exact_count_and_columns_emit_all_effective_values_under_one_budget() {
    let large =
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(exact_positive("2147483648"))
            .unwrap();
    assert_eq!(
        large.serialize_specified(Default::default()).unwrap(),
        "2147483648"
    );
    let count = CssColumnCount::Count(large);
    assert_eq!(
        count.serialize_specified(Default::default()).unwrap(),
        "2147483648"
    );
    let columns = CssColumns::new(CssSizeValue::Auto, count);
    assert_eq!(columns.width(), &CssSizeValue::Auto);
    assert_eq!(
        columns
            .count()
            .serialize_specified(Default::default())
            .unwrap(),
        "2147483648"
    );
    let expected = "auto 2147483648";
    assert_eq!(
        columns.serialize_specified(Default::default()).unwrap(),
        expected
    );
    assert_eq!(
        columns
            .serialize_specified(CssSerializationLimits::new(3, 3, expected.len(),))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, expected.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, expected.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, expected.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&columns.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}

#[test]
fn parsed_and_programmatic_exact_counts_agree_without_origin_equality() {
    let report = parse_style_attribute("column-count:2147483648");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ColumnCount(parsed) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("column-count")
    };
    let direct = CssColumnCount::Count(
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(exact_positive("2147483648"))
            .unwrap(),
    );
    assert_eq!(parsed.value(), &direct);
    let (CssColumnCount::Count(from_source), CssColumnCount::Count(programmatic)) =
        (parsed.value(), &direct)
    else {
        panic!("exact counts")
    };
    let (Some(from_source), Some(programmatic)) = (from_source.literal(), programmatic.literal())
    else {
        panic!("exact counts")
    };
    assert_ne!(from_source.origin(), programmatic.origin());
    assert_ne!(
        direct,
        CssColumnCount::Count(
            CssNumeric::<CssInteger, CssPositive>::try_from_literal(exact_positive("2147483649"))
                .unwrap()
        )
    );
    assert_ne!(direct, CssColumnCount::Auto);
}

#[test]
fn box_size_and_integer_math_remain_symbolic_at_the_column_boundary() {
    let width = CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_token("2.5%").unwrap(),
        )
        .unwrap(),
    ));
    let columns = CssColumns::new(width, CssColumnCount::Auto);
    assert_eq!(
        columns.serialize_specified(Default::default()).unwrap(),
        "2.5% auto"
    );
    let expression = CssCalculation::<CssInteger>::try_from_components(
        parse_component_values("calc(1 + 2)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let count = CssNumeric::<CssInteger, CssPositive>::try_from_calculation(expression).unwrap();
    assert_eq!(
        count.serialize_specified(Default::default()).unwrap(),
        "calc(3)"
    );
    let columns = CssColumns::new(CssSizeValue::Auto, CssColumnCount::Count(count));
    assert_eq!(
        columns.serialize_specified(Default::default()).unwrap(),
        "auto calc(3)"
    );
    let report = parse_style_attribute("columns:calc(1 + 2)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Columns(parsed) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("columns")
    };
    assert_eq!(parsed.value(), &columns);
}

#[test]
fn two_symbolic_children_share_projection_and_output_limits() {
    let width = CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                parse_component_values("calc(1px + 2%)").unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap(),
    ));
    let count = CssColumnCount::Count(
        CssNumeric::<CssInteger, CssPositive>::try_from_calculation(
            CssCalculation::<CssInteger>::try_from_components(
                parse_component_values("calc(1 + 2)").unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    let columns = CssColumns::new(width, count);
    let expected = "calc(2% + 1px) calc(3)";
    assert_eq!(
        columns.serialize_specified(Default::default()).unwrap(),
        expected
    );
    // Input: pair + two four-node calc() expressions. Projection: pair + five nodes
    // for the mixed-unit width + three for the folded integer count.
    assert_eq!(
        columns
            .serialize_specified(CssSerializationLimits::new(9, 9, expected.len(),))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(8, 9, expected.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(9, 8, expected.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(9, 9, expected.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&columns.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}
