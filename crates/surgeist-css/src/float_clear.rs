//! Canonical authored float and clear keywords, before writing-mode mapping.

use crate::{
    CssClear, CssFloat, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

impl CssFloat {
    /// Serializes the specified float keyword without resolving the containing block's writing mode.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical keyword under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::None => "none",
            Self::Left => "left",
            Self::Right => "right",
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
        };
        writer.keyword(text)
    }
}

impl CssClear {
    /// Serializes the specified clear keyword without resolving the containing block's writing mode.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the canonical keyword under explicit resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::None => "none",
            Self::Left => "left",
            Self::Right => "right",
            Self::Both => "both",
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
        };
        writer.keyword(text)
    }
}
