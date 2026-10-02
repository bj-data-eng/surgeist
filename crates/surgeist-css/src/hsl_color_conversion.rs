//! Concrete HSL/HWB and encoded-sRGB coordinate math, without gamut mapping.

use std::fmt;

/// Invalid input or a required result outside finite binary64 coordinates.
///
/// Constructors check present H, then S/W, then L/B for finiteness before
/// rejecting negative HSL saturation. RGB conversions check R, G, B in that
/// order. Result overflow is distinct from nonfinite caller input.
///
/// This error can gain variants; downstream matches need a fallback.
///
/// ```compile_fail
/// use surgeist_css::CssHslHwbConversionError;
/// fn exhaustive(error: CssHslHwbConversionError) {
///     match error {
///         CssHslHwbConversionError::NonFiniteInput { .. } => (),
///         CssHslHwbConversionError::NegativeSaturation => (),
///         CssHslHwbConversionError::UnrepresentableResult => (),
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHslHwbConversionError {
    /// The first nonfinite component, using the documented input order.
    NonFiniteInput { component: usize },
    /// Concrete HSL saturation must already be nonnegative.
    NegativeSaturation,
    /// A required amplitude, channel, or percentage cannot be a finite `f64`.
    UnrepresentableResult,
}

impl fmt::Display for CssHslHwbConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteInput { component } => write!(
                formatter,
                "HSL/HWB conversion input component {component} is not finite"
            ),
            Self::NegativeSaturation => formatter.write_str("HSL saturation is negative"),
            Self::UnrepresentableResult => {
                formatter.write_str("HSL/HWB conversion result is not representable as finite f64")
            }
        }
    }
}

impl std::error::Error for CssHslHwbConversionError {}

/// Finite concrete HSL coordinates with optional hue in degrees.
///
/// S and L use percentage reference numbers: ordinary white has L = 100,
/// not 1. Saturation is nonnegative; extended finite S and L are accepted.
/// Present hue is normalized to `[0, 360)`. Construction retains missing or
/// present hue independently of saturation, and preserves lightness bits.
/// This carrier has no alpha or authored CSS semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssHslColorCoordinates {
    hue_degrees: Option<f64>,
    saturation: f64,
    lightness: f64,
}

impl CssHslColorCoordinates {
    /// Checks H (if present), S, L in index order before rejecting negative S.
    ///
    /// Hue and saturation zero are canonical positive zero. Lightness retains
    /// its input bits, including negative zero. No inverse powerless threshold
    /// is applied here, and no authored saturation clamp is performed.
    ///
    /// ```
    /// use surgeist_css::CssHslColorCoordinates;
    /// let hsl = CssHslColorCoordinates::try_new(Some(-90.0), 100.0, 50.0).unwrap();
    /// assert_eq!(hsl.hue_degrees(), Some(270.0));
    /// assert_eq!(hsl.saturation(), 100.0);
    /// assert_eq!(hsl.lightness(), 50.0);
    /// ```
    pub fn try_new(
        hue_degrees: Option<f64>,
        saturation: f64,
        lightness: f64,
    ) -> Result<Self, CssHslHwbConversionError> {
        validate_input([hue_degrees.unwrap_or(0.0), saturation, lightness])?;
        if saturation < 0.0 {
            return Err(CssHslHwbConversionError::NegativeSaturation);
        }
        Ok(Self {
            hue_degrees: hue_degrees.map(normalize_hue),
            saturation: if saturation == 0.0 { 0.0 } else { saturation },
            lightness,
        })
    }

    /// Normalized hue in degrees, or explicitly missing hue.
    #[must_use]
    pub const fn hue_degrees(&self) -> Option<f64> {
        self.hue_degrees
    }

    /// Nonnegative saturation in percentage reference numbers.
    #[must_use]
    pub const fn saturation(&self) -> f64 {
        self.saturation
    }

    /// Lightness in percentage reference numbers, including extended values.
    #[must_use]
    pub const fn lightness(&self) -> f64 {
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
pub struct CssHwbColorCoordinates {
    hue_degrees: Option<f64>,
    whiteness: f64,
    blackness: f64,
}

impl CssHwbColorCoordinates {
    /// Checks H (if present), W, B in index order and normalizes only hue.
    ///
    /// W and B retain their exact input bits, including negative zero. No
    /// inverse powerless threshold or authored CSS clamp is applied.
    ///
    /// ```
    /// use surgeist_css::CssHwbColorCoordinates;
    /// let hwb = CssHwbColorCoordinates::try_new(Some(510.0), 25.0, 12.5).unwrap();
    /// assert_eq!(hwb.hue_degrees(), Some(150.0));
    /// assert_eq!(hwb.whiteness(), 25.0);
    /// assert_eq!(hwb.blackness(), 12.5);
    /// ```
    pub fn try_new(
        hue_degrees: Option<f64>,
        whiteness: f64,
        blackness: f64,
    ) -> Result<Self, CssHslHwbConversionError> {
        validate_input([hue_degrees.unwrap_or(0.0), whiteness, blackness])?;
        Ok(Self {
            hue_degrees: hue_degrees.map(normalize_hue),
            whiteness,
            blackness,
        })
    }

    /// Normalized hue in degrees, or explicitly missing hue.
    #[must_use]
    pub const fn hue_degrees(&self) -> Option<f64> {
        self.hue_degrees
    }

    /// Whiteness in percentage reference numbers, including extended values.
    #[must_use]
    pub const fn whiteness(&self) -> f64 {
        self.whiteness
    }

    /// Blackness in percentage reference numbers, including extended values.
    #[must_use]
    pub const fn blackness(&self) -> f64 {
        self.blackness
    }
}

/// Converts concrete HSL to encoded sRGB using Color 4 §7.1.
///
/// S and L use `0..100` reference numbers; RGB uses `0..1` reference numbers.
/// Finite extended results are retained without clipping or gamut mapping.
/// Missing hue substitutes 0 degrees, even at nonzero saturation. A required
/// amplitude or channel that overflows returns [`CssHslHwbConversionError::UnrepresentableResult`].
/// Ordinary binary64 rounding and underflow apply; exact round trips are not
/// promised, including after powerless inverse conversion.
///
/// ```
/// use surgeist_css::{CssHslColorCoordinates, convert_hsl_to_srgb};
/// let hsl = CssHslColorCoordinates::try_new(None, 200.0, 50.0).unwrap();
/// assert_eq!(convert_hsl_to_srgb(&hsl).unwrap(), [1.5, -0.5, -0.5]);
/// ```
pub fn convert_hsl_to_srgb(
    hsl: &CssHslColorCoordinates,
) -> Result<[f64; 3], CssHslHwbConversionError> {
    let saturation = hsl.saturation / 100.0;
    let lightness = hsl.lightness / 100.0;
    let amplitude = finite_result(saturation * lightness.min(1.0 - lightness))?;
    let hue = hsl.hue_degrees.unwrap_or(0.0);
    let mut rgb = [0.0; 3];
    for (channel, offset) in rgb.iter_mut().zip([0.0, 8.0, 4.0]) {
        let k = (offset + hue / 30.0) % 12.0;
        let shape = (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        *channel = finite_result(lightness - amplitude * shape)?;
    }
    Ok(rgb)
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
/// Inputs are checked R, G, B before arithmetic. Stable midpoint and scaled
/// differences avoid needless intermediate overflow; a required S/L percentage
/// that overflows returns an error. Binary64 rounding, underflow, and powerless
/// hue loss apply, so conversion does not promise exact inversion.
///
/// ```
/// use surgeist_css::convert_srgb_to_hsl;
/// let hsl = convert_srgb_to_hsl([-1.0, -0.5, 0.0]).unwrap();
/// assert_eq!(hsl.hue_degrees(), Some(30.0));
/// assert_eq!(hsl.saturation(), 100.0);
/// assert_eq!(hsl.lightness(), -50.0);
/// ```
pub fn convert_srgb_to_hsl(
    rgb: [f64; 3],
) -> Result<CssHslColorCoordinates, CssHslHwbConversionError> {
    validate_input(rgb)?;
    let minimum = rgb.into_iter().fold(f64::INFINITY, f64::min);
    let maximum = rgb.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let sum = minimum + maximum;
    let lightness = if sum.is_finite() {
        sum / 2.0
    } else {
        minimum / 2.0 + maximum / 2.0
    };
    let lightness_percent = finite_result(lightness * 100.0)?;
    // These branches discard hue. In particular MAX - (-MAX) is irrelevant
    // for singular lightness and must not reject a representable result.
    if minimum == maximum || lightness == 0.0 || lightness == 1.0 {
        return CssHslColorCoordinates::try_new(None, 0.0, lightness_percent);
    }
    let difference = maximum - minimum;
    let half_difference = if difference.is_finite() {
        difference / 2.0
    } else {
        maximum / 2.0 - minimum / 2.0
    };
    let saturation = half_difference / lightness.min(1.0 - lightness);
    let saturation_percent = finite_result(saturation.abs() * 100.0)?;
    let hue = if saturation.abs() <= 0.00001 {
        None
    } else {
        let hue = rgb_hue(rgb, minimum, maximum);
        Some(if saturation < 0.0 { hue + 180.0 } else { hue })
    };
    CssHslColorCoordinates::try_new(hue, saturation_percent, lightness_percent)
}

/// Converts concrete HWB to encoded sRGB using Color 4 §8.1.
///
/// W/B use percentage reference numbers; RGB uses `0..1` reference numbers.
/// Missing hue substitutes 0 degrees. When normalized W + B >= 1, all three
/// channels equal W / (W + B); otherwise the hue basis is scaled by 1 - W - B
/// and shifted by W. Extended coordinates are not clipped or gamut mapped.
///
/// Checked finite W/B divided by 100 bound all intermediate sums and channels,
/// making this operation infallible even at finite extremes. Ordinary binary64
/// rounding and underflow apply, with no exact round-trip promise.
///
/// ```
/// use surgeist_css::{CssHwbColorCoordinates, convert_hwb_to_srgb};
/// let hwb = CssHwbColorCoordinates::try_new(Some(150.0), 25.0, 12.5).unwrap();
/// assert_eq!(convert_hwb_to_srgb(&hwb), [0.25, 0.875, 0.5625]);
/// ```
#[must_use]
pub fn convert_hwb_to_srgb(hwb: &CssHwbColorCoordinates) -> [f64; 3] {
    let whiteness = hwb.whiteness / 100.0;
    let blackness = hwb.blackness / 100.0;
    let sum = whiteness + blackness;
    if sum >= 1.0 {
        return [whiteness / sum; 3];
    }
    let hue = hwb.hue_degrees.unwrap_or(0.0);
    [0.0, 8.0, 4.0].map(|offset| {
        let k = (offset + hue / 30.0) % 12.0;
        let basis = 0.5 - 0.5 * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        basis * (1.0 - sum) + whiteness
    })
}

/// Converts finite encoded sRGB to concrete HWB using Color 4 §8.2.
///
/// Inputs use `0..1` reference numbers and are checked R, G, B in index order.
/// Every finite extended input is accepted without clipping. W = min(R,G,B)
/// and B = 1 - max(R,G,B), returned in percentage reference numbers; overflow
/// of a required percentage returns an error. Hue is missing for exact gray or
/// normalized W + B >= `1 - 0.00001`. Computed W/B are retained, selecting the
/// HWB sample and frozen WebKit behavior over the conflicting general powerless
/// cleanup prose; see the crate's conformance reference.
///
/// Hue uses the RGB hexagon without HSL's negative-saturation rotation. Scaled
/// differences avoid needless intermediate overflow. Ordinary binary64
/// rounding, underflow, and powerless hue loss preclude exact inversion.
///
/// ```
/// use surgeist_css::convert_srgb_to_hwb;
/// let hwb = convert_srgb_to_hwb([-1.0, -0.5, 0.0]).unwrap();
/// assert_eq!(hwb.hue_degrees(), Some(210.0));
/// assert_eq!(hwb.whiteness(), -100.0);
/// assert_eq!(hwb.blackness(), 100.0);
/// ```
pub fn convert_srgb_to_hwb(
    rgb: [f64; 3],
) -> Result<CssHwbColorCoordinates, CssHslHwbConversionError> {
    validate_input(rgb)?;
    let minimum = rgb.into_iter().fold(f64::INFINITY, f64::min);
    let maximum = rgb.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let blackness = 1.0 - maximum;
    let whiteness_percent = finite_result(minimum * 100.0)?;
    let blackness_percent = finite_result(blackness * 100.0)?;
    let hue = if minimum == maximum || minimum + blackness >= 1.0 - 0.00001 {
        None
    } else {
        Some(rgb_hue(rgb, minimum, maximum))
    };
    CssHwbColorCoordinates::try_new(hue, whiteness_percent, blackness_percent)
}

fn validate_input(input: [f64; 3]) -> Result<(), CssHslHwbConversionError> {
    for (component, value) in input.into_iter().enumerate() {
        if !value.is_finite() {
            return Err(CssHslHwbConversionError::NonFiniteInput { component });
        }
    }
    Ok(())
}

fn finite_result(value: f64) -> Result<f64, CssHslHwbConversionError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CssHslHwbConversionError::UnrepresentableResult)
    }
}

fn normalize_hue(hue: f64) -> f64 {
    let normalized = hue.rem_euclid(360.0);
    // A tiny negative remainder can round to the excluded endpoint 360.
    if normalized == 0.0 || normalized >= 360.0 {
        0.0
    } else {
        normalized
    }
}

fn rgb_hue(rgb: [f64; 3], minimum: f64, maximum: f64) -> f64 {
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
