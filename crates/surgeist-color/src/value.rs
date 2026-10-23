//! Fixed finite used values and total intrinsic conversion kernels.
//!
//! These kernels are called by admitted operation actions. They neither parse
//! source nor obtain context facts, allocate scratch, or emit output.

use crate::finite::{Finite, UnitInterval, closest_finite};
use crate::{
    ColorSpace as Space, HslCoordinates, HwbCoordinates, PolarCoordinates,
    PredefinedColorSpace as Predefined, RectangularCoordinates, RectangularSpace as Rectangular,
    convert_hsl_to_srgb, convert_hwb_to_srgb, convert_lab_to_lch, convert_lch_to_lab,
    convert_oklab_to_oklch, convert_oklch_to_oklab, convert_srgb_to_hsl, convert_srgb_to_hwb,
};

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
    /// Convert to sRGB when needed, then clamp encoded sRGB and round positive
    /// ties into actual byte channels.
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
    Floating([Option<Finite>; 3]),
    SrgbBytes([u8; 3]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Alpha {
    Floating(Option<UnitInterval>),
    Byte(u8),
}
impl Alpha {
    const fn value(self) -> Option<f64> {
        match self {
            Self::Floating(value) => match value {
                Some(value) => Some(value.get()),
                None => None,
            },
            Self::Byte(value) => Some(value as f64 / 255.0),
        }
    }
    fn unit(self) -> Option<UnitInterval> {
        match self {
            Self::Floating(value) => value,
            Self::Byte(value) => UnitInterval::new(f64::from(value) / 255.0),
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
/// quantization can establish byte-backed sRGB. Every operation is total: a
/// result beyond binary64 is the signed closest finite value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    space: Space,
    channels: Channels,
    alpha: Alpha,
    color_function: bool,
}

impl Color {
    pub fn identical(self, other: Self) -> bool {
        let bits = |value: Option<Finite>| value.map(|value| value.get().to_bits());
        let channels = match (self.channels, other.channels) {
            (Channels::Floating(a), Channels::Floating(b)) => a.map(bits) == b.map(bits),
            (Channels::SrgbBytes(a), Channels::SrgbBytes(b)) => a == b,
            _ => false,
        };
        let alpha = match (self.alpha, other.alpha) {
            (Alpha::Floating(a), Alpha::Floating(b)) => {
                a.map(|value| value.get().to_bits()) == b.map(|value| value.get().to_bits())
            }
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

    #[must_use]
    pub fn prepared_for_interpolation(self, destination: Space) -> Self {
        crate::interpolation::prepare(self, destination)
    }

    #[must_use]
    pub fn interpolated(
        self,
        second: Self,
        method: crate::InterpolationMethod,
        progress: UnitInterval,
    ) -> Self {
        crate::interpolation::interpolate(self, second, method, progress)
    }

    /// Executes one fixed transformation after the caller's work admission.
    #[must_use]
    pub fn transformed(self, transformation: Transformation) -> Self {
        match transformation {
            Transformation::ForceOpaqueAlpha => self.with_opaque_alpha(),
            Transformation::Convert(space) => self.converted(space),
            Transformation::QuantizeSrgb => self.quantize_srgb(),
            Transformation::ColorFunction => self.with_color_function(),
        }
    }

    /// Constructs the natural space form of finite channels and unit alpha,
    /// without parsing or clamping.
    ///
    /// sRGB uses `color(srgb ...)`, not a byte representation inferred from
    /// integral floating-point values.
    #[must_use]
    pub fn new(space: Space, channels: [Option<Finite>; 3], alpha: Option<UnitInterval>) -> Self {
        // Relative Color5 coordinates are retained without reference-range
        // clamping, including signed saturation/chroma.
        Self {
            space,
            channels: Channels::Floating(channels),
            alpha: Alpha::Floating(alpha),
            color_function: matches!(space, Space::Predefined(_)),
        }
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
        self.finite_channels()
            .map(|channel| channel.map(Finite::get))
    }

    /// The same coordinates as their finite proofs; byte channels are
    /// normalized to zero to one.
    #[must_use]
    pub fn finite_channels(&self) -> [Option<Finite>; 3] {
        match self.channels {
            Channels::Floating(value) => value,
            Channels::SrgbBytes(value) => {
                value.map(|channel| Some(closest_finite(f64::from(channel) / 255.0)))
            }
        }
    }

    /// Resolved alpha, including explicitly missing alpha.
    #[must_use]
    pub const fn alpha(&self) -> Option<f64> {
        self.alpha.value()
    }

    /// The same alpha as its unit-interval proof; byte alpha is normalized.
    #[must_use]
    pub fn unit_alpha(&self) -> Option<UnitInterval> {
        self.alpha.unit()
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

    /// Replaces alpha, retaining actual byte channels rather than inferring
    /// new byte proof.
    #[must_use]
    pub fn with_alpha(mut self, alpha: Option<UnitInterval>) -> Self {
        self.alpha = Alpha::Floating(alpha);
        self
    }

    pub(crate) const fn with_opaque_alpha(mut self) -> Self {
        self.alpha = match self.alpha {
            Alpha::Byte(_) => Alpha::Byte(255),
            Alpha::Floating(_) => Alpha::Floating(UnitInterval::new(1.0)),
        };
        self
    }

    pub(crate) const fn with_legacy_srgb(mut self) -> Self {
        self.color_function = false;
        self
    }

    /// Selects the modern color-function form for subsequent serialization.
    #[must_use]
    pub const fn with_color_function(mut self) -> Self {
        self.color_function = true;
        self
    }

    /// Converts to sRGB when needed and selects the legacy `rgb()` family.
    #[must_use]
    pub fn to_legacy_srgb(self) -> Self {
        self.converted(Space::Predefined(Predefined::Srgb))
            .with_legacy_srgb()
    }

    /// Converts to `destination`; a channel beyond binary64 is the signed
    /// closest finite value, and no gamut mapping is applied.
    #[must_use]
    pub fn converted(self, destination: Space) -> Self {
        if self.space == destination {
            return self;
        }
        let coordinates = self.rectangular();
        let alpha = self.alpha.unit();
        let value = match destination {
            Space::Predefined(space) => Self::new(
                destination,
                coordinates
                    .convert_to(Rectangular::Predefined(space))
                    .channels(),
                alpha,
            ),
            Space::Lab | Space::Oklab => {
                let target = if destination == Space::Lab {
                    Rectangular::Lab
                } else {
                    Rectangular::Oklab
                };
                Self::new(
                    destination,
                    coordinates.convert_to(target).channels(),
                    alpha,
                )
            }
            Space::Lch | Space::Oklch => {
                let target = if destination == Space::Lch {
                    Rectangular::Lab
                } else {
                    Rectangular::Oklab
                };
                let channels = present(coordinates.convert_to(target).channels());
                let polar = if destination == Space::Lch {
                    convert_lab_to_lch(channels)
                } else {
                    convert_oklab_to_oklch(channels)
                };
                Self::new(
                    destination,
                    [
                        Some(polar.lightness()),
                        Some(polar.chroma()),
                        polar.hue_degrees(),
                    ],
                    alpha,
                )
            }
            Space::Hsl | Space::Hwb => {
                let channels = present(
                    coordinates
                        .convert_to(Rectangular::Predefined(Predefined::Srgb))
                        .channels(),
                );
                let channels = if destination == Space::Hsl {
                    let value = convert_srgb_to_hsl(channels);
                    [
                        value.hue_degrees(),
                        Some(value.saturation()),
                        Some(value.lightness()),
                    ]
                } else {
                    let value = convert_srgb_to_hwb(channels);
                    [
                        value.hue_degrees(),
                        Some(value.whiteness()),
                        Some(value.blackness()),
                    ]
                };
                Self::new(destination, channels, alpha)
            }
        };
        // Preserve the source-selected modern family through fixed channel
        // conversions. A target predefined space also selects color(...).
        Self {
            alpha: self.alpha,
            color_function: self.color_function || value.color_function,
            ..value
        }
    }

    fn rectangular(self) -> RectangularCoordinates {
        let channels = self.finite_channels();
        let zero = closest_finite(0.0);
        let (space, channels) = match self.space {
            Space::Predefined(space) => (Rectangular::Predefined(space), channels),
            Space::Lab => (Rectangular::Lab, channels),
            Space::Oklab => (Rectangular::Oklab, channels),
            Space::Lch | Space::Oklch => {
                // Signed polar radius is the same Cartesian point at the
                // opposite hue; the carrier applies that rotation.
                let polar = PolarCoordinates::new(
                    channels[0].unwrap_or(zero),
                    channels[1].unwrap_or(zero),
                    channels[2],
                );
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
                let hsl = HslCoordinates::new(
                    channels[0],
                    channels[1].unwrap_or(zero),
                    channels[2].unwrap_or(zero),
                );
                (
                    Rectangular::Predefined(Predefined::Srgb),
                    convert_hsl_to_srgb(&hsl).map(Some),
                )
            }
            Space::Hwb => {
                let hwb = HwbCoordinates::new(
                    channels[0],
                    channels[1].unwrap_or(zero),
                    channels[2].unwrap_or(zero),
                );
                (
                    Rectangular::Predefined(Predefined::Srgb),
                    convert_hwb_to_srgb(&hwb).map(Some),
                )
            }
        };
        RectangularCoordinates::new(space, channels)
    }

    /// Converts to sRGB when needed, then quantizes present channels.
    pub(crate) fn quantize_srgb(self) -> Self {
        let srgb = self.converted(Space::Predefined(Predefined::Srgb));
        let channels = srgb.channels().map(|value| value.map(quantize_channel));
        // Missing coordinates remain missing. They cannot prove all three byte
        // channels or select HTML hex; source-selected serializer handling owns
        // any eventual missing-to-zero conversion.
        let representation = match channels {
            [Some(red), Some(green), Some(blue)] => Channels::SrgbBytes([red, green, blue]),
            _ => Channels::Floating(
                channels.map(|value| value.map(|value| closest_finite(f64::from(value) / 255.0))),
            ),
        };
        Self {
            channels: representation,
            ..srgb
        }
    }

    #[must_use]
    pub fn black_white_contrast(self) -> Self {
        let xyz = self.converted(Space::Predefined(Predefined::XyzD65));
        let luminance = xyz.channels()[1].unwrap_or(0.0).clamp(0.0, 1.0);
        let white = 1.05 / (luminance + 0.05);
        let black = (luminance + 0.05) / 0.05;
        Self::from_srgba8(if white >= black {
            [255; 4]
        } else {
            [0, 0, 0, 255]
        })
    }
}

/// Missing channels of a converted value locally read as zero.
fn present(channels: [Option<Finite>; 3]) -> [Finite; 3] {
    channels.map(|channel| channel.unwrap_or(closest_finite(0.0)))
}

fn quantize_channel(channel: f64) -> u8 {
    let scaled = channel.clamp(0.0, 1.0) * 255.0;
    let lower = scaled.floor();
    (lower + f64::from(scaled - lower >= 0.5)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finite(channels: [Option<f64>; 3]) -> [Option<Finite>; 3] {
        channels.map(|channel| channel.map(|value| Finite::new(value).unwrap()))
    }
    fn unit(value: f64) -> Option<UnitInterval> {
        Some(UnitInterval::new(value).unwrap())
    }

    #[test]
    fn floating_integer_channels_do_not_prove_byte_backing() {
        let value = Color::new(
            Space::Predefined(Predefined::Srgb),
            finite([Some(1.0), Some(0.0), Some(0.0)]),
            unit(1.0),
        );
        assert_eq!(value.srgb_bytes(), None);
        assert_eq!(value.quantize_srgb().srgb_bytes(), Some([255, 0, 0]));
    }

    #[test]
    fn quantization_has_positive_ties_and_preserves_missingness() {
        let value = Color::new(
            Space::Predefined(Predefined::Srgb),
            finite([Some(0.5), Some(-2.0), Some(2.0)]),
            None,
        )
        .quantize_srgb();
        assert_eq!(value.srgb_bytes(), Some([128, 0, 255]));
        assert_eq!(value.alpha(), None);
        let missing = Color::new(
            Space::Predefined(Predefined::Srgb),
            finite([None, Some(0.5), Some(0.0)]),
            unit(1.0),
        )
        .quantize_srgb();
        assert_eq!(missing.srgb_bytes(), None);
        assert_eq!(missing.channels(), [None, Some(128.0 / 255.0), Some(0.0)]);
    }

    #[test]
    fn quantization_converts_a_non_srgb_value_to_srgb_first() {
        // Color 4 §10.2: linear 0.5 encodes to 1.055 * 0.5^(1/2.4) - 0.055.
        let linear = Color::new(
            Space::Predefined(Predefined::SrgbLinear),
            finite([Some(1.0), Some(0.0), Some(0.5)]),
            unit(0.25),
        );
        let quantized = linear.quantize_srgb();
        assert_eq!(quantized.space(), Space::Predefined(Predefined::Srgb));
        assert_eq!(quantized.srgb_bytes(), Some([255, 0, 188]));
        assert_eq!(quantized.alpha(), Some(0.25));
    }

    #[test]
    fn channel_construction_does_not_clamp_native_values() {
        let value = Color::new(
            Space::Lab,
            finite([Some(-2.0), Some(3.0), None]),
            UnitInterval::new(-0.0),
        );
        assert_eq!(value.channels(), [Some(-2.0), Some(3.0), None]);
        assert_eq!(value.alpha().unwrap().to_bits(), (-0.0_f64).to_bits());
    }

    #[test]
    fn same_space_retains_bits_and_conversion_retains_extended_range() {
        let value = Color::new(
            Space::Predefined(Predefined::SrgbLinear),
            finite([Some(-0.0), None, Some(2.0)]),
            unit(0.5),
        );
        assert_eq!(
            value.converted(value.space()).channels()[0]
                .unwrap()
                .to_bits(),
            (-0.0_f64).to_bits()
        );
        let result = value.converted(Space::Predefined(Predefined::Srgb));
        assert!(result.channels()[2].unwrap() > 1.0);
        assert_eq!(result.alpha(), Some(0.5));
    }

    #[test]
    fn modern_family_is_retained_through_cylindrical_conversion_and_alpha() {
        let absolute = Color::new(
            Space::Hsl,
            finite([Some(0.0), Some(100.0), Some(50.0)]),
            unit(1.0),
        );
        assert!(!absolute.uses_color_function());
        let relative = absolute.with_color_function();
        let rgb = relative.converted(Space::Predefined(Predefined::Srgb));
        let hwb = rgb.converted(Space::Hwb);
        assert_eq!(hwb.channels(), [Some(0.0), Some(0.0), Some(0.0)]);
        assert!(hwb.uses_color_function());
        assert!(hwb.with_alpha(unit(0.5)).uses_color_function());
        assert!(hwb.with_opaque_alpha().uses_color_function());
        // The marker describes source-selected serialization, not coordinate
        // equality: the absolute and relative values retain distinct forms.
        assert_eq!(absolute.channels(), relative.channels());
        assert!(!absolute.identical(relative));
    }

    #[test]
    fn legacy_srgb_converts_first_and_selects_the_rgb_family() {
        let hsl = Color::new(
            Space::Hsl,
            finite([Some(120.0), Some(100.0), Some(50.0)]),
            unit(1.0),
        );
        let legacy = hsl.to_legacy_srgb();
        assert_eq!(legacy.space(), Space::Predefined(Predefined::Srgb));
        assert_eq!(legacy.channels(), [Some(0.0), Some(1.0), Some(0.0)]);
        assert_eq!(
            legacy.serialization_family(),
            SerializationFamily::LegacySrgb
        );
        let bytes = Color::from_srgba8([1, 2, 3, 4]).with_color_function();
        assert!(
            bytes
                .to_legacy_srgb()
                .identical(Color::from_srgba8([1, 2, 3, 4]))
        );
    }

    #[test]
    fn contrast_uses_selected_luminance_and_ignores_alpha() {
        assert_eq!(
            Color::opaque_black().black_white_contrast().srgb_bytes(),
            Some([255; 3])
        );
        assert_eq!(
            Color::from_srgba8([255; 4])
                .black_white_contrast()
                .srgb_bytes(),
            Some([0; 3])
        );
        let dark = Color::new(
            Space::Predefined(Predefined::XyzD65),
            finite([Some(0.0), Some(-2.0), Some(0.0)]),
            unit(0.0),
        );
        assert_eq!(dark.black_white_contrast().srgb_bytes(), Some([255; 3]));
        assert_eq!(
            dark.with_opaque_alpha().black_white_contrast(),
            dark.black_white_contrast()
        );
    }

    #[test]
    fn overflowing_conversion_is_the_signed_closest_finite_value() {
        // The exact XYZ of sRGB red at f64::MAX exceeds binary64 (its linear
        // light is the 2.4th power of MAX), so every positive row saturates.
        let red = Color::new(
            Space::Predefined(Predefined::Srgb),
            finite([Some(f64::MAX), Some(0.0), Some(0.0)]),
            None,
        );
        let xyz = red.converted(Space::Predefined(Predefined::XyzD65));
        assert_eq!(xyz.channels(), [Some(f64::MAX); 3]);
        assert_eq!(
            red.black_white_contrast().srgb_bytes(),
            Some([0, 0, 0]),
            "luminance clamps to one before contrast"
        );
    }

    #[test]
    fn byte_alpha_proof_is_preserved_and_never_inferred() {
        let bytes = Color::from_srgba8([1, 2, 3, 237]);
        assert_eq!(bytes.alpha_byte(), Some(237));
        assert_eq!(bytes.converted(Space::Lab).alpha_byte(), Some(237));
        assert_eq!(bytes.with_opaque_alpha().alpha_byte(), Some(255));
        let floating = Color::new(
            bytes.space(),
            finite(bytes.channels()),
            bytes.alpha().and_then(UnitInterval::new),
        );
        assert_eq!(floating.alpha_byte(), None);
        assert_eq!(floating.quantize_srgb().alpha_byte(), None);
        assert_eq!(floating.with_opaque_alpha().alpha_byte(), None);
    }
}
