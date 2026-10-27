//! Canonical authored profiles using the enclosing cumulative specified writer.
use crate::rule::color_profile::{
    CssColorProfileDescriptor, CssColorProfileDescriptorValue, CssColorProfileDescriptorValueRef,
    CssColorProfileRule, CssColorProfileRuleName,
};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;

impl SpecifiedRuleWriter {
    pub(crate) fn color_profile(
        &mut self,
        rule: &CssColorProfileRule,
    ) -> Result<(), CssSerializationError> {
        self.node()?;
        self.append("@color-profile ")?;
        match rule.name() {
            CssColorProfileRuleName::Custom(name) => self.append_identifier(name.as_str())?,
            CssColorProfileRuleName::DeviceCmyk => self.append("device-cmyk")?,
        }
        self.append(" {")?;
        for descriptor in rule.descriptors() {
            self.append(" ")?;
            self.profile_descriptor(descriptor)?;
            self.append(";")?;
        }
        self.append(" }")
    }

    fn profile_descriptor(
        &mut self,
        descriptor: &CssColorProfileDescriptor,
    ) -> Result<(), CssSerializationError> {
        self.node()?;
        self.append(descriptor.value().kind().as_str())?;
        self.append(": ")?;
        self.profile_value(descriptor.value())
    }

    fn profile_value(
        &mut self,
        value: &CssColorProfileDescriptorValue,
    ) -> Result<(), CssSerializationError> {
        match value.view() {
            CssColorProfileDescriptorValueRef::Src(url) => url.append_specified(self),
            CssColorProfileDescriptorValueRef::RenderingIntent(intent) => {
                self.node()?;
                self.append(intent.as_str())
            }
            CssColorProfileDescriptorValueRef::Components(names) => {
                for (index, name) in names.iter().enumerate() {
                    if index != 0 {
                        self.append(", ")?;
                    }
                    self.node()?;
                    self.append_identifier(name.as_str())?;
                }
                Ok(())
            }
            CssColorProfileDescriptorValueRef::Pending(_) => {
                crate::serialization::pending_serialization::append_pending_specified(
                    value.components(),
                    &mut self.context,
                    &mut self.css,
                )
            }
        }
    }
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssColorProfileDescriptorValue
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.profile_value(self)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssColorProfileDescriptorValue
);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssColorProfileDescriptor
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.profile_descriptor(self)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssColorProfileDescriptor
);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssColorProfileRule {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.color_profile(self)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssColorProfileRule);
