use crate::semantics::support::{builder, context, session};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
    HtmlTimeElementDescendantAuthorState, HtmlTimeElementReadback, HtmlTimeElementValueOrigin,
};

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    namespace: Namespace,
    name: &str,
    attributes: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(namespace, None, name),
        OrderedAttributes::new(attributes).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn datetime(raw: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, "datetime"), raw)
}
fn leaf(
    raw: Option<DocumentString>,
    text: Option<&[DocumentString]>,
) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let t = element(
        &mut b,
        &d,
        Namespace::html(),
        "time",
        raw.map(datetime).into_iter().collect(),
    );
    if let Some(text) = text {
        let nodes: Vec<_> = text
            .iter()
            .map(|text| b.add_text(&d, text.clone()).unwrap())
            .collect();
        b.establish_children(&t, &nodes).unwrap();
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&t).unwrap();
    (c, id)
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(100000000, 100, 1000, 10000, 10000000, 1000000)
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::TimeElement)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
) -> HtmlSemanticReport {
    session(c, cx, limits(), 100000, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn readback(r: &HtmlSemanticReport) -> &HtmlTimeElementReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::TimeElement(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected actual time readback: {other:?}"),
    }
}
fn parsed(r: &HtmlSemanticReport) -> &HtmlMachineReadableTime {
    match readback(r).value_report().recovery() {
        HtmlValueRecoveryState::Value(value) => value,
        other => panic!("expected exact time equivalent: {other:?}"),
    }
}
fn author_valid(r: &HtmlSemanticReport) -> &HtmlTimeElementValue {
    match readback(r).value_report().author() {
        HtmlValueAuthorState::Valid(value) => value,
        other => panic!("expected author-valid value: {other:?}"),
    }
}

#[test]
fn ten_source_formats_have_literal_native_components_without_child_or_host_context() {
    for raw in [
        "2011-11",
        "2011-11-18",
        "--02-29",
        "14:54:39.929",
        "2011-11-18 14:54",
        "-0800",
        "2011-11-18T06:54-0800",
        "2011-W47",
        "0001",
        "PT4H18M3S",
    ] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        let value = readback(&r);
        assert_eq!(
            value.origin(),
            HtmlTimeElementValueOrigin::DatetimeAttribute
        );
        assert_eq!(
            value.descendant_author_state(),
            &HtmlTimeElementDescendantAuthorState::NotRequired
        );
        assert_eq!(value.value_report().raw(), Some(&DocumentString::from(raw)));
        assert_eq!(
            value.datetime_attribute().unwrap().value(),
            &DocumentString::from(raw)
        );
        author_valid(&r);
        match (raw, parsed(&r)) {
            ("2011-11", HtmlMachineReadableTime::Month(v)) => {
                assert_eq!(v.year().decimal_digits(), "2011");
                assert_eq!(v.month(), 11);
            }
            ("2011-11-18", HtmlMachineReadableTime::Date(v)) => {
                assert_eq!(v.year().decimal_digits(), "2011");
                assert_eq!((v.month(), v.day()), (11, 18));
            }
            ("--02-29", HtmlMachineReadableTime::YearlessDate(v)) => {
                assert_eq!((v.month(), v.day()), (2, 29))
            }
            ("14:54:39.929", HtmlMachineReadableTime::Time(v)) => {
                assert_eq!((v.hour(), v.minute()), (14, 54));
                assert_eq!(v.second().integer_part().decimal_digits(), "39");
                assert_eq!(v.second().fractional_digits(), Some("929"));
            }
            ("2011-11-18 14:54", HtmlMachineReadableTime::LocalDateTime(v)) => {
                assert_eq!(v.date().year().decimal_digits(), "2011");
                assert_eq!(v.date().day(), 18);
                assert_eq!((v.time().hour(), v.time().minute()), (14, 54));
            }
            ("-0800", HtmlMachineReadableTime::TimeZoneOffset(v)) => assert_eq!(v.minutes(), -480),
            ("2011-11-18T06:54-0800", HtmlMachineReadableTime::GlobalDateTime(v)) => {
                assert_eq!(
                    (v.local().time().hour(), v.local().time().minute()),
                    (6, 54)
                );
                assert_eq!(v.offset().minutes(), -480);
                assert_eq!((v.utc().time().hour(), v.utc().time().minute()), (14, 54));
                assert_eq!(v.utc().day().to_i64().unwrap(), 734458);
            }
            ("2011-W47", HtmlMachineReadableTime::Week(v)) => {
                assert_eq!(v.year().decimal_digits(), "2011");
                assert_eq!(v.week(), 47);
            }
            ("0001", HtmlMachineReadableTime::Year(v)) => assert_eq!(v.decimal_digits(), "1"),
            ("PT4H18M3S", HtmlMachineReadableTime::Duration(v)) => {
                assert_eq!(v.seconds().integer_part().decimal_digits(), "15483");
                assert_eq!(v.seconds().fractional_digits(), None);
            }
            other => panic!("wrong source-format result: {other:?}"),
        }
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn short_years_and_arbitrary_years_keep_processing_distinct_from_author() {
    for (raw, valid) in [
        ("1", false),
        ("001", false),
        ("0001", true),
        ("12345678901234567890123456789012345678901234567890", true),
    ] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        assert_eq!(
            matches!(
                readback(&r).value_report().author(),
                HtmlValueAuthorState::Valid(_)
            ),
            valid
        );
        let HtmlMachineReadableTime::Year(y) = parsed(&r) else {
            panic!()
        };
        assert_eq!(y.decimal_digits(), raw.trim_start_matches('0'));
    }
}

#[test]
fn exact_fractions_and_negative_zero_offsets_recover_without_author_projection() {
    for raw in [
        "14:54:00.0001",
        "2011-11-18T14:54:00.0001",
        "0001-01-01T00:00:00.0001+01:00",
        "-0000",
        "PT0.0001S",
    ] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        assert!(matches!(
            readback(&r).value_report().author(),
            HtmlValueAuthorState::Invalid(_)
        ));
        match parsed(&r) {
            HtmlMachineReadableTime::Time(v) => {
                assert_eq!(v.second().fractional_digits(), Some("0001"))
            }
            HtmlMachineReadableTime::LocalDateTime(v) => {
                assert_eq!(v.time().second().fractional_digits(), Some("0001"))
            }
            HtmlMachineReadableTime::GlobalDateTime(v) => {
                assert_eq!(v.local().time().second().fractional_digits(), Some("0001"));
                assert_eq!(v.utc().time().second().fractional_digits(), Some("0001"));
                assert_eq!(v.utc().day().to_i64().unwrap(), -1);
                assert_eq!((v.utc().time().hour(), v.utc().time().minute()), (23, 0));
            }
            HtmlMachineReadableTime::TimeZoneOffset(v) => assert_eq!(v.minutes(), 0),
            HtmlMachineReadableTime::Duration(v) => {
                assert_eq!(v.seconds().fractional_digits(), Some("0001"))
            }
            other => panic!("wrong exact recovery: {other:?}"),
        }
        assert_eq!(
            readback(&r).value_report().raw(),
            Some(&DocumentString::from(raw))
        );
    }
}

#[test]
fn duration_components_preserve_source_scales_disambiguation_and_exact_carry() {
    for (raw, seconds, fraction) in [
        ("PT0.9999S0.0001S", "1", None),
        (" 1w 2d 3h 4m 5s ", "788645", None),
        ("P0Y0M1DT2M", "86520", None),
        ("P0.1S0M", "0", Some("1")),
        ("1S1S", "2", None),
        ("PT.0000001S", "0", Some("0000001")),
    ] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        let HtmlMachineReadableTime::Duration(v) = parsed(&r) else {
            panic!()
        };
        assert_eq!(v.seconds().integer_part().decimal_digits(), seconds);
        assert_eq!(v.seconds().fractional_digits(), fraction);
    }
}

#[test]
fn zero_calendar_components_are_permitted_and_nonzero_calendar_is_source_failure() {
    for raw in ["P0Y", "P0M", "P0Y0M"] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        let HtmlMachineReadableTime::Duration(v) = parsed(&r) else {
            panic!()
        };
        assert!(v.seconds().is_zero());
    }
    for raw in [
        "P1Y",
        "P1M",
        "P1.5S1M",
        "0000",
        "+1",
        " 1",
        "1 ",
        "2011-11junk",
        "12:00junk",
    ] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        assert!(
            matches!(
                readback(&r).value_report().author(),
                HtmlValueAuthorState::Invalid(_)
            ),
            "{raw}"
        );
        assert!(
            matches!(
                readback(&r).value_report().recovery(),
                HtmlValueRecoveryState::Failed(_)
            ),
            "{raw}"
        );
    }
}

#[test]
fn child_text_is_immediate_exact_and_descendant_author_proof_is_independent() {
    let (mut b, d) = builder();
    let t = element(&mut b, &d, Namespace::html(), "time", vec![]);
    let a = b.add_text(&d, "2011-".into()).unwrap();
    let z = b.add_text(&d, "11".into()).unwrap();
    let span = element(
        &mut b,
        &d,
        Namespace::from_uri("urn:foreign"),
        "span",
        vec![],
    );
    // Its unavailable descendants are irrelevant to the already proven violation.
    let comment = b.add_comment(&d, "ignored".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "x".into(), "ignored".into())
        .unwrap();
    b.establish_children(&t, &[a, span.clone(), comment, pi, z])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = report(&c, &cx, &c.subject_id(&t).unwrap());
    let v = readback(&r);
    assert_eq!(v.origin(), HtmlTimeElementValueOrigin::ChildText);
    assert!(v.datetime_attribute().is_none());
    assert_eq!(v.value_report().raw(), Some(&"2011-11".into()));
    assert_eq!(
        v.descendant_author_state(),
        &HtmlTimeElementDescendantAuthorState::Nonconforming(c.subject_id(&span).unwrap())
    );
    assert!(matches!(parsed(&r), HtmlMachineReadableTime::Month(_)));
    assert!(matches!(author_valid(&r), HtmlTimeElementValue::Month(_)));
}

#[test]
fn complete_child_sequence_without_elements_proves_conforming_and_keeps_whitespace() {
    let (c, id) = leaf(None, Some(&[" 2011-".into(), "11 ".into()]));
    let cx = context(&c);
    let r = report(&c, &cx, &id);
    assert_eq!(
        readback(&r).descendant_author_state(),
        &HtmlTimeElementDescendantAuthorState::Conforming
    );
    assert_eq!(readback(&r).value_report().raw(), Some(&" 2011-11 ".into()));
    assert!(matches!(
        readback(&r).value_report().recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
}

#[test]
fn present_empty_never_falls_back_and_known_empty_child_text_is_not_missing_input() {
    for (raw, children, origin) in [
        (
            Some("".into()),
            Some(vec!["2011-11".into()]),
            HtmlTimeElementValueOrigin::DatetimeAttribute,
        ),
        (None, Some(vec![]), HtmlTimeElementValueOrigin::ChildText),
    ] {
        let (c, id) = leaf(raw, children.as_deref());
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        let v = readback(&r);
        assert_eq!(v.origin(), origin);
        assert_eq!(v.value_report().raw(), Some(&DocumentString::from("")));
        assert!(matches!(
            v.value_report().author(),
            HtmlValueAuthorState::Invalid(_)
        ));
        assert!(matches!(
            v.value_report().recovery(),
            HtmlValueRecoveryState::Failed(_)
        ));
    }
    let (c, id) = leaf(None, None);
    let cx = context(&c);
    let r = report(&c, &cx, &id);
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::TimeElement(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(id)
        ))
    );
}

#[test]
fn exact_namespace_case_and_attribute_prefix_preserve_selection() {
    for name in ["Datetime", "DATETIME"] {
        let (mut b, d) = builder();
        let t = element(
            &mut b,
            &d,
            Namespace::html(),
            "time",
            vec![Attribute::new(
                QualifiedName::new(Namespace::none(), None, name),
                "ignored long bad attribute",
            )],
        );
        let text = b.add_text(&d, "2011-11".into()).unwrap();
        b.establish_children(&t, &[text]).unwrap();
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = report(&c, &cx, &c.subject_id(&t).unwrap());
        assert_eq!(readback(&r).origin(), HtmlTimeElementValueOrigin::ChildText);
        assert!(matches!(parsed(&r), HtmlMachineReadableTime::Month(_)));
    }
    let (mut b, d) = builder();
    let attr = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("kept".into()), "datetime"),
        "2011-11",
    );
    let t = element(&mut b, &d, Namespace::html(), "time", vec![attr.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = report(&c, &cx, &c.subject_id(&t).unwrap());
    assert_eq!(readback(&r).datetime_attribute(), Some(&attr));
}

#[test]
fn foreign_namespaced_datetime_is_absent_and_does_not_admit_its_irrelevant_raw() {
    let (mut b, d) = builder();
    let t = element(
        &mut b,
        &d,
        Namespace::html(),
        "time",
        vec![Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:x"), Some("x".into()), "datetime"),
            "huge irrelevant content",
        )],
    );
    let text = b.add_text(&d, "1".into()).unwrap();
    b.establish_children(&t, &[text]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&c.subject_id(&t).unwrap())])
        .unwrap();
    assert_eq!(readback(&r).origin(), HtmlTimeElementValueOrigin::ChildText);
    assert!(matches!(parsed(&r), HtmlMachineReadableTime::Year(_)));
}

#[test]
fn exact_utf16_null_and_surrogate_values_are_preserved_as_source_failure() {
    for raw in [
        DocumentString::from_utf16(vec![49, 0, 50]),
        DocumentString::from_utf16(vec![49, 0xd800]),
        DocumentString::from_utf16(vec![0xd83d, 0xde00]),
    ] {
        for attribute in [true, false] {
            let (c, id) = if attribute {
                leaf(Some(raw.clone()), None)
            } else {
                leaf(None, Some(std::slice::from_ref(&raw)))
            };
            let cx = context(&c);
            let r = report(&c, &cx, &id);
            assert_eq!(readback(&r).value_report().raw(), Some(&raw));
            assert!(matches!(
                readback(&r).value_report().recovery(),
                HtmlValueRecoveryState::Failed(_)
            ));
        }
    }
}

#[test]
fn wrong_roles_and_foreign_subjects_do_not_consult_datetime_or_children() {
    let (mut b, d) = builder();
    let mut ids = vec![];
    for (namespace, name) in [
        (Namespace::html(), "div"),
        (Namespace::html(), "TIME"),
        (Namespace::html(), "custom-time"),
        (Namespace::svg(), "time"),
    ] {
        ids.push(element(
            &mut b,
            &d,
            namespace,
            name,
            vec![datetime("unadmitted large raw")],
        ));
    }
    let c = b.finish().unwrap();
    let cx = context(&c);
    for (n, t) in ids.iter().enumerate() {
        let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[request(&c.subject_id(t).unwrap())])
            .unwrap();
        let expected = if n == 3 {
            HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
        } else {
            HtmlSemanticValue::Inapplicable
        };
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::TimeElement(expected)
        );
        assert_eq!(r.usage().nested_output_records(), 0);
    }
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&c.subject_id(&d).unwrap())])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::TimeElement(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn gregorian_centuries_and_offset_day_carries_have_independent_exact_days() {
    for (raw, day, hour, minute) in [
        ("0001-01-01T00:00+00:01", -1, 23, 59),
        ("0001-01-01T23:59-00:01", 1, 0, 0),
        ("1900-02-28T00:00Z", 693653, 0, 0),
        ("2000-02-29T00:00Z", 730178, 0, 0),
        ("2400-02-29T00:00Z", 876275, 0, 0),
    ] {
        let (c, id) = leaf(Some(raw.into()), None);
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        let HtmlMachineReadableTime::GlobalDateTime(v) = parsed(&r) else {
            panic!()
        };
        assert_eq!(v.utc().day().to_i64().unwrap(), day, "{raw}");
        assert_eq!(
            (v.utc().time().hour(), v.utc().time().minute()),
            (hour, minute)
        );
    }
    let (c, id) = leaf(Some("1900-02-29T00:00Z".into()), None);
    let cx = context(&c);
    let r = report(&c, &cx, &id);
    assert!(matches!(
        readback(&r).value_report().recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
}

#[test]
fn long_global_fraction_accounts_both_retained_seconds_without_rounding() {
    let digits = "00000000000000000001".repeat(10);
    let raw = format!("0001-01-01T00:00:00.{digits}+01:00");
    let (c, id) = leaf(Some(raw.clone().into()), None);
    let cx = context(&c);
    let r = report(&c, &cx, &id);
    let HtmlMachineReadableTime::GlobalDateTime(v) = parsed(&r) else {
        panic!()
    };
    assert_eq!(
        v.local().time().second().fractional_digits(),
        Some(digits.as_str())
    );
    assert_eq!(
        v.utc().time().second().fractional_digits(),
        Some(digits.as_str())
    );
    assert_eq!(v.utc().day().to_i64().unwrap(), -1);
    assert!(r.usage().allocated_bytes() >= digits.len() * 2);
    assert_eq!(
        readback(&r).value_report().raw(),
        Some(&DocumentString::from(raw))
    );
}

#[test]
fn repeated_duration_accumulates_exactly_and_keeps_all_allocation_in_session_usage() {
    let raw = format!("PT{}", "1S".repeat(200));
    let (c, id) = leaf(Some(raw.into()), None);
    let cx = context(&c);
    let r = report(&c, &cx, &id);
    let HtmlMachineReadableTime::Duration(v) = parsed(&r) else {
        panic!()
    };
    assert_eq!(v.seconds().integer_part().decimal_digits(), "200");
    assert_eq!(v.seconds().fractional_digits(), None);
    assert!(r.usage().allocated_bytes() > 200);
    assert_eq!(r.usage().scratch_records(), 0);
}

fn limited(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    let l = limits();
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            l.work()
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            l.output_records()
        },
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            l.scratch_records()
        },
        if resource == HtmlSemanticResource::IntegerDigits {
            cap
        } else {
            l.integer_digits()
        },
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            l.allocated_bytes()
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            l.nested_output_records()
        },
    )
}

#[test]
fn selected_value_admission_is_whole_utf16_and_applies_to_child_text_before_attempts() {
    for attribute in [true, false] {
        let raw = DocumentString::from("2011-11");
        let (c, id) = if attribute {
            leaf(Some(raw.clone()), None)
        } else {
            leaf(None, Some(std::slice::from_ref(&raw)))
        };
        let cx = context(&c);
        for cap in [7, 6] {
            let mut ss = session(&c, &cx, limits(), cap, HtmlSemanticCancellation::new());
            let result = ss.query(&[request(&id)]);
            if cap == 7 {
                assert!(result.is_ok());
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(),HtmlSemanticFailureKind::Attribute(e) if e.capacity()==6 && e.required()==7)
                );
                assert_eq!(failure.qualification().unwrap().attribute_input_limit(), 6);
                assert_eq!(ss.query(&[]).unwrap_err(), failure);
            }
        }
    }
}

#[test]
fn actual_work_bytes_records_and_numeric_digit_limits_have_exact_one_short_boundaries() {
    for raw in ["12345", "2011-11-18T06:54:00.0001-0800", "PT0.9999S0.0001S"] {
        for attribute in [true, false] {
            let text = DocumentString::from(raw);
            let (c, id) = if attribute {
                leaf(Some(text.clone()), None)
            } else {
                leaf(None, Some(std::slice::from_ref(&text)))
            };
            let cx = context(&c);
            let usage = report(&c, &cx, &id).usage();
            for (resource, exact) in [
                (HtmlSemanticResource::Work, usage.work()),
                (HtmlSemanticResource::OutputRecords, usage.output_records()),
                (
                    HtmlSemanticResource::NestedOutputRecords,
                    usage.nested_output_records(),
                ),
                (
                    HtmlSemanticResource::AllocatedBytes,
                    usage.allocated_bytes(),
                ),
            ] {
                assert!(exact > 0);
                for cap in [exact, exact - 1] {
                    let mut ss = session(
                        &c,
                        &cx,
                        limited(resource, cap),
                        100000,
                        HtmlSemanticCancellation::new(),
                    );
                    let result = ss.query(&[request(&id)]);
                    if cap == exact {
                        assert!(result.is_ok(), "{raw} {resource:?}: {result:?}");
                    } else {
                        let failure = result.unwrap_err();
                        assert!(
                            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                            "{raw} {resource:?}: {failure:?}"
                        );
                        assert_eq!(failure.qualification().unwrap().capture(), c.identity());
                        assert_eq!(failure.qualification().unwrap().context(), cx.identity());
                        assert_eq!(
                            failure.qualification().unwrap().limits(),
                            limited(resource, cap)
                        );
                        assert_eq!(ss.query(&[]).unwrap_err(), failure);
                    }
                    assert_eq!(ss.usage().scratch_records(), 0);
                }
            }
        }
    }
    let (c, id) = leaf(Some("12345".into()), None);
    let cx = context(&c);
    assert!(
        session(
            &c,
            &cx,
            limited(HtmlSemanticResource::IntegerDigits, 5),
            5,
            HtmlSemanticCancellation::new()
        )
        .query(&[request(&id)])
        .is_ok()
    );
    let mut ss = session(
        &c,
        &cx,
        limited(HtmlSemanticResource::IntegerDigits, 4),
        5,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[request(&id)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::IntegerDigits,
            limit: 4,
            required: 5
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}

#[test]
fn later_numeric_failure_keeps_earlier_observed_buffers_and_never_tries_another_format() {
    let (c, id) = leaf(Some("9999-12-31T00:00Z".into()), None);
    let cx = context(&c);
    let mut ss = session(
        &c,
        &cx,
        limited(HtmlSemanticResource::IntegerDigits, 4),
        100000,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[request(&id)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::IntegerDigits,
            ..
        }
    ));
    assert!(failure.usage().allocated_bytes() > 0);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn late_mixed_batch_failure_discards_prefix_and_retains_original_qualified_stop() {
    let (mut b, d) = builder();
    let good = element(&mut b, &d, Namespace::html(), "time", vec![datetime("1")]);
    let bad = element(
        &mut b,
        &d,
        Namespace::html(),
        "time",
        vec![datetime("123456")],
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut ss = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    let failure = ss
        .query(&[
            HtmlSemanticRequest::new(c.subject_id(&good).unwrap(), HtmlSemanticQuery::Hidden),
            request(&c.subject_id(&good).unwrap()),
            request(&c.subject_id(&bad).unwrap()),
        ])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert!(failure.usage().allocated_bytes() > 0);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn cancellation_and_foreign_requests_remain_sticky_with_original_session_qualification() {
    let (c, id) = leaf(Some("2011-11".into()), None);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), 7, cancel.clone());
    let old = ss.query(&[request(&id)]).unwrap();
    cancel.cancel();
    let failure = ss.query(&[request(&id)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(
        failure.qualification().unwrap().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert!(matches!(parsed(&old), HtmlMachineReadableTime::Month(_)));
    let (other, foreign) = leaf(Some("1".into()), None);
    let mut ss = session(&c, &cx, limits(), 7, HtmlSemanticCancellation::new());
    let failure = ss.query(&[request(&id), request(&foreign)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    ));
    assert_ne!(failure.qualification().unwrap().capture(), other.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}

#[test]
fn changed_capture_text_attributes_and_context_keys_recompute_and_preserve_old_readbacks() {
    let mut reports = vec![];
    for (raw, text) in [
        (Some("2011-11".into()), None),
        (None, Some(vec!["2011-".into(), "12".into()])),
        (Some("".into()), Some(vec!["2011-11".into()])),
    ] {
        let (c, id) = leaf(raw, text.as_deref());
        let cx = context(&c);
        let r = report(&c, &cx, &id);
        assert_eq!(r.capture(), c.identity());
        assert_eq!(r.context(), cx.identity());
        assert_eq!(r.host_revision(), c.host_facts_revision());
        reports.push(r);
    }
    let HtmlMachineReadableTime::Month(first) = parsed(&reports[0]) else {
        panic!()
    };
    assert_eq!(first.month(), 11);
    let HtmlMachineReadableTime::Month(second) = parsed(&reports[1]) else {
        panic!()
    };
    assert_eq!(second.month(), 12);
    assert_eq!(readback(&reports[2]).value_report().raw(), Some(&"".into()));
    assert_ne!(reports[0].capture(), reports[1].capture());
    let (c, id) = leaf(Some("1".into()), None);
    let a = context(&c);
    let z = context(&c);
    let first = report(&c, &a, &id);
    let second = report(&c, &z, &id);
    assert_ne!(first.context(), second.context());
    assert_eq!(
        readback(&first).value_report(),
        readback(&second).value_report()
    );
}

#[test]
fn failed_fraction_reservation_records_the_earlier_integer_buffer_before_terminal_mapping() {
    let (mut b, d) = builder();
    let t = element(
        &mut b,
        &d,
        Namespace::html(),
        "time",
        vec![datetime("12:00:01.1234")],
    );
    let irrelevant = element(&mut b, &d, Namespace::html(), "div", vec![]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    // The wrong-role request admits only the common outer result vector.
    let baseline = report(&c, &cx, &c.subject_id(&irrelevant).unwrap())
        .usage()
        .allocated_bytes();
    let mut ss = session(
        &c,
        &cx,
        limited(HtmlSemanticResource::AllocatedBytes, baseline + 1),
        100000,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss
        .query(&[request(&c.subject_id(&t).unwrap())])
        .unwrap_err();
    // Four fractional digits are invalid author time syntax. Recovery first
    // allocates the one-digit seconds integer, then cannot reserve its fraction.
    assert_eq!(ss.usage().allocated_bytes(), baseline + 1);
    assert!(
        matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded {
        resource: HtmlSemanticResource::AllocatedBytes, limit, required
    } if *limit == baseline + 1 && *required == baseline + 5)
    );
    assert_eq!(ss.usage().nested_output_records(), 0);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}
