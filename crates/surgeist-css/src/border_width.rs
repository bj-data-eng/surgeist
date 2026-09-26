//! Exact authored border widths and the physical border triples that contain them.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssAuthoredColor, CssBorder, CssBorderStyle, CssBoxSideKind, CssCalcLength,
    CssComponentValueRef, CssLength, CssLengthPercentageCalculation, CssLengthUnit,
    CssSpecifiedNonNegativeLength, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
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

pub(crate) fn legacy_width(value: &CssBorderWidth) -> Option<CssLength> {
    match value {
        CssBorderWidth::Thin => Some(CssLength::Thin),
        CssBorderWidth::Medium => Some(CssLength::Medium),
        CssBorderWidth::Thick => Some(CssLength::Thick),
        CssBorderWidth::Length(length) => {
            if let Some(calculation) = length.calculation() {
                let legacy = CssLengthPercentageCalculation::from_expression(
                    calculation.expression.as_ref().clone(),
                );
                return Some(CssLength::Calc(CssCalcLength::Typed(legacy)));
            }
            match length.literal_component()?.view() {
                CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
                    (crate::opacity_scalar::exact_legacy_value(number.representation())? == 0.0)
                        .then_some(CssLength::Zero)
                }
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                    CssLength::try_dimension(
                        crate::opacity_scalar::exact_legacy_value(number.representation())?,
                        CssLengthUnit::from_css_unit(unit)?,
                    )
                }
                _ => None,
            }
        }
    }
}

pub(crate) fn legacy_shorthand(value: &CssBorderWidthShorthand) -> Option<crate::CssEdges> {
    if value.kind() == CssBoxSideKind::Logical {
        return None;
    }
    let [top, right, bottom, left] = value.assigned_values();
    Some(crate::CssEdges::new(
        legacy_width(top)?,
        legacy_width(right)?,
        legacy_width(bottom)?,
        legacy_width(left)?,
    ))
}

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
pub struct CssBorderValue {
    width: Option<CssBorderWidth>,
    style: Option<CssBorderStyle>,
    color: Option<CssAuthoredColor>,
}
impl CssBorderValue {
    #[must_use]
    pub fn try_new(
        width: Option<CssBorderWidth>,
        style: Option<CssBorderStyle>,
        color: Option<CssAuthoredColor>,
    ) -> Option<Self> {
        (width.is_some() || style.is_some() || color.is_some()).then_some(Self {
            width,
            style,
            color,
        })
    }
    pub fn width(&self) -> Option<&CssBorderWidth> {
        self.width.as_ref()
    }
    pub fn style(&self) -> Option<CssBorderStyle> {
        self.style
    }
    /// Borrows the exact authored color. Unlike `CssBorder::color`, this is not an I01 projection.
    pub fn color(&self) -> Option<&CssAuthoredColor> {
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
                append_keyword(
                    context,
                    text,
                    match style {
                        CssBorderStyle::None => "none",
                        CssBorderStyle::Hidden => "hidden",
                        CssBorderStyle::Dotted => "dotted",
                        CssBorderStyle::Dashed => "dashed",
                        CssBorderStyle::Solid => "solid",
                        CssBorderStyle::Double => "double",
                        CssBorderStyle::Groove => "groove",
                        CssBorderStyle::Ridge => "ridge",
                        CssBorderStyle::Inset => "inset",
                        CssBorderStyle::Outset => "outset",
                    },
                )?;
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

pub(crate) struct CssParsedBorderValue {
    current: CssBorderValue,
    legacy: Option<CssBorder>,
}
impl CssParsedBorderValue {
    pub(crate) fn new(
        width: Option<CssBorderWidth>,
        style: Option<CssBorderStyle>,
        color: Option<crate::syntax::CssParsedColor>,
    ) -> Self {
        let legacy_width = match &width {
            Some(value) => legacy_width(value).map(Some),
            None => Some(None),
        };
        let legacy = if color
            .as_ref()
            .is_none_or(|value| value.i01_subset().is_some())
        {
            legacy_width.map(|width| CssBorder::new_current(width, style, color.clone()))
        } else {
            None
        };
        let current =
            CssBorderValue::try_new(width, style, color.map(|value| value.into_parts().0))
                .expect("parsed nonempty border");
        Self { current, legacy }
    }
    pub(crate) fn into_parts(self) -> (CssBorderValue, Option<CssBorder>) {
        (self.current, self.legacy)
    }
}
