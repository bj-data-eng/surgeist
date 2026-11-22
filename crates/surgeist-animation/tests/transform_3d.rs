#![forbid(unsafe_code)]

use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationOutcome, InterpolationPair, InterpolationProgress, PropertyKey,
    TransformValue,
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
use surgeist_animation::transform::{
    CssPixels, ResolvedTransformList, RotationAxis, ScaleFactor, TransformAngle,
    TransformErrorKind, TransformFunction as F, TransformMatrix, TransformMatrix3d,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

const I: [f64; 16] = [
    1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
];
fn px(x: f64) -> CssPixels {
    CssPixels::new(x).unwrap()
}
fn sf(x: f64) -> ScaleFactor {
    ScaleFactor::new(x).unwrap()
}
fn deg(x: f64) -> TransformAngle {
    TransformAngle::from_degrees(x).unwrap()
}
fn axis(x: f64, y: f64, z: f64) -> RotationAxis {
    RotationAxis::new([x, y, z]).unwrap()
}
fn list(x: Vec<F>) -> ResolvedTransformList {
    ResolvedTransformList::new(x)
}
fn value(x: ResolvedTransformList) -> InterpolableValue {
    InterpolableValue::transform(TransformValue::resolved(x))
}
fn pair(a: ResolvedTransformList, b: ResolvedTransformList) -> InterpolationPair {
    InterpolationPair::new(PropertyKey::new("transform").unwrap(), value(a), value(b)).unwrap()
}
fn sample(a: &InterpolationPair, p: f64) -> ResolvedTransformList {
    let result = a.sample(InterpolationProgress::new(p).unwrap());
    let InterpolationOutcome::Value(InterpolableValue::Transform(value)) = result else {
        panic!("expected transform at {p}: {result:?}")
    };
    value.resolved_list().unwrap().clone()
}
fn matrix(m: [f64; 16]) -> F {
    F::Matrix3d(TransformMatrix3d::new(m).unwrap())
}
fn near(actual: &ResolvedTransformList, expected: [f64; 16]) {
    let actual = actual.to_matrix3d().unwrap().coefficients();
    for (index, (a, b)) in actual.into_iter().zip(expected).enumerate() {
        assert!(
            (a - b).abs() <= 3e-12 * b.abs().max(1.),
            "coefficient {index}: {a} != {b}"
        );
    }
}
fn rz(angle: f64, z: f64) -> [f64; 16] {
    let (s, c) = angle.to_radians().sin_cos();
    [c, s, 0., 0., -s, c, 0., 0., 0., 0., 1., 0., 0., 0., z, 1.]
}

#[test]
fn matrix_and_axis_guards_reject_nonfinite_inputs_and_keep_valid_zero_axes() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..16 {
            let mut m = I;
            m[index] = bad;
            assert!(TransformMatrix3d::new(m).is_err());
        }
        assert!(RotationAxis::new([bad, 0., 1.]).is_err());
    }
    assert_eq!(axis(0., 0., 0.).components(), [0., 0., 0.]);
    assert_eq!(axis(f64::MAX, f64::MAX, 0.), axis(1., 1., 0.));
    let spatial = list(vec![F::TranslateZ(px(1.))]);
    assert_eq!(
        spatial.to_matrix().unwrap_err().kind(),
        TransformErrorKind::NotTwoDimensional
    );
    assert_eq!(
        list(vec![matrix(I)]).to_matrix().unwrap(),
        TransformMatrix::IDENTITY
    );
}

#[test]
fn css_order_and_3d_primitives_have_independent_matrix_expectations() {
    near(
        &list(vec![
            F::Translate3d {
                x: px(10.),
                y: px(20.),
                z: px(30.),
            },
            F::Scale3d {
                x: sf(2.),
                y: sf(3.),
                z: sf(4.),
            },
        ]),
        [
            2., 0., 0., 0., 0., 3., 0., 0., 0., 0., 4., 0., 10., 20., 30., 1.,
        ],
    );
    near(
        &list(vec![F::RotateX(deg(90.))]),
        [
            1., 0., 0., 0., 0., 0., 1., 0., 0., -1., 0., 0., 0., 0., 0., 1.,
        ],
    );
    near(
        &list(vec![F::RotateY(deg(90.))]),
        [
            0., 0., -1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 0., 0., 0., 1.,
        ],
    );
    near(
        &list(vec![F::RotateZ(deg(90.))]),
        [
            0., 1., 0., 0., -1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
        ],
    );
    near(
        &list(vec![F::Rotate3d {
            axis: axis(0., 0., 0.),
            angle: deg(90.),
        }]),
        I,
    );
}

#[test]
fn promotion_preserves_same_named_3d_functions_and_common_primitives() {
    let p = pair(
        list(vec![
            F::TranslateX(px(100.)),
            F::ScaleY(sf(3.)),
            F::Rotate(deg(0.)),
        ]),
        list(vec![
            F::TranslateZ(px(100.)),
            F::ScaleZ(sf(5.)),
            F::RotateZ(deg(720.)),
        ]),
    );
    assert_eq!(
        sample(&p, 0.5).functions(),
        &[
            F::Translate3d {
                x: px(50.),
                y: px(0.),
                z: px(50.)
            },
            F::Scale3d {
                x: sf(1.),
                y: sf(2.),
                z: sf(3.)
            },
            F::Rotate3d {
                axis: axis(0., 0., 1.),
                angle: deg(360.)
            }
        ]
    );
    let p = pair(
        list(vec![F::TranslateZ(px(10.)), F::RotateX(deg(0.))]),
        list(vec![F::TranslateZ(px(30.)), F::RotateX(deg(720.))]),
    );
    assert_eq!(
        sample(&p, 0.5).functions(),
        &[F::TranslateZ(px(20.)), F::RotateX(deg(360.))]
    );
}

#[test]
fn parallel_axes_and_zero_angle_rotation_use_direct_angle_interpolation() {
    let p = pair(
        list(vec![F::Rotate3d {
            axis: axis(1., 1., 0.),
            angle: deg(0.),
        }]),
        list(vec![F::Rotate3d {
            axis: axis(4., 4., 0.),
            angle: deg(720.),
        }]),
    );
    assert_eq!(
        sample(&p, 0.5).functions(),
        &[F::Rotate3d {
            axis: axis(1., 1., 0.),
            angle: deg(360.)
        }]
    );
    let p = pair(
        list(vec![F::RotateX(deg(0.))]),
        list(vec![F::RotateY(deg(90.))]),
    );
    assert_eq!(
        sample(&p, 0.5).functions(),
        &[F::Rotate3d {
            axis: axis(0., 1., 0.),
            angle: deg(45.)
        }]
    );
}

#[test]
fn different_rotation_axes_use_one_matrix_without_collapsing_following_turns() {
    let p = pair(
        list(vec![F::RotateX(deg(90.)), F::Rotate(deg(0.))]),
        list(vec![F::RotateY(deg(90.)), F::Rotate(deg(720.))]),
    );
    let result = sample(&p, 0.5);
    assert_eq!(result.functions().len(), 2);
    assert!(matches!(result.functions()[0], F::Matrix3d(_)));
    assert_eq!(result.functions()[1], F::Rotate(deg(360.)));
    // SLERP gives q=(1,1,0,2)/sqrt(6).
    near(
        &list(vec![result.functions()[0]]),
        [
            2. / 3.,
            1. / 3.,
            -2. / 3.,
            0.,
            1. / 3.,
            2. / 3.,
            2. / 3.,
            0.,
            2. / 3.,
            -2. / 3.,
            1. / 3.,
            0.,
            0.,
            0.,
            0.,
            1.,
        ],
    );
}

#[test]
fn none_padding_uses_corresponding_spatial_identities() {
    let p = pair(
        list(vec![]),
        list(vec![
            F::Scale3d {
                x: sf(3.),
                y: sf(5.),
                z: sf(7.),
            },
            F::TranslateZ(px(20.)),
            F::Perspective(Some(px(400.))),
        ]),
    );
    let result = sample(&p, 0.5);
    assert_eq!(
        result.functions()[0],
        F::Scale3d {
            x: sf(2.),
            y: sf(3.),
            z: sf(4.)
        }
    );
    assert_eq!(result.functions()[1], F::TranslateZ(px(10.)));
    let mut expected = I;
    expected[11] = -0.00125;
    near(&list(vec![result.functions()[2]]), expected);
}

#[test]
fn positive_quarter_turn_and_mixed_sign_exact_half_turn_round_trip() {
    for original in [
        [
            1., 0., 0., 0., 0., 0., 1., 0., 0., -1., 0., 0., 0., 0., 1., 1.,
        ],
        [
            0., -1., 0., 0., -1., 0., 0., 0., 0., 0., -1., 0., 0., 0., 1., 1.,
        ],
    ] {
        near(
            &sample(
                &pair(list(vec![matrix(original)]), list(vec![matrix(original)])),
                0.5,
            ),
            original,
        );
    }
    let mut start = I;
    start[14] = 1.;
    let half = [
        0., -1., 0., 0., -1., 0., 0., 0., 0., 0., -1., 0., 0., 0., 1., 1.,
    ];
    let r = std::f64::consts::FRAC_1_SQRT_2;
    near(
        &sample(
            &pair(list(vec![matrix(start)]), list(vec![matrix(half)])),
            0.5,
        ),
        [
            0.5, -0.5, r, 0., -0.5, 0.5, r, 0., -r, -r, 0., 0., 0., 0., 1., 1.,
        ],
    );
}

#[test]
fn spatial_slerp_retains_unadjusted_arc_even_when_dot_rounds_to_negative_one() {
    for angle in [170., 180. - 1e-7] {
        let p = pair(
            list(vec![matrix(rz(angle, 1.))]),
            list(vec![matrix(rz(-angle, 3.))]),
        );
        let mut expected = I;
        expected[14] = 2.;
        near(&sample(&p, 0.5), expected);
        near(&sample(&p, 0.25), rz(angle / 2., 1.5));
    }
}

#[test]
fn matrix3d_with_only_2d_components_retains_2d_shortest_rotation_rules() {
    let p = pair(
        list(vec![matrix(rz(170., 0.))]),
        list(vec![matrix(rz(-170., 0.))]),
    );
    near(&sample(&p, 0.5), rz(180., 0.));
}

#[test]
fn positive_w_canonicalization_selects_negative_160_degree_rotation() {
    let mut start = I;
    start[14] = 1.;
    let p = pair(list(vec![matrix(start)]), list(vec![matrix(rz(200., 1.))]));
    near(&sample(&p, 0.5), rz(-80., 1.));
}

#[test]
fn close_but_distinct_rotations_do_not_freeze_when_the_dot_rounds_to_one() {
    let p = pair(
        list(vec![matrix(rz(0., 1.))]),
        list(vec![matrix(rz(1e-7, 1.))]),
    );
    near(&sample(&p, 0.5), rz(0.5e-7, 1.));
    near(&sample(&p, 0.25), rz(0.25e-7, 1.));
}

#[test]
fn exact_half_turn_dominant_axis_tie_uses_y_before_z() {
    let mut start = I;
    start[14] = 1.;
    let half = [
        -1., 0., 0., 0., 0., 0., -1., 0., 0., -1., 0., 0., 0., 0., 1., 1.,
    ];
    let r = std::f64::consts::FRAC_1_SQRT_2;
    near(
        &sample(
            &pair(list(vec![matrix(start)]), list(vec![matrix(half)])),
            0.5,
        ),
        [
            0., -r, -r, 0., r, 0.5, -0.5, 0., r, -0.5, 0.5, 0., 0., 0., 1., 1.,
        ],
    );
}

#[test]
fn perspective_identity_floor_and_extrapolation_follow_matrix_coefficients() {
    let p = pair(
        list(vec![F::Perspective(None)]),
        list(vec![F::Perspective(Some(px(400.)))]),
    );
    for (progress, coefficient) in [(0.5, -0.00125), (-0.5, 0.00125), (1.5, -0.00375)] {
        let mut expected = I;
        expected[11] = coefficient;
        near(&sample(&p, progress), expected);
    }
    for depth in [-20., 0., 0.5, 1.] {
        let mut expected = I;
        expected[11] = -1.;
        near(&list(vec![F::Perspective(Some(px(depth)))]), expected);
    }
    let p = pair(
        list(vec![F::Perspective(Some(px(100.)))]),
        list(vec![F::Perspective(Some(px(400.)))]),
    );
    let mut expected = I;
    expected[11] = -0.00625;
    near(&sample(&p, 0.5), expected);
}

#[test]
fn spatial_decomposition_restores_perspective_skew_scale_and_translation() {
    // Independently expanded P(100) * T(3,4,5) * shearXY(0.5) * S(2,3,4).
    let original = [
        2., 0., 0., 0., 1.5, 3., 0., 0., 0., 0., 4., -0.04, 3., 4., 5., 0.95,
    ];
    let expected = original.map(|x| x / 0.95);
    near(
        &sample(
            &pair(list(vec![matrix(original)]), list(vec![matrix(original)])),
            0.5,
        ),
        expected,
    );
    // P(100) * T(3,4,5) * Rz(90) * shearYZ(1) * shearXZ(2) * shearXY(3) * S(2,3,4).
    let rotated = [
        0., 2., 0., 0., -3., 9., 0., 0., -4., 8., 4., -0.04, 3., 4., 5., 0.95,
    ];
    near(
        &sample(
            &pair(list(vec![matrix(rotated)]), list(vec![matrix(rotated)])),
            0.5,
        ),
        rotated.map(|x| x / 0.95),
    );
    let projection = [
        1., 0., 0., 0.01, 0., 1., 0., 0.02, 0., 0., 1., -0.01, 3., 4., 5., 1.06,
    ];
    near(
        &sample(
            &pair(
                list(vec![matrix(projection)]),
                list(vec![matrix(projection)]),
            ),
            0.5,
        ),
        projection.map(|x| x / 1.06),
    );
}

#[test]
fn full_matrix_singularity_and_singular_perspective_base_select_discrete_values() {
    for bad in [
        [
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 1., 0., 0., 1., 1.,
        ],
        [
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0., 0., 1., 1.,
        ],
        [
            1e200, 1e200, 0., 0., 1e200, 1e200, 0., 0., 0., 0., 1e200, 0., 0., 0., 1., 1.,
        ],
    ] {
        let a = list(vec![matrix(bad)]);
        let b = list(vec![matrix(I)]);
        let p = pair(a.clone(), b.clone());
        assert_eq!(sample(&p, 0.25), a);
        assert_eq!(sample(&p, 0.75), b);
    }
}

#[test]
fn failed_spatial_decomposition_falls_back_without_rejecting_singular_samples() {
    let mut bad = I;
    bad[10] = 0.;
    bad[14] = 1.;
    let from = list(vec![
        matrix(bad),
        F::ScaleX(sf(f64::MAX)),
        F::ScaleX(sf(2.)),
    ]);
    let to = list(vec![
        matrix(I),
        F::TranslateX(px(1.)),
        F::TranslateX(px(1.)),
    ]);
    let p = pair(from.clone(), to.clone());
    assert_eq!(sample(&p, 0.25), from);
    assert_eq!(sample(&p, 0.5), to);
    let p = pair(
        list(vec![F::Scale3d {
            x: sf(1.),
            y: sf(1.),
            z: sf(1.),
        }]),
        list(vec![F::Scale3d {
            x: sf(1.),
            y: sf(1.),
            z: sf(-1.),
        }]),
    );
    let mut expected = I;
    expected[10] = 0.;
    near(&sample(&p, 0.5), expected);
    let mut no_normalization = I;
    no_normalization[15] = 0.;
    let a = list(vec![matrix(no_normalization)]);
    let b = list(vec![matrix(I)]);
    assert_eq!(sample(&pair(a.clone(), b), 0.25), a);

    let mut start = I;
    start[14] = 1.;
    let mut end = start;
    end[10] = -1.;
    let mut collapsed = [0.; 16];
    collapsed[14] = 1.;
    collapsed[15] = 1.;
    near(
        &sample(
            &pair(list(vec![matrix(start)]), list(vec![matrix(end)])),
            0.5,
        ),
        collapsed,
    );
}

#[test]
fn tiny_spatial_scales_remain_invertible_and_arithmetic_failure_recovers() {
    for magnitude in [1e-200, 1e200] {
        let mut a = I;
        a[0] = magnitude;
        a[5] = magnitude;
        a[10] = magnitude;
        a[14] = 1.;
        let mut b = a;
        b[14] = 3.;
        let mut expected = a;
        expected[14] = 2.;
        near(
            &sample(&pair(list(vec![matrix(a)]), list(vec![matrix(b)])), 0.5),
            expected,
        );
    }
    let a = list(vec![F::TranslateZ(px(-f64::MAX))]);
    let b = list(vec![F::TranslateZ(px(f64::MAX))]);
    let p = pair(a.clone(), b.clone());
    assert!(matches!(
        p.sample(InterpolationProgress::new(0.5).unwrap()),
        InterpolationOutcome::ArithmeticFailure(_)
    ));
    assert_eq!(sample(&p, 0.), a);
    assert_eq!(sample(&p, 1.), b);
}

#[test]
fn direct_accumulation_is_additive_with_one_based_scales_and_padding() {
    let a = list(vec![
        F::ScaleX(sf(2.)),
        F::TranslateZ(px(3.)),
        F::RotateY(deg(90.)),
    ]);
    let b = list(vec![F::ScaleX(sf(3.)), F::TranslateZ(px(4.))]);
    assert_eq!(
        a.accumulated_with(&b).unwrap().functions(),
        &[
            F::ScaleX(sf(4.)),
            F::TranslateZ(px(7.)),
            F::RotateY(deg(90.))
        ]
    );
    let a = list(vec![F::Scale3d {
        x: sf(2.),
        y: sf(3.),
        z: sf(4.),
    }]);
    let b = list(vec![F::Scale3d {
        x: sf(3.),
        y: sf(4.),
        z: sf(5.),
    }]);
    assert_eq!(
        a.accumulated_with(&b).unwrap().functions(),
        &[F::Scale3d {
            x: sf(4.),
            y: sf(6.),
            z: sf(8.)
        }]
    );
    let maximum = list(vec![F::ScaleX(sf(f64::MAX))]);
    assert_eq!(
        maximum
            .accumulated_with(&list(vec![F::ScaleX(sf(1.))]))
            .unwrap(),
        maximum
    );
    let tiny = list(vec![F::ScaleX(sf(f64::from_bits(1)))]);
    assert_eq!(
        tiny.accumulated_with(&list(vec![F::ScaleX(sf(1.))]))
            .unwrap(),
        tiny
    );
    assert_eq!(
        maximum
            .accumulated_with(&list(vec![F::ScaleX(sf(-f64::MAX))]))
            .unwrap()
            .functions(),
        &[F::ScaleX(sf(-1.))]
    );
}

#[test]
fn spatial_accumulation_reports_arithmetic_and_preserves_immutable_operands() {
    let underlying = list(vec![F::TranslateZ(px(f64::MAX))]);
    let effect = list(vec![F::TranslateZ(px(f64::MAX))]);
    let error = underlying.accumulated_with(&effect).unwrap_err();
    assert_eq!(error.kind(), TransformErrorKind::NonFinite);
    assert_eq!(underlying.functions(), &[F::TranslateZ(px(f64::MAX))]);
    let source = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 2., 1.,
    ];
    let original = TransformMatrix3d::new(source).unwrap();
    let mut changed = source;
    changed[14] = 200.;
    assert_eq!(original.coefficients()[14], 2.);
    assert_eq!(changed[14], 200.);
    assert_eq!(
        TransformValue::resolved(list(vec![F::Matrix3d(original)])).kind(),
        "resolved-3d"
    );
    let marker = TransformValue::unsupported("resolved-3d");
    assert!(marker.resolved_list().is_none());
    let p = InterpolationPair::new(
        PropertyKey::new("transform").unwrap(),
        InterpolableValue::transform(marker.clone()),
        InterpolableValue::transform(marker),
    )
    .unwrap();
    assert!(matches!(
        p.sample(InterpolationProgress::new(0.5).unwrap()),
        InterpolationOutcome::Unsupported(_)
    ));
}

#[test]
fn decomposed_accumulation_uses_underlying_times_effect_quaternion_order() {
    let x = [
        1., 0., 0., 0., 0., 0., 1., 0., 0., -1., 0., 0., 0., 0., 1., 1.,
    ];
    let y = [
        0., 0., -1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 0., 0., 2., 1.,
    ];
    let result = list(vec![matrix(x)])
        .accumulated_with(&list(vec![matrix(y)]))
        .unwrap();
    near(
        &result,
        [
            0., 1., 0., 0., 0., 0., 1., 0., 1., 0., 0., 0., 0., 0., 3., 1.,
        ],
    );
    let reversed = list(vec![matrix(y)])
        .accumulated_with(&list(vec![matrix(x)]))
        .unwrap();
    near(
        &reversed,
        [
            0., 0., -1., 0., 1., 0., 0., 0., 0., -1., 0., 0., 0., 0., 3., 1.,
        ],
    );
}

#[test]
fn affine_accumulation_keeps_residual_identity_and_adds_translation() {
    let a = list(vec![F::Matrix(
        TransformMatrix::new([2., 0., 0., 3., 10., 20.]).unwrap(),
    )]);
    let b = list(vec![F::Matrix(
        TransformMatrix::new([3., 0., 0., 4., 30., 40.]).unwrap(),
    )]);
    assert_eq!(
        a.accumulated_with(&b)
            .unwrap()
            .to_matrix()
            .unwrap()
            .coefficients(),
        [4., 0., 0., 6., 40., 60.]
    );
}

#[test]
fn additive_neutral_values_are_distinct_from_padding_identities() {
    assert_eq!(F::ScaleZ(sf(5.)).neutral_for_addition(), F::ScaleZ(sf(0.)));
    assert_eq!(
        F::Perspective(Some(px(20.))).neutral_for_addition(),
        F::Perspective(None)
    );
    assert_eq!(matrix(I).neutral_for_addition(), matrix([0.; 16]));
    let effect = list(vec![F::Perspective(Some(px(400.)))]);
    near(
        &list(vec![F::Perspective(None)])
            .accumulated_with(&effect)
            .unwrap(),
        effect.to_matrix3d().unwrap().coefficients(),
    );
    assert_eq!(
        list(vec![matrix([0.; 16])])
            .accumulated_with(&effect)
            .unwrap(),
        effect
    );
}

#[test]
fn actual_transition_and_keyframe_paths_sample_spatial_values() {
    let from = value(list(vec![matrix(rz(170., 1.))]));
    let to = value(list(vec![matrix(rz(-170., 3.))]));
    let property = PropertyKey::new("transform").unwrap();
    let delay = AnimationDelay::from_secs(0.).unwrap();
    let duration = AnimationDuration::from_secs(2.).unwrap();
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
    let frames = KeyframeTrack::new(
        KeyframeAnimationId::new("spatial").unwrap(),
        timing,
        vec![segment],
    )
    .unwrap();
    let elapsed = ElapsedTime::from_secs(1.).unwrap();
    for sample in [
        transition.sample(elapsed, AnimationPlayState::Running),
        frames
            .sample(elapsed, AnimationPlayState::Running)
            .remove(0),
    ] {
        assert_eq!(sample.classification(), SampleClassification::Active);
        let SampleValue::Value(value) = sample.value() else {
            panic!("expected sampled value")
        };
        let InterpolableValue::Transform(value) = value.value() else {
            panic!("expected transform")
        };
        let mut expected = I;
        expected[14] = 2.;
        near(value.resolved_list().unwrap(), expected);
    }
}
