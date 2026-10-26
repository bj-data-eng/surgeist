use surgeist_css::*;

fn component(text: &str) -> CssComponentValue {
    let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
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
        CssNumeric::<CssLengthPercentage>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLengthPercentage>::try_from_component(component(text)).unwrap()
    };
    CssInsetValue::LengthPercentage(numeric)
}

fn wrapper(name: &str, text: &str) -> CssDeclaration {
    let report = CssParser::default().parse_block_contents(format!("{name}:{text}"));
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
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            keyword
        );
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, keyword.len()))
                .unwrap(),
            keyword
        );
        assert!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, keyword.len() - 1))
                .unwrap_err()
                .is_byte_limit()
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
            "1e-999px" => "0px".to_owned(),
            "-1e-999%" => "0%".to_owned(),
            "-0px" => "0px".to_owned(),
            "calc(1px + 2%)" => "calc(2% + 1px)".to_owned(),
            other => other.to_owned(),
        };
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(parsed_value.as_css(), text);
        if let CssInsetValue::LengthPercentage(numeric) = &value {
            assert!(numeric.origin() != parsed_value.value().origin().unwrap());
        }
    }
    assert_eq!(inset("1px"), inset("1.0px"));
    assert_ne!(inset("calc(1px + 2%)"), inset("calc(2% + 1px)"));
    assert_ne!(inset("auto"), inset("0"));

    let programmatic = CssInsetValue::LengthPercentage(
        CssNumeric::<CssLengthPercentage>::try_from_component(
            CssComponentValue::try_dimension("1.5", "px").unwrap(),
        )
        .unwrap(),
    );
    assert_eq!(
        programmatic
            .serialize_specified(Default::default())
            .unwrap(),
        "1.5px"
    );
    assert_eq!(programmatic, inset("1.5px"));
    assert!(matches!(
        programmatic.origin(),
        Some(CssValueOrigin::Programmatic)
    ));
}

#[test]
fn pair_and_four_side_preserve_authored_arity_and_role_order() {
    let single = CssSidePair::new(inset("-1px"), None);
    assert_eq!(single.first(), single.second());
    assert!(single.authored_second().is_none());
    assert_eq!(
        single.serialize_specified(Default::default()).unwrap(),
        "-1px"
    );
    let explicit = CssSidePair::new(inset("-1px"), Some(inset("-1px")));
    assert_ne!(single, explicit);
    assert_eq!(
        explicit.serialize_specified(Default::default()).unwrap(),
        "-1px"
    );
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
                CssFourSides::try_new(kind, authored.iter().map(|text| inset(text)).collect())
                    .unwrap();
            assert_eq!(value.authored().len(), authored.len());
            assert_eq!(
                value
                    .assigned()
                    .map(|item| item.serialize_specified(Default::default()).unwrap()),
                expected
            );
        }
        let value =
            CssFourSides::try_new(kind, vec![inset("1px"), inset("2%"), inset("-3px")]).unwrap();
        let assigned = value.assigned();
        assert_eq!(
            assigned.map(|value| value.serialize_specified(Default::default()).unwrap()),
            ["1px", "2%", "-3px", "2%"]
        );
        assert_eq!(value.authored().len(), 3);
        let prefix = if kind == CssBoxSideKind::Logical {
            "logical "
        } else {
            ""
        };
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            format!("{prefix}1px 2% -3px")
        );
    }
    assert!(CssFourSides::<CssInsetValue>::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssFourSides::try_new(CssBoxSideKind::Physical, vec![inset("auto"); 5]).is_none());
}

#[test]
fn aggregate_serialization_limits_include_each_authored_component() {
    // One aggregate node plus each authored component.
    let pair = CssSidePair::new(inset("1px"), Some(inset("auto")));
    assert_eq!(
        pair.serialize_specified(CssSerializationLimits::new(3, 3, 8))
            .unwrap(),
        "1px auto"
    );
    assert!(
        pair.serialize_specified(CssSerializationLimits::new(2, 3, 8))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        pair.serialize_specified(CssSerializationLimits::new(3, 2, 8))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        pair.serialize_specified(CssSerializationLimits::new(3, 3, 7))
            .unwrap_err()
            .is_byte_limit()
    );
    let four =
        CssFourSides::try_new(CssBoxSideKind::Logical, vec![inset("auto"), inset("1px")]).unwrap();
    assert_eq!(
        four.serialize_specified(CssSerializationLimits::new(3, 3, 16))
            .unwrap(),
        "logical auto 1px"
    );
    assert!(
        four.serialize_specified(CssSerializationLimits::new(2, 3, 16))
            .unwrap_err()
            .is_input_node_limit()
    );
    let math = inset("calc(1px + 2%)");
    // `calc(1px + 2%)` has more than one authored input node.
    assert!(
        math.serialize_specified(CssSerializationLimits::new(1, 1, 100))
            .unwrap_err()
            .is_input_node_limit()
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
        value.value().authored(),
        &[inset("auto"), inset("10px"), inset("5%")]
    );
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "auto 10px 5%"
    );
    assert_eq!(
        value.value().assigned(),
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
    let first = CssParser::default().parse_block_contents("top:calc(1px + 2%);");
    let second = CssParser::default().parse_block_contents("color:red; top:calc(1px + 2%);");
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
    let second = CssParser::default().parse_block_contents("color:red; inset:auto calc(1px + 2%);");
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
