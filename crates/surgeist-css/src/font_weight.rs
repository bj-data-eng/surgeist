//! Exact authored CSS Fonts 4 weights before cascade, matching, or range ordering.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssNumber, CssNumeric, CssNumericLiteral, CssRange, CssSerializationError};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// The Fonts 4 font-weight range `[1,1000]` of a [`CssNumber`] root.
///
/// An ordinary number is compared with both bounds exactly, without binary64
/// conversion; math defers the range to computed-value handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssFontWeightRange {}

impl crate::sealed::RangeBounds<CssNumber> for CssFontWeightRange {
    fn admits(literal: &CssNumericLiteral<CssNumber>) -> bool {
        crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation())
            .in_font_weight_range()
    }
}
impl CssRange<CssNumber> for CssFontWeightRange {}

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
    Number(CssNumeric<CssNumber, CssFontWeightRange>),
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
