//! Exact finite-number arithmetic. Storage and admission belong to the caller.
//! Every repeated digit/bit operation is a separate cached request. A reply
//! copies only this fixed, ticket-free state and commits after validation.

pub(crate) mod best;
pub(crate) mod ingress;
pub(crate) mod lattice;
pub(crate) use best::BestFiniteNumberCursor;
pub(crate) use ingress::FiniteDecimalIngressCursor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeScratchRole {
    Coefficient,
    Lower,
    Upper,
    Left,
    Right,
    Quotient,
    Remainder,
    Trial,
}
impl RangeScratchRole {
    pub const ALL: [Self; 8] = [
        Self::Coefficient,
        Self::Lower,
        Self::Upper,
        Self::Left,
        Self::Right,
        Self::Quotient,
        Self::Remainder,
        Self::Trial,
    ];
    pub const fn index(self) -> usize {
        match self {
            Self::Coefficient => 0,
            Self::Lower => 1,
            Self::Upper => 2,
            Self::Left => 3,
            Self::Right => 4,
            Self::Quotient => 5,
            Self::Remainder => 6,
            Self::Trial => 7,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeCell {
    pub(crate) role: RangeScratchRole,
    pub(crate) index: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeNumericRequest {
    SourceUnit {
        index: usize,
    },
    ReadCells {
        cells: [Option<RangeCell>; 3],
    },
    Reserve {
        role: RangeScratchRole,
        required_len: usize,
    },
    AppendCell {
        role: RangeScratchRole,
        value: u8,
    },
    WriteCell {
        cell: RangeCell,
        value: u8,
    },
    PopCell {
        role: RangeScratchRole,
    },
    ScalarStep,
    ReserveOutput {
        required_len: usize,
    },
    EmitUnit {
        unit: u16,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeNumericReply {
    Unit(u16),
    Cells([Option<u8>; 3]),
    Reserved {
        actual_capacity: usize,
        actual_len: usize,
    },
    Ack,
    OutputReserved {
        actual_capacity: usize,
        actual_len: usize,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeNumericError {
    Integrity,
    Representation,
    Overflow,
    InvalidSyntax,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RangeBufferHeader {
    pub(crate) len: usize,
    pub(crate) capacity: usize,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RangeNumericHeaders {
    pub(crate) buffers: [RangeBufferHeader; 8],
    pub(crate) output: RangeBufferHeader,
}

/// Only the enclosing authoritative grammar owner can issue these headers.
#[derive(Clone, Copy, Debug)]
pub struct RangeDecimalParts {
    pub(super) negative: bool,
    pub(super) integer_start: usize,
    pub(super) integer_end: usize,
    pub(super) fraction_start: usize,
    pub(super) fraction_end: usize,
    pub(super) exponent_start: usize,
    pub(super) exponent_end: usize,
    pub(super) exponent_negative: bool,
}

#[derive(Clone, Copy, Debug)]
struct Io {
    headers: RangeNumericHeaders,
    request: Option<RangeNumericRequest>,
    source_len: usize,
    width: usize,
}
impl Io {
    fn new(headers: RangeNumericHeaders, source_len: usize, width: usize) -> Self {
        Self {
            headers,
            request: Some(RangeNumericRequest::ScalarStep),
            source_len,
            width,
        }
    }
    fn len(&self, role: RangeScratchRole) -> usize {
        self.headers.buffers[role.index()].len
    }
    fn queue(&mut self, request: RangeNumericRequest) {
        self.request = Some(request);
    }
    fn append(&mut self, role: RangeScratchRole, value: u8) -> Result<bool, RangeNumericError> {
        let required = self
            .len(role)
            .checked_add(1)
            .ok_or(RangeNumericError::Representation)?;
        if required > self.width {
            return Err(RangeNumericError::Representation);
        }
        if self.headers.buffers[role.index()].capacity < required {
            self.queue(RangeNumericRequest::Reserve {
                role,
                required_len: required,
            });
            Ok(false)
        } else {
            self.queue(RangeNumericRequest::AppendCell { role, value });
            Ok(true)
        }
    }
    fn cell_ok(&self, cell: Option<RangeCell>, value: Option<u8>) -> bool {
        match (cell, value) {
            (None, None) => true,
            (Some(c), Some(v)) => c.index < self.len(c.role) && v <= 9,
            _ => false,
        }
    }
    fn accept(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        use RangeNumericReply as P;
        use RangeNumericRequest as Q;
        let h = &self.headers.buffers;
        if h[0].len > h[0].capacity
            || h[1].len > h[1].capacity
            || h[2].len > h[2].capacity
            || h[3].len > h[3].capacity
            || h[4].len > h[4].capacity
            || h[5].len > h[5].capacity
            || h[6].len > h[6].capacity
            || h[7].len > h[7].capacity
            || self.headers.output.len > self.headers.output.capacity
        {
            return Err(RangeNumericError::Integrity);
        }
        match (self.request.ok_or(RangeNumericError::Integrity)?, reply) {
            (Q::SourceUnit { index }, P::Unit(_)) if index < self.source_len => {}
            (Q::ReadCells { cells }, P::Cells(values))
                if self.cell_ok(cells[0], values[0])
                    && self.cell_ok(cells[1], values[1])
                    && self.cell_ok(cells[2], values[2]) => {}
            (
                Q::Reserve { role, required_len },
                P::Reserved {
                    actual_capacity,
                    actual_len,
                },
            ) if actual_len == self.len(role)
                && required_len >= actual_len
                && required_len <= self.width
                && actual_capacity >= required_len
                && actual_capacity <= isize::MAX as usize =>
            {
                self.headers.buffers[role.index()].capacity = actual_capacity;
            }
            (Q::AppendCell { role, value }, P::Ack)
                if value <= 9
                    && self.len(role) < self.headers.buffers[role.index()].capacity
                    && self.len(role) < self.width =>
            {
                self.headers.buffers[role.index()].len += 1;
            }
            (Q::WriteCell { cell, value }, P::Ack)
                if value <= 9 && cell.index < self.len(cell.role) => {}
            (Q::PopCell { role }, P::Ack) if self.len(role) != 0 => {
                self.headers.buffers[role.index()].len -= 1;
            }
            (Q::ScalarStep, P::Ack) => {}
            (
                Q::ReserveOutput { required_len },
                P::OutputReserved {
                    actual_capacity,
                    actual_len,
                },
            ) if actual_len == self.headers.output.len
                && required_len >= actual_len
                && required_len <= 25
                && actual_capacity >= required_len
                && actual_capacity <= isize::MAX as usize =>
            {
                self.headers.output.capacity = actual_capacity;
            }
            (Q::EmitUnit { .. }, P::Ack)
                if self.headers.output.len < self.headers.output.capacity
                    && self.headers.output.len < 25 =>
            {
                self.headers.output.len += 1;
            }
            _ => return Err(RangeNumericError::Integrity),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum ProductPhase {
    Start,
    Clear,
    Read,
    Reserve,
    Append,
    Next,
    Carry,
    CarryReserve,
    CarryAppend,
    Done,
}
/// Exact distinct-role decimal coefficient times a fixed u64 mantissa.
/// d*M+carry < 10*M < 2^68; each digit and carry is a separate request.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeMantissaProductCursor {
    io: Io,
    source: RangeScratchRole,
    destination: RangeScratchRole,
    mantissa: u64,
    carry: u128,
    index: usize,
    digit: u8,
    phase: ProductPhase,
}
impl RangeMantissaProductCursor {
    pub(crate) fn new(
        headers: RangeNumericHeaders,
        source: RangeScratchRole,
        destination: RangeScratchRole,
        mantissa: u64,
    ) -> Result<Self, RangeNumericError> {
        if source == destination || headers.buffers[source.index()].len == 0 || mantissa == 0 {
            return Err(RangeNumericError::Integrity);
        }
        Ok(Self {
            io: Io::new(headers, 0, 1386),
            source,
            destination,
            mantissa,
            carry: 0,
            index: 0,
            digit: 0,
            phase: ProductPhase::Start,
        })
    }
    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        self.io.request
    }
    pub(crate) fn headers(&self) -> RangeNumericHeaders {
        self.io.headers
    }
    pub(crate) fn result(&self) -> Option<()> {
        matches!(self.phase, ProductPhase::Done).then_some(())
    }
    #[cfg(test)]
    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        let mut next = *self;
        next.accept_in_place(reply)?;
        *self = next;
        Ok(())
    }
    fn accept_in_place(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        self.io.accept(reply)?;
        use ProductPhase as S;
        use RangeNumericRequest as Q;
        match self.phase {
            S::Start | S::Clear => {
                if self.io.len(self.destination) != 0 {
                    self.phase = S::Clear;
                    self.io.queue(Q::PopCell {
                        role: self.destination,
                    });
                } else {
                    self.phase = S::Read;
                    self.queue_read();
                }
            }
            S::Read => {
                let RangeNumericReply::Cells([Some(digit), None, None]) = reply else {
                    return Err(RangeNumericError::Integrity);
                };
                let value = u128::from(digit) * u128::from(self.mantissa) + self.carry;
                self.digit = (value % 10) as u8;
                self.carry = value / 10;
                self.phase = if self.io.append(self.destination, self.digit)? {
                    S::Append
                } else {
                    S::Reserve
                };
            }
            S::Reserve | S::CarryReserve => {
                self.phase = if matches!(self.phase, S::Reserve) {
                    S::Append
                } else {
                    S::CarryAppend
                };
                self.io.queue(Q::AppendCell {
                    role: self.destination,
                    value: self.digit,
                });
            }
            S::Append => {
                self.phase = S::Next;
                self.io.queue(Q::ScalarStep);
            }
            S::Next => {
                self.index = self
                    .index
                    .checked_add(1)
                    .ok_or(RangeNumericError::Representation)?;
                if self.index < self.io.len(self.source) {
                    self.phase = S::Read;
                    self.queue_read();
                } else {
                    self.phase = S::Carry;
                    self.io.queue(Q::ScalarStep);
                }
            }
            S::Carry => {
                if self.carry == 0 {
                    self.phase = S::Done;
                    self.io.request = None;
                } else {
                    self.digit = (self.carry % 10) as u8;
                    self.carry /= 10;
                    self.phase = if self.io.append(self.destination, self.digit)? {
                        S::CarryAppend
                    } else {
                        S::CarryReserve
                    };
                }
            }
            S::CarryAppend => {
                self.phase = S::Carry;
                self.io.queue(Q::ScalarStep);
            }
            S::Done => return Err(RangeNumericError::Integrity),
        }
        Ok(())
    }
    fn queue_read(&mut self) {
        self.io.queue(RangeNumericRequest::ReadCells {
            cells: [
                Some(RangeCell {
                    role: self.source,
                    index: self.index,
                }),
                None,
                None,
            ],
        });
    }
}

#[derive(Clone, Copy, Debug)]
enum BuildPhase {
    Start,
    Clear,
    Normalize,
    Seed,
    SeedReserve,
    SeedAppend,
    MultiplyStart,
    Read,
    Write,
    NextCell,
    CarryReserve,
    CarryAppend,
    Done,
}

/// Coarse exact interval proof for a positive dyadic trial, using only prepaid
/// headers/scalars. Unknown overlap always proceeds to the exact cell comparison.
/// Decimal X lies in [10^(d-1),10^d); T lies in [2^b,2^(b+1)). Since
/// 2^3 < 10 < 2^4, the inequalities below stay valid at their open upper ends.
fn decimal_dyadic_ordering(
    decimal_order: i128,
    mantissa: u64,
    exponent: i32,
) -> Result<Option<std::cmp::Ordering>, RangeNumericError> {
    if mantissa == 0 {
        return Ok(None);
    }
    let binary_order = i128::from(63 - mantissa.leading_zeros())
        .checked_add(i128::from(exponent))
        .ok_or(RangeNumericError::Representation)?;
    let lower_decimal = decimal_order
        .checked_sub(1)
        .ok_or(RangeNumericError::Representation)?;
    let (upper_binary, lower_binary) = if decimal_order >= 1 {
        (decimal_order.checked_mul(4), lower_decimal.checked_mul(3))
    } else {
        (decimal_order.checked_mul(3), lower_decimal.checked_mul(4))
    };
    let upper_binary = upper_binary.ok_or(RangeNumericError::Representation)?;
    let lower_binary = lower_binary.ok_or(RangeNumericError::Representation)?;
    if binary_order >= upper_binary {
        Ok(Some(std::cmp::Ordering::Less))
    } else if binary_order
        .checked_add(1)
        .ok_or(RangeNumericError::Representation)?
        <= lower_binary
    {
        Ok(Some(std::cmp::Ordering::Greater))
    } else {
        Ok(None)
    }
}
/// Builds M*2^E as exact coefficient*10^scale. The scalar inventory is a
/// u64 seed, u16 cell/carry, bounded exponent/pass/index and fixed headers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeDyadicCursor {
    io: Io,
    role: RangeScratchRole,
    seed: u64,
    exponent: i32,
    remaining: u16,
    multiplier: u8,
    scale: i32,
    index: usize,
    carry: u16,
    digit: u8,
    phase: BuildPhase,
}
impl RangeDyadicCursor {
    pub(crate) fn new(
        headers: RangeNumericHeaders,
        role: RangeScratchRole,
        mantissa: u64,
        exponent: i32,
    ) -> Result<Self, RangeNumericError> {
        if !(-1075..=1024).contains(&exponent) {
            return Err(RangeNumericError::Representation);
        }
        Ok(Self {
            io: Io::new(headers, 0, 1386),
            role,
            seed: mantissa,
            exponent,
            remaining: 0,
            multiplier: 0,
            scale: 0,
            index: 0,
            carry: 0,
            digit: 0,
            phase: BuildPhase::Start,
        })
    }
    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        self.io.request
    }
    pub(crate) fn headers(&self) -> RangeNumericHeaders {
        self.io.headers
    }
    pub(crate) fn result(&self) -> Option<i32> {
        matches!(self.phase, BuildPhase::Done).then_some(self.scale)
    }
    #[cfg(test)]
    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        let mut next = *self;
        next.accept_in_place(reply)?;
        *self = next;
        Ok(())
    }
    // An enclosing cursor already owns its transactional receiver. It invokes
    // this transition directly, avoiding a second uncharged receiver copy.
    fn accept_in_place(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        self.io.accept(reply)?;
        self.advance(reply)
    }
    fn clear_or_normalize(&mut self) {
        if self.io.len(self.role) != 0 {
            self.phase = BuildPhase::Clear;
            self.io
                .queue(RangeNumericRequest::PopCell { role: self.role });
        } else {
            self.phase = BuildPhase::Normalize;
            self.io.queue(RangeNumericRequest::ScalarStep);
        }
    }
    fn advance(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        use BuildPhase as S;
        use RangeNumericRequest as Q;
        match self.phase {
            S::Start | S::Clear => self.clear_or_normalize(),
            S::Normalize => {
                // M*2^E = (M/2)*2^(E+1). One admitted scalar reply
                // removes one factor; no digit is read or written here.
                if self.seed != 0 && self.seed.is_multiple_of(2) {
                    self.seed /= 2;
                    self.exponent = self
                        .exponent
                        .checked_add(1)
                        .ok_or(RangeNumericError::Representation)?;
                    self.io.queue(Q::ScalarStep);
                } else {
                    self.remaining = if self.seed == 0 {
                        0
                    } else {
                        u16::try_from(self.exponent.unsigned_abs())
                            .map_err(|_| RangeNumericError::Representation)?
                    };
                    self.multiplier = if self.exponent < 0 { 5 } else { 2 };
                    self.scale = if self.seed == 0 {
                        0
                    } else {
                        self.exponent.min(0)
                    };
                    self.phase = S::Seed;
                    self.io.queue(Q::ScalarStep);
                }
            }
            S::Seed => {
                self.digit = (self.seed % 10) as u8;
                self.phase = if self.io.append(self.role, self.digit)? {
                    S::SeedAppend
                } else {
                    S::SeedReserve
                };
            }
            S::SeedReserve => {
                self.phase = S::SeedAppend;
                self.io.queue(Q::AppendCell {
                    role: self.role,
                    value: self.digit,
                });
            }
            S::SeedAppend => {
                self.seed /= 10;
                self.phase = if self.seed != 0 {
                    S::Seed
                } else {
                    S::MultiplyStart
                };
                self.io.queue(Q::ScalarStep);
            }
            S::MultiplyStart => {
                if self.remaining == 0 {
                    self.phase = S::Done;
                    self.io.request = None;
                } else {
                    self.index = 0;
                    self.carry = 0;
                    self.phase = S::Read;
                    self.io.queue(Q::ReadCells {
                        cells: [
                            Some(RangeCell {
                                role: self.role,
                                index: 0,
                            }),
                            None,
                            None,
                        ],
                    });
                }
            }
            S::Read => {
                let RangeNumericReply::Cells([Some(digit), None, None]) = reply else {
                    return Err(RangeNumericError::Integrity);
                };
                let value = u16::from(digit) * u16::from(self.multiplier) + self.carry;
                self.digit = (value % 10) as u8;
                self.carry = value / 10;
                self.phase = S::Write;
                self.io.queue(Q::WriteCell {
                    cell: RangeCell {
                        role: self.role,
                        index: self.index,
                    },
                    value: self.digit,
                });
            }
            S::Write => {
                self.phase = S::NextCell;
                self.io.queue(Q::ScalarStep);
            }
            S::NextCell => {
                self.index += 1;
                if self.index < self.io.len(self.role) {
                    self.phase = S::Read;
                    self.io.queue(Q::ReadCells {
                        cells: [
                            Some(RangeCell {
                                role: self.role,
                                index: self.index,
                            }),
                            None,
                            None,
                        ],
                    });
                } else if self.carry != 0 {
                    self.digit = self.carry as u8;
                    self.phase = if self.io.append(self.role, self.digit)? {
                        S::CarryAppend
                    } else {
                        S::CarryReserve
                    };
                } else {
                    self.remaining -= 1;
                    self.phase = S::MultiplyStart;
                    self.io.queue(Q::ScalarStep);
                }
            }
            S::CarryReserve => {
                self.phase = S::CarryAppend;
                self.io.queue(Q::AppendCell {
                    role: self.role,
                    value: self.digit,
                });
            }
            S::CarryAppend => {
                self.remaining -= 1;
                self.phase = S::MultiplyStart;
                self.io.queue(Q::ScalarStep);
            }
            S::Done => return Err(RangeNumericError::Integrity),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeScaledComparisonCursor {
    io: Io,
    left: RangeScratchRole,
    right: RangeScratchRole,
    left_scale: i128,
    right_scale: i128,
    left_zero: bool,
    right_zero: bool,
    index: usize,
    result: Option<std::cmp::Ordering>,
    started: bool,
}
impl RangeScaledComparisonCursor {
    pub(crate) fn new(
        headers: RangeNumericHeaders,
        left: RangeScratchRole,
        left_scale: i128,
        left_zero: bool,
        right: RangeScratchRole,
        right_scale: i128,
        right_zero: bool,
    ) -> Self {
        Self {
            io: Io::new(headers, 0, usize::MAX),
            left,
            right,
            left_scale,
            right_scale,
            left_zero,
            right_zero,
            index: 0,
            result: None,
            started: false,
        }
    }
    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        self.io.request
    }
    pub(crate) fn result(&self) -> Option<std::cmp::Ordering> {
        self.result
    }
    #[cfg(test)]
    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        let mut next = *self;
        next.accept_in_place(reply)?;
        *self = next;
        Ok(())
    }
    // An enclosing cursor already owns its transactional receiver. It invokes
    // this transition directly, avoiding a second uncharged receiver copy.
    fn accept_in_place(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        self.io.accept(reply)?;
        self.advance(reply)
    }
    fn queue_cells(&mut self) {
        let l = self.io.len(self.left);
        let r = self.io.len(self.right);
        self.io.queue(RangeNumericRequest::ReadCells {
            cells: [
                if self.index < l {
                    Some(RangeCell {
                        role: self.left,
                        index: l - 1 - self.index,
                    })
                } else {
                    None
                },
                if self.index < r {
                    Some(RangeCell {
                        role: self.right,
                        index: r - 1 - self.index,
                    })
                } else {
                    None
                },
                None,
            ],
        });
    }
    fn finish(&mut self, result: std::cmp::Ordering) {
        self.result = Some(result);
        self.io.request = None;
    }
    fn advance(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        use std::cmp::Ordering as O;
        if !self.started {
            self.started = true;
            if self.left_zero || self.right_zero {
                self.finish(match (self.left_zero, self.right_zero) {
                    (true, true) => O::Equal,
                    (true, false) => O::Less,
                    _ => O::Greater,
                });
            } else {
                if self.io.len(self.left) == 0 || self.io.len(self.right) == 0 {
                    return Err(RangeNumericError::Integrity);
                }
                let l = (self.io.len(self.left) as i128)
                    .checked_add(self.left_scale)
                    .ok_or(RangeNumericError::Representation)?;
                let r = (self.io.len(self.right) as i128)
                    .checked_add(self.right_scale)
                    .ok_or(RangeNumericError::Representation)?;
                if l != r {
                    self.finish(l.cmp(&r));
                } else {
                    self.queue_cells();
                }
            }
        } else {
            let RangeNumericReply::Cells([l, r, None]) = reply else {
                return Err(RangeNumericError::Integrity);
            };
            let order = l.unwrap_or(0).cmp(&r.unwrap_or(0));
            if order != O::Equal {
                self.finish(order);
            } else {
                self.index += 1;
                if self.index == self.io.len(self.left).max(self.io.len(self.right)) {
                    self.finish(O::Equal);
                } else {
                    self.queue_cells();
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RangePowerTenCutCursor {
    io: Io,
    role: RangeScratchRole,
    integer_cells: usize,
    zeroes: usize,
    index: usize,
    floor: u64,
    fractional: bool,
    done: bool,
    started: bool,
}
impl RangePowerTenCutCursor {
    pub(crate) fn new(
        headers: RangeNumericHeaders,
        role: RangeScratchRole,
        scale: i128,
        cut: i128,
    ) -> Result<Self, RangeNumericError> {
        let delta = scale
            .checked_sub(cut)
            .ok_or(RangeNumericError::Representation)?;
        let len = headers.buffers[role.index()].len;
        let fractional = if delta < 0 {
            usize::try_from(delta.unsigned_abs()).map_err(|_| RangeNumericError::Representation)?
        } else {
            0
        };
        let zeroes = if delta > 0 {
            usize::try_from(delta).map_err(|_| RangeNumericError::Representation)?
        } else {
            0
        };
        let integer_cells = len.saturating_sub(fractional);
        if integer_cells
            .checked_add(zeroes)
            .ok_or(RangeNumericError::Representation)?
            > 19
        {
            return Err(RangeNumericError::Representation);
        }
        Ok(Self {
            io: Io::new(headers, 0, usize::MAX),
            role,
            integer_cells,
            zeroes,
            index: 0,
            floor: 0,
            fractional: false,
            done: false,
            started: false,
        })
    }
    pub(crate) fn request(&self) -> Option<RangeNumericRequest> {
        self.io.request
    }
    pub(crate) fn result(&self) -> Option<(u64, bool)> {
        self.done.then_some((self.floor, !self.fractional))
    }
    #[cfg(test)]
    pub(crate) fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        let mut next = *self;
        next.accept_in_place(reply)?;
        *self = next;
        Ok(())
    }
    // An enclosing cursor already owns its transactional receiver. It invokes
    // this transition directly, avoiding a second uncharged receiver copy.
    fn accept_in_place(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        self.io.accept(reply)?;
        self.advance(reply)
    }
    fn queue_next(&mut self) {
        let len = self.io.len(self.role);
        if self.index < len {
            self.io.queue(RangeNumericRequest::ReadCells {
                cells: [
                    Some(RangeCell {
                        role: self.role,
                        index: len - 1 - self.index,
                    }),
                    None,
                    None,
                ],
            });
        } else if self.zeroes != 0 {
            self.io.queue(RangeNumericRequest::ScalarStep);
        } else {
            self.done = true;
            self.io.request = None;
        }
    }
    fn advance(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        if !self.started {
            self.started = true;
            if self.io.len(self.role) == 0 {
                return Err(RangeNumericError::Integrity);
            }
            self.queue_next();
        } else if self.index < self.io.len(self.role) {
            let RangeNumericReply::Cells([Some(digit), None, None]) = reply else {
                return Err(RangeNumericError::Integrity);
            };
            if self.index < self.integer_cells {
                self.floor = self
                    .floor
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(u64::from(digit)))
                    .ok_or(RangeNumericError::Representation)?;
            } else {
                self.fractional |= digit != 0;
            }
            self.index += 1;
            self.queue_next();
        } else {
            self.floor = self
                .floor
                .checked_mul(10)
                .ok_or(RangeNumericError::Representation)?;
            self.zeroes -= 1;
            self.queue_next();
        }
        Ok(())
    }
}

pub(crate) fn finite_dyadic(bits: u64) -> Result<(u64, i32), RangeNumericError> {
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = bits & ((1u64 << 52) - 1);
    if exponent == 0x7ff {
        return Err(RangeNumericError::Overflow);
    }
    Ok(if exponent == 0 {
        (mantissa, -1074)
    } else {
        (mantissa | (1u64 << 52), exponent - 1075)
    })
}
fn upper_midpoint(bits: u64) -> Result<(u64, i32), RangeNumericError> {
    let (mantissa, exponent) = finite_dyadic(bits)?;
    Ok((mantissa * 2 + 1, exponent - 1))
}
const POW10: [u64; 19] = [
    1,
    10,
    100,
    1000,
    10000,
    100000,
    1000000,
    10000000,
    100000000,
    1000000000,
    10000000000,
    100000000000,
    1000000000000,
    10000000000000,
    100000000000000,
    1000000000000000,
    10000000000000000,
    100000000000000000,
    1000000000000000000,
];

impl RangeCell {
    pub const fn new(role: RangeScratchRole, index: usize) -> Self {
        Self { role, index }
    }
    pub const fn role(self) -> RangeScratchRole {
        self.role
    }
    pub const fn index(self) -> usize {
        self.index
    }
}
impl RangeBufferHeader {
    /// Describe actual caller-owned backing. A live length cannot exceed it.
    pub fn new(len: usize, capacity: usize) -> Result<Self, RangeNumericError> {
        if len > capacity || capacity > isize::MAX as usize {
            return Err(RangeNumericError::Integrity);
        }
        Ok(Self { len, capacity })
    }
    pub const fn len(self) -> usize {
        self.len
    }
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
    pub const fn capacity(self) -> usize {
        self.capacity
    }
}
impl RangeNumericHeaders {
    pub const fn new(buffers: [RangeBufferHeader; 8], output: RangeBufferHeader) -> Self {
        Self { buffers, output }
    }
    pub const fn buffer(&self, role: RangeScratchRole) -> RangeBufferHeader {
        self.buffers[role.index()]
    }
    pub const fn output(&self) -> RangeBufferHeader {
        self.output
    }
}
/// Fixed layouts before construction; variable scratch and output belong to
/// the caller. Requested lengths and actual capacities travel in the protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeNumericLayout {
    registers: usize,
}
impl RangeNumericLayout {
    pub const fn register_bytes(self) -> usize {
        self.registers
    }
    pub const fn request_bytes(self) -> usize {
        size_of::<RangeNumericRequest>()
    }
    pub const fn reply_bytes(self) -> usize {
        size_of::<RangeNumericReply>()
    }
    pub const fn headers_bytes(self) -> usize {
        size_of::<RangeNumericHeaders>()
    }
    pub const fn scratch_cell_bytes(self) -> usize {
        size_of::<u8>()
    }
    pub const fn output_unit_bytes(self) -> usize {
        size_of::<u16>()
    }
}
impl RangeNumericRequest {
    /// Operand payload bytes inspected by the described action, excluding the
    /// fixed request/reply/register layouts and caller's allocation metadata.
    pub fn read_bytes(self) -> usize {
        match self {
            Self::SourceUnit { .. } => size_of::<u16>(),
            Self::ReadCells { cells } => cells.into_iter().flatten().count(),
            _ => 0,
        }
    }
    /// Initialized payload written after admission; reservations describe their
    /// requested logical length separately and report actual capacity in reply.
    pub const fn write_bytes(self) -> usize {
        match self {
            Self::AppendCell { .. } | Self::WriteCell { .. } => size_of::<u8>(),
            Self::EmitUnit { .. } => size_of::<u16>(),
            _ => 0,
        }
    }
}
impl std::fmt::Display for RangeNumericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RangeNumericError {}

#[cfg(test)]
mod tests {
    use super::super::{FloatingPointGrammarCursor, FloatingPointParseMode};
    use super::*;

    type Serialized = (String, Option<(u64, usize, i32)>);

    struct Driver {
        source: Vec<u16>,
        scratch: [Vec<u8>; 8],
        output: Vec<u16>,
        requests: [usize; 9],
    }
    impl Driver {
        fn new(source: &str) -> Self {
            Self {
                source: source.encode_utf16().collect(),
                scratch: std::array::from_fn(|_| Vec::new()),
                output: Vec::new(),
                requests: [0; 9],
            }
        }
        fn headers(&self) -> RangeNumericHeaders {
            RangeNumericHeaders {
                buffers: std::array::from_fn(|i| RangeBufferHeader {
                    len: self.scratch[i].len(),
                    capacity: self.scratch[i].capacity(),
                }),
                output: RangeBufferHeader {
                    len: self.output.len(),
                    capacity: self.output.capacity(),
                },
            }
        }
        // Ordinary test adapter: these allocations are not a controlled parser
        // claim. Actual admission/qualification is tested by the session owner.
        fn fulfill(&mut self, request: RangeNumericRequest) -> RangeNumericReply {
            use RangeNumericReply as P;
            use RangeNumericRequest as Q;
            match request {
                Q::SourceUnit { index } => {
                    self.requests[0] += 1;
                    P::Unit(self.source[index])
                }
                Q::ReadCells { cells } => {
                    self.requests[1] += 1;
                    P::Cells(cells.map(|c| c.map(|c| self.scratch[c.role.index()][c.index])))
                }
                Q::Reserve { role, required_len } => {
                    self.requests[2] += 1;
                    let v = &mut self.scratch[role.index()];
                    v.try_reserve_exact(required_len - v.len()).unwrap();
                    P::Reserved {
                        actual_capacity: v.capacity(),
                        actual_len: v.len(),
                    }
                }
                Q::AppendCell { role, value } => {
                    self.requests[3] += 1;
                    let v = &mut self.scratch[role.index()];
                    assert!(v.len() < v.capacity());
                    v.push(value);
                    P::Ack
                }
                Q::WriteCell { cell, value } => {
                    self.requests[4] += 1;
                    self.scratch[cell.role.index()][cell.index] = value;
                    P::Ack
                }
                Q::PopCell { role } => {
                    self.requests[5] += 1;
                    assert!(self.scratch[role.index()].pop().is_some());
                    P::Ack
                }
                Q::ScalarStep => {
                    self.requests[6] += 1;
                    P::Ack
                }
                Q::ReserveOutput { required_len } => {
                    self.requests[7] += 1;
                    self.output
                        .try_reserve_exact(required_len - self.output.len())
                        .unwrap();
                    P::OutputReserved {
                        actual_capacity: self.output.capacity(),
                        actual_len: self.output.len(),
                    }
                }
                Q::EmitUnit { unit } => {
                    self.requests[8] += 1;
                    assert!(self.output.len() < self.output.capacity());
                    self.output.push(unit);
                    P::Ack
                }
            }
        }
        fn parts(
            &self,
            mode: FloatingPointParseMode,
        ) -> Result<RangeDecimalParts, RangeNumericError> {
            let mut c = FloatingPointGrammarCursor::new(&self.source, mode);
            while !c.is_finished() {
                c.advance();
            }
            if !c.is_valid() {
                return Err(RangeNumericError::InvalidSyntax);
            }
            Ok(RangeDecimalParts {
                negative: c.negative,
                integer_start: c.integer_start,
                integer_end: c.integer_end,
                fraction_start: c.fraction_start,
                fraction_end: c.fraction_end,
                exponent_start: if c.has_exponent { c.exponent_start } else { 0 },
                exponent_end: if c.has_exponent { c.position } else { 0 },
                exponent_negative: c.exponent_negative,
            })
        }
        fn ingress(&mut self, mode: FloatingPointParseMode) -> Result<u64, RangeNumericError> {
            let mut c = FiniteDecimalIngressCursor::new(
                self.parts(mode)?,
                self.source.len(),
                self.headers(),
            )?;
            while let Some(q) = c.request() {
                let r = self.fulfill(q);
                c.reply(r)?;
            }
            assert_eq!(c.headers(), self.headers());
            c.result().unwrap()
        }
        fn best(&mut self, value: f64) -> Result<Serialized, RangeNumericError> {
            let mut c = BestFiniteNumberCursor::new(value, self.headers())?;
            while let Some(q) = c.request() {
                let r = self.fulfill(q);
                c.reply(r)?;
            }
            assert_eq!(c.headers(), self.headers());
            assert_eq!(c.result(), Some(self.output.len()));
            Ok((String::from_utf16(&self.output).unwrap(), c.selected()))
        }
    }
    #[test]
    fn ingress_literal_values_and_recovery_grammar_remain_separate() {
        for (raw, bits) in [
            ("0", 0),
            ("-0", 1u64 << 63),
            ("1", 0x3ff0000000000000),
            (".1", 0x3fb999999999999a),
            ("9007199254740993", 0x4340000000000000),
            ("9007199254740995", 0x4340000000000002),
        ] {
            assert_eq!(
                Driver::new(raw).ingress(FloatingPointParseMode::Author),
                Ok(bits),
                "{raw}"
            );
        }
        assert_eq!(
            Driver::new(" +1e2tail").ingress(FloatingPointParseMode::Recovery),
            Ok(0x4059000000000000)
        );
        for raw in ["+1", "1.", "1x", " 1", "1e", "NaN", "Infinity"] {
            assert_eq!(
                Driver::new(raw).ingress(FloatingPointParseMode::Author),
                Err(RangeNumericError::InvalidSyntax)
            );
        }
    }
    #[test]
    fn ingress_huge_exponents_classify_without_expanding_scratch() {
        for (raw, result) in [
            (
                format!("1e{}", "9".repeat(2000)),
                Err(RangeNumericError::Overflow),
            ),
            (format!("-1e-{}", "9".repeat(2000)), Ok(1u64 << 63)),
            (format!("0e{}", "9".repeat(2000)), Ok(0)),
        ] {
            let mut d = Driver::new(&raw);
            assert_eq!(d.ingress(FloatingPointParseMode::Author), result);
            assert!(d.scratch.iter().all(Vec::is_empty));
            assert_eq!(d.requests[0], 2001);
        }
    }
    #[test]
    fn ingress_adjacent_normal_midpoints_choose_even() {
        for (raw, bits) in [
            (
                "1.00000000000000011102230246251565404236316680908203125",
                0x3ff0000000000000,
            ),
            (
                "1.00000000000000033306690738754696212708950042724609375",
                0x3ff0000000000002,
            ),
            (
                "1.00000000000000011102230246251565404236316680908203126",
                0x3ff0000000000001,
            ),
        ] {
            assert_eq!(
                Driver::new(raw).ingress(FloatingPointParseMode::Author),
                Ok(bits)
            );
        }
    }
    #[test]
    fn ingress_underflow_and_finite_overflow_are_conversion_outcomes() {
        for (raw, result) in [
            ("2e-324", Ok(0)),
            ("3e-324", Ok(1)),
            ("5e-324", Ok(1)),
            ("1.7976931348623157e308", Ok(0x7fefffffffffffff)),
            ("1.7976931348623159e308", Err(RangeNumericError::Overflow)),
        ] {
            let mut d = Driver::new(raw);
            assert_eq!(d.ingress(FloatingPointParseMode::Author), result, "{raw}");
            assert!(d.scratch.iter().all(|s| s.len() <= d.source.len() + 1386));
        }
    }
    #[test]
    fn serializer_literal_notation_boundaries_and_signed_zero() {
        for (value, text) in [
            (0., "0"),
            (-0., "0"),
            (1., "1"),
            (-0.1, "-0.1"),
            (1e-6, "0.000001"),
            (1e-7, "1e-7"),
            (1e20, "100000000000000000000"),
            (1e21, "1e+21"),
            (1e23, "1e+23"),
        ] {
            assert_eq!(Driver::new("").best(value).unwrap().0, text);
        }
    }
    #[test]
    fn serializer_ieee_boundary_literals() {
        for (bits, text) in [
            (1, "5e-324"),
            (0x0010000000000000, "2.2250738585072014e-308"),
            (0x7fefffffffffffff, "1.7976931348623157e+308"),
            (0x3ff0000000000001, "1.0000000000000002"),
            (0x4340000000000000, "9007199254740992"),
        ] {
            let mut d = Driver::new("");
            let (output, _) = d.best(f64::from_bits(bits)).unwrap();
            assert_eq!(output, text);
            assert!(output.len() <= 25);
            assert!(d.scratch.iter().all(|s| s.len() <= 1386));
        }
    }
    #[test]
    fn serializer_shortest_candidate_uses_nearest_exact_real_not_a_roundtrip_alternative() {
        let (mut d, value) = (Driver::new(""), f64::from_bits(0x3fb999999999999a));
        let (text, selected) = d.best(value).unwrap();
        assert_eq!(text, "0.1");
        assert_eq!(selected, Some((1, 1, 0))); // Actual value lies strictly above 1/10, which is the unique closest one-digit roundtrip.
        let (text, selected) = Driver::new("")
            .best(f64::from_bits(0x43abc16d674ec801))
            .unwrap();
        assert_eq!(text, "1000000000000000100");
        assert_eq!(selected, Some((10000000000000001, 17, 19)));
    }
    #[test]
    fn serializer_rejects_nonfinite_before_any_action() {
        for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            assert!(matches!(
                BestFiniteNumberCursor::new(value, RangeNumericHeaders::default()),
                Err(RangeNumericError::Overflow)
            ));
        }
    }
    #[test]
    fn dyadic_builder_reuses_retained_capacity_and_exposes_every_write_pop() {
        let mut d = Driver::new("");
        d.scratch[RangeScratchRole::Trial.index()] = vec![9, 9, 9];
        let mut c = RangeDyadicCursor::new(d.headers(), RangeScratchRole::Trial, 3, -3).unwrap();
        while let Some(q) = c.request() {
            let r = d.fulfill(q);
            c.reply(r).unwrap();
        }
        assert_eq!(d.scratch[7], [5, 7, 3]);
        assert_eq!(c.result(), Some(-3));
        assert_eq!(d.requests[5], 3);
        assert!(d.requests[4] > 0);
        assert_eq!(c.headers(), d.headers());
    }
    #[test]
    fn power_ten_cut_preserves_exact_fractional_evidence() {
        for (scale, cut, floor, exact) in [
            (0, 0, 12345, true),
            (-2, 0, 123, false),
            (-5, 0, 0, false),
            (1, 0, 123450, true),
        ] {
            let mut d = Driver::new("");
            d.scratch[0] = vec![5, 4, 3, 2, 1];
            let mut c =
                RangePowerTenCutCursor::new(d.headers(), RangeScratchRole::Coefficient, scale, cut)
                    .unwrap();
            while let Some(q) = c.request() {
                let r = d.fulfill(q);
                c.reply(r).unwrap();
            }
            assert_eq!(c.result(), Some((floor, exact)));
        }
    }
    #[test]
    fn malformed_reply_is_transactional_and_does_not_advance_cached_request() {
        let mut d = Driver::new("1");
        let mut c = FiniteDecimalIngressCursor::new(
            d.parts(FloatingPointParseMode::Author).unwrap(),
            1,
            d.headers(),
        )
        .unwrap();
        assert_eq!(
            c.reply(RangeNumericReply::Unit(49)),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(c.request(), Some(RangeNumericRequest::ScalarStep));
        let r = d.fulfill(c.request().unwrap());
        c.reply(r).unwrap();
        let q = c.request();
        assert_eq!(
            c.reply(RangeNumericReply::Ack),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(c.request(), q);
        assert_eq!(c.headers(), d.headers());
        let mut io = Io::new(d.headers(), 0, 1386);
        io.queue(RangeNumericRequest::ReadCells {
            cells: [None, None, None],
        });
        assert_eq!(
            io.accept(RangeNumericReply::Cells([Some(1), None, None])),
            Err(RangeNumericError::Integrity)
        );
    }
    #[test]
    fn one_short_headers_and_out_of_range_cells_are_typed_integrity_failures() {
        let mut io = Io::new(RangeNumericHeaders::default(), 0, 1386);
        io.queue(RangeNumericRequest::Reserve {
            role: RangeScratchRole::Trial,
            required_len: 2,
        });
        assert_eq!(
            io.accept(RangeNumericReply::Reserved {
                actual_capacity: 1,
                actual_len: 0
            }),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(io.headers, RangeNumericHeaders::default());
        io.queue(RangeNumericRequest::AppendCell {
            role: RangeScratchRole::Trial,
            value: 1,
        });
        assert_eq!(
            io.accept(RangeNumericReply::Ack),
            Err(RangeNumericError::Integrity)
        );
        io.queue(RangeNumericRequest::ReadCells {
            cells: [
                Some(RangeCell {
                    role: RangeScratchRole::Trial,
                    index: 0,
                }),
                None,
                None,
            ],
        });
        assert_eq!(
            io.accept(RangeNumericReply::Cells([Some(10), None, None])),
            Err(RangeNumericError::Integrity)
        );
        io.queue(RangeNumericRequest::ReserveOutput { required_len: 25 });
        assert_eq!(
            io.accept(RangeNumericReply::OutputReserved {
                actual_capacity: 24,
                actual_len: 0
            }),
            Err(RangeNumericError::Integrity)
        );
    }
    #[test]
    fn every_closed_scratch_role_and_three_cell_shape_uses_the_same_owner() {
        let mut d = Driver::new("");
        for role in RangeScratchRole::ALL {
            let mut c = RangeDyadicCursor::new(d.headers(), role, 7, 0).unwrap();
            while let Some(q) = c.request() {
                let r = d.fulfill(q);
                c.reply(r).unwrap();
            }
            assert_eq!(d.scratch[role.index()], [7]);
        }
        let mut io = Io::new(d.headers(), 0, 1386);
        let q = RangeNumericRequest::ReadCells {
            cells: [
                Some(RangeCell {
                    role: RangeScratchRole::Right,
                    index: 0,
                }),
                Some(RangeCell {
                    role: RangeScratchRole::Quotient,
                    index: 0,
                }),
                Some(RangeCell {
                    role: RangeScratchRole::Remainder,
                    index: 0,
                }),
            ],
        };
        io.queue(q);
        io.accept(d.fulfill(q)).unwrap();
    }
    #[test]
    fn exact_ieee_extreme_midpoint_literals_use_even_and_virtual_overflow() {
        // Literal exact 2^-1075 = 5^1075 / 10^1075. The next decimal is
        // greater by 10^-1075. Constants were derived by integer powers,
        // independently of the decimal-cell multiplication implementation.
        for (raw, result) in [
            (
                "0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328125",
                Ok(0),
            ),
            (
                "0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024703282292062327208828439643411068618252990130716238221279284125033775363510437593264991818081799618989828234772285886546332835517796989819938739800539093906315035659515570226392290858392449105184435931802849936536152500319370457678249219365623669863658480757001585769269903706311928279558551332927834338409351978015531246597263579574622766465272827220056374006485499977096599470454020828166226237857393450736339007967761930577506740176324673600968951340535537458516661134223766678604162159680461914467291840300530057530849048765391711386591646239524912623653881879636239373280423891018672348497668235089863388587925628302755995657524455507255189313690836254779186948667994968324049705821028513185451396213837722826145437693412532098591327667236328126",
                Ok(1),
            ),
            (
                "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792",
                Err(RangeNumericError::Overflow),
            ),
            (
                "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791",
                Ok(0x7fefffffffffffff),
            ),
        ] {
            assert_eq!(
                Driver::new(raw).ingress(FloatingPointParseMode::Author),
                result
            );
        }
    }
    #[test]
    fn long_coefficients_reserve_actual_extents_and_preserve_exact_scaling() {
        let raw = format!("1{}e-4096", "0".repeat(4096));
        let mut d = Driver::new(&raw);
        assert_eq!(
            d.ingress(FloatingPointParseMode::Author),
            Ok(0x3ff0000000000000)
        );
        assert_eq!(d.scratch[0].len(), 4097);
        assert!(d.scratch[1..].iter().all(|v| v.len() <= 1386));
        assert_eq!(d.scratch[0].first(), Some(&0));
        assert_eq!(d.scratch[0].last(), Some(&1));
    }
    #[test]
    fn exact_midpoint_candidate_policy_is_even_independent_of_side() {
        use std::cmp::Ordering::*;
        assert!(best::prefer_candidate(Equal, true, 2, 1));
        assert!(!best::prefer_candidate(Equal, false, 1, 2));
        assert!(best::prefer_candidate(Equal, false, 4, 5));
        assert!(!best::prefer_candidate(Equal, true, 5, 4));
        assert!(best::prefer_candidate(Less, false, 1, 2));
        assert!(best::prefer_candidate(Greater, true, 5, 4));
    }
    #[test]
    fn scaled_comparison_reads_virtual_zeroes_without_materializing_them() {
        let mut d = Driver::new("");
        d.scratch[3] = vec![1];
        d.scratch[4] = vec![9, 9];
        let mut c = RangeScaledComparisonCursor::new(
            d.headers(),
            RangeScratchRole::Left,
            2,
            false,
            RangeScratchRole::Right,
            0,
            false,
        );
        while let Some(q) = c.request() {
            let r = d.fulfill(q);
            c.reply(r).unwrap();
        }
        assert_eq!(c.result(), Some(std::cmp::Ordering::Greater));
        assert_eq!(d.scratch[3], [1]);
        assert_eq!(d.scratch[4], [9, 9]);
    }
    #[test]
    fn dyadic_normalization_preserves_literal_values_one_scalar_factor_at_a_time() {
        for (mantissa, exponent, cells, scale, factors) in [
            (400, -3, vec![0, 5], 0, 4),
            (48, -5, vec![5, 1], -1, 4),
            (12, 1, vec![4, 2], 0, 2),
            (0, -1075, vec![0], 0, 0),
        ] {
            let mut d = Driver::new("");
            let mut c =
                RangeDyadicCursor::new(d.headers(), RangeScratchRole::Trial, mantissa, exponent)
                    .unwrap();
            let mut observed = 0;
            while let Some(q) = c.request() {
                let before = (c.phase, c.seed, c.exponent);
                let r = d.fulfill(q);
                c.reply(r).unwrap();
                if matches!(before.0, BuildPhase::Normalize)
                    && before.1 != 0
                    && before.1.is_multiple_of(2)
                {
                    assert_eq!(q, RangeNumericRequest::ScalarStep);
                    assert_eq!(c.seed, before.1 / 2);
                    assert_eq!(c.exponent, before.2 + 1);
                    observed += 1;
                }
            }
            assert_eq!(observed, factors);
            assert_eq!(d.scratch[7], cells);
            assert_eq!(c.result(), Some(scale));
            assert_eq!(c.headers(), d.headers());
        }
    }
    #[test]
    fn normalized_ieee_mantissa_boundary_requires_one_reply_per_factor() {
        // Minimum normal = 2^52 * 2^-1074 = 1 * 2^-1022.
        // Only admitted scalar replies may perform those 52 reductions.
        let mut d = Driver::new("");
        let mut c =
            RangeDyadicCursor::new(d.headers(), RangeScratchRole::Trial, 1 << 52, -1074).unwrap();
        c.reply(d.fulfill(c.request().unwrap())).unwrap();
        for factor in 1..=52 {
            assert!(matches!(c.phase, BuildPhase::Normalize));
            assert_eq!(c.request(), Some(RangeNumericRequest::ScalarStep));
            let before = c;
            assert_eq!(c.reply(RangeNumericReply::Ack), Ok(()));
            assert_eq!(c.seed, 1 << (52 - factor));
            assert_eq!(c.exponent, -1074 + factor);
            assert_eq!(c.io.headers, before.io.headers);
        }
        assert_eq!(c.seed, 1);
        assert_eq!(c.exponent, -1022);
        c.reply(RangeNumericReply::Ack).unwrap();
        assert!(matches!(c.phase, BuildPhase::Seed));
        assert_eq!(c.remaining, 1022);
        assert_eq!(c.scale, -1022);
        assert_eq!(d.requests[1], 0);
        assert_eq!(d.requests[4], 0);
        assert!(d.scratch.iter().all(Vec::is_empty));
    }
    #[test]
    fn coarse_orders_preserve_open_decimal_and_binary_boundaries() {
        use std::cmp::Ordering::{Greater, Less};
        for (d, m, e, expected) in [
            (1, 1, 4, Some(Less)),
            (1, 1, -1, Some(Greater)),
            (1, 1, 0, None),
            (1, 1, 3, None),
            (0, 1, 0, Some(Less)),
            (0, 1, -5, Some(Greater)),
            (0, 1, -3, None),
            (2, 1, 1023, Some(Less)),
            (-1, 1, -1074, Some(Greater)),
            (-323, 1, -1074, None),
            (309, (1 << 53) - 1, 971, None),
            (1, 0, 0, None),
        ] {
            assert_eq!(decimal_dyadic_ordering(d, m, e), Ok(expected));
        }
        assert_eq!(
            decimal_dyadic_ordering(i128::MAX, 1, 0),
            Err(RangeNumericError::Representation)
        );
        assert_eq!(
            decimal_dyadic_ordering(i128::MIN, 1, 0),
            Err(RangeNumericError::Representation)
        );
    }
    #[test]
    fn coarse_trial_rejection_preserves_signed_decade_and_binary_literals() {
        for (raw, expected) in [
            ("-50", 0xc049_0000_0000_0000),
            ("-.1", 0xbfb9_9999_9999_999a),
            ("10", 0x4024_0000_0000_0000),
            ("100", 0x4059_0000_0000_0000),
            (".125", 0x3fc0_0000_0000_0000),
            (".0625", 0x3fb0_0000_0000_0000),
            ("1e-323", 2),
        ] {
            assert_eq!(
                Driver::new(raw).ingress(FloatingPointParseMode::Author),
                Ok(expected)
            );
        }
    }
    #[test]
    fn fixed_mantissa_product_preserves_literal_cells_and_source_owner() {
        for (source, factor, expected) in [
            (vec![5, 2, 1], 8, "1000"),
            (vec![9, 9, 9], u64::MAX, "18428297329635842063385"),
            (vec![1], u64::MAX, "18446744073709551615"),
        ] {
            let mut d = Driver::new("");
            d.scratch[RangeScratchRole::Right.index()] = source.clone();
            d.scratch[RangeScratchRole::Trial.index()] = vec![9, 9];
            let mut c = RangeMantissaProductCursor::new(
                d.headers(),
                RangeScratchRole::Right,
                RangeScratchRole::Trial,
                factor,
            )
            .unwrap();
            while let Some(q) = c.request() {
                if let RangeNumericRequest::ReadCells { cells } = q {
                    assert_eq!(cells[0].unwrap().role, RangeScratchRole::Right);
                }
                let reply = d.fulfill(q);
                c.reply(reply).unwrap();
                assert!(c.carry < u128::from(factor));
            }
            assert_eq!(d.scratch[4], source);
            assert_eq!(d.requests[5], 2);
            let expected: Vec<_> = expected.bytes().rev().map(|b| b - b'0').collect();
            assert_eq!(d.scratch[7], expected);
            assert_eq!(c.headers(), d.headers());
            assert_eq!(c.result(), Some(()));
        }
    }
    #[test]
    fn product_alias_empty_source_zero_factor_and_bad_replies_are_refused() {
        let mut d = Driver::new("");
        d.scratch[4] = vec![1];
        for (source, destination, factor) in [
            (RangeScratchRole::Right, RangeScratchRole::Right, 2),
            (RangeScratchRole::Lower, RangeScratchRole::Trial, 2),
            (RangeScratchRole::Right, RangeScratchRole::Trial, 0),
        ] {
            assert!(matches!(
                RangeMantissaProductCursor::new(d.headers(), source, destination, factor),
                Err(RangeNumericError::Integrity)
            ));
        }
        let mut c = RangeMantissaProductCursor::new(
            d.headers(),
            RangeScratchRole::Right,
            RangeScratchRole::Trial,
            2,
        )
        .unwrap();
        assert_eq!(
            c.reply(RangeNumericReply::Unit(49)),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(c.request(), Some(RangeNumericRequest::ScalarStep));
        c.reply(RangeNumericReply::Ack).unwrap();
        let saved = c;
        assert_eq!(
            c.reply(RangeNumericReply::Cells([Some(1), Some(1), None])),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(c.request(), saved.request());
        assert_eq!(c.headers(), saved.headers());
        c.reply(RangeNumericReply::Cells([Some(1), None, None]))
            .unwrap();
        assert!(matches!(
            c.request(),
            Some(RangeNumericRequest::Reserve {
                required_len: 1,
                ..
            })
        ));
        let saved = c;
        assert_eq!(
            c.reply(RangeNumericReply::Reserved {
                actual_capacity: 0,
                actual_len: 0
            }),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(c.request(), saved.request());
        assert_eq!(c.headers(), saved.headers());
        assert!(d.scratch[7].is_empty());
    }
    #[test]
    fn ingress_cache_uses_raw_exponent_and_rebuilds_for_exact_midpoint() {
        let mut d = Driver::new("0.10000000000000000000");
        let mut c = FiniteDecimalIngressCursor::new(
            d.parts(FloatingPointParseMode::Author).unwrap(),
            d.source.len(),
            d.headers(),
        )
        .unwrap();
        let mut published = Vec::new();
        let mut reused_reads = 0;
        while let Some(q) = c.request() {
            let key = c.cache_key();
            match q {
                RangeNumericRequest::PopCell {
                    role: RangeScratchRole::Right,
                }
                | RangeNumericRequest::AppendCell {
                    role: RangeScratchRole::Right,
                    ..
                }
                | RangeNumericRequest::WriteCell {
                    cell:
                        RangeCell {
                            role: RangeScratchRole::Right,
                            ..
                        },
                    ..
                } => {
                    assert_eq!(key, None);
                }
                RangeNumericRequest::ReadCells { cells }
                    if cells[0].is_some_and(|cell| cell.role == RangeScratchRole::Right)
                        && key == Some(-56) =>
                {
                    reused_reads += 1
                }
                _ => {}
            }
            let before = c.headers();
            if let RangeNumericRequest::Reserve {
                role: RangeScratchRole::Right,
                required_len,
            } = q
            {
                assert_eq!(key, None);
                assert_eq!(
                    c.reply(RangeNumericReply::Reserved {
                        actual_capacity: required_len,
                        actual_len: before.buffers[4].len + 1,
                    }),
                    Err(RangeNumericError::Integrity)
                );
                assert_eq!(c.request(), Some(q));
                assert_eq!(c.headers(), before);
                assert_eq!(c.cache_key(), None);
            }
            if matches!(q, RangeNumericRequest::ReadCells { .. }) && key.is_some() {
                assert_eq!(
                    c.reply(RangeNumericReply::Cells([None; 3])),
                    Err(RangeNumericError::Integrity)
                );
                assert_eq!(c.request(), Some(q));
                assert_eq!(c.headers(), before);
                assert_eq!(c.cache_key(), key);
            }
            let reply = d.fulfill(q);
            c.reply(reply).unwrap();
            if key.is_none() && c.cache_key().is_some() {
                published.push(c.cache_key().unwrap());
                assert!(!d.scratch[4].is_empty());
            }
        }
        assert_eq!(c.result(), Some(Ok(0x3fb9_9999_9999_999a)));
        assert!(published.contains(&-56));
        assert_eq!(c.cache_key(), Some(-57));
        assert!(reused_reads > 40);
        let expected: Vec<_> = b"6938893903907228377647697925567626953125"
            .iter()
            .rev()
            .map(|b| b - b'0')
            .collect();
        assert_eq!(d.scratch[4], expected);
        assert_eq!(c.headers(), d.headers());
    }
    #[test]
    fn fresh_ingress_refuses_occupied_cache_role_without_touching_storage() {
        let mut d = Driver::new("50");
        d.scratch[4] = vec![9];
        let headers = d.headers();
        assert!(matches!(
            FiniteDecimalIngressCursor::new(
                d.parts(FloatingPointParseMode::Author).unwrap(),
                d.source.len(),
                headers
            ),
            Err(RangeNumericError::Integrity)
        ));
        assert_eq!(d.headers(), headers);
        assert_eq!(d.scratch[4], [9]);
        assert_eq!(d.requests, [0; 9]);
    }
    #[test]
    fn product_inconsistent_initialized_header_refuses_before_any_cell_action() {
        let mut d = Driver::new("");
        d.scratch[4] = vec![1];
        let mut headers = d.headers();
        headers.buffers[4].capacity = 0;
        let mut c = RangeMantissaProductCursor::new(
            headers,
            RangeScratchRole::Right,
            RangeScratchRole::Trial,
            2,
        )
        .unwrap();
        assert_eq!(
            c.reply(RangeNumericReply::Ack),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(c.request(), Some(RangeNumericRequest::ScalarStep));
        assert_eq!(c.headers(), headers);
        assert_eq!(c.result(), None);
        assert_eq!(d.scratch[4], [1]);
        assert!(d.scratch[7].is_empty());
        assert_eq!(d.requests, [0; 9]);
    }
    // SOURCE6 fallback and unchanged Best characterization: the FNV digest covers the complete
    // request/reply/acknowledged-header sequence under this exact-reserve adapter.
    fn representation_trace(
        hash: &mut u64,
        q: RangeNumericRequest,
        r: RangeNumericReply,
        h: RangeNumericHeaders,
    ) {
        for byte in format!("{q:?}|{r:?}|{h:?}\n").bytes() {
            *hash = (*hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    #[test]
    fn exclusive_helpers_preserve_fallback_and_best_request_and_header_traces() {
        for (raw, expected) in [
            ("50.000000000000000000", 0x89879303f20c29a6),
            ("0.10000000000000000000", 0x5394a0c48cb010b8),
            (".12500000000000000000", 0x3882e5b504c1e041),
            ("-0", 0xb8763caf96d91bf3),
            ("1e999", 0x12d24b9057be04f5),
        ] {
            let mut d = Driver::new(raw);
            let mut c = FiniteDecimalIngressCursor::new(
                d.parts(FloatingPointParseMode::Author).unwrap(),
                d.source.len(),
                d.headers(),
            )
            .unwrap();
            let mut hash = 0xcbf29ce484222325;
            while let Some(q) = c.request() {
                let r = d.fulfill(q);
                c.reply(r).unwrap();
                representation_trace(&mut hash, q, r, c.headers());
            }
            assert_eq!(hash, expected, "{raw}");
        }
        for (value, expected) in [
            (50.0, 0x2d545b4186c6e521),
            (0.2, 0xa767a4668f1ff5ab),
            (0.0, 0x6b0e404a381c9bf6),
        ] {
            let mut d = Driver::new("");
            let mut c = BestFiniteNumberCursor::new(value, d.headers()).unwrap();
            let mut hash = 0xcbf29ce484222325;
            while let Some(q) = c.request() {
                let r = d.fulfill(q);
                c.reply(r).unwrap();
                representation_trace(&mut hash, q, r, c.headers());
            }
            assert_eq!(hash, expected, "{value}");
        }
    }
    #[test]
    fn ingress_exclusive_helpers_retire_only_after_committed_cache_and_results() {
        let mut d = Driver::new("0.10000000000000000000");
        let mut c = FiniteDecimalIngressCursor::new(
            d.parts(FloatingPointParseMode::Author).unwrap(),
            d.source.len(),
            d.headers(),
        )
        .unwrap();
        let mut seen = [false; 4];
        while let Some(q) = c.request() {
            let kind = c.active_kind();
            seen[usize::from(kind)] = true;
            let snapshot = format!("{c:?}");
            assert_eq!(
                c.reply(RangeNumericReply::OutputReserved {
                    actual_capacity: 0,
                    actual_len: 0
                }),
                Err(RangeNumericError::Integrity)
            );
            assert_eq!(format!("{c:?}"), snapshot);
            let r = d.fulfill(q);
            c.reply(r).unwrap();
            assert_eq!(c.headers(), d.headers());
            if kind == 1 && c.active_kind() == 2 {
                assert!(c.cache_key().is_some());
                assert!(!d.scratch[4].is_empty());
            }
            if kind == 2 && c.active_kind() == 3 {
                assert!(c.cache_key().is_some());
                assert!(!d.scratch[7].is_empty());
            }
        }
        assert_eq!(seen, [true; 4]);
        assert_eq!(c.active_kind(), 0);
        assert_eq!(c.cache_key(), Some(-57));
        assert_eq!(c.result(), Some(Ok(0x3fb9_9999_9999_999a)));
        let snapshot = format!("{c:?}");
        assert_eq!(
            c.reply(RangeNumericReply::Ack),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(format!("{c:?}"), snapshot);
    }
    #[test]
    fn best_exclusive_helpers_preserve_shortest_result_and_reject_in_each_variant() {
        let mut seen = [false; 4];
        for (value, text, selected) in [
            (0.2, "0.2", (2, 1, 0)),
            // Exact binary interval contains both 17-digit candidates ending
            // 66 and 67; 67 is closer. The neighboring 16-digit candidates
            // lie outside it, so this exercises the real comparison helper.
            (
                f64::from_bits(0x3ff3c0ca428c59fb),
                "1.2345678901234567",
                (12345678901234567, 17, 1),
            ),
            (
                f64::from_bits(0x43abc16d674ec801),
                "1000000000000000100",
                (10000000000000001, 17, 19),
            ),
        ] {
            let mut d = Driver::new("");
            let mut c = BestFiniteNumberCursor::new(value, d.headers()).unwrap();
            while let Some(q) = c.request() {
                seen[usize::from(c.active_kind())] = true;
                let snapshot = format!("{c:?}");
                assert_eq!(
                    c.reply(RangeNumericReply::Unit(0)),
                    Err(RangeNumericError::Integrity)
                );
                assert_eq!(format!("{c:?}"), snapshot);
                let r = d.fulfill(q);
                c.reply(r).unwrap();
                assert_eq!(c.headers(), d.headers());
            }
            assert_eq!(c.active_kind(), 0);
            assert_eq!(c.selected(), Some(selected));
            assert_eq!(c.result(), Some(text.len()));
            assert_eq!(String::from_utf16(&d.output).unwrap(), text);
            let snapshot = format!("{c:?}");
            assert_eq!(
                c.reply(RangeNumericReply::Ack),
                Err(RangeNumericError::Integrity)
            );
            assert_eq!(format!("{c:?}"), snapshot);
        }
        assert_eq!(seen, [true; 4]);
    }

    #[test]
    fn small_decimal_nearest_even_carry_sign_and_power_literals() {
        // Exact decimal rationals, not the implementation's arithmetic output.
        for (raw, bits) in [
            ("50", 0x4049_0000_0000_0000),
            (".1", 0x3fb9_9999_9999_999a),
            (".3", 0x3fd3_3333_3333_3333),
            ("-.1", 0xbfb9_9999_9999_999a),
            ("9007199254740993", 0x4340_0000_0000_0000),
            ("9007199254740995", 0x4340_0000_0000_0002),
            ("9007199254740991.5", 0x4340_0000_0000_0000),
            ("1.9999999999999999", 0x4000_0000_0000_0000),
            ("1e-22", 0x3b5e_3920_1017_5ee6),
            ("1e22", 0x4480_f0cf_064d_d592),
            ("34028236692093846e22", 0x47f0_0000_0000_0000),
        ] {
            let mut d = Driver::new(raw);
            let mut c = FiniteDecimalIngressCursor::new(
                d.parts(FloatingPointParseMode::Author).unwrap(),
                d.source.len(),
                d.headers(),
            )
            .unwrap();
            let mut small = false;
            while let Some(q) = c.request() {
                small |= c.active_kind() == 4;
                assert!(c.active_kind() == 0 || c.active_kind() == 4, "{raw}");
                c.reply(d.fulfill(q)).unwrap();
                assert_eq!(c.headers(), d.headers());
            }
            assert!(small, "{raw}");
            assert_eq!(c.result(), Some(Ok(bits)), "{raw}");
            assert_eq!(c.cache_key(), None);
            assert!(d.scratch[RangeScratchRole::Right.index()].is_empty());
            assert!(d.scratch[RangeScratchRole::Trial.index()].is_empty());
        }
        for (raw, bits) in [("0", 0), ("-0", 1u64 << 63)] {
            let mut d = Driver::new(raw);
            assert_eq!(d.ingress(FloatingPointParseMode::Author), Ok(bits));
            assert!(d.scratch.iter().all(Vec::is_empty));
        }
    }
    #[test]
    fn small_decimal_eligibility_uses_actual_cells_scale_and_checked_product() {
        for (raw, small, search, bits) in [
            ("9999999999999999999", true, false, 0x43e1_58e4_6091_3d00),
            ("0.10000000000000000000", false, true, 0x3fb9_9999_9999_999a),
            ("1e-22", true, false, 0x3b5e_3920_1017_5ee6),
            ("1e-23", false, true, 0x3b28_2db3_4012_b251),
            ("1e22", true, false, 0x4480_f0cf_064d_d592),
            ("1e23", false, true, 0x44b5_2d02_c7e1_4af6),
            ("34028236692093847e22", true, true, 0x47f0_0000_0000_0000),
        ] {
            let mut d = Driver::new(raw);
            let mut c = FiniteDecimalIngressCursor::new(
                d.parts(FloatingPointParseMode::Author).unwrap(),
                d.source.len(),
                d.headers(),
            )
            .unwrap();
            let mut seen_small = false;
            let mut seen_search = false;
            while let Some(q) = c.request() {
                seen_small |= c.active_kind() == 4;
                seen_search |= (1..=3).contains(&c.active_kind());
                c.reply(d.fulfill(q)).unwrap();
            }
            assert_eq!(seen_small, small, "{raw}");
            assert_eq!(seen_search, search, "{raw}");
            assert_eq!(c.result(), Some(Ok(bits)), "{raw}");
        }
    }
    #[test]
    fn small_decimal_reads_each_initialized_cell_and_advances_each_power() {
        for (raw, expected_indices, expected_steps) in [
            ("50", vec![1, 0], 8),
            ("1e22", vec![0], 29),
            ("1e-22", vec![0], 29),
        ] {
            let mut d = Driver::new(raw);
            let mut c = FiniteDecimalIngressCursor::new(
                d.parts(FloatingPointParseMode::Author).unwrap(),
                d.source.len(),
                d.headers(),
            )
            .unwrap();
            let mut indices = Vec::new();
            let mut steps = 0;
            let mut original_coefficient = None;
            while let Some(q) = c.request() {
                if c.active_kind() == 4 {
                    original_coefficient.get_or_insert_with(|| d.scratch[0].clone());
                    match q {
                        RangeNumericRequest::ReadCells {
                            cells: [Some(cell), None, None],
                        } => {
                            assert_eq!(cell.role, RangeScratchRole::Coefficient);
                            assert!(cell.index < c.headers().buffers[0].len);
                            indices.push(cell.index);
                        }
                        RangeNumericRequest::ScalarStep => steps += 1,
                        _ => panic!("small ratio must not reserve, write, or inspect source"),
                    }
                }
                c.reply(d.fulfill(q)).unwrap();
            }
            assert_eq!(indices, expected_indices, "{raw}");
            assert_eq!(steps, expected_steps, "{raw}");
            assert_eq!(original_coefficient.unwrap(), d.scratch[0]);
            assert!(d.scratch[1..].iter().all(Vec::is_empty));
            assert_eq!(c.headers(), d.headers());
        }
    }
    #[test]
    fn small_decimal_every_reply_refusal_preserves_the_outer_transaction() {
        let mut d = Driver::new("9007199254740991.5");
        let mut c = FiniteDecimalIngressCursor::new(
            d.parts(FloatingPointParseMode::Author).unwrap(),
            d.source.len(),
            d.headers(),
        )
        .unwrap();
        let mut read = false;
        let mut scalar = false;
        while let Some(q) = c.request() {
            if c.active_kind() == 4 {
                read |= matches!(q, RangeNumericRequest::ReadCells { .. });
                scalar |= q == RangeNumericRequest::ScalarStep;
                let snapshot = format!("{c:?}");
                let headers = c.headers();
                assert_eq!(
                    c.reply(RangeNumericReply::Unit(0)),
                    Err(RangeNumericError::Integrity)
                );
                assert_eq!(format!("{c:?}"), snapshot);
                assert_eq!(c.headers(), headers);
                if let RangeNumericRequest::ReadCells { .. } = q {
                    assert_eq!(
                        c.reply(RangeNumericReply::Cells([Some(10), None, None])),
                        Err(RangeNumericError::Integrity)
                    );
                    assert_eq!(format!("{c:?}"), snapshot);
                }
            }
            c.reply(d.fulfill(q)).unwrap();
        }
        assert!(read && scalar);
        assert_eq!(c.result(), Some(Ok(0x4340_0000_0000_0000)));
        let snapshot = format!("{c:?}");
        assert_eq!(
            c.reply(RangeNumericReply::Ack),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(format!("{c:?}"), snapshot);
    }
    #[test]
    fn small_decimal_zero_high_cell_is_integrity_not_search_fallback() {
        let mut d = Driver::new("50");
        let mut c = FiniteDecimalIngressCursor::new(
            d.parts(FloatingPointParseMode::Author).unwrap(),
            d.source.len(),
            d.headers(),
        )
        .unwrap();
        while c.active_kind() != 4 {
            let q = c.request().unwrap();
            c.reply(d.fulfill(q)).unwrap();
        }
        assert_eq!(
            c.request(),
            Some(RangeNumericRequest::ReadCells {
                cells: [
                    Some(RangeCell {
                        role: RangeScratchRole::Coefficient,
                        index: 1
                    }),
                    None,
                    None
                ],
            })
        );
        let snapshot = format!("{c:?}");
        assert_eq!(
            c.reply(RangeNumericReply::Cells([Some(0), None, None])),
            Err(RangeNumericError::Integrity)
        );
        assert_eq!(format!("{c:?}"), snapshot);
        assert_eq!(c.active_kind(), 4);
        assert_eq!(c.cache_key(), None);
        assert_eq!(d.scratch[0], [0, 5]);
        assert!(d.scratch[4].is_empty());
        assert!(d.scratch[7].is_empty());
    }
}
