//! Literal coordinate expectations from HTML 877d's five temporal input states.
//! These native producers do not implement the raw input-constraints adapter.
use crate::html::support::year;
use surgeist_html::{
    DocumentString, HtmlDate, HtmlFloatingPointNumber, HtmlInputTemporalCoordinate,
    HtmlInputTemporalCoordinateUnit, HtmlInputTemporalStepError, HtmlInputType, HtmlInteger,
    HtmlMonth, HtmlNonNegativeDecimal, HtmlNonNegativeInteger, HtmlParsedLocalDateTime,
    HtmlParsedTime, HtmlSignedDecimal, HtmlTimeZoneOffset, HtmlUtcInstant, HtmlValueAuthorState,
    HtmlValueInterpreter, HtmlValueRecoveryState, HtmlWeek, HtmlYear,
};

fn date(y: u64, m: u8, d: u8) -> HtmlDate {
    HtmlDate::new(year(y), m, d).unwrap()
}
fn spelling(value: &HtmlInputTemporalCoordinate) -> String {
    value.value().as_document_string().to_utf8().unwrap()
}
fn seconds(integer: u128, fraction: &str) -> HtmlNonNegativeDecimal {
    HtmlNonNegativeDecimal::with_fraction_digits(integer.into(), &fraction.into()).unwrap()
}
fn time(h: u8, m: u8, s: u128, fraction: &str) -> HtmlParsedTime {
    HtmlParsedTime::new(h, m, seconds(s, fraction)).unwrap()
}

#[test]
fn dates_have_the_exact_epoch_and_signed_day_offsets() {
    for (y, m, d, expected) in [
        (1970, 1, 1, "0"),
        (1969, 12, 31, "-86400000"),
        (1970, 1, 2, "86400000"),
        (1, 1, 1, "-62135596800000"),
        (1900, 3, 1, "-2203891200000"),
        (2000, 2, 29, "951782400000"),
    ] {
        assert_eq!(spelling(&date(y, m, d).input_coordinate()), expected);
    }
}

#[test]
fn arbitrary_years_use_exact_four_hundred_year_cycles() {
    // Year = 400 * 10^40 + 1. Its ordinal is exactly 146097 * 10^40.
    let y = HtmlYear::from_decimal_digits("4000000000000000000000000000000000000000001").unwrap();
    assert_eq!(
        spelling(&HtmlDate::new(y.clone(), 1, 1).unwrap().input_coordinate()),
        "126227807999999999999999999999999999999937864403200000"
    );
    assert_eq!(
        spelling(&HtmlMonth::new(y.clone(), 1).unwrap().input_coordinate()),
        "47999999999999999999999999999999999999976372"
    );
    assert_eq!(
        spelling(&HtmlWeek::new(y, 1).unwrap().input_coordinate()),
        "126227807999999999999999999999999999999937864403200000"
    );
}

#[test]
fn month_coordinates_are_whole_months_not_day_durations() {
    for (y, m, expected) in [
        (1, 1, "-23628"),
        (1969, 12, "-1"),
        (1970, 1, "0"),
        (1970, 2, "1"),
        (2000, 2, "361"),
    ] {
        let value = HtmlMonth::new(year(y), m).unwrap().input_coordinate();
        assert_eq!(spelling(&value), expected);
        assert_eq!(value.unit(), HtmlInputTemporalCoordinateUnit::Months);
        assert_eq!(value.value().magnitude().fractional_digits(), None);
    }
}

#[test]
fn week_coordinates_use_monday_and_cross_gregorian_years() {
    for (y, w, expected) in [
        (1, 1, "-62135596800000"),
        (1970, 1, "-259200000"),
        (2020, 53, "1609113600000"),
        (2021, 1, "1609718400000"),
    ] {
        assert_eq!(
            spelling(&HtmlWeek::new(year(y), w).unwrap().input_coordinate()),
            expected
        );
    }
    assert!(HtmlWeek::new(year(2021), 53).is_err());
}

#[test]
fn processing_time_preserves_submillisecond_digits() {
    assert_eq!(
        spelling(&time(12, 34, 56, "123456789").input_coordinate()),
        "45296123.456789"
    );
    assert_eq!(
        spelling(&time(0, 0, 0, "000000001").input_coordinate()),
        "0.000001"
    );
    assert_eq!(
        spelling(&time(23, 59, 59, "999999999").input_coordinate()),
        "86399999.999999"
    );
}

#[test]
fn local_coordinates_borrow_across_the_epoch_without_negative_zero() {
    let before = HtmlParsedLocalDateTime::new(date(1969, 12, 31), time(23, 59, 59, "999999999"));
    assert_eq!(spelling(&before.input_coordinate()), "-0.000001");
    let epoch = HtmlParsedLocalDateTime::new(date(1970, 1, 1), time(0, 0, 0, "0"));
    assert_eq!(spelling(&epoch.input_coordinate()), "0");
    assert!(!epoch.input_coordinate().value().is_negative());
    let after = HtmlParsedLocalDateTime::new(date(1970, 1, 1), time(0, 0, 0, "000000001"));
    assert_eq!(spelling(&after.input_coordinate()), "0.000001");
}

#[test]
fn recovered_exact_time_is_not_projected_through_author_precision() {
    let raw: DocumentString = "12:34:56.123456789".into();
    let report = HtmlValueInterpreter::new(raw.len())
        .time(Some(&raw))
        .unwrap();
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    let HtmlValueRecoveryState::Value(value) = report.recovery() else {
        panic!("exact time must recover")
    };
    assert_eq!(spelling(&value.input_coordinate()), "45296123.456789");
    assert_eq!(report.raw(), Some(&raw));
}

#[test]
fn existing_local_grammar_owns_separator_and_complete_input() {
    for raw in ["1970-01-01T00:00", "1970-01-01 00:00"] {
        let raw: DocumentString = raw.into();
        let report = HtmlValueInterpreter::new(raw.len())
            .local_date_time(Some(&raw))
            .unwrap();
        let HtmlValueRecoveryState::Value(value) = report.recovery() else {
            panic!("local time must recover")
        };
        assert_eq!(spelling(&value.input_coordinate()), "0");
        assert!(matches!(report.author(), HtmlValueAuthorState::Valid(_)));
    }
    for raw in [
        "1970-01-01T00:00Z",
        "1970-01-01T00:00tail",
        "0000-01-01T00:00",
    ] {
        let raw: DocumentString = raw.into();
        let report = HtmlValueInterpreter::new(raw.len())
            .local_date_time(Some(&raw))
            .unwrap();
        assert!(!matches!(
            report.recovery(),
            HtmlValueRecoveryState::Value(_)
        ));
    }
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let report = HtmlValueInterpreter::new(1).date(Some(&raw)).unwrap();
    assert!(!matches!(
        report.recovery(),
        HtmlValueRecoveryState::Value(_)
    ));
}

#[test]
fn omitted_seconds_recover_an_integral_coordinate() {
    let raw: DocumentString = "12:34".into();
    let report = HtmlValueInterpreter::new(raw.len())
        .time(Some(&raw))
        .unwrap();
    let HtmlValueRecoveryState::Value(value) = report.recovery() else {
        panic!("omitted seconds must recover")
    };
    assert_eq!(spelling(&value.input_coordinate()), "45240000");
}

#[test]
fn all_five_states_qualify_exact_units_scales_defaults_and_bases() {
    let values = [
        date(1970, 1, 1).input_coordinate(),
        HtmlMonth::new(year(1970), 1).unwrap().input_coordinate(),
        HtmlWeek::new(year(1970), 1).unwrap().input_coordinate(),
        time(0, 0, 0, "0").input_coordinate(),
        HtmlParsedLocalDateTime::new(date(1970, 1, 1), time(0, 0, 0, "0")).input_coordinate(),
    ];
    let expected = [
        (
            HtmlInputType::Date,
            86400000,
            1,
            "86400000",
            "0",
            false,
            "date-state-(type=date)",
        ),
        (
            HtmlInputType::Month,
            1,
            1,
            "1",
            "0",
            false,
            "month-state-(type=month)",
        ),
        (
            HtmlInputType::Week,
            604800000,
            1,
            "604800000",
            "-259200000",
            false,
            "week-state-(type=week)",
        ),
        (
            HtmlInputType::Time,
            1000,
            60,
            "60000",
            "0",
            true,
            "time-state-(type=time)",
        ),
        (
            HtmlInputType::LocalDateTime,
            1000,
            60,
            "60000",
            "0",
            false,
            "local-date-and-time-state-(type=datetime-local)",
        ),
    ];
    for (value, (state, scale, default, allowed, base, periodic, anchor)) in
        values.iter().zip(expected)
    {
        assert_eq!(value.state(), state);
        assert_eq!(value.step_scale_factor(), scale);
        assert_eq!(value.default_step(), default);
        assert_eq!(
            value
                .default_allowed_step()
                .as_document_string()
                .to_utf8()
                .unwrap(),
            allowed
        );
        let base_value = value.default_step_base();
        assert_eq!(base_value.state(), state);
        assert_eq!(base_value.unit(), value.unit());
        assert_eq!(spelling(&base_value), base);
        assert_eq!(value.has_periodic_domain(), periodic);
        assert!(
            value
                .source_clause()
                .ends_with(&format!("#{anchor}:concept-input-value-string-number"))
        );
        assert!(
            value
                .source_clause()
                .contains("877d6146487f15b15bc8dd0e3c625063a37b456a")
        );
    }
}

#[test]
fn native_signed_decimal_retains_canonical_magnitude_and_normalizes_zero() {
    let negative = HtmlSignedDecimal::from_magnitude(seconds(0, "00100"), true);
    assert_eq!(
        negative.as_document_string(),
        DocumentString::from("-0.001")
    );
    assert_eq!(negative.magnitude().fractional_digits(), Some("001"));
    assert_eq!(
        HtmlSignedDecimal::from(HtmlInteger::from(-123)).as_document_string(),
        DocumentString::from("-123")
    );
    let zero = HtmlSignedDecimal::from_magnitude(HtmlNonNegativeInteger::from(0).into(), true);
    assert!(!zero.is_negative());
    assert_eq!(zero.as_document_string(), DocumentString::from("0"));
}

#[test]
fn positive_step_scaling_is_exact_and_nonpositive_is_not_a_default_guess() {
    let coordinate = time(0, 0, 0, "0").input_coordinate();
    assert_eq!(
        coordinate
            .scale_step(0.125.try_into().unwrap())
            .unwrap()
            .as_document_string(),
        DocumentString::from("125")
    );
    for value in [0.0, -0.0, -1.0] {
        assert_eq!(
            coordinate.scale_step(value.try_into().unwrap()),
            Err(HtmlInputTemporalStepError)
        );
    }
}

#[test]
fn step_scaling_preserves_the_actual_rounded_binary64_not_shortest_decimal() {
    let coordinate = time(0, 0, 0, "0").input_coordinate();
    let step = HtmlFloatingPointNumber::try_from(0.1).unwrap();
    assert_eq!(
        coordinate.scale_step(step).unwrap().as_document_string(),
        DocumentString::from("100.0000000000000055511151231257827021181583404541015625")
    );
}

#[test]
fn the_same_recovered_step_uses_each_states_source_scale() {
    let coordinates = [
        date(1970, 1, 1).input_coordinate(),
        HtmlMonth::new(year(1970), 1).unwrap().input_coordinate(),
        HtmlWeek::new(year(1970), 1).unwrap().input_coordinate(),
        time(0, 0, 0, "0").input_coordinate(),
        HtmlParsedLocalDateTime::new(date(1970, 1, 1), time(0, 0, 0, "0")).input_coordinate(),
    ];
    for (coordinate, expected) in coordinates
        .iter()
        .zip(["10800000", "0.125", "75600000", "125", "125"])
    {
        assert_eq!(
            coordinate
                .scale_step(0.125.try_into().unwrap())
                .unwrap()
                .as_document_string(),
            DocumentString::from(expected)
        );
    }
}

#[test]
fn source_recovered_underflow_is_zero_before_native_step_admission() {
    let raw: DocumentString = "1e-9999".into();
    let report = HtmlValueInterpreter::new(raw.len())
        .floating_point_number(Some(&raw))
        .unwrap();
    let HtmlValueRecoveryState::Value(step) = report.recovery() else {
        panic!("source underflow must recover zero")
    };
    assert_eq!(step.value(), 0.0);
    let coordinate = time(0, 0, 0, "0").input_coordinate();
    assert_eq!(
        coordinate.scale_step(*step),
        Err(HtmlInputTemporalStepError)
    );
    assert_eq!(
        coordinate.default_allowed_step().as_document_string(),
        DocumentString::from("60000")
    );
}

#[test]
fn step_prefix_recovery_remains_distinct_from_author_spelling() {
    let raw: DocumentString = "+0.125tail".into();
    let report = HtmlValueInterpreter::new(raw.len())
        .floating_point_number(Some(&raw))
        .unwrap();
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    let HtmlValueRecoveryState::Value(step) = report.recovery() else {
        panic!("step prefix must recover")
    };
    assert_eq!(
        time(0, 0, 0, "0")
            .input_coordinate()
            .scale_step(*step)
            .unwrap()
            .as_document_string(),
        DocumentString::from("125")
    );
    assert_eq!(report.raw(), Some(&raw));
}

#[test]
fn largest_finite_step_scales_without_binary64_product_overflow() {
    // Exact binary64 MAX = (2^53 - 1) * 2^971; multiply by 1000 mathematically.
    assert_eq!(
        time(0, 0, 0, "0")
            .input_coordinate()
            .scale_step(f64::MAX.try_into().unwrap())
            .unwrap()
            .as_document_string(),
        DocumentString::from(
            "179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368000"
        )
    );
}

#[test]
fn smallest_subnormal_step_keeps_every_exact_digit() {
    // Independent integer oracle: (1000 / 2^1074), reduced then expanded in base 10.
    let result = time(0, 0, 0, "0")
        .input_coordinate()
        .scale_step(f64::from_bits(1).try_into().unwrap())
        .unwrap();
    let expected = concat!(
        "0.",
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "4940656458412465441765687928682213723650598026143247644255856825006755072702087518652998363616359923797965646954457177309266567103559397963987747960107818781263007131903114045278458171678489821036887186360569987307230500063874091535649843873124733972731696151400317153853980741262385655911710266585566867681870395603106249319452715914924553293054565444011274801297099995419319894090804165633245247571478690147267801593552386115501348035264934720193790268107107491703332226844753335720832431936092382893458368060106011506169809753078342277318329247904982524730776375927247874656084778203734469699533647017972677717585125660551199131504891101451037862738167250955837389733598993664809941164205702637090279242767544565229087538682506419718265533447265625"
    );
    assert_eq!(result.as_document_string(), DocumentString::from(expected));
}

#[test]
fn shared_gregorian_factor_preserves_utc_origin_and_offset_carry() {
    let local = HtmlParsedLocalDateTime::new(date(1970, 1, 1), time(0, 0, 0, "0"));
    let utc = HtmlUtcInstant::from_local_and_offset(&local, HtmlTimeZoneOffset::new(0).unwrap());
    assert_eq!(
        utc.day().as_document_string(),
        DocumentString::from("719162")
    );
    let first = HtmlParsedLocalDateTime::new(date(1, 1, 1), time(0, 0, 0, "000000001"));
    let utc = HtmlUtcInstant::from_local_and_offset(&first, HtmlTimeZoneOffset::new(1439).unwrap());
    assert_eq!(utc.day().as_document_string(), DocumentString::from("-1"));
    assert_eq!(utc.time().hour(), 0);
    assert_eq!(utc.time().minute(), 1);
    assert_eq!(utc.time().second().fractional_digits(), Some("000000001"));
}
