#![forbid(unsafe_code)]

use surgeist_css::*;

fn component(text: &str) -> CssComponentValue {
    let components = parse_component_values(text).unwrap();
    let [value] = components.items() else {
        panic!("one component")
    };
    value.clone()
}

fn inset(text: &str) -> CssInsetValue {
    if text == "auto" {
        return CssInsetValue::Auto;
    }
    let numeric = if text.starts_with("calc(") {
        CssSpecifiedLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values(text).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedLengthPercentage::try_from_component(component(text)).unwrap()
    };
    CssInsetValue::LengthPercentage(numeric)
}

fn wrapper(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn position_keywords_serialize_canonically_with_exact_limits() {
    for (value, keyword) in [
        (CssLayoutPosition::Static, "static"),
        (CssLayoutPosition::Relative, "relative"),
        (CssLayoutPosition::Absolute, "absolute"),
        (CssLayoutPosition::Sticky, "sticky"),
        (CssLayoutPosition::Fixed, "fixed"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), keyword);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    keyword.len()
                ))
                .unwrap(),
            keyword
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    keyword.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }
}

#[test]
fn exact_literals_math_and_origin_blind_equality() {
    for text in [
        "1e999px",
        "-1e999%",
        "1e-999px",
        "-1e-999%",
        "0",
        "-0px",
        "calc(1px + 2%)",
    ] {
        let value = inset(text);
        let parsed = wrapper("top", text);
        let CssKnownPropertyValueRef::Top(parsed_value) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("top")
        };
        assert_eq!(parsed_value.value(), &value);
        let expected = match text {
            "1e999px" => format!("1{}px", "0".repeat(999)),
            "-1e999%" => format!("-1{}%", "0".repeat(999)),
            "1e-999px" => format!("0.{}1px", "0".repeat(998)),
            "-1e-999%" => format!("-0.{}1%", "0".repeat(998)),
            "-0px" => "0px".to_owned(),
            "calc(1px + 2%)" => "calc(2% + 1px)".to_owned(),
            other => other.to_owned(),
        };
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(parsed_value.as_css(), text);
        if let CssInsetValue::LengthPercentage(numeric) = &value {
            assert!(numeric.origin() != parsed_value.value().origin().unwrap());
        }
    }
    assert_ne!(inset("1px"), inset("1.0px"));
    assert_ne!(inset("calc(1px + 2%)"), inset("calc(2% + 1px)"));
    assert_ne!(inset("auto"), inset("0"));

    let programmatic = CssInsetValue::LengthPercentage(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_dimension("1.5", "px").unwrap(),
        )
        .unwrap(),
    );
    assert_eq!(programmatic.serialize_specified().unwrap(), "1.5px");
    assert_eq!(programmatic, inset("1.5px"));
    assert!(matches!(
        programmatic.origin(),
        Some(CssValueOrigin::Programmatic)
    ));
}

#[test]
fn pair_and_four_side_preserve_authored_arity_and_role_order() {
    let single = CssInsetPair::new(inset("-1px"), None);
    assert_eq!(single.start(), single.end());
    assert!(single.authored_end().is_none());
    assert_eq!(single.serialize_specified().unwrap(), "-1px");
    let explicit = CssInsetPair::new(inset("-1px"), Some(inset("-1px")));
    assert_ne!(single, explicit);
    assert_eq!(explicit.serialize_specified().unwrap(), "-1px -1px");
    for kind in [CssBoxSideKind::Physical, CssBoxSideKind::Logical] {
        for (authored, expected) in [
            (vec!["1px"], ["1px", "1px", "1px", "1px"]),
            (vec!["1px", "2%"], ["1px", "2%", "1px", "2%"]),
            (vec!["1px", "2%", "-3px"], ["1px", "2%", "-3px", "2%"]),
            (
                vec!["1px", "2%", "-3px", "auto"],
                ["1px", "2%", "-3px", "auto"],
            ),
        ] {
            let value =
                CssInsetShorthand::try_new(kind, authored.iter().map(|text| inset(text)).collect())
                    .unwrap();
            assert_eq!(value.authored_values().len(), authored.len());
            assert_eq!(
                value
                    .assigned_values()
                    .map(|item| item.serialize_specified().unwrap()),
                expected
            );
        }
        let value =
            CssInsetShorthand::try_new(kind, vec![inset("1px"), inset("2%"), inset("-3px")])
                .unwrap();
        let assigned = value.assigned_values();
        assert_eq!(
            assigned.map(|value| value.serialize_specified().unwrap()),
            ["1px", "2%", "-3px", "2%"]
        );
        assert_eq!(value.authored_values().len(), 3);
        let prefix = if kind == CssBoxSideKind::Logical {
            "logical "
        } else {
            ""
        };
        assert_eq!(
            value.serialize_specified().unwrap(),
            format!("{prefix}1px 2% -3px")
        );
    }
    assert!(CssInsetShorthand::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssInsetShorthand::try_new(CssBoxSideKind::Physical, vec![inset("auto"); 5]).is_none());
}

#[test]
fn aggregate_serialization_limits_include_each_authored_component() {
    let pair = CssInsetPair::new(inset("1px"), Some(inset("auto")));
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 8))
            .unwrap(),
        "1px auto"
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 2, 8))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 1, 8))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 7))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    let four =
        CssInsetShorthand::try_new(CssBoxSideKind::Logical, vec![inset("auto"), inset("1px")])
            .unwrap();
    assert_eq!(
        four.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 16))
            .unwrap(),
        "logical auto 1px"
    );
    assert_eq!(
        four.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 3, 16))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    let math = inset("calc(1px + 2%)");
    assert!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 100))
            .is_err()
    );
}

#[test]
fn physical_insets_preserve_all_checked_magnitudes() {
    for text in ["1px", "-1px", "1.5px", "10%", "auto", "0"] {
        let source = wrapper("top", text);
        let CssKnownPropertyValueRef::Top(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("top")
        };
        assert_eq!(value.value(), &inset(text));
    }
    for text in ["1e999px", "1e-999px", "16777216.00000000001px"] {
        let source = wrapper("top", text);
        let CssKnownPropertyValueRef::Top(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("top")
        };
        assert_eq!(value.value(), &inset(text));
    }
    let physical = wrapper("inset", "auto 10px 5%");
    let CssKnownPropertyValueRef::Inset(value) =
        physical.known().unwrap().property_value().unwrap()
    else {
        panic!("inset")
    };
    assert_eq!(
        value.value().authored_values(),
        &[inset("auto"), inset("10px"), inset("5%")]
    );
    assert_eq!(value.value().serialize_specified().unwrap(), "auto 10px 5%");
    assert_eq!(
        value.value().assigned_values(),
        [&inset("auto"), &inset("10px"), &inset("5%"), &inset("10px")]
    );
    assert_eq!(value.value().kind(), CssBoxSideKind::Physical);
    let logical = wrapper("inset", "logical auto 10px 5%");
    let CssKnownPropertyValueRef::Inset(value) = logical.known().unwrap().property_value().unwrap()
    else {
        panic!("inset")
    };
    assert_eq!(value.value().kind(), CssBoxSideKind::Logical);
}

#[test]
fn physical_wrappers_compare_authored_math_without_diagnostic_origins() {
    let first = parse_style_attribute("top:calc(1px + 2%);");
    let second = parse_style_attribute("color:red; top:calc(1px + 2%);");
    assert!(first.is_clean() && second.is_clean());
    let CssKnownPropertyValueRef::Top(left) =
        first.syntax()[0].known().unwrap().property_value().unwrap()
    else {
        panic!("top")
    };
    let CssKnownPropertyValueRef::Top(right) = second.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("top")
    };
    assert_eq!(left.as_css(), "calc(1px + 2%)");
    assert_eq!(left, right);
    assert_ne!(left.value().origin(), right.value().origin());

    let distinct = wrapper("top", "calc(2% + 1px)");
    let CssKnownPropertyValueRef::Top(distinct) =
        distinct.known().unwrap().property_value().unwrap()
    else {
        panic!("top")
    };
    assert_ne!(left, distinct);
    let spelling = wrapper("top", "CALC(1px + 2%)");
    let CssKnownPropertyValueRef::Top(spelling) =
        spelling.known().unwrap().property_value().unwrap()
    else {
        panic!("top")
    };
    assert_ne!(left, spelling);

    let first = wrapper("inset", "auto calc(1px + 2%)");
    let second = parse_style_attribute("color:red; inset:auto calc(1px + 2%);");
    assert!(second.is_clean());
    let CssKnownPropertyValueRef::Inset(left) = first.known().unwrap().property_value().unwrap()
    else {
        panic!("inset")
    };
    let CssKnownPropertyValueRef::Inset(right) = second.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("inset")
    };
    assert_eq!(left, right);
}
