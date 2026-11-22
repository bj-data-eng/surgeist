#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedDimension, ComputedErrorKind, ComputedNumeric, ComputedPosition, DimensionKind,
    DimensionPercentage, MathContext, MathNode, NumericLiteral, NumericType,
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
    BasicShape, Circle, CornerRadii, Ellipse, FillRule, Inset, LengthPercentage, Polygon, Radius,
    ReferenceBox, ShapeComponent, ShapeErrorKind, ShapeValue,
};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

#[test]
fn shape_numeric_boundaries_reject_other_dimensions_and_number_roots() {
    for value in [
        ComputedNumeric::number(1.0).unwrap(),
        dim(DimensionKind::Angle, 1.0),
        dim(DimensionKind::Time, 1.0),
        ComputedNumeric::source(MathContext::new().source(NumericType::Number)),
    ] {
        let error = LengthPercentage::new(value.clone()).unwrap_err();
        assert_eq!(error.kind(), ShapeErrorKind::InvalidNumericType);
        assert_eq!(error.actual_type(), Some(value.numeric_type()));
    }
    assert!(LengthPercentage::new(px(-4.0)).is_ok());
    assert!(LengthPercentage::new(percent(1.5)).is_ok());
    assert!(
        LengthPercentage::new(ComputedNumeric::source(
            MathContext::new().source(NumericType::DimensionPercentage(DimensionKind::Length))
        ))
        .is_ok()
    );
    assert_eq!(
        Polygon::new(vec![], FillRule::Nonzero, px(0.0))
            .unwrap_err()
            .kind(),
        ShapeErrorKind::EmptyPolygon
    );
    for rounding in [
        percent(0.5),
        ComputedNumeric::number(0.0).unwrap(),
        dim(DimensionKind::Angle, 0.0),
    ] {
        let error = Polygon::new(vec![point(0.0, 0.0)], FillRule::Nonzero, rounding).unwrap_err();
        assert_eq!(error.kind(), ShapeErrorKind::InvalidNumericType);
        assert_eq!(error.component(), Some(ShapeComponent::PolygonRounding));
    }
}

#[test]
fn circles_interpolate_radius_and_each_normalized_center_axis() {
    let from = boxed(BasicShape::Circle(Circle::new(
        Radius::Numeric(lp(20.0)),
        point(10.0, 30.0),
    )));
    let to = boxed(BasicShape::Circle(Circle::new(
        Radius::Numeric(pct(0.6)),
        ComputedPosition::new(percent(0.5), percent(0.25)).unwrap(),
    )));
    let value = from.interpolate(&to, progress(0.25)).unwrap();
    let BasicShape::Circle(circle) = value.shape() else {
        panic!("expected circle")
    };
    assert_eq!(circle.radius(), &Radius::Numeric(mix(15.0, 0.15)));
    assert_eq!(circle.center().x(), &numeric_mix(7.5, 0.125));
    assert_eq!(circle.center().y(), &numeric_mix(22.5, 0.0625));
}

#[test]
fn ellipses_interpolate_both_numeric_radii_without_resolving_percentages() {
    let from = boxed(BasicShape::Ellipse(Ellipse::new(
        Radius::Numeric(lp(10.0)),
        Radius::Numeric(lp(20.0)),
        point(0.0, 0.0),
    )));
    let to = boxed(BasicShape::Ellipse(Ellipse::new(
        Radius::Numeric(lp(30.0)),
        Radius::Numeric(pct(0.8)),
        point(20.0, 40.0),
    )));
    let value = from.interpolate(&to, progress(0.5)).unwrap();
    let BasicShape::Ellipse(ellipse) = value.shape() else {
        panic!("expected ellipse")
    };
    assert_eq!(ellipse.horizontal_radius(), &Radius::Numeric(lp(20.0)));
    assert_eq!(ellipse.vertical_radius(), &Radius::Numeric(mix(10.0, 0.4)));
    assert_eq!(ellipse.center(), &point(10.0, 20.0));
}

#[test]
fn insets_preserve_four_offset_and_eight_corner_components_in_order() {
    let from = inset(
        [1.0, 2.0, 3.0, 4.0],
        [5.0, 6.0, 7.0, 8.0],
        [9.0, 10.0, 11.0, 12.0],
    );
    let to = inset(
        [21.0, 22.0, 23.0, 24.0],
        [25.0, 26.0, 27.0, 28.0],
        [29.0, 30.0, 31.0, 32.0],
    );
    assert_eq!(
        from.interpolate(&to, progress(0.5)).unwrap(),
        inset(
            [11.0, 12.0, 13.0, 14.0],
            [15.0, 16.0, 17.0, 18.0],
            [19.0, 20.0, 21.0, 22.0]
        )
    );
}

#[test]
fn polygon_vertices_and_rounding_interpolate_including_one_and_two_vertex_cases() {
    for (from_points, to_points, expected_points) in [
        (
            vec![point(0.0, 0.0)],
            vec![point(20.0, 40.0)],
            vec![point(5.0, 10.0)],
        ),
        (
            vec![point(-4.0, -8.0), point(2.0, 4.0)],
            vec![point(4.0, 8.0), point(6.0, 12.0)],
            vec![point(-2.0, -4.0), point(3.0, 6.0)],
        ),
        (
            vec![point(0.0, 0.0), point(40.0, 0.0), point(0.0, 80.0)],
            vec![point(20.0, 40.0), point(60.0, 40.0), point(20.0, 120.0)],
            vec![point(5.0, 10.0), point(45.0, 10.0), point(5.0, 90.0)],
        ),
    ] {
        // Caller-lowered omitted rounding is the explicit computed 0px.
        let from = polygon(from_points, FillRule::Evenodd, px(0.0));
        let to = polygon(to_points, FillRule::Evenodd, px(12.0));
        let expected = polygon(expected_points, FillRule::Evenodd, px(3.0));
        assert_eq!(from.interpolate(&to, progress(0.25)).unwrap(), expected);
    }
}

#[test]
fn polygon_count_and_fill_rule_conflicts_fall_back_before_vertex_arithmetic() {
    let from = polygon(vec![point(0.0, 0.0)], FillRule::Nonzero, px(0.0));
    for to in [
        polygon(
            vec![point(f64::MAX, f64::MAX)],
            FillRule::Evenodd,
            px(f64::MAX),
        ),
        polygon(
            vec![point(f64::MAX, 0.0), point(0.0, f64::MAX)],
            FillRule::Nonzero,
            px(f64::MAX),
        ),
    ] {
        assert_discrete(&from, &to);
    }
}

#[test]
fn reference_box_and_shape_kind_conflicts_switch_the_complete_value() {
    let from = circle(0.0);
    let to = ShapeValue::new(
        ReferenceBox::PaddingBox,
        BasicShape::Circle(Circle::new(
            Radius::Numeric(lp(f64::MAX)),
            point(f64::MAX, f64::MAX),
        )),
    );
    assert_discrete(&from, &to);
    let ellipse = boxed(BasicShape::Ellipse(Ellipse::new(
        Radius::Numeric(lp(f64::MAX)),
        Radius::Numeric(lp(f64::MAX)),
        point(f64::MAX, f64::MAX),
    )));
    assert_discrete(&from, &ellipse);
}

#[test]
fn radius_keywords_even_when_equal_make_the_whole_shape_discrete() {
    for radius in [
        Radius::ClosestSide,
        Radius::FarthestSide,
        Radius::ClosestCorner,
        Radius::FarthestCorner,
    ] {
        let from = boxed(BasicShape::Circle(Circle::new(
            radius.clone(),
            point(0.0, 0.0),
        )));
        let to = boxed(BasicShape::Circle(Circle::new(
            radius.clone(),
            point(f64::MAX, f64::MAX),
        )));
        assert_discrete(&from, &to);
        // A keyword in the second axis is preflighted before the first axis,
        // whose numeric extrapolation would otherwise overflow.
        let from = boxed(BasicShape::Ellipse(Ellipse::new(
            Radius::Numeric(lp(0.0)),
            radius.clone(),
            point(0.0, 0.0),
        )));
        let to = boxed(BasicShape::Ellipse(Ellipse::new(
            Radius::Numeric(lp(f64::MAX)),
            radius,
            point(f64::MAX, f64::MAX),
        )));
        assert_discrete(&from, &to);
    }
}

#[test]
fn exact_endpoints_retain_each_complete_shape_including_symbolic_sources() {
    let source = MathContext::new().source(NumericType::Dimension(DimensionKind::Length));
    let from = boxed(BasicShape::Circle(Circle::new(
        Radius::Numeric(LengthPercentage::new(ComputedNumeric::source(source.clone())).unwrap()),
        point(0.0, 0.0),
    )));
    let to = circle(50.0);
    assert_eq!(from.interpolate(&to, progress(0.0)).unwrap(), from);
    assert_eq!(from.interpolate(&to, progress(1.0)).unwrap(), to);
    let sampled = from.interpolate(&to, progress(0.25)).unwrap();
    let BasicShape::Circle(circle) = sampled.shape() else {
        panic!("expected circle")
    };
    let Radius::Numeric(radius) = circle.radius() else {
        panic!("expected numeric radius")
    };
    let mut values = Vec::new();
    for node in radius.numeric().math_nodes().unwrap() {
        let value = match node {
            MathNode::Source(actual) if *actual == source => 10.0,
            MathNode::Literal(NumericLiteral::Dimension(value)) => value.value(),
            MathNode::Combine {
                operation: surgeist_animation::computed::ComputedOperation::Interpolate(p),
                first,
                second,
                ..
            } => (1.0 - p.value()) * values[*first] + p.value() * values[*second],
            node => panic!("unexpected symbolic node {node:?}"),
        };
        values.push(value);
    }
    assert_eq!(values.last(), Some(&20.0));
    for (from, to) in compatible_pairs() {
        assert_eq!(from.interpolate(&to, progress(0.0)).unwrap(), from);
        assert_eq!(from.interpolate(&to, progress(1.0)).unwrap(), to);
    }
}

#[test]
fn shape_sampling_retains_negative_radii_and_overlapping_insets_for_the_caller() {
    assert_eq!(
        circle(10.0)
            .interpolate(&circle(0.0), progress(2.0))
            .unwrap(),
        circle(-10.0)
    );
    let make = |amount| {
        boxed(BasicShape::Inset(Inset::new(
            std::array::from_fn(|_| pct(amount)),
            CornerRadii::new(
                std::array::from_fn(|_| lp(20.0)),
                std::array::from_fn(|_| lp(30.0)),
            ),
        )))
    };
    assert_eq!(
        make(0.75).interpolate(&make(1.25), progress(0.5)).unwrap(),
        make(1.0)
    );
}

#[test]
fn shape_arithmetic_failures_identify_the_component_and_keep_pair_endpoints_owned() {
    let from = circle(0.0);
    let to = circle(f64::MAX);
    let error = from.interpolate(&to, progress(2.0)).unwrap_err();
    assert_eq!(error.kind(), ShapeErrorKind::Numeric);
    assert_eq!(error.component(), Some(ShapeComponent::Radius));
    assert_eq!(
        error.numeric_error().unwrap().kind(),
        ComputedErrorKind::NonFiniteResult
    );
    let failure = {
        let pair = pair(from.clone(), to.clone());
        assert_eq!(
            pair.sample(progress(0.0)),
            InterpolationOutcome::Value(wrapped(from.clone()))
        );
        let InterpolationOutcome::ShapeFailure(failure) = pair.sample(progress(2.0)) else {
            panic!("expected sample failure")
        };
        failure
    };
    assert_eq!(failure.property().as_str(), "shape-outside");
    assert_eq!(failure.from(), &wrapped(from));
    assert_eq!(failure.to(), &wrapped(to));
    assert_eq!(failure.progress(), progress(2.0));
    assert_eq!(failure.error().component(), Some(ShapeComponent::Radius));
}

#[test]
fn a_late_corner_radius_overflow_returns_a_typed_failure() {
    let from = inset([0.0; 4], [0.0; 4], [0.0; 4]);
    let to = inset([1.0; 4], [1.0; 4], [1.0, 1.0, 1.0, f64::MAX]);
    let error = from.interpolate(&to, progress(2.0)).unwrap_err();
    assert_eq!(
        error.component(),
        Some(ShapeComponent::CornerVerticalRadius(3))
    );
    assert_eq!(
        error.numeric_error().unwrap().kind(),
        ComputedErrorKind::NonFiniteResult
    );
}

#[test]
fn every_shape_kind_samples_through_transition_and_keyframe_paths() {
    for (index, (from, to)) in compatible_pairs().into_iter().enumerate() {
        let expected = match index {
            0 => circle(20.0),
            1 => boxed(BasicShape::Ellipse(Ellipse::new(
                Radius::Numeric(lp(20.0)),
                Radius::Numeric(lp(30.0)),
                point(10.0, 20.0),
            ))),
            2 => inset([10.0; 4], [20.0; 4], [30.0; 4]),
            3 => polygon(vec![point(10.0, 20.0)], FillRule::Nonzero, px(5.0)),
            _ => panic!("unexpected shape pair"),
        };
        let transition = transition(from.clone(), to.clone(), Easing::linear());
        let sample = transition.sample(time(0.5), AnimationPlayState::Running);
        assert_eq!(sample.classification(), SampleClassification::Active);
        assert!(
            matches!(sample.value(), SampleValue::Value(value) if *value.value() == wrapped(expected.clone()))
        );
        let keyframe = keyframe(from, to, Easing::linear());
        let samples = keyframe.sample(time(0.5), AnimationPlayState::Running);
        assert_eq!(samples[0].classification(), SampleClassification::Active);
        assert!(
            matches!(samples[0].value(), SampleValue::Value(value) if *value.value() == wrapped(expected))
        );
    }
}

#[test]
fn track_samples_report_shape_failure_and_recover_at_exact_endpoints() {
    let from = circle(0.0);
    let to = circle(f64::MAX);
    let easing = overshoot();
    let transition = transition(from.clone(), to.clone(), easing.clone());
    let failed = transition.sample(time(0.5), AnimationPlayState::Running);
    assert_eq!(failed.classification(), SampleClassification::Failed);
    assert!(
        matches!(failed.value(), SampleValue::Error(InterpolationError::Shape(value)) if value.error().component() == Some(ShapeComponent::Radius))
    );
    assert!(
        matches!(transition.sample(time(0.0), AnimationPlayState::Running).value(), SampleValue::Value(value) if *value.value() == wrapped(from.clone()))
    );
    let track = keyframe(from, to.clone(), easing);
    assert_eq!(
        track.sample(time(0.5), AnimationPlayState::Running)[0].classification(),
        SampleClassification::Failed
    );
    assert!(
        matches!(track.sample(time(1.0), AnimationPlayState::Running)[0].value(), SampleValue::Value(value) if *value.value() == wrapped(to))
    );
}

#[test]
fn semantic_reference_box_tags_are_retained_without_selecting_used_geometry() {
    for reference_box in [
        ReferenceBox::ContentBox,
        ReferenceBox::PaddingBox,
        ReferenceBox::BorderBox,
        ReferenceBox::MarginBox,
        ReferenceBox::FillBox,
        ReferenceBox::StrokeBox,
        ReferenceBox::ViewBox,
    ] {
        let from = ShapeValue::new(reference_box, circle(10.0).shape().clone());
        let to = ShapeValue::new(reference_box, circle(30.0).shape().clone());
        let result = from.interpolate(&to, progress(0.5)).unwrap();
        assert_eq!(result.reference_box(), reference_box);
        assert_eq!(result.shape(), circle(20.0).shape());
    }
}

#[test]
fn native_running_transition_samples_shapes_and_propagates_shape_failures() {
    use surgeist_animation::time::OriginRelativeTime;
    use surgeist_animation::transition::{
        TransitionChange, TransitionDecisionErrorKind, TransitionKey, TransitionPropertyPolicy,
        TransitionSnapshot, TransitionState, TransitionTiming, Transitionability,
    };
    // These supplied pairs are known numeric circles; a production Style
    // adapter owns property metadata and pair-specific transition applicability.
    struct NumericCirclePolicy;
    impl TransitionPropertyPolicy for NumericCirclePolicy {
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
    for (to, easing, fails) in [
        (circle(30.0), Easing::linear(), false),
        (circle(f64::MAX), overshoot(), true),
    ] {
        let key = TransitionKey::new(1_u64, PropertyKey::new("shape-outside").unwrap());
        let change = TransitionChange {
            key: key.clone(),
            before: wrapped(circle(10.0)),
            after: wrapped(to),
            eligible: true,
            matching: Some(TransitionTiming::new(
                AnimationDuration::from_secs(1.0).unwrap(),
                AnimationDelay::from_secs(0.0).unwrap(),
                easing,
            )),
            new_id: TransitionTrackId::new(1),
        };
        let update = TransitionSnapshot::default()
            .update(
                Some(OriginRelativeTime::from_millis(0.0).unwrap()),
                &[change],
                &NumericCirclePolicy,
            )
            .unwrap()
            .unwrap();
        let Some(TransitionState::Running(run)) = update.state().get(&key) else {
            panic!("expected native running transition")
        };
        let result = run.sample(
            Some(OriginRelativeTime::from_millis(500.0).unwrap()),
            &NumericCirclePolicy,
        );
        if fails {
            let error = result.unwrap_err();
            assert_eq!(error.kind(), TransitionDecisionErrorKind::Interpolation);
            assert!(
                matches!(error.interpolation_error(), Some(InterpolationError::Shape(value)) if value.error().component() == Some(ShapeComponent::Radius))
            );
        } else {
            assert_eq!(result.unwrap().value(), Some(&wrapped(circle(20.0))));
        }
        assert_eq!(
            run.sample(
                Some(OriginRelativeTime::from_millis(0.0).unwrap()),
                &NumericCirclePolicy
            )
            .unwrap()
            .value(),
            Some(&wrapped(circle(10.0)))
        );
    }
}

fn compatible_pairs() -> Vec<(ShapeValue, ShapeValue)> {
    vec![
        (circle(10.0), circle(30.0)),
        (
            boxed(BasicShape::Ellipse(Ellipse::new(
                Radius::Numeric(lp(10.0)),
                Radius::Numeric(lp(20.0)),
                point(0.0, 0.0),
            ))),
            boxed(BasicShape::Ellipse(Ellipse::new(
                Radius::Numeric(lp(30.0)),
                Radius::Numeric(lp(40.0)),
                point(20.0, 40.0),
            ))),
        ),
        (
            inset([0.0; 4], [10.0; 4], [20.0; 4]),
            inset([20.0; 4], [30.0; 4], [40.0; 4]),
        ),
        (
            polygon(vec![point(0.0, 0.0)], FillRule::Nonzero, px(0.0)),
            polygon(vec![point(20.0, 40.0)], FillRule::Nonzero, px(10.0)),
        ),
    ]
}
fn assert_discrete(from: &ShapeValue, to: &ShapeValue) {
    for (p, expected) in [
        (-1.0, from),
        (0.0, from),
        (0.5_f64.next_down(), from),
        (0.5, to),
        (1.0, to),
        (2.0, to),
    ] {
        assert_eq!(from.interpolate(to, progress(p)).unwrap(), *expected);
        assert_eq!(
            pair(from.clone(), to.clone()).sample(progress(p)),
            InterpolationOutcome::Value(wrapped(expected.clone()))
        );
    }
}
fn dim(kind: DimensionKind, value: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(kind, value).unwrap())
}
fn px(value: f64) -> ComputedNumeric {
    dim(DimensionKind::Length, value)
}
fn percent(value: f64) -> ComputedNumeric {
    ComputedNumeric::percentage(value).unwrap()
}
fn lp(value: f64) -> LengthPercentage {
    LengthPercentage::new(px(value)).unwrap()
}
fn pct(value: f64) -> LengthPercentage {
    LengthPercentage::new(percent(value)).unwrap()
}
fn numeric_mix(length: f64, percentage: f64) -> ComputedNumeric {
    ComputedNumeric::mix(
        DimensionPercentage::new(
            ComputedDimension::new(DimensionKind::Length, length).unwrap(),
            percentage,
        )
        .unwrap(),
    )
}
fn mix(length: f64, percentage: f64) -> LengthPercentage {
    LengthPercentage::new(numeric_mix(length, percentage)).unwrap()
}
fn point(x: f64, y: f64) -> ComputedPosition {
    ComputedPosition::new(px(x), px(y)).unwrap()
}
fn boxed(shape: BasicShape) -> ShapeValue {
    ShapeValue::new(ReferenceBox::BorderBox, shape)
}
fn circle(radius: f64) -> ShapeValue {
    boxed(BasicShape::Circle(Circle::new(
        Radius::Numeric(lp(radius)),
        point(0.0, 0.0),
    )))
}
fn inset(offsets: [f64; 4], horizontal: [f64; 4], vertical: [f64; 4]) -> ShapeValue {
    boxed(BasicShape::Inset(Inset::new(
        offsets.map(lp),
        CornerRadii::new(horizontal.map(lp), vertical.map(lp)),
    )))
}
fn polygon(
    vertices: Vec<ComputedPosition>,
    fill_rule: FillRule,
    rounding: ComputedNumeric,
) -> ShapeValue {
    boxed(BasicShape::Polygon(
        Polygon::new(vertices, fill_rule, rounding).unwrap(),
    ))
}
fn wrapped(value: ShapeValue) -> InterpolableValue {
    InterpolableValue::shape(value)
}
fn pair(from: ShapeValue, to: ShapeValue) -> InterpolationPair {
    InterpolationPair::new(
        PropertyKey::new("shape-outside").unwrap(),
        wrapped(from),
        wrapped(to),
    )
    .unwrap()
}
fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
fn time(value: f64) -> ElapsedTime {
    ElapsedTime::from_secs(value).unwrap()
}
fn overshoot() -> Easing {
    Easing::linear_function(
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
    )
}
fn transition(from: ShapeValue, to: ShapeValue, easing: Easing) -> TransitionTrack {
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::new("shape-outside").unwrap(),
        wrapped(from),
        wrapped(to),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(0.0).unwrap(),
        easing,
    )
    .unwrap()
}
fn keyframe(from: ShapeValue, to: ShapeValue, easing: Easing) -> KeyframeTrack {
    let segment = KeyframeSegment::new(
        PropertyKey::new("shape-outside").unwrap(),
        UnitRatio::new(0.0).unwrap(),
        UnitRatio::new(1.0).unwrap(),
        wrapped(from),
        wrapped(to),
        easing,
    )
    .unwrap();
    KeyframeTrack::new(
        KeyframeAnimationId::new("shape").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        vec![segment],
    )
    .unwrap()
}
