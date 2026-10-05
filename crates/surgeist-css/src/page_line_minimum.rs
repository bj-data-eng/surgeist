//! Exact authored line minima for `orphans` and `widows`.

use crate::{
    CssComponentValue, CssComponentValueRef, CssIntegerCalculation, CssIntegerLiteral,
    CssNumericConstructionError, CssNumericConstructionErrorKind, CssPositiveIntegerLiteral,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssValueOrigin,
};

#[derive(Clone, Debug)]
enum PageLineMinimumValue {
    Literal(CssPositiveIntegerLiteral),
    Calculation(CssIntegerCalculation),
}

/// A positive authored line count or deferred integer math.
#[derive(Clone, Debug)]
pub struct CssPageLineMinimum {
    value: PageLineMinimumValue,
}

impl PartialEq for CssPageLineMinimum {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (PageLineMinimumValue::Literal(left), PageLineMinimumValue::Literal(right)) => {
                left == right
            }
            (PageLineMinimumValue::Calculation(left), PageLineMinimumValue::Calculation(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
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
            value: PageLineMinimumValue::Literal(positive),
        })
    }

    /// Retains math roots, while bare numeric roots pass through literal validation.
    pub fn try_from_calculation(
        calculation: CssIntegerCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        let root = crate::specified_numeric::significant_root(calculation.components())?;
        if matches!(root.view(), CssComponentValueRef::Token(_)) {
            return Self::try_from_component(root.clone());
        }
        if !matches!(root.view(), CssComponentValueRef::Function(_)) {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(root),
            ));
        }
        Ok(Self {
            value: PageLineMinimumValue::Calculation(calculation),
        })
    }

    /// Borrows the exact positive integer token, including its origin.
    #[must_use]
    pub fn exact_literal(&self) -> Option<&CssPositiveIntegerLiteral> {
        match &self.value {
            PageLineMinimumValue::Literal(value) => Some(value),
            PageLineMinimumValue::Calculation(_) => None,
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
            PageLineMinimumValue::Literal(_) => None,
            PageLineMinimumValue::Calculation(value) => Some(value),
        }
    }

    /// Returns the authored or programmatic root origin.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            PageLineMinimumValue::Literal(value) => value.integer().origin(),
            PageLineMinimumValue::Calculation(value) => value.origin(),
        }
    }

    /// Serializes without resolving symbolic math.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under cumulative input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match &self.value {
            PageLineMinimumValue::Literal(value) => {
                value.integer().append_specified(context, output)?;
            }
            PageLineMinimumValue::Calculation(value) => {
                value.serialize_specified_into(context, output)?;
            }
        }
        Ok(())
    }
}
