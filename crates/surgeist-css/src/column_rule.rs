//! Exact authored `column-rule` components and specified shorthand serialization.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssAuthoredColor, CssBorderStyle, CssBorderValue, CssBorderWidth,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

/// A nonempty authored column rule with exact width and color components.
///
/// Omitted shorthand members are reset to their initials during intrinsic
/// expansion. The stored components retain which values were explicitly given.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColumnRule {
    components: CssBorderValue,
}

impl CssColumnRule {
    /// Checks the nonempty `||` shorthand before constructing a value.
    #[must_use]
    pub fn try_new(
        width: Option<CssBorderWidth>,
        style: Option<CssBorderStyle>,
        color: Option<CssAuthoredColor>,
    ) -> Option<Self> {
        CssBorderValue::try_new(width, style, color).map(|components| Self { components })
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssBorderWidth> {
        self.components.width()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssBorderStyle> {
        self.components.style()
    }

    /// Borrows the exact authored color, including symbolic modern colors.
    #[must_use]
    pub const fn color(&self) -> Option<&CssAuthoredColor> {
        self.components.color()
    }

    /// Serializes the specified shorthand in width, style, color order.
    /// Initial-valued members are omitted; an all-initial rule emits `medium`.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one cumulative budget, including explicitly authored initial values
    /// that disappear from the canonical output.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        let emit_width = self
            .width()
            .is_some_and(|width| *width != CssBorderWidth::Medium);
        let emit_style = self
            .style()
            .is_some_and(|style| style != CssBorderStyle::None);
        let emit_color = self.color().is_some_and(|color| !color.is_current_color());
        let all_initial = !(emit_width || emit_style || emit_color);
        if let Some(width) = self.width() {
            if emit_width || all_initial {
                width.append_specified(&mut context, &mut output)?;
            } else {
                context.charge_input(1)?;
                context.charge_projection(1)?;
            }
        } else if all_initial {
            context.charge_projection(1)?;
            context.append(&mut output, "medium")?;
        }
        if let Some(style) = self.style() {
            if emit_style {
                if !output.is_empty() {
                    context.append(&mut output, " ")?;
                }
                style.append_specified(&mut context, &mut output)?;
            } else {
                context.charge_input(1)?;
                context.charge_projection(1)?;
            }
        }
        if let Some(color) = self.color() {
            if emit_color {
                if !output.is_empty() {
                    context.append(&mut output, " ")?;
                }
                color.append_specified(&mut context, &mut output)?;
            } else {
                context.charge_input(1)?;
                context.charge_projection(1)?;
            }
        }
        Ok(output)
    }
}
