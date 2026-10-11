//! Intrinsically valid HTML temporal values from the pinned
//! [HTML foundations §2.3.5](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dates-and-times).
//!
//! Author validation and exact processing results are separate. Processing
//! retains arbitrary decimal fractions and normalizes explicit offsets without
//! substituting permissive parsing for author conformance. No host clock or time
//! zone is consulted.

use crate::{
    DocumentString, HtmlInputType,
    numeric::{
        HtmlFloatingPointNumber, HtmlInteger, HtmlNonNegativeDecimal, HtmlNonNegativeInteger,
        HtmlSignedDecimal,
    },
};
use std::{error::Error, fmt, num::NonZeroU32};

/// A rejected temporal grammar or source-defined intrinsic component constraint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTemporalError {
    /// A year contains something other than one or more ASCII decimal digits.
    InvalidYearDigits,
    /// HTML years and week-years must be positive.
    ZeroYear,
    /// A month is outside 1 through 12.
    MonthOutOfRange,
    /// A day does not exist in the selected Gregorian month.
    DayOutOfRange,
    /// An hour is outside 0 through 23.
    HourOutOfRange,
    /// A minute is outside 0 through 59.
    MinuteOutOfRange,
    /// Seconds are at least 60; leap seconds are not representable. Authored
    /// integer seconds range from 0 through 59, with an optional fraction.
    SecondOutOfRange,
    /// A millisecond is outside 0 through 999.
    MillisecondOutOfRange,
    /// An offset is outside minus 23:59 through plus 23:59.
    OffsetOutOfRange,
    /// A week does not exist in the selected ISO week-year.
    WeekOutOfRange,
    /// Duration processing accumulated nonzero years or months, which cannot
    /// describe a fixed number of seconds.
    NonzeroCalendarDuration,
    /// The input fails the selected author grammar or processing algorithm.
    InvalidSyntax,
}

impl fmt::Display for HtmlTemporalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidYearDigits => "year requires ASCII decimal digits",
            Self::ZeroYear => "HTML year must be greater than zero",
            Self::MonthOutOfRange => "month must be between 1 and 12",
            Self::DayOutOfRange => "day does not exist in this month",
            Self::HourOutOfRange => "hour must be between 0 and 23",
            Self::MinuteOutOfRange => "minute must be between 0 and 59",
            Self::SecondOutOfRange => "seconds must be below 60",
            Self::MillisecondOutOfRange => "millisecond must be between 0 and 999",
            Self::OffsetOutOfRange => "time-zone offset exceeds 23 hours and 59 minutes",
            Self::WeekOutOfRange => "week does not exist in this week-year",
            Self::NonzeroCalendarDuration => "duration contains nonzero calendar years or months",
            Self::InvalidSyntax => "input is not a valid HTML temporal string",
        })
    }
}

impl Error for HtmlTemporalError {}

/// An exact positive Gregorian year or ISO week-year, without a machine limit.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlYear {
    value: HtmlNonNegativeInteger,
}

impl HtmlYear {
    /// Constructs a positive year from numeric decimal digits, removing leading
    /// zeroes. This is a component constructor, not a temporal-string parser.
    pub fn from_decimal_digits(digits: &str) -> Result<Self, HtmlTemporalError> {
        let value = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(digits))
            .map_err(|_| HtmlTemporalError::InvalidYearDigits)?;
        Self::from_number(value)
    }

    /// Constructs a year from an exact number, rejecting zero.
    pub fn from_number(value: HtmlNonNegativeInteger) -> Result<Self, HtmlTemporalError> {
        if value.is_zero() {
            return Err(HtmlTemporalError::ZeroYear);
        }
        Ok(Self { value })
    }

    /// Constructs a positive year from a native integer.
    pub fn new(year: u64) -> Result<Self, HtmlTemporalError> {
        Self::from_decimal_digits(&year.to_string())
    }

    /// Returns the exact numeric year, without leading zeroes.
    pub fn decimal_digits(&self) -> &str {
        self.value.decimal_digits()
    }

    /// Reports the Gregorian leap-year condition.
    pub fn is_leap_year(&self) -> bool {
        let year = self.modulo_400();
        year == 0 || (year.is_multiple_of(4) && !year.is_multiple_of(100))
    }

    fn modulo_400(&self) -> u16 {
        self.value
            .remainder(NonZeroU32::new(400).expect("400 is nonzero")) as u16
    }

    fn spelling(&self) -> String {
        format!("{:0>4}", self.decimal_digits())
    }

    fn last_week(&self) -> u8 {
        // Gregorian weekdays repeat every 400 years. January 1, year 1 is
        // Monday. Count preceding common days and leap days modulo seven.
        let year = u32::from(self.modulo_400());
        let preceding = if year == 0 { 399 } else { year - 1 };
        let weekday = (preceding + preceding / 4 - preceding / 100 + preceding / 400) % 7;
        if weekday == 3 || (weekday == 2 && self.is_leap_year()) {
            53
        } else {
            52
        }
    }
}

/// A year and month without a day or time zone.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlMonth {
    year: HtmlYear,
    month: u8,
}

impl HtmlMonth {
    /// Constructs a month, rejecting an invalid month number.
    pub fn new(year: HtmlYear, month: u8) -> Result<Self, HtmlTemporalError> {
        if !(1..=12).contains(&month) {
            return Err(HtmlTemporalError::MonthOutOfRange);
        }
        Ok(Self { year, month })
    }
    /// Returns the year.
    pub fn year(&self) -> &HtmlYear {
        &self.year
    }
    /// Returns the month number, from 1 through 12.
    pub fn month(&self) -> u8 {
        self.month
    }
    /// Produces a valid HTML month string.
    pub fn to_document_string(&self) -> DocumentString {
        format!("{}-{:02}", self.year.spelling(), self.month).into()
    }
}

fn days_in_month(month: u8, leap: bool) -> u8 {
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Shared by the UTC and input-coordinate owners. Gregorian 0001-01-01 has
// ordinal zero. Only the within-year month visits are iterative; arbitrary
// years are handled with one 400-year quotient/remainder.
fn gregorian_ordinal(date: &HtmlDate) -> HtmlNonNegativeInteger {
    let preceding = date
        .year()
        .value
        .decremented()
        .expect("HTML years are positive");
    let (cycles, remainder) =
        preceding.quotient_remainder(NonZeroU32::new(400).expect("400 is nonzero"));
    let mut ordinal = u32::from(date.day()) - 1;
    for month in 1..date.month() {
        ordinal += u32::from(days_in_month(month, date.year().is_leap_year()));
    }
    let within_cycle = remainder * 365 + remainder / 4 - remainder / 100 + ordinal;
    cycles
        .multiplied_by(146097)
        .added_to(&u128::from(within_cycle).into())
}

fn epoch_milliseconds(ordinal: HtmlNonNegativeInteger) -> HtmlSignedDecimal {
    HtmlSignedDecimal::integer_difference(ordinal, 719162_u128.into()).multiplied_by(86400000)
}

/// A proleptic Gregorian date without time-zone information.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlDate {
    month: HtmlMonth,
    day: u8,
}

impl HtmlDate {
    /// Constructs a date and checks Gregorian month length and leap years.
    pub fn new(year: HtmlYear, month: u8, day: u8) -> Result<Self, HtmlTemporalError> {
        let month = HtmlMonth::new(year, month)?;
        if day == 0 || day > days_in_month(month.month, month.year.is_leap_year()) {
            return Err(HtmlTemporalError::DayOutOfRange);
        }
        Ok(Self { month, day })
    }
    /// Returns the year.
    pub fn year(&self) -> &HtmlYear {
        self.month.year()
    }
    /// Returns the month number.
    pub fn month(&self) -> u8 {
        self.month.month()
    }
    /// Returns the day of the month.
    pub fn day(&self) -> u8 {
        self.day
    }
    /// Produces a valid HTML date string.
    pub fn to_document_string(&self) -> DocumentString {
        format!(
            "{}-{:02}-{:02}",
            self.year().spelling(),
            self.month(),
            self.day
        )
        .into()
    }
}

/// A Gregorian month and day without an associated year; February 29 is valid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HtmlYearlessDate {
    month: u8,
    day: u8,
}

impl HtmlYearlessDate {
    /// Constructs a yearless date using leap-year month lengths.
    pub fn new(month: u8, day: u8) -> Result<Self, HtmlTemporalError> {
        if !(1..=12).contains(&month) {
            return Err(HtmlTemporalError::MonthOutOfRange);
        }
        if day == 0 || day > days_in_month(month, true) {
            return Err(HtmlTemporalError::DayOutOfRange);
        }
        Ok(Self { month, day })
    }
    /// Returns the month number.
    pub fn month(self) -> u8 {
        self.month
    }
    /// Returns the day of the month.
    pub fn day(self) -> u8 {
        self.day
    }
    /// Produces the valid HTML spelling without the optional leading hyphens.
    pub fn to_document_string(self) -> DocumentString {
        format!("{:02}-{:02}", self.month, self.day).into()
    }
}

/// A valid HTML authored time, with exact millisecond precision and no time zone.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HtmlTime {
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
}

impl HtmlTime {
    /// Constructs a time; leap seconds and sub-millisecond values are excluded.
    pub fn new(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
    ) -> Result<Self, HtmlTemporalError> {
        if hour > 23 {
            return Err(HtmlTemporalError::HourOutOfRange);
        }
        if minute > 59 {
            return Err(HtmlTemporalError::MinuteOutOfRange);
        }
        if second > 59 {
            return Err(HtmlTemporalError::SecondOutOfRange);
        }
        if millisecond > 999 {
            return Err(HtmlTemporalError::MillisecondOutOfRange);
        }
        Ok(Self {
            hour,
            minute,
            second,
            millisecond,
        })
    }
    /// Returns the hour.
    pub fn hour(self) -> u8 {
        self.hour
    }
    /// Returns the minute.
    pub fn minute(self) -> u8 {
        self.minute
    }
    /// Returns the whole second.
    pub fn second(self) -> u8 {
        self.second
    }
    /// Returns the millisecond fraction.
    pub fn millisecond(self) -> u16 {
        self.millisecond
    }
    fn spelling(self) -> String {
        let mut result = format!("{:02}:{:02}", self.hour, self.minute);
        if self.second != 0 || self.millisecond != 0 {
            result.push_str(&format!(":{:02}", self.second));
            if self.millisecond != 0 {
                result.push('.');
                result.push_str(format!("{:03}", self.millisecond).trim_end_matches('0'));
            }
        }
        result
    }
    /// Produces the shortest valid HTML spelling for this time.
    pub fn to_document_string(self) -> DocumentString {
        self.spelling().into()
    }
}

/// A date and time with no supplied or inferred time zone.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlLocalDateTime {
    date: HtmlDate,
    time: HtmlTime,
}

impl HtmlLocalDateTime {
    /// Couples intrinsically valid date and time values.
    pub fn new(date: HtmlDate, time: HtmlTime) -> Self {
        Self { date, time }
    }
    /// Returns the date.
    pub fn date(&self) -> &HtmlDate {
        &self.date
    }
    /// Returns the time.
    pub fn time(&self) -> HtmlTime {
        self.time
    }
    /// Produces a valid normalized local date and time string: `T` and the
    /// shortest time spelling, as required by §2.3.5.5.
    pub fn to_document_string(&self) -> DocumentString {
        format!(
            "{}-{:02}-{:02}T{}",
            self.date.year().spelling(),
            self.date.month(),
            self.date.day(),
            self.time.spelling()
        )
        .into()
    }
}

/// An explicit HTML time-zone offset in minutes east of UTC.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HtmlTimeZoneOffset {
    minutes: i16,
}

impl HtmlTimeZoneOffset {
    /// Constructs an explicit offset in the HTML range, minus 1439 to 1439.
    pub fn new(minutes: i16) -> Result<Self, HtmlTemporalError> {
        if !(-1439..=1439).contains(&minutes) {
            return Err(HtmlTemporalError::OffsetOutOfRange);
        }
        Ok(Self { minutes })
    }
    /// Returns the signed minutes east of UTC.
    pub fn minutes(self) -> i16 {
        self.minutes
    }
    fn spelling(self) -> String {
        if self.minutes == 0 {
            return "Z".into();
        }
        let magnitude = self.minutes.unsigned_abs();
        format!(
            "{}{:02}:{:02}",
            if self.minutes < 0 { '-' } else { '+' },
            magnitude / 60,
            magnitude % 60
        )
    }
    /// Produces a valid HTML offset string, using `Z` for zero.
    pub fn to_document_string(self) -> DocumentString {
        self.spelling().into()
    }
}

/// A global moment represented exactly by its local components and explicit
/// offset. The equivalent UTC instant is local time minus that offset.
///
/// This representation retains offset information without imposing a host date
/// range or consulting political time-zone databases.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlGlobalDateTime {
    local: HtmlLocalDateTime,
    offset: HtmlTimeZoneOffset,
}

impl HtmlGlobalDateTime {
    /// Couples valid local components with an explicitly supplied offset.
    pub fn new(local: HtmlLocalDateTime, offset: HtmlTimeZoneOffset) -> Self {
        Self { local, offset }
    }
    /// Returns the local date and time components.
    pub fn local(&self) -> &HtmlLocalDateTime {
        &self.local
    }
    /// Returns the explicit time-zone offset.
    pub fn offset(&self) -> HtmlTimeZoneOffset {
        self.offset
    }
    /// Produces a valid HTML global date and time string.
    pub fn to_document_string(&self) -> DocumentString {
        let date = self.local.date();
        format!(
            "{}-{:02}-{:02}T{}{}",
            date.year().spelling(),
            date.month(),
            date.day(),
            self.local.time().spelling(),
            self.offset.spelling()
        )
        .into()
    }
}

/// An ISO week-year and week number, whose first day is Monday.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlWeek {
    year: HtmlYear,
    week: u8,
}

impl HtmlWeek {
    /// Constructs a week, checking whether the week-year has 52 or 53 weeks.
    pub fn new(year: HtmlYear, week: u8) -> Result<Self, HtmlTemporalError> {
        if week == 0 || week > year.last_week() {
            return Err(HtmlTemporalError::WeekOutOfRange);
        }
        Ok(Self { year, week })
    }
    /// Returns the week-year, which can differ from the Gregorian year.
    pub fn year(&self) -> &HtmlYear {
        &self.year
    }
    /// Returns the week number.
    pub fn week(&self) -> u8 {
        self.week
    }
    /// Produces a valid HTML week string.
    pub fn to_document_string(&self) -> DocumentString {
        format!("{}-W{:02}", self.year.spelling(), self.week).into()
    }
}

/// The complete processing alternatives returned by §2.3.5.10.
///
/// This processing clause defines no independent author grammar. Its time
/// alternatives can retain more precision than valid authored time strings.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HtmlDateOrTime {
    /// A date without a time or offset.
    Date(HtmlDate),
    /// A time without a date or offset.
    Time(HtmlParsedTime),
    /// A date and time with an explicit offset.
    Global(HtmlParsedGlobalDateTime),
}

/// A valid date string with optional time (§2.3.5.10). A supplied time requires
/// an explicit offset; a standalone time and a local date/time are excluded.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HtmlDateWithOptionalTime {
    /// A Gregorian date with no time.
    Date(HtmlDate),
    /// A date and time with an explicit offset.
    Global(HtmlGlobalDateTime),
}

impl HtmlDateWithOptionalTime {
    /// Produces a valid HTML date string with optional time.
    pub fn to_document_string(&self) -> DocumentString {
        match self {
            Self::Date(value) => value.to_document_string(),
            Self::Global(value) => value.to_document_string(),
        }
    }
}

/// An exact nonnegative duration in seconds, with millisecond precision.
///
/// HTML valid duration strings never contain months or years. Seconds are
/// unbounded; this value does not impose a native integer or host-clock limit.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlDuration {
    seconds: HtmlNonNegativeInteger,
    millisecond: u16,
}

impl HtmlDuration {
    /// Constructs a duration from whole seconds and a millisecond fraction.
    pub fn new(
        seconds: HtmlNonNegativeInteger,
        millisecond: u16,
    ) -> Result<Self, HtmlTemporalError> {
        if millisecond > 999 {
            return Err(HtmlTemporalError::MillisecondOutOfRange);
        }
        Ok(Self {
            seconds,
            millisecond,
        })
    }
    /// Returns the exact whole seconds.
    pub fn seconds(&self) -> &HtmlNonNegativeInteger {
        &self.seconds
    }
    /// Returns the millisecond fraction.
    pub fn millisecond(&self) -> u16 {
        self.millisecond
    }
    /// Produces a valid ISO-form HTML duration string, without rounding.
    pub fn to_document_string(&self) -> DocumentString {
        let mut result = format!("PT{}", self.seconds.decimal_digits());
        if self.millisecond != 0 {
            result.push('.');
            result.push_str(format!("{:03}", self.millisecond).trim_end_matches('0'));
        }
        result.push('S');
        result.into()
    }
}

/// An exact parsed temporal value cannot be represented at millisecond precision.
/// No rounding or truncation has occurred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlTemporalPrecisionError;

impl fmt::Display for HtmlTemporalPrecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("temporal value is not exactly representable in milliseconds")
    }
}
impl Error for HtmlTemporalPrecisionError {}

/// An exact HTML processing time, without a zone or a fractional precision cap.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlParsedTime {
    hour: u8,
    minute: u8,
    second: HtmlNonNegativeDecimal,
}

impl HtmlParsedTime {
    /// Checks clock components; exact seconds must be below 60, so leap seconds
    /// are rejected without projecting or rounding the decimal fraction.
    pub fn new(
        hour: u8,
        minute: u8,
        second: HtmlNonNegativeDecimal,
    ) -> Result<Self, HtmlTemporalError> {
        if hour > 23 {
            return Err(HtmlTemporalError::HourOutOfRange);
        }
        if minute > 59 {
            return Err(HtmlTemporalError::MinuteOutOfRange);
        }
        if !second.integer_part().to_u64().is_ok_and(|value| value < 60) {
            return Err(HtmlTemporalError::SecondOutOfRange);
        }
        Ok(Self {
            hour,
            minute,
            second,
        })
    }
    /// Returns the hour, from 0 through 23.
    pub fn hour(&self) -> u8 {
        self.hour
    }
    /// Returns the minute, from 0 through 59.
    pub fn minute(&self) -> u8 {
        self.minute
    }
    /// Returns exact decimal seconds, including every significant fraction digit.
    pub fn second(&self) -> &HtmlNonNegativeDecimal {
        &self.second
    }
}

fn exact_millisecond(value: &HtmlNonNegativeDecimal) -> Result<u16, HtmlTemporalPrecisionError> {
    let Some(digits) = value.fractional_digits() else {
        return Ok(0);
    };
    if digits.len() > 3 {
        return Err(HtmlTemporalPrecisionError);
    }
    let mut millisecond = digits
        .bytes()
        .fold(0, |value, digit| value * 10 + u16::from(digit - b'0'));
    for _ in digits.len()..3 {
        millisecond *= 10;
    }
    Ok(millisecond)
}

fn authored_decimal(integer: HtmlNonNegativeInteger, millisecond: u16) -> HtmlNonNegativeDecimal {
    if millisecond == 0 {
        return integer.into();
    }
    HtmlNonNegativeDecimal::with_fraction_digits(integer, &format!("{millisecond:03}").into())
        .expect("validated milliseconds produce ASCII fraction digits")
}

impl From<HtmlTime> for HtmlParsedTime {
    fn from(value: HtmlTime) -> Self {
        Self {
            hour: value.hour,
            minute: value.minute,
            second: authored_decimal(u128::from(value.second).into(), value.millisecond),
        }
    }
}

impl TryFrom<HtmlParsedTime> for HtmlTime {
    type Error = HtmlTemporalPrecisionError;
    fn try_from(value: HtmlParsedTime) -> Result<Self, Self::Error> {
        Ok(Self {
            hour: value.hour,
            minute: value.minute,
            second: value
                .second
                .integer_part()
                .to_u64()
                .expect("parsed time seconds are below 60") as u8,
            millisecond: exact_millisecond(&value.second)?,
        })
    }
}

/// A positive-year date and exact processing time, without an inferred zone.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlParsedLocalDateTime {
    date: HtmlDate,
    time: HtmlParsedTime,
}

impl HtmlParsedLocalDateTime {
    /// Couples intrinsically valid components without guessing a time zone.
    pub fn new(date: HtmlDate, time: HtmlParsedTime) -> Self {
        Self { date, time }
    }
    /// Returns the positive-year Gregorian date.
    pub fn date(&self) -> &HtmlDate {
        &self.date
    }
    /// Returns the exact time.
    pub fn time(&self) -> &HtmlParsedTime {
        &self.time
    }
}

/// Units selected by HTML's temporal input string-to-number algorithms.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlInputTemporalCoordinateUnit {
    /// Whole calendar months, relative to January 1970.
    Months,
    /// Exact milliseconds in the state's source-defined coordinate system.
    Milliseconds,
}

/// An exact native result of one of HTML's five temporal input conversions.
///
/// The state qualifies the epoch, unit and step metadata. These conversions do
/// not calculate input constraints, current validity, inverse string conversion,
/// IDL setters, or stepUp/stepDown. No host time zone or clock is consulted.
/// Native arithmetic uses ordinary allocations without cancellation or
/// allocator-failure recovery; raw admission remains the interpreter's task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInputTemporalCoordinate {
    state: HtmlInputType,
    value: HtmlSignedDecimal,
}

impl HtmlInputTemporalCoordinate {
    /// Actual source fallback without requiring successful content conversion.
    pub(crate) fn default_step_base_for(state: HtmlInputType) -> Option<Self> {
        let (_, _, _, base) = Self::definition_for(state)?;
        Some(Self {
            state,
            value: HtmlInteger::from(base).into(),
        })
    }

    /// Returns the input state whose conversion produced this quantity.
    pub fn state(&self) -> HtmlInputType {
        self.state
    }

    /// Returns the state's coordinate unit.
    pub fn unit(&self) -> HtmlInputTemporalCoordinateUnit {
        if self.state == HtmlInputType::Month {
            HtmlInputTemporalCoordinateUnit::Months
        } else {
            HtmlInputTemporalCoordinateUnit::Milliseconds
        }
    }

    /// Borrows the exact signed quantity, retaining every significant digit.
    pub fn value(&self) -> &HtmlSignedDecimal {
        &self.value
    }

    /// Returns the pinned clause defining this state's string-to-number result.
    pub fn source_clause(&self) -> &'static str {
        self.definition().0
    }

    /// Returns the integer multiplier from authored steps to coordinate units.
    pub fn step_scale_factor(&self) -> u32 {
        self.definition().1
    }

    /// Returns the default step before application of the scale factor.
    pub fn default_step(&self) -> u32 {
        self.definition().2
    }

    /// Returns the state's default step base, qualified with the same state.
    /// This does not select a base from min or value content attributes.
    pub fn default_step_base(&self) -> Self {
        Self {
            state: self.state,
            value: HtmlInteger::from(self.definition().3).into(),
        }
    }

    /// Reports HTML's periodic domain designation; only Time has this property.
    pub fn has_periodic_domain(&self) -> bool {
        self.state == HtmlInputType::Time
    }

    /// Returns the exact scaled default step in coordinate units.
    pub fn default_allowed_step(&self) -> HtmlNonNegativeDecimal {
        HtmlNonNegativeInteger::from(u128::from(self.default_step()))
            .multiplied_by(self.step_scale_factor())
            .into()
    }

    /// Scales an already recovered positive finite binary64 step exactly.
    ///
    /// The actual binary64 bits are changed to a terminating decimal before
    /// multiplication. No shortened decimal reparse, second rounding, or
    /// binary64 product overflow is introduced. This native operation does not
    /// select `any`, parse raw step syntax, or choose a default on failure.
    pub fn scale_step(
        &self,
        step: HtmlFloatingPointNumber,
    ) -> Result<HtmlNonNegativeDecimal, HtmlInputTemporalStepError> {
        if step.value() <= 0.0 {
            return Err(HtmlInputTemporalStepError);
        }
        Ok(step
            .exact_decimal()
            .magnitude()
            .multiplied_by(self.step_scale_factor()))
    }

    fn definition(&self) -> (&'static str, u32, u32, i128) {
        Self::definition_for(self.state).expect("only temporal producers construct coordinates")
    }

    fn definition_for(state: HtmlInputType) -> Option<(&'static str, u32, u32, i128)> {
        macro_rules! source {
            ($anchor:literal) => { concat!("https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#", $anchor) };
        }
        Some(match state {
            HtmlInputType::Date => (
                source!("date-state-(type=date):concept-input-value-string-number"),
                86400000,
                1,
                0,
            ),
            HtmlInputType::Month => (
                source!("month-state-(type=month):concept-input-value-string-number"),
                1,
                1,
                0,
            ),
            HtmlInputType::Week => (
                source!("week-state-(type=week):concept-input-value-string-number"),
                604800000,
                1,
                -259200000,
            ),
            HtmlInputType::Time => (
                source!("time-state-(type=time):concept-input-value-string-number"),
                1000,
                60,
                0,
            ),
            HtmlInputType::LocalDateTime => (
                source!(
                    "local-date-and-time-state-(type=datetime-local):concept-input-value-string-number"
                ),
                1000,
                60,
                0,
            ),
            _ => return None,
        })
    }
}

/// Native step scaling requires an already recovered quantity above zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputTemporalStepError;

impl fmt::Display for HtmlInputTemporalStepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("temporal input step must be positive")
    }
}
impl Error for HtmlInputTemporalStepError {}

impl HtmlDate {
    /// Returns signed milliseconds from 1970-01-01 UTC midnight to this date's
    /// UTC midnight, ignoring leap seconds as specified by the Date state.
    pub fn input_coordinate(&self) -> HtmlInputTemporalCoordinate {
        HtmlInputTemporalCoordinate {
            state: HtmlInputType::Date,
            value: epoch_milliseconds(gregorian_ordinal(self)),
        }
    }
}

impl HtmlMonth {
    /// Returns signed whole months from January 1970, without a year cap.
    pub fn input_coordinate(&self) -> HtmlInputTemporalCoordinate {
        let preceding = self
            .year
            .value
            .decremented()
            .expect("HTML years are positive");
        let ordinal = preceding
            .multiplied_by(12)
            .added_to(&u128::from(self.month - 1).into());
        HtmlInputTemporalCoordinate {
            state: HtmlInputType::Month,
            value: HtmlSignedDecimal::integer_difference(ordinal, 23628_u128.into()),
        }
    }
}

impl HtmlWeek {
    /// Returns signed milliseconds to Monday of this ISO week from 1970-01-01
    /// UTC midnight, ignoring leap seconds. Week 1 contains January 4.
    pub fn input_coordinate(&self) -> HtmlInputTemporalCoordinate {
        let january_four = HtmlDate::new(self.year.clone(), 1, 4).expect("January 4 exists");
        let ordinal = gregorian_ordinal(&january_four);
        let monday_offset = ordinal.remainder(NonZeroU32::new(7).expect("7 is nonzero"));
        // January 4's ordinal is at least three. Its containing Monday cannot
        // precede ordinal zero because year 1 begins on Monday.
        let week_one =
            HtmlSignedDecimal::integer_difference(ordinal, u128::from(monday_offset).into());
        debug_assert!(!week_one.is_negative());
        let monday = week_one
            .magnitude()
            .integer_part()
            .added_to(&u128::from(u32::from(self.week - 1) * 7).into());
        HtmlInputTemporalCoordinate {
            state: HtmlInputType::Week,
            value: epoch_milliseconds(monday),
        }
    }
}

impl HtmlParsedTime {
    /// Returns exact milliseconds since midnight on a day with no time changes.
    pub fn input_coordinate(&self) -> HtmlInputTemporalCoordinate {
        let seconds: HtmlNonNegativeDecimal = HtmlNonNegativeInteger::from(
            u128::from(self.hour) * 3600 + u128::from(self.minute) * 60,
        )
        .into();
        HtmlInputTemporalCoordinate {
            state: HtmlInputType::Time,
            value: HtmlSignedDecimal::from_magnitude(
                seconds.added_to(&self.second).multiplied_by(1000),
                false,
            ),
        }
    }
}

impl HtmlParsedLocalDateTime {
    /// Returns exact signed milliseconds from local 1970-01-01 midnight,
    /// without applying a time zone, DST or leap seconds.
    pub fn input_coordinate(&self) -> HtmlInputTemporalCoordinate {
        let date = self.date.input_coordinate();
        let time = self.time.input_coordinate();
        HtmlInputTemporalCoordinate {
            state: HtmlInputType::LocalDateTime,
            value: date.value.added_to(&time.value),
        }
    }
}

impl From<HtmlLocalDateTime> for HtmlParsedLocalDateTime {
    fn from(value: HtmlLocalDateTime) -> Self {
        Self::new(value.date, value.time.into())
    }
}
impl TryFrom<HtmlParsedLocalDateTime> for HtmlLocalDateTime {
    type Error = HtmlTemporalPrecisionError;
    fn try_from(value: HtmlParsedLocalDateTime) -> Result<Self, Self::Error> {
        Ok(Self::new(value.date, value.time.try_into()?))
    }
}

/// An exact UTC instant encoded as a signed day and a time within that day.
///
/// Day zero is UTC midnight beginning Gregorian 0001-01-01. This Rust coordinate
/// is not an HTML year-zero wire spelling. Explicit offset normalization can
/// produce day -1 even though the supplied HTML date must have a positive year.
/// Historical UT1 interpretation follows the source; no clock conversion is
/// inferred from this numerical representation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlUtcInstant {
    day: HtmlInteger,
    time: HtmlParsedTime,
}

impl HtmlUtcInstant {
    /// Converts positive-year local components and an explicit offset exactly.
    /// The result's signed day is at least -1; there is no upper magnitude cap.
    pub fn from_local_and_offset(
        local: &HtmlParsedLocalDateTime,
        offset: HtmlTimeZoneOffset,
    ) -> Self {
        let magnitude = gregorian_ordinal(&local.date);
        let shifted = i32::from(local.time.hour) * 60 + i32::from(local.time.minute)
            - i32::from(offset.minutes);
        let day = match shifted.div_euclid(1440) {
            -1 => magnitude.decremented().map_or_else(
                || HtmlInteger::from(-1),
                |magnitude| HtmlInteger::from_magnitude(magnitude, false),
            ),
            0 => HtmlInteger::from_magnitude(magnitude, false),
            1 => HtmlInteger::from_magnitude(magnitude.added_to(&1_u128.into()), false),
            _ => unreachable!("validated HTML offsets carry at most one day"),
        };
        let minute = shifted.rem_euclid(1440);
        Self {
            day,
            time: HtmlParsedTime {
                hour: (minute / 60) as u8,
                minute: (minute % 60) as u8,
                second: local.time.second.clone(),
            },
        }
    }
    /// Returns exact signed days from the documented year-1 origin.
    pub fn day(&self) -> &HtmlInteger {
        &self.day
    }
    /// Returns the exact UTC time within that day.
    pub fn time(&self) -> &HtmlParsedTime {
        &self.time
    }
}

/// Exact global processing result with UTC instant and retained display offset.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlParsedGlobalDateTime {
    local: HtmlParsedLocalDateTime,
    offset: HtmlTimeZoneOffset,
    utc: HtmlUtcInstant,
}

impl HtmlParsedGlobalDateTime {
    /// Computes UTC once from intrinsically valid components and explicit offset.
    pub fn new(local: HtmlParsedLocalDateTime, offset: HtmlTimeZoneOffset) -> Self {
        let utc = HtmlUtcInstant::from_local_and_offset(&local, offset);
        Self { local, offset, utc }
    }
    /// Returns supplied local components, without host-zone substitution.
    pub fn local(&self) -> &HtmlParsedLocalDateTime {
        &self.local
    }
    /// Returns the explicit minutes east of UTC.
    pub fn offset(&self) -> HtmlTimeZoneOffset {
        self.offset
    }
    /// Returns the exact normalized UTC instant. Compare this value for instant
    /// equality; the global result also retains offset/local display information.
    pub fn utc(&self) -> &HtmlUtcInstant {
        &self.utc
    }
}
impl From<HtmlGlobalDateTime> for HtmlParsedGlobalDateTime {
    fn from(value: HtmlGlobalDateTime) -> Self {
        Self::new(value.local.into(), value.offset)
    }
}
impl TryFrom<HtmlParsedGlobalDateTime> for HtmlGlobalDateTime {
    type Error = HtmlTemporalPrecisionError;
    fn try_from(value: HtmlParsedGlobalDateTime) -> Result<Self, Self::Error> {
        Ok(Self::new(value.local.try_into()?, value.offset))
    }
}

/// An exact nonnegative processing duration, with no fractional precision cap.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlParsedDuration {
    seconds: HtmlNonNegativeDecimal,
}
impl HtmlParsedDuration {
    /// Constructs a fixed duration from an intrinsically valid exact decimal.
    pub fn new(seconds: HtmlNonNegativeDecimal) -> Self {
        Self { seconds }
    }
    /// Returns the exact number of seconds.
    pub fn seconds(&self) -> &HtmlNonNegativeDecimal {
        &self.seconds
    }
}
impl From<HtmlDuration> for HtmlParsedDuration {
    fn from(value: HtmlDuration) -> Self {
        Self::new(authored_decimal(value.seconds, value.millisecond))
    }
}
impl TryFrom<HtmlParsedDuration> for HtmlDuration {
    type Error = HtmlTemporalPrecisionError;
    fn try_from(value: HtmlParsedDuration) -> Result<Self, Self::Error> {
        let millisecond = exact_millisecond(&value.seconds)?;
        Ok(Self {
            seconds: value.seconds.integer_part().clone(),
            millisecond,
        })
    }
}

struct Cursor<'a> {
    input: &'a [u16],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a DocumentString) -> Self {
        Self {
            input: input.code_units(),
            position: 0,
        }
    }
    fn peek(&self) -> Option<u16> {
        self.input.get(self.position).copied()
    }
    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(u16::from(byte)) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn digits(&mut self) -> &'a [u16] {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|unit| (u16::from(b'0')..=u16::from(b'9')).contains(&unit))
        {
            self.position += 1;
        }
        &self.input[start..self.position]
    }
    fn pair(&mut self) -> Result<u8, HtmlTemporalError> {
        let digits = self.digits();
        if digits.len() != 2 {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        Ok(((digits[0] - u16::from(b'0')) * 10 + digits[1] - u16::from(b'0')) as u8)
    }
    fn expect(&mut self, byte: u8) -> Result<(), HtmlTemporalError> {
        if self.take(byte) {
            Ok(())
        } else {
            Err(HtmlTemporalError::InvalidSyntax)
        }
    }
    fn end(&self) -> Result<(), HtmlTemporalError> {
        if self.position == self.input.len() {
            Ok(())
        } else {
            Err(HtmlTemporalError::InvalidSyntax)
        }
    }
    fn whitespace(&mut self) {
        while self
            .peek()
            .is_some_and(|unit| matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20))
        {
            self.position += 1;
        }
    }
    fn quantity(&mut self) -> Result<HtmlNonNegativeInteger, HtmlTemporalError> {
        let digits = self.digits();
        if digits.is_empty() {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(digits.to_vec()))
            .map_err(|_| HtmlTemporalError::InvalidSyntax)
    }
    fn fraction(&mut self) -> Result<u16, HtmlTemporalError> {
        if !self.take(b'.') {
            return Ok(0);
        }
        let digits = self.digits();
        if !(1..=3).contains(&digits.len()) {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        let mut value = digits
            .iter()
            .fold(0, |value, digit| value * 10 + digit - u16::from(b'0'));
        for _ in digits.len()..3 {
            value *= 10;
        }
        Ok(value)
    }
    fn year(&mut self) -> Result<HtmlYear, HtmlTemporalError> {
        let digits = self.digits();
        if digits.len() < 4 {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        let value = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(
            digits.to_vec(),
        ))
        .map_err(|_| HtmlTemporalError::InvalidYearDigits)?;
        HtmlYear::from_number(value)
    }
    fn month(&mut self) -> Result<HtmlMonth, HtmlTemporalError> {
        let year = self.year()?;
        self.expect(b'-')?;
        HtmlMonth::new(year, self.pair()?)
    }
    fn date(&mut self) -> Result<HtmlDate, HtmlTemporalError> {
        let month = self.month()?;
        self.expect(b'-')?;
        HtmlDate::new(month.year, month.month, self.pair()?)
    }
    fn time(&mut self) -> Result<HtmlTime, HtmlTemporalError> {
        let hour = self.pair()?;
        self.expect(b':')?;
        let minute = self.pair()?;
        let mut second = 0;
        let mut millisecond = 0;
        if self.take(b':') {
            second = self.pair()?;
            millisecond = self.fraction()?;
        }
        HtmlTime::new(hour, minute, second, millisecond)
    }
    fn local(&mut self) -> Result<HtmlLocalDateTime, HtmlTemporalError> {
        let date = self.date()?;
        if !self.take(b'T') && !self.take(b' ') {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        Ok(HtmlLocalDateTime::new(date, self.time()?))
    }
    fn offset(
        &mut self,
        reject_negative_zero: bool,
    ) -> Result<HtmlTimeZoneOffset, HtmlTemporalError> {
        if self.take(b'Z') {
            return HtmlTimeZoneOffset::new(0);
        }
        let negative = self.take(b'-');
        if !negative && !self.take(b'+') {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        let digits = self.digits();
        let (hour, minute) = match digits {
            [a, b] => {
                self.expect(b':')?;
                (
                    ((a - u16::from(b'0')) * 10 + b - u16::from(b'0')) as u8,
                    self.pair()?,
                )
            }
            [a, b, c, d] => (
                ((a - u16::from(b'0')) * 10 + b - u16::from(b'0')) as u8,
                ((c - u16::from(b'0')) * 10 + d - u16::from(b'0')) as u8,
            ),
            _ => return Err(HtmlTemporalError::InvalidSyntax),
        };
        if hour > 23 || minute > 59 {
            return Err(HtmlTemporalError::OffsetOutOfRange);
        }
        let minutes = i16::from(hour) * 60 + i16::from(minute);
        if reject_negative_zero && negative && minutes == 0 {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        HtmlTimeZoneOffset::new(if negative { -minutes } else { minutes })
    }

    // §2.3.5.4 collects the entire digits/dots run for seconds. Its accepted
    // fractions are unbounded, unlike the separate authored time grammar.
    fn parsed_time(&mut self) -> Result<HtmlParsedTime, HtmlTemporalError> {
        let hour = self.pair()?;
        self.expect(b':')?;
        let minute = self.pair()?;
        let second = if self.take(b':') {
            let start = self.position;
            while self
                .peek()
                .is_some_and(|unit| unit == 0x2e || (0x30..=0x39).contains(&unit))
            {
                self.position += 1;
            }
            let seconds = &self.input[start..self.position];
            if seconds.len() < 2
                || seconds[..2]
                    .iter()
                    .any(|unit| !(0x30..=0x39).contains(unit))
                || seconds.len() == 3
                || (seconds.len() > 3 && seconds[2] != 0x2e)
                || seconds.iter().filter(|&&unit| unit == 0x2e).count() > 1
            {
                return Err(HtmlTemporalError::InvalidSyntax);
            }
            let integer = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(
                seconds[..2].to_vec(),
            ))
            .map_err(|_| HtmlTemporalError::InvalidSyntax)?;
            if seconds.len() == 2 {
                integer.into()
            } else {
                HtmlNonNegativeDecimal::with_fraction_digits(
                    integer,
                    &DocumentString::from_utf16(seconds[3..].to_vec()),
                )
                .map_err(|_| HtmlTemporalError::InvalidSyntax)?
            }
        } else {
            HtmlNonNegativeInteger::from(0_u128).into()
        };
        HtmlParsedTime::new(hour, minute, second)
    }

    fn parsed_local(&mut self) -> Result<HtmlParsedLocalDateTime, HtmlTemporalError> {
        let date = self.date()?;
        if !self.take(b'T') && !self.take(b' ') {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        Ok(HtmlParsedLocalDateTime::new(date, self.parsed_time()?))
    }
}

pub(crate) fn validate_month(input: &DocumentString) -> Result<HtmlMonth, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.month()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn validate_date(input: &DocumentString) -> Result<HtmlDate, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.date()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn validate_yearless_date(
    input: &DocumentString,
) -> Result<HtmlYearlessDate, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    if cursor.take(b'-') {
        cursor.expect(b'-')?;
    }
    let month = cursor.pair()?;
    cursor.expect(b'-')?;
    let day = cursor.pair()?;
    cursor.end()?;
    HtmlYearlessDate::new(month, day)
}
pub(crate) fn validate_time(input: &DocumentString) -> Result<HtmlTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.time()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn validate_local_date_time(
    input: &DocumentString,
) -> Result<HtmlLocalDateTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.local()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn validate_normalized_local_date_time(
    input: &DocumentString,
) -> Result<HtmlLocalDateTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let date = cursor.date()?;
    cursor.expect(b'T')?;
    let time_start = cursor.position;
    let time = cursor.time()?;
    cursor.end()?;
    if !cursor.input[time_start..]
        .iter()
        .copied()
        .eq(time.spelling().encode_utf16())
    {
        return Err(HtmlTemporalError::InvalidSyntax);
    }
    Ok(HtmlLocalDateTime::new(date, time))
}
pub(crate) fn validate_time_zone_offset(
    input: &DocumentString,
) -> Result<HtmlTimeZoneOffset, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.offset(true)?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn validate_global_date_time(
    input: &DocumentString,
) -> Result<HtmlGlobalDateTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let local = cursor.local()?;
    let offset = cursor.offset(true)?;
    cursor.end()?;
    Ok(HtmlGlobalDateTime::new(local, offset))
}
pub(crate) fn validate_week(input: &DocumentString) -> Result<HtmlWeek, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let year = cursor.year()?;
    cursor.expect(b'-')?;
    cursor.expect(b'W')?;
    let week = cursor.pair()?;
    cursor.end()?;
    HtmlWeek::new(year, week)
}
// The month/date/yearless/week processing algorithms have the same value
// domains and full-consumption checks as their authored grammars (§2.3.5).
pub(crate) fn parse_month(input: &DocumentString) -> Result<HtmlMonth, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.month()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn parse_date(input: &DocumentString) -> Result<HtmlDate, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.date()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn parse_yearless_date(
    input: &DocumentString,
) -> Result<HtmlYearlessDate, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    if cursor.take(b'-') {
        cursor.expect(b'-')?;
    }
    let month = cursor.pair()?;
    cursor.expect(b'-')?;
    let day = cursor.pair()?;
    cursor.end()?;
    HtmlYearlessDate::new(month, day)
}
pub(crate) fn parse_week(input: &DocumentString) -> Result<HtmlWeek, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let year = cursor.year()?;
    cursor.expect(b'-')?;
    cursor.expect(b'W')?;
    let week = cursor.pair()?;
    cursor.end()?;
    HtmlWeek::new(year, week)
}
pub(crate) fn parse_time(input: &DocumentString) -> Result<HtmlParsedTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.parsed_time()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn parse_local_date_time(
    input: &DocumentString,
) -> Result<HtmlParsedLocalDateTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.parsed_local()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn parse_time_zone_offset(
    input: &DocumentString,
) -> Result<HtmlTimeZoneOffset, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = cursor.offset(false)?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn parse_global_date_time(
    input: &DocumentString,
) -> Result<HtmlParsedGlobalDateTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let local = cursor.parsed_local()?;
    let offset = cursor.offset(false)?;
    cursor.end()?;
    Ok(HtmlParsedGlobalDateTime::new(local, offset))
}
pub(crate) fn parse_date_or_time(
    input: &DocumentString,
) -> Result<HtmlDateOrTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let value = if let Ok(date) = cursor.date() {
        if cursor.take(b'T') || cursor.take(b' ') {
            let time = cursor.parsed_time()?;
            let offset = cursor.offset(false)?;
            HtmlDateOrTime::Global(HtmlParsedGlobalDateTime::new(
                HtmlParsedLocalDateTime::new(date, time),
                offset,
            ))
        } else {
            HtmlDateOrTime::Date(date)
        }
    } else {
        // The source resets the cursor after *any* failed date-component attempt.
        cursor.position = 0;
        HtmlDateOrTime::Time(cursor.parsed_time()?)
    };
    cursor.end()?;
    Ok(value)
}

pub(crate) fn parse_duration(
    input: &DocumentString,
) -> Result<HtmlParsedDuration, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let mut seconds = HtmlNonNegativeDecimal::from(HtmlNonNegativeInteger::from(0_u128));
    let mut months = false;
    let mut nonzero_calendar = false;
    let mut component_present = false;
    cursor.whitespace();
    if cursor.take(b'P') {
        months = true;
        cursor.whitespace();
    }
    while cursor.peek().is_some() {
        if cursor.take(b'T') {
            months = false;
            cursor.whitespace();
            continue;
        }
        let integer = if cursor.peek() == Some(0x2e) {
            0_u128.into()
        } else {
            cursor.quantity()?
        };
        let next = cursor.peek().ok_or(HtmlTemporalError::InvalidSyntax)?;
        cursor.position += 1;
        if next == 0x2e {
            let digits = cursor.digits();
            let quantity = HtmlNonNegativeDecimal::with_fraction_digits(
                integer,
                &DocumentString::from_utf16(digits.to_vec()),
            )
            .map_err(|_| HtmlTemporalError::InvalidSyntax)?;
            cursor.whitespace();
            if !cursor.take(b'S') && !cursor.take(b's') {
                return Err(HtmlTemporalError::InvalidSyntax);
            }
            seconds = seconds.added_to(&quantity);
            // Fractional S does not reset M-disambiguator (§2.3.5.9 step 8.9).
        } else {
            let unit = if matches!(next, 0x09 | 0x0a | 0x0c | 0x0d | 0x20) {
                cursor.whitespace();
                let unit = cursor.peek().ok_or(HtmlTemporalError::InvalidSyntax)?;
                cursor.position += 1;
                unit
            } else {
                next
            };
            let scale = match unit {
                0x59 | 0x79 => {
                    months = true;
                    None
                }
                0x4d | 0x6d if months => None,
                0x57 | 0x77 => {
                    months = false;
                    Some(604800)
                }
                0x44 | 0x64 => {
                    months = false;
                    Some(86400)
                }
                0x48 | 0x68 => {
                    months = false;
                    Some(3600)
                }
                0x4d | 0x6d => Some(60),
                0x53 | 0x73 => {
                    months = false;
                    Some(1)
                }
                _ => return Err(HtmlTemporalError::InvalidSyntax),
            };
            if let Some(scale) = scale {
                seconds =
                    seconds.added_to(&HtmlNonNegativeDecimal::from(integer).multiplied_by(scale));
            } else {
                nonzero_calendar |= !integer.is_zero();
            }
        }
        component_present = true;
        cursor.whitespace();
    }
    if !component_present {
        return Err(HtmlTemporalError::InvalidSyntax);
    }
    // Calendar quantities are nonnegative, so a nonzero component implies a
    // nonzero sum without allocating an otherwise unusable months magnitude.
    if nonzero_calendar {
        return Err(HtmlTemporalError::NonzeroCalendarDuration);
    }
    Ok(HtmlParsedDuration::new(seconds))
}

pub(crate) fn validate_date_with_optional_time(
    input: &DocumentString,
) -> Result<HtmlDateWithOptionalTime, HtmlTemporalError> {
    if let Ok(value) = validate_date(input) {
        return Ok(HtmlDateWithOptionalTime::Date(value));
    }
    validate_global_date_time(input).map(HtmlDateWithOptionalTime::Global)
}

pub(crate) fn validate_duration(input: &DocumentString) -> Result<HtmlDuration, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let mut seconds = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from("0"))
        .map_err(|_| HtmlTemporalError::InvalidSyntax)?;
    let mut millisecond = 0;
    let mut count = 0;
    if cursor.take(b'P') {
        // Valid ISO-form grammar: D, then optional T with H, M, S in order.
        let mut in_time = false;
        let mut previous_rank = 0;
        let mut time_count = 0;
        while cursor.peek().is_some() {
            if cursor.take(b'T') {
                if in_time {
                    return Err(HtmlTemporalError::InvalidSyntax);
                }
                in_time = true;
                continue;
            }
            let quantity = cursor.quantity()?;
            let has_fraction = cursor.peek() == Some(u16::from(b'.'));
            let fraction = cursor.fraction()?;
            let (rank, scale) = if !in_time && cursor.take(b'D') {
                (1, 86400)
            } else if in_time && cursor.take(b'H') {
                (2, 3600)
            } else if in_time && cursor.take(b'M') {
                (3, 60)
            } else if in_time && cursor.take(b'S') {
                (4, 1)
            } else {
                return Err(HtmlTemporalError::InvalidSyntax);
            };
            if rank <= previous_rank || (has_fraction && scale != 1) {
                return Err(HtmlTemporalError::InvalidSyntax);
            }
            seconds = seconds.added_to(&quantity.multiplied_by(scale));
            millisecond = fraction;
            count += 1;
            if in_time {
                time_count += 1;
            }
            previous_rank = rank;
        }
        if count == 0 || (in_time && time_count == 0) {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
    } else {
        // Human form permits any order but each scale occurs only once.
        let mut seen = 0_u8;
        cursor.whitespace();
        while cursor.peek().is_some() {
            let quantity = cursor.quantity()?;
            let has_fraction = cursor.peek() == Some(u16::from(b'.'));
            let fraction = cursor.fraction()?;
            cursor.whitespace();
            let (bit, scale) = match cursor.peek() {
                Some(0x57 | 0x77) => (1, 604800),
                Some(0x44 | 0x64) => (2, 86400),
                Some(0x48 | 0x68) => (4, 3600),
                Some(0x4d | 0x6d) => (8, 60),
                Some(0x53 | 0x73) => (16, 1),
                _ => return Err(HtmlTemporalError::InvalidSyntax),
            };
            cursor.position += 1;
            if seen & bit != 0 || (has_fraction && scale != 1) {
                return Err(HtmlTemporalError::InvalidSyntax);
            }
            seen |= bit;
            seconds = seconds.added_to(&quantity.multiplied_by(scale));
            if scale == 1 {
                millisecond = fraction;
            }
            count += 1;
            cursor.whitespace();
        }
        if count == 0 {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
    }
    cursor.end()?;
    HtmlDuration::new(seconds, millisecond)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_default_base_needs_no_successful_content_coordinate() {
        for (state, base, unit, step) in [
            (
                HtmlInputType::Date,
                "0",
                HtmlInputTemporalCoordinateUnit::Milliseconds,
                "86400000",
            ),
            (
                HtmlInputType::Month,
                "0",
                HtmlInputTemporalCoordinateUnit::Months,
                "1",
            ),
            (
                HtmlInputType::Week,
                "-259200000",
                HtmlInputTemporalCoordinateUnit::Milliseconds,
                "604800000",
            ),
            (
                HtmlInputType::Time,
                "0",
                HtmlInputTemporalCoordinateUnit::Milliseconds,
                "60000",
            ),
            (
                HtmlInputType::LocalDateTime,
                "0",
                HtmlInputTemporalCoordinateUnit::Milliseconds,
                "60000",
            ),
        ] {
            let value = HtmlInputTemporalCoordinate::default_step_base_for(state).unwrap();
            assert_eq!(value.state(), state);
            assert_eq!(
                value.value().as_document_string(),
                DocumentString::from(base)
            );
            assert_eq!(value.unit(), unit);
            assert_eq!(
                value.default_allowed_step().as_document_string(),
                DocumentString::from(step)
            );
        }
        for state in [
            HtmlInputType::Number,
            HtmlInputType::Range,
            HtmlInputType::Text,
        ] {
            assert_eq!(
                HtmlInputTemporalCoordinate::default_step_base_for(state),
                None
            );
        }
    }

    #[test]
    fn shared_ordinal_uses_the_gregorian_origin_and_complete_cycles() {
        for (year, month, day, expected) in [
            (1, 1, 1, "0"),
            (401, 1, 1, "146097"),
            (1970, 1, 1, "719162"),
            (2001, 1, 1, "730485"),
        ] {
            let date = HtmlDate::new(HtmlYear::new(year).unwrap(), month, day).unwrap();
            assert_eq!(gregorian_ordinal(&date).decimal_digits(), expected);
        }
    }

    fn text(value: &str) -> DocumentString {
        value.into()
    }
    fn spelling(value: DocumentString) -> String {
        value.to_utf8().unwrap()
    }
    fn year(value: u64) -> HtmlYear {
        HtmlYear::new(value).unwrap()
    }

    #[test]
    fn gregorian_centuries_and_unbounded_years_check_real_month_lengths() {
        assert_eq!(
            HtmlDate::new(year(1900), 2, 29),
            Err(HtmlTemporalError::DayOutOfRange)
        );
        assert_eq!(
            HtmlDate::new(year(2100), 2, 29),
            Err(HtmlTemporalError::DayOutOfRange)
        );
        assert!(HtmlDate::new(year(2000), 2, 29).is_ok());
        assert!(HtmlDate::new(year(2004), 2, 29).is_ok());
        assert_eq!(
            HtmlDate::new(year(2000), 4, 31),
            Err(HtmlTemporalError::DayOutOfRange)
        );
        assert_eq!(
            HtmlDate::new(year(2000), 1, 0),
            Err(HtmlTemporalError::DayOutOfRange)
        );
        let huge = "1000000000000000000000000000000400";
        let date = validate_date(&text(&format!("{huge}-02-29"))).unwrap();
        assert_eq!(date.year().decimal_digits(), huge);
        assert!(date.year().is_leap_year());
        assert_eq!(spelling(date.to_document_string()), format!("{huge}-02-29"));
    }

    #[test]
    fn native_components_reject_invalid_states_and_emit_valid_strings() {
        assert_eq!(HtmlYear::new(0), Err(HtmlTemporalError::ZeroYear));
        assert_eq!(
            HtmlYear::from_decimal_digits("+1"),
            Err(HtmlTemporalError::InvalidYearDigits)
        );
        assert_eq!(
            HtmlYear::from_decimal_digits(""),
            Err(HtmlTemporalError::InvalidYearDigits)
        );
        assert_eq!(
            HtmlMonth::new(year(1), 13),
            Err(HtmlTemporalError::MonthOutOfRange)
        );
        assert_eq!(
            spelling(HtmlMonth::new(year(1), 1).unwrap().to_document_string()),
            "0001-01"
        );
        assert_eq!(
            HtmlYearlessDate::new(0, 1),
            Err(HtmlTemporalError::MonthOutOfRange)
        );
        assert_eq!(
            HtmlYearlessDate::new(4, 31),
            Err(HtmlTemporalError::DayOutOfRange)
        );
        assert_eq!(
            spelling(HtmlYearlessDate::new(2, 29).unwrap().to_document_string()),
            "02-29"
        );
        assert_eq!(
            HtmlTime::new(24, 0, 0, 0),
            Err(HtmlTemporalError::HourOutOfRange)
        );
        assert_eq!(
            HtmlTime::new(0, 60, 0, 0),
            Err(HtmlTemporalError::MinuteOutOfRange)
        );
        assert_eq!(
            HtmlTime::new(0, 0, 60, 0),
            Err(HtmlTemporalError::SecondOutOfRange)
        );
        assert_eq!(
            HtmlTime::new(0, 0, 0, 1000),
            Err(HtmlTemporalError::MillisecondOutOfRange)
        );
        assert_eq!(
            HtmlTimeZoneOffset::new(i16::MIN),
            Err(HtmlTemporalError::OffsetOutOfRange)
        );
        let seconds =
            HtmlNonNegativeInteger::from_decimal_digits(&text("123456789012345678901234567890"))
                .unwrap();
        assert_eq!(
            HtmlDuration::new(seconds.clone(), 1000),
            Err(HtmlTemporalError::MillisecondOutOfRange)
        );
        let duration = HtmlDuration::new(seconds, 1).unwrap();
        assert_eq!(
            spelling(duration.to_document_string()),
            "PT123456789012345678901234567890.001S"
        );
    }

    #[test]
    fn full_temporal_grammars_reject_non_ascii_and_partial_inputs_without_loss() {
        for invalid in [
            "2000-02-29x",
            " 2000-02-29",
            "2000-02-29 ",
            "0000-01-01",
            "999-01-01",
            "2000-2-29",
            "2000-02-029",
            "２０００-02-29",
        ] {
            assert!(validate_date(&text(invalid)).is_err(), "{invalid}");
        }
        let mut units: Vec<_> = "2000-02-29".encode_utf16().collect();
        units.push(0xd800);
        let input = DocumentString::from_utf16(units.clone());
        assert!(validate_date(&input).is_err());
        assert_eq!(input.code_units(), units);
        assert_eq!(validate_month(&text("2000-01")).unwrap().month(), 1);
        assert!(validate_month(&text("2000-01-01")).is_err());
        assert_eq!(
            validate_yearless_date(&text("--02-29")).unwrap(),
            HtmlYearlessDate::new(2, 29).unwrap()
        );
        assert_eq!(
            validate_yearless_date(&text("02-29")).unwrap(),
            HtmlYearlessDate::new(2, 29).unwrap()
        );
        for invalid in ["-02-29", "---02-29", "02-30", "02-29Z"] {
            assert!(validate_yearless_date(&text(invalid)).is_err());
        }
    }

    #[test]
    fn valid_time_precision_and_shortest_normalization_are_exact() {
        for (input, expected, milliseconds) in [
            ("12:34:00.000", "12:34", 0),
            ("12:34:01.100", "12:34:01.1", 100),
            ("12:34:01.010", "12:34:01.01", 10),
            ("12:34:01.001", "12:34:01.001", 1),
            ("23:59:59.999", "23:59:59.999", 999),
        ] {
            let value = validate_time(&text(input)).unwrap();
            assert_eq!(value.millisecond(), milliseconds);
            assert_eq!(spelling(value.to_document_string()), expected);
        }
        for invalid in [
            "23:59:60",
            "23:59:61",
            "24:00",
            "12:34:00.",
            "12:34:00.0000",
            "12:34:0",
            "12:34.5",
            "12:34:000",
            "12:34Z",
            "12:34:01..2",
        ] {
            assert!(validate_time(&text(invalid)).is_err(), "{invalid}");
        }
        let raw = text("00002000-02-29 12:34:00.000");
        let value = validate_local_date_time(&raw).unwrap();
        assert_eq!(spelling(value.to_document_string()), "2000-02-29T12:34");
        assert_eq!(raw, text("00002000-02-29 12:34:00.000"));
        assert!(validate_local_date_time(&text("2000-02-29t12:34")).is_err());
        assert!(validate_local_date_time(&text("2000-02-29T12:34Z")).is_err());
        assert!(validate_normalized_local_date_time(&text("00002000-02-29T12:34")).is_ok());
        assert!(validate_normalized_local_date_time(&text("2000-02-29 12:34")).is_err());
        assert!(validate_normalized_local_date_time(&text("2000-02-29T12:34:00")).is_err());
        assert!(validate_normalized_local_date_time(&text("2000-02-29T12:34:01.010")).is_err());
        assert!(validate_normalized_local_date_time(&text("2000-02-29T12:34:01.01")).is_ok());
    }

    #[test]
    fn offsets_keep_explicit_minutes_and_valid_zero_spelling() {
        for (input, minutes, expected) in [
            ("Z", 0, "Z"),
            ("+0000", 0, "Z"),
            ("+23:59", 1439, "+23:59"),
            ("-2359", -1439, "-23:59"),
            ("-00:30", -30, "-00:30"),
        ] {
            let value = validate_time_zone_offset(&text(input)).unwrap();
            assert_eq!(value.minutes(), minutes);
            assert_eq!(spelling(value.to_document_string()), expected);
        }
        for invalid in [
            "-0000", "-00:00", "+24:00", "+00:60", "+0:00", "+00000", "z", "Z ",
        ] {
            assert!(
                validate_time_zone_offset(&text(invalid)).is_err(),
                "{invalid}"
            );
        }
        let value = validate_global_date_time(&text("0001-01-01 00:00+23:59")).unwrap();
        assert_eq!(value.offset().minutes(), 1439);
        assert_eq!(value.local().date().year().decimal_digits(), "1");
        assert_eq!(
            spelling(value.to_document_string()),
            "0001-01-01T00:00+23:59"
        );
        assert!(validate_global_date_time(&text("2000-01-01T00:00")).is_err());
    }

    #[test]
    fn iso_week_year_has_53_only_for_thursday_or_leap_wednesday() {
        // 2015 and 2026 start Thursday; leap 2020 starts Wednesday. 2021
        // starts Friday, and leap 2000 starts Saturday.
        for value in [2015, 2020, 2026] {
            assert!(HtmlWeek::new(year(value), 53).is_ok());
        }
        for value in [1900, 2000, 2021] {
            assert_eq!(
                HtmlWeek::new(year(value), 53),
                Err(HtmlTemporalError::WeekOutOfRange)
            );
        }
        assert_eq!(
            HtmlWeek::new(year(2020), 0),
            Err(HtmlTemporalError::WeekOutOfRange)
        );
        assert_eq!(
            HtmlWeek::new(year(2020), 54),
            Err(HtmlTemporalError::WeekOutOfRange)
        );
        let huge = validate_week(&text("10000000000000000000000000002020-W53")).unwrap();
        assert_eq!(huge.week(), 53);
        assert_eq!(
            spelling(huge.to_document_string()),
            "10000000000000000000000000002020-W53"
        );
        for invalid in ["2020-w53", "2020-W053", "2021-W53", "2020-W53x"] {
            assert!(validate_week(&text(invalid)).is_err());
        }
    }

    #[test]
    fn duration_grammars_sum_exact_seconds_and_reject_invalid_components() {
        // One day + two hours + three minutes + 4.05 seconds = 93784.05.
        for input in [
            "P1DT2H3M4.050S",
            "4.050s 3m 2h 1d",
            "\t1 D\n2 H\r3 M\x0c4.050 S ",
        ] {
            let value = validate_duration(&text(input)).unwrap();
            assert_eq!(value.seconds().decimal_digits(), "93784");
            assert_eq!(value.millisecond(), 50);
            assert_eq!(spelling(value.to_document_string()), "PT93784.05S");
        }
        let big = validate_duration(&text("1000000000000000000000000000000w")).unwrap();
        assert_eq!(
            big.seconds().decimal_digits(),
            "604800000000000000000000000000000000"
        );
        assert_eq!(
            spelling(
                validate_duration(&text("PT0S"))
                    .unwrap()
                    .to_document_string()
            ),
            "PT0S"
        );
        for invalid in [
            "",
            "P",
            "PT",
            "P1DT",
            "P1W",
            "P1Y",
            "P0Y",
            "P1M",
            "PT1M1H",
            "PT1H1H",
            "1h1H",
            "P1.0D",
            "PT1.000H",
            "1.0h",
            "PT.5S",
            "PT1.0000S",
            "1s junk",
            "P 1D",
            "p1d",
            "1\u{a0}s",
            "1s\0",
        ] {
            assert!(validate_duration(&text(invalid)).is_err(), "{invalid}");
        }
    }

    #[test]
    fn vaguer_forms_require_offsets_when_a_date_and_time_are_both_present() {
        assert!(matches!(
            parse_date_or_time(&text("2000-01-01")),
            Ok(HtmlDateOrTime::Date(_))
        ));
        assert!(matches!(
            parse_date_or_time(&text("00:00")),
            Ok(HtmlDateOrTime::Time(_))
        ));
        assert!(matches!(
            parse_date_or_time(&text("2000-01-01T00:00Z")),
            Ok(HtmlDateOrTime::Global(_))
        ));
        assert!(parse_date_or_time(&text("2000-01-01T00:00")).is_err());
        assert!(validate_date_with_optional_time(&text("00:00")).is_err());
        assert!(matches!(
            validate_date_with_optional_time(&text("2000-01-01")),
            Ok(HtmlDateWithOptionalTime::Date(_))
        ));
        assert!(matches!(
            validate_date_with_optional_time(&text("2000-01-01T00:00Z")),
            Ok(HtmlDateWithOptionalTime::Global(_))
        ));
    }

    #[test]
    fn processing_time_consumes_whole_fraction_runs_without_rounding() {
        for (raw, expected) in [
            ("12:34:59.000000000001", "59.000000000001"),
            ("12:34:00.0000", "0"),
            ("12:34:01.100000", "1.1"),
        ] {
            let parsed = parse_time(&text(raw)).unwrap();
            assert_eq!(spelling(parsed.second().as_document_string()), expected);
            assert!(validate_time(&text(raw)).is_err());
        }
        for raw in [
            "12:34:60.0000",
            "12:34:001",
            "12:34:00.",
            "12:34:00..1",
            "12:34:00.1.2",
            "12:34:00.1Z",
            "12:34:00.1 ",
        ] {
            assert!(parse_time(&text(raw)).is_err(), "{raw}");
        }
        let raw = DocumentString::from_utf16(vec![0x31, 0x32, 0x3a, 0x33, 0x34, 0xd800]);
        assert!(parse_time(&raw).is_err());
        assert_eq!(raw.code_units(), &[0x31, 0x32, 0x3a, 0x33, 0x34, 0xd800]);
    }

    #[test]
    fn processing_offsets_admit_negative_zero_and_preserve_strict_boundary() {
        for raw in ["-0000", "-00:00"] {
            assert_eq!(parse_time_zone_offset(&text(raw)).unwrap().minutes(), 0);
            assert!(validate_time_zone_offset(&text(raw)).is_err());
            assert!(parse_global_date_time(&text(&format!("0001-01-01T00:00{raw}"))).is_ok());
        }
        for raw in ["+00000", "+00:000", "-24:00", "Zx", "z"] {
            assert!(parse_time_zone_offset(&text(raw)).is_err());
        }
    }

    #[test]
    fn processing_duration_follows_marker_state_and_fractional_asymmetry() {
        for (raw, expected) in [
            ("P1S1M", "61"),
            ("PT1.5S1M", "61.5"),
            ("P0Y0M", "0"),
            ("P1DT", "86400"),
            ("TT1ST", "1"),
            ("1h1H", "7200"),
            (".5s .0001s", "0.5001"),
            ("P1D1M", "86460"),
            ("\tP\nT\r.000000000001\x0cS ", "0.000000000001"),
            (
                "9999999999999999999999999999999999999999s1s",
                "10000000000000000000000000000000000000000",
            ),
        ] {
            assert_eq!(
                spelling(
                    parse_duration(&text(raw))
                        .unwrap()
                        .seconds()
                        .as_document_string()
                ),
                expected,
                "{raw}"
            );
        }
        for raw in ["P1.5S1M", "P1Y", "P1M", "1y1m"] {
            assert_eq!(
                parse_duration(&text(raw)),
                Err(HtmlTemporalError::NonzeroCalendarDuration),
                "{raw}"
            );
        }
        for raw in [
            "PT", "", "P", ".0Y", "P0Yjunk", "P1Yjunk", "1\u{a0}s", "1.s", "1s\0", "p1s", "1t",
        ] {
            assert_eq!(
                parse_duration(&text(raw)),
                Err(HtmlTemporalError::InvalidSyntax),
                "{raw}"
            );
        }
    }

    #[test]
    fn processing_date_or_time_resets_failed_date_and_observes_full_consumption() {
        assert!(matches!(
            parse_date_or_time(&text("12:34:00.00000001")),
            Ok(HtmlDateOrTime::Time(_))
        ));
        assert!(matches!(
            parse_date_or_time(&text("2000-02-29T12:34:00.00000001-0000")),
            Ok(HtmlDateOrTime::Global(_))
        ));
        for raw in [
            "2000-01-01 ",
            "2000-01-01Z",
            "2000-01-01T12:34",
            "12:34Z",
            "2000-02-30",
            "2000-01-01T12:34Zx",
        ] {
            assert!(parse_date_or_time(&text(raw)).is_err(), "{raw}");
        }
    }
}
