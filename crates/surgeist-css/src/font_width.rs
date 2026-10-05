//! Exact authored Fonts 4 widths before font matching or computed range ordering.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssComponentValue, CssSpecifiedNonNegativePercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// The nine CSS Fonts 3 width keywords, also used by the Fonts 4 width property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontWidthKeyword {
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

/// An exact authored CSS Fonts 4 `font-width` property value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontWidth {
    Keyword(CssFontWidthKeyword),
    Percentage(CssSpecifiedNonNegativePercentage),
}

/// An authored `@font-face` width descriptor, before computed endpoint ordering.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceWidth {
    Auto,
    Range {
        start: CssFontWidth,
        end: Option<CssFontWidth>,
    },
}

impl CssFontFaceWidth {
    /// Returns the first authored width, or `None` for `auto`.
    #[must_use]
    pub const fn start(&self) -> Option<&CssFontWidth> {
        match self {
            Self::Auto => None,
            Self::Range { start, .. } => Some(start),
        }
    }

    /// Returns an explicitly authored second width, if present.
    #[must_use]
    pub const fn authored_end(&self) -> Option<&CssFontWidth> {
        match self {
            Self::Auto => None,
            Self::Range { end, .. } => end.as_ref(),
        }
    }

    /// Returns the effective second width, repeating the first when omitted.
    #[must_use]
    pub const fn end(&self) -> Option<&CssFontWidth> {
        match self {
            Self::Auto => None,
            Self::Range { start, end } => Some(match end {
                Some(value) => value,
                None => start,
            }),
        }
    }
}

impl CssFontWidthKeyword {
    pub(crate) const fn as_css(self) -> &'static str {
        match self {
            Self::UltraCondensed => "ultra-condensed",
            Self::ExtraCondensed => "extra-condensed",
            Self::Condensed => "condensed",
            Self::SemiCondensed => "semi-condensed",
            Self::Normal => "normal",
            Self::SemiExpanded => "semi-expanded",
            Self::Expanded => "expanded",
            Self::ExtraExpanded => "extra-expanded",
            Self::UltraExpanded => "ultra-expanded",
        }
    }

    /// Returns the exact specified percentage equivalent of a width keyword.
    #[must_use]
    pub fn percentage(self) -> CssSpecifiedNonNegativePercentage {
        let spelling = match self {
            Self::UltraCondensed => "50%",
            Self::ExtraCondensed => "62.5%",
            Self::Condensed => "75%",
            Self::SemiCondensed => "87.5%",
            Self::Normal => "100%",
            Self::SemiExpanded => "112.5%",
            Self::Expanded => "125%",
            Self::ExtraExpanded => "150%",
            Self::UltraExpanded => "200%",
        };
        CssSpecifiedNonNegativePercentage::try_from_component(
            CssComponentValue::try_token(spelling).expect("fixed percentage token"),
        )
        .expect("fixed nonnegative percentage")
    }
}

impl CssFontWidth {
    /// Serializes the authored keyword or percentage without font selection.
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
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Keyword(keyword) => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, keyword.as_css())
            }
            Self::Percentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}

impl CssFontFaceWidth {
    /// Serializes `auto` or the exact authored one/two-value range.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one descriptor node plus every explicitly authored width.
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
        context.charge_input(1)?;
        context.charge_projection(1)?;
        match self {
            Self::Auto => context.append(output, "auto")?,
            Self::Range { start, end } => {
                start.append_specified(context, output)?;
                if let Some(end) = end {
                    let omit_end = context.output_suppressed()
                        || start.specified_semantically_eq(end, context)?;
                    let previous = context.replace_output_suppression(omit_end);
                    let result = (|| {
                        context.append(output, " ")?;
                        end.append_specified(context, output)
                    })();
                    context.replace_output_suppression(previous);
                    result?;
                }
            }
        }
        Ok(())
    }
}

impl CssFontWidth {
    fn specified_semantically_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<bool> {
        if let (Self::Percentage(a), Self::Percentage(b)) = (self, other)
            && let (Some(a), Some(b)) = (a.calculation(), b.calculation())
        {
            return a.expression.specified_identity_eq(&b.expression, context);
        }
        Ok(match (self, other) {
            (Self::Percentage(a), Self::Percentage(b)) => {
                crate::font_rule_serialization::equal_literals(
                    a.literal_component(),
                    b.literal_component(),
                )
                .unwrap_or_else(|| a == b)
            }
            (Self::Keyword(keyword), Self::Percentage(value))
            | (Self::Percentage(value), Self::Keyword(keyword)) => {
                let magnitude = match keyword {
                    CssFontWidthKeyword::UltraCondensed => "50",
                    CssFontWidthKeyword::ExtraCondensed => "62.5",
                    CssFontWidthKeyword::Condensed => "75",
                    CssFontWidthKeyword::SemiCondensed => "87.5",
                    CssFontWidthKeyword::Normal => "100",
                    CssFontWidthKeyword::SemiExpanded => "112.5",
                    CssFontWidthKeyword::Expanded => "125",
                    CssFontWidthKeyword::ExtraExpanded => "150",
                    CssFontWidthKeyword::UltraExpanded => "200",
                };
                crate::font_rule_serialization::literal_equals(value.literal_component(), magnitude)
            }
            _ => self == other,
        })
    }
}
