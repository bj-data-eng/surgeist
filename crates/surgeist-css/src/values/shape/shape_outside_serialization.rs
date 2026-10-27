//! Cumulative specified serialization of authored Shapes 1 compositions.

use crate::serialization::CssSerializationError;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::shape::{CssShapeBox, CssShapeOutside, CssShapeOutsideShape};

type Result<T> = std::result::Result<T, CssSerializationError>;

crate::serialization::specified_rule_serialization::specified_serialization!(CssShapeBox);
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssShapeBox {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append(self.as_str())
    }
}

crate::serialization::specified_rule_serialization::specified_serialization!(CssShapeOutside);
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssShapeOutside {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                writer.node()?;
                writer.append("none")
            }
            Self::ShapeBox(value) => value.append_to_rule_writer(writer),
            Self::BasicShape(value) => value.append_to_rule_writer(writer),
            Self::Image(value) => value.append_to_rule_writer(writer),
        }
    }
}

crate::serialization::specified_rule_serialization::specified_serialization!(CssShapeOutsideShape);
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssShapeOutsideShape
{
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.shape().append_specified(writer)?;
        if let Some(reference_box) = self.reference_box() {
            writer.append(" ")?;
            reference_box.append_to_rule_writer(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::CssSerializationLimits;
    use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::values::image::{CssImage, CssImageValue, CssLightDarkImage};
    use crate::values::shape::{CssBasicShape, CssCircleRadius, CssCircleShape};

    fn values() -> [CssShapeOutside; 4] {
        [
            CssShapeOutside::BasicShape(CssShapeOutsideShape::new(
                CssBasicShape::Circle(CssCircleShape::new(CssCircleRadius::Default, None)),
                Some(CssShapeBox::MarginBox),
            )),
            CssShapeOutside::Image(
                CssImage::try_new(CssImageValue::LightDark(Box::new(
                    CssLightDarkImage::try_new(CssImageValue::None, CssImageValue::None).unwrap(),
                )))
                .unwrap(),
            ),
            CssShapeOutside::ShapeBox(CssShapeBox::BorderBox),
            CssShapeOutside::None,
        ]
    }

    #[test]
    fn prefix_and_all_outside_branches_share_semantic_work_and_final_bytes() {
        let values = values();
        let expected = "!circle() margin-box;light-dark(none, none);border-box;none";
        // Prefix one, shape aggregate/function/box three, image function/none/none
        // three, standalone box one, none one: nine cumulative nodes.
        let mut writer =
            SpecifiedRuleWriter::new(CssSerializationLimits::new(9, 9, expected.len()));
        writer.node().unwrap();
        writer.append("!").unwrap();
        for (index, value) in values.iter().enumerate() {
            if index != 0 {
                writer.append(";").unwrap();
            }
            value.append_to_rule_writer(&mut writer).unwrap();
        }
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
    }

    #[test]
    fn suppressed_outside_children_charge_work_without_final_bytes_and_restore_emission() {
        let values = values();
        let before = values.clone();
        let mut writer = SpecifiedRuleWriter::new(CssSerializationLimits::new(8, 8, 1));
        writer.append("!").unwrap();
        writer
            .without_output(|writer| {
                for value in &values {
                    value.append_to_rule_writer(writer)?;
                }
                Ok(())
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
        assert_eq!(values, before);
        for (limits, kind) in [
            (CssSerializationLimits::new(7, 8, 1), Kind::InputNodeLimit),
            (
                CssSerializationLimits::new(8, 7, 1),
                Kind::ProjectionNodeLimit,
            ),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let error = writer
                .without_output(|writer| {
                    for value in &values {
                        value.append_to_rule_writer(writer)?;
                    }
                    Ok(())
                })
                .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            writer.append("!").unwrap();
            assert_eq!(writer.css, "!");
            assert_eq!(values, before);
        }
    }
}
