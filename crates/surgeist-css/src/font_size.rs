//! Exact authored Fonts 4 font sizes, before parent-relative resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssLengthPercentage, CssNonNegative, CssNumeric, CssSerializationError};

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
    XxSmall,
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    XxLarge,
    XxxLarge,
    Larger,
    Smaller,
    Math,
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
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontSize {
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
crate::specified_rule_serialization::specified_serialization!(CssFontSize);
