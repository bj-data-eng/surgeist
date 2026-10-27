/// A `<resolution>` unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssResolutionUnit {
    /// `dpi`: dots per inch.
    Dpi,
    /// `dpcm`: dots per centimeter.
    Dpcm,
    /// `dppx`: dots per `px` unit; the canonical resolution unit.
    Dppx,
}

/// A finite `<ratio>` whose numerator is not negative and whose denominator is positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssRatio {
    numerator: CssFiniteNumber,
    denominator: CssFiniteNumber,
}

impl CssRatio {
    /// Checks that both terms are finite, the numerator is not negative and the denominator is greater than zero.
    ///
    /// Returns `None` otherwise.
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

    /// Returns the numerator.
    #[must_use]
    pub const fn numerator(self) -> CssFiniteNumber {
        self.numerator
    }

    /// Returns the denominator, which is greater than zero.
    #[must_use]
    pub const fn denominator(self) -> CssFiniteNumber {
        self.denominator
    }
}

/// A finite `f32` number, rejecting NaN and the infinities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFiniteNumber {
    value: f32,
}

impl CssFiniteNumber {
    /// Returns `None` when `value` is NaN or infinite.
    #[must_use]
    pub const fn try_new(value: f32) -> Option<Self> {
        if value.is_finite() {
            Some(Self { value })
        } else {
            None
        }
    }

    /// Returns the finite value.
    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }
}

/// A `<length>` unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssLengthUnit {
    /// `px`: pixels, 1/96th of `in`.
    Px,
    /// `em`: the font size of the element.
    Em,
    /// `rem`: the font size of the root element.
    Rem,
    /// `ex`: the x-height of the element's font.
    Ex,
    /// `rex`: the `ex` unit of the root element.
    Rex,
    /// `cap`: the cap height of the element's font.
    Cap,
    /// `rcap`: the `cap` unit of the root element.
    Rcap,
    /// `ch`: the advance measure of the `0` glyph in the element's font.
    Ch,
    /// `rch`: the `ch` unit of the root element.
    Rch,
    /// `ic`: the advance measure of the CJK water ideograph in the element's font.
    Ic,
    /// `ric`: the `ic` unit of the root element.
    Ric,
    /// `lh`: the computed line height of the element.
    Lh,
    /// `rlh`: the `lh` unit of the root element.
    Rlh,
    /// `vw`: 1% of the viewport width.
    Vw,
    /// `vh`: 1% of the viewport height.
    Vh,
    /// `vi`: 1% of the viewport size in the box's inline axis.
    Vi,
    /// `vb`: 1% of the viewport size in the box's block axis.
    Vb,
    /// `vmin`: 1% of the smaller viewport dimension.
    Vmin,
    /// `vmax`: 1% of the larger viewport dimension.
    Vmax,
    /// `svw`: 1% of the small viewport width.
    Svw,
    /// `svh`: 1% of the small viewport height.
    Svh,
    /// `svi`: 1% of the small viewport size in the box's inline axis.
    Svi,
    /// `svb`: 1% of the small viewport size in the box's block axis.
    Svb,
    /// `svmin`: 1% of the smaller small viewport dimension.
    Svmin,
    /// `svmax`: 1% of the larger small viewport dimension.
    Svmax,
    /// `lvw`: 1% of the large viewport width.
    Lvw,
    /// `lvh`: 1% of the large viewport height.
    Lvh,
    /// `lvi`: 1% of the large viewport size in the box's inline axis.
    Lvi,
    /// `lvb`: 1% of the large viewport size in the box's block axis.
    Lvb,
    /// `lvmin`: 1% of the smaller large viewport dimension.
    Lvmin,
    /// `lvmax`: 1% of the larger large viewport dimension.
    Lvmax,
    /// `dvw`: 1% of the dynamic viewport width.
    Dvw,
    /// `dvh`: 1% of the dynamic viewport height.
    Dvh,
    /// `dvi`: 1% of the dynamic viewport size in the box's inline axis.
    Dvi,
    /// `dvb`: 1% of the dynamic viewport size in the box's block axis.
    Dvb,
    /// `dvmin`: 1% of the smaller dynamic viewport dimension.
    Dvmin,
    /// `dvmax`: 1% of the larger dynamic viewport dimension.
    Dvmax,
    /// `cqw`: 1% of a query container's width.
    Cqw,
    /// `cqh`: 1% of a query container's height.
    Cqh,
    /// `cqi`: 1% of a query container's inline size.
    Cqi,
    /// `cqb`: 1% of a query container's block size.
    Cqb,
    /// `cqmin`: the smaller value of `cqi` or `cqb`.
    Cqmin,
    /// `cqmax`: the larger value of `cqi` or `cqb`.
    Cqmax,
    /// `cm`: centimeters.
    Cm,
    /// `mm`: millimeters.
    Mm,
    /// `Q`: quarter-millimeters, 1/40th of `cm`; units are ASCII case-insensitive and `as_str` returns `q`.
    Q,
    /// `in`: inches, 2.54cm or 96px.
    In,
    /// `pc`: picas, 1/6th of `in`.
    Pc,
    /// `pt`: points, 1/72nd of `in`.
    Pt,
}

impl CssLengthUnit {
    pub(crate) fn from_css_unit(unit: &str) -> Option<Self> {
        match crate::numeric::units::parse(unit)? {
            crate::numeric::CssNumericUnit::Length(unit) => Some(unit),
            _ => None,
        }
    }

    /// Returns the canonical lowercase CSS suffix of the unit, such as `px` or `vmin`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        crate::numeric::units::suffix(crate::numeric::CssNumericUnit::Length(self))
    }
}

/// A `<time>` unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTimeUnit {
    /// `s`: seconds.
    Seconds,
    /// `ms`: milliseconds.
    Milliseconds,
}

/// An `<angle>` unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAngleUnit {
    /// `deg`: degrees; a full circle is 360.
    Degrees,
    /// `grad`: gradians; a full circle is 400.
    Gradians,
    /// `rad`: radians; a full circle is 2 pi.
    Radians,
    /// `turn`: turns; a full circle is 1.
    Turns,
}

/// A `<frequency>` unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFrequencyUnit {
    /// `Hz`: hertz; units are ASCII case-insensitive and the canonical suffix is lowercase `hz`.
    Hertz,
    /// `kHz`: kilohertz, 1000 hertz; the canonical suffix is lowercase `khz`.
    Kilohertz,
}
