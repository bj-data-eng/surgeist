use surgeist_animation::easing::Easing;
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount, PhaseProgress,
    TimingParameters, TimingPhase,
};
fn sample(
    delay: f64,
    duration: f64,
    count: f64,
    elapsed: f64,
) -> surgeist_animation::timing::TimingSample {
    TimingParameters::try_new(
        AnimationDelay::from_secs(delay).unwrap(),
        AnimationDuration::from_secs(duration).unwrap(),
        IterationCount::finite(count).unwrap(),
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
#[test]
fn css_forward_fill_holds_the_last_iteration_despite_ratio_rounding() {
    let fill = sample(0.0, 0.1, 3.0, 0.4);
    assert_eq!(fill.phase(), TimingPhase::After);
    assert_eq!(fill.simple_progress().map(PhaseProgress::value), Some(1.0));
    assert_eq!(
        fill.current_iteration()
            .and_then(|index| index.finite_index()),
        Some(2.0)
    );
}
#[test]
fn css_positive_duration_survives_at_exact_start_with_a_tiny_interval() {
    let active = sample(1.0, 1e-16, 1.0, 1.0);
    assert_eq!(active.phase(), TimingPhase::Active);
    assert_eq!(
        active.simple_progress().map(PhaseProgress::value),
        Some(0.0)
    );
    assert_eq!(
        active
            .current_iteration()
            .and_then(|index| index.finite_index()),
        Some(0.0)
    );
}

#[test]
fn css_forward_fill_preserves_configured_progress_for_tiny_fractional_counts() {
    // The multiplied extent rounds to zero, but the configured fractional
    // iteration still determines the result of forward fill.
    let duration = f64::from_bits(1);
    let fill = sample(0.0, duration, 0.25, 0.0);
    assert_eq!(fill.phase(), TimingPhase::After);
    assert_eq!(fill.simple_progress().map(PhaseProgress::value), Some(0.25));
    assert_eq!(
        fill.current_iteration()
            .and_then(|index| index.finite_index()),
        Some(0.0)
    );
}
