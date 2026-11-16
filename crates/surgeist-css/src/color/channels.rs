//! Fixed intrinsic channel resolution after the canonical numeric grammar.
//!
//! The caller admits the three-channel kernel. No token, input, fact array or
//! variadic argument is observed here; those continuations supply typed values.

use super::keywords::ColorFunction as F;
use crate::color::CssColorSyntax as Syntax;
use surgeist_color::Color;
use surgeist_color::ColorSpace as Space;
use surgeist_color::PredefinedColorSpace as Predefined;
use surgeist_color::{Finite, UnitInterval};

/// The largest magnitude of a used colour component: `f32::MAX`, the largest
/// finite value of the single-precision range that bounds a component. Values 4
/// leaves the range of numeric values implementation-defined and asks for the
/// closest supported value beyond it; Color 4 allows an implementation-defined
/// limit for the unbounded components.
pub(crate) const LARGEST: f64 = f32::MAX as f64;

/// A component whose binary64 value, in the domain it enters the colour (the
/// number, the percentage scaled by its slot's reference, the angle in
/// degrees), has at least this magnitude is infinite: `2^128 - 2^103`, the
/// point at which a binary64 rounds to an infinite binary32. A value below it
/// is finite and keeps its binary64 value. Between [`LARGEST`] and this value a
/// component is finite but used resolution limits it to [`LARGEST`].
pub(crate) const THRESHOLD: f64 = 340_282_356_779_733_661_637_539_395_458_142_568_448.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Domain {
    Number,
    Percentage,
    Degrees,
    Missing,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Component {
    pub(crate) domain: Domain,
    pub(crate) value: f64,
}
impl Component {
    pub(crate) const MISSING: Self = Self {
        domain: Domain::Missing,
        value: 0.0,
    };
    pub(crate) const fn number(value: f64) -> Self {
        Self {
            domain: Domain::Number,
            value,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Grammar,
}

/// CSS's finite numeric rule for one resolved channel: Values 4 §10.9.1
/// censors a top-level NaN to zero and clamps an infinity to the largest
/// value the context allows, and §10.12 makes a value beyond the supported
/// range the closest supported value. A colour component is limited to
/// [`LARGEST`], so an unbounded channel beyond it, infinite or not, becomes
/// `f32::MAX` with its sign.
pub(crate) fn supported(value: f64) -> Finite {
    let limited = if value.is_nan() {
        0.0
    } else {
        value.clamp(-LARGEST, LARGEST)
    };
    Finite::new(limited).expect("a limited value is finite")
}

/// One percentage: the channel value that 1% denotes, as the unreduced ratio
/// that specified serialization multiplies exactly. 100% is the Color 4/5
/// reference value `100 × numerator / denominator`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Percent {
    pub(crate) numerator: u64,
    pub(crate) denominator: u64,
}
impl Percent {
    const fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
    /// The channel value of 100%.
    pub(crate) fn reference(self) -> f64 {
        percentage_reference(self.numerator, self.denominator)
    }
    /// The decimal exponent of a power-of-ten reference value, such as 2 for
    /// Lab lightness (100%% = 100) or 0 for Oklab lightness (100%% = 1).
    pub(crate) fn reference_power_of_ten(self) -> Option<i128> {
        let scaled = self.numerator * 100;
        if !scaled.is_multiple_of(self.denominator) {
            return None;
        }
        let reference = scaled / self.denominator;
        let exponent = reference.ilog10();
        (10u64.pow(exponent) == reference).then_some(i128::from(exponent))
    }
}

/// The parse-time range of an absolute channel. Color 5 relative channels
/// and calculation results are not clamped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Clamp {
    Unclamped,
    /// Negative values clamp to 0 (HSL saturation, LCH/OKLCH chroma).
    NonNegative,
    /// Values clamp to `[0, reference]` (RGB, Lab/Oklab/LCH/OKLCH lightness,
    /// device-cmyk inks).
    Reference,
}

/// One fixed channel slot of a Color 4/5 color function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Slot {
    /// A `<hue>` in degrees, normalized by [`normalize_hue`].
    Hue,
    Scalar {
        percent: Percent,
        clamp: Clamp,
    },
}

/// The Color 4/5 channel declarations, read by absolute and relative
/// resolution and by specified serialization: rgb "100% = 255"; hsl/hwb
/// percentages of 100; lab "100% = 100" lightness and "100% = 125" a/b;
/// lch "100% = 150" chroma; oklab/oklch "100% = 1" lightness and
/// "100% = 0.4" a/b/chroma; color() and device-cmyk "100% = 1".
pub(crate) const fn slot(function: F, index: usize) -> Slot {
    const UNIT: Percent = Percent::new(1, 100);
    const PERCENT: Percent = Percent::new(1, 1);
    const OK_AXIS: Percent = Percent::new(1, 250);
    const fn scalar(percent: Percent, clamp: Clamp) -> Slot {
        Slot::Scalar { percent, clamp }
    }
    match (function, index) {
        (F::Rgb, _) => scalar(Percent::new(255, 100), Clamp::Reference),
        (F::Hsl | F::Hwb, 0) | (F::Lch | F::Oklch, 2) => Slot::Hue,
        (F::Hsl, 1) => scalar(PERCENT, Clamp::NonNegative),
        (F::Hsl | F::Hwb, _) => scalar(PERCENT, Clamp::Unclamped),
        (F::Lab | F::Lch, 0) => scalar(PERCENT, Clamp::Reference),
        (F::Lab, _) => scalar(Percent::new(5, 4), Clamp::Unclamped),
        (F::Lch, _) => scalar(Percent::new(3, 2), Clamp::NonNegative),
        (F::Oklab | F::Oklch, 0) => scalar(UNIT, Clamp::Reference),
        (F::Oklab, _) => scalar(OK_AXIS, Clamp::Unclamped),
        (F::Oklch, _) => scalar(OK_AXIS, Clamp::NonNegative),
        (F::DeviceCmyk, _) => scalar(UNIT, Clamp::Reference),
        _ => scalar(UNIT, Clamp::Unclamped),
    }
}

/// Resolves one channel by its declared slot. Absolute channels apply the
/// declared clamp; relative channels keep every finite extended value. The
/// result is the closest supported finite value ([`supported`]).
pub(crate) fn declared(
    function: F,
    index: usize,
    component: Component,
    absolute: bool,
) -> Result<Option<Finite>, Error> {
    match slot(function, index) {
        Slot::Hue => hue(component),
        Slot::Scalar {
            percent,
            clamp: range,
        } => {
            let value = scalar(component, percent.reference())?;
            Ok(match (absolute, range) {
                (false, _) | (true, Clamp::Unclamped) => value,
                (true, Clamp::NonNegative) => value.map(|value| value.max(0.0)),
                (true, Clamp::Reference) => clamp(value, percent.reference()),
            }
            .map(supported))
        }
    }
}

pub(crate) fn legacy_rgb_domains(domains: [Domain; 3]) -> bool {
    matches!(domains[0], Domain::Number | Domain::Percentage)
        && domains[1] == domains[0]
        && domains[2] == domains[0]
}
/// The channel value of 100%, which is the slot's `numerator * 100 / denominator`.
pub(crate) fn percentage_reference(numerator: u64, denominator: u64) -> f64 {
    (numerator * 100) as f64 / denominator as f64
}
/// The channel value of a percentage numeral: `value / 100` times the
/// reference value of 100%. Used resolution (`scalar`, [`Entry`]) and the
/// specified classification of a numeral share this one binary64 evaluation.
pub(crate) fn percentage_of_reference(value: f64, reference: f64) -> f64 {
    value / 100.0 * reference
}
pub(crate) fn scalar(component: Component, percentage: f64) -> Result<Option<f64>, Error> {
    if component.domain == Domain::Missing {
        return Ok(None);
    }
    if component.domain == Domain::Degrees {
        return Err(Error::Grammar);
    }
    // CSS Values calculation NaN is censored to zero at the calculation root.
    let value = if component.value.is_nan() {
        0.0
    } else {
        component.value
    };
    Ok(Some(if component.domain == Domain::Percentage {
        percentage_of_reference(value, percentage)
    } else {
        value
    }))
}
/// Color 4 §4.3 normalizes every hue in degrees to `[0, 360)`; the
/// `hsl(360 0 0)` and `hsl(calc(±infinity) 0 0)` examples give 0 degrees.
/// This is the one CSS normalization for used and specified hues. A hue of at
/// least [`THRESHOLD`] degrees is infinite and is 0. Zero, including -0, is
/// the unsigned zero, and a negative hue whose remainder rounds up to the
/// excluded 360 in binary64 also normalizes to zero.
pub(crate) fn normalize_hue(degrees: f64) -> f64 {
    if !degrees.is_finite() || degrees.abs() >= THRESHOLD {
        return 0.0;
    }
    let hue = degrees.rem_euclid(360.0);
    if hue == 0.0 || hue >= 360.0 { 0.0 } else { hue }
}
pub(crate) fn hue(component: Component) -> Result<Option<Finite>, Error> {
    match component.domain {
        Domain::Missing => Ok(None),
        Domain::Percentage => Err(Error::Grammar),
        Domain::Degrees | Domain::Number => Ok(Some(supported(normalize_hue(component.value)))),
    }
}
fn clamp(value: Option<f64>, maximum: f64) -> Option<f64> {
    value.map(|value| {
        if value.is_nan() {
            0.0
        } else {
            value.clamp(0.0, maximum)
        }
    })
}
pub(crate) fn alpha(value: Option<Component>, legacy: bool) -> Result<Option<UnitInterval>, Error> {
    if legacy && value.is_some_and(|value| value.domain == Domain::Missing) {
        return Err(Error::Grammar);
    }
    Ok(value
        .map_or(Ok(Some(1.0)), |value| scalar(value, 1.0))?
        .map(|alpha| UnitInterval::clamped(supported(alpha))))
}

/// The numerical space of one three-channel color function.
pub(crate) fn function_space(function: F, predefined: Option<Predefined>) -> Result<Space, Error> {
    Ok(match function {
        F::Rgb => Space::Predefined(Predefined::Srgb),
        F::Hsl => Space::Hsl,
        F::Hwb => Space::Hwb,
        F::Lab => Space::Lab,
        F::Oklab => Space::Oklab,
        F::Lch => Space::Lch,
        F::Oklch => Space::Oklch,
        F::Color => Space::Predefined(predefined.ok_or(Error::Grammar)?),
        _ => return Err(Error::Grammar),
    })
}

/// The binary64 value a relative component has in the colour it builds, from
/// the authored number or percentage of its slot. The component range rule
/// classifies this value, so used resolution ([`relative_channels`]) and
/// specified serialization read the one scale.
///
/// Every slot holds its component in the unit its percentage reference names
/// (a number is that unit, `p%` is `p / 100` times the reference) except an
/// `rgb()` channel, which the colour holds as the unit coordinate: the number
/// divided by 255 and the percentage scaled to 255 and divided by 255.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Entry {
    reference: f64,
    divisor: f64,
}
impl Entry {
    /// The entry of a slot whose component the colour holds in the unit of its
    /// percentage reference.
    pub(crate) fn percent(numerator: u64, denominator: u64) -> Self {
        Self {
            reference: percentage_reference(numerator, denominator),
            divisor: 1.0,
        }
    }
    /// The entry of one slot of a colour function. A hue slot has no
    /// percentage and holds its degrees.
    pub(crate) fn of(function: F, index: usize) -> Self {
        match slot(function, index) {
            Slot::Hue => Self::percent(1, 1),
            Slot::Scalar { percent, .. } => Self {
                reference: percent.reference(),
                divisor: if function == F::Rgb {
                    percent.reference()
                } else {
                    1.0
                },
            },
        }
    }
    /// The component an authored number has in the colour.
    pub(crate) fn number(self, value: f64) -> f64 {
        value / self.divisor
    }
    /// The component an authored percentage has in the colour.
    pub(crate) fn percentage(self, value: f64) -> f64 {
        percentage_of_reference(value, self.reference) / self.divisor
    }
    /// The component of one resolved channel. A missing channel has none, and
    /// a calculation NaN is censored to zero at the calculation root.
    pub(crate) fn component(self, component: Component) -> Result<Option<f64>, Error> {
        if component.domain == Domain::Missing {
            return Ok(None);
        }
        if component.domain == Domain::Degrees {
            return Err(Error::Grammar);
        }
        let value = if component.value.is_nan() {
            0.0
        } else {
            component.value
        };
        Ok(Some(if component.domain == Domain::Percentage {
            self.percentage(value)
        } else {
            self.number(value)
        }))
    }
}

/// Resolves relative function channels by their declared slots, without
/// the absolute parse-time clamps (Color 5: relative components "are *not
/// clamped*"). Each scalar channel is its [`Entry`] component limited to the
/// supported range.
pub(crate) fn relative_channels(
    function: F,
    components: [Component; 3],
) -> Result<[Option<Finite>; 3], Error> {
    let channel = |index: usize| -> Result<Option<Finite>, Error> {
        match slot(function, index) {
            Slot::Hue => hue(components[index]),
            Slot::Scalar { .. } => Ok(Entry::of(function, index)
                .component(components[index])?
                .map(supported)),
        }
    };
    Ok([channel(0)?, channel(1)?, channel(2)?])
}

/// Function-space coordinates remain available to same-space relative syntax
/// before the final Color4 used-value conversion.
pub(crate) fn absolute_coordinates(
    function: F,
    predefined: Option<Predefined>,
    syntax: Syntax,
    components: [Component; 3],
    alpha_component: Option<Component>,
) -> Result<Color, Error> {
    let legacy = syntax == Syntax::Legacy;
    if legacy && !matches!(function, F::Rgb | F::Hsl) {
        return Err(Error::Grammar);
    }
    if legacy
        && components
            .iter()
            .any(|value| value.domain == Domain::Missing)
    {
        return Err(Error::Grammar);
    }
    let alpha = alpha(alpha_component, legacy)?;
    let space = function_space(function, predefined)?;
    let channels = match function {
        F::Rgb => {
            if legacy && !legacy_rgb_domains(components.map(|value| value.domain)) {
                return Err(Error::Grammar);
            }
            let Slot::Scalar { percent, .. } = slot(F::Rgb, 0) else {
                unreachable!("rgb channels are scalar")
            };
            // Absolute rgb() channels clamp to [0, 255] and are stored in
            // sRGB unit coordinates; a percentage needs no 255 round trip.
            let channel = |value: Component| -> Result<Option<Finite>, Error> {
                if value.domain == Domain::Missing {
                    return Ok(None);
                }
                if value.domain == Domain::Degrees {
                    return Err(Error::Grammar);
                }
                let value_number = if value.value.is_nan() {
                    0.0
                } else {
                    value.value
                };
                Ok(Some(supported(
                    (value_number
                        / if value.domain == Domain::Percentage {
                            100.0
                        } else {
                            percent.reference()
                        })
                    .clamp(0.0, 1.0),
                )))
            };
            [
                channel(components[0])?,
                channel(components[1])?,
                channel(components[2])?,
            ]
        }
        _ => {
            if legacy
                && (components[1].domain != Domain::Percentage
                    || components[2].domain != Domain::Percentage)
            {
                return Err(Error::Grammar);
            }
            [
                declared(function, 0, components[0], true)?,
                declared(function, 1, components[1], true)?,
                declared(function, 2, components[2], true)?,
            ]
        }
    };
    let value = Color::new(space, channels, alpha);
    // Color4 16.2.2 retains the absolute rgb()/rgba() serialization family.
    // Missing channels still select color(srgb ...) in the canonical serializer;
    // relative RGB is a separate Color5 color-function family. The value is
    // already sRGB, so selecting the legacy family converts nothing.
    Ok(if function == F::Rgb {
        value.to_legacy_srgb()
    } else {
        value
    })
}
