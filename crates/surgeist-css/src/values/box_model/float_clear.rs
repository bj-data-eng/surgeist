//! Canonical authored float and clear keywords, before writing-mode mapping.

use crate::values::box_model::{CssClear, CssFloat};

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFloat {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssFloat);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssClear {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssClear);
