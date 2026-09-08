//! Progress units used by easing, timing, and interpolation.
//!
//! [`NormalizedProgress`] and [`UnitRatio`] are closed unit-interval values.
//! [`EasingInput`] is an unrestricted finite easing input for CSS easing
//! functions that can extrapolate outside the normalized interval.

use crate::{EasingError, NumericError, NumericInput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedProgress {
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EasingInput {
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitRatio {
    value: f64,
}

impl NormalizedProgress {
    pub fn new(value: f64) -> Result<Self, NumericError> {
        validate_unit_interval(NumericInput::Progress, value)?;

        Ok(Self { value })
    }

    pub fn clamp_finite(value: f64) -> Result<Self, NumericError> {
        if !value.is_finite() {
            return Err(NumericError::non_finite(NumericInput::Progress));
        }

        Ok(Self {
            value: value.clamp(0.0, 1.0),
        })
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl EasingInput {
    pub fn new(value: f64) -> Result<Self, EasingError> {
        if !value.is_finite() {
            return Err(EasingError::NonFiniteEasingInput { value });
        }

        Ok(Self { value })
    }

    pub const fn from_normalized(progress: NormalizedProgress) -> Self {
        Self {
            value: progress.value(),
        }
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl UnitRatio {
    pub fn new(value: f64) -> Result<Self, NumericError> {
        validate_unit_interval(NumericInput::Ratio, value)?;

        Ok(Self { value })
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

fn validate_unit_interval(input: NumericInput, value: f64) -> Result<(), NumericError> {
    if !value.is_finite() {
        return Err(NumericError::non_finite(input));
    }

    if !(0.0..=1.0).contains(&value) {
        return Err(NumericError::out_of_range(input, 0.0, 1.0, value));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{EasingInput, NormalizedProgress, UnitRatio};
    use crate::{EasingError, NumericError, NumericInput};

    #[test]
    fn normalized_progress_accepts_bounds_and_midpoints() {
        assert_eq!(NormalizedProgress::new(0.0).unwrap().value(), 0.0);
        assert_eq!(NormalizedProgress::new(0.5).unwrap().value(), 0.5);
        assert_eq!(NormalizedProgress::new(1.0).unwrap().value(), 1.0);
    }

    #[test]
    fn easing_input_accepts_unrestricted_finite_values() {
        assert_eq!(EasingInput::new(-10.0).unwrap().value(), -10.0);
        assert_eq!(EasingInput::new(0.5).unwrap().value(), 0.5);
        assert_eq!(EasingInput::new(10.0).unwrap().value(), 10.0);
        assert_eq!(
            EasingInput::new(f64::NEG_INFINITY),
            Err(EasingError::NonFiniteEasingInput {
                value: f64::NEG_INFINITY
            })
        );
    }

    #[test]
    fn normalized_progress_rejects_non_finite_and_out_of_range_values() {
        assert_eq!(
            NormalizedProgress::new(f64::NAN),
            Err(NumericError::non_finite(NumericInput::Progress))
        );
        assert_eq!(
            NormalizedProgress::new(f64::INFINITY),
            Err(NumericError::non_finite(NumericInput::Progress))
        );
        assert_eq!(
            NormalizedProgress::new(f64::NEG_INFINITY),
            Err(NumericError::non_finite(NumericInput::Progress))
        );
        assert_eq!(
            NormalizedProgress::new(-0.01),
            Err(NumericError::out_of_range(
                NumericInput::Progress,
                0.0,
                1.0,
                -0.01
            ))
        );
        assert_eq!(
            NormalizedProgress::new(1.01),
            Err(NumericError::out_of_range(
                NumericInput::Progress,
                0.0,
                1.0,
                1.01
            ))
        );
    }

    #[test]
    fn normalized_progress_clamps_only_finite_values() {
        assert_eq!(NormalizedProgress::clamp_finite(-2.0).unwrap().value(), 0.0);
        assert_eq!(
            NormalizedProgress::clamp_finite(0.25).unwrap().value(),
            0.25
        );
        assert_eq!(NormalizedProgress::clamp_finite(3.0).unwrap().value(), 1.0);
        assert_eq!(
            NormalizedProgress::clamp_finite(f64::INFINITY),
            Err(NumericError::non_finite(NumericInput::Progress))
        );
    }

    #[test]
    fn unit_ratio_uses_ratio_diagnostics() {
        assert_eq!(UnitRatio::new(1.0).unwrap().value(), 1.0);
        assert_eq!(
            UnitRatio::new(f64::INFINITY),
            Err(NumericError::non_finite(NumericInput::Ratio))
        );
        assert_eq!(
            UnitRatio::new(f64::NEG_INFINITY),
            Err(NumericError::non_finite(NumericInput::Ratio))
        );
        assert_eq!(
            UnitRatio::new(2.0),
            Err(NumericError::out_of_range(
                NumericInput::Ratio,
                0.0,
                1.0,
                2.0
            ))
        );
    }
}
