//! Authored preferred aspect ratios and exact ratio operands.
//! Ordinary operands emit at most six fractional places, nearest with ties away
//! from zero; exact identity and range checks remain unchanged. Finite calculated
//! operands use the same text policy while retaining existing arithmetic and ranges.

use crate::{
    CssComponentValue, CssComponentValueRef, CssNumberCalculation, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssSerializationError, CssValueOrigin, CssValueTokenRef,
    numeric_formatting::format_css_number, specified_serialization::SpecifiedSerializationContext,
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
        let lexical = crate::exact_decimal::LexicalDecimal::new(number.representation());
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
        // This domain still retains recovered syntax from checked construction.
        crate::specified_numeric::admit_calculation(
            calculation,
            CssNumberCalculation::components,
            crate::specified_numeric::CalculationAdmission::Recovering,
            Self::try_from_component,
            |calculation| Self {
                value: RatioOperandValue::Calculation(calculation),
            },
        )
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

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        match &self.value {
            RatioOperandValue::Number(component) => {
                context.charge_nodes(1)?;
                if context.output_suppressed() {
                    return Ok(());
                }
                let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
                    component.view()
                else {
                    unreachable!("checked ratio number")
                };
                let text =
                    format_css_number(number.representation(), 0, context.remaining_bytes())?;
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
///
/// # Specified serialization
///
/// Serializes the specified pair; an omitted denominator is emitted as one.
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

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        self.numerator.append_specified(context, output)?;
        context.append(output, " / ")?;
        if let Some(denominator) = &self.denominator {
            denominator.append_specified(context, output)
        } else {
            context.charge_nodes(1)?;
            context.append(output, "1")
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssSpecifiedRatio {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssSpecifiedRatio);

impl crate::specified_rule_serialization::SpecifiedProvider for crate::CssAspectRatioValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")?;
            }
            Self::Ratio(ratio) => ratio.append_specified(context, output)?,
            Self::AutoRatio(ratio) => {
                context.charge_nodes(1)?;
                context.append(output, "auto ")?;
                ratio.append_specified(context, output)?;
            }
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(crate::CssAspectRatioValue);
