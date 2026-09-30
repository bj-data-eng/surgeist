#![forbid(unsafe_code)]

use surgeist_css::*;

fn programmatic_size(number: &str, unit: &str) -> CssSizeValue {
    let literal = CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_dimension(number, unit).unwrap(),
    )
    .unwrap();
    assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
    CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(literal))
}

fn parsed(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn pair_construction_preserves_omission_and_width_height_order() {
    let width = programmatic_size("1.5", "px");
    let single = CssSizePair::new(width.clone(), None);
    assert_eq!(single.width(), single.height());
    assert!(single.authored_height().is_none());
    assert_eq!(single.serialize_specified().unwrap(), "1.5px");
    let explicit = CssSizePair::new(width.clone(), Some(width.clone()));
    assert_ne!(single, explicit);
    assert_eq!(explicit.serialize_specified().unwrap(), "1.5px 1.5px");
    let pair = CssSizePair::new(width, Some(CssSizeValue::Auto));
    assert_eq!(pair.serialize_specified().unwrap(), "1.5px auto");
    let source = parsed("size", "1.5px auto");
    let CssKnownPropertyValueRef::Size(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("size")
    };
    assert_eq!(value.value(), &pair);
    assert_eq!(value.as_css(), "1.5px auto");
    assert_eq!(
        parsed("min-size", "1.5px auto").known().unwrap().property(),
        CssKnownProperty::MinSize
    );

    let maximum = CssMaxSizePair::new(
        CssMaxSizeValue::NONE,
        Some(
            CssMaxSizeValue::try_box_size(CssBoxSize::LengthPercentage(
                CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                    parse_component_values("2.5%").unwrap().items()[0].clone(),
                )
                .unwrap(),
            ))
            .unwrap(),
        ),
    );
    assert_eq!(maximum.serialize_specified().unwrap(), "none 2.5%");
    let source = parsed("max-size", "none 2.5%");
    let CssKnownPropertyValueRef::MaxSize(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("max-size")
    };
    assert_eq!(value.value(), &maximum);
    assert_eq!(maximum.width(), &CssMaxSizeValue::NONE);
    assert!(maximum.authored_height().is_some());
}

#[test]
fn exact_and_symbolic_values_share_a_pair_budget() {
    let literal = programmatic_size("1e999", "px");
    let pair = CssSizePair::new(literal, Some(CssSizeValue::Auto));
    let text = pair.serialize_specified().unwrap();
    assert_eq!(text.len(), 1007); // 1 + 999 zeros + px + space + auto.
    assert!(text.starts_with('1') && text.ends_with("px auto"));
    let exactly = CssSpecifiedValueSerializationLimits::new(2, 2, text.len());
    assert_eq!(pair.serialize_specified_with_limits(exactly).unwrap(), text);
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1,
            2,
            text.len()
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            2,
            1,
            text.len()
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            2,
            2,
            text.len() - 1
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );

    let raw_math = CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values("calc(1px + 2%)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    ));
    let raw_pair = CssSizePair::new(raw_math, Some(CssSizeValue::Auto));
    assert_eq!(
        raw_pair.serialize_specified().unwrap(),
        "calc(2% + 1px) auto"
    );

    let calc = CssCalcSize::try_from_component(
        CssComponentValue::try_function(
            "calc-size",
            parse_component_values("min-content, size + 1px").unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let maximum =
        CssMaxSizeValue::try_box_size(CssBoxSize::CalcSize(calc.try_into().unwrap())).unwrap();
    let pair = CssMaxSizePair::new(CssMaxSizeValue::NONE, Some(maximum));
    let text = "none calc-size(min-content, 1px + size)";
    assert_eq!(pair.serialize_specified().unwrap(), text);
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            100,
            100,
            text.len()
        ))
        .unwrap(),
        text
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            100,
            100,
            text.len() - 1
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    // One `none`, one calc-size root, one basis keyword, and three numeric
    // expression nodes (sum, size, length) consume six input nodes. The
    // Projection allocates the size and length leaves, a combined scalar,
    // and their sum; with the keyword and `none`, that is six nodes.
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            6,
            6,
            text.len()
        ))
        .unwrap(),
        text
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            5,
            6,
            text.len()
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            6,
            5,
            text.len()
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    let source = parsed("max-size", "none calc-size(min-content, size + 1px)");
    let CssKnownPropertyValueRef::MaxSize(parsed_pair) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("max-size")
    };
    assert_eq!(parsed_pair.value(), &pair);
    let (CssBoxSize::CalcSize(direct), CssBoxSize::CalcSize(from_source)) = (
        pair.authored_height().unwrap().box_size().unwrap(),
        parsed_pair
            .value()
            .authored_height()
            .unwrap()
            .box_size()
            .unwrap(),
    ) else {
        panic!("two symbolic maximum sizes")
    };
    assert_ne!(
        direct.as_calc_size().origin(),
        from_source.as_calc_size().origin()
    );
}

#[test]
fn finite_controls_have_canonical_keyword_sequences_and_limits() {
    for (value, expected) in [
        (CssFrameSizing::Auto, "auto"),
        (CssFrameSizing::ContentWidth, "content-width"),
        (CssFrameSizing::ContentHeight, "content-height"),
        (CssFrameSizing::ContentBlockSize, "content-block-size"),
        (CssFrameSizing::ContentInlineSize, "content-inline-size"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }
    for (value, expected, nodes) in [
        (CssMinIntrinsicSizing::Legacy, "legacy", 1),
        (CssMinIntrinsicSizing::ZeroIfScroll, "zero-if-scroll", 1),
        (
            CssMinIntrinsicSizing::ZeroIfExtrinsic,
            "zero-if-extrinsic",
            1,
        ),
        (
            CssMinIntrinsicSizing::ZeroIfScrollAndExtrinsic,
            "zero-if-scroll zero-if-extrinsic",
            2,
        ),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    nodes,
                    nodes,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    nodes,
                    nodes,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }
    for authored in [
        "zero-if-scroll zero-if-extrinsic",
        "zero-if-extrinsic zero-if-scroll",
    ] {
        let source = parsed("min-intrinsic-sizing", authored);
        let CssKnownPropertyValueRef::MinIntrinsicSizing(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("min-intrinsic-sizing")
        };
        assert_eq!(
            *value.value(),
            CssMinIntrinsicSizing::ZeroIfScrollAndExtrinsic
        );
        assert_eq!(value.as_css(), authored);
        assert_eq!(
            value.value().serialize_specified().unwrap(),
            "zero-if-scroll zero-if-extrinsic"
        );
    }
}

#[test]
fn pair_equality_ignores_numeric_origins_but_retains_structure() {
    let direct = CssSizePair::new(programmatic_size("1", "px"), None);
    let source = parsed("size", "1px");
    let CssKnownPropertyValueRef::Size(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("size")
    };
    assert_eq!(value.value(), &direct);
    assert_ne!(
        direct,
        CssSizePair::new(programmatic_size("1.0", "px"), None)
    );
    assert_ne!(
        direct,
        CssSizePair::new(
            programmatic_size("1", "px"),
            Some(programmatic_size("1", "px"))
        )
    );
    assert_ne!(
        CssMaxSizePair::new(CssMaxSizeValue::NONE, None),
        CssMaxSizePair::new(CssMaxSizeValue::NONE, Some(CssMaxSizeValue::NONE))
    );
}
