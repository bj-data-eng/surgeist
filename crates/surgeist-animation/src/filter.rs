//! Normalized filter-function lists and checked native animation algebra.
//!
//! Style supplies computed arguments, including omitted-value defaults, resolved
//! CSS-pixel lengths and explicit interpolation-space colors. This module owns
//! interpolation identities, not authored syntax or pixel execution. Finite
//! extrapolated samples remain unclamped for their consumer's final range policy.

use core::fmt;
use std::sync::Arc;

use crate::interpolation::{
    InterpolableColor, InterpolableNumber, InterpolablePercentage, InterpolationComponent,
    InterpolationProgress, checked_interpolate, interpolate_color,
};
use crate::transform::TransformAngle;

pub mod image;

/// The seven functions whose computed arguments interpolate as numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountFunction {
    Brightness,
    Contrast,
    Grayscale,
    Invert,
    Opacity,
    Saturate,
    Sepia,
}
impl AmountFunction {
    fn identity(self) -> f64 {
        match self {
            Self::Grayscale | Self::Invert | Self::Sepia => 0.,
            Self::Brightness | Self::Contrast | Self::Opacity | Self::Saturate => 1.,
        }
    }
}

/// A supplied computed number or percentage; percentage 1 means 100%.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterAmountInput {
    Number(InterpolableNumber),
    Percentage(InterpolablePercentage),
}

/// The shape of a function or opaque URL reference in an ordered filter list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterFunctionKind {
    Blur,
    Amount(AmountFunction),
    HueRotate,
    DropShadow,
    Url,
}

/// The argument rejected by checked construction or sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterComponent {
    Amount,
    Angle,
    StandardDeviation,
    OffsetX,
    OffsetY,
    Color(InterpolationComponent),
}

/// A filter input, compatibility or arithmetic failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterErrorKind {
    NonFinite,
    NegativeInput,
    InvalidInputAlpha,
    EmptyReference,
    IncompatibleFunctions,
    MismatchedColorSpaces,
    Capacity,
}

/// A whole-list failure, with optional function and argument context.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterError {
    kind: FilterErrorKind,
    index: Option<usize>,
    component: Option<FilterComponent>,
    value: Option<f64>,
}
impl FilterError {
    fn new(kind: FilterErrorKind) -> Self {
        Self {
            kind,
            index: None,
            component: None,
            value: None,
        }
    }
    fn component(mut self, component: FilterComponent, value: f64) -> Self {
        self.component = Some(component);
        self.value = Some(value);
        self
    }
    fn at(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }
    pub const fn kind(&self) -> FilterErrorKind {
        self.kind
    }
    pub const fn function_index(&self) -> Option<usize> {
        self.index
    }
    pub const fn failed_component(&self) -> Option<FilterComponent> {
        self.component
    }
    pub const fn value(&self) -> Option<f64> {
        self.value
    }
}
impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "filter {:?}", self.kind)?;
        if let Some(index) = self.index {
            write!(f, " at function {index}")?;
        }
        if let Some(component) = self.component {
            write!(f, " in {component:?}")?;
        }
        if let Some(value) = self.value {
            write!(f, ": {value}")?;
        }
        Ok(())
    }
}
impl std::error::Error for FilterError {}

fn finite(value: f64, component: FilterComponent) -> Result<f64, FilterError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FilterError::new(FilterErrorKind::NonFinite).component(component, value))
    }
}
fn nonnegative(value: f64, component: FilterComponent) -> Result<f64, FilterError> {
    finite(value, component)?;
    if value < 0. {
        Err(FilterError::new(FilterErrorKind::NegativeInput).component(component, value))
    } else {
        Ok(value)
    }
}

/// A finite singleton drop shadow. The third length is sigma, not blur radius.
/// There is no spread or inset slot. Sampled sigma/alpha can be extended.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterDropShadow {
    offsets: [f64; 2],
    sigma: f64,
    color: InterpolableColor,
}
impl FilterDropShadow {
    pub const fn offsets_pixels(&self) -> [f64; 2] {
        self.offsets
    }
    pub const fn standard_deviation_pixels(&self) -> f64 {
        self.sigma
    }
    pub const fn color(&self) -> InterpolableColor {
        self.color
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Payload {
    Blur(f64),
    Amount(AmountFunction, f64),
    HueRotate(TransformAngle),
    DropShadow(FilterDropShadow),
    Url(Arc<str>),
}

/// An immutable finite function sample, or a retained opaque URL reference.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterFunction(Payload);
impl FilterFunction {
    pub const fn kind(&self) -> FilterFunctionKind {
        match &self.0 {
            Payload::Blur(_) => FilterFunctionKind::Blur,
            Payload::Amount(kind, _) => FilterFunctionKind::Amount(*kind),
            Payload::HueRotate(_) => FilterFunctionKind::HueRotate,
            Payload::DropShadow(_) => FilterFunctionKind::DropShadow,
            Payload::Url(_) => FilterFunctionKind::Url,
        }
    }
    pub const fn blur_pixels(&self) -> Option<f64> {
        if let Payload::Blur(value) = self.0 {
            Some(value)
        } else {
            None
        }
    }
    pub const fn amount(&self) -> Option<(AmountFunction, f64)> {
        if let Payload::Amount(kind, value) = self.0 {
            Some((kind, value))
        } else {
            None
        }
    }
    pub const fn angle(&self) -> Option<TransformAngle> {
        if let Payload::HueRotate(value) = self.0 {
            Some(value)
        } else {
            None
        }
    }
    pub const fn drop_shadow(&self) -> Option<&FilterDropShadow> {
        if let Payload::DropShadow(value) = &self.0 {
            Some(value)
        } else {
            None
        }
    }
    pub fn url_reference(&self) -> Option<&str> {
        if let Payload::Url(value) = &self.0 {
            Some(value)
        } else {
            None
        }
    }

    fn identity(&self) -> Result<Self, FilterError> {
        Ok(Self(match &self.0 {
            Payload::Blur(_) => Payload::Blur(0.),
            Payload::Amount(kind, _) => Payload::Amount(*kind, kind.identity()),
            Payload::HueRotate(_) => Payload::HueRotate(TransformAngle::ZERO),
            Payload::DropShadow(value) => Payload::DropShadow(FilterDropShadow {
                offsets: [0.; 2],
                sigma: 0.,
                color: InterpolableColor::from_sampled_premultiplied_components(
                    value.color.space(),
                    0.,
                    0.,
                    0.,
                    0.,
                )
                .map_err(|_| FilterError::new(FilterErrorKind::NonFinite))?,
            }),
            Payload::Url(_) => {
                return Err(FilterError::new(FilterErrorKind::IncompatibleFunctions));
            }
        }))
    }
    fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, FilterError> {
        let scalar = |a, b, component| {
            checked_interpolate(a, b, progress.value()).map_err(|value| {
                FilterError::new(FilterErrorKind::NonFinite).component(component, value)
            })
        };
        Ok(Self(match (&self.0, &other.0) {
            (Payload::Blur(a), Payload::Blur(b)) => {
                Payload::Blur(scalar(*a, *b, FilterComponent::StandardDeviation)?)
            }
            (Payload::Amount(ka, a), Payload::Amount(kb, b)) if ka == kb => {
                Payload::Amount(*ka, scalar(*a, *b, FilterComponent::Amount)?)
            }
            (Payload::HueRotate(a), Payload::HueRotate(b)) => {
                let radians = scalar(a.radians(), b.radians(), FilterComponent::Angle)?;
                Payload::HueRotate(TransformAngle::from_radians(radians).map_err(|_| {
                    FilterError::new(FilterErrorKind::NonFinite)
                        .component(FilterComponent::Angle, radians)
                })?)
            }
            (Payload::DropShadow(a), Payload::DropShadow(b)) => {
                if a.color.space() != b.color.space() {
                    return Err(FilterError::new(FilterErrorKind::MismatchedColorSpaces));
                }
                let [ax, ay] = a.offsets;
                let [bx, by] = b.offsets;
                let offsets = [
                    scalar(ax, bx, FilterComponent::OffsetX)?,
                    scalar(ay, by, FilterComponent::OffsetY)?,
                ];
                let sigma = scalar(a.sigma, b.sigma, FilterComponent::StandardDeviation)?;
                let color = interpolate_color(a.color, b.color, progress).map_err(
                    |(component, value)| {
                        FilterError::new(FilterErrorKind::NonFinite)
                            .component(FilterComponent::Color(component), value)
                    },
                )?;
                Payload::DropShadow(FilterDropShadow {
                    offsets,
                    sigma,
                    color,
                })
            }
            _ => return Err(FilterError::new(FilterErrorKind::IncompatibleFunctions)),
        }))
    }
}

/// A guarded computed input, before eased sampling. Supplied defaults are
/// distinct from interpolation identities; in particular grayscale()/invert()/
/// sepia() omitted arguments compute to one, while their padding identity is zero.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterInput(FilterFunction);
impl FilterInput {
    pub fn blur_pixels(sigma: f64) -> Result<Self, FilterError> {
        Ok(Self(FilterFunction(Payload::Blur(nonnegative(
            sigma,
            FilterComponent::StandardDeviation,
        )?))))
    }
    /// Converts a supplied percentage ratio to its numerical amount. All
    /// nonnegative finite amounts, including those above one, remain intact.
    pub fn amount(function: AmountFunction, input: FilterAmountInput) -> Result<Self, FilterError> {
        let value = match input {
            FilterAmountInput::Number(value) => value.value(),
            FilterAmountInput::Percentage(value) => value.value(),
        };
        Ok(Self(FilterFunction(Payload::Amount(
            function,
            nonnegative(value, FilterComponent::Amount)?,
        ))))
    }
    pub fn hue_rotate(angle: TransformAngle) -> Self {
        Self(FilterFunction(Payload::HueRotate(angle)))
    }
    /// Uses supplied resolved color and CSS-pixel offsets/sigma. Omitted colors
    /// are resolved by Style, never substituted with interpolation transparency.
    pub fn drop_shadow_pixels(
        offsets: [f64; 2],
        sigma: f64,
        color: InterpolableColor,
    ) -> Result<Self, FilterError> {
        for (value, component) in offsets
            .into_iter()
            .zip([FilterComponent::OffsetX, FilterComponent::OffsetY])
        {
            finite(value, component)?;
        }
        nonnegative(sigma, FilterComponent::StandardDeviation)?;
        if !(0. ..=1.).contains(&color.alpha()) {
            return Err(
                FilterError::new(FilterErrorKind::InvalidInputAlpha).component(
                    FilterComponent::Color(InterpolationComponent::ColorAlpha),
                    color.alpha(),
                ),
            );
        }
        Ok(Self(FilterFunction(Payload::DropShadow(
            FilterDropShadow {
                offsets,
                sigma,
                color,
            },
        ))))
    }
    /// Retains a caller-owned reference token without parsing, fetching or
    /// resolving it. Any reference makes property interpolation discrete.
    pub fn url_reference(reference: Arc<str>) -> Result<Self, FilterError> {
        if reference.chars().all(char::is_whitespace) {
            return Err(FilterError::new(FilterErrorKind::EmptyReference));
        }
        Ok(Self(FilterFunction(Payload::Url(reference))))
    }
}

/// An immutable complete value. Empty is `none`; sampled lists are reusable
/// endpoints without reapplying input range guards or losing reference identity.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterList {
    functions: Arc<[FilterFunction]>,
}
fn reserve(length: usize) -> Result<Vec<FilterFunction>, FilterError> {
    let mut functions = Vec::new();
    functions
        .try_reserve_exact(length)
        .map_err(|_| FilterError::new(FilterErrorKind::Capacity))?;
    Ok(functions)
}
impl FilterList {
    pub fn new(inputs: Vec<FilterInput>) -> Result<Self, FilterError> {
        let mut functions = reserve(inputs.len())?;
        functions.extend(inputs.into_iter().map(|input| input.0));
        Ok(Self {
            functions: functions.into(),
        })
    }
    pub fn functions(&self) -> &[FilterFunction] {
        &self.functions
    }
    fn compatible(&self, other: &Self) -> bool {
        !self
            .functions
            .iter()
            .chain(other.functions.iter())
            .any(|function| function.kind() == FilterFunctionKind::Url)
            && self
                .functions
                .iter()
                .zip(other.functions.iter())
                .all(|(a, b)| a.kind() == b.kind())
    }
    // Filter images must route incompatibility to generic image interpolation,
    // rather than receiving the property-level discrete fallback.
    pub(super) fn try_interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Option<Self>, FilterError> {
        if !self.compatible(other) {
            return Ok(None);
        }
        let mut functions = reserve(self.functions.len().max(other.functions.len()))?;
        for (index, (a, b)) in self
            .functions
            .iter()
            .zip(other.functions.iter())
            .enumerate()
        {
            functions.push(
                a.interpolate(b, progress)
                    .map_err(|error| error.at(index))?,
            );
        }
        let (tail, missing_from) = if self.functions.len() > other.functions.len() {
            (self.functions.iter().skip(other.functions.len()), false)
        } else {
            (other.functions.iter().skip(self.functions.len()), true)
        };
        for function in tail {
            let index = functions.len();
            let identity = function.identity().map_err(|error| error.at(index))?;
            let result = if missing_from {
                identity.interpolate(function, progress)
            } else {
                function.interpolate(&identity, progress)
            };
            functions.push(result.map_err(|error| error.at(index))?);
        }
        Ok(Some(Self {
            functions: functions.into(),
        }))
    }
    /// Interpolates URL-free corresponding functions with identity end padding.
    /// Any URL or incompatible pair switches the whole original list at 0.5.
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, FilterError> {
        Ok(self.try_interpolate(other, progress)?.unwrap_or_else(|| {
            if progress.value() < 0.5 {
                self.clone()
            } else {
                other.clone()
            }
        }))
    }
    /// Concatenates underlying first, without modifying either source list.
    pub fn added_to(&self, other: &Self) -> Result<Self, FilterError> {
        let length = self
            .functions
            .len()
            .checked_add(other.functions.len())
            .ok_or_else(|| FilterError::new(FilterErrorKind::Capacity))?;
        let mut functions = reserve(length)?;
        functions.extend(self.functions.iter().chain(other.functions.iter()).cloned());
        Ok(Self {
            functions: functions.into(),
        })
    }
    /// The selected Filter Effects 1 accumulation result is the complete effect B.
    pub fn accumulated_with(&self, effect: &Self) -> Self {
        effect.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{FilterErrorKind, reserve};
    #[test]
    fn unrepresentable_filter_capacity_returns_a_typed_error() {
        assert_eq!(
            reserve(usize::MAX).unwrap_err().kind(),
            FilterErrorKind::Capacity
        );
    }
}
