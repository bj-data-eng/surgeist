//! Exact authored Fonts 4 font sizes, before parent-relative resolution.

use crate::numeric::{CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;

type SerializationResult<T> = Result<T, CssSerializationError>;

/// An authored `font-size` value. Relative sizes and math remain unresolved.
/// Equality compares authored structure without comparing source origins.
///
/// # Specified serialization
///
/// Serializes the authored keyword or exact length-percentage without resolving it.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssFontSize {
    /// `xx-small`: the smallest `<absolute-size>` keyword, mapped to a size by the user agent's font-size table.
    XxSmall,
    /// `x-small`: an `<absolute-size>` keyword between `xx-small` and `small`.
    XSmall,
    /// `small`: an `<absolute-size>` keyword just below `medium`.
    Small,
    /// `medium`: the reference middle `<absolute-size>` keyword, and the initial `font-size`.
    Medium,
    /// `large`: an `<absolute-size>` keyword just above `medium`.
    Large,
    /// `x-large`: an `<absolute-size>` keyword between `large` and `xx-large`.
    XLarge,
    /// `xx-large`: an `<absolute-size>` keyword above `x-large`.
    XxLarge,
    /// `xxx-large`: the largest `<absolute-size>` keyword.
    XxxLarge,
    /// `larger`: a `<relative-size>` keyword, interpreted relative to the parent's computed `font-size` and possibly the table of font sizes; the next larger entry, or a ratio of about 1.2 to 1.5.
    Larger,
    /// `smaller`: a `<relative-size>` keyword, interpreted relative to the parent's computed `font-size` and possibly the table of font sizes; the previous entry, or a ratio of about 1.2 to 1.5.
    Smaller,
    /// `math`: special mathematical scaling rules are applied when determining the computed value.
    Math,
    /// A non-negative `<length-percentage>`; a percentage is relative to the parent's computed `font-size`. It stays unresolved.
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
}

impl PartialEq for CssFontSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            (Self::LengthPercentage(_), _) | (_, Self::LengthPercentage(_)) => false,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}
impl Eq for CssFontSize {}

impl CssFontSize {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
            }
            keyword => {
                context.charge_nodes(1)?;
                context.append(output, keyword.as_css())?;
            }
        }
        Ok(())
    }

    fn as_css(&self) -> &'static str {
        match self {
            Self::XxSmall => "xx-small",
            Self::XSmall => "x-small",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::XLarge => "x-large",
            Self::XxLarge => "xx-large",
            Self::XxxLarge => "xxx-large",
            Self::Larger => "larger",
            Self::Smaller => "smaller",
            Self::Math => "math",
            Self::LengthPercentage(_) => unreachable!("numeric sizes serialize separately"),
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFontSize {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontSize);
