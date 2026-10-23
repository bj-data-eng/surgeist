//! Exact authored line minima for `orphans` and `widows`.

use crate::{
    CssComponentValue, CssIntegerCalculation, CssIntegerLiteral, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssPositiveIntegerLiteral, CssPositiveIntegerValue,
    CssValueOrigin,
};

/// A positive authored line count or deferred integer math.
///
/// Fragmentation 3 `<integer [1,∞]>` is the shared positive-integer production.
///
/// # Specified serialization
///
/// Serializes without resolving symbolic math.
#[derive(Clone, Debug)]
pub struct CssPageLineMinimum {
    value: CssPositiveIntegerValue,
}

impl PartialEq for CssPageLineMinimum {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl CssPageLineMinimum {
    /// Builds a positive programmatic count.
    #[must_use]
    pub fn try_literal(value: i32) -> Option<Self> {
        (value > 0).then(|| {
            Self::try_from_component(
                CssComponentValue::try_number(&value.to_string()).expect("integer spelling"),
            )
            .expect("positive integer component")
        })
    }

    /// The intrinsic initial value of both properties.
    #[must_use]
    pub fn initial() -> Self {
        Self::try_literal(2).expect("positive initial line minimum")
    }

    /// Checks exact integer syntax and positivity without bounding magnitude.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        let integer = CssIntegerLiteral::try_from_component(component.clone()).map_err(|_| {
            CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            )
        })?;
        let positive = CssPositiveIntegerLiteral::try_new(integer).ok_or_else(|| {
            CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            )
        })?;
        Ok(Self {
            value: CssPositiveIntegerValue::Literal(positive),
        })
    }

    /// Retains math roots, while bare numeric roots pass through literal validation.
    pub fn try_from_calculation(
        calculation: CssIntegerCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        // This domain still retains recovered syntax from checked construction.
        crate::specified_numeric::admit_calculation(
            calculation,
            CssIntegerCalculation::components,
            crate::specified_numeric::CalculationAdmission::Recovering,
            Self::try_from_component,
            |calculation| Self {
                value: CssPositiveIntegerValue::Calculation(calculation),
            },
        )
    }

    /// Borrows the exact positive integer token, including its origin.
    #[must_use]
    pub fn exact_literal(&self) -> Option<&CssPositiveIntegerLiteral> {
        match &self.value {
            CssPositiveIntegerValue::Literal(value) => Some(value),
            CssPositiveIntegerValue::Calculation(_) => None,
        }
    }

    /// Projects a literal to `i32` only when its exact magnitude fits.
    #[must_use]
    pub fn literal(&self) -> Option<i32> {
        self.exact_literal().and_then(|value| {
            crate::integer_value::exact_i32(value.integer().numeric().representation())
        })
    }

    /// Borrows deferred integer math.
    #[must_use]
    pub fn calculation(&self) -> Option<&CssIntegerCalculation> {
        match &self.value {
            CssPositiveIntegerValue::Literal(_) => None,
            CssPositiveIntegerValue::Calculation(value) => Some(value),
        }
    }

    /// Returns the authored or programmatic root origin.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            CssPositiveIntegerValue::Literal(value) => value.integer().origin(),
            CssPositiveIntegerValue::Calculation(value) => value.origin(),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssPageLineMinimum {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.value.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPageLineMinimum);
