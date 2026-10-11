//! Intrinsic numeric values from the pinned HTML common microsyntaxes.
//!
//! Author validation and user-agent recovery are distinct entry points. Parsing
//! stays crate-private so the controlled interpretation owner can apply its
//! request limits before invoking these algorithms.

use crate::DocumentString;
use std::{error::Error, fmt, num::NonZeroU32};

/// An exact HTML integer, without a machine-integer magnitude limit.
///
/// Leading zeroes and negative zero are normalized. Decimal magnitude digits
/// remain exact; bounded machine projections report range failure separately.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlInteger {
    negative: bool,
    magnitude: HtmlNonNegativeInteger,
}

impl HtmlInteger {
    /// Constructs an integer from an intrinsically valid magnitude and sign.
    /// A zero magnitude always produces zero, regardless of `negative`.
    pub fn from_magnitude(magnitude: HtmlNonNegativeInteger, negative: bool) -> Self {
        Self {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    /// Reports whether the mathematical value is below zero.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns the exact absolute value.
    pub fn magnitude(&self) -> &HtmlNonNegativeInteger {
        &self.magnitude
    }

    /// Returns normalized author syntax, with no leading zeroes or plus sign.
    pub fn as_document_string(&self) -> DocumentString {
        let prefix = if self.negative { "-" } else { "" };
        DocumentString::from(format!("{prefix}{}", self.magnitude.digits))
    }

    /// Projects the value into `i128`, reporting a range error without loss.
    pub fn to_i128(&self) -> Result<i128, HtmlNumericRangeError> {
        let magnitude = self.magnitude.to_u128()?;
        if self.negative {
            if magnitude == i128::MIN.unsigned_abs() {
                Ok(i128::MIN)
            } else {
                i128::try_from(magnitude)
                    .map(|value| -value)
                    .map_err(|_| HtmlNumericRangeError)
            }
        } else {
            i128::try_from(magnitude).map_err(|_| HtmlNumericRangeError)
        }
    }

    /// Projects the value into `i64`, reporting a range error without loss.
    pub fn to_i64(&self) -> Result<i64, HtmlNumericRangeError> {
        i64::try_from(self.to_i128()?).map_err(|_| HtmlNumericRangeError)
    }
}

impl From<i128> for HtmlInteger {
    fn from(value: i128) -> Self {
        Self::from_magnitude(value.unsigned_abs().into(), value.is_negative())
    }
}

/// An exact zero or positive HTML integer.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlNonNegativeInteger {
    digits: String,
}

impl HtmlNonNegativeInteger {
    /// Checks an exact sequence of one or more ASCII decimal digits.
    ///
    /// No sign, whitespace, trimming, or prefix recovery is accepted. There is
    /// no machine-integer magnitude limit; leading zeroes are normalized.
    pub fn from_decimal_digits(input: &DocumentString) -> Result<Self, HtmlDecimalDigitsError> {
        let units = input.code_units();
        check_decimal_digits(units)?;
        Ok(Self::from_digits(units))
    }

    fn from_digits(digits: &[u16]) -> Self {
        let first = digits.iter().position(|&unit| unit != b'0' as u16);
        let digits = first.map_or_else(
            || "0".to_owned(),
            |first| {
                digits[first..]
                    .iter()
                    .map(|&unit| char::from(unit as u8))
                    .collect()
            },
        );
        Self { digits }
    }

    /// Reports whether the mathematical value is zero.
    pub fn is_zero(&self) -> bool {
        self.digits == "0"
    }

    /// Returns canonical ASCII decimal digits, without leading zeroes.
    pub fn decimal_digits(&self) -> &str {
        &self.digits
    }

    /// Returns the exact remainder without projecting the whole value.
    pub fn remainder(&self, divisor: NonZeroU32) -> u32 {
        let divisor = u64::from(divisor.get());
        self.digits.bytes().fold(0_u64, |remainder, digit| {
            (remainder * 10 + u64::from(digit - b'0')) % divisor
        }) as u32
    }

    pub(crate) fn multiplied_by(&self, multiplier: u32) -> Self {
        if multiplier == 0 || self.is_zero() {
            return Self::from(0);
        }
        let mut reversed = Vec::new();
        let mut carry = 0_u64;
        for digit in self.digits.bytes().rev() {
            let product = u64::from(digit - b'0') * u64::from(multiplier) + carry;
            reversed.push(char::from(b'0' + (product % 10) as u8));
            carry = product / 10;
        }
        while carry > 0 {
            reversed.push(char::from(b'0' + (carry % 10) as u8));
            carry /= 10;
        }
        Self {
            digits: reversed.into_iter().rev().collect(),
        }
    }

    pub(crate) fn added_to(&self, other: &Self) -> Self {
        let mut left = self.digits.bytes().rev();
        let mut right = other.digits.bytes().rev();
        let mut reversed = Vec::new();
        let mut carry = 0;
        loop {
            let a = left.next();
            let b = right.next();
            if a.is_none() && b.is_none() && carry == 0 {
                break;
            }
            let sum = a.map_or(0, |digit| digit - b'0') + b.map_or(0, |digit| digit - b'0') + carry;
            reversed.push(char::from(b'0' + sum % 10));
            carry = sum / 10;
        }
        Self {
            digits: reversed.into_iter().rev().collect(),
        }
    }

    /// Subtracts one exactly, with no result below zero.
    pub(crate) fn decremented(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let mut digits = self.digits.as_bytes().to_vec();
        for digit in digits.iter_mut().rev() {
            if *digit > b'0' {
                *digit -= 1;
                break;
            }
            *digit = b'9';
        }
        // A canonical positive magnitude can lose at most its first digit.
        let digits = if digits.len() > 1 && digits[0] == b'0' {
            &digits[1..]
        } else {
            &digits
        };
        Some(Self {
            digits: digits.iter().copied().map(char::from).collect(),
        })
    }

    /// Divides an exact magnitude by a bounded nonzero integer.
    /// The remainder is strictly less than the divisor.
    pub(crate) fn quotient_remainder(&self, divisor: NonZeroU32) -> (Self, u32) {
        let divisor = u64::from(divisor.get());
        let mut quotient = String::new();
        let mut remainder = 0_u64;
        for digit in self.digits.bytes() {
            // remainder < divisor <= u32::MAX, so the partial dividend is
            // below 10 * u32::MAX and the quotient digit is in 0..=9.
            let partial = remainder * 10 + u64::from(digit - b'0');
            let digit = (partial / divisor) as u8;
            remainder = partial % divisor;
            if digit != 0 || !quotient.is_empty() {
                quotient.push(char::from(b'0' + digit));
            }
        }
        if quotient.is_empty() {
            quotient.push('0');
        }
        (Self { digits: quotient }, remainder as u32)
    }

    /// Returns normalized ASCII decimal magnitude digits.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.digits.as_str())
    }

    /// Projects the value into `u128`, reporting a range error without loss.
    pub fn to_u128(&self) -> Result<u128, HtmlNumericRangeError> {
        self.digits.parse().map_err(|_| HtmlNumericRangeError)
    }

    /// Projects the value into `u64`, reporting a range error without loss.
    pub fn to_u64(&self) -> Result<u64, HtmlNumericRangeError> {
        self.digits.parse().map_err(|_| HtmlNumericRangeError)
    }
}

impl From<u128> for HtmlNonNegativeInteger {
    fn from(value: u128) -> Self {
        Self {
            digits: value.to_string(),
        }
    }
}

impl TryFrom<HtmlInteger> for HtmlNonNegativeInteger {
    type Error = HtmlNegativeIntegerError;

    fn try_from(value: HtmlInteger) -> Result<Self, Self::Error> {
        if value.negative {
            Err(HtmlNegativeIntegerError)
        } else {
            Ok(value.magnitude)
        }
    }
}

/// An exact decimal-magnitude constructor received no digits or a non-digit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDecimalDigitsError {
    /// There were no code units.
    Empty,
    /// The first non-ASCII-digit code unit, with its zero-based UTF-16 offset.
    NonAsciiDigit {
        /// The zero-based UTF-16 code-unit offset.
        code_unit_offset: usize,
        /// The exact invalid code unit.
        code_unit: u16,
    },
}

impl fmt::Display for HtmlDecimalDigitsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("HTML integer magnitude has no decimal digits"),
            Self::NonAsciiDigit {
                code_unit_offset,
                code_unit,
            } => write!(
                formatter,
                "non-ASCII-digit U+{code_unit:04X} at UTF-16 code-unit offset {code_unit_offset}"
            ),
        }
    }
}

impl Error for HtmlDecimalDigitsError {}

/// An exact numeric value cannot fit the requested machine-integer range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNumericRangeError;

impl fmt::Display for HtmlNumericRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTML numeric value exceeds the requested integer range")
    }
}

impl Error for HtmlNumericRangeError {}

/// A negative integer cannot become a nonnegative integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNegativeIntegerError;

impl fmt::Display for HtmlNegativeIntegerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTML integer is negative")
    }
}

impl Error for HtmlNegativeIntegerError {}

/// A finite IEEE 754 binary64 HTML numeric value, with zero normalized to +0.
///
/// The HTML recovery algorithm rounds the exact decimal value once to the
/// nearest binary64 value, breaking ties with an even significand. Overflow is
/// an error; underflow may produce zero. Integer values have a separate exact
/// representation and must not be routed through this type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlFloatingPointNumber {
    value: f64,
}

impl HtmlFloatingPointNumber {
    /// Returns the finite binary64 value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns deterministic valid HTML author syntax for this binary64 value.
    ///
    /// This is native Rust decimal serialization, not the ECMAScript
    /// `ToString` operation used by HTML's separate best-representation rule.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.value.to_string())
    }
}

impl TryFrom<f64> for HtmlFloatingPointNumber {
    type Error = HtmlNonFiniteNumberError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(HtmlNonFiniteNumberError);
        }
        Ok(Self {
            value: if value == 0.0 { 0.0 } else { value },
        })
    }
}

/// Infinity and NaN cannot become HTML floating-point values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNonFiniteNumberError;

impl fmt::Display for HtmlNonFiniteNumberError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTML floating-point value is not finite")
    }
}

impl Error for HtmlNonFiniteNumberError {}

/// A list of intrinsically valid HTML floating-point values.
///
/// Attribute-specific cardinality and numeric-range constraints are applied by
/// the attribute owner, rather than guessed by this common microsyntax value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlFloatingPointList {
    values: Vec<HtmlFloatingPointNumber>,
}

impl HtmlFloatingPointList {
    /// Constructs a list from finite, already checked numeric values.
    pub fn new(values: impl Into<Vec<HtmlFloatingPointNumber>>) -> Self {
        Self {
            values: values.into(),
        }
    }

    /// Returns the ordered numeric values.
    pub fn values(&self) -> &[HtmlFloatingPointNumber] {
        &self.values
    }

    /// Returns valid author syntax with commas and no whitespace.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(
            self.values
                .iter()
                .map(|number| number.value.to_string())
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

/// An exact nonnegative decimal number used by HTML dimension and temporal
/// algorithms.
///
/// Unlike HTML floating-point recovery, these clauses specify no binary64
/// conversion, rounding, or overflow rule for their decimal quantities. This
/// representation retains the base-ten value exactly, independently of
/// machine-number limits. Temporal owners apply their own range and authored
/// precision rules to this shared numeric value.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlNonNegativeDecimal {
    integer: HtmlNonNegativeInteger,
    fraction: String,
}

impl HtmlNonNegativeDecimal {
    /// Constructs a decimal from an exact integer and one or more fractional
    /// ASCII digits. Signs, whitespace, and recovery syntax are rejected.
    ///
    /// Fractional leading zeroes are significant; trailing zeroes are removed.
    pub fn with_fraction_digits(
        integer: HtmlNonNegativeInteger,
        fraction: &DocumentString,
    ) -> Result<Self, HtmlDecimalDigitsError> {
        check_decimal_digits(fraction.code_units())?;
        Ok(Self::from_parts(integer, fraction.code_units()))
    }

    fn from_parts(integer: HtmlNonNegativeInteger, fraction: &[u16]) -> Self {
        let end = fraction
            .iter()
            .rposition(|&unit| unit != b'0' as u16)
            .map_or(0, |index| index + 1);
        Self {
            integer,
            fraction: fraction[..end]
                .iter()
                .map(|&unit| char::from(unit as u8))
                .collect(),
        }
    }

    /// Adds exact decimal quantities, preserving significant fractional zeroes.
    pub(crate) fn added_to(&self, other: &Self) -> Self {
        let scale = self.fraction.len().max(other.fraction.len());
        let mut fraction = vec![b'0'; scale];
        let mut carry = 0;
        for index in (0..scale).rev() {
            let left = self
                .fraction
                .as_bytes()
                .get(index)
                .map_or(0, |&digit| digit - b'0');
            let right = other
                .fraction
                .as_bytes()
                .get(index)
                .map_or(0, |&digit| digit - b'0');
            let sum = left + right + carry;
            fraction[index] = b'0' + sum % 10;
            carry = sum / 10;
        }
        let mut integer = self.integer.added_to(&other.integer);
        if carry != 0 {
            integer = integer.added_to(&HtmlNonNegativeInteger::from(u128::from(carry)));
        }
        Self::from_ascii_fraction(integer, fraction)
    }

    /// Multiplies an exact decimal quantity by a bounded nonnegative integer.
    pub(crate) fn multiplied_by(&self, multiplier: u32) -> Self {
        if multiplier == 0 || self.is_zero() {
            return HtmlNonNegativeInteger::from(0).into();
        }
        let mut fraction = vec![b'0'; self.fraction.len()];
        let mut carry = 0_u64;
        for (index, digit) in self.fraction.bytes().enumerate().rev() {
            // Each partial product is below 10 * u32::MAX; carry fits u32.
            let product = u64::from(digit - b'0') * u64::from(multiplier) + carry;
            fraction[index] = b'0' + (product % 10) as u8;
            carry = product / 10;
        }
        let mut integer = self.integer.multiplied_by(multiplier);
        if carry != 0 {
            integer = integer.added_to(&HtmlNonNegativeInteger::from(u128::from(carry)));
        }
        Self::from_ascii_fraction(integer, fraction)
    }

    // Only the arithmetic above constructs these ASCII digit buffers. Leading
    // fractional zeroes retain scale; removing trailing zeroes is lossless.
    fn from_ascii_fraction(integer: HtmlNonNegativeInteger, mut fraction: Vec<u8>) -> Self {
        while fraction.last() == Some(&b'0') {
            fraction.pop();
        }
        Self {
            integer,
            fraction: fraction.into_iter().map(char::from).collect(),
        }
    }

    /// Returns the exact integer part.
    pub fn integer_part(&self) -> &HtmlNonNegativeInteger {
        &self.integer
    }

    /// Returns significant fractional digits, including any leading zeroes.
    /// An integer value has no fractional digits.
    pub fn fractional_digits(&self) -> Option<&str> {
        (!self.fraction.is_empty()).then_some(self.fraction.as_str())
    }

    /// Reports whether the exact value is zero, without underflow.
    pub fn is_zero(&self) -> bool {
        self.integer.is_zero() && self.fraction.is_empty()
    }

    /// Returns normalized decimal syntax without sign or exponent.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.decimal_string())
    }

    fn decimal_string(&self) -> String {
        if self.fraction.is_empty() {
            self.integer.digits.clone()
        } else {
            format!("{}.{}", self.integer.digits, self.fraction)
        }
    }
}

impl From<HtmlNonNegativeInteger> for HtmlNonNegativeDecimal {
    fn from(integer: HtmlNonNegativeInteger) -> Self {
        Self {
            integer,
            fraction: String::new(),
        }
    }
}

/// An exact HTML dimension with its intrinsic unit category.
///
/// Relative values belong to the dimension-list algorithm. The single-value
/// dimension algorithm returns only absolute lengths or percentages. These
/// categories do not resolve CSS units or downstream layout context.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HtmlDimension {
    /// An absolute length.
    Absolute(HtmlNonNegativeDecimal),
    /// A percentage.
    Percentage(HtmlNonNegativeDecimal),
    /// A relative dimension-list quantity.
    Relative(HtmlNonNegativeDecimal),
}

impl HtmlDimension {
    /// Returns the exact nonnegative decimal quantity.
    pub fn value(&self) -> &HtmlNonNegativeDecimal {
        match self {
            Self::Absolute(value) | Self::Percentage(value) | Self::Relative(value) => value,
        }
    }

    /// Returns normalized dimension syntax, with `%` or `*` where applicable.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(self.dimension_string())
    }

    fn dimension_string(&self) -> String {
        let suffix = match self {
            Self::Absolute(_) => "",
            Self::Percentage(_) => "%",
            Self::Relative(_) => "*",
        };
        format!("{}{suffix}", self.value().decimal_string())
    }
}

/// An ordered list of exact HTML dimensions.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct HtmlDimensionList {
    values: Vec<HtmlDimension>,
}

impl HtmlDimensionList {
    /// Constructs a list from intrinsically valid dimensions.
    pub fn new(values: impl Into<Vec<HtmlDimension>>) -> Self {
        Self {
            values: values.into(),
        }
    }

    /// Returns the ordered dimensions.
    pub fn values(&self) -> &[HtmlDimension] {
        &self.values
    }

    /// Returns normalized comma-separated dimension syntax.
    pub fn as_document_string(&self) -> DocumentString {
        DocumentString::from(
            self.values
                .iter()
                .map(HtmlDimension::dimension_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

/// Failures of the common numeric algorithms; request limits are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericParseError {
    InvalidSyntax,
    Negative,
    FloatingPointOverflow,
    ZeroDimension,
}

fn is_digit(unit: u16) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&unit)
}

fn check_decimal_digits(units: &[u16]) -> Result<(), HtmlDecimalDigitsError> {
    if units.is_empty() {
        return Err(HtmlDecimalDigitsError::Empty);
    }
    if let Some(code_unit_offset) = units.iter().position(|&unit| !is_digit(unit)) {
        return Err(HtmlDecimalDigitsError::NonAsciiDigit {
            code_unit_offset,
            code_unit: units[code_unit_offset],
        });
    }
    Ok(())
}

fn is_ascii_whitespace(unit: u16) -> bool {
    matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}

fn skip_whitespace(units: &[u16], position: &mut usize) {
    while units
        .get(*position)
        .is_some_and(|&u| is_ascii_whitespace(u))
    {
        *position += 1;
    }
}

fn collect_digits(units: &[u16], position: &mut usize) {
    while units.get(*position).is_some_and(|&u| is_digit(u)) {
        *position += 1;
    }
}

/// Author grammar: an optional minus followed by one or more ASCII digits.
pub(crate) fn validate_integer(input: &DocumentString) -> Result<HtmlInteger, NumericParseError> {
    integer(input.code_units(), false)
}

/// §2.3.4.1 recovery consumes only the initial signed digit sequence.
pub(crate) fn parse_integer(input: &DocumentString) -> Result<HtmlInteger, NumericParseError> {
    integer(input.code_units(), true)
}

fn integer(units: &[u16], recover: bool) -> Result<HtmlInteger, NumericParseError> {
    let mut position = 0;
    if recover {
        skip_whitespace(units, &mut position);
    }
    let negative = units.get(position) == Some(&(b'-' as u16));
    if negative || (recover && units.get(position) == Some(&(b'+' as u16))) {
        position += 1;
    }
    let start = position;
    collect_digits(units, &mut position);
    if start == position || (!recover && position != units.len()) {
        return Err(NumericParseError::InvalidSyntax);
    }
    Ok(HtmlInteger::from_magnitude(
        HtmlNonNegativeInteger::from_digits(&units[start..position]),
        negative,
    ))
}

/// Author grammar: one or more ASCII digits, without either sign.
pub(crate) fn validate_nonnegative_integer(
    input: &DocumentString,
) -> Result<HtmlNonNegativeInteger, NumericParseError> {
    let units = input.code_units();
    if units.is_empty() || !units.iter().copied().all(is_digit) {
        return Err(NumericParseError::InvalidSyntax);
    }
    Ok(HtmlNonNegativeInteger::from_digits(units))
}

/// §2.3.4.2 recovery rejects negative values, while accepting recovered -0.
pub(crate) fn parse_nonnegative_integer(
    input: &DocumentString,
) -> Result<HtmlNonNegativeInteger, NumericParseError> {
    parse_integer(input)?
        .try_into()
        .map_err(|_| NumericParseError::Negative)
}

/// Author grammar, followed by the binary64 conversion contract.
/// Grammatical overflow is distinct from invalid author syntax.
pub(crate) fn validate_floating_point_number(
    input: &DocumentString,
) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    floating_point(input.code_units(), false)
}

/// §2.3.4.3 recovery, including ignored leading plus and incomplete exponents.
pub(crate) fn parse_floating_point_number(
    input: &DocumentString,
) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    floating_point(input.code_units(), true)
}

fn floating_point(
    units: &[u16],
    recover: bool,
) -> Result<HtmlFloatingPointNumber, NumericParseError> {
    let mut position = 0;
    if recover {
        skip_whitespace(units, &mut position);
    }
    let start = position;
    if units.get(position) == Some(&(b'-' as u16))
        || (recover && units.get(position) == Some(&(b'+' as u16)))
    {
        position += 1;
    }
    let integer_start = position;
    collect_digits(units, &mut position);
    let has_integer = position > integer_start;
    let mut has_fraction = false;
    if units.get(position) == Some(&(b'.' as u16)) {
        position += 1;
        let fraction_start = position;
        collect_digits(units, &mut position);
        has_fraction = position > fraction_start;
        if !recover && !has_fraction {
            return Err(NumericParseError::InvalidSyntax);
        }
    }
    if !has_integer && !has_fraction {
        return Err(NumericParseError::InvalidSyntax);
    }
    let significand_end = position;
    if matches!(units.get(position), Some(0x45 | 0x65)) {
        position += 1;
        if matches!(units.get(position), Some(0x2b | 0x2d)) {
            position += 1;
        }
        let exponent_start = position;
        collect_digits(units, &mut position);
        if position == exponent_start {
            if !recover {
                return Err(NumericParseError::InvalidSyntax);
            }
            position = significand_end;
        }
    }
    if !recover && position != units.len() {
        return Err(NumericParseError::InvalidSyntax);
    }
    // Preserve all decimal digits until the single final binary64 conversion.
    // Accumulating digits in f64 would round intermediate values and can turn
    // a long significand with a compensating exponent into spurious overflow.
    let token: String = units[start..position]
        .iter()
        .map(|&unit| char::from(unit as u8))
        .collect();
    let value = token
        .parse::<f64>()
        .map_err(|_| NumericParseError::InvalidSyntax)?;
    HtmlFloatingPointNumber::try_from(value).map_err(|_| NumericParseError::FloatingPointOverflow)
}

/// Author list grammar is comma-separated author numbers, without whitespace.
pub(crate) fn validate_floating_point_list(
    input: &DocumentString,
) -> Result<HtmlFloatingPointList, NumericParseError> {
    if input.is_empty() {
        return Ok(HtmlFloatingPointList::default());
    }
    let mut values = Vec::new();
    let mut overflow = false;
    for units in input.code_units().split(|&unit| unit == b',' as u16) {
        match floating_point(units, false) {
            Ok(value) => values.push(value),
            Err(NumericParseError::FloatingPointOverflow) => overflow = true,
            Err(error) => return Err(error),
        }
    }
    // Overflow describes a representational failure only when the complete
    // list satisfies author grammar. A later invalid token takes precedence.
    if overflow {
        Err(NumericParseError::FloatingPointOverflow)
    } else {
        Ok(HtmlFloatingPointList::new(values))
    }
}

/// §2.3.4.6 recovery preserves token order and substitutes zero on failure.
pub(crate) fn parse_floating_point_list(input: &DocumentString) -> HtmlFloatingPointList {
    let units = input.code_units();
    let mut position = 0;
    let mut values = Vec::new();
    let is_delimiter = |unit| is_ascii_whitespace(unit) || matches!(unit, 0x2c | 0x3b);
    while units.get(position).is_some_and(|&u| is_delimiter(u)) {
        position += 1;
    }
    while position < units.len() {
        while units
            .get(position)
            .is_some_and(|&u| !is_delimiter(u) && !is_digit(u) && !matches!(u, 0x2e | 0x2d))
        {
            position += 1;
        }
        let start = position;
        while units.get(position).is_some_and(|&u| !is_delimiter(u)) {
            position += 1;
        }
        values.push(
            floating_point(&units[start..position], true)
                .unwrap_or(HtmlFloatingPointNumber { value: 0.0 }),
        );
        while units.get(position).is_some_and(|&u| is_delimiter(u)) {
            position += 1;
        }
    }
    HtmlFloatingPointList::new(values)
}

/// §2.3.4.4 digit-led dimension recovery, without an invented author grammar.
pub(crate) fn parse_dimension(input: &DocumentString) -> Result<HtmlDimension, NumericParseError> {
    let units = input.code_units();
    let mut position = 0;
    skip_whitespace(units, &mut position);
    let start = position;
    collect_digits(units, &mut position);
    if start == position {
        return Err(NumericParseError::InvalidSyntax);
    }
    let integer = HtmlNonNegativeInteger::from_digits(&units[start..position]);
    let mut fraction = &[][..];
    if units.get(position) == Some(&(b'.' as u16)) {
        position += 1;
        let start = position;
        collect_digits(units, &mut position);
        fraction = &units[start..position];
    }
    let value = HtmlNonNegativeDecimal::from_parts(integer, fraction);
    if units.get(position) == Some(&(b'%' as u16)) {
        Ok(HtmlDimension::Percentage(value))
    } else {
        Ok(HtmlDimension::Absolute(value))
    }
}

/// §2.3.4.5 rejects exact zero, including arbitrarily small nonzero decimals.
pub(crate) fn parse_nonzero_dimension(
    input: &DocumentString,
) -> Result<HtmlDimension, NumericParseError> {
    let dimension = parse_dimension(input)?;
    if dimension.value().is_zero() {
        Err(NumericParseError::ZeroDimension)
    } else {
        Ok(dimension)
    }
}

/// §2.3.4.7 with the selected #1133 decimal-cursor correction.
///
/// [Retained cursor witness](../../../references/html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md).
///
/// Advance once after recognizing `.` before collecting whitespace/digits.
/// All other pinned steps remain unchanged, including Infra comma trimming,
/// exact decimals, empty-token relative zero, and prefix unit categorization.
pub(crate) fn parse_dimension_list(input: &DocumentString) -> HtmlDimensionList {
    let units = input.code_units();
    let units = units.strip_suffix(&[b',' as u16]).unwrap_or(units);
    let mut position = 0;
    let mut values = Vec::new();
    // Infra split-on-commas does not emit a token once the cursor reaches end.
    while position < units.len() {
        let start = position;
        while units.get(position).is_some_and(|&unit| unit != b',' as u16) {
            position += 1;
        }
        let mut token = &units[start..position];
        while token.first().is_some_and(|&unit| is_ascii_whitespace(unit)) {
            token = &token[1..];
        }
        while token.last().is_some_and(|&unit| is_ascii_whitespace(unit)) {
            token = &token[..token.len() - 1];
        }
        values.push(dimension_list_token(token));
        if position < units.len() {
            position += 1;
        }
    }
    HtmlDimensionList::new(values)
}

fn dimension_list_token(units: &[u16]) -> HtmlDimension {
    if units.is_empty() {
        return HtmlDimension::Relative(HtmlNonNegativeInteger::from(0).into());
    }
    let mut position = 0;
    collect_digits(units, &mut position);
    let integer = HtmlNonNegativeInteger::from_digits(&units[..position]);
    let mut fraction = Vec::new();
    if units.get(position) == Some(&(b'.' as u16)) {
        position += 1; // Selected correction to the pinned step 5.7.
        while let Some(&unit) = units.get(position) {
            if is_digit(unit) {
                fraction.push(unit);
            } else if !is_ascii_whitespace(unit) {
                break;
            }
            position += 1;
        }
    }
    skip_whitespace(units, &mut position);
    let value = HtmlNonNegativeDecimal::from_parts(integer, &fraction);
    match units.get(position) {
        Some(0x25) => HtmlDimension::Percentage(value),
        Some(0x2a) => HtmlDimension::Relative(value),
        _ => HtmlDimension::Absolute(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(input: &str) -> DocumentString {
        input.into()
    }

    #[test]
    fn integer_author_syntax_rejects_recovery_only_spellings() {
        for input in ["", "+1", " 1", "1 ", "1suffix", "--1", "١"] {
            assert_eq!(
                validate_integer(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(validate_integer(&text("-0012")).unwrap().to_i64(), Ok(-12));
        assert_eq!(
            validate_integer(&text("-000")).unwrap(),
            HtmlInteger::from(0)
        );
    }

    #[test]
    fn integer_recovery_preserves_unbounded_magnitude_and_consumes_a_prefix() {
        let input = text("\t+340282366920938463463374607431768211456tail");
        let integer = parse_integer(&input).unwrap();
        assert_eq!(
            integer.as_document_string(),
            text("340282366920938463463374607431768211456")
        );
        assert_eq!(integer.to_i128(), Err(HtmlNumericRangeError));
        assert_eq!(integer.magnitude().to_u128(), Err(HtmlNumericRangeError));
        assert_eq!(
            parse_integer(&text("\u{a0}1")),
            Err(NumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_integer(&DocumentString::from_utf16(vec![49, 0xd800]))
                .unwrap()
                .to_i64(),
            Ok(1)
        );
        assert_eq!(
            validate_integer(&DocumentString::from_utf16(vec![49, 0xd800])),
            Err(NumericParseError::InvalidSyntax)
        );
    }

    #[test]
    fn native_integer_construction_and_bounded_projection_keep_extreme_values() {
        for value in [i128::MIN, -1, 0, 1, i128::MAX] {
            assert_eq!(HtmlInteger::from(value).to_i128(), Ok(value));
        }
        let unsigned = HtmlNonNegativeInteger::from(u128::MAX);
        assert_eq!(unsigned.to_u128(), Ok(u128::MAX));
        assert_eq!(
            HtmlInteger::from_magnitude(unsigned, false).to_i128(),
            Err(HtmlNumericRangeError)
        );
        assert_eq!(HtmlInteger::from(i64::MIN as i128).to_i64(), Ok(i64::MIN));
        assert_eq!(
            HtmlInteger::from(i64::MAX as i128 + 1).to_i64(),
            Err(HtmlNumericRangeError)
        );
    }

    #[test]
    fn exact_decimal_constructor_and_arithmetic_preserve_unbounded_values() {
        let magnitude = HtmlNonNegativeInteger::from_decimal_digits(&text(
            "000340282366920938463463374607431768211456",
        ))
        .unwrap();
        assert_eq!(
            magnitude.decimal_digits(),
            "340282366920938463463374607431768211456"
        );
        assert_eq!(magnitude.to_u128(), Err(HtmlNumericRangeError));
        assert_eq!(magnitude.remainder(NonZeroU32::new(400).unwrap()), 256);
        assert_eq!(
            magnitude.multiplied_by(2).decimal_digits(),
            "680564733841876926926749214863536422912"
        );
        assert_eq!(magnitude.added_to(&magnitude), magnitude.multiplied_by(2));
        assert_eq!(
            HtmlNonNegativeInteger::from(999)
                .added_to(&HtmlNonNegativeInteger::from(1))
                .decimal_digits(),
            "1000"
        );
        assert_eq!(magnitude.multiplied_by(0).decimal_digits(), "0");
        assert_eq!(
            HtmlNonNegativeInteger::from(999)
                .multiplied_by(u32::MAX)
                .decimal_digits(),
            "4290672327705"
        );
        assert_eq!(
            HtmlNonNegativeInteger::from(0)
                .added_to(&HtmlNonNegativeInteger::from(0))
                .decimal_digits(),
            "0"
        );
        assert_eq!(
            HtmlNonNegativeInteger::from_decimal_digits(&text("")),
            Err(HtmlDecimalDigitsError::Empty)
        );
        assert_eq!(
            HtmlNonNegativeInteger::from_decimal_digits(&text("+1")),
            Err(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 0,
                code_unit: 43
            })
        );
        assert_eq!(
            HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(vec![
                49, 0xd800
            ])),
            Err(HtmlDecimalDigitsError::NonAsciiDigit {
                code_unit_offset: 1,
                code_unit: 0xd800
            })
        );
        let signed = HtmlInteger::from_magnitude(magnitude, true);
        assert!(signed.is_negative());
        assert_eq!(
            signed.as_document_string(),
            text("-340282366920938463463374607431768211456")
        );
    }

    #[test]
    fn nonnegative_recovery_accepts_negative_zero_but_author_grammar_does_not() {
        assert!(
            parse_nonnegative_integer(&text(" -000tail"))
                .unwrap()
                .is_zero()
        );
        assert_eq!(
            validate_nonnegative_integer(&text("-0")),
            Err(NumericParseError::InvalidSyntax)
        );
        assert_eq!(
            parse_nonnegative_integer(&text("-1")),
            Err(NumericParseError::Negative)
        );
        assert_eq!(
            validate_nonnegative_integer(&text("00042"))
                .unwrap()
                .to_u64(),
            Ok(42)
        );
        assert_eq!(
            HtmlNonNegativeInteger::try_from(HtmlInteger::from(-1)),
            Err(HtmlNegativeIntegerError)
        );
    }

    #[test]
    fn floating_author_grammar_and_recovery_observe_distinct_rules() {
        for (input, expected) in [(".5", 0.5), ("-.5", -0.5), ("1e+2", 100.0), ("1E-2", 0.01)] {
            assert_eq!(
                validate_floating_point_number(&text(input))
                    .unwrap()
                    .value(),
                expected
            );
        }
        for (input, expected) in [
            (" +1tail", 1.0),
            ("1.", 1.0),
            ("1.e2", 100.0),
            ("1e", 1.0),
            ("1e+", 1.0),
            ("1e-x", 1.0),
            ("1.2suffix", 1.2),
        ] {
            assert_eq!(
                validate_floating_point_number(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
            assert_eq!(
                parse_floating_point_number(&text(input)).unwrap().value(),
                expected
            );
        }
        for input in ["", ".", "-.", "NaN", "Infinity", "\u{a0}1"] {
            assert_eq!(
                parse_floating_point_number(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
    }

    #[test]
    fn floating_conversion_rounds_once_and_distinguishes_overflow() {
        let halfway_even = text("1.00000000000000011102230246251565404236316680908203125");
        assert_eq!(
            parse_floating_point_number(&halfway_even).unwrap().value(),
            1.0
        );
        let halfway_odd = text("1.00000000000000033306690738754696212708950042724609375");
        assert_eq!(
            parse_floating_point_number(&halfway_odd)
                .unwrap()
                .value()
                .to_bits(),
            1.0f64.to_bits() + 2
        );
        assert_eq!(
            parse_floating_point_number(&text("1.7976931348623158e308"))
                .unwrap()
                .value(),
            f64::MAX
        );
        assert_eq!(
            parse_floating_point_number(&text("1.7976931348623159e308")),
            Err(NumericParseError::FloatingPointOverflow)
        );
        assert_eq!(
            validate_floating_point_number(&text("1e9999")),
            Err(NumericParseError::FloatingPointOverflow)
        );
        let long = text(&format!("1{}e-400", "0".repeat(400)));
        assert_eq!(parse_floating_point_number(&long).unwrap().value(), 1.0);
        for input in ["-0", "-1e-9999", "0e9999"] {
            assert_eq!(
                parse_floating_point_number(&text(input))
                    .unwrap()
                    .value()
                    .to_bits(),
                0.0f64.to_bits()
            );
        }
    }

    #[test]
    fn checked_native_floats_reject_nonfinite_values_and_serialize_author_syntax() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                HtmlFloatingPointNumber::try_from(value),
                Err(HtmlNonFiniteNumberError)
            );
        }
        for value in [-0.0, -1.25, f64::MAX, f64::MIN_POSITIVE, f64::from_bits(1)] {
            let number = HtmlFloatingPointNumber::try_from(value).unwrap();
            assert_eq!(
                validate_floating_point_number(&number.as_document_string()),
                Ok(number)
            );
        }
        assert_eq!(
            HtmlFloatingPointNumber::try_from(-0.0)
                .unwrap()
                .as_document_string(),
            text("0")
        );
    }

    #[test]
    fn floating_lists_validate_author_delimiters_and_recover_garbage_as_specified() {
        assert_eq!(
            validate_floating_point_list(&text("1,-.5,2e2"))
                .unwrap()
                .values()
                .iter()
                .map(|n| n.value())
                .collect::<Vec<_>>(),
            vec![1.0, -0.5, 200.0]
        );
        for input in ["1, 2", "1;2", "1,", ",1", "1,,2", "+1"] {
            assert_eq!(
                validate_floating_point_list(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert!(
            validate_floating_point_list(&text(""))
                .unwrap()
                .values()
                .is_empty()
        );
        let recovered = parse_floating_point_list(&text(" ,;garbage+2;bad;.;-3tail;1e9999;"));
        assert_eq!(
            recovered
                .values()
                .iter()
                .map(|n| n.value())
                .collect::<Vec<_>>(),
            vec![2.0, 0.0, 0.0, -3.0, 0.0]
        );
        assert_eq!(recovered.as_document_string(), text("2,0,0,-3,0"));
        assert_eq!(
            parse_floating_point_list(&text("garbage")).values()[0].value(),
            0.0
        );
        assert!(parse_floating_point_list(&text(" ,; ")).values().is_empty());
    }

    #[test]
    fn author_list_overflow_does_not_hide_later_invalid_syntax() {
        for input in ["1e999,nope", "1e999,", "1e999, 1", "1e999,+1"] {
            assert_eq!(
                validate_floating_point_list(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
        assert_eq!(
            validate_floating_point_list(&text("1e999,1.5")),
            Err(NumericParseError::FloatingPointOverflow)
        );
    }

    #[test]
    fn native_dimension_decimals_preserve_fraction_zeroes_and_exact_magnitude() {
        let integer = HtmlNonNegativeInteger::from_decimal_digits(&text(
            "340282366920938463463374607431768211456",
        ))
        .unwrap();
        let decimal =
            HtmlNonNegativeDecimal::with_fraction_digits(integer, &text("0001200")).unwrap();
        assert_eq!(decimal.fractional_digits(), Some("00012"));
        assert_eq!(
            decimal.as_document_string(),
            text("340282366920938463463374607431768211456.00012")
        );
        let percentage = HtmlDimension::Percentage(decimal);
        assert_eq!(
            percentage.as_document_string(),
            text("340282366920938463463374607431768211456.00012%")
        );
        assert_eq!(
            HtmlNonNegativeDecimal::with_fraction_digits(
                HtmlNonNegativeInteger::from(1),
                &text("")
            ),
            Err(HtmlDecimalDigitsError::Empty)
        );
        assert!(
            HtmlNonNegativeDecimal::with_fraction_digits(
                HtmlNonNegativeInteger::from(1),
                &text(" 5")
            )
            .is_err()
        );
        let zero = HtmlNonNegativeDecimal::with_fraction_digits(
            HtmlNonNegativeInteger::from(0),
            &text("000"),
        )
        .unwrap();
        assert!(zero.is_zero());
        assert_eq!(zero.fractional_digits(), None);
        assert_eq!(zero.integer_part().decimal_digits(), "0");
    }

    #[test]
    fn exact_integer_decrement_borrows_and_stops_at_zero() {
        assert_eq!(HtmlNonNegativeInteger::from(0).decremented(), None);
        for (input, expected) in [
            ("1", "0"),
            ("10", "9"),
            ("1000", "999"),
            ("1010", "1009"),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211455",
            ),
        ] {
            let value = HtmlNonNegativeInteger::from_decimal_digits(&text(input)).unwrap();
            assert_eq!(value.decremented().unwrap().decimal_digits(), expected);
            assert_eq!(value.decimal_digits(), input);
        }
    }

    #[test]
    fn exact_integer_division_handles_large_magnitudes_and_boundary_divisors() {
        // Independent base-ten expectations: 2^128 / 400; and the geometric
        // identity (2^32)^4 = (2^32 - 1)(1 + 2^32 + 2^64 + 2^96) + 1.
        for (input, divisor, quotient, remainder) in [
            ("0", 400, "0", 0),
            ("399", 400, "0", 399),
            ("400", 400, "1", 0),
            ("401", 400, "1", 1),
            ("4294967294", u32::MAX, "0", u32::MAX - 1),
            ("4294967295", u32::MAX, "1", 0),
            ("4294967296", u32::MAX, "1", 1),
            (
                "340282366920938463463374607431768211456",
                400,
                "850705917302346158658436518579420528",
                256,
            ),
            (
                "340282366920938463463374607431768211456",
                u32::MAX,
                "79228162532711081671548469249",
                1,
            ),
            (
                "340282366920938463463374607431768211456",
                1,
                "340282366920938463463374607431768211456",
                0,
            ),
        ] {
            let value = HtmlNonNegativeInteger::from_decimal_digits(&text(input)).unwrap();
            let (actual_quotient, actual_remainder) =
                value.quotient_remainder(NonZeroU32::new(divisor).unwrap());
            assert_eq!(actual_quotient.decimal_digits(), quotient);
            assert_eq!(actual_remainder, remainder);
            assert!(actual_remainder < divisor);
            assert_eq!(
                actual_quotient
                    .multiplied_by(divisor)
                    .added_to(&HtmlNonNegativeInteger::from(u128::from(actual_remainder))),
                value
            );
        }
    }

    #[test]
    fn bounded_integer_division_matches_machine_arithmetic_without_projection_loss() {
        for value in [0, 1, 399, 400, 401, u64::MAX as u128, u128::MAX] {
            for divisor in [1, 2, 400, u32::MAX - 1, u32::MAX] {
                let (quotient, remainder) = HtmlNonNegativeInteger::from(value)
                    .quotient_remainder(NonZeroU32::new(divisor).unwrap());
                assert_eq!(quotient.to_u128(), Ok(value / u128::from(divisor)));
                assert_eq!(u128::from(remainder), value % u128::from(divisor));
            }
        }
    }

    fn decimal(integer: &str, fraction: &str) -> HtmlNonNegativeDecimal {
        HtmlNonNegativeDecimal::with_fraction_digits(
            HtmlNonNegativeInteger::from_decimal_digits(&text(integer)).unwrap(),
            &text(fraction),
        )
        .unwrap()
    }

    #[test]
    fn exact_decimal_addition_aligns_places_and_carries_into_huge_integers() {
        for (left_integer, left_fraction, right_integer, right_fraction, expected) in [
            ("0", "999", "0", "001", "1"),
            ("0", "0125", "0", "5", "0.5125"),
            ("0", "5", "0", "05", "0.55"),
            ("0", "00000001", "0", "00000009", "0.0000001"),
            ("9", "75", "0", "25", "10"),
            (
                "340282366920938463463374607431768211456",
                "9999",
                "0",
                "0001",
                "340282366920938463463374607431768211457",
            ),
        ] {
            let left = decimal(left_integer, left_fraction);
            let right = decimal(right_integer, right_fraction);
            let sum = left.added_to(&right);
            assert_eq!(sum.as_document_string(), text(expected));
            assert_eq!(right.added_to(&left), sum);
        }
        let integer = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(400));
        assert_eq!(integer.added_to(&integer).as_document_string(), text("800"));
        assert_eq!(
            integer.added_to(&decimal("0", "025")).as_document_string(),
            text("400.025")
        );
    }

    #[test]
    fn exact_decimal_multiplication_preserves_fractional_places_and_bounded_carries() {
        for (integer, fraction, multiplier, expected) in [
            ("0", "25", u32::MAX, "1073741823.75"),
            ("9", "75", 4, "39"),
            ("0", "0001", 100, "0.01"),
            ("0", "0001", 10_000, "1"),
            (
                "340282366920938463463374607431768211456",
                "75",
                4,
                "1361129467683753853853498429727072845827",
            ),
        ] {
            assert_eq!(
                decimal(integer, fraction)
                    .multiplied_by(multiplier)
                    .as_document_string(),
                text(expected)
            );
        }
        let integer = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(400));
        assert_eq!(
            integer.multiplied_by(400).as_document_string(),
            text("160000")
        );
        assert_eq!(integer.multiplied_by(1), integer);
    }

    #[test]
    fn decimal_arithmetic_normalizes_zero_and_equivalent_trailing_zeroes() {
        let zero = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(0));
        let fractional_zero = decimal("000", "0000");
        assert_eq!(fractional_zero, zero);
        assert_eq!(zero.added_to(&fractional_zero), zero);
        assert_eq!(fractional_zero.multiplied_by(u32::MAX), zero);
        let value = decimal("001", "0025000");
        assert_eq!(value, decimal("1", "0025"));
        assert_eq!(value.added_to(&zero), value);
        assert_eq!(value.multiplied_by(1), value);
        assert_eq!(value.multiplied_by(0), zero);
        assert_eq!(value.added_to(&value).as_document_string(), text("2.005"));
    }

    #[test]
    fn single_dimension_recovery_uses_digit_prefix_and_immediate_percentage_marker() {
        for (input, expected) in [
            (" 12.50%tail", "12.5%"),
            ("12.50 %", "12.5"),
            ("1.%", "1%"),
            ("1. %", "1"),
            ("3*", "3"),
            ("2e3%", "2"),
            ("7junk%", "7"),
        ] {
            assert_eq!(
                parse_dimension(&text(input)).unwrap().as_document_string(),
                text(expected)
            );
        }
        for input in ["", " ", ".5%", "+1", "-0", "\u{a0}1", "١"] {
            assert_eq!(
                parse_dimension(&text(input)),
                Err(NumericParseError::InvalidSyntax)
            );
        }
    }

    #[test]
    fn nonzero_dimension_checks_exact_zero_without_binary64_underflow() {
        for input in ["0", "0.000%", "000.0tail"] {
            assert_eq!(
                parse_nonzero_dimension(&text(input)),
                Err(NumericParseError::ZeroDimension)
            );
        }
        let tiny = format!("0.{}1%", "0".repeat(400));
        let value = parse_nonzero_dimension(&text(&tiny)).unwrap();
        assert_eq!(value.as_document_string(), text(&tiny));
        let huge = "340282366920938463463374607431768211456.125%";
        assert_eq!(
            parse_nonzero_dimension(&text(huge))
                .unwrap()
                .as_document_string(),
            text(huge)
        );
    }

    #[test]
    fn dimension_list_selected_decimal_cursor_correction_exposes_fraction_and_unit() {
        // Independent expectations selected in #1133:
        // ../../../references/html-dimension-list-cursor--webkit-73aa6c89e2cb-blink-7984f9d11800.md
        for (input, expected) in [
            ("1.5%", "1.5%"),
            ("1.5*", "1.5*"),
            ("1.5", "1.5"),
            (".5%", "0.5%"),
            ("1.%", "1%"),
            ("1. 5 0%", "1.5%"),
            ("2", "2"),
            ("3%", "3%"),
            ("4*", "4*"),
        ] {
            let list = parse_dimension_list(&text(input));
            assert_eq!(list.values().len(), 1);
            assert_eq!(list.as_document_string(), text(expected));
        }
        assert!(matches!(
            parse_dimension_list(&text("1.5%")).values()[0],
            HtmlDimension::Percentage(_)
        ));
        assert!(matches!(
            parse_dimension_list(&text("1.5*")).values()[0],
            HtmlDimension::Relative(_)
        ));
        assert!(matches!(
            parse_dimension_list(&text("1.5")).values()[0],
            HtmlDimension::Absolute(_)
        ));
    }

    #[test]
    fn dimension_list_preserves_infra_comma_trimming_and_pinned_empty_token_rules() {
        for input in ["", ","] {
            assert!(parse_dimension_list(&text(input)).values().is_empty());
        }
        for (input, expected) in [
            (" , ", "0*,0*"),
            (",,", "0*"),
            ("1,,2,,", "1,0*,2"),
            (" 1. 5 0% , .5* , 2,", "1.5%,0.5*,2"),
            ("junk, * , % , 1tail%", "0,0*,0%,1"),
            ("\u{a0}1%", "0"),
        ] {
            assert_eq!(
                parse_dimension_list(&text(input)).as_document_string(),
                text(expected)
            );
        }
        let huge = "340282366920938463463374607431768211456.0001200*";
        assert_eq!(
            parse_dimension_list(&text(huge)).as_document_string(),
            text("340282366920938463463374607431768211456.00012*")
        );
    }
}
