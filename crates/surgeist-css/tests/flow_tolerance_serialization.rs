#![forbid(unsafe_code)]
//! Grid3 #placement-tolerance specifies signed length-percentage and symbolic
//! keywords. Canonical scalar outputs follow the shared specified-value contract.
//! Determinate math keeps a calc wrapper, as independently specified by
//! opacity_specified_serialization's determinate-calculation expectations.
//! This new serializer API has no executable preimplementation RED boundary.

use surgeist_css::{
    CssComponentValueRef, CssFlowTolerance, CssFlowToleranceRef, CssKnownPropertyValueRef,
    CssLengthPercentageCalculation, CssSpecifiedLengthPercentage,
    CssSpecifiedValueSerializationErrorKind as ErrorKind,
    CssSpecifiedValueSerializationLimits as Limits, CssValueTokenRef, parse_component_values,
    parse_style_attribute,
};

fn constructed(text: &str) -> CssFlowTolerance {
    match text {
        "normal" => CssFlowTolerance::normal(),
        "infinite" => CssFlowTolerance::infinite(),
        _ => {
            let components = parse_component_values(text).unwrap();
            let scalar = if text.contains('(') {
                CssSpecifiedLengthPercentage::try_from_calculation(
                    CssLengthPercentageCalculation::try_from_components(components).unwrap(),
                )
                .unwrap()
            } else {
                CssSpecifiedLengthPercentage::try_from_component(components.items()[0].clone())
                    .unwrap()
            };
            CssFlowTolerance::length_percentage(scalar)
        }
    }
}

fn parsed(text: &str) -> CssFlowTolerance {
    let report = parse_style_attribute(&format!("flow-tolerance:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let Some(CssKnownPropertyValueRef::FlowTolerance(wrapper)) =
        report.syntax()[0].known().unwrap().property_value()
    else {
        panic!("ordinary checked flow tolerance")
    };
    wrapper.value().clone()
}

#[test]
fn parsed_and_constructed_tolerance_emit_independent_canonical_values() {
    for (input, expected) in [
        ("normal", "normal"),
        ("infinite", "infinite"),
        ("+02.500EM", "2.5em"),
        ("-25.000%", "-25%"),
        ("-0", "0"),
        ("-0px", "0px"),
        ("-0%", "0%"),
        (r"2\50 X", "2px"),
        ("calc(2px + 3px)", "calc(5px)"),
        ("calc(2em + 3px)", "calc(2em + 3px)"),
        ("calc(-2px - 3%)", "calc(-3% - 2px)"),
        ("calc(2 * 3em)", "calc(6em)"),
        ("min(-2px, 3px)", "calc(-2px)"),
        // Values 4 §10.10.1 keeps comparisons symbolic while the percentage
        // basis is unresolved; coefficient order alone is not a proof.
        ("max(-2%, -3%)", "max(-2%, -3%)"),
        ("clamp(-2px, 1px, 3px)", "calc(1px)"),
        ("min(1px, 2em)", "min(1px, 2em)"),
        ("max(1px, 2%)", "max(1px, 2%)"),
        ("clamp(1px, 2em, 3px)", "clamp(1px, 2em, 3px)"),
    ] {
        for value in [parsed(input), constructed(input)] {
            let before = value.clone();
            let css = value.serialize_specified().unwrap();
            assert_eq!(css, expected, "{input}");
            assert_eq!(parsed(&css).serialize_specified().unwrap(), expected);
            assert_eq!(value, before);
        }
    }
    for input in ["NoRmAl", r"\69 nfinite"] {
        let expected = if input == "NoRmAl" {
            "normal"
        } else {
            "infinite"
        };
        assert_eq!(parsed(input).serialize_specified().unwrap(), expected);
    }
    assert_eq!(
        CssFlowTolerance::default().serialize_specified().unwrap(),
        "normal"
    );
}

#[test]
fn extreme_signed_literals_preserve_exact_magnitudes_and_output_limits() {
    for (input, expected) in [
        ("-1e999px", format!("-1{}px", "0".repeat(999))),
        ("-1e-999%", "0%".to_owned()),
    ] {
        for value in [parsed(input), constructed(input)] {
            let CssFlowToleranceRef::LengthPercentage(scalar) = value.as_ref() else {
                panic!("numeric tolerance")
            };
            let expected_coefficient = input.trim_end_matches("px").trim_end_matches('%');
            assert!(
                matches!(
                    scalar.literal_component().unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                        if number.representation() == expected_coefficient && unit == "px"
                ) || matches!(
                    scalar.literal_component().unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
                        if number.representation() == expected_coefficient
                )
            );
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert_eq!(
                value
                    .serialize_specified_with_limits(Limits::new(1, 1, expected.len()))
                    .unwrap(),
                expected
            );
            let before = value.clone();
            assert_eq!(
                value
                    .serialize_specified_with_limits(Limits::new(1, 1, expected.len() - 1))
                    .unwrap_err()
                    .kind(),
                ErrorKind::ByteLimit
            );
            assert_eq!(value, before);
        }
    }
}

#[test]
fn keywords_literals_and_calculations_obey_each_exact_resource_boundary() {
    // The shared scalar owner charges four input nodes (function, sum, two
    // literals) and five projection nodes for this two-unit symbolic sum.
    for (input, expected, input_nodes, projection_nodes) in [
        ("normal", "normal", 1, 1),
        ("infinite", "infinite", 1, 1),
        ("-2px", "-2px", 1, 1),
        ("calc(1px + 2em)", "calc(2em + 1px)", 4, 5),
    ] {
        let value = constructed(input);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(
                    input_nodes,
                    projection_nodes,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(input_nodes - 1, projection_nodes, expected.len()),
                ErrorKind::InputNodeLimit,
            ),
            (
                Limits::new(input_nodes, projection_nodes - 1, expected.len()),
                ErrorKind::ProjectionNodeLimit,
            ),
            (
                Limits::new(input_nodes, projection_nodes, expected.len() - 1),
                ErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind,
                "{input}"
            );
            assert_eq!(value, before);
        }
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}
