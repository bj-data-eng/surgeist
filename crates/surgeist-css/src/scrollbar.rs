//! Authored CSS Scrollbars 1 values, before contextual color or UI resolution.

use crate::specified_serialization::{SpecifiedSerializationContext, serialize_keyword_sequence};
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
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        match &self.colors {
            None => context.append(&mut output, "auto")?,
            Some([thumb, track]) => {
                thumb.append_specified(&mut context, &mut output)?;
                context.append(&mut output, " ")?;
                track.append_specified(&mut context, &mut output)?;
            }
        }
        Ok(output)
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
        serialize_keyword_sequence(
            match self {
                Self::Auto => "auto",
                Self::Thin => "thin",
                Self::None => "none",
            },
            limits,
        )
    }
}
