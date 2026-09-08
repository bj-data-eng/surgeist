//! Typed diagnostics shared by animation time, progress, easing, and
//! interpolation constructors.
//!
//! Diagnostics name the rejected semantic input instead of returning prose-only
//! errors. Placeholder diagnostics without a production origin are intentionally
//! absent from the public API.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericInput {
    Duration,
    Delay,
    ElapsedTime,
    Progress,
    Percentage,
    Ratio,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericErrorKind {
    NonFinite {
        input: NumericInput,
    },
    Negative {
        input: NumericInput,
        actual: f64,
    },
    OutOfRange {
        input: NumericInput,
        min: f64,
        max: f64,
        actual: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericError {
    kind: NumericErrorKind,
}

impl NumericError {
    pub const fn non_finite(input: NumericInput) -> Self {
        Self {
            kind: NumericErrorKind::NonFinite { input },
        }
    }

    pub const fn negative(input: NumericInput, actual: f64) -> Self {
        Self {
            kind: NumericErrorKind::Negative { input, actual },
        }
    }

    pub const fn out_of_range(input: NumericInput, min: f64, max: f64, actual: f64) -> Self {
        Self {
            kind: NumericErrorKind::OutOfRange {
                input,
                min,
                max,
                actual,
            },
        }
    }

    pub const fn kind(self) -> NumericErrorKind {
        self.kind
    }

    pub const fn input(self) -> NumericInput {
        match self.kind {
            NumericErrorKind::NonFinite { input }
            | NumericErrorKind::Negative { input, .. }
            | NumericErrorKind::OutOfRange { input, .. } => input,
        }
    }
}

impl fmt::Display for NumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            NumericErrorKind::NonFinite { input } => {
                write!(f, "{} must be finite", numeric_input_name(input))
            }
            NumericErrorKind::Negative { input, actual } => write!(
                f,
                "{} must be non-negative, got {}",
                numeric_input_name(input),
                actual
            ),
            NumericErrorKind::OutOfRange {
                input,
                min,
                max,
                actual,
            } => write!(
                f,
                "{} must be between {} and {}, got {}",
                numeric_input_name(input),
                min,
                max,
                actual
            ),
        }
    }
}

impl std::error::Error for NumericError {}

fn numeric_input_name(input: NumericInput) -> &'static str {
    match input {
        NumericInput::Duration => "duration",
        NumericInput::Delay => "delay",
        NumericInput::ElapsedTime => "elapsed time",
        NumericInput::Progress => "progress",
        NumericInput::Percentage => "percentage",
        NumericInput::Ratio => "ratio",
    }
}

#[cfg(test)]
mod tests {
    use super::{NumericError, NumericErrorKind, NumericInput};

    #[test]
    fn numeric_error_names_rejected_input() {
        let error = NumericError::non_finite(NumericInput::Duration);

        assert_eq!(error.input(), NumericInput::Duration);
        assert!(matches!(
            error.kind(),
            NumericErrorKind::NonFinite {
                input: NumericInput::Duration
            }
        ));
    }

    #[test]
    fn percentage_numeric_input_is_available_for_diagnostics() {
        let error = NumericError::non_finite(NumericInput::Percentage);

        assert_eq!(error.input(), NumericInput::Percentage);
        assert!(matches!(
            error.kind(),
            NumericErrorKind::NonFinite {
                input: NumericInput::Percentage
            }
        ));
    }

    #[test]
    fn range_error_preserves_bounds_and_actual_value() {
        let error = NumericError::out_of_range(NumericInput::Progress, 0.0, 1.0, 1.25);

        assert_eq!(error.input(), NumericInput::Progress);
        assert!(matches!(
            error.kind(),
            NumericErrorKind::OutOfRange {
                input: NumericInput::Progress,
                min: 0.0,
                max: 1.0,
                actual: 1.25
            }
        ));
    }

    #[test]
    fn retained_numeric_diagnostics_have_display_text() {
        let non_finite = NumericError::non_finite(NumericInput::Duration);

        assert_eq!(non_finite.to_string(), "duration must be finite");
        assert!(std::error::Error::source(&non_finite).is_none());
        assert_eq!(
            NumericError::negative(NumericInput::ElapsedTime, -1.0).to_string(),
            "elapsed time must be non-negative, got -1"
        );
        assert_eq!(
            NumericError::out_of_range(NumericInput::Progress, 0.0, 1.0, 1.25).to_string(),
            "progress must be between 0 and 1, got 1.25"
        );
    }
}
