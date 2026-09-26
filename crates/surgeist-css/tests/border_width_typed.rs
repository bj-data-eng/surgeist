#![forbid(unsafe_code)]

use surgeist_css::*;

fn length(number: &str, unit: &str) -> CssBorderWidth {
    CssBorderWidth::Length(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension(number, unit).unwrap(),
        )
        .unwrap(),
    )
}

fn parsed_width(text: &str) -> CssBorderWidth {
    let report = parse_style_attribute(&format!("border-top-width:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderTopWidth(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed width")
    };
    value.current().clone()
}

fn red() -> CssAuthoredColor {
    let report = parse_style_attribute("border-top-color:red");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BorderTopColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed color")
    };
    value.current().clone()
}

#[test]
fn checked_widths_preserve_exact_literals_and_origin_blind_structure() {
    for (value, expected) in [
        (CssBorderWidth::Thin, "thin"),
        (CssBorderWidth::Medium, "medium"),
        (CssBorderWidth::Thick, "thick"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    assert_eq!(
        length("1e100", "px").serialize_specified().unwrap(),
        format!("1{}px", "0".repeat(100))
    );
    assert_eq!(
        length("1e-100", "px").serialize_specified().unwrap(),
        format!("0.{}1px", "0".repeat(99))
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("-1e-100", "px").unwrap()
        )
        .is_err()
    );
    assert_eq!(length("-0", "px").serialize_specified().unwrap(), "0px");
    let programmatic = length("1", "px");
    let parsed = parsed_width("1px");
    assert_eq!(programmatic, parsed);
    assert!(matches!(
        programmatic.origin(),
        Some(CssValueOrigin::Programmatic)
    ));
    assert!(matches!(parsed.origin(), Some(CssValueOrigin::Parsed(_))));
    assert_ne!(programmatic, parsed_width("1.0px"));
    assert_ne!(CssBorderWidth::Thin, CssBorderWidth::Medium);
    assert_ne!(CssBorderWidth::Thin, parsed_width("1px"));
}

#[test]
fn pair_and_four_side_keep_arity_roles_and_cumulative_budgets() {
    let pair = CssBorderWidthPair::new(CssBorderWidth::Thin, Some(CssBorderWidth::Thick));
    assert_eq!(pair.start(), &CssBorderWidth::Thin);
    assert_eq!(pair.authored_end(), Some(&CssBorderWidth::Thick));
    assert_eq!(pair.end(), &CssBorderWidth::Thick);
    assert_eq!(pair.serialize_specified().unwrap(), "thin thick");
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 10))
            .unwrap(),
        "thin thick"
    );
    for limit in [
        CssSpecifiedValueSerializationLimits::new(2, 3, 10),
        CssSpecifiedValueSerializationLimits::new(3, 2, 10),
        CssSpecifiedValueSerializationLimits::new(3, 3, 9),
    ] {
        assert!(pair.serialize_specified_with_limits(limit).is_err());
    }
    let one = CssBorderWidthPair::new(CssBorderWidth::Thin, None);
    assert_eq!(one.authored_end(), None);
    assert_eq!(one.end(), one.start());
    assert_ne!(
        one,
        CssBorderWidthPair::new(CssBorderWidth::Thin, Some(CssBorderWidth::Thin))
    );
    assert!(CssBorderWidthShorthand::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(
        CssBorderWidthShorthand::try_new(CssBoxSideKind::Physical, vec![CssBorderWidth::Thin; 5])
            .is_none()
    );
    for (values, expected) in [
        (vec![CssBorderWidth::Thin], ["thin", "thin", "thin", "thin"]),
        (
            vec![CssBorderWidth::Thin, CssBorderWidth::Medium],
            ["thin", "medium", "thin", "medium"],
        ),
        (
            vec![
                CssBorderWidth::Thin,
                CssBorderWidth::Medium,
                CssBorderWidth::Thick,
            ],
            ["thin", "medium", "thick", "medium"],
        ),
        (
            vec![
                CssBorderWidth::Thin,
                CssBorderWidth::Medium,
                CssBorderWidth::Thick,
                length("0", "px"),
            ],
            ["thin", "medium", "thick", "0px"],
        ),
    ] {
        for kind in [CssBoxSideKind::Physical, CssBoxSideKind::Logical] {
            let value = CssBorderWidthShorthand::try_new(kind, values.clone()).unwrap();
            assert_eq!(value.kind(), kind);
            assert_eq!(value.authored_values().len(), values.len());
            assert_eq!(
                value
                    .assigned_values()
                    .map(|part| part.serialize_specified().unwrap()),
                expected
            );
        }
    }
    let shorthand = CssBorderWidthShorthand::try_new(
        CssBoxSideKind::Logical,
        vec![
            CssBorderWidth::Thin,
            CssBorderWidth::Medium,
            CssBorderWidth::Thick,
            CssBorderWidth::Length(CssSpecifiedNonNegativeLength::zero()),
        ],
    )
    .unwrap();
    assert_eq!(
        shorthand.serialize_specified().unwrap(),
        "logical thin medium thick 0"
    );
    assert_eq!(
        shorthand
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 27))
            .unwrap(),
        "logical thin medium thick 0"
    );
    for limit in [
        CssSpecifiedValueSerializationLimits::new(4, 5, 27),
        CssSpecifiedValueSerializationLimits::new(5, 4, 27),
        CssSpecifiedValueSerializationLimits::new(5, 5, 26),
    ] {
        assert!(shorthand.serialize_specified_with_limits(limit).is_err());
    }
    assert_ne!(
        shorthand,
        CssBorderWidthShorthand::try_new(
            CssBoxSideKind::Physical,
            shorthand.authored_values().to_vec()
        )
        .unwrap()
    );
}

#[test]
fn border_triple_retains_omissions_and_exact_shared_budget() {
    assert!(CssBorderValue::try_new(None, None, None).is_none());
    let value = CssBorderValue::try_new(
        Some(CssBorderWidth::Thin),
        Some(CssBorderStyle::Solid),
        Some(red()),
    )
    .unwrap();
    assert_eq!(value.width(), Some(&CssBorderWidth::Thin));
    assert_eq!(value.style(), Some(CssBorderStyle::Solid));
    assert_eq!(value.color().unwrap().to_specified_css().unwrap(), "red");
    assert_eq!(value.serialize_specified().unwrap(), "thin solid red");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 4, 14))
            .unwrap(),
        "thin solid red"
    );
    for limit in [
        CssSpecifiedValueSerializationLimits::new(3, 4, 14),
        CssSpecifiedValueSerializationLimits::new(4, 3, 14),
        CssSpecifiedValueSerializationLimits::new(4, 4, 13),
    ] {
        assert!(value.serialize_specified_with_limits(limit).is_err());
    }
    let reordered = parse_style_attribute("border:red solid thin");
    assert!(reordered.is_clean());
    let CssKnownPropertyValueRef::Border(wrapper) = reordered.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed border")
    };
    assert_eq!(wrapper.current(), &value);
    assert_eq!(wrapper.as_css(), "red solid thin");
    assert_ne!(
        value,
        CssBorderValue::try_new(
            Some(CssBorderWidth::Thin),
            Some(CssBorderStyle::Solid),
            None
        )
        .unwrap()
    );
}

#[test]
fn pure_length_calculation_projects_without_a_percentage_basis_or_tree_reinterpretation() {
    let report = parse_style_attribute("border-top-width:calc(10% / 10% * 1px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderTopWidth(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed width")
    };
    let CssBorderWidth::Length(current) = wrapper.current() else {
        panic!("exact math")
    };
    let original = current.calculation().unwrap();
    let CssLength::Calc(CssCalcLength::Typed(legacy)) = wrapper.i01_subset().unwrap() else {
        panic!("typed I01 math")
    };
    assert_eq!(legacy.components(), original.components());
    assert_eq!(legacy.numeric_type(), original.numeric_type());
    assert_eq!(legacy.numeric_type().percent_hint(), None);
    assert_eq!(
        legacy
            .numeric_type()
            .exponent(CssNumericDimension::Percentage),
        0
    );
    assert_eq!(
        legacy.numeric_type().exponent(CssNumericDimension::Length),
        1
    );
    assert_eq!(legacy.origin(), original.origin());
}

#[test]
fn physical_compatibility_projection_never_narrows_exact_literals() {
    for (text, expected) in [
        ("1px", true),
        ("16777216px", true),
        ("-0px", true),
        ("1e100px", false),
        ("1e-100px", false),
        ("16777216.00000000001px", false),
    ] {
        let report = parse_style_attribute(&format!("border-top-width:{text}"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderTopWidth(width) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("physical width")
        };
        assert_eq!(width.i01_subset().is_some(), expected, "{text}");
        assert_eq!(width.current(), &parsed_width(text));
    }
    let logical = parse_style_attribute("border-width:logical thin medium");
    assert!(logical.is_clean());
    let CssKnownPropertyValueRef::BorderWidth(value) = logical.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("logical widths")
    };
    assert!(value.i01_subset().is_none());
    assert_eq!(value.current().kind(), CssBoxSideKind::Logical);
}

#[test]
fn symbolic_border_triple_shares_one_numeric_and_color_budget() {
    // Numeric specified serialization orders unlike units canonically; the
    // same order is asserted by the Overflow 3 checked length consumer.
    let value = CssBorderValue::try_new(
        Some(parsed_width("calc(1px + 2em)")),
        Some(CssBorderStyle::Solid),
        Some(red()),
    )
    .unwrap();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc(2em + 1px) solid red"
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(7, 8, 25))
            .unwrap(),
        "calc(2em + 1px) solid red"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, 8, 25),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 7, 25),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 8, 24),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}
