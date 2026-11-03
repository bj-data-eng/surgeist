use crate::semantics::support::{
    Charge, ExpectedUsage, assert_exact_and_one_short_caps, builder, context, result_record_bytes,
    scalar_limits as resource_limits, scalar_semantic_limits as limits,
    session_without_raw_values as session, validated_work,
};
use surgeist_html::{
    construction::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment},
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
    vocabulary::KnownHtmlElement,
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticDependencies, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    prefix: Option<&str>,
    local: DocumentString,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, prefix.map(DocumentString::from), local),
        OrderedAttributes::new([]).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), None, local.into())
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
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    session(c, cx, limits(), HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlSemanticValue<Option<usize>> {
    match r.results()[0].fact() {
        HtmlSemanticFact::TableRowIndex(v)
        | HtmlSemanticFact::TableSectionRowIndex(v)
        | HtmlSemanticFact::TableCellIndex(v) => v,
        other => panic!("unexpected {other:?}"),
    }
}
fn check(
    c: &HtmlCapture,
    h: &HtmlCaptureBuilderSubject,
    q: HtmlSemanticQuery,
    expected: Option<usize>,
) {
    let id = c.subject_id(h).unwrap();
    assert_eq!(
        value(&query(c, &context(c), &id, q)),
        &HtmlSemanticValue::Known(expected)
    );
}
fn ordered() -> (HtmlCapture, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let foot = html(&mut b, &d, "tfoot");
    let f0 = html(&mut b, &d, "tr");
    let f1 = html(&mut b, &d, "tr");
    children(&mut b, &foot, &[f0.clone(), f1.clone()]);
    let direct = html(&mut b, &d, "tr");
    let body = html(&mut b, &d, "tbody");
    let b0 = html(&mut b, &d, "tr");
    let b1 = html(&mut b, &d, "tr");
    children(&mut b, &body, &[b0.clone(), b1.clone()]);
    let head0 = html(&mut b, &d, "thead");
    let h0 = html(&mut b, &d, "tr");
    children(&mut b, &head0, std::slice::from_ref(&h0));
    let head1 = html(&mut b, &d, "thead");
    let h1 = html(&mut b, &d, "tr");
    children(&mut b, &head1, std::slice::from_ref(&h1));
    children(&mut b, &t, &[foot, direct.clone(), body, head0, head1]);
    b.establish_root(&t).unwrap();
    let c = b.finish().unwrap();
    let ids = [h0, h1, direct, b0, b1, f0, f1]
        .iter()
        .map(|h| c.subject_id(h).unwrap())
        .collect();
    (c, ids)
}

#[test]
fn row_index_counts_headers_then_direct_and_body_rows_then_footers() {
    // HTML #dom-table-rows: ordinary footer-first placement does not set its ordinal.
    let (c, ids) = ordered();
    let cx = context(&c);
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(
            value(&query(&c, &cx, id, HtmlSemanticQuery::TableRowIndex)),
            &HtmlSemanticValue::Known(Some(i))
        );
    }
}
#[test]
fn section_row_index_uses_own_section_but_direct_table_rows_use_reordered_table_rows() {
    let (c, ids) = ordered();
    let cx = context(&c);
    for (id, i) in ids.iter().zip([0, 0, 2, 0, 1, 0, 1]) {
        assert_eq!(
            value(&query(&c, &cx, id, HtmlSemanticQuery::TableSectionRowIndex)),
            &HtmlSemanticValue::Known(Some(i))
        );
    }
}
#[test]
fn detached_section_row_index_does_not_require_table_grandparent() {
    for tag in ["thead", "tbody", "tfoot"] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, tag);
        let r = html(&mut b, &d, "tr");
        children(&mut b, &p, std::slice::from_ref(&r));
        b.establish_root(&p).unwrap();
        let c = b.finish().unwrap();
        check(&c, &r, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
        check(&c, &r, HtmlSemanticQuery::TableRowIndex, None);
    }
}
#[test]
fn section_unknown_parent_affects_row_index_but_not_section_index() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "tbody");
    let r = html(&mut b, &d, "tr");
    children(&mut b, &p, std::slice::from_ref(&r));
    let c = b.finish().unwrap();
    check(&c, &r, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
    let r = query(
        &c,
        &context(&c),
        &c.subject_id(&r).unwrap(),
        HtmlSemanticQuery::TableRowIndex,
    );
    assert_eq!(
        value(&r),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(c.subject_id(&p).unwrap()))
    );
}
#[test]
fn non_table_grandparent_and_wrapped_rows_never_use_nearest_table_ancestor() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let w = html(&mut b, &d, "div");
    let p = html(&mut b, &d, "tbody");
    let r = html(&mut b, &d, "tr");
    let wrapped = html(&mut b, &d, "tr");
    children(&mut b, &p, std::slice::from_ref(&r));
    children(&mut b, &w, &[p, wrapped.clone()]);
    children(&mut b, &t, &[w]);
    let c = b.finish().unwrap();
    check(&c, &r, HtmlSemanticQuery::TableRowIndex, None);
    check(&c, &r, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
    check(&c, &wrapped, HtmlSemanticQuery::TableRowIndex, None);
    check(&c, &wrapped, HtmlSemanticQuery::TableSectionRowIndex, None);
}
#[test]
fn cells_count_only_direct_html_td_and_th_without_descending_into_wrappers() {
    let (mut b, d) = builder();
    let row = html(&mut b, &d, "tr");
    let c0 = html(&mut b, &d, "th");
    let foreign = element(&mut b, &d, Namespace::svg(), None, "td".into());
    let w = html(&mut b, &d, "div");
    let nested = html(&mut b, &d, "td");
    let c1 = html(&mut b, &d, "td");
    children(&mut b, &w, std::slice::from_ref(&nested));
    children(&mut b, &row, &[c0.clone(), foreign, w, c1.clone()]);
    let c = b.finish().unwrap();
    check(&c, &c0, HtmlSemanticQuery::TableCellIndex, Some(0));
    check(&c, &c1, HtmlSemanticQuery::TableCellIndex, Some(1));
    check(&c, &nested, HtmlSemanticQuery::TableCellIndex, None);
}
#[test]
fn root_subjects_are_known_none_and_unknown_parent_is_missing() {
    for (tag, q) in [
        ("tr", HtmlSemanticQuery::TableRowIndex),
        ("tr", HtmlSemanticQuery::TableSectionRowIndex),
        ("td", HtmlSemanticQuery::TableCellIndex),
        ("th", HtmlSemanticQuery::TableCellIndex),
    ] {
        let (mut b, d) = builder();
        let known = html(&mut b, &d, tag);
        let unknown = html(&mut b, &d, tag);
        b.establish_root(&known).unwrap();
        let c = b.finish().unwrap();
        check(&c, &known, q, None);
        let id = c.subject_id(&unknown).unwrap();
        assert_eq!(
            value(&query(&c, &context(&c), &id, q)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))
        );
    }
}
#[test]
fn known_non_element_parent_returns_none_without_its_child_order() {
    let (mut b, d) = builder();
    let r = html(&mut b, &d, "tr");
    children(&mut b, &d, std::slice::from_ref(&r));
    let c = b.finish().unwrap();
    check(&c, &r, HtmlSemanticQuery::TableRowIndex, None);
    check(&c, &r, HtmlSemanticQuery::TableSectionRowIndex, None);
}
#[test]
fn first_header_target_ignores_later_unavailable_header_body_and_footer_children() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let h = html(&mut b, &d, "thead");
    let r = html(&mut b, &d, "tr");
    children(&mut b, &h, std::slice::from_ref(&r));
    let h2 = html(&mut b, &d, "thead");
    let body = html(&mut b, &d, "tbody");
    let foot = html(&mut b, &d, "tfoot");
    children(&mut b, &t, &[foot, body, h, h2]);
    let c = b.finish().unwrap();
    check(&c, &r, HtmlSemanticQuery::TableRowIndex, Some(0));
}
#[test]
fn direct_target_requires_headers_even_when_they_follow_it_in_ordinary_order() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let r = html(&mut b, &d, "tr");
    let h = html(&mut b, &d, "thead");
    children(&mut b, &t, &[r.clone(), h.clone()]);
    let c = b.finish().unwrap();
    for q in [
        HtmlSemanticQuery::TableRowIndex,
        HtmlSemanticQuery::TableSectionRowIndex,
    ] {
        assert_eq!(
            value(&query(&c, &context(&c), &c.subject_id(&r).unwrap(), q)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
                c.subject_id(&h).unwrap()
            ))
        );
    }
}
#[test]
fn footer_target_requires_preceding_body_phase_but_ignores_later_footer_gap() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let foot = html(&mut b, &d, "tfoot");
    let r = html(&mut b, &d, "tr");
    children(&mut b, &foot, std::slice::from_ref(&r));
    let body = html(&mut b, &d, "tbody");
    let later = html(&mut b, &d, "tfoot");
    children(&mut b, &t, &[foot, body.clone(), later]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&query(
            &c,
            &context(&c),
            &c.subject_id(&r).unwrap(),
            HtmlSemanticQuery::TableRowIndex
        )),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&body).unwrap()
        ))
    );
    check(&c, &r, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
}
#[test]
fn known_body_counts_and_complete_first_footer_prove_index_before_later_footer_gap() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let body = html(&mut b, &d, "tbody");
    let b0 = html(&mut b, &d, "tr");
    children(&mut b, &body, &[b0]);
    let foot = html(&mut b, &d, "tfoot");
    let r = html(&mut b, &d, "tr");
    children(&mut b, &foot, std::slice::from_ref(&r));
    let later = html(&mut b, &d, "tfoot");
    children(&mut b, &t, &[foot, body, later]);
    let c = b.finish().unwrap();
    check(&c, &r, HtmlSemanticQuery::TableRowIndex, Some(1));
}
#[test]
fn unavailable_qualifying_container_order_is_missing_not_none() {
    for (parent, tag, q) in [
        ("table", "tr", HtmlSemanticQuery::TableRowIndex),
        ("tbody", "tr", HtmlSemanticQuery::TableSectionRowIndex),
        ("tr", "td", HtmlSemanticQuery::TableCellIndex),
    ] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, parent);
        let r = html(&mut b, &d, tag);
        b.establish_parent(&r, &p).unwrap();
        let c = b.finish().unwrap();
        assert_eq!(
            value(&query(&c, &context(&c), &c.subject_id(&r).unwrap(), q)),
            &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
                c.subject_id(&p).unwrap()
            ))
        );
    }
}
#[test]
fn namespaces_case_prefix_and_isolated_utf16_are_exact_for_subject_and_parent() {
    let (mut b, d) = builder();
    let p = element(&mut b, &d, Namespace::html(), Some("x"), "tr".into());
    let good = element(&mut b, &d, Namespace::html(), Some("y"), "td".into());
    let upper = html(&mut b, &d, "TD");
    let odd = element(
        &mut b,
        &d,
        Namespace::html(),
        None,
        DocumentString::from_utf16(vec![0x74, 0x64, 0xd800]),
    );
    let foreign = element(&mut b, &d, Namespace::svg(), None, "td".into());
    children(
        &mut b,
        &p,
        &[upper.clone(), odd.clone(), foreign.clone(), good.clone()],
    );
    let c = b.finish().unwrap();
    check(&c, &good, HtmlSemanticQuery::TableCellIndex, Some(0));
    for h in [upper, odd] {
        assert_eq!(
            value(&query(
                &c,
                &context(&c),
                &c.subject_id(&h).unwrap(),
                HtmlSemanticQuery::TableCellIndex
            )),
            &HtmlSemanticValue::Inapplicable
        );
    }
    assert_eq!(
        value(&query(
            &c,
            &context(&c),
            &c.subject_id(&foreign).unwrap(),
            HtmlSemanticQuery::TableCellIndex
        )),
        &HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    );
}
#[test]
fn foreign_or_uppercase_parent_does_not_supply_an_html_collection() {
    for (ns, tag) in [(Namespace::svg(), "tr"), (Namespace::html(), "TR")] {
        let (mut b, d) = builder();
        let p = element(&mut b, &d, ns, None, tag.into());
        let td = html(&mut b, &d, "td");
        children(&mut b, &p, std::slice::from_ref(&td));
        let c = b.finish().unwrap();
        check(&c, &td, HtmlSemanticQuery::TableCellIndex, None);
    }
}
#[test]
fn wrong_roles_and_non_element_queries_are_inapplicable() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "table");
    let r = html(&mut b, &d, "tr");
    let td = html(&mut b, &d, "td");
    let c = b.finish().unwrap();
    for (h, q) in [
        (p, HtmlSemanticQuery::TableRowIndex),
        (r, HtmlSemanticQuery::TableCellIndex),
        (td, HtmlSemanticQuery::TableSectionRowIndex),
        (d, HtmlSemanticQuery::TableRowIndex),
    ] {
        assert_eq!(
            value(&query(&c, &context(&c), &c.subject_id(&h).unwrap(), q)),
            &HtmlSemanticValue::Inapplicable
        );
    }
}
#[test]
fn ignored_attributes_spans_text_and_member_descendants_need_no_raw_or_nested_budget() {
    let (mut b, d) = builder();
    let row = html(&mut b, &d, "tr");
    let td = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "td"),
            OrderedAttributes::new([
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "rowspan"),
                    "65534",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "headers"),
                    DocumentString::from_utf16(vec![0xd800; 4096]),
                ),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0xdfff; 4096]))
        .unwrap();
    children(&mut b, &row, &[text, td.clone()]);
    let c = b.finish().unwrap();
    let report = query(
        &c,
        &context(&c),
        &c.subject_id(&td).unwrap(),
        HtmlSemanticQuery::TableCellIndex,
    );
    assert_eq!(value(&report), &HtmlSemanticValue::Known(Some(0)));
    assert_eq!(report.usage().nested_output_records(), 0);
    assert_eq!(report.usage().peak_scratch_records(), 0);
}
/// `K(l)` of the semantic query work reference (docs/reference/html-operations.md): the recognition charge of an HTML
/// element whose local name has `l` units.
fn known(l: usize) -> usize {
    (l + 1) * KnownHtmlElement::ALL.len() + "http://www.w3.org/1999/xhtml".len()
}
/// `D`: a matched HTML namespace test.
const D: usize = 1 + "http://www.w3.org/1999/xhtml".len();

#[test]
fn exact_one_short_work_output_and_physical_batch_bytes_stop_stickily_without_interim_snapshot() {
    let (c, ids) = ordered();
    let cx = context(&c);
    // Documented table walks for `ordered()`: a table whose children are
    // tfoot[f0, f1], a direct tr, one tbody[b0, b1] and two theads of one tr each.
    // `next(l)` is one `table_next` over an element of `l` units (1, a lookup,
    // `K(l)`); a retained or counted row costs 2.
    let next = |l: usize| 2 + known(l);
    // children_walk: five element children and the final exhausted `table_next`,
    // each element also charging 1: tfoot, tr, tbody, thead, thead.
    let children_walk = 4 * (next(5) + 1) + (next(2) + 1) + 1;
    // A section's row list: `table_children` (2), each tr child's next, 1 and 2,
    // and a final exhausted next (1).
    let section = |rows: usize| 2 + rows * (next(2) + 1 + 2) + 1;
    // Rows collection stopping at f1 (the last row): head pass, body pass, foot
    // pass; the foot pass returns at the target without a final next.
    let head_pass = 1 + 2 + children_walk + 2 * section(1);
    let body_pass = 1 + 2 + children_walk + section(2) + 2;
    let foot_pass = 1 + 2 + (next(5) + 1) + 2 + 2 * (next(2) + 1 + 2);
    let rows_walk = head_pass + body_pass + foot_pass;
    // Query prefix (reservation, lookup, D), the subject's lookup, D and K(2),
    // the parent step, lookup and K(5) of the tfoot, then the step to the table.
    let row_index =
        1 + 1 + D + 1 + D + known(2) + (1 + 1 + known(5)) + (1 + 1 + known(5)) + rows_walk + 1;
    // The section-row collection walks only the tfoot's two rows.
    let section_row_index =
        1 + 1 + D + 1 + D + known(2) + (1 + 1 + known(5)) + (2 + 2 * (next(2) + 1 + 2)) + 1;
    for (q, work, expected) in [
        (HtmlSemanticQuery::TableRowIndex, row_index, 6),
        (
            HtmlSemanticQuery::TableSectionRowIndex,
            section_row_index,
            1,
        ),
    ] {
        // An ordinal is a scalar answer: one result record and no nested or
        // scratch storage.
        let report = assert_exact_and_one_short_caps(
            &format!("{q:?}"),
            &c,
            &cx,
            0,
            limits(),
            &[request(&ids[6], q)],
            ExpectedUsage {
                work: Charge::Exactly(work),
                output_records: Charge::Exactly(1),
                allocated_bytes: Charge::Exactly(result_record_bytes()),
                ..ExpectedUsage::none()
            },
        );
        // The last footer row is the final row of the footer section and of
        // the reordered table collection (headers, direct and body rows,
        // footers).
        assert_eq!(value(&report), &HtmlSemanticValue::Known(Some(expected)));
    }
}
#[test]
fn duplicate_queries_repeat_actual_work_and_late_batch_failure_exposes_no_prefix() {
    let (c, ids) = ordered();
    let cx = context(&c);
    let q = HtmlSemanticQuery::TableRowIndex;
    let mut ss = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let first = ss.query(&[request(&ids[0], q)]).unwrap();
    let u = first.usage();
    let second = ss.query(&[request(&ids[0], q)]).unwrap();
    assert_eq!(second.usage().work(), 2 * u.work());
    assert_eq!(second.usage().allocated_bytes(), 2 * u.allocated_bytes());
    assert_eq!(value(&first), &HtmlSemanticValue::Known(Some(0)));
    let mut limited = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::OutputRecords, 1),
        HtmlSemanticCancellation::new(),
    );
    let f = limited
        .query(&[request(&ids[0], q), request(&ids[6], q)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
    assert_eq!(limited.query(&[]).unwrap_err(), f);
    assert_eq!(limited.usage().scratch_records(), 0);
}
#[test]
fn cancellation_and_foreign_capture_failures_keep_original_qualification_and_stop() {
    let (c, ids) = ordered();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), cancel);
    let f = ss
        .query(&[request(&ids[0], HtmlSemanticQuery::TableRowIndex)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    let (_, foreign) = ordered();
    let mut ss = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let f = ss
        .query(&[request(&foreign[0], HtmlSemanticQuery::TableRowIndex)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn fresh_capture_context_and_limits_qualify_conservative_recomputation_without_invalidating_old_report()
 {
    let (c, ids) = ordered();
    let cx = context(&c);
    let q = HtmlSemanticQuery::TableRowIndex;
    let old = query(&c, &cx, &ids[2], q);
    let other = context(&c);
    let fresh = query(&c, &other, &ids[2], q);
    assert_ne!(old.context(), fresh.context());
    let limited = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, 1_000_000),
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&ids[2], q)])
    .unwrap();
    assert_ne!(old.qualification(), limited.qualification());
    let (c2, ids2) = ordered();
    let r2 = query(&c2, &context(&c2), &ids2[2], q);
    assert_ne!(old.capture(), r2.capture());
    assert_ne!(old.host_revision(), r2.host_revision());
    assert_eq!(
        old.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(value(&old), &HtmlSemanticValue::Known(Some(2)));
}

#[test]
fn template_and_shadow_hosts_never_substitute_for_ordinary_parent_or_grandparent() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let t = html(&mut b, &d, "table");
        let host = html(&mut b, &d, if shadow { "div" } else { "template" });
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
        let section = html(&mut b, &d, "tbody");
        let r = html(&mut b, &d, "tr");
        children(&mut b, &section, std::slice::from_ref(&r));
        children(&mut b, &f, &[section]);
        children(&mut b, &t, &[host]);
        let c = b.finish().unwrap();
        check(&c, &r, HtmlSemanticQuery::TableRowIndex, None);
        check(&c, &r, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
    }
}
#[test]
fn direct_section_and_cell_prefixes_ignore_unknown_wrapped_descendants_and_later_members() {
    for (pname, targetname, q) in [
        ("tbody", "tr", HtmlSemanticQuery::TableSectionRowIndex),
        ("tr", "td", HtmlSemanticQuery::TableCellIndex),
    ] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, pname);
        let gap = html(&mut b, &d, "div");
        let target = html(&mut b, &d, targetname);
        let later = html(&mut b, &d, targetname);
        children(&mut b, &p, &[gap, target.clone(), later]);
        let c = b.finish().unwrap();
        check(&c, &target, q, Some(0));
    }
}
#[test]
fn row_index_ignores_wrapped_and_nested_table_subtrees_even_when_unavailable() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let wrapper = html(&mut b, &d, "div");
    let nested = html(&mut b, &d, "table");
    let r = html(&mut b, &d, "tr");
    children(&mut b, &t, &[wrapper, nested, r.clone()]);
    let c = b.finish().unwrap();
    check(&c, &r, HtmlSemanticQuery::TableRowIndex, Some(0));
}
#[test]
fn cell_exact_one_short_budgets_and_duplicate_batch_keep_zero_nested_and_scratch_usage() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "tr");
    let a = html(&mut b, &d, "th");
    let target = html(&mut b, &d, "td");
    children(&mut b, &p, &[a, target.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&target).unwrap();
    let q = HtmlSemanticQuery::TableCellIndex;
    // Documented: prefix (reservation, lookup, D), the cell's lookup, D and K(2),
    // the parent step, lookup and K(2) of the tr, then the row's direct cell walk:
    // `table_children` (2) and, per cell, next, 1 and 2, ending at the target.
    let next = 2 + known(2);
    let work = 1 + 1 + D + 1 + D + known(2) + (1 + 1 + known(2)) + 2 + 2 * (next + 1 + 2) + 1;
    let report = assert_exact_and_one_short_caps(
        "cell index",
        &c,
        &cx,
        0,
        limits(),
        &[request(&id, q)],
        ExpectedUsage {
            work: Charge::Exactly(work),
            output_records: Charge::Exactly(1),
            allocated_bytes: Charge::Exactly(result_record_bytes()),
            ..ExpectedUsage::none()
        },
    );
    assert_eq!(value(&report), &HtmlSemanticValue::Known(Some(1)));
    let pair = session(&c, &cx, limits(), HtmlSemanticCancellation::new())
        .query(&[request(&id, q), request(&id, q)])
        .unwrap();
    assert_eq!(pair.results().len(), 2);
    assert_eq!(pair.usage().nested_output_records(), 0);
    assert_eq!(pair.usage().peak_scratch_records(), 0);
    assert_eq!(pair.usage().output_records(), 2);
}
#[test]
fn later_index_work_failure_is_atomic_and_retains_earlier_physical_batch_allocation() {
    let (c, ids) = ordered();
    let cx = context(&c);
    let q = HtmlSemanticQuery::TableRowIndex;
    let early = query(&c, &cx, &ids[0], q);
    let late = query(&c, &cx, &ids[6], q);
    assert!(late.usage().work() > early.usage().work());
    // The late request's own work, proved least-admitting, admits the shared
    // reservation and the early request but not the late request after it.
    let late_work = validated_work(
        "late row index",
        &c,
        &cx,
        0,
        limits(),
        &[request(&ids[6], q)],
    );
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::Work, late_work),
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[request(&ids[0], q), request(&ids[6], q)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert!(f.usage().allocated_bytes() > 0);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(value(&early), &HtmlSemanticValue::Known(Some(0)));
}
#[test]
fn changed_source_child_order_changes_ordinal_in_a_fresh_qualified_capture() {
    for reverse in [false, true] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, "tr");
        let a = html(&mut b, &d, "td");
        let target = html(&mut b, &d, "th");
        let cs = if reverse {
            [target.clone(), a]
        } else {
            [a, target.clone()]
        };
        children(&mut b, &p, &cs);
        let c = b.finish().unwrap();
        check(
            &c,
            &target,
            HtmlSemanticQuery::TableCellIndex,
            Some(usize::from(!reverse)),
        );
    }
}
#[test]
fn each_operation_retains_its_exact_pinned_defining_clause() {
    for (q, anchor) in [
        (HtmlSemanticQuery::TableRowIndex, "dom-tr-rowindex"),
        (
            HtmlSemanticQuery::TableSectionRowIndex,
            "dom-tr-sectionrowindex",
        ),
        (HtmlSemanticQuery::TableCellIndex, "dom-tdth-cellindex"),
    ] {
        assert_eq!(
            q.source_clause(),
            format!(
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#{anchor}"
            )
        );
    }
}

#[test]
fn earlier_header_gap_influences_a_later_header_target() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let gap = html(&mut b, &d, "thead");
    let h = html(&mut b, &d, "thead");
    let row = html(&mut b, &d, "tr");
    children(&mut b, &h, std::slice::from_ref(&row));
    children(&mut b, &t, &[gap.clone(), h]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&query(
            &c,
            &context(&c),
            &c.subject_id(&row).unwrap(),
            HtmlSemanticQuery::TableRowIndex
        )),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&gap).unwrap()
        ))
    );
    check(&c, &row, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
}

#[test]
fn direct_body_phase_target_ignores_later_body_and_footer_gaps() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let row = html(&mut b, &d, "tr");
    let later = html(&mut b, &d, "tbody");
    let foot = html(&mut b, &d, "tfoot");
    children(&mut b, &t, &[foot, row.clone(), later]);
    let c = b.finish().unwrap();
    check(&c, &row, HtmlSemanticQuery::TableRowIndex, Some(0));
    check(&c, &row, HtmlSemanticQuery::TableSectionRowIndex, Some(0));
}

#[test]
fn text_comment_pi_and_spanning_cells_never_create_row_ordinals() {
    let (mut b, d) = builder();
    let t = html(&mut b, &d, "table");
    let first = html(&mut b, &d, "tr");
    let td = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "td"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "rowspan"),
                "65534",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    children(&mut b, &first, &[td]);
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0xd800; 4096]))
        .unwrap();
    let comment = b.add_comment(&d, "unread".into()).unwrap();
    let pi = b
        .add_processing_instruction(&d, "tr".into(), "unread".into())
        .unwrap();
    let target = html(&mut b, &d, "tr");
    children(&mut b, &t, &[first, text, comment, pi, target.clone()]);
    let c = b.finish().unwrap();
    check(&c, &target, HtmlSemanticQuery::TableRowIndex, Some(1));
    check(
        &c,
        &target,
        HtmlSemanticQuery::TableSectionRowIndex,
        Some(1),
    );
}
