//! Literal outcomes from HTML 877d §4.10.5 and the Date time-value boundary.

use std::convert::Infallible;
use surgeist_html::{
    attributes::HtmlInputType as Input,
    bounded::input::{
        HtmlInputDateMilliseconds, HtmlInputDateRead, HtmlInputValueAssignment,
        HtmlInputValueConversionError, input_value_as_date, input_value_as_number,
        input_value_from_date, input_value_from_number,
    },
    values::DocumentString,
};

fn funded(_: usize) -> Result<(), Infallible> {
    Ok(())
}

fn number(state: Input, value: &str) -> Option<f64> {
    input_value_as_number(state, &DocumentString::from(value), funded)
        .unwrap()
        .map(|number| number.value())
}

fn date(state: Input, value: &str) -> HtmlInputDateRead {
    input_value_as_date(state, &DocumentString::from(value), funded).unwrap()
}

fn time(value: i64) -> HtmlInputDateRead {
    HtmlInputDateRead::Time(HtmlInputDateMilliseconds::new(value).unwrap())
}

fn text(value: &str) -> HtmlInputValueAssignment {
    HtmlInputValueAssignment::Text(DocumentString::from(value))
}

#[test]
fn number_conversion_selects_the_actual_state_and_projects_exact_coordinates_once() {
    for (state, input, expected) in [
        (Input::Date, "0001-01-01", -62_135_596_800_000.0),
        (Input::Date, "2000-02-29", 951_782_400_000.0),
        (Input::Month, "1969-12", -1.0),
        (Input::Week, "1970-W01", -259_200_000.0),
        (Input::Week, "2020-W53", 1_609_113_600_000.0),
        (Input::Time, "00:00:00.0019", 1.9),
        (Input::LocalDateTime, "1969-12-31 23:59:59.999", -1.0),
        // These exact millisecond coordinates are 2^53 + 1 and 2^53 + 3.
        (
            Input::LocalDateTime,
            "287396-10-12T08:59:00.993",
            9_007_199_254_740_992.0,
        ),
        (
            Input::LocalDateTime,
            "287396-10-12T08:59:00.995",
            9_007_199_254_740_996.0,
        ),
        // The recovery algorithm runs on the actual value. An earlier author
        // sanitization or Range clamping remains with the caller.
        (Input::Number, " +1.25e2tail", 125.0),
        (Input::Range, "1e", 1.0),
        (Input::Date, "275760-09-14", 8_640_000_086_400_000.0),
    ] {
        assert_eq!(number(state, input), Some(expected), "{state:?} {input}");
    }
    for (state, input) in [
        (Input::Text, "1"),
        (Input::Number, ""),
        (Input::Number, "infinity"),
        (Input::Number, "1e999"),
        (Input::Date, "1900-02-29"),
        (Input::Date, "0000-12-31"),
        (Input::Week, "2021-W53"),
        (Input::Time, "24:00"),
        (Input::LocalDateTime, "2000-01-01T00:00Z"),
    ] {
        assert_eq!(number(state, input), None, "{state:?} {input}");
    }
}

#[test]
fn date_read_distinguishes_invalid_syntax_from_a_successful_date_outside_timeclip() {
    for (state, input, expected) in [
        (Input::Date, "1969-12-31", time(-86_400_000)),
        (Input::Month, "1969-12", time(-2_678_400_000)),
        (Input::Week, "1970-W01", time(-259_200_000)),
        (Input::Time, "00:00:00.0019", time(1)),
        (
            Input::Time,
            "23:59:59.99999999999999999999",
            time(86_399_999),
        ),
        (Input::Date, "275760-09-13", time(8_640_000_000_000_000)),
        (Input::Date, "275760-09-14", HtmlInputDateRead::InvalidTime),
        (
            Input::Date,
            "400000000000000000000000-01-01",
            HtmlInputDateRead::InvalidTime,
        ),
        (Input::Date, "2024-02-30", HtmlInputDateRead::Null),
        (
            Input::LocalDateTime,
            "1970-01-01T00:00",
            HtmlInputDateRead::Null,
        ),
        (Input::Number, "1", HtmlInputDateRead::Null),
    ] {
        assert_eq!(date(state, input), expected, "{state:?} {input}");
    }
    assert!(HtmlInputDateMilliseconds::new(-8_640_000_000_000_000).is_some());
    assert!(HtmlInputDateMilliseconds::new(8_640_000_000_000_001).is_none());
    assert!(HtmlInputDateMilliseconds::new(-8_640_000_000_000_001).is_none());
    assert!(HtmlInputDateMilliseconds::new(i64::MIN).is_none());
}

#[test]
fn finite_number_assignment_selects_a_canonical_temporal_spelling() {
    // Exact binary64 values of the large WPT time fixture and 2^128.
    const LARGE_TIME: f64 = f64::from_bits(0x456c_45b8_974d_3727);
    const BEYOND_U128: f64 = f64::from_bits(0x47f0_0000_0000_0000);
    for (state, input, expected) in [
        (Input::Date, -0.25, "1969-12-31"),
        (Input::Date, -62_135_596_800_000.0, "0001-01-01"),
        (Input::Date, 951_782_400_000.0, "2000-02-29"),
        (Input::Date, 8_640_000_086_400_000.0, "275760-09-14"),
        (
            Input::Date,
            BEYOND_U128,
            "10783118943836478994022447719-04-03",
        ),
        (Input::Month, -23_628.0, "0001-01"),
        (Input::Month, -1.0, "1969-12"),
        (Input::Month, -0.5, "1969-12"),
        (Input::Month, 0.5, "1970-02"),
        (
            Input::Month,
            BEYOND_U128,
            "28356863910078205288614550619314019591-05",
        ),
        (Input::Week, -259_200_000.0, "1970-W01"),
        (Input::Week, -259_200_001.0, "1969-W52"),
        (Input::Week, 1_609_459_200_000.0, "2020-W53"),
        (Input::Time, -1.0, "23:59:59.999"),
        (Input::Time, 86_400_000.0, "00:00"),
        (Input::Time, LARGE_TIME, "10:54:10.944"),
        (Input::Time, f64::MAX, "22:14:18.368"),
        (Input::Time, -f64::MAX, "01:45:41.632"),
        (Input::Time, 0.5, "00:00:00.001"),
        (Input::LocalDateTime, -1.0, "1969-12-31T23:59:59.999"),
        (Input::LocalDateTime, 0.0, "1970-01-01T00:00"),
        (Input::Number, 2.5, "2.5"),
    ] {
        assert_eq!(
            input_value_from_number(state, input, funded),
            Ok(text(expected)),
            "{state:?} {input}"
        );
    }
    for state in [Input::Date, Input::Week, Input::LocalDateTime] {
        assert_eq!(
            input_value_from_number(state, -62_135_596_800_001.0, funded),
            Ok(HtmlInputValueAssignment::Empty)
        );
    }
    assert_eq!(
        input_value_from_number(Input::Month, -23_629.0, funded),
        Ok(HtmlInputValueAssignment::Empty)
    );
    let Ok(HtmlInputValueAssignment::RangeNumber(range)) =
        input_value_from_number(Input::Range, 1e21, funded)
    else {
        panic!("Range must request the existing best serialization");
    };
    assert_eq!(range.value(), 1e21);
}

#[test]
fn date_assignment_uses_utc_for_month_week_and_clock_components() {
    for (state, input, expected) in [
        (Input::Date, -1, "1969-12-31"),
        (Input::Month, -1, "1969-12"),
        (Input::Week, 1_609_459_200_000, "2020-W53"),
        (Input::Time, -1, "23:59:59.999"),
        (Input::Time, 86_401_005, "00:00:01.005"),
        (Input::Date, 8_640_000_000_000_000, "275760-09-13"),
    ] {
        assert_eq!(
            input_value_from_date(state, HtmlInputDateMilliseconds::new(input), funded),
            Ok(text(expected))
        );
    }
    assert_eq!(
        input_value_from_date(Input::Date, None, funded),
        Ok(HtmlInputValueAssignment::Empty)
    );
    assert_eq!(
        input_value_from_date(Input::LocalDateTime, None, funded),
        Err(HtmlInputValueConversionError::Inapplicable)
    );
}

#[test]
fn rejected_states_and_values_observe_the_source_order_before_admission() {
    fn unused(_: usize) -> Result<(), &'static str> {
        panic!("this route does not start a conversion");
    }
    assert_eq!(
        input_value_from_number(Input::Text, f64::INFINITY, unused),
        Err(HtmlInputValueConversionError::Infinite)
    );
    assert_eq!(
        input_value_from_number(Input::Text, f64::NAN, unused),
        Err(HtmlInputValueConversionError::Inapplicable)
    );
    assert_eq!(
        input_value_from_number(Input::Range, f64::NAN, unused),
        Ok(HtmlInputValueAssignment::Empty)
    );
    assert_eq!(
        input_value_as_number(Input::Text, &"1".into(), unused),
        Ok(None)
    );
    assert_eq!(
        input_value_as_date(Input::LocalDateTime, &"1970-01-01T00:00".into(), unused),
        Ok(HtmlInputDateRead::Null)
    );

    let refuse = |requested| {
        assert!(requested > 0);
        Err("original work refusal")
    };
    assert_eq!(
        input_value_as_number(Input::Week, &"2020-W53".into(), refuse),
        Err(HtmlInputValueConversionError::Admission(
            "original work refusal"
        ))
    );
    assert_eq!(
        input_value_as_date(Input::Month, &"2000-01".into(), refuse),
        Err(HtmlInputValueConversionError::Admission(
            "original work refusal"
        ))
    );
    assert_eq!(
        input_value_from_number(Input::Time, f64::MAX, refuse),
        Err(HtmlInputValueConversionError::Admission(
            "original work refusal"
        ))
    );
    assert_eq!(
        input_value_from_date(Input::Date, HtmlInputDateMilliseconds::new(0), refuse),
        Err(HtmlInputValueConversionError::Admission(
            "original work refusal"
        ))
    );
}
