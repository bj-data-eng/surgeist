//! Literal expectations from HTML #attr-mod-datetime and §2.3.5.10.
use surgeist_html::*;

fn attr(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Datetime.qualified_name(), value)
}

fn read(element: KnownHtmlElement, raw: Option<&Attribute>) -> HtmlModificationDatetimeReport {
    HtmlValueInterpreter::new(256)
        .modification_datetime(&element.qualified_name(), raw)
        .unwrap()
}

#[test]
fn missing_is_distinct_from_empty_and_whitespace_parse_failures() {
    for element in [KnownHtmlElement::Ins, KnownHtmlElement::Del] {
        let missing = read(element, None);
        assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(missing.timestamp(), HtmlModificationTimestamp::Missing);
        assert_eq!(
            missing.common().unwrap().author(),
            &HtmlValueAuthorState::Missing
        );
        assert_eq!(
            missing.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Missing
        );
        assert_eq!(missing.input_code_units(), 0);
        for value in ["", " \t\n"] {
            let raw = attr(value);
            let report = read(element, Some(&raw));
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.observation(),
                if value.is_empty() {
                    HtmlAttributeObservation::PresentEmpty
                } else {
                    HtmlAttributeObservation::PresentNonempty
                }
            );
            assert!(matches!(
                report.common().unwrap().author(),
                HtmlValueAuthorState::Invalid(_)
            ));
            assert!(matches!(
                report.common().unwrap().recovery(),
                HtmlValueRecoveryState::Failed(_)
            ));
            assert_eq!(
                report.timestamp(),
                HtmlModificationTimestamp::NoTimestamp(
                    HtmlModificationTimestampAbsence::ParseFailed
                )
            );
        }
    }
}

#[test]
fn date_is_an_associated_date_without_host_time_conversion() {
    for element in [KnownHtmlElement::Ins, KnownHtmlElement::Del] {
        let raw = attr("2000-02-29");
        let report = read(element, Some(&raw));
        assert!(matches!(
            report.common().unwrap().author(),
            HtmlValueAuthorState::Valid(HtmlDateWithOptionalTime::Date(_))
        ));
        let HtmlModificationTimestamp::Date(date) = report.timestamp() else {
            panic!("expected date")
        };
        assert_eq!(date.year().decimal_digits(), "2000");
        assert_eq!((date.month(), date.day()), (2, 29));
        let HtmlValueRecoveryState::Value(HtmlDateOrTime::Date(common)) =
            report.common().unwrap().recovery()
        else {
            panic!("expected common date")
        };
        assert!(std::ptr::eq(date, common));
        assert_eq!(report.attribute(), KnownHtmlAttribute::Datetime);
        assert_eq!(report.inventory_rows(), &[36]);
        assert!(report.source_clause().ends_with("/#attr-mod-datetime"));
        assert_eq!(report.raw(), Some(raw.value()));
    }
}

#[test]
fn global_retains_source_offset_local_clock_and_utc() {
    for (text, offset, utc_hour, utc_minute) in [
        ("2000-02-29T12:34Z", 0, 12, 34),
        ("2000-02-29T12:34-05:30", -330, 18, 4),
    ] {
        for element in [KnownHtmlElement::Ins, KnownHtmlElement::Del] {
            let raw = attr(text);
            let report = read(element, Some(&raw));
            assert!(matches!(
                report.common().unwrap().author(),
                HtmlValueAuthorState::Valid(HtmlDateWithOptionalTime::Global(_))
            ));
            let HtmlModificationTimestamp::Global(global) = report.timestamp() else {
                panic!("expected global")
            };
            assert_eq!(global.offset().minutes(), offset);
            assert_eq!(
                (global.local().time().hour(), global.local().time().minute()),
                (12, 34)
            );
            assert_eq!(
                (global.utc().time().hour(), global.utc().time().minute()),
                (utc_hour, utc_minute)
            );
            assert_eq!(global.utc().day(), &HtmlInteger::from(730178));
            let HtmlValueRecoveryState::Value(HtmlDateOrTime::Global(common)) =
                report.common().unwrap().recovery()
            else {
                panic!("expected common global")
            };
            assert!(std::ptr::eq(global, common));
        }
    }
}

#[test]
fn standalone_time_remains_recovery_evidence_but_never_a_modification_timestamp() {
    for text in ["00:00", "12:34:00.00000001"] {
        for element in [KnownHtmlElement::Ins, KnownHtmlElement::Del] {
            let raw = attr(text);
            let report = read(element, Some(&raw));
            assert!(matches!(
                report.common().unwrap().author(),
                HtmlValueAuthorState::Invalid(_)
            ));
            assert!(matches!(
                report.common().unwrap().recovery(),
                HtmlValueRecoveryState::Value(HtmlDateOrTime::Time(_))
            ));
            assert_eq!(
                report.timestamp(),
                HtmlModificationTimestamp::NoTimestamp(
                    HtmlModificationTimestampAbsence::RecoveredTimeOnly
                )
            );
            let common = HtmlValueInterpreter::new(256)
                .date_with_optional_time(Some(raw.value()))
                .unwrap();
            assert_eq!(report.common(), Some(&common));
        }
    }
}

#[test]
fn recovered_global_can_supply_timestamp_despite_strict_precision_failure() {
    let raw = attr("2000-02-29 12:34:00.00000001-0000");
    let report = read(KnownHtmlElement::Del, Some(&raw));
    assert!(matches!(
        report.common().unwrap().author(),
        HtmlValueAuthorState::Invalid(_)
    ));
    let HtmlModificationTimestamp::Global(global) = report.timestamp() else {
        panic!("expected recovered global")
    };
    assert_eq!(global.offset().minutes(), 0);
    assert_eq!(
        global.local().time().second().fractional_digits(),
        Some("00000001")
    );
    assert_eq!(
        global.utc().time().second().fractional_digits(),
        Some("00000001")
    );
    assert_eq!(report.raw_attribute(), Some(&raw));
}

#[test]
fn huge_year_is_exact_and_not_projected_to_a_machine_date() {
    let raw = attr("340282366920938463463374607431768211456-01-01");
    let report = read(KnownHtmlElement::Ins, Some(&raw));
    let HtmlModificationTimestamp::Date(date) = report.timestamp() else {
        panic!("expected huge date")
    };
    assert_eq!(
        date.year().decimal_digits(),
        "340282366920938463463374607431768211456"
    );
    assert!(matches!(
        report.common().unwrap().author(),
        HtmlValueAuthorState::Valid(_)
    ));
}

#[test]
fn invalid_calendar_missing_offset_and_trailing_garbage_have_no_timestamp() {
    for text in [
        "2001-02-29",
        "2000-02-29T12:34",
        "2000-02-29Z",
        "2000-02-29 ",
        "2000-02-29T12:34Z!",
    ] {
        let raw = attr(text);
        let report = read(KnownHtmlElement::Ins, Some(&raw));
        assert!(matches!(
            report.common().unwrap().author(),
            HtmlValueAuthorState::Invalid(_)
        ));
        assert_eq!(
            report.timestamp(),
            HtmlModificationTimestamp::NoTimestamp(HtmlModificationTimestampAbsence::ParseFailed)
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn native_date_and_global_authoring_read_back_without_a_second_native_parser() {
    let date = HtmlDate::new(HtmlYear::from_decimal_digits("2000").unwrap(), 2, 29).unwrap();
    let local = HtmlLocalDateTime::new(date.clone(), HtmlTime::new(12, 34, 1, 250).unwrap());
    let global = HtmlGlobalDateTime::new(local, HtmlTimeZoneOffset::new(-330).unwrap());
    for (value, spelling) in [
        (HtmlDateWithOptionalTime::Date(date), "2000-02-29"),
        (
            HtmlDateWithOptionalTime::Global(global),
            "2000-02-29T12:34:01.25-05:30",
        ),
    ] {
        let raw = value.clone().into_modification_datetime_attribute();
        assert_eq!(raw.name(), &KnownHtmlAttribute::Datetime.qualified_name());
        assert_eq!(raw.value(), &DocumentString::from(spelling));
        for element in [KnownHtmlElement::Ins, KnownHtmlElement::Del] {
            assert_eq!(
                read(element, Some(&raw)).common().unwrap().author(),
                &HtmlValueAuthorState::Valid(value.clone())
            );
        }
    }
}

#[test]
fn expanded_names_control_applicability_and_prefixes_are_retained() {
    let interpreter = HtmlValueInterpreter::new(256);
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "ins");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "datetime"),
        "2000-02-29",
    );
    let report = interpreter
        .modification_datetime(&element, Some(&raw))
        .unwrap();
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&raw));
    for (element, raw, applicability) in [
        (
            QualifiedName::new(Namespace::none(), None, "ins"),
            attr("00:00"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            KnownHtmlElement::Time.qualified_name(),
            attr("00:00"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "INS"),
            attr("00:00"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            KnownHtmlElement::Del.qualified_name(),
            Attribute::new(
                QualifiedName::new(Namespace::html(), None, "datetime"),
                "00:00",
            ),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            KnownHtmlElement::Ins.qualified_name(),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "DATETIME"),
                "00:00",
            ),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            KnownHtmlElement::Del.qualified_name(),
            Attribute::new(KnownHtmlAttribute::Cite.qualified_name(), "00:00"),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = interpreter
            .modification_datetime(&element, Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), applicability);
        assert_eq!(report.common(), None);
        assert_eq!(
            report.timestamp(),
            HtmlModificationTimestamp::Unapplied(applicability)
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn admission_precedes_wrong_role_and_counts_exact_utf16_units() {
    let raw = attr(DocumentString::from_utf16(vec![0xd800, 0xd83d, 0xde00]));
    let interpreter = HtmlValueInterpreter::new(2);
    for element in [KnownHtmlElement::Ins, KnownHtmlElement::Div] {
        let error = interpreter
            .modification_datetime(&element.qualified_name(), Some(&raw))
            .unwrap_err();
        assert_eq!(error.capacity(), 2);
        assert_eq!(error.required(), 3);
    }
    let report = HtmlValueInterpreter::new(3)
        .modification_datetime(&KnownHtmlElement::Ins.qualified_name(), Some(&raw))
        .unwrap();
    assert_eq!(report.input_code_units(), 3);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(
        report.timestamp(),
        HtmlModificationTimestamp::NoTimestamp(HtmlModificationTimestampAbsence::ParseFailed)
    );
}

#[test]
fn zero_capacity_admits_absence_and_empty_and_retained_reports_are_immutable() {
    let interpreter = HtmlValueInterpreter::new(0);
    let element = KnownHtmlElement::Ins.qualified_name();
    let absent = interpreter
        .modification_datetime(&element, None::<&surgeist_html::Attribute>)
        .unwrap();
    let empty = attr("");
    let present = interpreter
        .modification_datetime(&element, Some(&empty))
        .unwrap();
    assert_eq!(absent.timestamp(), HtmlModificationTimestamp::Missing);
    assert_eq!(
        present.observation(),
        HtmlAttributeObservation::PresentEmpty
    );
    assert_eq!(
        present.timestamp(),
        HtmlModificationTimestamp::NoTimestamp(HtmlModificationTimestampAbsence::ParseFailed)
    );
    assert_eq!(absent.raw(), None);
}
