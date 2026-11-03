//! Intrinsically valid HTML temporal values and the input coordinate conversion.

pub use crate::internal::microsyntax::temporal::input_coordinate::{
    HtmlInputTemporalCoordinate, HtmlInputTemporalCoordinateUnit, HtmlInputTemporalStepError,
};
pub use crate::internal::microsyntax::temporal::{
    HtmlDate, HtmlDateOrTime, HtmlDateWithOptionalTime, HtmlDuration, HtmlGlobalDateTime,
    HtmlLocalDateTime, HtmlMachineReadableTime, HtmlMonth, HtmlParsedDuration,
    HtmlParsedGlobalDateTime, HtmlParsedLocalDateTime, HtmlParsedTime, HtmlTemporalError,
    HtmlTemporalPrecisionError, HtmlTime, HtmlTimeElementValue, HtmlTimeZoneOffset, HtmlUtcInstant,
    HtmlWeek, HtmlYear, HtmlYearlessDate,
};
