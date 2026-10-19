use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlAdvisoryInformation, HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject,
    HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedNodeKind, HtmlCapturedShadowProperties, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue,
};
#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 100000, 1000, 1000000, 1000, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn attr(name: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    expanded(
        b,
        d,
        QualifiedName::new(Namespace::html(), None, name),
        attrs,
    )
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        OrderedAttributes::new(attrs).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn fixture(
    local: &str,
    title: Option<DocumentString>,
    root: bool,
) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        local,
        title.map(|s| attr("title", s)).into_iter().collect(),
    );
    if root {
        b.establish_root(&e).unwrap();
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}

fn limits(work: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(work, 100, 0, 0, 1000000, 10000)
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::AdvisoryInformation)
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId, raw: usize) -> HtmlSemanticReport {
    session(
        c,
        &context(c),
        limits(1000000),
        raw,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(id)])
    .unwrap()
}
fn value(report: &HtmlSemanticReport) -> &HtmlSemanticValue<HtmlAdvisoryInformation> {
    match report.results()[0].fact() {
        HtmlSemanticFact::AdvisoryInformation(v) => v,
        other => panic!("unexpected fact: {other:?}"),
    }
}
fn known(report: &HtmlSemanticReport) -> &HtmlAdvisoryInformation {
    match value(report) {
        HtmlSemanticValue::Known(v) => v,
        other => panic!("unexpected advisory outcome: {other:?}"),
    }
}

#[test]
fn own_title_normalizes_newlines_and_retains_exact_raw_units() {
    let raw = vec![
        32, 13, 10, 13, 13, 10, 10, 0, 0xd83d, 0xde00, 0xd800, 65, 0xdc00, 32,
    ];
    let expected = [
        32, 10, 10, 10, 10, 0, 0xd83d, 0xde00, 0xd800, 65, 0xdc00, 32,
    ];
    let (c, id) = fixture("p", Some(DocumentString::from_utf16(raw.clone())), false);
    let r = report(&c, &id, raw.len());
    assert_eq!(known(&r).value().code_units(), expected);
    assert_eq!(known(&r).source(), Some(&id));
    let HtmlCapturedNodeKind::Element(e) = c.node(&id).unwrap().kind() else {
        panic!()
    };
    assert_eq!(
        e.attributes().iter().next().unwrap().value().code_units(),
        raw
    );
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
}

#[test]
fn selected_empty_and_known_absence_have_distinct_source_evidence() {
    for title in [Some(DocumentString::from("")), None] {
        let (c, id) = fixture("p", title.clone(), true);
        let r = report(&c, &id, 0);
        assert!(known(&r).value().is_empty());
        assert_eq!(known(&r).source(), title.as_ref().map(|_| &id));
        assert_eq!(r.usage().nested_output_records(), 0);
    }
}

#[test]
fn nearest_selected_title_wins_including_empty_and_unknown_parent_beyond_it() {
    for title in ["", "nearest\r\nline"] {
        let (mut b, d) = builder();
        let higher = element(&mut b, &d, "div", vec![attr("title", "oversized ancestor")]);
        let near = element(&mut b, &d, "section", vec![attr("title", title)]);
        let child = element(&mut b, &d, "span", vec![]);
        b.establish_children(&higher, std::slice::from_ref(&near))
            .unwrap();
        b.establish_children(&near, std::slice::from_ref(&child))
            .unwrap();
        let c = b.finish().unwrap();
        let child = c.subject_id(&child).unwrap();
        let near = c.subject_id(&near).unwrap();
        let r = report(&c, &child, title.len());
        assert_eq!(
            known(&r).value().to_utf8().unwrap(),
            title.replace("\r\n", "\n")
        );
        assert_eq!(known(&r).source(), Some(&near));
    }
}

#[test]
fn own_empty_discharges_unknown_parent_and_irrelevant_children_and_host() {
    let (c, id) = fixture("input", Some("".into()), false);
    let r = report(&c, &id, 0);
    assert!(known(&r).value().is_empty());
    assert_eq!(known(&r).source(), Some(&id));
}

#[test]
fn absent_title_with_unavailable_parent_is_missing_not_empty() {
    let (c, id) = fixture("p", None, false);
    assert_eq!(
        value(&report(&c, &id, 0)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))
    );
}

#[test]
fn undefined_html_local_names_participate_without_a_vocabulary_gate() {
    let (mut b, d) = builder();
    let parent = element(&mut b, &d, "custom-ancestor", vec![attr("title", "custom")]);
    let child = element(&mut b, &d, "unknown-child", vec![]);
    b.establish_children(&parent, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&child).unwrap(), 6);
    assert_eq!(known(&r).value().to_utf8().unwrap(), "custom");
    assert_eq!(known(&r).source(), Some(&c.subject_id(&parent).unwrap()));
}

#[test]
fn attribute_namespace_and_exact_local_name_select_title_prefix_does_not() {
    let (mut b, d) = builder();
    let attrs = vec![
        attr("TITLE", "wrong"),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:foreign"), None, "title"),
            "wrong",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::none(), Some("prefix".into()), "title"),
            "right",
        ),
    ];
    let e = expanded(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("h".into()), "p"),
        attrs,
    );
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    assert_eq!(
        known(&report(&c, &id, 5)).value().to_utf8().unwrap(),
        "right"
    );
    let (mut b, d) = builder();
    let parent = element(&mut b, &d, "p", vec![attr("title", "inherited")]);
    let child = element(&mut b, &d, "span", vec![attr("TITLE", "wrong")]);
    b.establish_children(&parent, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        known(&report(&c, &c.subject_id(&child).unwrap(), 9))
            .value()
            .to_utf8()
            .unwrap(),
        "inherited"
    );
}

#[test]
fn foreign_ancestor_is_an_ordinary_bridge_not_a_global_title_or_exception() {
    for local in ["link", "style", "bogus"] {
        let (mut b, d) = builder();
        let parent = element(&mut b, &d, "p", vec![attr("title", "outer")]);
        let bridge = expanded(
            &mut b,
            &d,
            QualifiedName::new(Namespace::svg(), None, local),
            vec![attr("title", "ignored oversized")],
        );
        let child = element(&mut b, &d, "span", vec![]);
        b.establish_children(&parent, std::slice::from_ref(&bridge))
            .unwrap();
        b.establish_children(&bridge, std::slice::from_ref(&child))
            .unwrap();
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&child).unwrap(), 5);
        assert_eq!(known(&r).value().to_utf8().unwrap(), "outer");
        assert_eq!(known(&r).source(), Some(&c.subject_id(&parent).unwrap()));
        assert_eq!(
            value(&report(&c, &c.subject_id(&bridge).unwrap(), 0)),
            &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
        );
    }
}

#[test]
fn titleless_link_and_style_stop_even_with_unknown_parent_and_any_rel() {
    for local in ["link", "style"] {
        for rel in [
            None,
            Some("stylesheet"),
            Some("alternate stylesheet"),
            Some("nonsense"),
        ] {
            let (mut b, d) = builder();
            let attrs = rel.map(|r| attr("rel", r)).into_iter().collect();
            let e = element(&mut b, &d, local, attrs);
            let c = b.finish().unwrap();
            let r = report(&c, &c.subject_id(&e).unwrap(), 0);
            assert!(known(&r).value().is_empty());
            assert_eq!(known(&r).source(), None);
        }
    }
}

#[test]
fn boundary_roles_use_exact_html_expanded_names_with_prefix_ignored() {
    for local in ["LINK", "Link", "sTYle"] {
        let (c, id) = fixture(local, None, false);
        assert_eq!(
            value(&report(&c, &id, 0)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))
        );
    }
    let (mut b, d) = builder();
    let e = expanded(
        &mut b,
        &d,
        QualifiedName::new(
            Namespace::from_uri("http://www.w3.org/1999/xhtml"),
            Some("unusual".into()),
            "link",
        ),
        vec![],
    );
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&e).unwrap(), 0);
    assert!(known(&r).value().is_empty());
    assert_eq!(known(&r).source(), None);
}

#[test]
fn descendant_stops_at_link_style_boundary_connected_or_detached() {
    for local in ["link", "style"] {
        for connected in [true, false] {
            let (mut b, d) = builder();
            let higher = element(&mut b, &d, "div", vec![attr("title", "irrelevant")]);
            let boundary = element(&mut b, &d, local, vec![]);
            let child = element(&mut b, &d, "p", vec![]);
            b.establish_children(&higher, std::slice::from_ref(&boundary))
                .unwrap();
            b.establish_children(&boundary, std::slice::from_ref(&child))
                .unwrap();
            if connected {
                b.establish_children(&d, std::slice::from_ref(&higher))
                    .unwrap();
            } else {
                b.establish_root(&higher).unwrap();
            }
            let c = b.finish().unwrap();
            for e in [&boundary, &child] {
                let r = report(&c, &c.subject_id(e).unwrap(), 0);
                assert!(known(&r).value().is_empty());
                assert_eq!(known(&r).source(), None);
            }
        }
    }
}

#[test]
fn link_and_style_own_empty_or_nonempty_title_wins_before_the_boundary() {
    for local in ["link", "style"] {
        for title in ["", "own\rtitle"] {
            let (mut b, d) = builder();
            let e = element(&mut b, &d, local, vec![attr("title", title)]);
            let child = element(&mut b, &d, "p", vec![]);
            b.establish_children(&e, std::slice::from_ref(&child))
                .unwrap();
            let c = b.finish().unwrap();
            for target in [&e, &child] {
                let r = report(&c, &c.subject_id(target).unwrap(), title.len());
                assert_eq!(
                    known(&r).value().to_utf8().unwrap(),
                    title.replace('\r', "\n")
                );
                assert_eq!(known(&r).source(), Some(&c.subject_id(&e).unwrap()));
            }
        }
    }
}

#[test]
fn unavailable_parent_before_any_boundary_remains_influencing() {
    let (mut b, d) = builder();
    let unused_boundary = element(&mut b, &d, "link", vec![]);
    let child = element(&mut b, &d, "p", vec![]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&child).unwrap();
    assert_eq!(
        value(&report(&c, &id, 0)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))
    );
    assert!(
        known(&report(&c, &c.subject_id(&unused_boundary).unwrap(), 0))
            .value()
            .is_empty()
    );
}

#[test]
fn document_shadow_and_template_fragment_parents_stop_without_host_substitution() {
    let (mut b, d) = builder();
    let ordinary = element(&mut b, &d, "p", vec![]);
    b.establish_children(&d, std::slice::from_ref(&ordinary))
        .unwrap();
    let host = element(&mut b, &d, "div", vec![attr("title", "host title")]);
    let shadow = b
        .add_shadow_root(
            &d,
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let shadow_child = element(&mut b, &d, "slot", vec![]);
    b.establish_children(&shadow, std::slice::from_ref(&shadow_child))
        .unwrap();
    let template = element(
        &mut b,
        &d,
        "template",
        vec![attr("title", "template title")],
    );
    let contents = b.add_fragment(&d).unwrap();
    b.establish_template_contents(&template, &contents).unwrap();
    let template_child = element(&mut b, &d, "p", vec![]);
    b.establish_children(&contents, std::slice::from_ref(&template_child))
        .unwrap();
    let plain = b.add_fragment(&d).unwrap(); // Its host is explicitly unavailable.
    let plain_child = element(&mut b, &d, "p", vec![]);
    b.establish_children(&plain, std::slice::from_ref(&plain_child))
        .unwrap();
    let c = b.finish().unwrap();
    for e in [ordinary, shadow_child, template_child, plain_child] {
        let r = report(&c, &c.subject_id(&e).unwrap(), 0);
        assert!(known(&r).value().is_empty());
        assert_eq!(known(&r).source(), None);
    }
}

#[test]
fn non_elements_are_inapplicable_and_source_clause_names_the_actual_algorithm() {
    let (mut b, d) = builder();
    let text = b.add_text(&d, "title".into()).unwrap();
    let fragment = b.add_fragment(&d).unwrap();
    let c = b.finish().unwrap();
    for e in [d, text, fragment] {
        assert_eq!(
            value(&report(&c, &c.subject_id(&e).unwrap(), 0)),
            &HtmlSemanticValue::Inapplicable
        );
    }
    assert_eq!(
        HtmlSemanticQuery::AdvisoryInformation.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#advisory-information"
    );
}

#[test]
fn whole_selected_raw_cap_precedes_processing_and_ignores_unselected_values() {
    let raw = DocumentString::from_utf16(vec![13, 10, 13, 10]);
    let (c, id) = fixture("p", Some(raw), false);
    let cx = context(&c);
    let r = report(&c, &id, 4);
    assert_eq!(known(&r).value().code_units(), [10, 10]);
    let mut s = session(&c, &cx, limits(1000), 3, HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&id)]).unwrap_err();
    let HtmlSemanticFailureKind::Attribute(admission) = failure.kind() else {
        panic!("unexpected failure: {failure:?}")
    };
    assert_eq!(admission.capacity(), 3);
    assert_eq!(admission.required(), 4);
    assert_eq!(s.usage().nested_output_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}

#[test]
fn literal_work_costs_cover_exact_normalization_and_boundary_reads() {
    // Batch reserve1 + subject1 + namespace(1+28)=31. Own title lookup6,
    // presence1 + raw length/admission1=39; normalization U+2N+finish1;
    // record construction1 + batch slot1. Empty title:42; x:45;
    // A CR LF B:52. Absent p:31+presence1+name length1+parent1+empty1+slot1=36.
    // Absent link adds its four exact local-unit comparisons:39.
    for (local, title, work) in [
        ("p", Some(""), 42),
        ("p", Some("x"), 45),
        ("p", Some("A\r\nB"), 52),
        ("p", None, 36),
        ("link", None, 39),
    ] {
        let (c, id) = fixture(local, title.map(DocumentString::from), true);
        let cx = context(&c);
        let req = request(&id);
        let mut exact = session(&c, &cx, limits(work), 100, HtmlSemanticCancellation::new());
        let r = exact.query(std::slice::from_ref(&req)).unwrap();
        assert_eq!(r.usage().work(), work);
        assert_eq!(known(&r).source(), title.map(|_| &id));
        let mut short = session(
            &c,
            &cx,
            limits(work - 1),
            100,
            HtmlSemanticCancellation::new(),
        );
        let failure = short.query(&[req]).unwrap_err();
        assert_eq!(
            failure.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: work - 1,
                required: work
            }
        );
        assert_eq!(short.usage().scratch_records(), 0);
        assert_eq!(short.query(&[]).unwrap_err(), failure);
    }
}

#[test]
fn partial_normalization_stops_before_an_unadmitted_write_and_drops_local_storage() {
    let (c, id) = fixture("p", Some("AB".into()), false);
    let cx = context(&c);
    // Own selection39, A read/reserve/write3, B read/reserve2=44;
    // the B slot requires45 and must not execute. No scratch is introduced.
    let mut s = session(&c, &cx, limits(44), 2, HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            limit: 44,
            required: 45
        }
    );
    assert_eq!(failure.usage().work(), 44);
    assert_eq!(failure.usage().nested_output_records(), 2);
    assert_eq!(failure.usage().scratch_records(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    // The supplied immutable source remains exact; retained capacity accounting
    // is cumulative admission, not a claim that the dropped buffer is live.
    let HtmlCapturedNodeKind::Element(e) = c.node(&id).unwrap().kind() else {
        panic!()
    };
    assert_eq!(
        e.attributes().iter().next().unwrap().value().code_units(),
        [65, 66]
    );
}

#[test]
fn inherited_selection_charges_each_actual_parent_and_namespace_read() {
    let (mut b, d) = builder();
    let parent = element(&mut b, &d, "div", vec![attr("title", "x")]);
    let child = element(&mut b, &d, "p", vec![]);
    b.establish_children(&parent, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&child).unwrap();
    let cx = context(&c);
    // Initial31 + absent-title1 + nonboundary name length1 + parent1 +
    // parent lookup1 + kind1 + HTML namespace29 =65. Parent title lookup6,
    // presence1 + raw admission1 + x read/reserve/write3 + normalization finish1
    // + record construction1 + batch slot1 =14. Total79.
    let mut exact = session(&c, &cx, limits(79), 1, HtmlSemanticCancellation::new());
    let r = exact.query(&[request(&id)]).unwrap();
    assert_eq!(r.usage().work(), 79);
    assert_eq!(known(&r).source(), Some(&c.subject_id(&parent).unwrap()));
    assert_eq!(known(&r).value().code_units(), [120]);
    // With35 credits the parent lookup succeeds; the following kind read36
    // fails, so no attribute value or output buffer has been consulted/admitted.
    let mut short = session(&c, &cx, limits(35), 1, HtmlSemanticCancellation::new());
    let f = short.query(&[request(&id)]).unwrap_err();
    assert_eq!(
        f.kind(),
        &HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            limit: 35,
            required: 36
        }
    );
    assert_eq!(f.usage().nested_output_records(), 0);
    assert_eq!(short.query(&[]).unwrap_err(), f);
}

#[test]
fn actual_output_capacity_limits_and_repeated_queries_are_cumulative() {
    let (c, id) = fixture("p", Some("A\r\nB".into()), false);
    let cx = context(&c);
    let req = request(&id);
    let r = report(&c, &id, 4);
    let u = r.usage();
    assert_eq!(u.nested_output_records(), 3);
    assert_eq!(
        u.allocated_bytes(),
        u.output_records() * std::mem::size_of::<HtmlSemanticResult>()
            + 3 * std::mem::size_of::<u16>()
    );
    let make =
        |bytes, nested| HtmlSemanticLimits::new(10000, u.output_records(), 0, 0, bytes, nested);
    let mut s = session(
        &c,
        &cx,
        make(u.allocated_bytes(), 3),
        4,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        known(&s.query(std::slice::from_ref(&req)).unwrap())
            .value()
            .code_units(),
        [65, 10, 66]
    );
    for (l, resource) in [
        (
            make(u.allocated_bytes() - 1, 3),
            HtmlSemanticResource::AllocatedBytes,
        ),
        (
            make(u.allocated_bytes(), 2),
            HtmlSemanticResource::NestedOutputRecords,
        ),
        (
            HtmlSemanticLimits::new(1000, 0, 0, 0, 1000000, 3),
            HtmlSemanticResource::OutputRecords,
        ),
    ] {
        let mut s = session(&c, &cx, l, 4, HtmlSemanticCancellation::new());
        let f = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), f);
    }
    let mut s = session(&c, &cx, limits(10000), 4, HtmlSemanticCancellation::new());
    let first = s.query(std::slice::from_ref(&req)).unwrap();
    let second = s.query(&[req]).unwrap();
    assert_eq!(second.usage().nested_output_records(), 6);
    assert_eq!(second.usage().output_records(), 2 * u.output_records());
    assert_eq!(second.usage().allocated_bytes(), 2 * u.allocated_bytes());
    assert_eq!(known(&first).value().code_units(), [65, 10, 66]);
}

#[test]
fn late_raw_cap_and_cancellation_failures_are_atomic_sticky_and_leave_prior_reports_valid() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "p", vec![attr("title", "x")]);
    let z = element(&mut b, &d, "p", vec![attr("title", "long")]);
    let c = b.finish().unwrap();
    let a = c.subject_id(&a).unwrap();
    let z = c.subject_id(&z).unwrap();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(10000), 1, HtmlSemanticCancellation::new());
    let earlier = s.query(&[request(&a)]).unwrap();
    let failure = s.query(&[request(&a), request(&z)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(known(&earlier).value().code_units(), [120]);
    assert_eq!(s.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(10000), 100, cancel.clone());
    let earlier = s.query(&[request(&a)]).unwrap();
    let before = s.usage();
    cancel.cancel();
    let failure = s.query(&[request(&a)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.usage(), before);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
    assert_eq!(known(&earlier).value().code_units(), [120]);
}

#[test]
fn readbacks_retain_exact_qualification_and_foreign_subject_errors_never_become_missing() {
    let (c, id) = fixture("p", Some("x".into()), true);
    let cx = context(&c);
    let r = session(&c, &cx, limits(1000000), 1, HtmlSemanticCancellation::new())
        .query(&[request(&id)])
        .unwrap();
    assert_eq!(r.qualification().capture(), c.identity());
    assert_eq!(r.qualification().context(), cx.identity());
    assert_eq!(r.qualification().host_revision(), c.host_facts_revision());
    assert_eq!(r.qualification().attribute_input_limit(), 1);
    assert_eq!(r.qualification().limits(), limits(1000000));
    let (other, foreign) = fixture("p", Some("y".into()), true);
    assert!(
        matches!(HtmlSemanticSession::new(&other,&cx,HtmlValueInterpreter::new(1),limits(1000),HtmlSemanticCancellation::new()),Err(e) if *e.kind()==HtmlSemanticFailureKind::ContextCaptureMismatch)
    );
    let mut s = session(&c, &cx, limits(1000), 1, HtmlSemanticCancellation::new());
    let f = s.query(&[request(&foreign)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(known(&r).source(), Some(&id));
}
