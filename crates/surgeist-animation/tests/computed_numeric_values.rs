use std::cmp::Ordering;
use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedErrorKind, ComputedInteger, ComputedNumeric,
    ComputedOperation, ComputedPosition, ComputedRatio, ComputedValue, DimensionKind,
    DimensionPercentage, MathContext, MathNode, NumericLiteral, NumericType,
};
use surgeist_animation::interpolation::DiscreteValue;
use surgeist_animation::interpolation::{
    InterpolableNumber, InterpolablePercentage, InterpolableValue, InterpolationOutcome,
    InterpolationPair, InterpolationProgress, PropertyKey,
};

#[test]
fn opposite_finite_numbers_have_a_finite_midpoint() {
    let pair = pair("number", number(f64::MAX), number(-f64::MAX));
    assert_eq!(
        pair.sample(progress(0.5)),
        InterpolationOutcome::Value(number(0.0))
    );
}

#[test]
fn opposite_finite_percentages_have_a_finite_midpoint() {
    let pair = pair("percentage", percentage(f64::MAX), percentage(-f64::MAX));
    assert_eq!(
        pair.sample(progress(0.5)),
        InterpolationOutcome::Value(percentage(0.0))
    );
}

#[test]
fn an_unrepresentable_extrapolation_fails_and_the_endpoint_remains_usable() {
    let pair = pair("number", number(0.0), number(f64::MAX));
    assert!(matches!(
        pair.sample(progress(2.0)),
        InterpolationOutcome::ArithmeticFailure(_)
    ));
    assert_eq!(
        pair.sample(progress(1.0)),
        InterpolationOutcome::Value(number(f64::MAX))
    );
}

#[test]
fn constant_extreme_values_survive_any_finite_progress() {
    for value in [f64::MAX, -f64::MAX, 1.0] {
        assert_eq!(
            pair("constant", number(value), number(value)).sample(progress(f64::MAX)),
            InterpolationOutcome::Value(number(value))
        );
    }
}

#[test]
fn opposite_finite_premultiplied_colors_have_a_finite_midpoint() {
    use surgeist_animation::interpolation::{ColorInterpolationSpace, InterpolableColor};
    use surgeist_animation::progress::UnitRatio;
    let color = |red| {
        InterpolableValue::color(
            InterpolableColor::from_straight_components(
                ColorInterpolationSpace::Srgb,
                red,
                0.0,
                0.0,
                UnitRatio::new(1.0).unwrap(),
            )
            .unwrap(),
        )
    };
    assert_eq!(
        pair("color", color(f64::MAX), color(-f64::MAX)).sample(progress(0.5)),
        InterpolationOutcome::Value(color(0.0))
    );
}

#[test]
fn integers_round_nearest_with_positive_infinity_ties() {
    for (first, second, expected) in [
        (1.0, 2.0, 2.0),
        (-2.0, -1.0, -1.0),
        (-1.0, 0.0, 0.0),
        (0.0, 1.0, 1.0),
    ] {
        let actual = integer(first)
            .interpolate(integer(second), progress(0.5))
            .unwrap();
        assert_eq!(actual.value(), expected);
    }
    for (input, expected) in [
        (0.5_f64.next_down(), 0.0),
        (0.5_f64.next_up(), 1.0),
        ((-0.5_f64).next_down(), -1.0),
        ((-0.5_f64).next_up(), 0.0),
    ] {
        assert_eq!(
            integer(0.0)
                .interpolate(integer(1.0), progress(input))
                .unwrap()
                .value(),
            expected
        );
    }
    assert_eq!(integer(-3.0).add(&integer(5.0)).unwrap().value(), 2.0);
    assert_eq!(
        integer(f64::MAX)
            .interpolate(integer(-f64::MAX), progress(0.5))
            .unwrap()
            .value(),
        0.0
    );
    assert_eq!(
        ComputedInteger::new(1.25).unwrap_err().kind(),
        ComputedErrorKind::FractionalInteger
    );
    assert_eq!(
        ComputedInteger::new(f64::INFINITY).unwrap_err().kind(),
        ComputedErrorKind::NonFiniteInput
    );
}

#[test]
fn dimensions_and_percentage_mixes_keep_independent_components_and_canonical_units() {
    for kind in [
        DimensionKind::Length,
        DimensionKind::Angle,
        DimensionKind::Time,
        DimensionKind::Frequency,
        DimensionKind::Resolution,
    ] {
        let first = dim(kind, 10.0);
        let second = ComputedNumeric::percentage(0.5).unwrap();
        let value = first.interpolate(&second, progress(0.5)).unwrap();
        assert_eq!(
            value.as_literal(),
            Some(NumericLiteral::Mix(
                DimensionPercentage::new(ComputedDimension::new(kind, 5.0).unwrap(), 0.25).unwrap()
            ))
        );
        assert_eq!(first.add(&dim(kind, 20.0)).unwrap(), dim(kind, 30.0));
        assert_eq!(first.accumulate(&dim(kind, 20.0)).unwrap(), dim(kind, 30.0));
        assert_eq!(
            value.add(&dim(kind, -5.0)).unwrap().as_literal(),
            Some(NumericLiteral::Percentage(0.25))
        );
    }
    assert_eq!(
        dim(DimensionKind::Length, 1.0)
            .add(&dim(DimensionKind::Time, 1.0))
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
    assert_eq!(
        ComputedNumeric::number(1.0)
            .unwrap()
            .add(&ComputedNumeric::percentage(1.0).unwrap())
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
    let zero_percentage = DimensionPercentage::new(
        ComputedDimension::new(DimensionKind::Length, 10.0).unwrap(),
        0.0,
    )
    .unwrap();
    assert_eq!(
        ComputedNumeric::mix(zero_percentage),
        dim(DimensionKind::Length, 10.0)
    );
    assert_eq!(
        ComputedNumeric::literal(NumericLiteral::Mix(zero_percentage)).unwrap(),
        dim(DimensionKind::Length, 10.0)
    );
}

#[test]
fn negative_intermediates_are_corrected_by_later_addition_without_clamping() {
    let value = dim(DimensionKind::Length, 10.0)
        .interpolate(&dim(DimensionKind::Length, 0.0), progress(2.0))
        .unwrap();
    assert_eq!(value, dim(DimensionKind::Length, -10.0));
    assert_eq!(
        value.add(&dim(DimensionKind::Length, 15.0)).unwrap(),
        dim(DimensionKind::Length, 5.0)
    );
}

#[test]
fn ratios_compare_quotients_and_interpolate_geometrically() {
    assert_eq!(
        ratio(3.0, 2.0).compare(ratio(2.0, 1.0)),
        Some(Ordering::Less)
    );
    assert_eq!(
        ratio(3.0, 2.0).compare(ratio(300.0, 200.0)),
        Some(Ordering::Equal)
    );
    assert_eq!(
        ratio(f64::MAX, f64::MIN_POSITIVE).compare(ratio(f64::MAX, 1.0)),
        Some(Ordering::Greater)
    );
    assert_eq!(
        ratio(f64::MIN_POSITIVE, f64::MAX).compare(ratio(f64::from_bits(1), f64::MAX)),
        Some(Ordering::Greater)
    );
    let value = ratio(1.0, 4.0)
        .interpolate(ratio(4.0, 1.0), progress(0.5))
        .unwrap();
    assert_eq!(value, ratio(1.0, 1.0));
    let scaled = ratio(100.0, 400.0)
        .interpolate(ratio(400.0, 100.0), progress(0.5))
        .unwrap();
    assert!((scaled.numerator() - 1.0).abs() < 1e-14);
    let published_example = ratio(5.0, 1.0)
        .interpolate(ratio(3.0, 2.0), progress(0.5))
        .unwrap();
    assert!((published_example.numerator() - 7.5_f64.sqrt()).abs() < 1e-14);
    let first = ratio(300.0, 200.0);
    assert!(
        (ratio(1.0, 1.0)
            .interpolate(ratio(4.0, 1.0), progress(2.0))
            .unwrap()
            .numerator()
            - 16.0)
            .abs()
            < 1e-13
    );
    assert!(
        (ratio(1.0, 1.0)
            .interpolate(ratio(4.0, 1.0), progress(-1.0))
            .unwrap()
            .numerator()
            - 0.25)
            .abs()
            < 1e-14
    );
    assert_eq!(
        first.interpolate(ratio(4.0, 1.0), progress(0.0)).unwrap(),
        first
    );
    assert_eq!(first.add(ratio(7.0, 3.0)), ratio(7.0, 3.0));
}

#[test]
fn degenerate_ratios_have_explicit_comparison_and_interpolation_results() {
    for value in [
        ratio(0.0, 1.0),
        ratio(1.0, 0.0),
        ratio(f64::INFINITY, 1.0),
        ratio(1.0, f64::INFINITY),
        ratio(0.0, 0.0),
        ratio(f64::INFINITY, f64::INFINITY),
    ] {
        assert!(value.is_degenerate());
        for progress in [progress(0.0), progress(0.5), progress(1.0)] {
            assert_eq!(
                value
                    .interpolate(ratio(1.0, 1.0), progress)
                    .unwrap_err()
                    .kind(),
                ComputedErrorKind::DegenerateRatio
            );
        }
    }
    assert_eq!(ratio(0.0, 0.0).compare(ratio(1.0, 1.0)), None);
    assert_eq!(
        ratio(f64::INFINITY, f64::INFINITY).compare(ratio(1.0, 1.0)),
        None
    );
    assert_eq!(
        ratio(1.0, 0.0).compare(ratio(1.0, 1.0)),
        Some(Ordering::Greater)
    );
    assert_eq!(
        ratio(1.0, f64::INFINITY).compare(ratio(0.0, 1.0)),
        Some(Ordering::Equal)
    );
    assert_eq!(
        ComputedRatio::new(-1.0, 1.0).unwrap_err().kind(),
        ComputedErrorKind::NegativeRatioComponent
    );
    assert_eq!(
        ComputedRatio::new(f64::NAN, 1.0).unwrap_err().kind(),
        ComputedErrorKind::NonFiniteInput
    );
    assert_eq!(
        ratio(1.0, 1.0)
            .interpolate(ratio(2.0, 1.0), progress(f64::MAX))
            .unwrap_err()
            .kind(),
        ComputedErrorKind::NonFiniteResult
    );
    assert_eq!(
        ratio(2.0, 1.0)
            .interpolate(ratio(1.0, 1.0), progress(f64::MAX))
            .unwrap_err()
            .kind(),
        ComputedErrorKind::RatioUnderflow
    );
}

#[test]
fn positions_sample_and_add_each_normalized_axis_without_a_reference_box() {
    let first = ComputedPosition::new(
        dim(DimensionKind::Length, 10.0),
        ComputedNumeric::percentage(0.25).unwrap(),
    )
    .unwrap();
    let second = ComputedPosition::new(
        ComputedNumeric::percentage(0.5).unwrap(),
        dim(DimensionKind::Length, 20.0),
    )
    .unwrap();
    let value = first.interpolate(&second, progress(0.5)).unwrap();
    assert_eq!(
        value.x().as_literal(),
        Some(NumericLiteral::Mix(
            DimensionPercentage::new(
                ComputedDimension::new(DimensionKind::Length, 5.0).unwrap(),
                0.25
            )
            .unwrap()
        ))
    );
    assert_eq!(
        value.y().as_literal(),
        Some(NumericLiteral::Mix(
            DimensionPercentage::new(
                ComputedDimension::new(DimensionKind::Length, 10.0).unwrap(),
                0.125
            )
            .unwrap()
        ))
    );
    assert_eq!(
        first.add(&second).unwrap(),
        first.accumulate(&second).unwrap()
    );
    assert!(
        ComputedPosition::new(
            dim(DimensionKind::Angle, 2.0),
            dim(DimensionKind::Length, 2.0)
        )
        .is_err()
    );
}

#[test]
fn contextual_math_keeps_source_identity_and_exposes_a_typed_composition_plan() {
    let context = MathContext::new();
    let source = context.source(NumericType::DimensionPercentage(DimensionKind::Length));
    let other_context = MathContext::new();
    let second = other_context.source(source.numeric_type());
    assert_eq!(source.clone(), source);
    assert_ne!(source, second);
    assert_ne!(source.context(), second.context());
    let first = ComputedNumeric::source(source.clone());
    let second_value = ComputedNumeric::source(second.clone());
    let value = first
        .interpolate(&second_value, progress(0.25))
        .unwrap()
        .add(&dim(DimensionKind::Length, 5.0))
        .unwrap();
    assert_eq!(
        value.numeric_type(),
        NumericType::DimensionPercentage(DimensionKind::Length)
    );
    // Independent caller-side source evaluation: contextual sources resolve to
    // 10px and 50px here, so quarter interpolation plus 5px is 25px.
    let mut results = Vec::new();
    for node in value.math_nodes().unwrap() {
        let result = match node {
            MathNode::Source(value) if *value == source => 10.0,
            MathNode::Source(value) if *value == second => 50.0,
            MathNode::Literal(NumericLiteral::Dimension(value)) => value.value(),
            MathNode::Combine {
                operation: ComputedOperation::Interpolate(progress),
                first,
                second,
                ..
            } => (1.0 - progress.value()) * results[*first] + progress.value() * results[*second],
            MathNode::Combine {
                operation: ComputedOperation::Add,
                first,
                second,
                ..
            } => results[*first] + results[*second],
            other => panic!("unexpected numeric plan node {other:?}"),
        };
        results.push(result);
    }
    assert_eq!(results.last(), Some(&25.0));
    assert_eq!(
        first.interpolate(&second_value, progress(0.0)).unwrap(),
        first
    );
    assert_eq!(
        first.interpolate(&second_value, progress(1.0)).unwrap(),
        second_value
    );
    assert_eq!(
        first
            .add(&ComputedNumeric::number(1.0).unwrap())
            .unwrap_err()
            .kind(),
        ComputedErrorKind::IncompatibleTypes
    );
}

#[test]
fn a_deep_symbolic_fold_is_iterable_and_drops_without_recursive_ownership() {
    let source = ComputedNumeric::source(MathContext::new().source(NumericType::Number));
    let mut value = source.clone();
    let one = ComputedNumeric::number(1.0).unwrap();
    for _ in 0..256 {
        value = value.accumulate(&one).unwrap();
    }
    let mut results = Vec::new();
    for node in value.math_nodes().unwrap() {
        results.push(match node {
            MathNode::Source(_) => 10.0,
            MathNode::Literal(NumericLiteral::Number(value)) => *value,
            MathNode::Combine {
                operation: ComputedOperation::Accumulate,
                first,
                second,
                ..
            } => results[*first] + results[*second],
            node => panic!("unexpected accumulation node {node:?}"),
        });
    }
    assert_eq!(results.last(), Some(&266.0));
    drop(value);
    assert!(matches!(
        source.math_nodes().unwrap().last(),
        Some(MathNode::Source(_))
    ));
}

#[test]
fn every_numeric_admission_path_rejects_nonfinite_literals() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for result in [
            ComputedNumeric::number(value),
            ComputedNumeric::percentage(value),
            ComputedNumeric::literal(NumericLiteral::Number(value)),
            ComputedNumeric::literal(NumericLiteral::Percentage(value)),
        ] {
            assert_eq!(
                result.unwrap_err().kind(),
                ComputedErrorKind::NonFiniteInput
            );
        }
        assert_eq!(
            ComputedDimension::new(DimensionKind::Length, value)
                .unwrap_err()
                .kind(),
            ComputedErrorKind::NonFiniteInput
        );
        assert_eq!(
            DimensionPercentage::new(
                ComputedDimension::new(DimensionKind::Length, 1.0).unwrap(),
                value
            )
            .unwrap_err()
            .kind(),
            ComputedErrorKind::NonFiniteInput
        );
    }
    assert_eq!(
        ComputedNumeric::number(f64::MAX)
            .unwrap()
            .add(&ComputedNumeric::number(f64::MAX).unwrap())
            .unwrap_err()
            .kind(),
        ComputedErrorKind::NonFiniteResult
    );
    assert_eq!(
        ComputedNumeric::integer(integer(f64::MAX))
            .accumulate(&ComputedNumeric::integer(integer(f64::MAX)))
            .unwrap_err()
            .operation(),
        Some(ComputedOperation::Accumulate)
    );
}

#[test]
fn nonadditive_ratios_preserve_other_components_addition_even_when_degenerate() {
    let first = property_value(vec![
        numeric(10.0),
        ComputedComponent::Ratio(ratio(0.0, 1.0)),
    ]);
    let second = property_value(vec![
        numeric(20.0),
        ComputedComponent::Ratio(ratio(3.0, 2.0)),
    ]);
    let expected = property_value(vec![
        numeric(30.0),
        ComputedComponent::Ratio(ratio(3.0, 2.0)),
    ]);
    assert_eq!(first.add(&second).unwrap(), expected);
    assert_eq!(first.accumulate(&second).unwrap(), expected);
    assert_eq!(first.interpolate(&second, progress(0.25)).unwrap(), first);
    assert_eq!(first.interpolate(&second, progress(0.5)).unwrap(), second);
}

#[test]
fn computed_ratio_and_integer_components_sample_through_an_ordinary_pair() {
    let first = property_value(vec![
        ComputedComponent::Ratio(ratio(1.0, 4.0)),
        ComputedComponent::Numeric(ComputedNumeric::integer(integer(1.0))),
    ]);
    let second = property_value(vec![
        ComputedComponent::Ratio(ratio(4.0, 1.0)),
        ComputedComponent::Numeric(ComputedNumeric::integer(integer(2.0))),
    ]);
    let result = pair(
        "computed",
        InterpolableValue::computed(first),
        InterpolableValue::computed(second),
    )
    .sample(progress(0.5));
    assert_eq!(
        result,
        InterpolationOutcome::Value(InterpolableValue::computed(property_value(vec![
            ComputedComponent::Ratio(ratio(1.0, 1.0)),
            ComputedComponent::Numeric(ComputedNumeric::integer(integer(2.0)))
        ])))
    );
}

#[test]
fn a_discrete_component_conflict_switches_the_whole_property_before_numeric_arithmetic() {
    let first = property_value(vec![numeric(0.0), keyword("auto")]);
    let second = property_value(vec![numeric(f64::MAX), keyword("content")]);
    assert_eq!(first.interpolate(&second, progress(0.25)).unwrap(), first);
    assert_eq!(first.interpolate(&second, progress(0.5)).unwrap(), second);
    // The arithmetic would overflow, but incompatibility is established first.
    assert_eq!(first.interpolate(&second, progress(2.0)).unwrap(), second);
    assert_eq!(first.add(&second).unwrap(), second);
    assert_eq!(first.accumulate(&second).unwrap(), second);
    let fewer = ComputedValue::single(numeric(1.0));
    assert_eq!(first.interpolate(&fewer, progress(0.5)).unwrap(), fewer);
    assert!(ComputedValue::new(vec![]).is_err());
}

#[test]
fn whole_properties_combine_numeric_components_and_report_the_failing_component() {
    let first = property_value(vec![numeric(10.0), keyword("normal")]);
    let second = property_value(vec![numeric(20.0), keyword("normal")]);
    assert_eq!(
        first.interpolate(&second, progress(0.5)).unwrap(),
        property_value(vec![numeric(15.0), keyword("normal")])
    );
    assert_eq!(
        first.add(&second).unwrap(),
        property_value(vec![numeric(30.0), keyword("normal")])
    );
    let first = property_value(vec![numeric(0.0), numeric(0.0)]);
    let second = property_value(vec![numeric(1.0), numeric(f64::MAX)]);
    let error = first.interpolate(&second, progress(2.0)).unwrap_err();
    assert_eq!(error.kind(), ComputedErrorKind::NonFiniteResult);
    assert_eq!(error.component(), Some(1));
    assert_eq!(
        error.operation(),
        Some(ComputedOperation::Interpolate(progress(2.0)))
    );
}

#[test]
fn computed_values_sample_through_transition_and_keyframe_tracks_with_recoverable_errors() {
    use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
    use surgeist_animation::keyframe::{
        KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
    };
    use surgeist_animation::progress::UnitRatio;
    use surgeist_animation::sample::{SampleClassification, SampleValue};
    use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
    use surgeist_animation::timing::{
        AnimationDirection, AnimationPlayState, FillMode, IterationCount,
    };
    use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};
    let first = InterpolableValue::computed(ComputedValue::single(numeric(0.0)));
    let second = InterpolableValue::computed(ComputedValue::single(numeric(f64::MAX)));
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 0.5,
                output: 2.0,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    );
    let track = TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("computed").unwrap(),
        first.clone(),
        second.clone(),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        easing.clone(),
    )
    .unwrap();
    let sample = track.sample(
        ElapsedTime::from_secs(0.5).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(sample.classification(), SampleClassification::Failed);
    assert!(
        matches!(sample.value(), SampleValue::Error(surgeist_animation::interpolation::InterpolationError::Computed(value)) if value.property().as_str() == "computed" && value.error().component() == Some(0))
    );
    let recovered = track.sample(
        ElapsedTime::from_secs(0.0).unwrap(),
        AnimationPlayState::Running,
    );
    assert!(matches!(recovered.value(), SampleValue::Value(value) if *value.value() == first));
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    let segment = KeyframeSegment::new(
        PropertyKey::new("computed").unwrap(),
        UnitRatio::new(0.0).unwrap(),
        UnitRatio::new(1.0).unwrap(),
        first,
        second.clone(),
        easing,
    )
    .unwrap();
    let keyframes = KeyframeTrack::new(
        KeyframeAnimationId::new("computed").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();
    assert_eq!(
        keyframes.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running
        )[0]
        .classification(),
        SampleClassification::Failed
    );
    assert!(
        matches!(keyframes.sample(ElapsedTime::from_secs(1.0).unwrap(), AnimationPlayState::Running)[0].value(), SampleValue::Value(value) if *value.value() == second)
    );
}

fn integer(value: f64) -> ComputedInteger {
    ComputedInteger::new(value).unwrap()
}
fn ratio(first: f64, second: f64) -> ComputedRatio {
    ComputedRatio::new(first, second).unwrap()
}
fn dim(kind: DimensionKind, value: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(kind, value).unwrap())
}
fn numeric(value: f64) -> ComputedComponent {
    ComputedComponent::Numeric(ComputedNumeric::number(value).unwrap())
}
fn keyword(value: &str) -> ComputedComponent {
    ComputedComponent::Discrete(DiscreteValue::new(value).unwrap())
}
fn property_value(components: Vec<ComputedComponent>) -> ComputedValue {
    ComputedValue::new(components).unwrap()
}

fn pair(property: &str, from: InterpolableValue, to: InterpolableValue) -> InterpolationPair {
    InterpolationPair::new(PropertyKey::from_text(property).unwrap(), from, to).unwrap()
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn percentage(value: f64) -> InterpolableValue {
    InterpolableValue::percentage(InterpolablePercentage::new(value).unwrap())
}

fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
