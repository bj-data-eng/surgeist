#![forbid(unsafe_code)]

use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, OriginRelativeTime};
use surgeist_animation::transition::{
    RunningTransition, TransitionChange, TransitionKey, TransitionOutcome,
    TransitionPropertyPolicy, TransitionSamplePhase, TransitionSnapshot, TransitionState,
    TransitionTiming, TransitionTrackId, Transitionability,
};
use surgeist_color::{
    Color, ColorSpace, Finite, InterpolationMethod, PredefinedColorSpace, UnitInterval,
};

const SRGB: ColorSpace = ColorSpace::Predefined(PredefinedColorSpace::Srgb);

struct ColorPolicy;

impl ColorPolicy {
    fn pair(
        property: &PropertyKey,
        from: &InterpolableValue,
        to: &InterpolableValue,
    ) -> Result<InterpolationPair, InterpolationError> {
        if let (InterpolableValue::ResolvedColor(from), InterpolableValue::ResolvedColor(to)) =
            (from, to)
        {
            return Ok(InterpolationPair::resolved_colors(
                property.clone(),
                from.color(),
                to.color(),
                InterpolationMethod::try_new(SRGB, None).unwrap(),
            ));
        }
        InterpolationPair::new(property.clone(), from.clone(), to.clone())
    }
}

impl TransitionPropertyPolicy for ColorPolicy {
    fn transitionability(
        &self,
        property: &PropertyKey,
        from: &InterpolableValue,
        to: &InterpolableValue,
    ) -> Transitionability {
        if Self::pair(property, from, to).is_ok() {
            Transitionability::Continuous
        } else {
            Transitionability::Discrete
        }
    }

    fn interpolate(
        &self,
        property: &PropertyKey,
        from: &InterpolableValue,
        to: &InterpolableValue,
        progress: InterpolationProgress,
    ) -> InterpolationOutcome {
        match Self::pair(property, from, to) {
            Ok(pair) => pair.sample(progress),
            Err(error) => InterpolationOutcome::Unsupported(error),
        }
    }
}

#[test]
fn retargeting_keeps_the_extrapolated_color_as_the_new_start() {
    let before = color(0.25, 0.5);
    let old_target = color(0.25, 1.0);
    let new_target = color(1.0, 0.25);
    let overshoot = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 0.25,
                output: 1.5,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    );
    let initial = start(before, old_target.clone(), overshoot);
    let current = running(&initial).sample(at(250.0), &ColorPolicy).unwrap();
    assert_eq!(current.phase(), TransitionSamplePhase::Active);
    assert_checked_color(current.value(), 0.25, 1.25);

    let update = initial
        .update(
            at(250.0),
            &[change(old_target, new_target.clone(), Easing::linear(), 2)],
            &ColorPolicy,
        )
        .unwrap()
        .unwrap();
    assert_replacement(update.outcomes());
    let run = running(update.state());
    assert_eq!(run.start_value(), current.value().unwrap());
    assert_eq!(run.end_value(), &new_target);
    assert_eq!(run.end_time().as_millis(), 1_250.0);
    assert_eq!(
        run.sample(at(250.0), &ColorPolicy).unwrap().value(),
        current.value()
    );

    let later = run.sample(at(750.0), &ColorPolicy).unwrap();
    assert_eq!(later.phase(), TransitionSamplePhase::Active);
    // The red channel is the alpha-weighted mean of 0.25 and 1.0.
    assert_checked_color(later.value(), 0.375, 0.75);
}

#[test]
fn reversing_an_active_color_preserves_the_current_sample_and_shortens_the_run() {
    let before = color(0.0, 1.0);
    let old_target = color(1.0, 1.0);
    let initial = start(before.clone(), old_target.clone(), Easing::linear());
    let current = running(&initial).sample(at(250.0), &ColorPolicy).unwrap();
    assert_eq!(current.phase(), TransitionSamplePhase::Active);
    assert_checked_color(current.value(), 0.25, 1.0);

    let update = initial
        .update(
            at(250.0),
            &[change(
                old_target.clone(),
                before.clone(),
                Easing::linear(),
                2,
            )],
            &ColorPolicy,
        )
        .unwrap()
        .unwrap();
    assert_replacement(update.outcomes());
    let run = running(update.state());
    assert_eq!(run.start_value(), current.value().unwrap());
    assert_eq!(run.end_value(), &before);
    assert_eq!(run.reversing_adjusted_start_value(), &old_target);
    assert_eq!(run.shortening_factor().value(), 0.25);
    assert_eq!(run.duration().as_secs(), 0.25);
    assert_eq!(run.end_time().as_millis(), 500.0);
    assert_eq!(
        run.sample(at(250.0), &ColorPolicy).unwrap().value(),
        current.value()
    );

    let later = run.sample(at(375.0), &ColorPolicy).unwrap();
    assert_eq!(later.phase(), TransitionSamplePhase::Active);
    assert_checked_color(later.value(), 0.125, 1.0);
}

fn color(red: f64, alpha: f64) -> InterpolableValue {
    InterpolableValue::resolved_color(Color::new(
        SRGB,
        [
            Some(Finite::new(red).unwrap()),
            None,
            Some(Finite::new(-0.25).unwrap()),
        ],
        Some(UnitInterval::new(alpha).unwrap()),
    ))
}

fn at(millis: f64) -> Option<OriginRelativeTime> {
    Some(OriginRelativeTime::from_millis(millis).unwrap())
}

fn change(
    before: InterpolableValue,
    after: InterpolableValue,
    easing: Easing,
    id: u64,
) -> TransitionChange<u32> {
    TransitionChange {
        key: TransitionKey::new(1, PropertyKey::new("color").unwrap()),
        before,
        after,
        eligible: true,
        matching: Some(TransitionTiming::new(
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            easing,
        )),
        new_id: TransitionTrackId::new(id),
    }
}

fn start(
    before: InterpolableValue,
    after: InterpolableValue,
    easing: Easing,
) -> TransitionSnapshot<u32> {
    TransitionSnapshot::default()
        .update(at(0.0), &[change(before, after, easing, 1)], &ColorPolicy)
        .unwrap()
        .unwrap()
        .state()
        .clone()
}

fn running(state: &TransitionSnapshot<u32>) -> &RunningTransition<u32> {
    let [TransitionState::Running(run)] = state.records() else {
        panic!(
            "expected one running color transition, got {:?}",
            state.records()
        );
    };
    run
}

fn assert_replacement(outcomes: &[TransitionOutcome<u32>]) {
    assert!(
        matches!(outcomes, [TransitionOutcome::Cancelled(old), TransitionOutcome::Started(new)]
            if old.id() == TransitionTrackId::new(1) && new.id() == TransitionTrackId::new(2)),
        "expected the running color transition to be replaced, got {outcomes:?}",
    );
}

fn assert_checked_color(value: Option<&InterpolableValue>, red: f64, alpha: f64) {
    let Some(InterpolableValue::ColorSample(sample)) = value else {
        panic!("expected a checked color sample, got {value:?}");
    };
    assert_eq!(sample.space(), SRGB);
    assert_eq!(
        sample.channels().map(|channel| channel.map(Finite::get)),
        [Some(red), None, Some(-0.25)]
    );
    assert_eq!(sample.alpha().map(Finite::get), Some(alpha));
}
