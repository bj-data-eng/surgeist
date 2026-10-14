//! Fixed, checked used values and finite intrinsic conversion kernels.
//!
//! These kernels are called by admitted operation actions. They neither parse
//! source nor obtain context facts, allocate scratch, or emit output.

use crate::{
    ColorSpace as Space, HslCoordinates, HwbCoordinates, PolarCoordinates,
    PredefinedColorSpace as Predefined, RectangularCoordinates, RectangularSpace as Rectangular,
    convert_hsl_to_srgb, convert_hwb_to_srgb, convert_lab_to_lch, convert_lch_to_lab,
    convert_oklab_to_oklch, convert_oklch_to_oklab, convert_srgb_to_hsl, convert_srgb_to_hwb,
};

/// A failed checked used-value construction or finite conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ColorError {
    /// A present channel is not finite, in channel order followed by alpha.
    NonFinite {
        component: usize,
    },
    /// Used alpha must already be within its resolved range.
    AlphaOutOfRange,
    /// A concrete coordinate violates its intrinsic domain.
    InvalidCoordinates,
    /// The final destination cannot be represented by finite binary64 channels.
    Unrepresentable,
    /// Byte quantization requires encoded sRGB coordinates.
    RequiresSrgb,
    InvalidSerializationFamily,
}

impl std::fmt::Display for ColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite { component } => {
                write!(f, "nonfinite used color component {component}")
            }
            Self::AlphaOutOfRange => f.write_str("used color alpha is outside zero to one"),
            Self::InvalidCoordinates => f.write_str("invalid intrinsic color coordinates"),
            Self::Unrepresentable => {
                f.write_str("used color conversion is not finitely representable")
            }
            Self::InvalidSerializationFamily => {
                f.write_str("serialization family is incompatible with the color space")
            }
            Self::RequiresSrgb => f.write_str("byte quantization requires sRGB"),
        }
    }
}
impl std::error::Error for ColorError {}

/// One source-selected finite transformation at a used-value checkpoint.
///
/// Order is chosen by the consuming language. This operation does not parse,
/// emit output, obtain facts, or select a fallback for a failed parse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transformation {
    /// Replace missing or present alpha with fully opaque alpha.
    ForceOpaqueAlpha,
    /// Perform intrinsic conversion with the existing binary64 numerical owner.
    Convert(Space),
    /// Clamp encoded sRGB and round positive ties into actual byte channels.
    QuantizeSrgb,
    /// Select the modern color-function form for subsequent serialization.
    ColorFunction,
}

/// Effective resolved serialization family; no authored syntax is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationFamily {
    Natural,
    ColorFunction,
    LegacySrgb,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Channels {
    Floating([Option<f64>; 3]),
    SrgbBytes([u8; 3]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Alpha {
    Floating(Option<f64>),
    Byte(u8),
}
impl Alpha {
    const fn value(self) -> Option<f64> {
        match self {
            Self::Floating(value) => value,
            Self::Byte(value) => Some(value as f64 / 255.0),
        }
    }
}

/// A resolved finite color, independent of authored syntax and scratch storage.
///
/// Space supplies number units: predefined spaces use normalized coordinates,
/// HSL/HWB use degree hue and percentage reference numbers, Lab/LCH use native
/// lightness and axes/chroma, and Oklab/Oklch use their native normalized units.
/// Missing components and missing alpha remain explicit. Extended finite color
/// channels are not gamut-clamped. Only actual byte construction or an explicit
/// quantization can establish byte-backed sRGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    space: Space,
    channels: Channels,
    alpha: Alpha,
    color_function: bool,
}

impl Color {
    pub fn identical(self, other: Self) -> bool {
        let channels = match (self.channels, other.channels) {
            (Channels::Floating(a), Channels::Floating(b)) => {
                a.map(|v| v.map(f64::to_bits)) == b.map(|v| v.map(f64::to_bits))
            }
            (Channels::SrgbBytes(a), Channels::SrgbBytes(b)) => a == b,
            _ => false,
        };
        let alpha = match (self.alpha, other.alpha) {
            (Alpha::Floating(a), Alpha::Floating(b)) => a.map(f64::to_bits) == b.map(f64::to_bits),
            (Alpha::Byte(a), Alpha::Byte(b)) => a == b,
            _ => false,
        };
        self.space == other.space
            && channels
            && alpha
            && self.color_function == other.color_function
    }

    pub fn serialization_family(&self) -> SerializationFamily {
        if self.color_function {
            SerializationFamily::ColorFunction
        } else if self.space == Space::Predefined(Predefined::Srgb) {
            SerializationFamily::LegacySrgb
        } else {
            SerializationFamily::Natural
        }
    }
    pub fn with_serialization_family(
        mut self,
        family: SerializationFamily,
    ) -> Result<Self, ColorError> {
        self.color_function = match family {
            SerializationFamily::Natural => matches!(self.space, Space::Predefined(_)),
            SerializationFamily::ColorFunction => true,
            SerializationFamily::LegacySrgb
                if self.space == Space::Predefined(Predefined::Srgb) =>
            {
                false
            }
            SerializationFamily::LegacySrgb => return Err(ColorError::InvalidSerializationFamily),
        };
        Ok(self)
    }
    pub fn prepared_for_interpolation(self, destination: Space) -> Result<Self, ColorError> {
        crate::interpolation::prepare(self, destination)
    }
    pub fn interpolated(
        self,
        second: Self,
        method: crate::InterpolationMethod,
        progress: f64,
    ) -> Result<Self, crate::InterpolationError> {
        crate::interpolation::interpolate(self, second, method, progress)
    }

    /// Executes one fixed transformation after the caller's work admission.
    /// Failure preserves the original copyable value; no output is produced.
    pub fn transformed(self, transformation: Transformation) -> Result<Self, ColorError> {
        match transformation {
            Transformation::ForceOpaqueAlpha => Ok(self.with_opaque_alpha()),
            Transformation::Convert(space) => self.converted(space),
            Transformation::QuantizeSrgb => self.quantize_srgb(),
            Transformation::ColorFunction => Ok(self.with_color_function()),
        }
    }

    /// Checks fixed finite resolved channels and alpha without parsing or clamping.
    ///
    /// This constructs the natural space form; sRGB uses `color(srgb ...)`, not
    /// a byte representation inferred from integral floating-point values.
    pub fn try_new(
        space: Space,
        channels: [Option<f64>; 3],
        alpha: Option<f64>,
    ) -> Result<Self, ColorError> {
        for (component, channel) in channels.into_iter().chain([alpha]).enumerate() {
            if channel.is_some_and(|value| !value.is_finite()) {
                return Err(ColorError::NonFinite { component });
            }
        }
        if alpha.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
            return Err(ColorError::AlphaOutOfRange);
        }
        // Relative Color5 coordinates are retained without reference-range
        // clamping, including signed saturation/chroma. Concrete numerical
        // converter carriers retain their own stricter construction contract.
        Ok(Self {
            space,
            channels: Channels::Floating(channels),
            alpha: Alpha::Floating(alpha),
            color_function: matches!(space, Space::Predefined(_)),
        })
    }

    /// Constructs exact byte-backed sRGB from decoded channel bytes.
    #[must_use]
    pub const fn from_srgba8(value: [u8; 4]) -> Self {
        Self {
            space: Space::Predefined(Predefined::Srgb),
            channels: Channels::SrgbBytes([value[0], value[1], value[2]]),
            alpha: Alpha::Byte(value[3]),
            color_function: false,
        }
    }

    /// The fixed CSS opaque-black value; it does not certify an author's input.
    #[must_use]
    pub const fn opaque_black() -> Self {
        Self::from_srgba8([0, 0, 0, 255])
    }

    /// The resolved numerical space and channel units.
    #[must_use]
    pub const fn space(&self) -> Space {
        self.space
    }

    /// Fixed finite/missing coordinates; byte channels are normalized to zero to one.
    #[must_use]
    pub fn channels(&self) -> [Option<f64>; 3] {
        match self.channels {
            Channels::Floating(value) => value,
            Channels::SrgbBytes(value) => value.map(|channel| Some(f64::from(channel) / 255.0)),
        }
    }

    /// Resolved alpha, including explicitly missing alpha.
    #[must_use]
    pub const fn alpha(&self) -> Option<f64> {
        self.alpha.value()
    }

    /// Exact alpha byte backing, without inferring it from a floating value.
    #[must_use]
    pub const fn alpha_byte(&self) -> Option<u8> {
        match self.alpha {
            Alpha::Byte(value) => Some(value),
            Alpha::Floating(_) => None,
        }
    }

    /// Exact byte backing, without guessing from floating channels.
    #[must_use]
    pub const fn srgb_bytes(&self) -> Option<[u8; 3]> {
        match self.channels {
            Channels::SrgbBytes(value) => Some(value),
            Channels::Floating(_) => None,
        }
    }

    /// Whether the selected sRGB form is the modern color function.
    #[must_use]
    pub(crate) const fn uses_color_function(&self) -> bool {
        self.color_function
    }

    pub fn with_alpha(mut self, alpha: Option<f64>) -> Result<Self, ColorError> {
        // The checked constructor validates fixed alpha before mutation, while
        // retaining actual byte channels rather than inferring new byte proof.
        Self::try_new(self.space, self.channels(), alpha)?;
        self.alpha = Alpha::Floating(alpha);
        Ok(self)
    }

    pub(crate) const fn with_opaque_alpha(mut self) -> Self {
        self.alpha = match self.alpha {
            Alpha::Byte(_) => Alpha::Byte(255),
            Alpha::Floating(_) => Alpha::Floating(Some(1.0)),
        };
        self
    }

    pub(crate) const fn with_legacy_srgb(mut self) -> Self {
        self.color_function = false;
        self
    }

    pub(crate) const fn with_color_function(mut self) -> Self {
        self.color_function = true;
        self
    }

    pub fn converted(self, destination: Space) -> Result<Self, ColorError> {
        if self.space == destination {
            return Ok(self);
        }
        let coordinates = self.rectangular()?;
        let value = match destination {
            Space::Predefined(space) => {
                let converted = coordinates
                    .convert_to(Rectangular::Predefined(space))
                    .map_err(|_| ColorError::Unrepresentable)?;
                Self::try_new(destination, *converted.channels(), self.alpha())?
            }
            Space::Lab | Space::Oklab => {
                let target = if destination == Space::Lab {
                    Rectangular::Lab
                } else {
                    Rectangular::Oklab
                };
                let converted = coordinates
                    .convert_to(target)
                    .map_err(|_| ColorError::Unrepresentable)?;
                Self::try_new(destination, *converted.channels(), self.alpha())?
            }
            Space::Lch | Space::Oklch => {
                let target = if destination == Space::Lch {
                    Rectangular::Lab
                } else {
                    Rectangular::Oklab
                };
                let converted = coordinates
                    .convert_to(target)
                    .map_err(|_| ColorError::Unrepresentable)?;
                let channels = converted.channels().map(|value| value.unwrap_or(0.0));
                let polar = if destination == Space::Lch {
                    convert_lab_to_lch(channels)
                } else {
                    convert_oklab_to_oklch(channels)
                }
                .map_err(|_| ColorError::Unrepresentable)?;
                Self::try_new(
                    destination,
                    [
                        Some(polar.lightness()),
                        Some(polar.chroma()),
                        polar.hue_degrees(),
                    ],
                    self.alpha(),
                )?
            }
            Space::Hsl | Space::Hwb => {
                let converted = coordinates
                    .convert_to(Rectangular::Predefined(Predefined::Srgb))
                    .map_err(|_| ColorError::Unrepresentable)?;
                let channels = converted.channels().map(|value| value.unwrap_or(0.0));
                let channels = if destination == Space::Hsl {
                    let value =
                        convert_srgb_to_hsl(channels).map_err(|_| ColorError::Unrepresentable)?;
                    [
                        value.hue_degrees(),
                        Some(value.saturation()),
                        Some(value.lightness()),
                    ]
                } else {
                    let value =
                        convert_srgb_to_hwb(channels).map_err(|_| ColorError::Unrepresentable)?;
                    [
                        value.hue_degrees(),
                        Some(value.whiteness()),
                        Some(value.blackness()),
                    ]
                };
                Self::try_new(destination, channels, self.alpha())?
            }
        };
        // Preserve the source-selected modern family through fixed channel
        // conversions. A target predefined space also selects color(...).
        Ok(Self {
            alpha: self.alpha,
            color_function: self.color_function || value.color_function,
            ..value
        })
    }

    fn rectangular(self) -> Result<RectangularCoordinates, ColorError> {
        let channels = self.channels();
        let (space, channels) = match self.space {
            Space::Predefined(space) => (Rectangular::Predefined(space), channels),
            Space::Lab => (Rectangular::Lab, channels),
            Space::Oklab => (Rectangular::Oklab, channels),
            Space::Lch | Space::Oklch => {
                let chroma = channels[1].unwrap_or(0.0);
                // Signed polar radius is the same Cartesian point at the
                // opposite hue. This adapter preserves the resolved value;
                // it does not broaden the existing converter's native type.
                let hue = if chroma < 0.0 {
                    Some((channels[2].unwrap_or(0.0).rem_euclid(360.0) + 180.0).rem_euclid(360.0))
                } else {
                    channels[2]
                };
                let polar =
                    PolarCoordinates::try_new(channels[0].unwrap_or(0.0), chroma.abs(), hue)
                        .map_err(|_| ColorError::InvalidCoordinates)?;
                let channels = if self.space == Space::Lch {
                    convert_lch_to_lab(&polar)
                } else {
                    convert_oklch_to_oklab(&polar)
                };
                let space = if self.space == Space::Lch {
                    Rectangular::Lab
                } else {
                    Rectangular::Oklab
                };
                (space, channels.map(Some))
            }
            Space::Hsl => {
                let saturation = channels[1].unwrap_or(0.0);
                let hue = if saturation < 0.0 {
                    Some((channels[0].unwrap_or(0.0).rem_euclid(360.0) + 180.0).rem_euclid(360.0))
                } else {
                    channels[0]
                };
                let hsl =
                    HslCoordinates::try_new(hue, saturation.abs(), channels[2].unwrap_or(0.0))
                        .map_err(|_| ColorError::InvalidCoordinates)?;
                let channels =
                    convert_hsl_to_srgb(&hsl).map_err(|_| ColorError::Unrepresentable)?;
                (
                    Rectangular::Predefined(Predefined::Srgb),
                    channels.map(Some),
                )
            }
            Space::Hwb => {
                let hwb = HwbCoordinates::try_new(
                    channels[0],
                    channels[1].unwrap_or(0.0),
                    channels[2].unwrap_or(0.0),
                )
                .map_err(|_| ColorError::InvalidCoordinates)?;
                (
                    Rectangular::Predefined(Predefined::Srgb),
                    convert_hwb_to_srgb(&hwb).map(Some),
                )
            }
        };
        RectangularCoordinates::try_new(space, channels).map_err(|_| ColorError::Unrepresentable)
    }

    pub(crate) fn quantize_srgb(self) -> Result<Self, ColorError> {
        if self.space != Space::Predefined(Predefined::Srgb) {
            return Err(ColorError::RequiresSrgb);
        }
        let channels = self.channels().map(|value| value.map(quantize_channel));
        // Missing coordinates remain missing. They cannot prove all three byte
        // channels or select HTML hex; source-selected serializer handling owns
        // any eventual missing-to-zero conversion.
        let representation = match channels {
            [Some(red), Some(green), Some(blue)] => Channels::SrgbBytes([red, green, blue]),
            _ => Channels::Floating(
                channels.map(|value| value.map(|value| f64::from(value) / 255.0)),
            ),
        };
        Ok(Self {
            channels: representation,
            ..self
        })
    }

    pub fn black_white_contrast(self) -> Result<Self, ColorError> {
        let xyz = self.converted(Space::Predefined(Predefined::XyzD65))?;
        let luminance = xyz.channels()[1].unwrap_or(0.0).clamp(0.0, 1.0);
        let white = 1.05 / (luminance + 0.05);
        let black = (luminance + 0.05) / 0.05;
        Ok(Self::from_srgba8(if white >= black {
            [255; 4]
        } else {
            [0, 0, 0, 255]
        }))
    }
}

fn quantize_channel(channel: f64) -> u8 {
    let scaled = channel.clamp(0.0, 1.0) * 255.0;
    let lower = scaled.floor();
    (lower + f64::from(scaled - lower >= 0.5)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_integer_channels_do_not_prove_byte_backing() {
        let value = Color::try_new(
            Space::Predefined(Predefined::Srgb),
            [Some(1.0), Some(0.0), Some(0.0)],
            Some(1.0),
        )
        .unwrap();
        assert_eq!(value.srgb_bytes(), None);
        assert_eq!(
            value.quantize_srgb().unwrap().srgb_bytes(),
            Some([255, 0, 0])
        );
    }

    #[test]
    fn quantization_has_positive_ties_and_preserves_missingness() {
        let value = Color::try_new(
            Space::Predefined(Predefined::Srgb),
            [Some(0.5), Some(-2.0), Some(2.0)],
            None,
        )
        .unwrap()
        .quantize_srgb()
        .unwrap();
        assert_eq!(value.srgb_bytes(), Some([128, 0, 255]));
        assert_eq!(value.alpha(), None);
        let missing = Color::try_new(
            Space::Predefined(Predefined::Srgb),
            [None, Some(0.5), Some(0.0)],
            Some(1.0),
        )
        .unwrap()
        .quantize_srgb()
        .unwrap();
        assert_eq!(missing.srgb_bytes(), None);
        assert_eq!(missing.channels(), [None, Some(128.0 / 255.0), Some(0.0)]);
    }

    #[test]
    fn alpha_and_channel_checks_do_not_clamp_native_values() {
        assert_eq!(
            Color::try_new(Space::Lab, [Some(f64::INFINITY), None, None], Some(2.0)),
            Err(ColorError::NonFinite { component: 0 })
        );
        assert_eq!(
            Color::try_new(Space::Lab, [None; 3], Some(2.0)),
            Err(ColorError::AlphaOutOfRange)
        );
        let value = Color::try_new(Space::Lab, [Some(-2.0), Some(3.0), None], Some(-0.0)).unwrap();
        assert_eq!(value.channels(), [Some(-2.0), Some(3.0), None]);
        assert_eq!(value.alpha().unwrap().to_bits(), (-0.0_f64).to_bits());
    }

    #[test]
    fn same_space_retains_bits_and_conversion_retains_extended_range() {
        let value = Color::try_new(
            Space::Predefined(Predefined::SrgbLinear),
            [Some(-0.0), None, Some(2.0)],
            Some(0.5),
        )
        .unwrap();
        assert_eq!(
            value.converted(value.space()).unwrap().channels()[0]
                .unwrap()
                .to_bits(),
            (-0.0_f64).to_bits()
        );
        let result = value
            .converted(Space::Predefined(Predefined::Srgb))
            .unwrap();
        assert!(result.channels()[2].unwrap() > 1.0);
        assert_eq!(result.alpha(), Some(0.5));
    }

    #[test]
    fn modern_family_is_retained_through_cylindrical_conversion_and_alpha() {
        let absolute =
            Color::try_new(Space::Hsl, [Some(0.0), Some(100.0), Some(50.0)], Some(1.0)).unwrap();
        assert!(!absolute.uses_color_function());
        let relative = absolute.with_color_function();
        let rgb = relative
            .converted(Space::Predefined(Predefined::Srgb))
            .unwrap();
        let hwb = rgb.converted(Space::Hwb).unwrap();
        assert_eq!(hwb.channels(), [Some(0.0), Some(0.0), Some(0.0)]);
        assert!(hwb.uses_color_function());
        assert!(hwb.with_alpha(Some(0.5)).unwrap().uses_color_function());
        assert!(hwb.with_opaque_alpha().uses_color_function());
        // The marker describes source-selected serialization, not coordinate
        // equality: the absolute and relative values retain distinct forms.
        assert_eq!(absolute.channels(), relative.channels());
        assert!(!absolute.identical(relative));
    }

    #[test]
    fn contrast_uses_selected_luminance_and_ignores_alpha() {
        assert_eq!(
            Color::opaque_black()
                .black_white_contrast()
                .unwrap()
                .srgb_bytes(),
            Some([255; 3])
        );
        assert_eq!(
            Color::from_srgba8([255; 4])
                .black_white_contrast()
                .unwrap()
                .srgb_bytes(),
            Some([0; 3])
        );
        let dark = Color::try_new(
            Space::Predefined(Predefined::XyzD65),
            [Some(0.0), Some(-2.0), Some(0.0)],
            Some(0.0),
        )
        .unwrap();
        assert_eq!(
            dark.black_white_contrast().unwrap().srgb_bytes(),
            Some([255; 3])
        );
        assert_eq!(
            dark.with_opaque_alpha().black_white_contrast().unwrap(),
            dark.black_white_contrast().unwrap()
        );
    }
    #[test]
    fn byte_alpha_proof_is_preserved_and_never_inferred() {
        let bytes = Color::from_srgba8([1, 2, 3, 237]);
        assert_eq!(bytes.alpha_byte(), Some(237));
        assert_eq!(bytes.converted(Space::Lab).unwrap().alpha_byte(), Some(237));
        assert_eq!(bytes.with_opaque_alpha().alpha_byte(), Some(255));
        let floating = Color::try_new(bytes.space(), bytes.channels(), bytes.alpha()).unwrap();
        assert_eq!(floating.alpha_byte(), None);
        assert_eq!(floating.quantize_srgb().unwrap().alpha_byte(), None);
        assert_eq!(floating.with_opaque_alpha().alpha_byte(), None);
    }
}
