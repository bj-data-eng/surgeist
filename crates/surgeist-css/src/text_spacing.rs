//! Exact authored word and letter spacing from CSS Text 4.

use crate::{
    CssSpecifiedLengthPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

/// A checked specified spacing adjustment, shared by word and letter spacing.
///
/// Percentage resolution uses the element's used font size and remains outside
/// this authored-value type.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTextSpacingAdjustment {
    /// The symbolic initial adjustment.
    Normal,
    /// A signed exact length-percentage, including deferred math.
    LengthPercentage(CssSpecifiedLengthPercentage),
}

impl CssTextSpacingAdjustment {
    /// Borrows the checked numeric adjustment, if present.
    pub fn length_percentage(&self) -> Option<&CssSpecifiedLengthPercentage> {
        match self {
            Self::Normal => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Borrows the original numeric origin; `normal` has no numeric origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssSpecifiedLengthPercentage::origin)
    }

    /// Serializes the canonical specified value before font-size resolution.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under input, projection, and output resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::Normal => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "normal")?;
            }
            Self::LengthPercentage(value) => {
                let captured = value.capture_specified(context)?;
                context.append(output, &captured)?;
            }
        }
        Ok(())
    }
}

impl PartialEq for CssTextSpacingAdjustment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => {
                left.structural_eq(right)
            }
            _ => false,
        }
    }
}

impl Eq for CssTextSpacingAdjustment {}
