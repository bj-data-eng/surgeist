//! Authored CSS Scrollbars 1 values, before contextual color or UI resolution.

use crate::{
    CssColor, CssScrollbarWidth, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

/// Automatic scrollbar coloring or an exact thumb/track pair of checked colors.
///
/// Colors retain their specified graphs and symbolic context references. The
/// pair cannot omit either role; constructing it does not resolve colors or
/// choose scrollbar contrast, geometry, or platform appearance.
/// CSS Scrollbars 1 makes this inherited, initially `auto`, applicable to
/// scroll containers, and animated by computed value; those contextual and
/// animation operations belong to downstream owners.
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

    /// Writes canonical specified CSS while keeping contextual colors symbolic.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Writes atomically under one cumulative root-and-colors resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_input(1)?;
        context.charge_projection(1)?;
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

impl CssScrollbarWidth {
    /// Writes the specified keyword without resolving scrollbar geometry.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Writes one keyword under the shared specified-value resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Thin => "thin",
            Self::None => "none",
        })
    }
}
