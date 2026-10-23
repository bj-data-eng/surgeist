//! Pure uncalibrated numerical formulas from Color 5 WD 2026-09-08 §6.1.
//! These samples are concrete normalized coordinates, independent of authored CSS.

use crate::finite::{Finite, closest_finite};

/// Naively converts cyan, magenta, yellow, black, alpha to encoded sRGB red,
/// green, blue, alpha, using Color 5 WD 2026-09-08 §6.1.
///
/// Channels use normalized units, not 0–255 or linear-light coordinates.
/// Extended coordinates and alpha outside `[0, 1]` are accepted. Alpha is
/// copied bit-for-bit, including negative zero. Each channel evaluates
/// `max(0, (1 - ink) * (1 - black))`, including the formula's intrinsic zero
/// saturation; a positive product beyond binary64 is the closest finite value,
/// `f64::MAX`.
///
/// This is an uncalibrated numerical fallback, with ordinary `f64` rounding.
/// It neither resolves authored colors nor chooses profiles, performs gamut
/// mapping, clamps computed ink or alpha, or promises calibrated equivalence.
///
/// ```
/// use surgeist_color::{Finite, naively_convert_cmyk_to_srgba};
/// let cmyka = [0.25, 0.5, 0.75, 0.5, 0.375].map(|value| Finite::new(value).unwrap());
/// assert_eq!(
///     naively_convert_cmyk_to_srgba(cmyka).map(Finite::get),
///     [0.375, 0.25, 0.125, 0.375],
/// );
/// ```
#[must_use]
pub fn naively_convert_cmyk_to_srgba(cmyka: [Finite; 5]) -> [Finite; 4] {
    let [cyan, magenta, yellow, black, alpha] = cmyka;
    let remaining_black = 1.0 - black.get();
    [
        forward_channel(cyan.get(), remaining_black),
        forward_channel(magenta.get(), remaining_black),
        forward_channel(yellow.get(), remaining_black),
        alpha,
    ]
}

/// Naively converts encoded sRGB red, green, blue, alpha to cyan, magenta,
/// yellow, black, alpha, using Color 5 WD 2026-09-08 §6.1.
///
/// Normalized coordinates and alpha may extend outside `[0, 1]`. Alpha is
/// copied bit-for-bit. For maximum RGB `v`, black is `1 - v`; inks are zero
/// when `v == 0`, otherwise `1 - channel / v`. A result beyond binary64 is
/// the signed closest finite value.
///
/// This chooses a canonical naive ink decomposition, losing the original CMYK
/// decomposition. Ordinary `f64` rounding can make black equal one for a tiny
/// nonzero maximum; the ink ratios still use that original maximum. Exact
/// near-black round trips are therefore not promised. No profile resolution,
/// gamut mapping, alpha clamp or authored CSS evaluation is performed.
///
/// ```
/// use surgeist_color::{Finite, naively_convert_srgba_to_cmyk};
/// let rgba = [2.0, 1.0, 0.0, -0.0].map(|value| Finite::new(value).unwrap());
/// assert_eq!(
///     naively_convert_srgba_to_cmyk(rgba).map(Finite::get),
///     [0.0, 0.5, 1.0, -1.0, -0.0],
/// );
/// ```
#[must_use]
pub fn naively_convert_srgba_to_cmyk(rgba: [Finite; 4]) -> [Finite; 5] {
    let [red, green, blue, alpha] = rgba;
    let maximum = red.get().max(green.get()).max(blue.get());
    let black = closest_finite(1.0 - maximum);
    let ink = |channel: Finite| {
        if maximum == 0.0 {
            closest_finite(0.0)
        } else {
            closest_finite(1.0 - channel.get() / maximum)
        }
    };
    [ink(red), ink(green), ink(blue), black, alpha]
}

fn forward_channel(ink: f64, remaining_black: f64) -> Finite {
    let remaining_ink = 1.0 - ink;
    // Classify the exact formula's saturated side before a product can overflow.
    // Zero factors also give zero for every finite coordinate.
    if remaining_ink == 0.0
        || remaining_black == 0.0
        || remaining_ink.is_sign_negative() != remaining_black.is_sign_negative()
    {
        return closest_finite(0.0);
    }
    closest_finite(remaining_ink * remaining_black)
}
