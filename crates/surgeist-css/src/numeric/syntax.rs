#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResolutionUnit {
    Dpi,
    Dpcm,
    Dppx,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssRatio {
    numerator: CssFiniteNumber,
    denominator: CssFiniteNumber,
}

impl CssRatio {
    #[must_use]
    pub fn try_new(numerator: f32, denominator: f32) -> Option<Self> {
        if numerator >= 0.0 && denominator > 0.0 {
            Some(Self {
                numerator: CssFiniteNumber::try_new(numerator)?,
                denominator: CssFiniteNumber::try_new(denominator)?,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn numerator(self) -> CssFiniteNumber {
        self.numerator
    }

    #[must_use]
    pub const fn denominator(self) -> CssFiniteNumber {
        self.denominator
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFiniteNumber {
    value: f32,
}

impl CssFiniteNumber {
    #[must_use]
    pub const fn try_new(value: f32) -> Option<Self> {
        if value.is_finite() {
            Some(Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLengthUnit {
    Px,
    Em,
    Rem,
    Ex,
    Rex,
    Cap,
    Rcap,
    Ch,
    Rch,
    Ic,
    Ric,
    Lh,
    Rlh,
    Vw,
    Vh,
    Vi,
    Vb,
    Vmin,
    Vmax,
    Svw,
    Svh,
    Svi,
    Svb,
    Svmin,
    Svmax,
    Lvw,
    Lvh,
    Lvi,
    Lvb,
    Lvmin,
    Lvmax,
    Dvw,
    Dvh,
    Dvi,
    Dvb,
    Dvmin,
    Dvmax,
    Cqw,
    Cqh,
    Cqi,
    Cqb,
    Cqmin,
    Cqmax,
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
}

impl CssLengthUnit {
    pub(crate) fn from_css_unit(unit: &str) -> Option<Self> {
        match crate::numeric::units::parse(unit)? {
            crate::numeric::CssNumericUnit::Length(unit) => Some(unit),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        crate::numeric::units::suffix(crate::numeric::CssNumericUnit::Length(self))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTimeUnit {
    Seconds,
    Milliseconds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAngleUnit {
    Degrees,
    Gradians,
    Radians,
    Turns,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFrequencyUnit {
    Hertz,
    Kilohertz,
}
