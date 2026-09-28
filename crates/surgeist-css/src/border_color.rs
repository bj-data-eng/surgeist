//! Authored border colors and physical side expansion.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::syntax::CssColor;
use crate::{
    CssBoxSideKind, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
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

/// The four authored side colors expanded from a `border-color` shorthand.
///
/// Colors retain their specified branches, including symbolic `currentcolor`;
/// constructing this value does not resolve them against an element's style.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderColors {
    sides: [CssColor; 4],
}

impl CssBorderColors {
    /// Expands one through four checked colors in top, right, bottom, left order.
    ///
    /// One color applies to all sides. Two set the vertical and horizontal
    /// sides. Three set the top, horizontal sides, and bottom. Empty inputs and
    /// inputs with more than four colors are rejected.
    #[must_use]
    pub fn try_new(colors: Vec<CssColor>) -> Option<Self> {
        let sides = match colors.as_slice() {
            [all] => [all.clone(), all.clone(), all.clone(), all.clone()],
            [vertical, horizontal] => [
                vertical.clone(),
                horizontal.clone(),
                vertical.clone(),
                horizontal.clone(),
            ],
            [top, horizontal, bottom] => [
                top.clone(),
                horizontal.clone(),
                bottom.clone(),
                horizontal.clone(),
            ],
            [top, right, bottom, left] => {
                [top.clone(), right.clone(), bottom.clone(), left.clone()]
            }
            _ => return None,
        };
        Some(Self { sides })
    }

    #[must_use]
    pub const fn top(&self) -> &CssColor {
        &self.sides[0]
    }

    #[must_use]
    pub const fn right(&self) -> &CssColor {
        &self.sides[1]
    }

    #[must_use]
    pub const fn bottom(&self) -> &CssColor {
        &self.sides[2]
    }

    #[must_use]
    pub const fn left(&self) -> &CssColor {
        &self.sides[3]
    }
}

/// One or two authored flow-relative border colors.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderColorPair {
    start: CssColor,
    authored_end: Option<CssColor>,
}

impl CssBorderColorPair {
    #[must_use]
    pub const fn new(start: CssColor, authored_end: Option<CssColor>) -> Self {
        Self {
            start,
            authored_end,
        }
    }

    pub const fn start(&self) -> &CssColor {
        &self.start
    }

    pub const fn authored_end(&self) -> Option<&CssColor> {
        self.authored_end.as_ref()
    }

    pub fn end(&self) -> &CssColor {
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

/// One to four authored physical or flow-relative border colors.
///
/// Assignments are top/right/bottom/left in physical mode and
/// block-start/inline-start/block-end/inline-end in logical mode. These are
/// authored roles, not a complete shorthand expansion with settled resets.
/// Colors remain symbolic, including `currentcolor`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderColorShorthand {
    kind: CssBoxSideKind,
    authored_values: Box<[CssColor]>,
}

impl CssBorderColorShorthand {
    #[must_use]
    pub fn try_new(kind: CssBoxSideKind, values: Vec<CssColor>) -> Option<Self> {
        (1..=4).contains(&values.len()).then(|| Self {
            kind,
            authored_values: values.into_boxed_slice(),
        })
    }

    pub const fn kind(&self) -> CssBoxSideKind {
        self.kind
    }

    pub fn authored_values(&self) -> &[CssColor] {
        &self.authored_values
    }

    pub fn assigned_values(&self) -> [&CssColor; 4] {
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
            for (index, color) in self.authored_values.iter().enumerate() {
                if index > 0 {
                    context.append(output, " ")?;
                }
                color.append_specified(context, output)?;
            }
            Ok(())
        })
    }
}
