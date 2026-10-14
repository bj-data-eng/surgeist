//! Two-color conversion, missing-component carry and interpolation kernels.
//!
//! No authored channel domains, weights, recipe records or context facts cross
//! this boundary. The caller admits the fixed operands before invoking it.

use crate::{Color, ColorError};
use crate::{ColorSpace as Space, HueInterpolation as Hue};
use crate::{InterpolationError as Error, InterpolationMethod as Method};

// Categories are fixed Color4§13.3 identities, not authored channel names.
// HWB whiteness/blackness have no cross-space analogues.
fn categories(space: Space) -> [u8; 3] {
    match space {
        Space::Predefined(_) => [0, 1, 2],
        Space::Lab | Space::Oklab => [3, 6, 7],
        Space::Lch | Space::Oklch => [3, 4, 5],
        Space::Hsl => [5, 4, 3],
        Space::Hwb => [5, 8, 9],
    }
}
fn hue_index(space: Space) -> Option<usize> {
    match space {
        Space::Hsl | Space::Hwb => Some(0),
        Space::Lch | Space::Oklch => Some(2),
        _ => None,
    }
}
fn carried(value: Color, destination: Space) -> [bool; 3] {
    let source = categories(value.space());
    let target = categories(destination);
    let missing = value.channels().map(|value| value.is_none());
    let mut result = [false; 3];
    let mut unmatched_all_missing = true;
    let mut unmatched = false;
    for i in 0..3 {
        if let Some(j) = target.iter().position(|category| *category == source[i]) {
            result[j] = missing[i];
        } else {
            unmatched = true;
            unmatched_all_missing &= missing[i];
        }
    }
    if unmatched && unmatched_all_missing {
        for j in 0..3 {
            if !source.contains(&target[j]) {
                result[j] = true;
            }
        }
    }
    result
}

// Color4§4.4.1: apply only in the conversion phase. Missing hue is not
// replaced with zero. Tiny positive colorfulness is zeroed when powerless;
// negative extended values are not gamut-clamped here.
fn powerless(value: Color) -> Result<Color, ColorError> {
    let space = value.space();
    let mut channels = value.channels();
    let Some(index) = hue_index(space) else {
        return Ok(value);
    };
    let (component, epsilon, powerless) = match space {
        Space::Hsl => (1, 0.001, channels[1].unwrap_or(0.0) <= 0.001),
        Space::Lch => (1, 0.0015, channels[1].unwrap_or(0.0) <= 0.0015),
        Space::Oklch => (1, 0.000004, channels[1].unwrap_or(0.0) <= 0.000004),
        Space::Hwb => (
            1,
            99.999,
            channels[1].unwrap_or(0.0) + channels[2].unwrap_or(0.0) >= 99.999,
        ),
        _ => unreachable!(),
    };
    if !powerless {
        return Ok(value);
    }
    channels[index] = None;
    if space == Space::Hwb {
        let sum = channels[1].unwrap_or(0.0) + channels[2].unwrap_or(0.0);
        if sum >= epsilon && sum < 100.0 {
            match (channels[1], channels[2]) {
                (Some(w), Some(_)) => channels[2] = Some(100.0 - w),
                (Some(_), None) => channels[1] = Some(100.0),
                (None, Some(_)) => channels[2] = Some(100.0),
                (None, None) => {}
            }
        }
    } else if let Some(c) = channels[component]
        && c > 0.0
        && c <= epsilon
    {
        channels[component] = Some(0.0);
    }
    Color::try_new(space, channels, value.alpha())
}

pub(crate) fn prepare(value: Color, destination: Space) -> Result<Color, ColorError> {
    if value.space() == destination {
        return Ok(value);
    }
    let carry = carried(value, destination);
    let converted = powerless(powerless(value)?.converted(destination)?)?;
    let mut channels = converted.channels();
    for i in 0..3 {
        if carry[i] {
            channels[i] = None;
        }
    }
    Color::try_new(destination, channels, value.alpha())
}

fn hue_pair(mut first: f64, mut second: f64, method: Hue) -> (f64, f64) {
    first = first.rem_euclid(360.0);
    second = second.rem_euclid(360.0);
    let difference = second - first;
    match method {
        Hue::Shorter if difference > 180.0 => first += 360.0,
        Hue::Shorter if difference < -180.0 => second += 360.0,
        Hue::Longer if difference > 0.0 && difference < 180.0 => first += 360.0,
        Hue::Longer if difference > -180.0 && difference <= 0.0 => second += 360.0,
        Hue::Increasing if second < first => second += 360.0,
        Hue::Decreasing if first < second => first += 360.0,
        _ => {}
    }
    (first, second)
}

pub(crate) fn interpolate(
    first: Color,
    second: Color,
    method: Method,
    progress: f64,
) -> Result<Color, Error> {
    if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
        return Err(Error::InvalidProgress);
    }
    let first = prepare(first, method.space())?;
    let second = prepare(second, method.space())?;
    let mut a = first.channels();
    let mut b = second.channels();
    let aa = first.alpha().or(second.alpha());
    let ba = second.alpha().or(first.alpha());
    for i in 0..3 {
        let av = a[i];
        a[i] = av.or(b[i]);
        b[i] = b[i].or(av);
    }
    let hue = hue_index(method.space());
    if let Some(index) = hue
        && let (Some(x), Some(y)) = (a[index], b[index])
    {
        let (x, y) = hue_pair(x, y, method.hue().expect("polar strategy"));
        a[index] = Some(x);
        b[index] = Some(y);
    }
    let alpha = aa.zip(ba).map(|(a, b)| a * (1.0 - progress) + b * progress);
    let mut channels = [None; 3];
    for i in 0..3 {
        if let (Some(x), Some(y)) = (a[i], b[i]) {
            let (x, y) = if hue == Some(i) {
                (x, y)
            } else {
                (x * aa.unwrap_or(1.0), y * ba.unwrap_or(1.0))
            };
            let mut value = x * (1.0 - progress) + y * progress;
            if hue != Some(i)
                && let Some(alpha) = alpha
                && alpha != 0.0
            {
                value /= alpha;
            }
            if hue == Some(i) {
                value = value.rem_euclid(360.0);
            }
            channels[i] = Some(value);
        }
    }
    Ok(Color::try_new(method.space(), channels, alpha)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PredefinedColorSpace as Predefined;
    const SRGB: Space = Space::Predefined(Predefined::Srgb);
    fn color(space: Space, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
        Color::try_new(space, channels, alpha).unwrap()
    }
    fn method(space: Space, hue: Option<Hue>) -> Method {
        Method::try_new(space, hue).unwrap()
    }
    fn near(value: f64, expected: f64) {
        assert!((value - expected).abs() < 1e-12, "{value} != {expected}");
    }
    #[test]
    fn numerical_methods_resolve_polar_omission_and_reject_rectangular_hue() {
        assert_eq!(
            Method::try_new(SRGB, Some(Hue::Shorter)),
            Err(Error::HueRequiresPolarSpace)
        );
        assert_eq!(
            Method::try_new(Space::Oklch, None),
            Method::try_new(Space::Oklch, Some(Hue::Shorter))
        );
        assert_eq!(Method::try_new(SRGB, None).unwrap().hue(), None);
        for progress in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            assert_eq!(
                interpolate(
                    Color::opaque_black(),
                    Color::opaque_black(),
                    method(SRGB, None),
                    progress
                ),
                Err(Error::InvalidProgress)
            );
        }
    }
    #[test]
    fn missing_components_follow_individual_and_set_analogues() {
        let lab = color(Space::Lab, [Some(50.0), None, None], Some(0.5));
        assert_eq!(
            prepare(lab, Space::Lch).unwrap().channels(),
            [Some(50.0), None, None]
        );
        let absent_rgb = color(SRGB, [None, None, None], None);
        assert_eq!(
            prepare(absent_rgb, Space::Oklab).unwrap().channels(),
            [None, None, None]
        );
        let absent_hue = color(Space::Lch, [Some(50.0), Some(0.02), None], Some(0.5));
        assert_eq!(
            prepare(absent_hue, Space::Oklch).unwrap().channels()[2],
            None
        );
        let same = color(Space::Hwb, [Some(123.0), None, Some(50.0)], None);
        assert_eq!(prepare(same, Space::Hwb).unwrap(), same);
        let converted = prepare(
            color(SRGB, [None, Some(0.2), Some(0.3)], Some(1.0)),
            Space::Predefined(Predefined::XyzD65),
        )
        .unwrap();
        assert_eq!(converted.channels()[0], None);
        assert!(converted.channels()[1].is_some());
    }
    #[test]
    fn alpha_carry_and_premultiplication_use_the_source_order() {
        let first = color(SRGB, [Some(0.24), Some(0.12), Some(0.98)], Some(0.4));
        let second = color(SRGB, [Some(0.62), Some(0.26), Some(0.64)], Some(0.6));
        let mixed = interpolate(first, second, method(SRGB, None), 0.5).unwrap();
        for (actual, expected) in mixed.channels().into_iter().zip([0.468, 0.204, 0.776]) {
            near(actual.unwrap(), expected);
        }
        near(mixed.alpha().unwrap(), 0.5);
        let a = color(
            Space::Oklch,
            [Some(0.783), Some(0.108), Some(326.5)],
            Some(0.5),
        );
        let b = color(Space::Oklch, [Some(0.392), Some(0.4), None], None);
        let mixed = interpolate(a, b, method(Space::Oklch, None), 0.5).unwrap();
        near(mixed.channels()[0].unwrap(), 0.5875);
        near(mixed.channels()[1].unwrap(), 0.254);
        assert_eq!(mixed.channels()[2], Some(326.5));
        assert_eq!(mixed.alpha(), Some(0.5));
        let empty = color(SRGB, [None, None, None], None);
        assert_eq!(
            interpolate(empty, empty, method(SRGB, None), 0.5).unwrap(),
            empty
        );
    }
    #[test]
    fn polar_strategy_and_conversion_powerlessness_are_separate() {
        let a = color(Space::Oklch, [Some(0.6), Some(0.24), Some(30.0)], Some(1.0));
        let b = color(Space::Oklch, [Some(0.8), Some(0.15), Some(90.0)], Some(1.0));
        for (hue, expected) in [
            (Hue::Shorter, 60.0),
            (Hue::Longer, 240.0),
            (Hue::Increasing, 60.0),
            (Hue::Decreasing, 240.0),
        ] {
            let value = interpolate(a, b, method(Space::Oklch, Some(hue)), 0.5).unwrap();
            near(value.channels()[2].unwrap(), expected);
        }
        let zero = color(Space::Lch, [Some(50.0), Some(0.0), Some(123.0)], Some(1.0));
        assert_eq!(
            prepare(zero, Space::Lch).unwrap().channels()[2],
            Some(123.0)
        );
        assert_eq!(prepare(zero, Space::Oklch).unwrap().channels()[2], None);
        let tiny = powerless(color(
            Space::Oklch,
            [Some(0.5), Some(0.000004), Some(42.0)],
            Some(1.0),
        ))
        .unwrap();
        assert_eq!(tiny.channels(), [Some(0.5), Some(0.0), None]);
        let hwb = powerless(color(
            Space::Hwb,
            [Some(42.0), Some(50.0), Some(49.999)],
            Some(1.0),
        ))
        .unwrap();
        assert_eq!(hwb.channels(), [None, Some(50.0), Some(50.0)]);
    }
}
