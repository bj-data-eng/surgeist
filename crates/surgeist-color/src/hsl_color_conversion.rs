//! Concrete HSL/HWB and encoded-sRGB coordinate math, without gamut mapping.

use crate::finite::{Finite, closest_finite};

/// Finite concrete HSL coordinates with optional hue in degrees.
///
/// S and L use percentage reference numbers: ordinary white has L = 100,
/// not 1. Saturation is nonnegative; extended finite S and L are accepted.
/// Present hue is normalized to `[0, 360)`. Construction retains missing or
/// present hue independently of saturation, and preserves lightness bits.
/// This carrier has no alpha or authored CSS semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HslCoordinates {
    hue_degrees: Option<Finite>,
    saturation: Finite,
    lightness: Finite,
}

impl HslCoordinates {
    /// Builds coordinates, normalizing present hue to `[0, 360)`.
    ///
    /// Negative saturation is the same colour at the opposite hue: the hue,
    /// missing as 0 degrees, rotates by 180 degrees and the saturation keeps
    /// its magnitude. Hue and saturation zero are canonical positive zero.
    /// Lightness retains its input bits, including negative zero. No inverse
    /// powerless threshold is applied and no authored saturation clamp is
    /// performed.
    ///
    /// ```
    /// use surgeist_color::{Finite, HslCoordinates};
    /// let finite = |value| Finite::new(value).unwrap();
    /// let hsl = HslCoordinates::new(Some(finite(-90.0)), finite(100.0), finite(50.0));
    /// assert_eq!(hsl.hue_degrees().map(Finite::get), Some(270.0));
    /// assert_eq!(hsl.saturation().get(), 100.0);
    /// assert_eq!(hsl.lightness().get(), 50.0);
    /// let rotated = HslCoordinates::new(Some(finite(30.0)), finite(-100.0), finite(50.0));
    /// assert_eq!(rotated.hue_degrees().map(Finite::get), Some(210.0));
    /// assert_eq!(rotated.saturation().get(), 100.0);
    /// ```
    #[must_use]
    pub fn new(hue_degrees: Option<Finite>, saturation: Finite, lightness: Finite) -> Self {
        let (hue, saturation) = opposite_hue(hue_degrees, saturation);
        Self {
            hue_degrees: hue.map(normalize_hue),
            saturation: if saturation.get() == 0.0 {
                closest_finite(0.0)
            } else {
                saturation
            },
            lightness,
        }
    }

    /// Normalized hue in degrees, or explicitly missing hue.
    #[must_use]
    pub const fn hue_degrees(&self) -> Option<Finite> {
        self.hue_degrees
    }

    /// Nonnegative saturation in percentage reference numbers.
    #[must_use]
    pub const fn saturation(&self) -> Finite {
        self.saturation
    }

    /// Lightness in percentage reference numbers, including extended values.
    #[must_use]
    pub const fn lightness(&self) -> Finite {
        self.lightness
    }
}

/// Finite concrete HWB coordinates with optional normalized hue in degrees.
///
/// W and B use percentage reference numbers, normally `0..100`; every finite
/// extended value, including negative values, is accepted and retained. This
/// carrier preserves missing or present hue independently of W and B. HWB sum
/// normalization occurs during conversion, not construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HwbCoordinates {
    hue_degrees: Option<Finite>,
    whiteness: Finite,
    blackness: Finite,
}

impl HwbCoordinates {
    /// Builds coordinates, normalizing only present hue to `[0, 360)`.
    ///
    /// W and B retain their exact input bits, including negative zero. No
    /// inverse powerless threshold or authored CSS clamp is applied.
    ///
    /// ```
    /// use surgeist_color::{Finite, HwbCoordinates};
    /// let finite = |value| Finite::new(value).unwrap();
    /// let hwb = HwbCoordinates::new(Some(finite(510.0)), finite(25.0), finite(12.5));
    /// assert_eq!(hwb.hue_degrees().map(Finite::get), Some(150.0));
    /// assert_eq!(hwb.whiteness().get(), 25.0);
    /// assert_eq!(hwb.blackness().get(), 12.5);
    /// ```
    #[must_use]
    pub fn new(hue_degrees: Option<Finite>, whiteness: Finite, blackness: Finite) -> Self {
        Self {
            hue_degrees: hue_degrees.map(|hue| normalize_hue(hue.get())),
            whiteness,
            blackness,
        }
    }

    /// Normalized hue in degrees, or explicitly missing hue.
    #[must_use]
    pub const fn hue_degrees(&self) -> Option<Finite> {
        self.hue_degrees
    }

    /// Whiteness in percentage reference numbers, including extended values.
    #[must_use]
    pub const fn whiteness(&self) -> Finite {
        self.whiteness
    }

    /// Blackness in percentage reference numbers, including extended values.
    #[must_use]
    pub const fn blackness(&self) -> Finite {
        self.blackness
    }
}

/// Converts concrete HSL to encoded sRGB using Color 4 §7.1.
///
/// S and L use `0..100` reference numbers; RGB uses `0..1` reference numbers.
/// Finite extended results are retained without clipping or gamut mapping.
/// Missing hue substitutes 0 degrees, even at nonzero saturation. Each
/// channel is `L - S·min(L, 1 - L)·t`. For normalized L outside `[0, 2]`,
/// where `1 - L` or `S·L` can lose the result, it is evaluated in the
/// algebraically equal forms `L(1 - S·t)` (L < 0) and `L(1 + S·t) - S·t`
/// (L > 2), so a hexagon vertex stays exact however large L is; elsewhere
/// near the binary64 limit those forms round S·t and the product, an error
/// negligible relative to the channel magnitude. A channel beyond binary64
/// is the signed closest finite value (CSS Values 4 closest supported value).
/// Ordinary binary64 rounding and underflow apply; exact round trips are not
/// promised, including after powerless inverse conversion.
///
/// ```
/// use surgeist_color::{Finite, HslCoordinates, convert_hsl_to_srgb};
/// let finite = |value| Finite::new(value).unwrap();
/// let hsl = HslCoordinates::new(None, finite(200.0), finite(50.0));
/// assert_eq!(convert_hsl_to_srgb(&hsl).map(Finite::get), [1.5, -0.5, -0.5]);
/// ```
#[must_use]
pub fn convert_hsl_to_srgb(hsl: &HslCoordinates) -> [Finite; 3] {
    let saturation = hsl.saturation.get() / 100.0;
    let lightness = hsl.lightness.get() / 100.0;
    let hue = hsl.hue_degrees.map_or(0.0, Finite::get);
    [0.0, 8.0, 4.0].map(|offset| {
        let k = (offset + hue / 30.0) % 12.0;
        let shape = (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        // Within [0, 2] |min(L, 1 - L)| <= 1 and 1 - L is exact, so the
        // direct form is finite. Outside it |shape| <= 1 keeps S·t finite;
        // only the product with L can overflow, and no infinity can meet an
        // opposite infinity.
        let st = saturation * shape;
        closest_finite(if (0.0..=2.0).contains(&lightness) {
            lightness - saturation * lightness.min(1.0 - lightness) * shape
        } else if lightness < 0.0 {
            lightness * (1.0 - st)
        } else {
            lightness * (1.0 + st) - st
        })
    })
}

/// Converts finite encoded sRGB to concrete HSL using Color 4 §7.2.
///
/// RGB uses `0..1` reference numbers, with every finite extended input allowed.
/// Returned S/L use percentage reference numbers. Singular L = 0 or 100 and
/// exact gray produce S = 0 and missing hue. Negative computed saturation is
/// made positive with a 180-degree hue rotation. Hue is missing at normalized
/// S <= `0.00001` (reference S <= `0.001`), while computed S is retained. This
/// selects the HSL sample and frozen WebKit behavior where the general
/// powerless prose differs; see the crate's conformance reference.
///
/// Stable midpoint and scaled differences avoid needless intermediate
/// overflow; a required S/L percentage beyond binary64 is the signed closest
/// finite value. Binary64 rounding, underflow, and powerless hue loss apply,
/// so conversion does not promise exact inversion.
///
/// ```
/// use surgeist_color::{Finite, convert_srgb_to_hsl};
/// let hsl = convert_srgb_to_hsl([-1.0, -0.5, 0.0].map(|value| Finite::new(value).unwrap()));
/// assert_eq!(hsl.hue_degrees().map(Finite::get), Some(30.0));
/// assert_eq!(hsl.saturation().get(), 100.0);
/// assert_eq!(hsl.lightness().get(), -50.0);
/// ```
#[must_use]
pub fn convert_srgb_to_hsl(rgb: [Finite; 3]) -> HslCoordinates {
    let rgb = rgb.map(Finite::get);
    let minimum = rgb.into_iter().fold(f64::INFINITY, f64::min);
    let maximum = rgb.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let sum = minimum + maximum;
    let lightness = if sum.is_finite() {
        sum / 2.0
    } else {
        minimum / 2.0 + maximum / 2.0
    };
    let lightness_percent = closest_finite(lightness * 100.0);
    // These branches discard hue. In particular MAX - (-MAX) is irrelevant
    // for singular lightness.
    if minimum == maximum || lightness == 0.0 || lightness == 1.0 {
        return HslCoordinates::new(None, closest_finite(0.0), lightness_percent);
    }
    let difference = maximum - minimum;
    let half_difference = if difference.is_finite() {
        difference / 2.0
    } else {
        maximum / 2.0 - minimum / 2.0
    };
    let saturation = half_difference / lightness.min(1.0 - lightness);
    let saturation_percent = closest_finite(saturation.abs() * 100.0);
    let hue = if saturation.abs() <= 0.00001 {
        None
    } else {
        let hue = rgb_hue(rgb, minimum, maximum).get();
        Some(closest_finite(if saturation < 0.0 {
            hue + 180.0
        } else {
            hue
        }))
    };
    HslCoordinates::new(hue, saturation_percent, lightness_percent)
}

/// Converts concrete HWB to encoded sRGB using Color 4 §8.1.
///
/// W/B use percentage reference numbers; RGB uses `0..1` reference numbers.
/// Missing hue substitutes 0 degrees. When normalized W + B >= 1, all three
/// channels equal W / (W + B); otherwise the hue basis is scaled by 1 - W - B
/// and shifted by W. When normalized |W| or |B| exceeds 1 this is evaluated
/// as the equal `basis·(1 - B) + W·(1 - basis)`, so a hexagon vertex (1 - B
/// or W) stays exact however large |W| or |B| is.
/// Extended coordinates are not clipped or gamut mapped.
///
/// Finite W/B divided by 100 bound all intermediate sums and channels, even
/// at finite extremes. Ordinary binary64 rounding and underflow apply, with
/// no exact round-trip promise.
///
/// ```
/// use surgeist_color::{Finite, HwbCoordinates, convert_hwb_to_srgb};
/// let finite = |value| Finite::new(value).unwrap();
/// let hwb = HwbCoordinates::new(Some(finite(150.0)), finite(25.0), finite(12.5));
/// assert_eq!(convert_hwb_to_srgb(&hwb).map(Finite::get), [0.25, 0.875, 0.5625]);
/// ```
#[must_use]
pub fn convert_hwb_to_srgb(hwb: &HwbCoordinates) -> [Finite; 3] {
    let whiteness = hwb.whiteness.get() / 100.0;
    let blackness = hwb.blackness.get() / 100.0;
    let sum = whiteness + blackness;
    if sum >= 1.0 {
        return [closest_finite(whiteness / sum); 3];
    }
    let hue = hwb.hue_degrees.map_or(0.0, Finite::get);
    [0.0, 8.0, 4.0].map(|offset| {
        let k = (offset + hue / 30.0) % 12.0;
        let basis = 0.5 - 0.5 * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        closest_finite(if whiteness.abs() <= 1.0 && blackness.abs() <= 1.0 {
            basis * (1.0 - sum) + whiteness
        } else {
            basis * (1.0 - blackness) + whiteness * (1.0 - basis)
        })
    })
}

/// Converts finite encoded sRGB to concrete HWB using Color 4 §8.2.
///
/// Inputs use `0..1` reference numbers; every finite extended input is
/// accepted without clipping. W = min(R,G,B) and B = 1 - max(R,G,B), returned
/// in percentage reference numbers; a required percentage beyond binary64 is
/// the signed closest finite value. Hue is missing for exact gray or
/// normalized W + B >= `1 - 0.00001`. Computed W/B are retained, selecting the
/// HWB sample and frozen WebKit behavior over the conflicting general powerless
/// cleanup prose; see the crate's conformance reference.
///
/// Hue uses the RGB hexagon without HSL's negative-saturation rotation. Scaled
/// differences avoid needless intermediate overflow. Ordinary binary64
/// rounding, underflow, and powerless hue loss preclude exact inversion.
///
/// ```
/// use surgeist_color::{Finite, convert_srgb_to_hwb};
/// let hwb = convert_srgb_to_hwb([-1.0, -0.5, 0.0].map(|value| Finite::new(value).unwrap()));
/// assert_eq!(hwb.hue_degrees().map(Finite::get), Some(210.0));
/// assert_eq!(hwb.whiteness().get(), -100.0);
/// assert_eq!(hwb.blackness().get(), 100.0);
/// ```
#[must_use]
pub fn convert_srgb_to_hwb(rgb: [Finite; 3]) -> HwbCoordinates {
    let rgb = rgb.map(Finite::get);
    let minimum = rgb.into_iter().fold(f64::INFINITY, f64::min);
    let maximum = rgb.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let blackness = 1.0 - maximum;
    let whiteness_percent = closest_finite(minimum * 100.0);
    let blackness_percent = closest_finite(blackness * 100.0);
    let hue = if minimum == maximum || minimum + blackness >= 1.0 - 0.00001 {
        None
    } else {
        Some(rgb_hue(rgb, minimum, maximum))
    };
    HwbCoordinates::new(hue, whiteness_percent, blackness_percent)
}

/// Negative saturation or chroma is the same point at the opposite hue: a
/// missing hue reads as 0 degrees, rotates by 180 degrees, and the radius
/// keeps its magnitude. A nonnegative radius keeps its hue unchanged.
pub(crate) fn opposite_hue(hue: Option<Finite>, radius: Finite) -> (Option<f64>, Finite) {
    if radius.get() < 0.0 {
        (
            Some((hue.map_or(0.0, Finite::get).rem_euclid(360.0) + 180.0).rem_euclid(360.0)),
            closest_finite(radius.get().abs()),
        )
    } else {
        (hue.map(Finite::get), radius)
    }
}

pub(crate) fn normalize_hue(hue: f64) -> Finite {
    let normalized = hue.rem_euclid(360.0);
    // A tiny negative remainder can round to the excluded endpoint 360.
    closest_finite(if normalized == 0.0 || normalized >= 360.0 {
        0.0
    } else {
        normalized
    })
}

fn rgb_hue(rgb: [f64; 3], minimum: f64, maximum: f64) -> Finite {
    let difference = maximum - minimum;
    let ratio = |a: f64, b: f64| {
        if difference.is_finite() {
            (a - b) / difference
        } else {
            let scale = minimum.abs().max(maximum.abs());
            (a / scale - b / scale) / (maximum / scale - minimum / scale)
        }
    };
    let [red, green, blue] = rgb;
    let sector = if maximum == red {
        ratio(green, blue)
    } else if maximum == green {
        ratio(blue, red) + 2.0
    } else {
        ratio(red, green) + 4.0
    };
    normalize_hue(sector * 60.0)
}
