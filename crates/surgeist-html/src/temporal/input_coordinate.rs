//! Exact temporal input coordinates and their source-defined units, epochs,
//! steps, defaults, and periodic domains. Temporal grammars and shared calendar
//! arithmetic remain with the parent temporal owner.

use super::{
    HtmlDate, HtmlMonth, HtmlParsedLocalDateTime, HtmlParsedTime, HtmlWeek, gregorian_ordinal,
};
use crate::{
    HtmlInputType,
    numeric::{
        HtmlFloatingPointNumber, HtmlInteger, HtmlNonNegativeDecimal, HtmlNonNegativeInteger,
        HtmlSignedDecimal,
    },
};
use std::{error::Error, fmt, num::NonZeroU32};

fn epoch_milliseconds(ordinal: HtmlNonNegativeInteger) -> HtmlSignedDecimal {
    HtmlSignedDecimal::integer_difference(ordinal, 719162_u128.into()).multiplied_by(86400000)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentString;

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
}
