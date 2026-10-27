//! Pure rectangular/polar coordinate formulas from Color 4 CRD 2026-09-08 §§9.3–9.6.

use crate::finite::{Finite, closest_finite};
use crate::hsl_color_conversion::{normalize_hue, opposite_hue};

/// Finite concrete lightness, nonnegative chroma, and optional hue in degrees.
///
/// The selected conversion function determines Lab or Oklab units. This is
/// independent of authored CSS, alpha, profiles, and gamut handling. Present hue
/// lies in `[0, 360)`; zero chroma and hue have positive-zero bits. Lightness
/// retains its input bits, including negative zero. Missing hue is explicit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolarCoordinates {
    lightness: Finite,
    chroma: Finite,
    hue_degrees: Option<Finite>,
}

impl PolarCoordinates {
    /// Builds coordinates, normalizing present hue modulo 360.
    ///
    /// Negative chroma is the same Cartesian point at the opposite hue: the
    /// hue, missing as 0 degrees, rotates by 180 degrees and the chroma keeps
    /// its magnitude. Chroma and hue zero are canonical positive zero. No
    /// space-specific neutral threshold is applied: `None` remains missing at
    /// any nonnegative chroma, and a present hue stays present even at zero
    /// chroma.
    ///
    /// ```
    /// use surgeist_color::{Finite, PolarCoordinates};
    /// let finite = |value| Finite::new(value).unwrap();
    /// let polar = PolarCoordinates::new(finite(-2.0), finite(5.0), Some(finite(-90.0)));
    /// assert_eq!(polar.lightness().get(), -2.0);
    /// assert_eq!(polar.chroma().get(), 5.0);
    /// assert_eq!(polar.hue_degrees().map(Finite::get), Some(270.0));
    /// let rotated = PolarCoordinates::new(finite(50.0), finite(-5.0), None);
    /// assert_eq!(rotated.chroma().get(), 5.0);
    /// assert_eq!(rotated.hue_degrees().map(Finite::get), Some(180.0));
    /// ```
    #[must_use]
    pub fn new(lightness: Finite, chroma: Finite, hue_degrees: Option<Finite>) -> Self {
        let (hue, chroma) = opposite_hue(hue_degrees, chroma);
        Self {
            lightness,
            chroma: if chroma.get() == 0.0 {
                closest_finite(0.0)
            } else {
                chroma
            },
            hue_degrees: hue.map(normalize_hue),
        }
    }

    /// Concrete lightness in the selected Lab-family space's number units.
    #[must_use]
    pub const fn lightness(&self) -> Finite {
        self.lightness
    }

    /// Nonnegative chroma in the selected Lab-family space's number units.
    #[must_use]
    pub const fn chroma(&self) -> Finite {
        self.chroma
    }

    /// Normalized hue in degrees, or the explicitly missing hue.
    #[must_use]
    pub const fn hue_degrees(&self) -> Option<Finite> {
        self.hue_degrees
    }
}

/// Converts concrete `[L, a, b]` Lab coordinates to LCH using Color 4 §9.5.
///
/// Lab number units have typical L in `0..100`. All finite extended coordinates
/// are accepted without clamping. L is copied bit-for-bit; C uses stable hypot,
/// and a chroma beyond binary64 is the closest finite value, `f64::MAX`.
/// Exactly C greater than `0.0015` produces normalized atan2(b, a) hue in degrees;
/// at or below this threshold hue is missing and chroma becomes positive zero,
/// applying Color 4 §4.4.1's conversion-generated neutral cleanup.
/// Ordinary `f64` rounding applies, and neutral conversion loses the axes on
/// inversion. This does not evaluate authored CSS or convert between spaces.
///
/// ```
/// use surgeist_color::{Finite, convert_lab_to_lch};
/// let polar = convert_lab_to_lch([50.0, 3.0, 4.0].map(|value| Finite::new(value).unwrap()));
/// assert_eq!(polar.lightness().get(), 50.0);
/// assert_eq!(polar.chroma().get(), 5.0);
/// assert!((polar.hue_degrees().unwrap().get() - 53.13010235415598).abs() < 1e-12);
/// ```
#[must_use]
pub fn convert_lab_to_lch(lab: [Finite; 3]) -> PolarCoordinates {
    rectangular_to_polar(lab, 0.0015)
}

/// Converts concrete `[L, a, b]` Oklab coordinates to Oklch using Color 4 §9.5.
///
/// Oklab number units have typical L in `0..1`. This follows
/// [`convert_lab_to_lch`]'s extended-coordinate, lightness, stable hypot,
/// closest-finite chroma, neutral cleanup and rounding contract, but hue is
/// present only for C strictly greater than `0.000004`, as specified in §9.4.
///
/// ```
/// use surgeist_color::{Finite, convert_oklab_to_oklch};
/// let polar = convert_oklab_to_oklch([0.5, 0.0, 0.2].map(|value| Finite::new(value).unwrap()));
/// assert_eq!(polar.chroma().get(), 0.2);
/// assert_eq!(polar.hue_degrees().map(Finite::get), Some(90.0));
/// ```
#[must_use]
pub fn convert_oklab_to_oklch(oklab: [Finite; 3]) -> PolarCoordinates {
    rectangular_to_polar(oklab, 0.000004)
}

/// Converts concrete LCH coordinates to `[L, a, b]` Lab using Color 4 §9.6.
///
/// L is copied bit-for-bit. Missing hue sets both axes to positive zero at any
/// chroma. Present hue produces C cos(H) and C sin(H), with ordinary `f64`
/// rounding, even below the neutral threshold. Finite coordinates bound both
/// axes by the chroma; an exact round trip is not promised.
///
/// ```
/// use surgeist_color::{Finite, PolarCoordinates, convert_lch_to_lab};
/// let finite = |value| Finite::new(value).unwrap();
/// let polar = PolarCoordinates::new(finite(50.0), finite(5.0), None);
/// assert_eq!(convert_lch_to_lab(&polar).map(Finite::get), [50.0, 0.0, 0.0]);
/// ```
#[must_use]
pub fn convert_lch_to_lab(lch: &PolarCoordinates) -> [Finite; 3] {
    polar_to_rectangular(lch)
}

/// Converts concrete Oklch coordinates to `[L, a, b]` Oklab using Color 4 §9.6.
///
/// This follows [`convert_lch_to_lab`]'s missing-hue, trigonometry, bit-preserved
/// lightness, and rounding contract, in Oklab number units (typical L `0..1`).
///
/// ```
/// use surgeist_color::{Finite, PolarCoordinates, convert_oklch_to_oklab};
/// let finite = |value| Finite::new(value).unwrap();
/// let polar = PolarCoordinates::new(finite(0.5), finite(0.25), Some(finite(0.0)));
/// assert_eq!(convert_oklch_to_oklab(&polar).map(Finite::get), [0.5, 0.25, 0.0]);
/// ```
#[must_use]
pub fn convert_oklch_to_oklab(oklch: &PolarCoordinates) -> [Finite; 3] {
    polar_to_rectangular(oklch)
}

fn rectangular_to_polar(input: [Finite; 3], epsilon: f64) -> PolarCoordinates {
    let [_, a, b] = input.map(Finite::get);
    let chroma = a.hypot(b);
    if chroma <= epsilon {
        return PolarCoordinates {
            lightness: input[0],
            chroma: closest_finite(0.0),
            hue_degrees: None,
        };
    }
    PolarCoordinates {
        lightness: input[0],
        chroma: closest_finite(chroma),
        hue_degrees: Some(normalize_hue(b.atan2(a).to_degrees())),
    }
}

fn polar_to_rectangular(polar: &PolarCoordinates) -> [Finite; 3] {
    match polar.hue_degrees {
        Some(hue) => {
            let (sine, cosine) = hue.get().to_radians().sin_cos();
            [
                polar.lightness,
                closest_finite(polar.chroma.get() * cosine),
                closest_finite(polar.chroma.get() * sine),
            ]
        }
        None => [polar.lightness, closest_finite(0.0), closest_finite(0.0)],
    }
}
