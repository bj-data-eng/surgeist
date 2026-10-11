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
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
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
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::MapAreas)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    raw: usize,
) -> HtmlSemanticReport {
    session(c, cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}

fn areas(r: &HtmlSemanticReport) -> &HtmlAreaCollection {
    match r.results()[0].fact() {
        HtmlSemanticFact::MapAreas(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected complete areas, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::MapAreas(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing areas, got {other:?}"),
    }
}
fn direct(count: usize) -> (HtmlCapture, HtmlCaptureSubjectId, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let mut hs = Vec::new();
    for _ in 0..count {
        let a = html(&mut b, &d, "area", &[]);
        children(&mut b, &a, &[]);
        hs.push(a);
    }
    children(&mut b, &m, &hs);
    b.establish_root(&m).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&m).unwrap();
    let ids = hs.iter().map(|h| c.subject_id(h).unwrap()).collect();
    (c, id, ids)
}
#[test]
fn detached_complete_empty_map_returns_known_empty_without_parent_or_host_facts() {
    let (c, m, _) = direct(0);
    let r = report(&c, &context(&c), &m, 0);
    assert!(areas(&r).is_empty());
    assert_eq!(areas(&r).len(), 0);
    assert_eq!(areas(&r).areas(), &[]);
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 0);
}
#[test]
fn ordinary_preorder_includes_wrapped_areas_nested_maps_and_malformed_area_children() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let a = html(&mut b, &d, "area", &[]);
    let nested = html(&mut b, &d, "map", &[]);
    let barea = html(&mut b, &d, "area", &[]);
    let wrapper = html(&mut b, &d, "div", &[]);
    let inner = html(&mut b, &d, "area", &[]);
    let last = html(&mut b, &d, "area", &[]);
    children(&mut b, &a, &[]);
    children(&mut b, &barea, std::slice::from_ref(&inner));
    children(&mut b, &inner, &[]);
    children(&mut b, &nested, std::slice::from_ref(&barea));
    children(&mut b, &wrapper, &[nested]);
    children(&mut b, &last, &[]);
    children(&mut b, &m, &[a.clone(), wrapper, last.clone()]);
    let c = b.finish().unwrap();
    let expected = [
        c.subject_id(&a).unwrap(),
        c.subject_id(&barea).unwrap(),
        c.subject_id(&inner).unwrap(),
        c.subject_id(&last).unwrap(),
    ];
    let r = report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0);
    assert_eq!(areas(&r).areas(), &expected);
    assert_eq!(areas(&r).len(), 4);
    assert!(!areas(&r).is_empty());
}
#[test]
fn exact_html_area_names_are_filtered_but_foreign_wrappers_are_traversed() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let svg = element(
        &mut b,
        &d,
        Namespace::svg(),
        "area",
        OrderedAttributes::new([]).unwrap(),
    );
    let math = element(
        &mut b,
        &d,
        Namespace::mathml(),
        "map",
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "AREA", &[]);
    let actual = html(&mut b, &d, "area", &[]);
    children(&mut b, &actual, &[]);
    children(&mut b, &upper, std::slice::from_ref(&actual));
    children(&mut b, &math, &[upper]);
    children(&mut b, &svg, &[math]);
    children(&mut b, &m, &[svg]);
    let c = b.finish().unwrap();
    assert_eq!(
        areas(&report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0)).areas(),
        &[c.subject_id(&actual).unwrap()]
    );
}
#[test]
fn area_membership_ignores_all_attributes_visibility_and_uninspected_utf16_text() {
    let (mut b, d) = builder();
    let m = html(
        &mut b,
        &d,
        "map",
        &[("name", "bad whitespace enormous name"), ("hidden", "")],
    );
    let raw = DocumentString::from_utf16(vec![0xdfff, 0xd800, 49]);
    let a = element(
        &mut b,
        &d,
        Namespace::html(),
        "area",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "href"),
                raw.clone(),
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "coords"),
                raw.clone(),
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "alt"),
                "enormous unused",
            ),
            Attribute::new(QualifiedName::new(Namespace::none(), None, "hidden"), ""),
        ])
        .unwrap(),
    );
    let text = b.add_text(&d, raw).unwrap();
    children(&mut b, &a, &[text]);
    children(&mut b, &m, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    assert_eq!(
        areas(&report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0)).areas(),
        &[c.subject_id(&a).unwrap()]
    );
}
#[test]
fn actual_ids_are_preserved_without_raw_id_deduplication_or_scalar_conversion() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let mut hs = Vec::new();
    for _ in 0..2 {
        let a = element(
            &mut b,
            &d,
            Namespace::html(),
            "area",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "id"),
                DocumentString::from_utf16(vec![0xdfff]),
            )])
            .unwrap(),
        );
        children(&mut b, &a, &[]);
        hs.push(a);
    }
    children(&mut b, &m, &hs);
    let c = b.finish().unwrap();
    let expected = [c.subject_id(&hs[0]).unwrap(), c.subject_id(&hs[1]).unwrap()];
    assert_ne!(expected[0], expected[1]);
    assert_eq!(
        areas(&report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0)).areas(),
        &expected
    );
}
#[test]
fn shadow_and_template_contents_are_excluded_but_ordinary_host_children_participate() {
    for shadow in [true, false] {
        let (mut b, d) = builder();
        let m = html(&mut b, &d, "map", &[]);
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let f = if shadow {
            b.add_shadow_root(
                &host,
                HtmlCapturedShadowProperties::new(
                    HtmlShadowRootProperties::new(
                        HtmlShadowRootMode::Open,
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
        let hidden = html(&mut b, &d, "area", &[]);
        let ordinary = html(&mut b, &d, "area", &[]);
        children(&mut b, &hidden, &[]);
        children(&mut b, &ordinary, &[]);
        children(&mut b, &f, &[hidden]);
        children(&mut b, &host, std::slice::from_ref(&ordinary));
        children(&mut b, &m, &[host]);
        let c = b.finish().unwrap();
        assert_eq!(
            areas(&report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0)).areas(),
            &[c.subject_id(&ordinary).unwrap()]
        );
    }
}
#[test]
fn unavailable_root_children_is_missing_not_known_empty() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&m).unwrap();
    assert_eq!(
        missing(&report(&c, &context(&c), &id, 0)),
        &HtmlSemanticMissing::OrderedChildren(id)
    );
}
#[test]
fn unknown_descendant_children_discards_a_collected_prefix_and_releases_scratch() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let a = html(&mut b, &d, "area", &[]);
    let later = html(&mut b, &d, "div", &[]);
    children(&mut b, &a, &[]);
    children(&mut b, &m, &[a, later.clone()]);
    let c = b.finish().unwrap();
    let r = report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0);
    assert_eq!(
        missing(&r),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&later).unwrap())
    );
    assert_eq!(r.usage().scratch_records(), 0);
    assert!(r.usage().nested_output_records() > 0);
}
#[test]
fn even_a_matched_area_requires_complete_ordinary_descendants_for_the_full_collection() {
    let (mut b, d) = builder();
    let m = html(&mut b, &d, "map", &[]);
    let a = html(&mut b, &d, "area", &[]);
    children(&mut b, &m, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &context(&c), &c.subject_id(&m).unwrap(), 0);
    assert_eq!(
        missing(&r),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&a).unwrap())
    );
    assert_eq!(r.usage().scratch_records(), 0);
}
#[test]
fn map_role_case_namespace_and_non_element_subjects_have_typed_outcomes() {
    let (mut b, d) = builder();
    let upper = html(&mut b, &d, "MAP", &[]);
    let area = html(&mut b, &d, "area", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "map",
        OrderedAttributes::new([]).unwrap(),
    );
    let text = b.add_text(&d, "unread".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    for h in [upper, area, text, d] {
        assert_eq!(
            report(&c, &cx, &c.subject_id(&h).unwrap(), 0).results()[0].fact(),
            &HtmlSemanticFact::MapAreas(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &cx, &c.subject_id(&foreign).unwrap(), 0).results()[0].fact(),
        &HtmlSemanticFact::MapAreas(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}
fn resource_limits(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            limits().work()
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            100
        },
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            1000
        },
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            1000000
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            100000
        },
    )
}
#[test]
fn full_collection_has_exact_one_short_work_output_scratch_nested_capacity_and_bytes() {
    let (c, m, expected) = direct(3);
    let cx = context(&c);
    let u = report(&c, &cx, &m, 0).usage();
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            u.nested_output_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                0,
                HtmlSemanticCancellation::new(),
            );
            let result = ss.query(&[request(&m)]);
            if cap == exact {
                assert_eq!(areas(&result.unwrap()).areas(), expected);
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{failure:?}"
                );
                assert_eq!(ss.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn duplicate_queries_charge_cumulative_member_capacity_and_cannot_reuse_a_discarded_prefix() {
    let (c, m, _) = direct(2);
    let cx = context(&c);
    let one = report(&c, &cx, &m, 0).usage();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let r = ss.query(&[request(&m), request(&m)]).unwrap();
    assert_eq!(
        r.usage().nested_output_records(),
        2 * one.nested_output_records()
    );
    assert!(r.usage().allocated_bytes() > one.allocated_bytes());
    let mut ss = session(
        &c,
        &cx,
        resource_limits(
            HtmlSemanticResource::NestedOutputRecords,
            one.nested_output_records(),
        ),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[request(&m), request(&m)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(ss.usage().scratch_records(), 0);
}
#[test]
fn late_batch_output_failure_and_cancellation_are_qualified_atomic_and_sticky() {
    let (c, m, _) = direct(2);
    let cx = context(&c);
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::OutputRecords, 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[request(&m), request(&m)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(ss.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 0, cancel);
    let failure = ss.query(&[request(&m)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}
#[test]
fn changed_topology_and_context_configuration_qualify_conservative_recomputation() {
    let (a, m, _) = direct(1);
    let (b, n, _) = direct(2);
    let ax = context(&a);
    let bx = context(&b);
    let ar = report(&a, &ax, &m, 0);
    let br = report(&b, &bx, &n, 0);
    assert_eq!(areas(&ar).len(), 1);
    assert_eq!(areas(&br).len(), 2);
    assert_ne!(ar.capture(), br.capture());
    assert_ne!(ar.host_revision(), br.host_revision());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let another = context(&a);
    let cr = report(&a, &another, &m, 0);
    assert_eq!(areas(&cr).areas(), areas(&ar).areas());
    assert_ne!(cr.context(), ar.context());
    let limited = session(
        &a,
        &ax,
        resource_limits(HtmlSemanticResource::Work, ar.usage().work()),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&m)])
    .unwrap();
    assert_ne!(limited.qualification(), ar.qualification());
}
#[test]
fn foreign_subject_and_context_are_rejected_without_guessing_area_membership() {
    let (a, m, _) = direct(1);
    let (b, n, _) = direct(1);
    let ax = context(&a);
    let bx = context(&b);
    assert!(
        HtmlSemanticSession::new(
            &a,
            &bx,
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    let failure = session(&a, &ax, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&n)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert_eq!(failure.qualification().unwrap().capture(), a.identity());
    let _ = m;
}
