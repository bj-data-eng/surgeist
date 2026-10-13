//! Exact intersection of a Range step lattice with finite binary64 values.
//!
//! Every reply performs one fixed scalar transition. No floating arithmetic,
//! source access, storage access, allocation or unbounded lattice enumeration
//! occurs here. Mantissa congruences are evaluated in each representability
//! band; the only multiplication is bounded u64 x u64 -> u128.

use super::{RangeNumericError, RangeNumericReply, RangeNumericRequest};

const FRACTION: u64 = (1_u64 << 52) - 1;
const HIDDEN: u64 = 1_u64 << 52;
const MAX_MANTISSA: u64 = (1_u64 << 53) - 1;
const SIGN: u64 = 1_u64 << 63;
const BANDS: u16 = 4094;

#[derive(Clone, Copy, Debug)]
struct Dyadic {
    magnitude: u64,
    exponent: i16,
    negative: bool,
}
impl Dyadic {
    fn from_bits(bits: u64) -> Self {
        let exponent = ((bits >> 52) & 0x7ff) as i16;
        Self {
            magnitude: if exponent == 0 {
                bits & FRACTION
            } else {
                (bits & FRACTION) | HIDDEN
            },
            exponent: if exponent == 0 {
                -1074
            } else {
                exponent - 1075
            },
            negative: bits & SIGN != 0,
        }
    }
    fn odd(self) -> Self {
        if self.magnitude == 0 {
            return self;
        }
        let zeros = self.magnitude.trailing_zeros();
        Self {
            magnitude: self.magnitude >> zeros,
            exponent: self.exponent + zeros as i16,
            negative: self.negative,
        }
    }
}

/// Exact signed sum of the two finite endpoints followed by division by two
/// and one nearest/even binary64 rounding. Each reply reads only two fixed
/// mantissas at one binary position; the receiver never owns a bigint.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeMidpointCursor {
    first: Dyadic,
    second: Dyadic,
    subtract: bool,
    position: i16,
    carry: i8,
    window: u64,
    sticky: bool,
    highest: Option<i16>,
    selected_window: u64,
    selected_sticky: bool,
    result: Option<u64>,
}
impl RangeMidpointCursor {
    pub(crate) fn new(minimum: u64, maximum: u64) -> Result<Self, RangeNumericError> {
        let a = f64::from_bits(minimum);
        let b = f64::from_bits(maximum);
        if !a.is_finite() || !b.is_finite() {
            return Err(RangeNumericError::Integrity);
        }
        let (first, second) = if minimum & !SIGN >= maximum & !SIGN {
            (Dyadic::from_bits(minimum), Dyadic::from_bits(maximum))
        } else {
            (Dyadic::from_bits(maximum), Dyadic::from_bits(minimum))
        };
        Ok(Self {
            first,
            second,
            subtract: first.negative != second.negative,
            position: -1074,
            carry: 0,
            window: 0,
            sticky: false,
            highest: None,
            selected_window: 0,
            selected_sticky: false,
            result: None,
        })
    }
    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        self.result
            .is_none()
            .then_some(RangeNumericRequest::ScalarStep)
    }
    pub(crate) fn result(&self) -> Option<u64> {
        self.result
    }
    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        if self.result.is_some() || !matches!(reply, RangeNumericReply::Ack) {
            return Err(RangeNumericError::Integrity);
        }
        let mut next = *self;
        if next.position <= 1024 {
            let bit = |term: Dyadic| {
                let offset = next.position - term.exponent;
                if (0..53).contains(&offset) {
                    ((term.magnitude >> offset) & 1) as i8
                } else {
                    0
                }
            };
            let total = next.carry
                + bit(next.first)
                + if next.subtract {
                    -bit(next.second)
                } else {
                    bit(next.second)
                };
            let digit = total.rem_euclid(2) as u64;
            next.carry = total.div_euclid(2);
            next.sticky |= next.window & 1 != 0;
            next.window = (next.window >> 1) | (digit << 53);
            if digit != 0 {
                next.highest = Some(next.position);
                next.selected_window = next.window;
                next.selected_sticky = next.sticky;
            }
            next.position += 1;
        } else {
            if next.carry != 0 {
                return Err(RangeNumericError::Integrity);
            }
            next.result = Some(match next.highest {
                None => 0,
                Some(highest) => {
                    let sign = if next.first.negative { SIGN } else { 0 };
                    // The averaged highest exponent is highest-1.
                    if highest >= -1021 {
                        let mut mantissa = next.selected_window >> 1;
                        let guard = next.selected_window & 1 != 0;
                        if guard && (next.selected_sticky || mantissa & 1 != 0) {
                            mantissa += 1;
                        }
                        let mut exponent = highest - 1;
                        if mantissa == 1_u64 << 53 {
                            mantissa >>= 1;
                            exponent += 1;
                        }
                        sign | (((exponent + 1023) as u64) << 52) | (mantissa & FRACTION)
                    } else {
                        // At most 53 sum bits survive in the subnormal range.
                        // Sum bit -1074 is the sole rounding guard.
                        let shift = (-1021 - highest) as u32;
                        let sum = next.selected_window >> shift;
                        let mut mantissa = sum >> 1;
                        if sum & 1 != 0 && mantissa & 1 != 0 {
                            mantissa += 1;
                        }
                        sign | mantissa
                    }
                }
            });
        }
        *self = next;
        Ok(())
    }
}

/// One bit of exact 2*current - lower - upper per transition. Signed carry
/// uses Euclidean division, so final negative carry distinguishes a negative
/// integer from its nonnegative low bits without a temporary bigint.
#[derive(Clone, Copy, Debug)]
struct DistanceComparison {
    terms: [Dyadic; 3],
    bit: i16,
    end: i16,
    carry: i16,
    nonzero: bool,
}
impl DistanceComparison {
    fn new(current: u64, lower: u64, upper: u64) -> Self {
        let mut current = Dyadic::from_bits(current);
        current.exponent += 1;
        let mut lower = Dyadic::from_bits(lower);
        lower.negative = !lower.negative;
        let mut upper = Dyadic::from_bits(upper);
        upper.negative = !upper.negative;
        let terms = [current, lower, upper];
        let mut bit = 1026;
        let mut end = -1075;
        // Exactly three fixed headers; this is not a digit/source pass.
        for term in terms {
            if term.magnitude != 0 {
                bit = bit.min(term.exponent);
                end = end.max(term.exponent + 53);
            }
        }
        Self {
            terms,
            bit,
            end,
            carry: 0,
            nonzero: false,
        }
    }
    fn advance(&mut self) -> Option<std::cmp::Ordering> {
        if self.bit > self.end {
            return Some(if self.carry < 0 {
                std::cmp::Ordering::Less
            } else if self.nonzero {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            });
        }
        let mut total = self.carry;
        for term in self.terms {
            let offset = self.bit - term.exponent;
            if (0..53).contains(&offset) && (term.magnitude >> offset) & 1 != 0 {
                total += if term.negative { -1 } else { 1 };
            }
        }
        let digit = total.rem_euclid(2);
        self.nonzero |= digit != 0;
        self.carry = total.div_euclid(2);
        self.bit += 1;
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    BasePower,
    Band,
    Lift,
    Congruence,
    Candidates,
    Compare,
    Done,
}

/// Fixed scalar machine. The caller admits the actual Copy receiver and fixed
/// transition before each Ack; no request can read a source/scratch cell.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeLatticeCursor {
    current: u64,
    minimum: u64,
    maximum: Option<u64>,
    base: Dyadic,
    step: Dyadic,
    phase: Phase,
    band: u16,
    negative: bool,
    exponent: i16,
    low: u64,
    high: u64,
    target: u64,
    base_mod: u64,
    power_base: u64,
    power_value: u64,
    power_exponent: u16,
    inverse: u64,
    inverse_exponent: i16,
    residue: u64,
    modulus: u64,
    large_modulus: bool,
    candidate: u64,
    other: Option<u64>,
    below: Option<u64>,
    above: Option<u64>,
    comparison: Option<DistanceComparison>,
    result: Option<Option<u64>>,
}
impl RangeLatticeCursor {
    pub(crate) fn new(
        current: u64,
        minimum: u64,
        maximum: Option<u64>,
        base: u64,
        step: u64,
    ) -> Result<Self, RangeNumericError> {
        if !f64::from_bits(current).is_finite()
            || !f64::from_bits(minimum).is_finite()
            || maximum.is_some_and(|v| !f64::from_bits(v).is_finite())
            || !f64::from_bits(base).is_finite()
            || !f64::from_bits(step).is_finite()
            || f64::from_bits(step) <= 0.0
            || maximum.is_some_and(|v| f64::from_bits(v) < f64::from_bits(minimum))
        {
            return Err(RangeNumericError::Integrity);
        }
        let base = Dyadic::from_bits(base).odd();
        let step = Dyadic::from_bits(step).odd();
        let exponent = if base.magnitude != 0 && base.exponent >= step.exponent {
            (base.exponent - step.exponent) as u16
        } else {
            0
        };
        let modulus = step.magnitude;
        Ok(Self {
            current,
            minimum,
            maximum,
            base,
            step,
            phase: Phase::BasePower,
            band: 0,
            negative: false,
            exponent: -1074,
            low: 0,
            high: 0,
            target: 0,
            base_mod: base.magnitude % modulus,
            power_base: 2 % modulus,
            power_value: 1 % modulus,
            power_exponent: exponent,
            inverse: 1 % modulus,
            inverse_exponent: step.exponent,
            residue: 0,
            modulus: 0,
            large_modulus: false,
            candidate: 0,
            other: None,
            below: None,
            above: None,
            comparison: None,
            result: None,
        })
    }

    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        (self.phase != Phase::Done).then_some(RangeNumericRequest::ScalarStep)
    }

    pub(crate) fn result(&self) -> Option<Option<u64>> {
        self.result
    }

    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        if self.phase == Phase::Done || !matches!(reply, RangeNumericReply::Ack) {
            return Err(RangeNumericError::Integrity);
        }
        // Ticket-free actual fixed receiver; a rejected reply never commits.
        let mut next = *self;
        next.advance()?;
        *self = next;
        Ok(())
    }

    fn product_mod(a: u64, b: u64, modulus: u64) -> u64 {
        ((u128::from(a) * u128::from(b)) % u128::from(modulus)) as u64
    }

    fn advance(&mut self) -> Result<(), RangeNumericError> {
        match self.phase {
            Phase::BasePower => {
                if self.power_exponent != 0 {
                    if self.power_exponent & 1 != 0 {
                        self.power_value = Self::product_mod(
                            self.power_value,
                            self.power_base,
                            self.step.magnitude,
                        );
                    }
                    self.power_base =
                        Self::product_mod(self.power_base, self.power_base, self.step.magnitude);
                    self.power_exponent >>= 1;
                } else {
                    self.base_mod =
                        Self::product_mod(self.base_mod, self.power_value, self.step.magnitude);
                    self.phase = Phase::Band;
                }
            }
            Phase::Band => self.prepare_band(),
            Phase::Lift => {
                if self.inverse_exponent < self.exponent {
                    self.inverse = Self::product_mod(
                        self.inverse,
                        self.step.magnitude.div_ceil(2),
                        self.step.magnitude,
                    );
                    self.inverse_exponent += 1;
                } else {
                    self.phase = Phase::Congruence;
                }
            }
            Phase::Congruence => self.congruence(),
            Phase::Candidates => {
                self.consider(self.candidate);
                if self.phase != Phase::Done {
                    if let Some(other) = self.other.take() {
                        self.candidate = other;
                    } else {
                        self.band += 1;
                        self.phase = Phase::Band;
                    }
                }
            }
            Phase::Compare => {
                let Some(comparison) = &mut self.comparison else {
                    return Err(RangeNumericError::Integrity);
                };
                if let Some(order) = comparison.advance() {
                    self.finish(if order == std::cmp::Ordering::Less {
                        self.below
                    } else {
                        self.above
                    });
                }
            }
            Phase::Done => return Err(RangeNumericError::Integrity),
        }
        Ok(())
    }

    fn bits(&self, mantissa: u64) -> u64 {
        if mantissa == 0 {
            return 0;
        }
        let magnitude = if mantissa < HIDDEN {
            mantissa
        } else {
            (((self.exponent + 1075) as u64) << 52) | (mantissa - HIDDEN)
        };
        magnitude | if self.negative { SIGN } else { 0 }
    }

    fn band_mantissa(bits: u64) -> u64 {
        let value = Dyadic::from_bits(bits);
        value.magnitude
    }

    fn skip(&mut self) {
        self.band += 1;
        self.phase = Phase::Band;
    }

    fn finish(&mut self, result: Option<u64>) {
        self.result = Some(result);
        self.phase = Phase::Done;
    }

    fn prepare_band(&mut self) {
        if self.band == BANDS {
            match (self.below, self.above) {
                (Some(lower), Some(upper)) => {
                    self.comparison = Some(DistanceComparison::new(self.current, lower, upper));
                    self.phase = Phase::Compare;
                }
                (Some(value), None) | (None, Some(value)) => self.finish(Some(value)),
                (None, None) => self.finish(None),
            }
            return;
        }
        let band = self.band / 2;
        self.negative = self.band & 1 != 0;
        self.exponent = if band == 0 { -1074 } else { band as i16 - 1075 };
        self.low = if band == 0 {
            u64::from(self.negative)
        } else {
            HIDDEN
        };
        self.high = if band == 0 { FRACTION } else { MAX_MANTISSA };
        let low_value = f64::from_bits(self.bits(if self.negative { self.high } else { self.low }));
        let high_value =
            f64::from_bits(self.bits(if self.negative { self.low } else { self.high }));
        let minimum = f64::from_bits(self.minimum);
        let maximum = self.maximum.map(f64::from_bits);
        if minimum > high_value || maximum.is_some_and(|maximum| maximum < low_value) {
            self.skip();
            return;
        }
        if minimum >= low_value {
            if self.negative {
                self.high = Self::band_mantissa(self.minimum);
            } else {
                self.low = Self::band_mantissa(self.minimum);
            }
        }
        if let Some(maximum) = maximum
            && maximum <= high_value
        {
            let mantissa = Self::band_mantissa(self.maximum.expect("actual upper bound"));
            if self.negative {
                self.low = mantissa;
            } else {
                self.high = mantissa;
            }
        }
        let current = f64::from_bits(self.current);
        self.target = if current < low_value {
            if self.negative { self.high } else { self.low }
        } else if current > high_value {
            if self.negative { self.low } else { self.high }
        } else {
            Self::band_mantissa(self.current).clamp(self.low, self.high)
        };
        if self.exponent >= self.step.exponent {
            self.phase = Phase::Lift;
        } else {
            self.phase = Phase::Congruence;
        }
    }

    fn signed_remainder(&self, remainder: u64, modulus: u64) -> u64 {
        if self.base.negative != self.negative && remainder != 0 {
            modulus - remainder
        } else {
            remainder
        }
    }

    /// A finite band's M satisfies +/-M*2^e = b (mod h). Normalizing h
    /// leaves an odd modulus <=53 bits. Large powers are classified by length
    /// before shifts; no integer larger than u128 is constructed.
    fn congruence(&mut self) {
        if self.exponent >= self.step.exponent {
            if self.base.magnitude != 0 && self.base.exponent < self.step.exponent {
                self.skip();
                return;
            }
            self.modulus = self.step.magnitude;
            self.large_modulus = false;
            self.residue = Self::product_mod(
                self.signed_remainder(self.base_mod, self.step.magnitude),
                self.inverse,
                self.step.magnitude,
            );
        } else {
            if self.base.magnitude != 0 && self.base.exponent < self.exponent {
                self.skip();
                return;
            }
            let shift = (self.step.exponent - self.exponent) as u32;
            let width = 64 - self.step.magnitude.leading_zeros();
            self.large_modulus = width + shift > 53;
            self.modulus = if self.large_modulus {
                0
            } else {
                self.step.magnitude << shift
            };
            let (inner, shift) = if self.base.magnitude == 0 {
                (0, 0)
            } else if self.base.exponent >= self.step.exponent {
                (
                    self.signed_remainder(self.base_mod, self.step.magnitude),
                    shift,
                )
            } else {
                let difference = (self.step.exponent - self.base.exponent) as u32;
                let base_negative = self.base.negative != self.negative;
                let inner = if width + difference > 54 {
                    if base_negative {
                        self.skip();
                        return;
                    }
                    self.base.magnitude
                } else {
                    let modulus = self.step.magnitude << difference;
                    self.signed_remainder(self.base.magnitude % modulus, modulus)
                };
                (inner, (self.base.exponent - self.exponent) as u32)
            };
            if inner != 0 && (64 - inner.leading_zeros()) + shift > 53 {
                self.skip();
                return;
            }
            self.residue = if inner == 0 { 0 } else { inner << shift };
        }
        if self.large_modulus {
            if !(self.low..=self.high).contains(&self.residue) {
                self.skip();
                return;
            }
            self.candidate = self.residue;
            self.other = None;
        } else {
            let modulus = self.modulus;
            let first = self.low + (self.residue + modulus - self.low % modulus) % modulus;
            if first > self.high {
                self.skip();
                return;
            }
            let last = self.high - (self.high + modulus - self.residue) % modulus;
            let target = self.target.clamp(first, last);
            self.candidate = first + ((target - first) / modulus) * modulus;
            self.other = if self.candidate < target && self.candidate + modulus <= last {
                Some(self.candidate + modulus)
            } else {
                None
            };
        }
        self.phase = Phase::Candidates;
    }

    fn consider(&mut self, mantissa: u64) {
        let bits = self.bits(mantissa);
        let value = f64::from_bits(bits);
        let current = f64::from_bits(self.current);
        if value == current {
            // Preserve the actual current signed-zero representation as well.
            self.finish(Some(self.current));
        } else if value < current {
            if self
                .below
                .is_none_or(|previous| value > f64::from_bits(previous))
            {
                self.below = Some(bits);
            }
        } else if self
            .above
            .is_none_or(|previous| value < f64::from_bits(previous))
        {
            self.above = Some(bits);
        }
    }
}

#[cfg(test)]
mod midpoint_tests {
    use super::*;
    fn midpoint(a: f64, b: f64) -> f64 {
        let mut cursor = RangeMidpointCursor::new(a.to_bits(), b.to_bits()).unwrap();
        let mut actions = 0;
        while cursor.request().is_some() {
            cursor.reply(RangeNumericReply::Ack).unwrap();
            actions += 1;
            assert!(actions <= 2101);
        }
        f64::from_bits(cursor.result().unwrap())
    }
    #[test]
    fn exact_average_handles_overflow_underflow_signed_and_nearest_even() {
        assert_eq!(midpoint(0.0, 100.0), 50.0);
        assert_eq!(midpoint(f64::MAX, f64::MAX), f64::MAX);
        assert_eq!(midpoint(-f64::MAX, f64::MAX).to_bits(), 0);
        assert_eq!(midpoint(-100.0, 0.0), -50.0);
        assert_eq!(midpoint(f64::from_bits(1), 0.0).to_bits(), 0);
        assert_eq!(midpoint(f64::from_bits(3), 0.0).to_bits(), 2);
        assert_eq!(midpoint(f64::from_bits(1), f64::from_bits(2)).to_bits(), 2);
        assert_eq!(
            midpoint(1.0, f64::from_bits(1.0f64.to_bits() + 1)).to_bits(),
            1.0f64.to_bits()
        );
        assert_eq!(
            midpoint(
                f64::from_bits(1.0f64.to_bits() + 1),
                f64::from_bits(1.0f64.to_bits() + 2)
            )
            .to_bits(),
            1.0f64.to_bits() + 2
        );
    }
    #[test]
    fn mismatched_reply_preserves_cached_action_and_finite_result() {
        let mut cursor = RangeMidpointCursor::new(0, 100.0f64.to_bits()).unwrap();
        assert!(cursor.reply(RangeNumericReply::Unit(0)).is_err());
        assert!(matches!(
            cursor.request(),
            Some(RangeNumericRequest::ScalarStep)
        ));
        while cursor.request().is_some() {
            cursor.reply(RangeNumericReply::Ack).unwrap();
        }
        assert_eq!(cursor.result(), Some(50.0f64.to_bits()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choose(current: f64, min: f64, max: Option<f64>, base: f64, step: f64) -> Option<f64> {
        let mut cursor = RangeLatticeCursor::new(
            current.to_bits(),
            min.to_bits(),
            max.map(f64::to_bits),
            base.to_bits(),
            step.to_bits(),
        )
        .unwrap();
        let mut actions = 0;
        while cursor.request().is_some() {
            cursor.reply(RangeNumericReply::Ack).unwrap();
            actions += 1;
            assert!(actions < 100_000, "finite format bounds scalar transitions");
        }
        cursor.result().unwrap().map(f64::from_bits)
    }

    #[test]
    fn ties_choose_the_larger_exact_member_and_bounds_can_remove_both() {
        assert_eq!(choose(50.0, 0.0, Some(100.0), 0.0, 20.0), Some(60.0));
        assert_eq!(choose(0.0, -30.0, Some(10.0), -30.0, 20.0), Some(10.0));
        assert_eq!(choose(10.0, 0.0, Some(10.0), 0.0, 4.0), Some(8.0));
        assert_eq!(choose(1.0, 0.0, Some(1.0), 3.0, 4.0), None);
    }

    #[test]
    fn parsed_dyadic_step_intersection_does_not_round_a_nonrepresentable_point() {
        // The selected .1 binary64 dyadic has an odd 52-bit significand.
        // Its third mathematical multiple cannot be represented in this band;
        // .2 and .4 can, and literal .3 is closer to .2.
        assert_eq!(choose(0.3, 0.0, Some(1.0), 0.0, 0.1), Some(0.2));
        assert_eq!(choose(-0.3, -1.0, Some(0.0), 0.0, 0.1), Some(-0.2));
        assert_eq!(choose(0.2, 0.0, Some(1.0), 0.0, 0.1), Some(0.2));
    }

    #[test]
    fn zero_subnormal_and_largest_finite_domains_remain_exact() {
        let tiny = f64::from_bits(1);
        assert_eq!(
            choose(-0.0, -1.0, Some(1.0), 0.0, tiny).unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(
            choose(
                f64::from_bits(3),
                0.0,
                Some(f64::from_bits(8)),
                0.0,
                tiny * 2.0
            ),
            Some(f64::from_bits(4))
        );
        assert_eq!(
            choose(f64::MAX, 0.0, Some(f64::MAX), 0.0, f64::MAX),
            Some(f64::MAX)
        );
        assert_eq!(
            choose(-f64::MAX, -f64::MAX, Some(f64::MAX), 0.0, f64::MAX),
            Some(-f64::MAX)
        );
    }

    #[test]
    fn invalid_request_reply_and_nonfinite_inputs_never_advance() {
        assert!(
            RangeLatticeCursor::new(f64::INFINITY.to_bits(), 0, None, 0, 1.0f64.to_bits()).is_err()
        );
        assert!(RangeLatticeCursor::new(0, 0, None, 0, 0).is_err());
        let mut cursor = RangeLatticeCursor::new(0, 0, None, 0, 1.0f64.to_bits()).unwrap();
        let before = cursor;
        assert!(cursor.reply(RangeNumericReply::Unit(0)).is_err());
        assert_eq!(format!("{cursor:?}"), format!("{before:?}"));
        while cursor.request().is_some() {
            cursor.reply(RangeNumericReply::Ack).unwrap();
        }
        assert!(cursor.reply(RangeNumericReply::Ack).is_err());
    }
}
