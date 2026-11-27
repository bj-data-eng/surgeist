//! Controlled exact temporal coordinates for HTML Input stepping.

use super::{
    BorrowedDate, BorrowedTemporalPlan, BorrowedTime, BorrowedYear, GREGORIAN_CYCLE,
    TemporalAlternative, TemporalPhase, days_before_year_in_cycle,
    input_coordinate::{
        DAYS_BEFORE_EPOCH, DAYS_IN_WEEK, MILLISECONDS_PER_DAY, MONTHS_BEFORE_EPOCH, TemporalState,
        inverse::{
            DAYS_IN_CYCLE, MONTHS_PER_YEAR, month_day_in_cycle_year, time_at_millisecond,
            year_and_day_in_cycle,
        },
    },
    recognize_time_alternative, temporal_recognition_work_bound,
};
use crate::{
    bounded::numeric::{HtmlNaturalComparison, HtmlNaturalDigits},
    internal::{
        attributes::HtmlInputType,
        elements::input_step::HtmlInputStepCoordinate,
        microsyntax::numeric::{
            HtmlIntegerUnitStepError, HtmlNonNegativeDecimal, HtmlNonNegativeInteger,
            HtmlNumericMaterializationError, HtmlNumericMaterializationFailure,
            HtmlNumericMaterialized, HtmlSignedDecimal,
        },
    },
};
use std::{cmp::Ordering, error::Error, fmt, fmt::Write, mem::size_of, num::NonZeroU32};

/// State-specific constants for operations on temporal Input coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct HtmlInputTemporalStepParameters {
    /// The unit multiplier applied to the authored or default step.
    pub scale: u32,
    /// The unscaled positive step used when the attribute supplies none.
    pub default_step: u32,
    /// The origin used when neither minimum nor default value is recovered.
    pub default_base: i128,
}

/// Queries stepping constants without materializing a temporal value.
pub fn input_temporal_step_parameters(
    state: HtmlInputType,
) -> Option<HtmlInputTemporalStepParameters> {
    let (_, scale, default_step, default_base) = TemporalState::of(state)?.definition();
    Some(HtmlInputTemporalStepParameters {
        scale,
        default_step,
        default_base,
    })
}

/// The caller's work and temporary storage admission for a temporal Input conversion.
pub trait HtmlInputTemporalStepMeter {
    /// The original cause of a refused reached operation.
    type Stop;

    /// Checks cancellation and admits a checked logical work bound.
    fn work(&mut self, units: usize) -> Result<(), Self::Stop>;

    /// Admits the simultaneous envelope before an allocation is attempted.
    fn reserve(&mut self, bytes: usize, utf16_units: usize) -> Result<(), Self::Stop>;

    /// Releases storage within the originally admitted extent.
    fn release(&mut self, bytes: usize, utf16_units: usize);
}

/// A temporal Input operation could not reach its result.
#[derive(Debug, PartialEq, Eq)]
pub enum HtmlInputTemporalStepError<E> {
    /// The state has no temporal coordinate.
    Inapplicable,
    /// The required size or a qualified intermediate could not be represented.
    Representation,
    /// The work or storage admission was refused.
    Admission(E),
    /// The bounded storage materializer stopped.
    Materialization(HtmlNumericMaterializationError),
}

impl<E: fmt::Display> fmt::Display for HtmlInputTemporalStepError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inapplicable => formatter.write_str("Input state has no temporal coordinate"),
            Self::Representation => {
                formatter.write_str("Input temporal coordinate cannot be represented")
            }
            Self::Admission(error) => {
                write!(formatter, "Input temporal conversion refused: {error}")
            }
            Self::Materialization(error) => error.fmt(formatter),
        }
    }
}

impl<E: Error + 'static> Error for HtmlInputTemporalStepError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::Materialization(error) => Some(error),
            _ => None,
        }
    }
}

/// A result and its observed allocation and retained-storage extents.
#[derive(Debug)]
pub struct HtmlInputTemporalStepOutcome<T, E> {
    result: Result<T, HtmlInputTemporalStepError<E>>,
    observed: usize,
    retained: Storage,
}

impl<T, E> HtmlInputTemporalStepOutcome<T, E> {
    /// Returns the result, cumulative observed allocation bytes, and the
    /// still-admitted output's backing as (bytes, UTF-16 units). A failed
    /// conversion retains no storage, including when admission refused.
    pub fn into_parts(
        self,
    ) -> (
        Result<T, HtmlInputTemporalStepError<E>>,
        usize,
        (usize, usize),
    ) {
        (
            self.result,
            self.observed,
            (self.retained.bytes, self.retained.units),
        )
    }
}

type StepResult<T, M> =
    Result<T, HtmlInputTemporalStepError<<M as HtmlInputTemporalStepMeter>::Stop>>;
type Integer = HtmlNonNegativeInteger;

#[derive(Clone, Copy, Debug, Default)]
struct Storage {
    bytes: usize,
    units: usize,
}

struct Owned<T> {
    value: T,
    storage: Storage,
}

impl<T> Owned<T> {
    fn without_storage(value: T) -> Self {
        Self {
            value,
            storage: Storage::default(),
        }
    }

    fn map<U>(self, transform: impl FnOnce(T) -> U) -> Owned<U> {
        Owned {
            value: transform(self.value),
            storage: self.storage,
        }
    }
}

struct Session<'a, M> {
    meter: &'a mut M,
    max_bytes: usize,
    observed: usize,
    live: Storage,
}

/// The conversion's fixed admission register extent, excluding digit and
/// UTF-16 backing and the caller's own meter.
pub const fn input_temporal_step_register_bytes() -> usize {
    size_of::<Session<'_, ()>>()
}

impl<M: HtmlInputTemporalStepMeter> Session<'_, M> {
    fn work(&mut self, units: Option<usize>) -> StepResult<(), M> {
        self.meter
            .work(units.ok_or(HtmlInputTemporalStepError::Representation)?)
            .map_err(HtmlInputTemporalStepError::Admission)
    }

    fn reserve(&mut self, bytes: usize, units: usize) -> StepResult<(), M> {
        let required = self
            .live
            .bytes
            .checked_add(bytes)
            .ok_or(HtmlInputTemporalStepError::Representation)?;
        if required > self.max_bytes {
            return Err(boundary(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: self.max_bytes,
                    required,
                },
            ));
        }
        let live_units = self
            .live
            .units
            .checked_add(units)
            .ok_or(HtmlInputTemporalStepError::Representation)?;
        if bytes != 0 {
            self.meter
                .reserve(bytes, units)
                .map_err(HtmlInputTemporalStepError::Admission)?;
            self.live = Storage {
                bytes: required,
                units: live_units,
            };
        }
        Ok(())
    }

    fn release(&mut self, bytes: usize, units: usize) {
        if bytes != 0 {
            self.meter.release(bytes, units);
            self.live.bytes -= bytes;
            self.live.units -= units;
        }
    }

    fn discard<T>(&mut self, owned: Owned<T>) {
        let storage = owned.storage;
        drop(owned);
        self.release(storage.bytes, storage.units);
    }

    fn observe(&mut self, bytes: usize) -> StepResult<(), M> {
        self.observed = self
            .observed
            .checked_add(bytes)
            .ok_or(HtmlInputTemporalStepError::Representation)?;
        Ok(())
    }

    fn materialize<T>(
        &mut self,
        work: Option<usize>,
        bytes: Option<usize>,
        make: impl FnOnce(
            usize,
        )
            -> Result<HtmlNumericMaterialized<T>, HtmlNumericMaterializationFailure>,
    ) -> StepResult<Owned<T>, M> {
        self.work(work)?;
        let bytes = bytes.ok_or(HtmlInputTemporalStepError::Representation)?;
        self.reserve(bytes, 0)?;
        let (value, actual) = match make(bytes) {
            Ok(materialized) => materialized.into_parts(),
            Err(failure) => {
                self.observe(failure.capacity_bytes())?;
                return Err(HtmlInputTemporalStepError::Materialization(failure.kind()));
            }
        };
        self.observe(actual)?;
        self.release(bytes - actual, 0);
        Ok(Owned {
            value,
            storage: Storage {
                bytes: actual,
                units: 0,
            },
        })
    }

    fn buffer<T>(&mut self, count: usize, utf16: bool) -> StepResult<Owned<Vec<T>>, M> {
        let bytes = count
            .checked_mul(size_of::<T>())
            .filter(|&bytes| bytes <= isize::MAX as usize)
            .ok_or(HtmlInputTemporalStepError::Representation)?;
        self.work(bytes.checked_add(1))?;
        let units = if utf16 { count } else { 0 };
        self.reserve(bytes, units)?;
        let mut buffer = Vec::new();
        let allocation = buffer.try_reserve_exact(count);
        let actual = buffer.capacity() * size_of::<T>();
        self.observe(actual)?;
        allocation.map_err(|_| {
            boundary(HtmlIntegerUnitStepError::AllocationFailed { requested: bytes })
        })?;
        if actual > bytes {
            return Err(boundary(
                HtmlIntegerUnitStepError::ActualCapacityLimitExceeded {
                    limit: bytes,
                    requested: bytes,
                    capacity_bytes: actual,
                },
            ));
        }
        Ok(Owned {
            value: buffer,
            storage: Storage {
                bytes: actual,
                units,
            },
        })
    }
}

fn boundary<E>(error: HtmlIntegerUnitStepError) -> HtmlInputTemporalStepError<E> {
    HtmlInputTemporalStepError::Materialization(HtmlNumericMaterializationError::Boundary(error))
}

fn run<M: HtmlInputTemporalStepMeter, T>(
    meter: &mut M,
    max_bytes: usize,
    execute: impl FnOnce(&mut Session<'_, M>) -> StepResult<Owned<T>, M>,
) -> HtmlInputTemporalStepOutcome<T, M::Stop> {
    let mut session = Session {
        meter,
        max_bytes,
        observed: 0,
        live: Storage::default(),
    };
    let (result, retained) = match execute(&mut session) {
        Ok(owned) => (Ok(owned.value), owned.storage),
        Err(error) => (Err(error), Storage::default()),
    };
    session.release(
        session.live.bytes - retained.bytes,
        session.live.units - retained.units,
    );
    HtmlInputTemporalStepOutcome {
        result,
        observed: session.observed,
        retained,
    }
}

fn integer<M: HtmlInputTemporalStepMeter>(
    raw: &[u16],
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    meter.materialize(
        Integer::from_borrowed_digits_work_bound(raw.len()),
        Some(raw.len()),
        |capacity| Integer::try_from_borrowed_digits(raw, raw.len(), capacity),
    )
}

fn small<M: HtmlInputTemporalStepMeter>(
    value: u64,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    meter.work(Some(21))?;
    let mut digits = [0_u16; 20];
    let mut value = value;
    let mut index = digits.len();
    loop {
        index -= 1;
        digits[index] = u16::from(b'0') + (value % 10) as u16;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    integer(&digits[index..], meter)
}

fn width(value: &Integer) -> usize {
    value.decimal_digits().len()
}

fn add<M: HtmlInputTemporalStepMeter>(
    value: Owned<Integer>,
    amount: u32,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    if amount == 0 {
        return Ok(value);
    }
    let other = small(u64::from(amount), meter)?;
    let sum = meter.materialize(
        value.value.added_to_work_bound(&other.value),
        width(&value.value).max(width(&other.value)).checked_add(1),
        |bytes| value.value.try_added_to(&other.value, bytes, bytes),
    )?;
    meter.discard(value);
    meter.discard(other);
    Ok(sum)
}

fn multiply<M: HtmlInputTemporalStepMeter>(
    value: Owned<Integer>,
    factor: u32,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    let product = meter.materialize(
        value.value.multiplied_by_work_bound(),
        width(&value.value).checked_add(factor.checked_ilog10().unwrap_or(0) as usize + 1),
        |bytes| value.value.try_multiplied_by(factor, bytes, bytes),
    )?;
    meter.discard(value);
    Ok(product)
}

fn divide<M: HtmlInputTemporalStepMeter>(
    value: Owned<Integer>,
    divisor: NonZeroU32,
    meter: &mut Session<'_, M>,
) -> StepResult<(Owned<Integer>, u32), M> {
    let result = meter.materialize(
        value.value.quotient_remainder_work_bound(),
        Some(
            width(&value.value)
                .saturating_sub(divisor.get().ilog10() as usize)
                .max(1),
        ),
        |bytes| value.value.try_quotient_remainder(divisor, bytes, bytes),
    )?;
    meter.discard(value);
    let (quotient, remainder) = result.value;
    Ok((
        Owned {
            value: quotient,
            storage: result.storage,
        },
        remainder,
    ))
}

fn digit_units<M: HtmlInputTemporalStepMeter>(
    cells: &[u8],
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Vec<u16>>, M> {
    let mut buffer = meter.buffer(cells.len().max(1), true)?;
    meter.work(cells.len().checked_add(1))?;
    if cells.is_empty() {
        buffer.value.push(u16::from(b'0'));
    } else {
        buffer
            .value
            .extend(cells.iter().rev().map(|&digit| u16::from(b'0' + digit)));
    }
    Ok(buffer)
}

fn difference<M: HtmlInputTemporalStepMeter>(
    value: Owned<Integer>,
    amount: u32,
    meter: &mut Session<'_, M>,
) -> StepResult<(Owned<Integer>, bool), M> {
    let other = small(u64::from(amount), meter)?;
    let mut comparison = HtmlNaturalComparison::new(
        HtmlNaturalDigits::integer(&value.value),
        HtmlNaturalDigits::integer(&other.value),
    );
    let ordering = loop {
        meter.work(Some(1))?;
        if let Some(ordering) = comparison.step() {
            break ordering;
        }
    };
    let mut scratch = meter.buffer(comparison.difference_capacity(), false)?;
    {
        let mut diff =
            HtmlNaturalComparison::take_difference(&mut Some(comparison), &mut scratch.value)
                .map_err(|_| HtmlInputTemporalStepError::Representation)?;
        loop {
            meter.work(Some(1))?;
            if diff.step() {
                break;
            }
        }
    }
    meter.discard(value);
    meter.discard(other);
    let digits = digit_units(&scratch.value, meter)?;
    meter.discard(scratch);
    let result = integer(&digits.value, meter)?;
    meter.discard(digits);
    Ok((result, ordering == Ordering::Less))
}

fn preceding_year<M: HtmlInputTemporalStepMeter>(
    plan: BorrowedYear<'_>,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    let digits = plan.digits();
    meter.work(digits.len().checked_add(1))?;
    let first = digits
        .iter()
        .position(|&unit| unit != u16::from(b'0'))
        .ok_or(HtmlInputTemporalStepError::Representation)?;
    let year = integer(&digits[first..], meter)?;
    let preceding = meter.materialize(
        year.value.decremented_work_bound(),
        Some(width(&year.value)),
        |bytes| year.value.try_decremented(bytes, bytes),
    )?;
    meter.discard(year);
    let value = preceding
        .value
        .ok_or(HtmlInputTemporalStepError::Representation)?;
    Ok(Owned {
        value,
        storage: preceding.storage,
    })
}

fn day_ordinal<M: HtmlInputTemporalStepMeter>(
    year: BorrowedYear<'_>,
    within_cycle: u32,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    let preceding = preceding_year(year, meter)?;
    let (cycles, _) = divide(preceding, GREGORIAN_CYCLE, meter)?;
    let days = multiply(cycles, DAYS_IN_CYCLE.get(), meter)?;
    add(days, within_cycle, meter)
}

fn date_ordinal<M: HtmlInputTemporalStepMeter>(
    date: BorrowedDate<'_>,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Integer>, M> {
    calendar_work(meter)?;
    day_ordinal(date.year(), date.within_cycle_days(), meter)
}

fn calendar_work<M: HtmlInputTemporalStepMeter>(meter: &mut Session<'_, M>) -> StepResult<(), M> {
    // Gregorian and clock component checks visit at most twelve months.
    meter.work(Some(256))
}

#[derive(Clone, Copy)]
struct Clock<'a> {
    milliseconds: u32,
    fraction: &'a [u16],
}

impl<'a> Clock<'a> {
    fn from_plan(plan: BorrowedTime<'a>) -> Self {
        let (prefix, fraction) = plan
            .fraction_digits()
            .split_at(plan.fraction_digits().len().min(3));
        let milliseconds = prefix
            .iter()
            .zip([100, 10, 1])
            .map(|(&unit, place)| u32::from(unit - u16::from(b'0')) * place)
            .sum::<u32>();
        Self {
            milliseconds: u32::from(plan.hour) * 3_600_000
                + u32::from(plan.minute) * 60_000
                + u32::from(plan.second) * 1_000
                + milliseconds,
            fraction,
        }
    }

    const MIDNIGHT: Self = Self {
        milliseconds: 0,
        fraction: &[],
    };
}

fn decimal<M: HtmlInputTemporalStepMeter>(
    whole: Owned<Integer>,
    fraction: &[u16],
    negative: bool,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<HtmlSignedDecimal>, M> {
    let fraction = if fraction.is_empty() {
        fraction
    } else {
        meter.work(fraction.len().checked_add(1))?;
        let retained = fraction
            .iter()
            .rposition(|&digit| digit != u16::from(b'0'))
            .map_or(0, |index| index + 1);
        &fraction[..retained]
    };
    if fraction.is_empty() {
        return Ok(whole.map(|integer| HtmlSignedDecimal::from_magnitude(integer.into(), negative)));
    }
    let integer_len = width(&whole.value);
    let mut units = meter.buffer(integer_len, true)?;
    meter.work(integer_len.checked_add(1))?;
    units
        .value
        .extend(whole.value.decimal_digits().bytes().map(u16::from));
    meter.discard(whole);
    let value = meter.materialize(
        HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(integer_len, fraction.len()),
        integer_len.checked_add(fraction.len()),
        |bytes| {
            HtmlNonNegativeDecimal::try_from_borrowed_digits(&units.value, fraction, bytes, bytes)
        },
    )?;
    meter.discard(units);
    Ok(value.map(|magnitude| HtmlSignedDecimal::from_magnitude(magnitude, negative)))
}

fn timestamp<M: HtmlInputTemporalStepMeter>(
    ordinal: Owned<Integer>,
    clock: Clock<'_>,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<HtmlSignedDecimal>, M> {
    let (days, negative) = difference(ordinal, DAYS_BEFORE_EPOCH, meter)?;
    if !negative {
        let start = multiply(days, MILLISECONDS_PER_DAY.get(), meter)?;
        let whole = add(start, clock.milliseconds, meter)?;
        return decimal(whole, clock.fraction, false, meter);
    }

    // Negative HTML dates are bounded by the finite number of days before 1970.
    // Subtract the positive clock from the magnitude and borrow for a fraction.
    meter.work(
        width(&days.value)
            .checked_add(clock.fraction.len())
            .and_then(|n| n.checked_add(1)),
    )?;
    let days_count = days
        .value
        .to_u64()
        .map_err(|_| HtmlInputTemporalStepError::Representation)?;
    let mut whole =
        days_count * u64::from(MILLISECONDS_PER_DAY.get()) - u64::from(clock.milliseconds);
    meter.discard(days);
    let end = clock
        .fraction
        .iter()
        .rposition(|&digit| digit != u16::from(b'0'))
        .map_or(0, |end| end + 1);
    if end == 0 {
        let whole = small(whole, meter)?;
        return decimal(whole, &[], true, meter);
    }
    whole -= 1;
    let mut fraction = meter.buffer(end, true)?;
    meter.work(end.checked_add(1))?;
    for (index, &digit) in clock.fraction[..end].iter().enumerate() {
        fraction
            .value
            .push(u16::from(b'0' + b'9') - digit + u16::from(index + 1 == end));
    }
    let whole = small(whole, meter)?;
    let result = decimal(whole, &fraction.value, true, meter)?;
    meter.discard(fraction);
    Ok(result)
}

/// Decodes the actual recovery string into the state's exact stepping coordinate.
/// An absent or invalid string has no coordinate. Storage in a successful
/// coordinate remains admitted for the caller until it drops that coordinate.
pub fn decode_input_temporal_step<M: HtmlInputTemporalStepMeter>(
    state: HtmlInputType,
    raw: Option<&[u16]>,
    max_bytes: usize,
    meter: &mut M,
) -> HtmlInputTemporalStepOutcome<Option<HtmlSignedDecimal>, M::Stop> {
    run(meter, max_bytes, |meter| {
        let alternative = match TemporalState::of(state) {
            Some(TemporalState::Date) => TemporalAlternative::Date,
            Some(TemporalState::Month) => TemporalAlternative::Month,
            Some(TemporalState::Week) => TemporalAlternative::Week,
            Some(TemporalState::Time) => TemporalAlternative::Time,
            Some(TemporalState::LocalDateTime) => TemporalAlternative::LocalDateTime,
            None => return Err(HtmlInputTemporalStepError::Inapplicable),
        };
        let Some(raw) = raw else {
            return Ok(Owned::without_storage(None));
        };
        meter.work(temporal_recognition_work_bound(raw.len()))?;
        let Ok(plan) = recognize_time_alternative(raw, TemporalPhase::Recovery, alternative) else {
            return Ok(Owned::without_storage(None));
        };
        let value = match plan {
            BorrowedTemporalPlan::Month(month) => {
                let years = preceding_year(month.year(), meter)?;
                let months = multiply(years, MONTHS_PER_YEAR.get(), meter)?;
                let ordinal = add(months, u32::from(month.month - 1), meter)?;
                let (whole, negative) = difference(ordinal, MONTHS_BEFORE_EPOCH, meter)?;
                decimal(whole, &[], negative, meter)?
            }
            BorrowedTemporalPlan::Date(date) => {
                timestamp(date_ordinal(date, meter)?, Clock::MIDNIGHT, meter)?
            }
            BorrowedTemporalPlan::Week(week) => {
                calendar_work(meter)?;
                let year = week.year();
                let preceding = u32::from(year.remainder_400())
                    .checked_sub(1)
                    .unwrap_or(GREGORIAN_CYCLE.get() - 1);
                // A complete 400-year cycle contains an integral number of ISO weeks.
                let january_four = days_before_year_in_cycle(preceding) + 3;
                let days = january_four - january_four % DAYS_IN_WEEK.get()
                    + u32::from(week.week - 1) * DAYS_IN_WEEK.get();
                timestamp(day_ordinal(year, days, meter)?, Clock::MIDNIGHT, meter)?
            }
            BorrowedTemporalPlan::Time(time) => {
                meter.work(Some(1))?;
                let clock = Clock::from_plan(time);
                let whole = small(u64::from(clock.milliseconds), meter)?;
                decimal(whole, clock.fraction, false, meter)?
            }
            BorrowedTemporalPlan::LocalDateTime(local) => {
                meter.work(Some(1))?;
                timestamp(
                    date_ordinal(local.date(), meter)?,
                    Clock::from_plan(local.time()),
                    meter,
                )?
            }
            _ => return Err(HtmlInputTemporalStepError::Representation),
        };
        Ok(value.map(Some))
    })
}

fn rounded_coordinate<'a, M: HtmlInputTemporalStepMeter>(
    state: TemporalState,
    coordinate: HtmlInputStepCoordinate<'a>,
    meter: &mut Session<'_, M>,
) -> StepResult<(&'a [u8], bool), M> {
    meter.work(Some(1))?;
    let scale = coordinate.scale();
    let cells = coordinate.coefficient();
    let (fraction, integer) = cells.split_at(scale.min(cells.len()));
    let increment = match state {
        TemporalState::Date | TemporalState::Week => {
            if coordinate.is_negative() {
                meter.work(fraction.len().checked_add(1))?;
                fraction.iter().any(|&digit| digit != 0)
            } else {
                false
            }
        }
        TemporalState::Month | TemporalState::Time | TemporalState::LocalDateTime => scale
            .checked_sub(1)
            .and_then(|index| fraction.get(index))
            .is_some_and(|&digit| digit >= 5),
    };
    Ok((integer, increment))
}

fn origin_position<M: HtmlInputTemporalStepMeter>(
    magnitude: Owned<Integer>,
    negative: bool,
    origin: u32,
    meter: &mut Session<'_, M>,
) -> StepResult<Option<Owned<Integer>>, M> {
    if !negative {
        return add(magnitude, origin, meter).map(Some);
    }
    let (position, before_origin) = difference(magnitude, origin, meter)?;
    if before_origin || position.value.is_zero() {
        Ok(Some(position))
    } else {
        meter.discard(position);
        Ok(None)
    }
}

fn calendar_year<M: HtmlInputTemporalStepMeter>(
    ordinal: Owned<Integer>,
    meter: &mut Session<'_, M>,
) -> StepResult<(Owned<Integer>, u32, u32), M> {
    let (cycles, day) = divide(ordinal, DAYS_IN_CYCLE, meter)?;
    calendar_work(meter)?;
    let (preceding, day_in_year) = year_and_day_in_cycle(day);
    let years = multiply(cycles, GREGORIAN_CYCLE.get(), meter)?;
    let year = add(years, preceding + 1, meter)?;
    Ok((year, preceding, day_in_year))
}

#[derive(Default)]
struct Suffix {
    bytes: [u8; 20],
    len: usize,
}

impl Write for Suffix {
    fn write_str(&mut self, spelling: &str) -> fmt::Result {
        let end = self.len + spelling.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(spelling.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn write_clock(suffix: &mut Suffix, milliseconds: u32) -> fmt::Result {
    let time = time_at_millisecond(milliseconds).ok_or(fmt::Error)?;
    time.write_spelling(suffix)
}

fn emit<M: HtmlInputTemporalStepMeter>(
    year: Option<Owned<Integer>>,
    suffix: Suffix,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Vec<u16>>, M> {
    let digits = year.as_ref().map(|year| year.value.decimal_digits());
    let year_len = digits.map_or(0, |digits| digits.len().max(4));
    let length = year_len
        .checked_add(suffix.len)
        .ok_or(HtmlInputTemporalStepError::Representation)?;
    let mut output = meter.buffer(length, true)?;
    meter.work(length.checked_add(1))?;
    if let Some(digits) = digits {
        output.value.extend(std::iter::repeat_n(
            u16::from(b'0'),
            year_len - digits.len(),
        ));
        output.value.extend(digits.bytes().map(u16::from));
    }
    output
        .value
        .extend(suffix.bytes[..suffix.len].iter().copied().map(u16::from));
    if let Some(year) = year {
        meter.discard(year);
    }
    Ok(output)
}

fn inverse<M: HtmlInputTemporalStepMeter>(
    state: TemporalState,
    coordinate: HtmlInputStepCoordinate<'_>,
    meter: &mut Session<'_, M>,
) -> StepResult<Owned<Vec<u16>>, M> {
    let (integer_cells, increment) = rounded_coordinate(state, coordinate, meter)?;
    let mut suffix = Suffix::default();
    let formatting_error = |_| HtmlInputTemporalStepError::Representation;
    let day_milliseconds = MILLISECONDS_PER_DAY.get();

    if state == TemporalState::Time {
        meter.work(integer_cells.len().checked_add(1))?;
        let magnitude = integer_cells.iter().rev().fold(0_u32, |remainder, &digit| {
            (10 * remainder + u32::from(digit)) % day_milliseconds
        });
        let rounded = (magnitude + u32::from(increment)) % day_milliseconds;
        let time = if coordinate.is_negative() && rounded != 0 {
            day_milliseconds - rounded
        } else {
            rounded
        };
        calendar_work(meter)?;
        write_clock(&mut suffix, time).map_err(formatting_error)?;
        return emit(None, suffix, meter);
    }

    let units = digit_units(integer_cells, meter)?;
    let magnitude = integer(&units.value, meter)?;
    meter.discard(units);
    let magnitude = add(magnitude, u32::from(increment), meter)?;
    if state == TemporalState::Month {
        let Some(months) = origin_position(
            magnitude,
            coordinate.is_negative(),
            MONTHS_BEFORE_EPOCH,
            meter,
        )?
        else {
            return Ok(Owned::without_storage(Vec::new()));
        };
        let (years, month) = divide(months, MONTHS_PER_YEAR, meter)?;
        let year = add(years, 1, meter)?;
        write!(suffix, "-{:02}", month + 1).map_err(formatting_error)?;
        return emit(Some(year), suffix, meter);
    }

    let (days, milliseconds) = divide(magnitude, MILLISECONDS_PER_DAY, meter)?;
    let (days, milliseconds) = if coordinate.is_negative() && milliseconds != 0 {
        (add(days, 1, meter)?, day_milliseconds - milliseconds)
    } else {
        (days, milliseconds)
    };
    let Some(ordinal) = origin_position(days, coordinate.is_negative(), DAYS_BEFORE_EPOCH, meter)?
    else {
        return Ok(Owned::without_storage(Vec::new()));
    };
    if state == TemporalState::Week {
        meter.work(width(&ordinal.value).checked_add(1))?;
        let weekday = ordinal.value.remainder(DAYS_IN_WEEK);
        let thursday = if weekday > 3 {
            difference(ordinal, weekday - 3, meter)?.0
        } else {
            add(ordinal, 3 - weekday, meter)?
        };
        let (year, _, day_in_year) = calendar_year(thursday, meter)?;
        // The first Thursday is always within the first seven days of its year.
        write!(suffix, "-W{:02}", day_in_year / DAYS_IN_WEEK.get() + 1)
            .map_err(formatting_error)?;
        return emit(Some(year), suffix, meter);
    }

    let (year, preceding, day_in_year) = calendar_year(ordinal, meter)?;
    calendar_work(meter)?;
    let (month, date) = month_day_in_cycle_year(preceding, day_in_year)
        .ok_or(HtmlInputTemporalStepError::Representation)?;
    write!(suffix, "-{month:02}-{date:02}").map_err(formatting_error)?;
    if state == TemporalState::LocalDateTime {
        suffix.write_char('T').map_err(formatting_error)?;
        write_clock(&mut suffix, milliseconds).map_err(formatting_error)?;
    }
    emit(Some(year), suffix, meter)
}

/// Serializes an exact stepped temporal coordinate to admitted UTF-16 units.
///
/// Date and Week select the containing period. Month, Time and Local Date
/// and Time apply the same whole-unit, ties-away policy as the Number setter.
/// The clock wraps, and an unrepresentable calendar year produces empty text.
/// The returned backing remains admitted for the caller's output transfer.
pub fn serialize_input_temporal_step<M: HtmlInputTemporalStepMeter>(
    state: HtmlInputType,
    coordinate: HtmlInputStepCoordinate<'_>,
    max_bytes: usize,
    meter: &mut M,
) -> HtmlInputTemporalStepOutcome<Vec<u16>, M::Stop> {
    run(meter, max_bytes, |meter| {
        let state = TemporalState::of(state).ok_or(HtmlInputTemporalStepError::Inapplicable)?;
        inverse(state, coordinate, meter)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::elements::input_step::{
            HtmlInputStepBounds, HtmlInputStepDirection, HtmlInputStepOperand,
        },
        microsyntax::numeric::HtmlInteger,
        values::DocumentString,
    };

    #[derive(Default)]
    struct TestMeter {
        live: (usize, usize),
        reached: usize,
        refuse_at: Option<usize>,
    }

    impl TestMeter {
        fn action(&mut self) -> Result<(), usize> {
            self.reached += 1;
            if self.refuse_at == Some(self.reached) {
                return Err(self.reached);
            }
            Ok(())
        }
    }

    impl HtmlInputTemporalStepMeter for TestMeter {
        type Stop = usize;

        fn work(&mut self, _: usize) -> Result<(), usize> {
            self.action()
        }

        fn reserve(&mut self, bytes: usize, units: usize) -> Result<(), usize> {
            self.action()?;
            self.live.0 += bytes;
            self.live.1 += units;
            Ok(())
        }

        fn release(&mut self, bytes: usize, units: usize) {
            self.live.0 -= bytes;
            self.live.1 -= units;
        }
    }

    fn spelling(raw: &str) -> Vec<u16> {
        raw.encode_utf16().collect()
    }

    #[test]
    fn decode_preserves_selected_coordinate_units_and_unrounded_temporal_recovery() {
        use HtmlInputType as Input;
        for (state, raw, expected) in [
            (Input::Date, "0001-01-01", "-62135596800000"),
            (Input::Date, "2000-02-29", "951782400000"),
            (Input::Month, "1969-12", "-1"),
            (Input::Week, "2020-W53", "1609113600000"),
            (Input::Time, "23:59:59.9999999", "86399999.9999"),
            (Input::LocalDateTime, "1969-12-31 23:59:59.9999", "-0.1"),
            (
                Input::LocalDateTime,
                "287396-10-12T08:59:01.125",
                "9007199254741125",
            ),
        ] {
            let mut meter = TestMeter::default();
            let (result, observed, retained) =
                decode_input_temporal_step(state, Some(&spelling(raw)), usize::MAX, &mut meter)
                    .into_parts();
            let coordinate = result.unwrap().unwrap();
            assert_eq!(
                coordinate.as_document_string(),
                DocumentString::from(expected),
                "{raw}"
            );
            assert!(observed >= retained.0);
            assert_eq!(meter.live, retained);
            assert_eq!(retained.1, 0);
            drop(coordinate);
            meter.release(retained.0, retained.1);
            assert_eq!(meter.live, (0, 0));
        }
    }

    #[test]
    fn decode_distinguishes_invalid_and_absent_observations_from_admission_refusal() {
        let mut meter = TestMeter::default();
        for raw in [None, Some(&spelling("2021-W53")[..])] {
            assert_eq!(
                decode_input_temporal_step(HtmlInputType::Week, raw, 0, &mut meter).into_parts(),
                (Ok(None), 0, (0, 0))
            );
        }
        let mut meter = TestMeter::default();
        let units = spelling("1969-12-31T23:59:59.9999");
        let (value, _, retained) = decode_input_temporal_step(
            HtmlInputType::LocalDateTime,
            Some(&units),
            usize::MAX,
            &mut meter,
        )
        .into_parts();
        assert!(value.is_ok());
        let reached = meter.reached;
        drop(value);
        meter.release(retained.0, retained.1);
        for refuse_at in 1..=reached {
            let mut meter = TestMeter {
                refuse_at: Some(refuse_at),
                ..TestMeter::default()
            };
            let (value, _, retained) = decode_input_temporal_step(
                HtmlInputType::LocalDateTime,
                Some(&units),
                usize::MAX,
                &mut meter,
            )
            .into_parts();
            assert_eq!(value, Err(HtmlInputTemporalStepError::Admission(refuse_at)));
            assert_eq!(retained, (0, 0));
            assert_eq!(meter.live, (0, 0));
            assert_eq!(meter.reached, refuse_at);
        }
    }

    #[test]
    fn decode_applies_the_live_byte_limit_before_allocating() {
        let mut meter = TestMeter::default();
        let (result, observed, retained) = decode_input_temporal_step(
            HtmlInputType::Time,
            Some(&spelling("12:00")),
            7,
            &mut meter,
        )
        .into_parts();
        assert_eq!(
            result,
            Err(boundary(
                HtmlIntegerUnitStepError::RequestedCapacityLimitExceeded {
                    limit: 7,
                    required: 8,
                }
            ))
        );
        assert_eq!(observed, 0);
        assert_eq!(retained, (0, 0));
        assert_eq!(meter.live, (0, 0));

        let padded = spelling("12:00:00.00000000000000000000000000000000");
        let (result, observed, retained) =
            decode_input_temporal_step(HtmlInputType::Time, Some(&padded), 8, &mut meter)
                .into_parts();
        assert_eq!(
            result.unwrap().unwrap().as_document_string(),
            DocumentString::from("43200000")
        );
        assert_eq!(observed, 8);
        assert_eq!(retained, (8, 0));
        assert_eq!(meter.live, retained);
        meter.release(retained.0, retained.1);
    }

    fn exact(text: &str) -> HtmlSignedDecimal {
        let negative = text.starts_with('-');
        let text = text.trim_start_matches('-');
        let (integer, fraction) = text.split_once('.').unwrap_or((text, ""));
        let integer = Integer::from_decimal_digits(&DocumentString::from(integer)).unwrap();
        let magnitude = if fraction.is_empty() {
            integer.into()
        } else {
            HtmlNonNegativeDecimal::with_fraction_digits(integer, &DocumentString::from(fraction))
                .unwrap()
        };
        HtmlSignedDecimal::from_magnitude(magnitude, negative)
    }

    fn serialize(
        state: HtmlInputType,
        number: &str,
        meter: &mut TestMeter,
    ) -> HtmlInputTemporalStepOutcome<Vec<u16>, usize> {
        let coordinate = exact(number);
        let step = HtmlSignedDecimal::from(HtmlInteger::from(1));
        let operand = HtmlInputStepOperand::decimal(&coordinate);
        let grid = HtmlInputStepBounds {
            minimum: None,
            maximum: None,
            base: operand,
            step: HtmlInputStepOperand::magnitude(step.magnitude()),
        }
        .qualify(&mut [], || Ok::<_, std::convert::Infallible>(()))
        .unwrap()
        .unwrap();
        let mut scratch = vec![0; grid.scratch_cells(Some(operand)).unwrap()];
        let result = grid
            .project(
                Some(operand),
                HtmlInputStepDirection::Up,
                0,
                &mut scratch,
                || Ok::<_, std::convert::Infallible>(()),
            )
            .unwrap()
            .unwrap();
        serialize_input_temporal_step(state, result, usize::MAX, meter)
    }

    #[test]
    fn inverse_preserves_exact_calendar_and_submillisecond_boundary_decisions() {
        use HtmlInputType as Input;
        for (state, coordinate, expected) in [
            (Input::Date, "-0.25", "1969-12-31"),
            (Input::Date, "-62135596800000.1", ""),
            (Input::Date, "8640000086400000", "275760-09-14"),
            (
                Input::Date,
                "340282366920938463463374607431768211456",
                "10783118943836478994022447719-04-03",
            ),
            (Input::Month, "-0.5", "1969-12"),
            (Input::Month, "0.5", "1970-02"),
            (
                Input::Month,
                "340282366920938463463374607431768211456",
                "28356863910078205288614550619314019591-05",
            ),
            (Input::Week, "-259200000.01", "1969-W52"),
            (Input::Week, "1609459200000", "2020-W53"),
            (Input::Time, "-0.5", "23:59:59.999"),
            (Input::Time, "86399999.5", "00:00"),
            (
                Input::Time,
                "340282366920938463463374607431768211456",
                "09:23:31.456",
            ),
            (
                Input::LocalDateTime,
                "9007199254741250",
                "287396-10-12T08:59:01.25",
            ),
        ] {
            let mut meter = TestMeter::default();
            let (result, observed, retained) =
                serialize(state, coordinate, &mut meter).into_parts();
            let units = result.unwrap();
            assert_eq!(units, spelling(expected), "{state:?} {coordinate}");
            assert!(observed >= retained.0);
            assert_eq!(
                retained,
                (units.capacity() * size_of::<u16>(), units.capacity())
            );
            assert_eq!(meter.live, retained);
            drop(units);
            meter.release(retained.0, retained.1);
            assert_eq!(meter.live, (0, 0));
        }
    }

    #[test]
    fn inverse_first_refusal_discards_all_temporary_and_output_storage() {
        let mut meter = TestMeter::default();
        let coordinate = "9007199254741250";
        let (value, _, retained) =
            serialize(HtmlInputType::LocalDateTime, coordinate, &mut meter).into_parts();
        assert!(value.is_ok());
        drop(value);
        meter.release(retained.0, retained.1);
        for refuse_at in 1..=meter.reached {
            let mut meter = TestMeter {
                refuse_at: Some(refuse_at),
                ..TestMeter::default()
            };
            let (value, _, retained) =
                serialize(HtmlInputType::LocalDateTime, coordinate, &mut meter).into_parts();
            assert_eq!(value, Err(HtmlInputTemporalStepError::Admission(refuse_at)));
            assert_eq!(retained, (0, 0));
            assert_eq!(meter.live, (0, 0));
            assert_eq!(meter.reached, refuse_at);
        }
    }
}
