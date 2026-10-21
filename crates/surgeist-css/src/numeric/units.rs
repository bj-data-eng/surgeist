//! The one Values 4 unit registry.
//!
//! Values 4 WD 2024-03-12 §6 (lengths) and §7 (angle, time, frequency and
//! resolution) define each unit by its ASCII case-insensitive suffix, its
//! dimension and its fixed relation to the dimension's canonical unit. Each
//! unit is declared here exactly once; decoding, canonical suffixes, exact
//! rational ratios and their binary64 projections all derive from that entry.

use super::{CssNumericDimension, CssNumericUnit};
use crate::exact_decimal::ExactFactor;
use crate::{CssAngleUnit, CssFrequencyUnit, CssLengthUnit, CssResolutionUnit, CssTimeUnit};

/// How one unit relates to its dimension's canonical unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UnitRatio {
    /// Values 4 fixes this exact rational number of canonical units.
    Exact(ExactFactor),
    /// One radian is 180/π degrees, which no rational can represent. Binary64
    /// arithmetic uses `180.0 / PI`; exact comparison has no rational ratio.
    Radian,
    /// Font-relative, viewport, container and flex units need a context.
    Contextual,
}

const fn exact(numerator: u64, denominator: u64) -> UnitRatio {
    UnitRatio::Exact(ExactFactor {
        numerator,
        denominator,
    })
}

macro_rules! unit_registry {
    ($( $dimension:ident :: $enum:ident :: $variant:ident => $suffix:literal $(| $alias:literal)*, $ratio:expr; )*) => {
        /// Decodes an authored unit, ASCII case-insensitively.
        pub(crate) fn parse(unit: &str) -> Option<CssNumericUnit> {
            Some(cssparser::match_ignore_ascii_case! { unit,
                $( $suffix $(| $alias)* => CssNumericUnit::$dimension($enum::$variant), )*
                "fr" => CssNumericUnit::Flex,
                _ => return None,
            })
        }
        /// The canonical lowercase suffix that serializes this unit.
        pub(crate) const fn suffix(unit: CssNumericUnit) -> &'static str {
            match unit {
                $( CssNumericUnit::$dimension($enum::$variant) => $suffix, )*
                CssNumericUnit::Flex => "fr",
            }
        }
        /// The unit's relation to its dimension's canonical unit.
        pub(crate) const fn ratio(unit: CssNumericUnit) -> UnitRatio {
            match unit {
                $( CssNumericUnit::$dimension($enum::$variant) => $ratio, )*
                CssNumericUnit::Flex => UnitRatio::Contextual,
            }
        }
        #[cfg(test)]
        const ALL: &[CssNumericUnit] = &[
            $( CssNumericUnit::$dimension($enum::$variant), )*
            CssNumericUnit::Flex,
        ];
    };
}

unit_registry! {
    // §6.2 absolute lengths: 1in = 2.54cm = 96px, 1cm = 10mm = 40Q,
    // 1in = 6pc = 72pt. Canonical unit: px.
    Length::CssLengthUnit::Px => "px", exact(1, 1);
    Length::CssLengthUnit::Cm => "cm", exact(4800, 127);
    Length::CssLengthUnit::Mm => "mm", exact(480, 127);
    Length::CssLengthUnit::Q => "q", exact(120, 127);
    Length::CssLengthUnit::In => "in", exact(96, 1);
    Length::CssLengthUnit::Pc => "pc", exact(16, 1);
    Length::CssLengthUnit::Pt => "pt", exact(4, 3);
    // §6.1 relative lengths.
    Length::CssLengthUnit::Em => "em", UnitRatio::Contextual;
    Length::CssLengthUnit::Rem => "rem", UnitRatio::Contextual;
    Length::CssLengthUnit::Ex => "ex", UnitRatio::Contextual;
    Length::CssLengthUnit::Rex => "rex", UnitRatio::Contextual;
    Length::CssLengthUnit::Cap => "cap", UnitRatio::Contextual;
    Length::CssLengthUnit::Rcap => "rcap", UnitRatio::Contextual;
    Length::CssLengthUnit::Ch => "ch", UnitRatio::Contextual;
    Length::CssLengthUnit::Rch => "rch", UnitRatio::Contextual;
    Length::CssLengthUnit::Ic => "ic", UnitRatio::Contextual;
    Length::CssLengthUnit::Ric => "ric", UnitRatio::Contextual;
    Length::CssLengthUnit::Lh => "lh", UnitRatio::Contextual;
    Length::CssLengthUnit::Rlh => "rlh", UnitRatio::Contextual;
    Length::CssLengthUnit::Vw => "vw", UnitRatio::Contextual;
    Length::CssLengthUnit::Vh => "vh", UnitRatio::Contextual;
    Length::CssLengthUnit::Vi => "vi", UnitRatio::Contextual;
    Length::CssLengthUnit::Vb => "vb", UnitRatio::Contextual;
    Length::CssLengthUnit::Vmin => "vmin", UnitRatio::Contextual;
    Length::CssLengthUnit::Vmax => "vmax", UnitRatio::Contextual;
    Length::CssLengthUnit::Svw => "svw", UnitRatio::Contextual;
    Length::CssLengthUnit::Svh => "svh", UnitRatio::Contextual;
    Length::CssLengthUnit::Svi => "svi", UnitRatio::Contextual;
    Length::CssLengthUnit::Svb => "svb", UnitRatio::Contextual;
    Length::CssLengthUnit::Svmin => "svmin", UnitRatio::Contextual;
    Length::CssLengthUnit::Svmax => "svmax", UnitRatio::Contextual;
    Length::CssLengthUnit::Lvw => "lvw", UnitRatio::Contextual;
    Length::CssLengthUnit::Lvh => "lvh", UnitRatio::Contextual;
    Length::CssLengthUnit::Lvi => "lvi", UnitRatio::Contextual;
    Length::CssLengthUnit::Lvb => "lvb", UnitRatio::Contextual;
    Length::CssLengthUnit::Lvmin => "lvmin", UnitRatio::Contextual;
    Length::CssLengthUnit::Lvmax => "lvmax", UnitRatio::Contextual;
    Length::CssLengthUnit::Dvw => "dvw", UnitRatio::Contextual;
    Length::CssLengthUnit::Dvh => "dvh", UnitRatio::Contextual;
    Length::CssLengthUnit::Dvi => "dvi", UnitRatio::Contextual;
    Length::CssLengthUnit::Dvb => "dvb", UnitRatio::Contextual;
    Length::CssLengthUnit::Dvmin => "dvmin", UnitRatio::Contextual;
    Length::CssLengthUnit::Dvmax => "dvmax", UnitRatio::Contextual;
    Length::CssLengthUnit::Cqw => "cqw", UnitRatio::Contextual;
    Length::CssLengthUnit::Cqh => "cqh", UnitRatio::Contextual;
    Length::CssLengthUnit::Cqi => "cqi", UnitRatio::Contextual;
    Length::CssLengthUnit::Cqb => "cqb", UnitRatio::Contextual;
    Length::CssLengthUnit::Cqmin => "cqmin", UnitRatio::Contextual;
    Length::CssLengthUnit::Cqmax => "cqmax", UnitRatio::Contextual;
    // §7.1 angles: 1turn = 360deg = 400grad; canonical unit: deg.
    Angle::CssAngleUnit::Degrees => "deg", exact(1, 1);
    Angle::CssAngleUnit::Gradians => "grad", exact(9, 10);
    Angle::CssAngleUnit::Radians => "rad", UnitRatio::Radian;
    Angle::CssAngleUnit::Turns => "turn", exact(360, 1);
    // §7.2 durations; canonical unit: s.
    Time::CssTimeUnit::Seconds => "s", exact(1, 1);
    Time::CssTimeUnit::Milliseconds => "ms", exact(1, 1000);
    // §7.3 frequencies; canonical unit: hz.
    Frequency::CssFrequencyUnit::Hertz => "hz", exact(1, 1);
    Frequency::CssFrequencyUnit::Kilohertz => "khz", exact(1000, 1);
    // §7.4 resolutions: 1dppx = 96dpi, 1in = 2.54cm; canonical unit: dppx.
    Resolution::CssResolutionUnit::Dpi => "dpi", exact(1, 96);
    Resolution::CssResolutionUnit::Dpcm => "dpcm", exact(127, 4800);
    Resolution::CssResolutionUnit::Dppx => "dppx" | "x", exact(1, 1);
}

/// The canonical unit of a dimension with intrinsic conversions.
pub(crate) const fn canonical(dimension: CssNumericDimension) -> Option<CssNumericUnit> {
    Some(match dimension {
        CssNumericDimension::Length => CssNumericUnit::Length(CssLengthUnit::Px),
        CssNumericDimension::Angle => CssNumericUnit::Angle(CssAngleUnit::Degrees),
        CssNumericDimension::Time => CssNumericUnit::Time(CssTimeUnit::Seconds),
        CssNumericDimension::Frequency => CssNumericUnit::Frequency(CssFrequencyUnit::Hertz),
        CssNumericDimension::Resolution => CssNumericUnit::Resolution(CssResolutionUnit::Dppx),
        CssNumericDimension::Flex | CssNumericDimension::Percentage => return None,
    })
}

/// The canonical suffix of a dimension with intrinsic conversions.
pub(crate) const fn canonical_suffix(dimension: CssNumericDimension) -> Option<&'static str> {
    match canonical(dimension) {
        Some(unit) => Some(suffix(unit)),
        None => None,
    }
}

/// The exact rational ratio to the canonical unit, when Values 4 defines one.
pub(crate) const fn exact_factor(unit: CssNumericUnit) -> Option<ExactFactor> {
    match ratio(unit) {
        UnitRatio::Exact(factor) => Some(factor),
        UnitRatio::Radian | UnitRatio::Contextual => None,
    }
}

/// The exact binary64 value of `180.0 / PI`, the radian's binary64 projection.
/// Exact rational serialization of a radian uses this value so that it agrees
/// with binary64 arithmetic; Values 4 itself defines no rational ratio.
pub(crate) const RADIAN_BINARY64: ExactFactor = ExactFactor {
    numerator: 1_007_958_012_753_983,
    denominator: 17_592_186_044_416,
};

/// A unit's binary64 projection onto its dimension's canonical unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Binary64Projection {
    canonical: &'static str,
    ratio: UnitRatio,
}

impl Binary64Projection {
    /// The canonical unit's suffix.
    pub(crate) const fn canonical(self) -> &'static str {
        self.canonical
    }

    /// Projects a binary64 coefficient onto the canonical unit.
    ///
    /// An exact ratio multiplies by its numerator and then divides by its
    /// denominator. Whenever the coefficient and that product are exact, the
    /// result is the correctly rounded exact canonical value, so equal absolute
    /// values authored in different units project to identical binary64 values.
    /// A product beyond binary64 range uses the rounded quotient instead, so a
    /// finite canonical value never overflows only through the intermediate.
    pub(crate) fn apply(self, coefficient: f64) -> f64 {
        match self.ratio {
            UnitRatio::Exact(ExactFactor {
                numerator,
                denominator,
            }) => {
                let product = coefficient * numerator as f64;
                if product.is_infinite() && coefficient.is_finite() {
                    coefficient * (numerator as f64 / denominator as f64)
                } else {
                    product / denominator as f64
                }
            }
            UnitRatio::Radian => coefficient * (180.0 / std::f64::consts::PI),
            UnitRatio::Contextual => unreachable!("contextual units have no projection"),
        }
    }

    /// The rounded canonical value of one unit, for magnitude bounds.
    pub(crate) fn unit_value(self) -> f64 {
        match self.ratio {
            UnitRatio::Exact(ExactFactor {
                numerator,
                denominator,
            }) => numerator as f64 / denominator as f64,
            UnitRatio::Radian => 180.0 / std::f64::consts::PI,
            UnitRatio::Contextual => unreachable!("contextual units have no projection"),
        }
    }
}

/// The binary64 projection of an intrinsically convertible unit.
pub(crate) fn binary64_projection(unit: CssNumericUnit) -> Option<Binary64Projection> {
    let canonical = canonical_suffix(unit.dimension())?;
    match ratio(unit) {
        UnitRatio::Contextual => None,
        ratio => Some(Binary64Projection { canonical, ratio }),
    }
}

/// The decimal exponent shift for a power-of-ten ratio such as `ms`.
pub(crate) const fn decimal_shift(unit: CssNumericUnit) -> Option<i32> {
    let UnitRatio::Exact(ExactFactor {
        numerator,
        denominator,
    }) = ratio(unit)
    else {
        return None;
    };
    let (mut value, sign) = match (numerator, denominator) {
        (1, denominator) => (denominator, -1),
        (numerator, 1) => (numerator, 1),
        _ => return None,
    };
    let mut shift = 0;
    while value % 10 == 0 {
        value /= 10;
        shift += 1;
    }
    if value == 1 { Some(shift * sign) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_suffix_and_alias_decodes_to_its_unit_case_insensitively() {
        for &unit in ALL {
            let spelling = suffix(unit);
            assert_eq!(parse(spelling), Some(unit), "{spelling}");
            assert_eq!(parse(&spelling.to_ascii_uppercase()), Some(unit));
        }
        assert_eq!(
            parse("X"),
            Some(CssNumericUnit::Resolution(CssResolutionUnit::Dppx))
        );
        assert_eq!(parse("pixels"), None);
    }

    #[test]
    fn values_4_absolute_length_relations_hold_exactly_in_the_rational_ratios() {
        let px = |unit| {
            let factor = exact_factor(CssNumericUnit::Length(unit)).unwrap();
            (factor.numerator, factor.denominator)
        };
        // a/b == c/d  <=>  a*d == b*c, scaled by the authored multiples.
        let equal = |(a, b): (u64, u64), m: u64, (c, d): (u64, u64), n: u64| a * m * d == c * n * b;
        use CssLengthUnit::*;
        assert!(equal(px(In), 1, (96, 1), 1));
        assert!(equal(px(Cm), 254, px(In), 100));
        assert!(equal(px(Cm), 1, px(Mm), 10));
        assert!(equal(px(Cm), 1, px(Q), 40));
        assert!(equal(px(In), 1, px(Pc), 6));
        assert!(equal(px(In), 1, px(Pt), 72));
    }

    #[test]
    fn binary64_projection_is_the_correctly_rounded_exact_canonical_value() {
        use CssLengthUnit::*;
        let px = |coefficient: f64, unit| {
            binary64_projection(CssNumericUnit::Length(unit))
                .unwrap()
                .apply(coefficient)
        };
        // 1cm = 4800/127px, 10mm = 4800/127px, 40Q = 4800/127px.
        assert_eq!(px(1.0, Cm), 4800.0 / 127.0);
        assert_eq!(px(10.0, Mm), 4800.0 / 127.0);
        assert_eq!(px(40.0, Q), 4800.0 / 127.0);
        assert_eq!(px(3.0, Cm), px(30.0, Mm));
        assert_eq!(px(1.0, In), 96.0);
        assert_eq!(px(72.0, Pt), 96.0);
        assert_eq!(px(6.0, Pc), 96.0);
        // An overflowing intermediate product keeps a finite canonical value.
        assert_eq!(px(1e306, Cm), 1e306 * (4800.0 / 127.0));
        assert!(px(1e306, Cm).is_finite());
        assert_eq!(binary64_projection(CssNumericUnit::Length(Em)), None);
        assert_eq!(binary64_projection(CssNumericUnit::Flex), None);
        let radian = binary64_projection(CssNumericUnit::Angle(CssAngleUnit::Radians)).unwrap();
        assert_eq!(radian.canonical(), "deg");
        assert_eq!(radian.apply(1.0), 180.0 / std::f64::consts::PI);
    }

    #[test]
    fn the_radian_rational_is_the_exact_binary64_projection() {
        assert!(RADIAN_BINARY64.denominator.is_power_of_two());
        assert_eq!(
            RADIAN_BINARY64.numerator as f64 / RADIAN_BINARY64.denominator as f64,
            180.0 / std::f64::consts::PI
        );
        // The numerator fits binary64's 53-bit significand, so the quotient is exact.
        const { assert!(RADIAN_BINARY64.numerator < 1 << 53) };
    }

    #[test]
    fn power_of_ten_ratios_have_decimal_shifts() {
        assert_eq!(
            decimal_shift(CssNumericUnit::Time(CssTimeUnit::Milliseconds)),
            Some(-3)
        );
        assert_eq!(
            decimal_shift(CssNumericUnit::Time(CssTimeUnit::Seconds)),
            Some(0)
        );
        assert_eq!(
            decimal_shift(CssNumericUnit::Frequency(CssFrequencyUnit::Kilohertz)),
            Some(3)
        );
        assert_eq!(
            decimal_shift(CssNumericUnit::Length(CssLengthUnit::Cm)),
            None
        );
    }
}
