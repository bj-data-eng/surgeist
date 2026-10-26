//! Already-resolved line-relative content insets inside the outer line box.

use crate::{Result, TextNumericError, TextNumericPhase};

/// Alignment-only padding inside the unchanged outer available inline span.
///
/// These used line-left/right amounts do not alter source selection, natural
/// measurement, tab grids or hanging eligibility. A nonzero frame requires an
/// actual finite `LineContext::available_inline`; it can exceed that span when
/// signed indentation or advances require the existing overflow-start policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineAlignmentInsets {
    line_left: f64,
    line_right: f64,
}

impl LineAlignmentInsets {
    /// Affirmative absence of alignment-only padding.
    pub const ZERO: Self = Self {
        line_left: 0.0,
        line_right: 0.0,
    };

    /// Validates each used amount independently, canonicalizing signed zero.
    ///
    /// # Errors
    /// Rejects nonfinite or negative amounts. Whether the frame has a finite
    /// outer span is validated with the actual selected request.
    pub fn try_new(line_left: f64, line_right: f64) -> Result<Self> {
        fn amount(field: &'static str, value: f64) -> Result<f64> {
            let category = if !value.is_finite() {
                Some(TextNumericError::NonFinite)
            } else if value < 0.0 {
                Some(TextNumericError::OutOfRange)
            } else {
                None
            };
            if let Some(category) = category {
                return Err(crate::error::text_numeric_error(
                    field,
                    TextNumericPhase::Selection,
                    category,
                    value,
                ));
            }
            Ok(if value == 0.0 { 0.0 } else { value })
        }
        Ok(Self {
            line_left: amount("line-left alignment inset", line_left)?,
            line_right: amount("line-right alignment inset", line_right)?,
        })
    }

    #[must_use]
    pub const fn line_left(self) -> f64 {
        self.line_left
    }

    #[must_use]
    pub const fn line_right(self) -> f64 {
        self.line_right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorDetail;

    #[test]
    fn constructor_preserves_finite_sides_and_canonicalizes_both_zero_signs() {
        for (left, right) in [(0.0, 0.0), (-0.0, 0.0), (0.0, -0.0), (-0.0, -0.0)] {
            let frame = LineAlignmentInsets::try_new(left, right).unwrap();
            assert_eq!(frame, LineAlignmentInsets::ZERO);
            assert_eq!(frame.line_left().to_bits(), 0.0f64.to_bits());
            assert_eq!(frame.line_right().to_bits(), 0.0f64.to_bits());
        }
        let frame = LineAlignmentInsets::try_new(13.0, 29.0).unwrap();
        assert_eq!((frame.line_left(), frame.line_right()), (13.0, 29.0));
        // The type owns individual finite amounts, not a width or finite sum.
        let large = LineAlignmentInsets::try_new(f64::MAX, f64::MAX).unwrap();
        assert_eq!(
            (large.line_left(), large.line_right()),
            (f64::MAX, f64::MAX)
        );
    }

    #[test]
    fn invalid_sides_report_the_actual_field_and_numeric_category() {
        for (left, right, field, category) in [
            (
                -1.0,
                0.0,
                "line-left alignment inset",
                TextNumericError::OutOfRange,
            ),
            (
                0.0,
                -1.0,
                "line-right alignment inset",
                TextNumericError::OutOfRange,
            ),
            (
                f64::NAN,
                0.0,
                "line-left alignment inset",
                TextNumericError::NonFinite,
            ),
            (
                0.0,
                f64::INFINITY,
                "line-right alignment inset",
                TextNumericError::NonFinite,
            ),
            (
                f64::NEG_INFINITY,
                0.0,
                "line-left alignment inset",
                TextNumericError::NonFinite,
            ),
        ] {
            let error = LineAlignmentInsets::try_new(left, right).unwrap_err();
            assert!(
                matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
                field: actual, phase: TextNumericPhase::Selection, category: actual_category, ..
            }) if *actual == field && *actual_category == category)
            );
        }
    }
}
