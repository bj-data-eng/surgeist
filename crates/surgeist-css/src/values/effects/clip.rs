//! Bounded specified serialization for the deprecated CSS Masking `clip` value.

use crate::values::effects::{CssClip, CssClipRect};

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssClipRect {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.node()?;
        writer.append("rect(")?;
        for (index, edge) in [self.top(), self.right(), self.bottom(), self.left()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                writer.append(", ")?;
            }
            crate::serialization::specified_rule_serialization::SpecifiedProvider::append_to_rule_writer(
                edge, writer,
            )?;
        }
        writer.append(")")
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssClipRect);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssClip {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Rect(rect) => rect.append_to_rule_writer(writer),
        }
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssClip);
