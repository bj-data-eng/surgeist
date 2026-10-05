//! Canonical specified Backgrounds 3 shorthand composition.

use crate::{
    CssBackground, CssBackgroundAttachment, CssBackgroundBox, CssBackgroundLayer,
    CssBackgroundLayerBoxes, CssBackgroundRepeat, CssBackgroundRepeatStyle, CssBackgroundSize,
    CssBackgroundSizeComponent, CssComponentValueRef, CssHorizontalPosition, CssImageValue,
    CssSpecifiedLengthPercentage, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueTokenRef, CssVerticalPosition,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

impl CssBackground {
    /// Serializes canonical specified layers without resolving contextual values.
    /// Simple literal initials may be omitted; authored omissions remain unchanged.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares one input, projection and final byte budget across all authored children.
    /// Omitted children still count as visits. Failure returns no partial text.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        charge(writer)?;
        for (index, layer) in self.layers().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            append_layer(layer, writer)?;
        }
        Ok(())
    }
}

// Prove only ordinary exact zero offsets. Keywords and calculations remain authored.
fn is_zero(value: &CssSpecifiedLengthPercentage) -> bool {
    let Some(component) = value.literal_component() else {
        return false;
    };
    let number = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number))
        | CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
        | CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) => number,
        _ => return false,
    };
    crate::exact_decimal::LexicalDecimal::new(number.representation()).len == 0
}

fn initial_position(position: &crate::CssBackgroundPosition) -> bool {
    matches!((position.horizontal(), position.vertical()),
        (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) if is_zero(x) && is_zero(y))
}

fn initial_size(size: &CssBackgroundSize) -> bool {
    matches!(
        size,
        CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Auto,
            height: None | Some(CssBackgroundSizeComponent::Auto),
        }
    )
}

fn before_component(writer: &mut SpecifiedRuleWriter, emitted: &mut bool) -> Result<()> {
    if *emitted {
        writer.append(" ")?;
    }
    *emitted = true;
    Ok(())
}

fn component(
    writer: &mut SpecifiedRuleWriter,
    emitted: &mut bool,
    omit: bool,
    visit: impl FnOnce(&mut SpecifiedRuleWriter) -> Result<()>,
) -> Result<()> {
    if omit {
        writer.without_output(visit)
    } else {
        before_component(writer, emitted)?;
        visit(writer)
    }
}

fn append_layer(layer: &CssBackgroundLayer, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    charge(writer)?;
    let mut emitted = false;
    if let Some(image) = layer.image() {
        component(
            writer,
            &mut emitted,
            matches!(image, CssImageValue::None),
            |writer| image.append_specified(writer),
        )?;
    }
    let retained_size = layer.size().is_some_and(|size| !initial_size(size));
    if let Some(position) = layer.position() {
        component(
            writer,
            &mut emitted,
            !retained_size && initial_position(position),
            |writer| position.append_specified(writer),
        )?;
    }
    if let Some(size) = layer.size() {
        if retained_size {
            writer.append(" / ")?;
            size.append_specified(writer)?;
        } else {
            writer.without_output(|writer| size.append_specified(writer))?;
        }
    }
    if let Some(repeat) = layer.repeat() {
        let omit = matches!(
            repeat,
            CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::Repeat,
                y: CssBackgroundRepeatStyle::Repeat,
            }
        );
        component(writer, &mut emitted, omit, |writer| {
            repeat.append_specified(writer)
        })?;
    }
    if let Some(attachment) = layer.attachment() {
        component(
            writer,
            &mut emitted,
            attachment == CssBackgroundAttachment::Scroll,
            |writer| attachment.append_specified(writer),
        )?;
    }
    if let Some(boxes) = layer.boxes() {
        let origin = boxes.origin();
        let clip = boxes.clip();
        let omit = origin == CssBackgroundBox::PaddingBox && clip == CssBackgroundBox::BorderBox;
        component(writer, &mut emitted, omit, |writer| {
            origin.append_specified(writer)?;
            if matches!(boxes, CssBackgroundLayerBoxes::OriginAndClip { .. }) {
                if origin == clip {
                    // The second authored box still visits its existing provider.
                    writer.without_output(|writer| clip.append_specified(writer))?;
                } else {
                    writer.append(" ")?;
                    clip.append_specified(writer)?;
                }
            }
            Ok(())
        })?;
    }
    if let Some(color) = layer.color() {
        component(writer, &mut emitted, color.is_transparent(), |writer| {
            color.append_specified(&mut writer.context, &mut writer.css)
        })?;
    }
    if !emitted {
        // The grammar requires a token for an otherwise empty effective layer.
        writer.context.charge_projection(1)?;
        writer.append("none")?;
    }
    Ok(())
}
