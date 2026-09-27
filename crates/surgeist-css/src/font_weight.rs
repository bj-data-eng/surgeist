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
        let mut context = SpecifiedSerializationContext::new(limits);
        let captured = self.capture_specified(&mut context)?;
        let mut output = String::new();
        context.append(&mut output, &captured)?;
        Ok(output)
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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        match self {
            Self::Absolute(absolute) => absolute.append_specified(&mut context, &mut output)?,
            Self::Bolder | Self::Lighter => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(
                    &mut output,
                    if matches!(self, Self::Bolder) {
                        "bolder"
                    } else {
                        "lighter"
                    },
                )?;
            }
        }
        Ok(output)
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
        let mut context = SpecifiedSerializationContext::new(limits);
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let mut output = String::new();
        match self {
            Self::Auto => context.append(&mut output, "auto")?,
            Self::Range { start, end } => {
                start.append_specified(&mut context, &mut output)?;
                if let Some(end) = end {
                    context.append(&mut output, " ")?;
                    end.append_specified(&mut context, &mut output)?;
                }
            }
        }
        Ok(output)
    }
}
