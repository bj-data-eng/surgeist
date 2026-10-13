//! Incremental exact rounding of already grammar-checked decimal digits.
//!
//! Positive encodings are ordered. The threshold above encoding n is the exact
//! dyadic midpoint `(2 * significand(n) + 1) * 2^(exponent(n) - 1)`.
//! Zero uses exponent -1074; the last finite encoding uses exponent 971.
//! Thus even the zero threshold is represented exactly (2^-1075), and the
//! overflow threshold is (2^54 - 1) * 2^970. Equality selects the even encoding;
//! equality at the odd maximum finite encoding selects overflow.
//!
//! A midpoint starts with at most 17 decimal digits. Its binary power is in
//! -1075..=970. Multiplication by 5 (negative powers, with decimal scale) or 2
//! increases length by at most one per iteration. Therefore <=1092 scratch
//! digits suffice, without logarithms, token-sized integers or a fixed array.
//! The caller owns the separate scratch interval, admission and abandonment.

use std::cmp::Ordering;

const OVERFLOW_ENCODING: u64 = 0x7ff0_0000_0000_0000;
const MAX_BOUNDARY_DIGITS: usize = 1100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DecimalLayout {
    negative: bool,
    mantissa_digits: usize,
    fractional_digits: usize,
    exponent_negative: bool,
    exponent_digits: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalLayoutError {
    EmptyMantissa,
    EmptyExponent,
    FractionalExtent,
    IndexOverflow,
}
impl DecimalLayout {
    /// Without an explicit exponent, the grammar supplies one exponent digit 0.
    pub(crate) fn new(
        negative: bool,
        mantissa_digits: usize,
        fractional_digits: usize,
        exponent_negative: bool,
        exponent_digits: usize,
    ) -> Result<Self, DecimalLayoutError> {
        if mantissa_digits == 0 {
            return Err(DecimalLayoutError::EmptyMantissa);
        }
        if exponent_digits == 0 {
            return Err(DecimalLayoutError::EmptyExponent);
        }
        if fractional_digits > mantissa_digits {
            return Err(DecimalLayoutError::FractionalExtent);
        }
        mantissa_digits
            .checked_add(exponent_digits)
            .ok_or(DecimalLayoutError::IndexOverflow)?;
        Ok(Self {
            negative,
            mantissa_digits,
            fractional_digits,
            exponent_negative,
            exponent_digits,
        })
    }
}

/// One admitted scratch digit. This record carries no storage/owner authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DecimalDigit(u8);
impl DecimalDigit {
    pub(crate) fn new(digit: u8) -> Result<Self, DecimalProtocolError> {
        if digit > 9 {
            Err(DecimalProtocolError::InvalidDigit)
        } else {
            Ok(Self(digit))
        }
    }
    pub(crate) fn digit(&self) -> u8 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalInput {
    Mantissa,
    Exponent,
}
/// Each kernel only inspects the cursor's fixed registers and transitions once.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalKernel {
    Normalize,
    SelectBoundary,
    SeedDigit,
    BeginMultiplication,
    FinishMultiplication,
    CompareMagnitude,
    BeginComparison,
    UpdateSearch,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DecimalOutcome {
    Finite(f64),
    Overflow { negative: bool },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalArithmeticError {
    BoundaryExtent,
    SearchExtent,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DecimalRequest {
    ReadInput { kind: DecimalInput, index: usize },
    ReadScratch { index: usize },
    AppendScratch { index: usize, digit: DecimalDigit },
    ReplaceScratch { index: usize, digit: DecimalDigit },
    Compute(DecimalKernel),
    Done(DecimalOutcome),
    Failed(DecimalArithmeticError),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalReply {
    Input {
        kind: DecimalInput,
        index: usize,
        digit: u8,
    },
    Scratch {
        index: usize,
        digit: DecimalDigit,
    },
    Appended {
        index: usize,
    },
    Replaced {
        index: usize,
    },
    Computed(DecimalKernel),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalProtocolError {
    WrongReply,
    InvalidDigit,
    Terminal,
}
#[derive(Clone, Copy, Debug)]
enum Stage {
    Mantissa,
    Exponent,
    Kernel,
    SeedWrite,
    MultiplyRead,
    MultiplyWrite,
    CarryWrite,
    CompareInput,
    CompareScratch,
    Terminal,
}

/// Allocation-free fixed-register state. `next_request` only lends its cache.
#[derive(Debug)]
pub(crate) struct DecimalCursor {
    layout: DecimalLayout,
    request: DecimalRequest,
    stage: Stage,
    scan: usize,
    first: Option<usize>,
    last: usize,
    significant: usize,
    exponent: u128,
    huge_exponent: bool,
    magnitude: i128,
    low: u64,
    high: u64,
    candidate: u64,
    comparisons: u8,
    seed: u64,
    power: u16,
    factor: u8,
    scale: i128,
    stored: usize,
    length: usize,
    index: usize,
    carry: u8,
    compare_position: usize,
    authored_digit: u8,
    comparison: Ordering,
}

impl DecimalCursor {
    pub(crate) fn new(layout: DecimalLayout) -> Self {
        Self {
            layout,
            request: DecimalRequest::ReadInput {
                kind: DecimalInput::Mantissa,
                index: 0,
            },
            stage: Stage::Mantissa,
            scan: 0,
            first: None,
            last: 0,
            significant: 0,
            exponent: 0,
            huge_exponent: false,
            magnitude: 0,
            low: 0,
            high: OVERFLOW_ENCODING,
            candidate: 0,
            comparisons: 0,
            seed: 0,
            power: 0,
            factor: 0,
            scale: 0,
            stored: 0,
            length: 0,
            index: 0,
            carry: 0,
            compare_position: 0,
            authored_digit: 0,
            comparison: Ordering::Equal,
        }
    }
    pub(crate) fn next_request(&self) -> &DecimalRequest {
        &self.request
    }
    /// A refusal leaves every register and the pending request unchanged.
    /// Input reads inspect one supplied digit; scratch reads inspect one record;
    /// write acknowledgements inspect no caller storage and commit only indices.
    pub(crate) fn complete(&mut self, reply: DecimalReply) -> Result<(), DecimalProtocolError> {
        match (self.request, reply) {
            (
                DecimalRequest::ReadInput { kind: a, index: i },
                DecimalReply::Input {
                    kind: b,
                    index: j,
                    digit,
                },
            ) if a == b && i == j => {
                if digit > 9 {
                    return Err(DecimalProtocolError::InvalidDigit);
                }
            }
            (DecimalRequest::ReadScratch { index: i }, DecimalReply::Scratch { index: j, .. })
                if i == j => {}
            (
                DecimalRequest::AppendScratch { index: i, .. },
                DecimalReply::Appended { index: j },
            ) if i == j => {}
            (
                DecimalRequest::ReplaceScratch { index: i, .. },
                DecimalReply::Replaced { index: j },
            ) if i == j => {}
            (DecimalRequest::Compute(a), DecimalReply::Computed(b)) if a == b => {}
            (DecimalRequest::Done(_) | DecimalRequest::Failed(_), _) => {
                return Err(DecimalProtocolError::Terminal);
            }
            _ => return Err(DecimalProtocolError::WrongReply),
        }
        match reply {
            DecimalReply::Input { digit, .. } => self.input(digit),
            DecimalReply::Scratch { digit, .. } => self.scratch(digit.digit()),
            DecimalReply::Appended { .. } => {
                self.stored += 1;
                self.written();
            }
            DecimalReply::Replaced { .. } => self.written(),
            DecimalReply::Computed(kernel) => self.compute(kernel),
        }
        Ok(())
    }
    fn kernel(&mut self, kernel: DecimalKernel) {
        self.stage = Stage::Kernel;
        self.request = DecimalRequest::Compute(kernel);
    }
    fn finish(&mut self, encoding: u64) {
        self.stage = Stage::Terminal;
        self.request = DecimalRequest::Done(if encoding == OVERFLOW_ENCODING {
            DecimalOutcome::Overflow {
                negative: self.layout.negative,
            }
        } else {
            DecimalOutcome::Finite(f64::from_bits(
                encoding | (u64::from(self.layout.negative) << 63),
            ))
        });
    }
    fn fail(&mut self, error: DecimalArithmeticError) {
        self.stage = Stage::Terminal;
        self.request = DecimalRequest::Failed(error);
    }
    fn input(&mut self, digit: u8) {
        match self.stage {
            Stage::Mantissa => {
                if digit != 0 {
                    if self.first.is_none() {
                        self.first = Some(self.scan);
                    }
                    self.last = self.scan;
                }
                self.scan += 1;
                if self.scan == self.layout.mantissa_digits {
                    self.scan = 0;
                    self.stage = Stage::Exponent;
                    self.request = DecimalRequest::ReadInput {
                        kind: DecimalInput::Exponent,
                        index: 0,
                    };
                } else {
                    self.request = DecimalRequest::ReadInput {
                        kind: DecimalInput::Mantissa,
                        index: self.scan,
                    };
                }
            }
            Stage::Exponent => {
                // Any source-length correction is <=mantissa_digits. Once this
                // bound is exceeded, its sign cannot be cancelled into the
                // binary64 range. Still consume every declared exponent digit.
                let limit = self.layout.mantissa_digits as u128 + 2048;
                if !self.huge_exponent {
                    let value = self.exponent * 10 + u128::from(digit);
                    if value > limit {
                        self.huge_exponent = true;
                    } else {
                        self.exponent = value;
                    }
                }
                self.scan += 1;
                if self.scan == self.layout.exponent_digits {
                    self.kernel(DecimalKernel::Normalize);
                } else {
                    self.request = DecimalRequest::ReadInput {
                        kind: DecimalInput::Exponent,
                        index: self.scan,
                    };
                }
            }
            Stage::CompareInput => {
                self.authored_digit = digit;
                self.read_boundary_digit();
            }
            _ => unreachable!("validated input request stage"),
        }
    }
    fn write_digit(&mut self, index: usize, digit: u8) {
        if index >= MAX_BOUNDARY_DIGITS {
            self.fail(DecimalArithmeticError::BoundaryExtent);
            return;
        }
        let digit = DecimalDigit(digit);
        self.request = if index < self.stored {
            DecimalRequest::ReplaceScratch { index, digit }
        } else {
            DecimalRequest::AppendScratch { index, digit }
        };
    }
    fn written(&mut self) {
        match self.stage {
            Stage::SeedWrite => {
                self.length += 1;
                if self.seed == 0 {
                    self.kernel(DecimalKernel::BeginMultiplication);
                } else {
                    self.kernel(DecimalKernel::SeedDigit);
                }
            }
            Stage::MultiplyWrite => {
                self.index += 1;
                if self.index < self.length {
                    self.stage = Stage::MultiplyRead;
                    self.request = DecimalRequest::ReadScratch { index: self.index };
                } else if self.carry != 0 {
                    self.stage = Stage::CarryWrite;
                    self.write_digit(self.length, self.carry);
                } else {
                    self.kernel(DecimalKernel::FinishMultiplication);
                }
            }
            Stage::CarryWrite => {
                self.length += 1;
                self.kernel(DecimalKernel::FinishMultiplication);
            }
            _ => unreachable!("validated write request stage"),
        }
    }
    fn scratch(&mut self, digit: u8) {
        match self.stage {
            Stage::MultiplyRead => {
                let product = digit * self.factor + self.carry; // <=49
                self.carry = product / 10;
                self.stage = Stage::MultiplyWrite;
                self.write_digit(self.index, product % 10);
            }
            Stage::CompareScratch => self.compare_digit(digit),
            _ => unreachable!("validated scratch request stage"),
        }
    }
    fn read_boundary_digit(&mut self) {
        if self.compare_position < self.length {
            self.stage = Stage::CompareScratch;
            self.request = DecimalRequest::ReadScratch {
                index: self.length - 1 - self.compare_position,
            };
        } else {
            // No storage read is needed for an implicit decimal zero. This
            // finite comparison is part of the admitted input-read action.
            self.compare_digit(0);
        }
    }
    fn compare_digit(&mut self, boundary: u8) {
        self.comparison = self.authored_digit.cmp(&boundary);
        if self.comparison != Ordering::Equal {
            self.kernel(DecimalKernel::UpdateSearch);
        } else {
            self.compare_position += 1;
            self.kernel(DecimalKernel::BeginComparison);
        }
    }
    fn compute(&mut self, kernel: DecimalKernel) {
        match kernel {
            DecimalKernel::Normalize => {
                let Some(first) = self.first else {
                    self.finish(0);
                    return;
                };
                self.significant = self.last - first + 1;
                if self.huge_exponent {
                    self.finish(if self.layout.exponent_negative {
                        0
                    } else {
                        OVERFLOW_ENCODING
                    });
                    return;
                }
                let exponent = if self.layout.exponent_negative {
                    -(self.exponent as i128)
                } else {
                    self.exponent as i128
                };
                self.magnitude = exponent + (self.layout.mantissa_digits - first) as i128
                    - self.layout.fractional_digits as i128;
                if self.magnitude < -323 {
                    self.finish(0);
                } else if self.magnitude > 309 {
                    self.finish(OVERFLOW_ENCODING);
                } else {
                    self.kernel(DecimalKernel::SelectBoundary);
                }
            }
            DecimalKernel::SelectBoundary => {
                if self.low == self.high {
                    self.finish(self.low);
                    return;
                }
                if self.comparisons == 64 {
                    self.fail(DecimalArithmeticError::SearchExtent);
                    return;
                }
                self.comparisons += 1;
                self.candidate = self.low + (self.high - self.low) / 2;
                let biased = (self.candidate >> 52) as i32;
                let fraction = self.candidate & ((1_u64 << 52) - 1);
                let (significand, exponent) = if biased == 0 {
                    (fraction, -1074)
                } else {
                    ((1_u64 << 52) | fraction, biased - 1075)
                };
                self.seed = 2 * significand + 1;
                let power = exponent - 1;
                self.factor = if power < 0 { 5 } else { 2 };
                self.power = power.unsigned_abs() as u16;
                self.scale = if power < 0 { i128::from(power) } else { 0 };
                self.length = 0;
                self.kernel(DecimalKernel::SeedDigit);
            }
            DecimalKernel::SeedDigit => {
                let digit = (self.seed % 10) as u8;
                self.seed /= 10;
                self.stage = Stage::SeedWrite;
                self.write_digit(self.length, digit);
            }
            DecimalKernel::BeginMultiplication => {
                if self.power == 0 {
                    self.kernel(DecimalKernel::CompareMagnitude);
                } else {
                    self.index = 0;
                    self.carry = 0;
                    self.stage = Stage::MultiplyRead;
                    self.request = DecimalRequest::ReadScratch { index: 0 };
                }
            }
            DecimalKernel::FinishMultiplication => {
                self.power -= 1;
                self.kernel(DecimalKernel::BeginMultiplication);
            }
            DecimalKernel::CompareMagnitude => {
                self.comparison = self.magnitude.cmp(&(self.length as i128 + self.scale));
                if self.comparison != Ordering::Equal {
                    self.kernel(DecimalKernel::UpdateSearch);
                } else {
                    self.compare_position = 0;
                    self.kernel(DecimalKernel::BeginComparison);
                }
            }
            DecimalKernel::BeginComparison => {
                if self.compare_position >= self.significant.max(self.length) {
                    self.comparison = Ordering::Equal;
                    self.kernel(DecimalKernel::UpdateSearch);
                } else if self.compare_position < self.significant {
                    self.stage = Stage::CompareInput;
                    self.request = DecimalRequest::ReadInput {
                        kind: DecimalInput::Mantissa,
                        index: self.first.expect("nonzero search") + self.compare_position,
                    };
                } else {
                    self.authored_digit = 0;
                    self.read_boundary_digit();
                }
            }
            DecimalKernel::UpdateSearch => {
                if self.comparison == Ordering::Less
                    || (self.comparison == Ordering::Equal && self.candidate & 1 == 0)
                {
                    self.high = self.candidate;
                } else {
                    self.low = self.candidate + 1;
                }
                self.kernel(DecimalKernel::SelectBoundary);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Trace {
        scratch: Vec<DecimalDigit>,
        comparisons: usize,
        actions: usize,
        input_reads: usize,
        appends: usize,
        replacements: usize,
    }
    // This ordinary test owner supplies already separated checked digits. It
    // does not implement CSS grammar or a pretend production admission ledger.
    fn drive(
        negative: bool,
        mantissa: &str,
        fractional: usize,
        exponent_negative: bool,
        exponent: &str,
    ) -> (DecimalOutcome, Trace) {
        let layout = DecimalLayout::new(
            negative,
            mantissa.len(),
            fractional,
            exponent_negative,
            exponent.len(),
        )
        .unwrap();
        let mut cursor = DecimalCursor::new(layout);
        let mut trace = Trace {
            scratch: Vec::new(),
            comparisons: 0,
            actions: 0,
            input_reads: 0,
            appends: 0,
            replacements: 0,
        };
        loop {
            let request = *cursor.next_request();
            let reply = match request {
                DecimalRequest::ReadInput { kind, index } => {
                    trace.input_reads += 1;
                    let bytes = match kind {
                        DecimalInput::Mantissa => mantissa.as_bytes(),
                        DecimalInput::Exponent => exponent.as_bytes(),
                    };
                    let digit = bytes[index] - b'0';
                    DecimalReply::Input { kind, index, digit }
                }
                DecimalRequest::ReadScratch { index } => DecimalReply::Scratch {
                    index,
                    digit: trace.scratch[index],
                },
                DecimalRequest::AppendScratch { index, digit } => {
                    assert_eq!(index, trace.scratch.len());
                    trace.scratch.push(digit);
                    trace.appends += 1;
                    assert!(trace.scratch.len() <= MAX_BOUNDARY_DIGITS);
                    DecimalReply::Appended { index }
                }
                DecimalRequest::ReplaceScratch { index, digit } => {
                    trace.scratch[index] = digit;
                    trace.replacements += 1;
                    DecimalReply::Replaced { index }
                }
                DecimalRequest::Compute(kernel) => {
                    if kernel == DecimalKernel::SelectBoundary && cursor.low != cursor.high {
                        trace.comparisons += 1;
                    }
                    DecimalReply::Computed(kernel)
                }
                DecimalRequest::Done(outcome) => {
                    assert!(trace.comparisons <= 64);
                    assert_eq!(cursor.stored, trace.scratch.len());
                    return (outcome, trace);
                }
                DecimalRequest::Failed(error) => panic!("valid conversion failed: {error:?}"),
            };
            trace.actions += 1;
            cursor.complete(reply).unwrap();
        }
    }
    fn bits(
        mantissa: &str,
        fractional: usize,
        negative_exponent: bool,
        exponent: &str,
        expected: u64,
    ) {
        let (outcome, _) = drive(false, mantissa, fractional, negative_exponent, exponent);
        let DecimalOutcome::Finite(actual) = outcome else {
            panic!("expected finite")
        };
        assert_eq!(actual.to_bits(), expected);
    }

    #[test]
    fn literal_binary64_rounding_covers_normal_subnormal_and_even_ties() {
        for (digits, fractional, negative_exp, exp, expected) in [
            ("1", 0, false, "0", 0x3ff0_0000_0000_0000),
            ("01", 1, false, "0", 0x3fb9_9999_9999_999a),
            ("49406564584124654", 16, true, "324", 1),
            ("24703282292062327", 16, true, "324", 0),
            ("24703282292062328", 16, true, "324", 1),
            ("22250738585072014", 16, true, "308", 0x0010_0000_0000_0000),
            (
                "100000000000000011102230246251565404236316680908203125",
                53,
                false,
                "0",
                0x3ff0_0000_0000_0000,
            ),
            (
                "100000000000000033306690738754696212708950042724609375",
                53,
                false,
                "0",
                0x3ff0_0000_0000_0002,
            ),
            ("17976931348623157", 16, false, "308", 0x7fef_ffff_ffff_ffff),
        ] {
            bits(digits, fractional, negative_exp, exp, expected);
        }
    }

    #[test]
    fn exact_endpoint_midpoints_have_literal_ties_even_results() {
        // Independent integer identities: 5^1075 * 10^-1075 = 2^-1075;
        // (2^53-1)*5^1075 * 10^-1075 is the normal/subnormal midpoint;
        // (2^54-1)*2^970 is the maximum/overflow midpoint. The decimal digits
        // below were calculated by integer arithmetic, not this converter.
        let zero_midpoint = "24703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328125";
        bits(zero_midpoint, 0, true, "1075", 0);
        let normal_midpoint = "222507385850720113605740979670913197593481954635164564802342610972482222202107694551652952390813508791414915891303962110687008643869459464552765720740782062174337998814106326732925355228688137214901298112245145188984905722230728525513315575501591439747639798341180199932396254828901710708185069063066665599493827577257201576306269066333264756530000924588831643303777979186961204949739037782970490505108060994073026293712895895000358379996720725430436028407889577179615094551674824347103070260914462157228988025818254518032570701886087211312807951223342628836862232150377566662250398253433597456888442390026549819838548794829220689472168983109969836584681402285424333066033985088644580400103493397042756718644338377048603786162277173854562306587467901408672332763671875";
        bits(normal_midpoint, 0, true, "1075", 0x0010_0000_0000_0000);
        let overflow_midpoint = "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792";
        assert_eq!(
            drive(false, overflow_midpoint, 0, false, "0").0,
            DecimalOutcome::Overflow { negative: false }
        );
        let mut just_below = overflow_midpoint.to_owned();
        just_below.pop();
        just_below.push('1');
        bits(&just_below, 0, false, "0", 0x7fef_ffff_ffff_ffff);
        // An arbitrarily late nonzero tail distinguishes an exact tie from an
        // above-tie value; no bounded prefix may decide equality prematurely.
        let above_zero = format!("{zero_midpoint}{}1", "0".repeat(1200));
        bits(&above_zero, 0, true, "2276", 1);
    }

    #[test]
    fn signs_huge_exponents_and_length_cancellation_are_not_token_expansion() {
        for (negative, mantissa, exponent_negative, exponent, expected) in [
            (
                false,
                "000000",
                false,
                "999999999999999999999999999999999999999999",
                0,
            ),
            (
                true,
                "000",
                true,
                "999999999999999999999999999999999999999999",
                1_u64 << 63,
            ),
            (
                true,
                "1",
                true,
                "999999999999999999999999999999999999999999",
                1_u64 << 63,
            ),
            (true, "1", true, "400", 1_u64 << 63),
        ] {
            let (outcome, trace) = drive(negative, mantissa, 0, exponent_negative, exponent);
            let DecimalOutcome::Finite(value) = outcome else {
                panic!("finite zero")
            };
            assert_eq!(value.to_bits(), expected);
            assert_eq!(trace.input_reads, mantissa.len() + exponent.len());
            assert!(trace.scratch.is_empty());
        }
        assert_eq!(
            drive(
                true,
                "1",
                0,
                false,
                "999999999999999999999999999999999999999999"
            )
            .0,
            DecimalOutcome::Overflow { negative: true }
        );
        let padded = format!("{}1{}", "0".repeat(1300), "0".repeat(1400));
        bits(
            &padded,
            0,
            true,
            "00000000000000001400",
            0x3ff0_0000_0000_0000,
        );
        bits(
            &padded,
            1400,
            false,
            "0000000000000000",
            0x3ff0_0000_0000_0000,
        );
        let fractional = format!("{}1", "0".repeat(1500));
        bits(&fractional, 1501, false, "1501", 0x3ff0_0000_0000_0000);
    }

    #[test]
    fn request_refusals_preserve_pending_state_and_actual_caller_scratch() {
        assert_eq!(
            DecimalDigit::new(10),
            Err(DecimalProtocolError::InvalidDigit)
        );
        assert_eq!(DecimalDigit::new(9).unwrap().digit(), 9);
        assert_eq!(
            DecimalLayout::new(false, 0, 0, false, 1),
            Err(DecimalLayoutError::EmptyMantissa)
        );
        assert_eq!(
            DecimalLayout::new(false, 1, 0, false, 0),
            Err(DecimalLayoutError::EmptyExponent)
        );
        assert_eq!(
            DecimalLayout::new(false, 1, 2, false, 1),
            Err(DecimalLayoutError::FractionalExtent)
        );
        assert_eq!(
            DecimalLayout::new(false, usize::MAX, 0, false, 1),
            Err(DecimalLayoutError::IndexOverflow)
        );
        let (scratch, retained) = {
            let mut cursor = DecimalCursor::new(DecimalLayout::new(false, 1, 0, false, 1).unwrap());
            let original = format!("{cursor:?}");
            for reply in [
                DecimalReply::Input {
                    kind: DecimalInput::Exponent,
                    index: 0,
                    digit: 1,
                },
                DecimalReply::Input {
                    kind: DecimalInput::Mantissa,
                    index: 1,
                    digit: 1,
                },
                DecimalReply::Computed(DecimalKernel::Normalize),
            ] {
                assert_eq!(
                    cursor.complete(reply),
                    Err(DecimalProtocolError::WrongReply)
                );
                assert_eq!(format!("{cursor:?}"), original);
            }
            assert_eq!(
                cursor.complete(DecimalReply::Input {
                    kind: DecimalInput::Mantissa,
                    index: 0,
                    digit: 10
                }),
                Err(DecimalProtocolError::InvalidDigit)
            );
            assert_eq!(format!("{cursor:?}"), original);
            cursor
                .complete(DecimalReply::Input {
                    kind: DecimalInput::Mantissa,
                    index: 0,
                    digit: 1,
                })
                .unwrap();
            cursor
                .complete(DecimalReply::Input {
                    kind: DecimalInput::Exponent,
                    index: 0,
                    digit: 0,
                })
                .unwrap();
            for kernel in [
                DecimalKernel::Normalize,
                DecimalKernel::SelectBoundary,
                DecimalKernel::SeedDigit,
            ] {
                cursor.complete(DecimalReply::Computed(kernel)).unwrap();
            }
            let DecimalRequest::AppendScratch { index, digit } = *cursor.next_request() else {
                panic!("first admitted seed digit")
            };
            assert_eq!(index, 0);
            // Encoding0x3ff8000000000000 has significand6755399441055744;
            // its midpoint coefficient is13510798882111489, least digit9.
            assert_eq!(digit.digit(), 9);
            let pending = format!("{cursor:?}");
            let mut scratch = Vec::new();
            // Caller refusal before an append acknowledgement preserves the request.
            assert_eq!(cursor.stored, 0);
            assert_eq!(format!("{cursor:?}"), pending);
            assert_eq!(
                cursor.complete(DecimalReply::Appended { index: 1 }),
                Err(DecimalProtocolError::WrongReply)
            );
            assert_eq!(format!("{cursor:?}"), pending);
            scratch.push(digit);
            cursor
                .complete(DecimalReply::Appended { index: 0 })
                .unwrap();
            assert_eq!(cursor.stored, scratch.len());
            assert_eq!(scratch[0], digit);
            // Finish the seed through real caller records until multiplication
            // requires a scratch read. No incidental whole-run output is an oracle.
            loop {
                let reply = match *cursor.next_request() {
                    DecimalRequest::ReadScratch { .. } => break,
                    DecimalRequest::Compute(kernel) => DecimalReply::Computed(kernel),
                    DecimalRequest::AppendScratch { index, digit } => {
                        assert_eq!(index, scratch.len());
                        scratch.push(digit);
                        DecimalReply::Appended { index }
                    }
                    other => panic!("seed request: {other:?}"),
                };
                cursor.complete(reply).unwrap();
            }
            let pending = format!("{cursor:?}");
            assert_eq!(
                cursor.complete(DecimalReply::Scratch {
                    index: 1,
                    digit: DecimalDigit::new(9).unwrap()
                }),
                Err(DecimalProtocolError::WrongReply)
            );
            assert_eq!(format!("{cursor:?}"), pending);
            cursor
                .complete(DecimalReply::Scratch {
                    index: 0,
                    digit: scratch[0],
                })
                .unwrap();
            let DecimalRequest::ReplaceScratch {
                index,
                digit: replacement,
            } = *cursor.next_request()
            else {
                panic!("one multiplication digit")
            };
            assert_eq!(index, 0);
            let pending = format!("{cursor:?}");
            for reply in [
                DecimalReply::Replaced { index: 1 },
                DecimalReply::Appended { index: 0 },
            ] {
                assert_eq!(
                    cursor.complete(reply),
                    Err(DecimalProtocolError::WrongReply)
                );
                assert_eq!(format!("{cursor:?}"), pending);
                assert_eq!(scratch[0], digit);
            }
            scratch[0] = replacement;
            cursor
                .complete(DecimalReply::Replaced { index: 0 })
                .unwrap();
            assert_eq!(cursor.stored, scratch.len());
            let retained = scratch.clone();
            // Abandonment drops only fixed registers, not this caller's scratch.
            (scratch, retained)
        };
        assert_eq!(scratch, retained);
        let mut zero = DecimalCursor::new(DecimalLayout::new(true, 1, 0, false, 1).unwrap());
        zero.complete(DecimalReply::Input {
            kind: DecimalInput::Mantissa,
            index: 0,
            digit: 0,
        })
        .unwrap();
        zero.complete(DecimalReply::Input {
            kind: DecimalInput::Exponent,
            index: 0,
            digit: 0,
        })
        .unwrap();
        zero.complete(DecimalReply::Computed(DecimalKernel::Normalize))
            .unwrap();
        let done = *zero.next_request();
        assert_eq!(
            zero.complete(DecimalReply::Computed(DecimalKernel::Normalize)),
            Err(DecimalProtocolError::Terminal)
        );
        assert_eq!(*zero.next_request(), done);
    }

    #[test]
    fn scratch_reuse_and_search_are_bounded_independently_of_padding() {
        let (outcome, trace) = drive(false, "49406564584124654", 16, true, "324");
        let DecimalOutcome::Finite(value) = outcome else {
            panic!("finite")
        };
        assert_eq!(value.to_bits(), 1);
        assert!(trace.comparisons <= 64);
        assert!(trace.scratch.len() <= 1092);
        assert_eq!(trace.appends, trace.scratch.len());
        assert!(trace.replacements > trace.appends);
        // One pass normalizes; comparison rereads at most64 times, one digit
        // per request. Multiplication visits at most1092 digits per1075 powers
        // per64 boundaries, with one read + one write and finite transitions.
        assert!(trace.input_reads <= 17 * 65 + 3);
        assert!(trace.actions <= 64 * (1100 * 1075 * 2 + 1100 * 4 + 2200));
    }
}
