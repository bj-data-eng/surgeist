//! Bounded specified serialization for the deprecated CSS Masking `clip` value.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssClip, CssClipEdge, CssClipRect, CssSerializationError};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssClipEdge {
    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")
            }
            Self::Length(length) => length.append_specified(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssClipEdge {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssClipEdge);

impl CssClipRect {
    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        context.append(output, "rect(")?;
        for (index, edge) in [self.top(), self.right(), self.bottom(), self.left()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                context.append(output, ", ")?;
            }
            edge.serialize_specified_into(context, output)?;
        }
        context.append(output, ")")
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssClipRect {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
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
