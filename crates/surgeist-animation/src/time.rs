//! Time inputs for animation sampling.
//!
//! [`ElapsedTime`] is the effective, runtime-supplied non-negative elapsed time
//! used for sampling. This crate validates and consumes it, but does not own
//! clocks, scheduling, pause bookkeeping, or track-start resolution.

use crate::{NumericError, NumericInput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationDuration {
    secs: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationDelay {
    secs: f64,
}

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
    use crate::{NumericError, NumericInput};

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
