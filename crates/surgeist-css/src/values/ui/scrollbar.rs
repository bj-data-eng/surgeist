//! Authored CSS Scrollbars 1 values, before contextual color or UI resolution.

use crate::color::CssColor;
use crate::values::ui::CssScrollbarWidth;

/// Automatic scrollbar coloring or an exact thumb/track pair of checked colors.
///
/// Colors retain their specified graphs and symbolic context references. The
/// pair cannot omit either role; constructing it does not resolve colors or
/// choose scrollbar contrast, geometry, or platform appearance.
/// CSS Scrollbars 1 makes this inherited, initially `auto`, applicable to
/// scroll containers, and animated by computed value; those contextual and
/// animation operations belong to downstream owners.
///
/// # Specified serialization
///
/// Writes canonical specified CSS while keeping contextual colors symbolic.
#[derive(Clone, Debug, PartialEq)]
pub struct CssScrollbarColor {
    colors: Option<[CssColor; 2]>,
}

impl CssScrollbarColor {
    /// Requests automatic scrollbar coloring, the inherited property's initial value.
    #[must_use]
    pub const fn auto() -> Self {
        Self { colors: None }
    }

    /// Constructs an exact ordered pair of already checked thumb and track colors.
    #[must_use]
    pub const fn new(thumb: CssColor, track: CssColor) -> Self {
        Self {
            colors: Some([thumb, track]),
        }
    }

    /// Returns the authored thumb color, or `None` for `auto`.
    #[must_use]
    pub const fn thumb(&self) -> Option<&CssColor> {
        match &self.colors {
            Some(colors) => Some(&colors[0]),
            None => None,
        }
    }

    /// Returns the authored track color, or `None` for `auto`.
    #[must_use]
    pub const fn track(&self) -> Option<&CssColor> {
        match &self.colors {
            Some(colors) => Some(&colors[1]),
            None => None,
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssScrollbarColor {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        match &self.colors {
            None => context.append(output, "auto")?,
            Some([thumb, track]) => {
                thumb.append_specified(context, output)?;
                context.append(output, " ")?;
                track.append_specified(context, output)?;
            }
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssScrollbarColor);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssScrollbarWidth {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Thin => "thin",
            Self::None => "none",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssScrollbarWidth);
