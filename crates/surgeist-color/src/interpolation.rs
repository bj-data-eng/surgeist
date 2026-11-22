//! Two-color conversion, missing-component carry and interpolation kernels.
//!
//! No authored channel domains, weights, recipe records or context facts cross
//! this boundary. The caller admits the fixed operands before invoking it.

use crate::Color;
use crate::finite::{Finite, UnitInterval, closest_finite};
use crate::wide::{Wide, power_of_two};
use crate::{ColorSpace as Space, HueInterpolation as Hue, InterpolationMethod as Method};
use core::fmt;

/// A numerical interpolation result in the selected interpolation space.
///
/// Present coordinates and alpha are finite. Alpha can extend beyond the
/// unit interval. Absence records a missing component.
#[derive(Clone, Copy, PartialEq)]
pub struct InterpolationSample {
    space: Space,
    channels: [Finite; 3],
    alpha: Finite,
    // Three channel bits followed by the alpha bit. Missing components use
    // one canonical placeholder and need no separate stored discriminant.
    present: u8,
}

impl InterpolationSample {
    pub const fn space(self) -> Space {
        self.space
    }

    pub const fn channels(self) -> [Option<Finite>; 3] {
        [self.channel(0), self.channel(1), self.channel(2)]
    }

    pub const fn alpha(self) -> Option<Finite> {
        if self.present & (1 << 3) != 0 {
            Some(self.alpha)
        } else {
            None
        }
    }

    const fn channel(self, index: usize) -> Option<Finite> {
        if self.present & (1 << index) != 0 {
            Some(self.channels[index])
        } else {
            None
        }
    }
}

impl fmt::Debug for InterpolationSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterpolationSample")
            .field("space", &self.space)
            .field("channels", &self.channels())
            .field("alpha", &self.alpha())
            .finish()
    }
}

/// The component whose final value lies outside the finite binary64 range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InterpolationComponent {
    /// Zero-based channel index in the selected interpolation space.
    Channel(usize),
    Alpha,
}

/// Interpolation could not represent a final numerical component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterpolationSampleError {
    component: InterpolationComponent,
}

impl InterpolationSampleError {
    pub const fn component(self) -> InterpolationComponent {
        self.component
    }
}

impl fmt::Display for InterpolationSampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.component {
            InterpolationComponent::Channel(index) => {
                write!(
                    f,
                    "interpolated color channel {index} is outside the finite range"
                )
            }
            InterpolationComponent::Alpha => {
                f.write_str("interpolated alpha is outside the finite range")
            }
        }
    }
}

impl std::error::Error for InterpolationSampleError {}

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
fn powerless(value: Color) -> Color {
    let space = value.space();
    let mut channels = value.finite_channels();
    let Some(index) = hue_index(space) else {
        return value;
    };
    let plain = |channel: Option<Finite>| channel.map_or(0.0, Finite::get);
    let (component, epsilon, powerless) = match space {
        Space::Hsl => (1, 0.001, plain(channels[1]) <= 0.001),
        Space::Lch => (1, 0.0015, plain(channels[1]) <= 0.0015),
        Space::Oklch => (1, 0.000004, plain(channels[1]) <= 0.000004),
        Space::Hwb => (1, 99.999, plain(channels[1]) + plain(channels[2]) >= 99.999),
        _ => return value,
    };
    if !powerless {
        return value;
    }
    channels[index] = None;
    if space == Space::Hwb {
        let sum = plain(channels[1]) + plain(channels[2]);
        if sum >= epsilon && sum < 100.0 {
            match (channels[1], channels[2]) {
                (Some(w), Some(_)) => channels[2] = Some(closest_finite(100.0 - w.get())),
                (Some(_), None) => channels[1] = Some(closest_finite(100.0)),
                (None, Some(_)) => channels[2] = Some(closest_finite(100.0)),
                (None, None) => {}
            }
        }
    } else if let Some(c) = channels[component]
        && c.get() > 0.0
        && c.get() <= epsilon
    {
        channels[component] = Some(closest_finite(0.0));
    }
    // Rebuilding resets byte backing and the family, as Color::new specifies.
    Color::new(space, channels, value.unit_alpha())
}

pub(crate) fn prepare(value: Color, destination: Space) -> Color {
    if value.space() == destination {
        return value;
    }
    let carry = carried(value, destination);
    let converted = powerless(powerless(value).converted(destination));
    let mut channels = converted.finite_channels();
    for i in 0..3 {
        if carry[i] {
            channels[i] = None;
        }
    }
    Color::new(destination, channels, value.unit_alpha())
}

fn hue_turn(first: f64, second: f64, normalization: f64, method: Hue) -> f64 {
    let (difference, low) = Wide::from_f64(second).add_with_error(Wide::from_f64(-first));
    let (shifted, shift_error) = difference.add_with_error(Wide::from_f64(normalization));
    let (difference, correction) = shifted.add_with_error(low.add(shift_error));
    let difference = difference.to_f64();
    let half_turn = difference.abs().total_cmp(&180.0).then_with(|| {
        let correction = if difference.is_sign_negative() {
            correction.negated()
        } else {
            correction
        };
        if correction.is_zero() {
            core::cmp::Ordering::Equal
        } else {
            correction.to_f64().total_cmp(&0.0)
        }
    });
    match method {
        Hue::Shorter if half_turn.is_gt() => 360.0_f64.copysign(-difference),
        Hue::Longer if half_turn.is_lt() && difference > 0.0 => -360.0,
        Hue::Longer if half_turn.is_lt() => 360.0,
        Hue::Increasing if difference < 0.0 => 360.0,
        Hue::Decreasing if difference > 0.0 => -360.0,
        _ => 0.0,
    }
}

/// The unit-interval operation uses the closest-finite result policy.
pub(crate) fn interpolate(
    first: Color,
    second: Color,
    method: Method,
    progress: UnitInterval,
) -> Color {
    let (channels, alpha) = interpolate_components(first, second, method, progress.get());
    Color::new(
        method.space(),
        channels.map(|channel| channel.map(Wide::to_finite)),
        alpha.map(|alpha| UnitInterval::clamped(alpha.to_finite())),
    )
}

pub(crate) fn sample(
    first: Color,
    second: Color,
    method: Method,
    progress: Finite,
) -> Result<InterpolationSample, InterpolationSampleError> {
    let (wide_channels, wide_alpha) = interpolate_components(first, second, method, progress.get());
    let checked = |value: Wide, component| {
        Finite::new(value.to_f64()).ok_or(InterpolationSampleError { component })
    };
    let alpha = wide_alpha
        .map(|alpha| checked(alpha, InterpolationComponent::Alpha))
        .transpose()?;
    let mut channels = [Finite::ZERO; 3];
    let mut present = if alpha.is_some() { 1 << 3 } else { 0 };
    for (index, (channel, wide)) in channels.iter_mut().zip(wide_channels).enumerate() {
        if let Some(wide) = wide {
            *channel = checked(wide, InterpolationComponent::Channel(index))?;
            present |= 1 << index;
        }
    }
    Ok(InterpolationSample {
        space: method.space(),
        channels,
        alpha: alpha.unwrap_or(Finite::ZERO),
        present,
    })
}

/// Color 4 §13. Interpolate with the premultiplied coordinates and alpha
/// still widened so division sees their sign and range before narrowing.
fn interpolate_components(
    first: Color,
    second: Color,
    method: Method,
    progress: f64,
) -> ([Option<Wide>; 3], Option<Wide>) {
    let first = prepare(first, method.space());
    let second = prepare(second, method.space());
    let aa = first.alpha().or(second.alpha());
    let ba = second.alpha().or(first.alpha());
    let in_unit_interval = (0.0..=1.0).contains(&progress);
    let t = Wide::from_f64(progress);
    let complement = Wide::from_f64(1.0 - progress);
    let interpolate = |first: Wide, second: Wide| {
        if in_unit_interval {
            first.multiply(complement).add(second.multiply(t))
        } else {
            // The affine form keeps the offset at large progress. Retain
            // the low word of the slope, which can determine a near-zero
            // result after the rounded slope cancels the first endpoint.
            affine(first, second, t)
        }
    };
    let alphas = aa
        .zip(ba)
        .map(|(a, b)| (Wide::from_f64(a), Wide::from_f64(b)));
    let alpha = alphas.map(|(a, b)| interpolate(a, b));

    let a = first.channels();
    let b = second.channels();
    let hue = hue_index(method.space()).zip(method.hue());
    let channels = core::array::from_fn(|index| {
        let x = a[index].or(b[index])?;
        let y = b[index].or(a[index])?;
        if let Some((hue_index, strategy)) = hue
            && index == hue_index
        {
            let start = x % 360.0;
            let end = y % 360.0;
            let start_wrap = if start < 0.0 { 360.0 } else { 0.0 };
            let end_wrap = if end < 0.0 { 360.0 } else { 0.0 };
            let normalization = end_wrap - start_wrap;
            let turn = hue_turn(start, end, normalization, strategy);
            let angle = if in_unit_interval {
                let start_offset = start_wrap - turn.min(0.0);
                let end_offset = end_wrap + turn.max(0.0);
                let start = if start_offset != 0.0 {
                    start + start_offset
                } else {
                    start
                };
                let end = if end_offset != 0.0 {
                    end + end_offset
                } else {
                    end
                };
                start * (1.0 - progress) + end * progress
            } else {
                // Evaluate each term before reducing the whole angle. An
                // endpoint can be too small to survive subtraction or a
                // full-turn adjustment and still affect the final angle.
                start
                    + (hue_product_remainder(end, progress)
                        - hue_product_remainder(start, progress))
                    + hue_product_remainder(normalization + turn, progress)
            };
            return Some(Wide::from_f64(angle.rem_euclid(360.0)));
        }
        if x.to_bits() == y.to_bits() && alpha.is_none_or(|alpha| !alpha.is_zero()) {
            // A common coordinate factors out of the premultiplied sum.
            return Some(Wide::from_f64(x));
        }
        let x = Wide::from_f64(x);
        let y = Wide::from_f64(y);
        let Some(((a, b), alpha)) = alphas.zip(alpha) else {
            return Some(interpolate(x, y));
        };
        let numerator = if in_unit_interval {
            interpolate(x.multiply(a), y.multiply(b))
        } else {
            // Retain the product residuals so endpoint premultiplication
            // cannot erase a lane's slope before extrapolation.
            let (first, first_low) = x.multiply_with_error(a);
            let (second, second_low) = y.multiply_with_error(b);
            interpolate(first, second).add(interpolate(first_low, second_low))
        };
        if alpha.is_zero() {
            return Some(numerator);
        }
        let result = numerator.divide(alpha);
        if !result.exceeds_finite_range() {
            return Some(result);
        }

        // At the edge of the finite range, rounding the numerator before
        // division can make a representable quotient appear to overflow.
        // Re-evaluate using the more heavily weighted coordinate as anchor.
        let first_weight = a.add_product(a.negated(), t);
        let second_weight = b.multiply(t);
        Some(if first_weight.magnitude_order(second_weight).is_lt() {
            affine(y, x, first_weight.divide(alpha))
        } else {
            affine(x, y, second_weight.divide(alpha))
        })
    });
    (channels, alpha)
}

fn affine(first: Wide, second: Wide, progress: Wide) -> Wide {
    let (slope, correction) = second.add_with_error(first.negated());
    first
        .add_product(slope, progress)
        .add_product(correction, progress)
}

/// Reduce a hue product before rounding or adding the starting angle. Two
/// binary64 significands have at most 106 bits in their exact product.
fn hue_product_remainder(delta: f64, progress: f64) -> f64 {
    let (a, a_exponent) = Wide::from_f64(delta).unsigned_parts();
    let (b, b_exponent) = Wide::from_f64(progress).unsigned_parts();
    let product = u128::from(a) * u128::from(b);
    let exponent = a_exponent + b_exponent;
    let remainder = if exponent < -105 {
        // The product of the significands is below 2^106, so no wrap.
        Wide::from_f64(delta)
            .multiply(Wide::from_f64(progress))
            .to_f64()
            .abs()
    } else if exponent < 0 {
        let modulus = 360_u128 << -exponent;
        (product % modulus) as f64 * power_of_two(exponent)
    } else {
        // (2^exponent mod 360), using small bounded factors.
        let (mut power, mut factor, mut remainder) = (exponent, 2, product % 360);
        while power != 0 {
            if power % 2 != 0 {
                remainder = (remainder * factor) % 360;
            }
            factor = (factor * factor) % 360;
            power /= 2;
        }
        remainder as f64
    };
    if delta.is_sign_negative() == progress.is_sign_negative() {
        remainder
    } else {
        -remainder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PredefinedColorSpace as Predefined;
    const SRGB: Space = Space::Predefined(Predefined::Srgb);
    fn color(space: Space, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
        Color::new(
            space,
            channels.map(|channel| channel.map(|value| Finite::new(value).unwrap())),
            alpha.map(|value| UnitInterval::new(value).unwrap()),
        )
    }
    fn method(space: Space, hue: Option<Hue>) -> Method {
        Method::try_new(space, hue).unwrap()
    }
    fn progress(value: f64) -> UnitInterval {
        UnitInterval::new(value).unwrap()
    }
    fn near(value: f64, expected: f64) {
        assert!((value - expected).abs() < 1e-12, "{value} != {expected}");
    }
    #[test]
    fn numerical_methods_resolve_polar_omission_and_reject_rectangular_hue() {
        assert_eq!(
            Method::try_new(SRGB, Some(Hue::Shorter)),
            Err(crate::InterpolationError::HueRequiresPolarSpace)
        );
        assert_eq!(
            Method::try_new(Space::Oklch, None),
            Method::try_new(Space::Oklch, Some(Hue::Shorter))
        );
        assert_eq!(Method::try_new(SRGB, None).unwrap().hue(), None);
    }
    #[test]
    fn missing_components_follow_individual_and_set_analogues() {
        let lab = color(Space::Lab, [Some(50.0), None, None], Some(0.5));
        assert_eq!(
            prepare(lab, Space::Lch).channels(),
            [Some(50.0), None, None]
        );
        let absent_rgb = color(SRGB, [None, None, None], None);
        assert_eq!(
            prepare(absent_rgb, Space::Oklab).channels(),
            [None, None, None]
        );
        let absent_hue = color(Space::Lch, [Some(50.0), Some(0.02), None], Some(0.5));
        assert_eq!(prepare(absent_hue, Space::Oklch).channels()[2], None);
        let same = color(Space::Hwb, [Some(123.0), None, Some(50.0)], None);
        assert_eq!(prepare(same, Space::Hwb), same);
        let converted = prepare(
            color(SRGB, [None, Some(0.2), Some(0.3)], Some(1.0)),
            Space::Predefined(Predefined::XyzD65),
        );
        assert_eq!(converted.channels()[0], None);
        assert!(converted.channels()[1].is_some());
    }
    #[test]
    fn alpha_carry_and_premultiplication_use_the_source_order() {
        let first = color(SRGB, [Some(0.24), Some(0.12), Some(0.98)], Some(0.4));
        let second = color(SRGB, [Some(0.62), Some(0.26), Some(0.64)], Some(0.6));
        let mixed = interpolate(first, second, method(SRGB, None), progress(0.5));
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
        let mixed = interpolate(a, b, method(Space::Oklch, None), progress(0.5));
        near(mixed.channels()[0].unwrap(), 0.5875);
        near(mixed.channels()[1].unwrap(), 0.254);
        assert_eq!(mixed.channels()[2], Some(326.5));
        assert_eq!(mixed.alpha(), Some(0.5));
        let empty = color(SRGB, [None, None, None], None);
        assert_eq!(
            interpolate(empty, empty, method(SRGB, None), progress(0.5)),
            empty
        );
    }
    #[test]
    fn an_overflowing_operand_conversion_mixes_its_closest_finite_value() {
        // sRGB red at f64::MAX has XYZ beyond binary64 in every row (Color 4
        // §10.2 linear light is its 2.4th power), so it prepares as f64::MAX
        // and an even mix with black halves it exactly.
        let red = color(SRGB, [Some(f64::MAX), Some(0.0), Some(0.0)], Some(1.0));
        let black = color(SRGB, [Some(0.0), Some(0.0), Some(0.0)], Some(1.0));
        let xyz = method(Space::Predefined(Predefined::XyzD65), None);
        let mixed = interpolate(red, black, xyz, progress(0.5));
        assert_eq!(mixed.channels(), [Some(f64::MAX / 2.0); 3]);
        assert_eq!(mixed.alpha(), Some(1.0));
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
            let value = interpolate(a, b, method(Space::Oklch, Some(hue)), progress(0.5));
            near(value.channels()[2].unwrap(), expected);
        }
        let zero = color(Space::Lch, [Some(50.0), Some(0.0), Some(123.0)], Some(1.0));
        assert_eq!(prepare(zero, Space::Lch).channels()[2], Some(123.0));
        assert_eq!(prepare(zero, Space::Oklch).channels()[2], None);
        let tiny = powerless(color(
            Space::Oklch,
            [Some(0.5), Some(0.000004), Some(42.0)],
            Some(1.0),
        ));
        assert_eq!(tiny.channels(), [Some(0.5), Some(0.0), None]);
        let hwb = powerless(color(
            Space::Hwb,
            [Some(42.0), Some(50.0), Some(49.999)],
            Some(1.0),
        ));
        assert_eq!(hwb.channels(), [None, Some(50.0), Some(50.0)]);
    }
}
