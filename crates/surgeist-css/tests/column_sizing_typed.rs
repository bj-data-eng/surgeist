#![forbid(unsafe_code)]

use surgeist_css::*;

fn exact_positive(text: &str) -> CssPositiveIntegerLiteral {
    CssPositiveIntegerLiteral::try_new(
        CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn positive_integer_constructor_rejects_zero_and_negative_without_narrowing() {
    for text in ["0", "+0000", "-0", "-1", "-99999999999999999999"] {
        let integer =
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap();
        assert!(
            CssPositiveIntegerLiteral::try_new(integer).is_none(),
            "{text}"
        );
    }
    for text in ["1", "+0001", "2147483648", "9999999999999999999999"] {
        let exact = exact_positive(text);
        assert_eq!(exact.integer().numeric().representation(), text);
        assert!(matches!(
            exact.integer().origin(),
            CssValueOrigin::Programmatic
        ));
    }
    assert_eq!(
        CssPositiveIntegerValue::ExactLiteral(exact_positive("+0002147483648"))
            .serialize_specified()
            .unwrap(),
        "2147483648"
    );
    assert_eq!(
        CssPositiveIntegerValue::ExactLiteral(exact_positive("9999999999999999999999"))
            .serialize_specified()
            .unwrap(),
        "9999999999999999999999"
    );
    assert_eq!(CssPositiveInteger::try_new(1).unwrap().value(), 1);
}

#[test]
fn exact_count_and_columns_emit_all_effective_values_under_one_budget() {
    let large = CssPositiveIntegerValue::ExactLiteral(exact_positive("2147483648"));
    assert_eq!(large.serialize_specified().unwrap(), "2147483648");
    let count = CssColumnCount::Count(large);
    assert_eq!(count.serialize_specified().unwrap(), "2147483648");
    let columns = CssColumns::new(CssColumnWidth::Auto, count);
    assert_eq!(columns.width(), &CssColumnWidth::Auto);
    assert_eq!(columns.count().serialize_specified().unwrap(), "2147483648");
    let expected = "auto 2147483648";
    assert_eq!(columns.serialize_specified().unwrap(), expected);
    assert_eq!(
        columns
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                3,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            columns
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
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
    let direct = CssColumnCount::Count(CssPositiveIntegerValue::ExactLiteral(exact_positive(
        "2147483648",
    )));
    assert_eq!(parsed.count(), &direct);
    let (
        CssColumnCount::Count(CssPositiveIntegerValue::ExactLiteral(from_source)),
        CssColumnCount::Count(CssPositiveIntegerValue::ExactLiteral(programmatic)),
    ) = (parsed.count(), &direct)
    else {
        panic!("exact counts")
    };
    assert_ne!(
        from_source.integer().origin(),
        programmatic.integer().origin()
    );
    assert_ne!(
        direct,
        CssColumnCount::Count(CssPositiveIntegerValue::ExactLiteral(exact_positive(
            "2147483649"
        )))
    );
    assert_ne!(direct, CssColumnCount::Auto);
}

#[test]
fn box_size_and_integer_math_remain_symbolic_at_the_column_boundary() {
    let width = CssColumnWidth::BoxSize(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_token("2.5%").unwrap(),
        )
        .unwrap(),
    ));
    let columns = CssColumns::new(width, CssColumnCount::Auto);
    assert_eq!(columns.serialize_specified().unwrap(), "2.5% auto");
    let expression =
        CssIntegerCalculation::try_from_components(parse_component_values("calc(1 + 2)").unwrap())
            .unwrap();
    let count = CssPositiveIntegerValue::Calculation(expression);
    assert_eq!(count.serialize_specified().unwrap(), "calc(3)");
    let columns = CssColumns::new(CssColumnWidth::Auto, CssColumnCount::Count(count));
    assert_eq!(columns.serialize_specified().unwrap(), "auto calc(3)");
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
    assert_eq!(parsed.columns(), &columns);
}

#[test]
fn two_symbolic_children_share_projection_and_output_limits() {
    let width = CssColumnWidth::BoxSize(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values("calc(1px + 2%)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    ));
    let count = CssColumnCount::Count(CssPositiveIntegerValue::Calculation(
        CssIntegerCalculation::try_from_components(parse_component_values("calc(1 + 2)").unwrap())
            .unwrap(),
    ));
    let columns = CssColumns::new(width, count);
    let expected = "calc(2% + 1px) calc(3)";
    assert_eq!(columns.serialize_specified().unwrap(), expected);
    // Input: pair + two four-node calc() expressions. Projection: pair + five nodes
    // for the mixed-unit width + three for the folded integer count.
    assert_eq!(
        columns
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                9,
                9,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, 9, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 8, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 9, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            columns
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}
