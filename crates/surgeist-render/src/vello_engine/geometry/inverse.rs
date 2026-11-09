// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// Modified for surgeist-render; see ../../../NOTICE-VELLO.md.
//! Required image/blur inverse arithmetic, matching shared/transform.wgsl.
//!
//! Inputs are the actual accepted native factors. No CPU inverse is encoded.
//! The finite consumer envelope is format-derived: RasterParameters reserves
//! fifteen u32 units for 16-pixel tile rounding, native config rounds to those
//! tiles, and fine.wgsl forms xy from u32 global IDs and bounded lane offsets.
//! The integer-to-float conversion can reach 2^32. The lane and image-center
//! additions may each round upward by one neighboring binary32 step of 512,
//! so the complete consumer domain reaches 2^32 + 1024.

use super::{QualifiedTransform, Reason, action};
use crate::Result;
use crate::accounting::{Account, RenderWork};

const MIN_NORMAL: f64 = f32::MIN_POSITIVE as f64;
const MAX_NATIVE: f64 = f32::MAX as f64;
const U: f64 = f32::EPSILON as f64;
const MIN_SUBNORMAL: f64 = f32::from_bits(1) as f64;
const DIVISION_ABSOLUTE: f64 = 2.5 * MIN_SUBNORMAL;
const DEVICE_MAX: f64 = 4_294_968_320.0;
const MAX_DIVISOR: f64 = (1_u128 << 126) as f64;

#[derive(Clone, Copy)]
struct Bound {
    lower: f64,
    upper: f64,
    // Only retained when every required operation is exactly representable.
    exact: Option<f32>,
}

impl Bound {
    const ZERO: Self = Self {
        lower: 0.0,
        upper: 0.0,
        exact: Some(0.0),
    };
    const ONE: Self = Self {
        lower: 1.0,
        upper: 1.0,
        exact: Some(1.0),
    };

    fn native(value: f32, account: &Account) -> Result<Self> {
        let value64 = action(account, || f64::from(value))?;
        Ok(Self {
            lower: value64,
            upper: value64,
            exact: Some(value),
        })
    }

    fn finite(self, account: &Account) -> Result<Self> {
        if !action(account, || self.lower.is_finite())?
            || !action(account, || self.upper.is_finite())?
            || action(account, || self.lower > self.upper)?
            || action(account, || self.lower < -MAX_NATIVE)?
            || action(account, || self.upper > MAX_NATIVE)?
        {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
        Ok(self)
    }

    fn zero(self, account: &Account) -> Result<bool> {
        action(account, || self.exact == Some(0.0))
    }

    fn normal_domain(self, account: &Account) -> Result<bool> {
        Ok(self.zero(account)?
            || action(account, || self.lower >= MIN_NORMAL)?
            || action(account, || self.upper <= -MIN_NORMAL)?)
    }

    fn normal(self, account: &Account) -> Result<Self> {
        self.finite(account)?;
        if !self.normal_domain(account)? {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
        Ok(self)
    }

    fn neg(self, account: &Account) -> Result<Self> {
        if let Some(value) = self.exact {
            return Self::native(action(account, || -value)?, account);
        }
        Ok(Self {
            lower: action(account, || -self.upper)?,
            upper: action(account, || -self.lower)?,
            exact: None,
        })
    }

    fn flushed(mut self, account: &Account) -> Result<Self> {
        if action(account, || self.lower < MIN_NORMAL)?
            && action(account, || self.upper > -MIN_NORMAL)?
        {
            self.lower = action(account, || self.lower.min(0.0))?;
            self.upper = action(account, || self.upper.max(0.0))?;
        }
        Ok(self)
    }

    fn rounded(
        lower: f64,
        upper: f64,
        relative: f64,
        absolute: f64,
        account: &Account,
    ) -> Result<Self> {
        // x +/- (relative*abs(x)+absolute) is monotone because relative<1.
        // Use each endpoint's own error; a global max-magnitude error would
        // invent a zero crossing for a broad strictly-positive device range.
        let low = action(account, || lower.abs())?;
        let high = action(account, || upper.abs())?;
        let low_error = action(account, || low * relative)?;
        let low_error = action(account, || low_error.next_up())?;
        let low_error = action(account, || low_error + absolute)?;
        let low_error = action(account, || low_error.next_up())?;
        let high_error = action(account, || high * relative)?;
        let high_error = action(account, || high_error.next_up())?;
        let high_error = action(account, || high_error + absolute)?;
        let high_error = action(account, || high_error.next_up())?;
        let lower = action(account, || lower - low_error)?;
        let upper = action(account, || upper + high_error)?;
        Self {
            lower: action(account, || lower.next_down())?,
            upper: action(account, || upper.next_up())?,
            exact: None,
        }
        .finite(account)?
        .flushed(account)
    }

    fn hull(self, other: Self, account: &Account) -> Result<Self> {
        if let (Some(left), Some(right)) = (self.exact, other.exact)
            && action(account, || left == right)?
        {
            return Ok(self);
        }
        Self {
            lower: action(account, || self.lower.min(other.lower))?,
            upper: action(account, || self.upper.max(other.upper))?,
            exact: None,
        }
        .finite(account)
    }

    fn mul(self, other: Self, account: &Account) -> Result<Self> {
        let left = self.flushed(account)?;
        let right = other.flushed(account)?;
        if left.zero(account)? || right.zero(account)? {
            return Ok(Self::ZERO);
        }
        if let (Some(_), Some(_)) = (left.exact, right.exact) {
            // Products of native binary32 values are exact in binary64.
            let product = action(account, || left.lower * right.lower)?;
            let native = action(account, || product as f32)?;
            let back = action(account, || f64::from(native))?;
            if action(account, || product == back)?
                && action(account, || native.is_finite())?
                && (action(account, || native == 0.0)? || action(account, || native.is_normal())?)
            {
                return Self::native(native, account);
            }
        }
        let values = [
            action(account, || left.lower * right.lower)?,
            action(account, || left.lower * right.upper)?,
            action(account, || left.upper * right.lower)?,
            action(account, || left.upper * right.upper)?,
        ];
        let [first, second, third, fourth] = values;
        let mut lower = first;
        let mut upper = first;
        for value in [second, third, fourth] {
            lower = action(account, || lower.min(value))?;
            upper = action(account, || upper.max(value))?;
        }
        let lower = action(account, || lower.next_down())?;
        let upper = action(account, || upper.next_up())?;
        let scaled = Self {
            lower,
            upper,
            exact: None,
        }
        .finite(account)?;
        // A power-of-two multiplication is exact when both the native input
        // and complete output stay normal. This retains the common reciprocal
        // symbol through exact scaling, without assuming exact division.
        if (power_of_two(left.exact, account)? && right.normal_domain(account)?
            || power_of_two(right.exact, account)? && left.normal_domain(account)?)
            && scaled.normal_domain(account)?
        {
            return Ok(scaled);
        }
        Self::rounded(lower, upper, U, MIN_SUBNORMAL, account)
    }

    fn add(self, other: Self, account: &Account) -> Result<Self> {
        let left = self.flushed(account)?;
        let right = other.flushed(account)?;
        if left.zero(account)? {
            return Ok(right);
        }
        if right.zero(account)? {
            return Ok(left);
        }
        if let (Some(_), Some(_)) = (left.exact, right.exact) {
            let sum = action(account, || left.lower + right.lower)?;
            let z = action(account, || sum - left.lower)?;
            let t = action(account, || sum - z)?;
            let a = action(account, || left.lower - t)?;
            let b = action(account, || right.lower - z)?;
            let residual = action(account, || a + b)?;
            let native = action(account, || sum as f32)?;
            let back = action(account, || f64::from(native))?;
            if action(account, || residual == 0.0)?
                && action(account, || sum == back)?
                && action(account, || native.is_finite())?
                && (action(account, || native == 0.0)? || action(account, || native.is_normal())?)
            {
                return Self::native(native, account);
            }
        }
        let lower = action(account, || left.lower + right.lower)?;
        let upper = action(account, || left.upper + right.upper)?;
        Self::rounded(
            action(account, || lower.next_down())?,
            action(account, || upper.next_up())?,
            U,
            MIN_SUBNORMAL,
            account,
        )
    }

    fn divide(self, denominator: Self, account: &Account) -> Result<Self> {
        denominator.normal(account)?;
        if denominator.zero(account)?
            || action(account, || denominator.lower < -MAX_DIVISOR)?
            || action(account, || denominator.upper > MAX_DIVISOR)?
        {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
        if self.zero(account)? {
            // Division is only 2.5ULP accurate even at zero; do not claim exact
            // zero for a reassociated 0/D operation. ULP(0)=2^-149.
            return Ok(Self {
                lower: -DIVISION_ABSOLUTE,
                upper: DIVISION_ABSOLUTE,
                exact: None,
            });
        }
        let numerator = self.flushed(account)?;
        let values = [
            action(account, || numerator.lower / denominator.lower)?,
            action(account, || numerator.lower / denominator.upper)?,
            action(account, || numerator.upper / denominator.lower)?,
            action(account, || numerator.upper / denominator.upper)?,
        ];
        let [first, second, third, fourth] = values;
        let mut lower = first;
        let mut upper = first;
        for value in [second, third, fourth] {
            lower = action(account, || lower.min(value))?;
            upper = action(account, || upper.max(value))?;
        }
        Self::rounded(
            action(account, || lower.next_down())?,
            action(account, || upper.next_up())?,
            2.5 * U,
            DIVISION_ABSOLUTE,
            account,
        )
    }

    fn reciprocal(self, account: &Account) -> Result<Self> {
        Self::ONE.divide(self, account)?.normal(account)
    }
}

fn power_of_two(value: Option<f32>, account: &Account) -> Result<bool> {
    let Some(value) = value else {
        return Ok(false);
    };
    let bits = action(account, || value.to_bits())?;
    let magnitude = action(account, || bits & 0x7fff_ffff)?;
    let fraction = action(account, || magnitude & 0x007f_ffff)?;
    let exponent = action(account, || magnitude & 0x7f80_0000)?;
    Ok(action(account, || fraction == 0)?
        && action(account, || exponent != 0)?
        && action(account, || exponent != 0x7f80_0000)?)
}

fn required(value: Bound, nonzero: bool, account: &Account) -> Result<Bound> {
    if nonzero {
        value.normal(account)
    } else {
        value.finite(account)
    }
}

fn component(
    reciprocal: Bound,
    determinant: Bound,
    cofactor: Bound,
    account: &Account,
) -> Result<Bound> {
    let nonzero = !cofactor.zero(account)?;
    let literal = required(reciprocal.mul(cofactor, account)?, nonzero, account)?;
    let quotient = required(cofactor.divide(determinant, account)?, nonzero, account)?;
    required(literal.hull(quotient, account)?, nonzero, account)
}

// All placements of the division in the three-factor numerator. Required
// nonzero products are guarded BEFORE a later division can amplify FTZ loss.
fn term(
    reciprocal: Bound,
    determinant: Bound,
    cofactor: Bound,
    value: Bound,
    account: &Account,
) -> Result<Bound> {
    let nonzero = !cofactor.zero(account)? && !value.zero(account)?;
    let multiply =
        |a: Bound, b: Bound| -> Result<Bound> { required(a.mul(b, account)?, nonzero, account) };
    let divide =
        |a: Bound| -> Result<Bound> { required(a.divide(determinant, account)?, nonzero, account) };
    let literal = multiply(multiply(reciprocal, cofactor)?, value)?;
    let middle = multiply(multiply(reciprocal, value)?, cofactor)?;
    let numerator = multiply(cofactor, value)?;
    let factored = multiply(reciprocal, numerator)?;
    let first_division = multiply(divide(cofactor)?, value)?;
    let second_division = multiply(divide(value)?, cofactor)?;
    let final_division = divide(numerator)?;
    literal
        .hull(middle, account)?
        .hull(factored, account)?
        .hull(first_division, account)?
        .hull(second_division, account)?
        .hull(final_division, account)
}

// Products of the exact native operands are exact in f64; TwoSum establishes
// whether their REAL sum is zero, without treating approximate cancellation as
// an exact native result or requiring two divisions to share rounding errors.
fn zero_numerator(cofactors: [Bound; 2], values: [Bound; 2], account: &Account) -> Result<bool> {
    let [a, c] = cofactors;
    let [x, y] = values;
    let p = action(account, || a.lower * x.lower)?;
    let q = action(account, || c.lower * y.lower)?;
    let sum = action(account, || p + q)?;
    let z = action(account, || sum - p)?;
    let t = action(account, || sum - z)?;
    let pe = action(account, || p - t)?;
    let qe = action(account, || q - z)?;
    let residual = action(account, || pe + qe)?;
    Ok(action(account, || sum == 0.0)? && action(account, || residual == 0.0)?)
}

struct Translation {
    value: Bound,
    terms: [Bound; 2],
    numerator_terms: [Bound; 2],
}

fn translation(
    reciprocal: Bound,
    determinant: Bound,
    cofactors: [Bound; 2],
    values: [Bound; 2],
    account: &Account,
) -> Result<Translation> {
    let mut terms = [Bound::ZERO; 2];
    let mut numerators = [Bound::ZERO; 2];
    for ((output, numerator), (cofactor, value)) in terms
        .iter_mut()
        .zip(&mut numerators)
        .zip(cofactors.into_iter().zip(values))
    {
        let nonzero = !cofactor.zero(account)? && !value.zero(account)?;
        *numerator = required(cofactor.mul(value, account)?, nonzero, account)?;
        *output = term(reciprocal, determinant, cofactor, value, account)?;
    }
    let nonzero = !zero_numerator(cofactors, values, account)?;
    let [first_term, second_term] = terms;
    let [first_numerator, second_numerator] = numerators;
    let literal = required(first_term.add(second_term, account)?, nonzero, account)?;
    let numerator = required(
        first_numerator.add(second_numerator, account)?,
        nonzero,
        account,
    )?;
    let divided = required(numerator.divide(determinant, account)?, nonzero, account)?;
    let factored = required(reciprocal.mul(numerator, account)?, nonzero, account)?;
    let value = required(
        literal.hull(divided, account)?.hull(factored, account)?,
        nonzero,
        account,
    )?;
    Ok(Translation {
        value,
        terms,
        numerator_terms: numerators,
    })
}

// The absolute-sum envelope covers every partial sum/permutation of expanded
// terms, not just the compiler's nearest-rounded grouping of inverse translation.
fn finite_sum(terms: &[Bound], account: &Account) -> Result<()> {
    let mut total = 0.0;
    for term in terms {
        let low = action(account, || term.lower.abs())?;
        let high = action(account, || term.upper.abs())?;
        let magnitude = action(account, || low.max(high))?;
        total = action(account, || total + magnitude)?;
        total = action(account, || total.next_up())?;
        let error = action(account, || total * U)?;
        let error = action(account, || error + MIN_SUBNORMAL)?;
        let error = action(account, || error.next_up())?;
        total = action(account, || total + error)?;
        total = action(account, || total.next_up())?;
        if !action(account, || total <= MAX_NATIVE)? {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
    }
    Ok(())
}

fn consumer_axis(
    reciprocal: Bound,
    determinant: Bound,
    cofactors: [Bound; 2],
    translation: Translation,
    account: &Account,
) -> Result<()> {
    // Every nonzero image/blur sample coordinate is >=0.5. Zero is handled as a
    // separate source-zero case so a required nonzero product cannot flush and
    // recover through a later division. The upper bound includes tile padding.
    let device = Bound {
        lower: 0.5,
        upper: DEVICE_MAX,
        exact: None,
    };
    let [cofactor_a, cofactor_c] = cofactors;
    let Translation {
        value,
        terms: [tx, ty],
        numerator_terms: [nx, ny],
    } = translation;
    let a = term(reciprocal, determinant, cofactor_a, device, account)?.hull(
        term(reciprocal, determinant, cofactor_a, Bound::ZERO, account)?,
        account,
    )?;
    let c = term(reciprocal, determinant, cofactor_c, device, account)?.hull(
        term(reciprocal, determinant, cofactor_c, Bound::ZERO, account)?,
        account,
    )?;
    finite_sum(&[a, c, tx, ty], account)?;
    a.add(c, account)?.add(value, account)?.finite(account)?;
    a.add(value, account)?.add(c, account)?.finite(account)?;
    a.add(c.add(value, account)?, account)?.finite(account)?;
    let a_num = cofactor_a.mul(device, account)?;
    let c_num = cofactor_c.mul(device, account)?;
    let a_num = a_num.hull(Bound::ZERO, account)?;
    let c_num = c_num.hull(Bound::ZERO, account)?;
    finite_sum(&[a_num, c_num, nx, ny], account)?;
    let numerator = a_num
        .add(c_num, account)?
        .add(nx.add(ny, account)?, account)?;
    numerator.divide(determinant, account)?.finite(account)?;
    reciprocal.mul(numerator, account)?.finite(account)?;
    Ok(())
}

/// Qualifies only transforms whose surviving image/blur shader requires inverse.
pub(crate) fn qualify_inverse(transform: &QualifiedTransform, account: &Account) -> Result<()> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d] = transform.native.matrix;
    let [x, y] = transform.native.translation;
    let a = Bound::native(a, account)?.normal(account)?;
    let b = Bound::native(b, account)?.normal(account)?;
    let c = Bound::native(c, account)?.normal(account)?;
    let d = Bound::native(d, account)?.normal(account)?;
    let x = Bound::native(x, account)?.normal(account)?;
    let y = Bound::native(y, account)?.normal(account)?;
    let determinant = a
        .mul(d, account)?
        .normal(account)?
        .add(b.mul(c, account)?.normal(account)?.neg(account)?, account)?
        .normal(account)?;
    let reciprocal = determinant.reciprocal(account)?;
    let cofactor_b = b.neg(account)?;
    let cofactor_c = c.neg(account)?;
    component(reciprocal, determinant, d, account)?;
    component(reciprocal, determinant, cofactor_b, account)?;
    component(reciprocal, determinant, cofactor_c, account)?;
    component(reciprocal, determinant, a, account)?;
    let values = [x.neg(account)?, y.neg(account)?];
    let tx = translation(reciprocal, determinant, [d, cofactor_c], values, account)?;
    let ty = translation(reciprocal, determinant, [cofactor_b, a], values, account)?;
    consumer_axis(reciprocal, determinant, [d, cofactor_c], tx, account)?;
    consumer_axis(reciprocal, determinant, [cofactor_b, a], ty, account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
    use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};
    use kurbo::Affine;

    fn qualified(source: Affine) -> QualifiedTransform {
        super::super::qualify_transform(source, &Account::for_test()).unwrap()
    }

    #[test]
    fn image_center_domain_covers_permitted_upward_rounding() {
        fn upward(exact: f64) -> f32 {
            let nearest = exact as f32;
            if f64::from(nearest) < exact {
                nearest.next_up()
            } else {
                nearest
            }
        }
        // The target guard reserves fifteen units for 16-pixel tile padding.
        // Its last four-pixel thread starts four units before the padded edge.
        let base = upward(f64::from(u32::MAX - 15 - 4));
        let last_lane = upward(f64::from(base) + 3.0);
        let image_center = upward(f64::from(last_lane) + 0.5);
        // Both additions may select the upper adjacent binary32 result.
        assert_eq!(image_center.to_bits(), 0x4f80_0002);
        assert!(
            f64::from(image_center) <= DEVICE_MAX,
            "permitted image center {image_center} exceeds the inverse proof domain"
        );
    }

    #[test]
    fn required_inverse_keeps_normal_identity_reflection_rotation_and_decimal_maps() {
        for source in [
            Affine::IDENTITY,
            Affine::new([-2.0, 0.0, 0.0, 3.0, -4.0, 2.0]),
            Affine::new([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]),
            Affine::new([1.2, 0.125, 0.25, 1.5, 1.0, 2.0]),
            Affine::new([1.0, 0.0, 1.0, 1.0, 1.0, 1.0]),
        ] {
            qualify_inverse(&qualified(source), &Account::for_test()).unwrap();
        }
    }

    #[test]
    fn singular_underflow_overflow_and_undefined_divisor_domains_refuse() {
        for source in [
            Affine::scale(0.0),
            Affine::scale(2.0_f64.powi(-80)),
            Affine::scale(2.0_f64.powi(80)),
            Affine::scale_non_uniform(2.0_f64.powi(64), 2.0_f64.powi(63)),
        ] {
            let error = qualify_inverse(&qualified(source), &Account::for_test()).unwrap_err();
            assert_eq!(
                error.geometry_realization_diagnostic(),
                Some(&Reason::InsufficientNativeCapacity)
            );
        }
    }

    #[test]
    fn zero_division_has_a_bounded_neighborhood_without_claiming_exact_zero() {
        let account = Account::for_test();
        let result = Bound::ZERO.divide(Bound::ONE, &account).unwrap();
        assert!(result.lower < 0.0 && result.upper > 0.0);
        assert!(result.exact.is_none());
        result
            .mul(
                Bound::native(DEVICE_MAX as f32, &account).unwrap(),
                &account,
            )
            .unwrap()
            .finite(&account)
            .unwrap();
    }

    #[test]
    fn reassociated_nonzero_product_cannot_flush_before_amplifying_division() {
        let account = Account::for_test();
        let determinant = Bound::native(f32::MIN_POSITIVE, &account).unwrap();
        let reciprocal = determinant.reciprocal(&account).unwrap();
        let cofactor = Bound::native(f32::MIN_POSITIVE, &account).unwrap();
        let half = Bound::native(0.5, &account).unwrap();
        // Literal reciprocal*cofactor*half is normal; cofactor*half/D
        // requires a subnormal nonzero product first and must refuse.
        reciprocal
            .mul(cofactor, &account)
            .unwrap()
            .normal(&account)
            .unwrap()
            .mul(half, &account)
            .unwrap()
            .normal(&account)
            .unwrap();
        let error = match term(reciprocal, determinant, cofactor, half, &account) {
            Ok(_) => panic!("a required reassociated product lost its normal domain"),
            Err(error) => error,
        };
        assert_eq!(
            error.geometry_realization_diagnostic(),
            Some(&Reason::InsufficientNativeCapacity)
        );
    }

    #[test]
    fn format_device_envelope_refuses_an_overflowing_consumer_even_with_finite_inverse() {
        let error = qualify_inverse(
            &qualified(Affine::scale_non_uniform(2.0_f64.powi(-100), 1.0)),
            &Account::for_test(),
        )
        .unwrap_err();
        assert_eq!(
            error.geometry_realization_diagnostic(),
            Some(&Reason::InsufficientNativeCapacity)
        );
    }

    #[test]
    fn native_six_word_capture_refusal_keeps_its_exact_prefix_and_fresh_retry() {
        let transform = qualified(Affine::IDENTITY);
        // Exactly six native coefficient captures precede the first widening.
        let mut limited = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 6)
                    .unwrap(),
            ),
        );
        let error = limited
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                qualify_inverse(&transform, account)
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(error.geometry_realization_diagnostic().is_none());
        let usage = limited.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 6);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        let mut retry = RenderAccountingContext::default();
        retry
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                qualify_inverse(&transform, account)
            })
            .unwrap();
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }
}
