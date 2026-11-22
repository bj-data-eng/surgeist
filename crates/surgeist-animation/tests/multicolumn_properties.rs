use surgeist_animation::composition::{
    AnimationType, AnimationTypeOperations, PropertyValueOperations,
};
use surgeist_animation::computed::{
    CalcSize, CalcSizeBasis, CalcSizeCalculation, CalcSizeFootprint, CalcSizeNode,
    ComputedComponent, ComputedDimension, ComputedInteger, ComputedNumeric, ComputedSize,
    ComputedSizeValue, ComputedValue, DimensionKind, DimensionPercentage, MathContext, NumericType,
    SizingKeyword,
};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableValue, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleValue, SampledProperty, SampledPropertyResult};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};
use surgeist_color::{
    Color, ColorSpace, Finite, InterpolationMethod, PredefinedColorSpace, UnitInterval,
};

#[test]
fn column_count_transition_rounds_up_at_the_half_integer() {
    let track = TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("column-count").unwrap(),
        count(1.0),
        count(2.0),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        Easing::linear(),
    )
    .unwrap();

    // CSS Values 4 §5.2.1 interpolates in real numbers and then rounds once.
    for (elapsed, expected) in [
        (0.5_f64.next_down(), 1.0),
        (0.5, 2.0),
        (0.5_f64.next_up(), 2.0),
    ] {
        let sample = track.sample(
            ElapsedTime::from_secs(elapsed).unwrap(),
            AnimationPlayState::Running,
        );
        let SampleValue::Value(property) = sample.value() else {
            panic!("expected a column-count sample at {elapsed}");
        };
        assert_eq!(property.property().as_str(), "column-count");
        assert_eq!(property.value(), &count(expected), "at {elapsed}");
    }
}

#[test]
fn column_count_retains_the_small_endpoint_in_a_wide_interpolation() {
    let pair = InterpolationPair::new(
        PropertyKey::new("column-count").unwrap(),
        count((1_u64 << 54) as f64),
        count(1.0),
    )
    .unwrap();
    let progress = InterpolationProgress::new(1.0_f64.next_down()).unwrap();

    // At p = 1 - 2^-53, the two contributions are 2 and 1 - 2^-53.
    assert_eq!(
        pair.sample(progress),
        InterpolationOutcome::Value(count(3.0))
    );
}

#[test]
fn column_count_overshoot_rounds_before_composition_and_final_range_handling() {
    let track = transition(
        pair("column-count", count(1.0), count(6.0)),
        Easing::linear_function(
            LinearEasing::new(vec![
                LinearControlPointInput {
                    input: 0.0,
                    output: 0.0,
                },
                LinearControlPointInput {
                    input: 0.25,
                    output: -0.5,
                },
                LinearControlPointInput {
                    input: 1.0,
                    output: 1.0,
                },
            ])
            .unwrap(),
        ),
    );
    let sample = track.sample(time(0.25), AnimationPlayState::Running);
    let property = sampled_property(&sample);

    // A sample of -1.5 rounds toward positive infinity and remains signed.
    assert_eq!(property.value(), &count(-1.0));
    let operations = AnimationTypeOperations::new(AnimationType::ByComputedValue);
    assert_eq!(
        operations
            .add(property.property(), &count(3.0), property.value())
            .unwrap(),
        count(2.0),
    );
}

#[test]
fn multicolumn_keyword_keyframes_switch_whole_values_at_the_midpoint() {
    let endpoints = [
        (
            "column-count",
            computed(ComputedComponent::Discrete(token("auto"))),
            count(4.0),
        ),
        (
            "column-width",
            length_size(length(18.0)),
            size(ComputedSize::keyword(SizingKeyword::MinContent)),
        ),
        ("column-rule-style", discrete("solid"), discrete("hidden")),
        ("column-span", discrete("none"), discrete("all")),
        ("column-fill", discrete("balance"), discrete("balance-all")),
    ];
    let track = keyframes(
        endpoints
            .iter()
            .map(|(name, from, to)| pair(name, from.clone(), to.clone())),
    );

    for (elapsed, before_midpoint) in [(0.5_f64.next_down(), true), (0.5, false)] {
        let frame = track.sample(time(elapsed), AnimationPlayState::Running);
        assert_eq!(frame.len(), endpoints.len());
        for (sample, (name, from, to)) in frame.iter().zip(&endpoints) {
            let property = sampled_property(sample);
            assert_eq!(property.property().as_str(), *name);
            assert_eq!(
                property.value(),
                if before_midpoint { from } else { to },
                "{name}"
            );
        }
    }
}

#[test]
fn column_width_keeps_computed_zero_and_unresolved_percentages() {
    let percentage = |value| ComputedNumeric::percentage(value).unwrap();
    let mixed = mix(6.0, 0.25);
    for (from, to, expected) in [
        (
            length_size(length(12.0)),
            length_size(percentage(0.5)),
            length_size(mixed.clone()),
        ),
        (
            fit_content(length(12.0)),
            fit_content(percentage(0.5)),
            fit_content(mixed),
        ),
    ] {
        let track = transition(pair("column-width", from, to), Easing::linear());
        let midpoint = track.sample(time(0.5), AnimationPlayState::Running);
        assert_eq!(sampled_property(&midpoint).value(), &expected);
    }

    let zero = length_size(length(0.0));
    let track = transition(
        pair("column-width", zero.clone(), length_size(length(1.0))),
        Easing::linear(),
    );
    // The used Layout floor of 1px does not change these computed samples.
    for (elapsed, expected) in [(0.0, zero), (0.5, length_size(length(0.5)))] {
        let sample = track.sample(time(elapsed), AnimationPlayState::Running);
        assert_eq!(sampled_property(&sample).value(), &expected);
    }
}

#[test]
fn column_width_with_calc_size_interpolates_against_the_intrinsic_basis() {
    let auto = size(ComputedSize::keyword(SizingKeyword::Auto));
    let definite_calc = CalcSize::from_numeric(length(0.0)).unwrap();
    let track = transition(
        pair(
            "column-width",
            auto.clone(),
            size(ComputedSize::calc_size(definite_calc)),
        ),
        Easing::linear(),
    );
    let start = track.sample(time(0.0), AnimationPlayState::Running);
    assert_eq!(sampled_property(&start).value(), &auto);

    let sample = track.sample(time(0.5), AnimationPlayState::Running);
    let InterpolableValue::Computed(value) = sampled_property(&sample).value() else {
        panic!("expected a computed column-width");
    };
    let [ComputedComponent::BoxSize(width)] = value.components() else {
        panic!("expected one computed box size");
    };
    let ComputedSizeValue::CalcSize(calc) = width.value() else {
        panic!(
            "expected a symbolic calc-size result, got {:?}",
            width.value()
        );
    };
    // CSS Values 5 §10.3 gives calc-size(auto, size * .5), without a used width.
    assert_eq!(calc.basis(), CalcSizeBasis::Keyword(SizingKeyword::Auto));
    assert_eq!(
        calc.calculation().nodes(),
        [
            CalcSizeNode::Size,
            CalcSizeNode::Numeric(length(0.0)),
            CalcSizeNode::Interpolate {
                first: 0,
                second: 1,
                progress: InterpolationProgress::new(0.5).unwrap(),
            },
        ],
    );
}

#[test]
fn uncombinable_calc_sizes_switch_the_whole_value_at_the_midpoint() {
    let context = MathContext::new();
    let kind = NumericType::Dimension(DimensionKind::Length);
    let oversized_pair_operand = || {
        let calculation = CalcSizeCalculation::source(
            context.source(kind),
            CalcSizeFootprint::new(2500, 0, 0).unwrap(),
        )
        .unwrap();
        ComputedSize::calc_size(CalcSize::new(CalcSizeBasis::Any, calculation).unwrap())
    };
    let record = |first, second| {
        InterpolableValue::computed(
            ComputedValue::new(vec![
                ComputedComponent::Numeric(length(first)),
                ComputedComponent::BoxSize(second),
            ])
            .unwrap(),
        )
    };
    for (from, to) in [
        (
            ComputedSize::keyword(SizingKeyword::Auto),
            ComputedSize::unavailable_calc_size(context.source(kind)).unwrap(),
        ),
        (oversized_pair_operand(), oversized_pair_operand()),
    ] {
        let composite_from = record(10.0, from.clone());
        let composite_to = record(30.0, to.clone());
        let from = size(from);
        let to = size(to);
        let track = keyframes([
            pair("column-width", from.clone(), to.clone()),
            pair(
                "--size-record",
                composite_from.clone(),
                composite_to.clone(),
            ),
        ]);

        for (elapsed, width, composite) in [
            (0.5_f64.next_down(), from, composite_from),
            (0.5, to, composite_to),
        ] {
            let frame = track.sample(time(elapsed), AnimationPlayState::Running);
            assert_eq!(frame.len(), 2);
            assert_eq!(sampled_property(&frame[0]).value(), &width);
            assert_eq!(sampled_property(&frame[1]).value(), &composite);
        }
    }
}

#[test]
fn computed_column_width_rejects_non_length_operands() {
    let context = MathContext::new();
    let angle = context.source(NumericType::Dimension(DimensionKind::Angle));
    let numeric = ComputedNumeric::source(angle.clone());
    for invalid in [
        ComputedSize::length_percentage(numeric.clone()),
        ComputedSize::fit_content(numeric),
        ComputedSize::unavailable_calc_size(angle),
    ] {
        assert_eq!(
            invalid.unwrap_err().kind(),
            surgeist_animation::computed::ComputedErrorKind::IncompatibleTypes
        );
    }
}

#[test]
fn supplied_rule_width_and_color_sample_independently() {
    let space = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
    let color = |channels: [f64; 3], alpha| {
        Color::new(space, channels.map(Finite::new), UnitInterval::new(alpha))
    };
    let track = keyframes([
        pair(
            "column-rule-width",
            numeric(length(0.0)),
            numeric(length(1.5)),
        ),
        InterpolationPair::resolved_colors(
            PropertyKey::new("column-rule-color").unwrap(),
            color([1.0, 0.0, 0.0], 0.0),
            color([0.0, 0.0, 1.0], 0.5),
            InterpolationMethod::try_new(space, None).unwrap(),
        ),
    ]);
    let frame = track.sample(time(0.5), AnimationPlayState::Running);
    assert_eq!(frame.len(), 2);
    let width = sampled_property(&frame[0]);
    assert_eq!(width.property().as_str(), "column-rule-width");
    // The caller supplied effective widths, including zero for a hidden rule.
    assert_eq!(width.value(), &numeric(length(0.75)));
    let sampled_color = sampled_property(&frame[1]);
    assert_eq!(sampled_color.property().as_str(), "column-rule-color");
    let InterpolableValue::ColorSample(color) = sampled_color.value() else {
        panic!(
            "expected checked rule color, got {:?}",
            sampled_color.value()
        );
    };
    assert_eq!(color.space(), space);
    assert_eq!(color.alpha().map(Finite::get), Some(0.25));
    assert_eq!(
        color.channels().map(|channel| channel.map(Finite::get)),
        [Some(0.0), Some(0.0), Some(1.0)]
    );
}

fn count(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::integer(
        ComputedInteger::new(value).unwrap(),
    ))
}

fn computed(component: ComputedComponent) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(component))
}

fn numeric(value: ComputedNumeric) -> InterpolableValue {
    computed(ComputedComponent::Numeric(value))
}

fn length(pixels: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(DimensionKind::Length, pixels).unwrap())
}

fn mix(pixels: f64, fraction: f64) -> ComputedNumeric {
    ComputedNumeric::mix(
        DimensionPercentage::new(
            ComputedDimension::new(DimensionKind::Length, pixels).unwrap(),
            fraction,
        )
        .unwrap(),
    )
}

fn size(value: ComputedSize) -> InterpolableValue {
    computed(ComputedComponent::BoxSize(value))
}

fn length_size(value: ComputedNumeric) -> InterpolableValue {
    size(ComputedSize::length_percentage(value).unwrap())
}

fn fit_content(value: ComputedNumeric) -> InterpolableValue {
    size(ComputedSize::fit_content(value).unwrap())
}

fn token(value: &str) -> DiscreteValue {
    DiscreteValue::from_text(value).unwrap()
}

fn discrete(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(token(value))
}

fn time(seconds: f64) -> ElapsedTime {
    ElapsedTime::from_secs(seconds).unwrap()
}

fn pair(name: &str, from: InterpolableValue, to: InterpolableValue) -> InterpolationPair {
    InterpolationPair::new(PropertyKey::new(name).unwrap(), from, to).unwrap()
}

fn transition(pair: InterpolationPair, easing: Easing) -> TransitionTrack {
    TransitionTrack::from_pair(
        TransitionTrackId::new(1),
        pair,
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        easing,
    )
    .unwrap()
}

fn keyframes(pairs: impl IntoIterator<Item = InterpolationPair>) -> KeyframeTrack {
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::None,
    )
    .unwrap();
    let segments = pairs.into_iter().map(|pair| {
        KeyframeSegment::from_pair(
            pair,
            UnitRatio::new(0.0).unwrap(),
            UnitRatio::new(1.0).unwrap(),
            Easing::linear(),
        )
        .unwrap()
    });
    KeyframeTrack::new(
        KeyframeAnimationId::new("computed multicolumn longhands").unwrap(),
        timing,
        segments.collect(),
    )
    .unwrap()
}

fn sampled_property(result: &SampledPropertyResult) -> &SampledProperty {
    let SampleValue::Value(property) = result.value() else {
        panic!("expected a sampled property, got {:?}", result.value());
    };
    property
}
