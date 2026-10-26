//! Exact authored positioned offsets before writing-mode and layout resolution.

use crate::common_values::{four_sides_entry, side_element, side_pair_entry};
use crate::specified_inverse::InverseValueEquality as _;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssLengthPercentage, CssNumeric, CssSerializationError, CssValueOrigin};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// One physical or logical inset longhand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssInsetValue {
    /// The unresolved automatic offset.
    Auto,
    /// An exact signed length-percentage, including deferred math.
    LengthPercentage(CssNumeric<CssLengthPercentage>),
}

impl CssInsetValue {
    /// Borrows the checked numeric value, if present.
    pub fn length_percentage(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        match self {
            Self::Auto => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Borrows a retained numeric origin; `auto` has no numeric origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssNumeric::<CssLengthPercentage>::origin)
    }

    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")
            }
            Self::LengthPercentage(value) => {
                let text = value.capture_specified(context)?;
                context.append(output, &text)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssInsetValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssInsetValue);

impl PartialEq for CssInsetValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}
impl Eq for CssInsetValue {}

side_element!(
    CssInsetValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
four_sides_entry!(CssInsetValue);
side_pair_entry!(CssInsetValue);

impl crate::specified_rule_serialization::SpecifiedProvider for crate::CssLayoutPosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let text = match self {
            Self::Static => "static",
            Self::Relative => "relative",
            Self::Absolute => "absolute",
            Self::Fixed => "fixed",
            Self::Sticky => "sticky",
        };
        writer.keyword(text)
    }
}
crate::specified_rule_serialization::specified_serialization!(crate::CssLayoutPosition);
