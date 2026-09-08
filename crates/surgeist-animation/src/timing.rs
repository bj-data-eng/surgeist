//! Timing parameters and phase sampling.
//!
//! Timing accepts validated delays, durations, iteration counts, directions,
//! fill modes, play state, and easing. It derives phase, progress, directed
//! progress, eased progress, and before-flag data from effective elapsed time.

use core::fmt;

use crate::{
    AnimationDelay, AnimationDuration, EasedProgress, Easing, ElapsedTime, NormalizedProgress,
    NumericError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPlayState {
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingPhase {
    Idle,
    Before,
    Active,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IterationCount {
    value: IterationCountValue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrentIteration {
    value: CurrentIterationValue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum IterationCountValue {
    Finite(f64),
    Infinite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CurrentIterationValue {
    Finite(f64),
    Infinite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseProgress {
    value: NormalizedProgress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimingError {
    NonFiniteIterationCount {
        value: f64,
    },
    NegativeIterationCount {
        value: f64,
    },
    UnrepresentableActiveDuration {
        duration: AnimationDuration,
        iterations: f64,
    },
}

impl fmt::Display for TimingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NonFiniteIterationCount { value } => {
                write!(f, "iteration count must be finite, got {value}")
            }
            Self::NegativeIterationCount { value } => {
                write!(f, "iteration count must be non-negative, got {value}")
            }
            Self::UnrepresentableActiveDuration {
                duration,
                iterations,
            } => write!(
                f,
                "active duration {} seconds with {iterations} iterations is not representable",
                duration.as_secs()
            ),
        }
    }
}

impl std::error::Error for TimingError {}

/// Validated timing parameters for animation sampling.
///
/// ```
/// use surgeist_animation::{
///     AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, Easing,
///     ElapsedTime, FillMode, IterationCount, TimingParameters,
/// };
///
/// let timing = TimingParameters::try_new(
///     AnimationDelay::from_secs(0.0).unwrap(),
///     AnimationDuration::from_secs(1.0).unwrap(),
///     IterationCount::finite(1.0).unwrap(),
///     AnimationDirection::Normal,
///     FillMode::Both,
///     Easing::linear(),
/// )
/// .unwrap();
/// let sample = timing.sample(
///     ElapsedTime::from_secs(0.0).unwrap(),
///     AnimationPlayState::Running,
/// );
/// assert_eq!(sample.active_duration_secs(), Some(1.0));
/// ```
///
/// ```compile_fail
/// use surgeist_animation::{
///     AnimationDelay, AnimationDirection, AnimationDuration, Easing, FillMode, IterationCount,
///     TimingParameters,
/// };
///
/// let _ = TimingParameters::new(
///     AnimationDelay::from_secs(0.0).unwrap(),
///     AnimationDuration::from_secs(1.0).unwrap(),
///     IterationCount::finite(1.0).unwrap(),
///     AnimationDirection::Normal,
///     FillMode::Both,
///     Easing::linear(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TimingParameters {
    delay: AnimationDelay,
    duration: AnimationDuration,
    iterations: IterationCount,
    direction: AnimationDirection,
    fill_mode: FillMode,
    easing: Easing,
    active_duration: ActiveDuration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingSample {
    phase: TimingPhase,
    play_state: AnimationPlayState,
    local_time_secs: Option<f64>,
    active_duration_secs: Option<f64>,
    current_iteration: Option<CurrentIteration>,
    simple_progress: Option<PhaseProgress>,
    directed_progress: Option<PhaseProgress>,
    eased_progress: Option<EasedProgress>,
    easing_before_flag: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ActiveDuration {
    Finite(f64),
    Unbounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurrentDirection {
    Forwards,
    Reverse,
}

impl IterationCount {
    pub fn finite(value: f64) -> Result<Self, TimingError> {
        if !value.is_finite() {
            return Err(TimingError::NonFiniteIterationCount { value });
        }

        if value < 0.0 {
            return Err(TimingError::NegativeIterationCount { value });
        }

        Ok(Self {
            value: IterationCountValue::Finite(value),
        })
    }

    pub const fn infinite() -> Self {
        Self {
            value: IterationCountValue::Infinite,
        }
    }

    pub(crate) const fn once() -> Self {
        Self {
            value: IterationCountValue::Finite(1.0),
        }
    }

    pub const fn is_infinite(self) -> bool {
        matches!(self.value, IterationCountValue::Infinite)
    }

    pub const fn finite_value(self) -> Option<f64> {
        match self.value {
            IterationCountValue::Finite(value) => Some(value),
            IterationCountValue::Infinite => None,
        }
    }
}

impl CurrentIteration {
    pub(crate) const fn new(index: f64) -> Self {
        Self {
            value: CurrentIterationValue::Finite(index),
        }
    }

    pub(crate) const fn infinite() -> Self {
        Self {
            value: CurrentIterationValue::Infinite,
        }
    }

    pub const fn finite_index(self) -> Option<f64> {
        match self.value {
            CurrentIterationValue::Finite(index) => Some(index),
            CurrentIterationValue::Infinite => None,
        }
    }

    pub const fn is_infinite(self) -> bool {
        matches!(self.value, CurrentIterationValue::Infinite)
    }
}

impl PhaseProgress {
    pub fn new(value: f64) -> Result<Self, NumericError> {
        NormalizedProgress::new(value).map(Self::from_normalized)
    }

    pub const fn from_normalized(value: NormalizedProgress) -> Self {
        Self { value }
    }

    pub const fn value(self) -> f64 {
        self.value.value()
    }

    pub const fn normalized(self) -> NormalizedProgress {
        self.value
    }
}

impl TimingParameters {
    pub fn try_new(
        delay: AnimationDelay,
        duration: AnimationDuration,
        iterations: IterationCount,
        direction: AnimationDirection,
        fill_mode: FillMode,
        easing: Easing,
    ) -> Result<Self, TimingError> {
        let active_duration = ActiveDuration::try_new(duration, iterations)?;

        Ok(Self::from_validated_parts(
            delay,
            duration,
            iterations,
            direction,
            fill_mode,
            easing,
            active_duration,
        ))
    }

    pub(crate) fn for_transition(
        delay: AnimationDelay,
        duration: AnimationDuration,
        easing: Easing,
    ) -> Self {
        Self::from_validated_parts(
            delay,
            duration,
            IterationCount::once(),
            AnimationDirection::Normal,
            FillMode::Backwards,
            easing,
            ActiveDuration::Finite(duration.as_secs()),
        )
    }

    pub(crate) fn from_validated_parts(
        delay: AnimationDelay,
        duration: AnimationDuration,
        iterations: IterationCount,
        direction: AnimationDirection,
        fill_mode: FillMode,
        easing: Easing,
        active_duration: ActiveDuration,
    ) -> Self {
        Self {
            delay,
            duration,
            iterations,
            direction,
            fill_mode,
            easing,
            active_duration,
        }
    }

    pub(crate) fn validated_active_duration(
        duration: AnimationDuration,
        iterations: IterationCount,
    ) -> Result<ActiveDuration, TimingError> {
        ActiveDuration::try_new(duration, iterations)
    }

    pub(crate) const fn delay(&self) -> AnimationDelay {
        self.delay
    }

    pub(crate) const fn duration(&self) -> AnimationDuration {
        self.duration
    }

    pub const fn easing(&self) -> &Easing {
        &self.easing
    }

    pub fn sample(&self, elapsed: ElapsedTime, play_state: AnimationPlayState) -> TimingSample {
        let active_duration = self.active_duration;
        let active_duration_secs = active_duration.representable_secs();
        let local_time = checked_local_time_secs(elapsed, self.delay);

        let Some(local_time_secs) = local_time else {
            return TimingSample::new(
                active_duration.phase_for_positive_overflow(),
                play_state,
                None,
                active_duration_secs,
            )
            .with_timing_progress(self);
        };

        TimingSample::new(
            active_duration.phase_for_local_time(local_time_secs),
            play_state,
            Some(local_time_secs),
            active_duration_secs,
        )
        .with_timing_progress(self)
    }
}

impl TimingSample {
    pub const fn idle(play_state: AnimationPlayState) -> Self {
        Self {
            phase: TimingPhase::Idle,
            play_state,
            local_time_secs: None,
            active_duration_secs: None,
            current_iteration: None,
            simple_progress: None,
            directed_progress: None,
            eased_progress: None,
            easing_before_flag: false,
        }
    }

    const fn new(
        phase: TimingPhase,
        play_state: AnimationPlayState,
        local_time_secs: Option<f64>,
        active_duration_secs: Option<f64>,
    ) -> Self {
        Self {
            phase,
            play_state,
            local_time_secs,
            active_duration_secs,
            current_iteration: None,
            simple_progress: None,
            directed_progress: None,
            eased_progress: None,
            easing_before_flag: false,
        }
    }

    pub const fn phase(self) -> TimingPhase {
        self.phase
    }

    pub const fn play_state(self) -> AnimationPlayState {
        self.play_state
    }

    pub const fn local_time_secs(self) -> Option<f64> {
        self.local_time_secs
    }

    pub const fn active_duration_secs(self) -> Option<f64> {
        self.active_duration_secs
    }

    pub const fn current_iteration(self) -> Option<CurrentIteration> {
        self.current_iteration
    }

    pub const fn simple_progress(self) -> Option<PhaseProgress> {
        self.simple_progress
    }

    pub const fn directed_progress(self) -> Option<PhaseProgress> {
        self.directed_progress
    }

    pub const fn eased_progress(self) -> Option<EasedProgress> {
        self.eased_progress
    }

    pub const fn easing_before_flag(self) -> bool {
        self.easing_before_flag
    }

    pub const fn is_finished(self) -> bool {
        matches!(self.phase, TimingPhase::After)
    }

    fn with_timing_progress(self, timing: &TimingParameters) -> Self {
        match self.phase {
            TimingPhase::Before => self.with_before_fill_progress(timing),
            TimingPhase::Active => self.with_active_progress(timing),
            TimingPhase::After => self.with_after_fill_progress(timing),
            TimingPhase::Idle => self,
        }
    }

    fn with_active_progress(mut self, timing: &TimingParameters) -> Self {
        let Some(local_time_secs) = self.local_time_secs else {
            return self;
        };

        let duration_secs = timing.duration.as_secs();
        if duration_secs == 0.0 || timing.iterations.finite_value() == Some(0.0) {
            return self;
        }

        let overall_progress = local_time_secs / duration_secs;
        if !overall_progress.is_finite() {
            return self;
        }

        let current_iteration_index = overall_progress.floor();
        let simple_progress_value = overall_progress - current_iteration_index;
        let Ok(simple_progress) = PhaseProgress::new(simple_progress_value) else {
            return self;
        };

        let current_iteration = CurrentIteration::new(current_iteration_index);
        let (directed_progress, _) =
            directed_progress(timing.direction, current_iteration, simple_progress);
        let eased_progress = timing
            .easing
            .evaluate_with_before_flag(directed_progress.normalized(), false);

        self.current_iteration = Some(current_iteration);
        self.simple_progress = Some(simple_progress);
        self.directed_progress = Some(directed_progress);
        self.eased_progress = Some(eased_progress);
        self
    }

    fn with_before_fill_progress(mut self, timing: &TimingParameters) -> Self {
        if !matches!(timing.fill_mode, FillMode::Backwards | FillMode::Both) {
            return self;
        }

        let current_iteration = CurrentIteration::new(0.0);
        let simple_progress = PhaseProgress::new(0.0).unwrap();
        let (directed_progress, current_direction) =
            directed_progress(timing.direction, current_iteration, simple_progress);
        let before_flag = current_direction == CurrentDirection::Forwards;
        let eased_progress = timing
            .easing
            .evaluate_with_before_flag(directed_progress.normalized(), before_flag);

        self.current_iteration = Some(current_iteration);
        self.simple_progress = Some(simple_progress);
        self.directed_progress = Some(directed_progress);
        self.eased_progress = Some(eased_progress);
        self.easing_before_flag = before_flag;
        self
    }

    fn with_after_fill_progress(mut self, timing: &TimingParameters) -> Self {
        if !matches!(timing.fill_mode, FillMode::Forwards | FillMode::Both) {
            return self;
        }

        let Some((current_iteration, simple_progress)) = final_iteration_progress(timing) else {
            return self;
        };

        let (directed_progress, current_direction) =
            directed_progress(timing.direction, current_iteration, simple_progress);
        let before_flag = current_direction == CurrentDirection::Reverse;
        let eased_progress = timing
            .easing
            .evaluate_with_before_flag(directed_progress.normalized(), before_flag);

        self.current_iteration = Some(current_iteration);
        self.simple_progress = Some(simple_progress);
        self.directed_progress = Some(directed_progress);
        self.eased_progress = Some(eased_progress);
        self.easing_before_flag = before_flag;
        self
    }
}

impl ActiveDuration {
    fn try_new(
        duration: AnimationDuration,
        iterations: IterationCount,
    ) -> Result<Self, TimingError> {
        match iterations.value {
            IterationCountValue::Finite(iterations) => {
                let active_duration = duration.as_secs() * iterations;

                if active_duration.is_finite() {
                    Ok(Self::Finite(active_duration))
                } else {
                    Err(TimingError::UnrepresentableActiveDuration {
                        duration,
                        iterations,
                    })
                }
            }
            IterationCountValue::Infinite if duration.is_zero() => Ok(Self::Finite(0.0)),
            IterationCountValue::Infinite => Ok(Self::Unbounded),
        }
    }

    const fn representable_secs(self) -> Option<f64> {
        match self {
            Self::Finite(secs) => Some(secs),
            Self::Unbounded => None,
        }
    }

    const fn phase_for_local_time(self, local_time_secs: f64) -> TimingPhase {
        if local_time_secs < 0.0 {
            return TimingPhase::Before;
        }

        match self {
            Self::Finite(0.0) => TimingPhase::After,
            Self::Finite(active_duration_secs) if local_time_secs < active_duration_secs => {
                TimingPhase::Active
            }
            Self::Finite(_) => TimingPhase::After,
            Self::Unbounded => TimingPhase::Active,
        }
    }

    const fn phase_for_positive_overflow(self) -> TimingPhase {
        match self {
            Self::Finite(_) => TimingPhase::After,
            Self::Unbounded => TimingPhase::Active,
        }
    }
}

fn checked_local_time_secs(elapsed: ElapsedTime, delay: AnimationDelay) -> Option<f64> {
    let local_time_secs = elapsed.as_secs() - delay.as_secs();

    if local_time_secs.is_finite() {
        Some(local_time_secs)
    } else {
        None
    }
}

fn current_direction(
    direction: AnimationDirection,
    iteration: CurrentIteration,
) -> CurrentDirection {
    match direction {
        AnimationDirection::Normal => CurrentDirection::Forwards,
        AnimationDirection::Reverse => CurrentDirection::Reverse,
        AnimationDirection::Alternate | AnimationDirection::AlternateReverse
            if iteration.is_infinite() =>
        {
            CurrentDirection::Forwards
        }
        AnimationDirection::Alternate if is_odd_iteration(iteration) => CurrentDirection::Reverse,
        AnimationDirection::Alternate => CurrentDirection::Forwards,
        AnimationDirection::AlternateReverse if is_odd_iteration(iteration) => {
            CurrentDirection::Forwards
        }
        AnimationDirection::AlternateReverse => CurrentDirection::Reverse,
    }
}

fn directed_progress(
    direction: AnimationDirection,
    iteration: CurrentIteration,
    simple_progress: PhaseProgress,
) -> (PhaseProgress, CurrentDirection) {
    let current_direction = current_direction(direction, iteration);
    let directed_progress = match current_direction {
        CurrentDirection::Forwards => simple_progress,
        CurrentDirection::Reverse => PhaseProgress::new(1.0 - simple_progress.value()).unwrap(),
    };

    (directed_progress, current_direction)
}

fn is_odd_iteration(iteration: CurrentIteration) -> bool {
    match iteration.finite_index() {
        Some(index) => index.rem_euclid(2.0) == 1.0,
        None => false,
    }
}

fn final_iteration_progress(
    timing: &TimingParameters,
) -> Option<(CurrentIteration, PhaseProgress)> {
    match timing.iterations.value {
        IterationCountValue::Finite(0.0) => {
            Some((CurrentIteration::new(0.0), PhaseProgress::new(0.0).unwrap()))
        }
        IterationCountValue::Finite(iterations) => {
            let simple_progress_value = iterations.fract();
            let (iteration_index, simple_progress_value) = if simple_progress_value == 0.0 {
                (iterations - 1.0, 1.0)
            } else {
                (iterations.floor(), simple_progress_value)
            };

            let simple_progress = PhaseProgress::new(simple_progress_value).ok()?;
            Some((CurrentIteration::new(iteration_index), simple_progress))
        }
        IterationCountValue::Infinite if timing.duration.is_zero() => Some((
            CurrentIteration::infinite(),
            PhaseProgress::new(1.0).unwrap(),
        )),
        IterationCountValue::Infinite => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnimationDirection, AnimationPlayState, CurrentDirection, CurrentIteration, FillMode,
        IterationCount, PhaseProgress, TimingError, TimingParameters, TimingPhase, TimingSample,
        current_direction,
    };
    use crate::{AnimationDelay, AnimationDuration, Easing, ElapsedTime};

    fn timing(delay_secs: f64, duration_secs: f64, iterations: IterationCount) -> TimingParameters {
        TimingParameters::try_new(
            AnimationDelay::from_secs(delay_secs).unwrap(),
            AnimationDuration::from_secs(duration_secs).unwrap(),
            iterations,
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap()
    }

    #[test]
    fn iteration_count_accepts_zero_fractional_and_infinite_values() {
        assert_eq!(
            IterationCount::finite(0.0).unwrap().finite_value(),
            Some(0.0)
        );
        assert_eq!(
            IterationCount::finite(2.5).unwrap().finite_value(),
            Some(2.5)
        );
        assert!(IterationCount::infinite().is_infinite());
    }

    #[test]
    fn iteration_count_rejects_negative_nan_and_infinite_finite_values() {
        assert_eq!(
            IterationCount::finite(-0.1),
            Err(TimingError::NegativeIterationCount { value: -0.1 })
        );
        assert!(matches!(
            IterationCount::finite(f64::NAN),
            Err(TimingError::NonFiniteIterationCount { value }) if value.is_nan()
        ));
        assert_eq!(
            IterationCount::finite(f64::INFINITY),
            Err(TimingError::NonFiniteIterationCount {
                value: f64::INFINITY
            })
        );
    }

    #[test]
    fn classifies_before_active_and_after_phases() {
        let timing = timing(1.0, 2.0, IterationCount::finite(1.0).unwrap());

        assert_eq!(
            timing
                .sample(
                    ElapsedTime::from_secs(0.5).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::Before
        );
        assert_eq!(
            timing
                .sample(
                    ElapsedTime::from_secs(1.0).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::Active
        );
        assert_eq!(
            timing
                .sample(
                    ElapsedTime::from_secs(3.0).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::After
        );
    }

    #[test]
    fn negative_delay_can_start_active_or_after_at_elapsed_zero() {
        let active = timing(-1.0, 2.0, IterationCount::finite(2.0).unwrap());
        let after = timing(-5.0, 2.0, IterationCount::finite(2.0).unwrap());

        assert_eq!(
            active
                .sample(
                    ElapsedTime::from_secs(0.0).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::Active
        );
        assert_eq!(
            after
                .sample(
                    ElapsedTime::from_secs(0.0).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::After
        );
    }

    #[test]
    fn zero_duration_and_zero_iterations_finish_at_delay_boundary() {
        let zero_duration = timing(1.0, 0.0, IterationCount::finite(3.0).unwrap());
        let zero_iterations = timing(1.0, 2.0, IterationCount::finite(0.0).unwrap());

        assert_eq!(
            zero_duration
                .sample(
                    ElapsedTime::from_secs(1.0).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::After
        );
        assert_eq!(
            zero_iterations
                .sample(
                    ElapsedTime::from_secs(1.0).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::After
        );
    }

    #[test]
    fn infinite_iterations_remain_active_after_delay_when_duration_is_positive() {
        let timing = timing(0.25, 0.5, IterationCount::infinite());

        assert_eq!(
            timing
                .sample(
                    ElapsedTime::from_secs(1.0e6).unwrap(),
                    AnimationPlayState::Running
                )
                .phase(),
            TimingPhase::Active
        );
    }

    #[test]
    fn zero_duration_with_infinite_iterations_has_zero_length_active_interval() {
        let timing = timing(1.0, 0.0, IterationCount::infinite());
        let sample = timing.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(sample.active_duration_secs(), Some(0.0));
    }

    #[test]
    fn timing_accepts_max_duration_with_one_iteration() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(f64::MAX).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap();

        assert_eq!(
            timing
                .sample(
                    ElapsedTime::from_secs(0.0).unwrap(),
                    AnimationPlayState::Running
                )
                .active_duration_secs(),
            Some(f64::MAX)
        );
    }

    #[test]
    fn finite_timing_is_never_reported_as_unbounded() {
        let timing = timing(0.0, f64::MAX, IterationCount::finite(1.0).unwrap());
        let sample = timing.sample(
            ElapsedTime::from_secs(f64::MAX).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(sample.active_duration_secs(), Some(f64::MAX));
    }

    #[test]
    fn timing_rejects_unrepresentable_finite_active_duration() {
        let duration = AnimationDuration::from_secs(f64::MAX).unwrap();
        let iterations = IterationCount::finite(2.0).unwrap();

        assert_eq!(
            TimingParameters::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                duration,
                iterations,
                AnimationDirection::Normal,
                FillMode::None,
                Easing::linear(),
            ),
            Err(TimingError::UnrepresentableActiveDuration {
                duration,
                iterations: 2.0,
            })
        );
    }

    #[test]
    fn overflowing_adjusted_time_finishes_representable_finite_timing() {
        let timing = timing(-f64::MAX, f64::MAX, IterationCount::finite(1.0).unwrap());
        let sample = timing.sample(
            ElapsedTime::from_secs(f64::MAX).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(sample.local_time_secs(), None);
        assert_eq!(sample.active_duration_secs(), Some(f64::MAX));
    }

    #[test]
    fn infinite_iterations_remain_the_only_positive_unbounded_extent() {
        let finite = timing(0.0, f64::MAX, IterationCount::finite(1.0).unwrap());
        let zero_duration_infinite = timing(0.0, 0.0, IterationCount::infinite());
        let positive_duration_infinite = timing(0.0, 1.0, IterationCount::infinite());

        assert_eq!(
            finite
                .sample(
                    ElapsedTime::from_secs(1.0).unwrap(),
                    AnimationPlayState::Running
                )
                .active_duration_secs(),
            Some(f64::MAX)
        );
        assert_eq!(
            zero_duration_infinite
                .sample(
                    ElapsedTime::from_secs(0.0).unwrap(),
                    AnimationPlayState::Running
                )
                .active_duration_secs(),
            Some(0.0)
        );
        assert_eq!(
            positive_duration_infinite
                .sample(
                    ElapsedTime::from_secs(f64::MAX).unwrap(),
                    AnimationPlayState::Running
                )
                .active_duration_secs(),
            None
        );
        assert_eq!(
            positive_duration_infinite
                .sample(
                    ElapsedTime::from_secs(f64::MAX).unwrap(),
                    AnimationPlayState::Running,
                )
                .phase(),
            TimingPhase::Active
        );
    }

    #[test]
    fn active_sample_reports_iteration_simple_directed_and_eased_progress() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            IterationCount::finite(3.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(2.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Active);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(1.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(0.25)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.25)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(0.25)
        );
    }

    #[test]
    fn exact_interior_iteration_boundary_starts_next_iteration() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Active);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(1.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(0.0)
        );
    }

    #[test]
    fn fractional_iteration_count_reports_active_fractional_progress() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            IterationCount::finite(2.5).unwrap(),
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(4.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Active);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(2.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(0.25)
        );
    }

    #[test]
    fn infinite_iterations_use_finite_current_iteration_while_active() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            IterationCount::infinite(),
            AnimationDirection::Alternate,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(5.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Active);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(2.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(0.5)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.5)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(0.5)
        );
    }

    #[test]
    fn direction_variants_transform_simple_progress_by_even_iteration_index() {
        let normal = timing_with_direction(AnimationDirection::Normal);
        let reverse = timing_with_direction(AnimationDirection::Reverse);
        let alternate = timing_with_direction(AnimationDirection::Alternate);
        let alternate_reverse = timing_with_direction(AnimationDirection::AlternateReverse);
        let elapsed = ElapsedTime::from_secs(0.25).unwrap();

        assert_eq!(
            normal
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.25)
        );
        assert_eq!(
            reverse
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.75)
        );
        assert_eq!(
            alternate
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.25)
        );
        assert_eq!(
            alternate_reverse
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.75)
        );
    }

    #[test]
    fn direction_variants_transform_simple_progress_by_odd_iteration_index() {
        let normal = timing_with_direction(AnimationDirection::Normal);
        let reverse = timing_with_direction(AnimationDirection::Reverse);
        let alternate = timing_with_direction(AnimationDirection::Alternate);
        let alternate_reverse = timing_with_direction(AnimationDirection::AlternateReverse);
        let elapsed = ElapsedTime::from_secs(1.25).unwrap();

        assert_eq!(
            normal
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.25)
        );
        assert_eq!(
            reverse
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.75)
        );
        assert_eq!(
            alternate
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.75)
        );
        assert_eq!(
            alternate_reverse
                .sample(elapsed, AnimationPlayState::Running)
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.25)
        );
    }

    #[test]
    fn non_finite_overall_progress_does_not_construct_progress_fields() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(f64::MIN_POSITIVE).unwrap(),
            IterationCount::infinite(),
            AnimationDirection::Normal,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(f64::MAX).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Active);
        assert_eq!(sample.current_iteration(), None);
        assert_eq!(sample.simple_progress(), None);
        assert_eq!(sample.directed_progress(), None);
        assert_eq!(sample.eased_progress(), None);
    }

    #[test]
    fn infinite_current_iteration_uses_forwards_direction_for_alternate_modes() {
        assert_eq!(
            current_direction(AnimationDirection::Alternate, CurrentIteration::infinite()),
            CurrentDirection::Forwards
        );
        assert_eq!(
            current_direction(
                AnimationDirection::AlternateReverse,
                CurrentIteration::infinite()
            ),
            CurrentDirection::Forwards
        );
    }

    #[test]
    fn fill_modes_control_before_and_after_progress_exposure() {
        let before_none = timing_with_fill(FillMode::None).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let before_backwards = timing_with_fill(FillMode::Backwards).sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let after_backwards = timing_with_fill(FillMode::Backwards).sample(
            ElapsedTime::from_secs(3.0).unwrap(),
            AnimationPlayState::Running,
        );
        let after_forwards = timing_with_fill(FillMode::Forwards).sample(
            ElapsedTime::from_secs(3.0).unwrap(),
            AnimationPlayState::Running,
        );
        let both = timing_with_fill(FillMode::Both);

        assert_eq!(before_none.directed_progress(), None);
        assert_eq!(
            before_backwards
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(after_backwards.directed_progress(), None);
        assert_eq!(
            after_forwards.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            both.sample(
                ElapsedTime::from_secs(0.5).unwrap(),
                AnimationPlayState::Running
            )
            .directed_progress()
            .map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            both.sample(
                ElapsedTime::from_secs(3.0).unwrap(),
                AnimationPlayState::Running
            )
            .directed_progress()
            .map(PhaseProgress::value),
            Some(1.0)
        );
    }

    #[test]
    fn backwards_fill_uses_before_flag_for_step_easing() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Backwards,
            Easing::step_start(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Before);
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(0.0)
        );
    }

    #[test]
    fn reverse_backwards_fill_does_not_set_before_flag() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Reverse,
            FillMode::Backwards,
            Easing::step_end(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Before);
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(1.0)
        );
    }

    #[test]
    fn alternate_reverse_backwards_fill_does_not_set_before_flag_on_first_iteration() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.0).unwrap(),
            AnimationDirection::AlternateReverse,
            FillMode::Backwards,
            Easing::step_end(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::Before);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(0.0)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(1.0)
        );
    }

    #[test]
    fn forwards_fill_uses_fractional_iteration_end_progress() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(2.0).unwrap(),
            IterationCount::finite(2.5).unwrap(),
            AnimationDirection::Normal,
            FillMode::Forwards,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(5.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(2.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(0.5)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.5)
        );
    }

    #[test]
    fn after_fill_uses_before_flag_when_current_direction_is_reverse() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Reverse,
            FillMode::Forwards,
            Easing::step_start(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(0.0)
        );
    }

    #[test]
    fn alternate_integer_after_fill_uses_reverse_final_direction() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.0).unwrap(),
            AnimationDirection::Alternate,
            FillMode::Both,
            Easing::step_start(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(2.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(1.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(0.0)
        );
    }

    #[test]
    fn alternate_reverse_integer_after_fill_uses_forwards_final_direction() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.0).unwrap(),
            AnimationDirection::AlternateReverse,
            FillMode::Forwards,
            Easing::step_start(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(2.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(1.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            sample.eased_progress().map(|progress| progress.value()),
            Some(1.0)
        );
    }

    #[test]
    fn after_fill_applies_direction_for_fractional_final_iterations() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.25).unwrap(),
            AnimationDirection::AlternateReverse,
            FillMode::Forwards,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(2.25).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(2.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(0.25)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(0.75)
        );
    }

    #[test]
    fn negative_delay_skipping_to_after_phase_can_still_fill_forwards() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(-5.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(1.0)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
    }

    #[test]
    fn zero_duration_positive_iterations_can_fill_at_final_endpoint() {
        let timing = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(0.0).unwrap(),
            IterationCount::finite(3.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Forwards,
            Easing::linear(),
        )
        .unwrap();

        let sample = timing.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(sample.phase(), TimingPhase::After);
        assert_eq!(
            sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(2.0)
        );
        assert_eq!(
            sample.simple_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
    }

    #[test]
    fn zero_iterations_forwards_and_both_fill_use_zero_progress_current_iteration_0() {
        let forwards = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(0.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Forwards,
            Easing::linear(),
        )
        .unwrap();
        let both = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(0.0).unwrap(),
            AnimationDirection::Alternate,
            FillMode::Both,
            Easing::linear(),
        )
        .unwrap();
        let elapsed = ElapsedTime::from_secs(0.0).unwrap();

        let forwards_sample = forwards.sample(elapsed, AnimationPlayState::Running);
        let both_sample = both.sample(elapsed, AnimationPlayState::Running);

        assert_eq!(forwards_sample.phase(), TimingPhase::After);
        assert_eq!(both_sample.phase(), TimingPhase::After);
        assert_eq!(
            forwards_sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(0.0)
        );
        assert_eq!(
            both_sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(0.0)
        );
        assert_eq!(
            forwards_sample.simple_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            both_sample.simple_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            forwards_sample
                .directed_progress()
                .map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            both_sample.directed_progress().map(PhaseProgress::value),
            Some(0.0)
        );
    }

    #[test]
    fn zero_iterations_reverse_and_alternate_reverse_use_directed_progress_1() {
        let reverse = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(0.0).unwrap(),
            AnimationDirection::Reverse,
            FillMode::Forwards,
            Easing::linear(),
        )
        .unwrap();
        let alternate_reverse = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(0.0).unwrap(),
            AnimationDirection::AlternateReverse,
            FillMode::Both,
            Easing::linear(),
        )
        .unwrap();
        let elapsed = ElapsedTime::from_secs(0.0).unwrap();

        let reverse_sample = reverse.sample(elapsed, AnimationPlayState::Running);
        let alternate_reverse_sample =
            alternate_reverse.sample(elapsed, AnimationPlayState::Running);

        assert_eq!(reverse_sample.phase(), TimingPhase::After);
        assert_eq!(alternate_reverse_sample.phase(), TimingPhase::After);
        assert_eq!(
            reverse_sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(0.0)
        );
        assert_eq!(
            alternate_reverse_sample
                .current_iteration()
                .and_then(CurrentIteration::finite_index),
            Some(0.0)
        );
        assert_eq!(
            reverse_sample.simple_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            alternate_reverse_sample
                .simple_progress()
                .map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            reverse_sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            alternate_reverse_sample
                .directed_progress()
                .map(PhaseProgress::value),
            Some(1.0)
        );
    }

    #[test]
    fn zero_iteration_step_easing_respects_direction_and_before_flag() {
        let normal = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(0.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Forwards,
            Easing::step_start(),
        )
        .unwrap();
        let reverse = TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(0.0).unwrap(),
            AnimationDirection::Reverse,
            FillMode::Forwards,
            Easing::step_end(),
        )
        .unwrap();
        let elapsed = ElapsedTime::from_secs(0.0).unwrap();

        let normal_sample = normal.sample(elapsed, AnimationPlayState::Running);
        let reverse_sample = reverse.sample(elapsed, AnimationPlayState::Running);

        assert_eq!(normal_sample.phase(), TimingPhase::After);
        assert!(!normal_sample.easing_before_flag());
        assert_eq!(
            normal_sample.directed_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            normal_sample
                .eased_progress()
                .map(|progress| progress.value()),
            Some(1.0)
        );
        assert_eq!(reverse_sample.phase(), TimingPhase::After);
        assert!(reverse_sample.easing_before_flag());
        assert_eq!(
            reverse_sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            reverse_sample
                .eased_progress()
                .map(|progress| progress.value()),
            Some(0.0)
        );
    }

    #[test]
    fn zero_duration_infinite_iterations_fill_with_infinite_current_iteration() {
        let normal = zero_duration_infinite_timing(AnimationDirection::Normal);
        let reverse = zero_duration_infinite_timing(AnimationDirection::Reverse);
        let alternate = zero_duration_infinite_timing(AnimationDirection::Alternate);
        let alternate_reverse = zero_duration_infinite_timing(AnimationDirection::AlternateReverse);
        let elapsed = ElapsedTime::from_secs(0.0).unwrap();

        let normal_sample = normal.sample(elapsed, AnimationPlayState::Running);
        let reverse_sample = reverse.sample(elapsed, AnimationPlayState::Running);
        let alternate_sample = alternate.sample(elapsed, AnimationPlayState::Running);
        let alternate_reverse_sample =
            alternate_reverse.sample(elapsed, AnimationPlayState::Running);

        assert_eq!(normal_sample.phase(), TimingPhase::After);
        assert_eq!(reverse_sample.phase(), TimingPhase::After);
        assert_eq!(alternate_sample.phase(), TimingPhase::After);
        assert_eq!(alternate_reverse_sample.phase(), TimingPhase::After);
        assert_eq!(normal_sample.active_duration_secs(), Some(0.0));
        assert_eq!(reverse_sample.active_duration_secs(), Some(0.0));
        assert_eq!(alternate_sample.active_duration_secs(), Some(0.0));
        assert_eq!(alternate_reverse_sample.active_duration_secs(), Some(0.0));
        assert!(normal_sample.current_iteration().unwrap().is_infinite());
        assert!(reverse_sample.current_iteration().unwrap().is_infinite());
        assert!(alternate_sample.current_iteration().unwrap().is_infinite());
        assert!(
            alternate_reverse_sample
                .current_iteration()
                .unwrap()
                .is_infinite()
        );
        assert_eq!(
            normal_sample.simple_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            reverse_sample.simple_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            alternate_sample.simple_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            alternate_reverse_sample
                .simple_progress()
                .map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            normal_sample.directed_progress().map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            reverse_sample.directed_progress().map(PhaseProgress::value),
            Some(0.0)
        );
        assert_eq!(
            alternate_sample
                .directed_progress()
                .map(PhaseProgress::value),
            Some(1.0)
        );
        assert_eq!(
            alternate_reverse_sample
                .directed_progress()
                .map(PhaseProgress::value),
            Some(1.0)
        );
    }

    #[test]
    fn paused_play_state_is_reported_without_mutating_effective_elapsed_time() {
        let timing = timing_with_fill(FillMode::None);
        let running = timing.sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Running,
        );
        let paused = timing.sample(
            ElapsedTime::from_secs(1.5).unwrap(),
            AnimationPlayState::Paused,
        );

        assert_eq!(running.phase(), paused.phase());
        assert_eq!(running.directed_progress(), paused.directed_progress());
        assert_eq!(paused.play_state(), AnimationPlayState::Paused);
    }

    #[test]
    fn idle_sample_has_no_progress_and_is_not_finished() {
        let sample = TimingSample::idle(AnimationPlayState::Paused);

        assert_eq!(sample.phase(), TimingPhase::Idle);
        assert_eq!(sample.play_state(), AnimationPlayState::Paused);
        assert_eq!(sample.current_iteration(), None);
        assert_eq!(sample.directed_progress(), None);
        assert!(!sample.is_finished());
    }

    #[test]
    fn retained_timing_diagnostics_have_display_text() {
        assert_eq!(
            TimingError::NonFiniteIterationCount { value: f64::NAN }.to_string(),
            "iteration count must be finite, got NaN"
        );
        assert_eq!(
            TimingError::NegativeIterationCount { value: -1.0 }.to_string(),
            "iteration count must be non-negative, got -1"
        );

        let active_duration = TimingError::UnrepresentableActiveDuration {
            duration: AnimationDuration::from_secs(f64::MAX).unwrap(),
            iterations: 2.0,
        };
        let text = active_duration.to_string();
        assert!(text.contains("active duration"));
        assert!(text.contains("is not representable"));
        assert!(std::error::Error::source(&active_duration).is_none());
    }

    fn timing_with_direction(direction: AnimationDirection) -> TimingParameters {
        TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(3.0).unwrap(),
            direction,
            FillMode::None,
            Easing::linear(),
        )
        .unwrap()
    }

    fn timing_with_fill(fill_mode: FillMode) -> TimingParameters {
        TimingParameters::try_new(
            AnimationDelay::from_secs(1.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            fill_mode,
            Easing::linear(),
        )
        .unwrap()
    }

    fn zero_duration_infinite_timing(direction: AnimationDirection) -> TimingParameters {
        TimingParameters::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(0.0).unwrap(),
            IterationCount::infinite(),
            direction,
            FillMode::Both,
            Easing::linear(),
        )
        .unwrap()
    }
}
