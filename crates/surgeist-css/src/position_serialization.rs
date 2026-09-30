//! Bounded specified serialization for Values 4 positions and Backgrounds 3 layers.

use crate::{
    CssBackgroundPosition, CssBackgroundPositionList, CssHorizontalPosition, CssPosition,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssVerticalPosition,
    specified_rule_serialization::SpecifiedRuleWriter,
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
        append_axes(self.horizontal(), self.vertical(), writer)
    }
}

impl CssBackgroundPosition {
    /// Serializes the checked background position in horizontal-then-vertical order.
    /// In a three-component position, the absent edge offset remains omitted.
    /// Percentages and calculations retain their symbolic specified meaning.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both axes with one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_axes(self.horizontal(), self.vertical(), writer)
    }
}

impl CssBackgroundPositionList {
    /// Serializes the nonempty authored layers in comma order.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes every layer and numeric child with one cumulative resource budget.
    /// Failure returns no partial CSS and leaves the authored list unchanged.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        for (index, position) in self.positions().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            position.append_specified(&mut writer)?;
        }
        Ok(writer.css)
    }
}

fn append_axes(
    horizontal: &CssHorizontalPosition,
    vertical: &CssVerticalPosition,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    append_horizontal(horizontal, writer)?;
    writer.append(" ")?;
    append_vertical(vertical, writer)
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
    offset: &crate::CssSpecifiedLengthPercentage,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    offset.append_specified(&mut writer.context, &mut writer.css)
}
