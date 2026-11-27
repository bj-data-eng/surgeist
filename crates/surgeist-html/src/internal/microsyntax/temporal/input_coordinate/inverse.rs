//! Inverse temporal Input coordinates over exact integral magnitudes.

use super::{
    DAYS_BEFORE_EPOCH, DAYS_IN_WEEK, HtmlDate, HtmlMonth, HtmlNonNegativeInteger,
    HtmlSignedDecimal, HtmlWeek, MILLISECONDS_PER_DAY, MONTHS_BEFORE_EPOCH, first_iso_monday,
    monday_of,
};
use crate::internal::microsyntax::temporal::{
    GREGORIAN_CYCLE, HtmlTime, HtmlYear, days_before_year_in_cycle, days_in_month,
};
use std::num::NonZeroU32;

const DAYS_IN_CYCLE: NonZeroU32 = NonZeroU32::MIN.saturating_add(146_096);
const MONTHS_PER_YEAR: NonZeroU32 = NonZeroU32::MIN.saturating_add(11);

/// Splits an integral timestamp using a Euclidean day and milliseconds-of-day.
/// The clock remainder is available even before the first positive HTML year.
pub(crate) fn epoch_day_and_time(
    value: &HtmlSignedDecimal,
) -> (Option<HtmlNonNegativeInteger>, u32) {
    let (magnitude, remainder) = value
        .magnitude()
        .integer_part()
        .quotient_remainder(MILLISECONDS_PER_DAY);
    let origin = HtmlNonNegativeInteger::from(u128::from(DAYS_BEFORE_EPOCH));
    if !value.is_negative() {
        return (Some(origin.added_to(&magnitude)), remainder);
    }
    let (elapsed, time) = if remainder == 0 {
        (magnitude, 0)
    } else {
        (
            magnitude.added_to(&1_u128.into()),
            MILLISECONDS_PER_DAY.get() - remainder,
        )
    };
    (positive_origin_difference(origin, elapsed), time)
}

/// Interprets an integral signed number of months relative to January 1970.
pub(crate) fn epoch_month(value: &HtmlSignedDecimal) -> Option<HtmlMonth> {
    let origin = HtmlNonNegativeInteger::from(u128::from(MONTHS_BEFORE_EPOCH));
    let magnitude = value.magnitude().integer_part();
    let month = if value.is_negative() {
        positive_origin_difference(origin, magnitude.clone())?
    } else {
        origin.added_to(magnitude)
    };
    let (years, remainder) = month.quotient_remainder(MONTHS_PER_YEAR);
    let year = HtmlYear::from_number(years.added_to(&1_u128.into())).ok()?;
    HtmlMonth::new(year, u8::try_from(remainder + 1).ok()?).ok()
}

fn positive_origin_difference(
    origin: HtmlNonNegativeInteger,
    elapsed: HtmlNonNegativeInteger,
) -> Option<HtmlNonNegativeInteger> {
    let difference = HtmlSignedDecimal::integer_difference(origin, elapsed);
    (!difference.is_negative()).then(|| difference.magnitude().integer_part().clone())
}

/// Recovers a civil day from the zero-based ordinal of 0001-01-01.
pub(crate) fn date_at_day(ordinal: &HtmlNonNegativeInteger) -> Option<HtmlDate> {
    let (cycles, day) = ordinal.quotient_remainder(DAYS_IN_CYCLE);
    let cycle_years = GREGORIAN_CYCLE.get();
    // Leap days add fewer than 365 days in one cycle, so the division can
    // overestimate by at most one year. Clamp the last day to its own cycle.
    let estimate = (day / 365).min(cycle_years - 1);
    let preceding = estimate - u32::from(days_before_year_in_cycle(estimate) > day);
    let years = cycles
        .multiplied_by(cycle_years)
        .added_to(&u128::from(preceding + 1).into());
    let year = HtmlYear::from_number(years).ok()?;
    let leap = year.is_leap_year();
    let mut day_in_year = day - days_before_year_in_cycle(preceding);
    for month in 1..=12 {
        let length = u32::from(days_in_month(month, leap));
        if day_in_year < length {
            return HtmlDate::new(year, month, u8::try_from(day_in_year + 1).ok()?).ok();
        }
        day_in_year -= length;
    }
    None
}

/// Selects the week-year by the Thursday of the containing ISO week.
pub(crate) fn week_at_day(ordinal: HtmlNonNegativeInteger) -> Option<HtmlWeek> {
    let monday = monday_of(ordinal);
    let monday = monday.magnitude().integer_part();
    let thursday = date_at_day(&monday.added_to(&3_u128.into()))?;
    let year = thursday.year().clone();
    let first = first_iso_monday(&year);
    let offset =
        positive_origin_difference(monday.clone(), first.magnitude().integer_part().clone())?;
    let weeks = offset.quotient_remainder(DAYS_IN_WEEK).0.to_u64().ok()?;
    HtmlWeek::new(year, u8::try_from(weeks.checked_add(1)?).ok()?).ok()
}

/// Constructs an author time from whole milliseconds within one day.
pub(crate) fn time_at_millisecond(value: u32) -> Option<HtmlTime> {
    let seconds = value / 1_000;
    HtmlTime::new(
        u8::try_from(seconds / 3_600).ok()?,
        u8::try_from((seconds / 60) % 60).ok()?,
        u8::try_from(seconds % 60).ok()?,
        u16::try_from(value % 1_000).ok()?,
    )
    .ok()
}
