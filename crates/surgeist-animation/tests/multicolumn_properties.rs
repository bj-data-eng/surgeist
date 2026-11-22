use surgeist_animation::computed::{
    ComputedComponent, ComputedInteger, ComputedNumeric, ComputedValue,
};
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    InterpolableValue, InterpolationOutcome, InterpolationPair, InterpolationProgress, PropertyKey,
};
use surgeist_animation::sample::SampleValue;
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::AnimationPlayState;
use surgeist_animation::transition::{TransitionTrack, TransitionTrackId};

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
    for (elapsed, expected) in [(0.5_f64.next_down(), 1.0), (0.5, 2.0), (0.5_f64.next_up(), 2.0)] {
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
    assert_eq!(pair.sample(progress), InterpolationOutcome::Value(count(3.0)));
}

fn count(value: f64) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Numeric(
        ComputedNumeric::integer(ComputedInteger::new(value).unwrap()),
    )))
}
