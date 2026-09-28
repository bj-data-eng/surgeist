//! Bounded specified serialization for checked CSS2 table border spacing.

use crate::specified_serialization::{
    SpecifiedSerializationContext, serialize_checked_pure_length_into,
};
use crate::{
    CssBorderSpacing, CssBorderSpacingLength, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssBorderSpacingLength {
    /// Serializes one checked specified length without resolving font or layout context.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one length under cumulative numeric and output limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        serialize_checked_pure_length_into(self.value(), context, output)
    }
}

impl CssBorderSpacing {
    /// Serializes the effective horizontal and vertical lengths, including a repeated axis.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both axes with one shared input, projection, and CSS byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        self.horizontal()
            .serialize_specified_into(&mut context, &mut output)?;
        context.append(&mut output, " ")?;
        self.vertical()
            .serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }
}
