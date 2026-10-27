//! Exact authored border widths and the physical border triples that contain them.

use crate::color::CssColor;
use crate::component::CssValueOrigin;
use crate::numeric::{CssLength, CssNonNegative, CssNumeric};
use crate::serialization::CssSerializationError;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::specified_inverse::InverseValueEquality as _;
use crate::values::border::CssLineStyle;
use crate::values::common::common_values::{four_sides_entry, side_element, side_pair_entry};

type Result<T> = std::result::Result<T, CssSerializationError>;

/// One exact authored `<line-width>` before border-style-dependent used-value rules.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssLineWidth {
    /// `thin`: equivalent to `1px`.
    Thin,
    /// `medium`: equivalent to `3px`.
    Medium,
    /// `thick`: equivalent to `5px`.
    Thick,
    /// A non-negative `<length>`.
    Length(CssNumeric<CssLength, CssNonNegative>),
}

impl CssLineWidth {
    /// Returns the authored length, or `None` for the `thin`, `medium` and `thick` keywords.
    pub fn length(&self) -> Option<&CssNumeric<CssLength, CssNonNegative>> {
        match self {
            Self::Length(value) => Some(value),
            _ => None,
        }
    }
    /// Returns the value origin of the authored length, or `None` for the keywords.
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length()
            .map(CssNumeric::<CssLength, CssNonNegative>::origin)
    }
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Thin => context.keyword(output, "thin"),
            Self::Medium => context.keyword(output, "medium"),
            Self::Thick => context.keyword(output, "thick"),
            Self::Length(value) => {
                let text = value.capture_specified(context)?;
                context.append(output, &text)
            }
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssLineWidth {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<()> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssLineWidth);
impl PartialEq for CssLineWidth {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Thin, Self::Thin)
            | (Self::Medium, Self::Medium)
            | (Self::Thick, Self::Thick) => true,
            (Self::Length(left), Self::Length(right)) => left == right,
            _ => false,
        }
    }
}
impl Eq for CssLineWidth {}

side_element!(
    CssLineWidth,
    same = |a, b, context| a.inverse_equal(b, context)
);
four_sides_entry!(CssLineWidth);
side_pair_entry!(CssLineWidth);

/// Checked authored components of `border` or one physical side border.
///
/// Specified serialization charges one aggregate node plus each present
/// width, style, and color under one cumulative resource budget.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorder {
    width: Option<CssLineWidth>,
    style: Option<CssLineStyle>,
    color: Option<CssColor>,
}
impl CssBorder {
    /// Returns the components, or `None` when width, style and color are all omitted.
    #[must_use]
    pub fn try_new(
        width: Option<CssLineWidth>,
        style: Option<CssLineStyle>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        (width.is_some() || style.is_some() || color.is_some()).then_some(Self {
            width,
            style,
            color,
        })
    }
    /// Returns the authored width, or `None` when omitted.
    pub const fn width(&self) -> Option<&CssLineWidth> {
        self.width.as_ref()
    }
    /// Returns the authored line style, or `None` when omitted.
    pub const fn style(&self) -> Option<CssLineStyle> {
        self.style
    }
    /// Borrows the authored color without resolving it.
    pub const fn color(&self) -> Option<&CssColor> {
        self.color.as_ref()
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBorder {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<()> {
        let context = &mut writer.context;
        context.charge_nodes(1)?;
        if let Some(width) = &self.width {
            writer.source_group_member(0, 3, |writer| {
                width.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        if let Some(style) = self.style {
            writer.source_group_member(1, 3, |writer| {
                if self.width.is_some() {
                    writer.append(" ")?;
                }
                style.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        if let Some(color) = &self.color {
            writer.source_group_member(2, 3, |writer| {
                if self.width.is_some() || self.style.is_some() {
                    writer.append(" ")?;
                }
                color.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBorder);

#[cfg(test)]
mod cumulative_bridge_contract {
    use super::*;
    use crate::component::CssComponentValue;
    use crate::serialization::CssSerializationLimits as Limits;
    use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
    use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::values::common::{CssBoxSideKind, CssFourSides, CssSidePair};

    fn composition(
        expected: &str,
        nodes: usize,
        append: impl Fn(&mut SpecifiedRuleWriter) -> Result<()>,
    ) {
        let mut writer =
            SpecifiedRuleWriter::new(Limits::new(2 * nodes, 2 * nodes, 2 * expected.len() + 2));
        writer.append("!").unwrap();
        append(&mut writer).unwrap();
        writer.append(";").unwrap();
        append(&mut writer).unwrap();
        assert_eq!(writer.css, format!("!{expected};{expected}"));
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
        let mut suppressed = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
        suppressed.without_output(|writer| append(writer)).unwrap();
        assert!(suppressed.css.is_empty());
        assert!(!suppressed.context.output_suppressed());
        assert_eq!(
            suppressed.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            suppressed.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
    }

    #[test]
    fn border_separators_depend_on_this_border_components() {
        for (value, expected, nodes) in [
            (
                CssBorder::try_new(None, Some(CssLineStyle::Solid), None).unwrap(),
                "solid",
                2,
            ),
            (
                CssBorder::try_new(None, None, Some(CssColor::current_color())).unwrap(),
                "currentcolor",
                2,
            ),
            (
                CssBorder::try_new(
                    None,
                    Some(CssLineStyle::Solid),
                    Some(CssColor::current_color()),
                )
                .unwrap(),
                "solid currentcolor",
                3,
            ),
            (
                CssBorder::try_new(
                    Some(CssLineWidth::Thin),
                    Some(CssLineStyle::Solid),
                    Some(CssColor::current_color()),
                )
                .unwrap(),
                "thin solid currentcolor",
                4,
            ),
        ] {
            assert_eq!(
                value.serialize_specified(Default::default()).unwrap(),
                expected
            );
            composition(expected, nodes, |writer| {
                value.append_to_rule_writer(writer)
            });
        }
    }

    #[test]
    fn width_pairs_and_shorthands_keep_authored_slots() {
        let pair = CssSidePair::new(CssLineWidth::Thin, Some(CssLineWidth::Thick));
        assert_eq!(
            pair.serialize_specified(Default::default()).unwrap(),
            "thin thick"
        );
        composition("thin thick", 3, |writer| pair.append_to_rule_writer(writer));
        // An equal second value is charged but not written.
        let equal = CssSidePair::new(CssLineWidth::Thin, Some(CssLineWidth::Thin));
        assert_eq!(
            equal.serialize_specified(Default::default()).unwrap(),
            "thin"
        );
        composition("thin", 3, |writer| equal.append_to_rule_writer(writer));
        let single = CssSidePair::new(CssLineWidth::Medium, None);
        composition("medium", 2, |writer| single.append_to_rule_writer(writer));
        let quad = CssFourSides::try_new(
            CssBoxSideKind::Logical,
            vec![CssLineWidth::Thin, CssLineWidth::Thick],
        )
        .unwrap();
        assert_eq!(
            quad.serialize_specified(Default::default()).unwrap(),
            "logical thin thick"
        );
        composition("logical thin thick", 3, |writer| {
            quad.append_to_rule_writer(writer)
        });
    }

    #[test]
    fn suppressed_exact_width_retains_numeric_origin_and_uses_no_byte_scratch() {
        let width = CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension("1e50", "px").unwrap(),
        )
        .unwrap();
        let value = CssSidePair::new(CssLineWidth::Length(width), None);
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 2, 1));
        writer.append("x").unwrap();
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert_eq!(writer.css, "x");
        assert_eq!(value, before);
        assert!(matches!(
            value.first().origin(),
            Some(CssValueOrigin::Programmatic)
        ));
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
    }

    #[test]
    fn suppressed_child_failure_restores_nested_output_mode() {
        let value = CssSidePair::new(CssLineWidth::Thin, Some(CssLineWidth::Thick));
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 100, 1));
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
}
