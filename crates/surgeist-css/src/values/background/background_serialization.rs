//! Canonical specified Backgrounds 3 shorthand composition.

use crate::component::{CssComponentValueRef, CssValueTokenRef};
use crate::numeric::{CssLengthPercentage, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::background::{
    CssBackground, CssBackgroundAttachment, CssBackgroundBox, CssBackgroundLayer,
    CssBackgroundLayerBoxes, CssBackgroundRepeat, CssBackgroundRepeatStyle, CssBackgroundSize,
};
use crate::values::common::CssAutoOr;
use crate::values::image::CssImageValue;
use crate::values::position::{CssHorizontalPosition, CssVerticalPosition};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBackground {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.source_member(0, SpecifiedRuleWriter::node)?;
        for (index, layer) in self.layers().iter().enumerate() {
            if index != 0 {
                // The admitted image-list count owns the layer separator.
                writer.source_member(0, |writer| writer.append(", "))?;
            }
            append_layer(layer, writer)?;
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBackground);

// Prove only ordinary exact zero offsets. Keywords and calculations remain authored.
fn is_zero(value: &CssNumeric<CssLengthPercentage>) -> bool {
    let Some(component) = value.literal_component() else {
        return false;
    };
    let number = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number))
        | CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
        | CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) => number,
        _ => return false,
    };
    crate::serialization::exact_decimal::LexicalDecimal::new(number.representation()).is_zero()
}

fn initial_position(position: &crate::values::background::CssBackgroundPosition) -> bool {
    matches!((position.horizontal(), position.vertical()),
        (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) if is_zero(x) && is_zero(y))
}

fn initial_size(size: &CssBackgroundSize) -> bool {
    matches!(
        size,
        CssBackgroundSize::Explicit {
            width: CssAutoOr::Auto,
            height: None | Some(CssAutoOr::Auto),
        }
    )
}

fn component(
    writer: &mut SpecifiedRuleWriter,
    emitted: &mut bool,
    omit: bool,
    property: crate::property::CssKnownProperty,
    visit: impl FnOnce(&mut SpecifiedRuleWriter) -> Result<()>,
) -> Result<()> {
    writer.source_property(property, |writer| {
        if omit {
            writer.without_output(visit)
        } else {
            writer.space_before_item(emitted)?;
            visit(writer)
        }
    })
}

fn append_layer(layer: &CssBackgroundLayer, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    // The image list owns the admitted layer count; no item source is invented.
    writer.source_property(
        crate::property::CssKnownProperty::BackgroundImage,
        SpecifiedRuleWriter::node,
    )?;
    let mut emitted = false;
    if let Some(image) = layer.image() {
        component(
            writer,
            &mut emitted,
            matches!(image, CssImageValue::None),
            crate::property::CssKnownProperty::BackgroundImage,
            |writer| image.append_specified(writer),
        )?;
    }
    let retained_size = layer.size().is_some_and(|size| !initial_size(size));
    if let Some(position) = layer.position() {
        component(
            writer,
            &mut emitted,
            !retained_size && initial_position(position),
            crate::property::CssKnownProperty::BackgroundPosition,
            |writer| position.append_specified(writer),
        )?;
    }
    if let Some(size) = layer.size() {
        writer.source_property(
            crate::property::CssKnownProperty::BackgroundSize,
            |writer| {
                if retained_size {
                    writer.append(" / ")?;
                    size.append_specified(writer)
                } else {
                    writer.without_output(|writer| size.append_specified(writer))
                }
            },
        )?;
    }
    if let Some(repeat) = layer.repeat() {
        let omit = matches!(
            repeat,
            CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::Repeat,
                y: CssBackgroundRepeatStyle::Repeat,
            }
        );
        component(
            writer,
            &mut emitted,
            omit,
            crate::property::CssKnownProperty::BackgroundRepeat,
            |writer| repeat.append_specified(writer),
        )?;
    }
    if let Some(attachment) = layer.attachment() {
        component(
            writer,
            &mut emitted,
            attachment == CssBackgroundAttachment::Scroll,
            crate::property::CssKnownProperty::BackgroundAttachment,
            |writer| attachment.append_specified(writer),
        )?;
    }
    if let Some(boxes) = layer.boxes() {
        let origin = boxes.origin();
        let clip = boxes.clip();
        let omit = origin == CssBackgroundBox::PaddingBox && clip == CssBackgroundBox::BorderBox;
        let visit = |writer: &mut SpecifiedRuleWriter| {
            writer.source_property(
                crate::property::CssKnownProperty::BackgroundOrigin,
                |writer| origin.append_specified(writer),
            )?;
            if matches!(boxes, CssBackgroundLayerBoxes::OriginAndClip { .. }) {
                writer.source_property(
                    crate::property::CssKnownProperty::BackgroundClip,
                    |writer| {
                        if origin == clip {
                            writer.without_output(|writer| clip.append_specified(writer))
                        } else {
                            writer.append(" ")?;
                            clip.append_specified(writer)
                        }
                    },
                )?;
            }
            Ok(())
        };
        if omit {
            writer.without_output(visit)?;
        } else {
            writer.source_property(
                crate::property::CssKnownProperty::BackgroundOrigin,
                |writer| writer.space_before_item(&mut emitted),
            )?;
            visit(writer)?;
        }
    }
    if let Some(color) = layer.color() {
        component(
            writer,
            &mut emitted,
            color.is_transparent(),
            crate::property::CssKnownProperty::BackgroundColor,
            |writer| color.append_specified(&mut writer.context, &mut writer.css),
        )?;
    }
    if !emitted {
        // The grammar requires a token for an otherwise empty effective layer.
        writer.source_property(
            crate::property::CssKnownProperty::BackgroundImage,
            |writer| {
                writer.context.charge_projection(1)?;
                writer.append("none")
            },
        )?;
    }
    Ok(())
}
