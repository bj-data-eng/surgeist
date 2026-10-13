//! Fixed Color4 interpolation and one-item Values5 normalization kernels.
//!
//! Each caller-admitted action observes at most two fixed colors or one weight.
//! No recipe, argument array, context fact or original source is scanned here.

use super::{CssUsedColor, CssUsedColorError};
use crate::{
    CssColorInterpolationMethod as Method, CssColorInterpolationSpace as Space,
    CssHueInterpolationMethod as Hue,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    InvalidWeight,
    Empty,
    Overflow,
    Numerical(CssUsedColorError),
}
impl From<CssUsedColorError> for Error {
    fn from(value: CssUsedColorError) -> Self {
        Self::Numerical(value)
    }
}

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
fn carried(value: CssUsedColor, destination: Space) -> [bool; 3] {
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
fn powerless(value: CssUsedColor) -> Result<CssUsedColor, Error> {
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
    Ok(CssUsedColor::try_new(space, channels, value.alpha())?)
}

pub(crate) fn prepare(value: CssUsedColor, destination: Space) -> Result<CssUsedColor, Error> {
    if value.space() == destination {
        return Ok(value);
    }
    let carry = carried(value, destination);
    let converted = powerless(powerless(value)?.convert(destination)?)?;
    let mut channels = converted.channels();
    for i in 0..3 {
        if carry[i] {
            channels[i] = None;
        }
    }
    Ok(CssUsedColor::try_new(destination, channels, value.alpha())?)
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
    first: CssUsedColor,
    second: CssUsedColor,
    method: Method,
    progress: f64,
) -> Result<CssUsedColor, Error> {
    if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
        return Err(Error::InvalidWeight);
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
        let (x, y) = hue_pair(x, y, method.effective_hue().expect("polar strategy"));
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
    Ok(CssUsedColor::try_new(method.space(), channels, alpha)?)
}

/// One admitted weight observation in source order; no argument storage.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WeightSummary {
    length: usize,
    omitted: usize,
    specified: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Weights {
    pub(crate) omitted: f64,
    pub(crate) scale: f64,
    pub(crate) alpha_multiplier: f64,
}
impl WeightSummary {
    pub(crate) fn identical(self, other: Self) -> bool {
        self.length == other.length
            && self.omitted == other.omitted
            && self.specified.to_bits() == other.specified.to_bits()
    }

    pub(crate) fn observe(&mut self, percentage: Option<f64>) -> Result<(), Error> {
        let length = self.length.checked_add(1).ok_or(Error::Overflow)?;
        let mut omitted = self.omitted;
        let mut specified = self.specified;
        if let Some(p) = percentage {
            if !p.is_finite() || !(0.0..=100.0).contains(&p) {
                return Err(Error::InvalidWeight);
            }
            specified += p;
            if !specified.is_finite() {
                return Err(Error::Numerical(CssUsedColorError::Unrepresentable));
            }
        } else {
            omitted = omitted.checked_add(1).ok_or(Error::Overflow)?;
        }
        *self = Self {
            length,
            omitted,
            specified,
        };
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<Weights, Error> {
        if self.length == 0 {
            return Err(Error::Empty);
        }
        let remaining = 100.0 - self.specified.min(100.0);
        let omitted = if self.omitted == 0 {
            0.0
        } else {
            remaining / self.omitted as f64
        };
        let total = if self.omitted == 0 {
            self.specified
        } else {
            self.specified + remaining
        };
        Ok(Weights {
            omitted,
            scale: if total > 0.0 { 1.0 / total } else { 0.0 },
            alpha_multiplier: (total / 100.0).min(1.0),
        })
    }
}
impl Weights {
    pub(crate) fn weight(self, percentage: Option<f64>) -> Result<f64, Error> {
        let percentage = percentage.unwrap_or(self.omitted);
        if !percentage.is_finite() || !(0.0..=100.0).contains(&percentage) {
            return Err(Error::InvalidWeight);
        }
        Ok(percentage * self.scale)
    }
}

/// Color5§3.3 ordered fold. One color per admitted action, not one variadic loop.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MixFold {
    method: Method,
    value: Option<CssUsedColor>,
    weight: f64,
}
impl MixFold {
    pub(crate) fn identical(self, other: Self) -> bool {
        self.method == other.method
            && self.weight.to_bits() == other.weight.to_bits()
            && match (self.value, other.value) {
                (Some(a), Some(b)) => a.identical(b),
                (None, None) => true,
                _ => false,
            }
    }

    pub(crate) const fn new(method: Method) -> Self {
        Self {
            method,
            value: None,
            weight: 0.0,
        }
    }
    pub(crate) const fn has_value(&self) -> bool {
        self.value.is_some()
    }
    pub(crate) fn observe(&mut self, value: CssUsedColor, weight: f64) -> Result<(), Error> {
        if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
            return Err(Error::InvalidWeight);
        }
        let total = self.weight + weight;
        if !total.is_finite() {
            return Err(Error::Numerical(CssUsedColorError::Unrepresentable));
        }
        let value = if let Some(first) = self.value {
            interpolate(
                first,
                value,
                self.method,
                if total > 0.0 { weight / total } else { 0.5 },
            )?
        } else {
            prepare(value, self.method.space())?
        };
        self.value = Some(value);
        self.weight = total;
        Ok(())
    }
    pub(crate) fn finish(self, weights: Weights) -> Result<CssUsedColor, Error> {
        let value = self.value.ok_or(Error::Empty)?;
        // Color5 mixtures retain their modern serialization family; missing
        // HSL/HWB components still serialize in the natural cylindrical form.
        Ok(CssUsedColor::try_new(
            value.space(),
            value.channels(),
            value.alpha().map(|alpha| alpha * weights.alpha_multiplier),
        )?
        .with_color_function())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssPredefinedColorSpace as Predefined;
    const SRGB: Space = Space::Predefined(Predefined::Srgb);
    fn color(space: Space, channels: [Option<f64>; 3], alpha: Option<f64>) -> CssUsedColor {
        CssUsedColor::try_new(space, channels, alpha).unwrap()
    }
    fn method(space: Space, hue: Option<Hue>) -> Method {
        Method::try_new(space, hue).unwrap()
    }
    fn near(value: f64, expected: f64) {
        assert!((value - expected).abs() < 1e-12, "{value} != {expected}");
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
    #[test]
    fn normalization_is_one_item_at_a_time_and_preserves_leftover() {
        for (input, expected, multiplier) in [
            ([Some(80.0), Some(80.0)], [0.5, 0.5], 1.0),
            ([Some(30.0), Some(30.0)], [0.5, 0.5], 0.6),
            ([Some(80.0), None], [0.8, 0.2], 1.0),
            ([None, None], [0.5, 0.5], 1.0),
            ([Some(0.0), Some(0.0)], [0.0, 0.0], 0.0),
        ] {
            let mut summary = WeightSummary::default();
            for weight in input {
                summary.observe(weight).unwrap();
            }
            let weights = summary.finish().unwrap();
            assert_eq!(weights.alpha_multiplier, multiplier);
            for (input, expected) in input.into_iter().zip(expected) {
                near(weights.weight(input).unwrap(), expected);
            }
        }
        let mut summary = WeightSummary::default();
        summary.observe(Some(20.0)).unwrap();
        summary.observe(None).unwrap();
        summary.observe(None).unwrap();
        let weights = summary.finish().unwrap();
        assert_eq!(weights.omitted, 40.0);
        assert_eq!(summary.observe(Some(-1.0)), Err(Error::InvalidWeight));
        assert_eq!(summary.length, 3);
        assert_eq!(WeightSummary::default().finish(), Err(Error::Empty));
    }
    #[test]
    fn sequential_mix_and_zero_weights_do_not_guess_a_default_color() {
        let weights = {
            let mut s = WeightSummary::default();
            s.observe(Some(30.0)).unwrap();
            s.observe(Some(30.0)).unwrap();
            s.finish().unwrap()
        };
        let mut fold = MixFold::new(method(SRGB, None));
        fold.observe(
            CssUsedColor::from_srgba8([255, 0, 0, 255]),
            weights.weight(Some(30.0)).unwrap(),
        )
        .unwrap();
        fold.observe(
            CssUsedColor::from_srgba8([0, 0, 255, 255]),
            weights.weight(Some(30.0)).unwrap(),
        )
        .unwrap();
        let value = fold.finish(weights).unwrap();
        assert_eq!(value.channels(), [Some(0.5), Some(0.0), Some(0.5)]);
        assert_eq!(value.alpha(), Some(0.6));
        let mut s = WeightSummary::default();
        s.observe(Some(0.0)).unwrap();
        s.observe(Some(0.0)).unwrap();
        let zero = s.finish().unwrap();
        let mut fold = MixFold::new(method(SRGB, None));
        fold.observe(CssUsedColor::opaque_black(), 0.0).unwrap();
        fold.observe(CssUsedColor::from_srgba8([255; 4]), 0.0)
            .unwrap();
        let value = fold.finish(zero).unwrap();
        assert_eq!(value.channels(), [Some(0.5); 3]);
        assert_eq!(value.alpha(), Some(0.0));
    }
}
