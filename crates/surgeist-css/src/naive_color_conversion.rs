//! Pure uncalibrated numerical formulas from Color 5 WD 2026-09-08 §6.1.
//! These samples are concrete normalized coordinates, independent of authored CSS.

use std::fmt;

/// Why a concrete numerical sample could not be naively converted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNaiveColorConversionError {
    /// The first nonfinite input, indexed in the function's documented array order.
    NonFiniteInput { component: usize },
    /// A required result overflowed the implemented finite `f64` arithmetic.
    UnrepresentableResult,
}

impl fmt::Display for CssNaiveColorConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteInput { component } => write!(
                formatter,
                "naive color conversion input component {component} is not finite"
            ),
            Self::UnrepresentableResult => formatter
                .write_str("naive color conversion result is not representable as finite f64"),
        }
    }
}

impl std::error::Error for CssNaiveColorConversionError {}

/// Naively converts cyan, magenta, yellow, black, alpha to encoded sRGB red,
/// green, blue, alpha, using Color 5 WD 2026-09-08 §6.1.
///
/// Channels use normalized units, not 0–255 or linear-light coordinates. Every
/// input must be finite; extended coordinates and alpha outside `[0, 1]` are
/// accepted. Alpha is copied bit-for-bit, including negative zero. Each channel
/// evaluates `max(0, (1 - ink) * (1 - black))`, including the formula's intrinsic
/// zero saturation. A positive product overflow returns a typed error.
///
/// This is an uncalibrated numerical fallback, with ordinary `f64` rounding.
/// It neither resolves authored colors nor chooses profiles, performs gamut
/// mapping, clamps computed ink or alpha, or promises calibrated equivalence.
///
/// ```
/// use surgeist_css::naively_convert_cmyk_to_srgba;
/// assert_eq!(
///     naively_convert_cmyk_to_srgba([0.25, 0.5, 0.75, 0.5, 0.375]).unwrap(),
///     [0.375, 0.25, 0.125, 0.375],
/// );
/// ```
pub fn naively_convert_cmyk_to_srgba(
    cmyka: [f64; 5],
) -> Result<[f64; 4], CssNaiveColorConversionError> {
    validate_input(&cmyka)?;
    let [cyan, magenta, yellow, black, alpha] = cmyka;
    let remaining_black = 1.0 - black;
    Ok([
        forward_channel(cyan, remaining_black)?,
        forward_channel(magenta, remaining_black)?,
        forward_channel(yellow, remaining_black)?,
        alpha,
    ])
}

/// Naively converts encoded sRGB red, green, blue, alpha to cyan, magenta,
/// yellow, black, alpha, using Color 5 WD 2026-09-08 §6.1.
///
/// Every input must be finite; normalized coordinates and alpha may extend
/// outside `[0, 1]`. Alpha is copied bit-for-bit. For maximum RGB `v`, black is
/// `1 - v`; inks are zero when `v == 0`, otherwise `1 - channel / v`.
/// A nonfinite result returns a typed error.
///
/// This chooses a canonical naive ink decomposition, losing the original CMYK
/// decomposition. Ordinary `f64` rounding can make black equal one for a tiny
/// nonzero maximum; the ink ratios still use that original maximum. Exact
/// near-black round trips are therefore not promised. No profile resolution,
/// gamut mapping, alpha clamp or authored CSS evaluation is performed.
///
/// ```
/// use surgeist_css::naively_convert_srgba_to_cmyk;
/// assert_eq!(
///     naively_convert_srgba_to_cmyk([2.0, 1.0, 0.0, -0.0]).unwrap(),
///     [0.0, 0.5, 1.0, -1.0, -0.0],
/// );
/// ```
pub fn naively_convert_srgba_to_cmyk(
    rgba: [f64; 4],
) -> Result<[f64; 5], CssNaiveColorConversionError> {
    validate_input(&rgba)?;
    let [red, green, blue, alpha] = rgba;
    let maximum = red.max(green).max(blue);
    let black = finite_result(1.0 - maximum)?;
    let inks = if maximum == 0.0 {
        [0.0; 3]
    } else {
        [
            finite_result(1.0 - red / maximum)?,
            finite_result(1.0 - green / maximum)?,
            finite_result(1.0 - blue / maximum)?,
        ]
    };
    Ok([inks[0], inks[1], inks[2], black, alpha])
}

fn validate_input(input: &[f64]) -> Result<(), CssNaiveColorConversionError> {
    for (component, value) in input.iter().enumerate() {
        if !value.is_finite() {
            return Err(CssNaiveColorConversionError::NonFiniteInput { component });
        }
    }
    Ok(())
}

fn forward_channel(ink: f64, remaining_black: f64) -> Result<f64, CssNaiveColorConversionError> {
    let remaining_ink = 1.0 - ink;
    // Classify the exact formula's saturated side before a product can overflow.
    // Zero factors also give zero for every finite coordinate.
    if remaining_ink == 0.0
        || remaining_black == 0.0
        || remaining_ink.is_sign_negative() != remaining_black.is_sign_negative()
    {
        return Ok(0.0);
    }
    finite_result(remaining_ink * remaining_black)
}

fn finite_result(value: f64) -> Result<f64, CssNaiveColorConversionError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CssNaiveColorConversionError::UnrepresentableResult)
    }
}
