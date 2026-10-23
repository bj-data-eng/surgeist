//! Bounded specified serialization for checked CSS2 table border spacing.

use crate::CssBorderSpacing;

impl crate::specified_rule_serialization::SpecifiedProvider for CssBorderSpacing {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        self.horizontal().append_specified(context, output)?;
        context.append(output, " ")?;
        self.vertical().append_specified(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBorderSpacing);
