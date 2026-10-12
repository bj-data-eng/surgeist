use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 100000, 1000, 1000000, 1000, 10, 10),
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
fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 1000000, 1000000, 100000)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    raw: usize,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(raw), l, cancel).unwrap()
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

const GETTERS: [(HtmlSemanticQuery, &str); 2] = [
    (HtmlSemanticQuery::TableHead, "thead"),
    (HtmlSemanticQuery::TableFoot, "tfoot"),
];

fn value(r: &HtmlSemanticReport, index: usize) -> &HtmlSemanticValue<Option<HtmlCaptureSubjectId>> {
    let result = &r.results()[index];
    match (result.request().query(), result.fact()) {
        (HtmlSemanticQuery::TableHead, HtmlSemanticFact::TableHead(v))
        | (HtmlSemanticQuery::TableFoot, HtmlSemanticFact::TableFoot(v)) => v,
        other => panic!("wrong getter fact: {other:?}"),
    }
}
fn simple(local: &str) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let section = html(&mut b, &d, local, &[]);
    children(&mut b, &table, std::slice::from_ref(&section));
    let c = b.finish().unwrap();
    let id = c.subject_id(&table).unwrap();
    let expected = c.subject_id(&section).unwrap();
    (c, id, expected)
}
fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            20000000
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            100
        },
        0,
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            1000000
        },
        0,
    )
}

#[test]
fn interleaved_multiple_sections_select_first_identity_in_ordinary_order() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let foot = html(&mut b, &d, "tfoot", &[]);
    let body = html(&mut b, &d, "tbody", &[]);
    let head = html(&mut b, &d, "thead", &[]);
    let foot2 = html(&mut b, &d, "tfoot", &[]);
    let head2 = html(&mut b, &d, "thead", &[]);
    children(
        &mut b,
        &table,
        &[foot.clone(), body, head.clone(), foot2, head2],
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&table).unwrap();
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[
            request(&id, HtmlSemanticQuery::TableHead),
            request(&id, HtmlSemanticQuery::TableFoot),
        ])
        .unwrap();
    assert_eq!(
        value(&r, 0),
        &HtmlSemanticValue::Known(Some(c.subject_id(&head).unwrap()))
    );
    assert_eq!(
        value(&r, 1),
        &HtmlSemanticValue::Known(Some(c.subject_id(&foot).unwrap()))
    );
    assert!(matches!(
        query(&c, &id, HtmlSemanticQuery::TableRows, 0).results()[0].fact(),
        HtmlSemanticFact::TableRows(HtmlSemanticValue::Missing(_))
    ));
}

#[test]
fn first_empty_section_wins_over_later_nonempty_section_without_model_projection() {
    for (q, local) in GETTERS {
        let (mut b, d) = builder();
        let table = html(&mut b, &d, "table", &[]);
        let empty = html(&mut b, &d, local, &[]);
        let later = html(&mut b, &d, local, &[]);
        let row = html(&mut b, &d, "tr", &[]);
        children(&mut b, &empty, &[]);
        children(&mut b, &later, &[row]);
        children(&mut b, &table, &[empty.clone(), later]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&query(&c, &c.subject_id(&table).unwrap(), q, 0), 0),
            &HtmlSemanticValue::Known(Some(c.subject_id(&empty).unwrap()))
        );
    }
}

#[test]
fn known_section_and_later_unknown_rows_and_huge_names_need_no_extra_work() {
    for (q, local) in GETTERS {
        let (base, bid, _) = simple(local);
        let exact = query(&base, &bid, q, 0).usage().work();
        let (mut b, d) = builder();
        let table = html(&mut b, &d, "table", &[]);
        let section = html(&mut b, &d, local, &[]);
        let huge = html(&mut b, &d, &"x".repeat(10000), &[]);
        let row = html(&mut b, &d, "tr", &[]);
        children(&mut b, &table, &[section.clone(), huge, row]);
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, exact),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(&[request(&c.subject_id(&table).unwrap(), q)])
        .unwrap();
        assert_eq!(
            value(&r, 0),
            &HtmlSemanticValue::Known(Some(c.subject_id(&section).unwrap()))
        );
        assert_eq!(r.usage().work(), exact);
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn detached_empty_and_nonmatching_complete_children_are_null_unknown_order_missing() {
    let (mut b, d) = builder();
    let empty = html(&mut b, &d, "table", &[]);
    let absent = html(&mut b, &d, "table", &[]);
    let missing = html(&mut b, &d, "table", &[]);
    let body = html(&mut b, &d, "tbody", &[]);
    children(&mut b, &empty, &[]);
    b.establish_root(&empty).unwrap();
    children(&mut b, &absent, &[body]);
    let c = b.finish().unwrap();
    for (q, _) in GETTERS {
        for table in [&empty, &absent] {
            assert_eq!(
                value(&query(&c, &c.subject_id(table).unwrap(), q, 0), 0),
                &HtmlSemanticValue::Known(None)
            );
        }
        let id = c.subject_id(&missing).unwrap();
        assert_eq!(
            value(&query(&c, &id, q, 0), 0),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id))
        );
    }
}

#[test]
fn foreign_uppercase_and_isolated_utf16_names_skip_before_prefixed_real_sections() {
    for (q, local) in GETTERS {
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
        let svg = element(
            &mut b,
            &d,
            Namespace::svg(),
            local,
            OrderedAttributes::new([]).unwrap(),
        );
        let none = element(
            &mut b,
            &d,
            Namespace::none(),
            local,
            OrderedAttributes::new([]).unwrap(),
        );
        let upper = html(&mut b, &d, &local.to_ascii_uppercase(), &[]);
        let bad = b
            .add_element(
                &d,
                QualifiedName::new(
                    Namespace::html(),
                    None,
                    DocumentString::from_utf16(vec![0x74, 0xd800]),
                ),
                OrderedAttributes::new([]).unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let real = b
            .add_element(
                &d,
                QualifiedName::new(Namespace::html(), Some("q".into()), local),
                OrderedAttributes::new([]).unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        children(&mut b, &table, &[svg, none, upper, bad, real.clone()]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&query(&c, &c.subject_id(&table).unwrap(), q, 0), 0),
            &HtmlSemanticValue::Known(Some(c.subject_id(&real).unwrap()))
        );
    }
}

#[test]
fn nested_sections_do_not_count_text_comment_pi_data_and_attributes_are_unread() {
    for (q, local) in GETTERS {
        let (mut b, d) = builder();
        let table = html(&mut b, &d, "table", &[("title", &"x".repeat(10000))]);
        let div = html(&mut b, &d, "div", &[]);
        let nested = html(&mut b, &d, local, &[]);
        children(&mut b, &div, &[nested]);
        let text = b
            .add_text(&d, DocumentString::from_utf16(vec![0xd800, 10]))
            .unwrap();
        let comment = b.add_comment(&d, local.into()).unwrap();
        let pi = b
            .add_processing_instruction(&d, local.into(), "unread".into())
            .unwrap();
        children(&mut b, &table, &[text, comment, pi, div]);
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, 20000000),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(&[request(&c.subject_id(&table).unwrap(), q)])
        .unwrap();
        assert_eq!(value(&r, 0), &HtmlSemanticValue::Known(None));
        assert_eq!(
            r.usage().allocated_bytes(),
            std::mem::size_of::<HtmlSemanticResult>()
        );
    }
}

#[test]
fn template_and_shadow_associations_do_not_supply_direct_sections() {
    for (q, local) in GETTERS {
        for shadow in [false, true] {
            let (mut b, d) = builder();
            let table = html(&mut b, &d, "table", &[]);
            let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
            let f = if shadow {
                b.add_shadow_root(
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
            let section = html(&mut b, &d, local, &[]);
            children(&mut b, &f, &[section]);
            children(&mut b, &table, &[host]);
            let c = b.finish().unwrap();
            assert_eq!(
                value(&query(&c, &c.subject_id(&table).unwrap(), q, 0), 0),
                &HtmlSemanticValue::Known(None)
            );
        }
    }
}

#[test]
fn wrong_table_role_non_element_and_foreign_namespace_preserve_applicability() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let upper = html(&mut b, &d, "TABLE", &[]);
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
    for (q, _) in GETTERS {
        for subject in [&div, &upper, &d] {
            assert_eq!(
                value(&query(&c, &c.subject_id(subject).unwrap(), q, 0), 0),
                &HtmlSemanticValue::Inapplicable
            );
        }
        for subject in &foreign {
            assert_eq!(
                value(&query(&c, &c.subject_id(subject).unwrap(), q, 0), 0),
                &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
            );
        }
    }
}

#[test]
fn exact_and_one_short_work_output_bytes_stop_qualified_without_nested_allocations() {
    for (q, local) in GETTERS {
        let (c, id, expected) = simple(local);
        let cx = context(&c);
        let r = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, 20000000),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(&[request(&id, q)])
        .unwrap();
        let u = r.usage();
        assert_eq!(u.output_records(), 1);
        assert_eq!(
            u.allocated_bytes(),
            std::mem::size_of::<HtmlSemanticResult>()
        );
        assert_eq!(u.peak_scratch_records(), 0);
        assert_eq!(u.nested_output_records(), 0);
        for (resource, exact) in [
            (HtmlSemanticResource::Work, u.work()),
            (HtmlSemanticResource::OutputRecords, 1),
            (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
        ] {
            let req = request(&id, q);
            let r = session(
                &c,
                &cx,
                capped(resource, exact),
                0,
                HtmlSemanticCancellation::new(),
            )
            .query(std::slice::from_ref(&req))
            .unwrap();
            assert_eq!(
                value(&r, 0),
                &HtmlSemanticValue::Known(Some(expected.clone()))
            );
            let mut s = session(
                &c,
                &cx,
                capped(resource, exact - 1),
                0,
                HtmlSemanticCancellation::new(),
            );
            let f = s.query(std::slice::from_ref(&req)).unwrap_err();
            assert!(
                matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual == resource)
            );
            assert_eq!(s.query(&[]).unwrap_err(), f);
            let qual = f.qualification().unwrap();
            assert_eq!(qual.capture(), c.identity());
            assert_eq!(qual.context(), cx.identity());
            assert_eq!(qual.host_revision(), c.host_facts_revision());
            assert_eq!(qual.limits(), capped(resource, exact - 1));
            assert_eq!(s.usage().scratch_records(), 0);
            assert_eq!(
                value(&r, 0),
                &HtmlSemanticValue::Known(Some(expected.clone()))
            );
        }
    }
}

#[test]
fn duplicate_mixed_batch_and_repeated_queries_charge_again_and_late_failure_is_atomic() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let head = html(&mut b, &d, "thead", &[]);
    let foot = html(&mut b, &d, "tfoot", &[]);
    children(&mut b, &table, &[head, foot]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&table).unwrap();
    let reqs = [
        request(&id, HtmlSemanticQuery::TableHead),
        request(&id, HtmlSemanticQuery::TableFoot),
        request(&id, HtmlSemanticQuery::TableHead),
    ];
    let mut s = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, 20000000),
        0,
        HtmlSemanticCancellation::new(),
    );
    let old = s.query(&reqs).unwrap();
    let again = s.query(&reqs).unwrap();
    assert_eq!(again.usage().work(), 2 * old.usage().work());
    assert_eq!(
        again.usage().allocated_bytes(),
        2 * old.usage().allocated_bytes()
    );
    assert_eq!(old.results(), again.results());
    assert_eq!(
        old.usage().allocated_bytes(),
        3 * std::mem::size_of::<HtmlSemanticResult>()
    );
    for (resource, exact) in [
        (HtmlSemanticResource::Work, old.usage().work()),
        (HtmlSemanticResource::OutputRecords, 3),
        (
            HtmlSemanticResource::AllocatedBytes,
            old.usage().allocated_bytes(),
        ),
    ] {
        assert_eq!(
            session(
                &c,
                &cx,
                capped(resource, exact),
                0,
                HtmlSemanticCancellation::new()
            )
            .query(&reqs)
            .unwrap()
            .results(),
            old.results()
        );
        let mut bounded = session(
            &c,
            &cx,
            capped(resource, exact - 1),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = bounded.query(&reqs).unwrap_err();
        assert!(
            matches!(f.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual == resource)
        );
        assert_eq!(bounded.query(&[]).unwrap_err(), f);
        assert_eq!(bounded.usage().scratch_records(), 0);
        assert_eq!(old.results(), again.results());
    }
}

#[test]
fn cancellation_before_work_and_after_success_keeps_original_qualified_stop() {
    for (q, local) in GETTERS {
        let (c, id, _) = simple(local);
        let cx = context(&c);
        let req = request(&id, q);
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
        assert_eq!(s.usage().work(), old.usage().work());
        assert_eq!(s.query(&[]).unwrap_err(), f);
    }
}

#[test]
fn reordered_sections_recompute_and_old_reports_survive_changed_capture_host_context() {
    for (q, local) in GETTERS {
        let mut reports = vec![];
        for reverse in [false, true] {
            let (mut b, d) = builder();
            let table = html(&mut b, &d, "table", &[]);
            let a = html(&mut b, &d, local, &[]);
            let z = html(&mut b, &d, local, &[]);
            let cs = if reverse {
                [z.clone(), a.clone()]
            } else {
                [a.clone(), z.clone()]
            };
            children(&mut b, &table, &cs);
            let c = b.finish().unwrap();
            let id = c.subject_id(&table).unwrap();
            let expected = c.subject_id(if reverse { &z } else { &a }).unwrap();
            let r = query(&c, &id, q, 0);
            assert_eq!(
                value(&r, 0),
                &HtmlSemanticValue::Known(Some(expected.clone()))
            );
            let cx = context(&c);
            let alternate = session(
                &c,
                &cx,
                capped(HtmlSemanticResource::Work, 19999999),
                1,
                HtmlSemanticCancellation::new(),
            )
            .query(&[request(&id, q)])
            .unwrap();
            assert_ne!(r.context(), alternate.context());
            assert_eq!(alternate.qualification().attribute_input_limit(), 1);
            assert_eq!(alternate.qualification().limits().work(), 19999999);
            assert_eq!(
                r.dependencies(),
                HtmlSemanticDependencies::ConservativeRecompute
            );
            reports.push((r, c, id, expected));
        }
        assert_ne!(reports[0].0.capture(), reports[1].0.capture());
        assert_ne!(reports[0].0.host_revision(), reports[1].0.host_revision());
        let cx = context(&reports[0].1);
        assert!(
            HtmlSemanticSession::new(
                &reports[1].1,
                &cx,
                HtmlValueInterpreter::new(0),
                limits(),
                HtmlSemanticCancellation::new()
            )
            .is_err()
        );
        let mut s = session(
            &reports[0].1,
            &cx,
            limits(),
            0,
            HtmlSemanticCancellation::new(),
        );
        let f = s
            .query(&[request(&reports[0].2, q), request(&reports[1].2, q)])
            .unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
        assert_eq!(s.query(&[]).unwrap_err(), f);
        for (r, _, _, expected) in reports {
            assert_eq!(value(&r, 0), &HtmlSemanticValue::Known(Some(expected)));
        }
    }
}

#[test]
fn missing_head_does_not_hide_known_foot_in_a_later_request() {
    let (mut b, d) = builder();
    let missing = html(&mut b, &d, "table", &[]);
    let known = html(&mut b, &d, "table", &[]);
    let foot = html(&mut b, &d, "tfoot", &[]);
    children(&mut b, &known, std::slice::from_ref(&foot));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let missing_id = c.subject_id(&missing).unwrap();
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[
            request(&missing_id, HtmlSemanticQuery::TableHead),
            request(&c.subject_id(&known).unwrap(), HtmlSemanticQuery::TableFoot),
        ])
        .unwrap();
    assert_eq!(
        value(&r, 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(missing_id))
    );
    assert_eq!(
        value(&r, 1),
        &HtmlSemanticValue::Known(Some(c.subject_id(&foot).unwrap()))
    );
}
