//! Typed authored gap construction and specified serialization, from Box Alignment 3 §8.

use crate::css::support::normal_declaration as declaration;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::numeric::{CssCalculation, CssLengthPercentage, CssNonNegative, CssNumeric};
use surgeist_css::property::{CssKnownPropertyValueRef, CssPropertyGrammar, CssPropertyTarget};
use surgeist_css::values::common::CssSidePair;
use surgeist_css::values::layout::CssGapValue;

fn scalar(text: &str) -> CssGapValue {
    if text.eq_ignore_ascii_case("normal") {
        return CssGapValue::Normal;
    }
    let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let numeric = if text.starts_with("calc(") {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                components,
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        let [component] = components.items() else {
            panic!("one numeric gap component: {text}")
        };
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component.clone())
            .unwrap()
    };
    CssGapValue::LengthPercentage(numeric)
}

fn parsed_gap(value: &str) -> CssSidePair<CssGapValue> {
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
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension("1.25", "px").unwrap(),
        )
        .unwrap(),
    );
    let one = CssSidePair::new(row.clone(), None);
    assert!(one.authored_second().is_none());
    assert_eq!(one.first(), one.second());
    assert_eq!(
        one.serialize_specified(Default::default()).unwrap(),
        "1.25px"
    );
    assert_eq!(one, parsed_gap("1.25px"));
    let CssGapValue::LengthPercentage(row_numeric) = one.first() else {
        panic!("numeric row")
    };
    assert!(matches!(row_numeric.origin(), CssValueOrigin::Programmatic));
    let parsed = parsed_gap("1.25px 2%");
    assert_eq!(parsed, CssSidePair::new(row, Some(scalar("2%"))));
    assert!(parsed.authored_second().is_some());
    let CssGapValue::LengthPercentage(parsed_row) = parsed.first() else {
        panic!("parsed numeric row")
    };
    assert!(matches!(parsed_row.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        parsed.serialize_specified(Default::default()).unwrap(),
        "1.25px 2%"
    );
    assert_ne!(
        one,
        CssSidePair::new(scalar("1.25px"), Some(scalar("1.25px")))
    );
}

#[test]
fn normal_and_symbolic_numeric_values_remain_distinct() {
    let normal = CssGapValue::Normal;
    assert_eq!(
        normal.serialize_specified(Default::default()).unwrap(),
        "normal"
    );
    assert_eq!(
        parsed_gap("normal 2px"),
        CssSidePair::new(normal.clone(), Some(scalar("2px")))
    );
    assert_eq!(parsed_gap("normal").second(), &normal);

    let symbolic = scalar("calc(1px + 2%)");
    let CssGapValue::LengthPercentage(value) = &symbolic else {
        panic!("checked symbolic numeric")
    };
    assert!(value.calculation().is_some());
    assert_eq!(
        symbolic.serialize_specified(Default::default()).unwrap(),
        "calc(2% + 1px)"
    );
    assert_eq!(
        CssSidePair::new(symbolic, Some(scalar("3px")))
            .serialize_specified(Default::default())
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
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(CssPropertyGrammar::from_name(name).unwrap()),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let expected = match name {
            "gap" => CssSidePair::new(scalar("1px"), Some(scalar("2%")))
                .serialize_specified(Default::default())
                .unwrap(),
            "row-gap" => CssGapValue::Normal
                .serialize_specified(Default::default())
                .unwrap(),
            _ => scalar("calc(1px + 2%)")
                .serialize_specified(Default::default())
                .unwrap(),
        };
        for source in [parsed, checked] {
            let model = source.known().unwrap().property_value().unwrap();
            let rendered = match model {
                CssKnownPropertyValueRef::Gap(value) => value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                CssKnownPropertyValueRef::RowGap(value) => value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                CssKnownPropertyValueRef::ColumnGap(value) => value
                    .value()
                    .serialize_specified(Default::default())
                    .unwrap(),
                _ => panic!("gap-family wrapper"),
            };
            assert_eq!(rendered, expected, "{name}:{value}");
        }
    }
}
