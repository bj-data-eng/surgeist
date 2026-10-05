//! Specified Images 3 orientation syntax, before computed quarter-turn rounding.
//!
//! The enum carrier is transparent, as for other numeric/keyword carriers.
//! Each explicit keyword costs one input and projection node; an angle delegates
//! its complete cost to the shared angle owner. Separators cost only bytes.

use crate::{
    CssImageOrientation, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssImageOrientation {
    /// Emits canonical specified syntax, with an authored angle before `flip`.
    /// Omitted angles stay omitted; explicit keywords and zero angles remain
    /// explicit. This does not perform computed-value rotation normalization.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares cumulative input, projection and emitted-byte limits with children.
    /// Failure returns no partial CSS and preserves exact authored values and origins.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::FromImage => writer.keyword("from-image"),
            Self::None => writer.keyword("none"),
            Self::Angle(angle) => angle.append_specified(&mut writer.context, &mut writer.css),
            Self::Flip(angle) => {
                if let Some(angle) = angle {
                    angle.append_specified(&mut writer.context, &mut writer.css)?;
                    writer.append(" ")?;
                }
                writer.keyword("flip")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CssAngleCalculation, CssAngleLiteral, CssAngleUnit, CssAngleValue,
        CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits, parse_component_values,
    };

    fn value() -> CssImageOrientation {
        CssImageOrientation::Flip(Some(CssAngleValue::from_literal(
            CssAngleLiteral::try_new("3.000e1", CssAngleUnit::Degrees).unwrap(),
        )))
    }

    fn math() -> CssImageOrientation {
        CssImageOrientation::Flip(Some(
            CssAngleValue::try_from_calculation(
                CssAngleCalculation::try_from_components(
                    parse_component_values("calc(15deg + 15deg)").unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        ))
    }

    #[test]
    fn successive_orientations_share_prefix_bytes_and_both_node_budgets() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(4, 4, 27));
        writer.append("prefix ").unwrap();
        value().append_to_rule_writer(&mut writer).unwrap();
        value().append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "prefix 30deg flip30deg flip");
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
    fn prefixed_orientation_fails_at_each_cumulative_resource_boundary() {
        for (limits, kind) in [
            (Limits::new(1, 2, 17), Kind::InputNodeLimit),
            (Limits::new(2, 1, 17), Kind::ProjectionNodeLimit),
            (Limits::new(2, 2, 16), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("prefix ").unwrap();
            assert_eq!(
                value()
                    .append_to_rule_writer(&mut writer)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert!(writer.css.starts_with("prefix "));
            assert!(!writer.context.output_suppressed());
        }
    }

    #[test]
    fn suppressed_huge_angle_consumes_nodes_without_formatting_or_bytes() {
        let value = CssImageOrientation::Flip(Some(CssAngleValue::from_literal(
            CssAngleLiteral::try_new("1e999999999999999999999999", CssAngleUnit::Degrees).unwrap(),
        )));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 2, 1));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
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
    fn nested_suppression_restores_the_enclosing_mode_on_success_and_error() {
        for (limits, expected) in [
            (Limits::new(2, 2, 1), None),
            (Limits::new(0, 2, 1), Some(Kind::InputNodeLimit)),
            (Limits::new(2, 0, 1), Some(Kind::ProjectionNodeLimit)),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let previous = writer.context.replace_output_suppression(true);
            let result = writer.without_output(|writer| value().append_to_rule_writer(writer));
            assert_eq!(result.err().map(|error| error.kind()), expected);
            assert!(writer.context.output_suppressed());
            assert!(writer.css.is_empty());
            writer.context.replace_output_suppression(previous);
            writer.append("x").unwrap();
            assert_eq!(writer.css, "x");
        }
    }

    #[test]
    fn suppressed_calculation_still_visits_the_shared_numeric_owner_and_restores_mode() {
        let value = math();
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100, 100, 1));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        for (limits, kind) in [
            (Limits::new(1, 100, 1), Kind::InputNodeLimit),
            (Limits::new(100, 0, 1), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                writer
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert!(!writer.context.output_suppressed());
            assert!(writer.css.is_empty());
            writer.append("x").unwrap();
        }
        assert_eq!(value, before);
    }

    #[test]
    fn private_emit_suppression_and_failure_preserve_the_original_angle_token() {
        let source = "/* 🦀 */ 3.000e1deg";
        let components = parse_component_values(source).unwrap();
        let component = components.items().last().unwrap().clone();
        let value = CssImageOrientation::Flip(Some(CssAngleValue::from_literal(
            CssAngleLiteral::try_from_component(component.clone()).unwrap(),
        )));
        let mut emitted = SpecifiedRuleWriter::new(Limits::new(2, 2, 17));
        emitted.append("prefix ").unwrap();
        value.append_to_rule_writer(&mut emitted).unwrap();
        assert_eq!(emitted.css, "prefix 30deg flip");
        let mut suppressed = SpecifiedRuleWriter::new(Limits::new(2, 2, 0));
        suppressed
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(suppressed.css.is_empty());
        let mut failed = SpecifiedRuleWriter::new(Limits::new(2, 2, 9));
        assert_eq!(
            value.append_to_rule_writer(&mut failed).unwrap_err().kind(),
            Kind::ByteLimit
        );
        let CssImageOrientation::Flip(Some(angle)) = value else {
            unreachable!()
        };
        assert_eq!(angle.literal().unwrap().component(), &component);
        assert_eq!(angle.origin(), component.origin());
        let crate::CssValueOrigin::Parsed(origin) = angle.origin() else {
            panic!("original parsed origin")
        };
        assert_eq!(origin.source().as_str(), source);
        let start = source.find("3.000e1deg").unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(origin.span().end().byte_offset().value(), source.len());
        assert_eq!(
            origin.span().start().column().value() as usize,
            source[..start].encode_utf16().count()
        );
    }
}
