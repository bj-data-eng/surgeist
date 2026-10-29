//! Resolved native post-baseline displacement, independent of font discovery.
use crate::{
    DefaultScalar, FiniteScalarErrorOf, LayoutScalar, LengthPercentageOf, NonNegativeFiniteOf,
    PercentageBasisOf,
};

/// A finite displacement toward the containing line's over side.
///
/// Match the actual child baseline before applying this value. Normalization
/// helpers consume explicitly supplied used metrics; they do not infer ancestry,
/// query fonts, or change the child's font size or local geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineOffsetOf<S: LayoutScalar = DefaultScalar> {
    value: S,
}

pub type BaselineOffset = BaselineOffsetOf<DefaultScalar>;

/// Failure to normalize one resolved baseline displacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineOffsetResolutionError {
    MissingOwnLineHeight,
    NonFinite,
    NonZeroLost,
}

impl core::fmt::Display for BaselineOffsetResolutionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::MissingOwnLineHeight => {
                "baseline offset requires the child's own used line height"
            }
            Self::NonFinite => "baseline offset arithmetic must remain finite",
            Self::NonZeroLost => "baseline offset arithmetic lost a nonzero contribution",
        })
    }
}
impl std::error::Error for BaselineOffsetResolutionError {}

impl<S: LayoutScalar> BaselineOffsetOf<S> {
    pub const ZERO: Self = Self { value: S::ZERO };

    pub fn try_new(value: S) -> Result<Self, FiniteScalarErrorOf<S>> {
        if !value.is_finite() {
            return Err(FiniteScalarErrorOf::NonFinite { value });
        }
        Ok(Self {
            value: crate::scalar::canonical_zero(value),
        })
    }

    #[must_use]
    pub const fn get(self) -> S {
        self.value
    }

    /// Resolve against the child's own used line height, before sibling expansion.
    /// A zero percentage coefficient does not require that context. Definite
    /// zero is a real basis; overflow and lost nonzero contributions refuse.
    pub fn from_length_percentage(
        amount: LengthPercentageOf<S>,
        own_line_height: PercentageBasisOf<S>,
    ) -> Result<Self, BaselineOffsetResolutionError> {
        let length = amount.absolute_px();
        let percent = amount.percent_fraction();
        if percent == S::ZERO {
            return Ok(Self { value: length });
        }
        let PercentageBasisOf::Definite(height) = own_line_height else {
            return Err(BaselineOffsetResolutionError::MissingOwnLineHeight);
        };
        let height = height.get();
        let product = percent * height;
        finite(product)?;
        if height != S::ZERO && product == S::ZERO {
            return Err(BaselineOffsetResolutionError::NonZeroLost);
        }
        let value = length + product;
        finite(value)?;
        if (length != S::ZERO && value == product) || (product != S::ZERO && value == length) {
            return Err(BaselineOffsetResolutionError::NonZeroLost);
        }
        Ok(Self {
            value: crate::scalar::canonical_zero(value),
        })
    }

    /// Native Inline3 fallback: lower by one fifth of the parent used size.
    /// This is numeric policy, not a claim about browser font-table preference.
    pub fn subscript(
        parent_used_size: NonNegativeFiniteOf<S>,
    ) -> Result<Self, BaselineOffsetResolutionError> {
        Self::font_fraction(parent_used_size, 5.0, true)
    }

    /// Native Inline3 fallback: raise by one third of the parent used size.
    pub fn superscript(
        parent_used_size: NonNegativeFiniteOf<S>,
    ) -> Result<Self, BaselineOffsetResolutionError> {
        Self::font_fraction(parent_used_size, 3.0, false)
    }

    fn font_fraction(
        size: NonNegativeFiniteOf<S>,
        divisor: f64,
        negative: bool,
    ) -> Result<Self, BaselineOffsetResolutionError> {
        let size = size.get();
        let value = size / S::from_f64(divisor);
        finite(value)?;
        if size != S::ZERO && value == S::ZERO {
            return Err(BaselineOffsetResolutionError::NonZeroLost);
        }
        Ok(Self {
            value: crate::scalar::canonical_zero(if negative { -value } else { value }),
        })
    }
}

fn finite<S: LayoutScalar>(value: S) -> Result<(), BaselineOffsetResolutionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(BaselineOffsetResolutionError::NonFinite)
    }
}

/// Execute one signed contribution in the owning Layout scalar domain.
pub(crate) fn displaced<S: LayoutScalar>(
    base: S,
    delta: S,
) -> Result<S, crate::group_alignment::NumericError> {
    let result = base + delta;
    if !base.is_finite() || !delta.is_finite() || !result.is_finite() {
        return Err(crate::group_alignment::NumericError::new(
            result.to_f64(),
            crate::LayoutTextNumericError::NonFinite,
        ));
    }
    if (delta != S::ZERO && result == base) || (base != S::ZERO && result == delta) {
        return Err(crate::group_alignment::NumericError::new(
            delta.to_f64(),
            crate::LayoutTextNumericError::NonZeroLost,
        ));
    }
    Ok(crate::scalar::canonical_zero(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalization<S: LayoutScalar>() {
        let n = S::from_f64;
        let lp = LengthPercentageOf::from_coefficients(n(2.0), n(0.5)).unwrap();
        assert_eq!(
            BaselineOffsetOf::from_length_percentage(
                lp,
                PercentageBasisOf::definite(n(18.0)).unwrap()
            )
            .unwrap()
            .get(),
            n(11.0)
        );
        assert_eq!(
            BaselineOffsetOf::from_length_percentage(
                lp,
                PercentageBasisOf::definite(n(60.0)).unwrap()
            )
            .unwrap()
            .get(),
            n(32.0)
        );
        assert_eq!(
            BaselineOffsetOf::from_length_percentage(
                lp,
                PercentageBasisOf::definite(S::ZERO).unwrap()
            )
            .unwrap()
            .get(),
            n(2.0)
        );
        assert_eq!(
            BaselineOffsetOf::from_length_percentage(lp, PercentageBasisOf::MISSING),
            Err(BaselineOffsetResolutionError::MissingOwnLineHeight)
        );
        assert_eq!(
            BaselineOffsetOf::from_length_percentage(
                LengthPercentageOf::px(n(-3.0)).unwrap(),
                PercentageBasisOf::MISSING
            )
            .unwrap()
            .get(),
            n(-3.0)
        );
        let cancellation = LengthPercentageOf::from_coefficients(n(-9.0), n(0.5)).unwrap();
        assert_eq!(
            BaselineOffsetOf::from_length_percentage(
                cancellation,
                PercentageBasisOf::definite(n(18.0)).unwrap()
            )
            .unwrap(),
            BaselineOffsetOf::ZERO
        );
        assert_eq!(
            BaselineOffsetOf::subscript(NonNegativeFiniteOf::new(n(20.0)).unwrap())
                .unwrap()
                .get(),
            n(-4.0)
        );
        assert_eq!(
            BaselineOffsetOf::superscript(NonNegativeFiniteOf::new(n(20.0)).unwrap())
                .unwrap()
                .get(),
            n(20.0) / n(3.0)
        );
        assert_eq!(
            BaselineOffsetOf::superscript(NonNegativeFiniteOf::new(n(21.0)).unwrap())
                .unwrap()
                .get(),
            n(7.0)
        );
        assert_eq!(
            BaselineOffsetOf::subscript(NonNegativeFiniteOf::<S>::ZERO).unwrap(),
            BaselineOffsetOf::ZERO
        );
        assert_eq!(
            BaselineOffsetOf::superscript(NonNegativeFiniteOf::<S>::ZERO).unwrap(),
            BaselineOffsetOf::ZERO
        );
        assert!(BaselineOffsetOf::<S>::try_new(S::INFINITY).is_err());
        assert!(BaselineOffsetOf::<S>::try_new(S::NAN).is_err());
        assert!(NonNegativeFiniteOf::<S>::new(n(-1.0)).is_err());
        assert!(NonNegativeFiniteOf::<S>::new(S::INFINITY).is_err());
        assert!(PercentageBasisOf::<S>::definite(n(-1.0)).is_err());
        assert!(PercentageBasisOf::<S>::definite(S::NAN).is_err());
        assert_eq!(
            BaselineOffsetOf::<S>::try_new(-S::ZERO)
                .unwrap()
                .get()
                .to_f64()
                .to_bits(),
            0
        );
        assert_ne!(
            crate::VerticalAlignOf::<S>::Baseline,
            crate::VerticalAlignOf::Shift(BaselineOffsetOf::ZERO)
        );
    }

    #[test]
    fn own_height_parent_size_and_explicit_zero_are_distinct_both_scalars() {
        normalization::<f32>();
        normalization::<f64>();
    }

    fn losses<S: LayoutScalar>(large: S, tiny: S, maximum: S) {
        let resolve = |length, percent, height| {
            BaselineOffsetOf::from_length_percentage(
                LengthPercentageOf::from_coefficients(length, percent).unwrap(),
                PercentageBasisOf::definite(height).unwrap(),
            )
        };
        // Each side of the sum is independently required. Reverse the large
        // and small components to catch either absorption direction.
        assert_eq!(
            resolve(S::ONE, large, S::ONE),
            Err(BaselineOffsetResolutionError::NonZeroLost)
        );
        assert_eq!(
            resolve(large, S::ONE, S::ONE),
            Err(BaselineOffsetResolutionError::NonZeroLost)
        );
        assert_eq!(
            resolve(S::ZERO, tiny, S::from_f64(0.5)),
            Err(BaselineOffsetResolutionError::NonZeroLost)
        );
        assert_eq!(
            resolve(-maximum, maximum, S::from_f64(2.0)),
            Err(BaselineOffsetResolutionError::NonFinite)
        );
        assert_eq!(
            resolve(maximum, maximum, S::ONE),
            Err(BaselineOffsetResolutionError::NonFinite)
        );
        assert_eq!(
            BaselineOffsetOf::subscript(NonNegativeFiniteOf::new(tiny).unwrap()),
            Err(BaselineOffsetResolutionError::NonZeroLost)
        );
        assert_eq!(
            BaselineOffsetOf::superscript(NonNegativeFiniteOf::new(tiny).unwrap()),
            Err(BaselineOffsetResolutionError::NonZeroLost)
        );
        assert_eq!(
            resolve(S::ZERO, maximum, S::ZERO).unwrap(),
            BaselineOffsetOf::ZERO
        );
    }

    #[test]
    fn normalization_rejects_absorption_underflow_and_overflow_both_scalars() {
        losses::<f32>(16_777_216.0, f32::from_bits(1), f32::MAX);
        losses::<f64>(9_007_199_254_740_992.0, f64::from_bits(1), f64::MAX);
    }

    fn execution<S: LayoutScalar>(large: S, maximum: S) {
        let reason =
            |result: Result<S, crate::group_alignment::NumericError>| result.unwrap_err().reason();
        assert_eq!(
            reason(displaced(large, S::ONE)),
            crate::LayoutTextNumericError::NonZeroLost
        );
        assert_eq!(
            reason(displaced(S::ONE, large)),
            crate::LayoutTextNumericError::NonZeroLost
        );
        assert_eq!(
            reason(displaced(maximum, maximum)),
            crate::LayoutTextNumericError::NonFinite
        );
        assert_eq!(displaced(large, -large).unwrap(), S::ZERO);
    }

    #[test]
    fn execution_keeps_each_coordinate_contribution_both_scalars() {
        execution::<f32>(16_777_216.0, f32::MAX);
        execution::<f64>(9_007_199_254_740_992.0, f64::MAX);
    }
}
