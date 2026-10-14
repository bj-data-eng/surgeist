//! Incremental finite-bit formatting. Variable digits and output belong to the caller.

use super::RoundingMode;
use super::rounded_millionths;

/// (2^53 - 1) * 2^971 has 309 decimal digits. Every finite binary64 integer
/// lies below it; a rounded fractional coefficient fits in u128.
pub(crate) const MAX_FORMAT_DIGITS: usize = 309;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FormatDigit(u8);
impl FormatDigit {
    pub(crate) fn new(digit: u8) -> Result<Self, FormatProtocolError> {
        if digit <= 9 {
            Ok(Self(digit))
        } else {
            Err(FormatProtocolError::InvalidDigit)
        }
    }
    pub(crate) fn digit(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormatKernel {
    PlanFinite,
    TrimMillionths,
    ExtractDigit,
    MultiplyDigit,
    NextMultiply,
    PlanOutput,
    PlanEmission,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormatRequest {
    Compute(FormatKernel),
    ReadDigit { index: usize },
    AppendDigit { index: usize, digit: FormatDigit },
    ReplaceDigit { index: usize, digit: FormatDigit },
    EmitByte { unit: u8 },
    Done(FiniteFormatSummary),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormatReply {
    Computed(FormatKernel),
    Digit { index: usize, digit: FormatDigit },
    Appended { index: usize, digit: FormatDigit },
    Replaced { index: usize, digit: FormatDigit },
    Emitted { unit: u8 },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FiniteFormatSummary {
    pub(crate) is_zero: bool,
    /// Count of successfully acknowledged final ASCII writes.
    pub(crate) emitted: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormatProtocolError {
    NonFinite,
    InvalidDigit,
    WrongReply,
    WrongIndex,
    WrongDigit,
    Terminated,
}

#[derive(Clone, Copy)]
enum Phase {
    Compute(FormatKernel),
    AppendCoefficient(FormatDigit),
    ReadMultiply,
    WriteMultiply(FormatDigit),
    AppendCarry(FormatDigit),
    ReadEmission(usize),
    Emit(u8),
    Done,
}

/// Only fixed registers and a cached request phase. No owner, allocation,
/// borrowed scratch slice or output buffer is stored here. Caller qualification
/// of its real anchor/phase/storage is required before each reply; a logical
/// index alone does not prove that a record belongs to this operation.
pub(crate) struct FiniteFormatCursor {
    bits: u64,
    rounding: RoundingMode,
    negative: bool,
    coefficient: u128,
    multiplies: u32,
    len: usize,
    decimal_exponent: i32,
    index: usize,
    carry: u8,
    read_digit: FormatDigit,
    point: i32,
    extent: Option<usize>,
    emitted: usize,
    phase: Phase,
}
impl FiniteFormatCursor {
    /// The caller admits the actual fixed cursor layout before construction.
    pub(crate) fn new(value: f64, rounding: RoundingMode) -> Result<Self, FormatProtocolError> {
        if !value.is_finite() {
            return Err(FormatProtocolError::NonFinite);
        }
        Ok(Self {
            bits: value.to_bits(),
            rounding,
            negative: false,
            coefficient: 0,
            multiplies: 0,
            len: 0,
            decimal_exponent: 0,
            index: 0,
            carry: 0,
            read_digit: FormatDigit::new(0).expect("zero digit"),
            point: 0,
            extent: None,
            emitted: 0,
            phase: Phase::Compute(FormatKernel::PlanFinite),
        })
    }
    /// Fixed cached metadata; no payload is observed or computed here.
    pub(crate) fn next_request(&self) -> FormatRequest {
        match self.phase {
            Phase::Compute(kernel) => FormatRequest::Compute(kernel),
            Phase::AppendCoefficient(digit) | Phase::AppendCarry(digit) => {
                FormatRequest::AppendDigit {
                    index: self.len,
                    digit,
                }
            }
            Phase::ReadMultiply => FormatRequest::ReadDigit { index: self.index },
            Phase::WriteMultiply(digit) => FormatRequest::ReplaceDigit {
                index: self.index,
                digit,
            },
            Phase::ReadEmission(index) => FormatRequest::ReadDigit { index },
            Phase::Emit(unit) => FormatRequest::EmitByte { unit },
            Phase::Done => FormatRequest::Done(FiniteFormatSummary {
                is_zero: self.len == 0,
                emitted: self.emitted,
            }),
        }
    }
    /// Exact final length becomes available after PlanOutput, before any write.
    pub(crate) fn planned_length(&self) -> Option<usize> {
        self.extent
    }
    /// The caller has already admitted the described operation. Rejected
    /// replies leave every register and pending request intact. Successful
    /// append/replace/emit replies acknowledge the actual caller mutation;
    /// refusal before acknowledgement cannot advance a nonexistent record.
    pub(crate) fn complete(&mut self, reply: FormatReply) -> Result<(), FormatProtocolError> {
        match (self.next_request(), reply) {
            (FormatRequest::Done(_), _) => return Err(FormatProtocolError::Terminated),
            (FormatRequest::Compute(expected), FormatReply::Computed(actual)) => {
                if expected != actual {
                    return Err(FormatProtocolError::WrongReply);
                }
                self.compute(expected);
            }
            (
                FormatRequest::ReadDigit { index },
                FormatReply::Digit {
                    index: actual,
                    digit,
                },
            ) => {
                if index != actual {
                    return Err(FormatProtocolError::WrongIndex);
                }
                match self.phase {
                    Phase::ReadMultiply => {
                        self.read_digit = digit;
                        self.phase = Phase::Compute(FormatKernel::MultiplyDigit);
                    }
                    Phase::ReadEmission(_) => self.phase = Phase::Emit(b'0' + digit.digit()),
                    _ => unreachable!("read request has a read phase"),
                }
            }
            (
                FormatRequest::AppendDigit { index, digit },
                FormatReply::Appended {
                    index: actual,
                    digit: stored,
                },
            ) => {
                check_write(index, digit, actual, stored)?;
                self.len += 1;
                debug_assert!(self.len <= MAX_FORMAT_DIGITS);
                self.phase = match self.phase {
                    Phase::AppendCoefficient(_) if self.coefficient != 0 => {
                        Phase::Compute(FormatKernel::ExtractDigit)
                    }
                    Phase::AppendCoefficient(_) if self.multiplies != 0 => Phase::ReadMultiply,
                    Phase::AppendCoefficient(_) => Phase::Compute(FormatKernel::PlanOutput),
                    Phase::AppendCarry(_) => Phase::Compute(FormatKernel::NextMultiply),
                    _ => unreachable!("append request has an append phase"),
                };
            }
            (
                FormatRequest::ReplaceDigit { index, digit },
                FormatReply::Replaced {
                    index: actual,
                    digit: stored,
                },
            ) => {
                check_write(index, digit, actual, stored)?;
                self.index += 1;
                self.phase = if self.index < self.len {
                    Phase::ReadMultiply
                } else if self.carry != 0 {
                    Phase::AppendCarry(
                        FormatDigit::new(self.carry).expect("binary multiplication carry"),
                    )
                } else {
                    Phase::Compute(FormatKernel::NextMultiply)
                };
            }
            (FormatRequest::EmitByte { unit }, FormatReply::Emitted { unit: actual }) => {
                if unit != actual {
                    return Err(FormatProtocolError::WrongDigit);
                }
                self.emitted += 1;
                self.phase = if Some(self.emitted) == self.extent {
                    Phase::Done
                } else {
                    Phase::Compute(FormatKernel::PlanEmission)
                };
            }
            _ => return Err(FormatProtocolError::WrongReply),
        }
        Ok(())
    }
    /// Every branch is fixed scalar work: no digit/exponent loop or allocation.
    fn compute(&mut self, kernel: FormatKernel) {
        match kernel {
            FormatKernel::PlanFinite => {
                self.negative = self.bits >> 63 != 0;
                let encoded = ((self.bits >> 52) & 0x7ff) as i32;
                let fraction = self.bits & ((1_u64 << 52) - 1);
                let (significand, exponent) = if encoded == 0 {
                    (fraction, -1074)
                } else {
                    ((1_u64 << 52) | fraction, encoded - 1075)
                };
                if significand == 0 {
                    self.phase = Phase::Compute(FormatKernel::PlanOutput);
                } else if exponent < 0 {
                    self.coefficient =
                        rounded_millionths(significand, exponent, self.negative, self.rounding);
                    self.decimal_exponent = -6;
                    self.phase = if self.coefficient == 0 {
                        Phase::Compute(FormatKernel::PlanOutput)
                    } else {
                        Phase::Compute(FormatKernel::TrimMillionths)
                    };
                } else {
                    self.coefficient = u128::from(significand);
                    self.multiplies = exponent as u32;
                    self.phase = Phase::Compute(FormatKernel::ExtractDigit);
                }
            }
            FormatKernel::TrimMillionths => {
                if self.coefficient.is_multiple_of(10) {
                    self.coefficient /= 10;
                    self.decimal_exponent += 1;
                } else {
                    self.phase = Phase::Compute(FormatKernel::ExtractDigit);
                }
            }
            FormatKernel::ExtractDigit => {
                let digit = (self.coefficient % 10) as u8;
                self.coefficient /= 10;
                self.phase = Phase::AppendCoefficient(
                    FormatDigit::new(digit).expect("decimal remainder digit"),
                );
            }
            FormatKernel::MultiplyDigit => {
                let doubled = self.read_digit.digit() * 2 + self.carry;
                self.carry = doubled / 10;
                self.phase = Phase::WriteMultiply(
                    FormatDigit::new(doubled % 10).expect("decimal remainder digit"),
                );
            }
            FormatKernel::NextMultiply => {
                self.multiplies -= 1;
                self.index = 0;
                self.carry = 0;
                self.phase = if self.multiplies == 0 {
                    Phase::Compute(FormatKernel::PlanOutput)
                } else {
                    Phase::ReadMultiply
                };
            }
            FormatKernel::PlanOutput => {
                if self.len == 0 {
                    self.negative = false;
                    self.point = 1;
                    self.extent = Some(1);
                } else {
                    self.point = self.len as i32 + self.decimal_exponent;
                    let body = if self.point <= 0 {
                        self.len + 2 + self.point.unsigned_abs() as usize
                    } else {
                        (self.point as usize).max(self.len)
                            + usize::from((self.point as usize) < self.len)
                    };
                    self.extent = Some(body + usize::from(self.negative));
                }
                self.phase = Phase::Compute(FormatKernel::PlanEmission);
            }
            FormatKernel::PlanEmission => {
                let position = self.emitted;
                if self.negative && position == 0 {
                    self.phase = Phase::Emit(b'-');
                    return;
                }
                let position = position - usize::from(self.negative);
                if self.len == 0 {
                    self.phase = Phase::Emit(b'0');
                } else if self.point <= 0 {
                    let prefix = 2 + self.point.unsigned_abs() as usize;
                    self.phase = if position == 1 {
                        Phase::Emit(b'.')
                    } else if position < prefix {
                        Phase::Emit(b'0')
                    } else {
                        Phase::ReadEmission(self.len - 1 - (position - prefix))
                    };
                } else {
                    let point = self.point as usize;
                    self.phase = if point < self.len && position == point {
                        Phase::Emit(b'.')
                    } else {
                        let index = position - usize::from(point < self.len && position > point);
                        if index >= self.len {
                            Phase::Emit(b'0')
                        } else {
                            Phase::ReadEmission(self.len - 1 - index)
                        }
                    };
                }
            }
        }
    }
}
fn check_write(
    index: usize,
    digit: FormatDigit,
    actual: usize,
    stored: FormatDigit,
) -> Result<(), FormatProtocolError> {
    if index != actual {
        Err(FormatProtocolError::WrongIndex)
    } else if digit != stored {
        Err(FormatProtocolError::WrongDigit)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ordinary, actual caller storage. The cursor never receives these buffers.
    fn reply(
        request: FormatRequest,
        scratch: &mut Vec<FormatDigit>,
        output: &mut String,
    ) -> FormatReply {
        match request {
            FormatRequest::Compute(kernel) => FormatReply::Computed(kernel),
            FormatRequest::ReadDigit { index } => FormatReply::Digit {
                index,
                digit: scratch[index],
            },
            FormatRequest::AppendDigit { index, digit } => {
                assert_eq!(index, scratch.len());
                scratch.try_reserve(1).unwrap();
                scratch.push(digit);
                FormatReply::Appended { index, digit }
            }
            FormatRequest::ReplaceDigit { index, digit } => {
                scratch[index] = digit;
                FormatReply::Replaced { index, digit }
            }
            FormatRequest::EmitByte { unit } => {
                output.try_reserve(1).unwrap();
                output.push(char::from(unit));
                FormatReply::Emitted { unit }
            }
            FormatRequest::Done(_) => panic!("done has no completion reply"),
        }
    }
    fn drive(
        cursor: &mut FiniteFormatCursor,
        scratch: &mut Vec<FormatDigit>,
        output: &mut String,
        limit: usize,
    ) -> Result<FiniteFormatSummary, FormatRequest> {
        // Explicit fixture termination guard, not an estimated parser fee.
        for _ in 0..1_000_000 {
            let request = cursor.next_request();
            if let FormatRequest::Done(summary) = request {
                return Ok(summary);
            }
            if matches!(request, FormatRequest::EmitByte { .. }) && output.len() == limit {
                return Err(request);
            }
            let reply = reply(request, scratch, output);
            cursor.complete(reply).unwrap();
        }
        panic!("finite binary64 formatter did not terminate");
    }

    #[test]
    fn literal_bits_round_trim_carry_and_emit_only_final_text() {
        for (value, rounding, expected) in [
            (-0.0, RoundingMode::AwayFromZero, "0"),
            (f64::from_bits(1), RoundingMode::AwayFromZero, "0"),
            (f64::MIN_POSITIVE, RoundingMode::TowardPositiveInfinity, "0"),
            (1.0 / 3.0, RoundingMode::AwayFromZero, "0.333333"),
            (1.25, RoundingMode::AwayFromZero, "1.25"),
            (-12.5, RoundingMode::AwayFromZero, "-12.5"),
            (0.0001, RoundingMode::AwayFromZero, "0.0001"),
            (0.9999996, RoundingMode::AwayFromZero, "1"),
            (999.9999996, RoundingMode::AwayFromZero, "1000"),
            (
                f64::from_bits(0x3f80000000000000),
                RoundingMode::AwayFromZero,
                "0.007813",
            ),
            (
                -f64::from_bits(0x3f80000000000000),
                RoundingMode::AwayFromZero,
                "-0.007813",
            ),
            (
                -f64::from_bits(0x3f80000000000000),
                RoundingMode::TowardPositiveInfinity,
                "-0.007812",
            ),
            (
                f64::from_bits(0x3f7fffffffffffff),
                RoundingMode::TowardPositiveInfinity,
                "0.007812",
            ),
            (
                f64::from_bits(0x3f80000000000001),
                RoundingMode::TowardPositiveInfinity,
                "0.007813",
            ),
            (
                f64::from_bits(0x43ab_c16d_674e_c801),
                RoundingMode::AwayFromZero,
                "1000000000000000128",
            ),
        ] {
            let mut cursor = FiniteFormatCursor::new(value, rounding).unwrap();
            let mut scratch = Vec::new();
            let mut output = String::new();
            let summary = drive(&mut cursor, &mut scratch, &mut output, expected.len()).unwrap();
            assert_eq!(output, expected);
            assert_eq!(cursor.planned_length(), Some(expected.len()));
            assert_eq!(summary.emitted, expected.len());
            assert_eq!(summary.is_zero, expected == "0");
            assert!(scratch.len() <= MAX_FORMAT_DIGITS);
        }
    }

    #[test]
    fn maximum_finite_integer_uses_actual_309_digit_caller_scratch() {
        const INTEGER: &str = concat!(
            "179769313486231570814527423731704356798070567525844996598917476803157260",
            "780028538760589558632766878171540458953514382464234321326889464182768467",
            "546703537516986049910576551282076245490090389328944075868508455133942304",
            "583236903222948165808559332123348274797826204144723168738177180919299881",
            "250404026184124858368"
        );
        for value in [f64::MAX, -f64::MAX] {
            let expected = if value.is_sign_negative() {
                format!("-{INTEGER}")
            } else {
                INTEGER.into()
            };
            let mut cursor = FiniteFormatCursor::new(value, RoundingMode::AwayFromZero).unwrap();
            let mut scratch = Vec::new();
            let mut output = String::new();
            let summary = drive(&mut cursor, &mut scratch, &mut output, expected.len()).unwrap();
            assert_eq!(scratch.len(), 309);
            assert_eq!(output, expected);
            assert_eq!(summary.emitted, expected.len());
        }
    }

    #[test]
    fn wrong_reply_index_and_generated_digit_leave_pending_request_and_storage_intact() {
        let mut cursor =
            FiniteFormatCursor::new(9_007_199_254_740_992.0, RoundingMode::AwayFromZero).unwrap();
        let mut scratch = Vec::new();
        let mut output = String::new();
        let mut append = false;
        let mut read = false;
        let mut replace = false;
        let mut emit = false;
        for _ in 0..10_000 {
            let request = cursor.next_request();
            let wrong = match request {
                FormatRequest::Compute(_) => Some((
                    FormatReply::Emitted { unit: b'0' },
                    FormatProtocolError::WrongReply,
                )),
                FormatRequest::AppendDigit { index, digit } => {
                    append = true;
                    assert_eq!(
                        cursor.complete(FormatReply::Appended {
                            index: index + 1,
                            digit
                        }),
                        Err(FormatProtocolError::WrongIndex)
                    );
                    Some((
                        FormatReply::Appended {
                            index,
                            digit: FormatDigit::new((digit.digit() + 1) % 10).unwrap(),
                        },
                        FormatProtocolError::WrongDigit,
                    ))
                }
                FormatRequest::ReadDigit { index } => {
                    read = true;
                    Some((
                        FormatReply::Digit {
                            index: index + 1,
                            digit: FormatDigit::new(0).unwrap(),
                        },
                        FormatProtocolError::WrongIndex,
                    ))
                }
                FormatRequest::ReplaceDigit { index, digit } => {
                    replace = true;
                    assert_eq!(
                        cursor.complete(FormatReply::Replaced {
                            index: index + 1,
                            digit
                        }),
                        Err(FormatProtocolError::WrongIndex)
                    );
                    Some((
                        FormatReply::Replaced {
                            index,
                            digit: FormatDigit::new((digit.digit() + 1) % 10).unwrap(),
                        },
                        FormatProtocolError::WrongDigit,
                    ))
                }
                FormatRequest::EmitByte { unit } => {
                    emit = true;
                    Some((
                        FormatReply::Emitted { unit: unit ^ 1 },
                        FormatProtocolError::WrongDigit,
                    ))
                }
                FormatRequest::Done(_) => None,
            };
            if let Some((wrong, error)) = wrong {
                let stored = scratch.clone();
                let emitted = output.clone();
                assert_eq!(cursor.complete(wrong), Err(error));
                assert_eq!(cursor.next_request(), request);
                assert_eq!(scratch, stored);
                assert_eq!(output, emitted);
                let correct = reply(request, &mut scratch, &mut output);
                cursor.complete(correct).unwrap();
            } else {
                assert_eq!(
                    cursor.complete(FormatReply::Emitted { unit: b'0' }),
                    Err(FormatProtocolError::Terminated)
                );
                assert_eq!(cursor.next_request(), request);
                assert!(append && read && replace && emit);
                assert_eq!(output, "9007199254740992");
                return;
            }
        }
        panic!("small integer cursor did not terminate");
    }

    #[test]
    fn refused_actual_append_and_replace_leave_record_with_the_caller() {
        let mut scratch = Vec::new();
        let mut output = String::new();
        let (index, before, recovered) = {
            let mut cursor =
                FiniteFormatCursor::new(9_007_199_254_740_992.0, RoundingMode::AwayFromZero)
                    .unwrap();
            let mut refused_append = false;
            let mut retained = None;
            for _ in 0..10_000 {
                let request = cursor.next_request();
                if let FormatRequest::AppendDigit { index, digit } = request
                    && !refused_append
                {
                    // An impossible actual Vec capacity submission refuses
                    // before mutation. The generated record stays with this
                    // owner, with no acknowledgement or child-length advance.
                    let before_capacity = scratch.capacity();
                    assert!(scratch.try_reserve(usize::MAX).is_err());
                    let recovered = digit;
                    assert_eq!(cursor.next_request(), request);
                    assert_eq!(scratch.len(), index);
                    assert_eq!(scratch.capacity(), before_capacity);
                    assert_eq!(recovered, digit);
                    refused_append = true;
                }
                if let FormatRequest::ReplaceDigit { index, digit } = request {
                    // Refused fixed-write admission: retain old record and
                    // replacement, send no reply, then abandon the cursor.
                    assert_eq!(cursor.next_request(), request);
                    retained = Some((index, scratch[index], digit));
                    break;
                }
                let actual = reply(request, &mut scratch, &mut output);
                cursor.complete(actual).unwrap();
            }
            assert!(refused_append);
            retained.expect("actual multiply replacement was reached")
        };
        // Cursor's lexical lifetime ended without touching caller storage.
        assert_eq!(scratch[index], before);
        assert!(recovered.digit() <= 9);
        assert!(output.is_empty());
    }

    #[test]
    fn one_short_final_output_keeps_real_prefix_and_pending_byte_until_acknowledged() {
        for (value, expected) in [(0.0, "0"), (-0.0078125, "-0.007813"), (0.9999996, "1")] {
            let mut cursor = FiniteFormatCursor::new(value, RoundingMode::AwayFromZero).unwrap();
            let mut scratch = Vec::new();
            let mut output = String::new();
            let pending =
                drive(&mut cursor, &mut scratch, &mut output, expected.len() - 1).unwrap_err();
            assert_eq!(output, &expected[..expected.len() - 1]);
            assert_eq!(cursor.planned_length(), Some(expected.len()));
            assert_eq!(cursor.next_request(), pending);
            let summary = drive(&mut cursor, &mut scratch, &mut output, expected.len()).unwrap();
            assert_eq!(output, expected);
            assert_eq!(summary.emitted, expected.len());
        }
    }

    #[test]
    fn nonfinite_and_nondecimal_digit_construction_is_typed_refusal() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                FiniteFormatCursor::new(value, RoundingMode::AwayFromZero),
                Err(FormatProtocolError::NonFinite)
            ));
        }
        for digit in [10, 128, 255] {
            assert_eq!(
                FormatDigit::new(digit),
                Err(FormatProtocolError::InvalidDigit)
            );
        }
        for digit in 0..=9 {
            assert_eq!(FormatDigit::new(digit).unwrap().digit(), digit);
        }
    }
}
