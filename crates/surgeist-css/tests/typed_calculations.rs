use surgeist_css::{
    CssAngleCalculation, CssAngleUnit, CssAspectRatioValue, CssAuthoredColorComponent,
    CssAuthoredHue, CssCalcLength, CssCalculationExpressionRef, CssCalculationProductOperator,
    CssCalculationType, CssCalculationValueRef, CssErrorCode, CssFilterAmount,
    CssFilterFunctionValue, CssFilterNumber, CssFilterPercentage, CssFilterValue, CssFlexValue,
    CssFontSize, CssFrequencyCalculation, CssFrequencyUnit, CssGridFlowToleranceValue,
    CssIntegerCalculation, CssIntegerValue, CssKnownPropertyValueRef, CssLength,
    CssLengthCalculation, CssLengthUnit, CssLineHeight, CssNonNegativeNumberValue,
    CssNumberCalculation, CssOpacityValue, CssPercentageCalculation, CssPositiveNumber,
    CssPositiveNumberValue, CssRecoveryAction, CssRelativeColorChannel,
    CssRelativeColorExpressionValue, CssRelativeColorResultDomain, CssTimeCalculation, CssTimeUnit,
    CssZIndexValue, parse_style_attribute,
};

#[test]
fn core_font_calculations_preserve_number_and_length_percentage_domains() {
    let report = parse_style_attribute(concat!(
        "font-size: calc((1em + 10%) * 2); ",
        "line-height: calc(1 + 0.5); ",
        "line-height: calc((1em + 10%) * 2)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::FontSize(size) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-size");
    };
    assert!(matches!(
        size.size(),
        CssFontSize::LengthPercentage(value)
            if matches!(
                value.value(),
                CssLength::Calc(CssCalcLength::Typed(calculation))
                    if calculation.result_type() == CssCalculationType::LengthPercentage
            )
    ));

    let CssKnownPropertyValueRef::LineHeight(number) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected line-height number");
    };
    assert!(matches!(
        number.line_height(),
        CssLineHeight::Number(CssNonNegativeNumberValue::Calculation(calculation))
            if calculation.result_type() == CssCalculationType::Number
    ));

    let CssKnownPropertyValueRef::LineHeight(length) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected line-height length-percentage");
    };
    assert!(matches!(
        length.line_height(),
        CssLineHeight::LengthPercentage(value)
            if matches!(
                value.value(),
                CssLength::Calc(CssCalcLength::Typed(calculation))
                    if calculation.result_type() == CssCalculationType::LengthPercentage
            )
    ));
}

#[test]
fn relative_color_calculations_retain_typed_domains_and_closed_channel_references() {
    let report = parse_style_attribute(concat!(
        "color: rgb(from red calc(r + 1) calc(g * 2) calc(b / 2) / calc(alpha * 0.5)); ",
        "color: hsl(from red calc(h + 20deg) calc(s + 10%) l); ",
        "color: oklch(from red l c calc(h + 0.25turn))",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let relatives: Vec<_> = report
        .syntax()
        .iter()
        .map(|declaration| {
            let CssKnownPropertyValueRef::Color(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("expected color wrapper");
            };
            value.current().relative_value().unwrap()
        })
        .collect();

    let rgb = relatives[0];
    assert_eq!(
        rgb.channels()[0].result_domain(),
        CssRelativeColorResultDomain::NumberPercentage
    );
    let CssRelativeColorExpressionValue::Calculation(red) = rgb.channels()[0].value() else {
        panic!("expected typed red calculation");
    };
    assert_eq!(red.authored().as_css(), "calc(r + 1)");
    assert_eq!(red.result_type(), CssCalculationType::Number);
    assert_eq!(red.references(), &[CssRelativeColorChannel::R]);
    let CssRelativeColorExpressionValue::Calculation(alpha) = rgb.alpha().unwrap().value() else {
        panic!("expected typed alpha calculation");
    };
    assert_eq!(alpha.references(), &[CssRelativeColorChannel::Alpha]);
    assert_eq!(
        rgb.alpha().unwrap().result_domain(),
        CssRelativeColorResultDomain::Alpha
    );

    for (relative, hue_index) in [(relatives[1], 0), (relatives[2], 2)] {
        let hue = &relative.channels()[hue_index];
        assert_eq!(hue.result_domain(), CssRelativeColorResultDomain::Hue);
        let CssRelativeColorExpressionValue::Calculation(calculation) = hue.value() else {
            panic!("expected typed hue calculation");
        };
        assert_eq!(calculation.result_type(), CssCalculationType::Angle);
        assert_eq!(calculation.references(), &[CssRelativeColorChannel::H]);
    }
}

#[test]
fn perceptual_color_channels_preserve_typed_calculation_domains() {
    let report = parse_style_attribute(concat!(
        "color: lab(calc(40% + 10%) calc(1 + 2) calc(20% - 5%)); ",
        "color: lch(calc(50 + 10) calc(30% + 5%) calc(1turn - 90deg)); ",
        "color: color(rec2020 calc(1 + 2) calc(10% + 20%) none)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let mut colors = report.syntax().iter().map(|declaration| {
        let CssKnownPropertyValueRef::Color(value) = declaration
            .known()
            .expect("known color")
            .property_value()
            .expect("ordinary color")
        else {
            panic!("expected color wrapper");
        };
        value.current()
    });

    let lab = colors.next().unwrap().lab_value().unwrap();
    assert!(matches!(
        lab.lightness(),
        CssAuthoredColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(
        lab.a(),
        CssAuthoredColorComponent::NumberCalculation(_)
    ));
    assert!(matches!(
        lab.b(),
        CssAuthoredColorComponent::PercentageCalculation(_)
    ));

    let lch = colors.next().unwrap().lch_value().unwrap();
    assert!(matches!(
        lch.lightness(),
        CssAuthoredColorComponent::NumberCalculation(_)
    ));
    assert!(matches!(
        lch.chroma(),
        CssAuthoredColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(lch.hue(), CssAuthoredHue::AngleCalculation(_)));

    let predefined = colors.next().unwrap().predefined_value().unwrap();
    assert!(matches!(
        predefined.channels(),
        [
            CssAuthoredColorComponent::NumberCalculation(_),
            CssAuthoredColorComponent::PercentageCalculation(_),
            CssAuthoredColorComponent::None,
        ]
    ));
}

#[test]
fn number_calculation_literal_preserves_finite_authored_value() {
    let calculation =
        CssNumberCalculation::try_literal(-3.5).expect("finite authored number calculation");

    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    match calculation.expression() {
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Number(value)) => {
            assert_eq!(value.value(), -3.5);
        }
        _ => panic!("expected an authored number leaf"),
    }
}

#[test]
fn property_consumers_accept_typed_products_and_groups_with_later_siblings() {
    for value in ["calc(1px * 2)", "calc((1px + 2px))"] {
        let source = format!("width: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2);
        let CssKnownPropertyValueRef::Width(width) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected width wrapper");
        };
        assert!(matches!(
            width.i01_subset(),
            Some(CssLength::Calc(CssCalcLength::Typed(_)))
        ));
    }
}

#[test]
fn typed_calculation_roots_enforce_checked_literal_boundaries() {
    let integer_min = CssIntegerCalculation::literal(i32::MIN);
    let integer_max = CssIntegerCalculation::literal(i32::MAX);
    assert_eq!(integer_min.result_type(), CssCalculationType::Integer);
    assert_eq!(integer_max.result_type(), CssCalculationType::Integer);
    assert!(matches!(
        integer_min.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(i32::MIN))
    ));
    assert!(matches!(
        integer_max.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(i32::MAX))
    ));

    for value in [f32::MIN, -0.0, f32::MAX] {
        let number = CssNumberCalculation::try_literal(value).expect("finite number leaf");
        assert_eq!(number.result_type(), CssCalculationType::Number);
        assert!(matches!(
            number.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Number(inner))
                if inner.value() == value
        ));
    }

    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssNumberCalculation::try_literal(value).is_none());
        assert!(CssPercentageCalculation::try_literal(value).is_none());
        assert!(CssLengthCalculation::try_dimension(value, CssLengthUnit::Rem).is_none());
        assert!(CssLengthCalculation::try_percentage(value).is_none());
        assert!(CssAngleCalculation::try_literal(value, CssAngleUnit::Degrees).is_none());
        assert!(CssTimeCalculation::try_literal(value, CssTimeUnit::Seconds).is_none());
        assert!(CssFrequencyCalculation::try_literal(value, CssFrequencyUnit::Hertz).is_none());
    }

    for value in [f32::MIN, -0.0, f32::MAX] {
        let percentage =
            CssPercentageCalculation::try_literal(value).expect("finite percentage leaf");
        assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
        assert!(matches!(
            percentage.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(inner))
                if inner.value() == value
        ));

        let length = CssLengthCalculation::try_dimension(value, CssLengthUnit::Cqw)
            .expect("finite length leaf");
        assert_eq!(length.result_type(), CssCalculationType::Length);
        assert!(matches!(
            length.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(inner))
                if inner.value() == value && inner.unit() == CssLengthUnit::Cqw
        ));

        let angle = CssAngleCalculation::try_literal(value, CssAngleUnit::Turns)
            .expect("finite angle leaf");
        assert_eq!(angle.result_type(), CssCalculationType::Angle);
        assert!(matches!(
            angle.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(inner))
                if inner.value() == value && inner.unit() == CssAngleUnit::Turns
        ));

        let time = CssTimeCalculation::try_literal(value, CssTimeUnit::Milliseconds)
            .expect("finite signed time leaf");
        assert_eq!(time.result_type(), CssCalculationType::Time);
        assert!(matches!(
            time.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Time(inner))
                if inner.value() == value && inner.unit() == CssTimeUnit::Milliseconds
        ));

        let frequency = CssFrequencyCalculation::try_literal(value, CssFrequencyUnit::Kilohertz)
            .expect("finite frequency leaf");
        assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
        assert!(matches!(
            frequency.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Frequency(inner))
                if inner.value() == value && inner.unit() == CssFrequencyUnit::Kilohertz
        ));
    }

    let percentage_length =
        CssLengthCalculation::try_percentage(-25.0).expect("finite signed percentage");
    assert_eq!(
        percentage_length.result_type(),
        CssCalculationType::Percentage
    );
    assert!(matches!(
        percentage_length.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(value))
            if value.value() == -25.0
    ));
}

#[test]
fn existing_calc_consumer_preserves_exact_depth_boundary_and_later_sibling() {
    for depth in [255_usize, 256] {
        let source = format!(
            "width: {}1px{}; color: red",
            "calc(".repeat(depth),
            ")".repeat(depth)
        );
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2);
    }

    let depth = 257_usize;
    let source = format!(
        "width: {}1px{}; color: red",
        "calc(".repeat(depth),
        ")".repeat(depth)
    );
    let first_over_limit = source
        .match_indices("calc(")
        .nth(256)
        .expect("257th authored calculation")
        .0;
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("over-limit calculation must produce one diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        first_over_limit
    );
}

#[test]
fn opacity_percentage_calculation_preserves_exact_depth_boundary() {
    for depth in [255_usize, 256] {
        let source = format!(
            "opacity: {}1%{}; color: red",
            "calc(".repeat(depth),
            ")".repeat(depth)
        );
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2, "depth {depth}");
        let CssKnownPropertyValueRef::Opacity(opacity) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected opacity at depth {depth}");
        };
        assert!(matches!(
            opacity.value(),
            CssOpacityValue::PercentageCalculation(_)
        ));
    }

    let depth = 257_usize;
    let source = format!(
        "opacity: {}1%{}; color: red",
        "calc(".repeat(depth),
        ")".repeat(depth)
    );
    let first_over_limit = source
        .match_indices("calc(")
        .nth(256)
        .expect("257th authored calculation")
        .0;
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("over-limit opacity calculation must produce one diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        first_over_limit
    );
}

#[test]
fn typed_length_consumer_exposes_products_and_preserves_simple_sum_compatibility() {
    let report =
        parse_style_attribute("width: calc(1px + 2%); height: calc((1px + 2%) * 3); color: red");
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let width = report.syntax()[0].known().expect("known width");
    let CssKnownPropertyValueRef::Width(width) = width.property_value().unwrap() else {
        panic!("expected width wrapper");
    };
    let CssLength::Calc(CssCalcLength::Sum(terms)) = width.i01_subset().unwrap() else {
        panic!("the frozen simple sum must keep its exact I01 projection");
    };
    assert_eq!(terms.len(), 2);

    let height = report.syntax()[1].known().expect("known height");
    let CssKnownPropertyValueRef::Height(height) = height.property_value().unwrap() else {
        panic!("expected height wrapper");
    };
    let CssLength::Calc(calc) = height.i01_subset().unwrap() else {
        panic!("expected calculated height");
    };
    assert!(calc.uses_percentage());
    assert_eq!(calc.to_css_string(), "calc((1px + 2%) * 3)");
    let CssCalcLength::Typed(calculation) = calc else {
        panic!("new length syntax must use the additive typed compatibility branch");
    };
    assert_eq!(
        calculation.result_type(),
        CssCalculationType::LengthPercentage
    );
    let CssCalculationExpressionRef::Product(product) = calculation.expression() else {
        panic!("expected typed length product");
    };
    assert_eq!(product.len(), 2);
    assert_eq!(
        product.factor(1).unwrap().operator(),
        Some(CssCalculationProductOperator::Multiply)
    );
    assert!(matches!(
        product.factor(0).unwrap().expression(),
        CssCalculationExpressionRef::Group(_)
    ));
}

#[test]
fn scalar_property_accessors_distinguish_literals_from_deferred_calculations() {
    let source = concat!(
        "opacity: calc(-1 * 2); ",
        "flex-grow: calc(-1 + 2); ",
        "flex-shrink: calc((3 / 2)); ",
        "order: calc(2 * 3); ",
        "z-index: calc((4 + 1)); ",
        "aspect-ratio: calc(-1 * 2); ",
        "flex: calc(2 * 3) calc(-1 + 2) calc((10px * 2)); ",
        "grid-flow-tolerance: calc((5% + 1px) * 2)"
    );
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Opacity(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected opacity wrapper");
    };
    assert!(matches!(value.value(), CssOpacityValue::Calculation(_)));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::FlexGrow(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-grow wrapper");
    };
    assert!(matches!(
        value.factor(),
        CssNonNegativeNumberValue::Calculation(_)
    ));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::FlexShrink(value) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-shrink wrapper");
    };
    assert!(matches!(
        value.factor(),
        CssNonNegativeNumberValue::Calculation(_)
    ));

    let CssKnownPropertyValueRef::Order(value) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected order wrapper");
    };
    assert!(matches!(value.value(), CssIntegerValue::Calculation(_)));

    let CssKnownPropertyValueRef::ZIndex(value) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected z-index wrapper");
    };
    assert!(matches!(
        value.value(),
        CssZIndexValue::Integer(CssIntegerValue::Calculation(_))
    ));

    let CssKnownPropertyValueRef::AspectRatio(value) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected aspect-ratio wrapper");
    };
    let CssAspectRatioValue::Calculation(calculation) = value.ratio() else {
        panic!("expected deferred aspect-ratio calculation");
    };
    assert!(matches!(
        calculation.expression(),
        CssCalculationExpressionRef::Product(_)
    ));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::Flex(value) = report.syntax()[6]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex wrapper");
    };
    let CssFlexValue::Components(components) = value.value() else {
        panic!("expected flex components");
    };
    assert!(matches!(
        components.grow(),
        CssNonNegativeNumberValue::Calculation(_)
    ));
    assert!(matches!(
        components.shrink(),
        Some(CssNonNegativeNumberValue::Calculation(_))
    ));
    assert!(matches!(
        components.basis(),
        Some(CssLength::Calc(CssCalcLength::Typed(_)))
    ));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::GridFlowTolerance(value) = report.syntax()[7]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid-flow-tolerance wrapper");
    };
    assert!(matches!(
        value.value(),
        CssGridFlowToleranceValue::Length(CssLength::Calc(CssCalcLength::Typed(_)))
    ));
    assert!(value.i01_subset().is_none());
}

#[test]
fn opacity_keeps_number_and_percentage_calculation_roots_symbolic() {
    let report = parse_style_attribute(
        "opacity: calc(-1.5 * 2); opacity: calc((25% + 25%) * 2); color: red",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 3);

    let CssKnownPropertyValueRef::Opacity(number) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected number-root opacity");
    };
    let CssOpacityValue::Calculation(number_calculation) = number.value() else {
        panic!("expected retained number calculation");
    };
    assert_eq!(number_calculation.result_type(), CssCalculationType::Number);
    assert!(matches!(
        number_calculation.expression(),
        CssCalculationExpressionRef::Product(_)
    ));
    assert!(number.i01_subset().is_none());

    let CssKnownPropertyValueRef::Opacity(percentage) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected percentage-root opacity");
    };
    let CssOpacityValue::PercentageCalculation(percentage_calculation) = percentage.value() else {
        panic!("expected retained percentage calculation");
    };
    assert_eq!(
        percentage_calculation.result_type(),
        CssCalculationType::Percentage
    );
    assert!(matches!(
        percentage_calculation.expression(),
        CssCalculationExpressionRef::Product(_)
    ));
    assert!(percentage.i01_subset().is_none());
}

#[test]
fn scalar_property_accessors_preserve_literal_compatibility_projections() {
    let report = parse_style_attribute(
        "opacity: 0.5; flex-grow: 2; flex-shrink: 0; order: -2; z-index: auto; \
         aspect-ratio: 1.5; flex: 2 0 10rem",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Opacity(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected opacity wrapper");
    };
    assert!(matches!(value.value(), CssOpacityValue::Literal(value) if value.value() == 0.5));
    assert_eq!(value.i01_subset().unwrap().value(), 0.5);

    let CssKnownPropertyValueRef::FlexGrow(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-grow wrapper");
    };
    assert!(
        matches!(value.factor(), CssNonNegativeNumberValue::Literal(value) if value.value() == 2.0)
    );
    assert_eq!(value.i01_subset().unwrap().value(), 2.0);

    let CssKnownPropertyValueRef::FlexShrink(value) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-shrink wrapper");
    };
    assert!(
        matches!(value.factor(), CssNonNegativeNumberValue::Literal(value) if value.value() == 0.0)
    );
    assert_eq!(value.i01_subset().unwrap().value(), 0.0);

    let CssKnownPropertyValueRef::Order(value) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected order wrapper");
    };
    assert!(matches!(value.value(), CssIntegerValue::Literal(-2)));
    assert!(matches!(
        value.i01_subset(),
        Some(surgeist_css::CssOrder::Integer(-2))
    ));

    let CssKnownPropertyValueRef::ZIndex(value) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected z-index wrapper");
    };
    assert!(matches!(value.value(), CssZIndexValue::Auto));
    assert!(matches!(
        value.i01_subset(),
        Some(surgeist_css::CssZIndex::Auto)
    ));

    let CssKnownPropertyValueRef::AspectRatio(value) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected aspect-ratio wrapper");
    };
    assert!(matches!(value.ratio(), CssAspectRatioValue::Literal(value) if value.value() == 1.5));
    assert_eq!(value.i01_subset().unwrap().value(), 1.5);

    let CssKnownPropertyValueRef::Flex(value) = report.syntax()[6]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex wrapper");
    };
    let CssFlexValue::Components(components) = value.value() else {
        panic!("expected literal flex components");
    };
    assert!(
        matches!(components.grow(), CssNonNegativeNumberValue::Literal(value) if value.value() == 2.0)
    );
    assert!(
        matches!(components.shrink(), Some(CssNonNegativeNumberValue::Literal(value)) if value.value() == 0.0)
    );
    assert!(value.i01_subset().is_some());
}

#[test]
fn positive_number_model_checks_literals_while_calculation_range_stays_authored() {
    assert!(CssPositiveNumber::try_new(0.0).is_none());
    assert!(CssPositiveNumber::try_new(-1.0).is_none());
    assert!(CssPositiveNumber::try_new(f32::INFINITY).is_none());
    let literal = CssPositiveNumber::try_new(0.25).expect("finite positive literal");
    assert_eq!(literal.value(), 0.25);
    assert!(matches!(
        CssPositiveNumberValue::Literal(literal),
        CssPositiveNumberValue::Literal(value) if value.value() == 0.25
    ));

    let calculation = CssNumberCalculation::try_literal(-2.0).expect("finite authored number");
    assert!(matches!(
        CssPositiveNumberValue::Calculation(calculation),
        CssPositiveNumberValue::Calculation(value)
            if matches!(
                value.expression(),
                CssCalculationExpressionRef::Value(CssCalculationValueRef::Number(number))
                    if number.value() == -2.0
            )
    ));

    let literal_report = parse_style_attribute("aspect-ratio: 0; color: red");
    assert_eq!(literal_report.syntax().len(), 1);
    assert_eq!(literal_report.diagnostics().len(), 1);
    let calculation_report = parse_style_attribute("aspect-ratio: calc(-1 * 2); color: red");
    assert!(calculation_report.is_clean());
    assert_eq!(calculation_report.syntax().len(), 2);
}

#[test]
fn filter_amount_calculations_keep_number_and_percentage_roots_symbolic() {
    let report = parse_style_attribute(
        "filter: brightness(calc(-1 + 2)) opacity(calc(25% + 25%)); color: red",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter wrapper");
    };
    let CssFilterValue::Functions(functions) = filter.current() else {
        panic!("expected filter functions");
    };
    assert!(matches!(
        functions.functions()[0],
        CssFilterFunctionValue::Brightness(CssFilterAmount::Number(CssFilterNumber::Calculation(
            _
        )))
    ));
    assert!(matches!(
        functions.functions()[1],
        CssFilterFunctionValue::Opacity(CssFilterAmount::Percentage(
            CssFilterPercentage::Calculation(_)
        ))
    ));
    assert!(filter.i01_subset().is_none());
}
