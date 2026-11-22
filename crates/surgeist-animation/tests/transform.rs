#![forbid(unsafe_code)]

use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationComponent, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey, TransformValue, ValueFamily,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{NextFrameHint, SampleClassification, SampleValue};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transform::{
    CssPixels, ResolvedTransformList, ScaleFactor, TransformAngle, TransformComponent,
    TransformFunction, TransformMatrix,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

fn px(value: f64) -> CssPixels {
    CssPixels::new(value).unwrap()
}
fn scale(value: f64) -> ScaleFactor {
    ScaleFactor::new(value).unwrap()
}
fn deg(value: f64) -> TransformAngle {
    TransformAngle::from_degrees(value).unwrap()
}
fn list(functions: Vec<TransformFunction>) -> ResolvedTransformList {
    ResolvedTransformList::new(functions)
}
fn matrix(values: [f64; 6]) -> TransformFunction {
    TransformFunction::Matrix(TransformMatrix::new(values).unwrap())
}
fn translate(x: f64, y: f64) -> TransformFunction {
    TransformFunction::Translate {
        x: px(x),
        y: Some(px(y)),
    }
}
fn scaling(x: f64, y: f64) -> TransformFunction {
    TransformFunction::Scale {
        x: scale(x),
        y: Some(scale(y)),
    }
}
fn value(list: ResolvedTransformList) -> InterpolableValue {
    InterpolableValue::transform(TransformValue::resolved(list))
}
fn make_pair(from: ResolvedTransformList, to: ResolvedTransformList) -> InterpolationPair {
    InterpolationPair::new(
        PropertyKey::new("transform").unwrap(),
        value(from),
        value(to),
    )
    .unwrap()
}
fn sampled(pair: &InterpolationPair, progress: f64) -> ResolvedTransformList {
    let outcome = pair.sample(InterpolationProgress::new(progress).unwrap());
    let InterpolationOutcome::Value(InterpolableValue::Transform(value)) = outcome else {
        panic!("expected resolved transform at {progress}, got {outcome:?}");
    };
    value.resolved_list().unwrap().clone()
}
fn assert_matrix(list: &ResolvedTransformList, expected: [f64; 6]) {
    let actual = list.to_matrix().unwrap().coefficients();
    for (index, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() <= 2e-12 * expected.abs().max(1.0),
            "coefficient {index}: {actual} != {expected}"
        );
    }
}

#[test]
fn scalar_constructors_reject_nonfinite_and_angle_conversion_overflow() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(CssPixels::new(value).is_err());
        assert!(ScaleFactor::new(value).is_err());
        assert!(TransformAngle::from_radians(value).is_err());
        assert!(TransformAngle::from_degrees(value).is_err());
        assert!(TransformAngle::from_turns(value).is_err());
        assert!(TransformMatrix::new([1., 0., 0., 1., value, 0.]).is_err());
    }
    assert!(TransformAngle::from_turns(f64::MAX).is_err());
    assert!(TransformAngle::from_degrees(f64::MAX).is_ok());
    assert_eq!(px(-3.).value(), -3.);
    assert_eq!(scale(-2.).value(), -2.);
    assert_eq!(deg(180.).radians(), std::f64::consts::PI);
    assert_eq!(TransformAngle::from_turns(0.5).unwrap(), deg(180.));
}

#[test]
fn none_stays_empty_and_identity_padding_preserves_arity() {
    assert!(
        sampled(&make_pair(list(vec![]), list(vec![])), 0.5)
            .functions()
            .is_empty()
    );
    let to = TransformFunction::Scale {
        x: scale(3.),
        y: None,
    };
    assert_eq!(
        sampled(&make_pair(list(vec![]), list(vec![to])), 0.5).functions(),
        &[TransformFunction::Scale {
            x: scale(2.),
            y: None
        }]
    );
    let pair = make_pair(
        list(vec![scaling(1., 1.)]),
        list(vec![scaling(3., 5.), TransformFunction::Rotate(deg(90.))]),
    );
    assert_eq!(
        sampled(&pair, 0.5).functions(),
        &[scaling(2., 3.), TransformFunction::Rotate(deg(45.))]
    );
}

#[test]
fn translate_and_scale_derivatives_use_common_two_dimensional_primitives() {
    let pair = make_pair(
        list(vec![
            TransformFunction::TranslateX(px(100.)),
            TransformFunction::ScaleX(scale(3.)),
        ]),
        list(vec![
            TransformFunction::TranslateY(px(100.)),
            TransformFunction::ScaleY(scale(5.)),
        ]),
    );
    assert_eq!(
        sampled(&pair, 0.5).functions(),
        &[translate(50., 50.), scaling(2., 3.)]
    );
}

#[test]
fn omitted_scale_and_translate_arguments_expand_with_their_semantic_defaults() {
    let pair = make_pair(
        list(vec![
            TransformFunction::Translate {
                x: px(20.),
                y: None,
            },
            TransformFunction::Scale {
                x: scale(2.),
                y: None,
            },
        ]),
        list(vec![translate(40., 10.), scaling(4., 6.)]),
    );
    assert_eq!(
        sampled(&pair, 0.5).functions(),
        &[translate(30., 5.), scaling(3., 4.)]
    );
}

#[test]
fn direct_rotation_preserves_turns_and_unbounded_progress() {
    let pair = make_pair(
        list(vec![TransformFunction::Rotate(deg(0.))]),
        list(vec![TransformFunction::Rotate(deg(720.))]),
    );
    assert_eq!(
        sampled(&pair, 0.5).functions(),
        &[TransformFunction::Rotate(deg(360.))]
    );
    assert_eq!(
        sampled(&pair, 1.5).functions(),
        &[TransformFunction::Rotate(deg(1080.))]
    );
    assert_eq!(
        sampled(&pair, -0.5).functions(),
        &[TransformFunction::Rotate(deg(-360.))]
    );
}

#[test]
fn matched_rotation_prefix_survives_suffix_matrix_interpolation() {
    let pair = make_pair(
        list(vec![
            TransformFunction::Rotate(deg(0.)),
            scaling(1., 1.),
            translate(20., 0.),
        ]),
        list(vec![
            TransformFunction::Rotate(deg(270.)),
            translate(10., 0.),
            scaling(2., 2.),
        ]),
    );
    let result = sampled(&pair, 0.5);
    assert_eq!(result.functions().len(), 2);
    assert_eq!(result.functions()[0], TransformFunction::Rotate(deg(135.)));
    let TransformFunction::Matrix(suffix) = result.functions()[1] else {
        panic!("expected matrix suffix");
    };
    assert_eq!(suffix.coefficients(), [1.5, 0., 0., 1.5, 15., 0.]);
}

#[test]
fn list_composition_uses_css_postmultiplication_and_addition_retains_order() {
    let a = list(vec![translate(10., 20.)]);
    let b = list(vec![scaling(2., 3.)]);
    let combined = a.added_to(&b);
    assert_eq!(
        combined.functions(),
        &[translate(10., 20.), scaling(2., 3.)]
    );
    // Applying the matrix to (1,1) yields (12,23), independently of decomposition.
    assert_matrix(&combined, [2., 0., 0., 3., 10., 20.]);
    assert_matrix(&b.added_to(&a), [2., 0., 0., 3., 20., 60.]);
    assert_eq!(a.functions(), &[translate(10., 20.)]);
    assert_eq!(b.functions(), &[scaling(2., 3.)]);
    assert_eq!(a.added_to(&list(vec![])), a);
}

#[test]
fn shear_translation_is_preserved_for_identical_and_different_endpoints() {
    // #552's selected source repair: affine translation is independent of residual shear.
    let original = list(vec![matrix([1., 0., 1., 1., 10., 20.])]);
    assert_matrix(
        &sampled(&make_pair(original.clone(), original.clone()), 0.5),
        [1., 0., 1., 1., 10., 20.],
    );
    assert_matrix(
        &sampled(
            &make_pair(original, list(vec![matrix([1., 0., 1., 1., 30., 40.])])),
            0.5,
        ),
        [1., 0., 1., 1., 20., 30.],
    );
}

#[test]
fn matrix_interpolation_takes_short_rotation_and_normalizes_flipped_axes() {
    let (sin, cos) = 170_f64.to_radians().sin_cos();
    let pair = make_pair(
        list(vec![matrix([cos, sin, -sin, cos, 0., 0.])]),
        list(vec![matrix([cos, -sin, sin, cos, 0., 0.])]),
    );
    assert_matrix(&sampled(&pair, 0.5), [-1., 0., 0., -1., 0., 0.]);
    let reflected = make_pair(
        list(vec![matrix([-1., 0., 0., 1., 0., 0.])]),
        list(vec![matrix([1., 0., 0., -1., 0., 0.])]),
    );
    assert_matrix(&sampled(&reflected, 0.5), [0., -1., -1., 0., 0., 0.]);
}

#[test]
fn skew_functions_preserve_arguments_and_distinct_axes_use_matrix_fallback() {
    let pair = make_pair(
        list(vec![TransformFunction::Skew {
            x: deg(0.),
            y: None,
        }]),
        list(vec![TransformFunction::Skew {
            x: deg(60.),
            y: Some(deg(30.)),
        }]),
    );
    assert_eq!(
        sampled(&pair, 0.5).functions(),
        &[TransformFunction::Skew {
            x: deg(30.),
            y: Some(deg(15.))
        }]
    );
    let axes = make_pair(
        list(vec![TransformFunction::SkewX(deg(45.))]),
        list(vec![TransformFunction::SkewY(deg(45.))]),
    );
    assert!(matches!(
        sampled(&axes, 0.5).functions(),
        [TransformFunction::Matrix(_)]
    ));
    assert_matrix(
        &list(vec![TransformFunction::Skew {
            x: deg(45.),
            y: Some(deg(45.)),
        }]),
        [1., 1., 1., 1., 0., 0.],
    );
}

#[test]
fn singular_endpoint_falls_back_for_the_whole_value_and_switches_at_half() {
    let from = list(vec![
        TransformFunction::Rotate(deg(10.)),
        matrix([0., 0., 0., 1., 4., 5.]),
    ]);
    let to = list(vec![
        TransformFunction::Rotate(deg(100.)),
        matrix([1., 0., 0., 1., 8., 9.]),
    ]);
    let pair = make_pair(from.clone(), to.clone());
    assert_eq!(sampled(&pair, 0.499), from);
    assert_eq!(sampled(&pair, 0.5), to);
}

#[test]
fn later_singular_matrix_selects_whole_value_before_prefix_arithmetic() {
    let from = list(vec![
        matrix([f64::MAX, f64::MAX, 0., 1., 0., 0.]),
        matrix([0., 0., 0., 1., 0., 0.]),
    ]);
    let to = list(vec![
        matrix([1., 0., 0., 1., 0., 0.]),
        matrix([1., 0., 0., 1., 10., 20.]),
    ]);
    let pair = make_pair(from.clone(), to.clone());
    assert_eq!(sampled(&pair, 0.25), from);
    assert_eq!(sampled(&pair, 0.75), to);
}

#[test]
fn singular_matrix_fallback_precedes_overflowing_matched_rotation() {
    let from = list(vec![
        TransformFunction::Rotate(TransformAngle::from_radians(-f64::MAX).unwrap()),
        matrix([0., 0., 0., 1., 0., 0.]),
    ]);
    let to = list(vec![
        TransformFunction::Rotate(TransformAngle::from_radians(f64::MAX).unwrap()),
        matrix([1., 0., 0., 1., 10., 20.]),
    ]);
    let pair = make_pair(from.clone(), to.clone());
    assert_eq!(sampled(&pair, 0.25), from);
    assert_eq!(sampled(&pair, 0.75), to);
}

#[test]
fn matching_matrix_pairs_keep_following_function_interpolation() {
    let pair = make_pair(
        list(vec![
            matrix([1., 0., 0., 1., 10., 20.]),
            TransformFunction::Rotate(deg(0.)),
        ]),
        list(vec![
            matrix([1., 0., 0., 1., 30., 40.]),
            TransformFunction::Rotate(deg(720.)),
        ]),
    );
    let result = sampled(&pair, 0.5);
    assert_eq!(
        result.functions(),
        &[
            matrix([1., 0., 0., 1., 20., 30.]),
            TransformFunction::Rotate(deg(360.))
        ]
    );
}

#[test]
fn exact_half_turn_matrix_rotation_tie_preserves_the_pinned_direction() {
    let pair = make_pair(
        list(vec![matrix([0., 1., -1., 0., 0., 0.])]),
        list(vec![matrix([0., -1., 1., 0., 0., 0.])]),
    );
    assert_matrix(&sampled(&pair, 0.5), [1., 0., 0., 1., 0., 0.]);
}

#[test]
fn every_2d_function_accepts_matching_identity_padding_in_both_directions() {
    for function in [
        matrix([2., 0., 0., 3., 10., 20.]),
        TransformFunction::Translate {
            x: px(10.),
            y: None,
        },
        TransformFunction::TranslateX(px(10.)),
        TransformFunction::TranslateY(px(10.)),
        TransformFunction::Scale {
            x: scale(2.),
            y: None,
        },
        TransformFunction::ScaleX(scale(2.)),
        TransformFunction::ScaleY(scale(2.)),
        TransformFunction::Rotate(deg(30.)),
        TransformFunction::Skew {
            x: deg(30.),
            y: None,
        },
        TransformFunction::SkewX(deg(30.)),
        TransformFunction::SkewY(deg(30.)),
    ] {
        let left = sampled(&make_pair(list(vec![]), list(vec![function])), 0.5);
        let right = sampled(&make_pair(list(vec![function]), list(vec![])), 0.5);
        assert_eq!(left, right);
        assert_eq!(left.functions().len(), 1);
        assert!(left.to_matrix().is_ok());
    }
}

#[test]
fn primitive_and_matrix_interpolation_preserve_valid_singular_samples() {
    let primitives = make_pair(list(vec![scaling(1., 1.)]), list(vec![scaling(-1., 1.)]));
    assert_matrix(&sampled(&primitives, 0.5), [0., 0., 0., 1., 0., 0.]);
    let matrices = make_pair(
        list(vec![matrix([1., 0., 0., 1., 0., 0.])]),
        list(vec![matrix([-1., 0., 0., 1., 0., 0.])]),
    );
    assert_matrix(&sampled(&matrices, 0.5), [0., 0., 0., 1., 0., 0.]);
}

#[test]
fn finite_large_and_subnormal_scales_are_not_misclassified_as_singular() {
    for factor in [1e308, 1e-308, f64::from_bits(1)] {
        let from = list(vec![matrix([factor, 0., 0., factor, 0., 0.])]);
        let to = list(vec![matrix([factor, 0., 0., factor, 10., 20.])]);
        let sampled = sampled(&make_pair(from, to), 0.5);
        assert_eq!(
            sampled.to_matrix().unwrap().coefficients(),
            [factor, 0., 0., factor, 5., 10.]
        );
    }
}

#[test]
fn subnormal_non_axis_aligned_columns_preserve_the_other_axis() {
    let tiny = f64::from_bits(2);
    let original = [tiny, tiny, 0., 1., 0., 0.];
    let pair = make_pair(list(vec![matrix(original)]), list(vec![matrix(original)]));
    let result = sampled(&pair, 0.5).to_matrix().unwrap().coefficients();
    assert_eq!(result[0], tiny);
    assert_eq!(result[1], tiny);
    assert!((result[2]).abs() <= 2e-12);
    assert!(
        (result[3] - 1.).abs() <= 2e-12,
        "other axis changed: {result:?}"
    );
}

#[test]
fn non_axis_aligned_normalization_preserves_tiny_and_large_unequal_endpoints() {
    for (factor, multiplier) in [
        (f64::from_bits(2), 2.),
        (-f64::from_bits(2), 2.),
        (f64::MIN_POSITIVE / 2., 2.),
        (f64::MIN_POSITIVE, 2.),
        (1e-300, 2.),
        (f64::MAX / 8., 2.),
        (-f64::MAX / 8., 2.),
        (f64::MAX * 0.7, 1.),
        (-f64::MAX * 0.7, 1.),
    ] {
        // The endpoint linear parts have identical column directions. Their
        // independent midpoint scales each column and translation arithmetically.
        let from = [factor, factor, 1., 2., 10., 20.];
        let to = [factor * multiplier, factor * multiplier, 3., 6., 30., 40.];
        let pair = make_pair(list(vec![matrix(from)]), list(vec![matrix(to)]));
        let result = sampled(&pair, 0.5).to_matrix().unwrap().coefficients();
        let first_column = factor * (0.5 + multiplier * 0.5);
        let tolerance = (first_column.abs() * 2e-12).max(f64::from_bits(1));
        for actual in [result[0], result[1]] {
            assert!(
                (actual - first_column).abs() <= tolerance,
                "first column at scale {factor}: {result:?}"
            );
        }
        assert_matrix(
            &sampled(&pair, 0.5),
            [first_column, first_column, 2., 4., 20., 30.],
        );
    }
}

#[test]
fn singular_compatible_prefix_precedes_unmatched_suffix_overflow() {
    let from = list(vec![
        matrix([0., 0., 0., 1., 0., 0.]),
        TransformFunction::ScaleX(scale(f64::MAX)),
        TransformFunction::ScaleX(scale(2.)),
    ]);
    let to = list(vec![
        matrix([1., 0., 0., 1., 0., 0.]),
        TransformFunction::TranslateX(px(1.)),
        TransformFunction::TranslateX(px(1.)),
    ]);
    let pair = make_pair(from.clone(), to.clone());
    assert_eq!(sampled(&pair, 0.25), from);
    assert_eq!(sampled(&pair, 0.5), to);
    assert_eq!(sampled(&pair, 0.75), to);
}

#[test]
fn singularity_check_handles_dependent_columns_with_overflowing_products() {
    let from = list(vec![matrix([1e308, 1e308, 1e308, 1e308, 0., 0.])]);
    let to = list(vec![matrix([1., 0., 0., 1., 10., 20.])]);
    let pair = make_pair(from.clone(), to.clone());
    assert_eq!(sampled(&pair, 0.25), from);
    assert_eq!(sampled(&pair, 0.75), to);
}

#[test]
fn arithmetic_failure_retains_property_progress_and_recovers_at_endpoints() {
    let from = list(vec![TransformFunction::TranslateX(px(-f64::MAX))]);
    let to = list(vec![TransformFunction::TranslateX(px(f64::MAX))]);
    let pair = make_pair(from.clone(), to.clone());
    let InterpolationOutcome::ArithmeticFailure(error) =
        pair.sample(InterpolationProgress::new(0.5).unwrap())
    else {
        panic!("expected arithmetic failure");
    };
    assert_eq!(error.family(), ValueFamily::Transform);
    assert_eq!(error.property().as_str(), "transform");
    assert_eq!(error.progress().value(), 0.5);
    assert!(matches!(
        error.component(),
        InterpolationComponent::Transform {
            function_index: Some(0),
            component: TransformComponent::TranslationX
        }
    ));
    assert!(!error.value().is_finite());
    assert_eq!(sampled(&pair, 0.), from);
    assert_eq!(sampled(&pair, 1.), to);
    let overflow = list(vec![scaling(f64::MAX, 1.), scaling(2., 1.)]);
    assert!(overflow.to_matrix().is_err());
}

#[test]
fn resolved_lists_and_sampled_values_cannot_be_mutated_through_source_buffers() {
    let mut source = vec![translate(1., 2.)];
    let owned = list(source.clone());
    source[0] = translate(500., 600.);
    let pair = make_pair(owned.clone(), list(vec![translate(3., 4.)]));
    let output = sampled(&pair, 0.5);
    assert_eq!(owned.functions(), &[translate(1., 2.)]);
    assert_eq!(output.functions(), &[translate(2., 3.)]);
    assert_eq!(pair.from(), &value(owned));
    assert_eq!(sampled(&pair, 0.5), output);
}

#[test]
fn marker_endpoints_remain_unsupported_including_mixed_resolved_and_marker_values() {
    let marker = InterpolableValue::transform(TransformValue::unsupported("3D"));
    assert!(TransformValue::unsupported("3D").resolved_list().is_none());
    for to in [marker.clone(), value(list(vec![]))] {
        let pair =
            InterpolationPair::new(PropertyKey::new("transform").unwrap(), marker.clone(), to)
                .unwrap();
        assert!(matches!(
            pair.sample(InterpolationProgress::new(0.).unwrap()),
            InterpolationOutcome::Unsupported(_)
        ));
    }
}

#[test]
fn transition_and_keyframe_samplers_use_the_resolved_transform_path() {
    let from = value(list(vec![translate(10., 20.)]));
    let to = value(list(vec![translate(30., 40.)]));
    let property = PropertyKey::new("transform").unwrap();
    let duration = AnimationDuration::from_secs(2.).unwrap();
    let delay = AnimationDelay::from_secs(0.).unwrap();
    let transition = TransitionTrack::new(
        TransitionTrackId::new(1),
        property.clone(),
        from.clone(),
        to.clone(),
        duration,
        delay,
        Easing::linear(),
    )
    .unwrap();
    let timing = KeyframeTiming::try_new(
        delay,
        duration,
        IterationCount::finite(1.).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    let segment = KeyframeSegment::new(
        property,
        UnitRatio::new(0.).unwrap(),
        UnitRatio::new(1.).unwrap(),
        from,
        to,
        Easing::linear(),
    )
    .unwrap();
    let keyframes = KeyframeTrack::new(
        KeyframeAnimationId::new("move").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();
    let elapsed = ElapsedTime::from_secs(1.).unwrap();
    for sample in [
        transition.sample(elapsed, AnimationPlayState::Running),
        keyframes
            .sample(elapsed, AnimationPlayState::Running)
            .remove(0),
    ] {
        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_eq!(sample.next_frame(), NextFrameHint::MayChange);
        let SampleValue::Value(property) = sample.value() else {
            panic!("expected sampled property");
        };
        let InterpolableValue::Transform(transform) = property.value() else {
            panic!("expected transform");
        };
        assert_matrix(
            transform.resolved_list().unwrap(),
            [1., 0., 0., 1., 20., 30.],
        );
    }
}
