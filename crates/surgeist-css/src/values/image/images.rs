use super::{
    CssColorStopList, CssGradientColorStop, CssImageValue, CssLinearGradient,
    CssLinearGradientDirection, CssRadialGradient, CssRadialShape, CssRadialSize,
};
use crate::color::CssColor;
use crate::numeric::{CssLengthPercentage, CssNumeric};
use crate::values::effects::images::{enclosing_depth, image_depth};
/// Failure to admit a complete authored image subtree, gradient color stop or
/// `stripes()` image.
///
/// Unit failures are read through the `is_*` predicates; a rejected component
/// or `stripes()` grammar is exposed by its accessor and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssImageConstructionError {
    kind: ImageConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ImageConstructionKind {
    NotImage,
    NestingLimit,
    CapacityOverflow,
    TooManyPositions,
    EmptyStripes,
    Component(crate::component::CssComponentValueError),
    // Boxed so that every image construction failure stays small.
    Grammar(Box<crate::property::CssPropertyValueError>),
    RecoveredComponent(crate::component::CssValueOrigin),
}
impl CssImageConstructionError {
    pub(crate) const fn new(kind: ImageConstructionKind) -> Self {
        Self { kind }
    }
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether a property-specific `none` keyword was given as an image.
        is_not_image => ImageConstructionKind::NotImage,
        /// Returns whether the subtree exceeds the shared 256-level structural ceiling.
        is_nesting_limit => ImageConstructionKind::NestingLimit,
        /// Returns whether the composed structural depth cannot be represented.
        is_capacity_overflow => ImageConstructionKind::CapacityOverflow,
        /// Returns whether a color stop supplies more than its two authored positions.
        is_too_many_positions => ImageConstructionKind::TooManyPositions,
        /// Returns whether `stripes()` was given no stripes.
        is_empty_stripes => ImageConstructionKind::EmptyStripes,
    }
    /// Returns the rejected component of a `stripes()` image.
    #[must_use]
    pub const fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
        match &self.kind {
            ImageConstructionKind::Component(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the `stripes()` grammar failure.
    #[must_use]
    pub fn property_value_error(&self) -> Option<&crate::property::CssPropertyValueError> {
        match &self.kind {
            ImageConstructionKind::Grammar(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the origin of an implicitly closed component, which checked
    /// construction rejects.
    #[must_use]
    pub const fn recovered_origin(&self) -> Option<&crate::component::CssValueOrigin> {
        match &self.kind {
            ImageConstructionKind::RecoveredComponent(origin) => Some(origin),
            _ => None,
        }
    }
}
impl From<crate::component::CssComponentValueError> for CssImageConstructionError {
    fn from(error: crate::component::CssComponentValueError) -> Self {
        Self::new(ImageConstructionKind::Component(error))
    }
}
impl From<crate::property::CssPropertyValueError> for CssImageConstructionError {
    fn from(error: crate::property::CssPropertyValueError) -> Self {
        Self::new(ImageConstructionKind::Grammar(Box::new(error)))
    }
}
impl std::fmt::Display for CssImageConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self.kind {
            ImageConstructionKind::NotImage => "none is not an image",
            ImageConstructionKind::NestingLimit => "image nesting limit exceeded",
            ImageConstructionKind::CapacityOverflow => "image capacity overflow",
            ImageConstructionKind::TooManyPositions => {
                "a color stop admits at most two authored positions"
            }
            ImageConstructionKind::EmptyStripes => "stripes() requires at least one stripe",
            ImageConstructionKind::Component(_) => "invalid stripes() component",
            ImageConstructionKind::Grammar(_) => "invalid stripes() image",
            ImageConstructionKind::RecoveredComponent(_) => {
                "stripes() rejects an implicitly closed component"
            }
        })
    }
}
impl std::error::Error for CssImageConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ImageConstructionKind::Component(error) => Some(error),
            ImageConstructionKind::Grammar(error) => Some(&**error),
            _ => None,
        }
    }
}

/// Two checked authored image/none branches awaiting used-scheme selection.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLightDarkImage {
    light: CssImageValue,
    dark: CssImageValue,
    pub(crate) nesting_depth: u32,
}
impl CssLightDarkImage {
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<bool, crate::serialization::CssSerializationError> {
        use crate::specified_inverse::equality::BoundedStructuralEquality;
        Ok(self.light.bounded_eq(&other.light, context)?
            && self.dark.bounded_eq(&other.dark, context)?
            && self.nesting_depth == other.nesting_depth)
    }

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

/// One authored `<image>`, excluding property-specific `none` keywords.
///
/// # Specified serialization
///
/// Serializes the checked image without resolving its resource or geometry.
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
            return Err(CssImageConstructionError::new(
                ImageConstructionKind::NotImage,
            ));
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
    pub fn from_color(color: CssColor, position: Option<CssNumeric<CssLengthPercentage>>) -> Self {
        Self {
            color,
            positions: position.into_iter().collect(),
        }
    }

    /// Retains zero, one or two checked line positions on one authored color stop.
    pub fn try_from_positions(
        color: CssColor,
        positions: Vec<CssNumeric<CssLengthPercentage>>,
    ) -> Result<Self, CssImageConstructionError> {
        if positions.len() > 2 {
            return Err(CssImageConstructionError::new(
                ImageConstructionKind::TooManyPositions,
            ));
        }
        Ok(Self { color, positions })
    }
}

impl CssLinearGradient {
    /// Constructs a linear gradient from an optional checked direction and a
    /// checked nonempty list of color stops; interpolation is initially omitted.
    #[must_use]
    pub const fn new(
        direction: Option<CssLinearGradientDirection>,
        stops: CssColorStopList,
    ) -> Self {
        Self {
            direction,
            interpolation: None,
            stops,
        }
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
        position: Option<crate::values::position::CssPosition>,
        stops: CssColorStopList,
    ) -> Option<Self> {
        if !Self::allows_shape_size(shape, size.as_ref()) {
            return None;
        }
        Some(Self {
            shape,
            size,
            position,
            interpolation: None,
            stops,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CssImageConstructionError, ImageConstructionKind};

    #[test]
    fn construction_failures_display_their_invariant_without_a_source() {
        for (kind, text) in [
            (ImageConstructionKind::NotImage, "none is not an image"),
            (
                ImageConstructionKind::NestingLimit,
                "image nesting limit exceeded",
            ),
            (
                ImageConstructionKind::CapacityOverflow,
                "image capacity overflow",
            ),
        ] {
            let error = CssImageConstructionError::new(kind);
            assert_eq!(error.to_string(), text);
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}
