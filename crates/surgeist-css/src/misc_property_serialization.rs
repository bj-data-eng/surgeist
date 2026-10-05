//! Canonical specified output for represented caret, containment and blend values.
//! These providers preserve authored models and do not resolve rendering context.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssBlendModeList, CssCaretColor, CssContain, CssContainComponent,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! public_serialization {
    ($ty:ty, $description:literal) => {
        impl $ty {
            #[doc = $description]
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }

            /// Emits complete CSS with one cumulative input, projection and byte budget.
            /// Failure returns no partial output and leaves authored values unchanged.
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

public_serialization!(
    CssCaretColor,
    "Emits `auto` or the child color's specified CSS without resolving the caret color. The keyword charges one node in each budget; the Color branch retains the child's accounting without an extra carrier node."
);
public_serialization!(
    CssContain,
    "Emits represented Containment 1 syntax in grammar order, preserving `strict` and `content`. A keyword charges one node in each budget; a component list charges one aggregate plus each represented keyword. Authored component order remains unchanged."
);
public_serialization!(
    CssBlendModeList,
    "Emits every represented blend mode in its authored comma-list order, including duplicates. The list charges one aggregate plus each primitive in both node budgets; punctuation charges only final bytes."
);

impl CssCaretColor {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Auto => writer.keyword("auto"),
            // The enum is a transparent carrier: Color owns its node accounting.
            Self::Color(color) => color.append_specified(&mut writer.context, &mut writer.css),
        }
    }
}

impl CssContain {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => writer.keyword("none"),
            Self::Strict => writer.keyword("strict"),
            Self::Content => writer.keyword("content"),
            Self::Components(components) => {
                writer.node()?;
                let mut first = true;
                for (component, keyword) in [
                    (CssContainComponent::Size, "size"),
                    (CssContainComponent::Layout, "layout"),
                    (CssContainComponent::Paint, "paint"),
                ] {
                    if components.components().contains(&component) {
                        if !first {
                            writer.append(" ")?;
                        }
                        writer.keyword(keyword)?;
                        first = false;
                    }
                }
                Ok(())
            }
        }
    }
}

impl CssBlendModeList {
    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        for (index, mode) in self.modes().iter().enumerate() {
            if index != 0 {
                writer.append(", ")?;
            }
            mode.append_to_rule_writer(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::{CssBlendMode, CssColor, CssContainComponentList};
    use CssSpecifiedValueSerializationLimits as Limits;

    fn caret(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssCaretColor::Color(Box::new(CssColor::transparent())).append_to_rule_writer(writer)
    }

    fn auto(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssCaretColor::Auto.append_to_rule_writer(writer)
    }

    fn none(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssContain::None.append_to_rule_writer(writer)
    }

    fn strict(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssContain::Strict.append_to_rule_writer(writer)
    }

    fn content(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssContain::Content.append_to_rule_writer(writer)
    }

    fn contain(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssContain::Components(
            CssContainComponentList::try_new(vec![
                CssContainComponent::Paint,
                CssContainComponent::Size,
            ])
            .unwrap(),
        )
        .append_to_rule_writer(writer)
    }

    fn blend(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        CssBlendModeList::try_new(vec![
            CssBlendMode::ColorBurn,
            CssBlendMode::Normal,
            CssBlendMode::Normal,
        ])
        .unwrap()
        .append_to_rule_writer(writer)
    }

    fn compose(writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.append("prefix ")?;
        caret(writer)?;
        writer.append(" ")?;
        contain(writer)?;
        writer.append(" ")?;
        blend(writer)
    }

    #[test]
    fn prefix_and_siblings_share_exact_nodes_and_final_bytes() {
        // Transparent Color: 1; containment list and two keywords: 3;
        // blend list and three primitives: 4. Punctuation has only byte cost.
        let expected = "prefix transparent size paint color-burn, normal, normal";
        let mut writer = SpecifiedRuleWriter::new(Limits::new(8, 8, expected.len()));
        compose(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (Limits::new(7, 8, expected.len()), Kind::InputNodeLimit),
            (Limits::new(8, 7, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(8, 8, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
        }
        assert_eq!(
            CssCaretColor::Auto
                .append_to_rule_writer(&mut writer)
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
    }

    #[test]
    fn suppressed_values_charge_work_with_zero_output_bytes() {
        for (emit, nodes) in [
            (caret as fn(&mut SpecifiedRuleWriter) -> Result<()>, 1),
            (auto, 1),
            (none, 1),
            (strict, 1),
            (content, 1),
            (contain, 3),
            (blend, 4),
        ] {
            let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
            writer.without_output(emit).unwrap();
            assert!(writer.css.is_empty());
            assert!(!writer.context.output_suppressed());
            assert_eq!(
                CssCaretColor::Auto
                    .append_to_rule_writer(&mut writer)
                    .unwrap_err()
                    .kind(),
                Kind::InputNodeLimit
            );
        }
    }

    #[test]
    fn nested_suppression_restores_mode_after_each_resource_failure() {
        for (emit, nodes) in [
            (caret as fn(&mut SpecifiedRuleWriter) -> Result<()>, 1),
            (auto, 1),
            (none, 1),
            (strict, 1),
            (content, 1),
            (contain, 3),
            (blend, 4),
        ] {
            for (limits, kind) in [
                (Limits::new(nodes - 1, nodes, 0), Kind::InputNodeLimit),
                (Limits::new(nodes, nodes - 1, 0), Kind::ProjectionNodeLimit),
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
    fn suppression_leaves_prefix_bytes_and_restores_visible_sibling_emission() {
        let mut writer = SpecifiedRuleWriter::new(Limits::new(9, 9, "prefix auto".len()));
        writer.append("prefix ").unwrap();
        writer
            .without_output(|writer| {
                caret(writer)?;
                contain(writer)?;
                blend(writer)
            })
            .unwrap();
        assert_eq!(writer.css, "prefix ");
        CssCaretColor::Auto
            .append_to_rule_writer(&mut writer)
            .unwrap();
        assert_eq!(writer.css, "prefix auto");
    }

    #[test]
    fn suppressed_rgb_caret_color_has_no_scratch_or_final_byte_cost() {
        let report = crate::parse_style_attribute("caret-color:rgb(1 2 3)");
        assert!(report.is_clean());
        let crate::CssKnownPropertyValueRef::CaretColor(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("caret-color")
        };
        let before = value.caret().clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(65_536, 262_144, 0));
        writer
            .without_output(|writer| value.caret().append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(value.caret(), &before);
        assert_eq!(value.caret().serialize_specified().unwrap(), "rgb(1, 2, 3)");
    }
}
