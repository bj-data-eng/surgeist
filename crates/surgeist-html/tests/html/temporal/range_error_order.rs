//! Typed temporal failures follow the pinned parse step order (HTML foundations
//! 2.3.5): each range check precedes the later steps and the trailing-input
//! check. Expectations come from the step numbers cited at each group, not from
//! current output.
use surgeist_html::{
    microsyntax::temporal::HtmlTemporalError,
    values::{
        DocumentString, HtmlValueAuthorState, HtmlValueFailure, HtmlValueInterpreter,
        HtmlValueRecoveryState, HtmlValueSyntaxError,
    },
};

fn author_error<T: std::fmt::Debug>(state: &HtmlValueAuthorState<T>) -> Option<HtmlTemporalError> {
    match state {
        HtmlValueAuthorState::Valid(_) => None,
        HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::Temporal(error)) => Some(*error),
        other => panic!("unexpected author state {other:?}"),
    }
}

fn recovery_error<T: std::fmt::Debug>(
    state: &HtmlValueRecoveryState<T>,
) -> Option<HtmlTemporalError> {
    match state {
        HtmlValueRecoveryState::Value(_) => None,
        HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(
            HtmlValueSyntaxError::Temporal(error),
        )) => Some(*error),
        other => panic!("unexpected recovery state {other:?}"),
    }
}

fn raw(text: &str) -> DocumentString {
    DocumentString::from(text)
}

fn interpreter(text: &DocumentString) -> HtmlValueInterpreter {
    HtmlValueInterpreter::new(text.len())
}

#[test]
fn time_ranges_precede_later_time_steps() {
    use HtmlTemporalError::*;
    // Parse a time component: hour range (step 2) before the colon, minute
    // digits and seconds scan; minute range (step 5) before the seconds scan;
    // seconds grammar (step 7.3) before the seconds range (step 7.4). Author
    // syntax also limits the fraction to three digits.
    for (text, author, recovery) in [
        ("24:00:00..1", Some(HourOutOfRange), Some(HourOutOfRange)),
        ("24:00x", Some(HourOutOfRange), Some(HourOutOfRange)),
        ("24", Some(HourOutOfRange), Some(HourOutOfRange)),
        (
            "23:60:00..1",
            Some(MinuteOutOfRange),
            Some(MinuteOutOfRange),
        ),
        ("23:59:60..1", Some(InvalidSyntax), Some(InvalidSyntax)),
        ("23:59:60.1234", Some(InvalidSyntax), Some(SecondOutOfRange)),
        ("23:59:60", Some(SecondOutOfRange), Some(SecondOutOfRange)),
        ("2x:00", Some(InvalidSyntax), Some(InvalidSyntax)),
        ("23:59:58.5", None, None),
    ] {
        let value = raw(text);
        let report = interpreter(&value).time(Some(&value)).unwrap();
        assert_eq!(author_error(report.author()), author, "author {text}");
        assert_eq!(
            recovery_error(report.recovery()),
            recovery,
            "recovery {text}"
        );
    }
}

#[test]
fn local_date_time_reports_the_first_failing_component_range() {
    use HtmlTemporalError::*;
    // Parse a local date and time string: date component (step 3, ending with
    // its day range) before the separator (4) and the time component (5), whose
    // own ranges precede the trailing-input check (6).
    for (text, expected) in [
        ("2023-02-30T24:00", Some(DayOutOfRange)),
        ("2023-02-28T24:00:XX", Some(HourOutOfRange)),
        ("2023-02-28 23:60:00..1", Some(MinuteOutOfRange)),
        ("2023-02-28T23:59:60x", Some(SecondOutOfRange)),
        ("2023-02-28T23:59:59x", Some(InvalidSyntax)),
        ("2023-02-28T23:59", None),
    ] {
        let value = raw(text);
        let report = interpreter(&value).local_date_time(Some(&value)).unwrap();
        assert_eq!(author_error(report.author()), expected, "author {text}");
        assert_eq!(
            recovery_error(report.recovery()),
            expected,
            "recovery {text}"
        );
    }
}

#[test]
fn week_range_precedes_the_trailing_input_check() {
    use HtmlTemporalError::*;
    // Parse a week string: week range (step 9) before trailing input (step 10).
    for (text, expected) in [
        ("2021-W53X", Some(WeekOutOfRange)),
        ("2020-W00x", Some(WeekOutOfRange)),
        ("2020-W53X", Some(InvalidSyntax)),
        ("2020-W5", Some(InvalidSyntax)),
        ("2020-W53", None),
    ] {
        let value = raw(text);
        let report = interpreter(&value).week(Some(&value)).unwrap();
        assert_eq!(author_error(report.author()), expected, "author {text}");
        assert_eq!(
            recovery_error(report.recovery()),
            expected,
            "recovery {text}"
        );
    }
}

#[test]
fn yearless_date_ranges_precede_the_hyphen_day_and_trailing_input() {
    use HtmlTemporalError::*;
    // Parse a yearless date component: month range (step 3) before the hyphen
    // (5) and day digits (6); day range (step 7) before trailing input (string
    // step 4).
    for (text, expected) in [
        ("13-01x", Some(MonthOutOfRange)),
        ("13", Some(MonthOutOfRange)),
        ("--00-xx", Some(MonthOutOfRange)),
        ("02-30x", Some(DayOutOfRange)),
        ("02-29x", Some(InvalidSyntax)),
        ("12-3", Some(InvalidSyntax)),
        ("--02-29", None),
    ] {
        let value = raw(text);
        let report = interpreter(&value).yearless_date(Some(&value)).unwrap();
        assert_eq!(author_error(report.author()), expected, "author {text}");
        assert_eq!(
            recovery_error(report.recovery()),
            expected,
            "recovery {text}"
        );
    }
}
