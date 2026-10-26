//! Exact authored gap values before layout and percentage resolution.

use crate::common_values::{side_element, side_pair_entry};
use crate::specified_inverse::InverseValueEquality as _;
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssLengthPercentage, CssNonNegative, CssNumeric, CssSerializationError};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// One `row-gap` or `column-gap` value, retaining symbolic normal or exact numeric syntax.
///
/// # Specified serialization
///
/// Serializes this specified value without resolving layout context.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssGapValue {
    Normal,
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
}

impl CssGapValue {
    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => {
                context.charge_nodes(1)?;
                context.append(output, "normal")
            }
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssGapValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssGapValue);

impl PartialEq for CssGapValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for CssGapValue {}

// `gap` takes `<'row-gap'> <'column-gap'>?`.
side_element!(
    CssGapValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_pair_entry!(CssGapValue);
