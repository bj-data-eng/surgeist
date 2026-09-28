//! Bounded specified serialization for Values 4 generic `<position>`.

use crate::{
    CssHorizontalPosition, CssPosition, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssVerticalPosition,
    specified_rule_serialization::SpecifiedRuleWriter,
    specified_serialization::serialize_checked_length_percentage_into,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssPosition {
    /// Serializes the checked generic position in horizontal-then-vertical order.
    /// Percentages and calculations retain their symbolic specified meaning.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the position with one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    /// Appends to an owning image or property writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        append_horizontal(self.horizontal(), writer)?;
        writer.append(" ")?;
        append_vertical(self.vertical(), writer)
    }
}

fn append_horizontal(
    value: &CssHorizontalPosition,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    match value {
        CssHorizontalPosition::Left => writer.append("left"),
        CssHorizontalPosition::Center => writer.append("center"),
        CssHorizontalPosition::Right => writer.append("right"),
        CssHorizontalPosition::Offset(offset) => append_offset(offset, writer),
        CssHorizontalPosition::LeftOffset(offset) => {
            writer.append("left ")?;
            append_offset(offset, writer)
        }
        CssHorizontalPosition::RightOffset(offset) => {
            writer.append("right ")?;
            append_offset(offset, writer)
        }
    }
}

fn append_vertical(value: &CssVerticalPosition, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    match value {
        CssVerticalPosition::Top => writer.append("top"),
        CssVerticalPosition::Center => writer.append("center"),
        CssVerticalPosition::Bottom => writer.append("bottom"),
        CssVerticalPosition::Offset(offset) => append_offset(offset, writer),
        CssVerticalPosition::TopOffset(offset) => {
            writer.append("top ")?;
            append_offset(offset, writer)
        }
        CssVerticalPosition::BottomOffset(offset) => {
            writer.append("bottom ")?;
            append_offset(offset, writer)
        }
    }
}

fn append_offset(
    offset: &crate::CssPositionOffset,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    serialize_checked_length_percentage_into(offset.value(), &mut writer.context, &mut writer.css)
}
