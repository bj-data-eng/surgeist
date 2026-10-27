use crate::numeric::{
    CssCalculation, CssHintedNumber, CssLength, CssLengthPercentage, CssNonNegative, CssNumber,
    CssNumeric, CssPercentage,
};
use crate::values::border::CssLineWidth;
use crate::values::image::CssImageValue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLineStyle {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// One non-negative authored `border-image-slice` component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBorderImageSliceComponent {
    Number(CssNumeric<CssNumber, CssNonNegative>),
    /// A Number result retaining a percentage basis; math range handling is deferred.
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
    Percentage(CssNumeric<CssPercentage, CssNonNegative>),
}

impl PartialEq for CssBorderImageSliceComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left == right
            }
            (Self::Percentage(left), Self::Percentage(right)) => left == right,
            _ => false,
        }
    }
}

/// The expanded four-edge authored `border-image-slice` value.
///
/// # Specified serialization
///
/// Serializes canonical specified CSS without resolving contextual values. Every stored edge is
/// visited, even when compressed or omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageSlice {
    values: [CssBorderImageSliceComponent; 4],
    fill: bool,
}

impl CssBorderImageSlice {
    #[must_use]
    pub fn try_new(values: Vec<CssBorderImageSliceComponent>, fill: bool) -> Option<Self> {
        expand_border_image_sides(values).map(|values| Self { values, fill })
    }

    #[must_use]
    pub const fn values(&self) -> &[CssBorderImageSliceComponent; 4] {
        &self.values
    }

    #[must_use]
    pub const fn fill(&self) -> bool {
        self.fill
    }
}

/// One authored `border-image-width` component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBorderImageWidthComponent {
    Auto,
    LengthPercentage(CssNumeric<CssLengthPercentage, CssNonNegative>),
    Number(CssNumeric<CssNumber, CssNonNegative>),
    /// A contextual Number multiplier, before computed-value range handling.
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
}

impl PartialEq for CssBorderImageWidthComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) => true,
            (Self::LengthPercentage(left), Self::LengthPercentage(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::HintedNumberCalculation(left), Self::HintedNumberCalculation(right)) => {
                left == right
            }
            _ => false,
        }
    }
}

/// The expanded four-edge authored `border-image-width` value.
///
/// # Specified serialization
///
/// Serializes canonical specified CSS without resolving contextual values. Every stored edge is
/// visited, even when compressed or omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageWidth {
    values: [CssBorderImageWidthComponent; 4],
}

impl CssBorderImageWidth {
    #[must_use]
    pub fn try_new(values: Vec<CssBorderImageWidthComponent>) -> Option<Self> {
        expand_border_image_sides(values).map(|values| Self { values })
    }

    #[must_use]
    pub const fn values(&self) -> &[CssBorderImageWidthComponent; 4] {
        &self.values
    }
}

/// One authored `border-image-outset` component.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssBorderImageOutsetComponent {
    Length(CssNumeric<CssLength, CssNonNegative>),
    Number(CssNumeric<CssNumber, CssNonNegative>),
}

impl PartialEq for CssBorderImageOutsetComponent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Length(left), Self::Length(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            _ => false,
        }
    }
}

/// The expanded four-edge authored `border-image-outset` value.
///
/// # Specified serialization
///
/// Serializes canonical specified CSS without resolving contextual values. Every stored edge is
/// visited, even when compressed or omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImageOutset {
    values: [CssBorderImageOutsetComponent; 4],
}

impl CssBorderImageOutset {
    #[must_use]
    pub fn try_new(values: Vec<CssBorderImageOutsetComponent>) -> Option<Self> {
        expand_border_image_sides(values).map(|values| Self { values })
    }

    #[must_use]
    pub const fn values(&self) -> &[CssBorderImageOutsetComponent; 4] {
        &self.values
    }
}

fn expand_border_image_sides<T: Clone>(values: Vec<T>) -> Option<[T; 4]> {
    match values.as_slice() {
        [all] => Some([all.clone(), all.clone(), all.clone(), all.clone()]),
        [vertical, horizontal] => Some([
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ]),
        [top, horizontal, bottom] => Some([
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ]),
        [top, right, bottom, left] => {
            Some([top.clone(), right.clone(), bottom.clone(), left.clone()])
        }
        [] | [_, _, _, _, ..] => None,
    }
}

/// One axis repetition keyword in `border-image-repeat`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssBorderImageRepeatKeyword {
    Stretch,
    Repeat,
    Round,
    Space,
}

/// The authored horizontal and vertical border-image repetition modes.
///
/// # Specified serialization
///
/// Serializes canonical specified CSS without resolving contextual values. Every stored edge is
/// visited, even when compressed or omitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssBorderImageRepeat {
    horizontal: CssBorderImageRepeatKeyword,
    vertical: CssBorderImageRepeatKeyword,
}

impl CssBorderImageRepeat {
    #[must_use]
    pub const fn new(
        horizontal: CssBorderImageRepeatKeyword,
        vertical: CssBorderImageRepeatKeyword,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub const fn horizontal(self) -> CssBorderImageRepeatKeyword {
        self.horizontal
    }

    #[must_use]
    pub const fn vertical(self) -> CssBorderImageRepeatKeyword {
        self.vertical
    }
}

/// An authored `border-image` shorthand preserving only explicitly supplied components.
///
/// # Specified serialization
///
/// Serializes canonical specified CSS without resolving contextual values. Every stored edge is
/// visited, even when compressed or omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBorderImage {
    source: Option<CssImageValue>,
    slice: Option<CssBorderImageSlice>,
    width: Option<CssBorderImageWidth>,
    outset: Option<CssBorderImageOutset>,
    repeat: Option<CssBorderImageRepeat>,
}

impl CssBorderImage {
    #[must_use]
    pub fn try_new(
        source: Option<CssImageValue>,
        slice: Option<CssBorderImageSlice>,
        width: Option<CssBorderImageWidth>,
        outset: Option<CssBorderImageOutset>,
        repeat: Option<CssBorderImageRepeat>,
    ) -> Option<Self> {
        if source.is_none() && slice.is_none() && repeat.is_none()
            || slice.is_none() && (width.is_some() || outset.is_some())
        {
            return None;
        }
        Some(Self {
            source,
            slice,
            width,
            outset,
            repeat,
        })
    }

    #[must_use]
    pub const fn source(&self) -> Option<&CssImageValue> {
        self.source.as_ref()
    }

    #[must_use]
    pub const fn slice(&self) -> Option<&CssBorderImageSlice> {
        self.slice.as_ref()
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssBorderImageWidth> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn outset(&self) -> Option<&CssBorderImageOutset> {
        self.outset.as_ref()
    }

    #[must_use]
    pub const fn repeat(&self) -> Option<CssBorderImageRepeat> {
        self.repeat
    }
}

/// Authored UI4 outline styles. Hidden is excluded from this semantic domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOutlineStyle {
    Auto,
    None,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// The authored UI4 outline-color domain: automatic, shared color or image-1D.
/// Automatic color stays
/// symbolic; accent-color and currentColor computation belong downstream.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum CssOutlineColor {
    #[default]
    Auto,
    Color(Box<CssColor>),
    Image1D(Box<crate::values::image::CssImage1D>),
}

impl CssOutlineColor {
    /// Borrows the reusable authored image-1D branch.
    pub fn image_1d(&self) -> Option<&crate::values::image::CssImage1D> {
        match self {
            Self::Image1D(value) => Some(value),
            _ => None,
        }
    }
    /// Borrows the shared authored color, when this is not the automatic branch.
    #[must_use]
    pub fn color(&self) -> Option<&CssColor> {
        match self {
            Self::Auto | Self::Image1D(_) => None,
            Self::Color(color) => Some(color),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssOutline {
    width: Option<CssLineWidth>,
    style: Option<CssOutlineStyle>,
    color: Option<CssOutlineColor>,
}

impl CssOutline {
    #[must_use]
    pub fn try_new(
        width: Option<CssLineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssOutlineColor>,
    ) -> Option<Self> {
        if width.is_none() && style.is_none() && color.is_none() {
            None
        } else {
            Some(Self::new(width, style, color))
        }
    }

    pub(crate) fn new(
        width: Option<CssLineWidth>,
        style: Option<CssOutlineStyle>,
        color: Option<CssOutlineColor>,
    ) -> Self {
        Self {
            width,
            style,
            color,
        }
    }

    #[must_use]
    pub const fn width(&self) -> Option<&CssLineWidth> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<CssOutlineStyle> {
        self.style
    }

    #[must_use]
    pub const fn color(&self) -> Option<&CssOutlineColor> {
        self.color.as_ref()
    }
}
use crate::color::CssColor;
