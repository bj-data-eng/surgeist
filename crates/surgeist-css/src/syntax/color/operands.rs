//! Borrowed headers and scalars from checked authored colors.
//!
//! These views perform fixed header observations only. Text, array elements,
//! calculation headers and original literal components remain separate caller
//! admissions; no values are lowered or reconstructed here.

use super::*;
use crate::numeric::color_operands::{Node, node};
use crate::{
    CssNumericTokenRef, CssNumericUnit, CssProfileColorExpression, CssProfileColorExpressionRef,
    CssValueOrigin,
};
use std::mem::size_of;

#[derive(Clone, Copy)]
pub(crate) struct ColorHeader<'a> {
    pub(crate) node: ColorNode<'a>,
    pub(crate) closed: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum ColorNode<'a> {
    CurrentColor,
    Transparent,
    Hex(&'a CssHexColor),
    Named(&'a CssNamedColor),
    System(CssSystemColor),
    Rgb(&'a CssRgbColor),
    Hsl(&'a CssHslColor),
    Hwb(&'a CssHwbColor),
    Lab(&'a CssLabColor),
    Lch(&'a CssLchColor),
    Oklab(&'a CssLabColor),
    Oklch(&'a CssLchColor),
    Predefined(&'a CssPredefinedColor),
    Custom(&'a CssCustomColor),
    RelativeCustom(&'a CssRelativeCustomColor),
    Alpha(&'a CssAlphaColor),
    Relative(&'a CssRelativeColor),
    ColorMix(&'a CssColorMix),
    LightDark(&'a CssLightDarkColor),
    ContrastColor(&'a CssContrastColor),
    DeviceCmyk(&'a CssDeviceCmykColor),
}

pub(crate) fn header(value: &CssColor) -> ColorHeader<'_> {
    let node = match &value.representation {
        CssColorRepresentation::CurrentColor => ColorNode::CurrentColor,
        CssColorRepresentation::Transparent => ColorNode::Transparent,
        CssColorRepresentation::Hex(value) => ColorNode::Hex(value),
        CssColorRepresentation::Named(value) => ColorNode::Named(value),
        CssColorRepresentation::System(value) => ColorNode::System(*value),
        CssColorRepresentation::Rgb(value) => ColorNode::Rgb(value),
        CssColorRepresentation::Hsl(value) => ColorNode::Hsl(value),
        CssColorRepresentation::Hwb(value) => ColorNode::Hwb(value),
        CssColorRepresentation::Lab(value) => ColorNode::Lab(value),
        CssColorRepresentation::Lch(value) => ColorNode::Lch(value),
        CssColorRepresentation::Oklab(value) => ColorNode::Oklab(value),
        CssColorRepresentation::Oklch(value) => ColorNode::Oklch(value),
        CssColorRepresentation::Predefined(value) => ColorNode::Predefined(value),
        CssColorRepresentation::Custom(value) => ColorNode::Custom(value),
        CssColorRepresentation::RelativeCustom(value) => ColorNode::RelativeCustom(value),
        CssColorRepresentation::Alpha(value) => ColorNode::Alpha(value),
        CssColorRepresentation::Relative(value) => ColorNode::Relative(value),
        CssColorRepresentation::ColorMix(value) => ColorNode::ColorMix(value),
        CssColorRepresentation::LightDark(value) => ColorNode::LightDark(value),
        CssColorRepresentation::ContrastColor(value) => ColorNode::ContrastColor(value),
        CssColorRepresentation::DeviceCmyk(value) => ColorNode::DeviceCmyk(value),
    };
    ColorHeader {
        node,
        closed: value.original_closed,
    }
}

pub(crate) const fn header_bytes() -> usize {
    size_of::<CssColor>()
}

impl ColorNode<'_> {
    /// Size of the actual selected local payload; no array/string is visited.
    pub(crate) fn local_bytes(self) -> usize {
        match self {
            Self::CurrentColor | Self::Transparent => 0,
            Self::Hex(_) => size_of::<CssHexColor>(),
            Self::Named(_) => size_of::<CssNamedColor>(),
            Self::System(_) => size_of::<CssSystemColor>(),
            Self::Rgb(_) => size_of::<CssRgbColor>(),
            Self::Hsl(_) => size_of::<CssHslColor>(),
            Self::Hwb(_) => size_of::<CssHwbColor>(),
            Self::Lab(_) | Self::Oklab(_) => size_of::<CssLabColor>(),
            Self::Lch(_) | Self::Oklch(_) => size_of::<CssLchColor>(),
            Self::Predefined(_) => size_of::<CssPredefinedColor>(),
            Self::Custom(_) => size_of::<CssCustomColor>(),
            Self::RelativeCustom(_) => size_of::<CssRelativeCustomColor>(),
            Self::Alpha(_) => size_of::<CssAlphaColor>(),
            Self::Relative(_) => size_of::<CssRelativeColor>(),
            Self::ColorMix(_) => size_of::<CssColorMix>(),
            Self::LightDark(_) => size_of::<CssLightDarkColor>(),
            Self::ContrastColor(_) => size_of::<CssContrastColor>(),
            Self::DeviceCmyk(_) => size_of::<CssDeviceCmykColor>(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Scalar<'a> {
    Missing {
        origin: Option<&'a CssValueOrigin>,
    },
    Literal {
        numeric: CssNumericTokenRef<'a>,
        unit: Option<CssNumericUnit>,
        percentage: bool,
        origin: &'a CssValueOrigin,
    },
    Variable {
        channel: CssRelativeColorChannel,
        origin: &'a CssValueOrigin,
    },
    ProfileReference {
        name: &'a CssColorProfileComponentName,
        origin: &'a CssValueOrigin,
    },
    Expression(Node<'a>),
}

fn number(value: &CssColorNumberLiteral) -> Scalar<'_> {
    Scalar::Literal {
        numeric: value.numeric(),
        unit: None,
        percentage: false,
        origin: value.origin(),
    }
}
fn percentage(value: &CssColorPercentageLiteral) -> Scalar<'_> {
    Scalar::Literal {
        numeric: value.numeric(),
        unit: None,
        percentage: true,
        origin: value.origin(),
    }
}
fn angle(value: &CssAngleLiteral) -> Scalar<'_> {
    Scalar::Literal {
        numeric: value.numeric(),
        unit: Some(CssNumericUnit::Angle(value.unit())),
        percentage: false,
        origin: value.origin(),
    }
}

pub(crate) fn component(value: &CssColorComponent) -> Scalar<'_> {
    match value {
        CssColorComponent::None => Scalar::Missing { origin: None },
        CssColorComponent::Number(value) => number(value),
        CssColorComponent::Percentage(value) => percentage(value),
        CssColorComponent::NumberCalculation(value) => Scalar::Expression(node(&value.expression)),
        CssColorComponent::HintedNumberCalculation(value) => {
            Scalar::Expression(node(&value.expression))
        }
        CssColorComponent::PercentageCalculation(value) => {
            Scalar::Expression(node(&value.expression))
        }
    }
}

pub(crate) fn hue(value: &CssColorHue) -> Scalar<'_> {
    match value {
        CssColorHue::None => Scalar::Missing { origin: None },
        CssColorHue::Number(value) => number(value),
        CssColorHue::Angle(value) => angle(value),
        CssColorHue::NumberCalculation(value) => Scalar::Expression(node(&value.expression)),
        CssColorHue::AngleCalculation(value) => Scalar::Expression(node(&value.expression)),
    }
}

pub(crate) fn relative(value: &CssRelativeColorExpression) -> Scalar<'_> {
    match &value.value {
        CssRelativeColorExpressionValue::None => Scalar::Missing {
            origin: Some(&value.origin),
        },
        CssRelativeColorExpressionValue::Number(value) => number(value),
        CssRelativeColorExpressionValue::Percentage(value) => percentage(value),
        CssRelativeColorExpressionValue::Angle(value) => angle(value),
        CssRelativeColorExpressionValue::Channel(channel) => Scalar::Variable {
            channel: *channel,
            origin: &value.origin,
        },
        CssRelativeColorExpressionValue::Calculation(value) => {
            Scalar::Expression(node(&value.data.expression))
        }
    }
}

pub(crate) fn profile(value: &CssProfileColorExpression) -> Scalar<'_> {
    match value.view() {
        CssProfileColorExpressionRef::Literal(literal) => match component(literal) {
            Scalar::Missing { .. } => Scalar::Missing {
                origin: Some(value.origin()),
            },
            scalar => scalar,
        },
        CssProfileColorExpressionRef::Reference(name) => Scalar::ProfileReference {
            name,
            origin: value.origin(),
        },
        CssProfileColorExpressionRef::Calculation(value) => {
            Scalar::Expression(node(value.checked_expression()))
        }
    }
}

pub(crate) fn weight(value: &CssColorMixWeight) -> Scalar<'_> {
    match &value.0 {
        ColorMixWeight::Literal(value) => percentage(value.literal()),
        ColorMixWeight::Calculation(value) => Scalar::Expression(node(&value.expression)),
    }
}

// Scalar discriminants/local headers have already been admitted before these
// layout getters are selected. Literal extraction reads the boxed original
// component record as well. Calculation extraction only lends its raw pointer;
// its separately admitted numeric header is not observed here.
pub(crate) fn component_bytes(value: &CssColorComponent) -> usize {
    size_of::<CssColorComponent>()
        + match value {
            CssColorComponent::Number(_) | CssColorComponent::Percentage(_) => {
                size_of::<crate::CssComponentValue>()
            }
            _ => 0,
        }
}
pub(crate) fn hue_bytes(value: &CssColorHue) -> usize {
    size_of::<CssColorHue>()
        + match value {
            CssColorHue::Number(_) | CssColorHue::Angle(_) => size_of::<crate::CssComponentValue>(),
            _ => 0,
        }
}
pub(crate) fn relative_bytes(value: &CssRelativeColorExpression) -> usize {
    size_of::<CssRelativeColorExpression>()
        + match &value.value {
            CssRelativeColorExpressionValue::Number(_)
            | CssRelativeColorExpressionValue::Percentage(_)
            | CssRelativeColorExpressionValue::Angle(_) => size_of::<crate::CssComponentValue>(),
            _ => 0,
        }
}
pub(crate) fn profile_bytes(value: &CssProfileColorExpression) -> usize {
    size_of::<CssProfileColorExpression>()
        + match value.view() {
            CssProfileColorExpressionRef::Literal(
                CssColorComponent::Number(_) | CssColorComponent::Percentage(_),
            ) => size_of::<crate::CssComponentValue>(),
            _ => 0,
        }
}
pub(crate) fn weight_bytes(value: &CssColorMixWeight) -> usize {
    size_of::<CssColorMixWeight>()
        + match &value.0 {
            ColorMixWeight::Literal(_) => size_of::<crate::CssComponentValue>(),
            ColorMixWeight::Calculation(_) => 0,
        }
}
