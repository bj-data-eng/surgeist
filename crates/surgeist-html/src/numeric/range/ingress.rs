//! Decimal ingress after the authoritative floating grammar has recognized it.
use super::*;
use std::cmp::Ordering;

const INFINITY: u64 = 0x7ff0_0000_0000_0000;
#[derive(Clone, Copy, Debug)]
enum Phase {
    Start,
    Mantissa,
    MantissaNext,
    Exponent,
    ExponentNext,
    Classify,
    Clear,
    CoefficientRead,
    CoefficientReserve,
    CoefficientAppend,
    CoefficientNext,
    Small,
    BuildTrial,
    ProductTrial,
    Compare,
    Decide,
    Done,
}

#[derive(Clone, Copy, Debug)]
enum ActiveHelper {
    None,
    Small(SmallDecimalCursor),
    Build(RangeDyadicCursor),
    Product(RangeMantissaProductCursor),
    Compare(RangeScaledComparisonCursor),
}

// The admitted coefficient is never shortened for eligibility. These transient
// registers occupy the exclusive helper variant, not a second scratch owner.
// Fixed inventory: acknowledged headers and one cached request, cell index,
// decimal scale and remaining power count, binary exponent, four u128 registers
// (numerator/denominator/quotient/remainder), phase and optional result.
// Read consumes one admitted cell; Next advances one index; Power multiplies at most once;
// Exponent, Scale, Quotient, Round and Finish each require their own ScalarStep.
#[derive(Clone, Copy, Debug)]
enum SmallPhase {
    Read,
    Next,
    Power,
    Exponent,
    Scale,
    Quotient,
    Round,
    Finish,
    Done,
}
#[derive(Clone, Copy, Debug)]
enum SmallResult {
    Bits(u64),
    Decline,
}
#[derive(Clone, Copy, Debug)]
struct SmallDecimalCursor {
    io: Io,
    phase: SmallPhase,
    index: usize,
    scale: i32,
    power: u32,
    exponent: i32,
    numerator: u128,
    denominator: u128,
    quotient: u128,
    remainder: u128,
    result: Option<SmallResult>,
}
impl SmallDecimalCursor {
    fn new(headers: RangeNumericHeaders, scale: i32) -> Result<Self, RangeNumericError> {
        let len = headers.buffers[RangeScratchRole::Coefficient.index()].len;
        if !(1..=19).contains(&len) || !(-22..=22).contains(&scale) {
            return Err(RangeNumericError::Integrity);
        }
        let mut cursor = Self {
            io: Io::new(headers, 0, 0),
            phase: SmallPhase::Read,
            index: len - 1,
            scale,
            power: scale.unsigned_abs(),
            exponent: 0,
            numerator: 0,
            denominator: 1,
            quotient: 0,
            remainder: 0,
            result: None,
        };
        cursor.read();
        Ok(cursor)
    }
    fn read(&mut self) {
        self.io.queue(RangeNumericRequest::ReadCells {
            cells: [
                Some(RangeCell {
                    role: RangeScratchRole::Coefficient,
                    index: self.index,
                }),
                None,
                None,
            ],
        });
    }
    fn shift(value: u128, count: u32) -> Result<u128, RangeNumericError> {
        let width = u128::BITS - value.leading_zeros();
        if count >= u128::BITS || width.checked_add(count).is_none_or(|n| n > u128::BITS) {
            return Err(RangeNumericError::Representation);
        }
        Ok(value << count)
    }
    fn validate_request(&self) -> Result<(), RangeNumericError> {
        let expected = match self.phase {
            SmallPhase::Read => Some(RangeNumericRequest::ReadCells {
                cells: [
                    Some(RangeCell {
                        role: RangeScratchRole::Coefficient,
                        index: self.index,
                    }),
                    None,
                    None,
                ],
            }),
            SmallPhase::Done => None,
            _ => Some(RangeNumericRequest::ScalarStep),
        };
        if self.io.request != expected {
            return Err(RangeNumericError::Integrity);
        }
        Ok(())
    }
    fn accept_in_place(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        self.validate_request()?;
        self.io.accept(reply)?;
        use SmallPhase as S;
        match self.phase {
            S::Read => {
                let RangeNumericReply::Cells([Some(digit), None, None]) = reply else {
                    return Err(RangeNumericError::Integrity);
                };
                if self.index + 1 == self.io.len(RangeScratchRole::Coefficient) && digit == 0 {
                    return Err(RangeNumericError::Integrity);
                }
                self.numerator = self
                    .numerator
                    .checked_mul(10)
                    .and_then(|n| n.checked_add(u128::from(digit)))
                    .ok_or(RangeNumericError::Representation)?;
                self.phase = S::Next;
            }
            S::Next => {
                if self.index == 0 {
                    self.phase = S::Power;
                } else {
                    self.index -= 1;
                    self.phase = S::Read;
                    self.read();
                    return Ok(());
                }
            }
            S::Power => {
                if self.power == 0 {
                    self.phase = S::Exponent;
                } else {
                    if self.scale >= 0 {
                        let Some(n) = self.numerator.checked_mul(10) else {
                            self.result = Some(SmallResult::Decline);
                            self.phase = S::Done;
                            self.io.request = None;
                            return Ok(());
                        };
                        self.numerator = n;
                    } else {
                        self.denominator = self
                            .denominator
                            .checked_mul(10)
                            .ok_or(RangeNumericError::Representation)?;
                    }
                    self.power -= 1;
                }
            }
            S::Exponent => {
                if self.numerator == 0 || self.denominator == 0 {
                    return Err(RangeNumericError::Integrity);
                }
                let a = (u128::BITS - 1 - self.numerator.leading_zeros()) as i32;
                let b = (u128::BITS - 1 - self.denominator.leading_zeros()) as i32;
                let e0 = a - b;
                let less = if e0 >= 0 {
                    self.numerator < Self::shift(self.denominator, e0 as u32)?
                } else {
                    Self::shift(self.numerator, e0.unsigned_abs())? < self.denominator
                };
                self.exponent = e0 - i32::from(less);
                if !(-74..=127).contains(&self.exponent) {
                    return Err(RangeNumericError::Integrity);
                }
                self.phase = S::Scale;
            }
            S::Scale => {
                let shift = 52 - self.exponent;
                if shift >= 0 {
                    self.numerator = Self::shift(self.numerator, shift as u32)?;
                } else {
                    self.denominator = Self::shift(self.denominator, shift.unsigned_abs())?;
                }
                self.phase = S::Quotient;
            }
            S::Quotient => {
                self.quotient = self.numerator / self.denominator;
                self.remainder = self.numerator % self.denominator;
                if !((1u128 << 52)..(1u128 << 53)).contains(&self.quotient) {
                    return Err(RangeNumericError::Integrity);
                }
                self.phase = S::Round;
            }
            S::Round => {
                // Compare R to B-R; doubling R could overflow its fixed register.
                let complement = self.denominator - self.remainder;
                if self.remainder > complement
                    || self.remainder == complement && self.quotient & 1 != 0
                {
                    self.quotient += 1;
                }
                if self.quotient == 1u128 << 53 {
                    self.quotient >>= 1;
                    self.exponent += 1;
                }
                self.phase = S::Finish;
            }
            S::Finish => {
                if !(-74..=128).contains(&self.exponent)
                    || !((1u128 << 52)..(1u128 << 53)).contains(&self.quotient)
                {
                    return Err(RangeNumericError::Integrity);
                }
                let bits =
                    ((self.exponent + 1023) as u64) << 52 | (self.quotient as u64 - (1u64 << 52));
                self.result = Some(SmallResult::Bits(bits));
                self.phase = S::Done;
                self.io.request = None;
                return Ok(());
            }
            S::Done => return Err(RangeNumericError::Integrity),
        }
        self.io.queue(RangeNumericRequest::ScalarStep);
        Ok(())
    }
}

/// Scalar inventory: original component offsets; one exponent accumulator;
/// digit/ordinal counters; the two 63-bit search endpoints; raw-exponent cache
/// key/extent; one active small-ratio, dyadic build, product or comparison receiver.
/// No source borrow, collection, or storage ticket.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FiniteDecimalIngressCursor {
    io: Io,
    parts: RangeDecimalParts,
    total: usize,
    ordinal: usize,
    first: Option<usize>,
    exponent_index: usize,
    exponent: usize,
    threshold: usize,
    scale: i128,
    digit: u8,
    low: u64,
    high: u64,
    trial: u64,
    midpoint: bool,
    active: ActiveHelper,
    trial_mantissa: u64,
    trial_exponent: i32,
    cache_exponent: Option<i32>,
    cache_len: usize,
    ordering: Ordering,
    phase: Phase,
    result: Option<Result<u64, RangeNumericError>>,
}
impl FiniteDecimalIngressCursor {
    pub(crate) fn new(
        parts: RangeDecimalParts,
        source_len: usize,
        headers: RangeNumericHeaders,
    ) -> Result<Self, RangeNumericError> {
        if parts.integer_start > parts.integer_end
            || parts.integer_end > source_len
            || parts.fraction_start > parts.fraction_end
            || parts.fraction_end > source_len
            || parts.exponent_start > parts.exponent_end
            || parts.exponent_end > source_len
            || headers.buffers[RangeScratchRole::Right.index()].len != 0
        {
            return Err(RangeNumericError::Integrity);
        }
        let total = (parts.integer_end - parts.integer_start)
            .checked_add(parts.fraction_end - parts.fraction_start)
            .ok_or(RangeNumericError::Representation)?;
        if total == 0 {
            return Err(RangeNumericError::InvalidSyntax);
        }
        let width = source_len
            .checked_add(1386)
            .ok_or(RangeNumericError::Representation)?;
        let threshold = source_len
            .checked_add(4096)
            .ok_or(RangeNumericError::Representation)?;
        Ok(Self {
            io: Io::new(headers, source_len, width),
            parts,
            total,
            ordinal: 0,
            first: None,
            exponent_index: parts.exponent_start,
            exponent: 0,
            threshold,
            scale: 0,
            digit: 0,
            low: 0,
            high: INFINITY,
            trial: 0,
            midpoint: false,
            active: ActiveHelper::None,
            trial_mantissa: 0,
            trial_exponent: 0,
            cache_exponent: None,
            cache_len: 0,
            ordering: Ordering::Equal,
            phase: Phase::Start,
            result: None,
        })
    }
    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        self.io.request
    }
    pub(crate) fn headers(&self) -> RangeNumericHeaders {
        self.io.headers
    }
    pub(crate) fn result(&self) -> Option<Result<u64, RangeNumericError>> {
        self.result
    }
    #[cfg(test)]
    pub(crate) fn cache_key(&self) -> Option<i32> {
        self.cache_exponent
    }
    fn validate_active(&self) -> Result<(), RangeNumericError> {
        let request = match (self.phase, &self.active) {
            (Phase::Small, ActiveHelper::Small(helper)) => {
                helper.validate_request()?;
                helper.io.request
            }
            (Phase::BuildTrial, ActiveHelper::Build(helper)) => helper.request(),
            (Phase::ProductTrial, ActiveHelper::Product(helper)) => helper.request(),
            (Phase::Compare, ActiveHelper::Compare(helper)) => helper.request(),
            (Phase::Small | Phase::BuildTrial | Phase::ProductTrial | Phase::Compare, _) => {
                return Err(RangeNumericError::Integrity);
            }
            (_, ActiveHelper::None) => return Ok(()),
            _ => return Err(RangeNumericError::Integrity),
        };
        if request == self.io.request {
            Ok(())
        } else {
            Err(RangeNumericError::Integrity)
        }
    }
    #[cfg(test)]
    pub(crate) fn active_kind(&self) -> u8 {
        match self.active {
            ActiveHelper::None => 0,
            ActiveHelper::Small(..) => 4,
            ActiveHelper::Build(..) => 1,
            ActiveHelper::Product(..) => 2,
            ActiveHelper::Compare(..) => 3,
        }
    }
    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        let mut next = *self;
        next.validate_active()?;
        next.io.accept(reply)?;
        next.advance(reply)?;
        next.validate_active()?;
        *self = next;
        Ok(())
    }
    fn source_index(&self, ordinal: usize) -> usize {
        let integer = self.parts.integer_end - self.parts.integer_start;
        if ordinal < integer {
            self.parts.integer_start + ordinal
        } else {
            self.parts.fraction_start + ordinal - integer
        }
    }
    fn finish(&mut self, result: Result<u64, RangeNumericError>) {
        self.result =
            Some(result.map(|bits| bits | if self.parts.negative { 1u64 << 63 } else { 0 }));
        self.active = ActiveHelper::None;
        self.phase = Phase::Done;
        self.io.request = None;
    }
    fn exponent_or_classify(&mut self) {
        if self.exponent_index < self.parts.exponent_end {
            self.phase = Phase::Exponent;
            self.io.queue(RangeNumericRequest::SourceUnit {
                index: self.exponent_index,
            });
        } else {
            self.phase = Phase::Classify;
            self.io.queue(RangeNumericRequest::ScalarStep);
        }
    }
    fn clear_or_coefficient(&mut self) {
        if self.io.len(RangeScratchRole::Coefficient) != 0 {
            self.phase = Phase::Clear;
            self.io.queue(RangeNumericRequest::PopCell {
                role: RangeScratchRole::Coefficient,
            });
        } else {
            self.ordinal = self.total - 1;
            self.phase = Phase::CoefficientRead;
            self.io.queue(RangeNumericRequest::SourceUnit {
                index: self.source_index(self.ordinal),
            });
        }
    }
    fn begin_trial(&mut self, mantissa: u64, exponent: i32) -> Result<(), RangeNumericError> {
        if !self.midpoint {
            let decimal_order = (self.io.len(RangeScratchRole::Coefficient) as i128)
                .checked_add(self.scale)
                .ok_or(RangeNumericError::Representation)?;
            if let Some(order) = decimal_dyadic_ordering(decimal_order, mantissa, exponent)? {
                self.ordering = order;
                self.phase = Phase::Decide;
                self.io.queue(RangeNumericRequest::ScalarStep);
                return Ok(());
            }
        }
        self.trial_mantissa = mantissa;
        self.trial_exponent = exponent;
        if self.cache_exponent == Some(exponent) {
            if self.cache_len == 0 || self.cache_len != self.io.len(RangeScratchRole::Right) {
                return Err(RangeNumericError::Integrity);
            }
            return self.begin_product();
        }
        // The old coefficient is no longer a valid cache once rebuilding starts.
        // Publish the raw exponent only after the complete admitted build.
        self.cache_exponent = None;
        self.cache_len = 0;
        let build = RangeDyadicCursor::new(self.io.headers, RangeScratchRole::Right, 1, exponent)?;
        self.io.request = build.request();
        self.active = ActiveHelper::Build(build);
        self.phase = Phase::BuildTrial;
        Ok(())
    }
    fn begin_product(&mut self) -> Result<(), RangeNumericError> {
        let product = RangeMantissaProductCursor::new(
            self.io.headers,
            RangeScratchRole::Right,
            RangeScratchRole::Trial,
            self.trial_mantissa,
        )?;
        self.io.request = product.request();
        self.active = ActiveHelper::Product(product);
        self.phase = Phase::ProductTrial;
        Ok(())
    }
    fn next_trial(&mut self) -> Result<(), RangeNumericError> {
        if self.high - self.low == 1 {
            self.midpoint = true;
            let (m, e) = upper_midpoint(self.low)?;
            self.begin_trial(m, e)
        } else {
            self.trial = self.low + (self.high - self.low) / 2;
            let (m, e) = finite_dyadic(self.trial)?;
            self.begin_trial(m, e)
        }
    }
    fn digit(reply: RangeNumericReply) -> Result<u8, RangeNumericError> {
        let RangeNumericReply::Unit(unit @ 0x30..=0x39) = reply else {
            return Err(RangeNumericError::Integrity);
        };
        Ok((unit - 0x30) as u8)
    }
    fn advance(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        use Phase as S;
        use RangeNumericRequest as Q;
        match self.phase {
            S::Start => {
                self.phase = S::Mantissa;
                self.io.queue(Q::SourceUnit {
                    index: self.source_index(0),
                });
            }
            S::Mantissa => {
                let digit = Self::digit(reply)?;
                if digit != 0 && self.first.is_none() {
                    self.first = Some(self.ordinal);
                }
                self.phase = S::MantissaNext;
                self.io.queue(Q::ScalarStep);
            }
            S::MantissaNext => {
                self.ordinal += 1;
                if self.ordinal < self.total {
                    self.phase = S::Mantissa;
                    self.io.queue(Q::SourceUnit {
                        index: self.source_index(self.ordinal),
                    });
                } else {
                    self.exponent_or_classify();
                }
            }
            S::Exponent => {
                let digit = usize::from(Self::digit(reply)?);
                self.exponent = if self.exponent > (self.threshold - digit) / 10 {
                    self.threshold
                } else {
                    self.exponent * 10 + digit
                };
                self.phase = S::ExponentNext;
                self.io.queue(Q::ScalarStep);
            }
            S::ExponentNext => {
                self.exponent_index += 1;
                self.exponent_or_classify();
            }
            S::Classify => {
                if let Some(first) = self.first {
                    let exponent =
                        self.exponent as i128 * if self.parts.exponent_negative { -1 } else { 1 };
                    self.scale =
                        exponent - (self.parts.fraction_end - self.parts.fraction_start) as i128;
                    let order = (self.total - first) as i128 + self.scale;
                    if order < -323 {
                        self.finish(Ok(0));
                    } else if order > 309 {
                        self.finish(Err(RangeNumericError::Overflow));
                    } else {
                        self.clear_or_coefficient();
                    }
                } else {
                    self.finish(Ok(0));
                }
            }
            S::Clear => self.clear_or_coefficient(),
            S::CoefficientRead => {
                self.digit = Self::digit(reply)?;
                self.phase = if self.io.append(RangeScratchRole::Coefficient, self.digit)? {
                    S::CoefficientAppend
                } else {
                    S::CoefficientReserve
                };
            }
            S::CoefficientReserve => {
                self.phase = S::CoefficientAppend;
                self.io.queue(Q::AppendCell {
                    role: RangeScratchRole::Coefficient,
                    value: self.digit,
                });
            }
            S::CoefficientAppend => {
                self.phase = S::CoefficientNext;
                self.io.queue(Q::ScalarStep);
            }
            S::CoefficientNext => {
                if Some(self.ordinal) == self.first {
                    let len = self.io.len(RangeScratchRole::Coefficient);
                    if (1..=19).contains(&len) && (-22..=22).contains(&self.scale) {
                        let small = SmallDecimalCursor::new(self.io.headers, self.scale as i32)?;
                        self.io.request = small.io.request;
                        self.active = ActiveHelper::Small(small);
                        self.phase = S::Small;
                    } else {
                        self.next_trial()?;
                    }
                } else {
                    self.ordinal -= 1;
                    self.phase = S::CoefficientRead;
                    self.io.queue(Q::SourceUnit {
                        index: self.source_index(self.ordinal),
                    });
                }
            }
            S::Small => {
                let ActiveHelper::Small(small) = &mut self.active else {
                    return Err(RangeNumericError::Integrity);
                };
                small.accept_in_place(reply)?;
                if let Some(result) = small.result {
                    match result {
                        SmallResult::Bits(bits) => self.finish(Ok(bits)),
                        SmallResult::Decline => {
                            self.active = ActiveHelper::None;
                            self.next_trial()?;
                        }
                    }
                } else {
                    self.io.request = small.io.request;
                }
            }
            S::BuildTrial => {
                let ActiveHelper::Build(build) = &mut self.active else {
                    return Err(RangeNumericError::Integrity);
                };
                build.accept_in_place(reply)?;
                self.io.headers = build.headers();
                if let Some(scale) = build.result() {
                    if scale != self.trial_exponent.min(0) {
                        return Err(RangeNumericError::Integrity);
                    }
                    self.cache_len = self.io.len(RangeScratchRole::Right);
                    self.cache_exponent = Some(self.trial_exponent);
                    self.begin_product()?;
                } else {
                    self.io.request = build.request();
                }
            }
            S::ProductTrial => {
                let ActiveHelper::Product(product) = &mut self.active else {
                    return Err(RangeNumericError::Integrity);
                };
                product.accept_in_place(reply)?;
                self.io.headers = product.headers();
                if product.result().is_some() {
                    let compare = RangeScaledComparisonCursor::new(
                        self.io.headers,
                        RangeScratchRole::Coefficient,
                        self.scale,
                        false,
                        RangeScratchRole::Trial,
                        i128::from(self.trial_exponent.min(0)),
                        false,
                    );
                    self.io.request = compare.request();
                    self.active = ActiveHelper::Compare(compare);
                    self.phase = S::Compare;
                } else {
                    self.io.request = product.request();
                }
            }
            S::Compare => {
                let ActiveHelper::Compare(compare) = &mut self.active else {
                    return Err(RangeNumericError::Integrity);
                };
                compare.accept_in_place(reply)?;
                if let Some(order) = compare.result() {
                    self.ordering = order;
                    self.active = ActiveHelper::None;
                    self.phase = S::Decide;
                    self.io.queue(Q::ScalarStep);
                } else {
                    self.io.request = compare.request();
                }
            }
            S::Decide => {
                if self.midpoint {
                    let selected = if self.ordering == Ordering::Greater
                        || self.ordering == Ordering::Equal && self.low & 1 != 0
                    {
                        self.high
                    } else {
                        self.low
                    };
                    self.finish(if selected == INFINITY {
                        Err(RangeNumericError::Overflow)
                    } else {
                        Ok(selected)
                    });
                } else if self.ordering == Ordering::Equal {
                    self.finish(Ok(self.trial));
                } else {
                    if self.ordering == Ordering::Greater {
                        self.low = self.trial;
                    } else {
                        self.high = self.trial;
                    }
                    self.next_trial()?;
                }
            }
            S::Done => return Err(RangeNumericError::Integrity),
        }
        Ok(())
    }
}

#[cfg(test)]
mod exclusive_tests {
    use super::*;
    #[test]
    fn active_phase_and_cached_request_mismatches_are_transactional_refusals() {
        let headers = RangeNumericHeaders {
            buffers: [RangeBufferHeader {
                len: 0,
                capacity: 0,
            }; 8],
            output: RangeBufferHeader {
                len: 0,
                capacity: 0,
            },
        };
        let original = FiniteDecimalIngressCursor::new(
            RangeDecimalParts {
                negative: false,
                integer_start: 0,
                integer_end: 1,
                fraction_start: 1,
                fraction_end: 1,
                exponent_start: 0,
                exponent_end: 0,
                exponent_negative: false,
            },
            1,
            headers,
        )
        .unwrap();
        let build = RangeDyadicCursor::new(headers, RangeScratchRole::Trial, 1, 0).unwrap();
        for case in 0..3 {
            let mut c = original;
            match case {
                0 => c.phase = Phase::BuildTrial,
                1 => c.active = ActiveHelper::Build(build),
                _ => {
                    c.phase = Phase::BuildTrial;
                    c.active = ActiveHelper::Build(build);
                    c.io.request = None;
                }
            }
            let snapshot = format!("{c:?}");
            assert_eq!(
                c.reply(RangeNumericReply::Ack),
                Err(RangeNumericError::Integrity)
            );
            assert_eq!(format!("{c:?}"), snapshot);
        }
    }
}

#[cfg(test)]
mod small_tests {
    use super::*;
    #[test]
    fn small_ratio_header_phase_and_request_refusals_do_not_commit() {
        let parts = RangeDecimalParts {
            negative: false,
            integer_start: 0,
            integer_end: 1,
            fraction_start: 1,
            fraction_end: 1,
            exponent_start: 0,
            exponent_end: 0,
            exponent_negative: false,
        };
        let mut headers = RangeNumericHeaders::default();
        headers.buffers[0] = RangeBufferHeader {
            len: 1,
            capacity: 1,
        };
        let small = SmallDecimalCursor::new(headers, 0).unwrap();
        for case in 0..5 {
            let mut c = FiniteDecimalIngressCursor::new(parts, 1, headers).unwrap();
            c.phase = Phase::Small;
            c.io.request = small.io.request;
            c.active = ActiveHelper::Small(small);
            match case {
                0 => c.io.headers.buffers[0].capacity = 0,
                1 => {
                    let ActiveHelper::Small(s) = &mut c.active else {
                        panic!()
                    };
                    s.io.headers.buffers[0].capacity = 0;
                }
                2 => c.phase = Phase::Compare,
                3 => c.io.request = Some(RangeNumericRequest::ScalarStep),
                _ => {
                    let ActiveHelper::Small(s) = &mut c.active else {
                        panic!()
                    };
                    s.phase = SmallPhase::Quotient;
                }
            }
            let snapshot = format!("{c:?}");
            assert_eq!(
                c.reply(RangeNumericReply::Cells([Some(1), None, None])),
                Err(RangeNumericError::Integrity)
            );
            assert_eq!(format!("{c:?}"), snapshot);
        }
    }
    #[test]
    fn small_ratio_checked_shift_rejects_value_loss_as_well_as_shift_count() {
        assert_eq!(
            SmallDecimalCursor::shift(1u128 << 127, 1),
            Err(RangeNumericError::Representation)
        );
        assert_eq!(
            SmallDecimalCursor::shift(1, 128),
            Err(RangeNumericError::Representation)
        );
        assert_eq!(SmallDecimalCursor::shift(1, 127), Ok(1u128 << 127));
        assert_eq!(SmallDecimalCursor::shift(u128::MAX, 0), Ok(u128::MAX));
    }
}
