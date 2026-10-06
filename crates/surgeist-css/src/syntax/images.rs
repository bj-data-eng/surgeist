//! Checked construction boundaries for authored Images 3 values.

use super::{
    CssAngleOrZero, CssColorStopListItem, CssGradient, CssHorizontalPosition, CssUrlModifier,
    CssVerticalPosition,
};
use super::{
    CssColor, CssColorStopList, CssGradientColorStop, CssImageValue, CssLinearGradient,
    CssLinearGradientDirection, CssPhysicalPosition, CssRadialGradient, CssRadialShape,
    CssRadialSize, CssSpecifiedLengthPercentage,
};

/// Failure to admit the complete authored image subtree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImageConstructionError {
    /// A property-specific `none` keyword is not an image.
    NotImage,
    /// The complete subtree exceeds the shared 256-level structural ceiling.
    NestingLimit,
    /// The composed structural depth cannot be represented.
    CapacityOverflow,
}
impl std::fmt::Display for CssImageConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotImage => "none is not an image",
            Self::NestingLimit => "image nesting limit exceeded",
            Self::CapacityOverflow => "image capacity overflow",
        })
    }
}
impl std::error::Error for CssImageConstructionError {}

/// Two checked authored image/none branches awaiting used-scheme selection.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLightDarkImage {
    light: CssImageValue,
    dark: CssImageValue,
    nesting_depth: u32,
}
impl CssLightDarkImage {
    /// Checks every supplied image, color, modifier and numeric component subtree.
    pub fn try_new(
        light: CssImageValue,
        dark: CssImageValue,
    ) -> Result<Self, CssImageConstructionError> {
        let nesting_depth = enclosing_depth(image_depth(&light)?.max(image_depth(&dark)?))?;
        Ok(Self {
            light,
            dark,
            nesting_depth,
        })
    }
    /// Borrows the complete authored light-scheme image/none branch.
    pub const fn light(&self) -> &CssImageValue {
        &self.light
    }
    /// Borrows the complete authored dark-scheme image/none branch.
    pub const fn dark(&self) -> &CssImageValue {
        &self.dark
    }
}

fn enclosing_depth(depth: u32) -> Result<u32, CssImageConstructionError> {
    let depth = depth
        .checked_add(1)
        .ok_or(CssImageConstructionError::CapacityOverflow)?;
    if depth > crate::STRUCTURAL_NESTING_LIMIT {
        Err(CssImageConstructionError::NestingLimit)
    } else {
        Ok(depth)
    }
}

fn length_percentage_depth(value: &CssSpecifiedLengthPercentage) -> u32 {
    value
        .calculation()
        .map_or(0, |v| v.components().nesting_depth())
}

fn stops_depth(stops: &CssColorStopList) -> Result<u32, CssImageConstructionError> {
    let mut depth = 0;
    for item in stops.items() {
        depth = depth.max(match item {
            CssColorStopListItem::Hint(value) => length_percentage_depth(value),
            CssColorStopListItem::Stop(value) => {
                let color = value.color().nesting_depth().map_err(|error| match error {
                    super::CssColorConstructionError::CapacityOverflow => {
                        CssImageConstructionError::CapacityOverflow
                    }
                    _ => CssImageConstructionError::NestingLimit,
                })?;
                color.max(value.position().map_or(0, length_percentage_depth))
            }
        });
    }
    Ok(depth)
}

fn position_depth(position: &CssPhysicalPosition) -> u32 {
    let horizontal = match position.horizontal() {
        CssHorizontalPosition::Offset(v)
        | CssHorizontalPosition::LeftOffset(v)
        | CssHorizontalPosition::RightOffset(v)
        | CssHorizontalPosition::XStartOffset(v)
        | CssHorizontalPosition::XEndOffset(v) => length_percentage_depth(v),
        _ => 0,
    };
    let vertical = match position.vertical() {
        CssVerticalPosition::Offset(v)
        | CssVerticalPosition::TopOffset(v)
        | CssVerticalPosition::BottomOffset(v)
        | CssVerticalPosition::YStartOffset(v)
        | CssVerticalPosition::YEndOffset(v) => length_percentage_depth(v),
        _ => 0,
    };
    horizontal.max(vertical)
}

fn image_depth(image: &CssImageValue) -> Result<u32, CssImageConstructionError> {
    match image {
        CssImageValue::None => Ok(0),
        CssImageValue::LightDark(value) => Ok(value.nesting_depth),
        CssImageValue::Url(value) => {
            let mut depth = 0;
            for modifier in value.modifiers() {
                if let CssUrlModifier::Function(function) = modifier {
                    depth = depth.max(enclosing_depth(
                        function.argument_components().nesting_depth(),
                    )?);
                }
            }
            enclosing_depth(depth)
        }
        CssImageValue::Gradient(gradient) => {
            let depth = match gradient {
                CssGradient::Linear(value) | CssGradient::RepeatingLinear(value) => {
                    let direction = match value.direction() {
                        Some(CssLinearGradientDirection::Angle(CssAngleOrZero::Angle(value))) => {
                            value
                                .calculation()
                                .map_or(0, |v| v.components().nesting_depth())
                        }
                        _ => 0,
                    };
                    stops_depth(value.stops())?.max(direction)
                }
                CssGradient::Radial(value) | CssGradient::RepeatingRadial(value) => {
                    let size = match value.size() {
                        Some(CssRadialSize::Circle(v)) => v
                            .calculation()
                            .map_or(0, |v| v.components().nesting_depth()),
                        Some(CssRadialSize::Ellipse(v)) => v
                            .horizontal()
                            .calculation()
                            .map_or(0, |v| v.components().nesting_depth())
                            .max(
                                v.vertical()
                                    .calculation()
                                    .map_or(0, |v| v.components().nesting_depth()),
                            ),
                        _ => 0,
                    };
                    stops_depth(value.stops())?
                        .max(size)
                        .max(value.position().map_or(0, position_depth))
                }
            };
            enclosing_depth(depth)
        }
    }
}

/// One authored `<image>`, excluding property-specific `none` keywords.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImage {
    value: CssImageValue,
}

impl CssImage {
    /// Checks the complete image graph without loading or resolving the image.
    ///
    /// A bare `none` is not an image. Every retained image, color, numeric and
    /// URL-modifier subtree must fit the shared 256-level structural ceiling.
    /// This checked carrier adds no CSS function level of its own.
    pub fn try_new(value: CssImageValue) -> Result<Self, CssImageConstructionError> {
        if matches!(value, CssImageValue::None) {
            return Err(CssImageConstructionError::NotImage);
        }
        image_depth(&value)?;
        Ok(Self { value })
    }

    /// Returns the checked authored image payload.
    #[must_use]
    pub const fn value(&self) -> &CssImageValue {
        &self.value
    }
}

impl CssGradientColorStop {
    /// Constructs a stop from a checked authored color and optional line position.
    /// The authored color stays symbolic; no legacy color projection is invented.
    #[must_use]
    pub fn from_color(color: CssColor, position: Option<CssSpecifiedLengthPercentage>) -> Self {
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
        position: Option<CssPhysicalPosition>,
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
