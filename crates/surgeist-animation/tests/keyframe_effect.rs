#![forbid(unsafe_code)]

use surgeist_animation::composition::{PropertyValueOperations, compose_value};
use surgeist_animation::computed::{
    ComputedComponent, ComputedError, ComputedNumeric, ComputedRatio, ComputedValue, NumericLiteral,
};
use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableNumber, InterpolableValue, InterpolationError, InterpolationOutcome,
    InterpolationPair, InterpolationProgress, PropertyKey, TransformValue,
};
use surgeist_animation::keyframe::{
    ComputedKeyframeErrorKind, ComputedKeyframeInput, ComputedKeyframes, KeyframeEndpointSource,
    KeyframeSampleInput, KeyframeSampleStage, KeyframeTarget,
};
use surgeist_animation::keyframe_input::{CompositeOperation, KeyframeInput, KeyframesInput};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::time::{AnimationTime, PlaybackRate};
use surgeist_animation::timeline::TimelineMonotonicity;
use surgeist_animation::timing::{
    AnimationDirection, AnimationReplaceState, EffectAnimationState, EffectPlayState, EffectTiming,
    EffectTimingInput, EffectTimingOptions, FillMode, IterationCount,
};
use surgeist_animation::transform::{
    CssPixels, ResolvedTransformList, ScaleFactor, TransformFunction,
};

#[derive(Debug)]
enum OperationError {
    Interpolation(InterpolationError),
    Computed(ComputedError),
    Unsupported(CompositeOperation),
}
impl std::fmt::Display for OperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interpolation(error) => write!(f, "{error}"),
            Self::Computed(error) => write!(f, "{error}"),
            Self::Unsupported(operation) => write!(f, "unsupported {operation:?}"),
        }
    }
}
impl std::error::Error for OperationError {}
impl From<ComputedError> for OperationError {
    fn from(value: ComputedError) -> Self {
        Self::Computed(value)
    }
}
impl From<InterpolationError> for OperationError {
    fn from(value: InterpolationError) -> Self {
        Self::Interpolation(value)
    }
}
struct Operations;
impl PropertyValueOperations for Operations {
    type Error = OperationError;
    fn interpolate(
        &self,
        property: &PropertyKey,
        a: &InterpolableValue,
        b: &InterpolableValue,
        p: InterpolationProgress,
    ) -> Result<InterpolableValue, Self::Error> {
        match InterpolationPair::new(property.clone(), a.clone(), b.clone())?.sample(p) {
            InterpolationOutcome::Value(value) => Ok(value),
            InterpolationOutcome::ArithmeticFailure(error) => {
                Err(InterpolationError::NonFiniteInterpolationResult(error).into())
            }
            InterpolationOutcome::ComputedFailure(error) => {
                Err(InterpolationError::Computed(error).into())
            }
            InterpolationOutcome::ShapeFailure(error) => {
                Err(InterpolationError::Shape(error).into())
            }
            InterpolationOutcome::FilterFailure(error) => {
                Err(InterpolationError::Filter(error).into())
            }
            InterpolationOutcome::ShadowFailure(error) => {
                Err(InterpolationError::Shadow(error).into())
            }
            InterpolationOutcome::GridFailure(error) => Err(InterpolationError::Grid(error).into()),
            InterpolationOutcome::Unsupported(error) => Err(error.into()),
        }
    }
    fn add(
        &self,
        _: &PropertyKey,
        a: &InterpolableValue,
        b: &InterpolableValue,
    ) -> Result<InterpolableValue, Self::Error> {
        match (a, b) {
            (InterpolableValue::Number(a), InterpolableValue::Number(b)) => numeric_result(
                ComputedNumeric::number(a.value())?.add(&ComputedNumeric::number(b.value())?)?,
            ),
            (InterpolableValue::Computed(a), InterpolableValue::Computed(b)) => {
                Ok(InterpolableValue::computed(a.add(b)?))
            }
            (InterpolableValue::Transform(a), InterpolableValue::Transform(b)) => {
                match (a.resolved_list(), b.resolved_list()) {
                    (Some(a), Some(b)) => Ok(InterpolableValue::transform(
                        TransformValue::resolved(a.added_to(b)),
                    )),
                    _ => Err(OperationError::Unsupported(CompositeOperation::Add)),
                }
            }
            _ => Err(OperationError::Unsupported(CompositeOperation::Add)),
        }
    }
    fn accumulate(
        &self,
        _: &PropertyKey,
        a: &InterpolableValue,
        b: &InterpolableValue,
    ) -> Result<InterpolableValue, Self::Error> {
        match (a, b) {
            (InterpolableValue::Number(a), InterpolableValue::Number(b)) => numeric_result(
                ComputedNumeric::number(a.value())?
                    .accumulate(&ComputedNumeric::number(b.value())?)?,
            ),
            (InterpolableValue::Computed(a), InterpolableValue::Computed(b)) => {
                Ok(InterpolableValue::computed(a.accumulate(b)?))
            }
            _ => Err(OperationError::Unsupported(CompositeOperation::Accumulate)),
        }
    }
}
fn numeric_result(value: ComputedNumeric) -> Result<InterpolableValue, OperationError> {
    let Some(NumericLiteral::Number(value)) = value.as_literal() else {
        panic!("expected numeric literal")
    };
    Ok(number(value))
}

#[test]
fn normalized_metadata_is_preserved_without_recomputing_or_merging_offsets() {
    let input = KeyframesInput::<InterpolableValue>::Sequence(vec![
        KeyframeInput {
            properties: vec![(key(), Some(number(0.0)))],
            ..KeyframeInput::default()
        },
        KeyframeInput {
            offset: Some(0.5),
            easing: Ok(Easing::step_start()),
            composite: Some(CompositeOperation::Add),
            properties: vec![(key(), Some(number(20.0)))],
        },
        KeyframeInput {
            offset: Some(0.5),
            properties: vec![(key(), Some(number(80.0)))],
            ..KeyframeInput::default()
        },
        KeyframeInput {
            properties: vec![(key(), Some(number(100.0)))],
            ..KeyframeInput::default()
        },
    ])
    .normalize()
    .unwrap();
    let frames =
        ComputedKeyframes::from_normalized(&input, KeyframeEndpointSource::Native).unwrap();
    assert_eq!(frames.frames().len(), 4);
    assert_eq!(frames.frames()[0].specified_offset(), None);
    assert_eq!(frames.frames()[0].computed_offset().value(), 0.0);
    assert_eq!(frames.frames()[1].specified_offset().unwrap().value(), 0.5);
    assert_eq!(frames.frames()[1].easing(), &Easing::step_start());
    assert_eq!(
        frames.frames()[1].composite(),
        Some(CompositeOperation::Add)
    );
    assert_eq!(sample(&frames, 0.5, 0.0), 80.0);
}

#[test]
fn explicit_offsets_must_match_computed_offsets_for_both_endpoint_sources() {
    for source in [
        KeyframeEndpointSource::Native,
        KeyframeEndpointSource::CssAssembled,
    ] {
        let mut start = frame(0.0, 10.0);
        start.specified_offset = Some(UnitRatio::new(0.25).unwrap());
        let error = ComputedKeyframes::new(vec![start, frame(1.0, 20.0)], source).unwrap_err();
        assert_eq!(
            error.kind(),
            ComputedKeyframeErrorKind::SpecifiedOffsetMismatch
        );
        assert_eq!(error.frame_index(), 0);
        assert_eq!(error.property(), None);

        let mut start = frame(0.0, 10.0);
        start.specified_offset = Some(UnitRatio::new(0.0).unwrap());
        assert!(ComputedKeyframes::new(vec![start, frame(1.0, 20.0)], source).is_ok());
    }
}

#[test]
fn computed_shape_rejects_descending_offsets_duplicates_and_missing_css_endpoints() {
    let error = ComputedKeyframes::new(
        vec![frame(0.8, 1.0), frame(0.2, 2.0)],
        KeyframeEndpointSource::Native,
    )
    .unwrap_err();
    assert_eq!(error.kind(), ComputedKeyframeErrorKind::OffsetOrder);
    assert_eq!(error.frame_index(), 1);
    let mut duplicate = frame(0.5, 1.0);
    duplicate.properties.push((key(), number(2.0)));
    let error =
        ComputedKeyframes::new(vec![duplicate], KeyframeEndpointSource::Native).unwrap_err();
    assert_eq!(error.kind(), ComputedKeyframeErrorKind::DuplicateProperty);
    let error = ComputedKeyframes::new(vec![frame(0.5, 1.0)], KeyframeEndpointSource::CssAssembled)
        .unwrap_err();
    assert_eq!(error.kind(), ComputedKeyframeErrorKind::MissingCssEndpoint);
}

#[test]
fn css_endpoint_coverage_is_checked_for_each_sparse_property() {
    let other = PropertyKey::new("other").unwrap();
    let mut middle = frame(0.5, 50.0);
    middle.properties = vec![(other.clone(), number(50.0))];
    let error = ComputedKeyframes::new(
        vec![frame(0.0, 0.0), middle, frame(1.0, 100.0)],
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap_err();
    assert_eq!(error.kind(), ComputedKeyframeErrorKind::MissingCssEndpoint);
    assert_eq!(error.frame_index(), 1);
    assert_eq!(error.property(), Some(&other));
}

#[test]
fn replace_does_not_require_a_supported_addition_or_matching_underlying_family() {
    let underlying = number(10.0);
    let value = InterpolableValue::discrete(DiscreteValue::new("open").unwrap());
    assert_eq!(
        compose_value(
            &key(),
            CompositeOperation::Replace,
            &underlying,
            &value,
            &Operations
        )
        .unwrap(),
        value
    );
    assert!(
        compose_value(
            &key(),
            CompositeOperation::Add,
            &underlying,
            &value,
            &Operations
        )
        .is_err()
    );
    assert_eq!(underlying, number(10.0));
}

#[test]
fn sparse_tracks_are_filtered_before_interval_selection_and_empty_returns_underlying() {
    let mut unrelated = frame(0.2, 999.0);
    unrelated.properties = vec![(PropertyKey::new("other").unwrap(), number(999.0))];
    let frames = native(vec![frame(0.0, 0.0), unrelated, frame(1.0, 100.0)]);
    assert_eq!(sample(&frames, 0.5, 7.0), 50.0);
    let empty = native(vec![]);
    assert_eq!(sample(&empty, 0.5, 7.0), 7.0);
    let only_other = native(vec![ComputedKeyframeInput {
        properties: vec![],
        ..frame(0.5, 0.0)
    }]);
    assert_eq!(sample(&only_other, 0.5, 9.0), 9.0);
}

#[test]
fn native_missing_endpoints_use_live_underlying_replace_and_linear() {
    let mut middle = frame(0.5, 50.0);
    middle.easing = Easing::step_end();
    let frames = native(vec![middle]);
    assert_eq!(
        sample_with_composite(&frames, 0.25, 10.0, CompositeOperation::Add),
        35.0
    );
    // Synthetic zero endpoint is replace/linear (10), explicit midpoint inherits
    // add (10+50). Local .5 yields 35; adding underlying twice would differ.
    assert_eq!(
        sample_with_composite(&frames, 0.25, 20.0, CompositeOperation::Add),
        45.0
    );
    assert_eq!(sample(&frames, 0.75, 10.0), 50.0);
}

#[test]
fn css_generated_endpoint_values_easing_and_composition_are_retained() {
    let mut initial = frame(0.0, 20.0);
    initial.easing = Easing::step_end();
    initial.composite = Some(CompositeOperation::Add);
    let frames = ComputedKeyframes::new(
        vec![initial, frame(0.5, 50.0), frame(1.0, 80.0)],
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap();
    assert_eq!(sample(&frames, 0.25, 10.0), 30.0);
    assert_eq!(sample(&frames, 1.0, 10.0), 80.0);
}

#[test]
fn local_interval_easing_is_applied_once_and_final_easing_is_ignored() {
    let mut middle = frame(0.4, 40.0);
    middle.easing = half();
    let mut last = frame(1.0, 100.0);
    last.easing = Easing::step_start();
    let frames = native(vec![frame(0.0, 0.0), middle, last]);
    assert_eq!(sample(&frames, 0.7, 0.0), 55.0);
    assert_eq!(sample(&frames, 1.0, 0.0), 70.0);
    let linear = native(vec![frame(0.0, 0.0), frame(0.4, 40.0), frame(1.0, 100.0)]);
    assert_eq!(sample(&linear, 0.7, 0.0), 70.0);
}

#[test]
fn finite_out_of_range_iteration_and_eased_progress_extrapolate() {
    let frames = native(vec![frame(0.0, 0.0), frame(1.0, 100.0)]);
    assert_eq!(sample(&frames, -0.25, 0.0), -25.0);
    assert_eq!(sample(&frames, 1.25, 0.0), 125.0);
    let mut first = frame(0.0, 0.0);
    first.easing = linear_points(&[(0.0, -0.5), (1.0, 1.5)]);
    let frames = native(vec![first, frame(1.0, 100.0)]);
    assert_eq!(sample(&frames, 0.0, 0.0), -50.0);
    assert_eq!(sample(&frames, 1.0, 0.0), 150.0);
}

#[test]
fn exact_overlap_branches_select_distinct_endpoint_values() {
    let frames = native(vec![
        frame(0.0, 10.0),
        frame(0.0, 20.0),
        frame(0.5, 30.0),
        frame(0.5, 80.0),
        frame(1.0, 90.0),
        frame(1.0, 100.0),
    ]);
    for (progress, value) in [
        (-0.25, 10.0),
        (0.0, 20.0),
        (0.25, 25.0),
        (0.5, 80.0),
        (0.75, 85.0),
        (1.0, 100.0),
        (1.25, 100.0),
    ] {
        assert_eq!(sample(&frames, progress, 0.0), value);
    }
}

#[test]
fn single_overlap_endpoint_is_composed_but_not_eased_or_interpolated() {
    let mut first = frame(0.0, 2.0);
    first.composite = Some(CompositeOperation::Add);
    first.easing = half();
    let frames = native(vec![first, frame(0.0, 3.0), frame(1.0, 100.0)]);
    assert_eq!(sample(&frames, -1.0, 10.0), 12.0);
}

#[test]
fn endpoint_overrides_are_composed_before_interpolation() {
    let mut start = frame(0.0, 2.0);
    start.composite = Some(CompositeOperation::Replace);
    let mut end = frame(1.0, 4.0);
    end.composite = Some(CompositeOperation::Add);
    assert_eq!(sample(&native(vec![start, end]), 0.5, 10.0), 8.0);
}

#[test]
fn underlying_first_composition_is_observable_with_real_transforms_and_ratios() {
    let underlying = transform(vec![TransformFunction::Scale {
        x: ScaleFactor::new(2.0).unwrap(),
        y: None,
    }]);
    let endpoint = transform(vec![TransformFunction::TranslateX(
        CssPixels::new(10.0).unwrap(),
    )]);
    let value = compose_value(
        &key(),
        CompositeOperation::Add,
        &underlying,
        &endpoint,
        &Operations,
    )
    .unwrap();
    let InterpolableValue::Transform(value) = value else {
        panic!("expected transform")
    };
    assert_eq!(
        value
            .resolved_list()
            .unwrap()
            .to_matrix()
            .unwrap()
            .coefficients(),
        [2.0, 0.0, 0.0, 2.0, 20.0, 0.0]
    );
    for operation in [CompositeOperation::Add, CompositeOperation::Accumulate] {
        let value = compose_value(
            &key(),
            operation,
            &ratio(1.0, 2.0),
            &ratio(3.0, 4.0),
            &Operations,
        )
        .unwrap();
        assert_eq!(value, ratio(3.0, 4.0));
    }
    assert!(matches!(
        compose_value(
            &key(),
            CompositeOperation::Accumulate,
            &underlying,
            &endpoint,
            &Operations
        ),
        Err(OperationError::Unsupported(CompositeOperation::Accumulate))
    ));
}

#[test]
fn real_computed_ratio_and_integer_interpolation_use_owned_operations() {
    let frames = native(vec![
        value_frame(0.0, ratio(1.0, 1.0)),
        value_frame(1.0, ratio(9.0, 1.0)),
    ]);
    let value = evaluate(
        &frames,
        Some(0.5),
        KeyframeTarget::Available(&number(0.0)),
        CompositeOperation::Replace,
    )
    .unwrap()
    .unwrap();
    let InterpolableValue::Computed(value) = value else {
        panic!("expected computed ratio")
    };
    let ComputedComponent::Ratio(value) = value.components()[0] else {
        panic!("expected ratio")
    };
    assert!((value.numerator() / value.denominator() - 3.0).abs() < 1e-12);
    use surgeist_animation::computed::ComputedInteger;
    let integer = |v| {
        InterpolableValue::computed(ComputedValue::single(ComputedComponent::Numeric(
            ComputedNumeric::integer(ComputedInteger::new(v).unwrap()),
        )))
    };
    let frames = native(vec![
        value_frame(0.0, integer(0.0)),
        value_frame(1.0, integer(1.0)),
    ]);
    assert_eq!(
        evaluate(
            &frames,
            Some(0.5),
            KeyframeTarget::Available(&integer(0.0)),
            CompositeOperation::Replace
        )
        .unwrap(),
        Some(integer(1.0))
    );
}

#[test]
fn discrete_threshold_is_applied_after_interval_easing() {
    let start = InterpolableValue::discrete(DiscreteValue::new("start").unwrap());
    let end = InterpolableValue::discrete(DiscreteValue::new("end").unwrap());
    let frames = native(vec![
        value_frame(0.0, start.clone()),
        value_frame(1.0, end.clone()),
    ]);
    assert_eq!(
        evaluate(
            &frames,
            Some(0.499),
            KeyframeTarget::Available(&start),
            CompositeOperation::Replace
        )
        .unwrap(),
        Some(start.clone())
    );
    assert_eq!(
        evaluate(
            &frames,
            Some(0.5),
            KeyframeTarget::Available(&start),
            CompositeOperation::Replace
        )
        .unwrap(),
        Some(end.clone())
    );
    let mut initial = value_frame(0.0, start.clone());
    initial.easing = half();
    let frames = native(vec![initial, value_frame(1.0, end.clone())]);
    assert_eq!(
        evaluate(
            &frames,
            Some(0.75),
            KeyframeTarget::Available(&start),
            CompositeOperation::Replace
        )
        .unwrap(),
        Some(start.clone())
    );
    assert_eq!(
        evaluate(
            &frames,
            Some(1.0),
            KeyframeTarget::Available(&start),
            CompositeOperation::Replace
        )
        .unwrap(),
        Some(end)
    );
}

#[test]
fn unresolved_progress_and_target_guards_abort_before_any_value_operation() {
    let frames = native(vec![frame(0.0, f64::MAX), frame(1.0, -f64::MAX)]);
    assert!(
        evaluate(
            &frames,
            None,
            KeyframeTarget::Available(&number(0.0)),
            CompositeOperation::Replace
        )
        .unwrap()
        .is_none()
    );
    for target in [
        KeyframeTarget::Absent,
        KeyframeTarget::Uncalculable,
        KeyframeTarget::NotAnimatable,
    ] {
        assert!(
            evaluate(&frames, Some(0.5), target, CompositeOperation::Replace)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn failure_retains_stage_and_original_error_without_mutating_inputs() {
    let mut last = frame(1.0, f64::MAX);
    last.composite = Some(CompositeOperation::Add);
    let frames = native(vec![frame(0.0, 1.0), last]);
    let error = evaluate(
        &frames,
        Some(0.5),
        KeyframeTarget::Available(&number(f64::MAX)),
        CompositeOperation::Replace,
    )
    .unwrap_err();
    assert_eq!(error.stage(), KeyframeSampleStage::Composition);
    assert_eq!(error.frame_index(), Some(1));
    assert!(matches!(
        error.operation_error(),
        Some(OperationError::Computed(_))
    ));
    // Extrapolation's mathematical result is 2*MAX, unlike the representable
    // midpoint between opposite-sign extremes handled by the numeric owner.
    let overflow = native(vec![frame(0.0, f64::MAX), frame(1.0, f64::MAX / 2.0)]);
    let error = evaluate(
        &overflow,
        Some(-2.0),
        KeyframeTarget::Available(&number(0.0)),
        CompositeOperation::Replace,
    )
    .unwrap_err();
    assert_eq!(error.stage(), KeyframeSampleStage::Interpolation);
    assert!(matches!(
        error.operation_error(),
        Some(OperationError::Interpolation(_))
    ));
    assert_eq!(sample(&overflow, 0.0, 0.0), f64::MAX);
    assert_eq!(frames.frames()[1].properties()[0].1, number(f64::MAX));
}

#[test]
fn unrepresentable_interval_distance_is_a_typed_easing_failure() {
    let frames = native(vec![
        frame(0.0, 0.0),
        frame(f64::from_bits(1), 1.0),
        frame(1.0, 2.0),
    ]);
    let error = evaluate(
        &frames,
        Some(-f64::MAX),
        KeyframeTarget::Available(&number(0.0)),
        CompositeOperation::Replace,
    )
    .unwrap_err();
    assert_eq!(error.stage(), KeyframeSampleStage::Easing);
    assert!(error.easing_error().is_some());
}

#[test]
fn native_effect_and_css_interval_easing_have_distinct_progress_adapters() {
    let mut middle = frame(0.4, 40.0);
    middle.easing = half();
    let native = native(vec![frame(0.0, 0.0), middle.clone(), frame(1.0, 100.0)]);
    let css = ComputedKeyframes::new(
        vec![frame(0.0, 0.0), middle, frame(1.0, 100.0)],
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap();
    let timing = timing(0.0, AnimationDirection::Normal, half())
        .sample(timing_input(700.0))
        .unwrap();
    let underlying = number(0.0);
    let input = request(
        KeyframeTarget::Available(&underlying),
        CompositeOperation::Replace,
    );
    let native_value = as_number(
        native
            .sample_timing(&key(), input, timing, &Operations)
            .unwrap()
            .unwrap(),
    );
    let css_value = as_number(
        css.sample_timing(&key(), input, timing, &Operations)
            .unwrap()
            .unwrap(),
    );
    assert!((native_value - 35.0).abs() < 1e-12);
    assert!((css_value - 55.0).abs() < 1e-12);
}

#[test]
fn resolved_transform_endpoints_are_composed_and_interpolated_by_owned_algorithms() {
    let underlying = transform(vec![TransformFunction::Scale {
        x: ScaleFactor::new(2.0).unwrap(),
        y: None,
    }]);
    let translated = |v| {
        transform(vec![TransformFunction::TranslateX(
            CssPixels::new(v).unwrap(),
        )])
    };
    let frames = native(vec![
        value_frame(0.0, translated(0.0)),
        value_frame(1.0, translated(100.0)),
    ]);
    let value = evaluate(
        &frames,
        Some(0.25),
        KeyframeTarget::Available(&underlying),
        CompositeOperation::Add,
    )
    .unwrap()
    .unwrap();
    let InterpolableValue::Transform(value) = value else {
        panic!("expected transform")
    };
    assert_eq!(
        value
            .resolved_list()
            .unwrap()
            .to_matrix()
            .unwrap()
            .coefficients(),
        [2.0, 0.0, 0.0, 2.0, 50.0, 0.0]
    );
}

#[test]
fn native_interval_step_uses_selected_clause_without_forwarding_effect_before_flag() {
    let mut initial = frame(0.0, 0.0);
    initial.easing = Easing::step_start();
    let native = native(vec![initial.clone(), frame(1.0, 100.0)]);
    let css = ComputedKeyframes::new(
        vec![initial, frame(1.0, 100.0)],
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap();
    let timing = timing(1000.0, AnimationDirection::Normal, Easing::linear())
        .sample(timing_input(500.0))
        .unwrap();
    assert!(timing.easing_before_flag());
    let underlying = number(0.0);
    let input = request(
        KeyframeTarget::Available(&underlying),
        CompositeOperation::Replace,
    );
    // Current selected §5.3.4 evaluates start-frame easing with local distance;
    // the whole-effect flag was already consumed by EffectTiming's easing.
    assert_eq!(
        as_number(
            native
                .sample_timing(&key(), input, timing, &Operations)
                .unwrap()
                .unwrap()
        ),
        100.0
    );
    assert_eq!(
        as_number(
            css.sample_timing(&key(), input, timing, &Operations)
                .unwrap()
                .unwrap()
        ),
        0.0
    );
}

#[test]
fn css_delay_holds_raw_directed_endpoint_and_retains_reverse_step_side() {
    let mut initial = frame(0.0, 0.0);
    initial.easing = linear_points(&[(0.0, 0.25), (1.0, 0.75)]);
    let css = ComputedKeyframes::new(
        vec![initial, frame(1.0, 100.0)],
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap();
    let underlying = number(0.0);
    let input = request(
        KeyframeTarget::Available(&underlying),
        CompositeOperation::Replace,
    );
    for (direction, expected) in [
        (AnimationDirection::Normal, 0.0),
        (AnimationDirection::Reverse, 100.0),
    ] {
        let sample = timing(1000.0, direction, Easing::linear())
            .sample(timing_input(500.0))
            .unwrap();
        assert_eq!(
            as_number(
                css.sample_timing(&key(), input, sample, &Operations)
                    .unwrap()
                    .unwrap()
            ),
            expected
        );
    }
    let mut initial = frame(0.0, 0.0);
    initial.easing = Easing::step_start();
    let css = ComputedKeyframes::new(
        vec![initial, frame(1.0, 100.0)],
        KeyframeEndpointSource::CssAssembled,
    )
    .unwrap();
    let sample = timing(0.0, AnimationDirection::Reverse, Easing::linear())
        .sample(timing_input(1000.0))
        .unwrap();
    assert!(sample.easing_before_flag());
    assert_eq!(
        as_number(
            css.sample_timing(&key(), input, sample, &Operations)
                .unwrap()
                .unwrap()
        ),
        0.0
    );
}

fn key() -> PropertyKey {
    PropertyKey::new("value").unwrap()
}
fn number(v: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(v).unwrap())
}
fn ratio(a: f64, b: f64) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Ratio(
        ComputedRatio::new(a, b).unwrap(),
    )))
}
fn transform(list: Vec<TransformFunction>) -> InterpolableValue {
    InterpolableValue::transform(TransformValue::resolved(ResolvedTransformList::new(list)))
}
fn value_frame(offset: f64, value: InterpolableValue) -> ComputedKeyframeInput {
    ComputedKeyframeInput {
        specified_offset: Some(UnitRatio::new(offset).unwrap()),
        computed_offset: UnitRatio::new(offset).unwrap(),
        easing: Easing::linear(),
        composite: None,
        properties: vec![(key(), value)],
    }
}
fn frame(offset: f64, value: f64) -> ComputedKeyframeInput {
    value_frame(offset, number(value))
}
fn native(frames: Vec<ComputedKeyframeInput>) -> ComputedKeyframes {
    ComputedKeyframes::new(frames, KeyframeEndpointSource::Native).unwrap()
}
fn request(
    target: KeyframeTarget<'_>,
    effect_composite: CompositeOperation,
) -> KeyframeSampleInput<'_> {
    KeyframeSampleInput {
        target,
        effect_composite,
    }
}
fn evaluate(
    frames: &ComputedKeyframes,
    progress: Option<f64>,
    target: KeyframeTarget<'_>,
    operation: CompositeOperation,
) -> Result<
    Option<InterpolableValue>,
    surgeist_animation::keyframe::KeyframeSampleError<OperationError>,
> {
    frames.sample(
        &key(),
        request(target, operation),
        progress.map(|p| InterpolationProgress::new(p).unwrap()),
        &Operations,
    )
}
fn sample(frames: &ComputedKeyframes, progress: f64, underlying: f64) -> f64 {
    sample_with_composite(frames, progress, underlying, CompositeOperation::Replace)
}
fn sample_with_composite(
    frames: &ComputedKeyframes,
    progress: f64,
    underlying: f64,
    operation: CompositeOperation,
) -> f64 {
    as_number(
        evaluate(
            frames,
            Some(progress),
            KeyframeTarget::Available(&number(underlying)),
            operation,
        )
        .unwrap()
        .unwrap(),
    )
}
fn as_number(value: InterpolableValue) -> f64 {
    let InterpolableValue::Number(n) = value else {
        panic!("expected number")
    };
    n.value()
}
fn half() -> Easing {
    linear_points(&[(0.0, 0.0), (1.0, 0.5)])
}
fn linear_points(points: &[(f64, f64)]) -> Easing {
    Easing::linear_function(
        LinearEasing::new(
            points
                .iter()
                .map(|&(input, output)| LinearControlPointInput { input, output })
                .collect(),
        )
        .unwrap(),
    )
}
fn timing(delay: f64, direction: AnimationDirection, easing: Easing) -> EffectTiming {
    EffectTiming::try_new(EffectTimingOptions {
        start_delay_millis: delay,
        duration_millis: 1000.0,
        direction,
        easing,
        fill: FillMode::Both,
        iterations: IterationCount::finite(1.0).unwrap(),
        ..EffectTimingOptions::default()
    })
    .unwrap()
}
fn timing_input(time: f64) -> EffectTimingInput {
    EffectTimingInput::Associated {
        local_time: Some(AnimationTime::from_millis(time).unwrap()),
        animation: EffectAnimationState::new(
            PlaybackRate::new(1.0).unwrap(),
            EffectPlayState::Running,
            Some(TimelineMonotonicity::Monotonic),
            AnimationReplaceState::Active,
        ),
    }
}
