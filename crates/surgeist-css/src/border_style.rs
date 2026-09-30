//! Authored border styles before writing-mode or painting resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBorderStyle, CssBoxSideKind, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn serialize(
    limits: CssSpecifiedValueSerializationLimits,
    append: impl FnOnce(&mut SpecifiedSerializationContext, &mut String) -> Result<()>,
) -> Result<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let mut output = String::new();
    append(&mut context, &mut output)?;
    Ok(output)
}

impl CssBorderStyle {
    /// Serializes one specified line-style keyword under the default budget.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one specified line-style keyword under independent budgets.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize(limits, |context, output| {
            self.append_specified(context, output)
        })
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(
            output,
            match self {
                Self::None => "none",
                Self::Hidden => "hidden",
                Self::Dotted => "dotted",
                Self::Dashed => "dashed",
                Self::Solid => "solid",
                Self::Double => "double",
                Self::Groove => "groove",
                Self::Ridge => "ridge",
                Self::Inset => "inset",
                Self::Outset => "outset",
            },
        )
    }
}

/// One or two authored flow-relative border styles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderStylePair {
    start: CssBorderStyle,
    authored_end: Option<CssBorderStyle>,
}

impl CssBorderStylePair {
    #[must_use]
    pub const fn new(start: CssBorderStyle, authored_end: Option<CssBorderStyle>) -> Self {
        Self {
            start,
            authored_end,
        }
    }

    pub const fn start(&self) -> &CssBorderStyle {
        &self.start
    }

    pub const fn authored_end(&self) -> Option<&CssBorderStyle> {
        self.authored_end.as_ref()
    }

    pub fn end(&self) -> &CssBorderStyle {
        self.authored_end.as_ref().unwrap_or(&self.start)
    }

    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize(limits, |context, output| {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            self.start.append_specified(context, output)?;
            if let Some(end) = &self.authored_end {
                context.append(output, " ")?;
                end.append_specified(context, output)?;
            }
            Ok(())
        })
    }
}

/// One to four authored physical or flow-relative border styles.
///
/// Assignments are top/right/bottom/left in physical mode and
/// block-start/inline-start/block-end/inline-end in logical mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderStyleShorthand {
    kind: CssBoxSideKind,
    authored_values: Box<[CssBorderStyle]>,
}

impl CssBorderStyleShorthand {
    #[must_use]
    pub fn try_new(kind: CssBoxSideKind, values: Vec<CssBorderStyle>) -> Option<Self> {
        (1..=4).contains(&values.len()).then(|| Self {
            kind,
            authored_values: values.into_boxed_slice(),
        })
    }

    pub const fn kind(&self) -> CssBoxSideKind {
        self.kind
    }

    pub fn authored_values(&self) -> &[CssBorderStyle] {
        &self.authored_values
    }

    pub fn assigned_values(&self) -> [&CssBorderStyle; 4] {
        let v = &self.authored_values;
        match v.len() {
            1 => [&v[0], &v[0], &v[0], &v[0]],
            2 => [&v[0], &v[1], &v[0], &v[1]],
            3 => [&v[0], &v[1], &v[2], &v[1]],
            4 => [&v[0], &v[1], &v[2], &v[3]],
            _ => unreachable!("checked arity"),
        }
    }

    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize(limits, |context, output| {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            if self.kind == CssBoxSideKind::Logical {
                context.append(output, "logical ")?;
            }
            for (index, style) in self.authored_values.iter().enumerate() {
                if index > 0 {
                    context.append(output, " ")?;
                }
                style.append_specified(context, output)?;
            }
            Ok(())
        })
    }
}
