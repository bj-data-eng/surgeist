//! Authored preferred aspect ratios and exact ratio operands.
//! Ordinary operands emit at most six fractional places, nearest with ties away
//! from zero; exact identity and range checks remain unchanged. Finite calculated
//! operands use the same text policy while retaining existing arithmetic and ranges.

use crate::{
    CssNonNegative, CssNumber, CssNumeric, CssSerializationError,
    specified_serialization::SpecifiedSerializationContext,
};

/// An authored ratio with an optional denominator; omission means one.
///
/// # Specified serialization
///
/// Serializes the specified pair; an omitted denominator is emitted as one.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSpecifiedRatio {
    numerator: CssNumeric<CssNumber, CssNonNegative>,
    denominator: Option<CssNumeric<CssNumber, CssNonNegative>>,
}

impl CssSpecifiedRatio {
    /// Composes already-checked operands without changing their provenance.
    pub const fn new(
        numerator: CssNumeric<CssNumber, CssNonNegative>,
        denominator: Option<CssNumeric<CssNumber, CssNonNegative>>,
    ) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub const fn numerator(&self) -> &CssNumeric<CssNumber, CssNonNegative> {
        &self.numerator
    }

    pub const fn denominator(&self) -> Option<&CssNumeric<CssNumber, CssNonNegative>> {
        self.denominator.as_ref()
    }

    fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        self.numerator.append_specified(context, output)?;
        context.append(output, " / ")?;
        if let Some(denominator) = &self.denominator {
            denominator.append_specified(context, output)
        } else {
            context.charge_nodes(1)?;
            context.append(output, "1")
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssSpecifiedRatio {
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
crate::specified_rule_serialization::specified_serialization!(CssSpecifiedRatio);

impl crate::specified_rule_serialization::SpecifiedProvider for crate::CssAspectRatioValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")?;
            }
            Self::Ratio(ratio) => ratio.append_specified(context, output)?,
            Self::AutoRatio(ratio) => {
                context.charge_nodes(1)?;
                context.append(output, "auto ")?;
                ratio.append_specified(context, output)?;
            }
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(crate::CssAspectRatioValue);
