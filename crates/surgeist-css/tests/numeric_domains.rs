use surgeist_css::{
    CssAnimationIterationCount, CssAnimationIterationNumber, CssAspectRatio, CssErrorCode,
    CssFiniteNumber, CssFlexFactor, CssFontFaceObliqueRange, CssFontFaceStretchValue,
    CssFontFaceWeightValue, CssFontSizeAdjust, CssFontWeightNumber, CssGridFlowTolerance,
    CssGridFlowToleranceValue, CssGridRepeatInteger, CssGridTrackBreadth, CssKeyframePercent,
    CssKnownProperty, CssKnownPropertyValueRef, CssLength, CssLengthDimension, CssLengthUnit,
    CssNonNegativeNumber, CssOpacity, CssRatio, CssRecoveryAction, CssResolution,
    CssResolutionUnit, CssRule, CssScaleValues, CssTime, CssTimeUnit, CssTokenKind, ErrorKind,
    parse_sheet, parse_style_attribute,
};

#[test]
fn checked_numeric_constructors_reject_non_finite_values_and_preserve_finite_boundaries() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(CssFiniteNumber::try_new(value), None);
        assert_eq!(CssNonNegativeNumber::try_new(value), None);
        assert_eq!(CssOpacity::try_new(value), None);
        assert_eq!(CssFlexFactor::try_new(value), None);
        assert_eq!(CssAspectRatio::try_new(value), None);
        assert_eq!(CssRatio::try_new(value, 1.0), None);
        assert_eq!(CssRatio::try_new(1.0, value), None);
        assert_eq!(CssKeyframePercent::try_new(value), None);
        assert_eq!(CssLength::try_px(value), None);
        assert_eq!(CssLength::try_percent(value), None);
        assert_eq!(CssLengthDimension::try_new(value, CssLengthUnit::Rem), None);
        assert_eq!(CssGridTrackBreadth::try_fraction(value), None);
        assert_eq!(CssScaleValues::try_new(vec![value]), None);
        assert_eq!(CssFontFaceWeightValue::try_new(value), None);
        assert_eq!(CssFontFaceObliqueRange::try_new(value, None), None);
        assert_eq!(CssFontFaceStretchValue::try_new_percent(value), None);
        assert_eq!(CssResolution::try_new(value, CssResolutionUnit::Dppx), None);
        assert_eq!(CssTime::try_new(value, CssTimeUnit::Seconds), None);
        assert_eq!(CssAnimationIterationNumber::try_new(value), None);
        assert_eq!(CssAnimationIterationCount::try_number(value), None);
    }

    assert_eq!(
        CssFiniteNumber::try_new(f32::MIN).unwrap().value(),
        f32::MIN
    );
    assert_eq!(
        CssFiniteNumber::try_new(f32::MAX).unwrap().value(),
        f32::MAX
    );
    assert_eq!(CssNonNegativeNumber::try_new(-0.0).unwrap().value(), -0.0);
    assert!(matches!(
        CssFontSizeAdjust::Number(CssNonNegativeNumber::try_new(f32::MAX).unwrap()),
        CssFontSizeAdjust::Number(value) if value.value() == f32::MAX
    ));
    assert_eq!(CssOpacity::try_new(1.0).unwrap().value(), 1.0);
    assert_eq!(CssFlexFactor::try_new(f32::MAX).unwrap().value(), f32::MAX);
    assert_eq!(
        CssRatio::try_new(0.0, f32::MAX)
            .unwrap()
            .numerator()
            .value(),
        0.0
    );
    assert_eq!(
        CssKeyframePercent::try_new(100.0).unwrap().value().value(),
        100.0
    );
    assert_eq!(CssFontWeightNumber::try_new(1).unwrap().value(), 1);
    assert_eq!(CssFontWeightNumber::try_new(1000).unwrap().value(), 1000);
    assert_eq!(CssGridRepeatInteger::try_new(1).unwrap().value(), 1);
    assert_eq!(CssTime::try_seconds(0.0).unwrap().value(), 0.0);
    assert_eq!(
        CssAnimationIterationNumber::try_new(f32::MAX)
            .unwrap()
            .value(),
        f32::MAX
    );

    let grid_tolerance = CssGridFlowTolerance::Percent(25.0);
    assert!(matches!(
        &grid_tolerance,
        CssGridFlowTolerance::Percent(value) if *value == 25.0
    ));
    assert_eq!(format!("{grid_tolerance:?}"), "Percent(25.0)");

    let report = parse_style_attribute("grid-flow-tolerance: 25%");
    assert!(report.is_clean());
    let property = report.syntax()[0]
        .known()
        .and_then(|known| known.property_value())
        .expect("parser-produced grid-flow-tolerance value");
    let CssKnownPropertyValueRef::GridFlowTolerance(value) = property else {
        panic!("expected grid-flow-tolerance wrapper");
    };
    assert_eq!(value.as_css(), "25%");
    assert!(matches!(
        value.value(),
        CssGridFlowToleranceValue::Percent(percent) if percent.value() == 25.0
    ));
    assert!(matches!(
        value.i01_subset(),
        Some(CssGridFlowTolerance::Percent(percent)) if *percent == 25.0
    ));
}

#[test]
fn opacity_rejects_non_finite_and_unrelated_numeric_domains() {
    for (value, responsible, token_kind) in [
        ("1e999", "1e999", CssTokenKind::Number),
        ("1e999%", "1e999%", CssTokenKind::Percentage),
        ("calc(1e999)", "1e999", CssTokenKind::Number),
        ("calc(1e999%)", "1e999%", CssTokenKind::Percentage),
        ("1px", "1px", CssTokenKind::Dimension),
    ] {
        let source = format!("opacity: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained sibling")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: invalid opacity must recover once");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected opacity property-value detail");
        };
        assert_eq!(detail.property(), CssKnownProperty::Opacity, "{source}");
        let encountered = detail.encountered().expect("responsible numeric token");
        assert_eq!(encountered.kind(), token_kind, "{source}");
        assert_eq!(encountered.authored(), responsible, "{source}");
    }
}

#[test]
fn non_finite_time_parse_drops_only_its_declaration_with_exact_diagnostic() {
    let invalid = "transition-duration: 1e999s;";
    let source = format!("color: red; {invalid} width: 2px");
    let report = parse_style_attribute(&source);

    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("non-finite time must produce one diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("1e999s").unwrap()
    );
    assert_eq!(diagnostic.error().position().line().value(), 0);
    assert_eq!(
        diagnostic.error().position().column().value(),
        source.find("1e999s").unwrap() as u32
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find(invalid).unwrap()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find(invalid).unwrap() + invalid.len()
    );
    match diagnostic.error().kind() {
        ErrorKind::InvalidPropertyValue(detail) => {
            assert_eq!(detail.property(), CssKnownProperty::TransitionDuration);
            let encountered = detail.encountered().expect("non-finite dimension token");
            assert_eq!(encountered.kind(), CssTokenKind::Dimension);
            assert_eq!(encountered.authored(), "1e999s");
        }
        _ => panic!("expected invalid property value"),
    }
}

#[test]
fn non_finite_iteration_parse_retains_sheet_siblings_with_exact_diagnostic() {
    let invalid = "animation-iteration-count: 1e999;";
    let source = format!(
        ".before {{ width: 1px; }}\n.bad {{ {invalid} opacity: .5; }}\n.after {{ height: 2px; }}"
    );
    let report = parse_sheet(&source);

    assert_eq!(report.syntax().rules().len(), 3);
    let CssRule::Style(bad) = &report.syntax().rules()[1] else {
        panic!("middle sibling must remain a style rule");
    };
    assert_eq!(bad.declarations().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("non-finite iteration count must produce one diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let value_start = source.find("1e999").unwrap();
    let declaration_start = source.find(invalid).unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        value_start
    );
    assert_eq!(diagnostic.error().position().line().value(), 1);
    assert_eq!(
        diagnostic.error().position().column().value(),
        (value_start - source.find("\n").unwrap() - 1) as u32
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        declaration_start
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_start + invalid.len()
    );
    match diagnostic.error().kind() {
        ErrorKind::InvalidPropertyValue(detail) => {
            assert_eq!(detail.property(), CssKnownProperty::AnimationIterationCount);
            let encountered = detail.encountered().expect("non-finite number token");
            assert_eq!(encountered.kind(), CssTokenKind::Number);
            assert_eq!(encountered.authored(), "1e999");
        }
        _ => panic!("expected invalid property value"),
    }
}

#[test]
fn percentage_conversion_overflow_drops_each_declaration_and_retains_siblings() {
    let cases = [
        (
            "grid-flow-tolerance",
            "3.5e38%",
            CssKnownProperty::GridFlowTolerance,
            0,
        ),
        (
            "grid-flow-tolerance",
            "calc(3.5e38%)",
            CssKnownProperty::GridFlowTolerance,
            "calc(".len(),
        ),
        (
            "grid-template-columns",
            "3.5e38%",
            CssKnownProperty::GridTemplateColumns,
            0,
        ),
    ];

    for (property, value, expected_property, responsible_offset) in cases {
        let invalid = format!("{property}: {value};");
        let source = format!("color: red; {invalid} width: 2px");
        let report = parse_style_attribute(&source);

        assert_eq!(report.syntax().len(), 2, "source: {source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("percentage overflow must produce one diagnostic for {source}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let value_start = source.find(value).unwrap();
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            value_start + responsible_offset
        );
        assert_eq!(diagnostic.error().position().line().value(), 0);
        assert_eq!(
            diagnostic.error().position().column().value(),
            (value_start + responsible_offset) as u32
        );
        let declaration_start = source.find(&invalid).unwrap();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            declaration_start
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            declaration_start + invalid.len()
        );
        match diagnostic.error().kind() {
            ErrorKind::InvalidPropertyValue(detail) => {
                assert_eq!(detail.property(), expected_property);
                let encountered = detail.encountered().expect("overflowing percentage token");
                assert_eq!(encountered.kind(), CssTokenKind::Percentage);
                assert_eq!(encountered.authored(), "3.5e38%");
            }
            _ => panic!("expected invalid property value"),
        }
    }
}
