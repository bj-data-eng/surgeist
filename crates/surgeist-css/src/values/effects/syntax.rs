use crate::numeric::CssAngleOrZero;
use crate::numeric::CssAngleUnit;
use crate::numeric::{
    CssAngle, CssCalculation, CssHintedNumber, CssLength, CssNonNegative, CssNumber, CssNumeric,
    CssNumericLiteral, CssPercentage,
};
use crate::values::background::{CssBackgroundRepeat, CssBackgroundSize};
use crate::values::common::CssAutoOr;
use crate::values::image::CssImageValue;

/// An exact ordinary scalar or a domain-checked symbolic opacity calculation.
///
/// # Specified serialization
///
/// Produces canonical specified opacity, without computed-value clamping. Ordinary input remains
/// exact; emitted scalar text rounds to six fractional places, nearest with ties away from zero,
/// after percentage conversion. Finite calculation text rounds the actual binary64 result with the
/// same six-place policy. Arithmetic uses the implementation-defined binary64 precision and range
/// policy described in the crate's numeric reference.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssOpacityValue {
    Scalar(crate::values::effects::CssOpacityScalar),
    NumberCalculation(CssCalculation<CssNumber>),
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
    PercentageCalculation(CssCalculation<CssPercentage>),
}

/// The authored `isolation` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssIsolation {
    Auto,
    Isolate,
}

/// The four authored edges of a deprecated Masking 1 clipping rectangle.
///
/// # Specified serialization
///
/// Serializes four effective edges in top, right, bottom, left order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssClipRect {
    top: CssAutoOr<CssNumeric<CssLength>>,
    right: CssAutoOr<CssNumeric<CssLength>>,
    bottom: CssAutoOr<CssNumeric<CssLength>>,
    left: CssAutoOr<CssNumeric<CssLength>>,
}

impl CssClipRect {
    #[must_use]
    pub const fn new(
        top: CssAutoOr<CssNumeric<CssLength>>,
        right: CssAutoOr<CssNumeric<CssLength>>,
        bottom: CssAutoOr<CssNumeric<CssLength>>,
        left: CssAutoOr<CssNumeric<CssLength>>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn top(&self) -> &CssAutoOr<CssNumeric<CssLength>> {
        &self.top
    }

    #[must_use]
    pub const fn right(&self) -> &CssAutoOr<CssNumeric<CssLength>> {
        &self.right
    }

    #[must_use]
    pub const fn bottom(&self) -> &CssAutoOr<CssNumeric<CssLength>> {
        &self.bottom
    }

    #[must_use]
    pub const fn left(&self) -> &CssAutoOr<CssNumeric<CssLength>> {
        &self.left
    }
}

/// The authored deprecated Masking 1 `clip` value.
///
/// # Specified serialization
///
/// Serializes `auto` or a checked rectangle without applying clipping.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClip {
    Auto,
    Rect(CssClipRect),
}

/// # Specified serialization
///
/// Serializes the authored shadow, retaining omitted components and symbolic values.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssBoxShadow {
    None,
    Shadows(CssBoxShadowList),
}

/// A nonempty ordered list of authored box shadows.
///
/// # Specified serialization
///
/// Serializes the authored shadow, retaining omitted components and symbolic values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBoxShadowList {
    shadows: Vec<CssShadow>,
}

impl CssBoxShadowList {
    /// Rejects an empty list; retains the authored member order.
    pub fn try_new(shadows: Vec<CssShadow>) -> Option<Self> {
        if shadows.is_empty() {
            None
        } else {
            Some(Self { shadows })
        }
    }

    #[must_use]
    pub fn shadows(&self) -> &[CssShadow] {
        &self.shadows
    }
}

/// An authored box shadow with optional blur, spread, color, and inset.
///
/// # Specified serialization
///
/// Serializes the authored shadow, retaining omitted components and symbolic values.
#[derive(Clone, Debug)]
pub struct CssShadow {
    inset: bool,
    offset_x: CssNumeric<CssLength>,
    offset_y: CssNumeric<CssLength>,
    blur_radius: Option<CssNumeric<CssLength, CssNonNegative>>,
    spread_radius: Option<CssNumeric<CssLength>>,
    color: Option<Box<CssColor>>,
}

impl PartialEq for CssShadow {
    fn eq(&self, other: &Self) -> bool {
        self.inset == other.inset
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && match (&self.blur_radius, &other.blur_radius) {
                (Some(left), Some(right)) => left == right,
                (None, None) => true,
                _ => false,
            }
            && match (&self.spread_radius, &other.spread_radius) {
                (Some(left), Some(right)) => left == right,
                (None, None) => true,
                _ => false,
            }
            && self.color == other.color
    }
}

impl CssShadow {
    /// Retains authored omissions and rejects a spread without a blur radius.
    pub fn try_new(
        inset: bool,
        offset_x: CssNumeric<CssLength>,
        offset_y: CssNumeric<CssLength>,
        blur_radius: Option<CssNumeric<CssLength, CssNonNegative>>,
        spread_radius: Option<CssNumeric<CssLength>>,
        color: Option<CssColor>,
    ) -> Option<Self> {
        if blur_radius.is_none() && spread_radius.is_some() {
            return None;
        }
        Some(Self::new(
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color,
        ))
    }
    pub(crate) fn new(
        inset: bool,
        offset_x: CssNumeric<CssLength>,
        offset_y: CssNumeric<CssLength>,
        blur_radius: Option<CssNumeric<CssLength, CssNonNegative>>,
        spread_radius: Option<CssNumeric<CssLength>>,
        color: Option<CssColor>,
    ) -> Self {
        Self {
            inset,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color: color.map(Box::new),
        }
    }
    pub const fn inset(&self) -> bool {
        self.inset
    }
    pub const fn offset_x(&self) -> &CssNumeric<CssLength> {
        &self.offset_x
    }
    pub const fn offset_y(&self) -> &CssNumeric<CssLength> {
        &self.offset_y
    }
    pub const fn blur_radius(&self) -> Option<&CssNumeric<CssLength, CssNonNegative>> {
        self.blur_radius.as_ref()
    }
    pub const fn spread_radius(&self) -> Option<&CssNumeric<CssLength>> {
        self.spread_radius.as_ref()
    }
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }
}
use crate::values::common::CssUrl;
use crate::values::position::CssPosition;

/// Authored color-space selection for SVG filter primitives.
///
/// `Auto` remains distinct from either explicit color space. Contextual selection,
/// filter execution and animation belong downstream; this property does not
/// change the color space of CSS filter functions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorInterpolationFilters {
    Auto,
    Srgb,
    #[default]
    LinearRgb,
}

/// The optional authored amount accepted by a filter amount function.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssFilterAmount {
    Default,
    Number(CssNumeric<CssNumber, CssNonNegative>),
    /// A Number with an unresolved percentage basis; no filter range is applied here.
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
    Percentage(CssNumeric<CssPercentage, CssNonNegative>),
}

impl PartialEq for CssFilterAmount {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Default, Self::Default) => true,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left == right
            }
            (Self::Percentage(left), Self::Percentage(right)) => left == right,
            _ => false,
        }
    }
}

/// A checked specified filter blur length, including the omitted radius's 0px default.
///
/// [`Self::authored_length`] preserves whether the radius was omitted;
/// [`Self::length`] borrows its effective specified value.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterBlur {
    length: CssNumeric<CssLength, CssNonNegative>,
    omitted: bool,
}

impl CssFilterBlur {
    /// Retains an explicitly authored checked radius.
    pub const fn new(length: CssNumeric<CssLength, CssNonNegative>) -> Self {
        Self {
            length,
            omitted: false,
        }
    }
    /// The omitted radius has an effective specified value of 0px.
    pub fn omitted() -> Self {
        Self {
            length: CssNumeric::<CssLength, CssNonNegative>::try_from_component(
                crate::component::CssComponentValue::try_token("0px").expect("zero pixels token"),
            )
            .expect("zero pixels length"),
            omitted: true,
        }
    }
    /// Borrows the effective nonnegative radius, including an omitted radius's 0px.
    pub const fn length(&self) -> &CssNumeric<CssLength, CssNonNegative> {
        &self.length
    }
    /// Borrows only an explicitly authored radius.
    pub const fn authored_length(&self) -> Option<&CssNumeric<CssLength, CssNonNegative>> {
        if self.omitted {
            None
        } else {
            Some(&self.length)
        }
    }
}

/// A specified hue rotation, retaining omission separately from its 0deg default.
///
/// [`Self::angle`] borrows the effective specified angle; [`Self::authored_angle`]
/// borrows only an explicitly authored angle. Literal units, unitless zero, and
/// calculation provenance keep the shared [`CssAngleOrZero`] contract.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterHueRotate {
    angle: CssAngleOrZero,
    omitted: bool,
}

impl CssFilterHueRotate {
    /// Retains an explicitly authored checked angle without normalizing it.
    pub const fn new(angle: CssAngleOrZero) -> Self {
        Self {
            angle,
            omitted: false,
        }
    }

    /// An omitted angle has the effective specified value 0deg.
    pub fn omitted() -> Self {
        Self {
            angle: CssAngleOrZero::Angle(CssNumeric::<CssAngle>::from_literal(
                CssNumericLiteral::<CssAngle>::try_new("0", CssAngleUnit::Degrees)
                    .expect("checked zero degrees"),
            )),
            omitted: true,
        }
    }

    /// Borrows the effective specified angle, including an omitted angle's 0deg.
    pub const fn angle(&self) -> &CssAngleOrZero {
        &self.angle
    }

    /// Borrows only an explicitly authored angle.
    pub const fn authored_angle(&self) -> Option<&CssAngleOrZero> {
        if self.omitted {
            None
        } else {
            Some(&self.angle)
        }
    }
}

/// An authored filter `drop-shadow()` with optional nonnegative standard deviation.
/// Unlike a box shadow, it has no inset or spread component.
///
/// # Specified serialization
///
/// Serializes the authored shadow, retaining omitted components and symbolic values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDropShadow {
    offset_x: CssNumeric<CssLength>,
    offset_y: CssNumeric<CssLength>,
    standard_deviation: Option<CssNumeric<CssLength, CssNonNegative>>,
    color: Option<Box<CssColor>>,
}

impl CssDropShadow {
    /// Retains two signed offsets, an optional checked standard deviation, and optional color.
    pub fn new(
        offset_x: CssNumeric<CssLength>,
        offset_y: CssNumeric<CssLength>,
        standard_deviation: Option<CssNumeric<CssLength, CssNonNegative>>,
        color: Option<CssColor>,
    ) -> Self {
        Self {
            offset_x,
            offset_y,
            standard_deviation,
            color: color.map(Box::new),
        }
    }
    pub const fn offset_x(&self) -> &CssNumeric<CssLength> {
        &self.offset_x
    }
    pub const fn offset_y(&self) -> &CssNumeric<CssLength> {
        &self.offset_y
    }
    /// The authored Gaussian standard deviation; omission remains distinct from zero.
    pub const fn standard_deviation(&self) -> Option<&CssNumeric<CssLength, CssNonNegative>> {
        self.standard_deviation.as_ref()
    }
    pub fn color(&self) -> Option<&CssColor> {
        self.color.as_deref()
    }
}

/// A parser-produced authored filter function with an exact typed payload.
///
/// # Specified serialization
///
/// Serializes specified filters in authored order, preserving omitted arguments. Child providers
/// retain exact ordinary magnitudes and symbolic values.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterFunction {
    Blur(CssFilterBlur),
    Brightness(CssFilterAmount),
    Contrast(CssFilterAmount),
    DropShadow(CssDropShadow),
    Grayscale(CssFilterAmount),
    HueRotate(CssFilterHueRotate),
    Invert(CssFilterAmount),
    Opacity(CssFilterAmount),
    Saturate(CssFilterAmount),
    Sepia(CssFilterAmount),
    Url(CssUrl),
}

/// A non-empty ordered list of authored filter functions.
///
/// # Specified serialization
///
/// Serializes specified filters in authored order, preserving omitted arguments. Child providers
/// retain exact ordinary magnitudes and symbolic values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterFunctionList {
    functions: Vec<CssFilterFunction>,
}

impl CssFilterFunctionList {
    #[must_use]
    pub fn try_new(functions: Vec<CssFilterFunction>) -> Option<Self> {
        (!functions.is_empty()).then_some(Self { functions })
    }

    #[must_use]
    pub fn functions(&self) -> &[CssFilterFunction] {
        &self.functions
    }
}

/// The authored value of `filter` or `backdrop-filter`.
///
/// # Specified serialization
///
/// Serializes specified filters in authored order, preserving omitted arguments. Child providers
/// retain exact ordinary magnitudes and symbolic values.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilter {
    None,
    Functions(CssFilterFunctionList),
}

use crate::values::effects::masking::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskLayer {
    pub(crate) image: Option<CssImageValue>,
    pub(crate) position: Option<CssPosition>,
    pub(crate) size: Option<CssBackgroundSize>,
    pub(crate) repeat: Option<CssBackgroundRepeat>,
    pub(crate) boxes: Option<CssMaskLayerBoxes>,
    pub(crate) composite: Option<CssMaskComposite>,
    pub(crate) mode: Option<CssMaskMode>,
}

impl CssMaskLayer {
    /// Constructs a nonempty layer from checked authored children. A size may
    /// omit position; specified output generates position without mutating it.
    #[must_use]
    pub fn try_new(
        image: Option<CssImageValue>,
        position: Option<CssPosition>,
        size: Option<CssBackgroundSize>,
        repeat: Option<CssBackgroundRepeat>,
        boxes: Option<CssMaskLayerBoxes>,
        composite: Option<CssMaskComposite>,
        mode: Option<CssMaskMode>,
    ) -> Option<Self> {
        if image.is_none()
            && position.is_none()
            && size.is_none()
            && repeat.is_none()
            && boxes.is_none()
            && composite.is_none()
            && mode.is_none()
        {
            None
        } else {
            Some(Self {
                image,
                position,
                size,
                repeat,
                boxes,
                composite,
                mode,
            })
        }
    }
    #[must_use]
    pub const fn image(&self) -> Option<&CssImageValue> {
        self.image.as_ref()
    }
    #[must_use]
    pub const fn position(&self) -> Option<&CssPosition> {
        self.position.as_ref()
    }
    #[must_use]
    pub const fn size(&self) -> Option<&CssBackgroundSize> {
        self.size.as_ref()
    }
    #[must_use]
    pub const fn repeat(&self) -> Option<CssBackgroundRepeat> {
        self.repeat
    }
    #[must_use]
    pub const fn boxes(&self) -> Option<CssMaskLayerBoxes> {
        self.boxes
    }
    #[must_use]
    pub const fn composite(&self) -> Option<CssMaskComposite> {
        self.composite
    }
    #[must_use]
    pub const fn mode(&self) -> Option<CssMaskMode> {
        self.mode
    }
}

/// # Specified serialization
///
/// Serializes all eight authored fields in canonical layer order. Children retain their providers'
/// costs. A size without position generates two projection-only `0%` tokens before the slash; it
/// does not add authored input or mutate the layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaskList {
    pub(crate) layers: Vec<CssMaskLayer>,
}

impl CssMaskList {
    #[must_use]
    pub fn try_new(layers: Vec<CssMaskLayer>) -> Option<Self> {
        if layers.is_empty() {
            None
        } else {
            Some(Self::new(layers))
        }
    }

    #[must_use]
    pub(crate) fn new(layers: Vec<CssMaskLayer>) -> Self {
        Self { layers }
    }

    #[must_use]
    pub fn layers(&self) -> &[CssMaskLayer] {
        &self.layers
    }
}
use crate::color::CssColor;
