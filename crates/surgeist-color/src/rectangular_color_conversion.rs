//! Intrinsic rectangular color conversion from Color 4 CRD 2026-09-08 §§10–11.
//!
//! Source: <https://www.w3.org/TR/2026/CRD-css-color-4-20260908/>. The pinned
//! reference SHA256 is bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba.
//! Normative primary xy coordinates and D50/D65 whites determine the RGB
//! matrices. The Bradford cone basis and XYZ/Lab/Oklab coefficient evidence are
//! identified in the private colorimetry module. Section 19's examples are
//! informative and do not override the normative unclamped conversion contract.
//! ProPhoto uses reflected absolute powers to repair §10.7's raw negative
//! fractional power, consistent with the extended real domain and §19. Rec.2020
//! uses §10.8's 2.4 curve rather than the older piecewise browser curve.

mod colorimetry;
mod transfer;
mod wide;

use crate::PredefinedColorSpace;
use crate::finite::Finite;
use colorimetry::WhitePoint;
use wide::Wide;

/// The eleven supported intrinsic rectangular color spaces.
///
/// This identifies numerical units and a whitepoint, without authored CSS,
/// profiles, alpha, interpolation or a physical output target. Predefined RGB
/// and XYZ use number units where 100% is 1. Lab uses usual L in `0..100` and
/// native a/b units; Oklab uses usual L in `0..1` and native a/b units. These
/// usual ranges do not restrict finite extended numerical coordinates.
///
/// Downstream matches must allow additional spaces:
///
/// ```compile_fail
/// use surgeist_color::RectangularSpace;
/// fn name(space: RectangularSpace) -> &'static str {
///     match space {
///         RectangularSpace::Predefined(_) => "predefined",
///         RectangularSpace::Lab => "lab",
///         RectangularSpace::Oklab => "oklab",
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RectangularSpace {
    /// A predefined RGB or XYZ space, with its numerical identity.
    Predefined(PredefinedColorSpace),
    /// Rectangular CIE Lab coordinates adapted to D50.
    Lab,
    /// Rectangular Oklab coordinates adapted to D65.
    Oklab,
}

/// Finite or explicitly missing coordinates in one rectangular space.
///
/// The space fixes channel order: RGB, XYZ, or L/a/b. Construction preserves
/// every present input bit, including negative zero, and retains missing
/// channels. All finite extended coordinates are accepted without clamping.
/// This numerical carrier does not resolve authored CSS or carry alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangularCoordinates {
    space: RectangularSpace,
    channels: [Option<Finite>; 3],
}

impl RectangularCoordinates {
    /// Builds coordinates from finite or missing channels in index order.
    ///
    /// ```
    /// use surgeist_color::{Finite, PredefinedColorSpace, RectangularCoordinates,
    ///     RectangularSpace};
    /// let color = RectangularCoordinates::new(
    ///     RectangularSpace::Predefined(PredefinedColorSpace::Srgb),
    ///     [Finite::new(-0.0), None, Finite::new(1.2)],
    /// );
    /// assert_eq!(color.channels()[0].unwrap().get().to_bits(), (-0.0_f64).to_bits());
    /// assert_eq!(color.channels()[1], None);
    /// ```
    #[must_use]
    pub const fn new(space: RectangularSpace, channels: [Option<Finite>; 3]) -> Self {
        Self { space, channels }
    }

    /// The space that determines units, channel order and whitepoint.
    #[must_use]
    pub const fn space(&self) -> RectangularSpace {
        self.space
    }

    /// The retained finite or explicitly missing channels, without resolution.
    #[must_use]
    pub const fn channels(&self) -> [Option<Finite>; 3] {
        self.channels
    }

    /// Converts to another intrinsic rectangular space without gamut mapping.
    ///
    /// Same-space conversion is identity, preserving missing channels and all
    /// present bits. For different spaces, missing input channels locally become
    /// zero, as required by Color 4 §11.2, and all destination channels are present.
    ///
    /// Conversion undoes RGB transfer, uses the native-whitepoint XYZ matrices,
    /// Bradford-adapts only when the whitepoint changes, then converts and encodes
    /// the destination. Lab and Oklab use their respective D50 and D65 formulas.
    /// Negative values and values above usual channel/lightness ranges survive;
    /// there is no authored-value clamp, profile lookup, interpolation, alpha
    /// premultiplication or physical output step.
    ///
    /// Private per-channel wide exponents preserve intermediate range. Each
    /// significand operation uses ordinary binary64 rounding, so tiny terms and
    /// cancellation residuals may be lost. This is not correctly rounded
    /// exact-real conversion. A final channel beyond binary64 is the signed
    /// closest finite value (CSS Values 4 closest supported value); final
    /// underflow to zero is allowed.
    ///
    /// ```
    /// use surgeist_color::{Finite, PredefinedColorSpace as P, RectangularCoordinates,
    ///     RectangularSpace as S};
    /// let encoded = RectangularCoordinates::new(
    ///     S::Predefined(P::Srgb), [Finite::new(0.01292), None, Finite::new(-0.01292)],
    /// );
    /// let linear = encoded.convert_to(S::Predefined(P::SrgbLinear));
    /// assert!((linear.channels()[0].unwrap().get() - 0.001).abs() < 1e-15);
    /// assert_eq!(linear.channels()[1].map(Finite::get), Some(0.0));
    /// assert_eq!(encoded.channels()[1], None);
    /// let extreme = RectangularCoordinates::new(
    ///     S::Predefined(P::Srgb), [Finite::new(f64::MAX), None, None],
    /// );
    /// let xyz = extreme.convert_to(S::Predefined(P::XyzD65));
    /// assert_eq!(xyz.channels()[1].map(Finite::get), Some(f64::MAX));
    /// ```
    #[must_use]
    pub fn convert_to(&self, destination: RectangularSpace) -> Self {
        if self.space == destination {
            return *self;
        }
        let source = self
            .channels
            .map(|value| Wide::from_f64(value.map_or(0.0, Finite::get)));
        let converted = if let Some((source_space, destination_space)) =
            same_rgb_primaries(self.space, destination)
        {
            source.map(|value| {
                transfer::encode(transfer::decode(value, source_space), destination_space)
            })
        } else {
            let xyz = to_xyz(source, self.space);
            let xyz = colorimetry::adapt(xyz, whitepoint(self.space), whitepoint(destination));
            from_xyz(xyz, destination)
        };
        Self {
            space: destination,
            channels: converted.map(|value| Some(value.to_finite())),
        }
    }
}
fn whitepoint(space: RectangularSpace) -> WhitePoint {
    use PredefinedColorSpace::{ProphotoRgb, XyzD50};
    match space {
        RectangularSpace::Lab | RectangularSpace::Predefined(ProphotoRgb | XyzD50) => {
            WhitePoint::D50
        }
        _ => WhitePoint::D65,
    }
}

fn same_rgb_primaries(
    source: RectangularSpace,
    destination: RectangularSpace,
) -> Option<(PredefinedColorSpace, PredefinedColorSpace)> {
    use PredefinedColorSpace::{DisplayP3, DisplayP3Linear, Srgb, SrgbLinear};
    use RectangularSpace::Predefined;
    if let (Predefined(source), Predefined(destination)) = (source, destination)
        && matches!(
            (source, destination),
            (Srgb | SrgbLinear, Srgb | SrgbLinear)
                | (DisplayP3 | DisplayP3Linear, DisplayP3 | DisplayP3Linear)
        )
    {
        return Some((source, destination));
    }
    None
}

fn to_xyz(channels: [Wide; 3], space: RectangularSpace) -> [Wide; 3] {
    use PredefinedColorSpace::{XyzD50, XyzD65};
    match space {
        RectangularSpace::Lab => colorimetry::lab_to_xyz(channels),
        RectangularSpace::Oklab => colorimetry::oklab_to_xyz(channels),
        RectangularSpace::Predefined(XyzD50 | XyzD65) => channels,
        RectangularSpace::Predefined(rgb) => {
            colorimetry::rgb_to_xyz(channels.map(|v| transfer::decode(v, rgb)), rgb)
        }
    }
}

fn from_xyz(channels: [Wide; 3], space: RectangularSpace) -> [Wide; 3] {
    use PredefinedColorSpace::{XyzD50, XyzD65};
    match space {
        RectangularSpace::Lab => colorimetry::xyz_to_lab(channels),
        RectangularSpace::Oklab => colorimetry::xyz_to_oklab(channels),
        RectangularSpace::Predefined(XyzD50 | XyzD65) => channels,
        RectangularSpace::Predefined(rgb) => {
            colorimetry::xyz_to_rgb(channels, rgb).map(|v| transfer::encode(v, rgb))
        }
    }
}
