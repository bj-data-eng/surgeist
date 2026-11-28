//! Exact, borrowed range comparison in the five temporal Input states.

use super::{
    BorrowedDate, BorrowedTemporalPlan, BorrowedTime, TemporalAlternative, TemporalPhase,
    input_coordinate::TemporalState, recognize_time_alternative, temporal_recognition_work_bound,
};
use crate::internal::{
    attributes::HtmlInputType,
    elements::{
        input_range_validity::HtmlInputRangeValidity,
        input_value_conversion::HtmlInputValueConversionError,
    },
};
use std::cmp::Ordering;

/// Checks temporal min/max validity using the exact recovered components.
/// `None` in the result means the current value is unavailable and a bound
/// could affect the answer. Input strings borrow the caller's backing.
/// # Errors
/// Returns inapplicability, an unrepresentable work bound, or the caller's refusal.
pub fn input_temporal_range_validity_with<E>(
    state: HtmlInputType,
    minimum: Option<&[u16]>,
    maximum: Option<&[u16]>,
    current: Option<&[u16]>,
    mut admit: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<HtmlInputRangeValidity>, HtmlInputValueConversionError<E>> {
    use HtmlInputValueConversionError as Error;
    let state = TemporalState::of(state).ok_or(Error::Inapplicable)?;
    let alternative = match state {
        TemporalState::Date => TemporalAlternative::Date,
        TemporalState::Month => TemporalAlternative::Month,
        TemporalState::Week => TemporalAlternative::Week,
        TemporalState::Time => TemporalAlternative::Time,
        TemporalState::LocalDateTime => TemporalAlternative::LocalDateTime,
    };
    let minimum = read_coordinate(minimum, alternative, &mut admit)?;
    let maximum = read_coordinate(maximum, alternative, &mut admit)?;
    if minimum.is_none() && maximum.is_none() {
        return Ok(Some(HtmlInputRangeValidity::default()));
    }
    let Some(raw) = current else {
        return Ok(None);
    };
    let Some(current) = read_coordinate(Some(raw), alternative, &mut admit)? else {
        return Ok(Some(HtmlInputRangeValidity::default()));
    };
    admit(1).map_err(Error::Admission)?;
    let reversed = matches!(state, TemporalState::Time)
        && minimum
            .zip(maximum)
            .is_some_and(|(lo, hi)| lo.compare(hi).is_gt());
    Ok(Some(HtmlInputRangeValidity::from_comparisons(
        minimum.map(|bound| current.compare(bound)),
        maximum.map(|bound| current.compare(bound)),
        reversed,
    )))
}

fn read_coordinate<'a, E>(
    raw: Option<&'a [u16]>,
    alternative: TemporalAlternative,
    admit: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Coordinate<'a>>, HtmlInputValueConversionError<E>> {
    use HtmlInputValueConversionError as Error;
    let Some(raw) = raw else {
        return Ok(None);
    };
    // Besides recognition, cover normalizing digits and each reached
    // comparison of a coordinate against the other operands.
    let work = temporal_recognition_work_bound(raw.len())
        .zip(raw.len().checked_mul(4))
        .and_then(|(scan, compare)| scan.checked_add(compare))
        .ok_or(Error::WorkOverflow)?;
    admit(work).map_err(Error::Admission)?;
    Ok(
        recognize_time_alternative(raw, TemporalPhase::Recovery, alternative)
            .ok()
            .and_then(Coordinate::from_plan),
    )
}

#[derive(Clone, Copy)]
struct Coordinate<'a> {
    year: &'a [u16],
    components: [u8; 5],
    fraction: &'a [u16],
}

impl<'a> Coordinate<'a> {
    fn from_plan(plan: BorrowedTemporalPlan<'a>) -> Option<Self> {
        // Within each state the recovered components have the same order as
        // its numeric coordinate. ISO week-years and weeks are monotone by
        // their Monday. No epoch conversion or binary64 rounding is needed.
        let (year, components, fraction) = match plan {
            BorrowedTemporalPlan::Month(month) => {
                (month.year.digits(), [month.month, 0, 0, 0, 0], &[][..])
            }
            BorrowedTemporalPlan::Week(week) => {
                (week.year.digits(), [week.week, 0, 0, 0, 0], &[][..])
            }
            BorrowedTemporalPlan::Date(date) => Self::date_time(Some(date), None),
            BorrowedTemporalPlan::Time(time) => Self::date_time(None, Some(time)),
            BorrowedTemporalPlan::LocalDateTime(local) => {
                Self::date_time(Some(local.date()), Some(local.time()))
            }
            _ => return None,
        };
        let zero = u16::from(b'0');
        let year = year
            .iter()
            .position(|&unit| unit != zero)
            .map_or(&[][..], |start| &year[start..]);
        let fraction = fraction
            .iter()
            .rposition(|&unit| unit != zero)
            .map_or(&[][..], |end| &fraction[..=end]);
        Some(Self {
            year,
            components,
            fraction,
        })
    }

    fn date_time(
        date: Option<BorrowedDate<'a>>,
        time: Option<BorrowedTime<'a>>,
    ) -> (&'a [u16], [u8; 5], &'a [u16]) {
        let (year, month, day) = date.map_or((&[][..], 0, 0), |date| {
            (date.year().digits(), date.month(), date.day)
        });
        let (hour, minute, second, fraction) = time.map_or((0, 0, 0, &[][..]), |time| {
            (time.hour, time.minute, time.second, time.fraction_digits())
        });
        (year, [month, day, hour, minute, second], fraction)
    }

    fn compare(self, other: Self) -> Ordering {
        self.year
            .len()
            .cmp(&other.year.len())
            .then_with(|| self.year.cmp(other.year))
            .then_with(|| self.components.cmp(&other.components))
            .then_with(|| self.fraction.cmp(other.fraction))
    }
}
