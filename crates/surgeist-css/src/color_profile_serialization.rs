//! Canonical authored profiles using the enclosing cumulative specified writer.
use crate::{
    CssColorProfileDescriptor, CssColorProfileDescriptorValue, CssColorProfileDescriptorValueRef,
    CssColorProfileRule, CssColorProfileRuleName, CssSerializationError,
    specified_rule_serialization::SpecifiedRuleWriter,
};

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
        self.append(descriptor.value().kind().css_name())?;
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
                self.append(intent.css_name())
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
                crate::pending_serialization::append_pending_specified(
                    value.components(),
                    &mut self.context,
                    &mut self.css,
                )
            }
        }
    }
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssColorProfileDescriptorValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.profile_value(self)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColorProfileDescriptorValue);

impl crate::specified_rule_serialization::SpecifiedProvider for CssColorProfileDescriptor {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.profile_descriptor(self)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColorProfileDescriptor);

impl crate::specified_rule_serialization::SpecifiedProvider for CssColorProfileRule {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.color_profile(self)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssColorProfileRule);
