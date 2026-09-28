//! Checked construction boundaries for authored Images 3 values.

use super::{
    CssColor, CssColorStopList, CssGradientColorStop, CssGradientLinePosition, CssImageValue,
    CssLinearGradient, CssLinearGradientDirection, CssPositionValue, CssRadialGradient,
    CssRadialShape, CssRadialSize,
};

/// One authored `<image>`, excluding property-specific `none` keywords.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImage {
    value: CssImageValue,
}

impl CssImage {
    /// Checks the image-only grammar without loading or resolving the image.
    #[must_use]
    pub fn try_new(value: CssImageValue) -> Option<Self> {
        (!matches!(value, CssImageValue::None)).then_some(Self { value })
    }

    /// Returns the checked URL or gradient payload.
    #[must_use]
    pub const fn value(&self) -> &CssImageValue {
        &self.value
    }
}

impl CssGradientColorStop {
    /// Constructs a stop from a checked authored color and optional line position.
    /// The authored color stays symbolic; no legacy color projection is invented.
    #[must_use]
    pub fn from_color(color: CssColor, position: Option<CssGradientLinePosition>) -> Self {
        Self { color, position }
    }
}

impl CssLinearGradient {
    /// Constructs a linear gradient from an optional checked direction and a
    /// checked list of at least two color stops.
    #[must_use]
    pub const fn new(
        direction: Option<CssLinearGradientDirection>,
        stops: CssColorStopList,
    ) -> Self {
        Self { direction, stops }
    }
}

impl CssRadialGradient {
    pub(crate) fn allows_shape_size(
        shape: Option<CssRadialShape>,
        size: Option<&CssRadialSize>,
    ) -> bool {
        !matches!(
            (shape, size),
            (
                Some(CssRadialShape::Circle),
                Some(CssRadialSize::Ellipse(_))
            ) | (
                Some(CssRadialShape::Ellipse),
                Some(CssRadialSize::Circle(_))
            )
        )
    }

    /// Constructs a radial gradient while preserving omitted shape, size, and
    /// position. An explicit circle requires a circle radius; an explicit
    /// ellipse requires a pair of ellipse radii. Extents fit either shape.
    #[must_use]
    pub fn try_new(
        shape: Option<CssRadialShape>,
        size: Option<CssRadialSize>,
        position: Option<CssPositionValue>,
        stops: CssColorStopList,
    ) -> Option<Self> {
        if !Self::allows_shape_size(shape, size.as_ref()) {
            return None;
        }
        Some(Self {
            shape,
            size,
            position,
            stops,
        })
    }
}
