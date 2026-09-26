//! Authored preferred aspect ratios and exact ratio operands.

use crate::{
    CssComponentValue, CssComponentValueRef, CssNumberCalculation, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
    specified_serialization::{SpecifiedSerializationContext, format_lexical_shift},
};

/// One checked, nonnegative ratio number or deferred number-valued math root.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRatioOperand {
    value: RatioOperandValue,
}

#[derive(Clone, Debug, PartialEq)]
enum RatioOperandValue {
    Number(Box<CssComponentValue>),
    Calculation(CssNumberCalculation),
}

impl CssRatioOperand {
    /// Checks an exact ordinary number without rounding its authored magnitude.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        let lexical = crate::opacity_scalar::LexicalDecimal::new(number.representation());
        if lexical.negative {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: RatioOperandValue::Number(Box::new(component)),
        })
    }

    /// Checks the original root of a typed number calculation. Bare ordinary
    /// numbers take the exact literal path, while math roots remain symbolic.
    pub fn try_from_calculation(
        calculation: CssNumberCalculation,
    ) -> Result<Self, CssNumericConstructionError> {
        let components = calculation.components().items();
        let Some(component) = components.iter().find(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Comment(_)
                    | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        }) else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::EmptyValue,
                None,
            ));
        };
        match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => {
                Self::try_from_component(component.clone())
            }
            CssComponentValueRef::Function(_) => Ok(Self {
                value: RatioOperandValue::Calculation(calculation),
            }),
            _ => Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(component),
            )),
        }
    }

    /// Borrows the exact ordinary number, if this operand has one.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            RatioOperandValue::Number(component) => Some(component),
            RatioOperandValue::Calculation(_) => None,
        }
    }

    /// Borrows deferred number math, if this operand has a math root.
    pub fn calculation(&self) -> Option<&CssNumberCalculation> {
        match &self.value {
            RatioOperandValue::Number(_) => None,
            RatioOperandValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Retains parsed or programmatic provenance of the root.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            RatioOperandValue::Number(component) => component.origin(),
            RatioOperandValue::Calculation(calculation) => calculation.origin(),
        }
    }

    pub(crate) fn exact_positive_f32(&self) -> Option<f32> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
            self.literal_component()?.view()
        else {
            unreachable!("checked ratio number")
        };
        crate::opacity_scalar::exact_legacy_value(number.representation()).filter(|n| *n > 0.0)
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match &self.value {
            RatioOperandValue::Number(component) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
                    component.view()
                else {
                    unreachable!("checked ratio number")
                };
                let text =
                    format_lexical_shift(number.representation(), 0, context.remaining_bytes())?;
                context.append(output, &text)
            }
            RatioOperandValue::Calculation(calculation) => {
                crate::numeric::project_specified_into(&calculation.expression, context, output)?;
                Ok(())
            }
        }
    }
}

/// An authored ratio with an optional denominator; omission means one.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSpecifiedRatio {
    numerator: CssRatioOperand,
    denominator: Option<CssRatioOperand>,
}

impl CssSpecifiedRatio {
    /// Composes already-checked operands without changing their provenance.
    pub const fn new(numerator: CssRatioOperand, denominator: Option<CssRatioOperand>) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub const fn numerator(&self) -> &CssRatioOperand {
        &self.numerator
    }

    pub const fn denominator(&self) -> Option<&CssRatioOperand> {
        self.denominator.as_ref()
    }

    /// Serializes the specified pair; an omitted denominator is emitted as one.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.numerator.append_specified(context, output)?;
        context.append(output, " / ")?;
        if let Some(denominator) = &self.denominator {
            denominator.append_specified(context, output)
        } else {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            context.append(output, "1")
        }
    }
}

impl crate::CssAspectRatioValue {
    /// Produces canonical specified aspect-ratio text with one cumulative budget.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        match self {
            Self::Auto => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, "auto")?;
            }
            Self::Ratio(ratio) => ratio.append_specified(&mut context, &mut output)?,
            Self::AutoRatio(ratio) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, "auto ")?;
                ratio.append_specified(&mut context, &mut output)?;
            }
        }
        Ok(output)
    }
}
