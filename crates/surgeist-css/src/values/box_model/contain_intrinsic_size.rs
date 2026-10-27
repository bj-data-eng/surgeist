//! Checked authored contain-intrinsic sizes before layout and remembered-size resolution.

use crate::numeric::{CssLength, CssNonNegative, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::specified_inverse::InverseValueEquality as _;
use crate::values::common::common_values::{side_element, side_pair_entry};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// The mandatory fallback after the optional `auto` marker.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssContainIntrinsicSizeFallback {
    /// No explicit intrinsic inner size when no remembered size applies.
    None,
    /// An exact nonnegative length, with math deferred to its owning stage.
    Length(CssNumeric<CssLength, CssNonNegative>),
}

impl PartialEq for CssContainIntrinsicSizeFallback {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Length(left), Self::Length(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for CssContainIntrinsicSizeFallback {}

/// One contained intrinsic size axis: `auto? [ none | <length [0,∞]> ]`.
///
/// # Specified serialization
///
/// Serializes the specified value without resolving remembered sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainIntrinsicSizeValue {
    uses_auto: bool,
    fallback: CssContainIntrinsicSizeFallback,
}

impl CssContainIntrinsicSizeValue {
    /// Constructs a fallback without the remembered-size marker.
    #[must_use]
    pub fn new(fallback: CssContainIntrinsicSizeFallback) -> Self {
        Self {
            uses_auto: false,
            fallback,
        }
    }

    /// Constructs a value that prefers a remembered size when one applies.
    #[must_use]
    pub fn with_auto(fallback: CssContainIntrinsicSizeFallback) -> Self {
        Self {
            uses_auto: true,
            fallback,
        }
    }

    /// Reports whether the `auto` marker was authored.
    #[must_use]
    pub fn uses_auto(&self) -> bool {
        self.uses_auto
    }

    /// Borrows the mandatory `none` or nonnegative length fallback.
    pub fn fallback(&self) -> &CssContainIntrinsicSizeFallback {
        &self.fallback
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        if self.uses_auto {
            context.keyword(output, "auto")?;
            context.append(output, " ")?;
        }
        match &self.fallback {
            CssContainIntrinsicSizeFallback::None => context.keyword(output, "none"),
            CssContainIntrinsicSizeFallback::Length(length) => {
                let captured = length.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssContainIntrinsicSizeValue
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssContainIntrinsicSizeValue
);

// `contain-intrinsic-size` takes `[ auto? [ none | <length [0,∞]> ] ]{1,2}` and, per Sizing 4,
// one value applies to both axes: an equal second value is omitted from the specified
// serialization, while the model keeps whether it was authored.
side_element!(
    CssContainIntrinsicSizeValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_pair_entry!(CssContainIntrinsicSizeValue);
