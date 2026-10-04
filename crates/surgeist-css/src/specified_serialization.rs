//! Bounded, context-independent specified-value text.

use crate::numeric_formatting::format_css_number;
use crate::{
    CssBorderCollapse, CssBoxSizing, CssCaptionSide, CssEmptyCells, CssOpacityScalarKind,
    CssOpacityValue, CssTableLayout,
};
use std::fmt;

/// Resource policy for specified-value projection and serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSpecifiedValueSerializationLimits {
    max_input_nodes: usize,
    max_projection_nodes: usize,
    max_css_bytes: usize,
}

impl CssSpecifiedValueSerializationLimits {
    /// Sets independent limits. Zero is a valid limit.
    #[must_use]
    pub const fn new(
        max_input_nodes: usize,
        max_projection_nodes: usize,
        max_css_bytes: usize,
    ) -> Self {
        Self {
            max_input_nodes,
            max_projection_nodes,
            max_css_bytes,
        }
    }
    #[must_use]
    pub const fn max_input_nodes(self) -> usize {
        self.max_input_nodes
    }
    #[must_use]
    pub const fn max_projection_nodes(self) -> usize {
        self.max_projection_nodes
    }
    #[must_use]
    pub const fn max_css_bytes(self) -> usize {
        self.max_css_bytes
    }
}

impl Default for CssSpecifiedValueSerializationLimits {
    fn default() -> Self {
        Self::new(65_536, 262_144, 1_048_576)
    }
}

/// An exhausted resource or a value that cannot be faithfully represented.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssSpecifiedValueSerializationErrorKind {
    InputNodeLimit,
    ProjectionNodeLimit,
    ByteLimit,
    CapacityOverflow,
    /// Retained component boundaries cannot be emitted without changing tokens.
    UnserializableBoundary,
    /// The selected numeric precision cannot emit a value allowed by its grammar.
    UnrepresentableValue,
}

/// Atomic failure: no partial CSS is returned and the authored input is unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSpecifiedValueSerializationError {
    kind: CssSpecifiedValueSerializationErrorKind,
}

impl CssSpecifiedValueSerializationError {
    pub(crate) const fn new(kind: CssSpecifiedValueSerializationErrorKind) -> Self {
        Self { kind }
    }
    #[must_use]
    pub const fn kind(&self) -> CssSpecifiedValueSerializationErrorKind {
        self.kind
    }
}
impl fmt::Display for CssSpecifiedValueSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit => {
                "specified-value input node limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit => {
                "specified-value projection node limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::ByteLimit => {
                "specified-value CSS byte limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow => {
                "specified-value capacity calculation overflowed"
            }
            CssSpecifiedValueSerializationErrorKind::UnserializableBoundary => {
                "specified-value component boundary cannot be serialized"
            }
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue => {
                "specified value cannot be represented within the selected serialization precision"
            }
        })
    }
}
impl std::error::Error for CssSpecifiedValueSerializationError {}

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;
use CssSpecifiedValueSerializationErrorKind as Kind;

/// Monotonic resources shared by every projection participating in one public
/// specified-value serialization. Dropping a child arena never refunds work.
pub(crate) struct SpecifiedSerializationContext {
    limits: CssSpecifiedValueSerializationLimits,
    input_nodes: usize,
    projection_nodes: usize,
    css_bytes: usize,
    output_suppressed: bool,
}

impl SpecifiedSerializationContext {
    pub(crate) const fn new(limits: CssSpecifiedValueSerializationLimits) -> Self {
        Self {
            limits,
            input_nodes: 0,
            projection_nodes: 0,
            css_bytes: 0,
            output_suppressed: false,
        }
    }

    fn charge(current: &mut usize, amount: usize, limit: usize, kind: Kind) -> Result<()> {
        let next = current
            .checked_add(amount)
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
        if next > limit {
            return Err(CssSpecifiedValueSerializationError::new(kind));
        }
        *current = next;
        Ok(())
    }

    pub(crate) fn charge_input(&mut self, amount: usize) -> Result<()> {
        Self::charge(
            &mut self.input_nodes,
            amount,
            self.limits.max_input_nodes(),
            Kind::InputNodeLimit,
        )
    }

    pub(crate) fn charge_projection(&mut self, amount: usize) -> Result<()> {
        Self::charge(
            &mut self.projection_nodes,
            amount,
            self.limits.max_projection_nodes(),
            Kind::ProjectionNodeLimit,
        )
    }

    pub(crate) fn remaining_bytes(&self) -> usize {
        self.limits.max_css_bytes() - self.css_bytes
    }

    pub(crate) const fn output_suppressed(&self) -> bool {
        self.output_suppressed
    }

    pub(crate) fn replace_output_suppression(&mut self, suppressed: bool) -> bool {
        std::mem::replace(&mut self.output_suppressed, suppressed)
    }

    pub(crate) fn append(&mut self, output: &mut String, text: &str) -> Result<()> {
        if self.output_suppressed {
            return Ok(());
        }
        debug_assert_eq!(output.len(), self.css_bytes);
        let next = self
            .css_bytes
            .checked_add(text.len())
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
        if next > self.limits.max_css_bytes() {
            return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
        }
        output
            .try_reserve(text.len())
            .map_err(|_| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
        output.push_str(text);
        self.css_bytes = next;
        Ok(())
    }

    /// Appends to a caller-owned scratch value without charging final-output
    /// bytes. The scratch value is still bounded by the final output space that
    /// remains, so branch selection cannot allocate a per-channel value larger
    /// than the public operation could return.
    pub(crate) fn append_temporary(&self, output: &mut String, text: &str) -> Result<()> {
        let next = output
            .len()
            .checked_add(text.len())
            .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
        if next > self.remaining_bytes() {
            return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
        }
        output
            .try_reserve(text.len())
            .map_err(|_| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
        output.push_str(text);
        Ok(())
    }
}

pub(crate) fn serialize_keyword_sequence(
    text: &str,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    context.charge_input(1)?;
    context.charge_projection(1)?;
    let mut output = String::new();
    context.append(&mut output, text)?;
    Ok(output)
}

impl crate::CssAngleLiteral {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if context.output_suppressed() {
            return Ok(());
        }
        let text = format_coefficient(
            self.numeric().representation(),
            0,
            crate::angle::suffix(self.unit()),
            context.remaining_bytes(),
        )?;
        context.append(output, &text)
    }
}
impl crate::CssZeroLiteral {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(output, "0")
    }
}
impl crate::CssAngleValue {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        if let Some(literal) = self.literal() {
            return literal.append_specified(context, output);
        }
        crate::numeric::project_calculation_specified_into(
            crate::numeric::SpecifiedCalculationRef::Angle(
                self.calculation().expect("checked angle branch"),
            ),
            context,
            output,
        )
        .map(|_| ())
    }
}
impl crate::CssAngleOrZero {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Angle(value) => value.append_specified(context, output),
            Self::Zero(value) => value.append_specified(context, output),
        }
    }
}

/// Formats a checked ordinary coefficient with six fractional places at most.
/// Suffix bytes belong to this operation's budget, not a later unchecked append.
pub(crate) fn format_coefficient(
    text: &str,
    shift: i128,
    suffix: &str,
    limit: usize,
) -> Result<String> {
    let remaining = limit.checked_sub(suffix.len()).ok_or_else(|| {
        crate::CssSpecifiedValueSerializationError::new(
            crate::CssSpecifiedValueSerializationErrorKind::ByteLimit,
        )
    })?;
    let mut result = format_css_number(text, shift, remaining)?;
    result.try_reserve(suffix.len()).map_err(|_| {
        crate::CssSpecifiedValueSerializationError::new(
            crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    result.push_str(suffix);
    Ok(result)
}

impl crate::CssOverflowWrap {
    /// Serializes the specified overflow wrapping keyword before layout resolution.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one keyword under exact input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize_keyword_sequence(
            match self {
                Self::Normal => "normal",
                Self::BreakWord => "break-word",
                Self::Anywhere => "anywhere",
            },
            limits,
        )
    }
}

impl CssBoxSizing {
    /// Serializes the intrinsic specified keyword without resolving a box size.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under input, projection, and output limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize_keyword_sequence(
            match self {
                Self::ContentBox => "content-box",
                Self::BorderBox => "border-box",
            },
            limits,
        )
    }
}

impl CssBorderCollapse {
    /// Serializes the specified table border model keyword.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under input, projection, and output limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize_keyword_sequence(
            match self {
                Self::Collapse => "collapse",
                Self::Separate => "separate",
            },
            limits,
        )
    }
}

impl CssCaptionSide {
    /// Serializes the specified table caption side keyword.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under input, projection, and output limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize_keyword_sequence(
            match self {
                Self::Top => "top",
                Self::Bottom => "bottom",
            },
            limits,
        )
    }
}

impl CssEmptyCells {
    /// Serializes the specified empty-cell visibility keyword.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under input, projection, and output limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize_keyword_sequence(
            match self {
                Self::Show => "show",
                Self::Hide => "hide",
            },
            limits,
        )
    }
}

impl CssTableLayout {
    /// Serializes the specified table layout algorithm keyword.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under input, projection, and output limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        serialize_keyword_sequence(
            match self {
                Self::Auto => "auto",
                Self::Fixed => "fixed",
            },
            limits,
        )
    }
}

impl crate::CssFlowTolerance {
    /// Serializes canonical specified tolerance without resolving `normal`,
    /// relative lengths, or the percentage basis.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under cumulative input, projection, and byte limits.
    /// Exact ordinary magnitudes and supported symbolic math use the shared
    /// length-percentage serializer. Failure leaves the authored value unchanged.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }

    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        let keyword = match self.as_ref() {
            crate::CssFlowToleranceRef::Normal => "normal",
            crate::CssFlowToleranceRef::Infinite => "infinite",
            crate::CssFlowToleranceRef::LengthPercentage(value) => {
                return value.append_specified(context, output);
            }
        };
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(output, keyword)
    }
}

impl CssOpacityValue {
    /// Produces canonical specified opacity, without computed-value clamping.
    ///
    /// Ordinary input remains exact; emitted scalar text rounds to six fractional
    /// places, nearest with ties away from zero, after percentage conversion.
    /// Finite calculation text rounds the actual binary64 result with the same
    /// six-place policy. Arithmetic uses the implementation-defined binary64
    /// precision and range policy described in the crate's numeric reference.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Projects and serializes with independent input, projection and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        match self {
            Self::NumberCalculation(value) => {
                return crate::numeric::project_specified(&value.expression, limits);
            }
            Self::PercentageCalculation(value) => {
                return crate::numeric::project_specified(&value.expression, limits);
            }
            _ => {}
        }
        if limits.max_input_nodes() == 0 {
            return Err(CssSpecifiedValueSerializationError::new(
                Kind::InputNodeLimit,
            ));
        }
        if limits.max_projection_nodes() == 0 {
            return Err(CssSpecifiedValueSerializationError::new(
                Kind::ProjectionNodeLimit,
            ));
        }
        match self {
            Self::Scalar(value) => format_lexical(
                value.numeric().representation(),
                value.kind() == CssOpacityScalarKind::Percentage,
                limits.max_css_bytes(),
            ),
            Self::NumberCalculation(_) | Self::PercentageCalculation(_) => {
                unreachable!("calculation branches handled above")
            }
        }
    }
}

fn format_lexical(text: &str, percentage: bool, limit: usize) -> Result<String> {
    format_css_number(text, if percentage { -2 } else { 0 }, limit)
}

#[cfg(test)]
mod composed_value_tests {
    use super::*;
    use crate::{CssColor, CssIntegerCalculation, CssIntegerValue};

    #[test]
    fn ordinary_siblings_use_rounded_bytes_in_one_context_without_partial_child_text() {
        let first = crate::CssSpecifiedNonNegativeNumber::try_from_component(
            crate::CssComponentValue::try_number(".12345649").unwrap(),
        )
        .unwrap();
        let second = crate::CssSpecifiedNonNegativeNumber::try_from_component(
            crate::CssComponentValue::try_number(".9999996").unwrap(),
        )
        .unwrap();
        let expected = "0.123456 1";
        let before = (first.clone(), second.clone());
        for (limits, failure) in [
            (CssSpecifiedValueSerializationLimits::new(2, 2, 10), None),
            (
                CssSpecifiedValueSerializationLimits::new(1, 2, 10),
                Some(Kind::InputNodeLimit),
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 1, 10),
                Some(Kind::ProjectionNodeLimit),
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 2, 9),
                Some(Kind::ByteLimit),
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            first.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, " ").unwrap();
            let result = second.append_specified(&mut context, &mut output);
            if let Some(kind) = failure {
                assert_eq!(result.unwrap_err().kind(), kind);
                assert_eq!(output, "0.123456 ");
            } else {
                result.unwrap();
                assert_eq!(output, expected);
            }
            assert_eq!((&first, &second), (&before.0, &before.1));
        }
    }

    #[test]
    fn flow_tolerance_keywords_and_math_share_monotonic_resources() {
        let keyword = crate::CssFlowTolerance::normal();
        let math = crate::CssFlowTolerance::length_percentage(
            crate::CssSpecifiedLengthPercentage::try_from_calculation(
                crate::CssLengthPercentageCalculation::try_from_components(
                    crate::parse_component_values("calc(1px + 2em)").unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let expected = "normal calc(2em + 1px)";
        let mut context = SpecifiedSerializationContext::new(
            CssSpecifiedValueSerializationLimits::new(5, 6, expected.len()),
        );
        let mut output = String::new();
        keyword.append_specified(&mut context, &mut output).unwrap();
        context.append(&mut output, " ").unwrap();
        math.append_specified(&mut context, &mut output).unwrap();
        assert_eq!(output, expected);

        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(4, 6, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(5, 5, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(5, 6, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            keyword.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, " ").unwrap();
            assert_eq!(
                math.append_specified(&mut context, &mut output)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(output, "normal ");
        }
    }

    #[test]
    fn color_and_integer_share_input_projection_and_output_limits() {
        let color = CssColor::transparent();
        let integer = CssIntegerValue::Literal(crate::CssIntegerLiteral::from_i32(7));

        let mut context =
            SpecifiedSerializationContext::new(CssSpecifiedValueSerializationLimits::new(2, 2, 13));
        let mut css = String::new();
        color.append_specified(&mut context, &mut css).unwrap();
        context.append(&mut css, " ").unwrap();
        integer.append_specified(&mut context, &mut css).unwrap();
        assert_eq!(css, "transparent 7");

        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, 2, 13),
                Kind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 1, 13),
                Kind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 2, 12),
                Kind::ByteLimit,
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut css = String::new();
            color.append_specified(&mut context, &mut css).unwrap();
            context.append(&mut css, " ").unwrap();
            assert_eq!(
                integer
                    .append_specified(&mut context, &mut css)
                    .unwrap_err()
                    .kind(),
                expected
            );
            assert_eq!(css, "transparent ");
        }
    }

    #[test]
    fn sequential_colors_and_integer_calculation_cannot_reset_byte_limit() {
        let color = CssColor::transparent();
        let calculation = CssIntegerValue::Calculation(CssIntegerCalculation::literal(2));

        let mut context = SpecifiedSerializationContext::new(
            CssSpecifiedValueSerializationLimits::new(100, 100, 21),
        );
        let mut css = String::new();
        color.append_specified(&mut context, &mut css).unwrap();
        assert_eq!(
            color
                .append_specified(&mut context, &mut css)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );

        let mut context = SpecifiedSerializationContext::new(
            CssSpecifiedValueSerializationLimits::new(100, 100, 11),
        );
        let mut css = String::new();
        color.append_specified(&mut context, &mut css).unwrap();
        assert_eq!(
            calculation
                .append_specified(&mut context, &mut css)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(css, "transparent");
    }
}

pub(crate) fn format_digits(
    digits: impl Iterator<Item = u8>,
    len: usize,
    exponent: i128,
    negative: bool,
    limit: usize,
) -> Result<String> {
    if len == 0 {
        return if limit == 0 {
            Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit))
        } else {
            Ok("0".into())
        };
    }
    let point = (len as i128)
        .checked_add(exponent)
        .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?;
    let length = if point <= 0 {
        (len as i128)
            .checked_add(2)
            .and_then(|n| n.checked_sub(point))
    } else {
        Some(point.max(len as i128) + i128::from(point < len as i128))
    }
    .and_then(|n| n.checked_add(i128::from(negative)))
    .ok_or_else(|| CssSpecifiedValueSerializationError::new(Kind::ByteLimit))?;
    if length > limit as i128 {
        return Err(CssSpecifiedValueSerializationError::new(Kind::ByteLimit));
    }
    let length = usize::try_from(length)
        .map_err(|_| CssSpecifiedValueSerializationError::new(Kind::CapacityOverflow))?;
    if length > isize::MAX as usize {
        return Err(CssSpecifiedValueSerializationError::new(
            Kind::CapacityOverflow,
        ));
    }
    let mut output = String::with_capacity(length);
    if negative {
        output.push('-');
    }
    if point <= 0 {
        output.push_str("0.");
        output.extend(std::iter::repeat_n('0', (-point) as usize));
    }
    for (index, digit) in digits.enumerate() {
        if point > 0 && index as i128 == point {
            output.push('.');
        }
        output.push(char::from(b'0' + digit));
    }
    if point > len as i128 {
        output.extend(std::iter::repeat_n('0', (point - len as i128) as usize));
    }
    debug_assert_eq!(output.len(), length);
    Ok(output)
}

// Rust's shortest round-trip spelling supplies the nearest decimal candidate.
// At an exact decimal midpoint choose the numerically greater candidate, even
// when the host formatter's last-digit tie rule chose the other neighbor.
pub(crate) fn format_binary64(value: f64) -> String {
    debug_assert!(value.is_finite());
    if value == 0.0 {
        return "0".into();
    }
    let negative = value.is_sign_negative();
    let shortest = value.abs().to_string();
    let (mantissa, exponent) = shortest.split_once(['e', 'E']).unwrap_or((&shortest, "0"));
    let fractional = mantissa
        .find('.')
        .map_or(0, |point| mantissa.len() - point - 1);
    let mut exponent =
        exponent.parse::<i32>().expect("binary64 decimal exponent") - fractional as i32;
    let significant = mantissa.trim_end_matches('0');
    let trailing = mantissa.len() - significant.len();
    // Trimming a whole-number suffix is balanced by an exponent shift; a
    // decimal point itself is skipped, never counted as a coefficient digit.
    exponent += trailing as i32;
    let mut coefficient = significant
        .bytes()
        .filter(|c| *c != b'.')
        .fold(0u64, |n, c| n * 10 + u64::from(c - b'0'));
    let next = if negative {
        coefficient - 1
    } else {
        coefficient + 1
    };
    let midpoint = (coefficient + next) * 5;
    if binary64_equals_decimal(value.abs(), midpoint, exponent - 1) {
        coefficient = next;
    }
    while coefficient.is_multiple_of(10) {
        coefficient /= 10;
        exponent += 1;
    }
    let coefficient = coefficient.to_string();
    format_digits(
        coefficient.bytes().map(|c| c - b'0'),
        coefficient.len(),
        i128::from(exponent),
        negative,
        usize::MAX,
    )
    .expect("finite binary64 text has bounded length")
}

fn binary64_equals_decimal(value: f64, mut coefficient: u64, mut exponent: i32) -> bool {
    if coefficient == 0 {
        return false;
    }
    while coefficient.is_multiple_of(10) {
        coefficient /= 10;
        exponent += 1;
    }
    let bits = value.to_bits();
    let encoded = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mut binary_coefficient, binary_exponent) = if encoded == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u64 << 52), encoded - 1075)
    };
    let mut digits = [0u8; 1100];
    let mut len = 0;
    while binary_coefficient != 0 {
        digits[len] = (binary_coefficient % 10) as u8;
        binary_coefficient /= 10;
        len += 1;
    }
    let multiplier = if binary_exponent < 0 { 5 } else { 2 };
    for _ in 0..binary_exponent.unsigned_abs() {
        let mut carry = 0;
        for digit in &mut digits[..len] {
            let product = *digit * multiplier + carry;
            *digit = product % 10;
            carry = product / 10;
        }
        if carry != 0 {
            digits[len] = carry;
            len += 1;
        }
    }
    let trailing = digits[..len].iter().take_while(|&&d| d == 0).count();
    if binary_exponent.min(0) + trailing as i32 != exponent {
        return false;
    }
    let decimal = coefficient.to_string();
    decimal.len() == len - trailing
        && decimal
            .bytes()
            .map(|c| c - b'0')
            .eq(digits[trailing..len].iter().rev().copied())
}

#[cfg(test)]
mod tests {
    use super::{binary64_equals_decimal, format_binary64};

    #[test]
    fn binary64_text_uses_fixed_notation_and_preserves_retained_value() {
        for (value, expected) in [
            (0.0, "0"),
            (-0.0, "0"),
            (0.1, "0.1"),
            (1.0 / 3.0, "0.3333333333333333"),
            (-1.0 / 3.0, "-0.3333333333333333"),
            (1e20, "100000000000000000000"),
            (1e-7, "0.0000001"),
        ] {
            assert_eq!(format_binary64(value), expected);
        }
        for bits in [
            1,
            2,
            3,
            0x0010_0000_0000_0000,
            0x3fef_ffff_ffff_ffff,
            0x3ff0_0000_0000_0001,
            0x7fef_ffff_ffff_ffff,
        ] {
            for value in [f64::from_bits(bits), -f64::from_bits(bits)] {
                let text = format_binary64(value);
                assert!(!text.contains(['e', 'E']));
                assert_eq!(text.parse::<f64>().unwrap().to_bits(), value.to_bits());
            }
        }
    }

    #[test]
    fn decimal_midpoint_comparison_uses_exact_binary_value() {
        assert!(binary64_equals_decimal(0.125, 125, -3));
        assert!(binary64_equals_decimal(1.5, 150, -2));
        assert!(binary64_equals_decimal(
            9007199254740992.0,
            9007199254740992,
            0
        ));
        assert!(!binary64_equals_decimal(0.1, 1, -1));
        assert!(!binary64_equals_decimal(f64::from_bits(1), 5, -324));
    }
}

impl crate::CssTimeLiteral {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if context.output_suppressed() {
            return Ok(());
        }
        let shift = match self.unit() {
            crate::CssTimeUnit::Seconds => 0,
            crate::CssTimeUnit::Milliseconds => -3,
        };
        let text = format_coefficient(
            self.numeric().representation(),
            shift,
            "s",
            context.remaining_bytes(),
        )?;
        context.append(output, &text)
    }
}
impl crate::CssTimeValue {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        if let Some(literal) = self.literal() {
            return literal.append_specified(context, output);
        }
        crate::numeric::project_calculation_specified_into(
            crate::numeric::SpecifiedCalculationRef::Time(
                self.calculation().expect("checked time branch"),
            ),
            context,
            output,
        )?;
        Ok(())
    }
}
impl crate::CssDuration {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        self.time().append_specified(context, output)
    }
}
impl crate::CssTimeLiteral {
    /// Emits canonical seconds with at most six fractional places.
    ///
    /// Conversion precedes rounding, nearest with ties away from zero. Original
    /// coefficients, units and origins remain exact. This literal operation does
    /// no calculation projection. Calculation values share the six-place text
    /// policy while retaining their arithmetic precision and range limitations.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits seconds using one cumulative input, projection and byte budget.
    ///
    /// Ordinary output rounds to six places without an exponential fallback.
    /// Bytes budget actual rounded text including the seconds suffix; original
    /// input remains unchanged. This literal operation does no math projection.
    /// This operation does not certify recovered syntax as a clean report.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }
}
impl crate::CssTimeValue {
    /// Emits canonical seconds with at most six fractional places.
    ///
    /// Conversion precedes rounding, nearest with ties away from zero. Original
    /// coefficients, units and origins remain exact. Finite calculation text
    /// rounds the projected binary64 value with the same policy; arithmetic
    /// precision and range remain unfinished. Literals perform no math projection.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits seconds using one cumulative input, projection and byte budget.
    ///
    /// Ordinary output rounds to six places without an exponential fallback.
    /// Bytes budget actual rounded text including the seconds suffix; original
    /// input remains unchanged. Finite calculation text rounds the actual binary64
    /// value; its arithmetic, traversal costs and symbolic behavior stay unchanged.
    /// This operation does not certify recovered syntax as a clean report.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }
}
impl crate::CssDuration {
    /// Emits canonical seconds with at most six fractional places.
    ///
    /// Conversion precedes rounding, nearest with ties away from zero. Original
    /// coefficients, units and origins remain exact. Finite calculation text
    /// rounds the projected binary64 value with the same policy; arithmetic
    /// precision and range remain unfinished. Literals perform no math projection.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits seconds using one cumulative input, projection and byte budget.
    ///
    /// Ordinary output rounds to six places without an exponential fallback.
    /// Bytes budget actual rounded text including the seconds suffix; original
    /// input remains unchanged. Finite calculation text rounds the actual binary64
    /// value; its arithmetic, traversal costs and symbolic behavior stay unchanged.
    /// This operation does not certify recovered syntax as a clean report.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }
}

impl crate::CssFrequencyLiteral {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if context.output_suppressed() {
            return Ok(());
        }
        let text = format_coefficient(
            self.numeric().representation(),
            0,
            crate::frequency::suffix(self.unit()),
            context.remaining_bytes(),
        )?;
        context.append(output, &text)
    }
}
impl crate::CssFrequencyValue {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        if let Some(literal) = self.literal() {
            return literal.append_specified(context, output);
        }
        crate::numeric::project_calculation_specified_into(
            crate::numeric::SpecifiedCalculationRef::Frequency(
                self.calculation().expect("checked frequency branch"),
            ),
            context,
            output,
        )?;
        Ok(())
    }
}
impl crate::CssFrequencyLiteral {
    /// Emits ordinary frequencies with the selected lowercase hz or khz unit.
    ///
    /// This authored phase follows the selected primitive serialization policy;
    /// CSSOM's specified/computed phase question remains open. Ordinary text
    /// rounds to six fractional places, nearest with ties away from zero; retained
    /// coefficients stay exact. This literal operation performs no calculation projection.
    /// FrequencyValue math projection has separate precision and range limitations.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits with one cumulative input, projection and byte budget.
    ///
    /// Ordinary units are retained lowercased with at most six fractional places,
    /// nearest with ties away from zero and no scientific fallback. Actual rounded
    /// text determines the byte budget; retained input stays exact. CSSOM's
    /// specified/computed frequency phase question stays open.
    /// This literal operation performs no calculation projection; FrequencyValue
    /// math has separate shared-projector precision and range limitations.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }
}
impl crate::CssFrequencyValue {
    /// Emits ordinary frequencies with the selected lowercase hz or khz unit.
    ///
    /// This authored phase follows the selected primitive serialization policy;
    /// CSSOM's specified/computed phase question remains open. Ordinary text
    /// rounds to six fractional places, nearest with ties away from zero; retained
    /// coefficients stay exact. Finite math text rounds the actual binary64 value
    /// to six places; arithmetic precision and range remain unfinished.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits with one cumulative input, projection and byte budget.
    ///
    /// Ordinary units are retained lowercased with at most six fractional places,
    /// nearest with ties away from zero and no scientific fallback. Actual rounded
    /// text determines the byte budget; retained input stays exact. CSSOM's
    /// specified/computed frequency phase question stays open.
    /// Actual math retains its canonical units and simplification. Finite number
    /// text follows the six-place policy; arithmetic precision and range remain unfinished.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.append_specified(&mut context, &mut output)?;
        Ok(output)
    }
}
