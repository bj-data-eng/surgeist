//! Fixed typed borrows selected from an admitted original color payload.

use super::super::grammar::Leaf;
use super::super::keywords::ColorFunction as F;
use super::super::protocol::ColorProtocolError;
use super::CssColorPathStep;
use crate::syntax::color::operands::{self, ColorNode, Scalar};
use crate::*;
use std::mem::size_of;

#[derive(Clone, Copy)]
pub(super) enum Operand<'a> {
    Color(&'a CssColor),
    Component(&'a CssColorComponent),
    Hue(&'a CssColorHue),
    Relative(&'a CssRelativeColorExpression),
    Profile(&'a CssProfileColorExpression),
    Weight(&'a CssColorMixWeight),
    Expression(crate::numeric::color_operands::Node<'a>),
}
impl std::fmt::Debug for Operand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never format a borrowed AST recursively or inspect its text.
        f.write_str(match self {
            Self::Color(_) => "Color",
            Self::Component(_) => "Component",
            Self::Hue(_) => "Hue",
            Self::Relative(_) => "Relative",
            Self::Profile(_) => "Profile",
            Self::Weight(_) => "Weight",
            Self::Expression(_) => "Expression",
        })
    }
}
impl<'a> Operand<'a> {
    pub(super) fn header_bytes(self) -> usize {
        match self {
            Self::Color(_) => operands::header_bytes(),
            Self::Component(_) => size_of::<CssColorComponent>(),
            Self::Hue(_) => size_of::<CssColorHue>(),
            Self::Relative(_) => size_of::<CssRelativeColorExpression>(),
            Self::Profile(_) => size_of::<CssProfileColorExpression>(),
            Self::Weight(_) => size_of::<CssColorMixWeight>(),
            Self::Expression(_) => crate::numeric::color_operands::header_bytes(),
        }
    }
    /// The enum/local wrapper has already been observed. Literal projection's
    /// boxed original component remains a separately admitted payload read.
    pub(super) fn scalar_bytes(self) -> Result<usize, ColorProtocolError> {
        Ok(match self {
            Self::Component(value) => operands::component_bytes(value),
            Self::Hue(value) => operands::hue_bytes(value),
            Self::Relative(value) => operands::relative_bytes(value),
            Self::Profile(value) => operands::profile_bytes(value),
            Self::Weight(value) => operands::weight_bytes(value),
            _ => return Err(ColorProtocolError::WrongSlot),
        })
    }
    pub(super) fn scalar(self) -> Result<Scalar<'a>, ColorProtocolError> {
        Ok(match self {
            Self::Component(value) => operands::component(value),
            Self::Hue(value) => operands::hue(value),
            Self::Relative(value) => operands::relative(value),
            Self::Profile(value) => operands::profile(value),
            Self::Weight(value) => operands::weight(value),
            _ => return Err(ColorProtocolError::WrongSlot),
        })
    }
}

#[derive(Clone, Copy)]
pub(super) enum Shape<'a> {
    Leaf(Leaf),
    Hex(&'a str),
    Function(Fields<'a>),
}
#[derive(Clone, Copy)]
pub(super) struct Fields<'a> {
    pub(super) function: F,
    pub(super) space: Option<surgeist_color::PredefinedColorSpace>,
    pub(super) syntax: CssColorSyntax,
    pub(super) count: usize,
    pub(super) alpha: bool,
    pub(super) source: bool,
    pub(super) children: [bool; 2],
    pub(super) mix_count: usize,
    pub(super) profile: Option<&'a CssColorProfileName>,
    pub(super) interpolation: Option<&'a CssColorInterpolation>,
}
fn fields<'a>(function: F) -> Fields<'a> {
    Fields {
        function,
        space: None,
        syntax: CssColorSyntax::Modern,
        count: 0,
        alpha: false,
        source: false,
        children: [false; 2],
        mix_count: 0,
        profile: None,
        interpolation: None,
    }
}
/// Called only after admitting the selected local payload reported by ColorNode.
/// Array/slice headers are borrowed here; individual elements remain future
/// direct child observations under their own admission.
pub(super) fn shape(node: ColorNode<'_>) -> Shape<'_> {
    let value = match node {
        ColorNode::CurrentColor => return Shape::Leaf(Leaf::Current),
        ColorNode::Transparent => return Shape::Leaf(Leaf::Transparent),
        ColorNode::Hex(v) => return Shape::Hex(v.digits()),
        ColorNode::Named(v) => return Shape::Leaf(Leaf::Named(v.checked_rgb())),
        ColorNode::System(v) => return Shape::Leaf(Leaf::System(v)),
        ColorNode::Rgb(v) => {
            let mut f = fields(F::Rgb);
            f.syntax = v.syntax();
            f.count = v.channels().len();
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Hsl(v) => {
            let mut f = fields(F::Hsl);
            f.syntax = v.syntax();
            f.count = 3;
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Hwb(v) => {
            let mut f = fields(F::Hwb);
            f.count = 3;
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Lab(v) | ColorNode::Oklab(v) => {
            let mut f = fields(if matches!(node, ColorNode::Lab(_)) {
                F::Lab
            } else {
                F::Oklab
            });
            f.count = 3;
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Lch(v) | ColorNode::Oklch(v) => {
            let mut f = fields(if matches!(node, ColorNode::Lch(_)) {
                F::Lch
            } else {
                F::Oklch
            });
            f.count = 3;
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Predefined(v) => {
            let mut f = fields(F::Color);
            f.space = Some(v.color_space());
            f.count = v.channels().len();
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Custom(v) => {
            let mut f = fields(F::Color);
            f.profile = Some(v.profile());
            f.count = v.channels().len();
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::RelativeCustom(v) => {
            let mut f = fields(F::Color);
            f.source = true;
            f.profile = Some(v.profile());
            f.count = v.channels().len();
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Alpha(v) => {
            let mut f = fields(F::Alpha);
            f.source = true;
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::Relative(v) => {
            let (fun, space) = match *v.function() {
                CssRelativeColorFunction::Rgb => (F::Rgb, None),
                CssRelativeColorFunction::Hsl => (F::Hsl, None),
                CssRelativeColorFunction::Hwb => (F::Hwb, None),
                CssRelativeColorFunction::Lab => (F::Lab, None),
                CssRelativeColorFunction::Lch => (F::Lch, None),
                CssRelativeColorFunction::Oklab => (F::Oklab, None),
                CssRelativeColorFunction::Oklch => (F::Oklch, None),
                CssRelativeColorFunction::Color(s) => (F::Color, Some(s)),
            };
            let mut f = fields(fun);
            f.space = space;
            f.source = true;
            f.count = v.channels().len();
            f.alpha = v.alpha().is_some();
            f
        }
        ColorNode::ColorMix(v) => {
            let mut f = fields(F::Mix);
            f.mix_count = v.components().len();
            f.interpolation = v.interpolation();
            f
        }
        ColorNode::LightDark(_) => {
            let mut f = fields(F::LightDark);
            f.children = [true, true];
            f
        }
        ColorNode::ContrastColor(_) => {
            let mut f = fields(F::Contrast);
            f.source = true;
            f
        }
        ColorNode::DeviceCmyk(v) => {
            let mut f = fields(F::DeviceCmyk);
            f.syntax = v.syntax();
            f.count = v.channels().len();
            f.alpha = v.alpha().is_some();
            f
        }
    };
    Shape::Function(value)
}

pub(super) fn same_operand(left: Operand<'_>, right: Operand<'_>) -> bool {
    match (left, right) {
        (Operand::Color(a), Operand::Color(b)) => std::ptr::eq(a, b),
        (Operand::Component(a), Operand::Component(b)) => std::ptr::eq(a, b),
        (Operand::Hue(a), Operand::Hue(b)) => std::ptr::eq(a, b),
        (Operand::Relative(a), Operand::Relative(b)) => std::ptr::eq(a, b),
        (Operand::Profile(a), Operand::Profile(b)) => std::ptr::eq(a, b),
        (Operand::Weight(a), Operand::Weight(b)) => std::ptr::eq(a, b),
        (Operand::Expression(a), Operand::Expression(b)) => {
            crate::numeric::color_operands::same(a, b)
        }
        _ => false,
    }
}

/// Reobserves only the selected original parent's pointer/optional-child record.
/// It never reads a pointed scalar/numeric header or walks a child subtree.
pub(super) fn child(node: ColorNode<'_>, edge: CssColorPathStep) -> Option<Operand<'_>> {
    match edge {
        CssColorPathStep::Source => match node {
            ColorNode::RelativeCustom(v) => Some(Operand::Color(v.source())),
            ColorNode::Alpha(v) => Some(Operand::Color(v.source())),
            ColorNode::Relative(v) => Some(Operand::Color(v.source())),
            ColorNode::ContrastColor(v) => Some(Operand::Color(v.color())),
            _ => None,
        },
        CssColorPathStep::First => match node {
            ColorNode::LightDark(v) => Some(Operand::Color(v.light())),
            _ => None,
        },
        CssColorPathStep::Second => match node {
            ColorNode::LightDark(v) => Some(Operand::Color(v.dark())),
            _ => None,
        },
        CssColorPathStep::MixColor(index) => match node {
            ColorNode::ColorMix(v) => v.components().get(index).map(|v| Operand::Color(v.color())),
            _ => None,
        },
        CssColorPathStep::MixWeight(index) => match node {
            ColorNode::ColorMix(v) => v
                .components()
                .get(index)
                .and_then(|v| v.weight())
                .map(Operand::Weight),
            _ => None,
        },
        CssColorPathStep::Channel(index) => match node {
            ColorNode::Rgb(v) => v.channels().get(index).map(Operand::Component),
            ColorNode::Predefined(v) => v.channels().get(index).map(Operand::Component),
            ColorNode::DeviceCmyk(v) => v.channels().get(index).map(Operand::Component),
            ColorNode::Custom(v) => v.channels().get(index).map(Operand::Component),
            ColorNode::Relative(v) => v.channels().get(index).map(Operand::Relative),
            ColorNode::RelativeCustom(v) => v.channels().get(index).map(Operand::Profile),
            ColorNode::Hsl(v) => match index {
                0 => Some(Operand::Hue(v.hue())),
                1 => Some(Operand::Component(v.saturation())),
                2 => Some(Operand::Component(v.lightness())),
                _ => None,
            },
            ColorNode::Hwb(v) => match index {
                0 => Some(Operand::Hue(v.hue())),
                1 => Some(Operand::Component(v.whiteness())),
                2 => Some(Operand::Component(v.blackness())),
                _ => None,
            },
            ColorNode::Lab(v) | ColorNode::Oklab(v) => match index {
                0 => Some(Operand::Component(v.lightness())),
                1 => Some(Operand::Component(v.a())),
                2 => Some(Operand::Component(v.b())),
                _ => None,
            },
            ColorNode::Lch(v) | ColorNode::Oklch(v) => match index {
                0 => Some(Operand::Component(v.lightness())),
                1 => Some(Operand::Component(v.chroma())),
                2 => Some(Operand::Hue(v.hue())),
                _ => None,
            },
            _ => None,
        },
        CssColorPathStep::Alpha => match node {
            ColorNode::Rgb(v) => v.alpha().map(Operand::Component),
            ColorNode::Hsl(v) => v.alpha().map(Operand::Component),
            ColorNode::Hwb(v) => v.alpha().map(Operand::Component),
            ColorNode::Lab(v) | ColorNode::Oklab(v) => v.alpha().map(Operand::Component),
            ColorNode::Lch(v) | ColorNode::Oklch(v) => v.alpha().map(Operand::Component),
            ColorNode::Predefined(v) => v.alpha().map(Operand::Component),
            ColorNode::Custom(v) => v.alpha().map(Operand::Component),
            ColorNode::RelativeCustom(v) => v.alpha().map(Operand::Profile),
            ColorNode::Alpha(v) => v.alpha().map(Operand::Relative),
            ColorNode::Relative(v) => v.alpha().map(Operand::Relative),
            ColorNode::DeviceCmyk(v) => v.alpha().map(Operand::Component),
            _ => None,
        },
        CssColorPathStep::ExpressionOperand(_) => None,
    }
}
pub(super) fn child_bytes(
    node: ColorNode<'_>,
    edge: CssColorPathStep,
) -> Result<usize, ColorProtocolError> {
    let element = match (node, edge) {
        (
            ColorNode::ColorMix(_),
            CssColorPathStep::MixColor(_) | CssColorPathStep::MixWeight(_),
        ) => size_of::<CssColorMixComponent>(),
        (ColorNode::Custom(_), CssColorPathStep::Channel(_)) => size_of::<CssColorComponent>(),
        (ColorNode::RelativeCustom(_), CssColorPathStep::Channel(_)) => {
            size_of::<CssProfileColorExpression>()
        }
        _ => 0,
    };
    node.local_bytes()
        .checked_add(element)
        .ok_or(ColorProtocolError::Overflow)
}
