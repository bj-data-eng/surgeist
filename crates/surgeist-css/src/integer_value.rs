//! Exact ordinary signed integers and specified integer/z-index serialization.

use crate::{
    CssComponentValue, CssComponentValueRef, CssNumericTokenKind, CssSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssValueTokenRef, CssZIndexValue,
    specified_serialization::SpecifiedSerializationContext,
};

/// The env() index grammar only needs lexical nonnegativity. Borrow the
/// original checked token instead of retaining another integer/component graph.
pub(crate) fn nonnegative_integer_literal_controlled(
    component: &CssComponentValue,
    control: &crate::utf16_value::Control<'_>,
) -> Result<bool, ()> {
    control.step()?;
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        return Ok(false);
    };
    if number.kind() != CssNumericTokenKind::Integer {
        return Ok(false);
    }
    let text = number.representation();
    if !text.starts_with('-') {
        return Ok(true);
    }
    // Exactly the same signed-zero rule as CssNumericLiteral::<CssInteger>::is_negative;
    // each actual digit read is admitted before looking at the next digit.
    for digit in text.as_bytes().iter().skip(1) {
        control.step()?;
        if *digit != b'0' {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod controlled_index_tests {

    use super::*;
    use crate::{
        CssComponentValueLimits, CssUtf16InputLimits, CssUtf16PropertyValueLimits,
        CssUtf16ValueAdmission,
    };
    use std::cell::Cell;

    fn limits() -> CssUtf16PropertyValueLimits {
        CssUtf16PropertyValueLimits {
            input: CssUtf16InputLimits::default(),
            syntax: CssComponentValueLimits::default(),
            max_work: 10_000,
            max_live_bytes: 0,
            max_retained_bytes: 0,
        }
    }

    #[test]
    fn borrowed_index_admits_huge_magnitudes_and_signed_zero_without_allocating_a_graph() {
        for (source, expected) in [
            ("0", true),
            ("+000", true),
            ("-000", true),
            ("-0001", false),
            ("9999999999999999999999999999999999999999", true),
            ("1.0", false),
            ("1e2", false),
            ("none", false),
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let admit = |action| {
                assert!(matches!(action, CssUtf16ValueAdmission::Work));
                Ok(())
            };
            let release = |_| {};
            let control = crate::utf16_value::Control::new(limits(), &admit, &release);
            assert_eq!(
                nonnegative_integer_literal_controlled(&values.items()[0], &control),
                Ok(expected),
                "{source}"
            );
            assert_eq!(control.live().bytes(), 0);
        }
    }

    #[test]
    fn real_signed_zero_scan_is_cancellable_and_keeps_original_token() {
        let values =
            crate::parse_component_values("-000000000000000000000000000000000000").unwrap();
        let before = values.clone();
        let work = Cell::new(0);
        let admit = |_| {
            work.set(work.get() + 1);
            if work.get() == 9 { Err(()) } else { Ok(()) }
        };
        let release = |_| {};
        let control = crate::utf16_value::Control::new(limits(), &admit, &release);
        assert_eq!(
            nonnegative_integer_literal_controlled(&values.items()[0], &control),
            Err(())
        );
        assert_eq!(
            control.failure(),
            Some(crate::utf16_value::Failure::Admission)
        );
        assert_eq!(work.get(), 9);
        assert_eq!(control.live().bytes(), 0);
        assert_eq!(values, before);
    }
}

// Input has already been checked as a lexical integer. Accumulating negatively
// admits i32::MIN without an overflowing positive intermediate. Leading zeroes
// never consume an artificial significant-digit allowance.
pub(crate) fn exact_i32(text: &str) -> Option<i32> {
    let negative = text.starts_with('-');
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    let mut value = 0_i32;
    for digit in digits.bytes() {
        value = value
            .checked_mul(10)?
            .checked_sub(i32::from(digit - b'0'))?;
    }
    if negative {
        Some(value)
    } else {
        value.checked_neg()
    }
}

impl CssZIndexValue {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<(), CssSerializationError> {
        match self {
            Self::Auto => {
                context.charge_nodes(1)?;
                context.append(output, "auto")
            }
            Self::Integer(value) => value.append_specified(context, output),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssZIndexValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssZIndexValue);

pub(crate) fn serialize_integer_digits(
    text: &str,
    byte_limit: usize,
) -> Result<String, CssSerializationError> {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    let negative = text.starts_with('-');
    let digits = text
        .strip_prefix(['+', '-'])
        .unwrap_or(text)
        .trim_start_matches('0');
    let (digits, negative) = if digits.is_empty() {
        ("0", false)
    } else {
        (digits, negative)
    };
    let length = digits
        .len()
        .checked_add(usize::from(negative))
        .ok_or_else(|| CssSerializationError::new(Kind::CapacityOverflow))?;
    if length > byte_limit {
        return Err(CssSerializationError::new(Kind::ByteLimit));
    }
    let mut output = String::with_capacity(length);
    if negative {
        output.push('-');
    }
    output.push_str(digits);
    Ok(output)
}

#[cfg(test)]
mod z_index_serialization_tests {
    use super::*;
    use crate::CssSerializationLimits;
    use crate::{CssCalculation, CssInteger, CssNumeric, CssNumericLiteral};
    use CssSpecifiedValueSerializationErrorKind as Kind;

    #[test]
    fn z_index_siblings_share_input_projection_and_utf8_byte_budgets() {
        let first = CssZIndexValue::Auto;
        let second =
            CssZIndexValue::Integer(CssNumeric::from_literal(CssNumericLiteral::from_i32(-2)));
        let before = (first.clone(), second.clone());
        // The separator is a UTF-8 accounting probe in the shared private writer,
        // not a proposed CSS property grammar. "autoé-2" occupies eight bytes.
        for (limits, failure) in [
            (CssSerializationLimits::new(2, 2, 8), None),
            (
                CssSerializationLimits::new(1, 2, 8),
                Some(Kind::InputNodeLimit),
            ),
            (
                CssSerializationLimits::new(2, 1, 8),
                Some(Kind::ProjectionNodeLimit),
            ),
            (CssSerializationLimits::new(2, 2, 7), Some(Kind::ByteLimit)),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            first.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, "é").unwrap();
            let result = second.append_specified(&mut context, &mut output);
            if let Some(kind) = failure {
                assert_eq!(result.unwrap_err().kind(), kind);
                assert_eq!(output, "autoé");
            } else {
                result.unwrap();
                assert_eq!(output, "autoé-2");
            }
            assert_eq!((&first, &second), (&before.0, &before.1));
        }
    }

    #[test]
    fn z_index_math_delegates_into_the_existing_cumulative_context() {
        let first = CssZIndexValue::Auto;
        // A math root; a bare root would re-enter literal admission.
        let second = CssZIndexValue::Integer(
            CssNumeric::<CssInteger>::try_from_calculation(
                CssCalculation::try_from_components(
                    crate::parse_component_values("calc(2)").unwrap(),
                    Default::default(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let before = second.clone();
        for (limits, failure) in [
            (CssSerializationLimits::new(100, 100, 12), None),
            (
                CssSerializationLimits::new(1, 100, 12),
                Some(Kind::InputNodeLimit),
            ),
            (
                CssSerializationLimits::new(100, 1, 12),
                Some(Kind::ProjectionNodeLimit),
            ),
            (
                CssSerializationLimits::new(100, 100, 11),
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
                // Internal math appends incrementally. Public serialization
                // discards this scratch output on error; it does not promise
                // rollback of a caller's private buffer.
                assert!(output.starts_with("auto "));
                assert!(output.len() <= limits.max_css_bytes());
                assert_eq!(
                    context.remaining_bytes(),
                    limits.max_css_bytes() - output.len()
                );
            } else {
                result.unwrap();
                assert_eq!(output, "auto calc(2)");
            }
            assert_eq!(second, before);
        }
    }
}
