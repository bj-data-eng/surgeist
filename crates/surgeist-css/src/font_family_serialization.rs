//! Canonical specified family names, retaining the authored property form.
//! Descriptor families own a decoded literal name rather than an authored form.

use crate::{
    CssFontFaceFamily, CssFontFamilyList, CssFontFamilyName, CssFontFamilyNameKind,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssFontFamilyName {
    /// Emits the retained quoted, identifier-sequence, or generic form.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits atomically, charging one name node and each retained identifier token.
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
        self.append_specified(writer)?;
        Ok(())
    }

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        match self.kind() {
            CssFontFamilyNameKind::Quoted => writer.append_string(self.as_str()),
            CssFontFamilyNameKind::Generic => writer.append(self.as_str()),
            CssFontFamilyNameKind::IdentSequence => {
                for (index, identifier) in self
                    .identifier_tokens()
                    .expect("identifier sequence retains tokens")
                    .iter()
                    .enumerate()
                {
                    if index != 0 {
                        writer.append(" ")?;
                    }
                    writer.context.charge_input(1)?;
                    writer.context.charge_projection(1)?;
                    writer.append_identifier(identifier)?;
                }
                Ok(())
            }
        }
    }
}

impl CssFontFamilyList {
    /// Emits the nonempty family list in retained order and authored forms.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative budget for names, identifier tokens, and emitted bytes.
    /// The list container adds no node charge, matching the embedded font writer.
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
        self.append_specified(writer)?;
        Ok(())
    }

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        for (index, family) in self.families().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            family.append_specified(writer)?;
        }
        Ok(())
    }
}

impl CssFontFaceFamily {
    /// Emits a decoded literal family name, quoting names that cannot be bare
    /// identifier sequences. This model has no generic or authored-quoting branch.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Emits atomically, charging one literal node even for a multiword name.
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
        self.append_specified(writer)?;
        Ok(())
    }

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        let name = self.as_str();
        if can_serialize_family_unquoted(name) {
            for (index, word) in name.split(' ').enumerate() {
                if index != 0 {
                    writer.append(" ")?;
                }
                writer.append_identifier(word)?;
            }
            Ok(())
        } else {
            writer.append_string(name)
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
