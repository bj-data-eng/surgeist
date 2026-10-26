//! Exact authored word and letter spacing from CSS Text 4.

use crate::{CssLengthPercentage, CssNumeric, CssValueOrigin};

/// A checked specified spacing adjustment, shared by word and letter spacing.
///
/// Percentage resolution uses the element's used font size and remains outside
/// this authored-value type.
///
/// # Specified serialization
///
/// Serializes the canonical specified value before font-size resolution.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssTextSpacingAdjustment {
    /// The symbolic initial adjustment.
    Normal,
    /// A signed exact length-percentage, including deferred math.
    LengthPercentage(CssNumeric<CssLengthPercentage>),
}

impl CssTextSpacingAdjustment {
    /// Borrows the checked numeric adjustment, if present.
    pub fn length_percentage(&self) -> Option<&CssNumeric<CssLengthPercentage>> {
        match self {
            Self::Normal => None,
            Self::LengthPercentage(value) => Some(value),
        }
    }

    /// Borrows the original numeric origin; `normal` has no numeric origin.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length_percentage()
            .map(CssNumeric::<CssLengthPercentage>::origin)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssTextSpacingAdjustment {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::Normal => {
                context.charge_nodes(1)?;
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
crate::specified_rule_serialization::specified_serialization!(CssTextSpacingAdjustment);

impl PartialEq for CssTextSpacingAdjustment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Normal, Self::Normal) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for CssTextSpacingAdjustment {}
