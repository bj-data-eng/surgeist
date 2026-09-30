use surgeist_css::*;

#[test]
fn checked_numeric_constructors_reject_non_finite_values_and_preserve_finite_boundaries() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(CssFiniteNumber::try_new(value), None);
        assert_eq!(CssNonNegativeNumber::try_new(value), None);
        assert_eq!(CssRatio::try_new(value, 1.0), None);
        assert_eq!(CssRatio::try_new(1.0, value), None);
        assert_eq!(CssKeyframePercent::try_new(value), None);

        assert_eq!(CssResolution::try_new(value, CssResolutionUnit::Dppx), None);
        assert_eq!(
            CssDurationLiteral::try_new(value, CssTimeUnit::Seconds),
            None
        );
        assert_eq!(CssDelayLiteral::try_new(value, CssTimeUnit::Seconds), None);
        assert_eq!(CssAnimationIterationNumber::try_new(value), None);
    }

    for invalid in ["NaN", "infinity", "-infinity"] {
        assert!(CssComponentValue::try_dimension(invalid, "rem").is_err());
        assert!(CssComponentValue::try_token(&format!("{invalid}%")).is_err());
    }

    for invalid in ["NaN", "infinity", "-infinity"] {
        let component = CssComponentValue::try_ident(invalid).unwrap();
        assert!(CssSpecifiedNonNegativeNumber::try_from_component(component.clone()).is_err());
        assert!(CssRatioOperand::try_from_component(component).is_err());
    }

    for invalid in ["-90.1deg", "90.1deg", "0"] {
        assert!(
            CssFontObliqueAngle::try_from_component(CssComponentValue::try_token(invalid).unwrap())
                .is_err()
        );
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
    let max_spelling = f32::MAX.to_string();
    let adjust = CssFontSizeAdjust::Number(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number(&max_spelling).unwrap(),
        )
        .unwrap(),
    );
    assert!(matches!(
        adjust,
        CssFontSizeAdjust::Number(value) if value.serialize_specified().unwrap() == max_spelling
    ));
    assert_eq!(
        surgeist_css::CssOpacityScalar::try_from_component(
            surgeist_css::CssComponentValue::try_number("1").unwrap()
        )
        .unwrap()
        .numeric()
        .representation(),
        "1"
    );
    let factor = CssSpecifiedNonNegativeNumber::try_from_component(
        CssComponentValue::try_number("340282346638528859811704183484516925440").unwrap(),
    )
    .unwrap();
    assert_eq!(
        factor.serialize_specified().unwrap(),
        "340282346638528859811704183484516925440"
    );
    for invalid in ["-1", "-0.00000000000000001"] {
        assert!(
            CssSpecifiedNonNegativeNumber::try_from_component(
                CssComponentValue::try_number(invalid).unwrap()
            )
            .is_err()
        );
    }
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
    assert_eq!(
        CssFontWeightNumber::try_from_component(CssComponentValue::try_number("1").unwrap())
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "1"
    );
    assert_eq!(
        CssFontWeightNumber::try_from_component(CssComponentValue::try_number("1000").unwrap())
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "1000"
    );
    assert_eq!(
        CssPositiveIntegerLiteral::try_new(CssIntegerLiteral::from_i32(1))
            .unwrap()
            .integer()
            .numeric()
            .representation(),
        "1"
    );
    assert_eq!(
        CssDurationLiteral::try_new(0.0, CssTimeUnit::Seconds)
            .unwrap()
            .value(),
        0.0
    );
    assert_eq!(
        CssAnimationIterationNumber::try_new(f32::MAX)
            .unwrap()
            .value(),
        f32::MAX
    );

    let tolerance = CssFlowTolerance::length_percentage(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token("25%").unwrap(),
        )
        .unwrap(),
    );
    assert!(matches!(
        tolerance.as_ref(),
        CssFlowToleranceRef::LengthPercentage(value) if exact_percentage(value.literal_component(),"25")
    ));

    let report = parse_style_attribute("flow-tolerance: 25%");
    assert!(report.is_clean());
    let property = report.syntax()[0]
        .known()
        .and_then(|known| known.property_value())
        .expect("parser-produced flow-tolerance value");
    let CssKnownPropertyValueRef::FlowTolerance(value) = property else {
        panic!("expected flow-tolerance wrapper");
    };
    assert_eq!(value.as_css(), "25%");
    assert!(matches!(
        value.value().as_ref(),
        CssFlowToleranceRef::LengthPercentage(value) if exact_percentage(value.literal_component(),"25")
    ));
    assert_eq!(value.value(), &tolerance);
}

#[test]
fn opacity_rejects_bare_non_finite_keywords_and_unrelated_numeric_domains() {
    for (value, responsible, token_kind) in [
        ("infinity", "infinity", CssTokenKind::Ident),
        ("NaN", "NaN", CssTokenKind::Ident),
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
fn opacity_preserves_finite_decimal_exponents_beyond_float_storage() {
    // A decimal exponent is mathematically finite. The tokenizer's infinity
    // cache does not change its authored numeric domain.
    for value in ["1e999", "1e999%"] {
        let report = parse_style_attribute(&format!("opacity: {value}; color: red"));
        assert!(report.is_clean());
        assert_eq!(report.syntax().len(), 2);
        let CssKnownPropertyValueRef::Opacity(opacity) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("ordinary opacity")
        };
        let CssOpacityValue::Scalar(scalar) = opacity.value() else {
            panic!("exact finite decimal")
        };
        assert_eq!(scalar.numeric().representation(), "1e999");
        assert_eq!(
            scalar.kind(),
            if value.ends_with('%') {
                CssOpacityScalarKind::Percentage
            } else {
                CssOpacityScalarKind::Number
            }
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
}

#[test]
fn calculations_preserve_finite_decimal_spellings_beyond_float_storage() {
    for source in [
        "opacity: calc(1e999); color: red",
        "opacity: calc(1e999%); color: red",
        "flow-tolerance: calc(3.5e38%); color: red",
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2);
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
fn flow_tolerance_nonzero_unitless_value_drops_declaration_and_retains_siblings() {
    let cases = [(
        "flow-tolerance",
        "3.5e38",
        CssKnownProperty::FlowTolerance,
        0,
    )];

    for (property, value, expected_property, responsible_offset) in cases {
        let invalid = format!("{property}: {value};");
        let source = format!("color: red; {invalid} width: 2px");
        let report = parse_style_attribute(&source);

        assert_eq!(report.syntax().len(), 2, "source: {source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("nonzero unitless length must produce one diagnostic for {source}");
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
                let encountered = detail.encountered().expect("nonzero unitless token");
                assert_eq!(encountered.kind(), CssTokenKind::Number);
                assert_eq!(encountered.authored(), "3.5e38");
            }
            _ => panic!("expected invalid property value"),
        }
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Width
        );
    }
}

#[test]
fn flow_tolerance_retains_exact_ordinary_percentages_beyond_float_storage() {
    for spelling in ["3.5e38", "1e999"] {
        let token = format!("{spelling}%");
        let source = format!("color: red; flow-tolerance: {token}; width: 2px");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 3);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[2].known().unwrap().property(),
            CssKnownProperty::Width
        );
        let CssKnownPropertyValueRef::FlowTolerance(wrapper) = report.syntax()[1]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("checked flow-tolerance")
        };
        let CssFlowToleranceRef::LengthPercentage(value) = wrapper.value().as_ref() else {
            panic!("exact ordinary percentage")
        };
        let component = value.literal_component().expect("literal, not calculation");
        let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = component.view()
        else {
            panic!("original percentage kind")
        };
        assert_eq!(number.representation(), spelling);
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("original parsed provenance")
        };
        assert_eq!(origin.source().as_str(), source);
        let token_start = source.find(&token).unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), token_start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            token_start + token.len()
        );
    }
}

fn exact_percentage(component: Option<&surgeist_css::CssComponentValue>, expected: &str) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation()==expected)
}
