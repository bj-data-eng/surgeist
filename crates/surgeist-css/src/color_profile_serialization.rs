//! Canonical authored profiles using the enclosing cumulative specified writer.
use crate::{
    CssColorProfileDescriptor, CssColorProfileDescriptorValue, CssColorProfileDescriptorValueRef,
    CssColorProfileRule, CssColorProfileRuleName, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, specified_rule_serialization::SpecifiedRuleWriter,
};

impl SpecifiedRuleWriter {
    pub(crate) fn color_profile(
        &mut self,
        rule: &CssColorProfileRule,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
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
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append(descriptor.value().kind().css_name())?;
        self.append(": ")?;
        self.profile_value(descriptor.value())
    }

    fn profile_value(
        &mut self,
        value: &CssColorProfileDescriptorValue,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match value.view() {
            CssColorProfileDescriptorValueRef::Src(url) => url.append_specified(self),
            CssColorProfileDescriptorValueRef::RenderingIntent(intent) => {
                self.context.charge_input(1)?;
                self.context.charge_projection(1)?;
                self.append(intent.css_name())
            }
            CssColorProfileDescriptorValueRef::Components(names) => {
                for (index, name) in names.iter().enumerate() {
                    if index != 0 {
                        self.append(", ")?;
                    }
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
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

impl CssColorProfileDescriptorValue {
    /// Emits ordinary checked grammar or symbolic pending tokens without substitution.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.profile_value(self)?;
        Ok(writer.css)
    }
}

impl CssColorProfileDescriptor {
    /// Emits one name and value, without the enclosing list's semicolon.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.profile_descriptor(self)?;
        Ok(writer.css)
    }
}

impl CssColorProfileRule {
    /// Emits every retained occurrence in authored order, including empty definitions.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.color_profile(self)?;
        Ok(writer.css)
    }
}
