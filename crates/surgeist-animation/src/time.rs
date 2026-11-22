//! Typed model coordinates and normalized inputs for animation sampling.
//!
//! Model coordinates are signed finite milliseconds; `Option<coordinate>`
//! represents unresolved time. Coordinate types cannot be interchanged implicitly.
//! These inputs do not imply that a timeline follows a wall clock.
//!
//! [`ElapsedTime`] is the effective, runtime-supplied non-negative elapsed time
//! used for sampling. This crate validates and consumes it, but does not own
//! clocks, scheduling, pause bookkeeping, or track-start resolution.

use crate::diagnostics::{NumericError, NumericInput};

/// A finite time in an animation's coordinate space, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationTime {
    millis: f64,
}

/// A finite time in an associated timeline's coordinate space, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineTime {
    millis: f64,
}

/// A finite time relative to a document's time origin, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginRelativeTime {
    millis: f64,
}

/// A finite playback multiplier, including negative values and zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackRate {
    value: f64,
}

/// The semantic input rejected by a model-time constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInput {
    Animation,
    Timeline,
    OriginRelative,
    PlaybackRate,
}

/// The model-time operation whose result cannot be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOperation {
    DocumentCurrentTime,
    TimelineToOrigin,
    AnimationToTimeline,
    TimelineToAnimation,
}

/// A typed model-time construction, arithmetic, or reporting failure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeError {
    kind: TimeErrorKind,
}

/// The distinction between invalid input, numeric limits, and decreasing reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeErrorKind {
    NonFinite {
        input: TimeInput,
    },
    Unrepresentable {
        operation: TimeOperation,
    },
    DecreasingDocumentTime {
        previous: TimelineTime,
        attempted: TimelineTime,
    },
}

impl AnimationTime {
    /// Constructs a signed, finite animation coordinate without clamping it.
    pub fn from_millis(millis: f64) -> Result<Self, TimeError> {
        validate_model_number(millis, TimeInput::Animation)?;
        Ok(Self { millis })
    }

    pub const fn as_millis(self) -> f64 {
        self.millis
    }
}

impl TimelineTime {
    /// Constructs a signed, finite timeline coordinate without clamping it.
    pub fn from_millis(millis: f64) -> Result<Self, TimeError> {
        validate_model_number(millis, TimeInput::Timeline)?;
        Ok(Self { millis })
    }

    pub const fn as_millis(self) -> f64 {
        self.millis
    }
}

impl OriginRelativeTime {
    /// Constructs a signed, finite origin-relative coordinate without clamping it.
    pub fn from_millis(millis: f64) -> Result<Self, TimeError> {
        validate_model_number(millis, TimeInput::OriginRelative)?;
        Ok(Self { millis })
    }

    pub const fn as_millis(self) -> f64 {
        self.millis
    }
}

impl PlaybackRate {
    pub fn new(value: f64) -> Result<Self, TimeError> {
        validate_model_number(value, TimeInput::PlaybackRate)?;
        Ok(Self { value })
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl TimeError {
    pub const fn kind(self) -> TimeErrorKind {
        self.kind
    }

    pub(crate) const fn unrepresentable(operation: TimeOperation) -> Self {
        Self {
            kind: TimeErrorKind::Unrepresentable { operation },
        }
    }

    pub(crate) const fn decreasing(previous: TimelineTime, attempted: TimelineTime) -> Self {
        Self {
            kind: TimeErrorKind::DecreasingDocumentTime {
                previous,
                attempted,
            },
        }
    }
}

impl core::fmt::Display for TimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            TimeErrorKind::NonFinite { input } => write!(f, "{input:?} time input must be finite"),
            TimeErrorKind::Unrepresentable { operation } => {
                write!(f, "{operation:?} time result is not representable")
            }
            TimeErrorKind::DecreasingDocumentTime {
                previous,
                attempted,
            } => write!(
                f,
                "document timeline cannot decrease from {} to {} milliseconds",
                previous.as_millis(),
                attempted.as_millis()
            ),
        }
    }
}

impl std::error::Error for TimeError {}

fn validate_model_number(value: f64, input: TimeInput) -> Result<(), TimeError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(TimeError {
            kind: TimeErrorKind::NonFinite { input },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationDuration {
    secs: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationDelay {
    secs: f64,
}

/// Effective elapsed seconds supplied to sampling after clock resolution.
///
/// A duration cannot be passed as elapsed time:
///
/// ```compile_fail,E0308
/// use surgeist_animation::time::AnimationDuration;
/// use surgeist_animation::timing::{AnimationPlayState, TimingParameters};
///
/// fn sample(timing: &TimingParameters) {
///     let _ = timing.sample(
///         AnimationDuration::from_secs(0.5).unwrap(),
///         AnimationPlayState::Running,
///     );
/// }
/// ```
///
/// The same sampling boundary accepts validated elapsed time:
///
/// ```
/// use surgeist_animation::time::ElapsedTime;
/// use surgeist_animation::timing::{AnimationPlayState, TimingParameters};
///
/// fn sample(timing: &TimingParameters) {
///     let _ = timing.sample(
///         ElapsedTime::from_secs(0.5).unwrap(),
///         AnimationPlayState::Running,
///     );
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElapsedTime {
    secs: f64,
}

impl AnimationDuration {
    pub fn from_secs(secs: f64) -> Result<Self, NumericError> {
        validate_non_negative_finite(NumericInput::Duration, secs)?;

        Ok(Self { secs })
    }

    pub const fn as_secs(self) -> f64 {
        self.secs
    }

    pub const fn is_zero(self) -> bool {
        self.secs == 0.0
    }
}

impl AnimationDelay {
    pub fn from_secs(secs: f64) -> Result<Self, NumericError> {
        validate_finite(NumericInput::Delay, secs)?;

        Ok(Self { secs })
    }

    pub const fn as_secs(self) -> f64 {
        self.secs
    }
}

impl ElapsedTime {
    pub fn from_secs(secs: f64) -> Result<Self, NumericError> {
        validate_non_negative_finite(NumericInput::ElapsedTime, secs)?;

        Ok(Self { secs })
    }

    pub const fn as_secs(self) -> f64 {
        self.secs
    }
}

fn validate_finite(input: NumericInput, secs: f64) -> Result<(), NumericError> {
    if !secs.is_finite() {
        return Err(NumericError::non_finite(input));
    }

    Ok(())
}

fn validate_non_negative_finite(input: NumericInput, secs: f64) -> Result<(), NumericError> {
    validate_finite(input, secs)?;

    if secs < 0.0 {
        return Err(NumericError::negative(input, secs));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AnimationDelay, AnimationDuration, ElapsedTime};
    use crate::diagnostics::{NumericError, NumericInput};

    #[test]
    fn duration_accepts_zero_and_large_finite_values() {
        assert_eq!(AnimationDuration::from_secs(0.0).unwrap().as_secs(), 0.0);
        assert_eq!(
            AnimationDuration::from_secs(1.0e12).unwrap().as_secs(),
            1.0e12
        );
    }

    #[test]
    fn duration_rejects_negative_and_non_finite_values() {
        assert_eq!(
            AnimationDuration::from_secs(-0.001),
            Err(NumericError::negative(NumericInput::Duration, -0.001))
        );
        assert_eq!(
            AnimationDuration::from_secs(f64::INFINITY),
            Err(NumericError::non_finite(NumericInput::Duration))
        );
        assert_eq!(
            AnimationDuration::from_secs(f64::NAN),
            Err(NumericError::non_finite(NumericInput::Duration))
        );
    }

    #[test]
    fn delay_accepts_negative_zero_and_positive_finite_values() {
        assert_eq!(AnimationDelay::from_secs(-0.25).unwrap().as_secs(), -0.25);
        assert_eq!(AnimationDelay::from_secs(0.0).unwrap().as_secs(), 0.0);
        assert_eq!(AnimationDelay::from_secs(2.5).unwrap().as_secs(), 2.5);
    }

    #[test]
    fn delay_rejects_non_finite_values() {
        assert_eq!(
            AnimationDelay::from_secs(f64::NEG_INFINITY),
            Err(NumericError::non_finite(NumericInput::Delay))
        );
    }

    #[test]
    fn elapsed_time_is_a_non_negative_finite_input() {
        assert_eq!(ElapsedTime::from_secs(4.0).unwrap().as_secs(), 4.0);
        assert_eq!(
            ElapsedTime::from_secs(-1.0),
            Err(NumericError::negative(NumericInput::ElapsedTime, -1.0))
        );
    }
}
