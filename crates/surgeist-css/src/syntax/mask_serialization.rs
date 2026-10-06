//! Specified serialization of authored Masking 1 layers.

use super::{CssMaskLayer, CssMaskList};
use crate::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssMaskList {
    /// Serializes all eight authored fields in canonical layer order.
    /// Authored omissions, symbolic values and source origins remain unchanged.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses cumulative input, projection and output-byte limits, returning no partial CSS.
    /// The list and each layer cost one input and projection node. Children retain
    /// their providers' costs. A size without position generates two projection-only
    /// `0%` tokens before the slash; it does not add authored input or mutate the layer.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.source_member(0, |writer| writer.node())?;
        for (index, layer) in self.layers.iter().enumerate() {
            if index != 0 {
                // The admitted image-list count owns the layer separator.
                writer.source_member(0, |writer| writer.append(", "))?;
            }
            append_layer(layer, writer)?;
        }
        Ok(())
    }
}

fn before_field(writer: &mut SpecifiedRuleWriter, emitted: &mut bool) -> Result<()> {
    if *emitted {
        writer.append(" ")?;
    }
    *emitted = true;
    Ok(())
}

fn append_layer(layer: &CssMaskLayer, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.source_property(crate::CssKnownProperty::MaskImage, |writer| writer.node())?;
    let mut emitted = false;
    if let Some(image) = &layer.image {
        writer.source_member(0, |writer| {
            before_field(writer, &mut emitted)?;
            image.append_to_rule_writer(writer)
        })?;
    }
    if let Some(position) = &layer.position {
        writer.source_member(1, |writer| {
            before_field(writer, &mut emitted)?;
            position.append_to_rule_writer(writer)
        })?;
    }
    if let Some(size) = &layer.size {
        if layer.position.is_none() {
            writer.source_property(crate::CssKnownProperty::MaskPosition, |writer| {
                before_field(writer, &mut emitted)?;
                writer.context.charge_projection(2)?;
                writer.append("0% 0%")
            })?;
        }
        writer.source_member(2, |writer| {
            writer.append(" / ")?;
            size.append_to_rule_writer(writer)
        })?;
    }
    if let Some(repeat) = &layer.repeat {
        writer.source_member(3, |writer| {
            before_field(writer, &mut emitted)?;
            repeat.append_to_rule_writer(writer)
        })?;
    }
    if let Some(boxes) = layer.boxes {
        let first = if matches!(boxes, crate::CssMaskLayerBoxes::NoClip) {
            crate::CssKnownProperty::MaskClip
        } else {
            crate::CssKnownProperty::MaskOrigin
        };
        writer.source_property(first, |writer| before_field(writer, &mut emitted))?;
        // Pair boxes retain two distinct sources; a collapsed box retains one
        // actual provider. The box owner selects those real branch transitions.
        boxes.append_to_rule_writer(writer)?;
    }
    if let Some(composite) = layer.composite {
        writer.source_member(6, |writer| {
            before_field(writer, &mut emitted)?;
            composite.append_to_rule_writer(writer)
        })?;
    }
    if let Some(mode) = layer.mode {
        writer.source_member(7, |writer| {
            before_field(writer, &mut emitted)?;
            mode.append_to_rule_writer(writer)
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssBackgroundRepeat, CssBackgroundRepeatStyle, CssBackgroundSize,
        CssBackgroundSizeComponent, CssImageValue, CssSpecifiedValueSerializationErrorKind as Kind,
    };

    fn sized() -> CssMaskList {
        CssMaskList::try_new(vec![
            CssMaskLayer::try_new(
                None,
                None,
                Some(CssBackgroundSize::Contain),
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        ])
        .unwrap()
    }

    #[test]
    fn synthesized_position_shares_prefix_siblings_and_exact_projection_costs() {
        let value = sized();
        let expected = "!0% 0% / contain;0% 0% / contain";
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
            7,
            11,
            expected.len(),
        ));
        writer.node().unwrap();
        writer.append("!").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        writer.append(";").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
        assert!(value.layers[0].position.is_none());
    }

    #[test]
    fn suppressed_synthetic_and_shortened_children_charge_all_semantic_visits() {
        let value = CssMaskList::try_new(vec![
            CssMaskLayer::try_new(
                Some(CssImageValue::None),
                None,
                Some(CssBackgroundSize::Explicit {
                    width: CssBackgroundSizeComponent::Auto,
                    height: Some(CssBackgroundSizeComponent::Auto),
                }),
                Some(CssBackgroundRepeat::Axes {
                    x: CssBackgroundRepeatStyle::Repeat,
                    y: CssBackgroundRepeatStyle::Repeat,
                }),
                None,
                None,
                None,
            )
            .unwrap(),
        ])
        .unwrap();
        // list + layer + none + size/width/height + repeat/x/y = 9;
        // generated position adds two projection nodes even with no output.
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(9, 11, 1));
        writer.append("x").unwrap();
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert_eq!(writer.css, "x");
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(
            value.serialize_specified().unwrap(),
            "none 0% 0% / auto repeat"
        );
    }

    #[test]
    fn nested_suppressed_failure_restores_enclosing_mode() {
        let value = sized();
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(2, 100, 1));
        let error = writer
            .without_output(|writer| {
                let error = writer
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err();
                assert!(writer.context.output_suppressed());
                Err::<(), _>(error)
            })
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }

    #[test]
    fn cumulative_counter_overflow_is_typed_and_leaves_source_unchanged() {
        let value = sized();
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
            usize::MAX,
            usize::MAX,
            0,
        ));
        writer.context.charge_input(usize::MAX).unwrap();
        let error = writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), Kind::CapacityOverflow);
        assert_eq!(value, before);
        assert!(!writer.context.output_suppressed());
        assert_eq!(value.serialize_specified().unwrap(), "0% 0% / contain");
    }

    #[test]
    fn suppressed_gradient_math_and_url_visit_without_byte_scratch() {
        let report = crate::parse_style_attribute(
            "mask:linear-gradient(red calc(1px + 5%), blue) calc(10px + 1em) top / contain no-repeat, url(\"a\\\"b\")",
        );
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let crate::CssKnownPropertyValueRef::Mask(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("mask")
        };
        let value = wrapper.value();
        let before = value.clone();
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(100, 100, 1));
        writer.append("x").unwrap();
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert_eq!(writer.css, "x");
        assert_eq!(*value, before);
        assert!(!writer.context.output_suppressed());
    }
}
