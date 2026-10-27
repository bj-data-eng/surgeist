//! Exact authored Fonts 4 widths before font matching or computed range ordering.

use crate::component::CssComponentValue;
use crate::numeric::{CssNonNegative, CssNumeric, CssPercentage};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;

type SerializationResult<T> = Result<T, CssSerializationError>;

/// The nine CSS Fonts 3 width keywords, also used by the Fonts 4 width property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontWidthKeyword {
    /// `normal`: the normal width, equivalent to `100%`.
    Normal,
    /// `ultra-condensed`: the most condensed width, equivalent to `50%`.
    UltraCondensed,
    /// `extra-condensed`: equivalent to `62.5%`.
    ExtraCondensed,
    /// `condensed`: equivalent to `75%`.
    Condensed,
    /// `semi-condensed`: equivalent to `87.5%`.
    SemiCondensed,
    /// `semi-expanded`: equivalent to `112.5%`.
    SemiExpanded,
    /// `expanded`: equivalent to `125%`.
    Expanded,
    /// `extra-expanded`: equivalent to `150%`.
    ExtraExpanded,
    /// `ultra-expanded`: the most expanded width, equivalent to `200%`.
    UltraExpanded,
}

/// An exact authored CSS Fonts 4 `font-width` property value.
///
/// # Specified serialization
///
/// Serializes the authored keyword or percentage without font selection.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontWidth {
    /// A width keyword.
    Keyword(CssFontWidthKeyword),
    /// A non-negative `<percentage>` width relative to the normal width; it may be a deferred calculation.
    Percentage(CssNumeric<CssPercentage, CssNonNegative>),
}

/// An authored `@font-face` width descriptor, before computed endpoint ordering.
///
/// # Specified serialization
///
/// Serializes `auto` or the exact authored one/two-value range.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceWidth {
    /// `auto`: for font selection the face is treated as if `normal` were chosen, and variation axis values are not clamped.
    Auto,
    /// One or two widths forming the range the face supports.
    Range {
        /// The first authored width.
        start: CssFontWidth,
        /// The second authored width, or `None` when only one was written.
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
    pub fn percentage(self) -> CssNumeric<CssPercentage, CssNonNegative> {
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
        CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_token(spelling).expect("fixed percentage token"),
        )
        .expect("fixed nonnegative percentage")
    }
}

impl CssFontWidth {
    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Keyword(keyword) => {
                context.charge_nodes(1)?;
                context.append(output, keyword.as_css())
            }
            Self::Percentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFontWidth {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontWidth);

impl CssFontFaceWidth {
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
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFontFaceWidth {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontFaceWidth);

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
                crate::rule::font_face::font_rule_serialization::equal_literals(
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
                crate::rule::font_face::font_rule_serialization::literal_equals(
                    value.literal_component(),
                    magnitude,
                )
            }
            _ => self == other,
        })
    }
}
