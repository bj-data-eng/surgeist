//! Canonical authored Masking values sharing one cumulative writer.
use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::border::border_image_serialization::{
    EdgeMembers, append_all_initial_fallback, append_edge_members, literal_initial,
};
use crate::values::border::{
    CssBorderImageSlice, CssBorderImageSliceComponent, CssBorderImageWidth,
    CssBorderImageWidthComponent,
};
use crate::values::effects::{
    CssClipRule, CssMaskBorder, CssMaskBoxList, CssMaskClip, CssMaskClipList, CssMaskComposite,
    CssMaskCompositeList, CssMaskLayerBoxes, CssMaskMode, CssMaskModeList, CssMaskType,
};
#[cfg(test)]
use crate::values::image::CssImageValue;
#[cfg(test)]
use crate::values::transform::CssCoordBox;
type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! provider {
    ($ty:ty, $value:ident, $writer:ident => $body:block) => {
        impl crate::serialization::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                let $writer = writer;
                $body
            }
        }
        crate::serialization::specified_rule_serialization::specified_serialization!($ty);
    };
}
provider!(CssMaskClip, value, writer => {
    match value { Self::Box(value) => value.append_to_rule_writer(writer), Self::NoClip => writer.keyword("no-clip") }
});
provider!(CssMaskMode, value, writer => {
    writer.keyword(match value { Self::Alpha => "alpha", Self::Luminance => "luminance", Self::MatchSource => "match-source" })
});
provider!(CssMaskComposite, value, writer => {
    writer.keyword(match value { Self::Add => "add", Self::Subtract => "subtract", Self::Intersect => "intersect", Self::Exclude => "exclude" })
});
provider!(CssMaskType, value, writer => {
    writer.keyword(match value { Self::Alpha => "alpha", Self::Luminance => "luminance" })
});
provider!(CssClipRule, value, writer => {
    writer.keyword(match value { Self::Nonzero => "nonzero", Self::Evenodd => "evenodd" })
});
macro_rules! list_provider {
    ($ty:ty, $accessor:ident) => {
        provider!($ty, value, writer => {
            writer.node()?;
            for (index, child) in value.$accessor().iter().enumerate() {
                if index != 0 { writer.append(", ")?; }
                child.append_to_rule_writer(writer)?;
            }
            Ok(())
        });
    };
}
list_provider!(CssMaskBoxList, boxes);
list_provider!(CssMaskClipList, clips);
list_provider!(CssMaskModeList, modes);
list_provider!(CssMaskCompositeList, operators);
provider!(CssMaskLayerBoxes, value, writer => {
    writer.node()?;
    match value {
        Self::Box(value) => writer.source_property(crate::property::CssKnownProperty::MaskOrigin, |writer| value.append_to_rule_writer(writer)),
        Self::NoClip => writer.source_property(crate::property::CssKnownProperty::MaskClip, |writer| CssMaskClip::NoClip.append_to_rule_writer(writer)),
        Self::Pair { origin, clip } => {
            writer.source_property(crate::property::CssKnownProperty::MaskOrigin, |writer| origin.append_to_rule_writer(writer))?;
            writer.source_property(crate::property::CssKnownProperty::MaskClip, |writer| { writer.append(" ")?; clip.append_to_rule_writer(writer) })
        }
    }
});
/// Masking 1 §8.3: `mask-border-slice` is initially `0`, with no `fill`.
fn initial_mask_border_slice(value: &CssBorderImageSlice) -> bool {
    !value.fill()
        && value.values().iter().all(|value| {
            matches!(value,
            CssBorderImageSliceComponent::Number(number)
                if literal_initial(number.literal_component(), None))
        })
}

/// Masking 1 §8.4: `mask-border-width` is initially `auto`.
fn initial_mask_border_width(value: &CssBorderImageWidth) -> bool {
    value
        .values()
        .iter()
        .all(|value| matches!(value, CssBorderImageWidthComponent::Auto))
}

// Masking 1 §8.7 gives `mask-border` the grammar of `border-image` plus a mode,
// so the shared members follow border-image's omission and the initial `alpha`
// mode is omitted last; all-initial is `none`, as for `border-image`.
provider!(CssMaskBorder, value, writer => {
    writer.node()?;
    let mut emitted = false;
    append_edge_members(
        writer,
        &mut emitted,
        &EdgeMembers {
            source: value.source(),
            slice: value.slice(),
            width: value.width(),
            outset: value.outset(),
            repeat: value.repeat(),
            initial_slice: initial_mask_border_slice,
            initial_width: initial_mask_border_width,
        },
    )?;
    if let Some(mode) = value.mode() {
        writer.source_member(5, |writer| {
            if mode == CssMaskType::Alpha {
                writer.without_output(|writer| mode.append_to_rule_writer(writer))?;
            } else {
                writer.space_before_item(&mut emitted)?;
                mode.append_to_rule_writer(writer)?;
            }
            Ok(())
        })?;
    }
    if !emitted {
        append_all_initial_fallback(writer)?;
    }
    Ok(())
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::CssSerializationLimits;
    use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;
    #[test]
    fn lists_and_groups_share_prefix_sibling_and_suppressed_work() {
        let list =
            CssMaskModeList::try_new(vec![CssMaskMode::Alpha, CssMaskMode::Luminance]).unwrap();
        let boxes = CssMaskLayerBoxes::Pair {
            origin: CssCoordBox::PaddingBox,
            clip: CssMaskClip::NoClip,
        };
        let expected = "!alpha, luminance;padding-box no-clip";
        // Prefix 1, list+two scalars 3, boxes+two scalars 3.
        let mut writer =
            SpecifiedRuleWriter::new(CssSerializationLimits::new(7, 7, expected.len()));
        writer.node().unwrap();
        writer.append("!").unwrap();
        list.append_to_rule_writer(&mut writer).unwrap();
        writer.append(";").unwrap();
        boxes.append_to_rule_writer(&mut writer).unwrap();
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
        let mut writer = SpecifiedRuleWriter::new(CssSerializationLimits::new(6, 6, 1));
        writer.append("!").unwrap();
        writer
            .without_output(|writer| {
                list.append_to_rule_writer(writer)?;
                boxes.append_to_rule_writer(writer)
            })
            .unwrap();
        assert_eq!(writer.css, "!");
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }
    #[test]
    fn border_aggregate_and_scalar_siblings_share_cumulative_limits() {
        let border = CssMaskBorder::try_new(
            Some(CssImageValue::None),
            None,
            None,
            None,
            None,
            Some(CssMaskType::Alpha),
        )
        .unwrap();
        // Prefix1 + border aggregate/source/mode3 + the generated `none` of an
        // all-initial border (projection only) + scalar sibling1.
        let expected = "!none;evenodd";
        let mut writer =
            SpecifiedRuleWriter::new(CssSerializationLimits::new(5, 6, expected.len()));
        writer.node().unwrap();
        writer.append("!").unwrap();
        border.append_to_rule_writer(&mut writer).unwrap();
        writer.append(";").unwrap();
        CssClipRule::Evenodd
            .append_to_rule_writer(&mut writer)
            .unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        let mut writer = SpecifiedRuleWriter::new(CssSerializationLimits::new(6, 8, 0));
        writer
            .without_output(|writer| {
                border.append_to_rule_writer(writer)?;
                border.append_to_rule_writer(writer)
            })
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }
    #[test]
    fn suppressed_failure_restores_writer_and_retained_boxes() {
        let boxes = CssMaskLayerBoxes::Pair {
            origin: CssCoordBox::BorderBox,
            clip: CssMaskClip::NoClip,
        };
        let before = boxes;
        let mut writer = SpecifiedRuleWriter::new(CssSerializationLimits::new(2, 3, 1));
        let error = writer
            .without_output(|writer| boxes.append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        assert_eq!(boxes, before);
        writer.append("!").unwrap();
        assert_eq!(writer.css, "!");
        assert_eq!(
            boxes.serialize_specified(Default::default()).unwrap(),
            "border-box no-clip"
        );
    }
}
