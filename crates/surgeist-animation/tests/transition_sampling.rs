use surgeist_animation::easing::{Easing, LinearControlPointInput, LinearEasing};
use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
use surgeist_animation::sample::SampleValue;
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::AnimationPlayState;
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

#[test]
fn delayed_transition_holds_its_start_value_until_the_active_interval() {
    let track = track(1.0);
    assert_eq!(opacity(&track, 0.5), 0.0);
    assert_eq!(opacity(&track, 1.5), 0.625);
}

#[test]
fn at_the_start_time_the_transition_returns_its_start_value() {
    let track = track(1.0);
    assert_eq!(opacity(&track, 1.0), 0.0);
}

#[test]
fn a_negative_delay_enters_the_active_interval_with_easing() {
    let track = track(-0.5);
    // At zero elapsed time the transition is halfway through its duration.
    assert_eq!(opacity(&track, 0.0), 0.625);
}

#[test]
fn delay_returns_the_raw_start_even_when_interpolation_is_unsupported() {
    use surgeist_animation::interpolation::TransformValue;
    let from = InterpolableValue::transform(TransformValue::unsupported("translate"));
    let to = InterpolableValue::transform(TransformValue::unsupported("scale"));
    let track = TransitionTrack::new(
        TransitionTrackId::new(2),
        PropertyKey::from_text("transform").unwrap(),
        from.clone(),
        to,
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(1.0).unwrap(),
        Easing::linear(),
    )
    .unwrap();
    let sample = track.sample(
        ElapsedTime::from_secs(0.5).unwrap(),
        AnimationPlayState::Running,
    );
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected raw delay value")
    };
    assert_eq!(value.value(), &from);
}

fn track(delay: f64) -> TransitionTrack {
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.25,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    );
    TransitionTrack::new(
        TransitionTrackId::new(1),
        PropertyKey::from_text("opacity").unwrap(),
        InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
        InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
        AnimationDuration::from_secs(1.0).unwrap(),
        AnimationDelay::from_secs(delay).unwrap(),
        easing,
    )
    .unwrap()
}

fn opacity(track: &TransitionTrack, elapsed: f64) -> f64 {
    let sample = track.sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    );
    let SampleValue::Value(value) = sample.value() else {
        panic!("expected a sampled value");
    };
    let InterpolableValue::Number(number) = value.value() else {
        panic!("expected a sampled opacity number");
    };
    number.value()
}
