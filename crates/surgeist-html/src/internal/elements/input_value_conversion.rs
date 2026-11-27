//! Shared local decisions for the HTML Input Date and Number IDL value routes.

use crate::internal::{
    attributes::HtmlInputType,
    microsyntax::{
        numeric::{
            HtmlFloatingPointNumber, HtmlInteger, HtmlSignedDecimal, parse_floating_point_number,
        },
        temporal::{
            HtmlDate, HtmlLocalDateTime, HtmlMonth,
            input_coordinate::{
                HtmlInputTemporalCoordinate, TemporalState,
                inverse::{
                    date_at_day, epoch_day_and_time, epoch_month, time_at_millisecond, week_at_day,
                },
            },
            parse_date, parse_local_date_time, parse_month, parse_time, parse_week,
            temporal_recognition_work_bound,
        },
    },
    values::DocumentString,
};
use std::{error::Error, fmt};

/// Inclusive ECMAScript TimeClip limit in milliseconds from the UTC epoch.
const DATE_TIME_LIMIT: u64 = 8_640_000_000_000_000;

/// A Date's exact representable UTC time value. Branding belongs to the engine adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputDateMilliseconds(i64);

impl HtmlInputDateMilliseconds {
    /// Qualifies an integral time value under ECMAScript TimeClip.
    pub const fn new(milliseconds: i64) -> Option<Self> {
        if milliseconds.unsigned_abs() <= DATE_TIME_LIMIT {
            Some(Self(milliseconds))
        } else {
            None
        }
    }

    /// Returns UTC milliseconds from Unix epoch midnight.
    pub const fn milliseconds(self) -> i64 {
        self.0
    }
}

/// Native readback for a Date-valued Input getter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputDateRead {
    /// The state is inapplicable or conversion of the current value failed.
    Null,
    /// Parsing succeeded but the time value lies outside the Date domain.
    InvalidTime,
    /// A Date can be constructed with this exact UTC time value.
    Time(HtmlInputDateMilliseconds),
}

/// A converted input to the canonical value mutator. Applying it owns dirty
/// state, sanitization, storage admission, and any subsequent DOM effects.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlInputValueAssignment {
    /// Assign the empty string for this reached branch.
    Empty,
    /// Assign this valid string through the ordinary value path.
    Text(DocumentString),
    /// Serialize the finite value with the caller-driven
    /// [`HtmlBestFiniteNumberCursor`](crate::bounded::numeric::HtmlBestFiniteNumberCursor),
    /// then assign its output through the ordinary Range value path.
    RangeNumber(HtmlFloatingPointNumber),
}

/// A conversion could not be performed in the selected state or allowance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueConversionError<E> {
    /// The selected setter does not apply to this Input state.
    Inapplicable,
    /// The number setter refuses infinity before testing applicability.
    Infinite,
    /// The work bound cannot fit in the caller's native counter.
    WorkOverflow,
    /// The caller refused the checked work reservation.
    Admission(E),
}

impl<E: fmt::Display> fmt::Display for HtmlInputValueConversionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inapplicable => {
                formatter.write_str("value conversion does not apply to this Input state")
            }
            Self::Infinite => formatter.write_str("Input valueAsNumber refuses infinity"),
            Self::WorkOverflow => {
                formatter.write_str("Input value conversion work does not fit in usize")
            }
            Self::Admission(error) => write!(
                formatter,
                "Input value conversion admission refused: {error}"
            ),
        }
    }
}

impl<E: Error + 'static> Error for HtmlInputValueConversionError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            _ => None,
        }
    }
}

impl HtmlInputType {
    /// Reports whether the current state supports the Date-valued IDL attribute.
    pub const fn supports_value_as_date(self) -> bool {
        matches!(TemporalState::of(self), Some(state) if !matches!(state, TemporalState::LocalDateTime))
    }

    /// Reports whether the current state supports the number-valued IDL attribute.
    pub const fn supports_value_as_number(self) -> bool {
        TemporalState::of(self).is_some() || matches!(self, Self::Number | Self::Range)
    }
}

/// Converts the actual current Input value to a finite binary64 Number.
///
/// `None` selects the IDL NaN result. A state that does not apply does no work.
/// The callback admits a checked bound for one atomic operation before the
/// string is read or copied. It can apply the caller's cancellation policy.
/// This helper uses ordinary allocating HTML value primitives during that bound.
///
/// # Errors
/// Returns the original admission refusal or an unrepresentable work bound.
pub fn input_value_as_number<E>(
    state: HtmlInputType,
    value: &DocumentString,
    admit: impl FnOnce(usize) -> Result<(), E>,
) -> Result<Option<HtmlFloatingPointNumber>, HtmlInputValueConversionError<E>> {
    if matches!(state, HtmlInputType::Number | HtmlInputType::Range) {
        admit_work(
            admit,
            crate::bounded::numeric::floating_point_number_work_bound(value.code_units().len()),
        )?;
        return Ok(parse_floating_point_number(value).ok());
    }
    let Some(state) = TemporalState::of(state) else {
        return Ok(None);
    };
    admit_work(admit, temporal_read_work(value.code_units().len()))?;
    Ok(temporal_coordinate(state, value).and_then(|value| value.value().to_floating_point().ok()))
}

/// Converts the actual current Input value to its Date readback.
///
/// Month uses the first day's UTC midnight, and Time truncates its exact
/// fractional milliseconds toward zero before Date storage is qualified.
/// Inapplicable states do no work. The callback runs before any current value
/// units are read, and admits one atomic bound including ordinary allocations.
///
/// # Errors
/// Returns the original admission refusal or an unrepresentable work bound.
pub fn input_value_as_date<E>(
    state: HtmlInputType,
    value: &DocumentString,
    admit: impl FnOnce(usize) -> Result<(), E>,
) -> Result<HtmlInputDateRead, HtmlInputValueConversionError<E>> {
    if !state.supports_value_as_date() {
        return Ok(HtmlInputDateRead::Null);
    }
    admit_work(admit, temporal_read_work(value.code_units().len()))?;
    let coordinate = if state == HtmlInputType::Month {
        parse_month(value)
            .ok()
            .and_then(|month| HtmlDate::new(month.year().clone(), month.month(), 1).ok())
            .map(|day| day.input_coordinate())
    } else {
        TemporalState::of(state).and_then(|state| temporal_coordinate(state, value))
    };
    let Some(coordinate) = coordinate else {
        return Ok(HtmlInputDateRead::Null);
    };
    let coordinate = coordinate.value();
    let magnitude = coordinate.magnitude().integer_part().to_u64().ok();
    Ok(magnitude
        .and_then(|magnitude| {
            let milliseconds = i64::try_from(magnitude).ok()?;
            HtmlInputDateMilliseconds::new(if coordinate.is_negative() {
                -milliseconds
            } else {
                milliseconds
            })
        })
        .map_or(HtmlInputDateRead::InvalidTime, HtmlInputDateRead::Time))
}

/// Selects the actual HTML assignment for the number-valued IDL setter.
///
/// Infinity precedes applicability, and applicable NaN clears the value.
/// Date and Week select their containing day or week. Where exact author
/// syntax cannot express an offset, Month rounds to a whole month and Time
/// and Local Date and Time round to whole milliseconds, with ties away from
/// zero, following the pinned [WebKit precedent](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/DateComponents.cpp#L564-L615).
/// Time wraps across day boundaries.
/// The Range result hands the finite value to the existing bounded serializer.
///
/// # Errors
/// Returns infinity, inapplicability, or the original admission refusal.
pub fn input_value_from_number<E>(
    state: HtmlInputType,
    value: f64,
    admit: impl FnOnce(usize) -> Result<(), E>,
) -> Result<HtmlInputValueAssignment, HtmlInputValueConversionError<E>> {
    if value.is_infinite() {
        return Err(HtmlInputValueConversionError::Infinite);
    }
    if !state.supports_value_as_number() {
        return Err(HtmlInputValueConversionError::Inapplicable);
    }
    let Ok(number) = HtmlFloatingPointNumber::try_from(value) else {
        return Ok(HtmlInputValueAssignment::Empty);
    };
    if state == HtmlInputType::Range {
        return Ok(HtmlInputValueAssignment::RangeNumber(number));
    }
    if state == HtmlInputType::Number {
        // A binary64 plain spelling, including the smallest subnormal, has
        // at most 327 UTF-16 units and is copied once into the owned value.
        admit_work(admit, Some(1024))?;
        return Ok(HtmlInputValueAssignment::Text(
            number.to_plain_decimal_string(),
        ));
    }
    let Some(state) = TemporalState::of(state) else {
        return Err(HtmlInputValueConversionError::Inapplicable);
    };
    let integral = match state {
        TemporalState::Date | TemporalState::Week => number.value().floor(),
        TemporalState::Month | TemporalState::Time | TemporalState::LocalDateTime => {
            number.value().round()
        }
    };
    let Ok(integral) = HtmlFloatingPointNumber::try_from(integral) else {
        return Err(HtmlInputValueConversionError::Infinite);
    };
    admit_work(admit, Some(temporal_number_write_work(integral.value())))?;
    let quantity = integral.exact_decimal();
    Ok(assignment(if matches!(state, TemporalState::Month) {
        epoch_month(&quantity).map(|month| month.to_document_string())
    } else {
        calendar_string(state, &quantity)
    }))
}

/// Selects the actual HTML assignment for a qualified Date or empty Date input.
///
/// Supply `None` for null or an already branded Date with invalid time.
/// The binding must check state applicability before it attempts branding.
/// The callback admits one bounded operation before any string is produced.
///
/// # Errors
/// Returns inapplicability or the original admission refusal.
pub fn input_value_from_date<E>(
    state: HtmlInputType,
    date: Option<HtmlInputDateMilliseconds>,
    admit: impl FnOnce(usize) -> Result<(), E>,
) -> Result<HtmlInputValueAssignment, HtmlInputValueConversionError<E>> {
    if !state.supports_value_as_date() {
        return Err(HtmlInputValueConversionError::Inapplicable);
    }
    let Some(date) = date else {
        return Ok(HtmlInputValueAssignment::Empty);
    };
    let Some(state) = TemporalState::of(state) else {
        return Err(HtmlInputValueConversionError::Inapplicable);
    };
    admit_work(admit, Some(temporal_number_write_work(0.0)))?;
    let quantity = HtmlSignedDecimal::from(HtmlInteger::from(i128::from(date.milliseconds())));
    Ok(assignment(calendar_string(state, &quantity)))
}

fn assignment(value: Option<DocumentString>) -> HtmlInputValueAssignment {
    value.map_or(
        HtmlInputValueAssignment::Empty,
        HtmlInputValueAssignment::Text,
    )
}

fn temporal_coordinate(
    state: TemporalState,
    value: &DocumentString,
) -> Option<HtmlInputTemporalCoordinate> {
    Some(match state {
        TemporalState::Date => parse_date(value).ok()?.input_coordinate(),
        TemporalState::Month => parse_month(value).ok()?.input_coordinate(),
        TemporalState::Week => parse_week(value).ok()?.input_coordinate(),
        TemporalState::Time => parse_time(value).ok()?.input_coordinate(),
        TemporalState::LocalDateTime => parse_local_date_time(value).ok()?.input_coordinate(),
    })
}

fn calendar_string(state: TemporalState, value: &HtmlSignedDecimal) -> Option<DocumentString> {
    let (day, milliseconds) = epoch_day_and_time(value);
    match state {
        TemporalState::Date => Some(date_at_day(&day?)?.to_document_string()),
        TemporalState::Month => {
            let date = date_at_day(&day?)?;
            Some(
                HtmlMonth::new(date.year().clone(), date.month())
                    .ok()?
                    .to_document_string(),
            )
        }
        TemporalState::Week => Some(week_at_day(day?)?.to_document_string()),
        TemporalState::Time => Some(time_at_millisecond(milliseconds)?.to_document_string()),
        TemporalState::LocalDateTime => {
            let date = date_at_day(&day?)?;
            let time = time_at_millisecond(milliseconds)?;
            Some(HtmlLocalDateTime::new(date, time).to_document_string())
        }
    }
}

fn admit_work<E>(
    admit: impl FnOnce(usize) -> Result<(), E>,
    work: Option<usize>,
) -> Result<(), HtmlInputValueConversionError<E>> {
    admit(work.ok_or(HtmlInputValueConversionError::WorkOverflow)?)
        .map_err(HtmlInputValueConversionError::Admission)
}

fn temporal_read_work(units: usize) -> Option<usize> {
    // Each exact addition, small-divisor division, small-factor multiplication,
    // and materialization is linear in its decimal columns. Epoch arithmetic
    // adds at most 12 fixed columns. The per-column envelope allows 24 whole
    // digit passes at four units per column, covering leading/trailing digit
    // walks, calendar arithmetic and conversion back to a binary64 token.
    // The fixed recognition allowance covers the calendar traversal.
    temporal_recognition_work_bound(units)?.checked_add(units.checked_add(12)?.checked_mul(96)?)
}

fn temporal_number_write_work(integral: f64) -> usize {
    // At most 309 decimal columns and 32 exact multiply passes are reachable
    // with finite integral binary64. For the small integral profile, the
    // temporary exact binary64 coefficient has at most 53 columns before
    // removing scale, with at most four multiplication passes. Both bounds
    // also allow the calendar's division, week qualification and output.
    if integral.abs() < 1e16 { 4096 } else { 65_536 }
}
