//! Validated Web Animations millisecond timing and supplied animation observations.

use super::{
    ActiveDuration, AnimationDirection, CurrentIteration, EasedProgress, Easing, FillMode,
    IterationCount, PhaseProgress, TimingPhase, kernel,
};
use crate::time::{AnimationTime, PlaybackRate};
use crate::timeline::TimelineMonotonicity;

/// The numeric option rejected while constructing effect timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectTimingField {
    StartDelay,
    EndDelay,
    Duration,
    IterationStart,
}

/// A finite arithmetic limit, distinct from source-defined infinite duration/count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingOperation {
    ActiveDuration,
    EndTime,
    ActiveTime,
    OverallProgress,
    Progress,
}

/// A rejected effect option or finite timing calculation limit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectTimingError {
    kind: EffectTimingErrorKind,
}

/// The category retained by an effect timing diagnostic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectTimingErrorKind {
    InvalidOption {
        field: EffectTimingField,
        value: f64,
    },
    Unrepresentable {
        operation: TimingOperation,
    },
}

impl EffectTimingError {
    pub const fn kind(self) -> EffectTimingErrorKind {
        self.kind
    }
    pub(super) const fn unrepresentable(operation: TimingOperation) -> Self {
        Self {
            kind: EffectTimingErrorKind::Unrepresentable { operation },
        }
    }
}

impl core::fmt::Display for EffectTimingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            EffectTimingErrorKind::InvalidOption { field, value } => {
                write!(f, "invalid {field:?} effect timing value {value}")
            }
            EffectTimingErrorKind::Unrepresentable { operation } => {
                write!(f, "{operation:?} timing result is not representable")
            }
        }
    }
}

impl std::error::Error for EffectTimingError {}

/// Raw millisecond options validated together by [`EffectTiming::try_new`].
///
/// Delays and iteration start must be finite. Duration may be positive infinity;
/// duration and iteration start must be nonnegative. These are numeric model
/// options, without CSS parsing or an authored `auto` value.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectTimingOptions {
    pub start_delay_millis: f64,
    pub end_delay_millis: f64,
    pub duration_millis: f64,
    pub iterations: IterationCount,
    pub iteration_start: f64,
    pub direction: AnimationDirection,
    pub fill: FillMode,
    pub easing: Easing,
}

impl Default for EffectTimingOptions {
    fn default() -> Self {
        Self {
            start_delay_millis: 0.0,
            end_delay_millis: 0.0,
            duration_millis: 0.0,
            iterations: IterationCount::once(),
            iteration_start: 0.0,
            direction: AnimationDirection::Normal,
            fill: FillMode::None,
            easing: Easing::linear(),
        }
    }
}

/// Validated effect timing in the Web Animations model's millisecond time space.
///
/// ```
/// use surgeist_animation::time::{AnimationTime, PlaybackRate};
/// use surgeist_animation::timing::{
///     AnimationReplaceState, EffectAnimationState, EffectPlayState, EffectTiming,
///     EffectTimingInput, EffectTimingOptions, FillMode, PhaseProgress,
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let timing = EffectTiming::try_new(EffectTimingOptions {
///     start_delay_millis: 100.0, duration_millis: 200.0,
///     end_delay_millis: -50.0, fill: FillMode::Forwards,
///     ..EffectTimingOptions::default()
/// })?;
/// let sample = timing.sample(EffectTimingInput::Associated {
///     local_time: Some(AnimationTime::from_millis(250.0)?),
///     animation: EffectAnimationState::new(
///         PlaybackRate::new(1.0)?, EffectPlayState::Running,
///         None, AnimationReplaceState::Active,
///     ),
/// })?;
/// assert_eq!(sample.simple_progress().map(PhaseProgress::value), Some(0.75));
/// assert!(sample.is_in_effect());
/// # Ok(())
/// # }
/// ```
///
/// ```compile_fail,E0616
/// use surgeist_animation::timing::EffectTiming;
/// let mut timing = EffectTiming::default();
/// timing.end = -1.0;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EffectTiming {
    options: EffectTimingOptions,
    active_duration: ActiveDuration,
    end: f64,
    before: f64,
    after: f64,
}

impl Default for EffectTiming {
    fn default() -> Self {
        Self {
            options: EffectTimingOptions::default(),
            active_duration: ActiveDuration::Finite(0.0),
            end: 0.0,
            before: 0.0,
            after: 0.0,
        }
    }
}

impl EffectTiming {
    /// Validates all options and derives active duration and interval boundaries.
    ///
    /// # Errors
    ///
    /// Rejects invalid numeric options and unrepresentable finite active duration
    /// or end time. Mathematical infinity is retained where the model permits it.
    pub fn try_new(options: EffectTimingOptions) -> Result<Self, EffectTimingError> {
        for (field, value, valid) in [
            (
                EffectTimingField::StartDelay,
                options.start_delay_millis,
                options.start_delay_millis.is_finite(),
            ),
            (
                EffectTimingField::EndDelay,
                options.end_delay_millis,
                options.end_delay_millis.is_finite(),
            ),
            (
                EffectTimingField::Duration,
                options.duration_millis,
                !options.duration_millis.is_nan() && options.duration_millis >= 0.0,
            ),
            (
                EffectTimingField::IterationStart,
                options.iteration_start,
                options.iteration_start.is_finite() && options.iteration_start >= 0.0,
            ),
        ] {
            if !valid {
                return Err(EffectTimingError {
                    kind: EffectTimingErrorKind::InvalidOption { field, value },
                });
            }
        }
        let duration = options.duration_millis;
        let count = kernel::count_value(options.iterations);
        let active_duration = if duration == 0.0 || count == 0.0 {
            ActiveDuration::Finite(0.0)
        } else if duration == f64::INFINITY || count == f64::INFINITY {
            ActiveDuration::Unbounded
        } else {
            let product = duration * count;
            if !product.is_finite() || product == 0.0 {
                return Err(kernel::limit(TimingOperation::ActiveDuration));
            }
            ActiveDuration::Finite(product)
        };
        let mut timing = Self {
            options,
            active_duration,
            end: 0.0,
            before: 0.0,
            after: 0.0,
        };
        let bounds = kernel::Bounds::new(&timing.parameters());
        timing.end = bounds.end.value()?;
        timing.before = bounds.before;
        timing.after = bounds.after.value()?;
        Ok(timing)
    }
    pub const fn options(&self) -> &EffectTimingOptions {
        &self.options
    }
    pub const fn active_duration_millis(&self) -> f64 {
        kernel::extent_value(self.active_duration)
    }
    pub const fn end_time_millis(&self) -> f64 {
        self.end
    }
    pub const fn before_active_boundary_millis(&self) -> f64 {
        self.before
    }
    pub const fn active_after_boundary_millis(&self) -> f64 {
        self.after
    }

    /// Samples one effect from its associated animation's supplied current time.
    ///
    /// # Errors
    ///
    /// Returns a typed finite numeric limit rather than treating overflow as
    /// infinite progress or unresolved time. Sampling has no frame history.
    pub fn sample(
        &self,
        input: EffectTimingInput,
    ) -> Result<EffectTimingSample, EffectTimingError> {
        let (local_time, animation) = match input {
            EffectTimingInput::Unassociated => (None, None),
            EffectTimingInput::Associated {
                local_time,
                animation,
            } => (local_time, Some(animation)),
        };
        let backwards = animation.is_some_and(|state| state.playback_rate.value() < 0.0);
        let kernel = kernel::sample(
            &self.parameters(),
            local_time.map(AnimationTime::as_millis),
            backwards,
        );
        if let Some(error) = kernel.error {
            return Err(error);
        }
        Ok(EffectTimingSample {
            local_time,
            animation,
            kernel,
        })
    }
    fn parameters(&self) -> kernel::Parameters<'_> {
        kernel::Parameters {
            delay: self.options.start_delay_millis,
            end_delay: self.options.end_delay_millis,
            duration: self.options.duration_millis,
            iterations: self.options.iterations,
            iteration_start: self.options.iteration_start,
            direction: self.options.direction,
            fill: self.options.fill,
            easing: &self.options.easing,
            active_duration: self.active_duration,
        }
    }
}

/// The supplied animation's full play-state observation.
///
/// This is separate from the running/paused control of normalized CSS tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectPlayState {
    Idle,
    Running,
    Paused,
    Finished,
}

/// The supplied animation's replacement observation, without lifecycle ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationReplaceState {
    Active,
    Removed,
    Persisted,
}

/// Animation facts needed by effect-phase, currentness and relevance predicates.
///
/// Callers supply coherent observations from one update; this model does not
/// derive play/replace state, resolve holds, or query a timeline registry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectAnimationState {
    playback_rate: PlaybackRate,
    play_state: EffectPlayState,
    timeline: Option<TimelineMonotonicity>,
    replace_state: AnimationReplaceState,
}

impl EffectAnimationState {
    pub const fn new(
        playback_rate: PlaybackRate,
        play_state: EffectPlayState,
        timeline: Option<TimelineMonotonicity>,
        replace_state: AnimationReplaceState,
    ) -> Self {
        Self {
            playback_rate,
            play_state,
            timeline,
            replace_state,
        }
    }
    pub const fn playback_rate(self) -> PlaybackRate {
        self.playback_rate
    }
    pub const fn play_state(self) -> EffectPlayState {
        self.play_state
    }
    pub const fn timeline(self) -> Option<TimelineMonotonicity> {
        self.timeline
    }
    pub const fn replace_state(self) -> AnimationReplaceState {
        self.replace_state
    }
}

/// Zero or one associated animation supplies the local time for a keyframe effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectTimingInput {
    Unassociated,
    Associated {
        local_time: Option<AnimationTime>,
        animation: EffectAnimationState,
    },
}

/// A coherent sampled effect phase, iteration progress and derived states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectTimingSample {
    local_time: Option<AnimationTime>,
    animation: Option<EffectAnimationState>,
    kernel: kernel::KernelSample,
}

impl EffectTimingSample {
    pub const fn local_time(self) -> Option<AnimationTime> {
        self.local_time
    }
    pub const fn phase(self) -> TimingPhase {
        self.kernel.phase
    }
    pub const fn active_time_millis(self) -> Option<f64> {
        self.kernel.active_time
    }
    pub const fn overall_progress(self) -> Option<f64> {
        self.kernel.overall_progress
    }
    pub const fn current_iteration(self) -> Option<CurrentIteration> {
        self.kernel.current_iteration
    }
    pub const fn simple_progress(self) -> Option<PhaseProgress> {
        self.kernel.simple_progress
    }
    pub const fn directed_progress(self) -> Option<PhaseProgress> {
        self.kernel.directed_progress
    }
    pub const fn eased_progress(self) -> Option<EasedProgress> {
        self.kernel.eased_progress
    }
    pub const fn easing_before_flag(self) -> bool {
        self.kernel.before_flag
    }
    pub fn is_in_play(self) -> bool {
        self.phase() == TimingPhase::Active
            && self
                .animation
                .is_some_and(|animation| animation.play_state != EffectPlayState::Finished)
    }
    pub fn is_current(self) -> bool {
        self.is_in_play()
            || self.animation.is_some_and(|animation| {
                (animation.playback_rate.value() > 0.0 && self.phase() == TimingPhase::Before)
                    || (animation.playback_rate.value() < 0.0 && self.phase() == TimingPhase::After)
                    || (animation.play_state != EffectPlayState::Idle
                        && animation.timeline == Some(TimelineMonotonicity::NonMonotonic))
            })
    }
    pub const fn is_in_effect(self) -> bool {
        self.kernel.active_time.is_some()
    }
    /// Applies the relevance predicate to this effect and its supplied animation state.
    pub fn is_relevant(self) -> bool {
        self.animation
            .is_some_and(|animation| animation.replace_state != AnimationReplaceState::Removed)
            && (self.is_current() || self.is_in_effect())
    }
}
