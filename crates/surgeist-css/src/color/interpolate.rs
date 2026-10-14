//! Fixed Color4 interpolation and one-item Values5 normalization kernels.
//!
//! Each caller-admitted action observes at most two fixed colors or one weight.
//! No recipe, argument array, context fact or original source is scanned here.

use super::numerical::interpolation::{Method as NumericalMethod, interpolate, prepare};
use super::{CssUsedColor, CssUsedColorError};
use crate::CssColorInterpolationMethod as Method;

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

impl From<super::numerical::interpolation::Error> for Error {
    fn from(value: super::numerical::interpolation::Error) -> Self {
        match value {
            super::numerical::interpolation::Error::InvalidProgress => Self::InvalidWeight,
            super::numerical::interpolation::Error::Numerical(error) => Self::Numerical(error),
        }
    }
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
                NumericalMethod::try_new(self.method.space(), self.method.effective_hue())
                    .expect("checked authored interpolation space and effective hue"),
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
    use crate::{
        CssColorInterpolationSpace as Space, CssHueInterpolationMethod as Hue,
        CssPredefinedColorSpace as Predefined,
    };
    const SRGB: Space = Space::Predefined(Predefined::Srgb);
    fn method(space: Space, hue: Option<Hue>) -> Method {
        Method::try_new(space, hue).unwrap()
    }
    fn near(value: f64, expected: f64) {
        assert!((value - expected).abs() < 1e-12, "{value} != {expected}");
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
