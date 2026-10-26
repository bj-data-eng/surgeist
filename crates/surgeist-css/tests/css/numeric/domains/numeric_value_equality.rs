//! Numeric value equality from CSS Typed OM 1 WD 2024-03-21 "equal numeric
//! values": unit values are equal when their units and values are equal, and
//! math values when their operations and ordered operands are equal. Values
//! reify per §5 "reify a numeric value", so a length's unitless zero has unit
//! px. Equality ignores source origin and the spelling of an exact value.

use surgeist_css::*;

fn parsed(text: &str) -> CssComponentValue {
    CssComponentValues::parse(&CssParser::default(), text)
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone()
}
fn numeric<T: CssNumericRoot>(component: CssComponentValue) -> CssNumeric<T> {
    CssNumeric::<T>::try_from_component(component).unwrap()
}
fn math<T: CssNumericRoot>(text: &str) -> CssNumeric<T> {
    CssNumeric::<T>::try_from_calculation(
        CssCalculation::<T>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap()
}
/// The typed `height` value; the property wrapper also retains authored text.
fn height(context: CssParserContext, value: &str) -> CssSizeValue {
    let report = CssDeclaration::parse(
        &CssParser::new(context, Default::default()),
        format!("height:{value}"),
    );
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let declaration = report.syntax().as_ref().unwrap();
    let CssKnownPropertyValueRef::Height(height) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("height value")
    };
    height.value().clone()
}

#[test]
fn equal_exact_values_ignore_spelling_and_origin() {
    let programmatic = numeric::<CssNumber>(CssComponentValue::try_number("2").unwrap());
    for spelling in ["2", "2.0", "+2", "2e0", "20e-1", " /* x */ 2.000"] {
        let value = numeric::<CssNumber>(parsed(spelling));
        assert!(
            matches!(value.origin(), CssValueOrigin::Parsed(_)),
            "{spelling}"
        );
        assert_eq!(value, programmatic, "{spelling}");
        assert_eq!(
            value.literal().unwrap(),
            programmatic.literal().unwrap(),
            "{spelling}"
        );
    }
    assert_eq!(numeric::<CssNumber>(parsed("-0")), numeric(parsed("0.0")));
    assert_eq!(
        numeric::<CssInteger>(parsed("+0002")),
        numeric(CssComponentValue::try_number("2").unwrap())
    );
    assert_eq!(
        numeric::<CssLength>(parsed("2.0PX")),
        numeric(CssComponentValue::try_dimension("2", "px").unwrap())
    );
    assert_eq!(
        numeric::<CssPercentage>(parsed("25.0%")),
        numeric(CssComponentValue::try_token("25%").unwrap())
    );
    assert_eq!(
        numeric::<CssTime>(parsed("1.5S")),
        numeric(CssComponentValue::try_dimension("1.5", "s").unwrap())
    );
}

#[test]
fn units_values_and_kinds_stay_distinct() {
    assert_ne!(numeric::<CssNumber>(parsed("2")), numeric(parsed("3")));
    assert_ne!(numeric::<CssLength>(parsed("96px")), numeric(parsed("1in")));
    assert_ne!(numeric::<CssLength>(parsed("2px")), numeric(parsed("2em")));
    assert_ne!(numeric::<CssTime>(parsed("1s")), numeric(parsed("1000ms")));
    assert_ne!(
        numeric::<CssAngle>(parsed("90deg")),
        numeric(parsed("100grad"))
    );
    assert_ne!(
        numeric::<CssLengthPercentage>(parsed("0")),
        numeric(parsed("0%"))
    );
    // An ordinary literal and math are different kinds of numeric value.
    assert_ne!(numeric::<CssLength>(parsed("2px")), math("calc(2px)"));
}

#[test]
fn length_unitless_zero_and_quirky_numbers_reify_as_px() {
    assert_eq!(numeric::<CssLength>(parsed("0")), numeric(parsed("0px")));
    assert_eq!(
        numeric::<CssLengthPercentage>(parsed("-0")),
        numeric(parsed("0.0px"))
    );
    assert_ne!(numeric::<CssLength>(parsed("0")), numeric(parsed("0em")));
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let standards = CssParserContext::new(CssParserMode::Standards);
    assert_eq!(height(quirks, "7"), height(standards, "7.0px"));
    assert_ne!(height(quirks, "7"), height(standards, "7em"));
}

#[test]
fn math_compares_operations_and_ordered_operand_values() {
    let value = math::<CssLength>("calc(1px + 2px)");
    assert_eq!(value, math("  CALC(1.0px + +2PX)"));
    assert_eq!(
        value.calculation().unwrap(),
        math::<CssLength>("calc(1px + 2.0px)")
            .calculation()
            .unwrap()
    );
    let programmatic = CssNumeric::<CssLengthPercentage>::try_from_calculation(
        CssCalculation::<CssLengthPercentage>::try_sum(
            CssCalculation::<CssLengthPercentage>::try_dimension(1.0, CssLengthUnit::Px).unwrap(),
            [(
                CssCalculationSumOperator::Add,
                CssCalculation::<CssLengthPercentage>::try_percentage(2.0).unwrap(),
            )],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(programmatic, math("calc(1px + 2%)"));
    for different in [
        "calc(2px + 1px)",
        "calc(1px - 2px)",
        "calc(1px + 3px)",
        "calc(1px + 2em)",
        "min(1px, 2px)",
    ] {
        assert_ne!(value, math(different), "{different}");
    }
    assert_ne!(math::<CssLength>("min(1px, 2px)"), math("max(1px, 2px)"));
}

#[test]
fn nested_math_compares_each_operand_by_value_and_position() {
    let value = math::<CssLength>("calc(2 * 3px - min(4px, 5px) / 6)");
    assert_eq!(value, math("CALC(2.0 * +3PX - MIN(4.0px, 5e0px) / 6.00)"));
    for different in [
        "calc(3px * 2 - min(4px, 5px) / 6)",
        "calc(2 * 3px + min(4px, 5px) / 6)",
        "calc(2 * 3px - min(5px, 4px) / 6)",
        "calc(2 * 3px - min(4px, 5px) * 6)",
        "calc(2 * 3px - min(4px, 5px) / 7)",
        "calc(2 * 3px - max(4px, 5px) / 6)",
    ] {
        assert_ne!(value, math(different), "{different}");
    }
}

/// Values 4 "parse a calculation" replaces a parenthesised block by the result
/// of parsing its contents and a nested math function leaf by its own
/// representation, so grouping and nested `calc()` leave no node: they compare
/// equal. Only grouping that changes the operator tree stays distinct.
#[test]
fn redundant_grouping_and_nested_calc_do_not_change_equality() {
    for (left, right) in [
        ("calc(calc(1px + 2px) + 3px)", "calc((1px + 2px) + 3px)"),
        ("calc(1px + (2px + 3px))", "calc(1px + calc(2px + 3px))"),
        ("calc((1px))", "calc(1px)"),
        ("calc((1px + 2px))", "calc(1px + 2px)"),
        ("calc(calc(1px + 2px))", "calc(1px + 2px)"),
        ("calc(((1px + 2px)))", "calc(calc((1px + 2px)))"),
        ("min((1px), 2px)", "min(1px, 2px)"),
        ("min(calc(1px), (2px))", "min(1px, 2px)"),
        ("calc(2 * (3px))", "calc(2 * 3px)"),
        ("calc(2 * (3px) - (4px))", "calc(2 * 3px - 4px)"),
        ("calc((1px + 2px) * 2)", "calc(calc(1px + 2px) * 2)"),
        (
            "calc(max((1px), 2px) + 3px)",
            "calc(max(1px, calc(2px)) + 3px)",
        ),
    ] {
        assert_eq!(
            math::<CssLength>(left),
            math(right),
            "{left} versus {right}"
        );
        assert_eq!(
            math::<CssLength>(right),
            math(left),
            "{right} versus {left}"
        );
    }
    for (left, right) in [
        // Grouping that changes precedence builds a different operator tree.
        ("calc((1px + 2px) * 2)", "calc(1px + 2px * 2)"),
        ("calc(1px - (2px + 3px))", "calc(1px - 2px + 3px)"),
        ("calc(1px + (2px + 3px))", "calc(1px + 2px + 3px)"),
        // A math function other than `calc()` is a real node.
        ("calc(min((1px), 2px))", "calc(max(1px, 2px))"),
        ("calc((1px + 2px))", "calc(2px + 1px)"),
        ("calc((1px + 2px))", "calc((1px + 3px))"),
    ] {
        assert_ne!(
            math::<CssLength>(left),
            math(right),
            "{left} versus {right}"
        );
    }
}

/// Typed OM "reify a math expression" reifies a root `calc()` as a sum around
/// its argument and "equal numeric values" compares interfaces first, so the
/// root function is not unwrapped: `calc(min(1px, 2px))` differs from
/// `min(1px, 2px)` as `calc(1px)` differs from `1px`. Nested `calc()` inside the
/// argument leaves no node.
#[test]
fn root_function_is_not_unwrapped_but_nested_calc_is_seen_through() {
    assert_ne!(
        math::<CssLength>("calc(min(1px, 2px))"),
        math("min(1px, 2px)")
    );
    assert_ne!(math::<CssLength>("calc(1px)"), numeric(parsed("1px")));
    assert_ne!(
        math::<CssLength>("calc(max(1px, 2px))"),
        math("max(1px, 2px)")
    );
    assert_eq!(
        math::<CssLength>("calc(calc(min(1px, 2px)))"),
        math("calc(min(1px, 2px))")
    );
    assert_eq!(
        math::<CssLength>("calc((calc(min(1px, (2px)))))"),
        math("calc(min(1px, 2px))")
    );
}

/// Values 4 §6.4.1 and Typed OM compare units, not spellings: `x` is the
/// `dppx` unit, also inside math, and units of one dimension stay distinct.
#[test]
fn calculation_leaves_compare_resolved_units_not_unit_spellings() {
    let resolution = |text| math::<CssResolution>(text);
    assert_eq!(resolution("calc(2x)"), resolution("calc(2dppx)"));
    assert_eq!(resolution("calc(2X)"), resolution("calc(2.0DPPX)"));
    assert_eq!(
        resolution("calc(1x + 2dppx)"),
        resolution("calc(1dppx + 2x)")
    );
    assert_ne!(resolution("calc(2x)"), resolution("calc(2dpi)"));
    assert_ne!(resolution("calc(2x)"), resolution("calc(3dppx)"));
    // A literal and the same unit inside math agree on the alias.
    assert_eq!(
        numeric::<CssResolution>(parsed("2x")),
        numeric(parsed("2dppx"))
    );
}
