//! Canonical specified palette values.

use crate::{
    CssFontFamilyName, CssFontPaletteBase, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssFontPaletteValuesRule,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
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
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
                    self.append_family(family.as_str())?;
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

    fn append_family(&mut self, name: &str) -> Result<(), CssSpecifiedValueSerializationError> {
        if can_serialize_family_unquoted(name) {
            for (index, word) in name.split(' ').enumerate() {
                if index != 0 {
                    self.append(" ")?;
                }
                self.append_identifier(word)?;
            }
            Ok(())
        } else {
            self.append_string(name)
        }
    }
}

fn can_serialize_family_unquoted(name: &str) -> bool {
    if name.is_empty() || name.starts_with(' ') || name.ends_with(' ') || name.contains("  ") {
        return false;
    }
    if CssFontFamilyName::try_ident_sequence(name.split(' ').map(str::to_owned).collect()).is_none()
    {
        return false;
    }
    name.split(' ').all(|word| {
        let mut chars = word.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == '_' || first == '-' || first as u32 >= 0x80) {
            return false;
        }
        if first == '-' && word.len() == 1 {
            return false;
        }
        if first == '-' && chars.clone().next().is_some_and(|c| c.is_ascii_digit()) {
            return false;
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c as u32 >= 0x80)
    })
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
