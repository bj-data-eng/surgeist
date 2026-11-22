//! Computed box sizes passed by the property owner.

use super::{
    CalcSize, ComputedError, ComputedErrorKind, ComputedNumeric, ComputedOperation, MathSource,
    SizingKeyword, calc_size::require_length_percentage,
};

/// A checked, complete computed box size, including the sizing forms used by
/// `column-width`.
///
/// Wrap it in [`super::ComputedComponent::BoxSize`] to sample it in a property.
/// Numeric values retain symbolic sources and signed intermediate values.
/// Style supplies computed endpoints and percentage context. Layout resolves
/// the sampled value to used geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedSize(ComputedSizeValue);

/// The form and payload of a [`ComputedSize`], for sample consumers.
#[derive(Debug, Clone, PartialEq)]
pub enum ComputedSizeValue {
    Keyword(SizingKeyword),
    LengthPercentage(ComputedNumeric),
    FitContent(ComputedNumeric),
    CalcSize(CalcSize),
    /// An original `calc-size()` whose canonical form exceeded the budget.
    UnavailableCalcSize(MathSource),
}

impl ComputedSize {
    pub const fn keyword(keyword: SizingKeyword) -> Self {
        Self(ComputedSizeValue::Keyword(keyword))
    }

    pub fn length_percentage(value: ComputedNumeric) -> Result<Self, ComputedError> {
        require_length_percentage(value.numeric_type())?;
        Ok(Self(ComputedSizeValue::LengthPercentage(value)))
    }

    /// The functional form, with a computed length-percentage argument.
    pub fn fit_content(value: ComputedNumeric) -> Result<Self, ComputedError> {
        require_length_percentage(value.numeric_type())?;
        Ok(Self(ComputedSizeValue::FitContent(value)))
    }

    pub const fn calc_size(value: CalcSize) -> Self {
        Self(ComputedSizeValue::CalcSize(value))
    }

    /// Keeps the original operand for discrete selection if its `calc-size()`
    /// cannot be canonicalized within the supported budget.
    pub fn unavailable_calc_size(source: MathSource) -> Result<Self, ComputedError> {
        require_length_percentage(source.numeric_type())?;
        Ok(Self(ComputedSizeValue::UnavailableCalcSize(source)))
    }

    pub const fn value(&self) -> &ComputedSizeValue {
        &self.0
    }

    pub(super) fn compatible(&self, other: &Self, operation: ComputedOperation) -> bool {
        use ComputedSizeValue::{CalcSize, FitContent, Keyword, LengthPercentage};
        match (&self.0, &other.0) {
            (Keyword(first), Keyword(second)) => first == second,
            (LengthPercentage(_), LengthPercentage(_)) | (FitContent(_), FitContent(_)) => true,
            (CalcSize(_), CalcSize(_) | Keyword(_) | LengthPercentage(_))
            | (Keyword(_) | LengthPercentage(_), CalcSize(_))
                if matches!(operation, ComputedOperation::Interpolate(_)) =>
            {
                let first = self.as_calc_size();
                let second = other.as_calc_size();
                first
                    .zip(second)
                    .is_some_and(|(first, second)| first.can_interpolate(&second))
            }
            _ => false,
        }
    }

    pub(super) fn combine(
        &self,
        other: &Self,
        operation: ComputedOperation,
    ) -> Result<Self, ComputedError> {
        use ComputedSizeValue::{CalcSize, FitContent, Keyword, LengthPercentage};
        match (&self.0, &other.0) {
            (Keyword(first), Keyword(second)) if first == second => Ok(other.clone()),
            (LengthPercentage(first), LengthPercentage(second)) => {
                Self::length_percentage(first.combine(second, operation)?)
            }
            (FitContent(first), FitContent(second)) => {
                Self::fit_content(first.combine(second, operation)?)
            }
            (CalcSize(_), CalcSize(_) | Keyword(_) | LengthPercentage(_))
            | (Keyword(_) | LengthPercentage(_), CalcSize(_))
                if let ComputedOperation::Interpolate(progress) = operation =>
            {
                let (first, second) = self
                    .as_calc_size()
                    .zip(other.as_calc_size())
                    .ok_or_else(|| ComputedError::new(ComputedErrorKind::IncompatibleTypes))?;
                first
                    .interpolate(&second, progress)?
                    .map(Self::calc_size)
                    .ok_or_else(|| ComputedError::new(ComputedErrorKind::IncompatibleTypes))
            }
            _ => Err(ComputedError::new(ComputedErrorKind::IncompatibleTypes).during(operation)),
        }
    }

    fn as_calc_size(&self) -> Option<CalcSize> {
        match &self.0 {
            ComputedSizeValue::Keyword(keyword) => Some(CalcSize::from_keyword(*keyword)),
            ComputedSizeValue::LengthPercentage(numeric) => {
                CalcSize::from_numeric(numeric.clone()).ok()
            }
            ComputedSizeValue::CalcSize(value) => Some(value.clone()),
            ComputedSizeValue::FitContent(_) | ComputedSizeValue::UnavailableCalcSize(_) => None,
        }
    }
}
