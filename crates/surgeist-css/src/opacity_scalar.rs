//! Exact ordinary opacity scalars and lossless legacy-payload admission.

use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef,
    CssFiniteNumber, CssNumericTokenRef, CssOpacity, CssOpacityValue, CssValueOrigin,
    CssValueTokenRef,
};

/// The authored numeric token kind of an exact ordinary opacity scalar.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssOpacityScalarKind {
    Number,
    Percentage,
}

/// One exact finite decimal number or percentage token for authored opacity.
///
/// The lexical value remains authoritative even when a cached floating-point
/// approximation would overflow or underflow. Equality includes provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssOpacityScalar {
    component: Box<CssComponentValue>,
}

impl CssOpacityScalar {
    /// Checks the token kind without rounding or imposing a floating-point range.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssComponentValueError> {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(_) | CssValueTokenRef::Percentage(_)
            )
        ) {
            Ok(Self {
                component: Box::new(component),
            })
        } else {
            Err(CssComponentValueError::new(
                CssComponentValueErrorKind::InvalidToken,
                component.origin().clone(),
            ))
        }
    }

    /// Returns whether the authored scalar has a percentage suffix.
    #[must_use]
    pub fn kind(&self) -> CssOpacityScalarKind {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => {
                CssOpacityScalarKind::Number
            }
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                CssOpacityScalarKind::Percentage
            }
            _ => unreachable!("checked opacity scalar token"),
        }
    }

    /// Returns the complete numeric spelling, excluding a percentage suffix.
    #[must_use]
    pub fn numeric(&self) -> CssNumericTokenRef<'_> {
        match self.component.view() {
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(value) | CssValueTokenRef::Percentage(value),
            ) => value,
            _ => unreachable!("checked opacity scalar token"),
        }
    }

    /// Borrows the original checked component without replacing its provenance.
    #[must_use]
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }

    /// Returns the original parsed or explicitly programmatic token origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
}

pub(crate) fn admit_opacity_scalar(
    component: CssComponentValue,
) -> Result<CssOpacityValue, CssComponentValueError> {
    let scalar = CssOpacityScalar::try_from_component(component)?;
    if let Some(value) = exact_legacy_value(scalar.numeric().representation()) {
        let finite = CssFiniteNumber::try_new(value).expect("checked finite legacy value");
        return Ok(match scalar.kind() {
            CssOpacityScalarKind::Number => CssOpacity::try_new(value)
                .map(CssOpacityValue::Literal)
                .unwrap_or(CssOpacityValue::Number(finite)),
            CssOpacityScalarKind::Percentage => CssOpacityValue::Percentage(finite),
        });
    }
    Ok(CssOpacityValue::ExactScalar(scalar))
}

/// Normalized borrowed metadata for an already checked finite decimal token.
/// The coefficient remains in the original source regardless of its length.
pub(crate) struct LexicalDecimal<'a> {
    significant: &'a str,
    pub(crate) len: usize,
    pub(crate) exponent: Option<i128>,
    pub(crate) negative: bool,
    pub(crate) exponent_negative: bool,
    exponent_digits: &'a str,
    exponent_adjustment: i128,
}
impl<'a> LexicalDecimal<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        let negative = text.starts_with('-');
        let text = text.strip_prefix(['+', '-']).unwrap_or(text);
        let (mantissa, exponent_text) = text.split_once(['e', 'E']).unwrap_or((text, "0"));
        let exponent_negative = exponent_text.starts_with('-');
        let exponent_digits = exponent_text
            .strip_prefix(['+', '-'])
            .unwrap_or(exponent_text)
            .trim_start_matches('0');
        let exponent_digits = if exponent_digits.is_empty() {
            "0"
        } else {
            exponent_digits
        };
        let Some(first) = mantissa.bytes().position(|c| matches!(c, b'1'..=b'9')) else {
            return Self {
                significant: "",
                len: 0,
                exponent: Some(0),
                negative: false,
                exponent_negative: false,
                exponent_digits,
                exponent_adjustment: 0,
            };
        };
        let last = mantissa
            .bytes()
            .rposition(|c| matches!(c, b'1'..=b'9'))
            .expect("nonzero mantissa");
        let significant = &mantissa[first..=last];
        let len = significant.len() - usize::from(significant.contains('.'));
        let fractional = mantissa
            .find('.')
            .map_or(0, |point| mantissa.len() - point - 1);
        let trailing = mantissa[last + 1..].bytes().filter(|c| *c != b'.').count();
        let exponent_adjustment = -(fractional as i128) + trailing as i128;
        let exponent = exponent_text
            .parse::<i128>()
            .ok()
            .and_then(|e| e.checked_add(exponent_adjustment));
        Self {
            significant,
            len,
            exponent,
            negative,
            exponent_negative,
            exponent_digits,
            exponent_adjustment,
        }
    }
    pub(crate) fn digits(&self) -> impl Iterator<Item = u8> + '_ {
        self.significant
            .bytes()
            .filter(|c| *c != b'.')
            .map(|c| c - b'0')
    }
    pub(crate) fn in_percentage_range(&self) -> bool {
        if self.len == 0 {
            return true;
        }
        if self.negative {
            return false;
        }
        let Some(exponent) = self.exponent else {
            return self.exponent_negative;
        };
        let point = exponent.saturating_add(self.len as i128);
        point < 3 || (point == 3 && self.len == 1 && self.digits().next() == Some(1))
    }
}

const DECIMAL_LIMB_DIGITS: usize = 9;
const DECIMAL_LIMB_BASE: u64 = 1_000_000_000;

#[derive(Clone, Copy)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ExactFactor {
    pub(crate) numerator: u64,
    pub(crate) denominator: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BigCoefficient {
    // Little-endian base 10^9. Zero has no limbs.
    limbs: Vec<u32>,
}

impl BigCoefficient {
    fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    fn limb_count_for_digits(digits: usize) -> Option<usize> {
        digits
            .checked_add(DECIMAL_LIMB_DIGITS - 1)
            .map(|n| n / DECIMAL_LIMB_DIGITS)
    }

    fn from_lexical(
        value: &LexicalDecimal<'_>,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if value.len == 0 {
            return Ok(Self::zero());
        }
        let limb_count = Self::limb_count_for_digits(value.len).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        context.charge_projection(limb_count)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(limb_count).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let mut limb = 0u32;
        let mut place = 1u32;
        let mut count = 0;
        for byte in value.significant.bytes().rev().filter(|byte| *byte != b'.') {
            limb += u32::from(byte - b'0') * place;
            count += 1;
            if count == DECIMAL_LIMB_DIGITS {
                limbs.push(limb);
                limb = 0;
                place = 1;
                count = 0;
            } else {
                place *= 10;
            }
        }
        if count != 0 {
            limbs.push(limb);
        }
        debug_assert_eq!(limbs.len(), limb_count);
        Ok(Self { limbs })
    }

    fn from_u64(
        mut value: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if value == 0 {
            return Ok(Self::zero());
        }
        let limb_count = if value < DECIMAL_LIMB_BASE { 1 } else { 2 };
        context.charge_projection(limb_count)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(limb_count).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        while value != 0 {
            limbs.push((value % DECIMAL_LIMB_BASE) as u32);
            value /= DECIMAL_LIMB_BASE;
        }
        Ok(Self { limbs })
    }

    fn from_decimal_digit_slice(
        digits: &[u8],
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if digits.is_empty() {
            return Ok(Self::zero());
        }
        let limb_count = Self::limb_count_for_digits(digits.len()).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        context.charge_projection(limb_count)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(limb_count).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let mut end = digits.len();
        while end != 0 {
            let start = end.saturating_sub(DECIMAL_LIMB_DIGITS);
            let limb = digits[start..end]
                .iter()
                .fold(0u32, |value, digit| value * 10 + u32::from(*digit));
            limbs.push(limb);
            end = start;
        }
        Ok(Self { limbs })
    }

    fn normalize(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    fn decimal_len(&self) -> usize {
        let Some(high) = self.limbs.last() else {
            return 1;
        };
        (self.limbs.len() - 1) * DECIMAL_LIMB_DIGITS + high.ilog10() as usize + 1
    }

    fn decimal_string(
        &self,
        limit: usize,
    ) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        use std::fmt::Write as _;
        let length = self.decimal_len();
        if length > limit {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        let mut output = String::new();
        output.try_reserve_exact(length).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let mut limbs = self.limbs.iter().rev();
        if let Some(high) = limbs.next() {
            write!(&mut output, "{high}").expect("writing to String");
            for limb in limbs {
                write!(&mut output, "{limb:09}").expect("writing to String");
            }
        } else {
            output.push('0');
        }
        debug_assert_eq!(output.len(), length);
        Ok(output)
    }

    fn mul_small(
        &self,
        factor: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.is_zero() || factor == 0 {
            return Ok(Self::zero());
        }
        let mut carry = 0u128;
        for &limb in &self.limbs {
            carry += u128::from(limb) * u128::from(factor);
            carry /= u128::from(DECIMAL_LIMB_BASE);
        }
        let mut extra = 0usize;
        while carry != 0 {
            extra += 1;
            carry /= u128::from(DECIMAL_LIMB_BASE);
        }
        let length = self.limbs.len().checked_add(extra).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        context.charge_projection(length)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(length).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let mut carry = 0u128;
        for &limb in &self.limbs {
            let product = u128::from(limb) * u128::from(factor) + carry;
            limbs.push((product % u128::from(DECIMAL_LIMB_BASE)) as u32);
            carry = product / u128::from(DECIMAL_LIMB_BASE);
        }
        while carry != 0 {
            limbs.push((carry % u128::from(DECIMAL_LIMB_BASE)) as u32);
            carry /= u128::from(DECIMAL_LIMB_BASE);
        }
        Ok(Self { limbs })
    }

    fn multiply(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.is_zero() || other.is_zero() {
            return Ok(Self::zero());
        }
        let capacity = self
            .limbs
            .len()
            .checked_add(other.limbs.len())
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        context.charge_projection(capacity)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(capacity).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        limbs.resize(capacity, 0u32);
        for (left_index, &left) in self.limbs.iter().enumerate() {
            let mut carry = 0u128;
            for (right_index, &right) in other.limbs.iter().enumerate() {
                let index = left_index + right_index;
                let value = u128::from(limbs[index]) + u128::from(left) * u128::from(right) + carry;
                limbs[index] = (value % u128::from(DECIMAL_LIMB_BASE)) as u32;
                carry = value / u128::from(DECIMAL_LIMB_BASE);
            }
            let mut index = left_index + other.limbs.len();
            while carry != 0 {
                let value = u128::from(limbs[index]) + carry;
                limbs[index] = (value % u128::from(DECIMAL_LIMB_BASE)) as u32;
                carry = value / u128::from(DECIMAL_LIMB_BASE);
                index += 1;
            }
        }
        let mut result = Self { limbs };
        result.normalize();
        Ok(result)
    }

    fn mul_pow10(
        &self,
        power: usize,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.is_zero() {
            return Ok(Self::zero());
        }
        let whole = power / DECIMAL_LIMB_DIGITS;
        let remainder = power % DECIMAL_LIMB_DIGITS;
        let factor = 10u64.pow(remainder as u32);
        let multiplied = if factor == 1 {
            self.clone_with_budget(context)?
        } else {
            self.mul_small(factor, context)?
        };
        let length = multiplied.limbs.len().checked_add(whole).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        context.charge_projection(length)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(length).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        limbs.resize(whole, 0);
        limbs.extend(multiplied.limbs);
        Ok(Self { limbs })
    }

    fn clone_with_budget(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        context.charge_projection(self.limbs.len())?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(self.limbs.len()).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        limbs.extend_from_slice(&self.limbs);
        Ok(Self { limbs })
    }

    fn div_rem_small(
        &self,
        denominator: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<(Self, u64), crate::CssSpecifiedValueSerializationError> {
        debug_assert_ne!(denominator, 0);
        if self.is_zero() {
            return Ok((Self::zero(), 0));
        }
        context.charge_projection(self.limbs.len())?;
        let mut quotient = Vec::new();
        quotient.try_reserve_exact(self.limbs.len()).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        quotient.resize(self.limbs.len(), 0);
        let mut remainder = 0u128;
        for index in (0..self.limbs.len()).rev() {
            let value = remainder * u128::from(DECIMAL_LIMB_BASE) + u128::from(self.limbs[index]);
            quotient[index] = (value / u128::from(denominator)) as u32;
            remainder = value % u128::from(denominator);
        }
        let mut quotient = Self { limbs: quotient };
        quotient.normalize();
        Ok((quotient, remainder as u64))
    }

    fn div_rem_pow10(
        &self,
        power: usize,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<(Self, Self), crate::CssSpecifiedValueSerializationError> {
        if self.is_zero() || power == 0 {
            return Ok((self.clone_with_budget(context)?, Self::zero()));
        }
        let whole = power / DECIMAL_LIMB_DIGITS;
        let remainder_digits = power % DECIMAL_LIMB_DIGITS;
        if whole >= self.limbs.len() {
            return Ok((Self::zero(), self.clone_with_budget(context)?));
        }
        let upper_len = self.limbs.len() - whole;
        context.charge_projection(upper_len)?;
        let mut upper = Vec::new();
        upper.try_reserve_exact(upper_len).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        upper.extend_from_slice(&self.limbs[whole..]);
        let upper = Self { limbs: upper };
        let (quotient, upper_remainder) = if remainder_digits == 0 {
            (upper, 0)
        } else {
            upper.div_rem_small(10u64.pow(remainder_digits as u32), context)?
        };
        let lower_len = whole + usize::from(upper_remainder != 0);
        context.charge_projection(lower_len)?;
        let mut lower = Vec::new();
        lower.try_reserve_exact(lower_len).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        lower.extend_from_slice(&self.limbs[..whole]);
        if upper_remainder != 0 {
            lower.push(upper_remainder as u32);
        }
        let mut lower = Self { limbs: lower };
        lower.normalize();
        Ok((quotient, lower))
    }

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.limbs.len().cmp(&other.limbs.len()) {
            std::cmp::Ordering::Equal => self.limbs.iter().rev().cmp(other.limbs.iter().rev()),
            ordering => ordering,
        }
    }

    fn mod_small(&self, modulus: u64) -> u64 {
        self.limbs.iter().rev().fold(0u64, |remainder, limb| {
            ((u128::from(remainder) * u128::from(DECIMAL_LIMB_BASE) + u128::from(*limb))
                % u128::from(modulus)) as u64
        })
    }

    fn add(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        let capacity = self
            .limbs
            .len()
            .max(other.limbs.len())
            .checked_add(1)
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        context.charge_projection(capacity)?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(capacity).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let mut carry = 0u64;
        for index in 0..capacity - 1 {
            let sum = self.limbs.get(index).copied().map_or(0, u64::from)
                + other.limbs.get(index).copied().map_or(0, u64::from)
                + carry;
            limbs.push((sum % DECIMAL_LIMB_BASE) as u32);
            carry = sum / DECIMAL_LIMB_BASE;
        }
        limbs.push(carry as u32);
        let mut result = Self { limbs };
        result.normalize();
        Ok(result)
    }

    fn subtract(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        debug_assert!(self.cmp(other) != std::cmp::Ordering::Less);
        context.charge_projection(self.limbs.len())?;
        let mut limbs = Vec::new();
        limbs.try_reserve_exact(self.limbs.len()).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let mut borrow = 0i64;
        for index in 0..self.limbs.len() {
            let mut value = i64::from(self.limbs[index])
                - other.limbs.get(index).copied().map_or(0, i64::from)
                - borrow;
            if value < 0 {
                value += DECIMAL_LIMB_BASE as i64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            limbs.push(value as u32);
        }
        debug_assert_eq!(borrow, 0);
        let mut result = Self { limbs };
        result.normalize();
        Ok(result)
    }

    fn increment(
        &mut self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<(), crate::CssSpecifiedValueSerializationError> {
        for limb in &mut self.limbs {
            if *limb + 1 < DECIMAL_LIMB_BASE as u32 {
                *limb += 1;
                return Ok(());
            }
            *limb = 0;
        }
        context.charge_projection(1)?;
        self.limbs.try_reserve(1).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        self.limbs.push(1);
        Ok(())
    }

    fn add_small(
        &mut self,
        mut value: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<(), crate::CssSpecifiedValueSerializationError> {
        let mut index = 0usize;
        while value != 0 {
            if index == self.limbs.len() {
                context.charge_projection(1)?;
                self.limbs.try_reserve(1).map_err(|_| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
                self.limbs.push(0);
            }
            let sum = u64::from(self.limbs[index])
                .checked_add(value)
                .ok_or_else(|| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
            self.limbs[index] = (sum % DECIMAL_LIMB_BASE) as u32;
            value = sum / DECIMAL_LIMB_BASE;
            index += 1;
        }
        Ok(())
    }

    fn strip_decimal_zeros(&mut self) -> i128 {
        let mut removed = 0i128;
        while self
            .limbs
            .first()
            .is_some_and(|limb| limb.is_multiple_of(10))
        {
            let mut carry = 0u64;
            for limb in self.limbs.iter_mut().rev() {
                let value = carry * DECIMAL_LIMB_BASE + u64::from(*limb);
                *limb = (value / 10) as u32;
                carry = value % 10;
            }
            self.normalize();
            removed += 1;
        }
        removed
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct ExactRational {
    coefficient: BigCoefficient,
    exponent: i128,
    denominator: u64,
    negative: bool,
    unbounded_tiny: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
impl ExactRational {
    /// Returns a bounded binary approximation without first forcing the exact
    /// value into binary64's exponent range. The pair represents
    /// `coefficient * 10^exponent`, with a zero coefficient for exact zero.
    ///
    /// Color conversion uses this only after exact lexical scaling and, for
    /// hues, exact modular reduction. Keeping the decimal exponent separate
    /// prevents large finite authored values from becoming infinities merely
    /// because binary64 cannot hold their intermediate magnitude.
    pub(crate) fn scaled_binary64(
        &self,
    ) -> Result<(f64, i128), crate::CssSpecifiedValueSerializationError> {
        if self.coefficient.is_zero() || self.unbounded_tiny {
            return Ok((0.0, 0));
        }

        let high_index = self.coefficient.limbs.len() - 1;
        let mut coefficient = f64::from(self.coefficient.limbs[high_index]);
        if high_index != 0 {
            coefficient +=
                f64::from(self.coefficient.limbs[high_index - 1]) / DECIMAL_LIMB_BASE as f64;
        }
        coefficient /= self.denominator as f64;
        let decimal_shift = i128::try_from(high_index)
            .ok()
            .and_then(|index| index.checked_mul(DECIMAL_LIMB_DIGITS as i128))
            .and_then(|shift| self.exponent.checked_add(shift))
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        let normalization = coefficient.log10().floor() as i128;
        let mut coefficient = coefficient / 10f64.powi(normalization as i32);
        if self.negative {
            coefficient = -coefficient;
        }
        Ok((
            coefficient,
            decimal_shift.checked_add(normalization).ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?,
        ))
    }

    pub(crate) fn compare_integer(
        &self,
        value: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<std::cmp::Ordering, crate::CssSpecifiedValueSerializationError> {
        let other = Self::integer(value, context)?;
        self.compare(&other, context)
    }

    pub(crate) fn max_zero(
        self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.negative || self.is_zero() {
            Self::integer(0, context)
        } else {
            Ok(self)
        }
    }

    pub(crate) fn add_signed(
        self,
        other: Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.unbounded_tiny || other.unbounded_tiny {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        let left = self;
        let right = other;
        let exponent = left.exponent.min(right.exponent);
        let denominator_gcd = greatest_common_divisor(left.denominator, right.denominator);
        let left_factor = right.denominator / denominator_gcd;
        let right_factor = left.denominator / denominator_gcd;
        let denominator = left.denominator.checked_mul(left_factor).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let left_coefficient = left.scaled_coefficient(exponent, left_factor, context)?;
        let right_coefficient = right.scaled_coefficient(exponent, right_factor, context)?;
        let (coefficient, negative) = if left.negative == right.negative {
            (
                left_coefficient.add(&right_coefficient, context)?,
                left.negative,
            )
        } else {
            match left_coefficient.cmp(&right_coefficient) {
                std::cmp::Ordering::Greater => (
                    left_coefficient.subtract(&right_coefficient, context)?,
                    left.negative,
                ),
                std::cmp::Ordering::Less => (
                    right_coefficient.subtract(&left_coefficient, context)?,
                    right.negative,
                ),
                std::cmp::Ordering::Equal => (BigCoefficient::zero(), false),
            }
        };
        Self {
            coefficient,
            exponent,
            denominator,
            negative,
            unbounded_tiny: false,
        }
        .reduce_fraction(context)
    }

    pub(crate) fn subtract_signed(
        self,
        mut other: Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if !other.is_zero() {
            other.negative = !other.negative;
        }
        self.add_signed(other, context)
    }

    pub(crate) fn multiply_signed(
        self,
        other: Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.unbounded_tiny || other.unbounded_tiny {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        Self {
            coefficient: self.coefficient.multiply(&other.coefficient, context)?,
            exponent: self.exponent.checked_add(other.exponent).ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?,
            denominator,
            negative: self.negative != other.negative,
            unbounded_tiny: false,
        }
        .reduce_fraction(context)
    }

    pub(crate) fn scale_ratio(
        mut self,
        numerator: u64,
        denominator: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if denominator == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        self.coefficient = self.coefficient.mul_small(numerator, context)?;
        self.denominator = self.denominator.checked_mul(denominator).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        self.reduce_fraction(context)
    }

    pub(crate) fn rounded_positive_ratio(
        &self,
        denominator: &Self,
        unit_scale: u64,
        places: u32,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<u64, crate::CssSpecifiedValueSerializationError> {
        debug_assert!(!self.negative && !denominator.negative);
        debug_assert!(!denominator.is_zero());
        if self.unbounded_tiny || denominator.unbounded_tiny {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        let mut numerator = self
            .coefficient
            .mul_small(denominator.denominator, context)?;
        let mut divisor = denominator
            .coefficient
            .mul_small(self.denominator, context)?;
        match self.exponent.cmp(&denominator.exponent) {
            std::cmp::Ordering::Greater => {
                let shift =
                    usize::try_from(self.exponent - denominator.exponent).map_err(|_| {
                        crate::CssSpecifiedValueSerializationError::new(
                            crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                        )
                    })?;
                numerator = numerator.mul_pow10(shift, context)?;
            }
            std::cmp::Ordering::Less => {
                let shift =
                    usize::try_from(denominator.exponent - self.exponent).map_err(|_| {
                        crate::CssSpecifiedValueSerializationError::new(
                            crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                        )
                    })?;
                divisor = divisor.mul_pow10(shift, context)?;
            }
            std::cmp::Ordering::Equal => {}
        }
        let decimal_scale = 10u64.checked_pow(places).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let maximum = unit_scale.checked_mul(decimal_scale).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let scaled = numerator.mul_small(maximum, context)?;
        let mut low = 0u64;
        let mut high = maximum;
        while low < high {
            let midpoint = low + (high - low).div_ceil(2);
            if divisor.mul_small(midpoint, context)?.cmp(&scaled).is_le() {
                low = midpoint;
            } else {
                high = midpoint - 1;
            }
        }
        if low == maximum {
            return Ok(low);
        }
        let doubled = scaled.mul_small(2, context)?;
        let threshold = divisor.mul_small(
            low.checked_mul(2)
                .and_then(|value| value.checked_add(1))
                .ok_or_else(|| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?,
            context,
        )?;
        Ok(if doubled.cmp(&threshold).is_ge() {
            low + 1
        } else {
            low
        })
    }

    fn reduce_fraction(
        mut self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.coefficient.is_zero() {
            self.negative = false;
            self.exponent = 0;
            self.denominator = 1;
            return Ok(self);
        }
        let common = greatest_common_divisor(
            self.coefficient.mod_small(self.denominator),
            self.denominator,
        );
        if common > 1 {
            let (coefficient, remainder) = self.coefficient.div_rem_small(common, context)?;
            debug_assert_eq!(remainder, 0);
            self.coefficient = coefficient;
            self.denominator /= common;
        }
        Ok(self)
    }

    pub(crate) fn integer(
        value: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        Ok(Self {
            coefficient: BigCoefficient::from_u64(value, context)?,
            exponent: 0,
            denominator: 1,
            negative: false,
            unbounded_tiny: false,
        })
    }

    fn scaled_coefficient(
        &self,
        target_exponent: i128,
        denominator_factor: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<BigCoefficient, crate::CssSpecifiedValueSerializationError> {
        if self.unbounded_tiny {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        let shift = self.exponent.checked_sub(target_exponent).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let shift = usize::try_from(shift).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        self.coefficient
            .mul_pow10(shift, context)?
            .mul_small(denominator_factor, context)
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.coefficient.is_zero()
    }

    pub(crate) fn clone_with_budget(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        Ok(Self {
            coefficient: self.coefficient.clone_with_budget(context)?,
            exponent: self.exponent,
            denominator: self.denominator,
            negative: self.negative,
            unbounded_tiny: self.unbounded_tiny,
        })
    }

    pub(crate) fn compare(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<std::cmp::Ordering, crate::CssSpecifiedValueSerializationError> {
        match (self.is_zero(), other.is_zero()) {
            (true, true) => return Ok(std::cmp::Ordering::Equal),
            (true, false) => {
                return Ok(if other.negative {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                });
            }
            (false, true) => {
                return Ok(if self.negative {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                });
            }
            (false, false) => {}
        }
        if self.negative != other.negative {
            return Ok(if self.negative {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            });
        }
        if self.unbounded_tiny || other.unbounded_tiny {
            return match (self.unbounded_tiny, other.unbounded_tiny) {
                (true, true) if self == other => Ok(std::cmp::Ordering::Equal),
                (true, false) if other.is_zero() => Ok(std::cmp::Ordering::Greater),
                (false, true) if self.is_zero() => Ok(std::cmp::Ordering::Less),
                (true, false) => Ok(std::cmp::Ordering::Less),
                (false, true) => Ok(std::cmp::Ordering::Greater),
                _ => Err(crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
                )),
            };
        }
        let left = self.coefficient.mul_small(other.denominator, context)?;
        let right = other.coefficient.mul_small(self.denominator, context)?;
        let left_magnitude = i128::try_from(left.decimal_len())
            .ok()
            .and_then(|digits| digits.checked_add(self.exponent))
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        let right_magnitude = i128::try_from(right.decimal_len())
            .ok()
            .and_then(|digits| digits.checked_add(other.exponent))
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        let ordering = match left_magnitude.cmp(&right_magnitude) {
            std::cmp::Ordering::Equal => {
                let exponent = self.exponent.min(other.exponent);
                let left_shift = usize::try_from(self.exponent - exponent).map_err(|_| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
                let right_shift = usize::try_from(other.exponent - exponent).map_err(|_| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
                left.mul_pow10(left_shift, context)?
                    .cmp(&right.mul_pow10(right_shift, context)?)
            }
            ordering => ordering,
        };
        Ok(if self.negative {
            ordering.reverse()
        } else {
            ordering
        })
    }

    fn compare_with_one(
        &self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<std::cmp::Ordering, crate::CssSpecifiedValueSerializationError> {
        if self.negative {
            return Ok(std::cmp::Ordering::Less);
        }
        if self.is_zero() {
            return Ok(std::cmp::Ordering::Less);
        }
        if self.unbounded_tiny {
            return Ok(std::cmp::Ordering::Less);
        }
        let coefficient_digits = self.coefficient.decimal_len();
        let denominator_digits = self.denominator.ilog10() as usize + 1;
        if self.exponent >= 0 {
            let Ok(shift) = usize::try_from(self.exponent) else {
                return Ok(std::cmp::Ordering::Greater);
            };
            let Some(left_digits) = coefficient_digits.checked_add(shift) else {
                return Ok(std::cmp::Ordering::Greater);
            };
            match left_digits.cmp(&denominator_digits) {
                std::cmp::Ordering::Equal => {
                    let left = self.coefficient.mul_pow10(shift, context)?;
                    let right = BigCoefficient::from_u64(self.denominator, context)?;
                    Ok(left.cmp(&right))
                }
                ordering => Ok(ordering),
            }
        } else {
            let Ok(power) = usize::try_from(self.exponent.unsigned_abs()) else {
                return Ok(std::cmp::Ordering::Less);
            };
            let Some(right_digits) = denominator_digits.checked_add(power) else {
                return Ok(std::cmp::Ordering::Less);
            };
            match coefficient_digits.cmp(&right_digits) {
                std::cmp::Ordering::Equal => {
                    let right = BigCoefficient::from_u64(self.denominator, context)?
                        .mul_pow10(power, context)?;
                    Ok(self.coefficient.cmp(&right))
                }
                ordering => Ok(ordering),
            }
        }
    }

    pub(crate) fn equals_ratio(
        &self,
        numerator: u64,
        denominator: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<bool, crate::CssSpecifiedValueSerializationError> {
        if denominator == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        if numerator == denominator {
            return self
                .compare_with_one(context)
                .map(|ordering| ordering == std::cmp::Ordering::Equal);
        }
        if numerator == 0 {
            return Ok(self.is_zero());
        }
        let other = Self {
            coefficient: BigCoefficient::from_u64(numerator, context)?,
            exponent: 0,
            denominator,
            negative: false,
            unbounded_tiny: false,
        };
        self.compare(&other, context)
            .map(|ordering| ordering == std::cmp::Ordering::Equal)
    }

    pub(crate) fn clamp_unit(
        self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.negative || self.is_zero() {
            return Self::integer(0, context);
        }
        if self.compare_with_one(context)? == std::cmp::Ordering::Greater {
            Self::integer(1, context)
        } else {
            Ok(self)
        }
    }

    pub(crate) fn add_nonnegative(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        debug_assert!(!self.negative && !other.negative);
        let exponent = self.exponent.min(other.exponent);
        let denominator = self
            .denominator
            .checked_mul(other.denominator)
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        let left = self.scaled_coefficient(exponent, other.denominator, context)?;
        let right = other.scaled_coefficient(exponent, self.denominator, context)?;
        Ok(Self {
            coefficient: left.add(&right, context)?,
            exponent,
            denominator,
            negative: false,
            unbounded_tiny: false,
        })
    }

    pub(crate) fn min_integer(
        self,
        maximum: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        debug_assert!(!self.negative);
        let maximum = Self::integer(maximum, context)?;
        if self.compare(&maximum, context)? == std::cmp::Ordering::Greater {
            Ok(maximum)
        } else {
            Ok(self)
        }
    }

    pub(crate) fn subtract_from_integer(
        &self,
        minuend: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        debug_assert!(!self.negative);
        let upper = Self::integer(minuend, context)?;
        let exponent = self.exponent.min(0);
        let denominator = self.denominator;
        let left = upper.scaled_coefficient(exponent, denominator, context)?;
        let right = self.scaled_coefficient(exponent, 1, context)?;
        Ok(Self {
            coefficient: left.subtract(&right, context)?,
            exponent,
            denominator,
            negative: false,
            unbounded_tiny: false,
        })
    }

    pub(crate) fn divide_by(
        mut self,
        divisor: usize,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.unbounded_tiny {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        let divisor = u64::try_from(divisor).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        if divisor == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        self.denominator = self.denominator.checked_mul(divisor).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        Ok(self)
    }

    fn into_terminating(
        mut self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if self.unbounded_tiny {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        if self.coefficient.is_zero() {
            self.negative = false;
            self.exponent = 0;
            self.denominator = 1;
            return Ok(self);
        }
        let common = greatest_common_divisor(
            self.coefficient.mod_small(self.denominator),
            self.denominator,
        );
        if common > 1 {
            let (coefficient, remainder) = self.coefficient.div_rem_small(common, context)?;
            debug_assert_eq!(remainder, 0);
            self.coefficient = coefficient;
            self.denominator /= common;
        }
        let mut denominator = self.denominator;
        let mut twos = 0usize;
        let mut fives = 0usize;
        while denominator.is_multiple_of(2) {
            denominator /= 2;
            twos += 1;
        }
        while denominator.is_multiple_of(5) {
            denominator /= 5;
            fives += 1;
        }
        if denominator != 1 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        for _ in fives..twos {
            self.coefficient = self.coefficient.mul_small(5, context)?;
        }
        for _ in twos..fives {
            self.coefficient = self.coefficient.mul_small(2, context)?;
        }
        self.exponent = self
            .exponent
            .checked_sub(twos.max(fives) as i128)
            .and_then(|value| value.checked_add(self.coefficient.strip_decimal_zeros()))
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
                )
            })?;
        self.denominator = 1;
        Ok(self)
    }

    pub(crate) fn modulo(
        self,
        modulus: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if modulus == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        let mut value = self.into_terminating(context)?;
        if value.coefficient.is_zero() {
            return Ok(value);
        }
        if value.exponent >= 0 {
            let exponent = u128::try_from(value.exponent).map_err(|_| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            let residue = (u128::from(value.coefficient.mod_small(modulus))
                * u128::from(pow_mod(10, exponent, modulus))
                % u128::from(modulus)) as u64;
            let residue = if value.negative && residue != 0 {
                modulus - residue
            } else {
                residue
            };
            return Self::integer(residue, context);
        }
        let power = usize::try_from(value.exponent.checked_neg().ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            )
        })?)
        .map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            )
        })?;
        let (integer, fraction) = value.coefficient.div_rem_pow10(power, context)?;
        let integer_remainder = integer.mod_small(modulus);
        let scale = BigCoefficient::from_u64(1, context)?.mul_pow10(power, context)?;
        let coefficient = if !value.negative {
            BigCoefficient::from_u64(integer_remainder, context)?
                .mul_pow10(power, context)?
                .add(&fraction, context)?
        } else if fraction.is_zero() {
            return Self::integer(
                if integer_remainder == 0 {
                    0
                } else {
                    modulus - integer_remainder
                },
                context,
            );
        } else {
            let whole = modulus - integer_remainder - 1;
            BigCoefficient::from_u64(whole, context)?
                .mul_pow10(power, context)?
                .add(&scale.subtract(&fraction, context)?, context)?
        };
        value.coefficient = coefficient;
        value.negative = false;
        value.denominator = 1;
        Ok(value)
    }

    pub(crate) fn from_lexical_factor(
        text: &str,
        factor: ExactFactor,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if factor.denominator == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        let lexical = LexicalDecimal::new(text);
        let exponent = lexical.exponent.ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            )
        })?;
        let coefficient = BigCoefficient::from_lexical(&lexical, context)?
            .mul_small(factor.numerator, context)?;
        Ok(Self {
            negative: lexical.negative && !coefficient.is_zero(),
            coefficient,
            exponent,
            denominator: factor.denominator,
            unbounded_tiny: false,
        })
    }

    pub(crate) fn from_lexical_factor_clamped_unit(
        text: &str,
        factor: ExactFactor,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if factor.denominator == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        let lexical = LexicalDecimal::new(text);
        if lexical.exponent.is_some() {
            return Self::from_lexical_factor(text, factor, context)?.clamp_unit(context);
        }
        let coefficient = BigCoefficient::from_lexical(&lexical, context)?
            .mul_small(factor.numerator, context)?;
        if coefficient.is_zero() || lexical.negative {
            return Self::integer(0, context);
        }
        if !lexical.exponent_negative {
            return Self::integer(1, context);
        }
        Ok(Self {
            coefficient,
            exponent: 0,
            denominator: factor.denominator,
            negative: false,
            unbounded_tiny: true,
        })
    }

    pub(crate) fn from_binary32_factor(
        value: f32,
        factor: ExactFactor,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if !value.is_finite() || factor.denominator == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        let decimal = Decimal::binary32(value);
        let coefficient =
            BigCoefficient::from_decimal_digit_slice(&decimal.digits[..decimal.len], context)?
                .mul_small(factor.numerator, context)?;
        Ok(Self {
            coefficient,
            exponent: decimal.exponent,
            denominator: factor.denominator,
            negative: decimal.negative && decimal.len != 0,
            unbounded_tiny: false,
        })
    }

    pub(crate) fn from_lexical_factor_modulo(
        text: &str,
        factor: ExactFactor,
        modulus: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<Self, crate::CssSpecifiedValueSerializationError> {
        if factor.denominator == 0 || modulus == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        let lexical = LexicalDecimal::new(text);
        if lexical.len == 0 {
            return Ok(Self {
                coefficient: BigCoefficient::zero(),
                exponent: 0,
                denominator: 1,
                negative: false,
                unbounded_tiny: false,
            });
        }
        if lexical.exponent_negative && lexical.exponent.is_none() {
            // An exponent outside i128 would require more fractional zeroes
            // than the cumulative output budget can contain. Report that
            // actual serialization limit rather than treating magnitude as a
            // machine-capacity failure.
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ));
        }
        if lexical.exponent.is_some_and(|exponent| exponent < 0) {
            return Self::from_lexical_factor(text, factor, context)?.modulo(modulus, context);
        }

        let mut denominator = factor.denominator;
        let mut twos = 0u32;
        let mut fives = 0u32;
        while denominator.is_multiple_of(2) {
            denominator /= 2;
            twos += 1;
        }
        while denominator.is_multiple_of(5) {
            denominator /= 5;
            fives += 1;
        }
        let available = lexical
            .exponent
            .map(|exponent| u32::try_from(exponent).unwrap_or(u32::MAX));
        let cancelled_twos = available.map_or(twos, |value| value.min(twos));
        let cancelled_fives = available.map_or(fives, |value| value.min(fives));
        denominator = denominator
            .checked_mul(2u64.pow(twos - cancelled_twos))
            .and_then(|value| value.checked_mul(5u64.pow(fives - cancelled_fives)))
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        let ring = modulus.checked_mul(denominator).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let work_limbs = BigCoefficient::limb_count_for_digits(lexical.len).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        context.charge_projection(work_limbs)?;
        let coefficient = lexical.digits().fold(0u64, |current, digit| {
            ((u128::from(current) * 10 + u128::from(digit)) % u128::from(ring)) as u64
        });
        let mut residue =
            (u128::from(coefficient) * u128::from(factor.numerator) % u128::from(ring)) as u64;
        residue = (u128::from(residue)
            * u128::from(lexical.pow_mod_after(2, cancelled_twos, ring, context)?)
            % u128::from(ring)) as u64;
        residue = (u128::from(residue)
            * u128::from(lexical.pow_mod_after(5, cancelled_fives, ring, context)?)
            % u128::from(ring)) as u64;
        if lexical.negative && residue != 0 {
            residue = ring - residue;
        }
        Ok(Self {
            coefficient: BigCoefficient::from_u64(residue, context)?,
            exponent: 0,
            denominator,
            negative: false,
            unbounded_tiny: false,
        })
    }

    /// Keeps terminating conversions exact and rounds only repeating ratios.
    pub(crate) fn format_exact_or_rounded(
        self,
        places: usize,
        byte_limit: usize,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        let value = self.reduce_fraction(context)?;
        let mut denominator = value.denominator;
        while denominator.is_multiple_of(2) {
            denominator /= 2;
        }
        while denominator.is_multiple_of(5) {
            denominator /= 5;
        }
        if denominator == 1 {
            value.format_exact(byte_limit, context)
        } else {
            value.format_rounded(places, byte_limit, context)
        }
    }

    pub(crate) fn format_exact(
        self,
        byte_limit: usize,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        let self_ = self.into_terminating(context)?;
        if self_.coefficient.is_zero() {
            return crate::specified_serialization::format_digits(
                std::iter::empty(),
                0,
                0,
                false,
                byte_limit,
            );
        }
        let digits = self_.coefficient.decimal_string(byte_limit)?;
        crate::specified_serialization::format_digits(
            digits.bytes().map(|byte| byte - b'0'),
            digits.len(),
            self_.exponent,
            self_.negative,
            byte_limit,
        )
    }

    pub(crate) fn format_rounded(
        self,
        places: usize,
        byte_limit: usize,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        if self.unbounded_tiny {
            return crate::specified_serialization::format_digits(
                std::iter::empty(),
                0,
                0,
                false,
                byte_limit,
            );
        }
        if self.coefficient.is_zero() {
            return crate::specified_serialization::format_digits(
                std::iter::empty(),
                0,
                0,
                false,
                byte_limit,
            );
        }
        let places_i128 = i128::try_from(places).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        let shift = self.exponent.checked_add(places_i128).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
            )
        })?;
        let (mut rounded, rounding) = if shift >= 0 {
            let denominator_digits = self.denominator.ilog10() as i128 + 1;
            let maximum_materialized_shift = i128::try_from(byte_limit)
                .ok()
                .and_then(|limit| limit.checked_add(denominator_digits))
                .ok_or_else(|| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
            if shift > maximum_materialized_shift {
                return Err(crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
                ));
            }
            let power = usize::try_from(shift).map_err(|_| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            let scaled = self.coefficient.mul_pow10(power, context)?;
            let (quotient, remainder) = scaled.div_rem_small(self.denominator, context)?;
            (
                quotient,
                (u128::from(remainder) * 2).cmp(&u128::from(self.denominator)),
            )
        } else {
            let power = shift.checked_neg().ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            if power > self.coefficient.decimal_len() as i128 {
                (BigCoefficient::zero(), std::cmp::Ordering::Less)
            } else {
                let power = usize::try_from(power).map_err(|_| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
                let (whole, denominator_remainder) =
                    self.coefficient.div_rem_small(self.denominator, context)?;
                let (quotient, low) = whole.div_rem_pow10(power, context)?;
                let mut residual = low
                    .mul_small(self.denominator, context)?
                    .mul_small(2, context)?;
                residual.add_small(
                    denominator_remainder.checked_mul(2).ok_or_else(|| {
                        crate::CssSpecifiedValueSerializationError::new(
                            crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                        )
                    })?,
                    context,
                )?;
                let denominator_digits = self.denominator.ilog10() as usize + 1;
                let comparison_len = denominator_digits.checked_add(power).ok_or_else(|| {
                    crate::CssSpecifiedValueSerializationError::new(
                        crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    )
                })?;
                let ordering = match residual.decimal_len().cmp(&comparison_len) {
                    std::cmp::Ordering::Equal => {
                        let right = BigCoefficient::from_u64(self.denominator, context)?
                            .mul_pow10(power, context)?;
                        residual.cmp(&right)
                    }
                    ordering => ordering,
                };
                (quotient, ordering)
            }
        };
        if rounding == std::cmp::Ordering::Greater
            || (rounding == std::cmp::Ordering::Equal && !self.negative)
        {
            rounded.increment(context)?;
        }
        let mut exponent = -places_i128;
        exponent = exponent
            .checked_add(rounded.strip_decimal_zeros())
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        let negative = self.negative && !rounded.is_zero();
        let digits = rounded.decimal_string(byte_limit)?;
        crate::specified_serialization::format_digits(
            digits.bytes().map(|byte| byte - b'0'),
            if rounded.is_zero() { 0 } else { digits.len() },
            exponent,
            negative,
            byte_limit,
        )
    }
}

impl LexicalDecimal<'_> {
    fn pow_mod_after(
        &self,
        base: u64,
        subtract: u32,
        modulus: u64,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> Result<u64, crate::CssSpecifiedValueSerializationError> {
        if let Some(exponent) = self.exponent {
            let exponent = u128::try_from(exponent).map_err(|_| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            let exponent = exponent.checked_sub(u128::from(subtract)).ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
            return Ok(pow_mod(base, exponent, modulus));
        }
        if self.exponent_negative {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        let work_digits = self.exponent_digits.len().checked_add(1).ok_or_else(|| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        context.charge_projection(work_digits)?;
        let mut digits = Vec::new();
        digits.try_reserve_exact(work_digits).map_err(|_| {
            crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        digits.extend(self.exponent_digits.bytes().map(|byte| byte - b'0'));
        let adjustment = self
            .exponent_adjustment
            .checked_sub(i128::from(subtract))
            .ok_or_else(|| {
                crate::CssSpecifiedValueSerializationError::new(
                    crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                )
            })?;
        if adjustment < 0 {
            decimal_digits_sub_small(&mut digits, adjustment.unsigned_abs())?;
        } else {
            decimal_digits_add_small(&mut digits, adjustment as u128)?;
        }
        let mut result = 1u64 % modulus;
        for digit in digits {
            result = pow_mod(result, 10, modulus);
            result = (u128::from(result) * u128::from(pow_mod(base, u128::from(digit), modulus))
                % u128::from(modulus)) as u64;
        }
        Ok(result)
    }
}

fn pow_mod(mut base: u64, mut exponent: u128, modulus: u64) -> u64 {
    let mut result = 1u64 % modulus;
    base %= modulus;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = (u128::from(result) * u128::from(base) % u128::from(modulus)) as u64;
        }
        base = (u128::from(base) * u128::from(base) % u128::from(modulus)) as u64;
        exponent >>= 1;
    }
    result
}

fn greatest_common_divisor(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn decimal_digits_sub_small(
    digits: &mut Vec<u8>,
    mut value: u128,
) -> Result<(), crate::CssSpecifiedValueSerializationError> {
    let mut index = digits.len();
    let mut borrow = 0u8;
    while value != 0 || borrow != 0 {
        if index == 0 {
            return Err(crate::CssSpecifiedValueSerializationError::new(
                crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            ));
        }
        index -= 1;
        let sub = (value % 10) as u8 + borrow;
        value /= 10;
        if digits[index] >= sub {
            digits[index] -= sub;
            borrow = 0;
        } else {
            digits[index] = digits[index] + 10 - sub;
            borrow = 1;
        }
    }
    let first = digits
        .iter()
        .position(|digit| *digit != 0)
        .unwrap_or(digits.len() - 1);
    digits.drain(..first);
    Ok(())
}

fn decimal_digits_add_small(
    digits: &mut Vec<u8>,
    mut value: u128,
) -> Result<(), crate::CssSpecifiedValueSerializationError> {
    let mut index = digits.len();
    let mut carry = 0u8;
    while value != 0 || carry != 0 {
        if index == 0 {
            let digit = (value % 10) as u8 + carry;
            value /= 10;
            debug_assert!(digits.len() < digits.capacity());
            digits.insert(0, digit % 10);
            carry = digit / 10;
            continue;
        }
        index -= 1;
        let sum = digits[index] + (value % 10) as u8 + carry;
        value /= 10;
        digits[index] = sum % 10;
        carry = sum / 10;
    }
    Ok(())
}

// A binary32 exact decimal needs at most 112 significant coefficient digits.
// This fixed bound is used only to prove membership in the legacy subset: a
// larger lexical coefficient remains valid in CssOpacityScalar.
const DIGITS: usize = 192;

#[derive(Debug, Eq, PartialEq)]
struct Decimal {
    digits: [u8; DIGITS],
    len: usize,
    exponent: i128,
    negative: bool,
}

impl Decimal {
    fn zero() -> Self {
        Self {
            digits: [0; DIGITS],
            len: 0,
            exponent: 0,
            negative: false,
        }
    }

    // Input is already a checked CSS numeric representation. Delay trailing
    // zeros so an arbitrarily long redundant suffix does not fill the buffer.
    fn lexical(text: &str) -> Option<Self> {
        let lexical = LexicalDecimal::new(text);
        if lexical.len > DIGITS {
            return None;
        }
        let mut value = Self::zero();
        value.len = lexical.len;
        for (target, digit) in value.digits.iter_mut().zip(lexical.digits()) {
            *target = digit;
        }
        value.exponent = lexical.exponent?;
        value.negative = lexical.negative;
        Some(value)
    }

    fn binary32(value: f32) -> Self {
        let bits = value.to_bits();
        let fraction = bits & 0x7f_ffff;
        let encoded_exponent = (bits >> 23) & 0xff;
        let (mut coefficient, exponent) = if encoded_exponent == 0 {
            (fraction, -149)
        } else {
            (fraction | (1 << 23), encoded_exponent as i32 - 150)
        };
        if coefficient == 0 {
            return Self::zero();
        }
        let mut reversed = [0u8; DIGITS];
        let mut len = 0;
        while coefficient != 0 {
            reversed[len] = (coefficient % 10) as u8;
            coefficient /= 10;
            len += 1;
        }
        let multiplier = if exponent < 0 { 5 } else { 2 };
        for _ in 0..exponent.unsigned_abs() {
            let mut carry = 0;
            for digit in &mut reversed[..len] {
                let product = *digit * multiplier + carry;
                *digit = product % 10;
                carry = product / 10;
            }
            if carry != 0 {
                reversed[len] = carry;
                len += 1;
            }
        }
        let trailing = reversed[..len]
            .iter()
            .take_while(|&&digit| digit == 0)
            .count();
        let mut result = Self::zero();
        result.len = len - trailing;
        for (target, source) in result.digits[..result.len]
            .iter_mut()
            .zip(reversed[trailing..len].iter().rev())
        {
            *target = *source;
        }
        result.exponent = i128::from(exponent.min(0)) + trailing as i128;
        result.negative = bits >> 31 != 0;
        result
    }
}

pub(crate) fn serialize_binary32(
    value: f32,
    percentage: bool,
    limit: usize,
) -> Result<String, crate::CssSpecifiedValueSerializationError> {
    let value = Decimal::binary32(value);
    crate::specified_serialization::format_digits(
        value.digits[..value.len].iter().copied(),
        value.len,
        value.exponent - if percentage { 2 } else { 0 },
        value.negative,
        limit,
    )
}

// Exact equality left * numerator / denominator == right, for finite binary32
// operands and small fixed color-space bases. No floating-point arithmetic.
pub(crate) fn binary32_scaled_eq(left: f32, numerator: u32, denominator: u32, right: f32) -> bool {
    fn multiply(mut value: Decimal, factor: u32) -> Option<Decimal> {
        if value.len == 0 || factor == 0 {
            return Some(Decimal::zero());
        }
        let mut carry = 0_u64;
        for digit in value.digits[..value.len].iter_mut().rev() {
            let product = u64::from(*digit) * u64::from(factor) + carry;
            *digit = (product % 10) as u8;
            carry = product / 10;
        }
        while carry != 0 {
            if value.len == DIGITS {
                return None;
            }
            value.digits.copy_within(..value.len, 1);
            value.digits[0] = (carry % 10) as u8;
            value.len += 1;
            carry /= 10;
        }
        while value.digits[value.len - 1] == 0 {
            value.len -= 1;
            value.exponent += 1;
        }
        Some(value)
    }
    left.is_finite()
        && right.is_finite()
        && denominator != 0
        && multiply(Decimal::binary32(left), numerator)
            .zip(multiply(Decimal::binary32(right), denominator))
            .is_some_and(|(a, b)| a == b)
}

// Shared exact-fidelity proof for ordinary numeric consumers. The input must be
// a checked CSS numeric representation, without a unit or percentage suffix.
pub(crate) fn exact_legacy_value(text: &str) -> Option<f32> {
    let decimal = Decimal::lexical(text)?;
    if decimal.len == 0 {
        return Some(0.0);
    }
    let candidate = text.parse::<f32>().ok()?;
    (candidate.is_finite() && decimal == Decimal::binary32(candidate)).then_some(candidate)
}

#[cfg(test)]
mod tests {
    use super::{ExactFactor, ExactRational, exact_legacy_value};
    use crate::{
        CssSpecifiedValueSerializationErrorKind as ErrorKind,
        CssSpecifiedValueSerializationLimits as Limits,
        specified_serialization::SpecifiedSerializationContext,
    };

    #[test]
    fn equality_uses_exact_decimal_value_instead_of_rounded_candidate() {
        for (text, expected) in [
            (".5", 0.5),
            ("-1.50", -1.5),
            ("150", 150.0),
            ("000.50000e+0", 0.5),
        ] {
            assert_eq!(exact_legacy_value(text), Some(expected));
        }
        for text in [".1", "1e100", "-1e100", "1e-47", "-1e-47", "16777217"] {
            assert_eq!(exact_legacy_value(text), None, "{text}");
        }
        assert_eq!(
            exact_legacy_value("0.100000001490116119384765625"),
            Some(0.1)
        );
        assert_eq!(
            exact_legacy_value("340282346638528859811704183484516925440"),
            Some(f32::MAX)
        );
        assert_eq!(
            exact_legacy_value(
                "0.00000000000000000000000000000000000000000000140129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125"
            ),
            Some(f32::from_bits(1))
        );
    }

    fn exact(text: &str, factor: ExactFactor) -> String {
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_lexical_factor(text, factor, &mut context).unwrap();
        value.format_exact(1_048_576, &mut context).unwrap()
    }

    fn rounded(text: &str, places: usize) -> String {
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_lexical_factor(
            text,
            ExactFactor {
                numerator: 1,
                denominator: 1,
            },
            &mut context,
        )
        .unwrap();
        value
            .format_rounded(places, 1_048_576, &mut context)
            .unwrap()
    }

    #[test]
    fn fixed_decimal_rounding_uses_nearest_with_ties_toward_positive_infinity() {
        for (input, expected) in [
            ("0.12345649", "0.123456"),
            ("0.1234565", "0.123457"),
            ("-0.1234565", "-0.123456"),
            ("-0.12345651", "-0.123457"),
            ("-0.0000005", "0"),
            ("0.9999996", "1"),
        ] {
            assert_eq!(rounded(input, 6), expected, "{input}");
        }
        assert_eq!(rounded("-0.5", 0), "0");
        assert_eq!(rounded("0.5", 0), "1");
        assert_eq!(
            rounded("1e-170141183460469231731687303715884105727", 6),
            "0"
        );
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let huge = ExactRational::from_lexical_factor(
            "1e170141183460469231731687303715884105727",
            ExactFactor {
                numerator: 1,
                denominator: 1,
            },
            &mut context,
        )
        .unwrap();
        assert_eq!(
            huge.format_rounded(6, 100, &mut context)
                .unwrap_err()
                .kind(),
            ErrorKind::ByteLimit
        );
        assert_eq!(
            ExactRational::from_lexical_factor(
                "40",
                ExactFactor {
                    numerator: 1,
                    denominator: 3,
                },
                &mut SpecifiedSerializationContext::new(Limits::default()),
            )
            .and_then(|value| {
                value.format_rounded(
                    6,
                    100,
                    &mut SpecifiedSerializationContext::new(Limits::default()),
                )
            })
            .unwrap(),
            "13.333333"
        );
    }

    #[test]
    fn selected_radian_factor_is_exact_before_formatting_or_modulo() {
        let factor = ExactFactor {
            numerator: 1_007_958_012_753_983,
            denominator: 17_592_186_044_416,
        };
        assert_eq!(
            exact("1", factor),
            "57.29577951308232286464772187173366546630859375"
        );
        assert_eq!(
            exact("0.5", factor),
            "28.647889756541161432323860935866832733154296875"
        );
        assert_eq!(exact("17592186044416", factor), "1007958012753983");

        let huge = "17592186044416e999999999999999999999999999999999999";
        for (input, expected) in [(huge.to_owned(), "320"), (format!("-{huge}"), "40")] {
            let mut context = SpecifiedSerializationContext::new(Limits::default());
            let value =
                ExactRational::from_lexical_factor_modulo(&input, factor, 360, &mut context)
                    .unwrap();
            assert_eq!(value.format_exact(100, &mut context).unwrap(), expected);
        }
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        assert_eq!(
            ExactRational::from_lexical_factor_modulo(
                "1e-999999999999999999999999999999999999999999999999",
                factor,
                360,
                &mut context,
            )
            .unwrap_err()
            .kind(),
            ErrorKind::ByteLimit
        );
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        assert_eq!(
            ExactRational::from_lexical_factor_modulo(
                "1e-170141183460469231731687303715884105727",
                factor,
                360,
                &mut context,
            )
            .unwrap_err()
            .kind(),
            ErrorKind::ByteLimit
        );
    }

    #[test]
    fn fractional_hues_reduce_modulo_without_a_magnitude_capacity_error() {
        let factor = ExactFactor {
            numerator: 1,
            denominator: 1,
        };
        for (input, expected) in [
            ("0.1", "0.1"),
            ("-0.1", "359.9"),
            ("360.1", "0.1"),
            ("-360.1", "359.9"),
        ] {
            let mut context = SpecifiedSerializationContext::new(Limits::default());
            let value = ExactRational::from_lexical_factor_modulo(input, factor, 360, &mut context)
                .unwrap();
            assert_eq!(value.format_exact(100, &mut context).unwrap(), expected);
        }
    }

    #[test]
    fn programmatic_binary32_uses_its_exact_owned_value_before_scaling() {
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_binary32_factor(
            0.1,
            ExactFactor {
                numerator: 1,
                denominator: 1,
            },
            &mut context,
        )
        .unwrap();
        assert_eq!(
            value.format_exact(100, &mut context).unwrap(),
            "0.100000001490116119384765625"
        );

        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_binary32_factor(
            f32::from_bits(1),
            ExactFactor {
                numerator: 255,
                denominator: 100,
            },
            &mut context,
        )
        .unwrap();
        assert_eq!(value.format_rounded(6, 100, &mut context).unwrap(), "0");
    }

    #[test]
    fn exact_format_cancels_a_shared_non_decimal_denominator_factor_first() {
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_lexical_factor(
            "255",
            ExactFactor {
                numerator: 1,
                denominator: 255,
            },
            &mut context,
        )
        .unwrap();
        assert_eq!(value.format_exact(100, &mut context).unwrap(), "1");

        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_binary32_factor(
            510.0,
            ExactFactor {
                numerator: 1,
                denominator: 255,
            },
            &mut context,
        )
        .unwrap();
        assert_eq!(value.format_exact(100, &mut context).unwrap(), "2");
    }

    #[test]
    fn exact_coefficient_limbs_consume_the_cumulative_projection_budget() {
        let factor = ExactFactor {
            numerator: 1,
            denominator: 1,
        };
        let mut short = SpecifiedSerializationContext::new(Limits::new(0, 5, 100));
        assert_eq!(
            ExactRational::from_lexical_factor("1234567890123456789", factor, &mut short,)
                .unwrap_err()
                .kind(),
            ErrorKind::ProjectionNodeLimit
        );
        let mut exact_context = SpecifiedSerializationContext::new(Limits::new(0, 6, 100));
        assert_eq!(
            ExactRational::from_lexical_factor("1234567890123456789", factor, &mut exact_context,)
                .unwrap()
                .format_exact(100, &mut exact_context)
                .unwrap(),
            "1234567890123456789"
        );

        let mut context = SpecifiedSerializationContext::new(Limits::new(0, 2, 100));
        let value = ExactRational::from_lexical_factor("1", factor, &mut context).unwrap();
        assert_eq!(
            value.clone_with_budget(&mut context).unwrap_err().kind(),
            ErrorKind::ProjectionNodeLimit
        );
        let mut context = SpecifiedSerializationContext::new(Limits::new(0, 3, 100));
        let value = ExactRational::from_lexical_factor("1", factor, &mut context).unwrap();
        assert_eq!(
            value
                .clone_with_budget(&mut context)
                .unwrap()
                .format_exact(100, &mut context)
                .unwrap(),
            "1"
        );
    }

    #[test]
    fn clamped_alpha_handles_exponents_beyond_i128_without_binary_narrowing() {
        let factor = ExactFactor {
            numerator: 1,
            denominator: 1,
        };
        let huge = "999999999999999999999999999999999999999999999999";
        for (input, expected, equal_zero, equal_one) in [
            (format!("1e{huge}"), "1", false, true),
            (format!("-1e{huge}"), "0", true, false),
            (format!("1e-{huge}"), "0", false, false),
            (format!("-1e-{huge}"), "0", true, false),
            ("1e1000000".to_owned(), "1", false, true),
            ("1e-1000000".to_owned(), "0", false, false),
        ] {
            let mut context = SpecifiedSerializationContext::new(Limits::default());
            let value =
                ExactRational::from_lexical_factor_clamped_unit(&input, factor, &mut context)
                    .unwrap();
            assert_eq!(value.equals_ratio(0, 1, &mut context).unwrap(), equal_zero);
            assert_eq!(value.equals_ratio(1, 1, &mut context).unwrap(), equal_one);
            assert_eq!(
                value.format_rounded(6, 100, &mut context).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn exact_weight_arithmetic_precedes_generated_share_rounding() {
        let factor = ExactFactor {
            numerator: 1,
            denominator: 1,
        };
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let sixty = ExactRational::from_lexical_factor("60", factor, &mut context).unwrap();
        let remainder = sixty
            .min_integer(100, &mut context)
            .unwrap()
            .subtract_from_integer(100, &mut context)
            .unwrap()
            .divide_by(3)
            .unwrap();
        assert_eq!(
            remainder
                .clone_with_budget(&mut context)
                .unwrap()
                .format_rounded(6, 100, &mut context)
                .unwrap(),
            "13.333333"
        );
        assert!(remainder.equals_ratio(40, 3, &mut context).unwrap());

        let seventy = ExactRational::from_lexical_factor("70", factor, &mut context).unwrap();
        let sum = seventy
            .add_nonnegative(&seventy, &mut context)
            .unwrap()
            .min_integer(100, &mut context)
            .unwrap();
        let remainder = sum
            .subtract_from_integer(100, &mut context)
            .unwrap()
            .divide_by(1)
            .unwrap();
        assert!(remainder.is_zero());
    }

    #[test]
    fn exact_alpha_clamp_and_unity_test_precede_six_place_rounding() {
        let factor = ExactFactor {
            numerator: 1,
            denominator: 1,
        };
        for (input, expected) in [("-0.1", "0"), ("1.1", "1")] {
            let mut context = SpecifiedSerializationContext::new(Limits::default());
            let value = ExactRational::from_lexical_factor(input, factor, &mut context)
                .unwrap()
                .clamp_unit(&mut context)
                .unwrap();
            assert_eq!(
                value.format_rounded(6, 100, &mut context).unwrap(),
                expected
            );
        }
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        let value = ExactRational::from_lexical_factor("0.9999996", factor, &mut context)
            .unwrap()
            .clamp_unit(&mut context)
            .unwrap();
        assert!(!value.equals_ratio(1, 1, &mut context).unwrap());
        assert_eq!(value.format_rounded(6, 100, &mut context).unwrap(), "1");
    }

    #[test]
    fn redundant_digits_and_large_exponents_do_not_require_large_numeric_storage() {
        assert_eq!(
            exact_legacy_value("-0e9999999999999999999999999999999999999999999"),
            Some(0.0)
        );
        assert_eq!(
            exact_legacy_value("1e9999999999999999999999999999999999999999999"),
            None
        );
        assert_eq!(
            exact_legacy_value(&format!("0.5{}", "0".repeat(4096))),
            Some(0.5)
        );
        assert_eq!(
            exact_legacy_value(&format!("1{}e-4096", "0".repeat(4096))),
            Some(1.0)
        );
    }
}
