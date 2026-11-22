#![forbid(unsafe_code)]

use surgeist_animation::composition::{
    AnimationType, AnimationTypeOperations, PropertyOperationErrorKind, PropertyValueOperations,
};
use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedErrorKind, ComputedNumeric, ComputedOperation,
    ComputedPosition, ComputedValue, DimensionKind, DimensionPercentage, LayerSize, LayerSizeAxis,
    MathContext, MathNode, NumericType, RepeatableList,
};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
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

#[test]
fn size_axes_guard_the_length_percentage_domain_without_clamping_samples() {
    for value in [
        ComputedNumeric::number(1.).unwrap(),
        dimension(DimensionKind::Angle, 1.),
        dimension(DimensionKind::Time, 1.),
        ComputedNumeric::source(MathContext::new().source(NumericType::Number)),
    ] {
        assert_eq!(
            LayerSizeAxis::numeric(value).unwrap_err().kind(),
            ComputedErrorKind::IncompatibleTypes
        );
    }
    for value in [
        px(-4.),
        percent(-0.5),
        mixed(-3., 0.25),
        ComputedNumeric::source(
            MathContext::new().source(NumericType::DimensionPercentage(DimensionKind::Length)),
        ),
    ] {
        assert_eq!(
            LayerSizeAxis::numeric(value.clone())
                .unwrap()
                .numeric_value(),
            Some(&value)
        );
    }
    assert_eq!(LayerSizeAxis::auto().numeric_value(), None);
}

#[test]
fn background_and_mask_positions_repeat_and_combine_as_normalized_positions() {
    let first = positions(&[(0., 10.), (20., 30.)]);
    let second = positions(&[(10., 20.), (30., 40.), (50., 60.)]);
    let midpoint = positions(&[
        (5., 15.),
        (25., 35.),
        (25., 35.),
        (15., 25.),
        (15., 25.),
        (35., 45.),
    ]);
    let sum = positions(&[
        (10., 30.),
        (50., 70.),
        (50., 70.),
        (30., 50.),
        (30., 50.),
        (70., 90.),
    ]);
    for property in ["background-position", "mask-position"] {
        assert_operations(property, &first, &second, &midpoint, &sum);
    }
}

#[test]
fn background_and_mask_sizes_reuse_the_same_lcm_numeric_pairing() {
    let first = sizes(&[(0., 10.), (20., 30.)]);
    let second = sizes(&[(10., 20.), (30., 40.), (50., 60.)]);
    let midpoint = sizes(&[
        (5., 15.),
        (25., 35.),
        (25., 35.),
        (15., 25.),
        (15., 25.),
        (35., 45.),
    ]);
    let sum = sizes(&[
        (10., 30.),
        (50., 70.),
        (50., 70.),
        (30., 50.),
        (30., 50.),
        (70., 90.),
    ]);
    for property in ["background-size", "mask-size"] {
        assert_operations(property, &first, &second, &midpoint, &sum);
    }
}

#[test]
fn matched_auto_contain_and_cover_are_constant_while_numeric_axes_interpolate() {
    // WPT background-size's matched-keyword case; Masking delegates these values to Backgrounds.
    let from = size_list(vec![
        auto_height(0.),
        explicit(px(0.), px(0.)),
        LayerSize::Contain,
        LayerSize::Cover,
    ]);
    let to = size_list(vec![
        auto_height(40.),
        explicit(px(40.), px(40.)),
        LayerSize::Contain,
        LayerSize::Cover,
    ]);
    let expected = size_list(vec![
        auto_height(20.),
        explicit(px(20.), px(20.)),
        LayerSize::Contain,
        LayerSize::Cover,
    ]);
    for property in ["background-size", "mask-size"] {
        assert_sample(property, &from, &to, 0.5, &expected);
        assert_eq!(from.add(&to).unwrap(), to);
        assert_eq!(from.accumulate(&to).unwrap(), to);
    }
    let a = size_list(vec![auto_height(0.), auto_height(20.)]);
    let b = size_list(vec![auto_height(10.), auto_height(30.), auto_height(50.)]);
    assert_eq!(
        a.interpolate(&b, p(0.5)).unwrap(),
        size_list(vec![
            auto_height(5.),
            auto_height(25.),
            auto_height(25.),
            auto_height(15.),
            auto_height(15.),
            auto_height(35.)
        ])
    );
    // Compatible lists retain repeated cardinality even at exact endpoints.
    assert_eq!(
        a.interpolate(&b, p(0.)).unwrap(),
        size_list(vec![
            auto_height(0.),
            auto_height(20.),
            auto_height(0.),
            auto_height(20.),
            auto_height(0.),
            auto_height(20.)
        ])
    );
}

#[test]
fn matching_keywords_do_not_prevent_addition_of_the_other_axis() {
    let first = size_list(vec![auto_width(10.), LayerSize::Contain, LayerSize::Cover]);
    let second = size_list(vec![auto_width(30.), LayerSize::Contain, LayerSize::Cover]);
    let expected = size_list(vec![auto_width(40.), LayerSize::Contain, LayerSize::Cover]);
    let operations = operations();
    for property in ["background-size", "mask-size"] {
        assert_eq!(
            operations
                .add(
                    &key(property),
                    &wrapped(first.clone()),
                    &wrapped(second.clone())
                )
                .unwrap(),
            wrapped(expected.clone())
        );
        assert_eq!(
            operations
                .accumulate(
                    &key(property),
                    &wrapped(first.clone()),
                    &wrapped(second.clone())
                )
                .unwrap(),
            wrapped(expected.clone())
        );
    }
}

#[test]
fn late_keyword_mismatch_selects_original_whole_lists_before_arithmetic() {
    let from = size_list(vec![auto_height(0.), auto_height(20.)]);
    for keyword in [LayerSize::Contain, LayerSize::Cover, auto_width(30.)] {
        let to = size_list(vec![auto_height(f64::MAX), auto_height(30.), keyword]);
        for property in ["background-size", "mask-size"] {
            assert_fallback(property, &from, &to);
        }
    }
    assert_fallback(
        "background-size",
        &size_list(vec![LayerSize::Contain]),
        &size_list(vec![LayerSize::Cover]),
    );
    // A second-axis conflict is also preflighted before the first axis overflows.
    assert_fallback(
        "mask-size",
        &size_list(vec![auto_height(0.)]),
        &sizes(&[(f64::MAX, 10.)]),
    );
}

#[test]
fn keyword_incompatible_underlying_sizes_are_replaced_for_addition() {
    // WPT composition: auto/contain underlying cannot combine with an explicit effect.
    let underlying = size_list(vec![
        LayerSize::Explicit {
            horizontal: LayerSizeAxis::auto(),
            vertical: LayerSizeAxis::auto(),
        },
        LayerSize::Contain,
    ]);
    let effect = sizes(&[(100., 150.)]);
    for property in ["background-size", "mask-size"] {
        let operations = operations();
        assert_eq!(
            operations
                .add(
                    &key(property),
                    &wrapped(underlying.clone()),
                    &wrapped(effect.clone())
                )
                .unwrap(),
            wrapped(effect.clone())
        );
        assert_eq!(
            operations
                .accumulate(
                    &key(property),
                    &wrapped(underlying.clone()),
                    &wrapped(effect.clone())
                )
                .unwrap(),
            wrapped(effect.clone())
        );
    }
}

#[test]
fn ordinary_background_clip_and_origin_keep_any_discrete_whole_value_fallback() {
    for property in ["background-clip", "background-origin"] {
        let from = tags(&["content-box"]);
        let to = tags(&["padding-box", "border-box"]);
        assert_fallback(property, &from, &to);
        // Even equal ordinary discrete components do not become constant size tags.
        assert_fallback(
            property,
            &tags(&["border-box"]),
            &tags(&["border-box", "border-box"]),
        );
    }
    let item = |amount| {
        ComputedValue::new(vec![
            ComputedComponent::Size(auto_height(amount)),
            ComputedComponent::Discrete(DiscreteValue::new("same").unwrap()),
        ])
        .unwrap()
    };
    let first = RepeatableList::new(vec![item(0.)]).unwrap();
    let second = RepeatableList::new(vec![item(40.), item(80.)]).unwrap();
    assert_fallback("--supplied-repeatable-record", &first, &second);
}

#[test]
fn percentage_terms_and_signed_extrapolated_sizes_remain_unresolved() {
    let from = size_list(vec![explicit(px(10.), percent(0.25))]);
    let to = size_list(vec![explicit(percent(0.5), px(26.))]);
    let midpoint = size_list(vec![explicit(mixed(5., 0.25), mixed(13., 0.125))]);
    let extrapolated = size_list(vec![explicit(mixed(-5., 0.75), mixed(39., -0.125))]);
    for property in ["background-size", "mask-size"] {
        assert_sample(property, &from, &to, 0.5, &midpoint);
        assert_sample(property, &from, &to, 1.5, &extrapolated);
    }
    let from = sizes(&[(10., 20.)]);
    let to = sizes(&[(0., 0.)]);
    assert_sample("background-size", &from, &to, 2., &sizes(&[(-10., -20.)]));
}

#[test]
fn repeated_position_and_size_sources_keep_their_complete_context_identity() {
    let context = MathContext::new();
    let source = context.source(NumericType::DimensionPercentage(DimensionKind::Length));
    let numeric = ComputedNumeric::source(source.clone());
    let cases = [
        (
            "background-position",
            RepeatableList::new(vec![ComputedValue::single(ComputedComponent::Position(
                ComputedPosition::new(numeric.clone(), px(0.)).unwrap(),
            ))])
            .unwrap(),
            positions(&[(10., 20.), (30., 40.)]),
        ),
        (
            "mask-size",
            size_list(vec![explicit(numeric, px(0.))]),
            sizes(&[(10., 20.), (30., 40.)]),
        ),
    ];
    for (property, first, second) in cases {
        let InterpolationOutcome::Value(InterpolableValue::RepeatableList(result)) =
            pair(property, &first, &second).sample(p(0.5))
        else {
            panic!("expected repeatable sample")
        };
        assert_eq!(result.items().len(), 2);
        for item in result.items() {
            let numeric = match &item.components()[0] {
                ComputedComponent::Position(value) => value.x(),
                ComputedComponent::Size(LayerSize::Explicit { horizontal, .. }) => {
                    horizontal.numeric_value().unwrap()
                }
                value => panic!("unexpected component {value:?}"),
            };
            assert!(numeric.math_nodes().unwrap().iter().any(|node| matches!(node, MathNode::Source(actual) if *actual == source && actual.context() == &context)));
        }
    }
}

#[test]
fn position_and_size_failures_are_typed_indexed_and_recoverable() {
    for (property, from, to) in [
        (
            "background-position",
            positions(&[(0., 0.), (0., 0.)]),
            positions(&[(1., 2.), (f64::MAX, 0.)]),
        ),
        (
            "mask-size",
            sizes(&[(0., 0.), (0., 0.)]),
            sizes(&[(1., 2.), (f64::MAX, 0.)]),
        ),
    ] {
        let pair = pair(property, &from, &to);
        let InterpolationOutcome::ComputedFailure(error) = pair.sample(p(2.)) else {
            panic!("expected arithmetic failure")
        };
        assert_eq!(error.property().as_str(), property);
        assert_eq!(error.error().kind(), ComputedErrorKind::NonFiniteResult);
        assert_eq!(
            error.error().operation(),
            Some(ComputedOperation::Interpolate(p(2.)))
        );
        assert_eq!(error.error().component(), Some(1));
        assert_eq!(
            pair.sample(p(1.)),
            InterpolationOutcome::Value(wrapped(to.clone()))
        );
        let error = operations()
            .add(&key(property), &wrapped(to.clone()), &wrapped(to))
            .unwrap_err();
        let PropertyOperationErrorKind::Computed(error) = error.kind() else {
            panic!("expected computed error")
        };
        assert_eq!(error.kind(), ComputedErrorKind::NonFiniteResult);
        assert_eq!(error.component(), Some(1));
    }
}

#[test]
fn background_and_mask_lists_share_transition_and_keyframe_sampling_paths() {
    for (property, from, to, expected) in [
        (
            "background-position",
            positions(&[(0., 10.)]),
            positions(&[(20., 30.), (40., 50.)]),
            positions(&[(10., 20.), (20., 30.)]),
        ),
        (
            "mask-position",
            positions(&[(0., 10.)]),
            positions(&[(20., 30.), (40., 50.)]),
            positions(&[(10., 20.), (20., 30.)]),
        ),
        (
            "background-size",
            size_list(vec![auto_height(0.)]),
            size_list(vec![auto_height(20.), auto_height(40.)]),
            size_list(vec![auto_height(10.), auto_height(20.)]),
        ),
        (
            "mask-size",
            size_list(vec![auto_width(0.)]),
            size_list(vec![auto_width(20.), auto_width(40.)]),
            size_list(vec![auto_width(10.), auto_width(20.)]),
        ),
    ] {
        let transition = transition(property, &from, &to, Easing::linear());
        let actual = transition.sample(time(0.5), AnimationPlayState::Running);
        assert_eq!(actual.classification(), SampleClassification::Active);
        assert!(
            matches!(actual.value(), SampleValue::Value(value) if *value.value() == wrapped(expected.clone()))
        );
        let segment = KeyframeSegment::new(
            key(property),
            UnitRatio::new(0.).unwrap(),
            UnitRatio::new(1.).unwrap(),
            wrapped(from),
            wrapped(to),
            Easing::linear(),
        )
        .unwrap();
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("layers").unwrap(),
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.).unwrap(),
                AnimationDuration::from_secs(1.).unwrap(),
                IterationCount::finite(1.).unwrap(),
                AnimationDirection::Normal,
                FillMode::Both,
            )
            .unwrap(),
            vec![segment],
        )
        .unwrap();
        assert!(
            matches!(track.sample(time(0.5), AnimationPlayState::Running)[0].value(), SampleValue::Value(value) if *value.value() == wrapped(expected))
        );
    }
}

#[test]
fn a_track_can_recover_from_size_overshoot_at_a_later_active_endpoint() {
    let from = size_list(vec![auto_height(0.)]);
    let to = size_list(vec![auto_height(f64::MAX)]);
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.,
                output: 0.,
            },
            LinearControlPointInput {
                input: 0.5,
                output: 2.,
            },
            LinearControlPointInput {
                input: 0.75,
                output: 1.,
            },
            LinearControlPointInput {
                input: 1.,
                output: 1.,
            },
        ])
        .unwrap(),
    );
    let transition = transition("background-size", &from, &to, easing);
    let sample = transition.sample(time(0.5), AnimationPlayState::Running);
    assert_eq!(sample.classification(), SampleClassification::Failed);
    assert!(matches!(
        sample.value(),
        SampleValue::Error(InterpolationError::Computed(_))
    ));
    assert!(
        matches!(transition.sample(time(0.75), AnimationPlayState::Running).value(), SampleValue::Value(value) if *value.value() == wrapped(to))
    );
}

fn assert_operations(
    property: &str,
    from: &RepeatableList,
    to: &RepeatableList,
    midpoint: &RepeatableList,
    sum: &RepeatableList,
) {
    assert_sample(property, from, to, 0.5, midpoint);
    let operations = operations();
    let from = wrapped(from.clone());
    let to = wrapped(to.clone());
    assert_eq!(
        operations
            .interpolate(&key(property), &from, &to, p(0.5))
            .unwrap(),
        wrapped(midpoint.clone())
    );
    assert_eq!(
        operations.add(&key(property), &from, &to).unwrap(),
        wrapped(sum.clone())
    );
    assert_eq!(
        operations.accumulate(&key(property), &from, &to).unwrap(),
        wrapped(sum.clone())
    );
}
fn assert_fallback(property: &str, from: &RepeatableList, to: &RepeatableList) {
    for (progress, expected) in [
        (-1., from),
        (0., from),
        (0.5_f64.next_down(), from),
        (0.5, to),
        (1., to),
        (2., to),
    ] {
        assert_sample(property, from, to, progress, expected);
    }
    assert_eq!(from.add(to).unwrap(), *to);
    assert_eq!(from.accumulate(to).unwrap(), *to);
}
fn assert_sample(
    property: &str,
    from: &RepeatableList,
    to: &RepeatableList,
    progress: f64,
    expected: &RepeatableList,
) {
    assert_eq!(
        pair(property, from, to).sample(p(progress)),
        InterpolationOutcome::Value(wrapped(expected.clone()))
    );
}
fn operations() -> AnimationTypeOperations {
    AnimationTypeOperations::new(AnimationType::RepeatableList)
}
fn size_list(sizes: Vec<LayerSize>) -> RepeatableList {
    RepeatableList::new(
        sizes
            .into_iter()
            .map(|size| ComputedValue::single(ComputedComponent::Size(size)))
            .collect(),
    )
    .unwrap()
}
fn sizes(values: &[(f64, f64)]) -> RepeatableList {
    size_list(
        values
            .iter()
            .map(|&(x, y)| explicit(px(x), px(y)))
            .collect(),
    )
}
fn positions(values: &[(f64, f64)]) -> RepeatableList {
    RepeatableList::new(
        values
            .iter()
            .map(|&(x, y)| {
                ComputedValue::single(ComputedComponent::Position(
                    ComputedPosition::new(px(x), px(y)).unwrap(),
                ))
            })
            .collect(),
    )
    .unwrap()
}
fn tags(values: &[&str]) -> RepeatableList {
    RepeatableList::new(
        values
            .iter()
            .map(|value| {
                ComputedValue::single(ComputedComponent::Discrete(
                    DiscreteValue::new(*value).unwrap(),
                ))
            })
            .collect(),
    )
    .unwrap()
}
fn explicit(x: ComputedNumeric, y: ComputedNumeric) -> LayerSize {
    LayerSize::Explicit {
        horizontal: LayerSizeAxis::numeric(x).unwrap(),
        vertical: LayerSizeAxis::numeric(y).unwrap(),
    }
}
fn auto_height(x: f64) -> LayerSize {
    LayerSize::Explicit {
        horizontal: LayerSizeAxis::numeric(px(x)).unwrap(),
        vertical: LayerSizeAxis::auto(),
    }
}
fn auto_width(y: f64) -> LayerSize {
    LayerSize::Explicit {
        horizontal: LayerSizeAxis::auto(),
        vertical: LayerSizeAxis::numeric(px(y)).unwrap(),
    }
}
fn dimension(kind: DimensionKind, value: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(kind, value).unwrap())
}
fn px(value: f64) -> ComputedNumeric {
    dimension(DimensionKind::Length, value)
}
fn percent(value: f64) -> ComputedNumeric {
    ComputedNumeric::percentage(value).unwrap()
}
fn mixed(length: f64, percentage: f64) -> ComputedNumeric {
    ComputedNumeric::mix(
        DimensionPercentage::new(
            ComputedDimension::new(DimensionKind::Length, length).unwrap(),
            percentage,
        )
        .unwrap(),
    )
}
fn wrapped(list: RepeatableList) -> InterpolableValue {
    InterpolableValue::repeatable_list(list)
}
fn pair(property: &str, from: &RepeatableList, to: &RepeatableList) -> InterpolationPair {
    InterpolationPair::new(key(property), wrapped(from.clone()), wrapped(to.clone())).unwrap()
}
fn key(property: &str) -> PropertyKey {
    PropertyKey::new(property).unwrap()
}
fn p(progress: f64) -> InterpolationProgress {
    InterpolationProgress::new(progress).unwrap()
}
fn time(value: f64) -> ElapsedTime {
    ElapsedTime::from_secs(value).unwrap()
}
fn transition(
    property: &str,
    from: &RepeatableList,
    to: &RepeatableList,
    easing: Easing,
) -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        key(property),
        wrapped(from.clone()),
        wrapped(to.clone()),
        AnimationDuration::from_secs(1.).unwrap(),
        AnimationDelay::from_secs(0.).unwrap(),
        easing,
    )
    .unwrap()
}
