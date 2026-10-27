//! Checked Image graphs, including the imported Filter Effects 1 §12 image function.

use super::{CssFilterAmount, CssFilterFunction, CssFilterFunctionList};
use crate::color::CssColor;
use crate::component::values::CssComponentValueErrorKind;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueRef, CssValueOrigin,
    CssValueTokenRef,
};
use crate::numeric::CssAngleOrZero;
use crate::numeric::{CssLengthPercentage, CssNumeric};
use crate::values::common::CssUrlModifier;
use crate::values::image::images::ImageConstructionKind;
use crate::values::image::{
    CssColorStopList, CssImageValue, CssLinearGradientDirection, CssRadialSize,
};
use crate::values::image::{CssColorStopListItem, CssGradient};
use crate::values::image::{CssImage, CssImageConstructionError};

/// A complete quoted String operand, without assigning resource interpretation.
///
/// # Specified serialization
///
/// Emits the authored operand/function without assigning downstream execution meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFilterImageString {
    component: CssComponentValue,
}
impl CssFilterImageString {
    /// Constructs a String with explicit programmatic provenance, including empty Strings.
    pub fn try_new(decoded: impl Into<String>) -> Result<Self, CssComponentValueError> {
        Self::try_from_component(CssComponentValue::try_string(decoded)?)
    }
    /// Retains exactly one complete original String token and its provenance.
    /// Wrong tokens or recovered termination report InvalidToken at that original origin.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if !matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::String(_))
        ) {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ));
        }
        if let Some(origin) = component.implicit_termination_origin() {
            return Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                origin.clone(),
            ));
        }
        Ok(Self { component })
    }
    /// Borrows the decoded String; no URL equivalence or execution meaning is assigned.
    #[must_use]
    pub fn as_str(&self) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::String(value)) = self.component.view()
        else {
            unreachable!("checked String operand")
        };
        value
    }
    /// Borrows the unchanged original checked token.
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    /// Borrows the original token origin, or explicit programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}

#[derive(Clone, Debug, PartialEq)]
enum FilterImageInput {
    String(CssFilterImageString),
    Image(CssImage),
}

/// One admitted first operand of `filter()`, preserving String/Image identity.
///
/// # Specified serialization
///
/// Emits the authored operand/function without assigning downstream execution meaning.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterImageInput {
    value: FilterImageInput,
}

/// A borrowed authored `filter()` operand, without serialized inference.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFilterImageInputRef<'a> {
    /// A checked String, with undefined downstream processing-input interpretation.
    String(&'a CssFilterImageString),
    /// A complete checked Image, excluding a bare property None keyword.
    Image(&'a CssImage),
}
impl CssFilterImageInput {
    /// Retains a checked String without converting it into a URL.
    #[must_use]
    pub const fn from_string(value: CssFilterImageString) -> Self {
        Self {
            value: FilterImageInput::String(value),
        }
    }
    /// Retains an already checked Image without adding a function depth level.
    #[must_use]
    pub const fn from_image(value: CssImage) -> Self {
        Self {
            value: FilterImageInput::Image(value),
        }
    }
    /// Admits an Image through its canonical complete graph boundary.
    pub fn try_from_image(value: CssImageValue) -> Result<Self, CssImageConstructionError> {
        CssImage::try_new(value).map(Self::from_image)
    }
    /// Borrows the distinct admitted String or Image operand.
    #[must_use]
    pub const fn view(&self) -> CssFilterImageInputRef<'_> {
        match &self.value {
            FilterImageInput::String(value) => CssFilterImageInputRef::String(value),
            FilterImageInput::Image(value) => CssFilterImageInputRef::Image(value),
        }
    }
}

/// A complete checked Filter Effects 1 §12 authored image function.
///
/// The selected draft does not define String-to-processing-input interpretation.
/// This model retains authored operands only and never loads or filters an image.
///
/// # Specified serialization
///
/// Emits the authored operand/function without assigning downstream execution meaning.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFilterImage {
    input: CssFilterImageInput,
    filters: CssFilterFunctionList,
    nesting_depth: u32,
}
impl CssFilterImage {
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<bool, crate::serialization::CssSerializationError> {
        use crate::specified_inverse::equality::BoundedStructuralEquality;
        Ok(self.input.bounded_eq(&other.input, context)?
            && self.filters.bounded_eq(&other.filters, context)?
            && self.nesting_depth == other.nesting_depth)
    }

    /// Checks the enclosing function and every retained image/filter/color/numeric/URL child.
    /// Operand and list carriers contribute no function nesting level.
    pub fn try_new(
        input: CssFilterImageInput,
        filters: CssFilterFunctionList,
    ) -> Result<Self, CssImageConstructionError> {
        let input_depth = match input.view() {
            CssFilterImageInputRef::String(_) => 0,
            CssFilterImageInputRef::Image(value) => image_depth(value.value())?,
        };
        let mut depth = input_depth;
        for function in filters.functions() {
            depth = depth.max(filter_depth(function)?);
        }
        let nesting_depth = enclosing_depth(depth)?;
        Ok(Self {
            input,
            filters,
            nesting_depth,
        })
    }
    /// Borrows the first operand, preserving String/Image identity.
    #[must_use]
    pub const fn input(&self) -> &CssFilterImageInput {
        &self.input
    }
    /// Borrows the complete nonempty ordered filter-function/URL list.
    #[must_use]
    pub const fn filters(&self) -> &CssFilterFunctionList {
        &self.filters
    }
}

pub(crate) fn enclosing_depth(depth: u32) -> Result<u32, CssImageConstructionError> {
    let depth = depth.checked_add(1).ok_or(CssImageConstructionError::new(
        ImageConstructionKind::CapacityOverflow,
    ))?;
    if depth > crate::STRUCTURAL_NESTING_LIMIT {
        Err(CssImageConstructionError::new(
            ImageConstructionKind::NestingLimit,
        ))
    } else {
        Ok(depth)
    }
}

fn length_percentage_depth(value: &CssNumeric<CssLengthPercentage>) -> u32 {
    value
        .calculation()
        .map_or(0, |v| v.components().nesting_depth())
}

fn color_depth(value: &CssColor) -> Result<u32, CssImageConstructionError> {
    value.nesting_depth().map_err(|error| match error {
        error if error.is_capacity_overflow() => {
            CssImageConstructionError::new(ImageConstructionKind::CapacityOverflow)
        }
        _ => CssImageConstructionError::new(ImageConstructionKind::NestingLimit),
    })
}

fn stops_depth(stops: &CssColorStopList) -> Result<u32, CssImageConstructionError> {
    let mut depth = 0;
    for item in stops.items() {
        depth = depth.max(match item {
            CssColorStopListItem::Hint(value) => length_percentage_depth(value),
            CssColorStopListItem::Stop(value) => {
                let color = color_depth(value.color())?;
                color.max(
                    value
                        .positions()
                        .iter()
                        .map(length_percentage_depth)
                        .max()
                        .unwrap_or(0),
                )
            }
        });
    }
    Ok(depth)
}

pub(crate) fn image_depth(image: &CssImageValue) -> Result<u32, CssImageConstructionError> {
    match image {
        CssImageValue::None => Ok(0),
        CssImageValue::LightDark(value) => Ok(value.nesting_depth),
        CssImageValue::Filter(value) => Ok(value.nesting_depth),
        CssImageValue::Url(value) => url_depth(value),
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
                        .max(value.position().map_or(
                            0,
                            crate::values::position::CssPosition::numeric_nesting_depth,
                        ))
                }
                CssGradient::Conic(value) => {
                    let mut depth = value.from().map_or(0, |from| match from {
                        CssAngleOrZero::Angle(angle) => angle
                            .calculation()
                            .map_or(0, |v| v.components().nesting_depth()),
                        CssAngleOrZero::Zero(_) => 0,
                    });
                    depth = depth.max(value.position().map_or(
                        0,
                        crate::values::position::CssPosition::numeric_nesting_depth,
                    ));
                    for item in value.stops().items() {
                        depth = depth.max(match item {
                            crate::values::image::CssAngularColorStopListItem::Hint(hint) => hint.nesting_depth(),
                            crate::values::image::CssAngularColorStopListItem::Stop(stop) => {
                                color_depth(stop.color())?.max(
                                    stop.positions()
                                        .iter()
                                        .map(crate::values::image::CssAngularColorStopPosition::nesting_depth)
                                        .max()
                                        .unwrap_or(0),
                                )
                            }
                        });
                    }
                    depth
                }
            };
            enclosing_depth(depth)
        }
    }
}

fn url_depth(value: &crate::values::common::CssUrl) -> Result<u32, CssImageConstructionError> {
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

fn filter_depth(value: &CssFilterFunction) -> Result<u32, CssImageConstructionError> {
    let amount = |value: &CssFilterAmount| match value {
        CssFilterAmount::Default => 0,
        CssFilterAmount::Number(v) => v
            .calculation()
            .map_or(0, |v| v.components().nesting_depth()),
        CssFilterAmount::Percentage(v) => v
            .calculation()
            .map_or(0, |v| v.components().nesting_depth()),
        CssFilterAmount::HintedNumberCalculation(v) => v.components().nesting_depth(),
    };
    let depth = match value {
        CssFilterFunction::Url(value) => return url_depth(value),
        CssFilterFunction::Blur(value) => value
            .authored_length()
            .and_then(|v| v.calculation())
            .map_or(0, |v| v.components().nesting_depth()),
        CssFilterFunction::HueRotate(value) => match value.authored_angle() {
            Some(CssAngleOrZero::Angle(value)) => value
                .calculation()
                .map_or(0, |v| v.components().nesting_depth()),
            _ => 0,
        },
        CssFilterFunction::Brightness(v)
        | CssFilterFunction::Contrast(v)
        | CssFilterFunction::Grayscale(v)
        | CssFilterFunction::Invert(v)
        | CssFilterFunction::Opacity(v)
        | CssFilterFunction::Saturate(v)
        | CssFilterFunction::Sepia(v) => amount(v),
        CssFilterFunction::DropShadow(value) => {
            let color = value.color().map(color_depth).transpose()?.unwrap_or(0);
            color
                .max(
                    value
                        .offset_x()
                        .calculation()
                        .map_or(0, |v| v.components().nesting_depth()),
                )
                .max(
                    value
                        .offset_y()
                        .calculation()
                        .map_or(0, |v| v.components().nesting_depth()),
                )
                .max(
                    value
                        .standard_deviation()
                        .and_then(|v| v.calculation())
                        .map_or(0, |v| v.components().nesting_depth()),
                )
        }
    };
    enclosing_depth(depth)
}
