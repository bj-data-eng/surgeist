//! Exact authored CSS Fonts 4 weights before cascade, matching, or range ordering.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssComponentValue, CssComponentValueRef, CssNumberCalculation, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssSerializationError, CssValueOrigin, CssValueTokenRef,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSerializationError>;

/// An exact authored CSS Fonts 4 `<number [1,1000]>`.
///
/// # Specified serialization
///
/// Serializes without resolving symbolic number math.
#[derive(Clone, Debug)]
pub struct CssFontWeightNumber {
    value: FontWeightNumberValue,
}

#[derive(Clone, Debug)]
enum FontWeightNumberValue {
    Literal(Box<CssComponentValue>),
    Calculation(CssNumberCalculation),
}

impl PartialEq for CssFontWeightNumber {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (FontWeightNumberValue::Literal(left), FontWeightNumberValue::Literal(right)) => {
                left.structural_eq_ignoring_origin(right)
            }
            (
                FontWeightNumberValue::Calculation(left),
                FontWeightNumberValue::Calculation(right),
            ) => left.expression.structural_eq(&right.expression),
            _ => false,
        }
    }
}

impl Eq for CssFontWeightNumber {}

impl CssFontWeightNumber {
    /// Checks the original number token against inclusive `[1,1000]` exactly.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        if !crate::exact_decimal::LexicalDecimal::new(number.representation())
            .in_font_weight_range()
        {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: FontWeightNumberValue::Literal(Box::new(component)),
        })
    }

    /// Retains checked number math; a bare number root reenters literal bounds.
    pub fn try_from_calculation(calculation: CssNumberCalculation) -> ConstructionResult<Self> {
        // This domain still retains recovered syntax from checked construction.
        crate::specified_numeric::admit_calculation(
            calculation,
            CssNumberCalculation::components,
            crate::specified_numeric::CalculationAdmission::Recovering,
            Self::try_from_component,
            |calculation| Self {
                value: FontWeightNumberValue::Calculation(calculation),
            },
        )
    }

    /// Borrows the exact authored number token, if this is a literal.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            FontWeightNumberValue::Literal(component) => Some(component),
            FontWeightNumberValue::Calculation(_) => None,
        }
    }

    /// Borrows checked symbolic number math, when present.
    pub fn calculation(&self) -> Option<&CssNumberCalculation> {
        match &self.value {
            FontWeightNumberValue::Literal(_) => None,
            FontWeightNumberValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the original parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            FontWeightNumberValue::Literal(component) => component.origin(),
            FontWeightNumberValue::Calculation(calculation) => calculation.origin(),
        }
    }

    fn capture_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<String> {
        match &self.value {
            FontWeightNumberValue::Literal(component) => {
                crate::specified_numeric::capture_literal(component, context)
            }
            FontWeightNumberValue::Calculation(calculation) => {
                crate::numeric::capture_specified(&calculation.expression, context)
                    .map(|(text, _)| text)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontWeightNumber {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontWeightNumber);

/// A common absolute weight accepted by the property and font-face descriptor.
///
/// # Specified serialization
///
/// Serializes the authored keyword or number without matching a font.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAbsoluteFontWeight {
    Normal,
    Bold,
    Number(CssFontWeightNumber),
}

/// An authored `font-weight` property value.
///
/// # Specified serialization
///
/// Serializes the authored absolute or relative weight.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontWeight {
    Absolute(CssAbsoluteFontWeight),
    Bolder,
    Lighter,
}

/// An authored `@font-face` weight descriptor before computed range ordering.
///
/// # Specified serialization
///
/// Serializes `auto` or the exact authored one/two-value range.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceWeight {
    Auto,
    Range {
        start: CssAbsoluteFontWeight,
        end: Option<CssAbsoluteFontWeight>,
    },
}

impl CssAbsoluteFontWeight {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal | Self::Bold => {
                context.charge_nodes(1)?;
                context.append(
                    output,
                    if matches!(self, Self::Normal) {
                        "normal"
                    } else {
                        "bold"
                    },
                )
            }
            Self::Number(number) => {
                let captured = number.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssAbsoluteFontWeight {
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
crate::specified_rule_serialization::specified_serialization!(CssAbsoluteFontWeight);

impl CssFontWeight {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Absolute(absolute) => absolute.append_specified(context, output)?,
            Self::Bolder | Self::Lighter => {
                context.charge_nodes(1)?;
                context.append(
                    output,
                    if matches!(self, Self::Bolder) {
                        "bolder"
                    } else {
                        "lighter"
                    },
                )?;
            }
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontWeight {
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
crate::specified_rule_serialization::specified_serialization!(CssFontWeight);

impl CssFontFaceWeight {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        context.charge_nodes(1)?;
        match self {
            Self::Auto => context.append(output, "auto")?,
            Self::Range { start, end } => {
                start.append_specified(context, output)?;
                if let Some(end) = end {
                    let omit_end = context.output_suppressed()
                        || start.specified_semantically_eq(end, context)?;
                    let suppressed = context.replace_output_suppression(omit_end);
                    let result = (|| {
                        context.append(output, " ")?;
                        end.append_specified(context, output)
                    })();
                    context.replace_output_suppression(suppressed);
                    result?;
                }
            }
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceWeight {
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
crate::specified_rule_serialization::specified_serialization!(CssFontFaceWeight);

impl CssAbsoluteFontWeight {
    fn specified_semantically_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<bool> {
        if let (Self::Number(a), Self::Number(b)) = (self, other)
            && let (Some(a), Some(b)) = (a.calculation(), b.calculation())
        {
            return a.expression.specified_identity_eq(&b.expression, context);
        }
        Ok(match (self, other) {
            (Self::Number(a), Self::Number(b)) => crate::font_rule_serialization::equal_literals(
                a.literal_component(),
                b.literal_component(),
            )
            .unwrap_or_else(|| a == b),
            (Self::Normal, Self::Number(value)) | (Self::Number(value), Self::Normal) => {
                crate::font_rule_serialization::literal_equals(value.literal_component(), "400")
            }
            (Self::Bold, Self::Number(value)) | (Self::Number(value), Self::Bold) => {
                crate::font_rule_serialization::literal_equals(value.literal_component(), "700")
            }
            _ => self == other,
        })
    }
}
