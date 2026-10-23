//! Exact authored CSS 2.1 line heights, before font metrics and inheritance resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssHintedNumberCalculation, CssSerializationError, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedNonNegativeNumber,
};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// An authored `line-height`. Numbers, lengths, and percentages remain distinct.
/// Equality compares authored structure while ignoring source origin.
///
/// # Specified serialization
///
/// Serializes the specified value without resolving font metrics.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssLineHeight {
    Normal,
    Number(CssSpecifiedNonNegativeNumber),
    /// A contextual Number before font or percentage basis and math range resolution.
    HintedNumberCalculation(CssHintedNumberCalculation),
    LengthPercentage(CssSpecifiedNonNegativeLengthPercentage),
}

impl PartialEq for CssLineHeight {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::Number(left), Self::Number(right)) => left.structural_eq(right),
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left.structural_eq(right)
            }
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}
impl Eq for CssLineHeight {}

impl CssLineHeight {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        match self {
            Self::Normal => {
                context.charge_nodes(1)?;
                context.append(output, "normal")?;
            }
            Self::Number(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
            }
            Self::HintedNumberCalculation(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
            }
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
            }
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssLineHeight {
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
crate::specified_rule_serialization::specified_serialization!(CssLineHeight);
