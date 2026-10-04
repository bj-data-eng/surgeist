//! Exact authored CSS 2.1 line heights, before font metrics and inheritance resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssHintedNumberCalculation, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedNonNegativeNumber, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// An authored `line-height`. Numbers, lengths, and percentages remain distinct.
/// Equality compares authored structure while ignoring source origin.
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
    /// Serializes the specified value without resolving font metrics.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under explicit resource limits.
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
        match self {
            Self::Normal => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
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
