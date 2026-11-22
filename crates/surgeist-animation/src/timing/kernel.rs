//! One unit-consistent timing algorithm shared by model and CSS elapsed inputs.

use super::{
    ActiveDuration, AnimationDirection, CurrentDirection, CurrentIteration, EasedProgress, Easing,
    EffectTimingError, FillMode, IterationCount, PhaseProgress, TimingOperation, TimingPhase,
    directed_progress,
};

pub(super) struct Parameters<'a> {
    pub delay: f64,
    pub end_delay: f64,
    pub duration: f64,
    pub iterations: IterationCount,
    pub iteration_start: f64,
    pub direction: AnimationDirection,
    pub fill: FillMode,
    pub easing: &'a Easing,
    pub active_duration: ActiveDuration,
}

/// AboveRange is a finite sum beyond f64::MAX, not mathematical infinity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Boundary {
    Finite(f64),
    AboveRange,
    Infinite,
}

impl Boundary {
    fn nonnegative(value: f64) -> Self {
        if value == f64::INFINITY {
            Self::AboveRange
        } else {
            Self::Finite(value.max(0.0))
        }
    }
    fn min(self, other: Self) -> Self {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(a.min(b)),
            (finite @ Self::Finite(_), _) | (_, finite @ Self::Finite(_)) => finite,
            (Self::AboveRange, _) | (_, Self::AboveRange) => Self::AboveRange,
            (Self::Infinite, Self::Infinite) => Self::Infinite,
        }
    }
    pub(super) fn value(self) -> Result<f64, EffectTimingError> {
        match self {
            Self::Finite(value) => Ok(value),
            Self::Infinite => Ok(f64::INFINITY),
            Self::AboveRange => Err(limit(TimingOperation::EndTime)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Bounds {
    pub before: f64,
    pub after: Boundary,
    pub end: Boundary,
}

impl Bounds {
    pub fn new(parameters: &Parameters<'_>) -> Self {
        let (end, after) = match parameters.active_duration {
            ActiveDuration::Unbounded => (Boundary::Infinite, Boundary::Infinite),
            ActiveDuration::Finite(duration) => {
                let end = Boundary::nonnegative(sum_three(
                    parameters.delay,
                    duration,
                    parameters.end_delay,
                ));
                let after = Boundary::nonnegative(parameters.delay + duration).min(end);
                (end, after)
            }
        };
        let before = match end {
            Boundary::Finite(end) => parameters.delay.min(end).max(0.0),
            _ => parameters.delay.max(0.0),
        };
        Self { before, after, end }
    }

    fn phase(self, local: f64, backwards: bool) -> TimingPhase {
        if local < self.before || (backwards && local == self.before) {
            return TimingPhase::Before;
        }
        if let Boundary::Finite(after) = self.after
            && (local > after || (!backwards && local == after))
        {
            return TimingPhase::After;
        }
        TimingPhase::Active
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct KernelSample {
    pub phase: TimingPhase,
    pub active_time: Option<f64>,
    pub overall_progress: Option<f64>,
    pub current_iteration: Option<CurrentIteration>,
    pub simple_progress: Option<PhaseProgress>,
    pub directed_progress: Option<PhaseProgress>,
    pub eased_progress: Option<EasedProgress>,
    pub before_flag: bool,
    pub error: Option<EffectTimingError>,
}

impl KernelSample {
    pub const fn idle() -> Self {
        Self {
            phase: TimingPhase::Idle,
            active_time: None,
            overall_progress: None,
            current_iteration: None,
            simple_progress: None,
            directed_progress: None,
            eased_progress: None,
            before_flag: false,
            error: None,
        }
    }
}

pub(super) fn sample_model(
    parameters: &Parameters<'_>,
    local: Option<f64>,
    backwards: bool,
) -> KernelSample {
    let Some(local) = local else {
        return KernelSample::idle();
    };
    let phase = Bounds::new(parameters).phase(local, backwards);
    sample_progress(parameters, phase, local - parameters.delay)
}

pub(super) fn sample_elapsed(parameters: &Parameters<'_>, elapsed: f64) -> KernelSample {
    // Compare the delay-relative coordinate so a short active interval is
    // still active at its start even if delay + duration rounds to delay.
    let delay_adjusted = elapsed - parameters.delay;
    let phase = if delay_adjusted < 0.0 {
        TimingPhase::Before
    } else {
        match parameters.active_duration {
            ActiveDuration::Finite(duration) if delay_adjusted >= duration => TimingPhase::After,
            _ => TimingPhase::Active,
        }
    };
    sample_progress(parameters, phase, delay_adjusted)
}

fn sample_progress(
    parameters: &Parameters<'_>,
    phase: TimingPhase,
    delay_adjusted: f64,
) -> KernelSample {
    let mut sample = KernelSample {
        phase,
        ..KernelSample::idle()
    };
    let result = (|| {
        let active = match phase {
            TimingPhase::Before
                if matches!(parameters.fill, FillMode::Backwards | FillMode::Both) =>
            {
                delay_adjusted.max(0.0)
            }
            TimingPhase::Active => delay_adjusted,
            TimingPhase::After
                if matches!(parameters.fill, FillMode::Forwards | FillMode::Both) =>
            {
                delay_adjusted
                    .min(extent_value(parameters.active_duration))
                    .max(0.0)
            }
            _ => return Ok(()),
        };
        if !active.is_finite() {
            return Err(limit(TimingOperation::ActiveTime));
        }
        sample.active_time = Some(active);
        let count = count_value(parameters.iterations);
        let at_active_end = matches!(phase, TimingPhase::Active | TimingPhase::After)
            && matches!(
                parameters.active_duration,
                ActiveDuration::Finite(duration) if active == duration
            );
        let initial = if parameters.duration == 0.0 {
            if sample.phase == TimingPhase::Before {
                0.0
            } else {
                count
            }
        } else if at_active_end {
            // Dividing the rounded active duration can move a whole iteration
            // across its endpoint. The configured count already gives its value.
            count
        } else {
            let progress = active / parameters.duration;
            if !progress.is_finite() {
                return Err(limit(TimingOperation::OverallProgress));
            }
            progress
        };
        // Infinity here is source-defined (zero duration with infinite count),
        // never a sentinel for overflow of finite division or addition.
        let overall = if initial == f64::INFINITY {
            initial
        } else {
            let progress = initial + parameters.iteration_start;
            if !progress.is_finite() {
                return Err(limit(TimingOperation::OverallProgress));
            }
            progress
        };
        sample.overall_progress = Some(overall);
        let mut simple = if overall == f64::INFINITY {
            parameters.iteration_start.fract()
        } else {
            overall.fract()
        };
        if simple == 0.0 && at_active_end && count != 0.0 {
            simple = 1.0;
        }
        let iteration = if (sample.phase == TimingPhase::After
            && parameters.iterations.is_infinite())
            || overall == f64::INFINITY
        {
            CurrentIteration::infinite()
        } else {
            CurrentIteration::new(overall.floor() - if simple == 1.0 { 1.0 } else { 0.0 })
        };
        let simple = if simple == 0.0 {
            PhaseProgress::from_normalized(super::NormalizedProgress::ZERO)
        } else if simple == 1.0 {
            PhaseProgress::from_normalized(super::NormalizedProgress::ONE)
        } else {
            PhaseProgress::new(simple).map_err(|_| limit(TimingOperation::Progress))?
        };
        let (directed, direction) = directed_progress(parameters.direction, iteration, simple);
        let before_flag = (sample.phase == TimingPhase::Before
            && direction == CurrentDirection::Forwards)
            || (sample.phase == TimingPhase::After && direction == CurrentDirection::Reverse);
        sample.current_iteration = Some(iteration);
        sample.simple_progress = Some(simple);
        sample.directed_progress = Some(directed);
        sample.eased_progress = Some(
            parameters
                .easing
                .evaluate_with_before_flag(directed.normalized(), before_flag),
        );
        sample.before_flag = before_flag;
        Ok(())
    })();
    sample.error = result.err();
    sample
}

pub(super) fn count_value(count: IterationCount) -> f64 {
    count.finite_value().unwrap_or(f64::INFINITY)
}

pub(super) const fn extent_value(duration: ActiveDuration) -> f64 {
    match duration {
        ActiveDuration::Finite(value) => value,
        ActiveDuration::Unbounded => f64::INFINITY,
    }
}

pub(super) const fn limit(operation: TimingOperation) -> EffectTimingError {
    EffectTimingError::unrepresentable(operation)
}

fn sum_three(first: f64, second: f64, third: f64) -> f64 {
    let sum = first + second;
    if sum.is_finite() {
        return sum + third;
    }
    // At most three finite inputs can exceed the native range. Quarter scaling
    // preserves cancellation of an overflowing intermediate by the third term.
    (first * 0.25 + second * 0.25 + third * 0.25) * 4.0
}
