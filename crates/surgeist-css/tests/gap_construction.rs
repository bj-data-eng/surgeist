#![forbid(unsafe_code)]

//! Typed authored gap construction and specified serialization, from Box Alignment 3 §8.

use surgeist_css::*;

fn scalar(text: &str) -> CssGapValue {
    if text.eq_ignore_ascii_case("normal") {
        return CssGapValue::Normal;
    }
    let components = parse_component_values(text).unwrap();
    let numeric = if text.starts_with("calc(") {
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        let [component] = components.items() else {
            panic!("one numeric gap component: {text}")
        };
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component.clone()).unwrap()
    };
    CssGapValue::LengthPercentage(numeric)
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}");
    let report = parse_style_attribute(&source);
    assert!(
        report.is_clean(),
        "{source}: {} diagnostic(s)",
        report.diagnostics().len()
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one gap declaration: {source}")
    };
    declaration.clone()
}

fn parsed_gap(value: &str) -> CssGapShorthand {
    let source = declaration("gap", value);
    let CssKnownPropertyValueRef::Gap(gap) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("gap wrapper")
    };
    gap.value().clone()
}

#[test]
fn constructed_gap_preserves_authored_arity_and_numeric_origins() {
    let row = CssGapValue::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_dimension("1.25", "px").unwrap(),
        )
        .unwrap(),
    );
    let one = CssGapShorthand::new(row.clone(), None);
    assert!(one.authored_column().is_none());
    assert_eq!(one.row(), one.column());
    assert_eq!(one.serialize_specified().unwrap(), "1.25px");
    assert_eq!(one, parsed_gap("1.25px"));
    let CssGapValue::LengthPercentage(row_numeric) = one.row() else {
        panic!("numeric row")
    };
    assert!(matches!(row_numeric.origin(), CssValueOrigin::Programmatic));
    let parsed = parsed_gap("1.25px 2%");
    assert_eq!(parsed, CssGapShorthand::new(row, Some(scalar("2%"))));
    assert!(parsed.authored_column().is_some());
    let CssGapValue::LengthPercentage(parsed_row) = parsed.row() else {
        panic!("parsed numeric row")
    };
    assert!(matches!(parsed_row.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(parsed.serialize_specified().unwrap(), "1.25px 2%");
    assert_ne!(
        one,
        CssGapShorthand::new(scalar("1.25px"), Some(scalar("1.25px")))
    );
}

#[test]
fn normal_and_symbolic_numeric_values_remain_distinct() {
    let normal = CssGapValue::Normal;
    assert_eq!(normal.serialize_specified().unwrap(), "normal");
    assert_eq!(
        parsed_gap("normal 2px"),
        CssGapShorthand::new(normal.clone(), Some(scalar("2px")))
    );
    assert_eq!(parsed_gap("normal").column(), &normal);

    let symbolic = scalar("calc(1px + 2%)");
    let CssGapValue::LengthPercentage(value) = &symbolic else {
        panic!("checked symbolic numeric")
    };
    assert!(value.calculation().is_some());
    assert_eq!(symbolic.serialize_specified().unwrap(), "calc(2% + 1px)");
    assert_eq!(
        CssGapShorthand::new(symbolic, Some(scalar("3px")))
            .serialize_specified()
            .unwrap(),
        "calc(2% + 1px) 3px"
    );
    for text in ["1e100px", "1e100%", "1e-100px", "1e-100%", "-0px", "-0%"] {
        assert!(
            matches!(scalar(text), CssGapValue::LengthPercentage(_)),
            "{text}"
        );
    }
}

#[test]
fn parsed_and_checked_wrappers_expose_exact_current_models() {
    for (name, value) in [
        ("gap", "1px 2%"),
        ("row-gap", "normal"),
        ("column-gap", "calc(1px + 2%)"),
    ] {
        let parsed = declaration(name, value);
        let checked = parse_property_value_for_grammar(
            CssPropertyGrammar::from_name(name).unwrap(),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let expected = match name {
            "gap" => CssGapShorthand::new(scalar("1px"), Some(scalar("2%")))
                .serialize_specified()
                .unwrap(),
            "row-gap" => CssGapValue::Normal.serialize_specified().unwrap(),
            _ => scalar("calc(1px + 2%)").serialize_specified().unwrap(),
        };
        for source in [parsed, checked] {
            let model = source.known().unwrap().property_value().unwrap();
            let rendered = match model {
                CssKnownPropertyValueRef::Gap(value) => {
                    value.value().serialize_specified().unwrap()
                }
                CssKnownPropertyValueRef::RowGap(value) => {
                    value.value().serialize_specified().unwrap()
                }
                CssKnownPropertyValueRef::ColumnGap(value) => {
                    value.value().serialize_specified().unwrap()
                }
                _ => panic!("gap-family wrapper"),
            };
            assert_eq!(rendered, expected, "{name}:{value}");
        }
    }
}
