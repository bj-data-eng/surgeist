//! Independent literal expectations from HTML §4.5.14 and its §2.3.5 grammars.
use crate::html::support::year;
use surgeist_html::*;

fn read(text: &str) -> HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime> {
    let raw = DocumentString::from(text);
    HtmlValueInterpreter::new(raw.len())
        .time_element_value(Some(&raw))
        .unwrap()
}

fn recovered(
    report: &HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
) -> &HtmlMachineReadableTime {
    match report.recovery() {
        HtmlValueRecoveryState::Value(value) => value,
        other => panic!("expected machine-readable equivalent, got {other:?}"),
    }
}

#[test]
fn ten_source_alternatives_produce_their_distinct_authored_and_processing_domains() {
    let cases = [
        (
            "2011-11",
            HtmlTimeElementValue::Month(HtmlMonth::new(year(2011), 11).unwrap()),
        ),
        (
            "2011-11-18",
            HtmlTimeElementValue::Date(HtmlDate::new(year(2011), 11, 18).unwrap()),
        ),
        (
            "--02-29",
            HtmlTimeElementValue::YearlessDate(HtmlYearlessDate::new(2, 29).unwrap()),
        ),
        (
            "14:54:39.929",
            HtmlTimeElementValue::Time(HtmlTime::new(14, 54, 39, 929).unwrap()),
        ),
        (
            "2011-11-18 14:54",
            HtmlTimeElementValue::LocalDateTime(HtmlLocalDateTime::new(
                HtmlDate::new(year(2011), 11, 18).unwrap(),
                HtmlTime::new(14, 54, 0, 0).unwrap(),
            )),
        ),
        (
            "-0800",
            HtmlTimeElementValue::TimeZoneOffset(HtmlTimeZoneOffset::new(-480).unwrap()),
        ),
        (
            "2011-11-18T06:54-0800",
            HtmlTimeElementValue::GlobalDateTime(HtmlGlobalDateTime::new(
                HtmlLocalDateTime::new(
                    HtmlDate::new(year(2011), 11, 18).unwrap(),
                    HtmlTime::new(6, 54, 0, 0).unwrap(),
                ),
                HtmlTimeZoneOffset::new(-480).unwrap(),
            )),
        ),
        (
            "2011-W47",
            HtmlTimeElementValue::Week(HtmlWeek::new(year(2011), 47).unwrap()),
        ),
        ("0001", HtmlTimeElementValue::Year(year(1))),
        (
            "PT4H18M3S",
            HtmlTimeElementValue::Duration(HtmlDuration::new(15483_u128.into(), 0).unwrap()),
        ),
    ];
    for (text, expected) in cases {
        let report = read(text);
        assert_eq!(
            report.author(),
            &HtmlValueAuthorState::Valid(expected.clone())
        );
        assert_eq!(report.raw(), Some(&DocumentString::from(text)));
        assert_eq!(report.syntax(), HtmlValueSyntax::TimeElement);
        assert_eq!(
            report.syntax().source_clause(),
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-time-element"
        );
        match (&expected, recovered(&report)) {
            (HtmlTimeElementValue::Month(_), HtmlMachineReadableTime::Month(value)) => {
                assert_eq!(value.year().decimal_digits(), "2011");
                assert_eq!(value.month(), 11);
            }
            (HtmlTimeElementValue::Date(_), HtmlMachineReadableTime::Date(value)) => {
                assert_eq!(value.year().decimal_digits(), "2011");
                assert_eq!((value.month(), value.day()), (11, 18));
            }
            (
                HtmlTimeElementValue::YearlessDate(_),
                HtmlMachineReadableTime::YearlessDate(value),
            ) => {
                assert_eq!((value.month(), value.day()), (2, 29))
            }
            (HtmlTimeElementValue::Time(_), HtmlMachineReadableTime::Time(value)) => {
                assert_eq!((value.hour(), value.minute()), (14, 54));
                assert_eq!(value.second().integer_part().decimal_digits(), "39");
                assert_eq!(value.second().fractional_digits(), Some("929"));
            }
            (
                HtmlTimeElementValue::LocalDateTime(_),
                HtmlMachineReadableTime::LocalDateTime(value),
            ) => {
                assert_eq!((value.date().month(), value.date().day()), (11, 18));
                assert_eq!((value.time().hour(), value.time().minute()), (14, 54));
            }
            (
                HtmlTimeElementValue::TimeZoneOffset(_),
                HtmlMachineReadableTime::TimeZoneOffset(value),
            ) => assert_eq!(value.minutes(), -480),
            (
                HtmlTimeElementValue::GlobalDateTime(_),
                HtmlMachineReadableTime::GlobalDateTime(value),
            ) => {
                assert_eq!(value.offset().minutes(), -480);
                assert_eq!(
                    (value.local().time().hour(), value.local().time().minute()),
                    (6, 54)
                );
                assert_eq!(
                    (value.utc().time().hour(), value.utc().time().minute()),
                    (14, 54)
                );
                // 2010*365 + 502 - 20 + 5 + 304 + 17 days from 0001-01-01.
                assert_eq!(value.utc().day(), &HtmlInteger::from(734458));
            }
            (HtmlTimeElementValue::Week(_), HtmlMachineReadableTime::Week(value)) => {
                assert_eq!(value.year().decimal_digits(), "2011");
                assert_eq!(value.week(), 47);
            }
            (HtmlTimeElementValue::Year(_), HtmlMachineReadableTime::Year(value)) => {
                assert_eq!(value.decimal_digits(), "1")
            }
            (HtmlTimeElementValue::Duration(_), HtmlMachineReadableTime::Duration(value)) => {
                assert_eq!(value.seconds().integer_part().decimal_digits(), "15483");
                assert_eq!(value.seconds().fractional_digits(), None);
            }
            other => panic!("wrong processing domain for {text}: {other:?}"),
        }
    }
}

#[test]
fn processing_year_has_no_author_only_four_digit_minimum() {
    for (text, canonical, author_valid) in [
        ("1", "1", false),
        ("001", "1", false),
        ("123", "123", false),
        ("0001", "1", true),
        ("0000001", "1", true),
        (
            "340282366920938463463374607431768211456",
            "340282366920938463463374607431768211456",
            true,
        ),
    ] {
        let report = read(text);
        assert_eq!(
            matches!(
                report.author(),
                HtmlValueAuthorState::Valid(HtmlTimeElementValue::Year(_))
            ),
            author_valid
        );
        if !author_valid {
            assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
        }
        let HtmlMachineReadableTime::Year(value) = recovered(&report) else {
            panic!("expected year")
        };
        assert_eq!(value.decimal_digits(), canonical);
    }
}

#[test]
fn zero_whitespace_and_non_ascii_digits_do_not_produce_years() {
    for text in ["0", "0000", " 0001", "0001 ", "١٢٣٤", "１２３４"] {
        let report = read(text);
        assert!(
            matches!(report.author(), HtmlValueAuthorState::Invalid(_)),
            "{text}"
        );
        assert!(
            matches!(report.recovery(), HtmlValueRecoveryState::Failed(_)),
            "{text}"
        );
    }
}

#[test]
fn signed_four_digit_values_are_offsets_rather_than_years() {
    for (text, minutes) in [("+0001", 1), ("-0001", -1)] {
        let report = read(text);
        assert_eq!(
            report.author(),
            &HtmlValueAuthorState::Valid(HtmlTimeElementValue::TimeZoneOffset(
                HtmlTimeZoneOffset::new(minutes).unwrap()
            ))
        );
        let HtmlMachineReadableTime::TimeZoneOffset(value) = recovered(&report) else {
            panic!("expected offset")
        };
        assert_eq!(value.minutes(), minutes);
    }
}

#[test]
fn parsed_time_keeps_precision_unavailable_to_the_authored_time() {
    let report = read("14:54:00.00000001");
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    let HtmlMachineReadableTime::Time(value) = recovered(&report) else {
        panic!("expected time")
    };
    assert_eq!((value.hour(), value.minute()), (14, 54));
    assert_eq!(value.second().integer_part().decimal_digits(), "0");
    assert_eq!(value.second().fractional_digits(), Some("00000001"));
}

#[test]
fn local_processing_is_not_narrowed_to_normalized_or_global_syntax() {
    for text in ["2011-11-18T14:54:00.0001", "2011-11-18 14:54:00.0001"] {
        let report = read(text);
        assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
        let HtmlMachineReadableTime::LocalDateTime(value) = recovered(&report) else {
            panic!("expected local")
        };
        assert_eq!(value.date().year().decimal_digits(), "2011");
        assert_eq!(value.time().second().fractional_digits(), Some("0001"));
    }
}

#[test]
fn global_processing_retains_fraction_source_offset_and_exact_utc() {
    let report = read("0001-01-01 00:00:00.00000001+23:59");
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    let HtmlMachineReadableTime::GlobalDateTime(value) = recovered(&report) else {
        panic!("expected global")
    };
    assert_eq!(value.offset().minutes(), 1439);
    assert_eq!(value.utc().day(), &HtmlInteger::from(-1));
    assert_eq!(
        (value.utc().time().hour(), value.utc().time().minute()),
        (0, 1)
    );
    assert_eq!(
        value.local().time().second().fractional_digits(),
        Some("00000001")
    );
    assert_eq!(
        value.utc().time().second().fractional_digits(),
        Some("00000001")
    );
}

#[test]
fn negative_zero_offset_recovery_does_not_certify_authored_syntax() {
    for text in ["-0000", "-00:00"] {
        let report = read(text);
        assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
        let HtmlMachineReadableTime::TimeZoneOffset(value) = recovered(&report) else {
            panic!("expected offset")
        };
        assert_eq!(value.minutes(), 0);
        assert_eq!(report.raw(), Some(&DocumentString::from(text)));
    }
    let report = read("2000-02-29T12:34-0000");
    assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
    let HtmlMachineReadableTime::GlobalDateTime(value) = recovered(&report) else {
        panic!("expected global")
    };
    assert_eq!(value.offset().minutes(), 0);
}

#[test]
fn duration_recovery_keeps_its_own_whitespace_and_exact_fraction_rules() {
    for (text, integer, fraction, author_valid) in [
        ("PT0.0001S", "0", Some("0001"), false),
        (" 1h ", "3600", None, true),
        ("1h1H", "7200", None, false),
    ] {
        let report = read(text);
        assert_eq!(
            matches!(
                report.author(),
                HtmlValueAuthorState::Valid(HtmlTimeElementValue::Duration(_))
            ),
            author_valid,
            "{text}"
        );
        if !author_valid {
            assert!(
                matches!(report.author(), HtmlValueAuthorState::Invalid(_)),
                "{text}"
            );
        }
        let HtmlMachineReadableTime::Duration(value) = recovered(&report) else {
            panic!("expected duration")
        };
        assert_eq!(value.seconds().integer_part().decimal_digits(), integer);
        assert_eq!(value.seconds().fractional_digits(), fraction);
    }
    // The processing grammar admits zero calendar quantities, not nonzero ones.
    assert!(matches!(
        recovered(&read("P0Y")),
        HtmlMachineReadableTime::Duration(_)
    ));
    assert!(matches!(
        read("P1Y").recovery(),
        HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(HtmlValueSyntaxError::Temporal(
            HtmlTemporalError::NonzeroCalendarDuration
        )))
    ));
}

#[test]
fn all_format_attempts_require_full_consumption_and_intrinsic_components() {
    for text in [
        "2011-11!",
        "2011-11-18!",
        "--02-29!",
        "14:54!",
        "2011-11-18T14:54!",
        "+0800!",
        "2011-11-18T14:54Z!",
        "2011-W47!",
        "0001!",
        "PT1S!",
        "2001-02-29",
        "--02-30",
        "24:00",
        "23:59:60",
        "+24:00",
        "2011-W53",
        "garbage",
    ] {
        let report = read(text);
        assert!(
            matches!(report.author(), HtmlValueAuthorState::Invalid(_)),
            "{text}"
        );
        assert!(
            matches!(report.recovery(), HtmlValueRecoveryState::Failed(_)),
            "{text}"
        );
        assert_eq!(report.raw(), Some(&DocumentString::from(text)));
    }
}

#[test]
fn native_ten_format_values_serialize_conforming_attributes() {
    let date = HtmlDate::new(year(2011), 11, 18).unwrap();
    let time = HtmlTime::new(14, 54, 39, 929).unwrap();
    let local = HtmlLocalDateTime::new(date.clone(), time);
    let cases = [
        (
            HtmlTimeElementValue::Month(HtmlMonth::new(year(2011), 11).unwrap()),
            "2011-11",
        ),
        (HtmlTimeElementValue::Date(date), "2011-11-18"),
        (
            HtmlTimeElementValue::YearlessDate(HtmlYearlessDate::new(2, 29).unwrap()),
            "02-29",
        ),
        (HtmlTimeElementValue::Time(time), "14:54:39.929"),
        (
            HtmlTimeElementValue::LocalDateTime(local.clone()),
            "2011-11-18T14:54:39.929",
        ),
        (
            HtmlTimeElementValue::TimeZoneOffset(HtmlTimeZoneOffset::new(-480).unwrap()),
            "-08:00",
        ),
        (
            HtmlTimeElementValue::GlobalDateTime(HtmlGlobalDateTime::new(
                local,
                HtmlTimeZoneOffset::new(-480).unwrap(),
            )),
            "2011-11-18T14:54:39.929-08:00",
        ),
        (
            HtmlTimeElementValue::Week(HtmlWeek::new(year(2011), 47).unwrap()),
            "2011-W47",
        ),
        (HtmlTimeElementValue::Year(year(1)), "0001"),
        (
            HtmlTimeElementValue::Duration(HtmlDuration::new(15483_u128.into(), 250).unwrap()),
            "PT15483.25S",
        ),
    ];
    for (value, spelling) in cases {
        assert_eq!(value.to_document_string(), DocumentString::from(spelling));
        let attribute = value.clone().into_datetime_attribute();
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Datetime.qualified_name()
        );
        assert_eq!(attribute.value(), &DocumentString::from(spelling));
        let report = HtmlValueInterpreter::new(attribute.value().len())
            .time_element_value(Some(attribute.value()))
            .unwrap();
        assert_eq!(report.author(), &HtmlValueAuthorState::Valid(value));
        assert!(matches!(
            report.recovery(),
            HtmlValueRecoveryState::Value(_)
        ));
    }
}

#[test]
fn missing_standalone_input_is_not_present_empty_or_child_text_fallback() {
    let interpreter = HtmlValueInterpreter::new(0);
    let absent = interpreter.time_element_value(None).unwrap();
    assert_eq!(absent.raw(), None);
    assert_eq!(absent.author(), &HtmlValueAuthorState::Missing);
    assert_eq!(absent.recovery(), &HtmlValueRecoveryState::Missing);
    let empty = DocumentString::from("");
    let present = interpreter.time_element_value(Some(&empty)).unwrap();
    assert_eq!(present.raw(), Some(&empty));
    assert!(matches!(present.author(), HtmlValueAuthorState::Invalid(_)));
    assert!(matches!(
        present.recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
    assert_eq!(absent.input_code_units(), 0);
    assert_eq!(present.input_code_units(), 0);
}

#[test]
fn whole_utf16_admission_precedes_any_syntax_and_preserves_non_scalar_units() {
    for raw in [
        DocumentString::from("0001"),
        DocumentString::from_utf16(vec![0xd800, 0, 0xd83d, 0xde00]),
    ] {
        let error = HtmlValueInterpreter::new(3)
            .time_element_value(Some(&raw))
            .unwrap_err();
        assert_eq!((error.capacity(), error.required()), (3, 4));
        let report = HtmlValueInterpreter::new(4)
            .time_element_value(Some(&raw))
            .unwrap();
        assert_eq!(report.raw(), Some(&raw));
        assert_eq!(report.input_code_units(), 4);
        if raw.code_units()[0] == 0xd800 {
            assert!(matches!(report.author(), HtmlValueAuthorState::Invalid(_)));
            assert!(matches!(
                report.recovery(),
                HtmlValueRecoveryState::Failed(_)
            ));
        } else {
            assert!(matches!(
                report.author(),
                HtmlValueAuthorState::Valid(HtmlTimeElementValue::Year(_))
            ));
        }
    }
}

#[test]
fn prior_reports_and_narrow_common_and_modification_domains_remain_unchanged() {
    let raw = DocumentString::from("14:54");
    let interpreter = HtmlValueInterpreter::new(64);
    let old = interpreter.time_element_value(Some(&raw)).unwrap();
    let common = interpreter.date_with_optional_time(Some(&raw)).unwrap();
    assert!(matches!(common.author(), HtmlValueAuthorState::Invalid(_)));
    assert!(matches!(
        common.recovery(),
        HtmlValueRecoveryState::Value(HtmlDateOrTime::Time(_))
    ));
    let date_or_time = interpreter
        .date_or_time(Some(&DocumentString::from("2011-11")))
        .unwrap();
    assert_eq!(date_or_time.author(), &HtmlValueAuthorState::NotDefined);
    assert!(matches!(
        date_or_time.recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
    let attribute = Attribute::new(KnownHtmlAttribute::Datetime.qualified_name(), raw.clone());
    let modification = interpreter
        .modification_datetime(&KnownHtmlElement::Ins.qualified_name(), Some(&attribute))
        .unwrap();
    assert_eq!(
        modification.timestamp(),
        HtmlModificationTimestamp::NoTimestamp(HtmlModificationTimestampAbsence::RecoveredTimeOnly)
    );
    let other = interpreter
        .time_element_value(Some(&DocumentString::from("2011-W47")))
        .unwrap();
    assert!(matches!(
        recovered(&other),
        HtmlMachineReadableTime::Week(_)
    ));
    assert_eq!(old.raw(), Some(&raw));
    assert!(matches!(
        old.author(),
        HtmlValueAuthorState::Valid(HtmlTimeElementValue::Time(_))
    ));
    assert!(matches!(recovered(&old), HtmlMachineReadableTime::Time(_)));
}
