//! Checked authored overflow values before writing-mode or computed-value resolution.

use crate::CssOverflow;

impl CssOverflow {
    pub(crate) const fn as_css(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Clip => "clip",
            Self::Scroll => "scroll",
            Self::Auto => "auto",
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssOverflow {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(self.as_css())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOverflow);

/// One or two authored physical overflow axes, without computed-value coupling.
///
/// # Specified serialization
///
/// Serializes the authored one- or two-axis specified value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssOverflowValue {
    x: CssOverflow,
    authored_y: Option<CssOverflow>,
}

impl CssOverflowValue {
    /// Retains the exact authored axis count and order.
    #[must_use]
    pub const fn new(x: CssOverflow, authored_y: Option<CssOverflow>) -> Self {
        Self { x, authored_y }
    }

    /// Returns the authored x-axis keyword.
    #[must_use]
    pub const fn x(&self) -> CssOverflow {
        self.x
    }

    /// Returns the explicitly authored y-axis keyword, if present.
    #[must_use]
    pub const fn authored_y(&self) -> Option<CssOverflow> {
        self.authored_y
    }

    /// Returns the y-axis keyword, repeating x when only one was authored.
    #[must_use]
    pub const fn y(&self) -> CssOverflow {
        match self.authored_y {
            Some(value) => value,
            None => self.x,
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssOverflowValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        context.charge_nodes(1)?;
        writer.source_member(0, |writer| writer.keyword(self.x.as_css()))?;
        if let Some(y) = self.authored_y {
            writer.source_member(1, |writer| {
                writer.node()?;
                writer.append(" ")?;
                writer.append(y.as_css())
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOverflowValue);
