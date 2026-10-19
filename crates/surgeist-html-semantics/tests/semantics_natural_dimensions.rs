use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlDimensionApplicability,
    HtmlDimensionAuthorSyntax, HtmlDimensionOmission, HtmlDimensionPair, HtmlDimensionTolerance,
    HtmlEmbeddedDimensionAuthorReadback, HtmlNaturalDimensionAxis, HtmlNaturalDimensionFact,
    HtmlNaturalDimensionFactsError, HtmlNaturalDimensionLimits, HtmlNaturalDimensionMissing,
    HtmlNaturalDimensionProducerId, HtmlNaturalDimensionRevision, HtmlNaturalDimensionUnsupported,
    HtmlNaturalDimensions, HtmlNaturalResourceId, HtmlNaturalResourceKey, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextError,
    HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticValue,
};

fn integer(s: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&s.into()).unwrap()
}
fn rational(n: &str, d: &str) -> HtmlNaturalDimensionAxis {
    HtmlNaturalDimensionAxis::KnownPresent(
        HtmlNonNegativeRational::new(integer(n), integer(d)).unwrap(),
    )
}
fn revision(selection: u64) -> HtmlNaturalDimensionRevision {
    HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 7, selection)
}
fn key() -> HtmlNaturalResourceKey {
    HtmlNaturalResourceKey::new(HtmlNaturalResourceId::fresh(), 11, 13)
}
fn fixture(local: &str, attrs: &[(&str, &str)]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 100000, 1000, 1000000, 1000, 10, 10, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let e = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, local),
            OrderedAttributes::new(
                attrs.iter().map(|(n, v)| {
                    Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)
                }),
            )
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_children(&e, &[]).unwrap();
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn fact_limits() -> HtmlNaturalDimensionLimits {
    HtmlNaturalDimensionLimits::new(1000, 10000000, 100000)
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 10000, 10000, 10000000, 100)
}
fn context(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    axes: Option<(HtmlNaturalDimensionAxis, HtmlNaturalDimensionAxis)>,
) -> HtmlSemanticContext {
    let rev = revision(17);
    let k = key();
    let expected = vec![HtmlNaturalDimensionFact::selection(
        id.clone(),
        Some(k.clone()),
    )];
    let obs = axes.map_or_else(Vec::new, |(w, h)| {
        vec![HtmlNaturalDimensionFact::observation(
            id.clone(),
            rev.clone(),
            k,
            w,
            h,
        )]
    });
    let facts = HtmlNaturalDimensions::new(
        c,
        c.host_facts_revision().clone(),
        rev.clone(),
        expected,
        obs,
        fact_limits(),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_natural_dimensions(rev, Some(facts)),
        100,
    )
    .unwrap()
}
fn empty_context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        100,
    )
    .unwrap()
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(
        id.clone(),
        HtmlSemanticQuery::EmbeddedDimensionAuthorConformance,
    )
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancellation: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(raw), l, cancellation).unwrap()
}
fn run(c: &HtmlCapture, cx: &HtmlSemanticContext, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    session(c, cx, limits(), 10000, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn result(r: &HtmlSemanticReport) -> &HtmlEmbeddedDimensionAuthorReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::EmbeddedDimensionAuthorConformance(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}

#[test]
fn rational_rejects_zero_denominator_and_normalizes_zero_without_false_structural_equality() {
    assert!(matches!(
        HtmlNonNegativeRational::new(1u128.into(), 0u128.into()),
        Err(HtmlRationalError::ZeroDenominator)
    ));
    let zero =
        HtmlNonNegativeRational::new(0u128.into(), integer("9999999999999999999999999")).unwrap();
    assert_eq!(zero.numerator().decimal_digits(), "0");
    assert_eq!(zero.denominator().decimal_digits(), "1");
}
#[test]
fn literal_half_boundaries_and_outside_values_use_independent_alternatives() {
    use HtmlDimensionPair::*;
    use HtmlDimensionTolerance::*;
    for (nw, nh, w, h, expected) in [
        ("2", "1", "3", "2", Conforming(Height)),
        ("2", "1", "6", "2", Nonconforming),
        ("1", "2", "2", "3", Conforming(Width)),
        ("1", "2", "1", "3", Conforming(Width)),
        ("3", "2", "2", "1", Conforming(Width)),
        ("3", "2", "1", "1", Conforming(Width)),
        ("3", "2", "3", "1", Nonconforming),
        ("2", "1", "5", "2", Conforming(Height)),
        ("2", "1", "2", "2", Nonconforming),
        ("2", "1", "4", "2", Conforming(Width)),
        ("2", "1", "0", "1", Nonconforming),
    ] {
        let (c, id) = fixture("img", &[("width", w), ("height", h)]);
        let cx = context(&c, &id, Some((rational(nw, "1"), rational(nh, "1"))));
        let r = run(&c, &cx, &id);
        assert_eq!(result(&r).pair(), expected, "natural {nw}/{nh}, {w}/{h}");
        assert_eq!(
            result(&r).omission(),
            HtmlDimensionOmission::PresentDimensions
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}
#[test]
fn fractional_unreduced_dimensions_and_carries_are_exact() {
    for (wn, wd, hn, hd) in [
        ("3", "2", "3", "4"),
        ("3", "6", "2", "8"),
        (
            "9999999999999999999998",
            "9999999999999999999999",
            "4999999999999999999999",
            "9999999999999999999999",
        ),
    ] {
        let (c, id) = fixture("img", &[("width", "0003"), ("height", "0002")]);
        let cx = context(&c, &id, Some((rational(wn, wd), rational(hn, hd))));
        assert_eq!(
            result(&run(&c, &cx, &id)).pair(),
            HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Height)
        );
    }
}
#[test]
fn huge_equal_values_pass_and_one_unit_difference_fails_beyond_binary64() {
    for (w, h, expected) in [
        (
            "9999999999999999999999999999999999999999",
            "9999999999999999999999999999999999999999",
            HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Width),
        ),
        (
            "9999999999999999999999999999999999999999",
            "9999999999999999999999999999999999999998",
            HtmlDimensionPair::Nonconforming,
        ),
        (
            "9999999999999999999999999999999999999998",
            "9999999999999999999999999999999999999999",
            HtmlDimensionPair::Nonconforming,
        ),
    ] {
        let (c, id) = fixture("object", &[("width", w), ("height", h)]);
        let cx = context(&c, &id, Some((rational("1", "1"), rational("1", "1"))));
        assert_eq!(result(&run(&c, &cx, &id)).pair(), expected);
    }
}
#[test]
fn omission_and_pair_shortcuts_remain_independent_of_missing_or_absent_measurements() {
    let (c, id) = fixture("img", &[]);
    let cx = empty_context(&c);
    let r = run(&c, &cx, &id);
    assert_eq!(result(&r).omission(), HtmlDimensionOmission::OmittedBoth);
    assert_eq!(result(&r).pair(), HtmlDimensionPair::NotRequired);
    assert!(result(&r).resource().is_none());
    for axes in [
        None,
        Some((
            HtmlNaturalDimensionAxis::KnownAbsent,
            HtmlNaturalDimensionAxis::Unavailable,
        )),
        Some((
            HtmlNaturalDimensionAxis::KnownPresentButUnrepresentable(
                HtmlNaturalDimensionUnsupported::NonFinite,
            ),
            rational("0", "1"),
        )),
    ] {
        let (c, id) = fixture("video", &[("width", "0"), ("height", "0")]);
        let cx = context(&c, &id, axes.clone());
        let r = run(&c, &cx, &id);
        assert_eq!(
            result(&r).pair(),
            HtmlDimensionPair::Conforming(HtmlDimensionTolerance::BothZero)
        );
        assert_eq!(r.usage().peak_scratch_records(), 0);
        let expected = match axes {
            None => HtmlDimensionOmission::Unavailable(HtmlNaturalDimensionMissing::Observation),
            Some((HtmlNaturalDimensionAxis::KnownAbsent, _)) => HtmlDimensionOmission::Violated,
            _ => HtmlDimensionOmission::PresentDimensions,
        };
        assert_eq!(result(&r).omission(), expected);
    }
    let (c, id) = fixture("img", &[("width", "17")]);
    let cx = context(
        &c,
        &id,
        Some((
            HtmlNaturalDimensionAxis::KnownAbsent,
            HtmlNaturalDimensionAxis::Unavailable,
        )),
    );
    let r = run(&c, &cx, &id);
    assert_eq!(result(&r).pair(), HtmlDimensionPair::NotRequired);
    assert_eq!(result(&r).omission(), HtmlDimensionOmission::Violated);
}
#[test]
fn whole_author_grammar_does_not_use_recovered_prefixes_and_retains_both_errors() {
    for attrs in [
        [("width", "+2tail"), ("height", "-0")],
        [("width", ""), ("height", " 2")],
    ] {
        let (c, id) = fixture("img", &attrs);
        let cx = context(
            &c,
            &id,
            Some((
                HtmlNaturalDimensionAxis::KnownAbsent,
                HtmlNaturalDimensionAxis::Unavailable,
            )),
        );
        let r = run(&c, &cx, &id);
        assert_eq!(
            result(&r).width_syntax(),
            HtmlDimensionAuthorSyntax::Invalid
        );
        assert_eq!(
            result(&r).height_syntax(),
            HtmlDimensionAuthorSyntax::Invalid
        );
        assert_eq!(result(&r).pair(), HtmlDimensionPair::InvalidAuthoredInputs);
        assert_eq!(result(&r).omission(), HtmlDimensionOmission::Violated);
        assert_eq!(
            result(&r).width_attribute().unwrap().value(),
            &DocumentString::from(attrs[0].1)
        );
    }
}
#[test]
fn degenerate_and_unrepresentable_presence_never_becomes_absence_or_infinity_arithmetic() {
    for (nw, nh, w, expected) in [
        (
            rational("0", "1"),
            rational("5", "2"),
            "0",
            HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Width),
        ),
        (
            rational("0", "1"),
            rational("5", "2"),
            "1",
            HtmlDimensionPair::UnsupportedNaturalArithmetic,
        ),
        (
            rational("2", "1"),
            rational("0", "1"),
            "1",
            HtmlDimensionPair::UnsupportedNaturalArithmetic,
        ),
        (
            rational("0", "1"),
            rational("0", "1"),
            "1",
            HtmlDimensionPair::UnsupportedNaturalArithmetic,
        ),
        (
            HtmlNaturalDimensionAxis::KnownPresentButUnrepresentable(
                HtmlNaturalDimensionUnsupported::NonRational,
            ),
            rational("2", "1"),
            "1",
            HtmlDimensionPair::UnsupportedNaturalArithmetic,
        ),
    ] {
        let (c, id) = fixture("embed", &[("width", w), ("height", "7")]);
        let cx = context(&c, &id, Some((nw, nh)));
        let r = run(&c, &cx, &id);
        assert_eq!(result(&r).pair(), expected);
        assert_eq!(
            result(&r).omission(),
            HtmlDimensionOmission::PresentDimensions
        );
    }
}
#[test]
fn missing_selection_observation_and_axes_have_distinct_influencing_causes() {
    let (c, id) = fixture("img", &[("width", "3"), ("height", "2")]);
    let cases = [
        (
            Some((HtmlNaturalDimensionAxis::Unavailable, rational("1", "1"))),
            HtmlNaturalDimensionMissing::Width,
        ),
        (
            Some((rational("1", "1"), HtmlNaturalDimensionAxis::Unavailable)),
            HtmlNaturalDimensionMissing::Height,
        ),
        (None, HtmlNaturalDimensionMissing::Observation),
    ];
    for (axes, gap) in cases {
        let cx = context(&c, &id, axes);
        let r = run(&c, &cx, &id);
        assert_eq!(
            result(&r).omission(),
            HtmlDimensionOmission::Unavailable(gap)
        );
        assert_eq!(result(&r).pair(), HtmlDimensionPair::Unavailable(gap));
    }
    let rev = revision(1);
    let facts = HtmlNaturalDimensions::new(
        &c,
        c.host_facts_revision().clone(),
        rev.clone(),
        vec![HtmlNaturalDimensionFact::selection(id.clone(), None)],
        vec![],
        fact_limits(),
    )
    .unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_natural_dimensions(rev, Some(facts)),
        100,
    )
    .unwrap();
    assert_eq!(
        result(&run(&c, &cx, &id)).pair(),
        HtmlDimensionPair::Unavailable(HtmlNaturalDimensionMissing::Selection)
    );
    assert_eq!(
        result(&run(&c, &empty_context(&c), &id)).pair(),
        HtmlDimensionPair::Unavailable(HtmlNaturalDimensionMissing::Supplement)
    );
}
#[test]
fn resource_content_metadata_and_selection_revision_mismatches_reject_the_join() {
    let (c, id) = fixture("img", &[]);
    let rev = revision(2);
    let expected = key();
    for actual in [
        key(),
        HtmlNaturalResourceKey::new(expected.resource().clone(), 12, 13),
        HtmlNaturalResourceKey::new(expected.resource().clone(), 11, 14),
    ] {
        assert!(matches!(
            HtmlNaturalDimensions::new(
                &c,
                c.host_facts_revision().clone(),
                rev.clone(),
                vec![HtmlNaturalDimensionFact::selection(
                    id.clone(),
                    Some(expected.clone())
                )],
                vec![HtmlNaturalDimensionFact::observation(
                    id.clone(),
                    rev.clone(),
                    actual,
                    rational("1", "1"),
                    rational("1", "1")
                )],
                fact_limits()
            ),
            Err(HtmlNaturalDimensionFactsError::ResourceMismatch(_))
        ));
    }
    for stale in [
        HtmlNaturalDimensionRevision::new(rev.producer().clone(), 7, 1),
        HtmlNaturalDimensionRevision::new(rev.producer().clone(), 8, 2),
        revision(2),
    ] {
        assert!(matches!(
            HtmlNaturalDimensions::new(
                &c,
                c.host_facts_revision().clone(),
                rev.clone(),
                vec![HtmlNaturalDimensionFact::selection(
                    id.clone(),
                    Some(expected.clone())
                )],
                vec![HtmlNaturalDimensionFact::observation(
                    id.clone(),
                    stale,
                    expected.clone(),
                    rational("1", "1"),
                    rational("1", "1")
                )],
                fact_limits()
            ),
            Err(HtmlNaturalDimensionFactsError::SelectionRevisionMismatch(_))
        ));
    }
}
#[test]
fn duplicate_foreign_subject_host_and_context_revision_are_checked_eagerly() {
    let (c, id) = fixture("img", &[]);
    let (other, oid) = fixture("img", &[]);
    let rev = revision(1);
    assert!(matches!(
        HtmlNaturalDimensions::new(
            &c,
            c.host_facts_revision().clone(),
            rev.clone(),
            vec![
                HtmlNaturalDimensionFact::selection(id.clone(), None),
                HtmlNaturalDimensionFact::selection(id.clone(), None)
            ],
            vec![],
            fact_limits()
        ),
        Err(HtmlNaturalDimensionFactsError::DuplicateSubject(_))
    ));
    assert!(matches!(
        HtmlNaturalDimensions::new(
            &c,
            c.host_facts_revision().clone(),
            rev.clone(),
            vec![HtmlNaturalDimensionFact::selection(oid, None)],
            vec![],
            fact_limits()
        ),
        Err(HtmlNaturalDimensionFactsError::Capture(_))
    ));
    assert!(matches!(
        HtmlNaturalDimensions::new(
            &c,
            other.host_facts_revision().clone(),
            rev.clone(),
            vec![],
            vec![],
            fact_limits()
        ),
        Err(HtmlNaturalDimensionFactsError::HostRevisionMismatch)
    ));
    let facts = HtmlNaturalDimensions::new(
        &c,
        c.host_facts_revision().clone(),
        rev,
        vec![],
        vec![],
        fact_limits(),
    )
    .unwrap();
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_natural_dimensions(revision(2), Some(facts)),
            100
        ),
        Err(HtmlSemanticContextError::NaturalDimensionsRevisionMismatch)
    ));
}
#[test]
fn constructor_caps_actual_retained_capacity_bytes_and_validation_work() {
    let (c, id) = fixture("img", &[]);
    for (l, expected) in [
        (
            HtmlNaturalDimensionLimits::new(0, 10000, 100),
            HtmlNaturalDimensionFactsError::RecordLimit,
        ),
        (
            HtmlNaturalDimensionLimits::new(10, 0, 100),
            HtmlNaturalDimensionFactsError::ByteLimit,
        ),
        (
            HtmlNaturalDimensionLimits::new(10, 10000, 0),
            HtmlNaturalDimensionFactsError::WorkLimit,
        ),
    ] {
        assert_eq!(
            HtmlNaturalDimensions::new(
                &c,
                c.host_facts_revision().clone(),
                revision(1),
                vec![HtmlNaturalDimensionFact::selection(id.clone(), None)],
                vec![],
                l
            )
            .unwrap_err(),
            expected
        );
    }
}
#[test]
fn successful_reuse_cumulative_usage_and_old_report_survival_use_the_actual_session() {
    let (c, id) = fixture("img", &[("width", "3"), ("height", "2")]);
    let cx = context(&c, &id, Some((rational("2", "1"), rational("1", "1"))));
    let token = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 100, token.clone());
    let first = s.query(&[request(&id)]).unwrap();
    let second = s.query(&[request(&id)]).unwrap();
    assert_eq!(first.results(), second.results());
    assert!(second.usage().work() > first.usage().work());
    assert!(second.usage().allocated_bytes() > first.usage().allocated_bytes());
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(
        first.qualification().natural_dimension_revision(),
        cx.natural_dimension_revision()
    );
    assert!(result(&first).resource().is_some());
    token.cancel();
    let err = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(err.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[request(&id)]).unwrap_err(), err);
    assert_eq!(
        result(&first).pair(),
        HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Height)
    );
}
#[test]
fn every_real_work_cut_discards_the_batch_releases_scratch_and_preserves_first_stop() {
    let (c, id) = fixture("img", &[("width", "39"), ("height", "20")]);
    let cx = context(&c, &id, Some((rational("2", "1"), rational("1", "1"))));
    let req = [request(&id), request(&id)];
    let full = session(&c, &cx, limits(), 100, HtmlSemanticCancellation::new())
        .query(&req)
        .unwrap();
    let needed = full.usage().work();
    let mut saw_scratch = false;
    for work in 0..needed {
        let mut s = session(
            &c,
            &cx,
            HtmlSemanticLimits::new(work, 100, 10000, 10000, 10000000, 100),
            100,
            HtmlSemanticCancellation::new(),
        );
        let err = s.query(&req).unwrap_err();
        assert!(matches!(
            err.kind(),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                ..
            }
        ));
        saw_scratch |= err.usage().scratch_records() > 0;
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.query(&req).unwrap_err(), err);
    }
    assert!(saw_scratch);
    let mut exact = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(needed, 100, 10000, 10000, 10000000, 100),
        100,
        HtmlSemanticCancellation::new(),
    );
    assert!(exact.query(&req).is_ok());
}
#[test]
fn real_scratch_digit_byte_and_raw_caps_stop_without_conformance_or_partial_results() {
    let (c, id) = fixture("img", &[("width", "99"), ("height", "49")]);
    let cx = context(&c, &id, Some((rational("2", "1"), rational("1", "1"))));
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(10000000, 100, 0, 10000, 10000000, 100),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(10000000, 100, 10000, 1, 10000000, 100),
            HtmlSemanticResource::IntegerDigits,
        ),
        (
            HtmlSemanticLimits::new(10000000, 100, 10000, 10000, 0, 100),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut s = session(&c, &cx, l, 100, HtmlSemanticCancellation::new());
        let e = s.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded { resource:r,.. } if *r==resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.query(&[request(&id)]).unwrap_err(), e);
    }
    let (c, id) = fixture("canvas", &[("width", "9999"), ("height", "0")]);
    let cx = empty_context(&c);
    let mut s = session(&c, &cx, limits(), 3, HtmlSemanticCancellation::new());
    assert!(matches!(
        s.query(&[request(&id)]).unwrap_err().kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}

#[test]
fn applicability_uses_actual_input_state_and_preserves_unapplied_local_spelling() {
    for (local, attrs, expected) in [
        (
            "input",
            vec![("type", "ImAgE"), ("width", "0"), ("height", "0")],
            HtmlDimensionApplicability::Applicable,
        ),
        (
            "input",
            vec![("type", "text"), ("width", "-0"), ("height", "+1")],
            HtmlDimensionApplicability::Inapplicable,
        ),
        (
            "input",
            vec![("type", "unknown"), ("width", "-0")],
            HtmlDimensionApplicability::Inapplicable,
        ),
        (
            "canvas",
            vec![("width", "-0"), ("height", "+1")],
            HtmlDimensionApplicability::Inapplicable,
        ),
        (
            "div",
            vec![("width", "-0")],
            HtmlDimensionApplicability::Inapplicable,
        ),
        (
            "source",
            vec![("width", "-0")],
            HtmlDimensionApplicability::Inapplicable,
        ),
    ] {
        let (c, id) = fixture(local, &attrs);
        let cx = empty_context(&c);
        let r = run(&c, &cx, &id);
        assert_eq!(result(&r).applicability(), &expected);
        if expected == HtmlDimensionApplicability::Inapplicable {
            assert_eq!(
                result(&r).width_syntax(),
                HtmlDimensionAuthorSyntax::Unapplied
            );
            assert_eq!(result(&r).pair(), HtmlDimensionPair::NotApplicable);
        }
    }
}
#[test]
fn source_uses_actual_picture_parent_foreign_name_and_unavailable_parent_are_distinct() {
    for (parent, ns, missing, expected) in [
        (
            "picture",
            Namespace::html(),
            false,
            HtmlDimensionApplicability::Applicable,
        ),
        (
            "div",
            Namespace::html(),
            false,
            HtmlDimensionApplicability::Inapplicable,
        ),
        (
            "picture",
            Namespace::svg(),
            false,
            HtmlDimensionApplicability::Inapplicable,
        ),
        (
            "picture",
            Namespace::html(),
            true,
            HtmlDimensionApplicability::Inapplicable,
        ),
    ] {
        let mut b = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let d = b
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let p = b
            .add_element(
                &d,
                QualifiedName::new(ns, None, parent),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let e = b
            .add_element(
                &d,
                QualifiedName::new(Namespace::html(), None, "source"),
                OrderedAttributes::new([
                    Attribute::new(QualifiedName::new(Namespace::none(), None, "width"), "0"),
                    Attribute::new(QualifiedName::new(Namespace::none(), None, "height"), "0"),
                ])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        b.establish_children(&e, &[]).unwrap();
        if !missing {
            b.establish_children(&p, std::slice::from_ref(&e)).unwrap();
        }
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        let cx = empty_context(&c);
        let r = run(&c, &cx, &id);
        if missing {
            assert_eq!(
                result(&r).applicability(),
                &HtmlDimensionApplicability::Unavailable(HtmlSemanticMissing::Parent(id))
            );
            assert_eq!(
                result(&r).pair(),
                HtmlDimensionPair::ApplicabilityUnavailable
            );
        } else {
            assert_eq!(result(&r).applicability(), &expected);
        }
    }
}
#[test]
fn established_foreign_subject_and_namespaced_dimension_are_not_html_geometry() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::svg(), None, "img"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let html = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "img"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "width"),
                "9",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let fid = c.subject_id(&foreign).unwrap();
    let hid = c.subject_id(&html).unwrap();
    let cx = empty_context(&c);
    assert_eq!(
        result(&run(&c, &cx, &fid)).applicability(),
        &HtmlDimensionApplicability::Inapplicable
    );
    assert_eq!(
        result(&run(&c, &cx, &hid)).omission(),
        HtmlDimensionOmission::OmittedBoth
    );
}

#[test]
fn isolated_surrogate_author_value_is_preserved_invalid_and_cannot_supply_a_prefix() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let raw = DocumentString::from_utf16(vec![u16::from(b'2'), 0xd800]);
    let e = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "img"),
            OrderedAttributes::new([
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "width"),
                    raw.clone(),
                ),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "height"), "1"),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c, &id, Some((rational("2", "1"), rational("1", "1"))));
    let r = run(&c, &cx, &id);
    assert_eq!(result(&r).width_attribute().unwrap().value(), &raw);
    assert_eq!(
        result(&r).width_syntax(),
        HtmlDimensionAuthorSyntax::Invalid
    );
    assert_eq!(result(&r).pair(), HtmlDimensionPair::InvalidAuthoredInputs);
    assert_eq!(
        result(&r).omission(),
        HtmlDimensionOmission::PresentDimensions
    );
}
#[test]
fn actual_successful_peak_and_byte_admissions_accept_exact_capacity_and_refuse_one_less() {
    let (c, id) = fixture("img", &[("width", "99"), ("height", "49")]);
    let cx = context(&c, &id, Some((rational("2", "1"), rational("1", "1"))));
    let full = run(&c, &cx, &id);
    let peak = full.usage().peak_scratch_records();
    let bytes = full.usage().allocated_bytes();
    let mut exact = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(10000000, 100, peak, 10000, bytes, 100),
        100,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        result(&exact.query(&[request(&id)]).unwrap()).pair(),
        result(&full).pair()
    );
    assert_eq!(exact.usage().scratch_records(), 0);
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(10000000, 100, peak - 1, 10000, bytes, 100),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(10000000, 100, peak, 10000, bytes - 1, 100),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut s = session(&c, &cx, l, 100, HtmlSemanticCancellation::new());
        let e = s.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded { resource:r,.. } if *r==resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn completed_report_remains_qualified_to_its_old_resource_when_a_new_context_selects_another_resource()
 {
    let (c, id) = fixture("img", &[("width", "3"), ("height", "2")]);
    let old = context(&c, &id, Some((rational("2", "1"), rational("1", "1"))));
    let first = run(&c, &old, &id);
    let new = context(&c, &id, Some((rational("1", "1"), rational("1", "1"))));
    let second = run(&c, &new, &id);
    assert_ne!(first.context(), second.context());
    assert_ne!(result(&first).resource(), result(&second).resource());
    assert_eq!(
        result(&first).pair(),
        HtmlDimensionPair::Conforming(HtmlDimensionTolerance::Height)
    );
    assert_eq!(result(&second).pair(), HtmlDimensionPair::Nonconforming);
    assert_eq!(
        first.qualification().natural_dimension_revision(),
        old.natural_dimension_revision()
    );
    assert_eq!(
        second.qualification().natural_dimension_revision(),
        new.natural_dimension_revision()
    );
}
