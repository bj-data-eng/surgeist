//! Bounded specified serialization for checked Images 3 values.

use crate::numeric::{SpecifiedCalculationRef, project_calculation_specified_into};
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::specified_serialization::serialize_checked_length_percentage_into;
use crate::{
    CssAngleUnit, CssColorStopList, CssColorStopListItem, CssGradient, CssGradientAngle,
    CssHorizontalGradientSide, CssHorizontalPosition, CssImage, CssImageValue, CssImageValueList,
    CssLength, CssLinearGradient, CssLinearGradientDirection, CssPosition, CssRadialExtent,
    CssRadialGradient, CssRadialShape, CssRadialSize, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssVerticalGradientSide, CssVerticalPosition,
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
            if let CssGradientAngle::Literal(literal) = angle {
                charge(writer, 1)?;
                let omitted = matches!(
                    (literal.unit(), literal.value()),
                    (CssAngleUnit::Degrees, 180.0)
                        | (CssAngleUnit::Gradians, 200.0)
                        | (CssAngleUnit::Turns, 0.5)
                );
                if !omitted {
                    writer.append(&crate::syntax::format_css_number(literal.value()))?;
                    writer.append(angle_unit(literal.unit()))?;
                }
                Ok(!omitted)
            } else {
                match angle {
                    CssGradientAngle::Zero => {
                        charge(writer, 1)?;
                        writer.append("0")?;
                    }
                    CssGradientAngle::Calculation(calculation) => {
                        project_calculation_specified_into(
                            SpecifiedCalculationRef::Angle(calculation),
                            &mut writer.context,
                            &mut writer.css,
                        )?;
                    }
                    CssGradientAngle::Literal(_) => unreachable!(),
                }
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

fn angle_unit(unit: CssAngleUnit) -> &'static str {
    match unit {
        CssAngleUnit::Degrees => "deg",
        CssAngleUnit::Gradians => "grad",
        CssAngleUnit::Radians => "rad",
        CssAngleUnit::Turns => "turn",
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
                    append_line_position(circle.radius(), writer)?;
                }
                CssRadialSize::Ellipse(ellipse) => {
                    append_line_position(ellipse.horizontal(), writer)?;
                    writer.append(" ")?;
                    append_line_position(ellipse.vertical(), writer)?;
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

fn radial_center_offset_count(position: &CssPosition) -> usize {
    usize::from(matches!(
        position.horizontal(),
        CssHorizontalPosition::Offset(_)
    )) + usize::from(matches!(
        position.vertical(),
        CssVerticalPosition::Offset(_)
    ))
}

fn radial_position_is_center(position: &CssPosition) -> bool {
    let horizontal = match position.horizontal() {
        CssHorizontalPosition::Center => true,
        CssHorizontalPosition::Offset(offset) => {
            matches!(offset.value(), CssLength::Percent(value) if value.value() == 50.0)
        }
        _ => false,
    };
    let vertical = match position.vertical() {
        CssVerticalPosition::Center => true,
        CssVerticalPosition::Offset(offset) => {
            matches!(offset.value(), CssLength::Percent(value) if value.value() == 50.0)
        }
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
                    let omit = (index == 0 && is_direct_zero(position.value()))
                        || (index == last && is_direct_hundred_percent(position.value()));
                    if omit {
                        charge(writer, 1)?;
                    } else {
                        writer.append(" ")?;
                        append_line_position(position.value(), writer)?;
                    }
                }
            }
            CssColorStopListItem::Hint(position) => append_line_position(position.value(), writer)?,
        }
    }
    Ok(())
}

fn is_direct_zero(value: &CssLength) -> bool {
    match value {
        CssLength::Zero => true,
        CssLength::Px(number) | CssLength::Percent(number) => number.value() == 0.0,
        CssLength::Dimension(dimension) => dimension.value() == 0.0,
        _ => false,
    }
}

fn is_direct_hundred_percent(value: &CssLength) -> bool {
    matches!(value, CssLength::Percent(number) if number.value() == 100.0)
}

fn append_line_position(value: &CssLength, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    serialize_checked_length_percentage_into(value, &mut writer.context, &mut writer.css)
}
