//! Bounded specified serialization for checked CSS2 table border spacing.

use crate::{
    CssBorderSpacing, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

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
        context.charge_input(1)?;
        context.charge_projection(1)?;
        self.horizontal().append_specified(context, output)?;
        context.append(output, " ")?;
        self.vertical().append_specified(context, output)?;
        Ok(())
    }
}
