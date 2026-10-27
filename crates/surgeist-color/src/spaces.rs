//! Numerical space identities and checked interpolation policy.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PredefinedColorSpace {
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    XyzD50,
    XyzD65,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ColorSpace {
    Predefined(PredefinedColorSpace),
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
}
impl ColorSpace {
    pub const fn is_polar(self) -> bool {
        matches!(self, Self::Hsl | Self::Hwb | Self::Lch | Self::Oklch)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HueInterpolation {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterpolationMethod {
    space: ColorSpace,
    hue: Option<HueInterpolation>,
}
impl InterpolationMethod {
    pub const fn try_new(
        space: ColorSpace,
        hue: Option<HueInterpolation>,
    ) -> Result<Self, InterpolationError> {
        if !space.is_polar() && hue.is_some() {
            return Err(InterpolationError::HueRequiresPolarSpace);
        }
        Ok(Self {
            space,
            hue: if space.is_polar() {
                Some(match hue {
                    Some(h) => h,
                    None => HueInterpolation::Shorter,
                })
            } else {
                None
            },
        })
    }
    pub const fn space(self) -> ColorSpace {
        self.space
    }
    pub const fn hue(self) -> Option<HueInterpolation> {
        self.hue
    }
}
/// An interpolation method that cannot exist: a hue interpolation method
/// applies only to a polar space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InterpolationError {
    HueRequiresPolarSpace,
}
impl std::fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HueRequiresPolarSpace => f.write_str("hue interpolation requires a polar space"),
        }
    }
}
impl std::error::Error for InterpolationError {}
