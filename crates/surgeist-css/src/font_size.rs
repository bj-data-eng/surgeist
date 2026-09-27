//! Exact authored Fonts 4 font sizes, before parent-relative resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssSpecifiedNonNegativeLengthPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// An authored `font-size` value. Relative sizes and math remain unresolved.
/// Equality compares authored structure without comparing source origins.
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
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
}

impl PartialEq for CssFontSize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            (Self::LengthPercentage(_), _) | (_, Self::LengthPercentage(_)) => false,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}
impl Eq for CssFontSize {}

impl CssFontSize {
    /// Serializes the authored keyword or exact length-percentage without resolving it.
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
        match self {
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(&mut context)?;
                context.append(&mut output, &captured)?;
            }
            keyword => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, keyword.as_css())?;
            }
        }
        Ok(output)
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
