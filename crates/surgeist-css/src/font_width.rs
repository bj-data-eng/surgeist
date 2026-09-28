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

/// Historical keyword type name for the unchanged `font` shorthand component.
pub type CssFontStretch = CssFontWidthKeyword;

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
