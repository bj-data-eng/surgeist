use super::{logical_add, logical_multiply};
use crate::{ErrorDetail, TextNumericError};

#[test]
fn logical_spacing_retains_values_below_the_provider_float_domain() {
    assert_eq!(
        logical_add("spacing", 0.0, f64::MIN_POSITIVE).unwrap(),
        f64::MIN_POSITIVE
    );
    assert_eq!(logical_add("spacing", 4.0, -4.0).unwrap(), 0.0);
}

#[test]
fn positive_logical_height_must_not_underflow_to_zero() {
    let failure = logical_multiply("line height", f64::from_bits(1), 0.5).unwrap_err();
    assert!(matches!(
        failure.detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            category: TextNumericError::PositiveLost,
            ..
        })
    ));
    assert_eq!(
        logical_multiply("line height", f64::MIN_POSITIVE, 1.0).unwrap(),
        f64::MIN_POSITIVE
    );
}

#[test]
fn actual_lost_delta_and_nonfinite_spacing_sums_refuse_precisely() {
    let lost = logical_add("spacing", 1.0, f64::EPSILON / 4.0).unwrap_err();
    assert!(matches!(
        lost.detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            category: TextNumericError::NonZeroLost,
            ..
        })
    ));
    let overflow = logical_add("spacing", f64::MAX, f64::MAX).unwrap_err();
    assert!(matches!(
        overflow.detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            category: TextNumericError::NonFinite,
            ..
        })
    ));
}
