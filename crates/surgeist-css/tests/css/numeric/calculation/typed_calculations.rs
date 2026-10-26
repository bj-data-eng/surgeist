use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::NumericFailure as Kind;
use crate::css::support::failure_classes::numeric_failure;
use crate::css::support::math_depth::MathDepthBoundary;
use crate::css::support::value_assertions::assert_numeric_kind;
use surgeist_css::*;

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
            if value.calculation().is_some_and(|calculation|
                calculation.result_type() == CssCalculationType::LengthPercentage
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
        CssLineHeight::Number(value)
            if value.calculation().is_some_and(|calculation| calculation.result_type() == CssCalculationType::Number)
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
                value.calculation(),
                Some(calculation)
                    if calculation.result_type() == CssCalculationType::LengthPercentage
            )
    ));
}

#[test]
fn relative_color_calculations_retain_typed_domains_and_closed_channel_references() {
    let report = parse_style_attribute(concat!(
        "color: rgb(from red calc(r + 1) calc(g * 2) calc(b / 2) / calc(alpha * 0.5)); ",
        "color: hsl(from red calc(h + 20) calc(s + 10) l); ",
        "color: oklch(from red l c calc(h + 0.25))",
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
            value.value().relative_value().unwrap()
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
        assert_eq!(calculation.result_type(), CssCalculationType::Number);
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
        value.value()
    });

    let lab = colors.next().unwrap().lab_value().unwrap();
    assert!(matches!(
        lab.lightness(),
        CssColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(lab.a(), CssColorComponent::NumberCalculation(_)));
    assert!(matches!(
        lab.b(),
        CssColorComponent::PercentageCalculation(_)
    ));

    let lch = colors.next().unwrap().lch_value().unwrap();
    assert!(matches!(
        lch.lightness(),
        CssColorComponent::NumberCalculation(_)
    ));
    assert!(matches!(
        lch.chroma(),
        CssColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(lch.hue(), CssColorHue::AngleCalculation(_)));

    let predefined = colors.next().unwrap().predefined_value().unwrap();
    assert!(matches!(
        predefined.channels(),
        [
            CssColorComponent::NumberCalculation(_),
            CssColorComponent::PercentageCalculation(_),
            CssColorComponent::None,
        ]
    ));
}

#[test]
fn number_calculation_literal_preserves_finite_authored_value() {
    let calculation =
        CssCalculation::<CssNumber>::try_literal(-3.5).expect("finite authored number calculation");

    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    match calculation_body(calculation.expression()) {
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Number(value)) => {
            assert_eq!(value.representation(), "-3.5");
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
        assert!(matches!(width.value(), CssSizeValue::BoxSize(
            CssBoxSize::LengthPercentage(length)
        ) if length.calculation().is_some()));
    }
}

#[test]
fn typed_calculation_roots_enforce_checked_literal_boundaries() {
    let integer_min = CssCalculation::<CssInteger>::literal(i32::MIN);
    let integer_max = CssCalculation::<CssInteger>::literal(i32::MAX);
    assert_eq!(integer_min.result_type(), CssCalculationType::Number);
    assert_eq!(integer_max.result_type(), CssCalculationType::Number);
    assert!(matches!(
        integer_min.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(value)) if value.representation() == "-2147483648"
    ));
    assert!(matches!(
        integer_max.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(value)) if value.representation() == "2147483647"
    ));

    for value in [f32::MIN, -0.0, f32::MAX] {
        let number = CssCalculation::<CssNumber>::try_literal(value).expect("finite number leaf");
        assert_eq!(number.result_type(), CssCalculationType::Number);
        assert!(matches!(
            number.expression(),
            CssCalculationExpressionRef::Value(inner)
                if inner.literal().representation() == value.to_string()
        ));
    }

    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssCalculation::<CssNumber>::try_literal(value).is_none());
        assert!(CssCalculation::<CssPercentage>::try_literal(value).is_none());
        assert!(CssCalculation::<CssLength>::try_dimension(value, CssLengthUnit::Rem).is_none());
        assert!(CssCalculation::<CssLengthPercentage>::try_percentage(value).is_none());
        assert_numeric_kind(
            CssCalculation::<CssAngle>::try_literal(&value.to_string(), CssAngleUnit::Degrees),
            NumericFailure::Component(ComponentFailure::InvalidDimension),
        );
        assert_numeric_kind(
            CssCalculation::<CssTime>::try_literal(&value.to_string(), CssTimeUnit::Seconds),
            NumericFailure::Component(ComponentFailure::InvalidDimension),
        );
        assert_numeric_kind(
            CssCalculation::<CssFrequency>::try_literal(
                &value.to_string(),
                CssFrequencyUnit::Hertz,
            ),
            NumericFailure::Component(ComponentFailure::InvalidDimension),
        );
    }

    for value in [f32::MIN, -0.0, f32::MAX] {
        let percentage =
            CssCalculation::<CssPercentage>::try_literal(value).expect("finite percentage leaf");
        assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
        assert!(matches!(
            percentage.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(inner))
                if inner.representation() == value.to_string()
        ));

        let length = CssCalculation::<CssLength>::try_dimension(value, CssLengthUnit::Cqw)
            .expect("finite length leaf");
        assert_eq!(length.result_type(), CssCalculationType::Length);
        assert!(matches!(
            length.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(inner))
                if inner.representation() == value.to_string() && inner.unit() == Some("cqw")
        ));

        let angle =
            CssCalculation::<CssAngle>::try_literal(&value.to_string(), CssAngleUnit::Turns)
                .expect("finite angle leaf");
        assert_eq!(angle.result_type(), CssCalculationType::Angle);
        assert!(matches!(
            angle.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(inner))
                if inner.representation() == value.to_string() && inner.unit() == Some("turn")
        ));

        let time =
            CssCalculation::<CssTime>::try_literal(&value.to_string(), CssTimeUnit::Milliseconds)
                .expect("finite signed time leaf");
        assert_eq!(time.result_type(), CssCalculationType::Time);
        assert!(matches!(
            time.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Time(inner))
                if inner.representation() == value.to_string() && inner.unit() == Some("ms")
        ));

        let frequency = CssCalculation::<CssFrequency>::try_literal(
            &value.to_string(),
            CssFrequencyUnit::Kilohertz,
        )
        .expect("finite frequency leaf");
        assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
        assert!(matches!(
            frequency.expression(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Frequency(inner))
                if inner.representation() == value.to_string() && inner.unit() == Some("khz")
        ));
    }

    let percentage_length = CssCalculation::<CssLengthPercentage>::try_percentage(-25.0)
        .expect("finite signed percentage");
    assert_eq!(
        percentage_length.result_type(),
        CssCalculationType::LengthPercentage
    );
    assert!(matches!(
        percentage_length.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(value))
            if value.representation() == "-25"
    ));
}

#[test]
fn calc_consumer_preserves_exact_depth_boundary_and_later_sibling() {
    MathDepthBoundary {
        template: "width: {}1px{}; color: red",
        accepted: &[255, 256],
        rejected: &[257],
        sibling_retained: true,
    }
    .assert();
}

#[test]
fn opacity_percentage_calculation_preserves_exact_depth_boundary() {
    MathDepthBoundary {
        template: "opacity: {}1%{}; color: red",
        accepted: &[255, 256],
        rejected: &[257],
        sibling_retained: true,
    }
    .assert_with(
        |depth, _, report| {
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
        },
        |_, _, _| {},
    );
}

#[test]
fn typed_length_consumer_exposes_exact_products_and_sums() {
    let report =
        parse_style_attribute("width: calc(1px + 2%); height: calc((1px + 2%) * 3); color: red");
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let width = report.syntax()[0].known().expect("known width");
    let CssKnownPropertyValueRef::Width(width) = width.property_value().unwrap() else {
        panic!("expected width wrapper");
    };
    let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = width.value() else {
        panic!("the sum must use the exact numeric owner");
    };
    let calculation = length.calculation().expect("exact numeric owner");
    let CssCalculationExpressionRef::Sum(terms) = calculation_body(calculation.expression()) else {
        panic!("expected exact sum")
    };
    assert_eq!(terms.len(), 2);

    let height = report.syntax()[1].known().expect("known height");
    let CssKnownPropertyValueRef::Height(height) = height.property_value().unwrap() else {
        panic!("expected height wrapper");
    };
    let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length)) = height.value() else {
        panic!("expected calculated height");
    };
    assert_eq!(height.as_css(), "calc((1px + 2%) * 3)");
    // Distribution is derived from 3 × (1px + 2%), while the typed tree
    // below retains the authored product for later layout resolution.
    assert_eq!(
        length.serialize_specified(Default::default()).unwrap(),
        "calc(6% + 3px)"
    );
    let calculation = length.calculation().expect("exact typed calculation");
    assert_eq!(
        calculation.result_type(),
        CssCalculationType::LengthPercentage
    );
    let CssCalculationExpressionRef::Product(product) = calculation_body(calculation.expression())
    else {
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
        "flow-tolerance: calc((5% + 1px) * 2)"
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
    assert!(matches!(
        value.value(),
        CssOpacityValue::NumberCalculation(_)
    ));

    let CssKnownPropertyValueRef::FlexGrow(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-grow wrapper");
    };
    assert!(value.factor().calculation().is_some());

    let CssKnownPropertyValueRef::FlexShrink(value) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-shrink wrapper");
    };
    assert!(value.factor().calculation().is_some());

    let CssKnownPropertyValueRef::Order(value) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected order wrapper");
    };
    assert!(value.value().calculation().is_some());

    let CssKnownPropertyValueRef::ZIndex(value) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected z-index wrapper");
    };
    assert!(
        matches!(value.value(), CssZIndexValue::Integer(value) if value.calculation().is_some())
    );

    let CssKnownPropertyValueRef::AspectRatio(value) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected aspect-ratio wrapper");
    };
    let CssAspectRatioValue::Ratio(ratio) = value.ratio() else {
        panic!("expected deferred aspect-ratio ratio");
    };
    let calculation = ratio.numerator().calculation().expect("math numerator");
    assert!(matches!(
        calculation_body(calculation.expression()),
        CssCalculationExpressionRef::Product(_)
    ));

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
    assert!(components.grow().unwrap().calculation().is_some());
    assert!(components.shrink().unwrap().calculation().is_some());
    assert!(matches!(
        components.basis().map(|value| value.view()),
        Some(CssFlexBasisRef::Size(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(value))))
            if value.calculation().is_some()
    ));

    let CssKnownPropertyValueRef::FlowTolerance(value) = report.syntax()[7]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flow-tolerance wrapper");
    };
    assert!(matches!(
        value.value().as_ref(),
        CssFlowToleranceRef::LengthPercentage(length) if length.calculation().is_some()
    ));
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
    let CssOpacityValue::NumberCalculation(number_calculation) = number.value() else {
        panic!("expected retained number calculation");
    };
    assert_eq!(number_calculation.result_type(), CssCalculationType::Number);
    assert!(matches!(
        calculation_body(number_calculation.expression()),
        CssCalculationExpressionRef::Product(_)
    ));
    assert_eq!(number_calculation.result_type(), CssCalculationType::Number);

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
        calculation_body(percentage_calculation.expression()),
        CssCalculationExpressionRef::Product(_)
    ));
    assert_eq!(
        percentage_calculation.result_type(),
        CssCalculationType::Percentage
    );
}

#[test]
fn scalar_property_accessors_preserve_exact_literal_domains() {
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
    assert!(
        matches!(value.value(), CssOpacityValue::Scalar(value) if value.numeric().representation() == "0.5")
    );
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "0.5"
    );

    let CssKnownPropertyValueRef::FlexGrow(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-grow wrapper");
    };
    assert_eq!(
        value
            .factor()
            .serialize_specified(Default::default())
            .unwrap(),
        "2"
    );

    let CssKnownPropertyValueRef::FlexShrink(value) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected flex-shrink wrapper");
    };
    assert_eq!(
        value
            .factor()
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );

    let CssKnownPropertyValueRef::Order(value) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected order wrapper");
    };
    assert!(
        value
            .value()
            .literal()
            .is_some_and(|value| value.numeric().representation() == "-2")
    );
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "-2"
    );

    let CssKnownPropertyValueRef::ZIndex(value) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected z-index wrapper");
    };
    assert!(matches!(value.value(), CssZIndexValue::Auto));

    let CssKnownPropertyValueRef::AspectRatio(value) = report.syntax()[5]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected aspect-ratio wrapper");
    };
    assert!(
        matches!(value.ratio(), CssAspectRatioValue::Ratio(ratio) if ratio.denominator().is_none() && ratio.numerator().literal().map(|literal| literal.component()).is_some())
    );

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
    assert_eq!(
        components
            .grow()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "2"
    );
    assert_eq!(
        components
            .shrink()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );
}

#[test]
fn ratio_literal_zero_and_symbolic_negative_math_keep_their_domains() {
    let literal_report = parse_style_attribute("aspect-ratio: 0; color: red");
    assert_eq!(literal_report.syntax().len(), 2);
    assert!(literal_report.is_clean());
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
    let CssFilter::Functions(functions) = filter.value() else {
        panic!("expected filter functions");
    };
    assert!(matches!(
        &functions.functions()[0],
        CssFilterFunction::Brightness(CssFilterAmount::Number(value)) if value.calculation().is_some()
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssFilterFunction::Opacity(CssFilterAmount::Percentage(value)) if value.calculation().is_some()
    ));
}

fn calculation_body(
    expression: CssCalculationExpressionRef<'_>,
) -> CssCalculationExpressionRef<'_> {
    match expression {
        CssCalculationExpressionRef::NestedCalc(root) => root.operand(),
        other => other,
    }
}

#[test]
fn exact_construction_preserves_mixed_origins_and_canonical_operator_mapping() {
    use surgeist_css::{
        CssComponentValue, CssComponentValues, CssValueOrigin, parse_component_values,
    };
    let children = parse_component_values("1px + 2px").unwrap();
    let original = children.items()[0].origin().clone();
    let operator = children.items()[2].origin().clone();
    let function = CssComponentValue::try_function("CALC", children).unwrap();
    let calculation = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::try_new(vec![function], Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(calculation.origin(), CssValueOrigin::Programmatic));
    let rebuilt = CssCalculation::<CssLength>::try_from_components(
        calculation.components().clone(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(calculation, rebuilt);
    let CssCalculationExpressionRef::Sum(sum) = calculation_body(calculation.expression()) else {
        panic!("expected sum")
    };
    assert_eq!(sum.term(1).unwrap().operator_origin(), Some(&operator));
    let CssCalculationExpressionRef::Value(leaf) = sum.term(0).unwrap().expression() else {
        panic!("expected leaf")
    };
    let CssValueOrigin::Parsed(found) = leaf.literal().origin() else {
        panic!("expected parsed leaf")
    };
    let CssValueOrigin::Parsed(original) = original else {
        panic!("expected parsed source")
    };
    assert!(found.source().same_snapshot(original.source()));
    let serialized = calculation.serialize_tokens(Default::default()).unwrap();
    assert_eq!(serialized.as_css(), "calc(1px + 2px)");
    assert!(
        matches!(serialized.origin_at(9), Some(surgeist_css::CssSerializedOrigin::Token(origin)) if origin == &operator)
    );
}

#[test]
fn exact_public_roots_keep_long_numbers_special_values_and_integer_requirements_distinct() {
    use surgeist_css::{CssCalculation, CssNumericConstant, CssResolution, parse_component_values};
    let integer = CssCalculation::<CssInteger>::try_from_components(
        parse_component_values("1234567890123456789012345678901234567890").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(!integer.requires_rounding());
    let CssCalculationExpressionRef::Value(leaf) = integer.expression() else {
        panic!("expected exact integer")
    };
    assert_eq!(
        leaf.literal().representation(),
        "1234567890123456789012345678901234567890"
    );
    let rounded = CssCalculation::<CssInteger>::try_from_components(
        parse_component_values("calc(1.5)").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(rounded.requires_rounding());
    let finite = CssCalculation::<CssNumber>::try_from_components(
        parse_component_values("calc(1e99999)").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(
        matches!(calculation_body(finite.expression()), CssCalculationExpressionRef::Value(v) if v.literal().representation() == "1e99999")
    );
    let symbolic = CssCalculation::<CssNumber>::try_from_components(
        parse_component_values("calc(infinity)").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(
        matches!(calculation_body(symbolic.expression()), CssCalculationExpressionRef::Constant(v) if v.value() == CssNumericConstant::Infinity)
    );
    let zero = CssCalculation::<CssNumber>::try_from_components(
        parse_component_values("-0").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(
        matches!(zero.expression(), CssCalculationExpressionRef::Value(v) if v.literal().representation() == "-0")
    );
    assert_eq!(
        CssCalculation::<CssResolution>::try_from_components(
            parse_component_values("calc(-1dppx / 0)").unwrap(),
            Default::default()
        )
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css(),
        "calc(-1dppx / 0)"
    );
    assert!(
        CssCalculation::<CssLength>::try_from_components(
            parse_component_values("calc(1px + 2%)").unwrap(),
            Default::default()
        )
        .unwrap_err()
        .is_root_domain_mismatch()
    );
    assert!(
        CssCalculation::<CssLengthPercentage>::try_from_components(
            parse_component_values("calc(1px + 2%)").unwrap(),
            Default::default()
        )
        .is_ok()
    );
}

#[test]
fn checked_numeric_construction_reports_resource_substitution_and_recovery_failures() {
    use surgeist_css::{CssComponentValueLimits, parse_component_values};
    let components = parse_component_values("calc(1*2)").unwrap();
    for limits in [
        CssComponentValueLimits::try_new(0, 4, 11).unwrap(),
        CssComponentValueLimits::try_new(1, 3, 11).unwrap(),
        CssComponentValueLimits::try_new(1, 4, 9).unwrap(),
    ] {
        let error = CssCalculation::<CssNumber>::try_from_components(components.clone(), limits)
            .unwrap_err();
        assert!(error.is_resource_limit(), "{error:?}");
        assert!(error.origin().is_some());
    }
    assert!(
        CssCalculation::<CssNumber>::try_from_components(
            components,
            CssComponentValueLimits::try_new(1, 4, 11).unwrap()
        )
        .is_ok()
    );
    for (source, kind) in [
        ("calc(var(--n))", Kind::SubstitutionRequired),
        ("calc(1", Kind::RecoveredComponent),
        ("1 + 2", Kind::MultipleValues),
        ("min()", Kind::Arity),
    ] {
        assert_eq!(
            numeric_failure(
                &CssCalculation::<CssNumber>::try_from_components(
                    parse_component_values(source).unwrap(),
                    Default::default()
                )
                .unwrap_err()
            ),
            kind,
            "{source}"
        );
    }
}
