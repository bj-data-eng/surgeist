use surgeist_html::{
    DocumentString, HtmlDate, HtmlDuration, HtmlGlobalDateTime, HtmlLocalDateTime,
    HtmlNonNegativeDecimal, HtmlNonNegativeInteger, HtmlParsedDuration, HtmlParsedGlobalDateTime,
    HtmlParsedLocalDateTime, HtmlParsedTime, HtmlTemporalError, HtmlTemporalPrecisionError,
    HtmlTime, HtmlTimeZoneOffset, HtmlUtcInstant, HtmlYear,
};

fn decimal(integer: u128, fraction: &str) -> HtmlNonNegativeDecimal {
    HtmlNonNegativeDecimal::with_fraction_digits(integer.into(), &DocumentString::from(fraction))
        .unwrap()
}

fn local(year: HtmlYear, month: u8, day: u8, hour: u8, minute: u8) -> HtmlParsedLocalDateTime {
    HtmlParsedLocalDateTime::new(
        HtmlDate::new(year, month, day).unwrap(),
        HtmlParsedTime::new(hour, minute, HtmlNonNegativeInteger::from(0_u128).into()).unwrap(),
    )
}

#[test]
fn parsed_clock_checks_range_without_rounding_exact_seconds() {
    let value = HtmlParsedTime::new(23, 59, decimal(59, "999999999999999999999999999999")).unwrap();
    assert_eq!(value.hour(), 23);
    assert_eq!(value.minute(), 59);
    assert_eq!(value.second().integer_part().decimal_digits(), "59");
    assert_eq!(
        value.second().fractional_digits(),
        Some("999999999999999999999999999999")
    );
    assert_eq!(HtmlTime::try_from(value), Err(HtmlTemporalPrecisionError));
    assert_eq!(
        HtmlParsedTime::new(24, 0, decimal(0, "1")),
        Err(HtmlTemporalError::HourOutOfRange)
    );
    assert_eq!(
        HtmlParsedTime::new(0, 60, decimal(0, "1")),
        Err(HtmlTemporalError::MinuteOutOfRange)
    );
    assert_eq!(
        HtmlParsedTime::new(0, 0, decimal(60, "0")),
        Err(HtmlTemporalError::SecondOutOfRange)
    );
    let huge = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(
        "10000000000000000000000000000000000000000",
    ))
    .unwrap();
    assert_eq!(
        HtmlParsedTime::new(0, 0, huge.into()),
        Err(HtmlTemporalError::SecondOutOfRange)
    );
}

#[test]
fn authored_conversions_are_infallible_and_precision_projection_is_checked() {
    let time = HtmlTime::new(12, 34, 5, 10).unwrap();
    let parsed = HtmlParsedTime::from(time);
    assert_eq!(parsed.second().fractional_digits(), Some("01"));
    assert_eq!(HtmlTime::try_from(parsed), Ok(time));
    let trailing_zero = HtmlParsedTime::new(12, 34, decimal(5, "010000000000")).unwrap();
    assert_eq!(HtmlTime::try_from(trailing_zero), Ok(time));
    let finer = HtmlParsedTime::new(12, 34, decimal(5, "010000000001")).unwrap();
    assert_eq!(
        HtmlTime::try_from(finer.clone()),
        Err(HtmlTemporalPrecisionError)
    );
    let date = HtmlDate::new(HtmlYear::new(2000).unwrap(), 2, 29).unwrap();
    let authored_local = HtmlLocalDateTime::new(date.clone(), time);
    let parsed_local = HtmlParsedLocalDateTime::from(authored_local.clone());
    assert_eq!(
        HtmlLocalDateTime::try_from(parsed_local),
        Ok(authored_local.clone())
    );
    assert_eq!(
        HtmlLocalDateTime::try_from(HtmlParsedLocalDateTime::new(date, finer)),
        Err(HtmlTemporalPrecisionError)
    );
    let authored_global =
        HtmlGlobalDateTime::new(authored_local, HtmlTimeZoneOffset::new(-30).unwrap());
    assert_eq!(
        HtmlGlobalDateTime::try_from(HtmlParsedGlobalDateTime::from(authored_global.clone())),
        Ok(authored_global)
    );
    let duration = HtmlDuration::new(100_u128.into(), 1).unwrap();
    assert_eq!(
        HtmlDuration::try_from(HtmlParsedDuration::from(duration.clone())),
        Ok(duration)
    );
    assert_eq!(
        HtmlDuration::try_from(HtmlParsedDuration::new(decimal(100, "0001"))),
        Err(HtmlTemporalPrecisionError)
    );
    assert_eq!(
        HtmlDuration::try_from(HtmlParsedDuration::new(decimal(100, "00100000")))
            .unwrap()
            .millisecond(),
        1
    );
}

#[test]
fn utc_origin_and_calendar_constants_are_exact_across_centuries() {
    // Independent Gregorian ordinal constants: 400 years contain 146097 days;
    // century 1900 excludes Feb 29, while century 2000 includes it.
    for (year, month, day, expected) in [
        (1, 1, 1, 0),
        (401, 1, 1, 146097),
        (1900, 3, 1, 693654),
        (2000, 3, 1, 730179),
    ] {
        let instant = HtmlUtcInstant::from_local_and_offset(
            &local(HtmlYear::new(year).unwrap(), month, day, 0, 0),
            HtmlTimeZoneOffset::new(0).unwrap(),
        );
        assert_eq!(instant.day().to_i128(), Ok(expected));
    }
    let before_origin = HtmlUtcInstant::from_local_and_offset(
        &local(HtmlYear::new(1).unwrap(), 1, 1, 0, 0),
        HtmlTimeZoneOffset::new(1439).unwrap(),
    );
    assert_eq!(before_origin.day().to_i128(), Ok(-1));
    assert_eq!(
        (before_origin.time().hour(), before_origin.time().minute()),
        (0, 1)
    );
    let after_midnight = HtmlUtcInstant::from_local_and_offset(
        &local(HtmlYear::new(1).unwrap(), 1, 1, 23, 59),
        HtmlTimeZoneOffset::new(-1439).unwrap(),
    );
    assert_eq!(after_midnight.day().to_i128(), Ok(1));
    assert_eq!(
        (after_midnight.time().hour(), after_midnight.time().minute()),
        (23, 58)
    );
}

#[test]
fn unbounded_year_utc_keeps_exact_cycle_magnitude_and_fraction() {
    // Year 400 * 10^40 + 1 begins after exactly 10^40 Gregorian cycles.
    let year = HtmlYear::from_decimal_digits(&format!("4{}1", "0".repeat(41))).unwrap();
    let instant = HtmlUtcInstant::from_local_and_offset(
        &local(year, 1, 1, 0, 0),
        HtmlTimeZoneOffset::new(0).unwrap(),
    );
    assert_eq!(
        instant.day().magnitude().decimal_digits(),
        format!("146097{}", "0".repeat(40))
    );
    assert!(instant.day().to_i128().is_err());
    let date = HtmlDate::new(HtmlYear::new(2000).unwrap(), 1, 1).unwrap();
    let a = HtmlParsedGlobalDateTime::new(
        HtmlParsedLocalDateTime::new(
            date.clone(),
            HtmlParsedTime::new(1, 0, decimal(0, "000000000001")).unwrap(),
        ),
        HtmlTimeZoneOffset::new(60).unwrap(),
    );
    let b = HtmlParsedGlobalDateTime::new(
        HtmlParsedLocalDateTime::new(
            date,
            HtmlParsedTime::new(0, 0, decimal(0, "000000000001")).unwrap(),
        ),
        HtmlTimeZoneOffset::new(0).unwrap(),
    );
    assert_ne!(a, b);
    assert_eq!(a.utc(), b.utc());
    assert_eq!(
        a.utc().time().second().fractional_digits(),
        Some("000000000001")
    );
}

#[test]
fn calendar_cycle_oracle_matches_successive_day_count_for_every_year() {
    // Advance a bounded independent calendar from Monday 0001-01-01 instead
    // of reusing the producer's quotient/remainder formula.
    let mut january_first = 0_i128;
    for year in 1..=800_u64 {
        let instant = HtmlUtcInstant::from_local_and_offset(
            &local(HtmlYear::new(year).unwrap(), 1, 1, 0, 0),
            HtmlTimeZoneOffset::new(0).unwrap(),
        );
        assert_eq!(instant.day().to_i128(), Ok(january_first), "year {year}");
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        january_first += if leap { 366 } else { 365 };
    }
}
