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
//!
//! A carry that is still pending makes the total inexact, so the operations that need an exact
//! total refuse it with a typed error instead of walking it uncharged.

#[cfg(test)]
use super::count_digit_steps;
use super::{
    HtmlIntegerUnitStepError, HtmlNonNegativeDecimal, HtmlNonNegativeInteger,
    HtmlNumericMaterializationFailure, HtmlNumericMaterialized, ascii_digit_string, bounded_work,
    check_integer_actual_capacity, check_integer_digit_capacity, materialization_boundary,
    reserve_materialization,
};

/// Why a step of the total was not made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlDecimalTotalError {
    /// The step needs an exact total, but a carry is pending and only `try_carried` may run.
    CarryPending,
    /// `try_carried` has no carry to walk.
    NoCarryPending,
    /// The step hit a digit or allocation boundary.
    Materialization(HtmlNumericMaterializationFailure),
}

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
    /// Whether a carry still has to be walked, which only `try_carried` does. While it is pending,
    /// `try_added_to` and `try_finished` refuse with `CarryPending`.
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
    /// failure reports the capacity observed so far. It refuses a total with a carry pending.
    pub(crate) fn try_added_to(
        &mut self,
        addend: &HtmlNonNegativeDecimal,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<()>, HtmlDecimalTotalError> {
        if self.carry.is_some() {
            return Err(HtmlDecimalTotalError::CarryPending);
        }
        let mut observed = 0;
        self.add(addend, max_digits, max_bytes, &mut observed)
            .map(|()| HtmlNumericMaterialized {
                value: (),
                capacity_bytes: observed,
            })
            .map_err(|kind| boundary(kind, observed))
    }

    /// Walks a further chunk of the pending carry, and refuses a total with none pending.
    pub(crate) fn try_carried(
        &mut self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<()>, HtmlDecimalTotalError> {
        if self.carry.is_none() {
            return Err(HtmlDecimalTotalError::NoCarryPending);
        }
        let mut observed = 0;
        self.walk_carry(max_digits, max_bytes, &mut observed)
            .map(|()| HtmlNumericMaterialized {
                value: (),
                capacity_bytes: observed,
            })
            .map_err(|kind| boundary(kind, observed))
    }

    /// The canonical value, with no leading integer zeroes and no trailing fractional zeroes. It
    /// refuses a total with a carry pending.
    pub(crate) fn try_finished(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<HtmlNonNegativeDecimal>, HtmlDecimalTotalError> {
        if self.carry.is_some() {
            return Err(HtmlDecimalTotalError::CarryPending);
        }
        self.finish(max_digits, max_bytes)
            .map_err(HtmlDecimalTotalError::Materialization)
    }

    fn finish(
        &self,
        max_digits: usize,
        max_bytes: usize,
    ) -> Result<HtmlNumericMaterialized<HtmlNonNegativeDecimal>, HtmlNumericMaterializationFailure>
    {
        let mut observed = 0;
        let fraction_len = self
            .fraction
            .iter()
            .rposition(|&place| place != 0)
            .map_or(0, |last| last + 1);
        #[cfg(test)]
        count_digit_steps(self.fraction.len() - fraction_len + 1);
        let mut integer = reserve_materialization(
            self.integer.len().max(1),
            Some(max_digits),
            max_bytes,
            &mut observed,
        )?;
        #[cfg(test)]
        count_digit_steps(self.integer.len() + fraction_len);
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
        self.walk_carry(max_digits, max_bytes, observed)
    }

    /// Walks at most one chunk of the pending carry. It adds one place when the carry leaves the
    /// top, and then needs one digit more than the total had.
    fn walk_carry(
        &mut self,
        max_digits: usize,
        max_bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlIntegerUnitStepError> {
        let mut walked = 0;
        while let Some(position) = self.carry {
            if walked == CARRY_CHUNK {
                return Ok(());
            }
            walked += 1;
            #[cfg(test)]
            count_digit_steps(1);
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

fn boundary(kind: HtmlIntegerUnitStepError, observed: usize) -> HtmlDecimalTotalError {
    HtmlDecimalTotalError::Materialization(materialization_boundary(kind, observed))
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
        #[cfg(test)]
        count_digit_steps(1);
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
        let before = places.capacity();
        // Reallocating copies the places that exist.
        #[cfg(test)]
        count_digit_steps(places.len());
        let allocation = places.try_reserve_exact(target - places.len());
        // Even an allocation failure retains any backing it obtained, which is new only if the
        // capacity changed.
        if places.capacity() != before {
            *observed = observed
                .checked_add(places.capacity())
                .ok_or(HtmlIntegerUnitStepError::RepresentationExhausted)?;
        }
        allocation.map_err(|_| HtmlIntegerUnitStepError::AllocationFailed { requested: target })?;
        check_integer_actual_capacity(required, *observed, max_bytes)?;
    }
    #[cfg(test)]
    count_digit_steps(target - places.len());
    places.resize(target, 0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{HtmlNumericMaterializationError, digit_steps_during};
    use super::*;
    use crate::internal::values::DocumentString;

    /// The boundary a refused step reports, and the capacity it observed.
    fn boundary_of(error: HtmlDecimalTotalError) -> (HtmlIntegerUnitStepError, usize) {
        match error {
            HtmlDecimalTotalError::Materialization(HtmlNumericMaterializationFailure {
                kind: HtmlNumericMaterializationError::Boundary(boundary),
                capacity_bytes,
            }) => (boundary, capacity_bytes),
            other => panic!("expected a boundary failure, got {other:?}"),
        }
    }

    fn nines(count: usize) -> HtmlNonNegativeDecimal {
        decimal(&"9".repeat(count), "")
    }

    fn zeros_after_one(count: usize) -> String {
        format!("1{}", "0".repeat(count))
    }

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
                        // The observed capacity differs, because the total grows in place.
                        let HtmlDecimalTotalError::Materialization(controlled) = controlled else {
                            panic!("{controlled:?}");
                        };
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
        let error = total.try_added_to(&addend, usize::MAX, 7).unwrap_err();
        // The integer places were reserved before the fraction places hit the limit.
        assert_eq!(
            boundary_of(error),
            (
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 7,
                    required: 8
                },
                5
            )
        );
        let mut total = HtmlDecimalTotal::default();
        let step = total.try_added_to(&addend, usize::MAX, 8).unwrap();
        assert_eq!(step.capacity_bytes, 5 + 3);
        let repeat = total.try_added_to(&addend, usize::MAX, 0).unwrap();
        assert_eq!(repeat.capacity_bytes, 0);
    }

    /// While a carry is pending, only `try_carried` runs; with none pending it refuses.
    #[test]
    fn a_pending_carry_refuses_every_step_but_its_own() {
        let mut total = HtmlDecimalTotal::default();
        assert_eq!(
            total.try_carried(usize::MAX, usize::MAX).unwrap_err(),
            HtmlDecimalTotalError::NoCarryPending
        );
        add(&mut total, &nines(CARRY_CHUNK + 8), usize::MAX);
        // A carry out of the fraction reaches the nines too.
        add(&mut total, &decimal("0", "5"), usize::MAX);
        total
            .try_added_to(&decimal("0", "5"), usize::MAX, usize::MAX)
            .unwrap();
        assert!(total.carry_pending());
        assert_eq!(
            total
                .try_added_to(&decimal("1", ""), usize::MAX, usize::MAX)
                .unwrap_err(),
            HtmlDecimalTotalError::CarryPending
        );
        assert_eq!(
            total.try_finished(usize::MAX, usize::MAX).unwrap_err(),
            HtmlDecimalTotalError::CarryPending
        );
        total.try_carried(usize::MAX, usize::MAX).unwrap();
        assert!(!total.carry_pending());
        assert_eq!(
            total.try_carried(usize::MAX, usize::MAX).unwrap_err(),
            HtmlDecimalTotalError::NoCarryPending
        );
        assert_eq!(
            finished(&mut total).integer_part().decimal_digits(),
            zeros_after_one(CARRY_CHUNK + 8)
        );
    }

    /// The charged bound of each step covers the digit visits it makes.
    fn assert_charged_for_every_step(
        total: &mut HtmlDecimalTotal,
        addend: &HtmlNonNegativeDecimal,
    ) {
        let bound = total.added_work_bound(addend).unwrap();
        let (result, steps) =
            digit_steps_during(|| total.try_added_to(addend, usize::MAX, usize::MAX));
        result.unwrap();
        assert!(
            steps as usize <= bound,
            "an addition made {steps} visits for a charge of {bound}"
        );
        while total.carry_pending() {
            let bound = total.carried_work_bound().unwrap();
            let (result, steps) = digit_steps_during(|| total.try_carried(usize::MAX, usize::MAX));
            result.unwrap();
            assert!(
                steps as usize <= bound,
                "a carry made {steps} visits for a charge of {bound}"
            );
        }
    }

    fn assert_finish_charged(total: &HtmlDecimalTotal) -> HtmlNonNegativeDecimal {
        let bound = total.finished_work_bound().unwrap();
        let (result, steps) = digit_steps_during(|| total.try_finished(usize::MAX, usize::MAX));
        assert!(
            steps as usize <= bound,
            "a read-out made {steps} visits for a charge of {bound}"
        );
        result.unwrap().value
    }

    /// The growth inputs of the work-growth test, an ever longer series and random series: no
    /// step makes more digit visits than it was charged for.
    #[test]
    fn every_step_is_charged_for_the_digit_visits_it_makes() {
        for digits in [300_usize, 1_200] {
            let mut total = HtmlDecimalTotal::default();
            assert_charged_for_every_step(
                &mut total,
                &decimal(&format!("{}1", "9".repeat(digits)), ""),
            );
            for _ in 0..digits / 3 {
                assert_charged_for_every_step(&mut total, &decimal("1", ""));
            }
            let value = assert_finish_charged(&total);
            assert_eq!(value.integer_part().decimal_digits().len(), digits + 2);
        }
        // Each addend is longer than the total, so the places grow every time.
        let mut total = HtmlDecimalTotal::default();
        for length in 1..200 {
            let addend = decimal(&"7".repeat(length), &"3".repeat(length / 2));
            assert_charged_for_every_step(&mut total, &addend);
        }
        assert_finish_charged(&total);
        let mut random = Random(0x0bad_cafe_f00d_5eed);
        for _ in 0..300 {
            let mut total = HtmlDecimalTotal::default();
            for _ in 0..1 + random.below(30) {
                let long = random.below(6) == 0;
                let integer = random.digits(1, if long { 150 } else { 8 });
                let fraction = random.digits(0, if long { 100 } else { 6 });
                assert_charged_for_every_step(&mut total, &decimal(&integer, &fraction));
            }
            assert_finish_charged(&total);
        }
    }

    /// A carry out of the top needs one place more, which the byte limit and the digit limit each
    /// refuse before anything is allocated, in the first step and in a resumed step.
    #[test]
    fn growth_at_the_top_of_a_carry_fails_under_the_byte_and_digit_limits() {
        for (places, resumed) in [(5, false), (CARRY_CHUNK + 8, true)] {
            let new_top = places + 1;
            let build = || {
                let mut total = HtmlDecimalTotal::default();
                add(&mut total, &nines(places), usize::MAX);
                assert_eq!(total.integer.capacity(), places);
                total
            };
            // Each limit fails at the step that reaches the top, and nowhere earlier.
            for (max_digits, max_bytes, expected) in [
                (
                    usize::MAX,
                    places,
                    HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                        limit: places,
                        required: new_top,
                    },
                ),
                (
                    places,
                    usize::MAX,
                    HtmlIntegerUnitStepError::DigitLimitExceeded {
                        limit: places,
                        required: new_top,
                    },
                ),
            ] {
                let mut total = build();
                let mut step = total.try_added_to(&decimal("1", ""), max_digits, max_bytes);
                if resumed {
                    step.unwrap();
                    assert!(total.carry_pending());
                    step = total.try_carried(max_digits, max_bytes);
                }
                assert_eq!(boundary_of(step.unwrap_err()), (expected, 0));
            }
            // With room, the step allocates exactly the new place and observes its capacity.
            let mut total = build();
            let mut step = total
                .try_added_to(&decimal("1", ""), usize::MAX, new_top)
                .unwrap();
            if resumed {
                step = total.try_carried(usize::MAX, new_top).unwrap();
            }
            assert_eq!(step.capacity_bytes, new_top);
            assert_eq!(
                finished(&mut total).integer_part().decimal_digits(),
                zeros_after_one(places)
            );
        }
    }

    /// An allocation the allocator cannot make fails with the requested places and observes none.
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn an_allocation_failure_is_reported_without_observed_capacity() {
        let mut places = Vec::new();
        let mut observed = 0;
        let target = isize::MAX as usize;
        assert_eq!(
            grow(&mut places, target, usize::MAX, &mut observed),
            Err(HtmlIntegerUnitStepError::AllocationFailed { requested: target })
        );
        assert_eq!((places.len(), observed), (0, 0));
    }

    /// The read-out reserves the integer and then the fraction under the byte limit, and reports
    /// the allocation it had made when the second fails.
    #[test]
    fn the_read_out_fails_under_the_byte_limit_and_reports_its_capacity() {
        let mut total = HtmlDecimalTotal::default();
        add(&mut total, &decimal("123", "45"), usize::MAX);
        assert_eq!(
            boundary_of(total.try_finished(usize::MAX, 2).unwrap_err()),
            (
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 2,
                    required: 3
                },
                0
            )
        );
        assert_eq!(
            boundary_of(total.try_finished(usize::MAX, 4).unwrap_err()),
            (
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 4,
                    required: 5
                },
                3
            )
        );
        assert_eq!(total.try_finished(usize::MAX, 5).unwrap().capacity_bytes, 5);
    }

    /// A carry out of the fraction that grows the integer part is held to the digit limit when it
    /// is walked, and the read-out holds the grown integer part to it again.
    #[test]
    fn a_fraction_carry_into_a_new_top_place_meets_the_digit_limit() {
        let places = CARRY_CHUNK + 8;
        let build = |max_digits| {
            let mut total = HtmlDecimalTotal::default();
            add(&mut total, &nines(places), usize::MAX);
            add(&mut total, &decimal("0", "5"), usize::MAX);
            let mut step = total.try_added_to(&decimal("0", "5"), max_digits, usize::MAX);
            while step.is_ok() && total.carry_pending() {
                step = total.try_carried(max_digits, usize::MAX);
            }
            (total, step)
        };
        // The walk fails at the top, once, with the digits it needs.
        let (_, step) = build(places);
        assert_eq!(
            boundary_of(step.unwrap_err()),
            (
                HtmlIntegerUnitStepError::DigitLimitExceeded {
                    limit: places,
                    required: places + 1
                },
                0
            )
        );
        // The grown total exists under a larger limit, and the read-out refuses a smaller one.
        let (total, step) = build(places + 1);
        step.unwrap();
        assert_eq!(
            boundary_of(total.try_finished(places, usize::MAX).unwrap_err()),
            (
                HtmlIntegerUnitStepError::DigitLimitExceeded {
                    limit: places,
                    required: places + 1
                },
                0
            )
        );
        let value = total.try_finished(places + 1, usize::MAX).unwrap().value;
        assert_eq!(
            value.integer_part().decimal_digits(),
            zeros_after_one(places)
        );
        assert_eq!(value.fractional_digits(), None);
    }
}
