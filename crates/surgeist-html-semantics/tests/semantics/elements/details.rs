use crate::semantics::support::builder;
use crate::semantics::support::context;
use crate::semantics::support::session;
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties, HtmlDetailsNameGroup,
    HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUnsupported,
    HtmlSemanticValue,
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

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 1000000, 1000000, 100000)
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
fn summary(r: &HtmlSemanticReport) -> Option<&HtmlCaptureSubjectId> {
    match r.results()[0].fact() {
        HtmlSemanticFact::DetailsSummary(HtmlSemanticValue::Known(v)) => v.as_ref(),
        other => panic!("expected summary, got {other:?}"),
    }
}
fn predicate(r: &HtmlSemanticReport) -> bool {
    match r.results()[0].fact() {
        HtmlSemanticFact::SummaryForParentDetails(HtmlSemanticValue::Known(v)) => *v,
        other => panic!("expected predicate, got {other:?}"),
    }
}
fn group(r: &HtmlSemanticReport) -> &HtmlDetailsNameGroup {
    match r.results()[0].fact() {
        HtmlSemanticFact::DetailsNameGroup(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected group, got {other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::DetailsSummary(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::SummaryForParentDetails(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::DetailsNameGroup(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("expected missing, got {other:?}"),
    }
}
#[test]
fn first_direct_summary_after_text_and_other_children_wins_without_later_descendant_facts() {
    let (mut b, d) = builder();
    let p = html(
        &mut b,
        &d,
        "details",
        &[("open", "bad"), ("name", "long unconsulted")],
    );
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0xd800]))
        .unwrap();
    let div = html(&mut b, &d, "div", &[]);
    let wrapped = html(&mut b, &d, "summary", &[]);
    children(&mut b, &div, &[wrapped]);
    let first = html(&mut b, &d, "summary", &[]);
    let later = html(&mut b, &d, "summary", &[]);
    children(&mut b, &p, &[text, div, first.clone(), later.clone()]);
    let c = b.finish().unwrap();
    let pid = c.subject_id(&p).unwrap();
    let firstid = c.subject_id(&first).unwrap();
    assert_eq!(
        summary(&query(&c, &pid, HtmlSemanticQuery::DetailsSummary, 0)),
        Some(&firstid)
    );
    assert!(predicate(&query(
        &c,
        &firstid,
        HtmlSemanticQuery::SummaryForParentDetails,
        0
    )));
    assert!(!predicate(&query(
        &c,
        &c.subject_id(&later).unwrap(),
        HtmlSemanticQuery::SummaryForParentDetails,
        0
    )));
}
#[test]
fn wrapped_foreign_and_uppercase_summaries_do_not_replace_direct_html_summary() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "details", &[]);
    let svg = element(
        &mut b,
        &d,
        Namespace::svg(),
        "summary",
        OrderedAttributes::new([]).unwrap(),
    );
    let upper = html(&mut b, &d, "SUMMARY", &[]);
    let div = html(&mut b, &d, "div", &[]);
    let wrapped = html(&mut b, &d, "summary", &[]);
    children(&mut b, &div, std::slice::from_ref(&wrapped));
    children(&mut b, &p, &[svg, upper, div]);
    let c = b.finish().unwrap();
    assert_eq!(
        summary(&query(
            &c,
            &c.subject_id(&p).unwrap(),
            HtmlSemanticQuery::DetailsSummary,
            0
        )),
        None
    );
    assert!(!predicate(&query(
        &c,
        &c.subject_id(&wrapped).unwrap(),
        HtmlSemanticQuery::SummaryForParentDetails,
        0
    )));
}
#[test]
fn detached_complete_empty_details_has_no_captured_summary_and_no_fabricated_legend() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "details", &[]);
    children(&mut b, &p, &[]);
    b.establish_root(&p).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &c.subject_id(&p).unwrap(),
        HtmlSemanticQuery::DetailsSummary,
        0,
    );
    assert_eq!(summary(&r), None);
    assert_eq!(r.usage().nested_output_records(), 0);
    assert_eq!(r.usage().peak_scratch_records(), 0);
}
#[test]
fn unknown_details_child_order_and_summary_parent_are_distinct_missing_evidence() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "details", &[]);
    let s = html(&mut b, &d, "summary", &[]);
    let c = b.finish().unwrap();
    let pid = c.subject_id(&p).unwrap();
    let sid = c.subject_id(&s).unwrap();
    assert_eq!(
        missing(&query(&c, &pid, HtmlSemanticQuery::DetailsSummary, 0)),
        &HtmlSemanticMissing::OrderedChildren(pid)
    );
    assert_eq!(
        missing(&query(
            &c,
            &sid,
            HtmlSemanticQuery::SummaryForParentDetails,
            0
        )),
        &HtmlSemanticMissing::Parent(sid)
    );
}
#[test]
fn established_root_summary_and_non_details_parent_are_false_without_child_scans() {
    for parent in [
        None,
        Some((Namespace::html(), "div")),
        Some((Namespace::svg(), "details")),
    ] {
        let (mut b, d) = builder();
        let s = html(&mut b, &d, "summary", &[]);
        if let Some((ns, local)) = parent {
            let p = element(&mut b, &d, ns, local, OrderedAttributes::new([]).unwrap());
            children(&mut b, &p, std::slice::from_ref(&s));
        } else {
            b.establish_root(&s).unwrap();
        }
        let c = b.finish().unwrap();
        assert!(!predicate(&query(
            &c,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SummaryForParentDetails,
            0
        )));
    }
}
#[test]
fn known_details_parent_with_unknown_child_order_does_not_guess_summary_predicate() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "details", &[]);
    let s = html(&mut b, &d, "summary", &[]);
    b.establish_parent(&s, &p).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        missing(&query(
            &c,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SummaryForParentDetails,
            0
        )),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&p).unwrap())
    );
}
#[test]
fn absent_and_empty_name_are_distinct_raw_singletons_without_any_topology() {
    for attrs in [vec![], vec![("name", "")]] {
        let (mut b, d) = builder();
        let a = html(&mut b, &d, "details", &attrs);
        let c = b.finish().unwrap();
        let id = c.subject_id(&a).unwrap();
        let r = query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 0);
        let g = group(&r);
        assert_eq!(g.members(), std::slice::from_ref(&id));
        assert_eq!(g.len(), 1);
        assert!(!g.is_empty());
        assert_eq!(
            g.name_attribute().map(|a| a.value().len()),
            if attrs.is_empty() { None } else { Some(0) }
        );
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}
fn grouped() -> (HtmlCapture, HtmlCaptureSubjectId, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "details", &[("name", "x"), ("open", "invalid")]);
    let wrap = element(
        &mut b,
        &d,
        Namespace::svg(),
        "g",
        OrderedAttributes::new([]).unwrap(),
    );
    let nested = html(
        &mut b,
        &d,
        "details",
        &[("name", "x"), ("open", "also invalid")],
    );
    let different = html(&mut b, &d, "details", &[("name", "X")]);
    let last = html(&mut b, &d, "details", &[("name", "x")]);
    for h in [&nested, &different, &last] {
        children(&mut b, h, &[]);
    }
    children(&mut b, &wrap, std::slice::from_ref(&nested));
    children(&mut b, &root, &[wrap, different, last.clone()]);
    b.establish_root(&root).unwrap();
    let other = html(&mut b, &d, "details", &[("name", "x")]);
    children(&mut b, &other, &[]);
    b.establish_root(&other).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&root).unwrap();
    let expected = vec![
        id.clone(),
        c.subject_id(&nested).unwrap(),
        c.subject_id(&last).unwrap(),
    ];
    (c, id, expected)
}
#[test]
fn named_group_includes_root_nested_and_wrapped_members_in_tree_order_not_other_trees() {
    let (c, id, expected) = grouped();
    let r = query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1);
    assert_eq!(group(&r).members(), expected);
    assert_eq!(
        group(&r).name_attribute().unwrap().value().code_units(),
        &[120]
    );
    // Both nested same-name and multiple-open author restrictions are violated;
    // membership still observes all actual members, without certifying conformance.
}
#[test]
fn exact_utf16_name_equality_does_not_trim_fold_or_replace_surrogates() {
    for units in [
        vec![0x20],
        vec![0xa0],
        vec![0xd800, 0x78],
        vec![0x78, 0xdfff],
    ] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "div", &[]);
        let mut hs = Vec::new();
        for name in [units.clone(), units.clone(), vec![0x78], vec![0xfffd]] {
            let h = element(
                &mut b,
                &d,
                Namespace::html(),
                "details",
                OrderedAttributes::new([Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "name"),
                    DocumentString::from_utf16(name),
                )])
                .unwrap(),
            );
            children(&mut b, &h, &[]);
            hs.push(h);
        }
        children(&mut b, &root, &hs);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&hs[0]).unwrap();
        let r = query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 2);
        assert_eq!(group(&r).members(), &[id, c.subject_id(&hs[1]).unwrap()]);
        assert_eq!(
            group(&r).name_attribute().unwrap().value().code_units(),
            units
        );
    }
}
#[test]
fn uppercase_and_namespaced_name_attributes_are_unselected_and_do_not_trigger_raw_caps() {
    for name in [
        QualifiedName::new(Namespace::none(), None, "Name"),
        QualifiedName::new(Namespace::svg(), None, "name"),
    ] {
        let (mut b, d) = builder();
        let h = element(
            &mut b,
            &d,
            Namespace::html(),
            "details",
            OrderedAttributes::new([Attribute::new(name, "long unselected")]).unwrap(),
        );
        let c = b.finish().unwrap();
        let id = c.subject_id(&h).unwrap();
        let r = query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 0);
        assert!(group(&r).name_attribute().is_none());
        assert_eq!(group(&r).members(), &[id]);
    }
}
#[test]
fn named_group_requires_influencing_ancestry_and_complete_order_without_prefix() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "details", &[("name", "x")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    assert_eq!(
        missing(&query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1)),
        &HtmlSemanticMissing::Parent(id)
    );
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "details", &[("name", "x")]);
    let unknown = html(&mut b, &d, "div", &[]);
    children(&mut b, &a, &[]);
    children(&mut b, &root, &[a.clone(), unknown.clone()]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &c.subject_id(&a).unwrap(),
        HtmlSemanticQuery::DetailsNameGroup,
        1,
    );
    assert_eq!(
        missing(&r),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&unknown).unwrap())
    );
    assert!(r.usage().nested_output_records() > 0);
    assert_eq!(r.usage().scratch_records(), 0);
}
#[test]
fn every_consulted_nonempty_name_uses_raw_admission_including_later_failure() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "details", &[("name", "x")]);
    let other = html(&mut b, &d, "details", &[("name", "long")]);
    children(&mut b, &a, &[]);
    children(&mut b, &other, &[]);
    children(&mut b, &root, &[a.clone(), other]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&a).unwrap();
    let mut ss = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new());
    let f = ss
        .query(&[request(&id, HtmlSemanticQuery::DetailsNameGroup)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert!(f.usage().nested_output_records() > 0);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(
        group(&query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 4)).members(),
        &[id]
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
        1000000,
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
fn exact_and_one_short_work_capacity_bytes_scratch_and_batch_output_are_sticky() {
    let (c, id, expected) = grouped();
    let cx = context(&c);
    let u = query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1).usage();
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::NestedOutputRecords,
            u.nested_output_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut ss = session(
                &c,
                &cx,
                resource_limits(resource, cap),
                1,
                HtmlSemanticCancellation::new(),
            );
            let result = ss.query(&[request(&id, HtmlSemanticQuery::DetailsNameGroup)]);
            if cap == exact {
                assert_eq!(group(&result.unwrap()).members(), expected);
            } else {
                let f = result.unwrap_err();
                assert!(
                    matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{f:?}"
                );
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn repeated_queries_are_cumulative_and_late_batch_failures_never_return_a_prefix() {
    let (c, id, _) = grouped();
    let cx = context(&c);
    let one = query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1).usage();
    let reqs = [
        request(&id, HtmlSemanticQuery::DetailsNameGroup),
        request(&id, HtmlSemanticQuery::DetailsNameGroup),
    ];
    let r = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&reqs)
        .unwrap();
    assert_eq!(
        r.usage().nested_output_records(),
        2 * one.nested_output_records()
    );
    let mut ss = session(
        &c,
        &cx,
        resource_limits(
            HtmlSemanticResource::NestedOutputRecords,
            one.nested_output_records(),
        ),
        1,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&reqs).unwrap_err();
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
    let mut ss = session(
        &c,
        &cx,
        resource_limits(HtmlSemanticResource::OutputRecords, 1),
        1,
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[
            request(&id, HtmlSemanticQuery::DetailsSummary),
            request(&id, HtmlSemanticQuery::DetailsNameGroup),
        ])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn cancelled_queries_keep_the_original_qualified_stop() {
    let (c, id, _) = grouped();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 1, cancel);
    let f = ss
        .query(&[request(&id, HtmlSemanticQuery::DetailsNameGroup)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn wrong_roles_foreign_names_and_non_elements_preserve_applicability() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let upper = html(&mut b, &d, "DETAILS", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "details",
        OrderedAttributes::new([]).unwrap(),
    );
    let c = b.finish().unwrap();
    for h in [&div, &upper, &d] {
        let r = query(
            &c,
            &c.subject_id(h).unwrap(),
            HtmlSemanticQuery::DetailsNameGroup,
            0,
        );
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::DetailsNameGroup(HtmlSemanticValue::Inapplicable)
        ));
    }
    assert!(matches!(
        query(
            &c,
            &c.subject_id(&foreign).unwrap(),
            HtmlSemanticQuery::DetailsNameGroup,
            0
        )
        .results()[0]
            .fact(),
        HtmlSemanticFact::DetailsNameGroup(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    ));
}
#[test]
fn exact_capture_context_host_and_configuration_qualify_conservative_results() {
    let (a, id, _) = grouped();
    let (b, other, _) = grouped();
    let ax = context(&a);
    let bx = context(&b);
    let ar = query(&a, &id, HtmlSemanticQuery::DetailsNameGroup, 1);
    let br = query(&b, &other, HtmlSemanticQuery::DetailsNameGroup, 1);
    assert_ne!(ar.capture(), br.capture());
    assert_ne!(ar.host_revision(), br.host_revision());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let another = context(&a);
    let cr = session(&a, &another, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&id, HtmlSemanticQuery::DetailsNameGroup)])
        .unwrap();
    assert_ne!(ar.context(), cr.context());
    let limited = session(
        &a,
        &ax,
        resource_limits(HtmlSemanticResource::Work, ar.usage().work()),
        1,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id, HtmlSemanticQuery::DetailsNameGroup)])
    .unwrap();
    assert_ne!(ar.qualification(), limited.qualification());
    assert!(
        HtmlSemanticSession::new(
            &a,
            &bx,
            HtmlValueInterpreter::new(1),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    let f = session(&a, &ax, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&other, HtmlSemanticQuery::DetailsNameGroup)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
}
#[test]
fn groups_and_summary_predicates_do_not_cross_shadow_or_template_host_associations() {
    for shadow in [true, false] {
        let (mut b, d) = builder();
        let root = html(&mut b, &d, "details", &[("name", "x")]);
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let f = if shadow {
            b.add_shadow_root(
                &d,
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
        let hidden = html(&mut b, &d, "details", &[("name", "x")]);
        let hidden_summary = html(&mut b, &d, "summary", &[]);
        let ordinary = html(&mut b, &d, "details", &[("name", "x")]);
        children(&mut b, &hidden, &[]);
        children(&mut b, &hidden_summary, &[]);
        children(&mut b, &ordinary, &[]);
        children(&mut b, &f, &[hidden.clone(), hidden_summary.clone()]);
        children(&mut b, &host, std::slice::from_ref(&ordinary));
        children(&mut b, &root, &[host]);
        b.establish_root(&root).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&root).unwrap();
        assert_eq!(
            group(&query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1)).members(),
            &[id, c.subject_id(&ordinary).unwrap()]
        );
        assert_eq!(
            group(&query(
                &c,
                &c.subject_id(&hidden).unwrap(),
                HtmlSemanticQuery::DetailsNameGroup,
                1
            ))
            .members(),
            &[c.subject_id(&hidden).unwrap()]
        );
        assert!(!predicate(&query(
            &c,
            &c.subject_id(&hidden_summary).unwrap(),
            HtmlSemanticQuery::SummaryForParentDetails,
            0
        )));
        assert_eq!(
            summary(&query(
                &c,
                &c.subject_id(&root).unwrap(),
                HtmlSemanticQuery::DetailsSummary,
                0
            )),
            None
        );
    }
}
#[test]
fn foreign_and_uppercase_details_do_not_join_even_with_equal_selected_names() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[]);
    let a = html(&mut b, &d, "details", &[("name", "x")]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "details",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "name"),
            "long irrelevant",
        )])
        .unwrap(),
    );
    let upper = html(&mut b, &d, "DETAILS", &[("name", "x")]);
    for h in [&a, &foreign, &upper] {
        children(&mut b, h, &[]);
    }
    children(&mut b, &root, &[a.clone(), foreign, upper]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    assert_eq!(
        group(&query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1)).members(),
        &[id]
    );
}

#[test]
fn exact_and_one_short_target_name_admission_differ_before_topology_materialization() {
    let (c, id, expected) = grouped();
    let cx = context(&c);
    assert_eq!(
        group(&query(&c, &id, HtmlSemanticQuery::DetailsNameGroup, 1)).members(),
        expected
    );
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss
        .query(&[request(&id, HtmlSemanticQuery::DetailsNameGroup)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(f.usage().nested_output_records(), 0);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
