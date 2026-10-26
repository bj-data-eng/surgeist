//! Values 4 length units and `calc()` keep authored dimensions in ordinary and
//! calculation contexts; unknown units and functions are rejected.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn sizing_calculation(value: &CssSizeValue) -> &CssCalculation<CssLengthPercentage> {
    let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = value else {
        panic!("checked length-percentage")
    };
    length.calculation().expect("checked calculation")
}

/// Every `<length>` unit, written from the specifications rather than from
/// the crate's unit enumeration: Values 4 (WD 2024-03-12) §6.1.1 font-relative
/// (12), §6.1.2 viewport-percentage with its small, large and dynamic variants
/// (24) and §6.2 absolute (7) units, and Contain 3 (WD 2022-08-18) §6
/// container-relative units (6).
const VALUES_4_LENGTH_UNITS: [(&str, CssLengthUnit); 49] = [
    ("em", CssLengthUnit::Em),
    ("rem", CssLengthUnit::Rem),
    ("ex", CssLengthUnit::Ex),
    ("rex", CssLengthUnit::Rex),
    ("cap", CssLengthUnit::Cap),
    ("rcap", CssLengthUnit::Rcap),
    ("ch", CssLengthUnit::Ch),
    ("rch", CssLengthUnit::Rch),
    ("ic", CssLengthUnit::Ic),
    ("ric", CssLengthUnit::Ric),
    ("lh", CssLengthUnit::Lh),
    ("rlh", CssLengthUnit::Rlh),
    ("vw", CssLengthUnit::Vw),
    ("vh", CssLengthUnit::Vh),
    ("vi", CssLengthUnit::Vi),
    ("vb", CssLengthUnit::Vb),
    ("vmin", CssLengthUnit::Vmin),
    ("vmax", CssLengthUnit::Vmax),
    ("svw", CssLengthUnit::Svw),
    ("svh", CssLengthUnit::Svh),
    ("svi", CssLengthUnit::Svi),
    ("svb", CssLengthUnit::Svb),
    ("svmin", CssLengthUnit::Svmin),
    ("svmax", CssLengthUnit::Svmax),
    ("lvw", CssLengthUnit::Lvw),
    ("lvh", CssLengthUnit::Lvh),
    ("lvi", CssLengthUnit::Lvi),
    ("lvb", CssLengthUnit::Lvb),
    ("lvmin", CssLengthUnit::Lvmin),
    ("lvmax", CssLengthUnit::Lvmax),
    ("dvw", CssLengthUnit::Dvw),
    ("dvh", CssLengthUnit::Dvh),
    ("dvi", CssLengthUnit::Dvi),
    ("dvb", CssLengthUnit::Dvb),
    ("dvmin", CssLengthUnit::Dvmin),
    ("dvmax", CssLengthUnit::Dvmax),
    ("cqw", CssLengthUnit::Cqw),
    ("cqh", CssLengthUnit::Cqh),
    ("cqi", CssLengthUnit::Cqi),
    ("cqb", CssLengthUnit::Cqb),
    ("cqmin", CssLengthUnit::Cqmin),
    ("cqmax", CssLengthUnit::Cqmax),
    ("cm", CssLengthUnit::Cm),
    ("mm", CssLengthUnit::Mm),
    ("Q", CssLengthUnit::Q),
    ("in", CssLengthUnit::In),
    ("pt", CssLengthUnit::Pt),
    ("pc", CssLengthUnit::Pc),
    ("px", CssLengthUnit::Px),
];

#[test]
fn parses_calc_width_as_css_calc_length() {
    let value = property_payload!(
        declaration(
            ".panel { width: calc(20px + 10%); }",
            CssKnownProperty::Width
        ),
        Width,
        value
    );
    let calculation = sizing_calculation(&value);
    assert_eq!(
        calculation.result_type(),
        CssCalculationType::LengthPercentage
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "calc(10% + 20px)"
    );
}

#[test]
fn parses_nested_calc_width_with_subtraction_as_css_syntax() {
    let value = property_payload!(
        declaration(
            ".panel { width: calc(100% - calc(12px + 3%)); }",
            CssKnownProperty::Width
        ),
        Width,
        value
    );
    let calculation = sizing_calculation(&value);
    assert_eq!(
        calculation.result_type(),
        CssCalculationType::LengthPercentage
    );
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("calc root")
    };
    let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
        panic!("outer sum")
    };
    assert_eq!(
        sum.term(1).unwrap().operator(),
        Some(CssCalculationSumOperator::Subtract)
    );
}

#[test]
fn exposes_nested_calc_terms_structurally() {
    let value = property_payload!(
        declaration(
            ".panel { width: calc(100% - calc(12px + 3%)); }",
            CssKnownProperty::Width
        ),
        Width,
        value
    );
    let calc = sizing_calculation(&value);
    let CssCalculationExpressionRef::NestedCalc(root) = calc.expression() else {
        panic!("expected calc root")
    };
    let CssCalculationExpressionRef::Sum(terms) = root.operand() else {
        panic!("expected sum")
    };
    assert_eq!(terms.len(), 2);
    assert!(
        matches!(terms.term(0).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(v)) if v.representation() == "100")
    );
    assert_eq!(
        terms.term(1).unwrap().operator(),
        Some(CssCalculationSumOperator::Subtract)
    );
    let CssCalculationExpressionRef::NestedCalc(nested) = terms.term(1).unwrap().expression()
    else {
        panic!("expected nested calc")
    };
    let CssCalculationExpressionRef::Sum(terms) = nested.operand() else {
        panic!("expected nested sum")
    };
    assert_eq!(terms.len(), 2);
    assert!(
        matches!(terms.term(0).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(v)) if v.representation() == "12" && v.unit() == Some("px"))
    );
    assert!(
        matches!(terms.term(1).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(v)) if v.representation() == "3")
    );
}

#[test]
fn successful_declarations_expose_authored_source_position() {
    let input = ".panel {\n  height: 20px;\n  width: calc(100% - 4px);\n}\n";
    let height = declaration(input, CssKnownProperty::Height);
    let width = declaration(input, CssKnownProperty::Width);
    let height_position = height.position().expect("parsed declaration position");
    let width_position = width.position().expect("parsed declaration position");

    assert_eq!(height_position.byte_offset().value(), 11);
    assert_eq!(height_position.line().value(), 1);
    assert_eq!(height_position.column().value(), 2);
    assert_eq!(width_position.byte_offset().value(), 27);
    assert_eq!(width_position.line().value(), 2);
    assert_eq!(width_position.column().value(), 2);
}

#[test]
fn parses_supported_length_units_as_authored_dimensions() {
    let cases = [
        ("1em", 1.0, CssLengthUnit::Em),
        ("2rem", 2.0, CssLengthUnit::Rem),
        ("3vw", 3.0, CssLengthUnit::Vw),
        ("4svh", 4.0, CssLengthUnit::Svh),
        ("5lvw", 5.0, CssLengthUnit::Lvw),
        ("6dvb", 6.0, CssLengthUnit::Dvb),
        ("7cqi", 7.0, CssLengthUnit::Cqi),
        ("8cm", 8.0, CssLengthUnit::Cm),
        ("9pt", 9.0, CssLengthUnit::Pt),
    ];

    for (authored, expected_value, expected_unit) in cases {
        let value = property_payload!(
            declaration(
                &format!(".panel {{ width: {authored}; }}"),
                CssKnownProperty::Width
            ),
            Width,
            value
        );
        let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = &value else {
            panic!("ordinary dimension")
        };
        let Some(component) = length.literal().map(|literal| literal.component()) else {
            panic!("ordinary dimension token")
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
            component.view()
        else {
            panic!("dimension token")
        };
        assert_eq!(
            number.representation().parse::<f32>().unwrap(),
            expected_value
        );
        // Values 4 §6 unit names are ASCII case-insensitive.
        assert!(
            expected_unit.as_str().eq_ignore_ascii_case(unit),
            "{authored} names {expected_unit:?}"
        );
        assert_eq!(authored, format!("{}{}", number.representation(), unit));
    }
}

#[test]
fn parses_supported_calc_length_units_as_authored_dimensions() {
    let cases = [
        ("1em", 1.0, CssLengthUnit::Em),
        ("2rem", 2.0, CssLengthUnit::Rem),
        ("3vw", 3.0, CssLengthUnit::Vw),
        ("4svh", 4.0, CssLengthUnit::Svh),
        ("5lvw", 5.0, CssLengthUnit::Lvw),
        ("6dvb", 6.0, CssLengthUnit::Dvb),
        ("7cqi", 7.0, CssLengthUnit::Cqi),
        ("8cm", 8.0, CssLengthUnit::Cm),
        ("9pt", 9.0, CssLengthUnit::Pt),
    ];

    for (authored, expected_value, expected_unit) in cases {
        let value = property_payload!(
            declaration(
                &format!(".panel {{ width: calc({authored} + 2px); }}"),
                CssKnownProperty::Width
            ),
            Width,
            value
        );
        let calc = sizing_calculation(&value);
        let CssCalculationExpressionRef::NestedCalc(root) = calc.expression() else {
            panic!("expected calc root")
        };
        let CssCalculationExpressionRef::Sum(terms) = root.operand() else {
            panic!("expected sum")
        };
        assert_eq!(terms.len(), 2);
        let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(length)) =
            terms.term(0).unwrap().expression()
        else {
            panic!("expected dimension")
        };
        assert_eq!(length.representation(), expected_value.to_string());
        assert_eq!(
            length.canonical_unit(),
            Some(CssNumericUnit::Length(expected_unit))
        );
        assert!(
            matches!(terms.term(1).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(v)) if v.representation() == "2" && v.unit() == Some("px"))
        );
    }
}

#[test]
fn unit_matrix_accepts_every_supported_length_unit_in_ordinary_length_contexts() {
    for (spelling, unit) in VALUES_4_LENGTH_UNITS {
        let authored = format!("1{spelling}");
        let declaration = parse_single_declaration("width", &authored);

        assert_eq!(
            declaration.known().map(|known| known.property()),
            Some(CssKnownProperty::Width)
        );
        let Some(CssKnownPropertyValueRef::Width(value)) =
            declaration.known().and_then(|known| known.property_value())
        else {
            panic!("checked width")
        };
        let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = value.value() else {
            panic!("ordinary length")
        };
        let Some(component) = length.literal().map(|literal| literal.component()) else {
            panic!("ordinary token")
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension {
            number,
            unit: authored_unit,
        }) = component.view()
        else {
            panic!("{authored} should retain a dimension token")
        };
        assert_eq!(number.representation(), "1", "{authored}");
        assert_eq!(
            authored_unit, spelling,
            "{authored} retains its unit spelling"
        );
        assert!(
            unit.as_str().eq_ignore_ascii_case(authored_unit),
            "{authored} names its Values 4 length unit {unit:?}"
        );
    }
}

#[test]
fn unit_matrix_accepts_every_supported_length_unit_in_calc_contexts() {
    for (spelling, unit) in VALUES_4_LENGTH_UNITS {
        let authored = format!("calc(1{spelling} + 2px)");
        let declaration = parse_single_declaration("width", &authored);
        let Some(CssKnownPropertyValueRef::Width(value)) =
            declaration.known().and_then(|known| known.property_value())
        else {
            panic!("checked width")
        };
        let calc = sizing_calculation(value.value());
        let CssCalculationExpressionRef::NestedCalc(root) = calc.expression() else {
            panic!("expected calc root")
        };
        let CssCalculationExpressionRef::Sum(terms) = root.operand() else {
            panic!("expected sum")
        };
        assert_eq!(terms.len(), 2);
        assert!(
            matches!(terms.term(0).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(v)) if v.representation() == "1" && v.unit() == Some(spelling) && v.canonical_unit() == Some(CssNumericUnit::Length(unit))),
            "{authored} names its Values 4 length unit"
        );
        assert!(
            matches!(terms.term(1).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(v)) if v.representation() == "2" && v.unit() == Some("px"))
        );
    }
}

#[test]
fn unit_matrix_rejects_unknown_length_units_in_ordinary_and_calc_contexts() {
    assert_sheet_rejected(
        ".panel { width: 1quux; }",
        &ExpectedErrorKind::InvalidPropertyValue { property: "width" },
    );
    assert_sheet_rejected(
        ".panel { width: calc(1quux + 2px); }",
        &ExpectedErrorKind::InvalidPropertyValue { property: "width" },
    );
}

#[test]
fn unknown_dimension_units_are_reported_as_unknown_units() {
    let error = parse_sheet(".panel { width: 1quux; }").expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "unexpected error kind: {error:?}"
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Width);
}

#[test]
fn unknown_calc_dimension_units_are_reported_as_unknown_units() {
    let error =
        parse_sheet(".panel { width: calc(1quux + 2px); }").expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "unexpected error kind: {error:?}"
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Width);
}

#[test]
fn flow_tolerance_calc_is_preserved_as_css_syntax() {
    let declaration = declaration(
        ".panel { flow-tolerance: calc(8px + 2%); }",
        CssKnownProperty::FlowTolerance,
    );
    let Some(CssKnownPropertyValueRef::FlowTolerance(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("expected flow-tolerance wrapper");
    };
    match value.value().as_ref() {
        CssFlowToleranceRef::LengthPercentage(length) => {
            let calc = length.calculation().expect("checked flow calculation");
            assert_eq!(
                calc.numeric_type().percent_hint(),
                Some(surgeist_css::CssNumericDimension::Length)
            );
            assert_eq!(
                calc.components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                "calc(8px + 2%)"
            );
        }
        other => panic!("expected calc flow-tolerance, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_calc_functions() {
    let error = parse_sheet(".panel { width: unknown-math(10px, 20px); }")
        .expect_err("rejected parse report");
    assert!(error.code() == CssErrorCode::InvalidPropertyValue);
}

#[test]
fn parses_calc_in_edge_shorthands() {
    let sheet = parse_sheet(".panel { margin: calc(4px + 1%) 2px; }").expect("clean parse report");
    let edges = property_payload!(
        declaration(
            ".panel { margin: calc(4px + 1%) 2px; }",
            CssKnownProperty::Margin
        ),
        Margin,
        value
    );
    let [top, right, bottom, left] = edges.assigned();
    let CssMarginValue::LengthPercentage(top) = top else {
        panic!("expected checked calculation")
    };
    assert_eq!(
        top.calculation().unwrap().result_type(),
        CssCalculationType::LengthPercentage
    );
    assert_eq!(
        top.serialize_specified(Default::default()).unwrap(),
        "calc(1% + 4px)"
    );
    assert_eq!(top, bottom.length_percentage().unwrap());
    assert_eq!(right, left);
    assert_eq!(
        right.serialize_specified(Default::default()).unwrap(),
        "2px"
    );
    assert_eq!(edges.authored().len(), 2);

    assert_eq!(style_rule(&sheet.rules()[0]).declarations().len(), 1);
}

#[test]
fn parses_authored_normal_gap_without_canonicalizing_it() {
    let declaration = declaration(".panel { gap: normal; }", CssKnownProperty::Gap);
    let CssKnownPropertyValueRef::Gap(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("gap wrapper")
    };
    assert_eq!(value.value().first(), &CssGapValue::Normal);
    assert!(value.value().authored_second().is_none());
    assert_eq!(value.as_css(), "normal");
}

#[test]
fn parses_authored_calc_gap_without_canonicalizing_it() {
    let declaration = declaration(".panel { gap: calc(8px + 2%); }", CssKnownProperty::Gap);
    let CssKnownPropertyValueRef::Gap(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("gap wrapper")
    };
    match value.value().first() {
        CssGapValue::LengthPercentage(length) => {
            assert!(length.calculation().is_some());
            assert_eq!(value.as_css(), "calc(8px + 2%)");
        }
        other => panic!("expected calc gap, got {other:?}"),
    }
}
