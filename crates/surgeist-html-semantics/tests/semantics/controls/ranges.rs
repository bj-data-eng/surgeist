use crate::semantics::support::context;
use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlProgressState, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource,
    HtmlSemanticResult, HtmlSemanticSession, HtmlSemanticValue,
};

fn capture(
    local: &str,
    namespace: Namespace,
    attributes: Vec<(&str, DocumentString)>,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 20, 100_000, 10, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let attributes = OrderedAttributes::new(attributes.into_iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
    }))
    .unwrap();
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(namespace, None, local),
            attributes,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let result = builder.finish().unwrap();
    let element = result.subject_id(&element).unwrap();
    let document = result.subject_id(&document).unwrap();
    (result, element, document)
}
fn attributes(values: &[(&'static str, &str)]) -> Vec<(&'static str, DocumentString)> {
    values.iter().map(|(n, v)| (*n, (*v).into())).collect()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1_000_000, 100, 0, 0, 1_000_000, 0)
}
fn run(
    local: &str,
    attrs: &[(&'static str, &str)],
    query: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    let (capture, subject, _) = capture(local, Namespace::html(), attributes(attrs));
    let context = context(&capture);
    HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(100_000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(subject, query)])
    .unwrap()
}
fn meter(report: &HtmlSemanticReport) -> [f64; 6] {
    let HtmlSemanticFact::Meter(HtmlSemanticValue::Known(state)) = report.results()[0].fact()
    else {
        panic!("expected meter: {:?}", report.results()[0].fact());
    };
    [
        state.minimum().value(),
        state.maximum().value(),
        state.value().value(),
        state.low().value(),
        state.high().value(),
        state.optimum().value(),
    ]
}
fn progress(report: &HtmlSemanticReport) -> Option<[f64; 3]> {
    let HtmlSemanticFact::Progress(HtmlSemanticValue::Known(state)) = report.results()[0].fact()
    else {
        panic!("expected progress: {:?}", report.results()[0].fact());
    };
    match state {
        HtmlProgressState::Indeterminate => None,
        HtmlProgressState::Determinate(state) => Some([
            state.maximum().value(),
            state.value().value(),
            state.position().value(),
        ]),
    }
}

#[test]
fn meter_missing_defaults_are_one_complete_six_point_state() {
    assert_eq!(
        meter(&run("meter", &[], HtmlSemanticQuery::Meter)),
        [0., 1., 0., 0., 1., 0.5]
    );
}

#[test]
fn meter_crossed_maximum_collapses_every_dependent_boundary() {
    assert_eq!(
        meter(&run(
            "meter",
            &[
                ("min", "10"),
                ("max", "2"),
                ("value", "9"),
                ("low", "8"),
                ("high", "7"),
                ("optimum", "20")
            ],
            HtmlSemanticQuery::Meter
        )),
        [10., 10., 10., 10., 10., 10.]
    );
}

#[test]
fn meter_high_clamps_to_effective_low_and_optimum_to_minimum() {
    assert_eq!(
        meter(&run(
            "meter",
            &[
                ("min", "-10"),
                ("max", "10"),
                ("value", "20"),
                ("low", "8"),
                ("high", "-4"),
                ("optimum", "-50")
            ],
            HtmlSemanticQuery::Meter
        )),
        [-10., 10., 10., 8., 8., -10.]
    );
}

#[test]
fn meter_recovery_retains_numeric_prefix_and_uses_dependent_invalid_defaults() {
    assert_eq!(
        meter(&run(
            "meter",
            &[
                ("min", " +2tail"),
                ("max", "6e+"),
                ("value", "3junk"),
                ("low", "bad"),
                ("high", ""),
                ("optimum", "NaN")
            ],
            HtmlSemanticQuery::Meter
        )),
        [2., 6., 3., 2., 6., 4.]
    );
}

#[test]
fn meter_midpoint_avoids_large_sum_and_difference_overflow() {
    let max = "1.7976931348623157e308";
    assert_eq!(
        meter(&run(
            "meter",
            &[("min", max), ("max", max)],
            HtmlSemanticQuery::Meter
        )),
        [f64::MAX; 6]
    );
    let opposite = meter(&run(
        "meter",
        &[("min", "-1.7976931348623157e308"), ("max", max)],
        HtmlSemanticQuery::Meter,
    ));
    assert_eq!(opposite, [-f64::MAX, f64::MAX, 0., -f64::MAX, f64::MAX, 0.]);
    let tiny = f64::from_bits(1);
    assert_eq!(
        meter(&run(
            "meter",
            &[("min", "5e-324"), ("max", "5e-324")],
            HtmlSemanticQuery::Meter
        )),
        [tiny; 6]
    );
}

#[test]
fn meter_default_optimum_rounds_positive_subnormal_midpoints_once() {
    // Decimal inputs select successive binary64 subnormals. Literal bit
    // expectations describe the finite midpoint output, independently of the
    // implementation's arithmetic expression or numeric grammar.
    for (minimum, maximum, min_bits, max_bits, expected) in [
        ("5e-324", "1e-323", 1, 2, 2),
        ("1e-323", "1.5e-323", 2, 3, 2),
        ("5e-324", "2e-323", 1, 4, 2),
    ] {
        let state = meter(&run(
            "meter",
            &[("min", minimum), ("max", maximum)],
            HtmlSemanticQuery::Meter,
        ));
        assert_eq!(state[0].to_bits(), min_bits);
        assert_eq!(state[1].to_bits(), max_bits);
        assert_eq!(
            state[5].to_bits(),
            expected,
            "midpoint of {minimum} and {maximum}"
        );
    }
}

#[test]
fn meter_default_optimum_rounds_negative_subnormal_midpoints_once() {
    for (minimum, maximum, min_bits, max_bits) in [
        (
            "-1e-323",
            "-5e-324",
            0x8000_0000_0000_0002,
            0x8000_0000_0000_0001,
        ),
        (
            "-1.5e-323",
            "-1e-323",
            0x8000_0000_0000_0003,
            0x8000_0000_0000_0002,
        ),
        (
            "-2e-323",
            "-5e-324",
            0x8000_0000_0000_0004,
            0x8000_0000_0000_0001,
        ),
    ] {
        let state = meter(&run(
            "meter",
            &[("min", minimum), ("max", maximum)],
            HtmlSemanticQuery::Meter,
        ));
        assert_eq!(state[0].to_bits(), min_bits);
        assert_eq!(state[1].to_bits(), max_bits);
        assert_eq!(
            state[5].to_bits(),
            0x8000_0000_0000_0002,
            "midpoint of {minimum} and {maximum}"
        );
    }
}

#[test]
fn meter_default_optimum_normalizes_zero_from_cross_sign_subnormal_ties() {
    for (minimum, maximum) in [("-1e-323", "5e-324"), ("-5e-324", "1e-323")] {
        let state = meter(&run(
            "meter",
            &[("min", minimum), ("max", maximum)],
            HtmlSemanticQuery::Meter,
        ));
        assert_eq!(state[5].to_bits(), 0);
    }
}

#[test]
fn progress_missing_value_and_present_invalid_are_distinct() {
    assert_eq!(
        progress(&run(
            "progress",
            &[("max", "5")],
            HtmlSemanticQuery::Progress
        )),
        None
    );
    for value in ["", "invalid", "NaN", "Infinity", "1e9999", "-2", "-0"] {
        assert_eq!(
            progress(&run(
                "progress",
                &[("value", value)],
                HtmlSemanticQuery::Progress
            )),
            Some([1., 0., 0.])
        );
    }
}

#[test]
fn progress_uses_positive_maximum_clamps_and_exposes_position() {
    assert_eq!(
        progress(&run(
            "progress",
            &[("max", "4"), ("value", "1")],
            HtmlSemanticQuery::Progress
        )),
        Some([4., 1., 0.25])
    );
    assert_eq!(
        progress(&run(
            "progress",
            &[("max", "4"), ("value", "9")],
            HtmlSemanticQuery::Progress
        )),
        Some([4., 4., 1.])
    );
    for maximum in ["", "bad", "0", "-1", "1e99999"] {
        assert_eq!(
            progress(&run(
                "progress",
                &[("max", maximum), ("value", "0.5")],
                HtmlSemanticQuery::Progress
            )),
            Some([1., 0.5, 0.5])
        );
    }
}

#[test]
fn isolated_utf16_and_long_compensating_significand_follow_existing_numeric_owner() {
    let long = format!("1{}e-3000", "0".repeat(3000));
    assert_eq!(
        progress(&run(
            "progress",
            &[("value", &long)],
            HtmlSemanticQuery::Progress
        )),
        Some([1., 1., 1.])
    );
    let (capture, subject, _) = capture(
        "progress",
        Namespace::html(),
        vec![("value", DocumentString::from_utf16(vec![0xd800]))],
    );
    let context = context(&capture);
    let report = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(100),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        subject,
        HtmlSemanticQuery::Progress,
    )])
    .unwrap();
    assert_eq!(progress(&report), Some([1., 0., 0.]));
}

#[test]
fn query_qualification_retains_subject_and_all_actual_session_inputs() {
    let (capture, subject, _) = capture("meter", Namespace::html(), vec![]);
    let context = context(&capture);
    let request = HtmlSemanticRequest::new(subject.clone(), HtmlSemanticQuery::Meter);
    let report = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(32),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(std::slice::from_ref(&request))
    .unwrap();
    assert_eq!(report.results()[0].request(), &request);
    assert_eq!(report.capture(), capture.identity());
    assert_eq!(report.context(), context.identity());
    assert_eq!(report.host_revision(), capture.host_facts_revision());
    assert_eq!(report.attribute_input_limit(), 32);
    assert_eq!(report.qualification().limits(), limits());
    assert_eq!(
        report.source_revision(),
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    );
}

#[test]
fn non_range_elements_documents_and_foreign_namespace_are_inapplicable() {
    for namespace in [Namespace::html(), Namespace::none()] {
        let (capture, subject, document) = capture("div", namespace, vec![]);
        let context = context(&capture);
        let report = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(32),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .query(&[
            HtmlSemanticRequest::new(subject.clone(), HtmlSemanticQuery::Meter),
            HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Progress),
            HtmlSemanticRequest::new(document, HtmlSemanticQuery::Meter),
        ])
        .unwrap();
        assert!(matches!(
            report.results()[0].fact(),
            HtmlSemanticFact::Meter(HtmlSemanticValue::Inapplicable)
        ));
        assert!(matches!(
            report.results()[1].fact(),
            HtmlSemanticFact::Progress(HtmlSemanticValue::Inapplicable)
        ));
        assert!(matches!(
            report.results()[2].fact(),
            HtmlSemanticFact::Meter(HtmlSemanticValue::Inapplicable)
        ));
    }
}

#[test]
fn pre_cancelled_session_never_admits_output_and_keeps_first_failure() {
    let (capture, subject, _) = capture("meter", Namespace::html(), vec![]);
    let context = context(&capture);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut session = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(32),
        limits(),
        cancel,
    )
    .unwrap();
    let first = session
        .query(&[HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Meter)])
        .unwrap_err();
    assert!(matches!(first.kind(), HtmlSemanticFailureKind::Cancelled));
    assert_eq!(first.usage().work(), 0);
    assert_eq!(first.usage().allocated_bytes(), 0);
    assert_eq!(session.query(&[]).unwrap_err(), first);
    assert_eq!(first.qualification().unwrap().context(), context.identity());
}

#[test]
fn raw_input_and_temporary_capacity_limits_stop_whole_batch() {
    let (capture, subject, _) = capture(
        "progress",
        Namespace::html(),
        attributes(&[("value", "12345")]),
    );
    let context = context(&capture);
    let request = HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Progress);
    let mut input = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(4),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(matches!(
        input
            .query(std::slice::from_ref(&request))
            .unwrap_err()
            .kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let cap = std::mem::size_of::<HtmlSemanticResult>() + 4;
    let limited = HtmlSemanticLimits::new(1_000_000, 1, 0, 0, cap, 0);
    let mut allocation = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(10),
        limited,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = allocation
        .query(std::slice::from_ref(&request))
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::AllocatedBytes,
            ..
        }
    ));
    assert_eq!(allocation.query(&[]).unwrap_err(), failure);
}

#[test]
fn numeric_conversion_rejection_still_counts_temporary_capacity_without_refund() {
    let (capture, subject, _) = capture(
        "progress",
        Namespace::html(),
        attributes(&[("value", "1e9999")]),
    );
    let context = context(&capture);
    let request = HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Progress);
    let mut session = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(10),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let first = session.query(std::slice::from_ref(&request)).unwrap();
    assert_eq!(progress(&first), Some([1., 0., 0.]));
    assert!(first.usage().allocated_bytes() >= std::mem::size_of::<HtmlSemanticResult>() + 6);
    let second = session.query(&[request]).unwrap();
    assert!(
        second.usage().allocated_bytes()
            >= first.usage().allocated_bytes() + std::mem::size_of::<HtmlSemanticResult>() + 6
    );
}

#[test]
fn work_and_output_caps_publish_no_partial_successful_batch() {
    let (capture, subject, _) = capture("meter", Namespace::html(), vec![]);
    let context = context(&capture);
    let request = HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Meter);
    for (selected, expected) in [
        (
            HtmlSemanticLimits::new(1, 2, 0, 0, 100_000, 0),
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(100_000, 1, 0, 0, 100_000, 0),
            HtmlSemanticResource::OutputRecords,
        ),
    ] {
        let mut session = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(32),
            selected,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let failure = session
            .query(&[request.clone(), request.clone()])
            .unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource, .. } if *resource == expected)
        );
        assert_eq!(session.query(&[]).unwrap_err(), failure);
    }
}
