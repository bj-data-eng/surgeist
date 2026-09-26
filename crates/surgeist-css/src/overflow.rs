//! Checked authored overflow values before writing-mode or computed-value resolution.

use crate::specified_serialization::{SpecifiedSerializationContext, serialize_keyword_sequence};
use crate::{
    CssOverflow, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

impl CssOverflow {
    /// Serializes the canonical authored overflow keyword.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one keyword under exact resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        serialize_keyword_sequence(self.as_css(), limits)
    }

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

/// One or two authored physical overflow axes, without computed-value coupling.
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

    /// Serializes the authored one- or two-axis specified value.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both authored axes under one shared resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(&mut output, self.x.as_css())?;
        if let Some(y) = self.authored_y {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            context.append(&mut output, " ")?;
            context.append(&mut output, y.as_css())?;
        }
        Ok(output)
    }
}
