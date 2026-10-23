//! Cumulative specified serialization for symbolic coordinate families and physical consumers.

use crate::{
    CssBackgroundPosition, CssBackgroundPositionList, CssBlockPosition, CssHorizontalPosition,
    CssInlinePosition, CssPhysicalPosition, CssPhysicalPositionList, CssPosition, CssPositionRef,
    CssRelativeAxisPosition, CssSerializationError, CssVerticalPosition,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssPosition {
    /// Appends to an owning image or property writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.view() {
            CssPositionRef::Cartesian(value) => {
                append_axes(value.horizontal(), value.vertical(), writer)
            }
            CssPositionRef::NamedFlow(value) => {
                writer.node()?;
                append_block(value.block(), writer)?;
                writer.append(" ")?;
                append_inline(value.inline(), writer)
            }
            CssPositionRef::RelativeFlow(value) => {
                writer.node()?;
                append_relative(value.block(), writer)?;
                writer.append(" ")?;
                append_relative(value.inline(), writer)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssPosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPosition);

impl CssPhysicalPosition {
    /// Appends to an owning image or property writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_axes(self.horizontal(), self.vertical(), writer)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssPhysicalPosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPhysicalPosition);

impl CssBackgroundPosition {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_axes(self.horizontal(), self.vertical(), writer)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBackgroundPosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBackgroundPosition);

impl crate::specified_rule_serialization::SpecifiedProvider for CssPhysicalPositionList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?;
        writer.comma_separated(self.positions(), |writer, position| {
            position.append_specified(writer)
        })?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssPhysicalPositionList);

impl crate::specified_rule_serialization::SpecifiedProvider for CssBackgroundPositionList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?;
        writer.comma_separated(self.positions(), |writer, position| {
            position.append_specified(writer)
        })?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBackgroundPositionList);

fn append_axes(
    horizontal: &CssHorizontalPosition,
    vertical: &CssVerticalPosition,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.node()?;
    append_horizontal(horizontal, writer)?;
    writer.append(" ")?;
    append_vertical(vertical, writer)
}

fn append_horizontal(
    value: &CssHorizontalPosition,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.node()?;
    match value {
        CssHorizontalPosition::Left => writer.append("left"),
        CssHorizontalPosition::Center => writer.append("center"),
        CssHorizontalPosition::Right => writer.append("right"),
        CssHorizontalPosition::XStart => writer.append("x-start"),
        CssHorizontalPosition::XEnd => writer.append("x-end"),
        CssHorizontalPosition::Offset(offset) => append_offset(offset, writer),
        CssHorizontalPosition::LeftOffset(offset) => {
            writer.append("left ")?;
            append_offset(offset, writer)
        }
        CssHorizontalPosition::RightOffset(offset) => {
            writer.append("right ")?;
            append_offset(offset, writer)
        }
        CssHorizontalPosition::XStartOffset(offset) => {
            writer.append("x-start ")?;
            append_offset(offset, writer)
        }
        CssHorizontalPosition::XEndOffset(offset) => {
            writer.append("x-end ")?;
            append_offset(offset, writer)
        }
    }
}

fn append_vertical(value: &CssVerticalPosition, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.node()?;
    match value {
        CssVerticalPosition::Top => writer.append("top"),
        CssVerticalPosition::Center => writer.append("center"),
        CssVerticalPosition::Bottom => writer.append("bottom"),
        CssVerticalPosition::YStart => writer.append("y-start"),
        CssVerticalPosition::YEnd => writer.append("y-end"),
        CssVerticalPosition::Offset(offset) => append_offset(offset, writer),
        CssVerticalPosition::TopOffset(offset) => {
            writer.append("top ")?;
            append_offset(offset, writer)
        }
        CssVerticalPosition::BottomOffset(offset) => {
            writer.append("bottom ")?;
            append_offset(offset, writer)
        }
        CssVerticalPosition::YStartOffset(offset) => {
            writer.append("y-start ")?;
            append_offset(offset, writer)
        }
        CssVerticalPosition::YEndOffset(offset) => {
            writer.append("y-end ")?;
            append_offset(offset, writer)
        }
    }
}

fn append_offset(
    offset: &crate::CssSpecifiedLengthPercentage,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    offset.append_specified(&mut writer.context, &mut writer.css)
}

fn append_block(value: &CssBlockPosition, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.node()?;
    match value {
        CssBlockPosition::Center => writer.append("center"),
        CssBlockPosition::Start => writer.append("block-start"),
        CssBlockPosition::End => writer.append("block-end"),
        CssBlockPosition::StartOffset(offset) => {
            writer.append("block-start ")?;
            append_offset(offset, writer)
        }
        CssBlockPosition::EndOffset(offset) => {
            writer.append("block-end ")?;
            append_offset(offset, writer)
        }
    }
}

fn append_inline(value: &CssInlinePosition, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.node()?;
    match value {
        CssInlinePosition::Center => writer.append("center"),
        CssInlinePosition::Start => writer.append("inline-start"),
        CssInlinePosition::End => writer.append("inline-end"),
        CssInlinePosition::StartOffset(offset) => {
            writer.append("inline-start ")?;
            append_offset(offset, writer)
        }
        CssInlinePosition::EndOffset(offset) => {
            writer.append("inline-end ")?;
            append_offset(offset, writer)
        }
    }
}

fn append_relative(
    value: &CssRelativeAxisPosition,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.node()?;
    match value {
        CssRelativeAxisPosition::Center => writer.append("center"),
        CssRelativeAxisPosition::Start => writer.append("start"),
        CssRelativeAxisPosition::End => writer.append("end"),
        CssRelativeAxisPosition::StartOffset(offset) => {
            writer.append("start ")?;
            append_offset(offset, writer)
        }
        CssRelativeAxisPosition::EndOffset(offset) => {
            writer.append("end ")?;
            append_offset(offset, writer)
        }
    }
}
