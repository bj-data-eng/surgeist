use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::assert_component_rejected;
use surgeist_css::*;

fn length(number: &str, unit: &str) -> CssLineWidth {
    CssLineWidth::Length(
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension(number, unit).unwrap(),
        )
        .unwrap(),
    )
}

fn parsed_width(text: &str) -> CssLineWidth {
    let report = CssParser::default().parse_block_contents(format!("border-top-width:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderTopWidth(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed width")
    };
    value.value().clone()
}

fn red() -> CssColor {
    let report = CssParser::default().parse_block_contents("border-top-color:red");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BorderTopColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed color")
    };
    value.value().clone()
}

#[test]
fn checked_widths_preserve_exact_literals_and_origin_blind_structure() {
    for (value, expected) in [
        (CssLineWidth::Thin, "thin"),
        (CssLineWidth::Medium, "medium"),
        (CssLineWidth::Thick, "thick"),
    ] {
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
    assert_eq!(
        length("1e100", "px")
            .serialize_specified(Default::default())
            .unwrap(),
        format!("1{}px", "0".repeat(100))
    );
    let tiny = length("1e-100", "px");
    let before = tiny.clone();
    assert_eq!(tiny.serialize_specified(Default::default()).unwrap(), "0px");
    assert!(matches!(
        tiny.length().unwrap().literal().map(|literal| literal.component()).unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
            if number.representation() == "1e-100" && unit == "px"
    ));
    assert_eq!(tiny, before);
    assert_component_rejected(
        CssNumeric::<CssLength, CssNonNegative>::try_from_component,
        CssComponentValue::try_dimension("-1e-100", "px").unwrap(),
        NumericFailure::OutOfRange,
    );
    assert_eq!(
        length("-0", "px")
            .serialize_specified(Default::default())
            .unwrap(),
        "0px"
    );
    let programmatic = length("1", "px");
    let parsed = parsed_width("1px");
    assert_eq!(programmatic, parsed);
    assert!(matches!(
        programmatic.origin(),
        Some(CssValueOrigin::Programmatic)
    ));
    assert!(matches!(parsed.origin(), Some(CssValueOrigin::Parsed(_))));
    assert_eq!(programmatic, parsed_width("1.0px"));
    assert_ne!(CssLineWidth::Thin, CssLineWidth::Medium);
    assert_ne!(CssLineWidth::Thin, parsed_width("1px"));
}

#[test]
fn pair_and_four_side_keep_arity_roles_and_cumulative_budgets() {
    let pair = CssSidePair::new(CssLineWidth::Thin, Some(CssLineWidth::Thick));
    assert_eq!(pair.first(), &CssLineWidth::Thin);
    assert_eq!(pair.authored_second(), Some(&CssLineWidth::Thick));
    assert_eq!(pair.second(), &CssLineWidth::Thick);
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "thin thick"
    );
    assert_eq!(
        pair.serialize_specified(CssSerializationLimits::new(3, 3, 10))
            .unwrap(),
        "thin thick"
    );
    for (limit, kind) in [
        (
            CssSerializationLimits::new(2, 3, 10),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 10),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 9),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&pair.serialize_specified(limit).unwrap_err()),
            kind
        );
    }
    let one = CssSidePair::new(CssLineWidth::Thin, None);
    assert_eq!(one.authored_second(), None);
    assert_eq!(one.second(), one.first());
    assert_ne!(
        one,
        CssSidePair::new(CssLineWidth::Thin, Some(CssLineWidth::Thin))
    );
    assert!(CssFourSides::<CssLineWidth>::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssFourSides::try_new(CssBoxSideKind::Physical, vec![CssLineWidth::Thin; 5]).is_none());
    for (values, expected) in [
        (vec![CssLineWidth::Thin], ["thin", "thin", "thin", "thin"]),
        (
            vec![CssLineWidth::Thin, CssLineWidth::Medium],
            ["thin", "medium", "thin", "medium"],
        ),
        (
            vec![
                CssLineWidth::Thin,
                CssLineWidth::Medium,
                CssLineWidth::Thick,
            ],
            ["thin", "medium", "thick", "medium"],
        ),
        (
            vec![
                CssLineWidth::Thin,
                CssLineWidth::Medium,
                CssLineWidth::Thick,
                length("0", "px"),
            ],
            ["thin", "medium", "thick", "0px"],
        ),
    ] {
        for kind in [CssBoxSideKind::Physical, CssBoxSideKind::Logical] {
            let value = CssFourSides::try_new(kind, values.clone()).unwrap();
            assert_eq!(value.kind(), kind);
            assert_eq!(value.authored().len(), values.len());
            assert_eq!(
                value
                    .assigned()
                    .map(|part| part.serialize_specified(Default::default()).unwrap()),
                expected
            );
        }
    }
    let shorthand = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        vec![
            CssLineWidth::Thin,
            CssLineWidth::Medium,
            CssLineWidth::Thick,
            CssLineWidth::Length(
                CssNumeric::<CssLength, CssNonNegative>::try_from_component(
                    CssComponentValue::try_number("0").unwrap(),
                )
                .unwrap(),
            ),
        ],
    )
    .unwrap();
    assert_eq!(
        shorthand.serialize_specified(Default::default()).unwrap(),
        "logical thin medium thick 0"
    );
    assert_eq!(
        shorthand
            .serialize_specified(CssSerializationLimits::new(5, 5, 27))
            .unwrap(),
        "logical thin medium thick 0"
    );
    for (limit, kind) in [
        (
            CssSerializationLimits::new(4, 5, 27),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 4, 27),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 5, 26),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&shorthand.serialize_specified(limit).unwrap_err()),
            kind
        );
    }
    assert_ne!(
        shorthand,
        CssFourSides::try_new(CssBoxSideKind::Physical, shorthand.authored().to_vec()).unwrap()
    );
}

#[test]
fn border_triple_retains_omissions_and_exact_shared_budget() {
    assert!(CssBorder::try_new(None, None, None).is_none());
    let value = CssBorder::try_new(
        Some(CssLineWidth::Thin),
        Some(CssLineStyle::Solid),
        Some(red()),
    )
    .unwrap();
    assert_eq!(value.width(), Some(&CssLineWidth::Thin));
    assert_eq!(value.style(), Some(CssLineStyle::Solid));
    assert_eq!(
        value
            .color()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "red"
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "thin solid red"
    );
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(4, 4, 14))
            .unwrap(),
        "thin solid red"
    );
    for (limit, kind) in [
        (
            CssSerializationLimits::new(3, 4, 14),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(4, 3, 14),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(4, 4, 13),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limit).unwrap_err()),
            kind
        );
    }
    let reordered = CssParser::default().parse_block_contents("border:red solid thin");
    assert!(reordered.is_clean());
    let CssKnownPropertyValueRef::Border(wrapper) = reordered.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed border")
    };
    assert_eq!(wrapper.value(), &value);
    assert_eq!(wrapper.as_css(), "red solid thin");
    assert_ne!(
        value,
        CssBorder::try_new(Some(CssLineWidth::Thin), Some(CssLineStyle::Solid), None).unwrap()
    );
}

#[test]
fn pure_length_calculation_projects_without_a_percentage_basis_or_tree_reinterpretation() {
    let report =
        CssParser::default().parse_block_contents("border-top-width:calc(10% / 10% * 1px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderTopWidth(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed width")
    };
    let CssLineWidth::Length(current) = wrapper.value() else {
        panic!("exact math")
    };
    let original = current.calculation().unwrap();
    let calculation = original;
    assert!(matches!(calculation.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(calculation.numeric_type().percent_hint(), None);
    assert_eq!(
        calculation
            .numeric_type()
            .exponent(CssNumericDimension::Percentage),
        0
    );
    assert_eq!(
        calculation
            .numeric_type()
            .exponent(CssNumericDimension::Length),
        1
    );
}

#[test]
fn physical_widths_retain_extreme_literals() {
    for text in [
        "1px",
        "16777216px",
        "-0px",
        "1e100px",
        "1e-100px",
        "16777216.00000000001px",
    ] {
        let report = CssParser::default().parse_block_contents(format!("border-top-width:{text}"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderTopWidth(width) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("physical width")
        };
        assert_eq!(width.value(), &parsed_width(text));
    }
    let logical = CssParser::default().parse_block_contents("border-width:logical thin medium");
    assert!(logical.is_clean());
    let CssKnownPropertyValueRef::BorderWidth(value) = logical.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("logical widths")
    };
    assert_eq!(value.value().kind(), CssBoxSideKind::Logical);
}

#[test]
fn symbolic_border_triple_shares_one_numeric_and_color_budget() {
    // Numeric specified serialization orders unlike units canonically; the
    // same order is asserted by the Overflow 3 checked length consumer.
    let value = CssBorder::try_new(
        Some(parsed_width("calc(1px + 2em)")),
        Some(CssLineStyle::Solid),
        Some(red()),
    )
    .unwrap();
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "calc(2em + 1px) solid red"
    );
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(7, 8, 25))
            .unwrap(),
        "calc(2em + 1px) solid red"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(6, 8, 25),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(7, 7, 25),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(7, 8, 24),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}
