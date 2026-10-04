//! Canonical specified palette values.

use crate::{
    CssFontPaletteBase, CssFontPaletteDescriptorValue, CssFontPaletteDescriptorValueRef,
    CssFontPaletteValuesRule, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, specified_rule_serialization::SpecifiedRuleWriter,
};

impl SpecifiedRuleWriter {
    pub(crate) fn palette(
        &mut self,
        rule: &CssFontPaletteValuesRule,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append("@font-palette-values ")?;
        self.append_identifier(rule.name().as_str())?;
        self.append(" {")?;
        for descriptor in rule.descriptors() {
            self.context.charge_input(1)?;
            self.context.charge_projection(1)?;
            self.append(" ")?;
            self.append(descriptor.value().kind().css_name())?;
            self.append(": ")?;
            self.descriptor_value(descriptor.value())?;
            self.append(";")?;
        }
        self.append(" }")
    }

    fn descriptor_value(
        &mut self,
        value: &CssFontPaletteDescriptorValue,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match value.view() {
            CssFontPaletteDescriptorValueRef::FontFamily(families) => {
                for (index, family) in families.iter().enumerate() {
                    if index != 0 {
                        self.append(", ")?;
                    }
                    family.append_specified(self)?;
                }
                Ok(())
            }
            CssFontPaletteDescriptorValueRef::BasePalette(base) => match base {
                CssFontPaletteBase::Light => self.append_keyword("light"),
                CssFontPaletteBase::Dark => self.append_keyword("dark"),
                CssFontPaletteBase::Index(index) => index
                    .value()
                    .append_specified(&mut self.context, &mut self.css),
            },
            CssFontPaletteDescriptorValueRef::OverrideColors(overrides) => {
                for (index, pair) in overrides.iter().enumerate() {
                    if index != 0 {
                        self.append(", ")?;
                    }
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
                    pair.index()
                        .value()
                        .append_specified(&mut self.context, &mut self.css)?;
                    self.append(" ")?;
                    pair.color()
                        .append_specified(&mut self.context, &mut self.css)?;
                }
                Ok(())
            }
            CssFontPaletteDescriptorValueRef::Pending(_) => {
                crate::pending_serialization::append_pending_specified(
                    value.components(),
                    &mut self.context,
                    &mut self.css,
                )
            }
        }
    }

    fn append_keyword(&mut self, keyword: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append(keyword)
    }
}

impl CssFontPaletteDescriptorValue {
    /// Canonical specified descriptor text; pending values retain their token stream.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.descriptor_value(self)?;
        Ok(writer.css)
    }
}

impl CssFontPaletteValuesRule {
    /// Canonical specified rule text. Descriptor duplicates retain authored order.
    pub fn to_specified_css(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.palette(self)?;
        Ok(writer.css)
    }
}
