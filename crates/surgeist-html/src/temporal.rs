//! Intrinsically valid HTML temporal values from the pinned
//! [HTML foundations §2.3.5](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dates-and-times).
//!
//! Lexical recognition below checks the valid-string grammars. In particular,
//! it does not substitute the more permissive duration/time parsing algorithms
//! for author-conformance validation. No host clock or time zone is consulted.

use crate::{DocumentString, numeric::HtmlNonNegativeInteger};
use std::{error::Error, fmt, num::NonZeroU32};

/// A rejected temporal component or valid-string grammar.
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
    /// A second is outside 0 through 59; leap seconds are not representable.
    SecondOutOfRange,
    /// A millisecond is outside 0 through 999.
    MillisecondOutOfRange,
    /// An offset is outside minus 23:59 through plus 23:59.
    OffsetOutOfRange,
    /// A week does not exist in the selected ISO week-year.
    WeekOutOfRange,
    /// The entire input does not satisfy the selected valid-string grammar.
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
            Self::SecondOutOfRange => "second must be between 0 and 59",
            Self::MillisecondOutOfRange => "millisecond must be between 0 and 999",
            Self::OffsetOutOfRange => "time-zone offset exceeds 23 hours and 59 minutes",
            Self::WeekOutOfRange => "week does not exist in this week-year",
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

/// The date, time, and global date/time alternatives of §2.3.5.10.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HtmlDateOrTime {
    /// A date without a time or offset.
    Date(HtmlDate),
    /// A time without a date or offset.
    Time(HtmlTime),
    /// A date and time with an explicit offset.
    Global(HtmlGlobalDateTime),
}

impl HtmlDateOrTime {
    /// Produces the appropriate valid HTML temporal string.
    pub fn to_document_string(&self) -> DocumentString {
        match self {
            Self::Date(value) => value.to_document_string(),
            Self::Time(value) => value.to_document_string(),
            Self::Global(value) => value.to_document_string(),
        }
    }
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
    fn offset(&mut self) -> Result<HtmlTimeZoneOffset, HtmlTemporalError> {
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
        if negative && minutes == 0 {
            return Err(HtmlTemporalError::InvalidSyntax);
        }
        HtmlTimeZoneOffset::new(if negative { -minutes } else { minutes })
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
    let value = cursor.offset()?;
    cursor.end()?;
    Ok(value)
}
pub(crate) fn validate_global_date_time(
    input: &DocumentString,
) -> Result<HtmlGlobalDateTime, HtmlTemporalError> {
    let mut cursor = Cursor::new(input);
    let local = cursor.local()?;
    let offset = cursor.offset()?;
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
pub(crate) fn validate_date_or_time(
    input: &DocumentString,
) -> Result<HtmlDateOrTime, HtmlTemporalError> {
    if let Ok(value) = validate_date(input) {
        return Ok(HtmlDateOrTime::Date(value));
    }
    if let Ok(value) = validate_global_date_time(input) {
        return Ok(HtmlDateOrTime::Global(value));
    }
    validate_time(input).map(HtmlDateOrTime::Time)
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
            validate_date_or_time(&text("2000-01-01")),
            Ok(HtmlDateOrTime::Date(_))
        ));
        assert!(matches!(
            validate_date_or_time(&text("00:00")),
            Ok(HtmlDateOrTime::Time(_))
        ));
        assert!(matches!(
            validate_date_or_time(&text("2000-01-01T00:00Z")),
            Ok(HtmlDateOrTime::Global(_))
        ));
        assert!(validate_date_or_time(&text("2000-01-01T00:00")).is_err());
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
}
