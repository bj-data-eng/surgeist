//! Exact authored CSS Fonts 4 weights before cascade, matching, or range ordering.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssComponentValue, CssComponentValueRef, CssNumberCalculation, CssNumericConstructionError,
    CssNumericConstructionErrorKind, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// An exact authored CSS Fonts 4 `<number [1,1000]>`.
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
            value: FontWeightNumberValue::Calculation(calculation),
        })
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

    /// Serializes without resolving symbolic number math.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically with cumulative resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let captured = self.capture_specified(context)?;
        let output = &mut writer.css;
        context.append(output, &captured)?;
        Ok(())
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

/// A common absolute weight accepted by the property and font-face descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAbsoluteFontWeight {
    Normal,
    Bold,
    Number(CssFontWeightNumber),
}

/// An authored `font-weight` property value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontWeight {
    Absolute(CssAbsoluteFontWeight),
    Bolder,
    Lighter,
}

/// An authored `@font-face` weight descriptor before computed range ordering.
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
    /// Serializes the authored keyword or number without matching a font.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under cumulative resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal | Self::Bold => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
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

impl CssFontWeight {
    /// Serializes the authored absolute or relative weight.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under cumulative resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Absolute(absolute) => absolute.append_specified(context, output)?,
            Self::Bolder | Self::Lighter => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
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

impl CssFontFaceWeight {
    /// Serializes `auto` or the exact authored one/two-value range.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one descriptor node and every explicitly authored endpoint.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
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
        self.append_specified(context, output)?;
        Ok(())
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
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
