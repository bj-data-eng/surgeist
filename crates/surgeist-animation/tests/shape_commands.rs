#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedDimension, ComputedErrorKind, ComputedNumeric, ComputedPosition, DimensionKind,
    DimensionPercentage, MathContext, MathNode, NumericType,
};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue};
use surgeist_animation::shape::{
    ArcCommand, ArcSize, ArcSweep, AxisCommand, BasicShape, CommandComponent, CommandPath,
    CommandShape, ControlAnchor, ControlPoint, CurveControls, Endpoint, FillRule, LengthPercentage,
    ReferenceBox, ShapeCommand, ShapeComponent, ShapeErrorKind, ShapeValue,
};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

#[test]
fn shape_requires_a_command_but_path_allows_only_its_initial_move() {
    assert_eq!(
        CommandShape::new(point(0., 0.), vec![], FillRule::Nonzero)
            .unwrap_err()
            .kind(),
        ShapeErrorKind::EmptyCommands
    );
    assert!(CommandPath::new(point(0., 0.), vec![], FillRule::Nonzero).is_ok());
}

#[test]
fn path_admission_rejects_percentages_and_non_svg_control_anchors() {
    let percent_point =
        ComputedPosition::new(ComputedNumeric::percentage(0.5).unwrap(), px(0.)).unwrap();
    let error = CommandPath::new(percent_point.clone(), vec![], FillRule::Nonzero).unwrap_err();
    assert_eq!(error.kind(), ShapeErrorKind::InvalidNumericType);
    assert_eq!(error.component(), Some(ShapeComponent::InitialPoint));
    let invalid = [
        ShapeCommand::Line(Endpoint::To(percent_point)),
        ShapeCommand::HorizontalLine(AxisCommand::By(
            LengthPercentage::new(ComputedNumeric::percentage(0.5).unwrap()).unwrap(),
        )),
        ShapeCommand::Curve {
            end: Endpoint::To(point(10., 20.)),
            controls: CurveControls::Quadratic(control(ControlAnchor::Start, 1., 2.)),
        },
        ShapeCommand::Smooth {
            end: Endpoint::By(point(10., 20.)),
            control: Some(control(ControlAnchor::End, 1., 2.)),
        },
        arc(
            Endpoint::To(point(1., 2.)),
            LengthPercentage::new(ComputedNumeric::percentage(0.5).unwrap()).unwrap(),
            lp(2.),
            0.,
            ArcSweep::Ccw,
            ArcSize::Small,
        ),
    ];
    for command in invalid {
        assert!(CommandShape::new(point(0., 0.), vec![command.clone()], FillRule::Nonzero).is_ok());
        assert!(CommandPath::new(point(0., 0.), vec![command], FillRule::Nonzero).is_err());
    }
    let error = ArcCommand::new(
        Endpoint::To(point(1., 2.)),
        lp(1.),
        lp(2.),
        px(3.),
        ArcSweep::Ccw,
        ArcSize::Small,
    )
    .unwrap_err();
    assert_eq!(error.kind(), ShapeErrorKind::InvalidNumericType);
    assert_eq!(
        error.actual_type(),
        Some(NumericType::Dimension(DimensionKind::Length))
    );
}

#[test]
fn every_command_interpolates_without_erasing_its_kind_form_or_implicit_controls() {
    for path_mode in [false, true] {
        let from = list(path_mode, point(0., 0.), commands(0.));
        let to = list(path_mode, point(20., 40.), commands(20.));
        assert_eq!(
            sample(&from, &to, 0.5),
            list(path_mode, point(10., 20.), commands(10.))
        );
        assert_eq!(sample(&from, &to, 0.), from);
        assert_eq!(sample(&from, &to, 1.), to);
    }
}

#[test]
fn compatible_mixed_sources_produce_shape_at_both_endpoints_and_inside() {
    let path = list(
        true,
        point(5., 5.),
        vec![
            ShapeCommand::HorizontalLine(AxisCommand::To(lp(5.))),
            ShapeCommand::VerticalLine(AxisCommand::To(lp(-5.))),
            ShapeCommand::Close,
        ],
    );
    let shape = list(
        false,
        point(15., 15.),
        vec![
            ShapeCommand::HorizontalLine(AxisCommand::To(lp(25.))),
            ShapeCommand::VerticalLine(AxisCommand::To(lp(-15.))),
            ShapeCommand::Close,
        ],
    );
    for (from, to) in [(&path, &shape), (&shape, &path)] {
        for p in [0., 0.5, 1.] {
            let actual = sample(from, to, p);
            assert!(matches!(actual.shape(), BasicShape::Shape(_)));
            let expected = if p == 0.5 {
                list(
                    false,
                    point(10., 10.),
                    vec![
                        ShapeCommand::HorizontalLine(AxisCommand::To(lp(15.))),
                        ShapeCommand::VerticalLine(AxisCommand::To(lp(-10.))),
                        ShapeCommand::Close,
                    ],
                )
            } else {
                let endpoint = if p == 0. { from } else { to };
                list(
                    false,
                    initial(endpoint).clone(),
                    command_slice(endpoint).to_vec(),
                )
            };
            assert_eq!(
                pair(from.clone(), to.clone()).sample(progress(p)),
                InterpolationOutcome::Value(wrapped(expected.clone()))
            );
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn mixed_coordinate_types_remain_symbolic_during_extrapolation() {
    let from = list(
        false,
        ComputedPosition::new(ComputedNumeric::percentage(0.05).unwrap(), px(5.)).unwrap(),
        vec![ShapeCommand::Line(Endpoint::By(point(10., 10.)))],
    );
    let to = list(
        true,
        point(15., 15.),
        vec![ShapeCommand::Line(Endpoint::By(point(20., 0.)))],
    );
    for (p, length, percentage, end) in [
        (0.5, 7.5, 0.025, point(15., 5.)),
        (1.5, 22.5, -0.025, point(25., -5.)),
    ] {
        let actual = sample(&from, &to, p);
        assert_eq!(
            initial(&actual).x(),
            &ComputedNumeric::mix(
                DimensionPercentage::new(
                    ComputedDimension::new(DimensionKind::Length, length).unwrap(),
                    percentage
                )
                .unwrap()
            )
        );
        assert_eq!(
            command_slice(&actual),
            &[ShapeCommand::Line(Endpoint::By(end))]
        );
    }
}

#[test]
fn all_command_metadata_conflicts_fall_back_before_any_numeric_arithmetic() {
    let from = list(
        false,
        point(0., 0.),
        vec![
            ShapeCommand::Line(Endpoint::To(point(0., 0.))),
            ShapeCommand::Curve {
                end: Endpoint::By(point(0., 0.)),
                controls: CurveControls::Quadratic(control(ControlAnchor::Start, 0., 0.)),
            },
        ],
    );
    let conflicts = [
        vec![
            ShapeCommand::Line(Endpoint::By(point(f64::MAX, 0.))),
            ShapeCommand::Curve {
                end: Endpoint::By(point(0., 0.)),
                controls: CurveControls::Quadratic(control(ControlAnchor::Start, 0., 0.)),
            },
        ],
        vec![
            ShapeCommand::Line(Endpoint::To(point(f64::MAX, 0.))),
            ShapeCommand::Curve {
                end: Endpoint::By(point(0., 0.)),
                controls: CurveControls::Quadratic(control(ControlAnchor::Origin, 0., 0.)),
            },
        ],
        vec![
            ShapeCommand::Line(Endpoint::To(point(f64::MAX, 0.))),
            ShapeCommand::Curve {
                end: Endpoint::By(point(0., 0.)),
                controls: CurveControls::Cubic(
                    control(ControlAnchor::Start, 0., 0.),
                    control(ControlAnchor::Start, 0., 0.),
                ),
            },
        ],
        vec![
            ShapeCommand::Line(Endpoint::To(point(f64::MAX, 0.))),
            ShapeCommand::Smooth {
                end: Endpoint::By(point(0., 0.)),
                control: None,
            },
        ],
        vec![ShapeCommand::Line(Endpoint::To(point(f64::MAX, 0.)))],
    ];
    for commands in conflicts {
        assert_discrete(&from, &list(false, point(f64::MAX, 0.), commands));
    }
    let different_fill = boxed(BasicShape::Shape(
        CommandShape::new(
            point(f64::MAX, 0.),
            command_slice(&from).to_vec(),
            FillRule::Evenodd,
        )
        .unwrap(),
    ));
    assert_discrete(&from, &different_fill);
    let different_box = ShapeValue::new(ReferenceBox::ContentBox, from.shape().clone());
    assert_discrete(&from, &different_box);
    let axis = list(
        false,
        point(0., 0.),
        vec![ShapeCommand::HorizontalLine(AxisCommand::To(lp(0.)))],
    );
    assert_discrete(
        &axis,
        &list(
            false,
            point(f64::MAX, 0.),
            vec![ShapeCommand::Line(Endpoint::To(point(0., 0.)))],
        ),
    );
}

#[test]
fn smooth_arity_and_each_explicit_anchor_are_compatibility_facts() {
    for anchor in [
        ControlAnchor::Start,
        ControlAnchor::End,
        ControlAnchor::Origin,
    ] {
        let from = list(
            false,
            point(10., 20.),
            vec![ShapeCommand::Smooth {
                end: Endpoint::To(point(50., 80.)),
                control: Some(control(anchor, 3., 4.)),
            }],
        );
        let to = list(
            false,
            point(30., 40.),
            vec![ShapeCommand::Smooth {
                end: Endpoint::To(point(70., 100.)),
                control: Some(control(anchor, 13., 24.)),
            }],
        );
        assert_eq!(
            command_slice(&sample(&from, &to, 0.5)),
            &[ShapeCommand::Smooth {
                end: Endpoint::To(point(60., 90.)),
                control: Some(control(anchor, 8., 14.))
            }]
        );
        assert_discrete(
            &from,
            &list(
                false,
                point(0., 0.),
                vec![ShapeCommand::Smooth {
                    end: Endpoint::To(point(0., 0.)),
                    control: None,
                }],
            ),
        );
        for other in [
            ControlAnchor::Start,
            ControlAnchor::End,
            ControlAnchor::Origin,
        ] {
            if anchor != other {
                assert_discrete(
                    &from,
                    &list(
                        false,
                        point(0., 0.),
                        vec![ShapeCommand::Smooth {
                            end: Endpoint::To(point(0., 0.)),
                            control: Some(control(other, 0., 0.)),
                        }],
                    ),
                );
            }
        }
    }
}

#[test]
fn differing_arc_flags_preserve_endpoints_outside_the_strict_interior() {
    use ArcSize::{Large, Small};
    use ArcSweep::{Ccw, Cw};
    // Literal CSS/WPT truth tables at [-.5, 0, .5, 1, 1.5], independently of
    // numerical flag interpolation and its nonzero-versus-positive distinction.
    for (sweep_a, sweep_b, sweeps) in [
        (Ccw, Cw, [Ccw, Ccw, Cw, Cw, Cw]),
        (Cw, Ccw, [Cw, Cw, Cw, Ccw, Ccw]),
        (Cw, Cw, [Cw; 5]),
        (Ccw, Ccw, [Ccw; 5]),
    ] {
        for (size_a, size_b, sizes) in [
            (Small, Large, [Small, Small, Large, Large, Large]),
            (Large, Small, [Large, Large, Large, Small, Small]),
            (Small, Small, [Small; 5]),
            (Large, Large, [Large; 5]),
        ] {
            let from = list(
                false,
                point(0., 0.),
                vec![arc(
                    Endpoint::To(point(25., 20.)),
                    lp(10.),
                    lp(5.),
                    30.,
                    sweep_a,
                    size_a,
                )],
            );
            let to = list(
                false,
                point(0., 0.),
                vec![arc(
                    Endpoint::To(point(25., 20.)),
                    lp(10.),
                    lp(5.),
                    270.,
                    sweep_b,
                    size_b,
                )],
            );
            for (((p, rotation), sweep), size) in [
                (-0.5, -90.),
                (0., 30.),
                (0.5, 150.),
                (1., 270.),
                (1.5, 390.),
            ]
            .into_iter()
            .zip(sweeps)
            .zip(sizes)
            {
                assert_eq!(
                    command_slice(&sample(&from, &to, p)),
                    &[arc(
                        Endpoint::To(point(25., 20.)),
                        lp(10.),
                        lp(5.),
                        rotation,
                        sweep,
                        size
                    )]
                );
            }
        }
    }
}

#[test]
fn a_second_cubic_anchor_or_arc_form_conflict_wins_before_endpoint_overflow() {
    let curve = |anchor, amount| ShapeCommand::Curve {
        end: Endpoint::To(point(amount, 0.)),
        controls: CurveControls::Cubic(
            control(ControlAnchor::Origin, 0., 0.),
            control(anchor, 0., 0.),
        ),
    };
    assert_discrete(
        &list(false, point(0., 0.), vec![curve(ControlAnchor::Origin, 0.)]),
        &list(
            false,
            point(0., 0.),
            vec![curve(ControlAnchor::End, f64::MAX)],
        ),
    );
    assert_discrete(
        &list(
            false,
            point(0., 0.),
            vec![arc(
                Endpoint::To(point(0., 0.)),
                lp(0.),
                lp(0.),
                0.,
                ArcSweep::Ccw,
                ArcSize::Small,
            )],
        ),
        &list(
            false,
            point(f64::MAX, 0.),
            vec![arc(
                Endpoint::By(point(f64::MAX, 0.)),
                lp(0.),
                lp(0.),
                0.,
                ArcSweep::Ccw,
                ArcSize::Small,
            )],
        ),
    );
}

#[test]
fn diagonal_radius_sources_retain_identity_and_sampled_radii_are_not_clamped() {
    // Supplying owner maps this source to 50% of diagonal/sqrt(2), without box geometry here.
    let source = MathContext::new().source(NumericType::Dimension(DimensionKind::Length));
    let diagonal = LengthPercentage::new(ComputedNumeric::source(source.clone())).unwrap();
    let from = list(
        false,
        point(0., 0.),
        vec![arc(
            Endpoint::To(point(10., 20.)),
            diagonal.clone(),
            diagonal,
            0.,
            ArcSweep::Ccw,
            ArcSize::Small,
        )],
    );
    let to = list(
        false,
        point(0., 0.),
        vec![arc(
            Endpoint::To(point(10., 20.)),
            lp(0.),
            lp(0.),
            0.,
            ArcSweep::Ccw,
            ArcSize::Small,
        )],
    );
    let actual = sample(&from, &to, 0.5);
    let ShapeCommand::Arc(arc) = &command_slice(&actual)[0] else {
        panic!("expected arc")
    };
    for radius in [arc.horizontal_radius(), arc.vertical_radius()] {
        assert!(
            radius
                .numeric()
                .math_nodes()
                .unwrap()
                .iter()
                .any(|node| matches!(node, MathNode::Source(actual) if *actual == source))
        );
    }
    let from = list(
        false,
        point(0., 0.),
        vec![self::arc(
            Endpoint::To(point(0., 0.)),
            lp(10.),
            lp(20.),
            0.,
            ArcSweep::Ccw,
            ArcSize::Small,
        )],
    );
    let to = list(
        false,
        point(0., 0.),
        vec![self::arc(
            Endpoint::To(point(0., 0.)),
            lp(0.),
            lp(0.),
            0.,
            ArcSweep::Ccw,
            ArcSize::Small,
        )],
    );
    assert_eq!(
        command_slice(&sample(&from, &to, 2.)),
        &[self::arc(
            Endpoint::To(point(0., 0.)),
            lp(-10.),
            lp(-20.),
            0.,
            ArcSweep::Ccw,
            ArcSize::Small
        )]
    );
}

#[test]
fn command_failure_identifies_its_operand_and_pair_can_recover() {
    let from = list(
        false,
        point(0., 0.),
        vec![
            ShapeCommand::Close,
            ShapeCommand::Line(Endpoint::By(point(0., 0.))),
        ],
    );
    let to = list(
        false,
        point(0., 0.),
        vec![
            ShapeCommand::Close,
            ShapeCommand::Line(Endpoint::By(point(f64::MAX, 0.))),
        ],
    );
    let pair = pair(from.clone(), to.clone());
    let InterpolationOutcome::ShapeFailure(failure) = pair.sample(progress(2.)) else {
        panic!("expected shape failure")
    };
    assert_eq!(
        failure.error().component(),
        Some(ShapeComponent::Command {
            index: 1,
            component: CommandComponent::Endpoint
        })
    );
    assert_eq!(
        failure.error().numeric_error().unwrap().kind(),
        ComputedErrorKind::NonFiniteResult
    );
    assert_eq!(
        pair.sample(progress(1.)),
        InterpolationOutcome::Value(wrapped(to))
    );
    assert_eq!(
        pair.sample(progress(0.)),
        InterpolationOutcome::Value(wrapped(from))
    );
}

#[test]
fn mixed_command_lists_sample_through_transition_and_keyframe_tracks() {
    let from = list(true, point(0., 0.), commands(0.));
    let to = list(false, point(20., 40.), commands(20.));
    let expected = list(false, point(10., 20.), commands(10.));
    let track = transition(from.clone(), to.clone(), Easing::linear());
    let sampled = track.sample(time(0.5), AnimationPlayState::Running);
    assert!(
        matches!(sampled.value(), SampleValue::Value(value) if *value.value() == wrapped(expected.clone()))
    );
    let frame = KeyframeSegment::new(
        PropertyKey::new("clip-path").unwrap(),
        UnitRatio::new(0.).unwrap(),
        UnitRatio::new(1.).unwrap(),
        wrapped(from),
        wrapped(to),
        Easing::linear(),
    )
    .unwrap();
    let track = KeyframeTrack::new(
        KeyframeAnimationId::new("commands").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.).unwrap(),
            AnimationDuration::from_secs(1.).unwrap(),
            IterationCount::finite(1.).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        vec![frame],
    )
    .unwrap();
    assert!(
        matches!(track.sample(time(0.5), AnimationPlayState::Running)[0].value(), SampleValue::Value(value) if *value.value() == wrapped(expected))
    );
}

#[test]
fn track_command_arithmetic_failure_recovers_on_later_endpoint() {
    let from = list(
        true,
        point(0., 0.),
        vec![ShapeCommand::Line(Endpoint::To(point(0., 0.)))],
    );
    let to = list(
        false,
        point(0., 0.),
        vec![ShapeCommand::Line(Endpoint::To(point(f64::MAX, 0.)))],
    );
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
    let track = transition(from, to.clone(), easing);
    let sampled = track.sample(time(0.5), AnimationPlayState::Running);
    assert_eq!(sampled.classification(), SampleClassification::Failed);
    assert!(matches!(
        sampled.value(),
        SampleValue::Error(InterpolationError::Shape(_))
    ));
    assert!(
        matches!(track.sample(time(0.75), AnimationPlayState::Running).value(), SampleValue::Value(value) if *value.value() == wrapped(to))
    );
}

#[test]
fn every_explicit_command_operand_uses_the_checked_numeric_failure_path() {
    let cases = [
        (
            ShapeCommand::Curve {
                end: Endpoint::To(point(0., 0.)),
                controls: CurveControls::Cubic(
                    control(ControlAnchor::Origin, 0., 0.),
                    control(ControlAnchor::Origin, 0., 0.),
                ),
            },
            ShapeCommand::Curve {
                end: Endpoint::To(point(0., 0.)),
                controls: CurveControls::Cubic(
                    control(ControlAnchor::Origin, 0., 0.),
                    control(ControlAnchor::Origin, f64::MAX, 0.),
                ),
            },
            CommandComponent::SecondControl,
        ),
        (
            arc(
                Endpoint::To(point(0., 0.)),
                lp(0.),
                lp(0.),
                0.,
                ArcSweep::Ccw,
                ArcSize::Small,
            ),
            arc(
                Endpoint::To(point(0., 0.)),
                lp(0.),
                lp(f64::MAX),
                0.,
                ArcSweep::Ccw,
                ArcSize::Small,
            ),
            CommandComponent::VerticalRadius,
        ),
        (
            arc(
                Endpoint::To(point(0., 0.)),
                lp(0.),
                lp(0.),
                0.,
                ArcSweep::Ccw,
                ArcSize::Small,
            ),
            arc(
                Endpoint::To(point(0., 0.)),
                lp(0.),
                lp(0.),
                f64::MAX,
                ArcSweep::Ccw,
                ArcSize::Small,
            ),
            CommandComponent::Rotation,
        ),
        (
            ShapeCommand::Smooth {
                end: Endpoint::By(point(0., 0.)),
                control: Some(control(ControlAnchor::Start, 0., 0.)),
            },
            ShapeCommand::Smooth {
                end: Endpoint::By(point(0., 0.)),
                control: Some(control(ControlAnchor::Start, f64::MAX, 0.)),
            },
            CommandComponent::FirstControl,
        ),
    ];
    for (first, second, component) in cases {
        let from = list(false, point(0., 0.), vec![first]);
        let to = list(false, point(0., 0.), vec![second]);
        let error = from.interpolate(&to, progress(2.)).unwrap_err();
        assert_eq!(
            error.component(),
            Some(ShapeComponent::Command {
                index: 0,
                component
            })
        );
        assert_eq!(
            error.numeric_error().unwrap().kind(),
            ComputedErrorKind::NonFiniteResult
        );
        assert_eq!(sample(&from, &to, 1.), to);
    }
}

#[test]
fn native_running_transition_samples_commands_and_recovers_after_a_failure() {
    use surgeist_animation::time::OriginRelativeTime;
    use surgeist_animation::transition::{
        TransitionChange, TransitionDecisionErrorKind, TransitionKey, TransitionPropertyPolicy,
        TransitionSnapshot, TransitionState, TransitionTiming, Transitionability,
    };
    struct SuppliedPolicy;
    impl TransitionPropertyPolicy for SuppliedPolicy {
        fn transitionability(
            &self,
            _: &PropertyKey,
            _: &InterpolableValue,
            _: &InterpolableValue,
        ) -> Transitionability {
            Transitionability::Continuous
        }
        fn interpolate(
            &self,
            property: &PropertyKey,
            from: &InterpolableValue,
            to: &InterpolableValue,
            progress: InterpolationProgress,
        ) -> InterpolationOutcome {
            match InterpolationPair::new(property.clone(), from.clone(), to.clone()) {
                Ok(pair) => pair.sample(progress),
                Err(error) => InterpolationOutcome::Unsupported(error),
            }
        }
    }
    let key = TransitionKey::new(1_u64, PropertyKey::new("clip-path").unwrap());
    let from = list(
        true,
        point(0., 0.),
        vec![ShapeCommand::Line(Endpoint::To(point(0., 0.)))],
    );
    for amount in [20., f64::MAX] {
        let to = list(
            false,
            point(0., 0.),
            vec![ShapeCommand::Line(Endpoint::To(point(amount, 0.)))],
        );
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
        let change = TransitionChange {
            key: key.clone(),
            before: wrapped(from.clone()),
            after: wrapped(to.clone()),
            eligible: true,
            matching: Some(TransitionTiming::new(
                AnimationDuration::from_secs(1.).unwrap(),
                AnimationDelay::from_secs(0.).unwrap(),
                easing,
            )),
            new_id: TransitionTrackId::new(1),
        };
        let update = TransitionSnapshot::default()
            .update(
                Some(OriginRelativeTime::from_millis(0.).unwrap()),
                &[change],
                &SuppliedPolicy,
            )
            .unwrap()
            .unwrap();
        let Some(TransitionState::Running(run)) = update.state().get(&key) else {
            panic!("expected running transition")
        };
        let sampled = run.sample(
            Some(OriginRelativeTime::from_millis(500.).unwrap()),
            &SuppliedPolicy,
        );
        if amount == f64::MAX {
            let error = sampled.unwrap_err();
            assert_eq!(error.kind(), TransitionDecisionErrorKind::Interpolation);
            assert!(matches!(
                error.interpolation_error(),
                Some(InterpolationError::Shape(_))
            ));
        } else {
            let expected = list(
                false,
                point(0., 0.),
                vec![ShapeCommand::Line(Endpoint::To(point(40., 0.)))],
            );
            assert_eq!(sampled.unwrap().value(), Some(&wrapped(expected)));
        }
        assert_eq!(
            run.sample(
                Some(OriginRelativeTime::from_millis(750.).unwrap()),
                &SuppliedPolicy
            )
            .unwrap()
            .value(),
            Some(&wrapped(to))
        );
    }
}

fn commands(v: f64) -> Vec<ShapeCommand> {
    vec![
        ShapeCommand::Move(Endpoint::By(point(v, v + 1.))),
        ShapeCommand::Line(Endpoint::To(point(v + 2., v + 3.))),
        ShapeCommand::HorizontalLine(AxisCommand::By(lp(v + 4.))),
        ShapeCommand::VerticalLine(AxisCommand::To(lp(v + 5.))),
        ShapeCommand::Curve {
            end: Endpoint::To(point(v + 6., v + 7.)),
            controls: CurveControls::Quadratic(control(ControlAnchor::Origin, v + 8., v + 9.)),
        },
        ShapeCommand::Curve {
            end: Endpoint::By(point(v + 10., v + 11.)),
            controls: CurveControls::Cubic(
                control(ControlAnchor::Start, v + 12., v + 13.),
                control(ControlAnchor::Start, v + 14., v + 15.),
            ),
        },
        ShapeCommand::Smooth {
            end: Endpoint::To(point(v + 16., v + 17.)),
            control: Some(control(ControlAnchor::Origin, v + 18., v + 19.)),
        },
        ShapeCommand::Smooth {
            end: Endpoint::By(point(v + 20., v + 21.)),
            control: None,
        },
        arc(
            Endpoint::By(point(v + 22., v + 23.)),
            lp(v + 24.),
            lp(v + 25.),
            v + 26.,
            ArcSweep::Cw,
            ArcSize::Large,
        ),
        ShapeCommand::Close,
        ShapeCommand::VerticalLine(AxisCommand::By(lp(v + 27.))),
    ]
}
fn control(anchor: ControlAnchor, x: f64, y: f64) -> ControlPoint {
    ControlPoint::new(anchor, point(x, y))
}
fn arc(
    end: Endpoint,
    x: LengthPercentage,
    y: LengthPercentage,
    rotation: f64,
    sweep: ArcSweep,
    size: ArcSize,
) -> ShapeCommand {
    ShapeCommand::Arc(
        ArcCommand::new(
            end,
            x,
            y,
            ComputedNumeric::dimension(
                ComputedDimension::new(DimensionKind::Angle, rotation).unwrap(),
            ),
            sweep,
            size,
        )
        .unwrap(),
    )
}
fn list(path: bool, initial: ComputedPosition, commands: Vec<ShapeCommand>) -> ShapeValue {
    boxed(if path {
        BasicShape::Path(CommandPath::new(initial, commands, FillRule::Nonzero).unwrap())
    } else {
        BasicShape::Shape(CommandShape::new(initial, commands, FillRule::Nonzero).unwrap())
    })
}
fn initial(value: &ShapeValue) -> &ComputedPosition {
    match value.shape() {
        BasicShape::Shape(list) => list.initial(),
        BasicShape::Path(list) => list.initial(),
        _ => panic!("expected commands"),
    }
}
fn command_slice(value: &ShapeValue) -> &[ShapeCommand] {
    match value.shape() {
        BasicShape::Shape(list) => list.commands(),
        BasicShape::Path(list) => list.commands(),
        _ => panic!("expected commands"),
    }
}
fn boxed(shape: BasicShape) -> ShapeValue {
    ShapeValue::new(ReferenceBox::BorderBox, shape)
}
fn wrapped(value: ShapeValue) -> InterpolableValue {
    InterpolableValue::shape(value)
}
fn pair(from: ShapeValue, to: ShapeValue) -> InterpolationPair {
    InterpolationPair::new(
        PropertyKey::new("clip-path").unwrap(),
        wrapped(from),
        wrapped(to),
    )
    .unwrap()
}
fn sample(from: &ShapeValue, to: &ShapeValue, p: f64) -> ShapeValue {
    from.interpolate(to, progress(p)).unwrap()
}
fn progress(p: f64) -> InterpolationProgress {
    InterpolationProgress::new(p).unwrap()
}
fn px(value: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(DimensionKind::Length, value).unwrap())
}
fn lp(value: f64) -> LengthPercentage {
    LengthPercentage::new(px(value)).unwrap()
}
fn point(x: f64, y: f64) -> ComputedPosition {
    ComputedPosition::new(px(x), px(y)).unwrap()
}
fn time(value: f64) -> ElapsedTime {
    ElapsedTime::from_secs(value).unwrap()
}
fn assert_discrete(from: &ShapeValue, to: &ShapeValue) {
    for (p, expected) in [
        (-1., from),
        (0., from),
        (0.5_f64.next_down(), from),
        (0.5, to),
        (1., to),
        (2., to),
    ] {
        assert_eq!(sample(from, to, p), *expected);
    }
}
fn transition(from: ShapeValue, to: ShapeValue, easing: Easing) -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("clip-path").unwrap(),
        wrapped(from),
        wrapped(to),
        AnimationDuration::from_secs(1.).unwrap(),
        AnimationDelay::from_secs(0.).unwrap(),
        easing,
    )
    .unwrap()
}
