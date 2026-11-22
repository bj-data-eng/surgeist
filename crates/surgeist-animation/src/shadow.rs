//! Checked normalized box/text shadow lists and their CSS combination rules.
//!
//! Callers resolve lengths to CSS pixels and colors to explicit common spaces.
//! Input range checks are separate from finite sampled geometry: extrapolated
//! blur, spread and alpha remain unclamped until their consumer applies range
//! policy. Filter drop shadows use standard deviation rather than blur radius
//! and are not admitted by this box/text input boundary.

use core::fmt;
use std::sync::Arc;

use crate::interpolation::{
    InterpolableColor, InterpolationComponent, InterpolationProgress, checked_interpolate,
    interpolate_color,
};

/// The selected property's normalized shadow shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowKind {
    Box,
    /// Selected Text Decoration 4 permits inset and nonnegative spread.
    Text,
}

/// The geometry or color component rejected by checked shadow arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowComponent {
    OffsetX,
    OffsetY,
    BlurRadius,
    Spread,
    Color(InterpolationComponent),
}

/// The reason normalized input or a shadow combination was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowErrorKind {
    NonFinite,
    NegativeInput,
    InvalidInputAlpha,
    MismatchedKinds,
    MismatchedColorSpaces,
    Capacity,
}

/// A whole-value failure, optionally identifying its shadow and component.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowError {
    kind: ShadowErrorKind,
    shadow_index: Option<usize>,
    component: Option<ShadowComponent>,
    value: Option<f64>,
}

impl ShadowError {
    fn new(kind: ShadowErrorKind) -> Self {
        Self {
            kind,
            shadow_index: None,
            component: None,
            value: None,
        }
    }
    fn component(mut self, component: ShadowComponent, value: f64) -> Self {
        self.component = Some(component);
        self.value = Some(value);
        self
    }
    fn at(mut self, index: usize) -> Self {
        self.shadow_index = Some(index);
        self
    }
    pub const fn kind(&self) -> ShadowErrorKind {
        self.kind
    }
    pub const fn shadow_index(&self) -> Option<usize> {
        self.shadow_index
    }
    pub const fn failed_component(&self) -> Option<ShadowComponent> {
        self.component
    }
    pub const fn value(&self) -> Option<f64> {
        self.value
    }
}
impl fmt::Display for ShadowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shadow {:?}", self.kind)?;
        if let Some(index) = self.shadow_index {
            write!(f, " at shadow {index}")?;
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
impl std::error::Error for ShadowError {}

const GEOMETRY_COMPONENTS: [ShadowComponent; 4] = [
    ShadowComponent::OffsetX,
    ShadowComponent::OffsetY,
    ShadowComponent::BlurRadius,
    ShadowComponent::Spread,
];

/// Finite CSS-pixel x/y offsets, blur radius and spread, in that order.
/// Sampled geometry can be negative; normalized inputs impose their own ranges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowGeometry([f64; 4]);
impl ShadowGeometry {
    pub fn new(components: [f64; 4]) -> Result<Self, ShadowError> {
        for (value, component) in components.into_iter().zip(GEOMETRY_COMPONENTS) {
            if !value.is_finite() {
                return Err(
                    ShadowError::new(ShadowErrorKind::NonFinite).component(component, value)
                );
            }
        }
        Ok(Self(components))
    }
    pub const fn components(self) -> [f64; 4] {
        self.0
    }
}

/// One finite sampled shadow, retaining its explicit premultiplied color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    geometry: ShadowGeometry,
    color: InterpolableColor,
    inset: bool,
}
impl Shadow {
    pub const fn geometry(&self) -> ShadowGeometry {
        self.geometry
    }
    pub const fn color(&self) -> InterpolableColor {
        self.color
    }
    pub const fn inset(&self) -> bool {
        self.inset
    }

    fn blank_like(self) -> Result<Self, ShadowError> {
        let color = InterpolableColor::from_sampled_premultiplied_components(
            self.color.space(),
            0.,
            0.,
            0.,
            0.,
        )
        .map_err(|_| ShadowError::new(ShadowErrorKind::NonFinite))?;
        Ok(Self {
            geometry: ShadowGeometry([0.; 4]),
            color,
            inset: self.inset,
        })
    }
}

/// A guarded normalized endpoint, before interpolation or accumulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowInput {
    kind: ShadowKind,
    shadow: Shadow,
}
impl ShadowInput {
    /// Guards nonnegative blur, Text spread and ordinary endpoint alpha.
    /// Length resolution, currentcolor and interpolation-space selection are caller-owned.
    pub fn new(
        kind: ShadowKind,
        geometry: ShadowGeometry,
        color: InterpolableColor,
        inset: bool,
    ) -> Result<Self, ShadowError> {
        let [_, _, blur, spread] = geometry.0;
        for (value, component) in [
            (blur, ShadowComponent::BlurRadius),
            (spread, ShadowComponent::Spread),
        ] {
            if value < 0. && (component == ShadowComponent::BlurRadius || kind == ShadowKind::Text)
            {
                return Err(
                    ShadowError::new(ShadowErrorKind::NegativeInput).component(component, value)
                );
            }
        }
        if !(0. ..=1.).contains(&color.alpha()) {
            return Err(
                ShadowError::new(ShadowErrorKind::InvalidInputAlpha).component(
                    ShadowComponent::Color(InterpolationComponent::ColorAlpha),
                    color.alpha(),
                ),
            );
        }
        Ok(Self {
            kind,
            shadow: Shadow {
                geometry,
                color,
                inset,
            },
        })
    }
}

/// An immutable complete shadow value; an empty list represents `none`.
/// Normalized inputs are guarded, while operation results retain finite extended
/// geometry and alpha and can be reused directly as computed endpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowList {
    kind: ShadowKind,
    shadows: Arc<[Shadow]>,
}

fn reserve(length: usize) -> Result<Vec<Shadow>, ShadowError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| ShadowError::new(ShadowErrorKind::Capacity))?;
    Ok(result)
}

impl ShadowList {
    pub fn new(kind: ShadowKind, inputs: Vec<ShadowInput>) -> Result<Self, ShadowError> {
        let mut shadows = reserve(inputs.len())?;
        for (index, input) in inputs.into_iter().enumerate() {
            if input.kind != kind {
                return Err(ShadowError::new(ShadowErrorKind::MismatchedKinds).at(index));
            }
            shadows.push(input.shadow);
        }
        Ok(Self {
            kind,
            shadows: shadows.into(),
        })
    }
    pub const fn kind(&self) -> ShadowKind {
        self.kind
    }
    pub fn shadows(&self) -> &[Shadow] {
        &self.shadows
    }

    fn require_kind(&self, other: &Self) -> Result<(), ShadowError> {
        if self.kind == other.kind {
            Ok(())
        } else {
            Err(ShadowError::new(ShadowErrorKind::MismatchedKinds))
        }
    }
    fn inset_mismatch(&self, other: &Self) -> bool {
        self.shadows
            .iter()
            .zip(other.shadows.iter())
            .any(|(a, b)| a.inset != b.inset)
    }
    fn pairs<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = Result<(Shadow, Shadow), ShadowError>> + 'a {
        // Chain each side's unmatched tail without indexing or fabricating an
        // authored shadow. Padding copies inset and space from its real partner.
        let common = self
            .shadows
            .iter()
            .zip(other.shadows.iter())
            .map(|(a, b)| Ok((*a, *b)));
        let from_tail = self
            .shadows
            .iter()
            .skip(other.shadows.len())
            .map(|a| a.blank_like().map(|b| (*a, b)));
        let to_tail = other
            .shadows
            .iter()
            .skip(self.shadows.len())
            .map(|b| b.blank_like().map(|a| (a, *b)));
        common.chain(from_tail).chain(to_tail)
    }
    /// Interpolates paired entries, padding at the end. An inset mismatch
    /// switches the complete original endpoint at progress 0.5.
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShadowError> {
        self.require_kind(other)?;
        if self.inset_mismatch(other) {
            return Ok(if progress.value() < 0.5 {
                self.clone()
            } else {
                other.clone()
            });
        }
        let mut shadows = reserve(self.shadows.len().max(other.shadows.len()))?;
        for (index, pair) in self.pairs(other).enumerate() {
            let result = (|| {
                let (a, b) = pair?;
                if a.color.space() != b.color.space() {
                    return Err(ShadowError::new(ShadowErrorKind::MismatchedColorSpaces));
                }
                let mut geometry = [0.; 4];
                for (((value, a), b), component) in geometry
                    .iter_mut()
                    .zip(a.geometry.0)
                    .zip(b.geometry.0)
                    .zip(GEOMETRY_COMPONENTS)
                {
                    *value = checked_interpolate(a, b, progress.value()).map_err(|value| {
                        ShadowError::new(ShadowErrorKind::NonFinite).component(component, value)
                    })?;
                }
                let color = interpolate_color(a.color, b.color, progress).map_err(
                    |(component, value)| {
                        ShadowError::new(ShadowErrorKind::NonFinite)
                            .component(ShadowComponent::Color(component), value)
                    },
                )?;
                Ok(Shadow {
                    geometry: ShadowGeometry(geometry),
                    color,
                    inset: a.inset,
                })
            })();
            shadows.push(result.map_err(|error: ShadowError| error.at(index))?);
        }
        Ok(Self {
            kind: self.kind,
            shadows: shadows.into(),
        })
    }
    /// Concatenates underlying first, without matching or changing colors.
    pub fn added_to(&self, other: &Self) -> Result<Self, ShadowError> {
        self.require_kind(other)?;
        let length = self
            .shadows
            .len()
            .checked_add(other.shadows.len())
            .ok_or_else(|| ShadowError::new(ShadowErrorKind::Capacity))?;
        let mut shadows = reserve(length)?;
        shadows.extend(self.shadows.iter().chain(other.shadows.iter()).copied());
        Ok(Self {
            kind: self.kind,
            shadows: shadows.into(),
        })
    }
    /// Adds padded geometry componentwise. Colors are not additive and select
    /// the effect color. Inset mismatch selects the complete effect value.
    pub fn accumulated_with(&self, effect: &Self) -> Result<Self, ShadowError> {
        self.require_kind(effect)?;
        if self.inset_mismatch(effect) {
            return Ok(effect.clone());
        }
        let mut shadows = reserve(self.shadows.len().max(effect.shadows.len()))?;
        for (index, pair) in self.pairs(effect).enumerate() {
            let (a, b) = pair.map_err(|error| error.at(index))?;
            let mut geometry = [0.; 4];
            for (((value, a), b), component) in geometry
                .iter_mut()
                .zip(a.geometry.0)
                .zip(b.geometry.0)
                .zip(GEOMETRY_COMPONENTS)
            {
                *value = a + b;
                if !value.is_finite() {
                    return Err(ShadowError::new(ShadowErrorKind::NonFinite)
                        .component(component, *value)
                        .at(index));
                }
            }
            shadows.push(Shadow {
                geometry: ShadowGeometry(geometry),
                color: b.color,
                inset: a.inset,
            });
        }
        Ok(Self {
            kind: self.kind,
            shadows: shadows.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ShadowErrorKind, reserve};
    #[test]
    fn unrepresentable_capacity_is_a_typed_failure() {
        assert_eq!(
            reserve(usize::MAX).unwrap_err().kind(),
            ShadowErrorKind::Capacity
        );
    }
}
