#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

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

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    session(c, cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}
fn ids(r: &HtmlSemanticReport) -> &[HtmlCaptureSubjectId] {
    match r.results()[0].fact() {
        HtmlSemanticFact::TableBodies(HtmlSemanticValue::Known(v)) => v.bodies(),
        HtmlSemanticFact::TableRows(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::TableSectionRows(HtmlSemanticValue::Known(v)) => v.rows(),
        HtmlSemanticFact::TableRowCells(HtmlSemanticValue::Known(v)) => v.cells(),
        other => panic!("expected complete collection, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::TableBodies(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::TableRows(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::TableSectionRows(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::TableRowCells(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing, got {other:?}"),
    }
}
fn ordered() -> (
    HtmlCapture,
    HtmlCaptureSubjectId,
    Vec<HtmlCaptureSubjectId>,
    Vec<HtmlCaptureSubjectId>,
) {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table", &[]);
    let mut parent = Vec::new();
    let mut rs = Vec::new();
    let mut bodies = Vec::new();
    for kind in [
        "tfoot", "tr", "tbody", "thead", "tr", "thead", "tbody", "tfoot",
    ] {
        let p = html(&mut b, &d, kind, &[]);
        if kind == "tr" {
            rs.push(p.clone());
        } else {
            let r = html(&mut b, &d, "tr", &[]);
            children(&mut b, &p, std::slice::from_ref(&r));
            rs.push(r);
        }
        if kind == "tbody" {
            bodies.push(p.clone());
        }
        parent.push(p);
    }
    children(&mut b, &t, &parent);
    b.establish_root(&t).unwrap();
    let c = b.finish().unwrap();
    let expected = [3, 5, 1, 2, 4, 6, 0, 7]
        .iter()
        .map(|i| c.subject_id(&rs[*i]).unwrap())
        .collect();
    let bodyids = bodies.iter().map(|h| c.subject_id(h).unwrap()).collect();
    let id = c.subject_id(&t).unwrap();
    (c, id, expected, bodyids)
}
#[test]
fn interleaved_multiple_headers_bodies_direct_rows_and_footers_use_source_collection_order() {
    let (c, t, expected, bodies) = ordered();
    let cx = context(&c);
    let r = query(&c, &cx, &t, HtmlSemanticQuery::TableRows);
    assert_eq!(ids(&r), expected);
    assert_eq!(r.usage().scratch_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
    let r = query(&c, &cx, &t, HtmlSemanticQuery::TableBodies);
    assert_eq!(ids(&r), bodies);
    if let HtmlSemanticFact::TableBodies(HtmlSemanticValue::Known(v)) = r.results()[0].fact() {
        assert_eq!(v.len(), 2);
        assert!(!v.is_empty());
    } else {
        panic!("missing bodies");
    }
}
#[test]
fn all_four_detached_complete_empty_roles_return_empty_without_parent_or_raw_facts() {
    for (kind, q) in [
        ("table", HtmlSemanticQuery::TableBodies),
        ("table", HtmlSemanticQuery::TableRows),
        ("thead", HtmlSemanticQuery::TableSectionRows),
        ("tbody", HtmlSemanticQuery::TableSectionRows),
        ("tfoot", HtmlSemanticQuery::TableSectionRows),
        ("tr", HtmlSemanticQuery::TableRowCells),
    ] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, kind, &[]);
        children(&mut b, &p, &[]);
        b.establish_root(&p).unwrap();
        let c = b.finish().unwrap();
        let r = query(&c, &context(&c), &c.subject_id(&p).unwrap(), q);
        assert!(ids(&r).is_empty());
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(r.usage().nested_output_records(), 0);
    }
}
#[test]
fn body_membership_is_direct_only_and_does_not_require_body_children() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table", &[]);
    let direct = html(&mut b, &d, "tbody", &[]);
    let w = html(&mut b, &d, "div", &[]);
    let nested = html(&mut b, &d, "tbody", &[]);
    children(&mut b, &w, &[nested]);
    children(&mut b, &t, &[direct.clone(), w]);
    let c = b.finish().unwrap();
    assert_eq!(
        ids(&query(
            &c,
            &context(&c),
            &c.subject_id(&t).unwrap(),
            HtmlSemanticQuery::TableBodies
        )),
        &[c.subject_id(&direct).unwrap()]
    );
}
#[test]
fn table_rows_exclude_wrapped_sections_rows_and_nested_tables_without_reading_their_children() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table", &[]);
    let w = html(&mut b, &d, "div", &[]);
    let nested = html(&mut b, &d, "table", &[]);
    let r = html(&mut b, &d, "tr", &[]);
    let wrapped = html(&mut b, &d, "tbody", &[]);
    children(&mut b, &w, &[wrapped]);
    children(&mut b, &t, &[w, nested, r.clone()]);
    let c = b.finish().unwrap();
    assert_eq!(
        ids(&query(
            &c,
            &context(&c),
            &c.subject_id(&t).unwrap(),
            HtmlSemanticQuery::TableRows
        )),
        &[c.subject_id(&r).unwrap()]
    );
}
#[test]
fn each_section_includes_only_direct_actual_rows_and_never_reads_row_descendants() {
    for kind in ["thead", "tbody", "tfoot"] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, kind, &[]);
        let direct = html(&mut b, &d, "tr", &[]);
        let w = html(&mut b, &d, "div", &[]);
        let nested = html(&mut b, &d, "tr", &[]);
        let foreign = element(
            &mut b,
            &d,
            Namespace::svg(),
            "tr",
            OrderedAttributes::new([]).unwrap(),
        );
        let upper = html(&mut b, &d, "TR", &[]);
        children(&mut b, &w, &[nested]);
        children(&mut b, &p, &[w, direct.clone(), foreign, upper]);
        let c = b.finish().unwrap();
        let r = query(
            &c,
            &context(&c),
            &c.subject_id(&p).unwrap(),
            HtmlSemanticQuery::TableSectionRows,
        );
        assert_eq!(ids(&r), &[c.subject_id(&direct).unwrap()]);
        if let HtmlSemanticFact::TableSectionRows(HtmlSemanticValue::Known(v)) =
            r.results()[0].fact()
        {
            assert_eq!(v.len(), 1);
            assert!(!v.is_empty());
        }
    }
}
#[test]
fn row_cells_include_direct_td_and_th_in_tree_order_but_not_wrapped_or_foreign_cells() {
    let (mut b, d) = builder();
    let r = html(&mut b, &d, "tr", &[]);
    let th = html(&mut b, &d, "th", &[]);
    let td = html(&mut b, &d, "td", &[]);
    let w = html(&mut b, &d, "div", &[]);
    let nested = html(&mut b, &d, "td", &[]);
    let svg = element(
        &mut b,
        &d,
        Namespace::svg(),
        "td",
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "TH", &[]);
    children(&mut b, &w, &[nested]);
    children(&mut b, &r, &[th.clone(), w, svg, td.clone(), upper]);
    let c = b.finish().unwrap();
    let result = query(
        &c,
        &context(&c),
        &c.subject_id(&r).unwrap(),
        HtmlSemanticQuery::TableRowCells,
    );
    assert_eq!(
        ids(&result),
        &[c.subject_id(&th).unwrap(), c.subject_id(&td).unwrap()]
    );
    if let HtmlSemanticFact::TableRowCells(HtmlSemanticValue::Known(v)) = result.results()[0].fact()
    {
        assert_eq!(v.len(), 2);
        assert!(!v.is_empty());
    }
}
#[test]
fn foreign_and_uppercase_section_names_do_not_supply_table_rows_or_bodies() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table", &[]);
    let svg = element(
        &mut b,
        &d,
        Namespace::svg(),
        "tbody",
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "THEAD", &[]);
    let actual = html(&mut b, &d, "tr", &[]);
    children(&mut b, &t, &[svg, upper, actual.clone()]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&t).unwrap();
    let cx = context(&c);
    assert!(ids(&query(&c, &cx, &id, HtmlSemanticQuery::TableBodies)).is_empty());
    assert_eq!(
        ids(&query(&c, &cx, &id, HtmlSemanticQuery::TableRows)),
        &[c.subject_id(&actual).unwrap()]
    );
}
#[test]
fn unknown_root_child_order_is_missing_for_each_applicable_collection() {
    for (kind, q) in [
        ("table", HtmlSemanticQuery::TableBodies),
        ("table", HtmlSemanticQuery::TableRows),
        ("thead", HtmlSemanticQuery::TableSectionRows),
        ("tr", HtmlSemanticQuery::TableRowCells),
    ] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, kind, &[]);
        let c = b.finish().unwrap();
        let id = c.subject_id(&p).unwrap();
        assert_eq!(
            missing(&query(&c, &context(&c), &id, q)),
            &HtmlSemanticMissing::OrderedChildren(id)
        );
    }
}
#[test]
fn unknown_applicable_section_children_returns_missing_without_successful_row_prefix() {
    for kind in ["thead", "tbody", "tfoot"] {
        let (mut b, d) = builder();
        let t = html(&mut b, &d, "table", &[]);
        let r = html(&mut b, &d, "tr", &[]);
        let group = html(&mut b, &d, kind, &[]);
        children(&mut b, &t, &[r, group.clone()]);
        let c = b.finish().unwrap();
        let result = query(
            &c,
            &context(&c),
            &c.subject_id(&t).unwrap(),
            HtmlSemanticQuery::TableRows,
        );
        assert_eq!(
            missing(&result),
            &HtmlSemanticMissing::OrderedChildren(c.subject_id(&group).unwrap())
        );
        assert_eq!(result.usage().scratch_records(), 0);
    }
}
#[test]
fn uninspected_surrogate_attributes_large_spans_and_text_do_not_change_membership() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table", &[("hidden", "")]);
    let r = html(&mut b, &d, "tr", &[]);
    let td = element(
        &mut b,
        &d,
        Namespace::html(),
        "td",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "rowspan"),
                "9999999999999999999999999999999",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "headers"),
                DocumentString::from_utf16(vec![0xdfff]),
            ),
        ])
        .unwrap(),
    );
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0xd800, 0xdfff]))
        .unwrap();
    children(&mut b, &r, &[text, td.clone()]);
    children(&mut b, &t, std::slice::from_ref(&r));
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        ids(&query(
            &c,
            &cx,
            &c.subject_id(&t).unwrap(),
            HtmlSemanticQuery::TableRows
        )),
        &[c.subject_id(&r).unwrap()]
    );
    assert_eq!(
        ids(&query(
            &c,
            &cx,
            &c.subject_id(&r).unwrap(),
            HtmlSemanticQuery::TableRowCells
        )),
        &[c.subject_id(&td).unwrap()]
    );
}
#[test]
fn namespace_case_and_non_element_subject_roles_preserve_typed_applicability() {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table", &[]);
    let body = html(&mut b, &d, "tbody", &[]);
    let row = html(&mut b, &d, "tr", &[]);
    let upper = html(&mut b, &d, "TABLE", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "table",
        OrderedAttributes::new([]).unwrap(),
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    for (h, q) in [
        (body.clone(), HtmlSemanticQuery::TableRows),
        (row, HtmlSemanticQuery::TableBodies),
        (table, HtmlSemanticQuery::TableSectionRows),
        (body, HtmlSemanticQuery::TableRowCells),
        (upper, HtmlSemanticQuery::TableRows),
        (d, HtmlSemanticQuery::TableBodies),
    ] {
        let r = query(&c, &cx, &c.subject_id(&h).unwrap(), q);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::TableRows(HtmlSemanticValue::Inapplicable)
                | HtmlSemanticFact::TableBodies(HtmlSemanticValue::Inapplicable)
                | HtmlSemanticFact::TableSectionRows(HtmlSemanticValue::Inapplicable)
                | HtmlSemanticFact::TableRowCells(HtmlSemanticValue::Inapplicable)
        ));
    }
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&foreign).unwrap(),
            HtmlSemanticQuery::TableRows
        )
        .results()[0]
            .fact(),
        &HtmlSemanticFact::TableRows(HtmlSemanticValue::Unsupported(
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
        0,
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
fn exact_one_short_work_output_actual_capacity_and_bytes_are_sticky_with_zero_scratch() {
    let (c, t, expected, _) = ordered();
    let cx = context(&c);
    let u = query(&c, &cx, &t, HtmlSemanticQuery::TableRows).usage();
    assert_eq!(u.peak_scratch_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
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
            let result = ss.query(&[request(&t, HtmlSemanticQuery::TableRows)]);
            if cap == exact {
                assert_eq!(ids(&result.unwrap()), expected);
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
fn repeated_queries_admit_cumulative_member_capacity_and_do_not_return_partial_batches() {
    let (c, t, _, _) = ordered();
    let cx = context(&c);
    let one = query(&c, &cx, &t, HtmlSemanticQuery::TableRows).usage();
    let requests = [
        request(&t, HtmlSemanticQuery::TableRows),
        request(&t, HtmlSemanticQuery::TableRows),
    ];
    let result = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&requests)
        .unwrap();
    assert_eq!(
        result.usage().nested_output_records(),
        2 * one.nested_output_records()
    );
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
    let failure = ss.query(&requests).unwrap_err();
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
fn cancellation_and_late_batch_limit_retain_original_qualification_and_sticky_stop() {
    let (c, t, _, _) = ordered();
    let cx = context(&c);
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::OutputRecords, 1),
        0,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss
        .query(&[
            request(&t, HtmlSemanticQuery::TableBodies),
            request(&t, HtmlSemanticQuery::TableRows),
        ])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 0, cancel);
    let failure = ss
        .query(&[request(&t, HtmlSemanticQuery::TableRows)])
        .unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}
#[test]
fn captures_contexts_host_revisions_and_limits_qualify_recomputation_and_foreign_rejection() {
    let (a, t, _, _) = ordered();
    let (b, u, _, _) = ordered();
    let ax = context(&a);
    let bx = context(&b);
    let ar = query(&a, &ax, &t, HtmlSemanticQuery::TableRows);
    let br = query(&b, &bx, &u, HtmlSemanticQuery::TableRows);
    assert_ne!(ar.capture(), br.capture());
    assert_ne!(ar.host_revision(), br.host_revision());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let another = context(&a);
    let cr = query(&a, &another, &t, HtmlSemanticQuery::TableRows);
    assert_ne!(ar.context(), cr.context());
    let limited = session(
        &a,
        &ax,
        resource_limits(HtmlSemanticResource::Work, ar.usage().work()),
        0,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&t, HtmlSemanticQuery::TableRows)])
    .unwrap();
    assert_ne!(ar.qualification(), limited.qualification());
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
        .query(&[request(&u, HtmlSemanticQuery::TableRows)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
}
