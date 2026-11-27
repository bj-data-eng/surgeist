//! Exact HTML877d step grid projection over shared decimal quantities.
//!
//! The adapter supplies the current value, effective attributes, a fixed
//! size workspace and a charge for each reached digit action. The grid
//! keeps the projection exact until the state-specific serialization step.

use crate::internal::microsyntax::numeric::{
    HtmlNonNegativeDecimal, HtmlSignedDecimal, natural_ratio::HtmlNaturalDigits,
};
use std::cmp::Ordering;

/// The invoked stepping method, independent of the sign of its count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputStepDirection {
    /// Apply stepUp and its final check against downward movement.
    Up,
    /// Apply stepDown and its final check against upward movement.
    Down,
}

/// Already qualified effective bounds, base and step for one Input type.
#[derive(Clone, Copy, Debug)]
pub struct HtmlInputStepBounds<'a> {
    /// Effective minimum, when supplied by the Input state.
    pub minimum: Option<HtmlInputStepOperand<'a>>,
    /// Effective maximum, when supplied by the Input state.
    pub maximum: Option<HtmlInputStepOperand<'a>>,
    /// Origin of the grid selected by the shared attribute rules.
    pub base: HtmlInputStepOperand<'a>,
    /// Positive allowed step in the same coordinate unit.
    pub step: HtmlInputStepOperand<'a>,
}

/// Borrowed exact operand, either a native decimal or an admitted coefficient.
#[derive(Clone, Copy, Debug)]
pub struct HtmlInputStepOperand<'a> {
    representation: OperandRepr<'a>,
    negative: bool,
}

#[derive(Clone, Copy, Debug)]
enum OperandRepr<'a> {
    Decimal(&'a HtmlNonNegativeDecimal),
    Coefficient {
        digits: HtmlNaturalDigits<'a>,
        scale: usize,
    },
    Cells {
        digits: &'a [u8],
        scale: usize,
    },
}

impl<'a> HtmlInputStepOperand<'a> {
    /// Borrow an exact signed decimal, retaining its actual sign.
    pub fn decimal(value: &'a HtmlSignedDecimal) -> Self {
        Self {
            representation: OperandRepr::Decimal(value.magnitude()),
            negative: value.is_negative(),
        }
    }

    /// Borrow a nonnegative decimal.
    pub fn magnitude(value: &'a HtmlNonNegativeDecimal) -> Self {
        Self {
            representation: OperandRepr::Decimal(value),
            negative: false,
        }
    }

    /// Borrow already checked coefficient digits and their fractional extent.
    /// The canonical zero coefficient has positive sign.
    pub fn coefficient(digits: HtmlNaturalDigits<'a>, scale: usize, negative: bool) -> Self {
        Self {
            representation: OperandRepr::Coefficient { digits, scale },
            negative: negative && !(digits.len() == 1 && digits.digit(0) == 0),
        }
    }

    /// Compares exact coordinates without allocating a common coefficient.
    /// The caller admits each reached digit or scalar comparison.
    pub fn compare<E>(
        self,
        other: Self,
        mut tick: impl FnMut() -> Result<(), E>,
    ) -> Result<Ordering, HtmlInputStepError<E>> {
        let (left, left_scale) = self.extent();
        let (right, right_scale) = other.extent();
        let scale = left_scale.max(right_scale);
        let width = left
            .max(right)
            .checked_add(scale)
            .ok_or(HtmlInputStepError::Workspace)?;
        let layout = Layout { width, scale };
        compare(
            self.quantity(layout),
            other.quantity(layout),
            width,
            &mut tick,
        )
    }

    /// Scratch cells sufficient for multiplication by any `u32` scale factor.
    pub fn multiplication_cells(self) -> Option<usize> {
        let (whole, scale) = self.extent();
        whole.checked_add(scale)?.checked_add(10)
    }

    /// Scales the operand exactly in caller-owned scratch.
    pub fn multiply<'work, E>(
        self,
        factor: u32,
        scratch: &'work mut [u8],
        mut tick: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlInputStepCoordinate<'work>, HtmlInputStepError<E>> {
        let width = self
            .multiplication_cells()
            .ok_or(HtmlInputStepError::Workspace)?;
        let scale = self.extent().1;
        let output = scratch
            .get_mut(..width)
            .ok_or(HtmlInputStepError::Workspace)?;
        write_product(
            self.quantity(Layout { width, scale }).digits,
            factor,
            output,
            &mut tick,
        )?;
        let length = digit_len(Digits::Cells(output), width, &mut tick)?;
        Ok(HtmlInputStepCoordinate {
            cells: output
                .get(..length.max(1))
                .ok_or(HtmlInputStepError::Workspace)?,
            scale,
            negative: self.negative && length != 0,
        })
    }

    fn extent(self) -> (usize, usize) {
        match self.representation {
            OperandRepr::Decimal(value) => (
                value.integer_part().decimal_digits().len(),
                value.fractional_digits().map_or(0, str::len),
            ),
            OperandRepr::Coefficient { digits, scale } => {
                (digits.len().saturating_sub(scale).max(1), scale)
            }
            OperandRepr::Cells { digits, scale } => {
                (digits.len().saturating_sub(scale).max(1), scale)
            }
        }
    }

    fn zero(self) -> bool {
        match self.representation {
            OperandRepr::Decimal(value) => value.is_zero(),
            OperandRepr::Coefficient { digits, .. } => digits.len() == 1 && digits.digit(0) == 0,
            OperandRepr::Cells { digits, .. } => digits == [0],
        }
    }

    fn quantity(self, layout: Layout) -> Quantity<'a> {
        let digits = match self.representation {
            OperandRepr::Decimal(value) => decimal_digits(value, layout.scale),
            OperandRepr::Coefficient { digits, scale } => Digits::Shifted {
                digits,
                shift: layout.scale - scale,
            },
            OperandRepr::Cells { digits, scale } => Digits::ShiftedCells {
                digits,
                shift: layout.scale - scale,
            },
        };
        Quantity {
            digits,
            negative: self.negative,
        }
    }
}

/// Failure to construct or execute an exact bounded step grid calculation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlInputStepError<E> {
    /// The supplied allowed step was zero or negative.
    NonPositiveStep,
    /// The layout was unrepresentable or the supplied workspace was too short.
    Workspace,
    /// The caller refused reached work.
    Work(E),
}

impl<E: std::fmt::Display> std::fmt::Display for HtmlInputStepError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonPositiveStep => f.write_str("an allowed Input step must be positive"),
            Self::Workspace => {
                f.write_str("the Input step workspace cannot represent the required digits")
            }
            Self::Work(error) => write!(f, "Input step work was refused ({error})"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for HtmlInputStepError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Work(error) => Some(error),
            _ => None,
        }
    }
}

/// A result borrowed from the admitted step workspace. Digits are exact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputStepCoordinate<'a> {
    cells: &'a [u8],
    scale: usize,
    negative: bool,
}

impl<'a> HtmlInputStepCoordinate<'a> {
    /// Uses a previously calculated exact coordinate as an operand.
    pub fn operand(self) -> HtmlInputStepOperand<'a> {
        HtmlInputStepOperand {
            representation: OperandRepr::Cells {
                digits: self.cells,
                scale: self.scale,
            },
            negative: self.negative,
        }
    }
    /// The nonnegative coefficient, least significant digit first.
    pub const fn coefficient(self) -> &'a [u8] {
        self.cells
    }

    /// The number of decimal places in the supplied coefficient.
    pub const fn scale(self) -> usize {
        self.scale
    }

    /// Whether the coordinate is negative. Zero has positive sign.
    pub const fn is_negative(self) -> bool {
        self.negative
    }
}

#[derive(Clone, Copy)]
enum Digits<'a> {
    Decimal {
        integer: &'a [u8],
        fraction: &'a [u8],
        scale: usize,
    },
    Shifted {
        digits: HtmlNaturalDigits<'a>,
        shift: usize,
    },
    ShiftedCells {
        digits: &'a [u8],
        shift: usize,
    },
    Cells(&'a [u8]),
}

impl Digits<'_> {
    fn digit(self, index: usize) -> u8 {
        match self {
            Self::Decimal {
                integer,
                fraction,
                scale,
            } => if index < scale {
                fraction.get(scale - index - 1)
            } else {
                index
                    .checked_sub(scale)
                    .and_then(|place| integer.len().checked_sub(place + 1))
                    .and_then(|place| integer.get(place))
            }
            .map_or(0, |digit| *digit - b'0'),
            Self::Shifted { digits, shift } => index
                .checked_sub(shift)
                .filter(|position| *position < digits.len())
                .map_or(0, |position| digits.digit(position)),
            Self::ShiftedCells { digits, shift } => index
                .checked_sub(shift)
                .and_then(|position| digits.get(position))
                .copied()
                .unwrap_or(0),
            Self::Cells(cells) => cells.get(index).copied().unwrap_or(0),
        }
    }
}

#[derive(Clone, Copy)]
struct Quantity<'a> {
    digits: Digits<'a>,
    negative: bool,
}

impl Quantity<'_> {
    fn negated(self) -> Self {
        Self {
            negative: !self.negative,
            ..self
        }
    }
}

fn decimal_digits(value: &HtmlNonNegativeDecimal, scale: usize) -> Digits<'_> {
    Digits::Decimal {
        integer: value.integer_part().decimal_digits().as_bytes(),
        fraction: value.fractional_digits().unwrap_or("").as_bytes(),
        scale,
    }
}

#[derive(Clone, Copy)]
struct Layout {
    width: usize,
    scale: usize,
}

impl<'input> HtmlInputStepBounds<'input> {
    fn layout(self, current: Option<HtmlInputStepOperand<'_>>) -> Option<Layout> {
        let mut integer = 1_usize;
        let mut scale = 0;
        for value in [
            self.minimum,
            self.maximum,
            current,
            Some(self.base),
            Some(self.step),
        ]
        .into_iter()
        .flatten()
        {
            let (whole, fraction) = value.extent();
            integer = integer.max(whole);
            scale = scale.max(fraction);
        }
        // An i32 magnitude adds at most ten decimal places. One further
        // place covers carry and the temporary remainder times ten.
        Some(Layout {
            width: integer.checked_add(scale)?.checked_add(11)?,
            scale,
        })
    }

    /// Size of the scratch for initial bound qualification, in byte cells.
    pub fn qualification_cells(self) -> Option<usize> {
        if self.minimum.is_none() || self.maximum.is_none() {
            Some(0)
        } else {
            self.layout(None)?.width.checked_mul(4)
        }
    }

    /// Qualify the interval before reading the actual current value. A
    /// reversed interval or interval with no grid point returns `None`.
    pub fn qualify<E>(
        self,
        scratch: &mut [u8],
        mut tick: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<HtmlInputStepGrid<'input>>, HtmlInputStepError<E>> {
        if self.step.negative || self.step.zero() {
            return Err(HtmlInputStepError::NonPositiveStep);
        }
        if let (Some(minimum), Some(maximum)) = (self.minimum, self.maximum) {
            let layout = self.layout(None).ok_or(HtmlInputStepError::Workspace)?;
            let minimum = minimum.quantity(layout);
            let maximum = maximum.quantity(layout);
            if compare(minimum, maximum, layout.width, &mut tick)? == Ordering::Greater {
                return Ok(None);
            }
            let mut grid = self.projection(layout, scratch)?;
            grid.advance(minimum, HtmlInputStepDirection::Up, 0, &mut tick)?;
            if compare(grid.result(), maximum, layout.width, &mut tick)? == Ordering::Greater {
                return Ok(None);
            }
        }
        Ok(Some(HtmlInputStepGrid { bounds: self }))
    }

    fn projection<'scratch, E>(
        self,
        layout: Layout,
        scratch: &'scratch mut [u8],
    ) -> Result<Projection<'input, 'scratch>, HtmlInputStepError<E>> {
        let mut lanes = scratch.chunks_exact_mut(layout.width);
        Ok(Projection {
            base: self.base.quantity(layout),
            step: self.step.quantity(layout).digits,
            gap: lanes.next().ok_or(HtmlInputStepError::Workspace)?,
            remainder: lanes.next().ok_or(HtmlInputStepError::Workspace)?,
            delta: lanes.next().ok_or(HtmlInputStepError::Workspace)?,
            output: lanes.next().ok_or(HtmlInputStepError::Workspace)?,
            negative: false,
        })
    }
}

/// An allowed step with its effective bounds already qualified.
#[derive(Clone, Copy, Debug)]
pub struct HtmlInputStepGrid<'a> {
    bounds: HtmlInputStepBounds<'a>,
}

impl HtmlInputStepGrid<'_> {
    /// Size of the reusable scratch for the current coordinate, in byte cells.
    pub fn scratch_cells(self, current: Option<HtmlInputStepOperand<'_>>) -> Option<usize> {
        self.bounds.layout(current)?.width.checked_mul(4)
    }

    /// Execute the pinned grid calculation. A missing current value supplies
    /// zero. An equal coordinate can still reach canonical serialization.
    /// A returned `None` is the algorithm's actual no-op.
    /// Each work call admits one reached decimal position or scalar operation.
    pub fn project<'work, E>(
        self,
        current: Option<HtmlInputStepOperand<'_>>,
        direction: HtmlInputStepDirection,
        count: i32,
        scratch: &'work mut [u8],
        mut tick: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<HtmlInputStepCoordinate<'work>>, HtmlInputStepError<E>> {
        let layout = self
            .bounds
            .layout(current)
            .ok_or(HtmlInputStepError::Workspace)?;
        let mut grid = self.bounds.projection(layout, scratch)?;
        let minimum = self.bounds.minimum.map(|value| value.quantity(layout));
        let maximum = self.bounds.maximum.map(|value| value.quantity(layout));
        let before = current.map_or(
            Quantity {
                negative: false,
                digits: Digits::Cells(&[]),
            },
            |value| value.quantity(layout),
        );
        grid.advance(before, direction, count, &mut tick)?;
        if let Some(minimum) = minimum
            && compare(grid.result(), minimum, layout.width, &mut tick)? == Ordering::Less
        {
            grid.advance(minimum, HtmlInputStepDirection::Up, 0, &mut tick)?;
        }
        if let Some(maximum) = maximum
            && compare(grid.result(), maximum, layout.width, &mut tick)? == Ordering::Greater
        {
            grid.advance(maximum, HtmlInputStepDirection::Down, 0, &mut tick)?;
        }
        let movement = compare(grid.result(), before, layout.width, &mut tick)?;
        if matches!(
            (direction, movement),
            (HtmlInputStepDirection::Up, Ordering::Less)
                | (HtmlInputStepDirection::Down, Ordering::Greater)
        ) {
            return Ok(None);
        }
        let used = digit_len(Digits::Cells(grid.output), grid.output.len(), &mut tick)?.max(1);
        Ok(Some(HtmlInputStepCoordinate {
            cells: grid
                .output
                .get(..used)
                .ok_or(HtmlInputStepError::Workspace)?,
            scale: layout.scale,
            negative: grid.negative,
        }))
    }
}

struct Projection<'input, 'scratch> {
    base: Quantity<'input>,
    step: Digits<'input>,
    gap: &'scratch mut [u8],
    remainder: &'scratch mut [u8],
    delta: &'scratch mut [u8],
    output: &'scratch mut [u8],
    negative: bool,
}

impl Projection<'_, '_> {
    fn result(&self) -> Quantity<'_> {
        Quantity {
            digits: Digits::Cells(self.output),
            negative: self.negative,
        }
    }

    fn advance<E>(
        &mut self,
        before: Quantity<'_>,
        direction: HtmlInputStepDirection,
        count: i32,
        tick: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlInputStepError<E>> {
        let negative_gap = write_sum(before, self.base.negated(), self.gap, tick)?;
        let remainder = write_remainder(self.gap, self.step, self.remainder, tick)?;
        let down = direction == HtmlInputStepDirection::Down;
        let delta = if remainder.is_empty() {
            write_product(self.step, count.unsigned_abs(), self.delta, tick)?;
            Quantity {
                digits: Digits::Cells(self.delta),
                negative: down ^ (count < 0),
            }
        } else if down == negative_gap {
            // The magnitude remainder measures the nearest grid point toward
            // the base. In the other direction the distance is step minus it.
            write_sum(
                Quantity {
                    digits: self.step,
                    negative: false,
                },
                Quantity {
                    digits: Digits::Cells(remainder),
                    negative: true,
                },
                self.delta,
                tick,
            )?;
            Quantity {
                digits: Digits::Cells(self.delta),
                negative: down,
            }
        } else {
            Quantity {
                digits: Digits::Cells(remainder),
                negative: down,
            }
        };
        self.negative = write_sum(before, delta, self.output, tick)?;
        Ok(())
    }
}

fn compare<E>(
    left: Quantity<'_>,
    right: Quantity<'_>,
    width: usize,
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<Ordering, HtmlInputStepError<E>> {
    tick().map_err(HtmlInputStepError::Work)?;
    if left.negative != right.negative {
        return Ok(right.negative.cmp(&left.negative));
    }
    let order = compare_digits(left.digits, right.digits, width, tick)?;
    Ok(if left.negative {
        order.reverse()
    } else {
        order
    })
}

fn compare_digits<E>(
    left: Digits<'_>,
    right: Digits<'_>,
    width: usize,
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<Ordering, HtmlInputStepError<E>> {
    for index in (0..width).rev() {
        tick().map_err(HtmlInputStepError::Work)?;
        let order = left.digit(index).cmp(&right.digit(index));
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(Ordering::Equal)
}

fn digit_len<E>(
    digits: Digits<'_>,
    width: usize,
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, HtmlInputStepError<E>> {
    for index in (0..width).rev() {
        tick().map_err(HtmlInputStepError::Work)?;
        if digits.digit(index) != 0 {
            return Ok(index + 1);
        }
    }
    Ok(0)
}

fn write_sum<'a, E>(
    mut left: Quantity<'a>,
    mut right: Quantity<'a>,
    output: &mut [u8],
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, HtmlInputStepError<E>> {
    let subtract = left.negative != right.negative;
    if subtract && compare_digits(left.digits, right.digits, output.len(), tick)? == Ordering::Less
    {
        std::mem::swap(&mut left, &mut right);
    }
    let mut carry = 0;
    let mut nonzero = false;
    for (index, cell) in output.iter_mut().enumerate() {
        tick().map_err(HtmlInputStepError::Work)?;
        let right_digit = i16::from(right.digits.digit(index));
        let digit = i16::from(left.digits.digit(index))
            + if subtract { -right_digit } else { right_digit }
            + carry;
        *cell = digit.rem_euclid(10) as u8;
        carry = digit.div_euclid(10);
        nonzero |= *cell != 0;
    }
    if carry != 0 {
        return Err(HtmlInputStepError::Workspace);
    }
    Ok(left.negative && nonzero)
}

fn write_product<E>(
    digits: Digits<'_>,
    multiplier: u32,
    output: &mut [u8],
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), HtmlInputStepError<E>> {
    let mut carry = 0;
    for (index, cell) in output.iter_mut().enumerate() {
        tick().map_err(HtmlInputStepError::Work)?;
        let total = u64::from(digits.digit(index)) * u64::from(multiplier) + carry;
        *cell = (total % 10) as u8;
        carry = total / 10;
    }
    if carry != 0 {
        return Err(HtmlInputStepError::Workspace);
    }
    Ok(())
}

fn write_remainder<'a, E>(
    dividend: &[u8],
    divisor: Digits<'_>,
    scratch: &'a mut [u8],
    tick: &mut impl FnMut() -> Result<(), E>,
) -> Result<&'a [u8], HtmlInputStepError<E>> {
    let divisor_len = digit_len(divisor, scratch.len(), tick)?;
    let remainder = scratch
        .get_mut(..=divisor_len)
        .ok_or(HtmlInputStepError::Workspace)?;
    for cell in &mut *remainder {
        tick().map_err(HtmlInputStepError::Work)?;
        *cell = 0;
    }
    let dividend_len = digit_len(Digits::Cells(dividend), dividend.len(), tick)?;
    let digits = dividend
        .get(..dividend_len)
        .ok_or(HtmlInputStepError::Workspace)?;
    for &digit in digits.iter().rev() {
        let mut carry = digit;
        for cell in &mut *remainder {
            tick().map_err(HtmlInputStepError::Work)?;
            // Shift one decimal place and install the newly reached digit.
            std::mem::swap(cell, &mut carry);
        }
        if carry != 0 {
            return Err(HtmlInputStepError::Workspace);
        }
        // The previous remainder was below divisor, so this digit cannot
        // bring it to ten times divisor. At most nine subtractions suffice.
        for _ in 0..9 {
            if compare_digits(Digits::Cells(remainder), divisor, remainder.len(), tick)?
                == Ordering::Less
            {
                break;
            }
            let mut borrow = 0;
            for (index, cell) in remainder.iter_mut().enumerate() {
                tick().map_err(HtmlInputStepError::Work)?;
                let digit = i16::from(*cell) - i16::from(divisor.digit(index)) - borrow;
                *cell = digit.rem_euclid(10) as u8;
                borrow = i16::from(digit < 0);
            }
        }
    }
    let used = digit_len(Digits::Cells(remainder), remainder.len(), tick)?;
    remainder.get(..used).ok_or(HtmlInputStepError::Workspace)
}
