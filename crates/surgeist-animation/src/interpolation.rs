//! Interpolable value contracts and checked sampling.
//!
//! This module owns neutral values that can be sampled without property-specific
//! clamping. Percentages are unrestricted finite ratios, colors carry an
//! explicit interpolation space and premultiplied components, and arithmetic
//! overflow is reported separately from permanently unsupported interpolation.
//!
//! Root and consumers own downstream color-space conversion, gamut mapping or
//! clipping, unpremultiplication, missing-component resolution, and final
//! property range clamping. This module interpolates supplied premultiplied
//! components and alpha directly without conversion or clipping.

use core::fmt;
use std::sync::Arc;

use crate::{EasedProgress, NumericError, NumericInput, UnitRatio};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyKey {
    name: Arc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueFamily {
    Number,
    Percentage,
    Color,
    Discrete,
    Transform,
    Composite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorInterpolationSpace {
    Srgb,
    Oklab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterpolableNumber {
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterpolablePercentage {
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterpolableColor {
    space: ColorInterpolationSpace,
    premultiplied_red: f64,
    premultiplied_green: f64,
    premultiplied_blue: f64,
    alpha: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscreteValue {
    token: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformValue {
    kind: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeValue {
    kind: Arc<str>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolableValue {
    Number(InterpolableNumber),
    Percentage(InterpolablePercentage),
    Color(InterpolableColor),
    Discrete(DiscreteValue),
    Transform(TransformValue),
    Composite(CompositeValue),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterpolationProgress {
    value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpolationPair {
    property: PropertyKey,
    endpoints: Arc<InterpolationEndpoints>,
}

#[derive(Debug, Clone, PartialEq)]
struct InterpolationEndpoints {
    from: InterpolableValue,
    to: InterpolableValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationOutcome {
    Value(InterpolableValue),
    ArithmeticFailure(NonFiniteInterpolationResult),
    Unsupported(InterpolationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationComponent {
    Number,
    Percentage,
    ColorPremultipliedRed,
    ColorPremultipliedGreen,
    ColorPremultipliedBlue,
    ColorAlpha,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonFiniteInterpolationResult {
    property: PropertyKey,
    family: ValueFamily,
    endpoints: Arc<InterpolationEndpoints>,
    progress: InterpolationProgress,
    component: InterpolationComponent,
    value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationError {
    EmptyPropertyKey,
    EmptyDiscreteValue,
    NonFiniteNumber {
        value: f64,
    },
    Numeric(NumericError),
    NonFiniteInterpolationResult(NonFiniteInterpolationResult),
    MismatchedFamilies {
        property: PropertyKey,
        from: ValueFamily,
        to: ValueFamily,
        from_value: InterpolableValue,
        to_value: InterpolableValue,
    },
    MismatchedColorSpaces {
        property: PropertyKey,
        from: ColorInterpolationSpace,
        to: ColorInterpolationSpace,
        from_value: InterpolableColor,
        to_value: InterpolableColor,
    },
    UnsupportedFamily {
        property: PropertyKey,
        family: ValueFamily,
        from: InterpolableValue,
        to: InterpolableValue,
        reason: &'static str,
    },
    NonFiniteProgress {
        value: f64,
    },
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPropertyKey => f.write_str("property key must not be empty"),
            Self::EmptyDiscreteValue => f.write_str("discrete value token must not be empty"),
            Self::NonFiniteNumber { value } => {
                write!(f, "interpolable number must be finite, got {value}")
            }
            Self::Numeric(error) => {
                write!(f, "numeric interpolation input is invalid: {error}")
            }
            Self::NonFiniteInterpolationResult(result) => write!(
                f,
                "interpolating {} produced non-finite {} component at progress {}: {}",
                result.property().as_str(),
                interpolation_component_name(result.component()),
                result.progress().value(),
                result.value()
            ),
            Self::MismatchedFamilies {
                property, from, to, ..
            } => write!(
                f,
                "property {} cannot interpolate {} to {}",
                property.as_str(),
                value_family_name(*from),
                value_family_name(*to)
            ),
            Self::MismatchedColorSpaces {
                property, from, to, ..
            } => write!(
                f,
                "property {} cannot interpolate colors in {} and {} spaces",
                property.as_str(),
                color_space_name(*from),
                color_space_name(*to)
            ),
            Self::UnsupportedFamily {
                property,
                family,
                reason,
                ..
            } => write!(
                f,
                "property {} does not support {} interpolation: {reason}",
                property.as_str(),
                value_family_name(*family)
            ),
            Self::NonFiniteProgress { value } => {
                write!(f, "interpolation progress must be finite, got {value}")
            }
        }
    }
}

impl std::error::Error for InterpolationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Numeric(error) => Some(error),
            _ => None,
        }
    }
}

fn value_family_name(family: ValueFamily) -> &'static str {
    match family {
        ValueFamily::Number => "number",
        ValueFamily::Percentage => "percentage",
        ValueFamily::Color => "color",
        ValueFamily::Discrete => "discrete",
        ValueFamily::Transform => "transform",
        ValueFamily::Composite => "composite",
    }
}

fn color_space_name(space: ColorInterpolationSpace) -> &'static str {
    match space {
        ColorInterpolationSpace::Srgb => "srgb",
        ColorInterpolationSpace::Oklab => "oklab",
    }
}

fn interpolation_component_name(component: InterpolationComponent) -> &'static str {
    match component {
        InterpolationComponent::Number => "number",
        InterpolationComponent::Percentage => "percentage",
        InterpolationComponent::ColorPremultipliedRed => "color premultiplied red",
        InterpolationComponent::ColorPremultipliedGreen => "color premultiplied green",
        InterpolationComponent::ColorPremultipliedBlue => "color premultiplied blue",
        InterpolationComponent::ColorAlpha => "color alpha",
    }
}

impl PropertyKey {
    pub fn new(name: impl Into<String>) -> Result<Self, InterpolationError> {
        let name = name.into();
        if text_is_blank(&name) {
            return Err(InterpolationError::EmptyPropertyKey);
        }

        Ok(Self { name: name.into() })
    }

    /// Copies nonblank text into owned shared storage, preserving its exact contents.
    /// Empty and Unicode-whitespace-only text returns `EmptyPropertyKey`.
    pub fn from_text(name: &str) -> Result<Self, InterpolationError> {
        if text_is_blank(name) {
            return Err(InterpolationError::EmptyPropertyKey);
        }

        Ok(Self {
            name: Arc::from(name),
        })
    }

    /// Takes ownership of nonblank shared text without copying or allocating.
    /// Empty and Unicode-whitespace-only text returns `EmptyPropertyKey`.
    pub fn from_shared(name: Arc<str>) -> Result<Self, InterpolationError> {
        if text_is_blank(&name) {
            return Err(InterpolationError::EmptyPropertyKey);
        }

        Ok(Self { name })
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl InterpolableNumber {
    pub fn new(value: f64) -> Result<Self, InterpolationError> {
        if !value.is_finite() {
            return Err(InterpolationError::NonFiniteNumber { value });
        }

        Ok(Self { value })
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl InterpolablePercentage {
    pub fn new(value: f64) -> Result<Self, InterpolationError> {
        if !value.is_finite() {
            return Err(InterpolationError::Numeric(NumericError::non_finite(
                NumericInput::Percentage,
            )));
        }

        Ok(Self { value })
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl InterpolableColor {
    pub fn from_straight_components(
        space: ColorInterpolationSpace,
        red: f64,
        green: f64,
        blue: f64,
        alpha: UnitRatio,
    ) -> Result<Self, InterpolationError> {
        let red = finite_color_component(red)?;
        let green = finite_color_component(green)?;
        let blue = finite_color_component(blue)?;
        let alpha = alpha.value();
        let (premultiplied_red, premultiplied_green, premultiplied_blue) = if alpha == 0.0 {
            (0.0, 0.0, 0.0)
        } else {
            (red * alpha, green * alpha, blue * alpha)
        };

        Self::from_sampled_premultiplied_components(
            space,
            premultiplied_red,
            premultiplied_green,
            premultiplied_blue,
            alpha,
        )
    }

    pub(crate) fn from_sampled_premultiplied_components(
        space: ColorInterpolationSpace,
        premultiplied_red: f64,
        premultiplied_green: f64,
        premultiplied_blue: f64,
        alpha: f64,
    ) -> Result<Self, InterpolationError> {
        Ok(Self {
            space,
            premultiplied_red: finite_color_component(premultiplied_red)?,
            premultiplied_green: finite_color_component(premultiplied_green)?,
            premultiplied_blue: finite_color_component(premultiplied_blue)?,
            alpha: finite_color_component(alpha)?,
        })
    }

    pub const fn space(self) -> ColorInterpolationSpace {
        self.space
    }

    pub const fn premultiplied_components(self) -> (f64, f64, f64) {
        (
            self.premultiplied_red,
            self.premultiplied_green,
            self.premultiplied_blue,
        )
    }

    pub const fn alpha(self) -> f64 {
        self.alpha
    }
}

impl DiscreteValue {
    pub fn new(token: impl Into<String>) -> Result<Self, InterpolationError> {
        let token = token.into();
        if text_is_blank(&token) {
            return Err(InterpolationError::EmptyDiscreteValue);
        }

        Ok(Self {
            token: token.into(),
        })
    }

    /// Copies nonblank text into owned shared storage, preserving its exact contents.
    /// Empty and Unicode-whitespace-only text returns `EmptyDiscreteValue`.
    pub fn from_text(token: &str) -> Result<Self, InterpolationError> {
        if text_is_blank(token) {
            return Err(InterpolationError::EmptyDiscreteValue);
        }

        Ok(Self {
            token: Arc::from(token),
        })
    }

    /// Takes ownership of nonblank shared text without copying or allocating.
    /// Empty and Unicode-whitespace-only text returns `EmptyDiscreteValue`.
    pub fn from_shared(token: Arc<str>) -> Result<Self, InterpolationError> {
        if text_is_blank(&token) {
            return Err(InterpolationError::EmptyDiscreteValue);
        }

        Ok(Self { token })
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

impl TransformValue {
    pub fn unsupported(kind: impl Into<String>) -> Self {
        Self {
            kind: unsupported_kind(kind),
        }
    }

    /// Copies exact marker text; empty or Unicode-whitespace-only text becomes `unsupported`.
    pub fn unsupported_from_text(kind: &str) -> Self {
        Self {
            kind: Arc::from(unsupported_text(kind)),
        }
    }

    /// Takes shared marker text, allocating only for empty or Unicode-whitespace-only fallback.
    pub fn unsupported_from_shared(kind: Arc<str>) -> Self {
        Self {
            kind: unsupported_shared(kind),
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
}

impl CompositeValue {
    pub fn unsupported(kind: impl Into<String>) -> Self {
        Self {
            kind: unsupported_kind(kind),
        }
    }

    /// Copies exact marker text; empty or Unicode-whitespace-only text becomes `unsupported`.
    pub fn unsupported_from_text(kind: &str) -> Self {
        Self {
            kind: Arc::from(unsupported_text(kind)),
        }
    }

    /// Takes shared marker text, allocating only for empty or Unicode-whitespace-only fallback.
    pub fn unsupported_from_shared(kind: Arc<str>) -> Self {
        Self {
            kind: unsupported_shared(kind),
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
}

impl InterpolableValue {
    pub const fn number(value: InterpolableNumber) -> Self {
        Self::Number(value)
    }

    pub const fn percentage(value: InterpolablePercentage) -> Self {
        Self::Percentage(value)
    }

    pub const fn color(value: InterpolableColor) -> Self {
        Self::Color(value)
    }

    pub fn discrete(value: DiscreteValue) -> Self {
        Self::Discrete(value)
    }

    pub fn transform(value: TransformValue) -> Self {
        Self::Transform(value)
    }

    pub fn composite(value: CompositeValue) -> Self {
        Self::Composite(value)
    }

    pub const fn family(&self) -> ValueFamily {
        match self {
            Self::Number(_) => ValueFamily::Number,
            Self::Percentage(_) => ValueFamily::Percentage,
            Self::Color(_) => ValueFamily::Color,
            Self::Discrete(_) => ValueFamily::Discrete,
            Self::Transform(_) => ValueFamily::Transform,
            Self::Composite(_) => ValueFamily::Composite,
        }
    }
}

impl NonFiniteInterpolationResult {
    fn new(
        pair: &InterpolationPair,
        progress: InterpolationProgress,
        component: InterpolationComponent,
        value: f64,
    ) -> Self {
        Self {
            property: pair.property.clone(),
            family: pair.endpoints.from.family(),
            endpoints: Arc::clone(&pair.endpoints),
            progress,
            component,
            value,
        }
    }

    pub const fn property(&self) -> &PropertyKey {
        &self.property
    }

    pub const fn family(&self) -> ValueFamily {
        self.family
    }

    pub fn from(&self) -> &InterpolableValue {
        &self.endpoints.from
    }

    pub fn to(&self) -> &InterpolableValue {
        &self.endpoints.to
    }

    pub const fn progress(&self) -> InterpolationProgress {
        self.progress
    }

    pub const fn component(&self) -> InterpolationComponent {
        self.component
    }

    pub const fn value(&self) -> f64 {
        self.value
    }
}

impl InterpolationProgress {
    pub fn new(value: f64) -> Result<Self, InterpolationError> {
        if !value.is_finite() {
            return Err(InterpolationError::NonFiniteProgress { value });
        }

        Ok(Self { value })
    }

    pub fn from_eased(progress: EasedProgress) -> Result<Self, InterpolationError> {
        Self::new(progress.value())
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl InterpolationPair {
    pub fn new(
        property: PropertyKey,
        from: InterpolableValue,
        to: InterpolableValue,
    ) -> Result<Self, InterpolationError> {
        let from_family = from.family();
        let to_family = to.family();

        if from_family != to_family {
            return Err(InterpolationError::MismatchedFamilies {
                property,
                from: from_family,
                to: to_family,
                from_value: from,
                to_value: to,
            });
        }

        match (&from, &to) {
            (InterpolableValue::Color(from_color), InterpolableValue::Color(to_color))
                if from_color.space() != to_color.space() =>
            {
                return Err(InterpolationError::MismatchedColorSpaces {
                    property,
                    from: from_color.space(),
                    to: to_color.space(),
                    from_value: *from_color,
                    to_value: *to_color,
                });
            }
            _ => {}
        }

        Ok(Self {
            property,
            endpoints: Arc::new(InterpolationEndpoints { from, to }),
        })
    }

    pub fn property(&self) -> &PropertyKey {
        &self.property
    }

    pub fn from(&self) -> &InterpolableValue {
        &self.endpoints.from
    }

    pub fn to(&self) -> &InterpolableValue {
        &self.endpoints.to
    }

    pub fn sample(&self, progress: InterpolationProgress) -> InterpolationOutcome {
        match (&self.endpoints.from, &self.endpoints.to) {
            (InterpolableValue::Number(from), InterpolableValue::Number(to)) => {
                self.sample_number(*from, *to, progress)
            }
            (InterpolableValue::Percentage(from), InterpolableValue::Percentage(to)) => {
                self.sample_percentage(*from, *to, progress)
            }
            (InterpolableValue::Color(from), InterpolableValue::Color(to)) => {
                self.sample_color(*from, *to, progress)
            }
            (InterpolableValue::Discrete(from), InterpolableValue::Discrete(to)) => {
                let value = if progress.value() < 0.5 { from } else { to };
                InterpolationOutcome::Value(InterpolableValue::discrete(value.clone()))
            }
            (InterpolableValue::Transform(_), InterpolableValue::Transform(_)) => {
                self.unsupported(ValueFamily::Transform, "transform interpolation deferred")
            }
            (InterpolableValue::Composite(_), InterpolableValue::Composite(_)) => {
                self.unsupported(ValueFamily::Composite, "composite interpolation deferred")
            }
            _ => unreachable!("InterpolationPair::new rejects mismatched value families"),
        }
    }

    fn sample_color(
        &self,
        from: InterpolableColor,
        to: InterpolableColor,
        progress: InterpolationProgress,
    ) -> InterpolationOutcome {
        if let Some(endpoint) = self.exact_endpoint(progress) {
            return endpoint;
        }

        let (from_red, from_green, from_blue) = from.premultiplied_components();
        let (to_red, to_green, to_blue) = to.premultiplied_components();
        let red = match checked_color_component(
            from_red,
            to_red,
            progress,
            InterpolationComponent::ColorPremultipliedRed,
        ) {
            Ok(value) => value,
            Err((component, value)) => return self.arithmetic_failure(progress, component, value),
        };
        let green = match checked_color_component(
            from_green,
            to_green,
            progress,
            InterpolationComponent::ColorPremultipliedGreen,
        ) {
            Ok(value) => value,
            Err((component, value)) => return self.arithmetic_failure(progress, component, value),
        };
        let blue = match checked_color_component(
            from_blue,
            to_blue,
            progress,
            InterpolationComponent::ColorPremultipliedBlue,
        ) {
            Ok(value) => value,
            Err((component, value)) => return self.arithmetic_failure(progress, component, value),
        };
        let alpha = match checked_color_component(
            from.alpha(),
            to.alpha(),
            progress,
            InterpolationComponent::ColorAlpha,
        ) {
            Ok(value) => value,
            Err((component, value)) => return self.arithmetic_failure(progress, component, value),
        };

        match InterpolableColor::from_sampled_premultiplied_components(
            from.space(),
            red,
            green,
            blue,
            alpha,
        ) {
            Ok(value) => InterpolationOutcome::Value(InterpolableValue::color(value)),
            Err(_) => self.arithmetic_failure(progress, InterpolationComponent::ColorAlpha, alpha),
        }
    }

    fn sample_number(
        &self,
        from: InterpolableNumber,
        to: InterpolableNumber,
        progress: InterpolationProgress,
    ) -> InterpolationOutcome {
        if let Some(endpoint) = self.exact_endpoint(progress) {
            return endpoint;
        }

        match checked_interpolate(from.value(), to.value(), progress.value()) {
            Ok(value) => match InterpolableNumber::new(value) {
                Ok(value) => InterpolationOutcome::Value(InterpolableValue::number(value)),
                Err(_) => self.arithmetic_failure(progress, InterpolationComponent::Number, value),
            },
            Err(value) => self.arithmetic_failure(progress, InterpolationComponent::Number, value),
        }
    }

    fn sample_percentage(
        &self,
        from: InterpolablePercentage,
        to: InterpolablePercentage,
        progress: InterpolationProgress,
    ) -> InterpolationOutcome {
        if let Some(endpoint) = self.exact_endpoint(progress) {
            return endpoint;
        }

        match checked_interpolate(from.value(), to.value(), progress.value()) {
            Ok(value) => match InterpolablePercentage::new(value) {
                Ok(value) => InterpolationOutcome::Value(InterpolableValue::percentage(value)),
                Err(_) => {
                    self.arithmetic_failure(progress, InterpolationComponent::Percentage, value)
                }
            },
            Err(value) => {
                self.arithmetic_failure(progress, InterpolationComponent::Percentage, value)
            }
        }
    }

    fn exact_endpoint(&self, progress: InterpolationProgress) -> Option<InterpolationOutcome> {
        if progress.value() == 0.0 {
            return Some(InterpolationOutcome::Value(self.endpoints.from.clone()));
        }

        if progress.value() == 1.0 {
            return Some(InterpolationOutcome::Value(self.endpoints.to.clone()));
        }

        None
    }

    fn arithmetic_failure(
        &self,
        progress: InterpolationProgress,
        component: InterpolationComponent,
        value: f64,
    ) -> InterpolationOutcome {
        InterpolationOutcome::ArithmeticFailure(NonFiniteInterpolationResult::new(
            self, progress, component, value,
        ))
    }

    fn unsupported(&self, family: ValueFamily, reason: &'static str) -> InterpolationOutcome {
        InterpolationOutcome::Unsupported(InterpolationError::UnsupportedFamily {
            property: self.property.clone(),
            family,
            from: self.endpoints.from.clone(),
            to: self.endpoints.to.clone(),
            reason,
        })
    }
}

fn text_is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn unsupported_text(kind: &str) -> &str {
    if text_is_blank(kind) {
        "unsupported"
    } else {
        kind
    }
}

fn unsupported_shared(kind: Arc<str>) -> Arc<str> {
    if text_is_blank(&kind) {
        Arc::from("unsupported")
    } else {
        kind
    }
}

fn unsupported_kind(kind: impl Into<String>) -> Arc<str> {
    let kind = kind.into();
    if text_is_blank(&kind) {
        Arc::from("unsupported")
    } else {
        kind.into()
    }
}

fn finite_color_component(value: f64) -> Result<f64, InterpolationError> {
    if !value.is_finite() {
        return Err(InterpolationError::NonFiniteNumber { value });
    }

    Ok(value)
}

fn checked_interpolate(from: f64, to: f64, progress: f64) -> Result<f64, f64> {
    let delta = to - from;
    if !delta.is_finite() {
        return Err(delta);
    }

    let scaled_delta = delta * progress;
    if !scaled_delta.is_finite() {
        return Err(scaled_delta);
    }

    let value = from + scaled_delta;
    if !value.is_finite() {
        return Err(value);
    }

    Ok(value)
}

fn checked_color_component(
    from: f64,
    to: f64,
    progress: InterpolationProgress,
    component: InterpolationComponent,
) -> Result<f64, (InterpolationComponent, f64)> {
    checked_interpolate(from, to, progress.value()).map_err(|value| (component, value))
}

#[cfg(test)]
mod tests {
    use super::{
        ColorInterpolationSpace, CompositeValue, DiscreteValue, InterpolableColor,
        InterpolableNumber, InterpolablePercentage, InterpolableValue, InterpolationComponent,
        InterpolationError, InterpolationOutcome, InterpolationPair, InterpolationProgress,
        NonFiniteInterpolationResult, PropertyKey, TransformValue, ValueFamily,
    };
    use crate::{NumericError, NumericInput};

    #[test]
    fn property_key_rejects_empty_names_and_preserves_identity() {
        assert_eq!(
            PropertyKey::new(""),
            Err(InterpolationError::EmptyPropertyKey)
        );
        assert_eq!(
            PropertyKey::new("   "),
            Err(InterpolationError::EmptyPropertyKey)
        );
        assert_eq!(PropertyKey::new("opacity").unwrap().as_str(), "opacity");
        assert_eq!(PropertyKey::new("opacity"), PropertyKey::new("opacity"));
        assert_ne!(PropertyKey::new("opacity"), PropertyKey::new("transform"));
    }

    #[test]
    fn value_families_are_explicit() {
        assert_eq!(
            InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()).family(),
            ValueFamily::Number
        );
        assert_eq!(
            InterpolableValue::percentage(InterpolablePercentage::new(0.5).unwrap()).family(),
            ValueFamily::Percentage
        );
        assert_eq!(
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.1,
                    0.2,
                    0.3,
                    crate::UnitRatio::new(1.0).unwrap(),
                )
                .unwrap(),
            )
            .family(),
            ValueFamily::Color
        );
        assert_eq!(
            InterpolableValue::discrete(DiscreteValue::new("auto").unwrap()).family(),
            ValueFamily::Discrete
        );
        assert_eq!(
            InterpolableValue::transform(TransformValue::unsupported("translate")).family(),
            ValueFamily::Transform
        );
        assert_eq!(
            InterpolableValue::composite(CompositeValue::unsupported("stack")).family(),
            ValueFamily::Composite
        );
    }

    #[test]
    fn numeric_values_validate_finite_inputs() {
        assert_eq!(InterpolableNumber::new(1.25).unwrap().value(), 1.25);
        assert!(matches!(
            InterpolableNumber::new(f64::INFINITY),
            Err(InterpolationError::NonFiniteNumber { value }) if value.is_infinite()
        ));
    }

    #[test]
    fn color_preserves_explicit_interpolation_space() {
        let color = InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Oklab,
            0.4,
            0.2,
            -0.1,
            crate::UnitRatio::new(0.5).unwrap(),
        )
        .unwrap();

        assert_eq!(color.space(), ColorInterpolationSpace::Oklab);
        assert_eq!(color.premultiplied_components(), (0.2, 0.1, -0.05));
        assert_eq!(color.alpha(), 0.5);
    }

    #[test]
    fn color_accepts_finite_out_of_range_components_and_rejects_non_finite_components() {
        let color = InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Srgb,
            -0.25,
            1.5,
            2.0,
            crate::UnitRatio::new(0.5).unwrap(),
        )
        .unwrap();

        assert_eq!(color.premultiplied_components(), (-0.125, 0.75, 1.0));
        assert!(matches!(
            InterpolableColor::from_straight_components(
                ColorInterpolationSpace::Srgb,
                f64::NAN,
                0.0,
                0.0,
                crate::UnitRatio::new(1.0).unwrap(),
            ),
            Err(InterpolationError::NonFiniteNumber { value }) if value.is_nan()
        ));
        assert!(matches!(
            InterpolableColor::from_straight_components(
                ColorInterpolationSpace::Srgb,
                0.0,
                f64::INFINITY,
                0.0,
                crate::UnitRatio::new(1.0).unwrap(),
            ),
            Err(InterpolationError::NonFiniteNumber { value }) if value.is_infinite()
        ));
    }

    #[test]
    fn zero_alpha_straight_endpoint_canonicalizes_components_to_zero() {
        let color = InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Srgb,
            f64::MAX,
            -12.0,
            3.5,
            crate::UnitRatio::new(0.0).unwrap(),
        )
        .unwrap();

        assert_eq!(color.premultiplied_components(), (0.0, 0.0, 0.0));
        assert_eq!(color.alpha(), 0.0);
    }

    #[test]
    fn unrestricted_percentages_accept_negative_and_greater_than_one_finite_values() {
        assert_eq!(InterpolablePercentage::new(-0.25).unwrap().value(), -0.25);
        assert_eq!(InterpolablePercentage::new(1.75).unwrap().value(), 1.75);
        assert_eq!(
            InterpolableValue::percentage(InterpolablePercentage::new(2.5).unwrap()).family(),
            ValueFamily::Percentage
        );
    }

    #[test]
    fn percentage_rejects_non_finite_values_with_percentage_diagnostics() {
        let expected = Err(InterpolationError::Numeric(NumericError::non_finite(
            NumericInput::Percentage,
        )));

        assert_eq!(InterpolablePercentage::new(f64::NAN), expected);
        assert_eq!(InterpolablePercentage::new(f64::INFINITY), expected);
        assert_eq!(InterpolablePercentage::new(f64::NEG_INFINITY), expected);
    }

    #[test]
    fn discrete_and_unsupported_marker_values_have_named_identity() {
        assert_eq!(DiscreteValue::new("block").unwrap().token(), "block");
        assert_eq!(TransformValue::unsupported("").kind(), "unsupported");
        assert_eq!(CompositeValue::unsupported("add").kind(), "add");
    }

    #[test]
    fn numeric_interpolation_supports_easing_overshoot() {
        let pair = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(10.0).unwrap()),
        )
        .unwrap();

        assert_eq!(
            pair.sample(InterpolationProgress::new(1.25).unwrap()),
            InterpolationOutcome::Value(InterpolableValue::number(
                InterpolableNumber::new(12.5).unwrap()
            ))
        );
    }

    #[test]
    fn numeric_interior_overflow_returns_typed_arithmetic_failure_and_does_not_panic() {
        let pair = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(f64::MAX).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(-f64::MAX).unwrap()),
        )
        .unwrap();

        let sample =
            std::panic::catch_unwind(|| pair.sample(InterpolationProgress::new(0.5).unwrap()))
                .expect("finite scalar interpolation must not panic on overflow");

        assert_arithmetic_failure(
            sample,
            "opacity",
            ValueFamily::Number,
            InterpolationComponent::Number,
        );
    }

    #[test]
    fn percentage_interpolation_preserves_negative_results_and_easing_overshoot() {
        let pair = InterpolationPair::new(
            PropertyKey::new("width-percent").unwrap(),
            InterpolableValue::percentage(InterpolablePercentage::new(-0.5).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(0.5).unwrap()),
        )
        .unwrap();

        assert_eq!(
            pair.sample(InterpolationProgress::new(0.25).unwrap()),
            InterpolationOutcome::Value(InterpolableValue::percentage(
                InterpolablePercentage::new(-0.25).unwrap()
            ))
        );
        assert_eq!(
            pair.sample(InterpolationProgress::new(1.5).unwrap()),
            InterpolationOutcome::Value(InterpolableValue::percentage(
                InterpolablePercentage::new(1.0).unwrap()
            ))
        );
    }

    #[test]
    fn scalar_and_percentage_interpolation_return_exact_extreme_endpoints() {
        let number = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(f64::MAX).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(-f64::MAX).unwrap()),
        )
        .unwrap();
        let percentage = InterpolationPair::new(
            PropertyKey::new("width").unwrap(),
            InterpolableValue::percentage(InterpolablePercentage::new(f64::MAX).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(-f64::MAX).unwrap()),
        )
        .unwrap();

        assert_eq!(
            number.sample(InterpolationProgress::new(0.0).unwrap()),
            InterpolationOutcome::Value(number.from().clone())
        );
        assert_eq!(
            number.sample(InterpolationProgress::new(1.0).unwrap()),
            InterpolationOutcome::Value(number.to().clone())
        );
        assert_eq!(
            percentage.sample(InterpolationProgress::new(0.0).unwrap()),
            InterpolationOutcome::Value(percentage.from().clone())
        );
        assert_eq!(
            percentage.sample(InterpolationProgress::new(1.0).unwrap()),
            InterpolationOutcome::Value(percentage.to().clone())
        );
    }

    #[test]
    fn percentage_interior_overflow_returns_typed_arithmetic_failure_and_does_not_panic() {
        let pair = InterpolationPair::new(
            PropertyKey::new("width").unwrap(),
            InterpolableValue::percentage(InterpolablePercentage::new(f64::MAX).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(-f64::MAX).unwrap()),
        )
        .unwrap();

        let sample =
            std::panic::catch_unwind(|| pair.sample(InterpolationProgress::new(0.5).unwrap()))
                .expect("finite percentage interpolation must not panic on overflow");

        assert_arithmetic_failure(
            sample,
            "width",
            ValueFamily::Percentage,
            InterpolationComponent::Percentage,
        );
    }

    #[test]
    fn color_pair_rejects_mismatched_spaces_with_typed_diagnostic() {
        let from = InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Srgb,
            0.1,
            0.2,
            0.3,
            crate::UnitRatio::new(1.0).unwrap(),
        )
        .unwrap();
        let to = InterpolableColor::from_straight_components(
            ColorInterpolationSpace::Oklab,
            0.1,
            0.2,
            0.3,
            crate::UnitRatio::new(1.0).unwrap(),
        )
        .unwrap();

        let error = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(from),
            InterpolableValue::color(to),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            InterpolationError::MismatchedColorSpaces {
                from: ColorInterpolationSpace::Srgb,
                to: ColorInterpolationSpace::Oklab,
                ..
            }
        ));
        if let InterpolationError::MismatchedColorSpaces {
            property,
            from_value,
            to_value,
            ..
        } = error
        {
            assert_eq!(property.as_str(), "color");
            assert_eq!(from_value, from);
            assert_eq!(to_value, to);
        }
    }

    #[test]
    fn transparent_to_opaque_color_uses_premultiplied_alpha_components() {
        let pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    1.0,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(0.0).unwrap(),
                )
                .unwrap(),
            ),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.0,
                    0.0,
                    1.0,
                    crate::UnitRatio::new(1.0).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        assert_sampled_color(
            pair.sample(InterpolationProgress::new(0.5).unwrap()),
            ColorInterpolationSpace::Srgb,
            (0.0, 0.0, 0.5),
            0.5,
        );
    }

    #[test]
    fn transparent_color_midpoint_does_not_leak_hidden_rgb() {
        let pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    1.0,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(0.0).unwrap(),
                )
                .unwrap(),
            ),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.0,
                    1.0,
                    0.0,
                    crate::UnitRatio::new(0.0).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        assert_sampled_color(
            pair.sample(InterpolationProgress::new(0.5).unwrap()),
            ColorInterpolationSpace::Srgb,
            (0.0, 0.0, 0.0),
            0.0,
        );
    }

    #[test]
    fn zero_alpha_output_preserves_premultiplied_components() {
        let pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(
                InterpolableColor::from_sampled_premultiplied_components(
                    ColorInterpolationSpace::Srgb,
                    0.25,
                    0.0,
                    0.0,
                    1.0,
                )
                .unwrap(),
            ),
            InterpolableValue::color(
                InterpolableColor::from_sampled_premultiplied_components(
                    ColorInterpolationSpace::Srgb,
                    0.75,
                    0.0,
                    0.0,
                    -1.0,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        assert_sampled_color(
            pair.sample(InterpolationProgress::new(0.5).unwrap()),
            ColorInterpolationSpace::Srgb,
            (0.5, 0.0, 0.0),
            0.0,
        );
    }

    #[test]
    fn sampled_color_reused_as_endpoint_is_not_premultiplied_twice() {
        let first_pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    1.0,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(0.5).unwrap(),
                )
                .unwrap(),
            ),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.0,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(0.0).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let InterpolationOutcome::Value(InterpolableValue::Color(sampled)) =
            first_pair.sample(InterpolationProgress::new(0.5).unwrap())
        else {
            panic!("expected sampled color");
        };

        let second_pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(sampled),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.0,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(0.0).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        assert_sampled_color(
            second_pair.sample(InterpolationProgress::new(0.5).unwrap()),
            ColorInterpolationSpace::Srgb,
            (0.125, 0.0, 0.0),
            0.125,
        );
    }

    #[test]
    fn color_overshoot_is_not_gamut_clipped() {
        let pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.25,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(1.0).unwrap(),
                )
                .unwrap(),
            ),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    0.75,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(1.0).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        assert_sampled_color(
            pair.sample(InterpolationProgress::new(2.0).unwrap()),
            ColorInterpolationSpace::Srgb,
            (1.25, 0.0, 0.0),
            1.0,
        );
    }

    #[test]
    fn color_arithmetic_overflow_returns_typed_failure_and_does_not_panic() {
        let pair = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    f64::MAX,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(1.0).unwrap(),
                )
                .unwrap(),
            ),
            InterpolableValue::color(
                InterpolableColor::from_straight_components(
                    ColorInterpolationSpace::Srgb,
                    -f64::MAX,
                    0.0,
                    0.0,
                    crate::UnitRatio::new(1.0).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let sample =
            std::panic::catch_unwind(|| pair.sample(InterpolationProgress::new(0.5).unwrap()))
                .expect("finite color interpolation must not panic on overflow");

        assert_arithmetic_failure(
            sample,
            "color",
            ValueFamily::Color,
            InterpolationComponent::ColorPremultipliedRed,
        );
    }

    #[test]
    fn discrete_interpolation_switches_at_half_progress() {
        let pair = InterpolationPair::new(
            PropertyKey::new("display").unwrap(),
            InterpolableValue::discrete(DiscreteValue::new("none").unwrap()),
            InterpolableValue::discrete(DiscreteValue::new("block").unwrap()),
        )
        .unwrap();

        assert_eq!(
            pair.sample(InterpolationProgress::new(0.49).unwrap()),
            InterpolationOutcome::Value(InterpolableValue::discrete(
                DiscreteValue::new("none").unwrap()
            ))
        );
        assert_eq!(
            pair.sample(InterpolationProgress::new(0.5).unwrap()),
            InterpolationOutcome::Value(InterpolableValue::discrete(
                DiscreteValue::new("block").unwrap()
            ))
        );
    }

    #[test]
    fn mismatched_and_unsupported_pairs_report_typed_diagnostics() {
        let mismatch_from = InterpolableValue::number(InterpolableNumber::new(0.0).unwrap());
        let mismatch_to = InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap());
        let mismatch = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            mismatch_from.clone(),
            mismatch_to.clone(),
        );
        assert!(matches!(
            mismatch,
            Err(InterpolationError::MismatchedFamilies {
                from: ValueFamily::Number,
                to: ValueFamily::Percentage,
                ..
            })
        ));
        if let Err(InterpolationError::MismatchedFamilies {
            from_value,
            to_value,
            ..
        }) = mismatch
        {
            assert_eq!(from_value, mismatch_from);
            assert_eq!(to_value, mismatch_to);
        }

        let transform = InterpolationPair::new(
            PropertyKey::new("transform").unwrap(),
            InterpolableValue::transform(TransformValue::unsupported("translate")),
            InterpolableValue::transform(TransformValue::unsupported("scale")),
        )
        .unwrap();
        assert!(matches!(
            transform.sample(InterpolationProgress::new(0.5).unwrap()),
            InterpolationOutcome::Unsupported(InterpolationError::UnsupportedFamily {
                family: ValueFamily::Transform,
                reason: "transform interpolation deferred",
                ..
            })
        ));

        let composite = InterpolationPair::new(
            PropertyKey::new("box-shadow").unwrap(),
            InterpolableValue::composite(CompositeValue::unsupported("shadow-list")),
            InterpolableValue::composite(CompositeValue::unsupported("shadow-list")),
        )
        .unwrap();
        let composite_result = composite.sample(InterpolationProgress::new(0.5).unwrap());
        assert!(matches!(
            composite_result,
            InterpolationOutcome::Unsupported(InterpolationError::UnsupportedFamily {
                family: ValueFamily::Composite,
                reason: "composite interpolation deferred",
                ..
            })
        ));
        if let InterpolationOutcome::Unsupported(InterpolationError::UnsupportedFamily {
            property,
            from,
            to,
            ..
        }) = composite_result
        {
            assert_eq!(property.as_str(), "box-shadow");
            assert_eq!(
                from,
                InterpolableValue::composite(CompositeValue::unsupported("shadow-list"))
            );
            assert_eq!(
                to,
                InterpolableValue::composite(CompositeValue::unsupported("shadow-list"))
            );
        }
    }

    #[test]
    fn interpolation_diagnostics_name_property_and_value_families() {
        let from = InterpolableValue::number(InterpolableNumber::new(0.0).unwrap());
        let to = InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap());
        let error = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            from.clone(),
            to.clone(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            InterpolationError::MismatchedFamilies {
                from: ValueFamily::Number,
                to: ValueFamily::Percentage,
                ..
            }
        ));
        if let InterpolationError::MismatchedFamilies {
            property,
            from_value,
            to_value,
            ..
        } = error
        {
            assert_eq!(property.as_str(), "opacity");
            assert_eq!(from_value, from);
            assert_eq!(to_value, to);
        }
    }

    #[test]
    fn retained_interpolation_diagnostics_have_display_text() {
        assert_eq!(
            InterpolationError::EmptyPropertyKey.to_string(),
            "property key must not be empty"
        );
        assert_eq!(
            InterpolationError::EmptyDiscreteValue.to_string(),
            "discrete value token must not be empty"
        );
        assert_eq!(
            InterpolationError::NonFiniteNumber {
                value: f64::INFINITY,
            }
            .to_string(),
            "interpolable number must be finite, got inf"
        );

        let numeric =
            InterpolationError::Numeric(NumericError::non_finite(NumericInput::Percentage));
        assert_eq!(
            numeric.to_string(),
            "numeric interpolation input is invalid: percentage must be finite"
        );
        assert_eq!(
            std::error::Error::source(&numeric).unwrap().to_string(),
            "percentage must be finite"
        );

        let arithmetic = InterpolationError::NonFiniteInterpolationResult(arithmetic_failure());
        let arithmetic_text = arithmetic.to_string();
        assert!(arithmetic_text.contains("interpolating opacity produced non-finite"));
        assert!(arithmetic_text.contains("number"));

        let families = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap()),
        )
        .unwrap_err();
        assert_eq!(
            families.to_string(),
            "property opacity cannot interpolate number to percentage"
        );

        let colors = InterpolationPair::new(
            PropertyKey::new("color").unwrap(),
            InterpolableValue::color(color_value(ColorInterpolationSpace::Srgb)),
            InterpolableValue::color(color_value(ColorInterpolationSpace::Oklab)),
        )
        .unwrap_err();
        assert_eq!(
            colors.to_string(),
            "property color cannot interpolate colors in srgb and oklab spaces"
        );

        let InterpolationOutcome::Unsupported(unsupported) =
            transform_pair().sample(InterpolationProgress::new(0.5).unwrap())
        else {
            panic!("expected unsupported interpolation");
        };
        assert_eq!(
            unsupported.to_string(),
            "property transform does not support transform interpolation: transform interpolation deferred"
        );

        assert_eq!(
            InterpolationError::NonFiniteProgress { value: f64::NAN }.to_string(),
            "interpolation progress must be finite, got NaN"
        );
        assert!(std::error::Error::source(&InterpolationError::EmptyPropertyKey).is_none());
    }

    fn assert_arithmetic_failure(
        outcome: InterpolationOutcome,
        property: &str,
        family: ValueFamily,
        component: InterpolationComponent,
    ) {
        let InterpolationOutcome::ArithmeticFailure(result) = outcome else {
            panic!("expected arithmetic failure, got {outcome:?}");
        };

        assert_eq!(result.property().as_str(), property);
        assert_eq!(result.family(), family);
        assert_eq!(result.from().family(), family);
        assert_eq!(result.to().family(), family);
        assert_eq!(result.progress(), InterpolationProgress::new(0.5).unwrap());
        assert_eq!(result.component(), component);
        assert!(!result.value().is_finite());
    }

    fn arithmetic_failure() -> NonFiniteInterpolationResult {
        let pair = InterpolationPair::new(
            PropertyKey::new("opacity").unwrap(),
            InterpolableValue::number(InterpolableNumber::new(f64::MAX).unwrap()),
            InterpolableValue::number(InterpolableNumber::new(-f64::MAX).unwrap()),
        )
        .unwrap();

        let InterpolationOutcome::ArithmeticFailure(result) =
            pair.sample(InterpolationProgress::new(0.5).unwrap())
        else {
            panic!("expected arithmetic failure");
        };

        result
    }

    fn color_value(space: ColorInterpolationSpace) -> InterpolableColor {
        InterpolableColor::from_straight_components(
            space,
            0.0,
            0.0,
            0.0,
            crate::UnitRatio::new(1.0).unwrap(),
        )
        .unwrap()
    }

    fn transform_pair() -> InterpolationPair {
        InterpolationPair::new(
            PropertyKey::new("transform").unwrap(),
            InterpolableValue::transform(TransformValue::unsupported("translate")),
            InterpolableValue::transform(TransformValue::unsupported("scale")),
        )
        .unwrap()
    }

    fn assert_sampled_color(
        outcome: InterpolationOutcome,
        expected_space: ColorInterpolationSpace,
        expected_components: (f64, f64, f64),
        expected_alpha: f64,
    ) {
        let InterpolationOutcome::Value(InterpolableValue::Color(color)) = outcome else {
            panic!("expected sampled color, got {outcome:?}");
        };

        assert_eq!(color.space(), expected_space);
        assert_eq!(color.premultiplied_components(), expected_components);
        assert_eq!(color.alpha(), expected_alpha);
    }
}
