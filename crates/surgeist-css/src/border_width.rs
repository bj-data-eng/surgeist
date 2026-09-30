//! Exact authored border widths and the physical border triples that contain them.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBorderStyle, CssBoxSideKind, CssColor, CssSpecifiedNonNegativeLength,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits, CssValueOrigin,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn output(
    limits: CssSpecifiedValueSerializationLimits,
    append: impl FnOnce(&mut SpecifiedSerializationContext, &mut String) -> Result<()>,
) -> Result<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let mut text = String::new();
    append(&mut context, &mut text)?;
    Ok(text)
}

fn append_keyword(
    context: &mut SpecifiedSerializationContext,
    output: &mut String,
    keyword: &str,
) -> Result<()> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    context.append(output, keyword)
}

/// One exact authored `<line-width>` before border-style-dependent used-value rules.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum CssBorderWidth {
    Thin,
    Medium,
    Thick,
    Length(CssSpecifiedNonNegativeLength),
}

impl CssBorderWidth {
    pub fn length(&self) -> Option<&CssSpecifiedNonNegativeLength> {
        match self {
            Self::Length(value) => Some(value),
            _ => None,
        }
    }
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.length().map(CssSpecifiedNonNegativeLength::origin)
    }
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        output(limits, |context, text| self.append_specified(context, text))
    }
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Thin => append_keyword(context, output, "thin"),
            Self::Medium => append_keyword(context, output, "medium"),
            Self::Thick => append_keyword(context, output, "thick"),
            Self::Length(value) => {
                let text = value.capture_specified(context)?;
                context.append(output, &text)
            }
        }
    }
}
impl PartialEq for CssBorderWidth {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Thin, Self::Thin)
            | (Self::Medium, Self::Medium)
            | (Self::Thick, Self::Thick) => true,
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}
impl Eq for CssBorderWidth {}

/// One or two authored widths for a flow-relative axis.
///
/// Specified serialization charges one aggregate node plus each authored
/// width, sharing input, projection, and byte limits across the entire pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderWidthPair {
    start: CssBorderWidth,
    authored_end: Option<CssBorderWidth>,
}
impl CssBorderWidthPair {
    #[must_use]
    pub fn new(start: CssBorderWidth, authored_end: Option<CssBorderWidth>) -> Self {
        Self {
            start,
            authored_end,
        }
    }
    pub fn start(&self) -> &CssBorderWidth {
        &self.start
    }
    pub fn authored_end(&self) -> Option<&CssBorderWidth> {
        self.authored_end.as_ref()
    }
    pub fn end(&self) -> &CssBorderWidth {
        self.authored_end.as_ref().unwrap_or(&self.start)
    }
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        output(limits, |context, text| {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            self.start.append_specified(context, text)?;
            if let Some(end) = &self.authored_end {
                context.append(text, " ")?;
                end.append_specified(context, text)?;
            }
            Ok(())
        })
    }
}

/// One to four authored physical or flow-relative border widths.
///
/// `assigned_values()` returns top/right/bottom/left roles in physical mode,
/// or block-start/inline-start/block-end/inline-end roles in logical mode.
/// Specified serialization charges one aggregate node plus each authored
/// width under one cumulative input, projection, and byte budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderWidthShorthand {
    kind: CssBoxSideKind,
    authored_values: Box<[CssBorderWidth]>,
}
impl CssBorderWidthShorthand {
    #[must_use]
    pub fn try_new(kind: CssBoxSideKind, values: Vec<CssBorderWidth>) -> Option<Self> {
        (1..=4).contains(&values.len()).then(|| Self {
            kind,
            authored_values: values.into_boxed_slice(),
        })
    }
    pub fn kind(&self) -> CssBoxSideKind {
        self.kind
    }
    pub fn authored_values(&self) -> &[CssBorderWidth] {
        &self.authored_values
    }
    pub fn assigned_values(&self) -> [&CssBorderWidth; 4] {
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
        output(limits, |context, text| {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            if self.kind == CssBoxSideKind::Logical {
                context.append(text, "logical ")?;
            }
            for (index, width) in self.authored_values.iter().enumerate() {
                if index > 0 {
                    context.append(text, " ")?;
                }
                width.append_specified(context, text)?;
            }
            Ok(())
        })
    }
}

/// Checked authored components of `border` or one physical side border.
///
/// Specified serialization charges one aggregate node plus each present
/// width, style, and color under one cumulative resource budget.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorder {
    width: Option<CssBorderWidth>,
    style: Option<CssBorderStyle>,
    color: Option<CssColor>,
}
impl CssBorder {
    #[must_use]
    pub fn try_new(
        width: Option<CssBorderWidth>,
        style: Option<CssBorderStyle>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        (width.is_some() || style.is_some() || color.is_some()).then_some(Self {
            width,
            style,
            color,
        })
    }
    pub const fn width(&self) -> Option<&CssBorderWidth> {
        self.width.as_ref()
    }
    pub const fn style(&self) -> Option<CssBorderStyle> {
        self.style
    }
    /// Borrows the authored color without resolving it.
    pub const fn color(&self) -> Option<&CssColor> {
        self.color.as_ref()
    }
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        output(limits, |context, text| {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            if let Some(width) = &self.width {
                width.append_specified(context, text)?;
            }
            if let Some(style) = self.style {
                if !text.is_empty() {
                    context.append(text, " ")?;
                }
                style.append_specified(context, text)?;
            }
            if let Some(color) = &self.color {
                if !text.is_empty() {
                    context.append(text, " ")?;
                }
                color.append_specified(context, text)?;
            }
            Ok(())
        })
    }
}
