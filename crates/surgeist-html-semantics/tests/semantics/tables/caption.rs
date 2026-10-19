use crate::semantics::support::{
    builder, context, scalar_limits as resource_limits,
    semantic_limits_with_integer_digits as limits, session,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        attrs,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(
        b,
        d,
        Namespace::html(),
        local,
        OrderedAttributes::new(
            attrs
                .iter()
                .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
        )
        .unwrap(),
    )
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
    raw: usize,
) -> HtmlSemanticReport {
    let cx = context(c);
    session(c, &cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}

fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<Option<HtmlCaptureSubjectId>> {
    match r.results()[0].fact() {
        HtmlSemanticFact::TableCaption(value) => value,
        other => panic!("expected table caption, got {other:?}"),
    }
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    query(c, id, HtmlSemanticQuery::TableCaption, 0)
}
fn simple() -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let caption = html(&mut b, &d, "caption", &[]);
    children(&mut b, &table, std::slice::from_ref(&caption));
    let c = b.finish().unwrap();
    let id = c.subject_id(&table).unwrap();
    let expected = c.subject_id(&caption).unwrap();
    (c, id, expected)
}

#[test]
fn first_direct_caption_wins_among_multiple_and_rows_need_no_contents() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let row = html(&mut b, &d, "tr", &[]);
    let first = html(&mut b, &d, "caption", &[]);
    let second = html(&mut b, &d, "caption", &[]);
    children(&mut b, &table, &[row, first.clone(), second]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&table).unwrap();
    assert_eq!(
        value(&report(&c, &id)),
        &HtmlSemanticValue::Known(Some(c.subject_id(&first).unwrap()))
    );
    assert!(matches!(
        query(&c, &id, HtmlSemanticQuery::TableModel, 0).results()[0].fact(),
        HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(_))
    ));
}

#[test]
fn first_witness_skips_later_name_work_and_unavailable_row_and_caption_children() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let caption = html(&mut b, &d, "caption", &[]);
    let huge = html(&mut b, &d, &"x".repeat(10000), &[]);
    let row = html(&mut b, &d, "tr", &[]);
    children(&mut b, &table, &[caption.clone(), huge, row]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let (baseline, bid, _) = simple();
    let exact = report(&baseline, &bid).usage().work();
    let r = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, exact),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(
        &c.subject_id(&table).unwrap(),
        HtmlSemanticQuery::TableCaption,
    )])
    .unwrap();
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
    assert_eq!(r.usage().work(), exact);
}

#[test]
fn complete_absence_and_empty_are_null_but_unavailable_direct_order_is_missing() {
    let (mut b, d) = builder();
    let empty = html(&mut b, &d, "table", &[]);
    let absent = html(&mut b, &d, "table", &[]);
    let missing = html(&mut b, &d, "table", &[]);
    let div = html(&mut b, &d, "div", &[]);
    children(&mut b, &empty, &[]);
    b.establish_root(&empty).unwrap();
    children(&mut b, &absent, &[div]);
    let c = b.finish().unwrap();
    for table in [&empty, &absent] {
        assert_eq!(
            value(&report(&c, &c.subject_id(table).unwrap())),
            &HtmlSemanticValue::Known(None)
        );
    }
    let id = c.subject_id(&missing).unwrap();
    assert_eq!(
        value(&report(&c, &id)),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id))
    );
}

#[test]
fn huge_attributes_and_text_have_no_raw_or_nested_output_requirement() {
    let (mut b, d) = builder();
    let huge = "x".repeat(10000);
    let table = html(&mut b, &d, "table", &[("title", &huge), ("id", &huge)]);
    let text = b.add_text(&d, huge.clone().into()).unwrap();
    let caption = html(&mut b, &d, "caption", &[("title", &huge)]);
    children(&mut b, &table, &[text, caption.clone()]);
    b.establish_root(&table).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, 20000000),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(
        &c.subject_id(&table).unwrap(),
        HtmlSemanticQuery::TableCaption,
    )])
    .unwrap();
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(
        r.usage().allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>()
    );
}

#[test]
fn exact_namespace_case_and_utf16_filter_only_actual_direct_html_caption() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let upper = html(&mut b, &d, "CAPTION", &[]);
    let svg = element(
        &mut b,
        &d,
        Namespace::svg(),
        "caption",
        OrderedAttributes::new([]).unwrap(),
    );
    let none = element(
        &mut b,
        &d,
        Namespace::none(),
        "caption",
        OrderedAttributes::new([]).unwrap(),
    );
    let lookalike = html(&mut b, &d, "captıon", &[]);
    let malformed = b
        .add_element(
            &d,
            QualifiedName::new(
                Namespace::html(),
                None,
                DocumentString::from_utf16(vec![0x63, 0xd800]),
            ),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let real = html(&mut b, &d, "caption", &[]);
    children(
        &mut b,
        &table,
        &[upper, svg, none, lookalike, malformed, real.clone()],
    );
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&table).unwrap())),
        &HtmlSemanticValue::Known(Some(c.subject_id(&real).unwrap()))
    );
}

#[test]
fn prefixes_do_not_change_expanded_names_and_ancestry_is_not_required() {
    let (mut b, d) = builder();
    let table = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), Some("p".into()), "table"),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let caption = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), Some("q".into()), "caption"),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    children(&mut b, &table, std::slice::from_ref(&caption));
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&table).unwrap())),
        &HtmlSemanticValue::Known(Some(c.subject_id(&caption).unwrap()))
    );
}

#[test]
fn nested_caption_does_not_count_and_non_element_data_is_not_inspected() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[("title", &"x".repeat(10000))]);
    let div = html(&mut b, &d, "div", &[]);
    let caption = html(&mut b, &d, "caption", &[]);
    children(&mut b, &div, &[caption]);
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0xd800, 10]))
        .unwrap();
    let comment = b.add_comment(&d, "caption".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "caption".into(), "unread".into())
        .unwrap();
    children(&mut b, &table, &[text, comment, pi, div]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&table).unwrap());
    assert_eq!(value(&r), &HtmlSemanticValue::Known(None));
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}

#[test]
fn template_and_shadow_associations_are_not_ordinary_caption_children() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let table = html(&mut b, &d, "table", &[]);
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let f = if shadow {
            b.add_shadow_root(
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
            .unwrap()
        } else {
            let f = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &f).unwrap();
            f
        };
        let caption = html(&mut b, &d, "caption", &[]);
        children(&mut b, &f, &[caption]);
        children(&mut b, &table, &[host]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&table).unwrap())),
            &HtmlSemanticValue::Known(None)
        );
    }
}

#[test]
fn wrong_roles_and_non_elements_are_inapplicable_foreign_tables_unsupported() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let upper = html(&mut b, &d, "TABLE", &[]);
    let text = b.add_text(&d, "table".into()).unwrap();
    let mut foreign = vec![];
    for ns in [Namespace::svg(), Namespace::mathml(), Namespace::none()] {
        foreign.push(element(
            &mut b,
            &d,
            ns,
            "table",
            OrderedAttributes::new([]).unwrap(),
        ));
    }
    let c = b.finish().unwrap();
    for subject in [&div, &upper, &text, &d] {
        assert_eq!(
            value(&report(&c, &c.subject_id(subject).unwrap())),
            &HtmlSemanticValue::Inapplicable
        );
    }
    for subject in foreign {
        assert_eq!(
            value(&report(&c, &c.subject_id(&subject).unwrap())),
            &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
        );
    }
}

#[test]
fn physical_outer_result_and_work_exact_one_short_stops_are_qualified_and_sticky() {
    let (c, id, expected) = simple();
    let cx = context(&c);
    let baseline = report(&c, &id);
    let u = baseline.usage();
    assert_eq!(u.output_records(), 1);
    assert_eq!(
        u.allocated_bytes(),
        std::mem::size_of::<HtmlSemanticResult>()
    );
    assert_eq!(u.scratch_records(), 0);
    assert_eq!(u.peak_scratch_records(), 0);
    assert_eq!(u.nested_output_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, 1),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        let req = request(&id, HtmlSemanticQuery::TableCaption);
        let r = session(
            &c,
            &cx,
            resource_limits(resource, exact),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(std::slice::from_ref(&req))
        .unwrap();
        assert_eq!(value(&r), &HtmlSemanticValue::Known(Some(expected.clone())));
        let mut s = session(
            &c,
            &cx,
            resource_limits(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual==resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), f);
        assert_eq!(s.usage().scratch_records(), 0);
        let q = f.qualification().unwrap();
        assert_eq!(q.capture(), baseline.qualification().capture());
        assert_eq!(q.context(), cx.identity());
        assert_eq!(q.host_revision(), baseline.qualification().host_revision());
        assert_eq!(q.limits(), resource_limits(resource, exact - 1));
        assert_eq!(q.attribute_input_limit(), 0);
    }
}

#[test]
fn repeats_and_duplicate_batch_charge_cumulatively_and_late_failure_is_atomic() {
    let (c, id, expected) = simple();
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::TableCaption);
    let mut s = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, 20000000),
        0,
        HtmlSemanticCancellation::new(),
    );
    let old = s.query(std::slice::from_ref(&req)).unwrap();
    let again = s.query(std::slice::from_ref(&req)).unwrap();
    assert_eq!(again.usage().work(), 2 * old.usage().work());
    assert_eq!(
        again.usage().allocated_bytes(),
        2 * old.usage().allocated_bytes()
    );
    let duplicate = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, 20000000),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[req.clone(), req.clone()])
    .unwrap();
    assert_eq!(duplicate.results().len(), 2);
    assert_eq!(duplicate.usage().work(), 2 * old.usage().work() - 1);
    let mut bounded = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, duplicate.usage().work() - 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = bounded.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(bounded.query(&[]).unwrap_err(), f);
    assert_eq!(bounded.usage().scratch_records(), 0);
    assert_eq!(value(&old), &HtmlSemanticValue::Known(Some(expected)));
}

#[test]
fn cancellation_before_query_and_after_success_preserves_reports_and_first_stop() {
    let (c, id, expected) = simple();
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::TableCaption);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 0, cancel);
    let f = s.query(std::slice::from_ref(&req)).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.usage().work(), 0);
    assert_eq!(s.query(&[]).unwrap_err(), f);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 0, cancel.clone());
    let old = s.query(std::slice::from_ref(&req)).unwrap();
    cancel.cancel();
    let f = s.query(std::slice::from_ref(&req)).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap(), old.qualification());
    assert_eq!(s.query(&[]).unwrap_err(), f);
    assert_eq!(value(&old), &HtmlSemanticValue::Known(Some(expected)));
}

#[test]
fn order_recomputation_and_capture_context_host_config_qualification_are_exact() {
    let mut reports = vec![];
    for reverse in [false, true] {
        let (mut b, d) = builder();
        let table = html(&mut b, &d, "table", &[]);
        let a = html(&mut b, &d, "caption", &[]);
        let z = html(&mut b, &d, "caption", &[]);
        let expected = if reverse { &z } else { &a };
        let cs = if reverse {
            [z.clone(), a.clone()]
        } else {
            [a.clone(), z.clone()]
        };
        children(&mut b, &table, &cs);
        let c = b.finish().unwrap();
        let id = c.subject_id(&table).unwrap();
        let r = report(&c, &id);
        assert_eq!(
            value(&r),
            &HtmlSemanticValue::Known(Some(c.subject_id(expected).unwrap()))
        );
        let cx = context(&c);
        let alternate = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
            .query(&[request(&id, HtmlSemanticQuery::TableCaption)])
            .unwrap();
        assert_ne!(r.context(), alternate.context());
        assert_eq!(alternate.qualification().attribute_input_limit(), 1);
        assert_eq!(
            r.dependencies(),
            HtmlSemanticDependencies::ConservativeRecompute
        );
        reports.push((r, c, id));
    }
    assert_ne!(reports[0].0.capture(), reports[1].0.capture());
    assert_ne!(reports[0].0.host_revision(), reports[1].0.host_revision());
    let cx = context(&reports[0].1);
    let mut s = session(
        &reports[0].1,
        &cx,
        limits(),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = s
        .query(&[
            request(&reports[0].2, HtmlSemanticQuery::TableCaption),
            request(&reports[1].2, HtmlSemanticQuery::TableCaption),
        ])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.query(&[]).unwrap_err(), f);
}

#[test]
fn missing_is_query_local_and_does_not_prevent_a_later_known_caption() {
    let (mut b, d) = builder();
    let missing = html(&mut b, &d, "table", &[]);
    let known = html(&mut b, &d, "table", &[]);
    let caption = html(&mut b, &d, "caption", &[]);
    children(&mut b, &known, std::slice::from_ref(&caption));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mid = c.subject_id(&missing).unwrap();
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[
            request(&mid, HtmlSemanticQuery::TableCaption),
            request(
                &c.subject_id(&known).unwrap(),
                HtmlSemanticQuery::TableCaption,
            ),
        ])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::TableCaption(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(mid)
        ))
    );
    assert_eq!(
        r.results()[1].fact(),
        &HtmlSemanticFact::TableCaption(HtmlSemanticValue::Known(Some(
            c.subject_id(&caption).unwrap()
        )))
    );
}

#[test]
fn physical_batch_bytes_and_output_admission_are_exact_and_one_short_atomic() {
    let (c, id, _) = simple();
    let cx = context(&c);
    let reqs = [
        request(&id, HtmlSemanticQuery::TableCaption),
        request(&id, HtmlSemanticQuery::TableCaption),
    ];
    let r = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, 20000000),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&reqs)
    .unwrap();
    assert_eq!(r.usage().output_records(), 2);
    assert_eq!(
        r.usage().allocated_bytes(),
        2 * std::mem::size_of::<HtmlSemanticResult>()
    );
    for (resource, exact) in [
        (HtmlSemanticResource::OutputRecords, 2),
        (
            HtmlSemanticResource::AllocatedBytes,
            r.usage().allocated_bytes(),
        ),
    ] {
        assert_eq!(
            session(
                &c,
                &cx,
                resource_limits(resource, exact),
                0,
                HtmlSemanticCancellation::new()
            )
            .query(&reqs)
            .unwrap()
            .results()
            .len(),
            2
        );
        let mut s = session(
            &c,
            &cx,
            resource_limits(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = s.query(&reqs).unwrap_err();
        assert!(
            matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual == resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), f);
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.usage().nested_output_records(), 0);
    }
}
