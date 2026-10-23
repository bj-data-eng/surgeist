//! Exact authored ordinary time dimensions and duration range admission.
use crate::{
    CssComponentValue, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssTimeCalculation, CssTimeUnit, CssValueOrigin,
};

pub(crate) const fn suffix(unit: CssTimeUnit) -> &'static str {
    crate::numeric::units::suffix(crate::CssNumericUnit::Time(unit))
}

crate::dimension::dimension_literal_owner!(
    /// An exact ordinary time dimension, retaining spelling, unit and provenance.
    ///
    /// # Specified serialization
    ///
    /// Emits canonical seconds with at most six fractional places. Conversion precedes rounding,
    /// nearest with ties away from zero. Original coefficients, units and origins remain exact.
    /// This literal operation does no calculation projection. Ordinary output rounds to six places
    /// without an exponential fallback, and the byte limit covers the rounded text including the
    /// seconds suffix. Calculation values share the six-place text policy while retaining their
    /// arithmetic precision and range limitations. This operation does not certify recovered
    /// syntax as a clean report.
    CssTimeLiteral,
    CssTimeUnit,
    suffix
);
impl CssTimeLiteral {
    /// Exact ordinary duration equivalence for specified shorthand omission.
    /// Coefficients and origins remain authored; only standard s/ms conversion
    /// participates, without floating-point rounding or exponent expansion.
    fn equivalent(&self, other: &Self) -> bool {
        fn seconds(literal: &CssTimeLiteral) -> crate::exact_decimal::LexicalDecimal<'_> {
            crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation()).shifted(
                crate::numeric::units::decimal_shift(crate::CssNumericUnit::Time(literal.unit()))
                    .expect("decimal time ratio"),
            )
        }
        seconds(self).value_eq(&seconds(other))
    }
}

crate::dimension::dimension_value_owner!(
    /// An authored time dimension or Time-root calculation; never a bare number.
    ///
    /// Checked construction requires closure; parsing can retain recovered syntax with diagnostics.
    ///
    /// # Specified serialization
    ///
    /// Emits canonical seconds with at most six fractional places. Conversion precedes rounding,
    /// nearest with ties away from zero. Original coefficients, units and origins remain exact.
    /// Finite calculation text rounds the projected binary64 value with the same policy; arithmetic
    /// precision and range remain unfinished. Literals perform no math projection. Ordinary output
    /// rounds to six places without an exponential fallback. This operation does not certify
    /// recovered syntax as a clean report.
    CssTimeValue,
    CssTimeLiteral,
    CssTimeCalculation
);
impl CssTimeValue {
    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssNumericConstructionError> {
        crate::dimension::DimensionValue::from_parser_component(
            component,
            context,
            crate::numeric::CalculationRoot::Time,
            true,
            CssTimeCalculation::from_expression,
        )
        .map(|value| Self { value })
    }
    pub(crate) fn first_implicit_origin(&self) -> Option<&CssValueOrigin> {
        self.value.first_implicit_origin()
    }
    pub(crate) fn ensure_closed(&self) -> Result<(), CssNumericConstructionError> {
        self.value.ensure_closed()
    }
    /// Whether this is an ordinary literal whose exact value is zero; math,
    /// even math that evaluates to zero, is not.
    pub(crate) fn is_exact_ordinary_zero(&self) -> bool {
        self.literal().is_some_and(|literal| {
            crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation()).is_zero()
        })
    }
    pub(crate) fn specified_value_eq(&self, other: &Self) -> bool {
        match (self.literal(), other.literal()) {
            (Some(left), Some(right)) => left.equivalent(right),
            _ => self.structural_eq(other),
        }
    }
    /// Exact specified equality for the new bounded inverse consumer. Ordinary
    /// unit equivalence remains borrowed; retained math schedules its real work.
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<bool, crate::CssSerializationError> {
        match (&self.value, &other.value) {
            (
                crate::dimension::DimensionValue::Literal(left),
                crate::dimension::DimensionValue::Literal(right),
            ) => Ok(left.equivalent(right)),
            (
                crate::dimension::DimensionValue::Calculation(left),
                crate::dimension::DimensionValue::Calculation(right),
            ) => left
                .expression
                .specified_inverse_eq(&right.expression, context),
            _ => Ok(false),
        }
    }
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        self.value.structural_eq(&other.value)
    }
}

/// An authored duration with a nonnegative ordinary literal or a retained time calculation.
///
/// # Specified serialization
///
/// Emits canonical seconds with at most six fractional places. Conversion precedes rounding,
/// nearest with ties away from zero. Original coefficients, units and origins remain exact. Finite
/// calculation text rounds the projected binary64 value with the same policy; arithmetic precision
/// and range remain unfinished. Literals perform no math projection. Ordinary output rounds to six
/// places without an exponential fallback. This operation does not certify recovered syntax as a
/// clean report.
#[derive(Clone, Debug)]
pub struct CssDuration {
    time: CssTimeValue,
}
impl CssDuration {
    /// Rejects recovered input and genuinely negative ordinary literals; calculations retain their
    /// authored range.
    pub fn try_new(time: CssTimeValue) -> Result<Self, CssNumericConstructionError> {
        time.ensure_closed()?;
        Self::from_parser_value(time)
    }
    pub(crate) fn from_parser_value(
        time: CssTimeValue,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(literal) = time.literal()
            && crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation())
                .negative
        {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::OutOfRange,
                literal.origin().clone(),
            ));
        }
        Ok(Self { time })
    }
    /// Borrows the retained time value without resolving calculations.
    pub const fn time(&self) -> &CssTimeValue {
        &self.time
    }
    /// Borrows the original selected branch provenance.
    pub fn origin(&self) -> &CssValueOrigin {
        self.time.origin()
    }
    pub(crate) fn specified_value_eq(&self, other: &Self) -> bool {
        self.time.specified_value_eq(&other.time)
    }
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<bool, crate::CssSerializationError> {
        if self.time.calculation().is_none() && other.time.calculation().is_none() {
            Ok(self.specified_value_eq(other))
        } else {
            self.time.specified_inverse_eq(&other.time, context)
        }
    }
    pub(crate) fn is_exact_ordinary_zero(&self) -> bool {
        self.time.is_exact_ordinary_zero()
    }
}
impl PartialEq for CssDuration {
    fn eq(&self, other: &Self) -> bool {
        self.time.structural_eq(&other.time)
    }
}
impl Eq for CssDuration {}
