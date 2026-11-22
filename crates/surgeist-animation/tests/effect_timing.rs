use surgeist_animation::easing::Easing;
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount, PhaseProgress,
    TimingParameters, TimingPhase, TimingSample,
};

#[test]
fn before_fill_keeps_the_animations_supplied_local_time() {
    let sample = sample(1.0, 2.0, 0.5);
    assert_eq!(sample.phase(), TimingPhase::Before);
    assert_eq!(
        sample.simple_progress().map(PhaseProgress::value),
        Some(0.0)
    );
    assert_eq!(sample.local_time_secs(), Some(0.5));
}

#[test]
fn active_progress_subtracts_the_delay_from_local_time_only_once() {
    let sample = sample(0.75, 1.5, 1.125);
    assert_eq!(sample.phase(), TimingPhase::Active);
    // The active interval begins at 0.75s, so 1.125s is one quarter through it.
    assert_eq!(
        sample.simple_progress().map(PhaseProgress::value),
        Some(0.25)
    );
    assert_eq!(sample.local_time_secs(), Some(1.125));
}

#[test]
fn after_phase_keeps_local_time_when_the_delay_adjustment_overflows() {
    let sample = sample(-f64::MAX, f64::MAX, f64::MAX);
    // The active interval's end is at zero. The supplied time stays finite
    // even though subtracting the negative delay cannot be expressed in f64.
    assert_eq!(sample.phase(), TimingPhase::After);
    assert_eq!(sample.local_time_secs(), Some(f64::MAX));
}

fn sample(delay: f64, duration: f64, elapsed: f64) -> TimingSample {
    TimingParameters::try_new(
        AnimationDelay::from_secs(delay).unwrap(),
        AnimationDuration::from_secs(duration).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
        Easing::linear(),
    )
    .unwrap()
    .sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    )
}
