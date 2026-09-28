//! Exact authored Fonts 4 style values, before matching or computed angle ranges.

use crate::specified_serialization::{SpecifiedSerializationContext, format_lexical_shift};
use crate::{
    CssAngleCalculation, CssAngleUnit, CssComponentValue, CssComponentValueRef,
    CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
};

type ConstructionResult<T> = Result<T, CssNumericConstructionError>;
type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// A keyword shared by the `font-style` property and its font-face descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStyleKeyword {
    Normal,
    Italic,
    Left,
    Right,
}

impl CssFontStyleKeyword {
    const fn as_css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Italic => "italic",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// An authored oblique angle with exact literal spelling or symbolic angle math.
#[derive(Clone, Debug)]
pub struct CssFontObliqueAngle {
    value: ObliqueAngleValue,
}

#[derive(Clone, Debug)]
enum ObliqueAngleValue {
    Literal(Box<CssComponentValue>),
    Calculation(CssAngleCalculation),
}

impl PartialEq for CssFontObliqueAngle {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (ObliqueAngleValue::Literal(a), ObliqueAngleValue::Literal(b)) => {
                a.structural_eq_ignoring_origin(b)
            }
            (ObliqueAngleValue::Calculation(a), ObliqueAngleValue::Calculation(b)) => {
                a.expression.structural_eq(&b.expression)
            }
            _ => false,
        }
    }
}
impl Eq for CssFontObliqueAngle {}

impl CssFontObliqueAngle {
    /// Checks one literal angle against the inclusive Fonts 4 oblique range.
    /// Degrees, gradians and turns use exact lexical bounds; radians compare
    /// their original decimal parsed as binary64 against `FRAC_PI_2`. The
    /// accepted component keeps its original spelling and source origin.
    pub fn try_from_component(component: CssComponentValue) -> ConstructionResult<Self> {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
            component.view()
        else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        let spelling = number.representation();
        let in_range = match unit.to_ascii_lowercase().as_str() {
            "deg" => crate::exact_decimal::LexicalDecimal::new(spelling).absolute_at_most("9", 1),
            "grad" => crate::exact_decimal::LexicalDecimal::new(spelling).absolute_at_most("1", 2),
            "turn" => {
                crate::exact_decimal::LexicalDecimal::new(spelling).absolute_at_most("25", -2)
            }
            "rad" => spelling
                .parse::<f64>()
                .is_ok_and(|value| value.abs() <= std::f64::consts::FRAC_PI_2),
            _ => {
                return Err(CssNumericConstructionError::at(
                    CssNumericConstructionErrorKind::RootDomainMismatch,
                    Some(&component),
                ));
            }
        };
        if !in_range {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::OutOfRange,
                Some(&component),
            ));
        }
        Ok(Self {
            value: ObliqueAngleValue::Literal(Box::new(component)),
        })
    }

    /// Retains already-admitted symbolic angle math and its recovery provenance;
    /// a bare dimension root reenters literal bounds.
    pub fn try_from_calculation(calculation: CssAngleCalculation) -> ConstructionResult<Self> {
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
            value: ObliqueAngleValue::Calculation(calculation),
        })
    }

    /// Borrows the original literal, including its unit and source origin.
    pub fn literal_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            ObliqueAngleValue::Literal(component) => Some(component),
            ObliqueAngleValue::Calculation(_) => None,
        }
    }

    /// Borrows symbolic angle math, if authored.
    pub fn calculation(&self) -> Option<&CssAngleCalculation> {
        match &self.value {
            ObliqueAngleValue::Literal(_) => None,
            ObliqueAngleValue::Calculation(calculation) => Some(calculation),
        }
    }

    /// Returns the parsed or programmatic root origin.
    pub fn origin(&self) -> &CssValueOrigin {
        match &self.value {
            ObliqueAngleValue::Literal(component) => component.origin(),
            ObliqueAngleValue::Calculation(calculation) => calculation.origin(),
        }
    }

    /// Serializes the specified angle without computing font matching.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under the supplied resource limits.
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
        match &self.value {
            ObliqueAngleValue::Literal(component) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
                    component.view()
                else {
                    unreachable!("checked literal angle")
                };
                let canonical_unit = match unit.to_ascii_lowercase().as_str() {
                    "deg" => CssAngleUnit::Degrees,
                    "grad" => CssAngleUnit::Gradians,
                    "rad" => CssAngleUnit::Radians,
                    "turn" => CssAngleUnit::Turns,
                    _ => unreachable!("checked angle unit"),
                };
                let suffix = match canonical_unit {
                    CssAngleUnit::Degrees => "deg",
                    CssAngleUnit::Gradians => "grad",
                    CssAngleUnit::Radians => "rad",
                    CssAngleUnit::Turns => "turn",
                };
                let coefficient_limit = context
                    .remaining_bytes()
                    .checked_sub(suffix.len())
                    .ok_or_else(|| {
                        CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::ByteLimit,
                        )
                    })?;
                let coefficient =
                    format_lexical_shift(number.representation(), 0, coefficient_limit)?;
                context.append(output, &coefficient)?;
                context.append(output, suffix)
            }
            ObliqueAngleValue::Calculation(calculation) => {
                let (captured, _) =
                    crate::numeric::capture_specified(&calculation.expression, context)?;
                context.append(output, &captured)
            }
        }
    }
}

/// An authored `font-style` property value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontStyle {
    Keyword(CssFontStyleKeyword),
    Oblique { angle: Option<CssFontObliqueAngle> },
}

/// One or two authored oblique descriptor endpoints, in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceObliqueRange {
    start: CssFontObliqueAngle,
    end: Option<CssFontObliqueAngle>,
}

impl CssFontFaceObliqueRange {
    /// Retains already-checked endpoints, including descending pairs.
    #[must_use]
    pub const fn new(start: CssFontObliqueAngle, end: Option<CssFontObliqueAngle>) -> Self {
        Self { start, end }
    }

    #[must_use]
    /// Borrows the required first authored endpoint.
    pub const fn start(&self) -> &CssFontObliqueAngle {
        &self.start
    }

    #[must_use]
    /// Borrows an explicitly authored second endpoint, if present.
    pub const fn end(&self) -> Option<&CssFontObliqueAngle> {
        self.end.as_ref()
    }

    /// Serializes the exact authored one- or two-endpoint range.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges both endpoints and their separator to one cumulative budget.
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
        self.start.append_specified(context, output)?;
        if let Some(end) = &self.end {
            context.append(output, " ")?;
            end.append_specified(context, output)?;
        }
        Ok(())
    }
}

/// An authored `@font-face` style descriptor, before font matching.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceStyle {
    Auto,
    Keyword(CssFontStyleKeyword),
    Oblique {
        range: Option<CssFontFaceObliqueRange>,
    },
}

fn append_keyword(
    keyword: CssFontStyleKeyword,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, keyword.as_css())
}

impl CssFontStyle {
    /// Serializes the authored keyword or optional oblique angle.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges the keyword and any explicit angle to one cumulative budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Keyword(keyword) => append_keyword(*keyword, context, output)?,
            Self::Oblique { angle } => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "oblique")?;
                if let Some(angle) = angle {
                    context.append(output, " ")?;
                    angle.append_specified(context, output)?;
                }
            }
        }
        Ok(())
    }
}

impl CssFontFaceStyle {
    /// Serializes `auto`, a keyword, or the authored oblique range.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges the descriptor keyword and each explicit endpoint cumulatively.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        match self {
            Self::Auto => append_text("auto", &mut context, &mut output)?,
            Self::Keyword(keyword) => append_keyword(*keyword, &mut context, &mut output)?,
            Self::Oblique { range } => {
                append_text("oblique", &mut context, &mut output)?;
                if let Some(range) = range {
                    context.append(&mut output, " ")?;
                    range.append_specified(&mut context, &mut output)?;
                }
            }
        }
        Ok(output)
    }
}

fn append_text(
    text: &str,
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
) -> SerializationResult<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, text)
}
