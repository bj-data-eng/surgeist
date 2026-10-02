//! Pure rectangular/polar coordinate formulas from Color 4 CRD 2026-09-08 §§9.3–9.6.

use std::fmt;

/// Why concrete Lab-family coordinates could not be converted or constructed.
///
/// Downstream matches must allow future error variants:
///
/// ```compile_fail
/// use surgeist_css::CssPolarColorConversionError;
/// fn classify(error: CssPolarColorConversionError) -> &'static str {
///     match error {
///         CssPolarColorConversionError::NonFiniteInput { .. } => "input",
///         CssPolarColorConversionError::NegativeChroma => "chroma",
///         CssPolarColorConversionError::UnrepresentableResult => "result",
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPolarColorConversionError {
    /// The first nonfinite coordinate in the documented L, a/C, b/H order.
    NonFiniteInput { component: usize },
    /// Chroma is negative; negative zero is accepted.
    NegativeChroma,
    /// Required chroma cannot be represented as a finite `f64`.
    UnrepresentableResult,
}

impl fmt::Display for CssPolarColorConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteInput { component } => write!(
                formatter,
                "polar color conversion input component {component} is not finite"
            ),
            Self::NegativeChroma => formatter.write_str("polar color chroma is negative"),
            Self::UnrepresentableResult => formatter
                .write_str("polar color conversion result is not representable as finite f64"),
        }
    }
}

impl std::error::Error for CssPolarColorConversionError {}

/// Finite concrete lightness, nonnegative chroma, and optional hue in degrees.
///
/// The selected conversion function determines Lab or Oklab units. This is
/// independent of authored CSS, alpha, profiles, and gamut handling. Present hue
/// lies in `[0, 360)`; zero chroma and hue have positive-zero bits. Lightness
/// retains its input bits, including negative zero. Missing hue is explicit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssPolarColorCoordinates {
    lightness: f64,
    chroma: f64,
    hue_degrees: Option<f64>,
}

impl CssPolarColorCoordinates {
    /// Checks L, C, then present H for finiteness before rejecting negative C.
    ///
    /// Accepts extended finite lightness and chroma, normalizes present hue
    /// modulo 360, and canonicalizes chroma/hue zero to positive zero. No
    /// space-specific neutral threshold is applied: `None` remains missing at
    /// any chroma, and a present hue stays present even at zero chroma.
    ///
    /// ```
    /// use surgeist_css::CssPolarColorCoordinates;
    /// let polar = CssPolarColorCoordinates::try_new(-2.0, 5.0, Some(-90.0)).unwrap();
    /// assert_eq!(polar.lightness(), -2.0);
    /// assert_eq!(polar.chroma(), 5.0);
    /// assert_eq!(polar.hue_degrees(), Some(270.0));
    /// ```
    pub fn try_new(
        lightness: f64,
        chroma: f64,
        hue_degrees: Option<f64>,
    ) -> Result<Self, CssPolarColorConversionError> {
        validate_input([lightness, chroma, hue_degrees.unwrap_or(0.0)])?;
        if chroma < 0.0 {
            return Err(CssPolarColorConversionError::NegativeChroma);
        }
        Ok(Self {
            lightness,
            chroma: if chroma == 0.0 { 0.0 } else { chroma },
            hue_degrees: hue_degrees.map(normalize_hue),
        })
    }

    /// Concrete lightness in the selected Lab-family space's number units.
    #[must_use]
    pub const fn lightness(&self) -> f64 {
        self.lightness
    }

    /// Nonnegative chroma in the selected Lab-family space's number units.
    #[must_use]
    pub const fn chroma(&self) -> f64 {
        self.chroma
    }

    /// Normalized hue in degrees, or the explicitly missing hue.
    #[must_use]
    pub const fn hue_degrees(&self) -> Option<f64> {
        self.hue_degrees
    }
}

/// Converts concrete `[L, a, b]` Lab coordinates to LCH using Color 4 §9.5.
///
/// Lab number units have typical L in `0..100`. All finite extended coordinates
/// are accepted without clamping. L is copied bit-for-bit; C uses stable hypot.
/// Exactly C greater than `0.0015` produces normalized atan2(b, a) hue in degrees;
/// at or below this threshold hue is missing and chroma becomes positive zero,
/// applying Color 4 §4.4.1's conversion-generated neutral cleanup.
/// Nonfinite inputs are rejected in
/// index order before arithmetic; overflowing required C returns an error.
/// Ordinary `f64` rounding applies, and neutral conversion loses the axes on
/// inversion. This does not evaluate authored CSS or convert between spaces.
///
/// ```
/// use surgeist_css::convert_lab_to_lch;
/// let polar = convert_lab_to_lch([50.0, 3.0, 4.0]).unwrap();
/// assert_eq!(polar.lightness(), 50.0);
/// assert_eq!(polar.chroma(), 5.0);
/// assert!((polar.hue_degrees().unwrap() - 53.13010235415598).abs() < 1e-12);
/// ```
pub fn convert_lab_to_lch(
    lab: [f64; 3],
) -> Result<CssPolarColorCoordinates, CssPolarColorConversionError> {
    rectangular_to_polar(lab, 0.0015)
}

/// Converts concrete `[L, a, b]` Oklab coordinates to Oklch using Color 4 §9.5.
///
/// Oklab number units have typical L in `0..1`. This follows
/// [`convert_lab_to_lch`]'s finite-input, extended-coordinate, lightness, stable
/// hypot, neutral cleanup, error, and rounding contract, but hue is present only
/// for C strictly greater than `0.000004`, as specified in §9.4.
///
/// ```
/// use surgeist_css::convert_oklab_to_oklch;
/// let polar = convert_oklab_to_oklch([0.5, 0.0, 0.2]).unwrap();
/// assert_eq!(polar.chroma(), 0.2);
/// assert_eq!(polar.hue_degrees(), Some(90.0));
/// ```
pub fn convert_oklab_to_oklch(
    oklab: [f64; 3],
) -> Result<CssPolarColorCoordinates, CssPolarColorConversionError> {
    rectangular_to_polar(oklab, 0.000004)
}

/// Converts concrete LCH coordinates to `[L, a, b]` Lab using Color 4 §9.6.
///
/// L is copied bit-for-bit. Missing hue sets both axes to positive zero at any
/// chroma. Present hue produces C cos(H) and C sin(H), with ordinary `f64`
/// rounding, even below the neutral threshold. Checked finite coordinates make
/// this infallible; an exact round trip is not promised.
///
/// ```
/// use surgeist_css::{CssPolarColorCoordinates, convert_lch_to_lab};
/// let polar = CssPolarColorCoordinates::try_new(50.0, 5.0, None).unwrap();
/// assert_eq!(convert_lch_to_lab(&polar), [50.0, 0.0, 0.0]);
/// ```
#[must_use]
pub fn convert_lch_to_lab(lch: &CssPolarColorCoordinates) -> [f64; 3] {
    polar_to_rectangular(lch)
}

/// Converts concrete Oklch coordinates to `[L, a, b]` Oklab using Color 4 §9.6.
///
/// This follows [`convert_lch_to_lab`]'s missing-hue, trigonometry, bit-preserved
/// lightness, and rounding contract, in Oklab number units (typical L `0..1`).
///
/// ```
/// use surgeist_css::{CssPolarColorCoordinates, convert_oklch_to_oklab};
/// let polar = CssPolarColorCoordinates::try_new(0.5, 0.25, Some(0.0)).unwrap();
/// assert_eq!(convert_oklch_to_oklab(&polar), [0.5, 0.25, 0.0]);
/// ```
#[must_use]
pub fn convert_oklch_to_oklab(oklch: &CssPolarColorCoordinates) -> [f64; 3] {
    polar_to_rectangular(oklch)
}

fn validate_input(input: [f64; 3]) -> Result<(), CssPolarColorConversionError> {
    for (component, value) in input.iter().enumerate() {
        if !value.is_finite() {
            return Err(CssPolarColorConversionError::NonFiniteInput { component });
        }
    }
    Ok(())
}

fn normalize_hue(hue: f64) -> f64 {
    let normalized = hue.rem_euclid(360.0);
    // A tiny negative remainder can round up to the excluded endpoint 360.
    if normalized == 0.0 || normalized >= 360.0 {
        0.0
    } else {
        normalized
    }
}

fn rectangular_to_polar(
    input: [f64; 3],
    epsilon: f64,
) -> Result<CssPolarColorCoordinates, CssPolarColorConversionError> {
    validate_input(input)?;
    let [lightness, a, b] = input;
    let chroma = a.hypot(b);
    if !chroma.is_finite() {
        return Err(CssPolarColorConversionError::UnrepresentableResult);
    }
    if chroma <= epsilon {
        return Ok(CssPolarColorCoordinates {
            lightness,
            chroma: 0.0,
            hue_degrees: None,
        });
    }
    Ok(CssPolarColorCoordinates {
        lightness,
        chroma,
        hue_degrees: Some(normalize_hue(b.atan2(a).to_degrees())),
    })
}

fn polar_to_rectangular(polar: &CssPolarColorCoordinates) -> [f64; 3] {
    match polar.hue_degrees {
        Some(hue) => {
            let (sine, cosine) = hue.to_radians().sin_cos();
            [polar.lightness, polar.chroma * cosine, polar.chroma * sine]
        }
        None => [polar.lightness, 0.0, 0.0],
    }
}
