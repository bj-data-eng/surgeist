//! Effective finite bounds and points for HTML Progress and Meter.

use super::HtmlFloatingPointNumber;

/// The effective positive maximum of an HTML Progress element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlProgressMaximum(HtmlFloatingPointNumber);

impl HtmlProgressMaximum {
    /// Uses the recovered positive maximum, or one when absent, invalid, or nonpositive.
    pub fn new(maximum: Option<HtmlFloatingPointNumber>) -> Self {
        Self(
            maximum
                .filter(|number| number.value > 0.0)
                .unwrap_or(HtmlFloatingPointNumber::ONE),
        )
    }

    /// Returns the effective positive maximum independently of value presence.
    pub fn maximum(self) -> HtmlFloatingPointNumber {
        self.0
    }

    /// Clamps a present Progress value to the interval from zero to its maximum.
    ///
    /// Pass `None` for a present attribute whose numeric recovery failed.
    pub fn determinate_value(
        self,
        value: Option<HtmlFloatingPointNumber>,
    ) -> HtmlFloatingPointNumber {
        clamp(
            value.unwrap_or(HtmlFloatingPointNumber::ZERO),
            HtmlFloatingPointNumber::ZERO,
            self.0,
        )
    }

    /// Computes a determinate position for a present value using this maximum.
    ///
    /// Pass `None` for a present attribute whose numeric recovery failed.
    pub fn determinate_position(
        self,
        value: Option<HtmlFloatingPointNumber>,
    ) -> HtmlFloatingPointNumber {
        // The numerator is nonnegative and no larger than the positive denominator.
        HtmlFloatingPointNumber {
            value: self.determinate_value(value).value / self.0.value,
        }
    }
}

/// A finite HTML Meter minimum and maximum, with the maximum at least the minimum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlMeterRange {
    minimum: HtmlFloatingPointNumber,
    maximum: HtmlFloatingPointNumber,
}

impl HtmlMeterRange {
    /// Resolves both bounds using the recovered numbers or their defaults.
    pub fn new(
        minimum: Option<HtmlFloatingPointNumber>,
        maximum: Option<HtmlFloatingPointNumber>,
    ) -> Self {
        let minimum = Self::minimum_from(minimum);
        let maximum = maximum.unwrap_or(HtmlFloatingPointNumber::ONE);
        let maximum = if maximum.value < minimum.value {
            minimum
        } else {
            maximum
        };
        Self { minimum, maximum }
    }

    /// Resolves the minimum from its own recovered number, defaulting to zero.
    pub fn minimum_from(minimum: Option<HtmlFloatingPointNumber>) -> HtmlFloatingPointNumber {
        minimum.unwrap_or(HtmlFloatingPointNumber::ZERO)
    }

    /// Returns the effective minimum.
    pub fn minimum(self) -> HtmlFloatingPointNumber {
        self.minimum
    }

    /// Returns the effective maximum.
    pub fn maximum(self) -> HtmlFloatingPointNumber {
        self.maximum
    }

    /// Clamps the recovered value, defaulting to zero before clamping.
    pub fn value(self, value: Option<HtmlFloatingPointNumber>) -> HtmlFloatingPointNumber {
        clamp(
            value.unwrap_or(HtmlFloatingPointNumber::ZERO),
            self.minimum,
            self.maximum,
        )
    }

    /// Clamps the recovered low boundary, defaulting to the minimum.
    pub fn low(self, low: Option<HtmlFloatingPointNumber>) -> HtmlFloatingPointNumber {
        clamp(low.unwrap_or(self.minimum), self.minimum, self.maximum)
    }

    /// Clamps the recovered high boundary between effective low and maximum.
    pub fn high(
        self,
        low: Option<HtmlFloatingPointNumber>,
        high: Option<HtmlFloatingPointNumber>,
    ) -> HtmlFloatingPointNumber {
        clamp(high.unwrap_or(self.maximum), self.low(low), self.maximum)
    }

    /// Clamps the recovered optimum, defaulting to the finite interval midpoint.
    pub fn optimum(self, optimum: Option<HtmlFloatingPointNumber>) -> HtmlFloatingPointNumber {
        let optimum = optimum.unwrap_or_else(|| {
            // Use one midpoint operation to avoid endpoint overflow, early
            // rounding, and underflow. Keep the scalar's canonical zero.
            let value = self.minimum.value.midpoint(self.maximum.value);
            if value == 0.0 {
                HtmlFloatingPointNumber::ZERO
            } else {
                HtmlFloatingPointNumber { value }
            }
        });
        clamp(optimum, self.minimum, self.maximum)
    }
}

fn clamp(
    number: HtmlFloatingPointNumber,
    minimum: HtmlFloatingPointNumber,
    maximum: HtmlFloatingPointNumber,
) -> HtmlFloatingPointNumber {
    if number.value < minimum.value {
        minimum
    } else if number.value > maximum.value {
        maximum
    } else {
        number
    }
}
