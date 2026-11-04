//! A controlled exact running total of nonnegative decimals.
//!
//! The total keeps the canonical digits of the running sum, so adding an operand visits the
//! operand's own digits and then only the places a carry crosses, however long the total is. Each
//! carry crossing a place turns a nine into a zero and each place an addition touches creates at
//! most one nine, so the places crossed over any series of additions are bounded by the digits
//! added. A carry is walked a bounded chunk at a time, and a longer carry is left pending for
//! further charged steps. The digit limit is therefore enforced exactly where a canonical sum
//! first needs more digits than allowed, and every allocation is reserved and observed under the
//! caller's limits.

use super::{
    HtmlIntegerUnitStepError, HtmlNonNegativeDecimal, HtmlNonNegativeInteger,
    HtmlNumericMaterializationFailure, HtmlNumericMaterialized, ascii_digit_string, bounded_work,
    check_integer_actual_capacity, check_integer_digit_capacity, materialization_boundary,
    reserve_materialization,
};

/// The places of a carry that one step walks. A pending carry is resumed by further steps.
const CARRY_CHUNK: usize = 32;

/// The exact sum of the decimals added so far, in canonical digits.
///
/// Integer places run from the least significant, and the most significant place is never zero,
/// so a zero integer part has no places. Fractional places run from the tenths, and trailing
/// zeroes are kept until the total is finished. A carry out of an addition's own places may be
/// left pending; the total is exact only once none is pending.
#[derive(Debug, Default)]
pub(crate) struct HtmlDecimalTotal {
    integer: Vec<u8>,
    fraction: Vec<u8>,
    /// The integer place that still has to receive one.
    carry: Option<usize>,
}

impl HtmlDecimalTotal {
    /// Whether a carry still has to be walked, which only `try_carried` does.
    pub(crate) fn carry_pending(&self) -> bool {
        self.carry.is_some()
    }

    /// The upper bound on the digit-visiting steps of the matching `try_added_to`: filling the
    /// places that `addend` newly needs, adding its digits, and walking the first chunk of a
    /// carry. A copy made by growing the integer places is bounded by the places the carry
    /// already walked or by the addend's digits. `None` means the bound is not representable.
    pub(crate) fn added_work_bound(&self, addend: &HtmlNonNegativeDecimal) -> Option<usize> {
        let n = addend
            .integer
            .digits
            .len()
            .checked_add(addend.fraction.len())?;
        bounded_work(&[n, n, n, n, CARRY_CHUNK, CARRY_CHUNK])
    }

    /// The upper bound on the digit-visiting steps of the matching `try_carried`.
    pub(crate) fn carried_work_bound(&self) -> Option<usize> {
        bounded_work(&[CARRY_CHUNK, CARRY_CHUNK])
    }

    /// The upper bound on the digit-visiting steps of the matching `try_finished`.
    pub(crate) fn finished_work_bound(&self) -> Option<usize> {
        let n = self.integer.len().checked_add(self.fraction.len())?;
        bounded_work(&[n, n, n, CARRY_CHUNK])
    }

    /// Adds `addend` exactly and walks the first chunk of its carry. The integer part of the sum
    /// obeys `max_digits`, and the allocation observed in this step stays within `max_bytes`;
    /// failure reports the capacity observed so far. A carry still pending from an earlier
    /// addition is walked to its end first.
    pub(crate) fn try_added_to(
        &mut self,
        addend: &HtmlNonNegativeDecimal,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<()>, HtmlNumericMaterializationFailure> {
        let mut observed = 0;
        self.add(addend, max_digits, max_bytes, &mut observed)
            .map(|()| HtmlNumericMaterialized {
                value: (),
                capacity_bytes: observed,
            })
            .map_err(|kind| materialization_boundary(kind, observed))
    }

    /// Walks a further chunk of the pending carry; with none pending it changes nothing.
    pub(crate) fn try_carried(
        &mut self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<()>, HtmlNumericMaterializationFailure> {
        let mut observed = 0;
        self.walk_carry(CARRY_CHUNK, max_digits, max_bytes, &mut observed)
            .map(|()| HtmlNumericMaterialized {
                value: (),
                capacity_bytes: observed,
            })
            .map_err(|kind| materialization_boundary(kind, observed))
    }

    /// The canonical value, with no leading integer zeroes and no trailing fractional zeroes. A
    /// carry still pending is walked to its end first.
    pub(crate) fn try_finished(
        &mut self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<HtmlNonNegativeDecimal>, HtmlNumericMaterializationFailure>
    {
        let mut observed = 0;
        self.walk_carry(usize::MAX, max_digits, max_bytes, &mut observed)
            .map_err(|kind| materialization_boundary(kind, observed))?;
        let fraction_len = self
            .fraction
            .iter()
            .rposition(|&place| place != 0)
            .map_or(0, |last| last + 1);
        let mut integer = reserve_materialization(
            self.integer.len().max(1),
            Some(max_digits),
            max_bytes,
            &mut observed,
        )?;
        if self.integer.is_empty() {
            integer.push(b'0');
        } else {
            integer.extend(self.integer.iter().rev().map(|&place| ascii(place)));
        }
        let mut fraction = reserve_materialization(fraction_len, None, max_bytes, &mut observed)?;
        fraction.extend(
            self.fraction
                .iter()
                .take(fraction_len)
                .map(|&place| ascii(place)),
        );
        Ok(HtmlNumericMaterialized {
            value: HtmlNonNegativeDecimal {
                integer: HtmlNonNegativeInteger {
                    digits: ascii_digit_string(integer),
                },
                fraction: ascii_digit_string(fraction),
            },
            capacity_bytes: observed,
        })
    }

    fn add(
        &mut self,
        addend: &HtmlNonNegativeDecimal,
        max_digits: usize,
        max_bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlIntegerUnitStepError> {
        self.walk_carry(usize::MAX, max_digits, max_bytes, observed)?;
        // A zero integer part has no places, so adding it cannot leave a leading zero.
        let addend_integer: &[u8] = if addend.integer.is_zero() {
            &[]
        } else {
            addend.integer.digits.as_bytes()
        };
        let integer_places = self.integer.len().max(addend_integer.len());
        check_integer_digit_capacity(integer_places, max_digits, usize::MAX)?;
        grow(&mut self.integer, integer_places, max_bytes, observed)?;
        let fraction_places = self.fraction.len().max(addend.fraction.len());
        grow(&mut self.fraction, fraction_places, max_bytes, observed)?;
        let carry = add_places(
            self.fraction.iter_mut().take(addend.fraction.len()).rev(),
            addend.fraction.bytes().rev(),
            0,
        );
        let carry = add_places(
            self.integer.iter_mut(),
            addend_integer.iter().rev().copied(),
            carry,
        );
        self.carry = (carry != 0).then_some(addend_integer.len());
        self.walk_carry(CARRY_CHUNK, max_digits, max_bytes, observed)
    }

    /// Walks at most `steps` places of the pending carry. It adds one place when the carry leaves
    /// the top, and then needs one digit more than the total had.
    fn walk_carry(
        &mut self,
        steps: usize,
        max_digits: usize,
        max_bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlIntegerUnitStepError> {
        let mut walked = 0;
        while let Some(position) = self.carry {
            if walked == steps {
                return Ok(());
            }
            walked += 1;
            match self.integer.get_mut(position) {
                Some(place) if *place == 9 => {
                    *place = 0;
                    self.carry = Some(position + 1);
                }
                Some(place) => {
                    *place += 1;
                    self.carry = None;
                }
                None => {
                    let required = position + 1;
                    check_integer_digit_capacity(required, max_digits, usize::MAX)?;
                    grow(&mut self.integer, required, max_bytes, observed)?;
                    if let Some(place) = self.integer.get_mut(position) {
                        *place = 1;
                    }
                    self.carry = None;
                }
            }
        }
        Ok(())
    }
}

fn ascii(place: u8) -> u8 {
    b'0'.wrapping_add(place)
}

/// Adds the digits of `operand` into `places`, both least significant first, then returns the
/// carry out of the last pair.
fn add_places<'a>(
    places: impl Iterator<Item = &'a mut u8>,
    operand: impl Iterator<Item = u8>,
    mut carry: u8,
) -> u8 {
    for (place, digit) in places.zip(operand) {
        let total = place
            .wrapping_add(digit.wrapping_sub(b'0'))
            .wrapping_add(carry);
        *place = total % 10;
        carry = total / 10;
    }
    carry
}

/// Extends `places` with zeroes to `target` places, reserving exactly the room that it lacks
/// under the byte limit and observing the capacity of the allocation even when it fails.
fn grow(
    places: &mut Vec<u8>,
    target: usize,
    max_bytes: usize,
    observed: &mut usize,
) -> Result<(), HtmlIntegerUnitStepError> {
    if target <= places.len() {
        return Ok(());
    }
    if places.capacity() < target {
        let required = observed
            .checked_add(target)
            .ok_or(HtmlIntegerUnitStepError::RepresentationExhausted)?;
        if required > max_bytes {
            return Err(HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                limit: max_bytes,
                required,
            });
        }
        let allocation = places.try_reserve_exact(target - places.len());
        // Even an allocation failure retains any backing it obtained.
        *observed = observed
            .checked_add(places.capacity())
            .ok_or(HtmlIntegerUnitStepError::RepresentationExhausted)?;
        allocation.map_err(|_| HtmlIntegerUnitStepError::AllocationFailed { requested: target })?;
        check_integer_actual_capacity(required, *observed, max_bytes)?;
    }
    places.resize(target, 0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::values::DocumentString;

    fn decimal(integer: &str, fraction: &str) -> HtmlNonNegativeDecimal {
        let integer = DocumentString::from(integer);
        let fraction = DocumentString::from(fraction);
        HtmlNonNegativeDecimal::try_from_borrowed_digits(
            integer.code_units(),
            fraction.code_units(),
            usize::MAX,
            usize::MAX,
        )
        .unwrap()
        .value
    }

    /// Adds `addend` and walks any pending carry, as the controlled stage would.
    fn add(total: &mut HtmlDecimalTotal, addend: &HtmlNonNegativeDecimal, max_digits: usize) {
        total.try_added_to(addend, max_digits, usize::MAX).unwrap();
        while total.carry_pending() {
            total.try_carried(max_digits, usize::MAX).unwrap();
        }
    }

    fn finished(total: &mut HtmlDecimalTotal) -> HtmlNonNegativeDecimal {
        total.try_finished(usize::MAX, usize::MAX).unwrap().value
    }

    /// A small deterministic generator (xorshift64*).
    struct Random(u64);
    impl Random {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            (self.0.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as usize % bound
        }
        /// `min` digits and up to `extra - 1` more.
        fn digits(&mut self, min: usize, extra: usize) -> String {
            let length = min + self.below(extra);
            (0..length)
                .map(|_| ['0', '9', '9', '1', '5'][self.below(5)])
                .collect()
        }
    }

    /// Hand-derived carries: through a run of nines, into a new top place and out of the fraction.
    #[test]
    fn carries_cross_nines_the_top_and_the_fraction() {
        let mut total = HtmlDecimalTotal::default();
        assert_eq!(finished(&mut total), decimal("0", ""));
        add(&mut total, &decimal(&"9".repeat(100), "5"), usize::MAX);
        add(&mut total, &decimal("0", "5"), usize::MAX);
        assert_eq!(
            finished(&mut total),
            decimal(&format!("1{}", "0".repeat(100)), "")
        );
        add(&mut total, &decimal("0", "05"), usize::MAX);
        add(&mut total, &decimal("0", "0500"), usize::MAX);
        let value = finished(&mut total);
        assert_eq!(
            value.integer_part().decimal_digits(),
            format!("1{}", "0".repeat(100))
        );
        assert_eq!(value.fractional_digits(), Some("1"));
    }

    /// A carry longer than one chunk stays pending, and each further step walks one more chunk.
    #[test]
    fn a_long_carry_stays_pending_across_steps() {
        let mut total = HtmlDecimalTotal::default();
        add(
            &mut total,
            &decimal(&"9".repeat(CARRY_CHUNK * 3 + 5), ""),
            usize::MAX,
        );
        total
            .try_added_to(&decimal("1", ""), usize::MAX, usize::MAX)
            .unwrap();
        let mut steps = 0;
        while total.carry_pending() {
            total.try_carried(usize::MAX, usize::MAX).unwrap();
            steps += 1;
        }
        assert_eq!(steps, 3);
        assert_eq!(
            finished(&mut total).integer_part().decimal_digits(),
            format!("1{}", "0".repeat(CARRY_CHUNK * 3 + 5))
        );
    }

    /// Random series of additions equal the crate's exact decimal addition place for place.
    #[test]
    fn random_series_equal_exact_decimal_addition() {
        let mut random = Random(0x9e37_79b9_7f4a_7c15);
        for _ in 0..400 {
            let mut total = HtmlDecimalTotal::default();
            let mut expected = decimal("0", "");
            for _ in 0..1 + random.below(30) {
                let long = random.below(6) == 0;
                let integer = random.digits(1, if long { 150 } else { 8 });
                let fraction = random.digits(0, if long { 100 } else { 6 });
                let addend = decimal(&integer, &fraction);
                add(&mut total, &addend, usize::MAX);
                expected = expected.added_to(&addend);
            }
            assert_eq!(finished(&mut total), expected);
        }
    }

    /// The digit limit fails at the same addition and with the same required digits as the
    /// whole-number addition does.
    #[test]
    fn the_digit_limit_fails_where_whole_number_addition_fails() {
        let mut random = Random(0x1234_5678_9abc_def1);
        let mut failures = 0;
        for _ in 0..600 {
            let limit = 1 + random.below(40);
            let mut total = HtmlDecimalTotal::default();
            let mut expected = decimal("0", "");
            for _ in 0..1 + random.below(20) {
                let integer = random.digits(1, limit);
                let fraction = random.digits(0, 4);
                let addend = decimal(&integer, &fraction);
                let whole = expected.try_added_to(&addend, limit, usize::MAX);
                let mut result = total.try_added_to(&addend, limit, usize::MAX);
                while result.is_ok() && total.carry_pending() {
                    result = total.try_carried(limit, usize::MAX);
                }
                match (whole, result) {
                    (Ok(sum), Ok(_)) => expected = sum.value,
                    (Err(whole), Err(controlled)) => {
                        assert_eq!(whole.kind, controlled.kind);
                        failures += 1;
                        break;
                    }
                    (whole, controlled) => {
                        panic!("limit {limit}: {whole:?} against {controlled:?}")
                    }
                }
            }
        }
        assert!(failures > 100, "{failures}");
    }

    /// The byte limit bounds what a step allocates, and the failure reports what it observed.
    #[test]
    fn the_byte_limit_bounds_a_step_and_reports_its_capacity() {
        let mut total = HtmlDecimalTotal::default();
        let addend = decimal("12345", "678");
        let failure = total.try_added_to(&addend, usize::MAX, 7).unwrap_err();
        assert!(matches!(
            failure.kind,
            super::super::HtmlNumericMaterializationError::Boundary(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 7,
                    required: 8
                }
            )
        ));
        // The integer places were reserved before the fraction places hit the limit.
        assert_eq!(failure.capacity_bytes, 5);
        let mut total = HtmlDecimalTotal::default();
        let step = total.try_added_to(&addend, usize::MAX, 8).unwrap();
        assert_eq!(step.capacity_bytes, 5 + 3);
        let repeat = total.try_added_to(&addend, usize::MAX, 0).unwrap();
        assert_eq!(repeat.capacity_bytes, 0);
    }
}
