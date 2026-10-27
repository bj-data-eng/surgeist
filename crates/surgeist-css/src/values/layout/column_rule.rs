//! Exact authored `column-rule` components and specified shorthand serialization.

use crate::color::CssColor;
use crate::values::border::{CssBorder, CssLineStyle, CssLineWidth};

/// A nonempty authored column rule with exact width and color components.
///
/// Omitted shorthand members are reset to their initials during intrinsic
/// expansion. The stored components retain which values were explicitly given.
///
/// # Specified serialization
///
/// Serializes the specified shorthand in width, style, color order. Initial-valued members are
/// omitted; an all-initial rule emits `none`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColumnRule {
    components: CssBorder,
}

impl CssColumnRule {
    /// Checks the nonempty `||` shorthand before constructing a value.
    #[must_use]
    pub fn try_new(
        width: Option<CssLineWidth>,
        style: Option<CssLineStyle>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        CssBorder::try_new(width, style, color).map(|components| Self { components })
    }

    /// Returns the authored rule width, or `None` when it was omitted.
    #[must_use]
    pub const fn width(&self) -> Option<&CssLineWidth> {
        self.components.width()
    }

    /// Returns the authored rule style, or `None` when it was omitted.
    #[must_use]
    pub const fn style(&self) -> Option<CssLineStyle> {
        self.components.style()
    }

    /// Borrows the exact authored color, including symbolic modern colors.
    #[must_use]
    pub const fn color(&self) -> Option<&CssColor> {
        self.components.color()
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssColumnRule {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let start = writer.css.len();
        writer.node()?;
        let emit_width = self
            .width()
            .is_some_and(|width| *width != CssLineWidth::Medium);
        let emit_style = self
            .style()
            .is_some_and(|style| style != CssLineStyle::None);
        let emit_color = self.color().is_some_and(|color| !color.is_current_color());
        let all_initial = !(emit_width || emit_style || emit_color);
        if let Some(width) = self.width() {
            writer.source_member(0, |writer| {
                if emit_width {
                    width.append_specified(&mut writer.context, &mut writer.css)
                } else {
                    writer.node()
                }
            })?;
        }
        // The `<line-width> || <line-style> || <color>` family keeps one
        // member when all are initial; `none` is its shortest representation.
        if let Some(style) = self.style() {
            writer.source_member(1, |writer| {
                if emit_style || all_initial {
                    if writer.css.len() != start {
                        writer.append(" ")?;
                    }
                    style.append_specified(&mut writer.context, &mut writer.css)
                } else {
                    writer.node()
                }
            })?;
        } else if all_initial {
            // A synthesized style charges one projection node and no input node.
            writer.source_member(1, |writer| {
                writer.context.charge_projection(1)?;
                writer.append("none")
            })?;
        }
        if let Some(color) = self.color() {
            writer.source_member(2, |writer| {
                if emit_color {
                    if writer.css.len() != start {
                        writer.append(" ")?;
                    }
                    color.append_specified(&mut writer.context, &mut writer.css)
                } else {
                    writer.node()
                }
            })?;
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssColumnRule);
