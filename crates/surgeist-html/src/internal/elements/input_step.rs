//! Exact HTML877d step grid projection over shared decimal quantities.
//!
//! The adapter supplies the current value, effective attributes, a fixed
//! size workspace and a charge for each reached digit action. The grid
//! keeps the projection exact until the state-specific serialization step.

use crate::internal::microsyntax::numeric::{HtmlNonNegativeDecimal, HtmlSignedDecimal};
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
    pub minimum: Option<&'a HtmlSignedDecimal>,
    /// Effective maximum, when supplied by the Input state.
    pub maximum: Option<&'a HtmlSignedDecimal>,
    /// Origin of the grid selected by the shared attribute rules.
    pub base: &'a HtmlSignedDecimal,
    /// Positive allowed step in the same coordinate unit.
    pub step: &'a HtmlNonNegativeDecimal,
}

/// Failure to construct or execute an exact bounded step grid calculation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlInputStepError<E> {
    /// The supplied allowed step was zero.
    ZeroStep,
    /// The layout was unrepresentable or the supplied workspace was too short.
    Workspace,
    /// The caller refused reached work.
    Work(E),
}

impl<E: std::fmt::Display> std::fmt::Display for HtmlInputStepError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroStep => f.write_str("an allowed Input step must be positive"),
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
            Self::Cells(cells) => cells.get(index).copied().unwrap_or(0),
        }
    }
}

#[derive(Clone, Copy)]
struct Quantity<'a> {
    digits: Digits<'a>,
    negative: bool,
}

impl<'a> Quantity<'a> {
    fn decimal(value: &'a HtmlSignedDecimal, scale: usize) -> Self {
        Self {
            digits: decimal_digits(value.magnitude(), scale),
            negative: value.is_negative(),
        }
    }

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
    fn layout(self, current: Option<&HtmlSignedDecimal>) -> Option<Layout> {
        let mut integer = 1_usize;
        let mut scale = 0;
        for value in [self.minimum, self.maximum, current, Some(self.base)]
            .into_iter()
            .flatten()
            .map(HtmlSignedDecimal::magnitude)
            .chain(std::iter::once(self.step))
        {
            integer = integer.max(value.integer_part().decimal_digits().len());
            scale = scale.max(value.fractional_digits().map_or(0, str::len));
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
        if self.step.is_zero() {
            return Err(HtmlInputStepError::ZeroStep);
        }
        if let (Some(minimum), Some(maximum)) = (self.minimum, self.maximum) {
            let layout = self.layout(None).ok_or(HtmlInputStepError::Workspace)?;
            let minimum = Quantity::decimal(minimum, layout.scale);
            let maximum = Quantity::decimal(maximum, layout.scale);
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
            base: Quantity::decimal(self.base, layout.scale),
            step: decimal_digits(self.step, layout.scale),
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
    pub fn scratch_cells(self, current: Option<&HtmlSignedDecimal>) -> Option<usize> {
        self.bounds.layout(current)?.width.checked_mul(4)
    }

    /// Execute the pinned grid calculation. A missing current value supplies
    /// zero. An equal coordinate can still reach canonical serialization.
    /// A returned `None` is the algorithm's actual no-op.
    /// Each work call admits one reached decimal position or scalar operation.
    pub fn project<'work, E>(
        self,
        current: Option<&HtmlSignedDecimal>,
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
        let minimum = self
            .bounds
            .minimum
            .map(|value| Quantity::decimal(value, layout.scale));
        let maximum = self
            .bounds
            .maximum
            .map(|value| Quantity::decimal(value, layout.scale));
        let before = current.map_or(
            Quantity {
                negative: false,
                digits: Digits::Cells(&[]),
            },
            |value| Quantity::decimal(value, layout.scale),
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
