//! Canonical authored Masking values sharing one cumulative writer.
use super::*;
use crate::{
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};
type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! provider {
    ($ty:ty, $value:ident, $writer:ident => $body:block) => {
        impl $ty {
            /// Emits canonical specified syntax without contextual resolution.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }
            /// Uses cumulative input, projection and UTF-8 byte limits atomically.
            /// Scalars cost one node; lists and authored groups add one node,
            /// visiting all retained children. Shared edge/image costs are unchanged.
            pub fn serialize_specified_with_limits(&self, limits: CssSpecifiedValueSerializationLimits) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }
            pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                let $writer = writer;
                $body
            }
        }
    };
}
provider!(CssMaskBox, value, writer => { writer.keyword(value.keyword()) });
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
        Self::Box(value) => writer.source_property(crate::CssKnownProperty::MaskOrigin, |writer| value.append_to_rule_writer(writer)),
        Self::NoClip => writer.source_property(crate::CssKnownProperty::MaskClip, |writer| CssMaskClip::NoClip.append_to_rule_writer(writer)),
        Self::Pair { origin, clip } => {
            writer.source_property(crate::CssKnownProperty::MaskOrigin, |writer| origin.append_to_rule_writer(writer))?;
            writer.source_property(crate::CssKnownProperty::MaskClip, |writer| { writer.append(" ")?; clip.append_to_rule_writer(writer) })
        }
    }
});
fn before_field(writer: &mut SpecifiedRuleWriter, emitted: &mut bool) -> Result<()> {
    if *emitted {
        writer.append(" ")?;
    }
    *emitted = true;
    Ok(())
}
provider!(CssMaskBorder, value, writer => {
    writer.node()?;
    let mut emitted = false;
    if let Some(source) = value.source() {
        writer.source_member(0, |writer| {
        before_field(writer, &mut emitted)?;
        source.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    if let Some(slice) = value.slice() {
        writer.source_member(1, |writer| {
        before_field(writer, &mut emitted)?;
        slice.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    if let Some(width) = value.width() {
        writer.source_member(2, |writer| {
        writer.append(" / ")?;
        width.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    if let Some(outset) = value.outset() {
        writer.source_member(3, |writer| {
        writer.append(if value.width().is_some() { " / " } else { " / / " })?;
        outset.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    if let Some(repeat) = value.repeat() {
        writer.source_member(4, |writer| {
        before_field(writer, &mut emitted)?;
        repeat.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    if let Some(mode) = value.mode() {
        writer.source_member(5, |writer| {
        before_field(writer, &mut emitted)?;
        mode.append_to_rule_writer(writer)?;
            Ok(())
        })?;
    }
    Ok(())
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    #[test]
    fn lists_and_groups_share_prefix_sibling_and_suppressed_work() {
        let list =
            CssMaskModeList::try_new(vec![CssMaskMode::Alpha, CssMaskMode::Luminance]).unwrap();
        let boxes = CssMaskLayerBoxes::Pair {
            origin: CssMaskBox::PaddingBox,
            clip: CssMaskClip::NoClip,
        };
        let expected = "!alpha, luminance;padding-box no-clip";
        // Prefix 1, list+two scalars 3, boxes+two scalars 3.
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
            7,
            7,
            expected.len(),
        ));
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
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(6, 6, 1));
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
        // Prefix1 + border aggregate/source/mode3 + scalar sibling1.
        let expected = "!none alpha;evenodd";
        let mut writer = SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(
            5,
            5,
            expected.len(),
        ));
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
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(6, 6, 0));
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
            origin: CssMaskBox::BorderBox,
            clip: CssMaskClip::NoClip,
        };
        let before = boxes;
        let mut writer =
            SpecifiedRuleWriter::new(CssSpecifiedValueSerializationLimits::new(2, 3, 1));
        let error = writer
            .without_output(|writer| boxes.append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        assert_eq!(boxes, before);
        writer.append("!").unwrap();
        assert_eq!(writer.css, "!");
        assert_eq!(boxes.serialize_specified().unwrap(), "border-box no-clip");
    }
}
