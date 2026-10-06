//! Specified output for the represented UI4 Outline domains.
//! Auto, shared Color and imported image-1D use their owning providers.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssOutline, CssOutlineColor, CssOutlineStyle, CssOutlineWidth,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! provider {
    ($ty:ty, $description:literal) => {
        impl $ty {
            #[doc = $description]
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Emits complete specified CSS using one cumulative resource context.
            /// Failure returns no partial CSS and leaves operands and origins unchanged.
            pub fn serialize_specified_with_limits(
                &self,
                limits: CssSpecifiedValueSerializationLimits,
            ) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }
        }
    };
}

provider!(
    CssOutlineStyle,
    "Emits one UI4 Outline style keyword, charging one input and projection node. Hidden is not constructible in this domain."
);
provider!(
    CssOutlineWidth,
    "Emits a represented Outline width. Keywords charge one node in each budget; checked lengths retain the shared numeric accounting."
);
provider!(
    CssOutlineColor,
    "Emits symbolic auto or shared Color specified syntax. Auto charges one node in each budget; Color is a transparent carrier of the child provider. Image-1D delegates transparently to its reusable provider."
);
provider!(
    CssOutline,
    "Emits width, style, color in selected UI4 grammar order. The aggregate and all retained children charge input and projection work, including an omitted second auto. A required synthesized none style charges one projection node and no input node."
);

impl CssOutlineStyle {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::Dotted => "dotted",
            Self::Dashed => "dashed",
            Self::Solid => "solid",
            Self::Double => "double",
            Self::Groove => "groove",
            Self::Ridge => "ridge",
            Self::Inset => "inset",
            Self::Outset => "outset",
        })
    }
}

impl CssOutlineWidth {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Thin => writer.keyword("thin"),
            Self::Medium => writer.keyword("medium"),
            Self::Thick => writer.keyword("thick"),
            Self::Length(length) => length.append_to_rule_writer(writer),
        }
    }
}

impl CssOutlineColor {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => writer.keyword("auto"),
            Self::Color(color) => color.append_specified(&mut writer.context, &mut writer.css),
            Self::Image1D(image) => image.append_to_rule_writer(writer),
        }
    }
}

fn slot(writer: &mut SpecifiedRuleWriter, started: &mut bool) -> Result<()> {
    if *started {
        writer.append(" ")?;
    }
    *started = true;
    Ok(())
}

impl CssOutline {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        let mut started = false;
        if let Some(width) = self.width() {
            slot(writer, &mut started)?;
            width.append_to_rule_writer(writer)?;
        }
        if let Some(style) = self.style() {
            slot(writer, &mut started)?;
            style.append_to_rule_writer(writer)?;
        } else if matches!(self.color(), Some(CssOutlineColor::Auto)) {
            // Lone auto would change the omitted initial None style to Auto.
            // This derived disambiguator consumes projection work, not authored input.
            slot(writer, &mut started)?;
            writer.context.charge_projection(1)?;
            writer.append("none")?;
        }
        if let Some(color) = self.color() {
            if self.style() == Some(CssOutlineStyle::Auto) && matches!(color, CssOutlineColor::Auto)
            {
                // A single auto expresses both semantic slots. The second stored
                // field still visits its provider once, preserving outer suppression.
                writer.without_output(|writer| color.append_to_rule_writer(writer))?;
            } else {
                slot(writer, &mut started)?;
                color.append_to_rule_writer(writer)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::{CssColor, CssKnownPropertyValueRef, parse_style_attribute};
    use CssSpecifiedValueSerializationLimits as Limits;

    fn width(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssOutlineWidth::Thin.append_to_rule_writer(writer)
    }
    fn style(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssOutlineStyle::Solid.append_to_rule_writer(writer)
    }
    fn color(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssOutlineColor::Color(Box::new(CssColor::transparent())).append_to_rule_writer(writer)
    }
    fn auto_color(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssOutlineColor::Auto.append_to_rule_writer(writer)
    }
    fn color_only(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssOutline::try_new(None, None, Some(CssOutlineColor::Auto))
            .unwrap()
            .append_to_rule_writer(writer)
    }
    fn both_auto(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssOutline::try_new(
            None,
            Some(CssOutlineStyle::Auto),
            Some(CssOutlineColor::Auto),
        )
        .unwrap()
        .append_to_rule_writer(writer)
    }
    fn compose(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.append("prefix ")?;
        width(writer)?;
        writer.append(" ")?;
        style(writer)?;
        writer.append(" ")?;
        color(writer)?;
        writer.append(" ")?;
        color_only(writer)
    }

    #[test]
    fn prefix_and_sibling_providers_share_exact_synthesis_and_byte_budgets() {
        let expected = "prefix thin solid transparent none auto";
        // Three scalar siblings plus the shorthand aggregate and Auto color:
        // five input nodes. Its synthesized None adds a sixth projection node.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(5, 6, expected.len()));
        compose(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(style(&mut writer).unwrap_err().kind(), Kind::InputNodeLimit);
        for (limits, kind) in [
            (Limits::new(4, 6, expected.len()), Kind::InputNodeLimit),
            (Limits::new(5, 5, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(5, 6, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
        }
    }

    #[test]
    fn suppression_visits_every_retained_and_synthetic_slot_without_bytes() {
        for (emit, input, projection) in [
            (width as fn(&mut SpecifiedRuleWriter) -> Result<()>, 1, 1),
            (style, 1, 1),
            (color, 1, 1),
            (auto_color, 1, 1),
            (color_only, 2, 3),
            (both_auto, 3, 3),
        ] {
            let mut writer = SpecifiedRuleWriter::new(Limits::new(input, projection, 0));
            writer.without_output(emit).unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            assert_eq!(style(&mut writer).unwrap_err().kind(), Kind::InputNodeLimit);
            for (limits, kind) in [
                (Limits::new(input - 1, projection, 0), Kind::InputNodeLimit),
                (
                    Limits::new(input, projection - 1, 0),
                    Kind::ProjectionNodeLimit,
                ),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                let error = writer
                    .without_output(|writer| {
                        let error = writer.without_output(emit).unwrap_err();
                        assert!(writer.context.output_suppressed());
                        assert!(writer.css.is_empty());
                        Err::<(), _>(error)
                    })
                    .unwrap_err();
                assert_eq!(error.kind(), kind);
                assert!(!writer.context.output_suppressed());
            }
        }
    }

    #[test]
    fn omitted_second_auto_restores_visible_and_enclosing_suppression_modes() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 7, "prefix auto solid".len()));
        writer.append("prefix ").unwrap();
        writer.without_output(both_auto).unwrap();
        assert_eq!(writer.css, "prefix ");
        both_auto(&mut writer).unwrap();
        assert!(!writer.context.output_suppressed());
        writer.append(" ").unwrap();
        style(&mut writer).unwrap();
        assert_eq!(writer.css, "prefix auto solid");
    }

    #[test]
    fn omitted_second_auto_failure_restores_visible_output_mode() {
        for (limits, kind) in [
            (Limits::new(2, 3, 5), Kind::InputNodeLimit),
            (Limits::new(3, 2, 5), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(both_auto(&mut writer).unwrap_err().kind(), kind);
            assert!(!writer.context.output_suppressed());
            // The private writer may retain bytes on error; its public owner
            // discards them. Subsequent punctuation still uses visible mode.
            assert_eq!(writer.css, "auto");
            writer.append("!").unwrap();
            assert_eq!(writer.css, "auto!");
        }
    }

    #[test]
    fn suppressed_numeric_and_rgb_children_charge_no_scratch_bytes() {
        let report = parse_style_attribute("outline:2px solid rgb(1 2 3)");
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::Outline(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("outline")
        };
        let before = value.value().clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(65_536, 262_144, 0));
        writer
            .without_output(|writer| value.value().append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(value.value(), &before);
        assert_eq!(
            value.value().serialize_specified().unwrap(),
            "2px solid rgb(1, 2, 3)"
        );
    }
}
