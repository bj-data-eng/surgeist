//! Selected ECMA decimal serialization: shortest, nearest, even significand.
//! Endpoint arithmetic is exact decimal over caller-owned cells, never dtoa.
use super::*;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
enum Phase {
    Start,
    Build,
    Order,
    Cut,
    Candidates,
    NextCandidate,
    SumBuild,
    Compare,
    Winner,
    NextOrder,
    NextK,
    Output,
    OutputReserve,
    Emit,
    Done,
}
#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
enum ActiveHelper {
    None,
    Build(RangeDyadicCursor),
    Cut(RangePowerTenCutCursor),
    Compare(RangeScaledComparisonCursor),
}

/// Serializes a finite `f64` as the shortest decimal that rounds back to it, choosing the nearest
/// candidate and then the even significand, as the selected ECMAScript algorithm does. The
/// arithmetic is exact decimal over caller-owned cells, never floating-point formatting. The output
/// is at most 25 UTF-16 code units, `-` first for a negative nonzero value, and the cursor emits it
/// through `EmitUnit` requests.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct HtmlBestFiniteNumberCursor {
    io: Io,
    bits: u64,
    negative: bool,
    inclusive: bool,
    mantissa: u64,
    exponent: i32,
    active: ActiveHelper,
    build_index: u8,
    cut_index: u8,
    scales: [i32; 4],
    lower_order: i32,
    upper_order: i32,
    n: i32,
    k: usize,
    lower: u64,
    upper: u64,
    floor: u64,
    exact: bool,
    below: Option<u64>,
    above: Option<u64>,
    candidate_index: u8,
    candidate: u64,
    best: Option<(u64, i32)>,
    sum_scale: i32,
    candidate_is_high: bool,
    ordering: Ordering,
    output_len: usize,
    output_index: usize,
    phase: Phase,
}
impl HtmlBestFiniteNumberCursor {
    /// Starts serializing `value`. `headers` describes the caller's empty output buffer and its
    /// scratch buffers. A non-finite `value` is an `Overflow` error, checked before the buffer, and
    /// a nonempty output buffer is an `Integrity` error.
    pub fn new(
        value: f64,
        headers: HtmlRangeNumericHeaders,
    ) -> Result<Self, HtmlRangeNumericError> {
        let raw = value.to_bits();
        let bits = raw & !(1u64 << 63);
        let (mantissa, exponent) = finite_dyadic(bits)?;
        if headers.output.len != 0 {
            return Err(HtmlRangeNumericError::Integrity);
        }
        Ok(Self {
            io: Io::new(headers, 0, 1386),
            bits,
            negative: raw >> 63 != 0 && bits != 0,
            inclusive: bits & 1 == 0,
            mantissa,
            exponent,
            active: ActiveHelper::None,
            build_index: 0,
            cut_index: 0,
            scales: [0; 4],
            lower_order: 0,
            upper_order: 0,
            n: 0,
            k: 1,
            lower: 0,
            upper: 0,
            floor: 0,
            exact: false,
            below: None,
            above: None,
            candidate_index: 0,
            candidate: 0,
            best: None,
            sum_scale: 0,
            candidate_is_high: false,
            ordering: Ordering::Equal,
            output_len: 0,
            output_index: 0,
            phase: Phase::Start,
        })
    }
    /// The next effect the cursor needs from its caller, or `None` once the cursor has finished.
    /// Perform and admit it, then pass the matching `HtmlRangeNumericReply` to `reply`; the same
    /// request stays pending until a valid reply arrives.
    pub fn request(&self) -> Option<HtmlRangeNumericRequest> {
        self.io.request
    }
    /// The cursor's copy of the buffer headers, as updated by the replies it accepted.
    pub fn headers(&self) -> HtmlRangeNumericHeaders {
        self.io.headers
    }
    /// The number of output units emitted once the cursor has finished, or `None` while it is
    /// running.
    pub fn result(&self) -> Option<usize> {
        matches!(self.phase, Phase::Done).then_some(self.output_len)
    }
    #[cfg(test)]
    pub(crate) fn selected(&self) -> Option<(u64, usize, i32)> {
        self.best.map(|(s, n)| (s, self.k, n))
    }
    fn validate_active(&self) -> Result<(), HtmlRangeNumericError> {
        let request = match (self.phase, &self.active) {
            (Phase::Build, ActiveHelper::Build(helper)) => helper.request(),
            (Phase::SumBuild, ActiveHelper::Build(helper)) => helper.request(),
            (Phase::Cut, ActiveHelper::Cut(helper)) => helper.request(),
            (Phase::Compare, ActiveHelper::Compare(helper)) => helper.request(),
            (Phase::Build | Phase::SumBuild | Phase::Cut | Phase::Compare, _) => {
                return Err(HtmlRangeNumericError::Integrity);
            }
            (_, ActiveHelper::None) => return Ok(()),
            _ => return Err(HtmlRangeNumericError::Integrity),
        };
        if request == self.io.request {
            Ok(())
        } else {
            Err(HtmlRangeNumericError::Integrity)
        }
    }
    #[cfg(test)]
    pub(crate) fn active_kind(&self) -> u8 {
        match self.active {
            ActiveHelper::None => 0,
            ActiveHelper::Build(..) => 1,
            ActiveHelper::Cut(..) => 2,
            ActiveHelper::Compare(..) => 3,
        }
    }
    /// Accepts the caller's answer to the pending request and advances. A reply that does not match
    /// the request, or violates the headers, returns an error and leaves the cursor exactly as it
    /// was.
    pub fn reply(&mut self, reply: HtmlRangeNumericReply) -> Result<(), HtmlRangeNumericError> {
        let mut next = *self;
        next.validate_active()?;
        next.io.accept(reply)?;
        next.advance(reply)?;
        next.validate_active()?;
        *self = next;
        Ok(())
    }
    fn scalar(&mut self, phase: Phase) {
        self.active = ActiveHelper::None;
        self.phase = phase;
        self.io.queue(HtmlRangeNumericRequest::ScalarStep);
    }
    fn begin_build(&mut self) -> Result<(), HtmlRangeNumericError> {
        let (role, m, e) = match self.build_index {
            0 => (
                HtmlRangeScratchRole::Coefficient,
                self.mantissa,
                self.exponent,
            ),
            1 => {
                let (m, e) = if self.bits == 1 {
                    (1, -1075)
                } else if self.bits & ((1u64 << 52) - 1) == 0 && (self.bits >> 52) > 1 {
                    (self.mantissa * 4 - 1, self.exponent - 2)
                } else {
                    (self.mantissa * 2 - 1, self.exponent - 1)
                };
                (HtmlRangeScratchRole::Lower, m, e)
            }
            2 => (
                HtmlRangeScratchRole::Upper,
                self.mantissa * 2 + 1,
                self.exponent - 1,
            ),
            3 => (HtmlRangeScratchRole::Left, self.mantissa * 2, self.exponent),
            _ => return Err(HtmlRangeNumericError::Integrity),
        };
        let build = RangeDyadicCursor::new(self.io.headers, role, m, e)?;
        self.io.request = build.request();
        self.active = ActiveHelper::Build(build);
        self.phase = Phase::Build;
        Ok(())
    }
    fn begin_cut(&mut self) -> Result<(), HtmlRangeNumericError> {
        let index = usize::from(self.cut_index);
        let role = match index {
            0 => HtmlRangeScratchRole::Lower,
            1 => HtmlRangeScratchRole::Upper,
            2 => HtmlRangeScratchRole::Coefficient,
            _ => return Err(HtmlRangeNumericError::Integrity),
        };
        let scale = self.scales[match index {
            0 => 1,
            1 => 2,
            _ => 0,
        }];
        let cut = RangePowerTenCutCursor::new(
            self.io.headers,
            role,
            i128::from(scale),
            i128::from(self.n) - self.k as i128,
        )?;
        self.io.request = cut.request();
        self.active = ActiveHelper::Cut(cut);
        self.phase = Phase::Cut;
        Ok(())
    }
    fn start_candidate(&mut self, s: u64) -> Result<(), HtmlRangeNumericError> {
        self.candidate = s;
        let Some((old, old_n)) = self.best else {
            self.best = Some((s, self.n));
            self.scalar(Phase::NextCandidate);
            return Ok(());
        };
        if old == s && old_n == self.n {
            self.scalar(Phase::NextCandidate);
            return Ok(());
        }
        self.candidate_is_high = self.n > old_n || self.n == old_n && s > old;
        let (low, low_n, high, high_n) = if self.candidate_is_high {
            (old, old_n, s, self.n)
        } else {
            (s, self.n, old, old_n)
        };
        let shift = high_n - low_n;
        if !(0..=1).contains(&shift) {
            return Err(HtmlRangeNumericError::Integrity);
        }
        let sum = high
            .checked_mul(if shift == 0 { 1 } else { 10 })
            .and_then(|v| v.checked_add(low))
            .ok_or(HtmlRangeNumericError::Representation)?;
        self.sum_scale = low_n - self.k as i32;
        let build = RangeDyadicCursor::new(self.io.headers, HtmlRangeScratchRole::Trial, sum, 0)?;
        self.io.request = build.request();
        self.active = ActiveHelper::Build(build);
        self.phase = Phase::SumBuild;
        Ok(())
    }
    fn next_candidate(&mut self) -> Result<(), HtmlRangeNumericError> {
        match self.candidate_index {
            0 => {
                if let Some(s) = self.below {
                    self.start_candidate(s)?;
                } else {
                    self.scalar(Phase::NextCandidate);
                }
            }
            1 => {
                if let Some(s) = self.above {
                    self.start_candidate(s)?;
                } else {
                    self.scalar(Phase::NextCandidate);
                }
            }
            _ => self.scalar(Phase::NextOrder),
        }
        Ok(())
    }
    fn decimal_digit(s: u64, k: usize, position: usize) -> u16 {
        0x30 + ((s / POW10[k - 1 - position]) % 10) as u16
    }
    fn unit_at(&self, position: usize) -> Result<u16, HtmlRangeNumericError> {
        if self.bits == 0 {
            return if position == 0 {
                Ok(0x30)
            } else {
                Err(HtmlRangeNumericError::Integrity)
            };
        }
        if self.negative && position == 0 {
            return Ok(0x2d);
        }
        let p = position - usize::from(self.negative);
        let (s, n) = self.best.ok_or(HtmlRangeNumericError::Integrity)?;
        if n > 0 && n <= 21 {
            let n = n as usize;
            Ok(if self.k <= n {
                if p < self.k {
                    Self::decimal_digit(s, self.k, p)
                } else {
                    0x30
                }
            } else if p == n {
                0x2e
            } else {
                Self::decimal_digit(s, self.k, if p < n { p } else { p - 1 })
            })
        } else if (-5..=0).contains(&n) {
            Ok(if p == 0 {
                0x30
            } else if p == 1 {
                0x2e
            } else if p < 2 + (-n) as usize {
                0x30
            } else {
                Self::decimal_digit(s, self.k, p - 2 - (-n) as usize)
            })
        } else {
            let significand = self.k + usize::from(self.k > 1);
            if p < significand {
                return Ok(if p == 1 && self.k > 1 {
                    0x2e
                } else {
                    Self::decimal_digit(s, self.k, if p == 0 { 0 } else { p - 1 })
                });
            }
            if p == significand {
                return Ok(0x65);
            }
            if p == significand + 1 {
                return Ok(if n - 1 < 0 { 0x2d } else { 0x2b });
            }
            let exponent = (n - 1).unsigned_abs();
            let digits = if exponent >= 100 {
                3
            } else if exponent >= 10 {
                2
            } else {
                1
            };
            let index = p - significand - 2;
            if index >= digits {
                return Err(HtmlRangeNumericError::Integrity);
            }
            Ok(0x30 + ((u64::from(exponent) / POW10[digits - 1 - index]) % 10) as u16)
        }
    }
    fn advance(&mut self, reply: HtmlRangeNumericReply) -> Result<(), HtmlRangeNumericError> {
        use HtmlRangeNumericRequest as Q;
        use Phase as S;
        match self.phase {
            S::Start => {
                if self.bits == 0 {
                    self.output_len = 1;
                    self.scalar(S::Output);
                } else {
                    self.begin_build()?;
                }
            }
            S::Build => {
                let ActiveHelper::Build(build) = &mut self.active else {
                    return Err(HtmlRangeNumericError::Integrity);
                };
                build.accept_in_place(reply)?;
                self.io.headers = build.headers();
                if let Some(scale) = build.result() {
                    self.scales[usize::from(self.build_index)] = scale;
                    self.build_index += 1;
                    if self.build_index == 4 {
                        self.scalar(S::Order);
                    } else {
                        self.begin_build()?;
                    }
                } else {
                    self.io.request = build.request();
                }
            }
            S::Order => {
                self.lower_order = self.io.len(HtmlRangeScratchRole::Lower) as i32 + self.scales[1];
                self.upper_order = self.io.len(HtmlRangeScratchRole::Upper) as i32 + self.scales[2];
                if self.upper_order - self.lower_order > 1 || self.lower_order > self.upper_order {
                    return Err(HtmlRangeNumericError::Integrity);
                }
                self.n = self.lower_order;
                self.cut_index = 0;
                self.begin_cut()?;
            }
            S::Cut => {
                let ActiveHelper::Cut(cut) = &mut self.active else {
                    return Err(HtmlRangeNumericError::Integrity);
                };
                cut.accept_in_place(reply)?;
                if let Some((floor, exact)) = cut.result() {
                    match self.cut_index {
                        0 => {
                            self.lower = floor
                                .checked_add(u64::from(!exact || !self.inclusive))
                                .ok_or(HtmlRangeNumericError::Representation)?
                        }
                        1 => self.upper = floor.saturating_sub(u64::from(exact && !self.inclusive)),
                        2 => {
                            self.floor = floor;
                            self.exact = exact;
                        }
                        _ => return Err(HtmlRangeNumericError::Integrity),
                    }
                    self.cut_index += 1;
                    if self.cut_index == 3 {
                        self.scalar(S::Candidates);
                    } else {
                        self.begin_cut()?;
                    }
                } else {
                    self.io.request = cut.request();
                }
            }
            S::Candidates => {
                self.lower = self.lower.max(POW10[self.k - 1]);
                self.upper = self.upper.min(POW10[self.k] - 1);
                self.below = None;
                self.above = None;
                if self.lower <= self.upper {
                    let mut below = self.floor.min(self.upper);
                    if below.is_multiple_of(10) {
                        below = below.saturating_sub(1);
                    }
                    if below >= self.lower {
                        self.below = Some(below);
                    }
                    let mut above = self
                        .floor
                        .checked_add(u64::from(!self.exact))
                        .ok_or(HtmlRangeNumericError::Representation)?
                        .max(self.lower);
                    if above.is_multiple_of(10) {
                        above = above
                            .checked_add(1)
                            .ok_or(HtmlRangeNumericError::Representation)?;
                    }
                    if above <= self.upper {
                        self.above = Some(above);
                    }
                }
                self.candidate_index = 0;
                self.next_candidate()?;
            }
            S::NextCandidate => {
                self.candidate_index += 1;
                self.next_candidate()?;
            }
            S::SumBuild => {
                let ActiveHelper::Build(build) = &mut self.active else {
                    return Err(HtmlRangeNumericError::Integrity);
                };
                build.accept_in_place(reply)?;
                self.io.headers = build.headers();
                if build.result().is_some() {
                    let compare = RangeScaledComparisonCursor::new(
                        self.io.headers,
                        HtmlRangeScratchRole::Left,
                        i128::from(self.scales[3]),
                        false,
                        HtmlRangeScratchRole::Trial,
                        i128::from(self.sum_scale),
                        false,
                    );
                    self.io.request = compare.request();
                    self.active = ActiveHelper::Compare(compare);
                    self.phase = S::Compare;
                } else {
                    self.io.request = build.request();
                }
            }
            S::Compare => {
                let ActiveHelper::Compare(compare) = &mut self.active else {
                    return Err(HtmlRangeNumericError::Integrity);
                };
                compare.accept_in_place(reply)?;
                if let Some(order) = compare.result() {
                    self.ordering = order;
                    self.scalar(S::Winner);
                } else {
                    self.io.request = compare.request();
                }
            }
            S::Winner => {
                let (old, _) = self.best.ok_or(HtmlRangeNumericError::Integrity)?;
                if prefer_candidate(self.ordering, self.candidate_is_high, self.candidate, old) {
                    self.best = Some((self.candidate, self.n));
                }
                self.scalar(S::NextCandidate);
            }
            S::NextOrder => {
                if self.n < self.upper_order {
                    self.n += 1;
                    self.cut_index = 0;
                    self.begin_cut()?;
                } else {
                    self.scalar(S::NextK);
                }
            }
            S::NextK => {
                if let Some((_, n)) = self.best {
                    let length = if n > 0 && n <= 21 {
                        if self.k <= n as usize {
                            n as usize
                        } else {
                            self.k + 1
                        }
                    } else if (-5..=0).contains(&n) {
                        2 + (-n) as usize + self.k
                    } else {
                        let exponent = (n - 1).unsigned_abs();
                        self.k
                            + usize::from(self.k > 1)
                            + 2
                            + if exponent >= 100 {
                                3
                            } else if exponent >= 10 {
                                2
                            } else {
                                1
                            }
                    };
                    self.output_len = length + usize::from(self.negative);
                    if self.output_len > 25 {
                        return Err(HtmlRangeNumericError::Representation);
                    }
                    self.scalar(S::Output);
                } else if self.k == 17 {
                    return Err(HtmlRangeNumericError::Representation);
                } else {
                    self.k += 1;
                    self.n = self.lower_order;
                    self.cut_index = 0;
                    self.begin_cut()?;
                }
            }
            S::Output => {
                self.phase = S::OutputReserve;
                self.io.queue(Q::ReserveOutput {
                    required_len: self.output_len,
                });
            }
            S::OutputReserve => {
                self.phase = S::Emit;
                self.io.queue(Q::EmitUnit {
                    unit: self.unit_at(0)?,
                });
            }
            S::Emit => {
                self.output_index += 1;
                if self.output_index == self.output_len {
                    self.phase = S::Done;
                    self.io.request = None;
                } else {
                    self.io.queue(Q::EmitUnit {
                        unit: self.unit_at(self.output_index)?,
                    });
                }
            }
            S::Done => return Err(HtmlRangeNumericError::Integrity),
        }
        Ok(())
    }
}

pub(super) fn prefer_candidate(
    order: Ordering,
    candidate_is_high: bool,
    candidate: u64,
    old: u64,
) -> bool {
    match order {
        Ordering::Less => !candidate_is_high,
        Ordering::Greater => candidate_is_high,
        Ordering::Equal => candidate.is_multiple_of(2) && !old.is_multiple_of(2),
    }
}

impl HtmlBestFiniteNumberCursor {
    /// The fixed state and message sizes of this cursor.
    pub const fn layout() -> HtmlRangeNumericLayout {
        HtmlRangeNumericLayout {
            registers: size_of::<Self>(),
        }
    }
}

#[cfg(test)]
mod exclusive_tests {
    use super::*;
    #[test]
    fn active_phase_and_cached_request_mismatches_are_transactional_refusals() {
        let headers = HtmlRangeNumericHeaders {
            buffers: [HtmlRangeBufferHeader {
                len: 0,
                capacity: 0,
            }; 8],
            output: HtmlRangeBufferHeader {
                len: 0,
                capacity: 0,
            },
        };
        let original = HtmlBestFiniteNumberCursor::new(1.0, headers).unwrap();
        let build = RangeDyadicCursor::new(headers, HtmlRangeScratchRole::Trial, 1, 0).unwrap();
        for case in 0..3 {
            let mut c = original;
            match case {
                0 => c.phase = Phase::Build,
                1 => c.active = ActiveHelper::Build(build),
                _ => {
                    c.phase = Phase::Build;
                    c.active = ActiveHelper::Build(build);
                    c.io.request = None;
                }
            }
            let snapshot = c;
            assert_eq!(
                c.reply(HtmlRangeNumericReply::Ack),
                Err(HtmlRangeNumericError::Integrity)
            );
            assert_eq!(c, snapshot);
        }
    }
}
