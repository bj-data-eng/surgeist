//! Cumulative specified serialization of authored Motion values.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::*;
type Result<T> = std::result::Result<T, CssSerializationError>;

crate::specified_rule_serialization::specified_serialization!(CssCoordBox);
impl crate::specified_rule_serialization::SpecifiedProvider for CssCoordBox {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.keyword(self.as_css_str())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssRaySize);
impl crate::specified_rule_serialization::SpecifiedProvider for CssRaySize {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.keyword(self.as_css_str())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssRay);
impl crate::specified_rule_serialization::SpecifiedProvider for CssRay {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.append("ray(")?;
        self.angle()
            .append_specified(&mut writer.context, &mut writer.css)?;
        if let Some(size) = self.size() {
            if size == CssRaySize::ClosestSide {
                writer.without_output(|writer| size.append_to_rule_writer(writer))?;
            } else {
                writer.append(" ")?;
                size.append_to_rule_writer(writer)?;
            }
        }
        if self.contain() {
            writer.append(" ")?;
            writer.keyword("contain")?;
        }
        if let Some(position) = self.position() {
            writer.append(" at ")?;
            position.append_to_rule_writer(writer)?;
        }
        writer.append(")")
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetPathKind);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetPathKind {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Ray(value) => value.append_to_rule_writer(writer),
            Self::Url(value) => value.append_to_rule_writer(writer),
            Self::BasicShape(value) => value.append_to_rule_writer(writer),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetPathValue);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetPathValue {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.path().append_to_rule_writer(writer)?;
        if let Some(coord_box) = self.coord_box() {
            writer.append(" ")?;
            coord_box.append_to_rule_writer(writer)?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetPath);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetPath {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self.view() {
            CssOffsetPathRef::None => writer.keyword("none"),
            CssOffsetPathRef::CoordBox(value) => value.append_to_rule_writer(writer),
            CssOffsetPathRef::Path(value) => value.append_to_rule_writer(writer),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetPosition);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetPosition {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Normal => writer.keyword("normal"),
            Self::Auto => writer.keyword("auto"),
            Self::Position(value) => value.append_to_rule_writer(writer),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetAnchor);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetAnchor {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Position(value) => value.append_to_rule_writer(writer),
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetRotateModifier);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetRotateModifier {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Reverse => "reverse",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffsetRotate);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffsetRotate {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        if let Some(modifier) = self.modifier() {
            modifier.append_to_rule_writer(writer)?;
        }
        if let Some(angle) = self.angle() {
            if self.modifier().is_some() {
                writer.append(" ")?;
            }
            angle.append_specified(&mut writer.context, &mut writer.css)?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOffset);
impl crate::specified_rule_serialization::SpecifiedProvider for CssOffset {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        if let Some(position) = self.position() {
            writer.source_member(0, |writer| position.append_to_rule_writer(writer))?;
        }
        if let Some(path) = self.path() {
            writer.source_member(1, |writer| {
                if self.position().is_some() {
                    writer.append(" ")?;
                }
                path.append_to_rule_writer(writer)
            })?;
        }
        if let Some(distance) = self.distance() {
            writer.source_member(2, |writer| {
                writer.append(" ")?;
                distance.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        if let Some(rotate) = self.rotate() {
            writer.source_member(3, |writer| {
                writer.append(" ")?;
                rotate.append_to_rule_writer(writer)
            })?;
        }
        if let Some(anchor) = self.anchor() {
            writer.source_member(4, |writer| {
                writer.append(" / ")?;
                anchor.append_to_rule_writer(writer)
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specified_rule_serialization::SpecifiedProvider as _;
    use CssSerializationLimits as L;
    use CssSpecifiedValueSerializationErrorKind as K;

    fn values() -> (CssOffsetPath, CssOffsetRotate) {
        let angle = |number| {
            CssAngleValue::from_literal(
                CssAngleLiteral::try_new(number, CssAngleUnit::Degrees).unwrap(),
            )
        };
        (
            CssOffsetPath::from_path(CssOffsetPathValue::new(
                CssOffsetPathKind::Ray(CssRay::new(
                    angle("90"),
                    Some(CssRaySize::ClosestSide),
                    false,
                    None,
                )),
                Some(CssCoordBox::BorderBox),
            )),
            CssOffsetRotate::try_new(Some(CssOffsetRotateModifier::Reverse), Some(angle("450")))
                .unwrap(),
        )
    }

    #[test]
    fn sibling_motion_values_share_prefix_work_and_final_bytes() {
        let (path, rotate) = values();
        let expected = "![ray(90deg) border-box;reverse 450deg]";
        // Prefix one, path pair/ray/angle/explicit default size/box five,
        // rotation aggregate/modifier/angle three: nine cumulative nodes.
        let mut writer = SpecifiedRuleWriter::new(L::new(9, 9, expected.len()));
        writer.node().unwrap();
        writer.append("![").unwrap();
        path.append_to_rule_writer(&mut writer).unwrap();
        writer.append(";").unwrap();
        rotate.append_to_rule_writer(&mut writer).unwrap();
        writer.append("]").unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            K::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            K::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), K::ByteLimit);
    }

    #[test]
    fn suppressed_motion_children_charge_work_and_restore_enclosing_emission_after_failure() {
        let (path, rotate) = values();
        let before = (path.clone(), rotate.clone());
        let visit = |writer: &mut SpecifiedRuleWriter| {
            path.append_to_rule_writer(writer)?;
            rotate.append_to_rule_writer(writer)
        };
        let mut writer = SpecifiedRuleWriter::new(L::new(8, 8, 1));
        writer.without_output(visit).unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            K::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            K::ProjectionNodeLimit
        );
        writer.append("!").unwrap();
        assert_eq!(writer.css, "!");
        for (limits, kind) in [
            (L::new(7, 8, 1), K::InputNodeLimit),
            (L::new(8, 7, 1), K::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(writer.without_output(visit).unwrap_err().kind(), kind);
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            writer.append("!").unwrap();
            assert_eq!(writer.css, "!");
        }
        assert_eq!((path, rotate), before);
    }
}
