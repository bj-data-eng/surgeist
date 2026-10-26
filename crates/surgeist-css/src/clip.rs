//! Bounded specified serialization for the deprecated CSS Masking `clip` value.

use crate::{CssClip, CssClipRect};

impl crate::specified_rule_serialization::SpecifiedProvider for CssClipRect {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?;
        writer.append("rect(")?;
        for (index, edge) in [self.top(), self.right(), self.bottom(), self.left()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                writer.append(", ")?;
            }
            crate::specified_rule_serialization::SpecifiedProvider::append_to_rule_writer(
                edge, writer,
            )?;
        }
        writer.append(")")
    }
}
crate::specified_rule_serialization::specified_serialization!(CssClipRect);

impl crate::specified_rule_serialization::SpecifiedProvider for CssClip {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Rect(rect) => rect.append_to_rule_writer(writer),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssClip);
