//! Bounded specified serialization for checked Images 3 values.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssAngleOrZero, CssColorStopList, CssColorStopListItem, CssGradient, CssHorizontalGradientSide,
    CssHorizontalPosition, CssImage, CssImageValue, CssImageValueList, CssLinearGradient,
    CssLinearGradientDirection, CssPhysicalPosition, CssRadialExtent, CssRadialGradient,
    CssRadialShape, CssRadialSize, CssSpecifiedLengthPercentage,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    CssVerticalGradientSide, CssVerticalPosition,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter, count: usize) -> Result<()> {
    writer.context.charge_input(count)?;
    writer.context.charge_projection(count)
}

impl CssImage {
    /// Serializes the checked image without resolving its resource or geometry.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    /// Appends to an enclosing specified writer without resetting its budget.
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        self.value().append_specified(writer)
    }
}

impl CssImageValue {
    /// Serializes an authored image value, including the property-level `none` branch.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                charge(writer, 1)?;
                writer.append("none")
            }
            Self::Url(url) => url.append_specified(writer),
            Self::Gradient(gradient) => gradient.append_specified(writer),
        }
    }
}

impl CssImageValueList {
    /// Serializes every image in its authored order.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the entire list under one cumulative resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        charge(&mut writer, 1)?;
        for (index, image) in self.images().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            image.append_specified(&mut writer)?;
        }
        Ok(writer.css)
    }
}

impl CssGradient {
    /// Serializes the checked gradient with its authored function identity.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes under one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_specified(&mut writer)?;
        Ok(writer.css)
    }

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge(writer, 1)?;
        match self {
            Self::Linear(value) => {
                writer.append("linear-gradient(")?;
                append_linear(value, writer)?;
            }
            Self::RepeatingLinear(value) => {
                writer.append("repeating-linear-gradient(")?;
                append_linear(value, writer)?;
            }
            Self::Radial(value) => {
                writer.append("radial-gradient(")?;
                append_radial(value, writer)?;
            }
            Self::RepeatingRadial(value) => {
                writer.append("repeating-radial-gradient(")?;
                append_radial(value, writer)?;
            }
        }
        writer.append(")")
    }
}

fn append_linear(value: &CssLinearGradient, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    if let Some(direction) = value.direction() {
        let emitted = append_direction(direction, writer)?;
        if emitted {
            writer.append(", ")?;
        }
    }
    append_stops(value.stops(), writer)
}

fn append_direction(
    direction: &CssLinearGradientDirection,
    writer: &mut SpecifiedRuleWriter,
) -> Result<bool> {
    charge(writer, 1)?;
    match direction {
        CssLinearGradientDirection::Angle(angle) => {
            if let CssAngleOrZero::Angle(value) = angle
                && let Some(literal) = value.literal()
            {
                let omitted = literal.is_default_gradient_direction();
                if omitted {
                    writer.without_output(|writer| {
                        angle.append_specified(&mut writer.context, &mut writer.css)
                    })?;
                } else {
                    angle.append_specified(&mut writer.context, &mut writer.css)?;
                }
                Ok(!omitted)
            } else {
                angle.append_specified(&mut writer.context, &mut writer.css)?;
                Ok(true)
            }
        }
        CssLinearGradientDirection::SideOrCorner(side) => {
            let omitted = side.horizontal().is_none()
                && side.vertical() == Some(CssVerticalGradientSide::Bottom);
            if !omitted {
                writer.append("to ")?;
            }
            if let Some(horizontal) = side.horizontal() {
                charge(writer, 1)?;
                if !omitted {
                    writer.append(match horizontal {
                        CssHorizontalGradientSide::Left => "left",
                        CssHorizontalGradientSide::Right => "right",
                    })?;
                }
            }
            if let Some(vertical) = side.vertical() {
                charge(writer, 1)?;
                if !omitted {
                    if side.horizontal().is_some() {
                        writer.append(" ")?;
                    }
                    writer.append(match vertical {
                        CssVerticalGradientSide::Top => "top",
                        CssVerticalGradientSide::Bottom => "bottom",
                    })?;
                }
            }
            Ok(!omitted)
        }
    }
}

fn append_radial(value: &CssRadialGradient, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    let mut emitted = false;
    if let Some(shape) = value.shape() {
        charge(writer, 1)?;
        let omit = matches!(shape, CssRadialShape::Ellipse)
            || (matches!(shape, CssRadialShape::Circle)
                && matches!(value.size(), Some(CssRadialSize::Circle(_))));
        if !omit {
            writer.append(match shape {
                CssRadialShape::Circle => "circle",
                CssRadialShape::Ellipse => "ellipse",
            })?;
            emitted = true;
        }
    }
    if let Some(size) = value.size() {
        charge(writer, 1)?;
        let omit = matches!(size, CssRadialSize::Extent(CssRadialExtent::FarthestCorner));
        if !omit {
            if emitted {
                writer.append(" ")?;
            }
            match size {
                CssRadialSize::Extent(extent) => {
                    charge(writer, 1)?;
                    writer.append(radial_extent(*extent))?;
                }
                CssRadialSize::Circle(circle) => {
                    circle.append_specified(&mut writer.context, &mut writer.css)?;
                }
                CssRadialSize::Ellipse(ellipse) => {
                    ellipse
                        .horizontal()
                        .append_specified(&mut writer.context, &mut writer.css)?;
                    writer.append(" ")?;
                    ellipse
                        .vertical()
                        .append_specified(&mut writer.context, &mut writer.css)?;
                }
            }
            emitted = true;
        } else {
            charge(writer, 1)?;
        }
    }
    if let Some(position) = value.position() {
        if radial_position_is_center(position) {
            // The checked position writer charges one aggregate and each axis,
            // plus one numeric leaf for each authored 50% offset.
            charge(writer, 3 + radial_center_offset_count(position))?;
        } else {
            if emitted {
                writer.append(" ")?;
            }
            writer.append("at ")?;
            position.append_specified(writer)?;
            emitted = true;
        }
    }
    if emitted {
        writer.append(", ")?;
    }
    append_stops(value.stops(), writer)
}

fn radial_extent(extent: CssRadialExtent) -> &'static str {
    match extent {
        CssRadialExtent::ClosestSide => "closest-side",
        CssRadialExtent::FarthestSide => "farthest-side",
        CssRadialExtent::ClosestCorner => "closest-corner",
        CssRadialExtent::FarthestCorner => "farthest-corner",
    }
}

fn radial_center_offset_count(position: &CssPhysicalPosition) -> usize {
    usize::from(matches!(
        position.horizontal(),
        CssHorizontalPosition::Offset(_)
    )) + usize::from(matches!(
        position.vertical(),
        CssVerticalPosition::Offset(_)
    ))
}

fn radial_position_is_center(position: &CssPhysicalPosition) -> bool {
    let horizontal = match position.horizontal() {
        CssHorizontalPosition::Center => true,
        CssHorizontalPosition::Offset(offset) => literal_percentage_is(offset, 5, 1),
        _ => false,
    };
    let vertical = match position.vertical() {
        CssVerticalPosition::Center => true,
        CssVerticalPosition::Offset(offset) => literal_percentage_is(offset, 5, 1),
        _ => false,
    };
    horizontal && vertical
}

fn append_stops(stops: &CssColorStopList, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(writer, 1)?;
    let last = stops.items().len() - 1;
    for (index, item) in stops.items().iter().enumerate() {
        if index != 0 {
            writer.append(", ")?;
        }
        charge(writer, 1)?;
        match item {
            CssColorStopListItem::Stop(stop) => {
                stop.color()
                    .append_specified(&mut writer.context, &mut writer.css)?;
                if let Some(position) = stop.position() {
                    let omit = (index == 0 && is_direct_zero(position))
                        || (index == last && is_direct_hundred_percent(position));
                    if omit {
                        charge(writer, 1)?;
                    } else {
                        writer.append(" ")?;
                        append_line_position(position, writer)?;
                    }
                }
            }
            CssColorStopListItem::Hint(position) => append_line_position(position, writer)?,
        }
    }
    Ok(())
}

fn is_direct_zero(value: &CssSpecifiedLengthPercentage) -> bool {
    value
        .literal_component()
        .is_some_and(|component| match component.view() {
            crate::CssComponentValueRef::Token(
                crate::CssValueTokenRef::Number(number)
                | crate::CssValueTokenRef::Percentage(number)
                | crate::CssValueTokenRef::Dimension { number, .. },
            ) => crate::exact_decimal::LexicalDecimal::new(number.representation()).len == 0,
            _ => false,
        })
}

fn literal_percentage_is(value: &CssSpecifiedLengthPercentage, digit: u8, exponent: i128) -> bool {
    value.literal_component().is_some_and(|component| {
        let crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number)) =
            component.view()
        else {
            return false;
        };
        let decimal = crate::exact_decimal::LexicalDecimal::new(number.representation());
        !decimal.negative
            && decimal.len == 1
            && decimal.exponent == Some(exponent)
            && decimal.digits().next() == Some(digit)
    })
}
fn is_direct_hundred_percent(value: &CssSpecifiedLengthPercentage) -> bool {
    literal_percentage_is(value, 1, 2)
}
fn append_line_position(
    value: &CssSpecifiedLengthPercentage,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    value.append_specified(&mut writer.context, &mut writer.css)
}
